mod counts;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Context;
use authoring_core::{normalize_with_options, NormalizationRequest, NormalizeOptions};
use base64::Engine as _;
use tempfile::TempDir;
use writer_core::{artifact_path_from_ref, BuildArtifactTarget};

use counts::{
    card_count_from_inspect_or_fallback, count_phase1_cards_without_inspect, inspect_metadata_count,
};

use crate::build::{
    ApkgArtifact, BuildCounts, BuildError, BuildFailureCause, BuildMetrics, BuildOptions,
    BuildReport, InspectSummary, ProjectNormalizeOptions,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath, ValidationReport};
use crate::product::{
    LoweringDiagnostic, LoweringPlan, Note, NoteType, ProductDiagnostic, ProductDocument,
    ProductLoweringError, ProductSourceMap, STOCK_BASIC_ID, STOCK_CLOZE_ID,
};

#[derive(Debug, Clone)]
pub struct Project {
    name: String,
    stable_id: Option<String>,
    default_deck: Option<String>,
    note_types: Vec<NoteType>,
    notes: Vec<Note>,
    media: crate::product::MediaRegistry,
    deck_source: Option<crate::deck::Deck>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stable_id: None,
            default_deck: None,
            note_types: Vec::new(),
            notes: Vec::new(),
            media: crate::product::MediaRegistry::default(),
            deck_source: None,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn default_deck(mut self, deck_name: impl Into<String>) -> Self {
        self.default_deck = Some(deck_name.into());
        self
    }

    pub fn add_notetype(&mut self, note_type: NoteType) -> anyhow::Result<&mut Self> {
        self.note_types.push(note_type);
        Ok(self)
    }

    pub fn add_note(&mut self, note: Note) -> anyhow::Result<&mut Self> {
        self.notes.push(note);
        Ok(self)
    }

    pub fn media_mut(&mut self) -> &mut crate::product::MediaRegistry {
        &mut self.media
    }

    pub fn validate(&self) -> ValidationReport {
        let mut diagnostics = Vec::new();
        let mut seen_stable_ids = BTreeSet::new();

        if let Some(deck) = &self.deck_source {
            match deck.validate_report() {
                Ok(report) => diagnostics.extend(
                    report
                        .diagnostics()
                        .iter()
                        .map(deck_validation_diagnostic_to_project_diagnostic),
                ),
                Err(error) => diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.DECK_VALIDATE_FAILED"),
                    severity: Severity::Error,
                    message: error.to_string(),
                    source: Some(SourcePath::new("project.deck")),
                    help: Some("inspect deck notes before building".into()),
                }),
            }

