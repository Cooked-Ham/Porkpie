//! Property-based fuzz tests for porkpie-crypto.

use porkpie_crypto::{decrypt_item, derive_key, encrypt_item, generate_nonce};
use proptest::prelude::*;

proptest! {
    /// Encryption followed by decryption with the same key and AAD must
    /// always return the original plaintext.
    #[test]
    fn encryption_roundtrip(
        plaintext in any::<Vec<u8>>(),
        key in any::<[u8; 32]>(),
        aad in any::<Vec<u8>>(),
    ) {
        let encrypted = encrypt_item(&plaintext, &key, &aad).unwrap();
        let decrypted: Vec<u8> = decrypt_item(&encrypted, &key, &aad).unwrap();
        prop_assert_eq!(plaintext, decrypted);
    }

    /// Decrypting with a wrong key must always fail (no false positives).
    #[test]
    fn wrong_key_fails(
        plaintext in any::<Vec<u8>>(),
        key in any::<[u8; 32]>(),
        wrong_key in any::<[u8; 32]>(),
        aad in any::<Vec<u8>>(),
    ) {
        prop_assume!(key != wrong_key);
        let encrypted = encrypt_item(&plaintext, &key, &aad).unwrap();
        let result: Result<Vec<u8>, _> = decrypt_item(&encrypted, &wrong_key, &aad);
        prop_assert!(result.is_err());
    }

    /// Decrypting with a wrong AAD must always fail (no false positives).
    #[test]
    fn wrong_aad_fails(
        plaintext in any::<Vec<u8>>(),
        key in any::<[u8; 32]>(),
        aad in any::<Vec<u8>>(),
        wrong_aad in any::<Vec<u8>>(),
    ) {
        prop_assume!(aad != wrong_aad);
        let encrypted = encrypt_item(&plaintext, &key, &aad).unwrap();
        let result: Result<Vec<u8>, _> = decrypt_item(&encrypted, &key, &wrong_aad);
        prop_assert!(result.is_err());
    }

    /// Nonce must be unique across multiple calls.
    #[test]
    fn nonce_uniqueness(
        _seed in any::<u64>(),
    ) {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        prop_assert_ne!(nonce1, nonce2);
    }

    /// Argon2id derivation must be deterministic for the same inputs.
    #[test]
    fn derive_key_deterministic(
        password in "[a-zA-Z0-9]{1,32}",
        secret_key in any::<[u8; 32]>(),
        salt in any::<[u8; 32]>(),
    ) {
        use porkpie_crypto::Argon2Params;
        use secrecy::Secret;

        let params = Argon2Params::default();
        let key1 = derive_key(&Secret::new(password.clone()), &secret_key, &salt, &params);
        let key2 = derive_key(&Secret::new(password), &secret_key, &salt, &params);
        prop_assert!(key1.is_ok());
        prop_assert!(key2.is_ok());
        prop_assert_eq!(key1.unwrap(), key2.unwrap());
    }

    /// Argon2id derivation with different passwords must produce different keys.
    #[test]
    fn derive_key_password_sensitivity(
        password1 in "[a-zA-Z0-9]{1,32}",
        password2 in "[a-zA-Z0-9]{1,32}",
        secret_key in any::<[u8; 32]>(),
        salt in any::<[u8; 32]>(),
    ) {
        prop_assume!(password1 != password2);
        use porkpie_crypto::Argon2Params;
        use secrecy::Secret;

        let params = Argon2Params::default();
        let key1 = derive_key(&Secret::new(password1), &secret_key, &salt, &params);
        let key2 = derive_key(&Secret::new(password2), &secret_key, &salt, &params);
        prop_assert!(key1.is_ok());
        prop_assert!(key2.is_ok());
        prop_assert_ne!(key1.unwrap(), key2.unwrap());
    }
}
