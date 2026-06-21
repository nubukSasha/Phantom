use super::cipher;
use super::ephemeral;
use super::kdf;
use super::static_key;
use super::types::{
    EncryptedMessage, EphemeralPublicKey, IdentityPublicKey, Nonce,
    StaticPublicKey, PROTOCOL_VERSION, TAG_SIZE,
};
use crate::error::CryptoError;
use crate::keystore::KeystoreOps;

/// Encrypt a plaintext message for a recipient.
///
/// * `keystore` — KeystoreOps implementation (used for ECDH via static key)
/// * `static_alias` — alias of the sender's X25519 static key in the keystore
/// * `sender_identity_pk` — sender's Ed25519 identity public key
/// * `sender_static_pk` — sender's X25519 static public key
/// * `recipient_static_pk` — recipient's X25519 static public key
/// * `plaintext` — message bytes to encrypt
pub fn encrypt_message(
    _keystore: &dyn KeystoreOps,
    _static_alias: &str,
    sender_identity_pk: &IdentityPublicKey,
    sender_static_pk: &StaticPublicKey,
    recipient_identity_pk: &IdentityPublicKey,
    recipient_static_pk: &StaticPublicKey,
    plaintext: &[u8],
) -> Result<EncryptedMessage, CryptoError> {
    // 1. Generate ephemeral keypair
    let (eph_secret, eph_pk) = ephemeral::generate();

    // 2. ECDH: shared = X25519(eph_sk, recipient_static_pk)
    let shared = static_key::agree_with_ephemeral(eph_secret, recipient_static_pk);

    // 3. Build session context and derive session key
    let ctx = kdf::build_session_context(
        *sender_identity_pk,
        *recipient_identity_pk,
        *sender_static_pk,
        *recipient_static_pk,
        eph_pk,
    );
    let session_key = kdf::derive_session_key(&shared, &ctx);

    // 4. Encrypt with ChaCha20-Poly1305
    let sealed = cipher::encrypt(&session_key, plaintext);

    // 5. Extract ciphertext and tag from SealedBox
    let (ct, tag) = cipher::split_ciphertext_tag(&sealed.ciphertext)
        .unwrap_or_else(|_| {
            // Should never happen: encrypt always produces ciphertext + tag
            (&[] as &[u8], [0u8; TAG_SIZE])
        });

    // 6. Build wire message
    Ok(EncryptedMessage {
        version: PROTOCOL_VERSION,
        sender_identity_pk: sender_identity_pk.0,
        sender_static_pk: sender_static_pk.0,
        eph_pk: eph_pk.0,
        nonce: sealed.nonce.0,
        ciphertext: ct.to_vec(),
        tag,
    })
}