            if !self.notes.is_empty() || !self.note_types.is_empty() {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.DECK_SOURCE_AUTHORING_STATE_UNSUPPORTED"),
                    severity: Severity::Error,
                    message:
                        "deck-backed projects cannot mix direct Project notes or note types yet"
                            .into(),
                    source: Some(SourcePath::new("project")),
                    help: Some(
                        "add notes to the Deck before converting it with Project::from".into(),
                    ),
                });
            }
        }

        let custom_note_type_ids = self
            .note_types
            .iter()
            .map(|note_type| note_type.id())
            .collect::<BTreeSet<_>>();

        for (index, note) in self.notes.iter().enumerate() {
            if let Some(stable_id) = note.stable_id_ref() {
                if stable_id.trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("AFID.STABLE_ID_BLANK"),
                        severity: Severity::Error,
                        message: "stable_id cannot be blank".into(),
                        source: Some(SourcePath::new(format!("project.notes[{index}]"))),
                        help: Some("choose a non-empty stable_id or omit it".into()),
                    });
                } else if !seen_stable_ids.insert(stable_id) {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("AFID.STABLE_ID_DUPLICATE"),
                        severity: Severity::Error,
                        message: format!("duplicate stable_id '{stable_id}'"),
                        source: Some(SourcePath::new(format!("project.notes[{index}]"))),
                        help: Some("choose a unique stable_id for each note".into()),
                    });
                }
            }

            if note.note_type_id() != STOCK_BASIC_ID
                && note.note_type_id() != STOCK_CLOZE_ID
                && !custom_note_type_ids.contains(note.note_type_id())
            {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.UNSUPPORTED_NOTE_TYPE"),
                    severity: Severity::Error,
                    message: format!(
                        "note type '{}' is not registered on the project",
                        note.note_type_id()
                    ),
                    source: Some(SourcePath::new(format!("project.notes[{index}]"))),
                    help: Some("add a matching NoteType with Project::add_notetype".into()),
                });
            }
        }

        let mut notetype_id_counts = BTreeMap::<&str, usize>::new();
        for note_type in &self.note_types {
            *notetype_id_counts.entry(note_type.id()).or_default() += 1;
        }
        let mut first_notetype_by_id = BTreeMap::<&str, (usize, Option<&str>)>::new();
        for (index, note_type) in self.note_types.iter().enumerate() {
            if let Some((first_index, first_name)) = first_notetype_by_id.get(note_type.id()) {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("NOTETYPE.ID_DUPLICATE"),
                    severity: Severity::Error,
                    message: duplicate_notetype_message(
                        note_type.id(),
                        *first_index,
                        *first_name,
                        index,
                        note_type.name_ref(),
                    ),
                    source: Some(SourcePath::new(format!("project.note_types[{index}]"))),
                    help: Some("choose a unique id for each custom note type".into()),
                });
            } else {
                first_notetype_by_id.insert(note_type.id(), (index, note_type.name_ref()));
            }
        }

        for (index, note_type) in self.note_types.iter().enumerate() {
            let note_type_source = if notetype_id_counts
                .get(note_type.id())
                .copied()
                .unwrap_or_default()
                > 1
            {
                format!("project.note_types[{index}]")
            } else {
                format!("project.note_types[{:?}]", note_type.id())
            };
            if note_type.identity_ref().is_none() {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("NOTETYPE.IDENTITY_RECIPE_MISSING"),
                    severity: Severity::Warning,
                    message: format!(
                        "custom note type '{}' has no identity recipe",
                        note_type.id()
                    ),
                    source: Some(SourcePath::new(note_type_source.clone())),
                    help: Some(
                        "add IdentityRecipe::fields([...]) before relying on update-safe builds"
                            .into(),
                    ),
                });
            }

            for field in note_type.fields() {
                if field.key_auto_derived() {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("NOTETYPE.FIELD_KEY_AUTO_DERIVED"),
                        severity: Severity::Warning,
                        message: format!(
                            "field '{}' in note type '{}' uses an auto-derived key",
                            field.name(),
                            note_type.id()
                        ),
                        source: Some(SourcePath::new(format!(
                            "{}.fields[\"{}\"]",
                            note_type_source,
                            field.name()
                        ))),
                        help: Some("call .key(\"stable-field-key\") explicitly".into()),
                    });
                }
            }
        }

        ValidationReport { diagnostics }
    }

    pub fn lower(&self) -> anyhow::Result<LoweringPlan> {
        if let Some(deck) = &self.deck_source {
            let product = deck.clone().into_product_document()?;
            let mut plan = product
                .lower()
                .map_err(|err| anyhow::anyhow!("lower deck product document: {:?}", err))?;
            self.apply_note_source_paths(&mut plan);
            return Ok(plan);
        }

        let mut plan = self
            .to_product_document()
            .lower()
            .map_err(|err| anyhow::anyhow!("lower product document: {:?}", err))?;
        self.apply_note_source_paths(&mut plan);
        plan.authoring_document
            .media
            .extend(product_media_to_authoring_media(self.media.media())?);
        record_project_media_source_paths(&mut plan, self.media.media());
        Ok(plan)
    }

    pub fn normalize(&self) -> anyhow::Result<authoring_core::NormalizedIr> {
        let temp_dir = tempfile::Builder::new()
            .prefix("anki-forge-project-normalize-")
            .tempdir()
            .context("create project normalize temp dir")?;
        self.normalize_with_dirs(
            temp_dir.path(),
            temp_dir.path().join(".anki-forge-media"),
            ProjectNormalizeOptions::default(),
        )
        .map(|output| output.normalized_ir)
        .map_err(anyhow::Error::from)
    }

    pub fn build(&self, options: BuildOptions) -> Result<BuildReport, BuildError> {
        let started = Instant::now();
        let artifact_workspace = ArtifactWorkspace::new(&options, started)?;
        let artifacts_dir = artifact_workspace.path.clone();
        let normalize_options = options.normalize_options.clone().unwrap_or_default();
        let media_input_dir = normalize_options
            .base_dir
            .clone()
            .unwrap_or_else(|| artifacts_dir.join(".anki-forge-media-input"));
        let media_store_dir = normalize_options
            .media_store_dir
            .clone()
            .unwrap_or_else(|| artifacts_dir.join(".anki-forge-media"));

        let validation = self.validate();
        let mut diagnostics = validation.diagnostics;

        let normalized_output =
            self.normalize_with_dirs(&media_input_dir, &media_store_dir, normalize_options);
        let normalized_output = match normalized_output {
            Ok(output) => output,
            Err(error) => {
                let ProjectNormalizeError {
                    message,
                    diagnostics: mut normalize_diagnostics,
                    normalized_ir,
                } = error;
                diagnostics.append(&mut normalize_diagnostics);
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.NORMALIZE_FAILED"),
                    severity: Severity::Error,
                    message,
                    source: Some(SourcePath::new("project")),
                    help: Some("inspect product notes and media registrations".into()),
                });
                let counts = normalized_ir
                    .as_ref()
                    .map(|normalized| BuildCounts {
                        notes: normalized.notes.len(),
                        cards: count_phase1_cards_without_inspect(normalized.as_ref()),
                        media: normalized.media_bindings.len(),
                    })
                    .unwrap_or_default();
                return Err(BuildError::new(
                    BuildReport {
                        artifact: None,
                        counts,
                        diagnostics,
                        metrics: BuildMetrics {
                            duration: started.elapsed(),
                        },
                        inspect: None,
                        status: "invalid".into(),
                    },
                    BuildFailureCause::Diagnostics,
                ));
            }
        };
        let normalized = normalized_output.normalized_ir;
        diagnostics.extend(normalized_output.diagnostics);

        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            return Err(BuildError::new(
                BuildReport {
                    artifact: None,
                    counts: BuildCounts {
                        notes: normalized.notes.len(),
                        cards: count_phase1_cards_without_inspect(&normalized),
                        media: normalized.media_bindings.len(),
                    },
                    diagnostics,
                    metrics: BuildMetrics {
                        duration: started.elapsed(),
                    },
                    inspect: None,
                    status: "invalid".into(),
                },
                BuildFailureCause::Diagnostics,
            ));
        }

        let current_dir = std::env::current_dir().map_err(|err| {
            BuildError::new(
                failure_report(started, "PROJECT.CURRENT_DIR_FAILED", err.to_string()),
                BuildFailureCause::Io,
            )
        })?;
        let (_runtime, writer_policy, build_context) =
            crate::runtime::load_default_writer_stack(current_dir).map_err(|err| {
                BuildError::new(
                    failure_report(started, "PROJECT.RUNTIME_DEFAULTS_FAILED", err.to_string()),
                    BuildFailureCause::Io,
                )
            })?;
        let stable_ref_prefix = self
            .stable_id
            .as_deref()
            .map(|stable_id| format!("artifacts/{stable_id}"))
            .unwrap_or_else(|| "artifacts".into());
        let artifact_target = BuildArtifactTarget::new(artifacts_dir.clone(), stable_ref_prefix)
            .with_media_store_dir(media_store_dir);
        let package_build_result = crate::writer_build(
            &normalized,
            &writer_policy,
            &build_context,
            &artifact_target,
        )
        .map_err(|err| {
            BuildError::new(
                failure_report(started, "PROJECT.WRITER_FAILED", err.to_string()),
                BuildFailureCause::BuildStatus,
            )
        })?;

        diagnostics.extend(
            package_build_result
                .diagnostics
                .items
                .iter()
                .map(|item| Diagnostic {
                    code: DiagnosticCode::new(item.code.clone()),
                    severity: severity_from_level(&item.level),
                    message: item.summary.clone(),
                    source: item.path.clone().map(SourcePath::new),
                    help: None,
                }),
        );

        let mut artifact = None;
        if let Some(apkg_ref) = package_build_result.apkg_ref.as_deref() {
            let built_path = artifact_path_from_ref(&artifact_target, apkg_ref).map_err(|err| {
                BuildError::new(
                    failure_report(started, "PROJECT.ARTIFACT_REF_FAILED", err.to_string()),
                    BuildFailureCause::Io,
                )
            })?;
            let final_path = if let Some(output) = options.output.as_ref() {
                if let Some(parent) = output.parent() {
                    std::fs::create_dir_all(parent).map_err(|err| {
                        BuildError::new(
                            failure_report(started, "PROJECT.OUTPUT_DIR_FAILED", err.to_string()),
                            BuildFailureCause::Io,
                        )
                    })?;
                }
                std::fs::copy(&built_path, output).map_err(|err| {
                    BuildError::new(
                        failure_report(started, "PROJECT.OUTPUT_COPY_FAILED", err.to_string()),
                        BuildFailureCause::Io,
                    )
                })?;
                output.clone()
            } else {
                built_path
            };
            artifact = Some(ApkgArtifact { path: final_path });
        }

        let inspect = if options.inspect {
            artifact
                .as_ref()
                .and_then(|artifact| crate::inspect_apkg(&artifact.path).ok())
                .map(|report| InspectSummary {
                    notes: inspect_metadata_count(&report, "note_count"),
                    cards: inspect_metadata_count(&report, "card_count"),
                    source_kind: report.source_kind,
                    observation_status: report.observation_status,
                    notetypes: report.observations.notetypes.len(),
                    templates: report.observations.templates.len(),
                    fields: report.observations.fields.len(),
                    media: report.observations.media.len(),
                })
        } else {
            None
        };

        let counts = BuildCounts {
            notes: normalized.notes.len(),
            cards: card_count_from_inspect_or_fallback(inspect.as_ref(), &normalized),
            media: normalized.media_bindings.len(),
        };

        if package_build_result.result_status != "success"
            && !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("PROJECT.BUILD_STATUS_FAILED"),
                severity: Severity::Error,
                message: format!("build status was {}", package_build_result.result_status),
                source: Some(SourcePath::new("project.build")),
                help: Some("inspect writer diagnostics for the failed stage".into()),
            });
        }

        let report = BuildReport {
            artifact,
            counts,
            diagnostics,
            metrics: BuildMetrics {
                duration: started.elapsed(),
            },
            inspect,
            status: package_build_result.result_status,
        };

        report.ensure_success()?;
        artifact_workspace.persist_if_requested();
        Ok(report)
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> Result<BuildReport, BuildError> {
        self.build(BuildOptions::new().output(path.as_ref().to_path_buf()))
    }

    fn to_product_document(&self) -> ProductDocument {
        let document_id = self.stable_id.clone().unwrap_or_else(|| self.name.clone());
        let default_deck = self
            .default_deck
            .clone()
            .unwrap_or_else(|| self.name.clone());
        let mut product = ProductDocument::new(document_id).with_default_deck(default_deck.clone());
        if self
            .notes
            .iter()
            .any(|note| note.note_type_id() == STOCK_BASIC_ID)
        {
            product = product.with_basic(STOCK_BASIC_ID);
        }
        if self
            .notes
            .iter()
            .any(|note| note.note_type_id() == STOCK_CLOZE_ID)
        {
            product = product.with_cloze(STOCK_CLOZE_ID);
        }

        for note_type in &self.note_types {
            let custom = crate::product::model::CustomNoteType {
                id: note_type.id().to_string(),
                name: note_type.name_ref().map(ToOwned::to_owned),
                fields: note_type
                    .fields()
                    .iter()
                    .map(|field| crate::product::model::CustomField {
                        name: field.name().to_string(),
                        key: Some(field.key_ref().as_str().to_string()),
                    })
                    .collect(),
                templates: note_type
                    .templates()
                    .iter()
                    .map(|template| crate::product::model::CustomTemplate {
                        name: template.name().to_string(),
                        key: Some(template.key_ref().as_str().to_string()),
                        question_format: template.front_source().as_str().to_string(),
                        answer_format: template.back_source().as_str().to_string(),
                        generation_rule: Some(custom_generation_rule(template.generation_rule())),
                    })
                    .collect(),
                css: None,
            };
            product = product.with_custom_notetype(custom);

            for template in note_type.templates() {
                if template.browser_front_source().is_some()
                    || template.browser_back_source().is_some()
                {
                    product = product.with_browser_appearance(
                        note_type.id().to_string(),
                        crate::product::metadata::TemplateBrowserAppearanceDeclaration {
                            template_name: template.name().to_string(),
                            question_format: template
                                .browser_front_source()
                                .map(|source| source.as_str().to_string()),
                            answer_format: template
                                .browser_back_source()
                                .map(|source| source.as_str().to_string()),
                            font_name: None,
                            font_size: None,
                        },
                    );
                }

                if let Some(deck_name) = template.target_deck_name() {
                    product = product.with_template_target_deck(
                        note_type.id().to_string(),
                        crate::product::metadata::TemplateTargetDeckDeclaration {
                            template_name: template.name().to_string(),
                            deck_name: deck_name.to_string(),
                        },
                    );
                }
            }
        }

        let stable_id_counts = self.note_stable_id_counts();
        for (index, note) in self.notes.iter().enumerate() {
            let note_id = match note.stable_id_ref() {
                Some(stable_id)
                    if !stable_id.trim().is_empty()
                        && stable_id_counts.get(stable_id).copied() == Some(1) =>
                {
                    stable_id.to_string()
                }
                _ => format!("generated:{}", index + 1),
            };
            let deck_name = note
                .deck_name()
                .unwrap_or(default_deck.as_str())
                .to_string();
            let fields = note.rendered_fields();
            if note.note_type_id() == STOCK_BASIC_ID {
                product = product.add_basic_note_with_tags(
                    STOCK_BASIC_ID,
                    note_id,
                    deck_name,
                    fields.get("Front").cloned().unwrap_or_default(),
                    fields.get("Back").cloned().unwrap_or_default(),
                    note.tags().iter().cloned(),
                );
            } else if note.note_type_id() == STOCK_CLOZE_ID {
                product = product.add_cloze_note_with_tags(
                    STOCK_CLOZE_ID,
                    note_id,
                    deck_name,
                    fields.get("Text").cloned().unwrap_or_default(),
                    fields.get("Back Extra").cloned().unwrap_or_default(),
                    note.tags().iter().cloned(),
                );
            } else {
                let fields = custom_note_fields_for_authoring(self, note);
                product = product.add_custom_note(crate::product::model::CustomNote {
                    id: note_id,
                    note_type_id: note.note_type_id().to_string(),
                    deck_name,
                    fields,
                    tags: note.tags().to_vec(),
                });
            }
        }
        product
    }

    fn note_stable_id_counts(&self) -> BTreeMap<&str, usize> {
        let mut counts = BTreeMap::new();
        for note in &self.notes {
            let Some(stable_id) = note.stable_id_ref() else {
                continue;
            };
            if stable_id.trim().is_empty() {
                continue;
            }
            *counts.entry(stable_id).or_default() += 1;
        }
        counts
    }

    fn apply_note_source_paths(&self, plan: &mut LoweringPlan) {
        if self.deck_source.is_some() {
            for (index, authoring_note) in plan.authoring_document.notes.iter().enumerate() {
                let note_source = format!("project.notes[{index}]");
                for field_name in authoring_note.fields.keys() {
                    plan.source_map.insert(
                        crate::product::lowering::authoring_note_field_path(
                            &authoring_note.id,
                            field_name,
                        ),
                        crate::product::lowering::product_note_field_source(
                            &note_source,
                            field_name,
                        ),
                    );
                }
            }
            return;
        }
        let stable_id_counts = self.note_stable_id_counts();
        for (index, authoring_note) in plan.authoring_document.notes.iter().enumerate() {
            let Some(product_note) = self.notes.get(index) else {
                continue;
            };
            let note_source = match product_note.stable_id_ref() {
                Some(stable_id)
                    if !stable_id.trim().is_empty()
                        && stable_id_counts.get(stable_id).copied() == Some(1) =>
                {
                    format!("project.notes[{stable_id:?}]")
                }
                _ => format!("project.notes[{index}]"),
            };
            for field_name in authoring_note.fields.keys() {
                plan.source_map.insert(
                    crate::product::lowering::authoring_note_field_path(
                        &authoring_note.id,
                        field_name,
                    ),
                    crate::product::lowering::product_note_field_source(&note_source, field_name),
                );
            }
        }
    }

    fn normalize_with_dirs(
        &self,
        base_dir: impl Into<PathBuf>,
        media_store_dir: impl Into<PathBuf>,
        mut options: ProjectNormalizeOptions,
    ) -> Result<ProjectNormalizeOutput, ProjectNormalizeError> {
        let base_dir = base_dir.into();
        let media_store_dir = media_store_dir.into();
        options.base_dir = options.base_dir.or(Some(base_dir.clone()));
        options.media_store_dir = options.media_store_dir.or(Some(media_store_dir.clone()));
        let mut lowering = self.lower_with_project_error()?;
        let lowering_diagnostics =
            map_lowering_diagnostics(std::mem::take(&mut lowering.lowering_diagnostics));
        self.apply_note_source_paths(&mut lowering);
        if let Some(deck) = &self.deck_source {
            let media = deck
                .registered_media()
                .values()
                .map(|media| media.to_authoring_media(&base_dir))
                .collect::<anyhow::Result<Vec<_>>>()
                .map_err(|error| ProjectNormalizeError {
                    message: "prepare deck media".into(),
                    diagnostics: vec![Diagnostic {
                        code: DiagnosticCode::new("PROJECT.DECK_MEDIA_FAILED"),
                        severity: Severity::Error,
                        message: error.to_string(),
                        source: Some(SourcePath::new("project.deck.media")),
                        help: Some("inspect deck media registrations and media paths".into()),
                    }],
                    normalized_ir: None,
                })?;
            lowering.authoring_document.media.extend(media);
        } else {
            let media = product_media_to_path_backed_authoring_media(self.media.media(), &base_dir)
                .map_err(|error| ProjectNormalizeError {
                    message: error.message,
                    diagnostics: error.diagnostics,
                    normalized_ir: None,
                })?;
            lowering.authoring_document.media.extend(media);
            record_project_media_source_paths(&mut lowering, self.media.media());
        }
        let source_map = lowering.source_map.clone();
        let result = normalize_with_options(
            NormalizationRequest::new(lowering.authoring_document),
            NormalizeOptions {
                base_dir,
                media_store_dir,
                media_policy: options.to_authoring_media_policy(),
            },
        );
        let result_status = result.result_status.clone();
        let diagnostics = combine_lowering_and_normalization_diagnostics(
            lowering_diagnostics,
            result
                .diagnostics
                .items
                .into_iter()
                .map(|item| normalization_diagnostic_to_product_diagnostic(item, &source_map))
                .collect(),
        );
        if result_status != "success" {
            return Err(ProjectNormalizeError {
                message: format!("normalization failed with status {result_status}"),
                diagnostics,
                normalized_ir: result.normalized_ir.map(Box::new),
            });
        }
        let normalized_ir = result.normalized_ir.ok_or_else(|| ProjectNormalizeError {
            message: "normalization did not produce normalized_ir".into(),
            diagnostics: diagnostics.clone(),
            normalized_ir: None,
        })?;
        Ok(ProjectNormalizeOutput {
            normalized_ir,
            diagnostics,
        })
    }

    fn lower_with_project_error(&self) -> Result<LoweringPlan, ProjectNormalizeError> {
        let product = if let Some(deck) = &self.deck_source {
            deck.clone()
                .into_product_document()
                .map_err(|error| ProjectNormalizeError {
                    message: "lower deck product document".into(),
                    diagnostics: vec![Diagnostic {
                        code: DiagnosticCode::new("PROJECT.DECK_LOWER_FAILED"),
                        severity: Severity::Error,
                        message: error.to_string(),
                        source: Some(SourcePath::new("project.deck")),
                        help: Some("inspect deck notes before lowering".into()),
                    }],
                    normalized_ir: None,
                })?
        } else {
            self.to_product_document()
        };

        product.lower().map_err(|error| {
            let diagnostics = map_product_lowering_error(&error);
            ProjectNormalizeError {
                message: if self.deck_source.is_some() {
                    "lower deck product document".into()
                } else {
                    "lower product document".into()
                },
                diagnostics,
                normalized_ir: None,
            }
        })
    }
}

