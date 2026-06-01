//! Property-based fuzz tests for porkpie-types.

use porkpie_types::{ItemId, PieUri, VaultId};
use proptest::prelude::*;

proptest! {
    /// ItemId and VaultId roundtrip through string representation.
    #[test]
    fn id_roundtrip(_seed in any::<u64>()) {
        let item_id = ItemId::new();
        let vault_id = VaultId::new();
        let item_str = item_id.to_string();
        let vault_str = vault_id.to_string();
        prop_assert_eq!(item_id, ItemId::from_string(&item_str).unwrap());
        prop_assert_eq!(vault_id, VaultId::from_string(&vault_str).unwrap());
    }

    /// PieUri parsing must not panic on arbitrary strings.
    #[test]
    fn pie_uri_parse_does_not_panic(uri in any::<String>()) {
        let _ = PieUri::parse(&uri);
    }

    /// PieUri display and parse roundtrip for valid URIs.
    #[test]
    fn pie_uri_roundtrip(
        vault_name in "[a-zA-Z0-9]{1,32}",
        item_name in "[a-zA-Z0-9]{1,32}",
        field_name in "[a-zA-Z0-9]{1,32}",
    ) {
        let uri_str = format!("pie://{}/{}/{}", vault_name, item_name, field_name);
        let uri = PieUri::parse(&uri_str).unwrap();
        prop_assert_eq!(uri.vault_name, vault_name);
        prop_assert_eq!(uri.item_name, item_name);
        prop_assert_eq!(uri.field_name, field_name);
    }

    /// LocalSecretKey hex encoding roundtrip.
    #[test]
    fn local_secret_key_hex_roundtrip(_seed in any::<u64>()) {
        let key = porkpie_types::LocalSecretKey::generate();
        let hex = key.to_hex();
        let decoded = porkpie_types::LocalSecretKey::from_hex(&hex).unwrap();
        prop_assert_eq!(key.to_hex(), decoded.to_hex());
    }
}
