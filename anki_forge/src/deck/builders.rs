use crate::deck::identity::resolve_inferred_identity;
use crate::deck::media::backfill_missing_raster_image_metadata;
use crate::deck::model::{
    normalize_stable_id, BasicIdentityOverride, BasicIdentitySelection, BasicNote, ClozeNote, Deck,
    DeckIdentityPolicy, DeckNote, IdentityProvenance, IoMode, IoNote, IoRect, MediaRef,
    ResolvedIdentitySnapshot,
};
use crate::deck::validation::{ValidationCode, ValidationDiagnostic, ValidationReport};

pub struct DeckBuilder {
    name: String,
    stable_id: Option<String>,
    identity_policy: DeckIdentityPolicy,
}

impl DeckBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stable_id: None,
            identity_policy: DeckIdentityPolicy::default(),
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = normalize_stable_id(stable_id.into());
        self
    }

    pub fn basic_identity(mut self, selection: BasicIdentitySelection) -> Self {
        self.identity_policy.basic = Some(selection);
        self
    }

    pub fn build(self) -> Deck {
        Deck {
            name: self.name,
            stable_id: self.stable_id,
            identity_policy: self.identity_policy,
            notes: Vec::new(),
            next_generated_note_id: 1,
            media: Default::default(),
            used_note_ids: Default::default(),
            identity_snapshot_by_id: Default::default(),
        }
    }
}

impl Deck {
    pub fn add_basic(
        &mut self,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.add(BasicNote::new(front, back))
    }

    pub fn basic(&mut self) -> BasicLane<'_> {
        BasicLane { deck: self }
    }

    pub fn cloze(&mut self) -> ClozeLane<'_> {
        ClozeLane { deck: self }
    }

    pub fn image_occlusion(&mut self) -> IoLane<'_> {
        IoLane { deck: self }
    }

    pub fn validate_report(&self) -> anyhow::Result<ValidationReport> {
        let mut diagnostics = Vec::new();
        let mut seen_ids = std::collections::BTreeSet::new();

        for note in &self.notes {
            match note.requested_stable_id().map(str::trim) {
                Some("") => diagnostics.push(ValidationDiagnostic {
                    code: ValidationCode::BlankStableId,
                    message: format!("note '{}' has a blank explicit stable_id", note.id()),
                    severity: "error".into(),
                }),
                None if note.generated() => diagnostics.push(ValidationDiagnostic {
                    code: ValidationCode::MissingStableId,
                    message: format!("note '{}' was assigned a generated id", note.id()),
                    severity: "warning".into(),
                }),
                None => {}
                Some(_) => {}
            }

            if !seen_ids.insert(note.id().to_string()) {
                diagnostics.push(ValidationDiagnostic {
                    code: ValidationCode::DuplicateStableId,
                    message: format!("id '{}' is duplicated", note.id()),
                    severity: "error".into(),
                });
            }

            if let DeckNote::ImageOcclusion(io) = note {
                if !self.media.contains_key(io.image.name()) {
                    diagnostics.push(ValidationDiagnostic {
                        code: ValidationCode::UnknownMediaRef,
                        message: format!(
                            "image occlusion note '{}' references unknown media '{}'",
                            io.id,
                            io.image.name()
                        ),
                        severity: "error".into(),
                    });
                }

                if io.rects.is_empty() {
                    diagnostics.push(ValidationDiagnostic {
                        code: ValidationCode::EmptyIoMasks,
                        message: format!(
                            "image occlusion note '{}' requires at least one rect",
                            io.id
                        ),
                        severity: "error".into(),
                    });
                }
            }
        }

        Ok(ValidationReport::new(diagnostics))
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let report = self.validate_report()?;
        anyhow::ensure!(!report.has_errors(), "deck validation failed");
        Ok(())
    }

    pub fn add(&mut self, note: impl Into<DeckNote>) -> anyhow::Result<()> {
        let mut note = note.into();
        assign_identity(self, &mut note)?;
        validate_note_shape_before_insert(self, &note)?;
        if let Some(snapshot) = note.resolved_identity_snapshot() {
            insert_identity_snapshot(&mut self.identity_snapshot_by_id, snapshot.clone())?;
        }
        anyhow::ensure!(
            !self.used_note_ids.contains(note.id()),
            "AFID.STABLE_ID_DUPLICATE: {}",
            note.id(),
        );
        self.used_note_ids.insert(note.id().to_string());
        self.notes.push(note);
        Ok(())
    }

    pub(crate) fn rebuild_runtime_indexes(&mut self) -> anyhow::Result<()> {
        self.used_note_ids.clear();
        self.identity_snapshot_by_id.clear();
        backfill_missing_raster_image_metadata(&mut self.media);

        for note in &mut self.notes {
            self.used_note_ids.insert(note.id().to_string());
            validate_requested_stable_id_namespace(note)?;

            if let Some(snapshot) = note.resolved_identity_snapshot().cloned() {
                validate_snapshot_for_note(note, &snapshot)?;
                insert_identity_snapshot(&mut self.identity_snapshot_by_id, snapshot.clone())?;
                validate_snapshot_hash(&snapshot)?;
                continue;
            }

            if note.id().starts_with("afid:v1:") {
                anyhow::bail!("AFID.IDENTITY_SNAPSHOT_MISSING: {}", note.id());
            }

            if note.id().starts_with("generated:") {
                continue;
            }
        }

        Ok(())
    }
}

