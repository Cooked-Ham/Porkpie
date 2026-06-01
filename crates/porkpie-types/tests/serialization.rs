use porkpie_types::*;
use std::collections::HashMap;

#[test]
fn test_round_trip_serialization() {
    let mut fields = HashMap::new();
    fields.insert("key".to_string(), "value".to_string());

    let secret = CustomSecret { fields };
    let item = ItemType::Custom(secret);

    let serialized = serde_json::to_string(&item).unwrap();
    let deserialized: ItemType = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        ItemType::Custom(c) => {
            assert_eq!(c.fields.get("key").unwrap(), "value");
        }
        _ => panic!("Wrong type"),
    }
}
