use porkpie_crypto::*;
use secrecy::Secret;

#[test]
fn test_argon2id_derivation() {
    let password = Secret::new("correct horse battery staple".to_string());
    let secret_key = [42u8; 32];
    let salt = [0u8; 32];
    let key1 = derive_key(&password, &secret_key, &salt, &Default::default()).unwrap();
    let key2 = derive_key(&password, &secret_key, &salt, &Default::default()).unwrap();
    assert_eq!(key1, key2);

    let salt2 = [1u8; 32];
    let key3 = derive_key(&password, &secret_key, &salt2, &Default::default()).unwrap();
    assert_ne!(key1, key3);
}

#[test]
fn test_derive_key_requires_secret_key() {
    let password = Secret::new("correct horse battery staple".to_string());
    let secret_a = [1u8; 32];
    let secret_b = [2u8; 32];
    let salt = [0u8; 32];
    let key_a = derive_key(&password, &secret_a, &salt, &Default::default()).unwrap();
    let key_b = derive_key(&password, &secret_b, &salt, &Default::default()).unwrap();
    assert_ne!(
        key_a, key_b,
        "different secret keys must produce different derived keys"
    );
}

#[test]
fn test_encryption_roundtrip() {
    let master_key = [1u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];
    let aad = b"test-aad";

    let encrypted = encrypt_item(&test_item, &master_key, aad).unwrap();
    let decrypted: Vec<u8> = decrypt_item(&encrypted, &master_key, aad).unwrap();

    assert_eq!(test_item, decrypted);
}

#[test]
fn test_encryption_wrong_key() {
    let master_key = [1u8; 32];
    let wrong_key = [2u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];
    let aad = b"test-aad";

    let encrypted = encrypt_item(&test_item, &master_key, aad).unwrap();
    let decrypted_result: Result<Vec<u8>, CryptoError> = decrypt_item(&encrypted, &wrong_key, aad);

    assert!(decrypted_result.is_err());
}

#[test]
fn test_tampering() {
    let master_key = [1u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];
    let aad = b"test-aad";

    let mut encrypted = encrypt_item(&test_item, &master_key, aad).unwrap();

    let len = encrypted.len();
    encrypted[len - 1] ^= 1;

    let decrypted_result: Result<Vec<u8>, CryptoError> = decrypt_item(&encrypted, &master_key, aad);
    assert!(decrypted_result.is_err());
}

#[test]
fn test_wrong_aad_fails_decrypt() {
    let master_key = [1u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];
    let correct_aad = b"vault-123|item-456|Login";
    let wrong_aad = b"vault-123|item-456|APIKey";

    let encrypted = encrypt_item(&test_item, &master_key, correct_aad).unwrap();
    let result: Result<Vec<u8>, CryptoError> = decrypt_item(&encrypted, &master_key, wrong_aad);
    assert!(result.is_err(), "wrong AAD must fail decryption");
}

#[test]
fn test_wrong_vault_id_aad_fails() {
    let master_key = [1u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];
    let correct_aad = b"porkpie-v1|vault-aaa|item-bbb|Login";
    let wrong_aad = b"porkpie-v1|vault-zzz|item-bbb|Login";

    let encrypted = encrypt_item(&test_item, &master_key, correct_aad).unwrap();
    let result: Result<Vec<u8>, CryptoError> = decrypt_item(&encrypted, &master_key, wrong_aad);
    assert!(
        result.is_err(),
        "wrong vault ID in AAD must fail decryption"
    );
}

#[test]
fn test_wrong_item_id_aad_fails() {
    let master_key = [1u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];
    let correct_aad = b"porkpie-v1|vault-aaa|item-bbb|Login";
    let wrong_aad = b"porkpie-v1|vault-aaa|item-ccc|Login";

    let encrypted = encrypt_item(&test_item, &master_key, correct_aad).unwrap();
    let result: Result<Vec<u8>, CryptoError> = decrypt_item(&encrypted, &master_key, wrong_aad);
    assert!(result.is_err(), "wrong item ID in AAD must fail decryption");
}

#[test]
fn test_wrong_schema_version_aad_fails() {
    let master_key = [1u8; 32];
    let test_item = vec![1, 2, 3, 4, 5];
    let correct_aad = b"porkpie-v1|vault-aaa|item-bbb|Login";
    let wrong_aad = b"porkpie-v2|vault-aaa|item-bbb|Login";

    let encrypted = encrypt_item(&test_item, &master_key, correct_aad).unwrap();
    let result: Result<Vec<u8>, CryptoError> = decrypt_item(&encrypted, &master_key, wrong_aad);
    assert!(
        result.is_err(),
        "wrong schema version in AAD must fail decryption"
    );
}

#[test]
fn test_vault_key_wrapping() {
    let master_key = [1u8; 32];
    let vault_key = [2u8; 32];

    let wrapped = wrap_vault_key(&master_key, &vault_key).unwrap();
    let unwrapped = unwrap_vault_key(&master_key, &wrapped).unwrap();

    assert_eq!(vault_key, unwrapped);

    let wrong_master_key = [3u8; 32];
    assert!(unwrap_vault_key(&wrong_master_key, &wrapped).is_err());
}

#[test]
fn test_nonce_uniqueness() {
    let n1 = generate_nonce();
    let n2 = generate_nonce();
    assert_ne!(n1, n2);
}
