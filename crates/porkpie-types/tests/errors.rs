use porkpie_types::*;

#[test]
fn test_error_formatting() {
    let err = VaultError::NotFound("test-vault".to_string());
    assert_eq!(err.to_string(), "Vault not found: test-vault");
}