fn custom_generation_rule(
    rule: &crate::product::GenerationRule,
) -> crate::product::model::CustomGenerationRule {
    match rule {
        crate::product::GenerationRule::AnkiDefault => {
            crate::product::model::CustomGenerationRule::AnkiDefault
        }
        crate::product::GenerationRule::All(fields) => {
            crate::product::model::CustomGenerationRule::All {
                fields: fields
                    .iter()
                    .map(|field| field.as_str().to_string())
                    .collect(),
            }
        }
        crate::product::GenerationRule::Any(fields) => {
            crate::product::model::CustomGenerationRule::Any {
                fields: fields
                    .iter()
                    .map(|field| field.as_str().to_string())
                    .collect(),
            }
        }
        crate::product::GenerationRule::Cloze { field } => {
            crate::product::model::CustomGenerationRule::Cloze {
                field: field.as_str().to_string(),
            }
        }
    }
}

fn duplicate_notetype_message(
    id: &str,
    first_index: usize,
    first_name: Option<&str>,
    duplicate_index: usize,
    duplicate_name: Option<&str>,
) -> String {
    format!(
        "duplicate note type id '{id}' at project.note_types[{duplicate_index}]{}; first definition is project.note_types[{first_index}]{}",
        display_name_suffix(duplicate_name),
        display_name_suffix(first_name),
    )
}

