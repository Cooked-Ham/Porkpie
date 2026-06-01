use porkpie_core::Vault;
use porkpie_import::import_csv_reader;
use porkpie_types::ItemType;

#[test]
fn csv_import_creates_encrypted_login_rows() {
    let mut vault = Vault::create("sixteen-character-password").expect("create vault");
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
        .decrypt_item(&result.encrypted_items[0].ciphertext)
        .expect("decrypt imported item");
    assert!(matches!(item.data, ItemType::Login(_)));
}

#[test]
fn csv_import_rejects_missing_required_fields() {
    let mut vault = Vault::create("sixteen-character-password").expect("create vault");
    let csv = "item_type,title,username,password,notes\nlogin,Email,me@example.com,,primary\n";

    let error = import_csv_reader(csv.as_bytes(), &mut vault).expect_err("row should fail");

    assert!(error.to_string().contains("invalid row 2"));
}
