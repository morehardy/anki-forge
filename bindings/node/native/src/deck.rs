use crate::authoring::{IoModeInput, RectInput};
use crate::state::{ProjectTask, SharedProject};
use crate::{parse, reports, NativeProject};
use anki_forge::deck::{
    BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, ClozeNote, Deck,
    DeckError, MediaSource,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct DeckOptions {
    stable_id: Option<String>,
    basic_identity: Option<Vec<BasicIdentityField>>,
}
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct NoteOptions {
    stable_id: Option<String>,
    tags: Vec<String>,
    extra: Option<String>,
    identity_override: Option<OverrideInput>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OverrideInput {
    fields: Vec<BasicIdentityField>,
    reason_code: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IoOptions {
    stable_id: Option<String>,
    mode: IoModeInput,
    rects: Vec<RectInput>,
    tags: Vec<String>,
    header: Option<String>,
    back_extra: Option<String>,
    comments: Option<String>,
}

#[napi]
pub struct NativeDeck {
    inner: NativeProject,
}
#[napi]
impl NativeDeck {
    #[napi(constructor)]
    pub fn new(name: String, input: String) -> Result<Self> {
        let options: DeckOptions = parse(&input)?;
        let mut builder = Deck::builder(name);
        if let Some(id) = options.stable_id {
            builder = builder.stable_id(id);
        }
        if let Some(fields) = options.basic_identity {
            builder =
                builder.basic_identity(BasicIdentitySelection::new(fields).map_err(|error| {
                    Error::new(Status::InvalidArg, reports::domain_failure("deck", error))
                })?);
        }
        Ok(Self {
            inner: NativeProject {
                shared: SharedProject::new_deck(builder.build()),
            },
        })
    }
    #[napi]
    pub fn add_basic(&self, front: String, back: String, input: String) -> Result<String> {
        let options: NoteOptions = parse(&input)?;
        self.inner.shared.with_ready(|context| {
            let mut note = BasicNote::new(front, back).tags(options.tags);
            if let Some(id) = options.stable_id {
                note = note.stable_id(id);
            }
            if let Some(value) = options.identity_override {
                match BasicIdentityOverride::new(value.fields, value.reason_code) {
                    Ok(value) => note = note.identity_override(value),
                    Err(error) => return reports::domain_failure("deck", error),
                }
            }
            match context.deck.as_mut().expect("deck context").add(note) {
                Ok(_) => reports::success(serde_json::Value::Null),
                Err(error) => reports::domain_failure("deck", error),
            }
        })
    }
    #[napi]
    pub fn add_cloze(&self, text: String, input: String) -> Result<String> {
        let options: NoteOptions = parse(&input)?;
        self.inner.shared.with_ready(|context| {
            let mut note = ClozeNote::new(text)
                .tags(options.tags)
                .extra(options.extra.unwrap_or_default());
            if let Some(id) = options.stable_id {
                note = note.stable_id(id);
            }
            match context.deck.as_mut().expect("deck context").add(note) {
                Ok(_) => reports::success(serde_json::Value::Null),
                Err(error) => reports::domain_failure("deck", error),
            }
        })
    }
    #[napi]
    pub fn add_image_occlusion(&self, filename: String, input: String) -> Result<String> {
        let options: IoOptions = parse(&input)?;
        self.inner.shared.with_ready(|context| {
            let deck = context.deck.as_mut().expect("deck context");
            let Some(image) = deck.media().get(&filename) else {
                return reports::domain_failure(
                    "deck",
                    DeckError::ImageOcclusionUnknownMedia {
                        media_name: filename,
                    }
                    .into(),
                );
            };
            let mut draft = deck
                .image_occlusion()
                .note(image)
                .mode(options.mode.into())
                .tags(options.tags);
            if let Some(id) = options.stable_id {
                draft = draft.stable_id(id);
            }
            if let Some(value) = options.header {
                draft = draft.header(value);
            }
            if let Some(value) = options.back_extra {
                draft = draft.back_extra(value);
            }
            if let Some(value) = options.comments {
                draft = draft.comments(value);
            }
            for rect in options.rects {
                draft = draft.rect(rect.x, rect.y, rect.width, rect.height);
            }
            match draft.add() {
                Ok(_) => reports::success(serde_json::Value::Null),
                Err(error) => reports::domain_failure("deck", error),
            }
        })
    }
    #[napi]
    pub fn add_media_file(&self, path: String) -> Result<AsyncTask<ProjectTask>> {
        Ok(AsyncTask::new(ProjectTask::deck_media(
            self.inner.shared.reserve()?,
            MediaSource::from_file(path),
        )))
    }
    #[napi]
    pub fn add_media_bytes(&self, name: String, bytes: Buffer) -> Result<AsyncTask<ProjectTask>> {
        Ok(AsyncTask::new(ProjectTask::deck_media(
            self.inner.shared.reserve()?,
            MediaSource::from_bytes(name, bytes.to_vec()),
        )))
    }
    #[napi]
    pub fn validate(&self) -> Result<AsyncTask<ProjectTask>> {
        self.inner.validate()
    }
    #[napi]
    pub fn build(&self, input: String) -> Result<AsyncTask<ProjectTask>> {
        self.inner.build(input)
    }
    #[napi]
    pub fn apkg_bytes(&self) -> Result<AsyncTask<crate::state::BytesTask>> {
        self.inner.apkg_bytes()
    }
    #[napi]
    pub fn diff_against_apkg(
        &self,
        path: String,
        limits: String,
    ) -> Result<AsyncTask<ProjectTask>> {
        self.inner.diff_against_apkg(path, limits)
    }
}
