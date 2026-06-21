use x25519_dalek::PublicKey as XPublicKey;

use super::types::{
    EphemeralPublicKey, SharedSecret, StaticPublicKey,
};
use crate::keystore::KeystoreOps;

pub fn generate(
    keystore: &dyn KeystoreOps,
    alias: &str,
) -> Result<StaticPublicKey, String> {
    keystore.x25519_generate(alias).map(StaticPublicKey)
}

pub fn load_public(
    keystore: &dyn KeystoreOps,
    alias: &str,
) -> Result<StaticPublicKey, String> {
    keystore.x25519_public(alias).map(StaticPublicKey)
}

pub fn delete(keystore: &dyn KeystoreOps, alias: &str) -> Result<(), String> {
    keystore.x25519_delete(alias)
}

/// Receiver side: static secret (in Keystore) × sender's ephemeral public key.
pub fn agree_with_static(
    keystore: &dyn KeystoreOps,
    alias: &str,
    peer_eph_pk: &EphemeralPublicKey,
) -> Result<SharedSecret, String> {
    keystore
        .x25519_agree(alias, &peer_eph_pk.0)
        .map(SharedSecret::new)
}

/// Sender side: ephemeral secret (in RAM) × recipient's static public key.
pub fn agree_with_ephemeral(
    eph_secret: x25519_dalek::EphemeralSecret,
    peer_static_pk: &StaticPublicKey,
) -> SharedSecret {
    let peer_public = XPublicKey::from(peer_static_pk.0);
    let shared = eph_secret.diffie_hellman(&peer_public);
    SharedSecret::new(shared.to_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::mock::MockKeystore;
    use rand::rngs::OsRng;

    #[test]
    fn test_generate_and_load() {
        let ks = MockKeystore::new();
        let pk = generate(&ks, "alice_static").unwrap();
        let loaded = load_public(&ks, "alice_static").unwrap();
        assert_eq!(pk, loaded);
    }

    #[test]
    fn test_agree_sender_receiver() {
        let ks = MockKeystore::new();

        // Receiver generates a static keypair
        let receiver_pk = generate(&ks, "bob_static").unwrap();

        // Sender generates ephemeral keypair
        let eph_secret =
            x25519_dalek::EphemeralSecret::random_from_rng(&mut OsRng);
        let eph_public =
            EphemeralPublicKey(XPublicKey::from(&eph_secret).to_bytes());

        // Sender computes shared secret
        let sender_shared =
            agree_with_ephemeral(eph_secret, &receiver_pk);

        // Receiver computes shared secret
        let receiver_shared =
            agree_with_static(&ks, "bob_static", &eph_public).unwrap();

        assert_eq!(sender_shared.0, receiver_shared.0);
    }

    #[test]
    fn test_agree_different_keys_different_secret() {
        let ks = MockKeystore::new();

        let alice_pk = generate(&ks, "alice_static").unwrap();
        let bob_pk = generate(&ks, "bob_static").unwrap();

        let eph1 =
            x25519_dalek::EphemeralSecret::random_from_rng(&mut OsRng);
        let eph2 =
            x25519_dalek::EphemeralSecret::random_from_rng(&mut OsRng);

        let shared1 = agree_with_ephemeral(eph1, &alice_pk);
        let shared2 = agree_with_ephemeral(eph2, &bob_pk);

        assert_ne!(shared1.0, shared2.0);
    }

    #[test]
    fn test_delete_and_missing() {
        let ks = MockKeystore::new();
        generate(&ks, "temp").unwrap();
        delete(&ks, "temp").unwrap();
        assert!(load_public(&ks, "temp").is_err());
    }
}
