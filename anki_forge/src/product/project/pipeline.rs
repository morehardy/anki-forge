//! A build is a private candidate followed by publication. Only finish creates
//! the public report; every failed stage retains the facts collected so far.
use super::*;
mod reconcile;

#[derive(Default)]
struct BuildFacts {
    artifact: Option<ApkgArtifact>,
    counts: BuildCounts,
    media: MediaSummary,
    diagnostics: Vec<Diagnostic>,
    inspect: Option<crate::build::InspectSummary>,
    previous_inspect: Option<crate::build::InspectSummary>,
    update_safety: Option<crate::build::UpdateSafetySummary>,
    comparison: ComparisonStatus,
    diff: Option<crate::diff::BuildDiffSummary>,
    risk: Option<crate::risk::ImportRiskReport>,
    policy: BuildPolicyResult,
    status: BuildStatus,
}

impl BuildFacts {
    fn record_normalized(
        &mut self,
        normalized: &crate::authoring_core::NormalizedIr,
        modes: &BTreeMap<String, MediaSourceMode>,
    ) {
        self.counts = BuildCounts {
            notes: normalized.notes.len(),
            cards: count_phase1_cards_without_inspect(normalized),
            media: normalized.media_bindings.len(),
        };
        self.media = MediaSummary::from_normalized_ir_with_source_modes(
            normalized,
            &self.diagnostics,
            modes,
        );
    }

    fn failure(
        &mut self,
        code: &str,
        message: String,
        cause: BuildFailureCause,
    ) -> BuildFailureCause {
        self.status = BuildStatus::Error;
        self.diagnostics.push(Diagnostic {
            code: DiagnosticCode::new(code),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message,
            source: Some(SourcePath::new("project.build")),
            help: None,
        });
        cause
    }

    fn path_failure(&mut self, diagnostic: Diagnostic) -> BuildFailureCause {
        let cause = if diagnostic.code.as_str() == "PROJECT.PATH_COLLISION" {
            self.status = BuildStatus::Invalid;
            BuildFailureCause::Diagnostics
        } else {
            self.status = BuildStatus::Error;
            BuildFailureCause::Io
        };
        self.diagnostics.push(diagnostic);
        cause
    }
}

struct PreparedBuild {
    artifact_workspace: ArtifactWorkspace,
    baseline: Option<crate::product::comparison::BaselineSnapshot>,
    normalized: crate::authoring_core::NormalizedIr,
    media_source_modes: BTreeMap<String, MediaSourceMode>,
    media_store_dir: PathBuf,
    writer_policy: WriterPolicy,
    build_context: BuildContext,
}

struct ReconciledBuild {
    prepared: PreparedBuild,
    identity_index: crate::update_safety::model::IdentityIndex,
    reconcile: crate::update_safety::reconcile::ReconcileOutput,
    writer_guid_plan: crate::writer_core::WriterGuidPlan,
    update_mode: crate::update_safety::EffectiveMode,
    write_identity_lockfile: bool,
    lockfile_index: Option<crate::update_safety::model::IdentityIndex>,
}

struct CandidateBuild {
    state: ReconciledBuild,
    directory: TempDir,
    path: Option<PathBuf>,
    writer_status: BuildStatus,
}

struct AcceptedCandidate {
    state: ReconciledBuild,
    _directory: TempDir,
    path: PathBuf,
}

struct BuildPipeline<'a> {
    input: BuildInput<'a>,
    options: BuildOptions,
    started: Instant,
    facts: BuildFacts,
    artifact_ref_prefix: String,
    writer_result: Option<crate::writer_core::PackageBuildResult>,
}

pub(super) fn build(
    input: BuildInput<'_>,
    options: BuildOptions,
    writer_stack: Option<(WriterPolicy, BuildContext)>,
) -> Result<BuildReport, BuildError> {
    execute(input, options, writer_stack, None).map(|(report, _)| report)
}

