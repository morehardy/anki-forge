use std::{collections::BTreeMap, path::Path};

use anyhow::{ensure, Context};
use serde::Deserialize;

use crate::{Content, Field, Media, Note, NoteType, Project, Template};

/// Loads `ankiforge-project-v1` tool input through the native authoring API.
/// Relative media and bundle paths resolve against `base_dir`, or the input's
/// directory. Media is snapshotted during this operation. Legacy product JSON
/// is rejected, and no editable or serializable internal IR is returned.
pub fn load_project(path: impl AsRef<Path>, base_dir: Option<&Path>) -> anyhow::Result<Project> {
    let path = path.as_ref();
    let input: Input = serde_json::from_reader(
        std::fs::File::open(path)
            .with_context(|| format!("open project input {}", path.display()))?,
    )
    .with_context(|| format!("decode ankiforge-project-v1 input {}", path.display()))?;
    ensure!(
        input.format_version == "ankiforge-project-v1",
        "unsupported project format_version"
    );
    let base = base_dir.unwrap_or_else(|| path.parent().unwrap_or_else(|| Path::new(".")));
    let mut project = Project::new(input.namespace)?;
    if let Some(name) = input.name {
        project = project.name(name);
    }
    if let Some(deck) = input.default_deck {
        project = project.default_deck(deck);
    }
    let mut assets = BTreeMap::new();
    for asset in input.assets {
        ensure!(
            !asset.key.trim().is_empty() && !assets.contains_key(&asset.key),
            "empty or duplicate asset key {:?}",
            asset.key
        );
        let mut media = match asset.source {
            AssetSource::File { path } => Media::file(base.join(path))?,
            AssetSource::Bytes { data, mime } => Media::bytes(data, mime)?,
        };
        if let Some(name) = asset.export_as {
            media = media.with_export_name(name)?;
        }
        project.add_asset(media.clone())?;
        assets.insert(asset.key, media);
    }
    let mut models = BTreeMap::new();
    for model in input.models {
        let model = match model {
            ModelInput::Bundle { path } => NoteType::from_bundle(base.join(path))?,
            ModelInput::Custom {
                key,
                name,
                fields,
                templates,
                css,
                cloze_field,
                assets: dependencies,
            } => {
                let mut model = NoteType::builder(key);
                if let Some(name) = name {
                    model = model.name(name);
                }
                if let Some(css) = css {
                    model = model.css(css);
                }
                if let Some(key) = cloze_field {
                    model = model.cloze_field(key);
                }
                for field in fields {
                    let mut value = Field::new(field.key);
                    if let Some(name) = field.name {
                        value = value.name(name);
                    }
                    if field.required {
                        value = value.required();
                    }
                    if field.sort {
                        value = value.sort();
                    }
                    model = model.field(value);
                }
                for template in templates {
                    let mut value = Template::new(template.key)
                        .front(template.front)
                        .back(template.back);
                    if let Some(name) = template.name {
                        value = value.name(name);
                    }
                    if let Some(front) = template.browser_front {
                        value = value.browser_front(front);
                    }
                    if let Some(back) = template.browser_back {
                        value = value.browser_back(back);
                    }
                    if let Some(deck) = template.target_deck {
                        value = value.target_deck(deck);
                    }
                    if let Some(rule) = template.generation_rule {
                        value = value.generate_when(match rule {
                            Generation::AnkiDefault => crate::schema::GenerationRule::AnkiDefault,
                            Generation::All { fields } => {
                                crate::schema::GenerationRule::all(fields)
                            }
                            Generation::Any { fields } => {
                                crate::schema::GenerationRule::any(fields)
                            }
                        });
                    }
                    model = model.template(value);
                }
                for key in dependencies {
                    model = model.asset(asset(&assets, &key)?.clone());
                }
                model.build()?
            }
        };
        ensure!(
            !models.contains_key(model.key()),
            "duplicate model key {:?}",
            model.key()
        );
        models.insert(model.key().to_owned(), model);
    }
    for input in input.notes {
        let mut note = match input.content {
            NoteInput::Basic { front, back } => {
                Note::basic(front.resolve(&assets)?, back.resolve(&assets)?)
            }
            NoteInput::Cloze { text, back_extra } => Note::cloze(text.resolve(&assets)?)
                .field("back_extra", back_extra.resolve(&assets)?),
            NoteInput::Custom { model, fields } => {
                let mut note = models
                    .get(&model)
                    .with_context(|| format!("unknown model {model:?}"))?
                    .note();
                for (key, value) in fields {
                    note = note.field(key, value.resolve(&assets)?);
                }
                note
            }
            NoteInput::ImageOcclusion {
                image,
                masks,
                mode,
                fields,
            } => {
                let mut builder =
                    Note::image_occlusion(asset(&assets, &image)?.clone()).mode(match mode {
                        OcclusionMode::HideAllGuessOne => {
                            crate::note::OcclusionMode::HideAllGuessOne
                        }
                        OcclusionMode::HideOneGuessOne => {
                            crate::note::OcclusionMode::HideOneGuessOne
                        }
                    });
                for mask in masks {
                    builder = builder.mask(crate::note::Mask::rect(
                        mask.key,
                        mask.x,
                        mask.y,
                        mask.width,
                        mask.height,
                    ));
                }
                let mut note = builder.build()?;
                for (key, value) in fields {
                    note = note.field(key, value.resolve(&assets)?);
                }
                note
            }
        };
        if let Some(deck) = input.deck {
            note = note.deck(deck);
        }
        project.add(input.key, note.tags(input.tags))?;
    }
    Ok(project)
}