fn display_name_suffix(name: Option<&str>) -> String {
    name.map(|name| format!(" ({name})")).unwrap_or_default()
}

fn custom_note_fields_for_authoring(
    project: &Project,
    note: &crate::product::Note,
) -> BTreeMap<String, String> {
    let rendered = note.rendered_fields();
    let Some(note_type) = project
        .note_types
        .iter()
        .find(|note_type| note_type.id() == note.note_type_id())
    else {
        return rendered;
    };

    let name_by_key = note_type
        .fields()
        .iter()
        .map(|field| (field.key_ref().as_str(), field.name()))
        .collect::<BTreeMap<_, _>>();
    let field_names = note_type
        .fields()
        .iter()
        .map(|field| field.name())
        .collect::<BTreeSet<_>>();

    let mut fields = BTreeMap::new();
    let mut field_priorities = BTreeMap::new();
    for (field_key_or_name, value) in rendered {
        let is_visible_name = field_names.contains(field_key_or_name.as_str());
        let field_name = if is_visible_name {
            field_key_or_name
        } else {
            name_by_key
                .get(field_key_or_name.as_str())
                .copied()
                .unwrap_or(field_key_or_name.as_str())
                .to_string()
        };
        let priority = u8::from(is_visible_name);
        if field_priorities
            .get(&field_name)
            .is_some_and(|existing| *existing > priority)
        {
            continue;
        }
        field_priorities.insert(field_name.clone(), priority);
        fields.insert(field_name, value);
    }
    fields
}

