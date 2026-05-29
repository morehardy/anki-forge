use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

use super::diagnostics::ProductDiagnostic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductDocument {
    document_id: String,
    #[serde(default)]
    pub(super) note_types: Vec<ProductNoteType>,
    #[serde(default)]
    pub(super) notes: Vec<ProductNote>,
    #[serde(default)]
    pub(super) helpers: Vec<(String, super::helpers::HelperDeclaration)>,
    #[serde(default)]
    pub(super) assets: Vec<super::assets::AssetSource>,
    #[serde(default)]
    pub(super) font_bindings: Vec<super::assets::FontBinding>,
    #[serde(default)]
    pub(super) field_metadata: Vec<(String, super::metadata::FieldMetadataDeclaration)>,
    #[serde(default)]
    pub(super) browser_appearance: Vec<(
        String,
        super::metadata::TemplateBrowserAppearanceDeclaration,
    )>,
    #[serde(default)]
    pub(super) template_target_decks: Vec<(String, super::metadata::TemplateTargetDeckDeclaration)>,
    #[serde(default)]
    pub(super) default_deck_name: Option<String>,
    #[serde(skip)]
    product_v2: Option<ProductDocumentV2Payload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ProductDocumentLegacy {
    document_id: String,
    #[serde(default)]
    note_types: Vec<ProductNoteType>,
    #[serde(default)]
    notes: Vec<ProductNote>,
    #[serde(default)]
    helpers: Vec<(String, super::helpers::HelperDeclaration)>,
    #[serde(default)]
    assets: Vec<super::assets::AssetSource>,
    #[serde(default)]
    font_bindings: Vec<super::assets::FontBinding>,
    #[serde(default)]
    field_metadata: Vec<(String, super::metadata::FieldMetadataDeclaration)>,
    #[serde(default)]
    browser_appearance: Vec<(
        String,
        super::metadata::TemplateBrowserAppearanceDeclaration,
    )>,
    #[serde(default)]
    template_target_decks: Vec<(String, super::metadata::TemplateTargetDeckDeclaration)>,
    #[serde(default)]
    default_deck_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProductDocumentV2Payload {
    pub(crate) note_types: Vec<ProductNoteTypeV2>,
    pub(crate) notes: Vec<ProductNoteV2>,
    pub(crate) media: Vec<ProductMediaV2>,
    pub(crate) transport_diagnostics: Vec<ProductDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ProductDocumentV2 {
    document_id: String,
    #[serde(default)]
    default_deck_name: Option<String>,
    #[serde(default)]
    note_types: Vec<ProductNoteTypeV2>,
    #[serde(default)]
    notes: Vec<ProductNoteV2>,
    #[serde(default)]
    media: Vec<ProductMediaV2>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProductNoteTypeV2 {
    Stock(ProductStockNoteTypeV2),
    Custom(ProductCustomNoteTypeV2),
    Unknown(UnknownProductObjectV2),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ProductStockNoteTypeV2 {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) fields: Vec<ProductFieldV2>,
    #[serde(default)]
    pub(crate) templates: Vec<ProductTemplateV2>,
    #[serde(default)]
    pub(crate) css: Option<String>,
    #[serde(default)]
    pub(crate) source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ProductCustomNoteTypeV2 {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) fields: Vec<ProductFieldV2>,
    #[serde(default)]
    pub(crate) templates: Vec<ProductTemplateV2>,
    #[serde(default)]
    pub(crate) identity: Option<ProductIdentityV2>,
    #[serde(default)]
    pub(crate) css: Option<String>,
    #[serde(default)]
    pub(crate) source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ProductFieldV2 {
    pub(crate) name: String,
    pub(crate) key: String,
    #[serde(default)]
    pub(crate) identity: bool,
    #[serde(default)]
    pub(crate) sort: bool,
    #[serde(default)]
    pub(crate) required: bool,
    #[serde(default)]
    pub(crate) source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ProductTemplateV2 {
    pub(crate) name: String,
    pub(crate) key: String,
    pub(crate) front: String,
    pub(crate) back: String,
    #[serde(default)]
    pub(crate) generation_rule: Option<ProductGenerationRuleV2>,
    #[serde(default)]
    pub(crate) source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ProductGenerationRuleV2 {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
    Cloze { field: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ProductFieldContentV2 {
    Text { value: String },
    Html { value: String },
    Sound { media_id: String },
    Image { media_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ProductMediaV2 {
    pub(crate) id: String,
    pub(crate) source: ProductMediaSourceV2,
    pub(crate) export_as: String,
    #[serde(default)]
    pub(crate) source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ProductMediaSourceV2 {
    File {
        path: String,
    },
    InlineBase64 {
        source_label: String,
        data_base64: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProductIdentityV2 {
    Fields { fields: Vec<String> },
    Unknown(UnknownProductObjectV2),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProductNoteV2 {
    Stock(ProductStockNoteV2),
    Custom(ProductCustomNoteV2),
    Unknown(UnknownProductObjectV2),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ProductStockNoteV2 {
    pub(crate) note_type_id: String,
    #[serde(default)]
    pub(crate) stable_id: Option<String>,
    pub(crate) deck_name: String,
    #[serde(default)]
    pub(crate) fields: BTreeMap<String, ProductFieldContentV2>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ProductCustomNoteV2 {
    pub(crate) note_type_id: String,
    #[serde(default)]
    pub(crate) stable_id: Option<String>,
    pub(crate) deck_name: String,
    #[serde(default)]
    pub(crate) fields: BTreeMap<String, ProductFieldContentV2>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnknownProductObjectV2 {
    pub(crate) kind: String,
    pub(crate) raw: serde_json::Value,
}

impl<'de> Deserialize<'de> for ProductDocument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value.get("product_document_version") {
            Some(serde_json::Value::String(version)) if version == "product-v2" => {
                serde_json::from_value::<ProductDocumentV2>(value)
                    .map(ProductDocument::from)
                    .map_err(serde::de::Error::custom)
            }
            Some(version) => Ok(ProductDocument::from_unknown_version(&value, version)),
            None => serde_json::from_value::<ProductDocumentLegacy>(value)
                .map(ProductDocument::from)
                .map_err(serde::de::Error::custom),
        }
    }
}

impl From<ProductDocumentLegacy> for ProductDocument {
    fn from(legacy: ProductDocumentLegacy) -> Self {
        Self {
            document_id: legacy.document_id,
            note_types: legacy.note_types,
            notes: legacy.notes,
            helpers: legacy.helpers,
            assets: legacy.assets,
            font_bindings: legacy.font_bindings,
            field_metadata: legacy.field_metadata,
            browser_appearance: legacy.browser_appearance,
            template_target_decks: legacy.template_target_decks,
            default_deck_name: legacy.default_deck_name,
            product_v2: None,
        }
    }
}

impl From<ProductDocumentV2> for ProductDocument {
    fn from(v2: ProductDocumentV2) -> Self {
        let mut document = ProductDocument::new(v2.document_id);
        document.default_deck_name = v2.default_deck_name;
        document.note_types = v2
            .note_types
            .iter()
            .filter_map(convert_note_type_v2)
            .collect();
        document.notes = v2.notes.iter().filter_map(convert_note_v2).collect();
        document.product_v2 = Some(ProductDocumentV2Payload {
            note_types: v2.note_types,
            notes: v2.notes,
            media: v2.media,
            transport_diagnostics: Vec::new(),
        });
        document
    }
}

impl<'de> Deserialize<'de> for ProductNoteTypeV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let kind = object_kind(&value).map_err(serde::de::Error::custom)?;
        match kind.as_str() {
            "stock" => serde_json::from_value::<ProductStockNoteTypeV2>(value)
                .map(ProductNoteTypeV2::Stock)
                .map_err(serde::de::Error::custom),
            "custom" => serde_json::from_value::<ProductCustomNoteTypeV2>(value)
                .map(ProductNoteTypeV2::Custom)
                .map_err(serde::de::Error::custom),
            _ => Ok(ProductNoteTypeV2::Unknown(UnknownProductObjectV2 {
                kind,
                raw: value,
            })),
        }
    }
}

impl<'de> Deserialize<'de> for ProductIdentityV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let kind = object_kind(&value).map_err(serde::de::Error::custom)?;
        match kind.as_str() {
            "fields" => {
                #[derive(Deserialize)]
                struct FieldsIdentity {
                    fields: Vec<String>,
                }

                serde_json::from_value::<FieldsIdentity>(value)
                    .map(|identity| ProductIdentityV2::Fields {
                        fields: identity.fields,
                    })
                    .map_err(serde::de::Error::custom)
            }
            _ => Ok(ProductIdentityV2::Unknown(UnknownProductObjectV2 {
                kind,
                raw: value,
            })),
        }
    }
}

impl<'de> Deserialize<'de> for ProductNoteV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let kind = object_kind(&value).map_err(serde::de::Error::custom)?;
        match kind.as_str() {
            "stock" => serde_json::from_value::<ProductStockNoteV2>(value)
                .map(ProductNoteV2::Stock)
                .map_err(serde::de::Error::custom),
            "custom" => serde_json::from_value::<ProductCustomNoteV2>(value)
                .map(ProductNoteV2::Custom)
                .map_err(serde::de::Error::custom),
            _ => Ok(ProductNoteV2::Unknown(UnknownProductObjectV2 {
                kind,
                raw: value,
            })),
        }
    }
}

fn object_kind(value: &serde_json::Value) -> Result<String, &'static str> {
    value
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or("product-v2 object is missing a string kind")
}

fn convert_note_type_v2(notetype: &ProductNoteTypeV2) -> Option<ProductNoteType> {
    match notetype {
        ProductNoteTypeV2::Stock(stock) => Some(match stock.id.as_str() {
            "cloze" => ProductNoteType::Cloze(ClozeNoteType {
                id: stock.id.clone(),
                name: stock.name.clone(),
            }),
            "image_occlusion" => ProductNoteType::ImageOcclusion(ImageOcclusionNoteType {
                id: stock.id.clone(),
                name: stock.name.clone(),
            }),
            _ => ProductNoteType::Basic(BasicNoteType {
                id: stock.id.clone(),
                name: stock.name.clone(),
            }),
        }),
        ProductNoteTypeV2::Custom(custom) => Some(ProductNoteType::Custom(CustomNoteType {
            id: custom.id.clone(),
            name: custom.name.clone(),
            fields: custom
                .fields
                .iter()
                .map(|field| CustomField {
                    name: field.name.clone(),
                    key: Some(field.key.clone()),
                })
                .collect(),
            templates: custom
                .templates
                .iter()
                .map(|template| CustomTemplate {
                    name: template.name.clone(),
                    key: Some(template.key.clone()),
                    question_format: template.front.clone(),
                    answer_format: template.back.clone(),
                    generation_rule: template
                        .generation_rule
                        .as_ref()
                        .map(convert_generation_rule_v2),
                })
                .collect(),
            css: custom.css.clone(),
        })),
        ProductNoteTypeV2::Unknown(_) => None,
    }
}

fn convert_note_v2(note: &ProductNoteV2) -> Option<ProductNote> {
    match note {
        ProductNoteV2::Stock(stock) => match stock.note_type_id.as_str() {
            "cloze" => Some(ProductNote::Cloze(ClozeNote {
                id: note_v2_id(stock.stable_id.as_deref(), stock.source_path.as_deref()),
                note_type_id: stock.note_type_id.clone(),
                deck_name: stock.deck_name.clone(),
                text: field_content_text(stock.fields.get("text")),
                back_extra: field_content_text(stock.fields.get("back_extra")),
                tags: stock.tags.clone(),
            })),
            "image_occlusion" => Some(ProductNote::ImageOcclusion(ImageOcclusionNote {
                id: note_v2_id(stock.stable_id.as_deref(), stock.source_path.as_deref()),
                note_type_id: stock.note_type_id.clone(),
                deck_name: stock.deck_name.clone(),
                occlusion: field_content_text(stock.fields.get("occlusion")),
                image: field_content_text(stock.fields.get("image")),
                header: field_content_text(stock.fields.get("header")),
                back_extra: field_content_text(stock.fields.get("back_extra")),
                comments: field_content_text(stock.fields.get("comments")),
                tags: stock.tags.clone(),
            })),
            _ => Some(ProductNote::Basic(BasicNote {
                id: note_v2_id(stock.stable_id.as_deref(), stock.source_path.as_deref()),
                note_type_id: stock.note_type_id.clone(),
                deck_name: stock.deck_name.clone(),
                front: field_content_text(stock.fields.get("front")),
                back: field_content_text(stock.fields.get("back")),
                tags: stock.tags.clone(),
            })),
        },
        ProductNoteV2::Custom(custom) => Some(ProductNote::Custom(CustomNote {
            id: note_v2_id(custom.stable_id.as_deref(), custom.source_path.as_deref()),
            note_type_id: custom.note_type_id.clone(),
            deck_name: custom.deck_name.clone(),
            fields: custom
                .fields
                .iter()
                .map(|(key, value)| (key.clone(), field_content_text(Some(value))))
                .collect(),
            tags: custom.tags.clone(),
        })),
        ProductNoteV2::Unknown(_) => None,
    }
}

fn convert_generation_rule_v2(rule: &ProductGenerationRuleV2) -> CustomGenerationRule {
    match rule {
        ProductGenerationRuleV2::AnkiDefault => CustomGenerationRule::AnkiDefault,
        ProductGenerationRuleV2::All { fields } => CustomGenerationRule::All {
            fields: fields.clone(),
        },
        ProductGenerationRuleV2::Any { fields } => CustomGenerationRule::Any {
            fields: fields.clone(),
        },
        ProductGenerationRuleV2::Cloze { field } => CustomGenerationRule::Cloze {
            field: field.clone(),
        },
    }
}

fn field_content_text(content: Option<&ProductFieldContentV2>) -> String {
    match content {
        Some(ProductFieldContentV2::Text { value })
        | Some(ProductFieldContentV2::Html { value }) => value.clone(),
        Some(ProductFieldContentV2::Sound { media_id })
        | Some(ProductFieldContentV2::Image { media_id }) => format!("[{media_id}]"),
        None => String::new(),
    }
}

fn note_v2_id(stable_id: Option<&str>, source_path: Option<&str>) -> String {
    stable_id
        .or(source_path)
        .map(str::to_owned)
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNoteType {
    Basic(BasicNoteType),
    Cloze(ClozeNoteType),
    ImageOcclusion(ImageOcclusionNoteType),
    Custom(CustomNoteType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNote {
    Basic(BasicNote),
    Cloze(ClozeNote),
    ImageOcclusion(ImageOcclusionNote),
    Custom(CustomNote),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageOcclusionNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomNoteType {
    pub id: String,
    pub name: Option<String>,
    pub fields: Vec<CustomField>,
    pub templates: Vec<CustomTemplate>,
    pub css: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub text: String,
    pub back_extra: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageOcclusionNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub occlusion: String,
    pub image: String,
    pub header: String,
    pub back_extra: String,
    pub comments: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomTemplate {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub question_format: String,
    pub answer_format: String,
    #[serde(default)]
    pub generation_rule: Option<CustomGenerationRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomGenerationRule {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
    Cloze { field: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub fields: BTreeMap<String, String>,
    pub tags: Vec<String>,
}

impl ProductDocument {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            note_types: Vec::new(),
            notes: Vec::new(),
            helpers: Vec::new(),
            assets: Vec::new(),
            font_bindings: Vec::new(),
            field_metadata: Vec::new(),
            browser_appearance: Vec::new(),
            template_target_decks: Vec::new(),
            default_deck_name: None,
            product_v2: None,
        }
    }

    fn from_unknown_version(raw: &serde_json::Value, version: &serde_json::Value) -> Self {
        let document_id = raw
            .get("document_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let mut document = ProductDocument::new(document_id);
        document.default_deck_name = raw
            .get("default_deck_name")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        document.product_v2 = Some(ProductDocumentV2Payload {
            note_types: Vec::new(),
            notes: Vec::new(),
            media: Vec::new(),
            transport_diagnostics: vec![ProductDiagnostic {
                code: "PRODUCT.VERSION_UNSUPPORTED",
                message: format!("Unsupported product document version '{version}'."),
            }],
        });
        document
    }

    pub fn with_basic(mut self, id: impl Into<String>) -> Self {
        self.note_types.push(ProductNoteType::Basic(BasicNoteType {
            id: id.into(),
            name: None,
        }));
        self
    }

    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    pub fn note_types(&self) -> &[ProductNoteType] {
        &self.note_types
    }

    pub fn notes(&self) -> &[ProductNote] {
        &self.notes
    }

    #[allow(dead_code)]
    pub(crate) fn product_v2(&self) -> Option<&ProductDocumentV2Payload> {
        self.product_v2.as_ref()
    }

    pub fn assets(&self) -> &[super::assets::AssetSource] {
        &self.assets
    }

    pub fn font_bindings(&self) -> &[super::assets::FontBinding] {
        &self.font_bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_string_product_version_records_transport_diagnostic() {
        let raw = r#"{
            "product_document_version": "product-v3",
            "document_id": "future",
            "note_types": [],
            "notes": []
        }"#;

        let doc: ProductDocument = serde_json::from_str(raw).expect("unsupported version");
        let payload = doc.product_v2().expect("transport payload");

        assert_eq!(doc.document_id(), "future");
        assert_eq!(payload.transport_diagnostics.len(), 1);
        assert_eq!(
            payload.transport_diagnostics[0].code,
            "PRODUCT.VERSION_UNSUPPORTED"
        );
    }

    #[test]
    fn non_string_product_version_records_transport_diagnostic() {
        let raw = r#"{
            "product_document_version": 2,
            "document_id": "numeric-version",
            "note_types": [],
            "notes": []
        }"#;

        let doc: ProductDocument = serde_json::from_str(raw).expect("unsupported version");
        let payload = doc.product_v2().expect("transport payload");

        assert_eq!(doc.document_id(), "numeric-version");
        assert_eq!(payload.transport_diagnostics.len(), 1);
        assert_eq!(
            payload.transport_diagnostics[0].code,
            "PRODUCT.VERSION_UNSUPPORTED"
        );
    }
}
