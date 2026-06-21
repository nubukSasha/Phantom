use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce as AeadNonce,
};

use super::types::{Nonce, SealedBox, SessionKey};
use crate::error::CryptoError;

pub fn encrypt(key: &SessionKey, plaintext: &[u8]) -> SealedBox {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
    let nonce_bytes = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let nonce = Nonce(nonce_bytes.into());

    let ciphertext = cipher
        .encrypt(&AeadNonce::from(nonce.0), plaintext)
        .expect("ChaCha20-Poly1305 encryption cannot fail with correct key length");

    SealedBox { nonce, ciphertext }
}

pub fn decrypt(
    key: &SessionKey,
    sealed: &SealedBox,
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
    let nonce = AeadNonce::from(sealed.nonce.0);

    cipher
        .decrypt(&nonce, sealed.ciphertext.as_ref())
        .map_err(|_| CryptoError::AuthFailed)
}

/// Encrypt with an explicit nonce (used when the nonce must be known
/// before encryption, e.g. when it's part of the HKDF context).
pub fn encrypt_with_nonce(
    key: &SessionKey,
    nonce: &Nonce,
    plaintext: &[u8],
) -> SealedBox {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
    let aead_nonce = AeadNonce::from(nonce.0);

    let ciphertext = cipher
        .encrypt(&aead_nonce, plaintext)
        .expect("ChaCha20-Poly1305 encryption cannot fail");

    SealedBox {
        nonce: *nonce,
        ciphertext,
    }
}

/// Decrypt with an explicit nonce (the inverse of `encrypt_with_nonce`).
pub fn decrypt_with_nonce(
    key: &SessionKey,
    nonce: &Nonce,
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
    let aead_nonce = AeadNonce::from(nonce.0);

    cipher
        .decrypt(&aead_nonce, ciphertext)
        .map_err(|_| CryptoError::AuthFailed)
}

/// Extract the concatenated tag from (ciphertext || tag).
pub fn split_ciphertext_tag(data: &[u8]) -> Result<(&[u8], [u8; 16]), CryptoError>
{
    if data.len() < 16 {
        return Err(CryptoError::InvalidFormat(
            "ciphertext too short to contain tag".into(),
        ));
    }
    let split_point = data.len() - 16;
    let ciphertext = &data[..split_point];
    let mut tag = [0u8; 16];
    tag.copy_from_slice(&data[split_point..]);
    Ok((ciphertext, tag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::types::{Nonce, SessionKey};

    fn test_key() -> SessionKey {
        SessionKey::new([
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
            0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        ])
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"hello phantom";

        let sealed = encrypt(&key, plaintext);
        let decrypted = decrypt(&key, &sealed).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let key = test_key();
        let plaintext = b"";

        let sealed = encrypt(&key, plaintext);
        let decrypted = decrypt(&key, &sealed).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = test_key();
        let key2 = SessionKey::new([0xff; 32]);
        let plaintext = b"secret message";

        let sealed = encrypt(&key1, plaintext);
        let result = decrypt(&key2, &sealed);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let key = test_key();
        let plaintext = b"tamper test";

        let mut sealed = encrypt(&key, plaintext);
        if !sealed.ciphertext.is_empty() {
            sealed.ciphertext[0] ^= 0xff;
        }
        let result = decrypt(&key, &sealed);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_with_nonce() {
        let key = test_key();
        let nonce = Nonce([0x42; 12]);
        let plaintext = b"deterministic nonce";

        let sealed = encrypt_with_nonce(&key, &nonce, plaintext);
        let decrypted = decrypt_with_nonce(&key, &nonce, &sealed.ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let key = test_key();
        let plaintext = b"same plaintext";

        let sealed1 = encrypt(&key, plaintext);
        let sealed2 = encrypt(&key, plaintext);

        assert_ne!(sealed1.ciphertext, sealed2.ciphertext);
    }

    #[test]
    fn test_split_ciphertext_tag() {
        let data = [0u8; 32];
        let (ct, tag) = split_ciphertext_tag(&data).unwrap();
        assert_eq!(ct.len(), 16);
        assert_eq!(tag, [0u8; 16]);
    }

    #[test]
    fn test_split_ciphertext_tag_too_short() {
        let data = [0u8; 15];
        assert!(split_ciphertext_tag(&data).is_err());
    }
}
