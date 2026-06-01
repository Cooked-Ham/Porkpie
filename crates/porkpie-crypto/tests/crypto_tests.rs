use porkpie_crypto::*;
use secrecy::Secret;

#[test]
fn test_argon2id_derivation() {
    let password = Secret::new("correct horse battery staple".to_string());
    let salt = [0u8; 32];
    let key1 = derive_key(&password, &salt, &Default::default()).unwrap();
    let key2 = derive_key(&password, &salt, &Default::default()).unwrap();
    assert_eq!(key1, key2);

    let salt2 = [1u8; 32];
    let key3 = derive_key(&password, &salt2, &Default::default()).unwrap();
    assert_ne!(key1, key3);
}

#[test]
fn test_encryption_roundtrip() {
    let master_key = [1u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];

    let encrypted = encrypt_item(&test_item, &master_key).unwrap();
    let decrypted: Vec<u8> = decrypt_item(&encrypted, &master_key).unwrap();

    assert_eq!(test_item, decrypted);
}

#[test]
fn test_encryption_wrong_key() {
    let master_key = [1u8; 32];
    let wrong_key = [2u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];

    let encrypted = encrypt_item(&test_item, &master_key).unwrap();
    let decrypted_result: Result<Vec<u8>, CryptoError> = decrypt_item(&encrypted, &wrong_key);

    assert!(decrypted_result.is_err());
}

#[test]
fn test_tampering() {
    let master_key = [1u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];

    let mut encrypted = encrypt_item(&test_item, &master_key).unwrap();

    // Tamper with the ciphertext (last byte is part of MAC or data)
    let len = encrypted.len();
    encrypted[len - 1] ^= 1;

    let decrypted_result: Result<Vec<u8>, CryptoError> = decrypt_item(&encrypted, &master_key);
    assert!(decrypted_result.is_err());
}

#[test]
fn test_vault_key_wrapping() {
    let master_key = [1u8; 32];
    let vault_key = [2u8; 32];

    let wrapped = wrap_vault_key(&master_key, &vault_key).unwrap();
    let unwrapped = unwrap_vault_key(&master_key, &wrapped).unwrap();

    assert_eq!(vault_key, unwrapped);

    // Test wrong master key
    let wrong_master_key = [3u8; 32];
    assert!(unwrap_vault_key(&wrong_master_key, &wrapped).is_err());
}

#[test]
fn test_nonce_uniqueness() {
    let n1 = generate_nonce();
    let n2 = generate_nonce();
    assert_ne!(n1, n2);
}