fn asset<'a>(assets: &'a BTreeMap<String, Media>, key: &str) -> anyhow::Result<&'a Media> {
    assets
        .get(key)
        .with_context(|| format!("unknown asset {key:?}"))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    format_version: String,
    namespace: String,
    name: Option<String>,
    default_deck: Option<String>,
    #[serde(default)]
    models: Vec<ModelInput>,
    #[serde(default)]
    assets: Vec<Asset>,
    notes: Vec<NoteSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    key: String,
    source: AssetSource,
    export_as: Option<String>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum AssetSource {
    File { path: String },
    Bytes { data: Vec<u8>, mime: String },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ModelInput {
    Bundle {
        path: String,
    },
    Custom {
        key: String,
        name: Option<String>,
        fields: Vec<FieldInput>,
        templates: Vec<TemplateInput>,
        css: Option<String>,
        cloze_field: Option<String>,
        #[serde(default)]
        assets: Vec<String>,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldInput {
    key: String,
    name: Option<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    sort: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateInput {
    key: String,
    name: Option<String>,
    front: String,
    back: String,
    browser_front: Option<String>,
    browser_back: Option<String>,
    target_deck: Option<String>,
    generation_rule: Option<Generation>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Generation {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoteSpec {
    key: String,
    deck: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    content: NoteInput,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum NoteInput {
    Basic {
        front: ContentInput,
        back: ContentInput,
    },
    Cloze {
        text: ContentInput,
        #[serde(default)]
        back_extra: ContentInput,
    },
    Custom {
        model: String,
        fields: BTreeMap<String, ContentInput>,
    },
    ImageOcclusion {
        image: String,
        masks: Vec<MaskInput>,
        #[serde(default)]
        mode: OcclusionMode,
        #[serde(default)]
        fields: BTreeMap<String, ContentInput>,
    },
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum OcclusionMode {
    #[default]
    HideAllGuessOne,
    HideOneGuessOne,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaskInput {
    key: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ContentInput {
    Text(String),
    Node(ContentNode),
}
impl Default for ContentInput {
    fn default() -> Self {
        Self::Text(String::new())
    }
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ContentNode {
    Text { value: String },
    Html { value: String },
    Image { asset: String },
    Sound { asset: String },
    Sequence { items: Vec<ContentInput> },
}
impl ContentInput {
    fn resolve(self, assets: &BTreeMap<String, Media>) -> anyhow::Result<Content> {
        Ok(match self {
            Self::Text(value) | Self::Node(ContentNode::Text { value }) => Content::text(value),
            Self::Node(ContentNode::Html { value }) => Content::html(value),
            Self::Node(ContentNode::Image { asset: key }) => asset(assets, &key)?.image(),
            Self::Node(ContentNode::Sound { asset: key }) => asset(assets, &key)?.sound(),
            Self::Node(ContentNode::Sequence { items }) => Content::sequence(
                items
                    .into_iter()
                    .map(|item| item.resolve(assets))
                    .collect::<anyhow::Result<Vec<_>>>()?,
            ),
        })
    }
}
