use hkdf::Hkdf;
use sha2::Sha256;

use super::types::{
    EphemeralPublicKey, IdentityPublicKey, SessionContext, SessionKey,
    SharedSecret, StaticPublicKey,
};

const INFO_PREFIX: &[u8] = b"phantom-v1";

pub fn derive_session_key(
    shared: &SharedSecret,
    ctx: &SessionContext,
) -> SessionKey {
    let mut info = Vec::with_capacity(INFO_PREFIX.len() + 32 * 5);

    info.extend_from_slice(INFO_PREFIX);
    info.extend_from_slice(&ctx.sender_identity_pk.0);
    info.extend_from_slice(&ctx.recipient_identity_pk.0);
    info.extend_from_slice(&ctx.sender_static_pk.0);
    info.extend_from_slice(&ctx.recipient_static_pk.0);
    info.extend_from_slice(&ctx.eph_pk.0);

    let salt = [0u8; 32];
    let hk = Hkdf::<Sha256>::new(Some(&salt), shared.as_bytes());

    let mut okm = [0u8; 32];
    hk.expand(&info, &mut okm)
        .expect("HKDF expand with 32-byte output cannot fail");

    SessionKey::new(okm)
}

pub fn build_session_context(
    sender_identity_pk: IdentityPublicKey,
    recipient_identity_pk: IdentityPublicKey,
    sender_static_pk: StaticPublicKey,
    recipient_static_pk: StaticPublicKey,
    eph_pk: EphemeralPublicKey,
) -> SessionContext {
    SessionContext {
        sender_identity_pk,
        recipient_identity_pk,
        sender_static_pk,
        recipient_static_pk,
        eph_pk,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::types::{
        EphemeralPublicKey, IdentityPublicKey, StaticPublicKey,
    };

    fn make_dummy_context() -> SessionContext {
        SessionContext {
            sender_identity_pk: IdentityPublicKey([1u8; 32]),
            recipient_identity_pk: IdentityPublicKey([2u8; 32]),
            sender_static_pk: StaticPublicKey([3u8; 32]),
            recipient_static_pk: StaticPublicKey([4u8; 32]),
            eph_pk: EphemeralPublicKey([5u8; 32]),
        }
    }

    #[test]
    fn test_derive_session_key_is_deterministic() {
        let shared = SharedSecret::new([0xab; 32]);
        let ctx = make_dummy_context();

        let key1 = derive_session_key(&shared, &ctx);
        let key2 = derive_session_key(&shared, &ctx);

        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_shared_produces_different_key() {
        let shared_a = SharedSecret::new([0xaa; 32]);
        let shared_b = SharedSecret::new([0xbb; 32]);
        let ctx = make_dummy_context();

        let key_a = derive_session_key(&shared_a, &ctx);
        let key_b = derive_session_key(&shared_b, &ctx);

        assert_ne!(key_a.as_bytes(), key_b.as_bytes());
    }

    #[test]
    fn test_different_context_produces_different_key() {
        let shared = SharedSecret::new([0xab; 32]);
        let ctx1 = make_dummy_context();
        let mut ctx2 = make_dummy_context();
        ctx2.eph_pk = EphemeralPublicKey([0xff; 32]);

        let key1 = derive_session_key(&shared, &ctx1);
        let key2 = derive_session_key(&shared, &ctx2);

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_output_length() {
        let shared = SharedSecret::new([0xab; 32]);
        let ctx = make_dummy_context();
        let key = derive_session_key(&shared, &ctx);
        assert_eq!(key.as_bytes().len(), 32);
    }
}
