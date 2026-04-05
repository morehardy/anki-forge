use anyhow::{bail, Result};

use crate::model::{AuthoringNotetype, NormalizedNotetype, NormalizedTemplate};

// Source-grounded text transcribed from the local Anki mirror:
// - /Users/hp/Desktop/2026/anki-forge/docs/source/rslib/src/notetype/cloze_styling.css
// - /Users/hp/Desktop/2026/anki-forge/docs/source/rslib/src/image_occlusion/notetype.rs
// - /Users/hp/Desktop/2026/anki-forge/docs/source/rslib/src/image_occlusion/notetype.css
const CLOZE_CSS: &str = ".cloze {\n    font-weight: bold;\n    color: blue;\n}\n.nightMode .cloze {\n    color: lightblue;\n}\n";

const IMAGE_OCCLUSION_QFMT: &str = r#"{{#Header}}<div>{{Header}}</div>{{/Header}}
<div style="display: none">{{cloze:Occlusion}}</div>
<div id="err"></div>
<div id="image-occlusion-container">
    {{Image}}
    <canvas id="image-occlusion-canvas"></canvas>
</div>
<script>
try {
    anki.imageOcclusion.setup();
} catch (exc) {
    document.getElementById("err").innerHTML = `Error loading image occlusion<br><br>${exc}`;
}
</script>
"#;

const IMAGE_OCCLUSION_AFMT: &str = r#"{{#Header}}<div>{{Header}}</div>{{/Header}}
<div style="display: none">{{cloze:Occlusion}}</div>
<div id="err"></div>
<div id="image-occlusion-container">
    {{Image}}
    <canvas id="image-occlusion-canvas"></canvas>
</div>
<script>
try {
    anki.imageOcclusion.setup();
} catch (exc) {
    document.getElementById("err").innerHTML = `Error loading image occlusion<br><br>${exc}`;
}
</script>

<div><button id="toggle">Toggle Masks</button></div>
{{#Back Extra}}<div>{{Back Extra}}</div>{{/Back Extra}}
"#;

pub fn resolve_stock_notetype(input: &AuthoringNotetype) -> Result<NormalizedNotetype> {
    let name = normalized_name(input);

    match input.kind.as_str() {
        "basic" => Ok(NormalizedNotetype {
            id: input.id.clone(),
            kind: "basic".into(),
            name: name.clone(),
            fields: vec!["Front".into(), "Back".into()],
            templates: vec![NormalizedTemplate {
                name: "Card 1".into(),
                question_format: "{{Front}}".into(),
                answer_format: "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}".into(),
            }],
            css: String::new(),
        }),
        "cloze" => Ok(NormalizedNotetype {
            id: input.id.clone(),
            kind: "cloze".into(),
            name: name.clone(),
            fields: vec!["Text".into(), "Back Extra".into()],
            templates: vec![NormalizedTemplate {
                name: name.clone(),
                question_format: "{{cloze:Text}}".into(),
                answer_format: "{{cloze:Text}}<br>\n{{Back Extra}}".into(),
            }],
            css: CLOZE_CSS.into(),
        }),
        "image_occlusion" => Ok(NormalizedNotetype {
            id: input.id.clone(),
            kind: "image_occlusion".into(),
            name: name.clone(),
            fields: vec![
                "Occlusion".into(),
                "Image".into(),
                "Header".into(),
                "Back Extra".into(),
                "Comments".into(),
            ],
            templates: vec![NormalizedTemplate {
                name: name.clone(),
                question_format: IMAGE_OCCLUSION_QFMT.into(),
                answer_format: IMAGE_OCCLUSION_AFMT.into(),
            }],
            css: "#image-occlusion-canvas {\n    --inactive-shape-color: #ffeba2;\n    --active-shape-color: #ff8e8e;\n    --inactive-shape-border: 1px #212121;\n    --active-shape-border: 1px #212121;\n    --highlight-shape-color: #ff8e8e00;\n    --highlight-shape-border: 1px #ff8e8e;\n}\n\n.card {\n    font-family: arial;\n    font-size: 20px;\n    text-align: center;\n    color: black;\n    background-color: white;\n}\n"
                .into(),
        }),
        other => bail!("unsupported stock notetype kind: {other}"),
    }
}

fn normalized_name(input: &AuthoringNotetype) -> String {
    input
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| match input.kind.as_str() {
            "basic" => "Basic".into(),
            "cloze" => "Cloze".into(),
            "image_occlusion" => "Image Occlusion".into(),
            _ => input.kind.clone(),
        })
}
