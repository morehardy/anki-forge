use anki_forge::build::BuildOptions;
use anki_forge::prelude::Project;
use napi::{Env, Error, Result, Status, Task};
use napi_derive::napi;
use serde_json::json;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::reports;

#[derive(Clone)]
pub struct ProjectContext {
    pub project: Project,
    pub deck: Option<anki_forge::deck::Deck>,
    pub media: BTreeMap<String, anki_forge::product::MediaRef>,
    staged_media: Vec<Arc<tempfile::TempDir>>,
}
enum State {
    Ready(Box<ProjectContext>),
    Busy,
    Failed,
}

#[derive(Clone)]
pub struct SharedProject(Arc<Mutex<State>>);

fn busy() -> Error {
    Error::new(Status::GenericFailure, "BINDING.PROJECT_BUSY")
}
fn failed() -> Error {
    Error::new(Status::GenericFailure, "BINDING.PROJECT_FAILED")
}

impl SharedProject {
    fn from_context(context: ProjectContext) -> Self {
        Self(Arc::new(Mutex::new(State::Ready(Box::new(context)))))
    }

    pub fn new(project: Project) -> Self {
        Self(Arc::new(Mutex::new(State::Ready(Box::new(
            ProjectContext {
                project,
                deck: None,
                media: BTreeMap::new(),
                staged_media: Vec::new(),
            },
        )))))
    }
    pub fn new_deck(deck: anki_forge::deck::Deck) -> Self {
        Self(Arc::new(Mutex::new(State::Ready(Box::new(
            ProjectContext {
                project: Project::new(deck.name()),
                deck: Some(deck),
                media: BTreeMap::new(),
                staged_media: Vec::new(),
            },
        )))))
    }

    pub fn with_ready<T>(&self, operation: impl FnOnce(&mut ProjectContext) -> T) -> Result<T> {
        let mut state = self.0.try_lock().map_err(|_| busy())?;
        match &mut *state {
            State::Ready(project) => {
                catch_unwind(AssertUnwindSafe(|| operation(project))).map_err(|_| {
                    *state = State::Failed;
                    failed()
                })
            }
            State::Busy => Err(busy()),
            State::Failed => Err(failed()),
        }
    }

    pub fn reserve(&self) -> Result<ProjectLease> {
        let mut state = self.0.try_lock().map_err(|_| busy())?;
        match &*state {
            State::Busy => return Err(busy()),
            State::Failed => return Err(failed()),
            State::Ready(_) => {}
        }
        let State::Ready(project) = std::mem::replace(&mut *state, State::Busy) else {
            unreachable!()
        };
        Ok(ProjectLease {
            shared: self.clone(),
            project: Some(project),
            failed: false,
        })
    }
}

pub struct ProjectLease {
    shared: SharedProject,
    project: Option<Box<ProjectContext>>,
    failed: bool,
}

impl ProjectLease {
    fn restore(&mut self) {
        if let Some(project) = self.project.take() {
            if let Ok(mut state) = self.shared.0.lock() {
                *state = if self.failed {
                    State::Failed
                } else {
                    State::Ready(project)
                };
            }
        }
    }
}

impl Drop for ProjectLease {
    fn drop(&mut self) {
        self.restore();
    }
}

enum Operation {
    DescribeDeck,
    Validate,
    MediaFile(PathBuf, String),
    MediaBytes(String, String, Vec<u8>, bool),
    TemplateBundle(PathBuf),
    Diff(PathBuf, anki_forge::prelude::InspectLimits),
    DeckMedia(Option<anki_forge::deck::MediaSource>),
}

pub struct ProjectTask {
    lease: ProjectLease,
    operation: Operation,
}

impl ProjectTask {
    pub fn describe_deck(lease: ProjectLease) -> Self {
        Self {
            lease,
            operation: Operation::DescribeDeck,
        }
    }
    pub fn validate(lease: ProjectLease) -> Self {
        Self {
            lease,
            operation: Operation::Validate,
        }
    }
    pub fn media_file(lease: ProjectLease, path: PathBuf, export_as: String) -> Self {
        Self {
            lease,
            operation: Operation::MediaFile(path, export_as),
        }
    }
    pub fn media_bytes(
        lease: ProjectLease,
        label: String,
        export_as: String,
        bytes: Vec<u8>,
        spool: bool,
    ) -> Self {
        Self {
            lease,
            operation: Operation::MediaBytes(label, export_as, bytes, spool),
        }
    }
    pub fn template_bundle(lease: ProjectLease, path: PathBuf) -> Self {
        Self {
            lease,
            operation: Operation::TemplateBundle(path),
        }
    }
    pub fn diff(
        lease: ProjectLease,
        path: PathBuf,
        limits: anki_forge::prelude::InspectLimits,
    ) -> Self {
        Self {
            lease,
            operation: Operation::Diff(path, limits),
        }
    }
    pub fn deck_media(lease: ProjectLease, source: anki_forge::deck::MediaSource) -> Self {
        Self {
            lease,
            operation: Operation::DeckMedia(Some(source)),
        }
    }

