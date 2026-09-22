use crate::{
    artifacts::NativeArtifact, domain_error, media, options, reports, state::ObjectState,
    NativeProject,
};
use anki_forge::build::json_report::DiagnosticJson;
use anki_forge::deck::{
    BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, ClozeNote, Deck,
    DeckError, MediaSource,
};
use anki_forge::diagnostics::{Diagnostic, DiagnosticCode};
use anki_forge::prelude::Project;
use pyo3::{exceptions::PyValueError, prelude::*};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

fn deck_error(error: anyhow::Error) -> PyErr {
    let code = error
        .downcast_ref::<DeckError>()
        .map(|error| error.code().to_string())
        .unwrap_or_else(|| "BINDING.DECK_FAILED".into());
    domain_error("deck", &code, &error.to_string(), Value::Null)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoteOptions {
    stable_id: Option<String>,
    tags: Vec<String>,
    extra: Option<String>,
    identity_override: Option<OverrideInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideInput {
    fields: Vec<BasicIdentityField>,
    reason_code: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IoOptions {
    stable_id: Option<String>,
    mode: crate::authoring::IoModeInput,
    rects: Vec<(u32, u32, u32, u32)>,
    tags: Vec<String>,
    header: String,
    back_extra: String,
    comments: String,
}

#[pyclass(frozen, module = "anki_forge._native")]
pub struct NativeDeckMediaRef {
    inner: anki_forge::deck::model::MediaRef,
}

#[pymethods]
impl NativeDeckMediaRef {
    #[getter]
    fn filename(&self) -> &str {
        self.inner.name()
    }
}

#[pyclass(frozen, module = "anki_forge._native")]
pub struct NativeDeck {
    state: ObjectState<Deck>,
}

#[pymethods]
impl NativeDeck {
    #[new]
    fn new(name: String, options: &str) -> PyResult<Self> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Options {
            stable_id: Option<String>,
            basic_identity: Option<Vec<BasicIdentityField>>,
        }
        let options: Options = serde_json::from_str(options)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let mut builder = Deck::builder(name);
        if let Some(id) = options.stable_id {
            builder = builder.stable_id(id);
        }
        if let Some(fields) = options.basic_identity {
            builder =
                builder.basic_identity(BasicIdentitySelection::new(fields).map_err(deck_error)?);
        }
        Ok(Self {
            state: ObjectState::new(builder.build(), "DECK"),
        })
    }

    fn add_basic(
        &self,
        py: Python<'_>,
        front: String,
        back: String,
        options: &str,
    ) -> PyResult<()> {
        let options: NoteOptions = serde_json::from_str(options)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        self.state.run(py, |deck| {
            let mut note = BasicNote::new(front, back).tags(options.tags);
            if let Some(id) = options.stable_id {
                note = note.stable_id(id);
            }
            if let Some(value) = options.identity_override {
                note = note.identity_override(
                    BasicIdentityOverride::new(value.fields, value.reason_code)
                        .map_err(deck_error)?,
                );
            }
            deck.add(note).map_err(deck_error)
        })
    }

    fn add_cloze(&self, py: Python<'_>, text: String, options: &str) -> PyResult<()> {
        let options: NoteOptions = serde_json::from_str(options)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        self.state.run(py, |deck| {
            let mut note = ClozeNote::new(text)
                .tags(options.tags)
                .extra(options.extra.unwrap_or_default());
            if let Some(id) = options.stable_id {
                note = note.stable_id(id);
            }
            deck.add(note).map_err(deck_error)
        })
    }

    fn add_image_occlusion(
        &self,
        py: Python<'_>,
        image: Py<NativeDeckMediaRef>,
        options: &str,
    ) -> PyResult<()> {
        let image = image.get().inner.clone();
        let options: IoOptions = serde_json::from_str(options)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        self.state.run(py, |deck| {
            let image = deck.media().get(image.name()).ok_or_else(|| {
                deck_error(
                    DeckError::ImageOcclusionUnknownMedia {
                        media_name: image.name().into(),
                    }
                    .into(),
                )
            })?;
            let mut draft = deck
                .image_occlusion()
                .note(image)
                .mode(options.mode.into())
                .tags(options.tags)
                .header(options.header)
                .back_extra(options.back_extra)
                .comments(options.comments);
            if let Some(id) = options.stable_id {
                draft = draft.stable_id(id);
            }
            for (x, y, width, height) in options.rects {
                draft = draft.rect(x, y, width, height);
            }
            draft.add().map_err(deck_error)
        })
    }

    fn add_media_file(&self, py: Python<'_>, path: PathBuf) -> PyResult<NativeDeckMediaRef> {
        self.state.run(py, |deck| {
            Ok(NativeDeckMediaRef {
                inner: deck
                    .media()
                    .add(MediaSource::from_file(&path))
                    .map_err(|error| media::file_error(error, &path))?,
            })
        })
    }

    fn add_media_bytes(
        &self,
        py: Python<'_>,
        name: String,
        bytes: Vec<u8>,
    ) -> PyResult<NativeDeckMediaRef> {
        self.state.run(py, |deck| {
            Ok(NativeDeckMediaRef {
                inner: deck
                    .media()
                    .add(MediaSource::from_bytes(name, bytes))
                    .map_err(media::media_error)?,
            })
        })
    }

    fn get_media(&self, py: Python<'_>, name: String) -> PyResult<Option<NativeDeckMediaRef>> {
        self.state.run(py, |deck| {
            Ok(deck
                .media()
                .get(&name)
                .map(|inner| NativeDeckMediaRef { inner }))
        })
    }

    fn validate(&self, py: Python<'_>) -> PyResult<String> {
        self.state.run(py, |deck| {
            let report = deck.validate_report().map_err(deck_error)?;
            let diagnostics: Vec<_> = report
                .diagnostics()
                .iter()
                .map(|item| {
                    DiagnosticJson::from(&Diagnostic {
                        code: DiagnosticCode::new(item.code.code().as_str()),
                        severity: item.severity,
                        message: item.message.clone(),
                        domain: None,
                        stage: None,
                        source: None,
                        help: None,
                    })
                })
                .collect();
            Ok(json!(diagnostics).to_string())
        })
    }

    fn build(&self, py: Python<'_>, options: &str) -> PyResult<(String, Option<NativeArtifact>)> {
        let options = serde_json::from_str::<options::BuildInput>(options)
            .map_err(|error| PyValueError::new_err(error.to_string()))?
            .options();
        self.state
            .run(py, |deck| reports::build(deck.build(options)))
    }

    fn to_project(&self, py: Python<'_>) -> PyResult<NativeProject> {
        self.state.run(py, |deck| {
            Ok(NativeProject {
                state: ObjectState::new(Project::from(deck.clone()), "PROJECT"),
            })
        })
    }
}
