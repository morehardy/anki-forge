pub const STOCK_BASIC_ID: &str = "basic";
pub const STOCK_CLOZE_ID: &str = "cloze";
pub const STOCK_IMAGE_OCCLUSION_ID: &str = "image_occlusion";

pub(crate) fn is_supported_stock_notetype_id(id: &str) -> bool {
    matches!(
        id,
        STOCK_BASIC_ID | STOCK_CLOZE_ID | STOCK_IMAGE_OCCLUSION_ID
    )
}