pub struct BasicLane<'a> {
    deck: &'a mut Deck,
}

pub struct ClozeLane<'a> {
    deck: &'a mut Deck,
}

pub struct IoLane<'a> {
    deck: &'a mut Deck,
}

pub struct BasicDraft<'a> {
    deck: &'a mut Deck,
    note: BasicNote,
}

pub struct ClozeDraft<'a> {
    deck: &'a mut Deck,
    note: ClozeNote,
}

pub struct IoDraft<'a> {
    deck: &'a mut Deck,
    note: IoNote,
}

fn assign_identity(deck: &mut Deck, note: &mut DeckNote) -> anyhow::Result<()> {
    ensure_note_id_index(deck);
    let requested = note
        .requested_stable_id()
        .map(|stable_id| stable_id.trim().to_string());

    match requested.as_deref() {
        Some("") => anyhow::bail!("stable_id must not be blank"),
        Some(stable_id) => {
            anyhow::ensure!(
                !stable_id.starts_with("afid:v1:"),
                "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: explicit stable_id cannot use reserved AFID namespace: {}",
                stable_id,
            );
            anyhow::ensure!(
                !deck.used_note_ids.contains(stable_id),
                "AFID.STABLE_ID_DUPLICATE: {}",
                stable_id,
            );
            note.assign_stable_id(stable_id.to_string());
            note.assign_resolved_identity(ResolvedIdentitySnapshot {
                stable_id: stable_id.to_string(),
                recipe_id: None,
                provenance: IdentityProvenance::ExplicitStableId,
                canonical_payload: None,
                used_override: false,
            });
        }
        None => {
            if matches!(
                note,
                DeckNote::Basic(_) | DeckNote::Cloze(_) | DeckNote::ImageOcclusion(_)
            ) {
                let resolved = resolve_inferred_identity(deck, note)?;
                note.assign_inferred_id(resolved.stable_id.clone());
                note.assign_resolved_identity(ResolvedIdentitySnapshot {
                    stable_id: resolved.stable_id,
                    recipe_id: Some(resolved.recipe_id),
                    provenance: resolved.provenance,
                    canonical_payload: Some(resolved.canonical_payload),
                    used_override: resolved.used_override,
                });
            } else {
                let generated = generate_unique_generated_id(deck);
                note.assign_generated_id(generated);
            }
        }
    }

    Ok(())
}

fn validate_requested_stable_id_namespace(note: &DeckNote) -> anyhow::Result<()> {
    if let Some(stable_id) = note.requested_stable_id().map(str::trim) {
        anyhow::ensure!(
            !stable_id.starts_with("afid:v1:"),
            "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: explicit stable_id cannot use reserved AFID namespace: {}",
            stable_id
        );
    }

    Ok(())
}

fn validate_snapshot_for_note(
    note: &DeckNote,
    snapshot: &ResolvedIdentitySnapshot,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        snapshot.stable_id == note.id(),
        "AFID.IDENTITY_SNAPSHOT_NOTE_ID_MISMATCH: {} != {}",
        snapshot.stable_id,
        note.id()
    );

    match snapshot.provenance {
        IdentityProvenance::ExplicitStableId => validate_explicit_snapshot_shape(note, snapshot),
        IdentityProvenance::InferredFromNoteFields
        | IdentityProvenance::InferredFromNotetypeFields
        | IdentityProvenance::InferredFromStockRecipe => {
            validate_inferred_snapshot_shape(note, snapshot)
        }
    }
}

fn validate_snapshot_hash(snapshot: &ResolvedIdentitySnapshot) -> anyhow::Result<()> {
    if let Some(canonical_payload) = &snapshot.canonical_payload {
        let expected = format!("afid:v1:{}", blake3::hash(canonical_payload.as_bytes()));
        anyhow::ensure!(
            snapshot.stable_id == expected,
            "AFID.IDENTITY_SNAPSHOT_HASH_MISMATCH: {}",
            snapshot.stable_id
        );
    }

    Ok(())
}