pub(super) fn execute(
    input: BuildInput<'_>,
    options: BuildOptions,
    writer_stack: Option<(WriterPolicy, BuildContext)>,
    artifact_ref_prefix: Option<String>,
) -> Result<(BuildReport, crate::writer_core::PackageBuildResult), BuildError> {
    let artifact_ref_prefix = artifact_ref_prefix.unwrap_or_else(|| {
        input
            .stable_id()
            .map(|id| format!("artifacts/{id}"))
            .unwrap_or_else(|| "artifacts".into())
    });
    let mut pipeline = BuildPipeline {
        input,
        options,
        started: Instant::now(),
        facts: BuildFacts::default(),
        artifact_ref_prefix,
        writer_result: None,
    };
    let outcome = pipeline.run(writer_stack);
    let writer_result = pipeline.writer_result.take();
    let report = pipeline.finish(outcome)?;
    Ok((
        report,
        writer_result.expect("successful build has a writer result"),
    ))
}

impl BuildPipeline<'_> {
    fn run(
        &mut self,
        writer_stack: Option<(WriterPolicy, BuildContext)>,
    ) -> Result<(), BuildFailureCause> {
        let prepared = self.prepare(writer_stack)?;
        let reconciled = self.reconcile(prepared)?;
        let candidate = self.generate(reconciled)?;
        let accepted = self.inspect(candidate)?;
        self.publish(accepted)
    }

    fn prepare(
        &mut self,
        writer_stack: Option<(WriterPolicy, BuildContext)>,
    ) -> Result<PreparedBuild, BuildFailureCause> {
        let input = self.input;
        let options = &self.options;
        let facts = &mut self.facts;
        if let Err(diagnostic) = BuildPathPlan::new(options).validate() {
            return Err(facts.path_failure(*diagnostic));
        }
        if options.report_json.is_some()
            && options.output.is_none()
            && options.artifacts_dir.is_none()
        {
            return Err(facts.failure(
                "PROJECT.REPORT_JSON_WRITE_FAILED",
                "report_json requires output or artifacts_dir; JSON cannot retain a temporary APKG"
                    .into(),
                BuildFailureCause::Io,
            ));
        }
        let baseline = options.compare_to.as_deref().map(|path| {
            crate::product::comparison::BaselineSnapshot::capture_with_limits(
                path,
                &options.inspect_limits,
            )
        });
        let artifact_workspace = ArtifactWorkspace::new(options).map_err(|error| {
            facts.failure(
                "PROJECT.ARTIFACTS_DIR_FAILED",
                error.to_string(),
                BuildFailureCause::Io,
            )
        })?;
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

        let validation = input.validate();
        facts.diagnostics = validation.diagnostics;

        let normalized_output =
            input.normalize_with_dirs(&media_input_dir, &media_store_dir, normalize_options);
        let normalized_output = match normalized_output {
            Ok(output) => output,
            Err(error) => {
                let ProjectNormalizeError {
                    message,
                    diagnostics: mut normalize_diagnostics,
                    normalized_ir,
                    media_source_modes,
                } = error;
                facts.diagnostics.append(&mut normalize_diagnostics);
                deduplicate_diagnostics(&mut facts.diagnostics);
                facts.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.NORMALIZE_FAILED"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
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
                let media = normalized_ir
                    .as_ref()
                    .map(|normalized| {
                        MediaSummary::from_normalized_ir_with_source_modes(
                            normalized.as_ref(),
                            &facts.diagnostics,
                            &media_source_modes,
                        )
                    })
                    .unwrap_or_default();
                facts.counts = counts;
                facts.media = media;
                facts.status = BuildStatus::Invalid;
                return Err(BuildFailureCause::Diagnostics);
            }
        };
        let normalized = normalized_output.normalized_ir;
        let media_source_modes = normalized_output.media_source_modes;
        facts.diagnostics.extend(normalized_output.diagnostics);
        deduplicate_diagnostics(&mut facts.diagnostics);
        facts.record_normalized(&normalized, &media_source_modes);

        if normalized.notes.is_empty() {
            facts.diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("PROJECT.EMPTY"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: "project contains no notes".into(),
                source: Some(SourcePath::new("project.notes")),
                help: Some("add at least one note before building".into()),
            });
            facts.status = BuildStatus::Invalid;
            return Err(BuildFailureCause::Invalid);
        }

        if facts
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            let media = MediaSummary::from_normalized_ir_with_source_modes(
                &normalized,
                &facts.diagnostics,
                &media_source_modes,
            );
            facts.media = media;
            facts.status = BuildStatus::Invalid;
            return Err(BuildFailureCause::Diagnostics);
        }

        let (writer_policy, build_context) = match writer_stack {
            Some((writer_policy, build_context)) => (writer_policy, build_context),
            None => {
                let (_runtime, writer_policy, build_context) =
                    crate::runtime::load_default_writer_stack().map_err(|err| {
                        facts.failure(
                            "PROJECT.RUNTIME_DEFAULTS_FAILED",
                            err.to_string(),
                            BuildFailureCause::Io,
                        )
                    })?;
                (writer_policy, build_context)
            }
        };
        Ok(PreparedBuild {
            artifact_workspace,
            baseline,
            normalized,
            media_source_modes,
            media_store_dir,
            writer_policy,
            build_context,
        })
    }

    fn generate(&mut self, state: ReconciledBuild) -> Result<CandidateBuild, BuildFailureCause> {
        let facts = &mut self.facts;
        let prepared = &state.prepared;
        let stable_ref_prefix = self.artifact_ref_prefix.clone();
        let candidate_dir =
            prepared
                .artifact_workspace
                .create_candidate_dir()
                .map_err(|error| {
                    facts.failure(
                        "PROJECT.ARTIFACTS_DIR_FAILED",
                        error.to_string(),
                        BuildFailureCause::Io,
                    )
                })?;
        let artifact_target = BuildArtifactTarget::new(
            prepared.artifact_workspace.path.clone(),
            stable_ref_prefix.clone(),
        )
        .with_media_store_dir(prepared.media_store_dir.clone());
        let apkg_target = BuildArtifactTarget::new(candidate_dir.path(), stable_ref_prefix)
            .with_media_store_dir(prepared.media_store_dir.clone());
        let notetype_ids = state
            .identity_index
            .notetypes
            .iter()
            .filter_map(|notetype| {
                notetype
                    .anki_model_id
                    .map(|id| (notetype.note_type_id.clone(), id))
            })
            .collect::<BTreeMap<_, _>>();
        let package_build_result = crate::writer_core::build_with_identity_plan(
            &prepared.normalized,
            &prepared.writer_policy,
            &prepared.build_context,
            &artifact_target,
            &apkg_target,
            Some(&state.writer_guid_plan),
            Some(&notetype_ids),
        )
        .map_err(|err| {
            facts.failure(
                "PROJECT.WRITER_FAILED",
                err.to_string(),
                BuildFailureCause::Internal,
            )
        })?;

        facts
            .diagnostics
            .extend(package_build_result.diagnostics.items.iter().map(|item| {
                Diagnostic {
                    code: DiagnosticCode::new(item.code.clone()),
                    severity: severity_from_level(&item.level),
                    domain: item
                        .domain
                        .clone()
                        .map(crate::diagnostics::DiagnosticDomain::new),
                    stage: item
                        .stage
                        .clone()
                        .map(crate::diagnostics::DiagnosticStage::new),
                    message: item.summary.clone(),
                    source: item.path.clone().map(SourcePath::new),
                    help: None,
                }
            }));
        if package_build_result.result_status != "success"
            && !facts
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            facts.diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("PROJECT.BUILD_STATUS_FAILED"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: format!("build status was {}", package_build_result.result_status),
                source: Some(SourcePath::new("project.build")),
                help: Some("inspect writer diagnostics for the failed stage".into()),
            });
        }
        let artifact = package_build_result
            .apkg_ref
            .as_deref()
            .map(|apkg_ref| artifact_path_from_ref(&apkg_target, apkg_ref))
            .transpose()
            .map_err(|error| {
                facts.failure(
                    "PROJECT.ARTIFACT_REF_FAILED",
                    error.to_string(),
                    BuildFailureCause::Io,
                )
            })?;

        let writer_status = build_status_from_writer_result(&package_build_result.result_status);
        self.writer_result = Some(package_build_result);
        Ok(CandidateBuild {
            state,
            directory: candidate_dir,
            path: artifact,
            writer_status,
        })
    }

    fn inspect(
        &mut self,
        candidate: CandidateBuild,
    ) -> Result<AcceptedCandidate, BuildFailureCause> {
        let options = &self.options;
        let facts = &mut self.facts;
        let prepared = &candidate.state.prepared;
        let normalized = &prepared.normalized;
        let update_mode = candidate.state.update_mode;
        let started = self.started;
        let media = MediaSummary::from_normalized_ir_with_source_modes(
            normalized,
            &facts.diagnostics,
            &prepared.media_source_modes,
        );
        let writer_status = candidate.writer_status;
        let mut inspect = None;
        let mut previous_inspect = None;
        let mut comparison = ComparisonStatus::NotRequested;
        let mut diff = None;
        let mut risk = None;
        let mut policy = BuildPolicyResult::default();
        let mut status =
            BuildStatus::highest([writer_status, diagnostics_status(&facts.diagnostics)]);
        if let Some(artifact) = candidate.path.as_ref() {
            let comparison_output = crate::product::comparison::assemble_comparison_with_limits(
                crate::product::comparison::ComparisonInput {
                    current_artifact: artifact,
                    previous_artifact: options.compare_to.as_deref(),
                    diagnostics: &facts.diagnostics,
                    update_safety: facts.update_safety.as_ref(),
                    started,
                },
                prepared.baseline.as_ref(),
                &options.inspect_limits,
            );
            facts.diagnostics = comparison_output.diagnostics;
            if options.inspect {
                inspect = comparison_output.current_inspect.clone();
            }
            previous_inspect = comparison_output.previous_inspect;
            let comparison_is_report_only = matches!(
                update_mode,
                crate::update_safety::EffectiveMode::Disabled
                    | crate::update_safety::EffectiveMode::ReportOnly
            );
            if comparison_is_report_only {
                downgrade_compare_errors_to_warnings(&mut facts.diagnostics);
            }
            comparison = comparison_output.comparison;
            diff = comparison_output.diff;
            risk = comparison_output.risk;
            attach_artifact_diff_risk_if_needed(&mut risk, diff.as_ref());
            policy = crate::risk::policy_from_risk_report(options.fail_on, risk.as_ref());
            let comparison_status = if comparison_is_report_only && options.fail_on.is_none() {
                BuildStatus::Success
            } else {
                comparison_output.status
            };
            status = BuildStatus::highest([
                writer_status,
                comparison_status,
                policy_status(&policy),
                diagnostics_status(&facts.diagnostics),
            ]);
        }
        let counts = BuildCounts {
            notes: normalized.notes.len(),
            cards: card_count_from_inspect_or_fallback(inspect.as_ref(), normalized),
            media: normalized.media_bindings.len(),
        };
        facts.counts = counts;
        facts.media = media;
        facts.inspect = inspect;
        facts.previous_inspect = previous_inspect;

        facts.comparison = comparison;
        facts.diff = diff;
        facts.risk = risk;
        facts.policy = policy;
        facts.status = status;

        if let Some(cause) = crate::build::report::failure_cause(
            &facts.diagnostics,
            facts.status,
            candidate.path.is_some(),
        ) {
            return Err(cause);
        }
        Ok(AcceptedCandidate {
            state: candidate.state,
            _directory: candidate.directory,
            path: candidate.path.expect("successful candidate exists"),
        })
    }

    fn publish(&mut self, candidate: AcceptedCandidate) -> Result<(), BuildFailureCause> {
        let input = self.input;
        let options = &self.options;
        let facts = &mut self.facts;
        let state = &candidate.state;
        let prepared = &state.prepared;
        if let Err(diagnostic) = BuildPathPlan::new(options).validate() {
            return Err(facts.path_failure(*diagnostic));
        }
        match prepared
            .artifact_workspace
            .publish(&candidate.path, options)
        {
            Ok(artifact) => facts.artifact = Some(artifact),
            Err(error) => {
                facts.status = BuildStatus::Error;
                facts.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.OUTPUT_WRITE_FAILED"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
                    source: Some(SourcePath::new("build.output")),
                    message: error.to_string(),
                    help: None,
                });
                return Err(BuildFailureCause::Io);
            }
        }
        // New paths can only reveal case-folding aliases after creation. Keep
        // the published APKG intact and reject any conflicting follow-up write.
        if let Err(diagnostic) = BuildPathPlan::new(options).validate() {
            return Err(facts.path_failure(*diagnostic));
        }
        if state.write_identity_lockfile {
            if let Some(path) = options.identity_lockfile.as_ref() {
                let selected_index = crate::update_safety::reconcile::selected_identity_index(
                    &state.identity_index,
                    &state.reconcile,
                    state.lockfile_index.as_ref(),
                );
                let writer_policy_ref = crate::writer_core::policy_ref(
                    &prepared.writer_policy.id,
                    &prepared.writer_policy.version,
                );
                let lockfile = crate::update_safety::model::IdentityLockfile {
                    schema_version: "identity-lockfile-v1".into(),
                    project_stable_id: input
                        .stable_id()
                        .map(str::to_string)
                        .expect("lockfile project identity was validated"),
                    writer_policy_ref: writer_policy_ref.clone(),
                    identity_index: selected_index,
                    generated_by: crate::update_safety::model::GeneratedBy {
                        tool: "anki-forge".into(),
                        tool_version: env!("CARGO_PKG_VERSION").into(),
                        writer_policy_ref,
                    },
                };
                if let Err(error) =
                    crate::update_safety::lockfile::write_lockfile_atomic(path, &lockfile)
                {
                    facts.status = BuildStatus::Error;
                    facts.diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("UPDATE.LOCKFILE_WRITE_FAILED"),
                        severity: Severity::Error,
                        domain: None,
                        stage: None,
                        source: Some(SourcePath::new(path.display().to_string())),
                        message: error.to_string(),
                        help: Some("verify the lockfile path is writable".into()),
                    });
                    return Err(BuildFailureCause::Io);
                }
                if let Some(summary) = facts.update_safety.as_mut() {
                    summary.lockfile_written = true;
                }
            }
        }
        Ok(())
    }

    fn finish(
        mut self,
        mut outcome: Result<(), BuildFailureCause>,
    ) -> Result<BuildReport, BuildError> {
        if let Err(cause) = outcome {
            let status = match cause {
                BuildFailureCause::Io | BuildFailureCause::Internal => BuildStatus::Error,
                BuildFailureCause::PolicyBlocked => BuildStatus::Blocked,
                _ => BuildStatus::Invalid,
            };
            self.facts.status = BuildStatus::highest([self.facts.status, status]);
        }
        // Recheck once after all writes. Never overwrite a protected file to
        // report the collision itself; a fresh collision still fails success.
        let write_json = match BuildPathPlan::new(&self.options).validate_report() {
            Ok(()) => true,
            Err(diagnostic) => {
                if outcome.is_ok() {
                    outcome = Err(self.facts.path_failure(*diagnostic));
                }
                false
            }
        };
        let facts = self.facts;
        let mut report = BuildReport {
            artifact: facts.artifact,
            counts: facts.counts,
            media: facts.media,
            diagnostics: facts.diagnostics,
            metrics: BuildMetrics {
                duration: self.started.elapsed(),
            },
            inspect: facts.inspect,
            previous_inspect: facts.previous_inspect,
            update_safety: facts.update_safety,
            comparison: facts.comparison,
            diff: facts.diff,
            risk: facts.risk,
            policy: facts.policy,
            status: facts.status,
        };
        if let Some(path) = self.options.report_json.as_ref().filter(|_| write_json) {
            if let Err(error) = crate::build::write_report_json_atomic(path, &report) {
                report.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.REPORT_JSON_WRITE_FAILED"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
                    message: format!("failed to write report_json: {error}"),
                    source: Some(SourcePath::new(path.display().to_string())),
                    help: Some("choose a writable report_json path".into()),
                });
                report.status = BuildStatus::Error;
                outcome = Err(BuildFailureCause::Io);
            }
        }
        match outcome {
            Ok(()) => Ok(report),
            Err(cause) => Err(BuildError::new(report, cause)),
        }
    }
}
