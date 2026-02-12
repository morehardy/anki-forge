#[must_use]
pub fn parse_template_tokens(template: &str) -> Vec<String> {
    template
        .split_whitespace()
        .map(std::borrow::ToOwned::to_owned)
        .collect()
}