fn validate_explicit_snapshot_shape(
    note: &DeckNote,
    snapshot: &ResolvedIdentitySnapshot,
) -> anyhow::Result<()> {
    let requested = note.requested_stable_id().map(str::trim);
    anyhow::ensure!(
        matches!(requested, Some(stable_id) if !stable_id.is_empty() && stable_id == note.id() && stable_id == snapshot.stable_id),
        "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: {}",
        snapshot.stable_id
    );
    anyhow::ensure!(
        !snapshot.stable_id.starts_with("afid:v1:"),
        "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: explicit provenance cannot use reserved AFID namespace: {}",
        snapshot.stable_id
    );
    anyhow::ensure!(
        snapshot.recipe_id.is_none() && snapshot.canonical_payload.is_none(),
        "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: {}",
        snapshot.stable_id
    );

    Ok(())
}

fn validate_inferred_snapshot_shape(
    note: &DeckNote,
    snapshot: &ResolvedIdentitySnapshot,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        note.requested_stable_id().is_none(),
        "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: {}",
        snapshot.stable_id
    );
    anyhow::ensure!(
        snapshot.recipe_id.is_some() && snapshot.canonical_payload.is_some(),
        "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: {}",
        snapshot.stable_id
    );
    anyhow::ensure!(
        snapshot
            .recipe_id
            .as_deref()
            .is_some_and(|recipe_id| !recipe_id.trim().is_empty()),
        "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: {}",
        snapshot.stable_id
    );
    anyhow::ensure!(
        snapshot
            .canonical_payload
            .as_deref()
            .is_some_and(|payload| !payload.is_empty()),
        "AFID.IDENTITY_SNAPSHOT_INCOMPLETE: {}",
        snapshot.stable_id
    );

    Ok(())
}

fn insert_identity_snapshot(
    snapshots: &mut std::collections::BTreeMap<String, ResolvedIdentitySnapshot>,
    snapshot: ResolvedIdentitySnapshot,
) -> anyhow::Result<()> {
    if let Some(existing) = snapshots.get(&snapshot.stable_id) {
        let code = match (
            existing.canonical_payload.as_deref(),
            snapshot.canonical_payload.as_deref(),
        ) {
            (Some(existing_payload), Some(new_payload)) if existing_payload == new_payload => {
                "AFID.IDENTITY_DUPLICATE_PAYLOAD"
            }
            (Some(_), Some(_)) => "AFID.IDENTITY_COLLISION",
            _ => "AFID.STABLE_ID_DUPLICATE",
        };
        anyhow::bail!("{}: {}", code, snapshot.stable_id);
    }

    snapshots.insert(snapshot.stable_id.clone(), snapshot);
    Ok(())
}

fn validate_note_shape_before_insert(deck: &Deck, note: &DeckNote) -> anyhow::Result<()> {
    if let DeckNote::ImageOcclusion(io) = note {
        anyhow::ensure!(
            deck.media.contains_key(io.image.name()),
            "image occlusion note references unknown media '{}'",
            io.image.name()
        );
        anyhow::ensure!(
            !io.rects.is_empty(),
            "image occlusion note requires at least one rect"
        );
    }
    Ok(())
}

fn generate_unique_generated_id(deck: &mut Deck) -> String {
    loop {
        let generated = format!("generated:{}:{}", deck.name(), deck.next_generated_note_id);
        deck.next_generated_note_id += 1;

        if !deck.used_note_ids.contains(&generated) {
            return generated;
        }
    }
}

fn ensure_note_id_index(deck: &mut Deck) {
    if deck.used_note_ids.is_empty() && !deck.notes.is_empty() {
        deck.used_note_ids = deck
            .notes
            .iter()
            .map(|note| note.id().to_string())
            .collect();
    }
}

impl<'a> BasicLane<'a> {
    pub fn note(self, front: impl Into<String>, back: impl Into<String>) -> BasicDraft<'a> {
        BasicDraft {
            deck: self.deck,
            note: BasicNote::new(front, back),
        }
    }
}

impl<'a> BasicDraft<'a> {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.note = self.note.stable_id(stable_id);
        self
    }

    pub fn tags<T, I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.note = self.note.tags(tags);
        self
    }

    pub fn identity_override(mut self, override_cfg: BasicIdentityOverride) -> Self {
        self.note = self.note.identity_override(override_cfg);
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        self.deck.add(self.note)
    }
}

