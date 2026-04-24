use crate::deck::model::{
    BasicIdentityField, BasicNote, ClozeNote, Deck, DeckNote, IdentityProvenance, IoMode, IoNote,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::num::NonZeroU32;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedIdentity {
    pub stable_id: String,
    pub recipe_id: String,
    pub canonical_payload: String,
    pub provenance: IdentityProvenance,
    pub used_override: bool,
}

#[derive(Debug, Serialize)]
struct CanonicalIdentityPayload<'a, T: Serialize> {
    algo_version: u8,
    recipe_id: &'a str,
    notetype_family: &'a str,
    notetype_key: &'a str,
    components: T,
}

#[derive(Debug, Serialize)]
struct BasicFieldComponent {
    name: &'static str,
    value: String,
}

#[derive(Debug, Serialize)]
struct BasicComponents {
    selected_fields: Vec<BasicFieldComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClozeSegment {
    Text(String),
    Deletion {
        ord: NonZeroU32,
        body: String,
        slot: usize,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ClozeDeletion {
    ord: u32,
    body: String,
    slot: usize,
}

#[derive(Debug, Serialize)]
struct ClozeComponents {
    text_skeleton: String,
    deletions: Vec<ClozeDeletion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum IoModeWire {
    HideAllGuessOne,
    HideOneGuessOne,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
enum IoShapeComponent {
    Rect {
        x_px: u32,
        y_px: u32,
        w_px: u32,
        h_px: u32,
    },
}

impl IoShapeComponent {
    fn sort_key(&self) -> (u8, u32, u32, u32, u32) {
        match self {
            Self::Rect {
                x_px,
                y_px,
                w_px,
                h_px,
            } => (0, *x_px, *y_px, *w_px, *h_px),
        }
    }
}

#[derive(Debug, Serialize)]
struct IoComponents {
    image_anchor: String,
    image_width_px: u32,
    image_height_px: u32,
    occlusion_mode: IoModeWire,
    shapes: Vec<IoShapeComponent>,
}

const BASIC_RECIPE_ID: &str = "basic.core.v1";
const CLOZE_RECIPE_ID: &str = "cloze.core.v2";
const IO_RECIPE_ID: &str = "io.core.v2";
const BASIC_DEFAULT_FIELDS: [BasicIdentityField; 1] = [BasicIdentityField::Front];

pub(crate) fn normalize_field_text_for_identity(value: &str) -> String {
    value
        .nfc()
        .collect::<String>()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

pub(crate) fn hash_payload<T: Serialize>(
    recipe_id: &str,
    notetype_family: &str,
    notetype_key: &str,
    components: T,
) -> anyhow::Result<(String, String)> {
    let payload = CanonicalIdentityPayload {
        algo_version: 1,
        recipe_id,
        notetype_family,
        notetype_key,
        components,
    };
    let canonical_payload = authoring_core::to_canonical_json(&payload)?;
    let stable_id = format!(
        "afid:v1:{}",
        blake3::hash(canonical_payload.as_bytes()).to_hex()
    );
    Ok((stable_id, canonical_payload))
}

pub(crate) fn resolve_inferred_identity(
    deck: &Deck,
    note: &DeckNote,
) -> anyhow::Result<ResolvedIdentity> {
    match note {
        DeckNote::Basic(note) => resolve_basic_identity(deck, note),
        DeckNote::Cloze(note) => resolve_cloze_identity(note),
        DeckNote::ImageOcclusion(note) => resolve_io_identity(deck, note),
    }
}

fn resolve_basic_identity(deck: &Deck, note: &BasicNote) -> anyhow::Result<ResolvedIdentity> {
    let (fields, provenance, used_override) =
        if let Some(override_cfg) = note.identity_override_config() {
            (
                override_cfg.fields(),
                IdentityProvenance::InferredFromNoteFields,
                true,
            )
        } else if let Some(selection) = deck.identity_policy().basic.as_ref() {
            (
                selection.as_slice(),
                IdentityProvenance::InferredFromNotetypeFields,
                false,
            )
        } else {
            (
                BASIC_DEFAULT_FIELDS.as_slice(),
                IdentityProvenance::InferredFromStockRecipe,
                false,
            )
        };

    let components = BasicComponents {
        selected_fields: fields
            .iter()
            .map(|field| match field {
                BasicIdentityField::Front => BasicFieldComponent {
                    name: "front",
                    value: normalize_field_text_for_identity(&note.front),
                },
                BasicIdentityField::Back => BasicFieldComponent {
                    name: "back",
                    value: normalize_field_text_for_identity(&note.back),
                },
            })
            .collect(),
    };
    let (stable_id, canonical_payload) =
        hash_payload(BASIC_RECIPE_ID, "stock", "basic", components)?;

    Ok(ResolvedIdentity {
        stable_id,
        recipe_id: BASIC_RECIPE_ID.to_string(),
        canonical_payload,
        provenance,
        used_override,
    })
}

fn resolve_cloze_identity(note: &ClozeNote) -> anyhow::Result<ResolvedIdentity> {
    let normalized_text = normalize_field_text_for_identity(&note.text);
    let segments = parse_cloze_segments(&normalized_text)?;
    let components = cloze_components_from_segments(&segments)?;
    let (stable_id, canonical_payload) =
        hash_payload(CLOZE_RECIPE_ID, "stock", "cloze", components)?;

    Ok(ResolvedIdentity {
        stable_id,
        recipe_id: CLOZE_RECIPE_ID.to_string(),
        canonical_payload,
        provenance: IdentityProvenance::InferredFromStockRecipe,
        used_override: false,
    })
}

fn resolve_io_identity(deck: &Deck, note: &IoNote) -> anyhow::Result<ResolvedIdentity> {
    if note.rects.is_empty() {
        anyhow::bail!("AFID.IDENTITY_COMPONENT_EMPTY: io rects");
    }

    let media = deck
        .media
        .get(note.image.name())
        .ok_or_else(|| anyhow::anyhow!("AFID.IDENTITY_COMPONENT_EMPTY: missing io media"))?;
    let raster_image = media
        .raster_image
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("AFID.IO_IMAGE_DIMENSIONS_MISSING"))?;

    let mut seen_rects = BTreeSet::new();
    let mut shapes = Vec::with_capacity(note.rects.len());
    for rect in &note.rects {
        if rect.width == 0 || rect.height == 0 {
            anyhow::bail!("AFID.IO_RECT_EMPTY");
        }

        let right = rect
            .x
            .checked_add(rect.width)
            .ok_or_else(|| anyhow::anyhow!("AFID.IO_RECT_OUT_OF_BOUNDS"))?;
        let bottom = rect
            .y
            .checked_add(rect.height)
            .ok_or_else(|| anyhow::anyhow!("AFID.IO_RECT_OUT_OF_BOUNDS"))?;
        if right > raster_image.width_px || bottom > raster_image.height_px {
            anyhow::bail!("AFID.IO_RECT_OUT_OF_BOUNDS");
        }

        let rect_key = (rect.x, rect.y, rect.width, rect.height);
        if !seen_rects.insert(rect_key) {
            anyhow::bail!("AFID.IO_RECT_DUPLICATE");
        }

        shapes.push(IoShapeComponent::Rect {
            x_px: rect.x,
            y_px: rect.y,
            w_px: rect.width,
            h_px: rect.height,
        });
    }
    shapes.sort_by_key(IoShapeComponent::sort_key);

    let components = IoComponents {
        image_anchor: media.sha1_hex.clone(),
        image_width_px: raster_image.width_px,
        image_height_px: raster_image.height_px,
        occlusion_mode: IoModeWire::from(note.mode),
        shapes,
    };
    let (stable_id, canonical_payload) =
        hash_payload(IO_RECIPE_ID, "stock", "image_occlusion", components)?;

    Ok(ResolvedIdentity {
        stable_id,
        recipe_id: IO_RECIPE_ID.to_string(),
        canonical_payload,
        provenance: IdentityProvenance::InferredFromStockRecipe,
        used_override: false,
    })
}

impl From<IoMode> for IoModeWire {
    fn from(mode: IoMode) -> Self {
        match mode {
            IoMode::HideAllGuessOne => Self::HideAllGuessOne,
            IoMode::HideOneGuessOne => Self::HideOneGuessOne,
        }
    }
}

fn parse_cloze_segments(input: &str) -> anyhow::Result<Vec<ClozeSegment>> {
    let mut segments = Vec::new();
    let mut cursor = 0;
    let mut slot = 0;

    while let Some(start) = find_next_cloze_start(input, cursor) {
        if start > cursor {
            segments.push(ClozeSegment::Text(input[cursor..start].to_string()));
        }

        let (segment, next_cursor) = parse_cloze_deletion(input, start, slot)?;
        segments.push(segment);
        cursor = next_cursor;
        slot += 1;
    }

    if cursor < input.len() {
        segments.push(ClozeSegment::Text(input[cursor..].to_string()));
    }

    if slot == 0 {
        anyhow::bail!("AFID.IDENTITY_COMPONENT_EMPTY: cloze deletions");
    }

    Ok(segments)
}

fn find_next_cloze_start(input: &str, cursor: usize) -> Option<usize> {
    let mut search_from = cursor;
    while search_from < input.len() {
        let relative = input[search_from..].find("{{c")?;
        let start = search_from + relative;
        let digit_index = start + "{{c".len();
        if input
            .as_bytes()
            .get(digit_index)
            .is_some_and(u8::is_ascii_digit)
        {
            return Some(start);
        }
        search_from = next_char_boundary(input, start);
    }

    None
}

fn parse_cloze_deletion(
    input: &str,
    start: usize,
    slot: usize,
) -> anyhow::Result<(ClozeSegment, usize)> {
    let digits_start = start + "{{c".len();
    let mut digits_end = digits_start;
    while input
        .as_bytes()
        .get(digits_end)
        .is_some_and(u8::is_ascii_digit)
    {
        digits_end += 1;
    }

    if !input[digits_end..].starts_with("::") {
        anyhow::bail!("AFID.CLOZE_MALFORMED: expected cloze body delimiter");
    }

    let ord = input[digits_start..digits_end]
        .parse::<u32>()
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or_else(|| anyhow::anyhow!("AFID.CLOZE_ORD_INVALID: cloze ordinal"))?;

    let body_start = digits_end + "::".len();
    let mut cursor = body_start;
    let mut hint_start = None;
    let mut close_start = None;

    while cursor < input.len() {
        if is_cloze_start_at(input, cursor) {
            anyhow::bail!("AFID.CLOZE_NESTED_UNSUPPORTED: nested cloze deletion");
        }
        if input[cursor..].starts_with("::") {
            hint_start = Some(cursor + "::".len());
            cursor += "::".len();
            break;
        }
        if input[cursor..].starts_with("}}") {
            close_start = Some(cursor);
            break;
        }
        cursor = next_char_boundary(input, cursor);
    }

    if hint_start.is_some() {
        while cursor < input.len() {
            if is_cloze_start_at(input, cursor) {
                anyhow::bail!("AFID.CLOZE_NESTED_UNSUPPORTED: nested cloze deletion");
            }
            if input[cursor..].starts_with("}}") {
                close_start = Some(cursor);
                break;
            }
            cursor = next_char_boundary(input, cursor);
        }
    }

    let close_start =
        close_start.ok_or_else(|| anyhow::anyhow!("AFID.CLOZE_MALFORMED: missing close"))?;
    let body_end = hint_start
        .map(|start| start - "::".len())
        .unwrap_or(close_start);
    if body_start == body_end {
        anyhow::bail!("AFID.CLOZE_MALFORMED: empty cloze body");
    }

    let body = normalize_field_text_for_identity(&input[body_start..body_end]);
    let next_cursor = close_start + "}}".len();

    Ok((ClozeSegment::Deletion { ord, body, slot }, next_cursor))
}

fn is_cloze_start_at(input: &str, cursor: usize) -> bool {
    input[cursor..].starts_with("{{c")
        && input
            .as_bytes()
            .get(cursor + "{{c".len())
            .is_some_and(u8::is_ascii_digit)
}

fn next_char_boundary(input: &str, cursor: usize) -> usize {
    input[cursor..]
        .chars()
        .next()
        .map(|ch| cursor + ch.len_utf8())
        .unwrap_or(input.len())
}

fn cloze_components_from_segments(segments: &[ClozeSegment]) -> anyhow::Result<ClozeComponents> {
    let mut text_skeleton = String::new();
    let mut deletions = Vec::new();

    for segment in segments {
        match segment {
            ClozeSegment::Text(text) => text_skeleton.push_str(&escape_cloze_skeleton_text(text)),
            ClozeSegment::Deletion { ord, body, slot } => {
                text_skeleton.push_str("[[CLOZE]]");
                deletions.push(ClozeDeletion {
                    ord: ord.get(),
                    body: body.clone(),
                    slot: *slot,
                });
            }
        }
    }

    if deletions.is_empty() {
        anyhow::bail!("AFID.IDENTITY_COMPONENT_EMPTY: cloze deletions");
    }

    Ok(ClozeComponents {
        text_skeleton,
        deletions,
    })
}

fn escape_cloze_skeleton_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace("[[CLOZE]]", "\\[[CLOZE]]")
}
