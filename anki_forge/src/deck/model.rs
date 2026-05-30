use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn canonicalize_fields<F, I>(fields: I) -> anyhow::Result<Vec<F>>
where
    F: Copy + Ord,
    I: IntoIterator<Item = F>,
{
    let mut values: Vec<F> = fields.into_iter().collect();
    values.sort();
    values.dedup();
    if values.is_empty() {
        return Err(DeckError::IdentityFieldsEmpty.into());
    }
    Ok(values)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasicIdentityField {
    Front,
    Back,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IdentitySelection<F> {
    fields: Vec<F>,
}

impl<F> IdentitySelection<F>
where
    F: Copy + Ord,
{
    pub fn new<I>(fields: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = F>,
    {
        Ok(Self {
            fields: canonicalize_fields(fields)?,
        })
    }

    pub fn as_slice(&self) -> &[F] {
        &self.fields
    }
}

impl<'de, F> Deserialize<'de> for IdentitySelection<F>
where
    F: Copy + Ord + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire<F> {
            fields: Vec<F>,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.fields).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IdentityOverride<F> {
    fields: Vec<F>,
    reason_code: String,
}

impl<F> IdentityOverride<F>
where
    F: Copy + Ord,
{
    pub fn new<I>(fields: I, reason_code: impl Into<String>) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = F>,
    {
        let reason_code = reason_code.into().trim().to_string();
        if reason_code.is_empty() {
            return Err(DeckError::NoteLevelIdentityOverrideReasonRequired.into());
        }
        Ok(Self {
            fields: canonicalize_fields(fields)?,
            reason_code,
        })
    }

    pub fn fields(&self) -> &[F] {
        &self.fields
    }

    pub fn reason_code(&self) -> &str {
        &self.reason_code
    }
}

impl<'de, F> Deserialize<'de> for IdentityOverride<F>
where
    F: Copy + Ord + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire<F> {
            fields: Vec<F>,
            reason_code: String,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.fields, wire.reason_code).map_err(serde::de::Error::custom)
    }
}

pub type BasicIdentitySelection = IdentitySelection<BasicIdentityField>;
pub type BasicIdentityOverride = IdentityOverride<BasicIdentityField>;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeckIdentityPolicy {
    pub basic: Option<BasicIdentitySelection>,
}

fn is_default_identity_policy(policy: &DeckIdentityPolicy) -> bool {
    policy == &DeckIdentityPolicy::default()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityProvenance {
    ExplicitStableId,
    InferredFromNoteFields,
    InferredFromNotetypeFields,
    InferredFromStockRecipe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedIdentitySnapshot {
    pub stable_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    pub provenance: IdentityProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_payload: Option<String>,
    #[serde(default)]
    pub used_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeckError {
    StableIdBlank,
    StableIdDuplicate {
        stable_id: String,
    },
    ReservedAfidNamespace {
        stable_id: String,
    },
    IdentitySnapshotMissing {
        stable_id: String,
    },
    IdentitySnapshotNoteIdMismatch {
        snapshot_stable_id: String,
        note_id: String,
    },
    IdentitySnapshotHashMismatch {
        stable_id: String,
    },
    IdentitySnapshotIncomplete {
        stable_id: String,
    },
    IdentityDuplicatePayload {
        stable_id: String,
    },
    IdentityCollision {
        stable_id: String,
    },
    IdentityFieldsEmpty,
    IdentityComponentEmpty {
        component: String,
    },
    NoteLevelIdentityOverrideReasonRequired,
    ClozeMalformed {
        message: String,
    },
    ClozeOrdInvalid {
        message: String,
    },
    ClozeNestedUnsupported,
    ImageOcclusionImageDimensionsMissing,
    ImageOcclusionRectEmpty,
    ImageOcclusionRectOutOfBounds,
    ImageOcclusionRectDuplicate,
    ImageOcclusionUnknownMedia {
        media_name: String,
    },
    ImageOcclusionEmptyMasks,
    ValidationFailed {
        code: crate::diagnostics::ErrorCode,
        message: String,
    },
}

impl DeckError {
    pub fn code(&self) -> crate::diagnostics::ErrorCode {
        match self {
            Self::StableIdBlank => crate::diagnostics::ErrorCode::StableIdBlank,
            Self::StableIdDuplicate { .. } => crate::diagnostics::ErrorCode::StableIdDuplicate,
            Self::ReservedAfidNamespace { .. } => {
                crate::diagnostics::ErrorCode::ReservedAfidNamespace
            }
            Self::IdentitySnapshotIncomplete { .. } => {
                crate::diagnostics::ErrorCode::IdentitySnapshotIncomplete
            }
            Self::IdentitySnapshotMissing { .. } => {
                crate::diagnostics::ErrorCode::IdentitySnapshotMissing
            }
            Self::IdentitySnapshotNoteIdMismatch { .. } => {
                crate::diagnostics::ErrorCode::IdentitySnapshotNoteIdMismatch
            }
            Self::IdentitySnapshotHashMismatch { .. } => {
                crate::diagnostics::ErrorCode::IdentitySnapshotHashMismatch
            }
            Self::IdentityDuplicatePayload { .. } => {
                crate::diagnostics::ErrorCode::IdentityDuplicatePayload
            }
            Self::IdentityCollision { .. } => crate::diagnostics::ErrorCode::IdentityCollision,
            Self::IdentityFieldsEmpty => crate::diagnostics::ErrorCode::IdentityFieldsEmpty,
            Self::IdentityComponentEmpty { component } => match component.as_str() {
                "io rects" => crate::diagnostics::ErrorCode::ImageOcclusionEmptyMasks,
                "missing io media" => crate::diagnostics::ErrorCode::ImageOcclusionUnknownMedia,
                _ => crate::diagnostics::ErrorCode::IdentityComponentEmpty,
            },
            Self::NoteLevelIdentityOverrideReasonRequired => {
                crate::diagnostics::ErrorCode::NoteLevelIdentityOverrideReasonRequired
            }
            Self::ClozeMalformed { .. } => crate::diagnostics::ErrorCode::ClozeMalformed,
            Self::ClozeOrdInvalid { .. } => crate::diagnostics::ErrorCode::ClozeOrdInvalid,
            Self::ClozeNestedUnsupported => crate::diagnostics::ErrorCode::ClozeNestedUnsupported,
            Self::ImageOcclusionImageDimensionsMissing => {
                crate::diagnostics::ErrorCode::ImageOcclusionImageDimensionsMissing
            }
            Self::ImageOcclusionRectEmpty => crate::diagnostics::ErrorCode::ImageOcclusionRectEmpty,
            Self::ImageOcclusionRectOutOfBounds => {
                crate::diagnostics::ErrorCode::ImageOcclusionRectOutOfBounds
            }
            Self::ImageOcclusionRectDuplicate => {
                crate::diagnostics::ErrorCode::ImageOcclusionRectDuplicate
            }
            Self::ImageOcclusionUnknownMedia { .. } => {
                crate::diagnostics::ErrorCode::ImageOcclusionUnknownMedia
            }
            Self::ImageOcclusionEmptyMasks => {
                crate::diagnostics::ErrorCode::ImageOcclusionEmptyMasks
            }
            Self::ValidationFailed { code, .. } => code.clone(),
        }
    }

    pub fn stable_id(&self) -> &str {
        match self {
            Self::StableIdBlank => "",
            Self::StableIdDuplicate { stable_id }
            | Self::ReservedAfidNamespace { stable_id }
            | Self::IdentitySnapshotMissing { stable_id }
            | Self::IdentitySnapshotHashMismatch { stable_id }
            | Self::IdentitySnapshotIncomplete { stable_id }
            | Self::IdentityDuplicatePayload { stable_id }
            | Self::IdentityCollision { stable_id } => stable_id,
            Self::IdentitySnapshotNoteIdMismatch {
                snapshot_stable_id, ..
            } => snapshot_stable_id,
            Self::IdentityFieldsEmpty => "",
            Self::IdentityComponentEmpty { component } => component,
            Self::ClozeMalformed { message } | Self::ClozeOrdInvalid { message } => message,
            Self::NoteLevelIdentityOverrideReasonRequired
            | Self::ClozeNestedUnsupported
            | Self::ImageOcclusionImageDimensionsMissing
            | Self::ImageOcclusionRectEmpty
            | Self::ImageOcclusionRectOutOfBounds
            | Self::ImageOcclusionRectDuplicate
            | Self::ImageOcclusionEmptyMasks
            | Self::ValidationFailed { .. } => "",
            Self::ImageOcclusionUnknownMedia { media_name } => media_name,
        }
    }
}

impl std::fmt::Display for DeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StableIdBlank => write!(f, "{}: stable_id must not be blank", self.code()),
            Self::ReservedAfidNamespace { stable_id } => write!(
                f,
                "{}: explicit stable_id cannot use reserved AFID namespace: {}",
                self.code(),
                stable_id
            ),
            Self::IdentitySnapshotNoteIdMismatch {
                snapshot_stable_id,
                note_id,
            } => write!(f, "{}: {} != {}", self.code(), snapshot_stable_id, note_id),
            Self::ImageOcclusionRectEmpty
            | Self::ImageOcclusionRectOutOfBounds
            | Self::ImageOcclusionRectDuplicate
            | Self::ImageOcclusionImageDimensionsMissing
            | Self::IdentityFieldsEmpty
            | Self::NoteLevelIdentityOverrideReasonRequired
            | Self::ClozeNestedUnsupported => write!(f, "{}", self.code()),
            Self::ImageOcclusionUnknownMedia { media_name } => {
                write!(f, "{}: unknown media {}", self.code(), media_name)
            }
            Self::ImageOcclusionEmptyMasks => {
                write!(
                    f,
                    "{}: image occlusion note requires at least one rect",
                    self.code()
                )
            }
            Self::ValidationFailed { code, message } => write!(f, "{code}: {message}"),
            _ => write!(f, "{}: {}", self.code(), self.stable_id()),
        }
    }
}

impl std::error::Error for DeckError {}

impl crate::diagnostics::ErrorCodeExt for DeckError {
    fn code(&self) -> crate::diagnostics::ErrorCode {
        DeckError::code(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub(crate) name: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) identity_policy: DeckIdentityPolicy,
    pub(crate) notes: Vec<DeckNote>,
    pub(crate) next_generated_note_id: u64,
    pub(crate) media: BTreeMap<String, RegisteredMedia>,
    pub(crate) used_note_ids: BTreeSet<String>,
    pub(crate) identity_snapshot_by_id: BTreeMap<String, ResolvedIdentitySnapshot>,
}

#[derive(Serialize, Deserialize)]
struct PersistedDeck {
    name: String,
    stable_id: Option<String>,
    #[serde(default, skip_serializing_if = "is_default_identity_policy")]
    identity_policy: DeckIdentityPolicy,
    notes: Vec<DeckNote>,
    next_generated_note_id: u64,
    media: BTreeMap<String, RegisteredMedia>,
}

impl Serialize for Deck {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        PersistedDeck {
            name: self.name.clone(),
            stable_id: self.stable_id.clone(),
            identity_policy: self.identity_policy.clone(),
            notes: self.notes.clone(),
            next_generated_note_id: self.next_generated_note_id,
            media: self.media.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Deck {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let persisted = PersistedDeck::deserialize(deserializer)?;
        let mut deck = Self {
            name: persisted.name,
            stable_id: persisted.stable_id,
            identity_policy: persisted.identity_policy,
            notes: persisted.notes,
            next_generated_note_id: persisted.next_generated_note_id,
            media: persisted.media,
            used_note_ids: BTreeSet::new(),
            identity_snapshot_by_id: BTreeMap::new(),
        };
        deck.rebuild_runtime_indexes()
            .map_err(serde::de::Error::custom)?;
        Ok(deck)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub(crate) root_deck: Deck,
    pub(crate) stable_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckNote {
    Basic(BasicNote),
    Cloze(ClozeNote),
    ImageOcclusion(IoNote),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) identity_override: Option<BasicIdentityOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_identity: Option<ResolvedIdentitySnapshot>,
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_identity: Option<ResolvedIdentitySnapshot>,
    pub(crate) text: String,
    pub(crate) extra: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IoRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IoNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_identity: Option<ResolvedIdentitySnapshot>,
    pub(crate) image: MediaRef,
    pub(crate) mode: IoMode,
    pub(crate) rects: Vec<IoRect>,
    pub(crate) header: String,
    pub(crate) back_extra: String,
    pub(crate) comments: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoMode {
    HideAllGuessOne,
    HideOneGuessOne,
}

/// Deck-scoped media handle used by the Deck image-occlusion API.
///
/// This is distinct from [`crate::product::MediaRef`], which is used by the
/// Project facade and is exported through [`crate::prelude`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaRef(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterImageMetadata {
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegisteredMedia {
    pub(crate) name: String,
    pub(crate) source: RegisteredMediaSource,
    pub(crate) declared_mime: Option<String>,
    pub(crate) sha1_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) raster_image: Option<RasterImageMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RegisteredMediaSource {
    File { path: PathBuf },
    InlineBytes { data_base64: String },
}

impl<'de> Deserialize<'de> for RegisteredMedia {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Current {
                name: String,
                source: RegisteredMediaSource,
                #[serde(default)]
                declared_mime: Option<String>,
                sha1_hex: String,
                #[serde(default)]
                raster_image: Option<RasterImageMetadata>,
            },
            LegacyInline {
                name: String,
                #[serde(default)]
                mime: Option<String>,
                data_base64: String,
                sha1_hex: String,
                #[serde(default)]
                raster_image: Option<RasterImageMetadata>,
            },
        }

        match Wire::deserialize(deserializer)? {
            Wire::Current {
                name,
                source,
                declared_mime,
                sha1_hex,
                raster_image,
            } => Ok(Self {
                name,
                source,
                declared_mime,
                sha1_hex,
                raster_image,
            }),
            Wire::LegacyInline {
                name,
                mime,
                data_base64,
                sha1_hex,
                raster_image,
            } => {
                let declared_mime = Some(mime.unwrap_or_else(|| legacy_mime_from_name(&name)));
                Ok(Self {
                    name,
                    source: RegisteredMediaSource::InlineBytes { data_base64 },
                    declared_mime,
                    sha1_hex,
                    raster_image,
                })
            }
        }
    }
}

fn legacy_mime_from_name(name: &str) -> String {
    authoring_core::mime_from_filename_or_octet(name)
}

impl Deck {
    pub fn builder(name: impl Into<String>) -> crate::deck::builders::DeckBuilder {
        crate::deck::builders::DeckBuilder::new(name)
    }

    pub fn new(name: impl Into<String>) -> Self {
        crate::deck::builders::DeckBuilder::new(name).build()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn stable_id(&self) -> Option<&str> {
        self.stable_id.as_deref()
    }

    pub fn identity_policy(&self) -> &DeckIdentityPolicy {
        &self.identity_policy
    }

    pub fn notes(&self) -> &[DeckNote] {
        &self.notes
    }
}

impl Package {
    pub fn single(root_deck: Deck) -> Self {
        let stable_id = root_deck.stable_id.clone();
        Self {
            root_deck,
            stable_id,
        }
    }

    pub fn with_stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = normalize_stable_id(stable_id.into());
        self
    }

    pub fn root_deck(&self) -> &Deck {
        &self.root_deck
    }

    pub fn stable_id(&self) -> Option<&str> {
        self.stable_id.as_deref()
    }
}

impl BasicNote {
    pub fn new(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            stable_id: None,
            identity_override: None,
            resolved_identity: None,
            front: front.into(),
            back: back.into(),
            tags: Vec::new(),
            generated: false,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn tags<T, I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn identity_override(mut self, override_cfg: BasicIdentityOverride) -> Self {
        self.identity_override = Some(override_cfg);
        self
    }

    pub fn identity_override_config(&self) -> Option<&BasicIdentityOverride> {
        self.identity_override.as_ref()
    }
}

impl ClozeNote {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            stable_id: None,
            resolved_identity: None,
            text: text.into(),
            extra: String::new(),
            tags: Vec::new(),
            generated: false,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn extra(mut self, extra: impl Into<String>) -> Self {
        self.extra = extra.into();
        self
    }

    pub fn tags<T, I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }
}

impl From<BasicNote> for DeckNote {
    fn from(note: BasicNote) -> Self {
        Self::Basic(note)
    }
}

impl From<ClozeNote> for DeckNote {
    fn from(note: ClozeNote) -> Self {
        Self::Cloze(note)
    }
}

impl From<IoNote> for DeckNote {
    fn from(note: IoNote) -> Self {
        Self::ImageOcclusion(note)
    }
}

impl IoNote {
    pub(crate) fn new(image: MediaRef) -> Self {
        Self {
            id: String::new(),
            stable_id: None,
            resolved_identity: None,
            image,
            mode: IoMode::HideAllGuessOne,
            rects: Vec::new(),
            header: String::new(),
            back_extra: String::new(),
            comments: String::new(),
            tags: Vec::new(),
            generated: false,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }
}

pub(crate) fn normalize_stable_id(stable_id: String) -> Option<String> {
    let trimmed = stable_id.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
