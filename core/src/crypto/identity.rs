use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use super::types::IdentityPublicKey;
use crate::error::CryptoError;
use crate::keystore::KeystoreOps;

pub fn generate(
    keystore: &dyn KeystoreOps,
    alias: &str,
) -> Result<IdentityPublicKey, String> {
    keystore.ed25519_generate(alias).map(IdentityPublicKey)
}

pub fn load_public(
    keystore: &dyn KeystoreOps,
    alias: &str,
) -> Result<IdentityPublicKey, String> {
    keystore.ed25519_public(alias).map(IdentityPublicKey)
}

pub fn sign(
    keystore: &dyn KeystoreOps,
    alias: &str,
    msg: &[u8],
) -> Result<[u8; 64], String> {
    keystore.ed25519_sign(alias, msg)
}

pub fn verify(
    pk: &IdentityPublicKey,
    msg: &[u8],
    sig: &[u8; 64],
) -> Result<(), CryptoError> {
    let verifying_key =
        VerifyingKey::from_bytes(&pk.0).map_err(|_| {
            CryptoError::InvalidFormat("invalid Ed25519 public key".into())
        })?;

    let signature = Signature::from_slice(sig.as_slice()).map_err(|_| {
        CryptoError::InvalidFormat("invalid Ed25519 signature".into())
    })?;

    verifying_key
        .verify(msg, &signature)
        .map_err(|_| CryptoError::SignatureVerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::mock::MockKeystore;

    #[test]
    fn test_generate_and_load_public() {
        let ks = MockKeystore::new();
        let pk = generate(&ks, "test_identity").unwrap();
        let loaded = load_public(&ks, "test_identity").unwrap();
        assert_eq!(pk, loaded);
    }

    #[test]
    fn test_sign_and_verify() {
        let ks = MockKeystore::new();
        let pk = generate(&ks, "test_identity").unwrap();
        let msg = b"hello phantom";
        let sig = sign(&ks, "test_identity", msg).unwrap();
        assert!(verify(&pk, msg, &sig).is_ok());
    }

    #[test]
    fn test_verify_wrong_message() {
        let ks = MockKeystore::new();
        let pk = generate(&ks, "test_identity").unwrap();
        let sig = sign(&ks, "test_identity", b"correct").unwrap();
        assert!(verify(&pk, b"wrong", &sig).is_err());
    }

    #[test]
    fn test_verify_invalid_signature() {
        let ks = MockKeystore::new();
        let pk = generate(&ks, "test_identity").unwrap();
        let bad_sig = [0u8; 64];
        assert!(verify(&pk, b"msg", &bad_sig).is_err());
    }

    #[test]
    fn test_key_not_found() {
        let ks = MockKeystore::new();
        assert!(load_public(&ks, "nonexistent").is_err());
        assert!(sign(&ks, "nonexistent", b"msg").is_err());
    }

    #[test]
    fn test_generate_unique_keys() {
        let ks = MockKeystore::new();
        let pk1 = generate(&ks, "alice").unwrap();
        let pk2 = generate(&ks, "bob").unwrap();
        assert_ne!(pk1, pk2);
    }
}
