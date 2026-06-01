use porkpie_types::{PieUri, PieUriError};

#[test]
fn pie_uri_parse_valid() {
    let uri = PieUri::parse("pie://Personal/GitHub/password").unwrap();
    assert_eq!(uri.vault_name, "Personal");
    assert_eq!(uri.item_name, "GitHub");
    assert_eq!(uri.field_name, "password");
}

#[test]
fn pie_uri_parse_missing_scheme() {
    let result = PieUri::parse("Personal/GitHub/password");
    assert!(matches!(result, Err(PieUriError::MissingScheme)));
}

#[test]
fn pie_uri_parse_invalid_format() {
    let result = PieUri::parse("pie://Personal/GitHub");
    assert!(matches!(result, Err(PieUriError::InvalidFormat)));
}

#[test]
fn pie_uri_parse_empty_vault() {
    let result = PieUri::parse("pie:///GitHub/password");
    assert!(matches!(result, Err(PieUriError::EmptyVaultName)));
}

#[test]
fn pie_uri_parse_empty_item() {
    let result = PieUri::parse("pie://Personal//password");
    assert!(matches!(result, Err(PieUriError::EmptyItemName)));
}

#[test]
fn pie_uri_parse_empty_field() {
    let result = PieUri::parse("pie://Personal/GitHub/");
    assert!(matches!(result, Err(PieUriError::EmptyFieldName)));
}

#[test]
fn pie_uri_display() {
    let uri = PieUri {
        vault_name: "Personal".to_string(),
        item_name: "GitHub".to_string(),
        field_name: "password".to_string(),
    };
    assert_eq!(uri.to_string(), "pie://Personal/GitHub/password");
}

#[test]
fn pie_uri_redacted_display() {
    let uri = PieUri {
        vault_name: "Personal".to_string(),
        item_name: "GitHub".to_string(),
        field_name: "password".to_string(),
    };
    assert_eq!(uri.to_string_redacted(), "pie://Personal/GitHub/[redacted]");
}

#[test]
fn pie_uri_error_does_not_leak_values() {
    let result = PieUri::parse("pie://Personal/GitHub/");
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.contains("Personal"));
    assert!(!error_msg.contains("GitHub"));
    assert!(!error_msg.contains("password"));
}
