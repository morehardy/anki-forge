use crate::deck::model::IoRect;
use crate::IoMode;

pub const STOCK_BASIC_ID: &str = "basic";
pub const STOCK_CLOZE_ID: &str = "cloze";
pub const STOCK_IMAGE_OCCLUSION_ID: &str = "image_occlusion";

pub fn render_image_occlusion_cloze(mode: IoMode, rects: &[IoRect]) -> anyhow::Result<String> {
    anyhow::ensure!(
        !rects.is_empty(),
        "image occlusion note requires at least one rect"
    );

    let prefix = match mode {
        IoMode::HideAllGuessOne => "c1",
        IoMode::HideOneGuessOne => "c1,2",
    };

    let mut rendered = String::new();
    for rect in rects {
        rendered.push_str(&format!(
            "{{{{{}::image-occlusion:rect:left={}:top={}:width={}:height={}}}}}<br>",
            prefix, rect.x, rect.y, rect.width, rect.height,
        ));
    }

    Ok(rendered)
}