impl<'a> ClozeLane<'a> {
    pub fn note(self, text: impl Into<String>) -> ClozeDraft<'a> {
        ClozeDraft {
            deck: self.deck,
            note: ClozeNote::new(text),
        }
    }
}

impl<'a> ClozeDraft<'a> {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.note = self.note.stable_id(stable_id);
        self
    }

    pub fn extra(mut self, extra: impl Into<String>) -> Self {
        self.note = self.note.extra(extra);
        self
    }

    pub fn tags<T, I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.note = self.note.tags(tags);
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        self.deck.add(self.note)
    }
}

impl<'a> IoLane<'a> {
    pub fn note(self, image: MediaRef) -> IoDraft<'a> {
        IoDraft {
            deck: self.deck,
            note: IoNote::new(image),
        }
    }
}

impl<'a> IoDraft<'a> {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.note = self.note.stable_id(stable_id);
        self
    }

    pub fn mode(mut self, mode: IoMode) -> Self {
        self.note.mode = mode;
        self
    }

    pub fn rect(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.note.rects.push(IoRect {
            x,
            y,
            width,
            height,
        });
        self
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.note.header = header.into();
        self
    }

    pub fn back_extra(mut self, back_extra: impl Into<String>) -> Self {
        self.note.back_extra = back_extra.into();
        self
    }

    pub fn comments(mut self, comments: impl Into<String>) -> Self {
        self.note.comments = comments.into();
        self
    }

    pub fn tags<T, I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.note.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        self.deck.add(self.note)
    }
}

impl DeckNote {
    pub fn id(&self) -> &str {
        match self {
            Self::Basic(note) => &note.id,
            Self::Cloze(note) => &note.id,
            Self::ImageOcclusion(note) => &note.id,
        }
    }

    pub(crate) fn requested_stable_id(&self) -> Option<&str> {
        match self {
            Self::Basic(note) => note.stable_id.as_deref(),
            Self::Cloze(note) => note.stable_id.as_deref(),
            Self::ImageOcclusion(note) => note.stable_id.as_deref(),
        }
    }

    pub(crate) fn generated(&self) -> bool {
        match self {
            Self::Basic(note) => note.generated,
            Self::Cloze(note) => note.generated,
            Self::ImageOcclusion(note) => note.generated,
        }
    }

    pub(crate) fn assign_stable_id(&mut self, stable_id: String) {
        match self {
            Self::Basic(note) => {
                note.id = stable_id.clone();
                note.stable_id = Some(stable_id);
                note.generated = false;
            }
            Self::Cloze(note) => {
                note.id = stable_id.clone();
                note.stable_id = Some(stable_id);
                note.generated = false;
            }
            Self::ImageOcclusion(note) => {
                note.id = stable_id.clone();
                note.stable_id = Some(stable_id);
                note.generated = false;
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn assign_inferred_id(&mut self, id: String) {
        match self {
            Self::Basic(note) => {
                note.id = id;
                note.stable_id = None;
                note.generated = false;
            }
            Self::Cloze(note) => {
                note.id = id;
                note.stable_id = None;
                note.generated = false;
            }
            Self::ImageOcclusion(note) => {
                note.id = id;
                note.stable_id = None;
                note.generated = false;
            }
        }
    }

    pub(crate) fn assign_generated_id(&mut self, id: String) {
        match self {
            Self::Basic(note) => {
                note.id = id;
                note.stable_id = None;
                note.generated = true;
            }
            Self::Cloze(note) => {
                note.id = id;
                note.stable_id = None;
                note.generated = true;
            }
            Self::ImageOcclusion(note) => {
                note.id = id;
                note.stable_id = None;
                note.generated = true;
            }
        }
    }

    pub(crate) fn resolved_identity_snapshot(&self) -> Option<&ResolvedIdentitySnapshot> {
        match self {
            Self::Basic(note) => note.resolved_identity.as_ref(),
            Self::Cloze(note) => note.resolved_identity.as_ref(),
            Self::ImageOcclusion(note) => note.resolved_identity.as_ref(),
        }
    }

    pub fn resolved_identity(&self) -> Option<&ResolvedIdentitySnapshot> {
        self.resolved_identity_snapshot()
    }

    pub(crate) fn assign_resolved_identity(&mut self, snapshot: ResolvedIdentitySnapshot) {
        match self {
            Self::Basic(note) => note.resolved_identity = Some(snapshot),
            Self::Cloze(note) => note.resolved_identity = Some(snapshot),
            Self::ImageOcclusion(note) => note.resolved_identity = Some(snapshot),
        }
    }
}
