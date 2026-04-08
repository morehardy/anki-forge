use anyhow::{Context, Result};
use authoring_core::{NormalizedField, NormalizedNotetype, NormalizedTemplate};
use prost::{Enumeration, Message};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Message)]
pub(crate) struct DeckCommon {
    #[prost(bool, tag = "1")]
    pub study_collapsed: bool,
    #[prost(bool, tag = "2")]
    pub browser_collapsed: bool,
    #[prost(uint32, tag = "3")]
    pub last_day_studied: u32,
    #[prost(int32, tag = "4")]
    pub new_studied: i32,
    #[prost(int32, tag = "5")]
    pub review_studied: i32,
    #[prost(int32, tag = "7")]
    pub milliseconds_studied: i32,
    #[prost(int32, tag = "6")]
    pub learning_studied: i32,
    #[prost(bytes = "vec", tag = "255")]
    pub other: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct DayLimit {
    #[prost(uint32, tag = "1")]
    pub limit: u32,
    #[prost(uint32, tag = "2")]
    pub today: u32,
    #[prost(bool, tag = "3")]
    pub today_only: bool,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct DeckNormal {
    #[prost(int64, tag = "1")]
    pub config_id: i64,
    #[prost(uint32, tag = "2")]
    pub extend_new: u32,
    #[prost(uint32, tag = "3")]
    pub extend_review: u32,
    #[prost(string, tag = "4")]
    pub description: String,
    #[prost(bool, tag = "5")]
    pub markdown_description: bool,
    #[prost(uint32, optional, tag = "6")]
    pub review_limit: Option<u32>,
    #[prost(message, optional, tag = "7")]
    pub review_limit_today: Option<DayLimit>,
    #[prost(message, optional, tag = "8")]
    pub new_limit_today: Option<DayLimit>,
    #[prost(float, optional, tag = "9")]
    pub desired_retention: Option<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct DeckFiltered {}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct DeckKindContainer {
    #[prost(oneof = "deck_kind_container::Kind", tags = "1, 2")]
    pub kind: Option<deck_kind_container::Kind>,
}

pub(crate) mod deck_kind_container {
    use super::{DeckFiltered, DeckNormal};
    use prost::Oneof;

    #[derive(Clone, PartialEq, Oneof)]
    pub(crate) enum Kind {
        #[prost(message, tag = "1")]
        Normal(DeckNormal),
        #[prost(message, tag = "2")]
        Filtered(DeckFiltered),
    }
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct DeckConfigConfig {
    #[prost(float, repeated, tag = "1")]
    pub learn_steps: Vec<f32>,
    #[prost(float, repeated, tag = "2")]
    pub relearn_steps: Vec<f32>,
    #[prost(uint32, tag = "9")]
    pub new_per_day: u32,
    #[prost(uint32, tag = "10")]
    pub reviews_per_day: u32,
    #[prost(float, tag = "11")]
    pub initial_ease: f32,
    #[prost(float, tag = "12")]
    pub easy_multiplier: f32,
    #[prost(float, tag = "13")]
    pub hard_multiplier: f32,
    #[prost(float, tag = "14")]
    pub lapse_multiplier: f32,
    #[prost(float, tag = "15")]
    pub interval_multiplier: f32,
    #[prost(uint32, tag = "16")]
    pub maximum_review_interval: u32,
    #[prost(uint32, tag = "17")]
    pub minimum_lapse_interval: u32,
    #[prost(uint32, tag = "18")]
    pub graduating_interval_good: u32,
    #[prost(uint32, tag = "19")]
    pub graduating_interval_easy: u32,
    #[prost(enumeration = "NewCardInsertOrder", tag = "20")]
    pub new_card_insert_order: i32,
    #[prost(enumeration = "LeechAction", tag = "21")]
    pub leech_action: i32,
    #[prost(uint32, tag = "22")]
    pub leech_threshold: u32,
    #[prost(bool, tag = "23")]
    pub disable_autoplay: bool,
    #[prost(uint32, tag = "24")]
    pub cap_answer_time_to_secs: u32,
    #[prost(bool, tag = "25")]
    pub show_timer: bool,
    #[prost(bool, tag = "26")]
    pub skip_question_when_replaying_answer: bool,
    #[prost(bool, tag = "27")]
    pub bury_new: bool,
    #[prost(bool, tag = "28")]
    pub bury_reviews: bool,
    #[prost(bool, tag = "29")]
    pub bury_interday_learning: bool,
    #[prost(enumeration = "ReviewMix", tag = "30")]
    pub new_mix: i32,
    #[prost(enumeration = "ReviewMix", tag = "31")]
    pub interday_learning_mix: i32,
    #[prost(enumeration = "NewCardSortOrder", tag = "32")]
    pub new_card_sort_order: i32,
    #[prost(enumeration = "ReviewCardOrder", tag = "33")]
    pub review_order: i32,
    #[prost(enumeration = "NewCardGatherPriority", tag = "34")]
    pub new_card_gather_priority: i32,
    #[prost(float, tag = "37")]
    pub desired_retention: f32,
    #[prost(bool, tag = "38")]
    pub stop_timer_on_answer: bool,
    #[prost(bytes = "vec", tag = "255")]
    pub other: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum NewCardInsertOrder {
    Due = 0,
    Random = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum NewCardGatherPriority {
    Deck = 0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum NewCardSortOrder {
    Template = 0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum ReviewCardOrder {
    Day = 0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum ReviewMix {
    MixWithReviews = 0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum LeechAction {
    Suspend = 0,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct NotetypeConfig {
    #[prost(enumeration = "NotetypeKind", tag = "1")]
    pub kind: i32,
    #[prost(uint32, tag = "2")]
    pub sort_field_idx: u32,
    #[prost(string, tag = "3")]
    pub css: String,
    #[prost(string, tag = "5")]
    pub latex_pre: String,
    #[prost(string, tag = "6")]
    pub latex_post: String,
    #[prost(bool, tag = "7")]
    pub latex_svg: bool,
    #[prost(message, repeated, tag = "8")]
    pub reqs: Vec<CardRequirement>,
    #[prost(enumeration = "OriginalStockKind", tag = "9")]
    pub original_stock_kind: i32,
    #[prost(int64, optional, tag = "10")]
    pub original_id: Option<i64>,
    #[prost(bytes = "vec", tag = "255")]
    pub other: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct CardRequirement {
    #[prost(uint32, tag = "1")]
    pub card_ord: u32,
    #[prost(enumeration = "CardRequirementKind", tag = "2")]
    pub kind: i32,
    #[prost(uint32, repeated, tag = "3")]
    pub field_ords: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum NotetypeKind {
    Normal = 0,
    Cloze = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum CardRequirementKind {
    None = 0,
    Any = 1,
    All = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub(crate) enum OriginalStockKind {
    Unknown = 0,
    Basic = 1,
    BasicAndReversed = 2,
    BasicOptionalReversed = 3,
    BasicTyping = 4,
    Cloze = 5,
    ImageOcclusion = 6,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct NoteFieldConfig {
    #[prost(bool, tag = "1")]
    pub sticky: bool,
    #[prost(bool, tag = "2")]
    pub rtl: bool,
    #[prost(string, tag = "3")]
    pub font_name: String,
    #[prost(uint32, tag = "4")]
    pub font_size: u32,
    #[prost(string, tag = "5")]
    pub description: String,
    #[prost(bool, tag = "6")]
    pub plain_text: bool,
    #[prost(bool, tag = "7")]
    pub collapsed: bool,
    #[prost(bool, tag = "8")]
    pub exclude_from_search: bool,
    #[prost(int64, optional, tag = "9")]
    pub id: Option<i64>,
    #[prost(uint32, optional, tag = "10")]
    pub tag: Option<u32>,
    #[prost(bool, tag = "11")]
    pub prevent_deletion: bool,
    #[prost(bytes = "vec", tag = "255")]
    pub other: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct TemplateConfig {
    #[prost(string, tag = "1")]
    pub q_format: String,
    #[prost(string, tag = "2")]
    pub a_format: String,
    #[prost(string, tag = "3")]
    pub q_format_browser: String,
    #[prost(string, tag = "4")]
    pub a_format_browser: String,
    #[prost(int64, tag = "5")]
    pub target_deck_id: i64,
    #[prost(string, tag = "6")]
    pub browser_font_name: String,
    #[prost(uint32, tag = "7")]
    pub browser_font_size: u32,
    #[prost(int64, optional, tag = "8")]
    pub id: Option<i64>,
    #[prost(bytes = "vec", tag = "255")]
    pub other: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NotetypeMetadata {
    pub anki_forge_notetype_id: String,
}

pub(crate) fn default_deck_common_bytes() -> Vec<u8> {
    DeckCommon {
        study_collapsed: false,
        browser_collapsed: false,
        last_day_studied: 0,
        new_studied: 0,
        review_studied: 0,
        milliseconds_studied: 0,
        learning_studied: 0,
        other: vec![],
    }
    .encode_to_vec()
}

pub(crate) fn default_deck_kind_bytes(config_id: i64) -> Vec<u8> {
    DeckKindContainer {
        kind: Some(deck_kind_container::Kind::Normal(DeckNormal {
            config_id,
            extend_new: 0,
            extend_review: 0,
            description: String::new(),
            markdown_description: false,
            review_limit: None,
            review_limit_today: None,
            new_limit_today: None,
            desired_retention: None,
        })),
    }
    .encode_to_vec()
}

pub(crate) fn default_deck_config_bytes() -> Vec<u8> {
    DeckConfigConfig {
        learn_steps: vec![1.0, 10.0],
        relearn_steps: vec![10.0],
        new_per_day: 20,
        reviews_per_day: 200,
        initial_ease: 2.5,
        easy_multiplier: 1.3,
        hard_multiplier: 1.2,
        lapse_multiplier: 0.0,
        interval_multiplier: 1.0,
        maximum_review_interval: 36500,
        minimum_lapse_interval: 1,
        graduating_interval_good: 1,
        graduating_interval_easy: 4,
        new_card_insert_order: NewCardInsertOrder::Due as i32,
        leech_action: LeechAction::Suspend as i32,
        leech_threshold: 8,
        disable_autoplay: false,
        cap_answer_time_to_secs: 60,
        show_timer: false,
        skip_question_when_replaying_answer: false,
        bury_new: false,
        bury_reviews: false,
        bury_interday_learning: false,
        new_mix: ReviewMix::MixWithReviews as i32,
        interday_learning_mix: ReviewMix::MixWithReviews as i32,
        new_card_sort_order: NewCardSortOrder::Template as i32,
        review_order: ReviewCardOrder::Day as i32,
        new_card_gather_priority: NewCardGatherPriority::Deck as i32,
        desired_retention: 0.9,
        stop_timer_on_answer: false,
        other: vec![],
    }
    .encode_to_vec()
}

pub(crate) fn encode_notetype_config(notetype: &NormalizedNotetype) -> Result<Vec<u8>> {
    let metadata = NotetypeMetadata {
        anki_forge_notetype_id: notetype.id.clone(),
    };

    Ok(NotetypeConfig {
        kind: storage_notetype_kind(notetype) as i32,
        sort_field_idx: 0,
        css: notetype.css.clone(),
        latex_pre: String::new(),
        latex_post: String::new(),
        latex_svg: false,
        reqs: storage_card_requirements(notetype),
        original_stock_kind: storage_original_stock_kind(notetype) as i32,
        original_id: notetype.original_id,
        other: serde_json::to_vec(&metadata).context("encode notetype storage metadata")?,
    }
    .encode_to_vec())
}

pub(crate) fn encode_field_config(field: &NormalizedField) -> Vec<u8> {
    NoteFieldConfig {
        sticky: false,
        rtl: false,
        font_name: "Arial".into(),
        font_size: 20,
        description: String::new(),
        plain_text: false,
        collapsed: false,
        exclude_from_search: false,
        id: field.config_id,
        tag: field.tag,
        prevent_deletion: field.prevent_deletion,
        other: vec![],
    }
    .encode_to_vec()
}

pub(crate) fn encode_template_config(template: &NormalizedTemplate) -> Vec<u8> {
    TemplateConfig {
        q_format: template.question_format.clone(),
        a_format: template.answer_format.clone(),
        q_format_browser: template.browser_question_format.clone().unwrap_or_default(),
        a_format_browser: template.browser_answer_format.clone().unwrap_or_default(),
        target_deck_id: 0,
        browser_font_name: template.browser_font_name.clone().unwrap_or_default(),
        browser_font_size: template.browser_font_size.unwrap_or_default(),
        id: template.config_id,
        other: vec![],
    }
    .encode_to_vec()
}

pub(crate) fn decode_notetype_config(bytes: &[u8]) -> Result<NotetypeConfig> {
    NotetypeConfig::decode(bytes).context("decode Anki notetype config")
}

pub(crate) fn decode_field_config(bytes: &[u8]) -> Result<NoteFieldConfig> {
    NoteFieldConfig::decode(bytes).context("decode Anki field config")
}

pub(crate) fn decode_template_config(bytes: &[u8]) -> Result<TemplateConfig> {
    TemplateConfig::decode(bytes).context("decode Anki template config")
}

pub(crate) fn decode_notetype_metadata(bytes: &[u8]) -> Result<Option<NotetypeMetadata>> {
    if bytes.is_empty() {
        return Ok(None);
    }

    let metadata =
        serde_json::from_slice(bytes).context("decode Anki-forge notetype storage metadata")?;
    Ok(Some(metadata))
}

fn storage_notetype_kind(notetype: &NormalizedNotetype) -> NotetypeKind {
    match notetype.kind.as_str() {
        "cloze" => NotetypeKind::Cloze,
        _ => NotetypeKind::Normal,
    }
}

fn storage_original_stock_kind(notetype: &NormalizedNotetype) -> OriginalStockKind {
    match notetype.original_stock_kind.as_deref().unwrap_or(notetype.kind.as_str()) {
        "basic" => OriginalStockKind::Basic,
        "cloze" => OriginalStockKind::Cloze,
        "image_occlusion" => OriginalStockKind::ImageOcclusion,
        _ => OriginalStockKind::Unknown,
    }
}

fn storage_card_requirements(notetype: &NormalizedNotetype) -> Vec<CardRequirement> {
    match notetype.original_stock_kind.as_deref().unwrap_or(notetype.kind.as_str()) {
        "basic" => vec![CardRequirement {
            card_ord: 0,
            kind: CardRequirementKind::All as i32,
            field_ords: vec![0],
        }],
        "image_occlusion" => vec![CardRequirement {
            card_ord: 0,
            kind: CardRequirementKind::All as i32,
            field_ords: vec![0, 1],
        }],
        "cloze" => vec![],
        _ => notetype
            .templates
            .iter()
            .enumerate()
            .map(|(ord, _)| CardRequirement {
                card_ord: ord as u32,
                kind: CardRequirementKind::Any as i32,
                field_ords: (0..notetype.fields.len() as u32).collect(),
            })
            .collect(),
    }
}
