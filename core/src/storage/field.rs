use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Key, XChaCha20Poly1305,
};

use super::error::StorageError;
use super::master_key::MasterKey;

/// Encrypt a plaintext value for storage.
///
/// Returns `(nonce, ciphertext_with_tag)` — both must be stored
/// alongside each other to enable decryption later.
pub fn encrypt_field(
    master_key: &MasterKey,
    plaintext: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), StorageError> {
    let key = Key::from_slice(master_key.as_bytes());
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| StorageError::AuthFailed)?;
    Ok((nonce.to_vec(), ciphertext))
}

/// Decrypt a field value that was encrypted with `encrypt_field`.
pub fn decrypt_field(
    master_key: &MasterKey,
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, StorageError> {
    if nonce.len() != 24 {
        return Err(StorageError::InvalidData(format!(
            "invalid nonce length: expected 24, got {}",
            nonce.len()
        )));
    }
    let key = Key::from_slice(master_key.as_bytes());
    let cipher = XChaCha20Poly1305::new(key);
    use chacha20poly1305::aead::Nonce as AeadNonce;
    let nonce_arr = AeadNonce::from_slice(nonce);
    cipher
        .decrypt(nonce_arr, ciphertext)
        .map_err(|_| StorageError::AuthFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::mock::MockKeystore;

    fn test_master_key() -> MasterKey {
        let ks = MockKeystore::new();
        let mk = MasterKey::generate();
        let wrapped = mk.wrap(&ks, "test").unwrap();
        MasterKey::unwrap(&ks, "test", &wrapped).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_field_roundtrip() {
        let mk = test_master_key();
        let plaintext = b"sensitive contact data";

        let (nonce, ciphertext) = encrypt_field(&mk, plaintext).unwrap();
        let decrypted = decrypt_field(&mk, &nonce, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let mk = test_master_key();
        let (nonce, ciphertext) = encrypt_field(&mk, b"").unwrap();
        let decrypted = decrypt_field(&mk, &nonce, &ciphertext).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let mk1 = test_master_key();
        let mk2 = MasterKey::generate(); // different key

        let (nonce, ciphertext) = encrypt_field(&mk1, b"secret").unwrap();
        let result = decrypt_field(&mk2, &nonce, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let mk = test_master_key();
        let (nonce, mut ciphertext) = encrypt_field(&mk, b"tamper").unwrap();
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0xff;
        }
        let result = decrypt_field(&mk, &nonce, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_invalid_nonce_length() {
        let mk = test_master_key();
        let result = decrypt_field(&mk, &[0u8; 12], b"ciphertext");
        assert!(matches!(result, Err(StorageError::InvalidData(_))));
    }
}