fn product_media_to_authoring_media<'a>(
    media: impl Iterator<Item = &'a crate::product::media_registry::ProductMedia>,
) -> anyhow::Result<Vec<crate::AuthoringMedia>> {
    media.map(product_media_item_to_authoring_media).collect()
}

fn record_project_media_source_paths<'a>(
    plan: &mut LoweringPlan,
    media: impl Iterator<Item = &'a crate::product::media_registry::ProductMedia>,
) {
    for item in media {
        crate::product::lowering::record_media_source_path(
            &mut plan.source_map,
            &item.id,
            &item.export_filename,
        );
    }
}

fn product_media_to_path_backed_authoring_media<'a>(
    media: impl Iterator<Item = &'a crate::product::media_registry::ProductMedia>,
    media_input_dir: &Path,
) -> Result<Vec<crate::AuthoringMedia>, ProductMediaPrepareError> {
    let mut prepared = Vec::new();
    let mut diagnostics = Vec::new();

    for item in media {
        match product_media_item_to_path_backed_authoring_media(item, media_input_dir) {
            Ok(media) => prepared.push(media),
            Err(mut error) => diagnostics.append(&mut error.diagnostics),
        }
    }

    if diagnostics.is_empty() {
        Ok(prepared)
    } else {
        Err(ProductMediaPrepareError {
            message: "prepare product media".into(),
            diagnostics,
        })
    }
}

