use porkpie_core::{LocalSecretKey, Vault};
use porkpie_import::import_csv_reader;
use porkpie_types::ItemType;

fn test_secret_key() -> LocalSecretKey {
    LocalSecretKey::from_hex("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        .unwrap()
}

#[test]
fn csv_import_creates_encrypted_login_rows() {
    let (mut vault, _) = Vault::create(
        "TestVault",
        "sixteen-character-password",
        &test_secret_key(),
    )
    .expect("create vault");
    let csv =
        "item_type,title,username,password,notes\nlogin,Email,me@example.com,secret,primary\n";

    let result = import_csv_reader(csv.as_bytes(), &mut vault).expect("import csv");

    assert_eq!(result.imported, 1);
    assert_eq!(result.encrypted_items.len(), 1);
    assert_eq!(result.encrypted_items[0].item_type, "Login");
    assert!(!result.encrypted_items[0]
        .ciphertext
        .windows(b"secret".len())
        .any(|window| window == b"secret"));

    let item = vault
        .decrypt_item(
            &result.encrypted_items[0].ciphertext,
            &result.encrypted_items[0].id,
            &result.encrypted_items[0].item_type,
        )
        .expect("decrypt imported item");
    assert!(matches!(item.data, ItemType::Login(_)));
}

#[test]
fn csv_import_rejects_missing_required_fields() {
    let (mut vault, _) = Vault::create(
        "TestVault",
        "sixteen-character-password",
        &test_secret_key(),
    )
    .expect("create vault");
    let csv = "item_type,title,username,password,notes\nlogin,Email,me@example.com,,primary\n";

    let error = import_csv_reader(csv.as_bytes(), &mut vault).expect_err("row should fail");

    assert!(error.to_string().contains("invalid row 2"));
}

#[test]
fn csv_imported_secret_is_encrypted_at_rest() {
    let (mut vault, _) = Vault::create(
        "TestVault",
        "sixteen-character-password",
        &test_secret_key(),
    )
    .expect("create vault");
    let csv =
        "item_type,title,username,password,notes\nlogin,Email,me@example.com,DO_NOT_LEAK_CSV_SECRET,primary\n";

    let result = import_csv_reader(csv.as_bytes(), &mut vault).expect("import csv");
    assert_eq!(result.imported, 1);

    // The imported secret must be encrypted immediately; the ciphertext must
    // not contain the plaintext marker.
    let ciphertext = &result.encrypted_items[0].ciphertext;
    let marker = b"DO_NOT_LEAK_CSV_SECRET";
    assert!(
        !ciphertext
            .windows(marker.len())
            .any(|window| window == marker),
        "imported secret must not appear in ciphertext"
    );
}
