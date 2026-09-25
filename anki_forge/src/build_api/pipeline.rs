use std::{collections::BTreeMap, error::Error, fs::File, time::Instant};

use super::{BuildError, BuildErrorKind as Kind, BuildOptions, BuildOutput, OutputTarget};
use crate::{
    product::model::{
        ProductCustomNoteTypeV2, ProductCustomNoteV2, ProductDocument, ProductDocumentV2Payload,
        ProductFieldContentV2, ProductFieldV2, ProductGenerationRuleV2, ProductMediaSourceV2,
        ProductMediaV2, ProductNoteTypeV2, ProductNoteV2, ProductTemplateV2,
    },
    schema::GenerationRule,
    Project,
};

impl Project {
    /// Builds and inspects an APKG using the explicit output request. Successful
    /// output always owns an artifact; a report alone never represents success.
    pub fn build(&self, options: BuildOptions) -> Result<BuildOutput, BuildError> {
        let started = Instant::now();
        let (candidate, mut report) = self.prepare_candidate(&options)?;
        if report
            .comparison
            .as_ref()
            .is_some_and(|c| !c.policy().allows_publication())
        {
            let mut error = BuildError::new(
                Kind::PolicyBlocked,
                "UPDATE.POLICY_BLOCKED",
                "completed comparison contains unaccepted blocking risks",
            );
            error.report = Box::new(report);
            return Err(error);
        }
        let artifact = match options.output {
            OutputTarget::Temporary => candidate,
            OutputTarget::Persistent(path) => candidate.persist_to(path).map_err(|cause| {
                let publication = cause.publication().clone();
                let mut error = BuildError::new(Kind::Publication, cause.code(), "publish APKG")
                    .caused_by(cause);
                error.report = Box::new(report.clone());
                error.publications.push(publication);
                error
            })?,
        };
        report.duration = started.elapsed();
        Ok(BuildOutput { artifact, report })
    }

    pub(crate) fn prepare_candidate(
        &self,
        options: &BuildOptions,
    ) -> Result<(super::ApkgArtifact, super::BuildReport), BuildError> {
        let started = Instant::now();
        crate::project::validate_deck(&self.default_deck).map_err(|cause| {
            BuildError::new(Kind::Configuration, cause.code(), "invalid default deck")
                .caused_by(cause)
        })?;
        if self.name.trim().is_empty() || self.name.chars().any(char::is_control) {
            return Err(BuildError::new(
                Kind::Configuration,
                "BUILD.NAME_INVALID",
                "project display name must be nonempty and contain no control characters",
            ));
        }
        if matches!(&options.output, OutputTarget::Persistent(path) if path.as_os_str().is_empty())
        {
            return Err(BuildError::new(
                Kind::Configuration,
                "BUILD.OUTPUT_INVALID",
                "output path cannot be empty",
            ));
        }
        if matches!(options.mode, super::BuildMode::Create) && options.update_policy.is_some() {
            return Err(BuildError::new(
                Kind::Configuration,
                "BUILD.UPDATE_POLICY_WITHOUT_BASELINE",
                "an explicit update policy requires update_from",
            ));
        }
        let baseline = match &options.mode {
            super::BuildMode::Create => None,
            super::BuildMode::Update(path) => {
                if path.as_os_str().is_empty() {
                    return Err(BuildError::new(
                        Kind::Configuration,
                        "BUILD.BASELINE_INVALID",
                        "baseline path cannot be empty",
                    ));
                }
                if let OutputTarget::Persistent(output) = &options.output {
                    let aliases_baseline = output == path
                        || (output.exists()
                            && path.exists()
                            && same_file::is_same_file(output, path).map_err(|cause| {
                                BuildError::new(
                                    Kind::Configuration,
                                    "BUILD.OUTPUT_INVALID",
                                    "could not distinguish output from update baseline",
                                )
                                .caused_by(cause)
                            })?);
                    if aliases_baseline {
                        return Err(BuildError::new(
                            Kind::Configuration,
                            "BUILD.OUTPUT_INVALID",
                            "output must not replace the original update baseline; choose a separate destination",
                        ));
                    }
                }
                let (summary, envelope) = crate::writer_core::inspect::inspect_native_package(
                    path,
                    &options.inspect_limits,
                )
                .map_err(|cause| inspection_error(cause, "inspect baseline APKG"))?;
                if summary.observation_status != "complete" {
                    return Err(BuildError::new(
                        Kind::Validation,
                        "UPDATE.BASELINE_INCOMPLETE",
                        "required baseline inspection was incomplete",
                    ));
                }
                Some((summary, envelope))
            }
        };
        let baseline_counts = baseline.as_ref().map(|(summary, _)| super::BuildCounts {
            notes: summary.notes,
            cards: summary.cards,
            media: summary.media,
        });
        match self.prepare_from_baseline(options, baseline) {
            Ok((candidate, mut report)) => {
                report.baseline_counts = baseline_counts;
                report.duration = started.elapsed();
                Ok((candidate, report))
            }
            Err(mut error) => {
                error.report.baseline_counts = baseline_counts;
                error.report.duration = started.elapsed();
                Err(error)
            }
        }
    }

