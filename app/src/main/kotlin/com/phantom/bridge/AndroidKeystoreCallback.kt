package com.phantom.bridge

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.PublicKey
import javax.crypto.Cipher
import javax.crypto.KeyAgreement
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

class AndroidKeystoreCallback : FfiKeystoreOps {

    private val keyStore by lazy {
        KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    }

    // ── Ed25519 ──────────────────────────────────────────

    override fun ed25519Generate(alias: String): ByteArray {
        if (keyStore.containsAlias(alias)) {
            return loadEd25519Public(alias)
        }
        val spec = KeyGenParameterSpec.Builder(
            alias, KeyProperties.PURPOSE_SIGN or KeyProperties.PURPOSE_VERIFY
        )
            .setAlgorithmParameterSpec(
                java.security.spec.ECGenParameterSpec("ed25519")
            )
            .build()
        val kpg = KeyPairGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_EC, "AndroidKeyStore"
        )
        kpg.initialize(spec)
        val kp = kpg.generateKeyPair()
        return extractRawPublicKey(kp.public)
    }

    override fun ed25519Public(alias: String): ByteArray {
        return loadEd25519Public(alias)
    }

    override fun ed25519Sign(alias: String, msg: ByteArray): ByteArray {
        val privateKey = loadEd25519Private(alias)
        val sig = java.security.Signature.getInstance("Ed25519")
        sig.initSign(privateKey)
        sig.update(msg)
        return sig.sign()
    }

    override fun ed25519Delete(alias: String) {
        keyStore.deleteEntry(alias)
    }

    private fun loadEd25519Public(alias: String): ByteArray {
        val entry = keyStore.getEntry(alias, null) as KeyStore.PrivateKeyEntry
        return extractRawPublicKey(entry.certificate.publicKey)
    }

    private fun loadEd25519Private(alias: String): PrivateKey {
        val entry = keyStore.getEntry(alias, null) as KeyStore.PrivateKeyEntry
        return entry.privateKey
    }

    // ── X25519 ──────────────────────────────────────────

    override fun x25519Generate(alias: String): ByteArray {
        if (keyStore.containsAlias(alias)) {
            return loadX25519Public(alias)
        }
        val spec = KeyGenParameterSpec.Builder(
            alias, KeyProperties.PURPOSE_AGREE_KEY
        )
            .setAlgorithmParameterSpec(
                java.security.spec.ECGenParameterSpec("X25519")
            )
            .build()
        val kpg = KeyPairGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_XDH, "AndroidKeyStore"
        )
        kpg.initialize(spec)
        val kp = kpg.generateKeyPair()
        return extractRawPublicKey(kp.public)
    }

    override fun x25519Public(alias: String): ByteArray {
        return loadX25519Public(alias)
    }

    override fun x25519Agree(alias: String, peerPublic: ByteArray): ByteArray {
        val privateKey = loadX25519Private(alias)
        // Wrap raw 32-byte public key in X25519 SPKI DER for KeyAgreement
        val spki = byteArrayOf(0x30, 0x2A, 0x30, 0x05, 0x06, 0x03, 0x2B, 0x65, 0x6E, 0x03, 0x21, 0x00) + peerPublic
        val peerKeySpec = java.security.spec.X509EncodedKeySpec(spki)
        val kf = java.security.KeyFactory.getInstance("XDH")
        val peerPubKey = kf.generatePublic(peerKeySpec)
        val ka = KeyAgreement.getInstance("XDH")
        ka.init(privateKey)
        ka.doPhase(peerPubKey, true)
        return ka.generateSecret()
    }

    override fun x25519Delete(alias: String) {
        keyStore.deleteEntry(alias)
    }

    private fun loadX25519Public(alias: String): ByteArray {
        val entry = keyStore.getEntry(alias, null) as KeyStore.PrivateKeyEntry
        return extractRawPublicKey(entry.certificate.publicKey)
    }

    private fun loadX25519Private(alias: String): PrivateKey {
        val entry = keyStore.getEntry(alias, null) as KeyStore.PrivateKeyEntry
        return entry.privateKey
    }

    // ── AES wrap/unwrap ─────────────────────────────────

    override fun wrapKey(alias: String, key: ByteArray): ByteArray {
        val wrappingKey = getOrCreateWrappingKey(alias)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.WRAP_MODE, wrappingKey)
        val secretKey = javax.crypto.spec.SecretKeySpec(key, "AES")
        return cipher.wrap(secretKey)
    }

    override fun unwrapKey(alias: String, wrapped: ByteArray): ByteArray {
        val wrappingKey = getOrCreateWrappingKey(alias)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.UNWRAP_MODE, wrappingKey)
        val secretKey = cipher.unwrap(wrapped, "AES", Cipher.SECRET_KEY)
        return secretKey.encoded
    }

    private fun getOrCreateWrappingKey(alias: String): SecretKey {
        if (keyStore.containsAlias(alias)) {
            return keyStore.getKey(alias, null) as SecretKey
        }
        val spec = KeyGenParameterSpec.Builder(
            alias, KeyProperties.PURPOSE_WRAP_KEY or KeyProperties.PURPOSE_UNWRAP_KEY
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .build()
        val kg = KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore"
        )
        kg.init(spec)
        return kg.generateKey()
    }

    // ── DER/SPKI helper ──────────────────────────────────

    private fun extractRawPublicKey(publicKey: PublicKey): ByteArray {
        val encoded = publicKey.encoded
        var idx = 0

        fun readLength(): Int {
            val first = encoded[idx++].toInt() and 0xFF
            if (first < 0x80) return first
            val numBytes = first and 0x7F
            var len = 0
            repeat(numBytes) { len = (len shl 8) or (encoded[idx++].toInt() and 0xFF) }
            return len
        }

        require(encoded[idx++].toInt() == 0x30) { "Expected SEQUENCE" }
        readLength()

        require(encoded[idx++].toInt() == 0x30) { "Expected algorithm SEQUENCE" }
        idx += readLength() // skip algorithm identifier contents

        require(encoded[idx++].toInt() == 0x03) { "Expected BIT STRING" }
        val bitStringLen = readLength()
        idx++ // skip unused bits byte
        return encoded.copyOfRange(idx, idx + bitStringLen - 1)
    }
}
