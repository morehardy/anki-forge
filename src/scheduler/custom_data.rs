#[must_use]
pub fn validate_custom_data_size(data: &str, max_len: usize) -> bool {
    data.len() <= max_len
}
