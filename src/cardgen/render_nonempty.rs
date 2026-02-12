#[must_use]
pub fn render_is_nonempty(rendered: &str) -> bool {
    !rendered.trim().is_empty()
}
