use anyhow::{bail, Result};

use crate::model::{
    AuthoringField, AuthoringFieldMetadata, AuthoringNotetype, AuthoringTemplate, NormalizedField,
    NormalizedFieldMetadata, NormalizedNotetype, NormalizedTemplate,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockLoweringDefaults {
    pub kind: String,
    pub original_stock_kind: String,
    pub name: String,
    pub fields: Vec<AuthoringField>,
    pub templates: Vec<AuthoringTemplate>,
    pub css: String,
    pub field_metadata: Vec<AuthoringFieldMetadata>,
}

pub fn stock_lowering_defaults(kind: &str) -> Result<StockLoweringDefaults> {
    match kind {
        "basic" => Ok(StockLoweringDefaults {
            kind: "basic".into(),
            original_stock_kind: "basic".into(),
            name: "Basic".into(),
            fields: vec![
                authoring_field("Front", Some(0), None, false),
                authoring_field("Back", Some(1), None, false),
            ],
            templates: vec![authoring_template(
                "Card 1",
                Some(0),
                "{{Front}}",
                "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}",
            )],
            css: String::new(),
            field_metadata: vec![],
        }),
        "cloze" => Ok(StockLoweringDefaults {
            kind: "cloze".into(),
            original_stock_kind: "cloze".into(),
            name: "Cloze".into(),
            fields: vec![
                authoring_field("Text", Some(0), Some(0), true),
                authoring_field("Back Extra", Some(1), Some(1), false),
            ],
            templates: vec![authoring_template(
                "Cloze",
                Some(0),
                "{{cloze:Text}}",
                "{{cloze:Text}}<br>\n{{Back Extra}}",
            )],
            css: CLOZE_CSS.into(),
            field_metadata: vec![],
        }),
        "image_occlusion" => Ok(StockLoweringDefaults {
            kind: "cloze".into(),
            original_stock_kind: "image_occlusion".into(),
            name: "Image Occlusion".into(),
            fields: vec![
                authoring_field("Occlusion", Some(0), Some(0), true),
                authoring_field("Image", Some(1), Some(1), true),
                authoring_field("Header", Some(2), Some(2), true),
                authoring_field("Back Extra", Some(3), Some(3), true),
                authoring_field("Comments", Some(4), Some(4), false),
            ],
            templates: vec![authoring_template(
                "Image Occlusion",
                Some(0),
                IMAGE_OCCLUSION_QFMT,
                IMAGE_OCCLUSION_AFMT,
            )],
            css: "#image-occlusion-canvas {\n    --inactive-shape-color: #ffeba2;\n    --active-shape-color: #ff8e8e;\n    --inactive-shape-border: 1px #212121;\n    --active-shape-border: 1px #212121;\n    --highlight-shape-color: #ff8e8e00;\n    --highlight-shape-border: 1px #ff8e8e;\n}\n\n.card {\n    font-family: arial;\n    font-size: 20px;\n    text-align: center;\n    color: black;\n    background-color: white;\n}\n"
                .into(),
            field_metadata: vec![],
        }),
        other => bail!("unsupported stock notetype kind: {other}"),
    }
}

pub fn resolve_stock_notetype(input: &AuthoringNotetype) -> Result<NormalizedNotetype> {
    if input.fields.is_some() ^ input.templates.is_some() {
        bail!(
            "explicit lowered notetype payloads must provide both fields and templates for {}",
            input.id
        );
    }

    if let (Some(fields), Some(templates)) = (input.fields.as_ref(), input.templates.as_ref()) {
        return Ok(NormalizedNotetype {
            id: input.id.clone(),
            kind: input.kind.clone(),
            name: normalized_name(input),
            original_stock_kind: input.original_stock_kind.clone(),
            original_id: input.original_id,
            fields: fields.iter().cloned().map(normalized_field).collect(),
            templates: templates.iter().cloned().map(normalized_template).collect(),
            css: input.css.clone().unwrap_or_default(),
            field_metadata: input
                .field_metadata
                .iter()
                .cloned()
                .map(normalized_field_metadata)
                .collect(),
        });
    }

    let stock_kind = input
        .original_stock_kind
        .as_deref()
        .unwrap_or(input.kind.as_str());
    let defaults = stock_lowering_defaults(stock_kind)?;

    Ok(NormalizedNotetype {
        id: input.id.clone(),
        kind: legacy_normalized_kind(input.kind.as_str(), stock_kind),
        name: input.name.clone().unwrap_or(defaults.name),
        original_stock_kind: Some(stock_kind.into()),
        original_id: input.original_id,
        fields: defaults.fields.into_iter().map(normalized_field).collect(),
        templates: defaults
            .templates
            .into_iter()
            .map(normalized_template)
            .collect(),
        css: defaults.css,
        field_metadata: defaults
            .field_metadata
            .into_iter()
            .map(normalized_field_metadata)
            .collect(),
    })
}

fn legacy_normalized_kind(input_kind: &str, stock_kind: &str) -> String {
    match input_kind {
        "basic" | "image_occlusion" => input_kind.to_string(),
        "cloze" => {
            if stock_kind == "image_occlusion" {
                "image_occlusion".into()
            } else {
                "cloze".into()
            }
        }
        "normal" => "basic".into(),
        _ => stock_kind.to_string(),
    }
}

fn normalized_name(input: &AuthoringNotetype) -> String {
    input
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            input
                .original_stock_kind
                .as_deref()
                .and_then(|kind| stock_lowering_defaults(kind).ok())
                .map(|defaults| defaults.name)
                .unwrap_or_else(|| match input.kind.as_str() {
                    "basic" => "Basic".into(),
                    "cloze" => "Cloze".into(),
                    "image_occlusion" => "Image Occlusion".into(),
                    _ => input.kind.clone(),
                })
        })
}

fn authoring_field(
    name: &str,
    ord: Option<u32>,
    tag: Option<u32>,
    prevent_deletion: bool,
) -> AuthoringField {
    AuthoringField {
        name: name.into(),
        ord,
        config_id: None,
        tag,
        prevent_deletion,
    }
}

fn authoring_template(
    name: &str,
    ord: Option<u32>,
    question_format: &str,
    answer_format: &str,
) -> AuthoringTemplate {
    AuthoringTemplate {
        name: name.into(),
        ord,
        config_id: None,
        question_format: question_format.into(),
        answer_format: answer_format.into(),
        browser_question_format: None,
        browser_answer_format: None,
        target_deck_name: None,
        browser_font_name: None,
        browser_font_size: None,
    }
}

fn normalized_field(field: AuthoringField) -> NormalizedField {
    NormalizedField {
        name: field.name,
        ord: field.ord,
        config_id: field.config_id,
        tag: field.tag,
        prevent_deletion: field.prevent_deletion,
    }
}

fn normalized_template(template: AuthoringTemplate) -> NormalizedTemplate {
    NormalizedTemplate {
        name: template.name,
        ord: template.ord,
        config_id: template.config_id,
        question_format: template.question_format,
        answer_format: template.answer_format,
        browser_question_format: template.browser_question_format,
        browser_answer_format: template.browser_answer_format,
        target_deck_name: template.target_deck_name,
        browser_font_name: template.browser_font_name,
        browser_font_size: template.browser_font_size,
    }
}

fn normalized_field_metadata(metadata: AuthoringFieldMetadata) -> NormalizedFieldMetadata {
    NormalizedFieldMetadata {
        field_name: metadata.field_name,
        label: metadata.label,
        role_hint: metadata.role_hint,
    }
}
