pub fn stable_config_id(namespace: &str, note_type_id: &str, key: &str) -> i64 {
    let payload = format!("{namespace}\0{note_type_id}\0{key}");
    let digest = blake3::hash(payload.as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest.as_bytes()[0..8]);
    i64::from_be_bytes(bytes) & i64::MAX
}