fn product_media_item_to_authoring_media(
    media: &crate::product::media_registry::ProductMedia,
) -> anyhow::Result<crate::AuthoringMedia> {
    let source = match &media.source {
        crate::product::media_registry::ProductMediaSource::File { path } => {
            media
                .verify_registered_source()
                .map_err(|diagnostic| anyhow::anyhow!(diagnostic.message))?;
            anyhow::ensure!(
                media.observed_size_bytes()
                    <= crate::product::media_registry::INLINE_MEDIA_LIMIT_BYTES as u64,
                "MEDIA.INLINE_TOO_LARGE: project.media[{filename:?}] has {} bytes, above inline limit {}",
                media.observed_size_bytes(),
                crate::product::media_registry::INLINE_MEDIA_LIMIT_BYTES,
                filename = &media.export_filename,
            );
            let bytes = std::fs::read(path)
                .with_context(|| format!("read media source file: {}", path.display()))?;
            crate::AuthoringMediaSource::InlineBytes {
                data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            }
        }
        crate::product::media_registry::ProductMediaSource::InlineBytes { data_base64, .. } => {
            crate::AuthoringMediaSource::InlineBytes {
                data_base64: data_base64.clone(),
            }
        }
    };

    Ok(crate::AuthoringMedia {
        id: media.id.clone(),
        desired_filename: media.export_filename.clone(),
        source,
        declared_mime: media.declared_mime.clone(),
    })
}

fn product_media_item_to_path_backed_authoring_media(
    media: &crate::product::media_registry::ProductMedia,
    media_input_dir: &Path,
) -> Result<crate::AuthoringMedia, ProductMediaPrepareError> {
    let source = match &media.source {
        crate::product::media_registry::ProductMediaSource::File { path } => {
            media
                .verify_registered_source()
                .map_err(ProductMediaPrepareError::from_source_diagnostic)?;
            ensure_safe_product_media_input_dir(media_input_dir)
                .map_err(ProductMediaPrepareError::from_prepare_error)?;
            let target = media_input_dir.join(&media.export_filename);
            ensure_not_symlink(&target).map_err(ProductMediaPrepareError::from_prepare_error)?;
            if !paths_are_same_file(path, &target)
                .map_err(ProductMediaPrepareError::from_prepare_error)?
            {
                std::fs::copy(path, &target).map_err(|err| {
                    let code = if err.kind() == std::io::ErrorKind::NotFound {
                        "MEDIA.SOURCE_MISSING"
                    } else {
                        "PROJECT.PRODUCT_MEDIA_FAILED"
                    };
                    ProductMediaPrepareError::single(
                        code,
                        format!(
                            "copy media source {} to {}: {err}",
                            path.display(),
                            target.display()
                        ),
                        media.export_filename.clone(),
                    )
                })?;
            }
            crate::AuthoringMediaSource::Path {
                path: media.export_filename.clone(),
            }
        }
        crate::product::media_registry::ProductMediaSource::InlineBytes { data_base64, .. } => {
            crate::AuthoringMediaSource::InlineBytes {
                data_base64: data_base64.clone(),
            }
        }
    };

    Ok(crate::AuthoringMedia {
        id: media.id.clone(),
        desired_filename: media.export_filename.clone(),
        source,
        declared_mime: media.declared_mime.clone(),
    })
}

fn paths_are_same_file(left: &Path, right: &Path) -> anyhow::Result<bool> {
    if !right.exists() {
        return Ok(false);
    }
    let left_metadata = std::fs::metadata(left)
        .with_context(|| format!("stat media source: {}", left.display()))?;
    let right_metadata = std::fs::metadata(right)
        .with_context(|| format!("stat media staging target: {}", right.display()))?;
    if let (Some(left_identity), Some(right_identity)) = (
        metadata_file_identity(&left_metadata),
        metadata_file_identity(&right_metadata),
    ) {
        return Ok(left_identity == right_identity);
    }

    let left = left
        .canonicalize()
        .with_context(|| format!("canonicalize media source: {}", left.display()))?;
    let right = right
        .canonicalize()
        .with_context(|| format!("canonicalize media staging target: {}", right.display()))?;
    Ok(left == right)
}

