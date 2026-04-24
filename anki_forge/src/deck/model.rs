use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

fn canonicalize_fields<F, I>(fields: I) -> anyhow::Result<Vec<F>>
where
    F: Copy + Ord,
    I: IntoIterator<Item = F>,
{
    let mut values: Vec<F> = fields.into_iter().collect();
    values.sort();
    values.dedup();
    anyhow::ensure!(!values.is_empty(), "AFID.IDENTITY_FIELDS_EMPTY");
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
        anyhow::ensure!(
            !reason_code.is_empty(),
            "AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED"
        );
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaRef(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterImageMetadata {
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredMedia {
    pub(crate) name: String,
    pub(crate) mime: String,
    pub(crate) data_base64: String,
    pub(crate) sha1_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) raster_image: Option<RasterImageMetadata>,
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