    fn run(&mut self) -> String {
        let context = self.lease.project.as_mut().expect("task owns a project");
        // Validation keeps the existing Project-level checks; diff has no Deck API.
        // Both snapshots last only for this operation instead of residing in the context.
        let mut snapshot = if matches!(&self.operation, Operation::Validate | Operation::Diff(_, _))
        {
            context
                .deck
                .as_ref()
                .map(|deck| Project::from(deck.clone()))
        } else {
            None
        };
        let project = snapshot.as_mut().unwrap_or(&mut context.project);
        match &mut self.operation {
            Operation::DescribeDeck => reports::success(crate::snapshots::deck(
                context.deck.as_ref().expect("deck context"),
            )),
            Operation::Validate => {
                let report = project.validate();
                reports::success(json!({"diagnostics": report.diagnostics.iter()
                    .map(anki_forge::build::json_report::DiagnosticJson::from).collect::<Vec<_>>()}))
            }
            Operation::MediaFile(path, export_as) => match project
                .media_mut()
                .add_file(path)
                .and_then(|pending| pending.export_as(export_as.clone()))
            {
                Ok(media) => {
                    context
                        .media
                        .insert(media.filename().to_string(), media.clone());
                    reports::success(json!({"filename": media.filename()}))
                }
                Err(error) => reports::domain_failure("media", error),
            },
            Operation::MediaBytes(label, export_as, bytes, spool) => {
                let result = (|| -> anyhow::Result<_> {
                    let mut staged = None;
                    let pending = if *spool
                        && bytes.len() > anki_forge::product::MediaRegistry::inline_limit_bytes()
                    {
                        let directory = tempfile::Builder::new()
                            .prefix("anki-forge-node-media-")
                            .tempdir()?;
                        let path = directory.path().join("payload");
                        std::fs::write(&path, &*bytes)?;
                        staged = Some(directory);
                        project.media_mut().add_file(path)?
                    } else {
                        project
                            .media_mut()
                            .add_bytes(label.clone(), std::mem::take(bytes))?
                    };
                    let media = pending.export_as(export_as.clone())?;
                    if let Some(directory) = staged {
                        context.staged_media.push(Arc::new(directory));
                    }
                    Ok(media)
                })();
                match result {
                    Ok(media) => {
                        context
                            .media
                            .insert(media.filename().to_string(), media.clone());
                        reports::success(json!({"filename": media.filename()}))
                    }
                    Err(error) => reports::domain_failure("media", error),
                }
            }
            Operation::TemplateBundle(path) => match project.import_template_bundle(path) {
                Ok(_) => reports::success(serde_json::Value::Null),
                Err(error) => reports::failure(
                    "template_bundle",
                    error.code(),
                    &error.to_string(),
                    json!({"path": error.path(), "byteOffset": error.byte_offset()}),
                ),
            },
            Operation::Diff(path, limits) => {
                match project.diff_against_apkg_with_limits(path, limits.clone()) {
                    Ok(report) => reports::success(reports::diff_report(&report)),
                    Err(error) => reports::failure(
                        "diff",
                        error.cause.code().as_str(),
                        "Project comparison failed",
                        json!({"report": reports::diff_report(&error.report), "cause": format!("{:?}", error.cause)}),
                    ),
                }
            }
            Operation::DeckMedia(source) => match context
                .deck
                .as_mut()
                .expect("deck context")
                .media()
                .add(source.take().expect("media task runs once"))
            {
                Ok(media) => reports::success(json!({"filename":media.name()})),
                Err(error) => reports::domain_failure("media", error),
            },
        }
    }
}

pub struct BuildTask {
    lease: ProjectLease,
    options: BuildOptions,
}

impl BuildTask {
    pub fn new(lease: ProjectLease, options: BuildOptions) -> Self {
        Self { lease, options }
    }
}

impl Task for BuildTask {
    type Output = crate::artifact::BuildOutcome;
    type JsValue = crate::artifact::NativeBuildResult;

    fn compute(&mut self) -> Result<Self::Output> {
        let result = catch_unwind(AssertUnwindSafe(|| {
            let context = self.lease.project.as_mut().expect("task owns a project");
            let result = match &context.deck {
                Some(deck) => deck.build(self.options.clone()),
                None => context.project.build(self.options.clone()),
            };
            crate::artifact::BuildOutcome::from_result(result)
        }));
        result.map_err(|_| {
            self.lease.failed = true;
            failed()
        })
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        self.lease.restore();
        Ok(output.into_native())
    }