#[cfg(unix)]
fn metadata_file_identity(metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;

    Some((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn metadata_file_identity(metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::windows::fs::MetadataExt;

    metadata
        .volume_serial_number()
        .zip(metadata.file_index())
        .map(|(volume, index)| (u64::from(volume), index))
}

#[cfg(not(any(unix, windows)))]
fn metadata_file_identity(_metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    None
}

fn ensure_safe_product_media_input_dir(media_input_dir: &Path) -> anyhow::Result<()> {
    match std::fs::symlink_metadata(media_input_dir) {
        Ok(metadata) => validate_product_media_input_dir(media_input_dir, &metadata)?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(media_input_dir).with_context(|| {
                format!(
                    "create media input directory: {}",
                    media_input_dir.display()
                )
            })?;
            let metadata = std::fs::symlink_metadata(media_input_dir).with_context(|| {
                format!("stat media input directory: {}", media_input_dir.display())
            })?;
            validate_product_media_input_dir(media_input_dir, &metadata)?;
        }
        Err(err) => {
            return Err(err).with_context(|| {
                format!("stat media input directory: {}", media_input_dir.display())
            });
        }
    }
    Ok(())
}

fn validate_product_media_input_dir(
    media_input_dir: &Path,
    metadata: &std::fs::Metadata,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !metadata.file_type().is_symlink(),
        "media input directory must not be a symlink: {}",
        media_input_dir.display()
    );
    anyhow::ensure!(
        metadata.is_dir(),
        "media input path must be a directory: {}",
        media_input_dir.display()
    );
    Ok(())
}

fn ensure_not_symlink(path: &Path) -> anyhow::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            anyhow::ensure!(
                !metadata.file_type().is_symlink(),
                "media input target must not be a symlink: {}",
                path.display()
            );
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err)
                .with_context(|| format!("stat media input target: {}", path.display()));
        }
    }
    Ok(())
}

impl From<crate::deck::Deck> for Project {
    fn from(deck: crate::deck::Deck) -> Self {
        let mut project = Project::new(deck.name().to_string());
        if let Some(stable_id) = deck.stable_id() {
            project = project.stable_id(stable_id.to_string());
        }
        project = project.default_deck(deck.name().to_string());
        project.deck_source = Some(deck);
        project
    }
}

struct ProjectNormalizeOutput {
    normalized_ir: authoring_core::NormalizedIr,
    diagnostics: Vec<Diagnostic>,
}

struct ArtifactWorkspace {
    path: PathBuf,
    temp_dir: Option<TempDir>,
    persist_temp: bool,
}

impl ArtifactWorkspace {
    fn new(options: &BuildOptions, started: Instant) -> Result<Self, BuildError> {
        if let Some(artifacts_dir) = options.artifacts_dir.clone() {
            return Ok(Self {
                path: artifacts_dir,
                temp_dir: None,
                persist_temp: false,
            });
        }

        let temp_dir = tempfile::Builder::new()
            .prefix("anki-forge-project-build-")
            .tempdir()
            .map_err(|err| {
                BuildError::new(
                    failure_report(started, "PROJECT.ARTIFACTS_DIR_FAILED", err.to_string()),
                    BuildFailureCause::Io,
                )
            })?;
        let path = temp_dir.path().to_path_buf();

        Ok(Self {
            path,
            temp_dir: Some(temp_dir),
            persist_temp: options.output.is_none(),
        })
    }

    fn persist_if_requested(self) {
        if self.persist_temp {
            if let Some(temp_dir) = self.temp_dir {
                let _persisted_artifacts_dir = temp_dir.into_path();
            }
        }
    }
}

#[derive(Debug)]
struct ProjectNormalizeError {
    message: String,
    diagnostics: Vec<Diagnostic>,
    normalized_ir: Option<Box<authoring_core::NormalizedIr>>,
}

struct ProductMediaPrepareError {
    message: String,
    diagnostics: Vec<Diagnostic>,
}

impl ProductMediaPrepareError {
    fn from_source_diagnostic(
        diagnostic: crate::product::media_registry::ProductMediaSourceDiagnostic,
    ) -> Self {
        Self {
            message: "prepare product media".into(),
            diagnostics: vec![Diagnostic {
                code: DiagnosticCode::new(diagnostic.code),
                severity: Severity::Error,
                message: diagnostic.message,
                source: Some(SourcePath::new(diagnostic.source_path)),
                help: Some("inspect product media registrations and source files".into()),
            }],
        }
    }

    fn from_prepare_error(error: anyhow::Error) -> Self {
        Self {
            message: "prepare product media".into(),
            diagnostics: vec![Diagnostic {
                code: DiagnosticCode::new("PROJECT.PRODUCT_MEDIA_FAILED"),
                severity: Severity::Error,
                message: error.to_string(),
                source: Some(SourcePath::new("project.media")),
                help: Some("inspect product media registrations and media paths".into()),
            }],
        }
    }

