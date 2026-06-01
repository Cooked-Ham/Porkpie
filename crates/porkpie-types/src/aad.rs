pub const SCHEMA_VERSION: u8 = 1;

pub fn build_item_aad(vault_id: &str, item_id: &str, item_type: &str) -> Vec<u8> {
    let mut aad = Vec::new();
    aad.extend_from_slice(b"porkpie-v");
    aad.push(SCHEMA_VERSION);
    aad.push(b'|');
    aad.extend_from_slice(vault_id.as_bytes());
    aad.push(b'|');
    aad.extend_from_slice(item_id.as_bytes());
    aad.push(b'|');
    aad.extend_from_slice(item_type.as_bytes());
    aad
}

pub fn build_payload_aad(vault_id: &str) -> Vec<u8> {
    let mut aad = Vec::new();
    aad.extend_from_slice(b"porkpie-payload-v");
    aad.push(SCHEMA_VERSION);
    aad.push(b'|');
    aad.extend_from_slice(vault_id.as_bytes());
    aad
}