    fn reject(&mut self, _env: Env, error: Error) -> Result<Self::JsValue> {
        self.lease.restore();
        Err(error)
    }
}

#[napi(object)]
pub struct NativeBytesResult {
    pub result: String,
    pub data: napi::bindgen_prelude::Buffer,
}
pub struct BytesTask {
    lease: ProjectLease,
}
impl BytesTask {
    pub fn new(lease: ProjectLease) -> Self {
        Self { lease }
    }
    fn run(&self) -> anyhow::Result<(String, Vec<u8>)> {
        let directory = tempfile::Builder::new()
            .prefix("anki-forge-node-buffer-")
            .tempdir()?;
        let output = directory.path().join("deck.apkg");
        let options = BuildOptions::new().output(&output);
        let context = self.lease.project.as_ref().expect("task owns project");
        let result = match &context.deck {
            Some(deck) => deck.build(options),
            None => context.project.build(options),
        };
        match result {
            Ok(report) => {
                report.ensure_success()?;
                Ok((
                    reports::success(serde_json::Value::Null),
                    std::fs::read(output)?,
                ))
            }
            Err(error) => {
                let mut details = reports::build_report(&error.report);
                details["cause"] = json!(format!("{:?}", error.cause));
                Ok((
                    reports::failure("build", error.code().as_str(), &error.to_string(), details),
                    Vec::new(),
                ))
            }
        }
    }
}
#[napi]
impl Task for BytesTask {
    type Output = (String, Vec<u8>);
    type JsValue = NativeBytesResult;
    fn compute(&mut self) -> Result<Self::Output> {
        match catch_unwind(AssertUnwindSafe(|| self.run())) {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(error)) => Err(Error::new(Status::GenericFailure, error.to_string())),
            Err(_) => {
                self.lease.failed = true;
                Err(failed())
            }
        }
    }
    fn resolve(&mut self, _env: Env, (result, bytes): Self::Output) -> Result<Self::JsValue> {
        self.lease.restore();
        Ok(NativeBytesResult {
            result,
            data: bytes.into(),
        })
    }
    fn reject(&mut self, _env: Env, error: Error) -> Result<Self::JsValue> {
        self.lease.restore();
        Err(error)
    }
}

#[napi]
impl Task for ProjectTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> Result<Self::Output> {
        match catch_unwind(AssertUnwindSafe(|| self.run())) {
            Ok(result) => Ok(result),
            Err(_) => {
                self.lease.failed = true;
                Err(failed())
            }
        }
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        self.lease.restore();
        Ok(output)
    }

    fn reject(&mut self, _env: Env, error: Error) -> Result<Self::JsValue> {
        self.lease.restore();
        Err(error)
    }
}

/// Copies core authoring state while its source lease is reserved.
/// Staged media is immutable and keeps shared ownership across independent copies.
pub struct ProjectCopyTask {
    lease: ProjectLease,
    from_deck: bool,
}
impl ProjectCopyTask {
    pub fn new(lease: ProjectLease, from_deck: bool) -> Self {
        Self { lease, from_deck }
    }
}
impl Task for ProjectCopyTask {
    type Output = SharedProject;
    type JsValue = crate::NativeProject;
    fn compute(&mut self) -> Result<Self::Output> {
        catch_unwind(AssertUnwindSafe(|| {
            let context = self.lease.project.as_ref().expect("task owns a deck");
            if self.from_deck {
                SharedProject::new(Project::from(
                    context.deck.as_ref().expect("deck context").clone(),
                ))
            } else {
                SharedProject::from_context(context.as_ref().clone())
            }
        }))
        .map_err(|_| {
            self.lease.failed = true;
            failed()
        })
    }
    fn resolve(&mut self, _env: Env, shared: Self::Output) -> Result<Self::JsValue> {
        self.lease.restore();
        Ok(crate::NativeProject { shared })
    }
    fn reject(&mut self, _env: Env, error: Error) -> Result<Self::JsValue> {
        self.lease.restore();
        Err(error)
    }
}

pub struct DeckCopyTask(ProjectCopyTask);
impl DeckCopyTask {
    pub fn new(lease: ProjectLease) -> Self {
        Self(ProjectCopyTask::new(lease, false))
    }
}
impl Task for DeckCopyTask {
    type Output = SharedProject;
    type JsValue = crate::deck::NativeDeck;
    fn compute(&mut self) -> Result<Self::Output> {
        self.0.compute()
    }
    fn resolve(&mut self, _env: Env, shared: Self::Output) -> Result<Self::JsValue> {
        self.0.lease.restore();
        Ok(crate::deck::NativeDeck::from_shared(shared))
    }
    fn reject(&mut self, _env: Env, error: Error) -> Result<Self::JsValue> {
        self.0.lease.restore();
        Err(error)
    }
}
