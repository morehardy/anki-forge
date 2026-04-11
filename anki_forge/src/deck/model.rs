use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck {
    pub(crate) name: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) notes: Vec<DeckNote>,
    pub(crate) next_generated_note_id: u64,
    pub(crate) media: BTreeMap<String, RegisteredMedia>,
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
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
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
pub struct MediaRef(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredMedia {
    pub(crate) name: String,
    pub(crate) mime: String,
    pub(crate) data_base64: String,
    pub(crate) sha1_hex: String,
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
        self.stable_id = Some(stable_id.into());
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
}

impl ClozeNote {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            stable_id: None,
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
