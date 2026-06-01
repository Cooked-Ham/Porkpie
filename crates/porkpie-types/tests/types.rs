use porkpie_types::*;

#[test]
fn test_id_generation() {
    let _v_id = VaultId::new();
    let _i_id = ItemId::new();
    let _u_id = UserId::new();
    assert_ne!(_v_id.as_uuid().to_string(), _i_id.as_uuid().to_string());
}
