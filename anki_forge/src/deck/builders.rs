use crate::deck::model::{
    normalize_stable_id, BasicNote, ClozeNote, Deck, DeckNote, IoMode, IoNote, IoRect, MediaRef,
};
use crate::deck::validation::{ValidationCode, ValidationDiagnostic, ValidationReport};

pub struct DeckBuilder {
    name: String,
    stable_id: Option<String>,
}

impl DeckBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stable_id: None,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = normalize_stable_id(stable_id.into());
        self
    }

    pub fn build(self) -> Deck {
        Deck {
            name: self.name,
            stable_id: self.stable_id,
            notes: Vec::new(),
            next_generated_note_id: 1,
            media: Default::default(),
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
        validate_note_shape_before_insert(&note)?;
        self.notes.push(note);
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
    let requested = note.requested_stable_id().map(str::trim);

    match requested {
        Some("") => anyhow::bail!("stable_id must not be blank"),
        Some(stable_id) => {
            anyhow::ensure!(
                deck.notes.iter().all(|existing| existing.id() != stable_id),
                "duplicate stable_id: {}",
                stable_id,
            );
            note.assign_stable_id(stable_id.to_string());
        }
        None => {
            let generated = generate_unique_generated_id(deck);
            note.assign_generated_id(generated);
        }
    }

    Ok(())
}

fn validate_note_shape_before_insert(note: &DeckNote) -> anyhow::Result<()> {
    if let DeckNote::ImageOcclusion(io) = note {
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

        if deck.notes.iter().all(|existing| existing.id() != generated) {
            return generated;
        }
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
}
