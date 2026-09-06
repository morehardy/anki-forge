//! The two build inputs share execution, never mutable authoring state.
use super::*;

#[derive(Clone, Copy)]
pub(super) enum BuildInput<'a> {
    Project(&'a Project),
    Document(&'a ProductDocument),
}

impl BuildInput<'_> {
    pub(super) fn stable_id(&self) -> Option<&str> {
        match self {
            Self::Project(project) => project.stable_id.as_deref(),
            Self::Document(document) => Some(document.document_id()),
        }
    }

    pub(super) fn validate(&self) -> ValidationReport {
        match self {
            Self::Project(project) => project.validate(),
            Self::Document(_) => ValidationReport {
                diagnostics: Vec::new(),
            },
        }
    }

    pub(super) fn resolved_note_identities(
        &self,
    ) -> BTreeMap<String, crate::update_safety::model::ResolvedNoteIdentity> {
        match self {
            Self::Project(project) => project.resolved_note_identities(),
            Self::Document(document) => document
                .product_v2()
                .map(product_v2_resolved_note_identities)
                .unwrap_or_default(),
        }
    }

    fn lower_with_project_error(&self) -> Result<LoweringPlan, ProjectNormalizeError> {
        let result = match self {
            Self::Project(project) => project.to_product_document().lower().map(|mut plan| {
                project.apply_note_source_paths(&mut plan);
                project.apply_notetype_source_paths(&mut plan);
                plan
            }),
            Self::Document(document) => document.lower(),
        };
        result.map_err(|error| ProjectNormalizeError {
            message: "lower product document".into(),
            diagnostics: map_product_lowering_error(&error),
            normalized_ir: None,
            media_source_modes: BTreeMap::new(),
        })
    }

    pub(super) fn normalize_with_dirs(
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
        let product_diagnostics =
            map_product_diagnostics(std::mem::take(&mut lowering.product_diagnostics));
        let lowering_diagnostics =
            map_lowering_diagnostics(std::mem::take(&mut lowering.lowering_diagnostics));
        let media_mode = options.media_mode;
        // Keep our input copies alive through ingestion, including error paths.
        // Caller-owned source paths and aliases never enter this ownership set.
        let _media_input_files = if let Self::Project(project) = self {
            let PreparedProductMedia { media, input_files } = match media_mode {
                ProjectMediaMode::PathBacked => {
                    product_media_to_path_backed_authoring_media(project.media.media(), &base_dir)
                }
                ProjectMediaMode::SelfContained => product_media_to_self_contained_authoring_media(
                    project.media.media(),
                )
                .map(|media| PreparedProductMedia {
                    media,
                    input_files: Vec::new(),
                }),
            }
            .map_err(|error| ProjectNormalizeError {
                message: error.message,
                diagnostics: error.diagnostics,
                normalized_ir: None,
                media_source_modes: BTreeMap::new(),
            })?;
            lowering.authoring_document.media.extend(media);
            record_project_media_source_paths(&mut lowering, project.media.media());
            input_files
        } else {
            Vec::new()
        };
        if media_mode == ProjectMediaMode::SelfContained {
            let inline_limit = options.to_authoring_media_policy().inline_bytes_max;
            self_contain_authoring_path_media(
                &mut lowering.authoring_document.media,
                &base_dir,
                inline_limit,
                &lowering.source_map,
            )
            .map_err(|error| ProjectNormalizeError {
                message: error.message,
                diagnostics: error.diagnostics,
                normalized_ir: None,
                media_source_modes: BTreeMap::new(),
            })?;
        }
        let source_map = lowering.source_map;
        let media_source_modes = authoring_media_source_modes(&lowering.authoring_document.media);
        let duplicate_notetype_media_diagnostics = duplicate_notetype_media_reference_diagnostics(
            &lowering.authoring_document,
            &source_map,
        );
        let result = normalize_with_options(
            NormalizationRequest::new(lowering.authoring_document),
            NormalizeOptions {
                base_dir,
                media_store_dir,
                media_policy: options.to_authoring_media_policy(),
            },
        );
        let result_status = result.result_status.clone();
        let mut normalization_diagnostics = result
            .diagnostics
            .items
            .into_iter()
            .map(|item| normalization_diagnostic_to_product_diagnostic(item, &source_map))
            .collect::<Vec<_>>();
        if normalization_diagnostics
            .iter()
            .any(|item| item.code.as_str() == "PHASE3.DUPLICATE_NOTETYPE_ID")
        {
            normalization_diagnostics.extend(duplicate_notetype_media_diagnostics);
        }
        let diagnostics = combine_lowering_and_normalization_diagnostics(
            product_diagnostics
                .into_iter()
                .chain(lowering_diagnostics)
                .collect(),
            normalization_diagnostics,
        );
        if result_status != "success" {
            return Err(ProjectNormalizeError {
                message: format!("normalization failed with status {result_status}"),
                diagnostics,
                normalized_ir: result.normalized_ir.map(Box::new),
                media_source_modes,
            });
        }
        let normalized_ir = result.normalized_ir.ok_or_else(|| ProjectNormalizeError {
            message: "normalization did not produce normalized_ir".into(),
            diagnostics: diagnostics.clone(),
            normalized_ir: None,
            media_source_modes: media_source_modes.clone(),
        })?;
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            return Err(ProjectNormalizeError {
                message: "normalization produced product errors".into(),
                diagnostics,
                normalized_ir: Some(Box::new(normalized_ir)),
                media_source_modes,
            });
        }
        Ok(ProjectNormalizeOutput {
            normalized_ir,
            diagnostics,
            media_source_modes,
        })
    }
}

impl ProductDocument {
    /// Build this versioned document without converting it into editable Project state.
    pub fn build(&self, options: BuildOptions) -> Result<BuildReport, BuildError> {
        pipeline::build(BuildInput::Document(self), options, None)
    }

    pub(crate) fn build_with_writer_stack(
        &self,
        options: BuildOptions,
        policy: WriterPolicy,
        context: BuildContext,
    ) -> Result<BuildReport, BuildError> {
        pipeline::build(BuildInput::Document(self), options, Some((policy, context)))
    }

    #[cfg(feature = "internal-tools")]
    pub fn normalize(&self) -> anyhow::Result<crate::authoring_core::NormalizedIr> {
        let temp = tempfile::Builder::new()
            .prefix("anki-forge-document-normalize-")
            .tempdir()?;
        BuildInput::Document(self)
            .normalize_with_dirs(
                temp.path(),
                temp.path().join(".anki-forge-media"),
                ProjectNormalizeOptions::default(),
            )
            .map(|output| output.normalized_ir)
            .map_err(anyhow::Error::from)
    }
}
