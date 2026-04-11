use crate::deck::model::{normalize_stable_id, Deck, DeckNote};

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
    pub fn add(&mut self, note: impl Into<DeckNote>) -> anyhow::Result<()> {
        let mut note = note.into();
        assign_identity(self, &mut note)?;
        self.notes.push(note);
        Ok(())
    }
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

fn generate_unique_generated_id(deck: &mut Deck) -> String {
    loop {
        let generated = format!("generated:{}:{}", deck.name(), deck.next_generated_note_id);
        deck.next_generated_note_id += 1;

        if deck.notes.iter().all(|existing| existing.id() != generated) {
            return generated;
        }
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
