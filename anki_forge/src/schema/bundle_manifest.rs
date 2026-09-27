use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    pub(super) format_version: String,
    pub(super) note_type: Model,
    pub(super) css_file: Option<String>,
    #[serde(default)]
    pub(super) assets: Vec<Asset>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Model {
    pub(super) key: String,
    pub(super) name: Option<String>,
    pub(super) cloze_field: Option<String>,
    pub(super) fields: Vec<Field>,
    pub(super) templates: Vec<Template>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Field {
    pub(super) key: String,
    pub(super) name: Option<String>,
    #[serde(default)]
    pub(super) required: bool,
    #[serde(default)]
    pub(super) sort: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Template {
    pub(super) key: String,
    pub(super) name: Option<String>,
    pub(super) front_file: String,
    pub(super) back_file: String,
    pub(super) browser_front_file: Option<String>,
    pub(super) browser_back_file: Option<String>,
    pub(super) target_deck: Option<String>,
    pub(super) generation_rule: Option<Generation>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Generation {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Asset {
    pub(super) path: String,
    pub(super) export_as: String,
}
