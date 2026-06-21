use rand::rngs::OsRng;
use x25519_dalek::PublicKey as XPublicKey;

use super::types::EphemeralPublicKey;

/// Generate an ephemeral X25519 keypair (RAM-only, consumed after DH).
pub fn generate() -> (x25519_dalek::EphemeralSecret, EphemeralPublicKey) {
    let secret = x25519_dalek::EphemeralSecret::random_from_rng(&mut OsRng);
    let public = EphemeralPublicKey(XPublicKey::from(&secret).to_bytes());
    (secret, public)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generates_unique_keys() {
        let (_, pk1) = generate();
        let (_, pk2) = generate();
        assert_ne!(pk1, pk2);
    }

    #[test]
    fn test_public_key_has_correct_length() {
        let (_, pk) = generate();
        assert_eq!(pk.0.len(), 32);
    }
}
