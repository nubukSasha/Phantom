package com.phantom.bridge

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.PublicKey
import javax.crypto.Cipher
import javax.crypto.KeyAgreement
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import com.phantom.FfiKeystoreOps
import com.phantom.PhantomApp

class AndroidKeystoreCallback : FfiKeystoreOps {

    private val keyStore by lazy {
        KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    }

    private val prefs by lazy {
        PhantomApp.instance.getSharedPreferences("phantom_keystore", Context.MODE_PRIVATE)
    }

    private val WRAP_KEY_ALIAS = "phantom_fallback_wrap"

    // ── Fallback helpers ────────────────────────────────

    private fun storeFallbackKey(alias: String, privateKey: PrivateKey, publicKey: PublicKey) {
        val wrapped = wrapKey(WRAP_KEY_ALIAS, privateKey.encoded)
        prefs.edit()
            .putString(alias, Base64.encodeToString(wrapped, Base64.NO_WRAP))
            .putString("$alias.pub", Base64.encodeToString(extractRawPublicKey(publicKey), Base64.NO_WRAP))
            .apply()
    }

    private fun loadFallbackPrivate(alias: String, algorithm: String): PrivateKey {
        val b64 = prefs.getString(alias, null) ?: throw IllegalStateException("No fallback key for $alias")
        val wrapped = Base64.decode(b64, Base64.NO_WRAP)
        val raw = unwrapKey(WRAP_KEY_ALIAS, wrapped)
        return java.security.KeyFactory.getInstance(algorithm)
            .generatePrivate(java.security.spec.PKCS8EncodedKeySpec(raw))
    }

    private fun loadFallbackPublic(alias: String): ByteArray {
        val b64 = prefs.getString("$alias.pub", null) ?: throw IllegalStateException("No fallback public key for $alias")
        return Base64.decode(b64, Base64.NO_WRAP)
    }

    private fun deleteFallbackKey(alias: String) {
        prefs.edit().remove(alias).remove("$alias.pub").apply()
    }

    private fun hasHardwareKey(alias: String): Boolean {
        return try {
            keyStore.containsAlias(alias)
        } catch (_: Exception) { false }
    }

    // ── Ed25519 ──────────────────────────────────────────

    override fun ed25519Generate(alias: String): ByteArray {
        try {
            if (hasHardwareKey(alias)) return loadEd25519Public(alias)
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
        } catch (_: java.security.InvalidAlgorithmParameterException) {
            // Hardware doesn't support ed25519 — software fallback
            val kpg = KeyPairGenerator.getInstance("Ed25519")
            val kp = kpg.generateKeyPair()
            storeFallbackKey(alias, kp.private, kp.public)
            return extractRawPublicKey(kp.public)
        }
    }

    override fun ed25519Public(alias: String): ByteArray {
        if (hasHardwareKey(alias)) return loadEd25519Public(alias)
        return loadFallbackPublic(alias)
    }

    override fun ed25519Sign(alias: String, msg: ByteArray): ByteArray {
        try {
            if (hasHardwareKey(alias)) {
                val privateKey = loadEd25519Private(alias)
                val sig = java.security.Signature.getInstance("Ed25519")
                sig.initSign(privateKey)
                sig.update(msg)
                return sig.sign()
            }
        } catch (_: Exception) { }
        val privateKey = loadFallbackPrivate(alias, "Ed25519")
        val sig = java.security.Signature.getInstance("Ed25519")
        sig.initSign(privateKey)
        sig.update(msg)
        return sig.sign()
    }

    override fun ed25519Delete(alias: String) {
        try {
            if (hasHardwareKey(alias)) keyStore.deleteEntry(alias)
        } catch (_: Exception) { }
        deleteFallbackKey(alias)
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
        try {
            if (hasHardwareKey(alias)) return loadX25519Public(alias)
            val spec = KeyGenParameterSpec.Builder(
                alias, KeyProperties.PURPOSE_AGREE_KEY
            )
                .setAlgorithmParameterSpec(
                    java.security.spec.ECGenParameterSpec("X25519")
                )
                .build()
            val kpg = KeyPairGenerator.getInstance("XDH", "AndroidKeyStore")
            kpg.initialize(spec)
            val kp = kpg.generateKeyPair()
            return extractRawPublicKey(kp.public)
        } catch (_: java.security.InvalidAlgorithmParameterException) {
            val kpg = KeyPairGenerator.getInstance("X25519")
            val kp = kpg.generateKeyPair()
            storeFallbackKey(alias, kp.private, kp.public)
            return extractRawPublicKey(kp.public)
        }
    }

    override fun x25519Public(alias: String): ByteArray {
        if (hasHardwareKey(alias)) return loadX25519Public(alias)
        return loadFallbackPublic(alias)
    }

    override fun x25519Agree(alias: String, peerPublic: ByteArray): ByteArray {
        try {
            if (hasHardwareKey(alias)) {
                return x25519AgreeHardware(alias, peerPublic)
            }
        } catch (_: Exception) { }
        val privateKey = loadFallbackPrivate(alias, "X25519")
        return x25519AgreeCompute(privateKey, peerPublic)
    }

    override fun x25519Delete(alias: String) {
        try {
            if (hasHardwareKey(alias)) keyStore.deleteEntry(alias)
        } catch (_: Exception) { }
        deleteFallbackKey(alias)
    }

    private fun loadX25519Public(alias: String): ByteArray {
        val entry = keyStore.getEntry(alias, null) as KeyStore.PrivateKeyEntry
        return extractRawPublicKey(entry.certificate.publicKey)
    }

    private fun loadX25519Private(alias: String): PrivateKey {
        val entry = keyStore.getEntry(alias, null) as KeyStore.PrivateKeyEntry
        return entry.privateKey
    }

    private fun x25519AgreeHardware(alias: String, peerPublic: ByteArray): ByteArray {
        val privateKey = loadX25519Private(alias)
        return x25519AgreeCompute(privateKey, peerPublic)
    }

    private fun x25519AgreeCompute(privateKey: PrivateKey, peerPublic: ByteArray): ByteArray {
        val spki = byteArrayOf(0x30, 0x2A, 0x30, 0x05, 0x06, 0x03, 0x2B, 0x65, 0x6E, 0x03, 0x21, 0x00) + peerPublic
        val peerKeySpec = java.security.spec.X509EncodedKeySpec(spki)
        val kf = java.security.KeyFactory.getInstance("XDH")
        val peerPubKey = kf.generatePublic(peerKeySpec)
        val ka = KeyAgreement.getInstance("XDH")
        ka.init(privateKey)
        ka.doPhase(peerPubKey, true)
        return ka.generateSecret()
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
            alias, KeyProperties.PURPOSE_WRAP_KEY or 64
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
        idx += readLength()

        require(encoded[idx++].toInt() == 0x03) { "Expected BIT STRING" }
        val bitStringLen = readLength()
        idx++
        return encoded.copyOfRange(idx, idx + bitStringLen - 1)
    }
}