    fn prepare_from_baseline(
        &self,
        options: &BuildOptions,
        baseline: Option<(
            crate::writer_core::inspect::ApkgInspectSummary,
            super::identity::IdentityEnvelope,
        )>,
    ) -> Result<(super::ApkgArtifact, super::BuildReport), BuildError> {
        let identity = super::identity::PackageIdentity::prepare(
            self,
            baseline.as_ref().map(|(_, envelope)| &envelope.identity),
        )?;
        let inputs = tempfile::Builder::new()
            .prefix("ankiforge-input-")
            .tempdir()
            .map_err(|cause| {
                BuildError::new(
                    Kind::Io,
                    "BUILD.WORKSPACE_FAILED",
                    "create private build inputs",
                )
                .caused_by(cause)
            })?;
        let assets_dir = inputs.path().join("assets");
        std::fs::create_dir(&assets_dir).map_err(|cause| {
            BuildError::new(
                Kind::Io,
                "BUILD.MEDIA_STAGING_FAILED",
                "create asset input directory",
            )
            .caused_by(cause)
        })?;
        let mut media = Vec::new();
        for asset in self.assets.values() {
            let mut input = asset.snapshot.reader().map_err(|cause| {
                BuildError::new(Kind::Io, cause.code(), "read owned media").caused_by(cause)
            })?;
            let path = assets_dir.join(asset.filename());
            let mut output = File::create(&path).map_err(|cause| {
                BuildError::new(
                    Kind::Io,
                    "BUILD.MEDIA_STAGING_FAILED",
                    "create private media input",
                )
                .caused_by(cause)
            })?;
            std::io::copy(&mut input, &mut output).map_err(|cause| {
                BuildError::new(
                    Kind::Io,
                    "BUILD.MEDIA_STAGING_FAILED",
                    "stage owned media bytes",
                )
                .caused_by(cause)
            })?;
            media.push(ProductMediaV2 {
                id: asset.filename().to_owned(),
                source: ProductMediaSourceV2::File {
                    path: format!("assets/{}", asset.filename()),
                },
                export_as: asset.filename().to_owned(),
                source_path: Some(format!("project.assets[{:?}]", asset.filename())),
            });
        }
        let note_types = self
            .models
            .values()
            .map(|model| {
                let fields = model
                    .fields()
                    .iter()
                    .map(|field| ProductFieldV2 {
                        name: field.display_name().into(),
                        key: field.key().as_str().into(),
                        identity: false,
                        sort: field.is_sort(),
                        required: field.is_required(),
                        source_path: None,
                    })
                    .collect();
                let bindings = model
                    .fields()
                    .iter()
                    .map(|field| (field.key().as_str(), field.display_name()))
                    .collect::<BTreeMap<_, _>>();
                let compile = |source: &str| {
                    let parsed = crate::authoring_core::parse_template(source);
                    crate::authoring_core::template_parser::rewrite_fields(
                        source, &parsed, &bindings,
                    )
                };
                let templates = model
                    .templates()
                    .iter()
                    .map(|template| ProductTemplateV2 {
                        name: template.display_name().into(),
                        key: template.key().as_str().into(),
                        front: compile(template.front_source()),
                        back: compile(template.back_source()),
                        browser_front: template.browser_front_source().map(compile),
                        browser_back: template.browser_back_source().map(compile),
                        target_deck: template.target_deck_name().map(str::to_owned),
                        source_path: None,
                        generation_rule: match template.generation_rule() {
                            GenerationRule::AnkiDefault => None,
                            GenerationRule::All(keys) => Some(ProductGenerationRuleV2::All {
                                fields: keys.iter().map(ToString::to_string).collect(),
                            }),
                            GenerationRule::Any(keys) => Some(ProductGenerationRuleV2::Any {
                                fields: keys.iter().map(ToString::to_string).collect(),
                            }),
                        },
                    })
                    .collect();
                ProductNoteTypeV2::Custom(ProductCustomNoteTypeV2 {
                    stock_kind: model.stock_kind().map(str::to_owned),
                    id: model_id(model.key()),
                    name: Some(model.display_name().into()),
                    note_type_kind: Some(
                        if model.cloze_field().is_some() {
                            "cloze"
                        } else {
                            "normal"
                        }
                        .into(),
                    ),
                    cloze_field: model.cloze_field().map(ToString::to_string),
                    fields,
                    templates,
                    identity: None,
                    css: Some(model.css().into()),
                    source_path: Some(format!("project.models[{:?}]", model.key())),
                })
            })
            .collect();
        let notes = self
            .notes
            .iter()
            .map(|(key, note)| {
                let mut fields: BTreeMap<_, _> = note
                    .fields
                    .iter()
                    .map(|(key, value)| {
                        (
                            key.to_string(),
                            ProductFieldContentV2::Html {
                                value: value.render(),
                            },
                        )
                    })
                    .collect();
                if let Some(occlusion) = &note.occlusion {
                    fields.insert(
                        "occlusion".into(),
                        ProductFieldContentV2::Html {
                            value: occlusion.render(&identity.mask_ordinals(key)),
                        },
                    );
                    fields.insert(
                        "image".into(),
                        ProductFieldContentV2::Html {
                            value: occlusion.image.image().render(),
                        },
                    );
                }
                ProductNoteV2::Custom(ProductCustomNoteV2 {
                    note_type_id: model_id(note.model.key()),
                    stable_id: Some(key.clone()),
                    deck_name: note.deck.as_ref().unwrap_or(&self.default_deck).clone(),
                    fields,
                    tags: note.tags.clone(),
                    source_path: Some(format!("project.notes[{key:?}]")),
                })
            })
            .collect();
        let document = ProductDocument::from_authored_payload(
            self.namespace.clone(),
            Some(self.default_deck.clone()),
            ProductDocumentV2Payload {
                version: 3,
                note_types,
                notes,
                media,
                transport_diagnostics: Vec::new(),
            },
        );
        let (candidate, mut report) =
            super::candidate::generate(self, &document, inputs.path(), identity)?;
        let (inspected, envelope) = crate::writer_core::inspect::inspect_native_package(
            candidate.path(),
            &options.inspect_limits,
        )
        .map_err(|cause| {
            let mut error = inspection_error(cause, "inspect candidate APKG");
            error.report = Box::new(report.clone());
            error
        })?;
        if inspected.observation_status != "complete" {
            let mut error = BuildError::new(
                Kind::Validation,
                "BUILD.INSPECT_INCOMPLETE",
                "required candidate inspection was incomplete",
            );
            error.report = Box::new(report);
            return Err(error);
        }
        report.counts.notes = inspected.notes;
        report.counts.cards = inspected.cards;
        report.counts.media = inspected.media;
        if let Some((summary, baseline)) = baseline {
            let comparison = crate::update::ComparisonReport::analyze(
                &baseline,
                &envelope,
                super::BuildCounts {
                    notes: summary.notes,
                    cards: summary.cards,
                    media: summary.media,
                },
                report.counts,
                &options.update_policy.clone().unwrap_or_default(),
            );
            report
                .diagnostics
                .extend(comparison.diagnostics().iter().cloned());
            report.comparison = Some(comparison);
        }
        Ok((candidate, report))
    }
}

fn model_id(key: &str) -> String {
    format!("model:{key}")
}

fn inspection_error(cause: crate::writer_core::InspectError, message: &str) -> BuildError {
    let (mut kind, mut code) = if cause.limit_exceeded().is_some() {
        (Kind::ResourceLimit, "INSPECT.RESOURCE_LIMIT_EXCEEDED")
    } else {
        (Kind::Validation, "BUILD.INSPECT_FAILED")
    };
    let mut source = cause.source();
    while let Some(error) = source {
        if let Some(evidence) = error.downcast_ref::<super::identity::EvidenceError>() {
            code = evidence.code;
        }
        if error.is::<std::io::Error>() && kind != Kind::ResourceLimit {
            kind = Kind::Io;
        }
        source = error.source();
    }
    BuildError::new(kind, code, message).caused_by(cause)
}