    fn single(code: &'static str, message: String, export_filename: String) -> Self {
        Self {
            message: "prepare product media".into(),
            diagnostics: vec![Diagnostic {
                code: DiagnosticCode::new(code),
                severity: Severity::Error,
                message,
                source: Some(SourcePath::new(format!(
                    "project.media[{export_filename:?}]"
                ))),
                help: Some("inspect product media registrations and source files".into()),
            }],
        }
    }
}

impl std::fmt::Display for ProjectNormalizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.diagnostics.is_empty() {
            return f.write_str(&self.message);
        }

        let codes = self
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}: {}", self.message, codes)
    }
}

impl std::error::Error for ProjectNormalizeError {}

fn normalization_diagnostic_to_product_diagnostic(
    item: authoring_core::model::DiagnosticItem,
    source_map: &ProductSourceMap,
) -> Diagnostic {
    let source = item.path.as_deref().and_then(|path| {
        source_map
            .source_for_diagnostic_path(path)
            .map(SourcePath::new)
    });
    Diagnostic {
        code: DiagnosticCode::new(item.code),
        severity: severity_from_level(&item.level),
        message: item.summary,
        source,
        help: None,
    }
}

fn map_product_lowering_error(error: &ProductLoweringError) -> Vec<Diagnostic> {
    map_product_diagnostics(error.product_diagnostics.clone())
        .into_iter()
        .chain(map_lowering_diagnostics(error.lowering_diagnostics.clone()))
        .collect()
}

fn map_product_diagnostics(diagnostics: Vec<ProductDiagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| Diagnostic {
            code: DiagnosticCode::new(diagnostic.code),
            severity: Severity::Error,
            message: diagnostic.message,
            source: Some(SourcePath::new("project.lower")),
            help: None,
        })
        .collect()
}

fn map_lowering_diagnostics(diagnostics: Vec<LoweringDiagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| Diagnostic {
            code: DiagnosticCode::new(diagnostic.code),
            severity: Severity::Warning,
            message: diagnostic.message,
            source: Some(SourcePath::new("project.lower")),
            help: None,
        })
        .collect()
}

fn failure_report(started: Instant, code: &str, message: String) -> BuildReport {
    BuildReport {
        artifact: None,
        counts: BuildCounts::default(),
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new(code),
            severity: Severity::Error,
            message,
            source: Some(SourcePath::new("project.build")),
            help: None,
        }],
        metrics: BuildMetrics {
            duration: started.elapsed(),
        },
        inspect: None,
        status: "error".into(),
    }
}

fn severity_from_level(level: &str) -> Severity {
    match level {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    }
}

fn deck_validation_diagnostic_to_project_diagnostic(
    diagnostic: &crate::deck::ValidationDiagnostic,
) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(deck_validation_code(&diagnostic.code)),
        severity: severity_from_deck_validation(&diagnostic.severity),
        message: diagnostic.message.clone(),
        source: Some(SourcePath::new("project.deck")),
        help: None,
    }
}

fn deck_validation_code(code: &crate::deck::ValidationCode) -> &'static str {
    match code {
        crate::deck::ValidationCode::MissingStableId => "DECK.MISSING_STABLE_ID",
        crate::deck::ValidationCode::DuplicateStableId => "DECK.DUPLICATE_STABLE_ID",
        crate::deck::ValidationCode::BlankStableId => "DECK.BLANK_STABLE_ID",
        crate::deck::ValidationCode::EmptyIoMasks => "DECK.EMPTY_IO_MASKS",
        crate::deck::ValidationCode::UnknownMediaRef => "DECK.UNKNOWN_MEDIA_REF",
        crate::deck::ValidationCode::NoteLevelIdentityOverrideUsed => {
            "DECK.NOTE_LEVEL_IDENTITY_OVERRIDE_USED"
        }
        crate::deck::ValidationCode::IdentityDuplicatePayload => "DECK.IDENTITY_DUPLICATE_PAYLOAD",
        crate::deck::ValidationCode::IdentityCollision => "DECK.IDENTITY_COLLISION",
        crate::deck::ValidationCode::StableIdDuplicate => "DECK.STABLE_ID_DUPLICATE",
    }
}

fn severity_from_deck_validation(severity: &str) -> Severity {
    match severity {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    }
}

fn combine_lowering_and_normalization_diagnostics(
    mut lowering_diagnostics: Vec<Diagnostic>,
    normalization_diagnostics: Vec<Diagnostic>,
) -> Vec<Diagnostic> {
    lowering_diagnostics.extend(normalization_diagnostics);
    lowering_diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostic(code: &str, severity: Severity) -> Diagnostic {
        Diagnostic {
            code: DiagnosticCode::new(code),
            severity,
            message: code.into(),
            source: None,
            help: None,
        }
    }

    #[test]
    fn normalization_failure_diagnostics_include_lowering_diagnostics() {
        let diagnostics = combine_lowering_and_normalization_diagnostics(
            vec![diagnostic("LOWERING.WARNING", Severity::Warning)],
            vec![diagnostic("NORMALIZE.ERROR", Severity::Error)],
        );

        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>(),
            vec!["LOWERING.WARNING", "NORMALIZE.ERROR"]
        );
    }

    #[test]
    fn artifact_workspace_for_output_removes_internal_tempdir_on_drop() {
        let temp_path = {
            let workspace = ArtifactWorkspace::new(
                &BuildOptions::new().output(std::env::temp_dir().join("deck.apkg")),
                Instant::now(),
            )
            .expect("workspace");

            assert!(workspace.temp_dir.is_some());
            assert!(!workspace.persist_temp);
            assert!(workspace.path.exists());
            workspace.path.clone()
        };

        assert!(!temp_path.exists());
    }
}
