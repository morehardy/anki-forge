mod authoring;
mod deck;
mod json_numbers;
mod options;
mod reports;
mod state;
mod tasks;

use anki_forge::build::{BuildOptions, ProjectNormalizeOptions, RiskLevel, UpdateSafetyMode};
use anki_forge::prelude::Project;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

use authoring::{NoteInput, NoteTypeInput};
use state::{ProjectTask, SharedProject};

fn parse<T: serde::de::DeserializeOwned>(input: &str) -> Result<T> {
    serde_json::from_str(input).map_err(|error| Error::new(Status::InvalidArg, error.to_string()))
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ProjectInput {
    stable_id: Option<String>,
    default_deck: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildInput {
    output: PathBuf,
    artifacts_dir: Option<PathBuf>,
    inspect: Option<bool>,
    compare_to: Option<PathBuf>,
    fail_on: Option<RiskLevel>,
    report_json: Option<PathBuf>,
    identity_lockfile: Option<PathBuf>,
    write_identity_lockfile: Option<bool>,
    update_safety: Option<String>,
    self_contained: Option<bool>,
    inspect_limits: Option<options::InspectInput>,
    media_mode: Option<String>,
    media_policy: Option<options::MediaPolicyInput>,
    media_store_dir: Option<PathBuf>,
}

impl BuildInput {
    fn options(self) -> Result<BuildOptions> {
        // Public paths are resolved by the facade before registration. Keep the
        // core's media staging directory inside its per-build artifact workspace.
        let mut normalize = ProjectNormalizeOptions::strict();
        if let Some(path) = self.media_store_dir {
            normalize = normalize.media_store_dir(path);
        }
        if let Some(policy) = self.media_policy {
            normalize = normalize.media_policy(policy.policy());
        }
        if let Some(mode) = self.media_mode {
            normalize = match mode.as_str() {
                "self-contained" => normalize.self_contained(),
                "path-backed" => normalize.path_backed_staging(),
                _ => return Err(Error::new(Status::InvalidArg, "unknown mediaMode")),
            };
        }
        let mut options = BuildOptions::new()
            .output(self.output)
            .normalize_options(normalize);
        if let Some(limits) = self.inspect_limits {
            options = options.inspect_limits(limits.limits());
        }
        if let Some(dir) = self.artifacts_dir {
            options = options.artifacts_dir(dir);
        }
        if let Some(value) = self.inspect {
            options = options.inspect(value);
        }
        if let Some(path) = self.compare_to {
            options = options.compare_to(path);
        }
        if let Some(level) = self.fail_on {
            options = options.fail_on(level);
        }
        if let Some(path) = self.report_json {
            options = options.report_json(path);
        }
        if let Some(path) = self.identity_lockfile {
            options = options.identity_lockfile(path);
        }
        if let Some(value) = self.write_identity_lockfile {
            options = options.write_identity_lockfile(value);
        }
        if self.self_contained == Some(true) {
            options = options.self_contained();
        }
        if let Some(mode) = self.update_safety {
            options = options.update_safety(match mode.as_str() {
                "strict" => UpdateSafetyMode::Strict,
                "report-only" => UpdateSafetyMode::ReportOnly,
                "disabled" => UpdateSafetyMode::Disabled,
                _ => return Err(Error::new(Status::InvalidArg, "unknown updateSafety mode")),
            });
        }
        Ok(options)
    }
}

#[napi]
pub fn binding_metadata() -> String {
    json!({
        "bindingVersion": env!("CARGO_PKG_VERSION"),
        "coreVersion": anki_forge::facade_api_version(),
        "contractVersion": anki_forge::embedded_contract_version(),
        "target": env!("ANKI_FORGE_NODE_TARGET"),
        "nodeApiVersion": 8,
    })
    .to_string()
}

#[napi]
pub struct NativeProject {
    shared: SharedProject,
}

#[napi]
pub struct NativeMediaRef {
    inner: anki_forge::product::MediaRef,
}
#[napi]
impl NativeMediaRef {
    #[napi]
    pub fn render_image(&self) -> String {
        self.inner.image().render()
    }
    #[napi]
    pub fn render_sound(&self) -> String {
        self.inner.sound().render()
    }
}

#[napi]
impl NativeProject {
    #[napi(constructor)]
    pub fn new(name: String, options_json: String) -> Result<Self> {
        let options: ProjectInput = parse(&options_json)?;
        let mut project = Project::new(name);
        if let Some(id) = options.stable_id {
            project = project.stable_id(id);
        }
        if let Some(deck) = options.default_deck {
            project = project.default_deck(deck);
        }
        Ok(Self {
            shared: SharedProject::new(project),
        })
    }

    #[napi]
    pub fn add_note(
        &self,
        note_json: String,
        references: Vec<ClassInstance<NativeMediaRef>>,
    ) -> Result<String> {
        let input = parse::<NoteInput>(&note_json)?;
        let media = references
            .iter()
            .map(|reference| {
                (
                    reference.inner.filename().to_string(),
                    reference.inner.clone(),
                )
            })
            .collect();
        self.shared.with_ready(|context| {
            let note = match input.into_note(&media) {
                Ok(note) => note,
                Err(error) => return reports::domain_failure("note", error),
            };
            match context.project.add_note(note) {
            Ok(_) => reports::success(Value::Null),
            Err(error) => reports::failure("add", error.code().as_str(), &error.to_string(),
                json!({"diagnostic": anki_forge::build::json_report::DiagnosticJson::from(error.diagnostic())})),
            }
        })
    }

    #[napi]
    pub fn media_ref(&self, filename: String) -> Result<NativeMediaRef> {
        self.shared.with_ready(|context| {
            context
                .media
                .get(&filename)
                .cloned()
                .map(|inner| NativeMediaRef { inner })
                .ok_or_else(|| Error::new(Status::InvalidArg, "unknown registered media"))
        })?
    }

    #[napi]
    pub fn add_note_type(&self, input: String) -> Result<String> {
        let note_type = parse::<NoteTypeInput>(&input)?.into_notetype();
        self.shared.with_ready(|context| match context.project.add_notetype(note_type) {
            Ok(_) => reports::success(Value::Null),
            Err(error) => reports::failure("add", error.code().as_str(), &error.to_string(),
                json!({"diagnostic": anki_forge::build::json_report::DiagnosticJson::from(error.diagnostic())})),
        })
    }

    #[napi]
    pub fn add_media_file<'env>(
        &self,
        env: &'env Env,
        path: String,
        export_as: String,
    ) -> Result<Object<'env>> {
        tasks::spawn(
            env,
            ProjectTask::media_file(self.shared.reserve()?, path.into(), export_as),
        )
    }

    #[napi]
    pub fn add_media_bytes<'env>(
        &self,
        env: &'env Env,
        label: String,
        export_as: String,
        bytes: Buffer,
        spool: bool,
    ) -> Result<Object<'env>> {
        tasks::spawn(
            env,
            ProjectTask::media_bytes(
                self.shared.reserve()?,
                label,
                export_as,
                bytes.to_vec(),
                spool,
            ),
        )
    }

    #[napi]
    pub fn import_template_bundle<'env>(
        &self,
        env: &'env Env,
        path: String,
    ) -> Result<Object<'env>> {
        tasks::spawn(
            env,
            ProjectTask::template_bundle(self.shared.reserve()?, path.into()),
        )
    }

    #[napi]
    pub fn validate<'env>(&self, env: &'env Env) -> Result<Object<'env>> {
        tasks::spawn(env, ProjectTask::validate(self.shared.reserve()?))
    }

    #[napi]
    pub fn build<'env>(&self, env: &'env Env, options_json: String) -> Result<Object<'env>> {
        let options = parse::<BuildInput>(&options_json)?.options()?;
        tasks::spawn(env, ProjectTask::build(self.shared.reserve()?, options))
    }

    #[napi]
    pub fn apkg_bytes<'env>(&self, env: &'env Env) -> Result<Object<'env>> {
        tasks::spawn(env, state::BytesTask::new(self.shared.reserve()?))
    }

    #[napi]
    pub fn diff_against_apkg<'env>(
        &self,
        env: &'env Env,
        path: String,
        limits_json: String,
    ) -> Result<Object<'env>> {
        let limits = parse::<options::InspectInput>(&limits_json)?.limits();
        tasks::spawn(
            env,
            ProjectTask::diff(self.shared.reserve()?, path.into(), limits),
        )
    }
}

#[napi]
pub fn default_inspect_limits() -> String {
    options::default_limits().to_string()
}

#[napi]
pub fn validate_template(source: String, fields: Vec<String>) -> String {
    use anki_forge::product::{TemplateEngine, TemplateIssueSeverity};
    let diagnostics: Vec<_> = TemplateEngine::validate(&source, fields).into_iter().map(|issue| json!({
        "code": issue.code, "severity": match issue.severity { TemplateIssueSeverity::Error => "error", TemplateIssueSeverity::Warning => "warning" },
        "domain": "template", "stage": "validate", "path": null,
        "span": {"byte_start": issue.byte_offset, "byte_end": issue.byte_offset},
        "message": issue.message, "suggested_fix": null,
    })).collect();
    reports::success(json!({"diagnostics": diagnostics}))
}

#[napi]
pub fn render_content(text: String, html: bool) -> String {
    use anki_forge::prelude::Content;
    if html {
        Content::html(text).render()
    } else {
        Content::text(text).render()
    }
}