/// Decrypt an `EncryptedMessage` received from a peer.
///
/// * `keystore` — KeystoreOps implementation
/// * `static_alias` — alias of the recipient's X25519 static key in the keystore
/// * `recipient_identity_pk` — recipient's own Ed25519 identity public key
/// * `recipient_static_pk` — recipient's own X25519 static public key
/// * `msg` — the encrypted message from wire
pub fn decrypt_message(
    keystore: &dyn KeystoreOps,
    static_alias: &str,
    recipient_identity_pk: &IdentityPublicKey,
    recipient_static_pk: &StaticPublicKey,
    msg: &EncryptedMessage,
) -> Result<Vec<u8>, CryptoError> {
    // 1. Version check
    if msg.version != PROTOCOL_VERSION {
        return Err(CryptoError::ProtocolVersionMismatch {
            expected: PROTOCOL_VERSION,
            got: msg.version,
        });
    }

    // 2. Reconstruct public keys from wire
    let sender_identity_pk = IdentityPublicKey(msg.sender_identity_pk);
    let sender_static_pk = StaticPublicKey(msg.sender_static_pk);
    let eph_pk = EphemeralPublicKey(msg.eph_pk);

    // 3. ECDH: shared = X25519(recipient_static_sk, sender_eph_pk)
    let shared = static_key::agree_with_static(keystore, static_alias, &eph_pk)
        .map_err(|e| CryptoError::InvalidFormat(e))?;

    // 4. Build session context and derive session key
    let ctx = kdf::build_session_context(
        sender_identity_pk,
        *recipient_identity_pk,
        sender_static_pk,
        *recipient_static_pk,
        eph_pk,
    );
    let session_key = kdf::derive_session_key(&shared, &ctx);

    // 5. Rebuild full ciphertext with tag, then decrypt
    let nonce = Nonce(msg.nonce);
    let mut full_ct = msg.ciphertext.clone();
    full_ct.extend_from_slice(&msg.tag);

    let sealed = super::types::SealedBox {
        nonce,
        ciphertext: full_ct,
    };

    cipher::decrypt(&session_key, &sealed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::mock::MockKeystore;
    use crate::crypto::identity;

    fn setup_alice_bob(
    ) -> (MockKeystore, IdentityPublicKey, StaticPublicKey, IdentityPublicKey, StaticPublicKey)
    {
        let ks = MockKeystore::new();

        let alice_identity =
            identity::generate(&ks, "alice_identity").unwrap();
        let alice_static =
            static_key::generate(&ks, "alice_static").unwrap();

        let bob_identity =
            identity::generate(&ks, "bob_identity").unwrap();
        let bob_static =
            static_key::generate(&ks, "bob_static").unwrap();

        (ks, alice_identity, alice_static, bob_identity, bob_static)
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (ks, alice_id, alice_st, bob_id, bob_st) = setup_alice_bob();

        let plaintext = b"hello bob, this is alice";

        let msg = encrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &bob_id,
            &bob_st,
            plaintext,
        )
        .unwrap();

        let decrypted = decrypt_message(
            &ks,
            "bob_static",
            &bob_id,
            &bob_st,
            &msg,
        )
        .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty_message() {
        let (ks, alice_id, alice_st, bob_id, bob_st) = setup_alice_bob();

        let msg = encrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &bob_id,
            &bob_st,
            b"",
        )
        .unwrap();

        let decrypted = decrypt_message(
            &ks,
            "bob_static",
            &bob_id,
            &bob_st,
            &msg,
        )
        .unwrap();

        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let (ks, alice_id, alice_st, bob_id, bob_st) = setup_alice_bob();

        let msg = encrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &bob_id,
            &bob_st,
            b"secret",
        )
        .unwrap();

        // Try to decrypt as alice (wrong static key)
        let result = decrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &msg,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_version_mismatch() {
        let (ks, alice_id, alice_st, bob_id, bob_st) = setup_alice_bob();

        let mut msg = encrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &bob_id,
            &bob_st,
            b"test",
        )
        .unwrap();

        msg.version = 99;

        let result = decrypt_message(
            &ks,
            "bob_static",
            &bob_id,
            &bob_st,
            &msg,
        );

        assert!(matches!(
            result,
            Err(CryptoError::ProtocolVersionMismatch { .. })
        ));
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let (ks, alice_id, alice_st, bob_id, bob_st) = setup_alice_bob();

        let mut msg = encrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &bob_id,
            &bob_st,
            b"tamper me",
        )
        .unwrap();

        if !msg.ciphertext.is_empty() {
            msg.ciphertext[0] ^= 0xff;
        }

        let result = decrypt_message(
            &ks,
            "bob_static",
            &bob_id,
            &bob_st,
            &msg,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_wire_format_encode_decode() {
        let (ks, alice_id, alice_st, bob_id, bob_st) = setup_alice_bob();

        let msg = encrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &bob_id,
            &bob_st,
            b"wire test",
        )
        .unwrap();

        let encoded = msg.encode().unwrap();
        let decoded = EncryptedMessage::decode(&encoded).unwrap();

        assert_eq!(decoded.version, msg.version);
        assert_eq!(decoded.ciphertext, msg.ciphertext);
        assert_eq!(decoded.tag, msg.tag);

        // Verify we can decrypt the decoded message
        let decrypted = decrypt_message(
            &ks,
            "bob_static",
            &bob_id,
            &bob_st,
            &decoded,
        )
        .unwrap();

        assert_eq!(decrypted, b"wire test");
    }

    #[test]
    fn test_different_ephemeral_per_message() {
        let (ks, alice_id, alice_st, bob_id, bob_st) = setup_alice_bob();

        let msg1 = encrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &bob_id,
            &bob_st,
            b"msg1",
        )
        .unwrap();

        let msg2 = encrypt_message(
            &ks,
            "alice_static",
            &alice_id,
            &alice_st,
            &bob_id,
            &bob_st,
            b"msg2",
        )
        .unwrap();

        // Different ephemeral keys → different ciphertext and eph_pk
        assert_ne!(msg1.eph_pk, msg2.eph_pk);
        assert_ne!(msg1.nonce, msg2.nonce);
    }
}
