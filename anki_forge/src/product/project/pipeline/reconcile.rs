//! Reconcile identity evidence before any candidate is generated.
use super::*;

impl BuildPipeline<'_> {
    pub(super) fn reconcile(
        &mut self,
        prepared: PreparedBuild,
    ) -> Result<ReconciledBuild, BuildFailureCause> {
        let input = self.input;
        let options = &self.options;
        let facts = &mut self.facts;
        let PreparedBuild {
            artifact_workspace,
            baseline,
            mut normalized,
            media_source_modes,
            media_store_dir,
            writer_policy,
            build_context,
        } = prepared;
        let resolved_note_identities = input.resolved_note_identities();

        let update_mode = match crate::update_safety::effective_mode(options) {
            Ok(mode) => mode,
            Err(err) => {
                facts.diagnostics.push(Diagnostic {
                    code: err.code,
                    severity: err.severity,
                    domain: None,
                    stage: None,
                    message: err.message,
                    source: Some(SourcePath::new("build.options")),
                    help: None,
                });
                let media = MediaSummary::from_normalized_ir_with_source_modes(
                    &normalized,
                    &facts.diagnostics,
                    &media_source_modes,
                );
                facts.media = media;
                facts.status = BuildStatus::Invalid;
                return Err(BuildFailureCause::Diagnostics);
            }
        };

        let project_stable_id_required =
            if matches!(update_mode, crate::update_safety::EffectiveMode::Disabled) {
                options.write_identity_lockfile
            } else {
                true
            };

        if input.stable_id().is_none() && project_stable_id_required {
            let condition =
                if options.identity_lockfile.is_some() || options.write_identity_lockfile {
                    crate::update_safety::EvidenceCondition::LockfileRequired
                } else {
                    crate::update_safety::EvidenceCondition::StrictCompareOnly
                };
            let classified = crate::update_safety::classify_project_stable_id_missing(condition);
            if let Some(code) = classified.diagnostic_code {
                facts.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new(code),
                    severity: if options.write_identity_lockfile {
                        Severity::Error
                    } else {
                        update_safety_blocking_severity(update_mode)
                    },
                    domain: None,
                    stage: None,
                    message: "project stable id is missing for update-safety proof".into(),
                    source: Some(SourcePath::new("project.stable_id")),
                    help: Some("set Project::stable_id(value) for update-safe builds".into()),
                });
            }
        }

        let mut current_identity = crate::update_safety::current::build_current_identity_index(
            crate::update_safety::current::CurrentIdentityInput {
                project_stable_id: input.stable_id(),
                normalized: &normalized,
                writer_policy: &writer_policy,
                mode: update_mode,
                resolved_note_identities: &resolved_note_identities,
            },
        );
        facts.diagnostics.extend(current_identity.diagnostics);
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

        let disabled_update_safety =
            matches!(update_mode, crate::update_safety::EffectiveMode::Disabled);

        let (reconcile, update_safety_summary_val, lockfile_index) = if disabled_update_safety {
            let mut baseline_sources = Vec::new();
            if let Some(path) = options.compare_to.as_ref() {
                facts.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("UPDATE.BASELINE_IGNORED_DISABLED"),
                    severity: Severity::Info,
                    domain: None,
                    stage: None,
                    message: "compare_to baseline ignored because update safety is disabled".into(),
                    source: Some(SourcePath::new(path.display().to_string())),
                    help: Some(
                        "remove update_safety(UpdateSafetyMode::Disabled) to analyze the baseline"
                            .into(),
                    ),
                });
                baseline_sources.push(crate::update_safety::report::ignored_previous_apkg_source(
                    path,
                ));
            }
            if let Some(path) = options.identity_lockfile.as_ref() {
                baseline_sources.push(crate::update_safety::report::ignored_lockfile_source(path));
            }
            let reconcile =
                crate::update_safety::reconcile::current_only_reconcile(&current_identity.index)
                    .map_err(|_err| {
                        facts.status = BuildStatus::Invalid;
                        BuildFailureCause::Diagnostics
                    })?;
            let summary = crate::update_safety::report::summary_from_disabled_mode(
                &current_identity.index,
                baseline_sources,
                facts
                    .diagnostics
                    .iter()
                    .filter(|item| item.severity == Severity::Error)
                    .map(|item| item.code.to_string())
                    .collect(),
            );
            (reconcile, summary, None)
        } else {
            let mut baseline_sources = Vec::new();
            let update_error_severity = update_safety_blocking_severity(update_mode);
            let lockfile = if let Some(path) = options.identity_lockfile.as_ref() {
                if path.exists() {
                    match crate::update_safety::lockfile::read_lockfile(path) {
                        Ok(lockfile) => {
                            baseline_sources.push(
                                crate::update_safety::report::loaded_lockfile_source(
                                    path,
                                    lockfile.identity_index.limitations.clone(),
                                ),
                            );
                            push_project_stable_id_mismatch_if_needed(
                                &mut facts.diagnostics,
                                input.stable_id(),
                                Some(lockfile.project_stable_id.as_str()),
                                path.display().to_string(),
                                update_error_severity,
                            );
                            Some(lockfile)
                        }
                        Err(err) => {
                            facts.diagnostics.push(Diagnostic {
                                code: DiagnosticCode::new("UPDATE.BASELINE_LOCKFILE_UNREADABLE"),
                                severity: update_error_severity,
                                domain: None,
                                stage: None,
                                message: err.to_string(),
                                source: Some(SourcePath::new(path.display().to_string())),
                                help: Some("fix or regenerate the identity lockfile".into()),
                            });
                            baseline_sources.push(
                                crate::update_safety::report::unreadable_lockfile_source(
                                    path,
                                    "UPDATE.BASELINE_LOCKFILE_UNREADABLE",
                                ),
                            );
                            None
                        }
                    }
                } else if options.write_identity_lockfile {
                    None
                } else {
                    facts.diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("UPDATE.BASELINE_LOCKFILE_UNREADABLE"),
                        severity: update_error_severity,
                        domain: None,
                        stage: None,
                        message: format!("identity lockfile {} does not exist", path.display()),
                        source: Some(SourcePath::new(path.display().to_string())),
                        help: Some(
                            "run with write_identity_lockfile(true) to create the first lockfile"
                                .into(),
                        ),
                    });
                    baseline_sources.push(
                        crate::update_safety::report::unreadable_lockfile_source(
                            path,
                            "UPDATE.BASELINE_LOCKFILE_UNREADABLE",
                        ),
                    );
                    None
                }
            } else {
                None
            };
            let lf_index = lockfile
                .as_ref()
                .map(|lockfile| lockfile.identity_index.clone());

            let previous_index = if let Some(path) = options.compare_to.as_ref() {
                match baseline
                    .as_ref()
                    .expect("requested baseline was captured")
                    .identity_index(Some(&current_identity.index), lf_index.as_ref())
                {
                    Ok(index) => {
                        baseline_sources.push(
                            crate::update_safety::report::loaded_previous_apkg_source(
                                path,
                                index.limitations.clone(),
                            ),
                        );
                        push_project_stable_id_mismatch_if_needed(
                            &mut facts.diagnostics,
                            input.stable_id(),
                            index.project_stable_id.as_deref(),
                            path.display().to_string(),
                            update_error_severity,
                        );
                        Some(index)
                    }
                    Err(err) => {
                        facts.diagnostics.push(Diagnostic {
                            code: DiagnosticCode::new("UPDATE.BASELINE_APKG_UNREADABLE"),
                            severity: update_error_severity,
                            domain: None,
                            stage: None,
                            message: err.to_string(),
                            source: Some(SourcePath::new(path.display().to_string())),
                            help: Some("verify the previous APKG path and package contents".into()),
                        });
                        crate::product::comparison::push_resource_diagnostic(
                            &mut facts.diagnostics,
                            &err,
                            path,
                            update_error_severity,
                        );
                        baseline_sources.push(
                            crate::update_safety::report::unreadable_previous_apkg_source(
                                path,
                                "UPDATE.BASELINE_APKG_UNREADABLE",
                            ),
                        );
                        None
                    }
                }
            } else {
                None
            };

            if matches!(update_mode, crate::update_safety::EffectiveMode::Strict)
                && options.compare_to.is_some()
                && previous_index.is_none()
            {
                if let Some(path) = options.compare_to.as_ref() {
                    facts.diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("COMPARE.BASELINE_UNAVAILABLE"),
                        severity: Severity::Error,
                        domain: None,
                        stage: None,
                        message: format!(
                            "APKG could not be inspected for comparison: {}",
                            path.display()
                        ),
                        source: Some(SourcePath::new(path.display().to_string())),
                        help: Some("verify the previous APKG path and package contents".into()),
                    });
                }
                let update_safety_summary = crate::update_safety::report::summary_from_reconcile(
                    update_mode,
                    &crate::update_safety::reconcile::current_only_reconcile(
                        &current_identity.index,
                    )
                    .map_err(|_err| {
                        facts.status = BuildStatus::Invalid;
                        BuildFailureCause::Diagnostics
                    })?,
                    &facts.diagnostics,
                    baseline_sources.clone(),
                    false,
                );
                let risk =
                    baseline_unavailable_risk(&facts.diagnostics, Some(&update_safety_summary));
                let policy = crate::risk::policy_from_risk_report(options.fail_on, Some(&risk));
                let status = BuildStatus::highest([
                    BuildStatus::Invalid,
                    policy_status(&policy),
                    diagnostics_status(&facts.diagnostics),
                ]);
                facts.update_safety = Some(update_safety_summary);
                facts.comparison = ComparisonStatus::Unavailable;
                facts.risk = Some(risk);
                facts.policy = policy;
                facts.status = status;
                return Err(BuildFailureCause::Diagnostics);
            }

            let reconcile = crate::update_safety::reconcile::reconcile_guid_plan(
                &current_identity.index,
                previous_index.as_ref(),
                lf_index.as_ref(),
            )
            .map_err(|err| {
                facts.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("UPDATE.GUID_DUPLICATE_AT_RECONCILE"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
                    message: err.to_string(),
                    source: Some(SourcePath::new("update_safety.reconcile")),
                    help: Some(
                        "choose unique stable ids or remove conflicting lockfile entries".into(),
                    ),
                });
                facts.status = BuildStatus::Invalid;
                BuildFailureCause::Diagnostics
            })?;
            facts.diagnostics.extend(reconcile.diagnostics.clone());
            let mut model_diagnostics = crate::update_safety::notetype_ids::reconcile_notetype_ids(
                &mut current_identity.index,
                previous_index.as_ref(),
                lf_index.as_ref(),
            );
            let model_id_collision = model_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.NOTETYPE_MODEL_ID_COLLISION");
            if matches!(update_mode, crate::update_safety::EffectiveMode::ReportOnly) {
                for diagnostic in &mut model_diagnostics {
                    if diagnostic.code.as_str() == "UPDATE.NOTETYPE_MODEL_ID_MISSING" {
                        diagnostic.severity = Severity::Warning;
                    }
                }
            }
            facts.diagnostics.extend(model_diagnostics);
            let mut revision_diagnostics = crate::update_safety::note_revisions::reconcile(
                &mut current_identity.index,
                &mut normalized,
                previous_index.as_ref(),
                lf_index.as_ref(),
            );
            let revision_overflow = revision_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.NOTE_MTIME_OVERFLOW");
            if matches!(update_mode, crate::update_safety::EffectiveMode::ReportOnly) {
                for diagnostic in &mut revision_diagnostics {
                    if diagnostic.code.as_str() == "UPDATE.NOTE_REVISION_MISSING" {
                        diagnostic.severity = Severity::Warning;
                    }
                }
            }
            facts.diagnostics.extend(revision_diagnostics);
            let lockfile_index = crate::update_safety::notetype_ids::combined_baseline(
                previous_index.as_ref(),
                lf_index.as_ref(),
            );
            if let Some(baseline_for_merge) = lockfile_index.as_ref() {
                let mut merge_diagnostics =
                    crate::update_safety::merge_safety::compare_notetype_merge_safety(
                        &current_identity.index,
                        baseline_for_merge,
                    );
                if matches!(update_mode, crate::update_safety::EffectiveMode::ReportOnly) {
                    downgrade_update_errors_to_warnings(&mut merge_diagnostics);
                }
                facts.diagnostics.extend(merge_diagnostics);
            }
            if model_id_collision
                || revision_overflow
                || (matches!(update_mode, crate::update_safety::EffectiveMode::Strict)
                    && facts
                        .diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.severity == Severity::Error))
            {
                let update_safety_summary = crate::update_safety::report::summary_from_reconcile(
                    update_mode,
                    &reconcile,
                    &facts.diagnostics,
                    baseline_sources.clone(),
                    false,
                );
                let risk = crate::risk::classify_import_risk(crate::risk::rules::RiskInput {
                    diagnostics: &facts.diagnostics,
                    comparison: ComparisonStatus::NotRequested,
                    diff: None,
                    current_inspect: None,
                    previous_inspect: None,
                    update_safety: Some(&update_safety_summary),
                });
                let policy = crate::risk::policy_from_risk_report(options.fail_on, Some(&risk));
                let status = BuildStatus::highest([
                    BuildStatus::Invalid,
                    policy_status(&policy),
                    diagnostics_status(&facts.diagnostics),
                ]);
                facts.update_safety = Some(update_safety_summary);
                facts.risk = Some(risk);
                facts.policy = policy;
                facts.status = status;
                return Err(BuildFailureCause::Diagnostics);
            }
            let summary = crate::update_safety::report::summary_from_reconcile(
                update_mode,
                &reconcile,
                &facts.diagnostics,
                baseline_sources,
                false,
            );
            (reconcile, summary, lockfile_index)
        };

        let lockfile_evidence_unverified =
            matches!(update_mode, crate::update_safety::EffectiveMode::ReportOnly)
                && facts.diagnostics.iter().any(|diagnostic| {
                    matches!(
                        diagnostic.code.as_str(),
                        "UPDATE.BASELINE_LOCKFILE_UNREADABLE"
                            | "UPDATE.BASELINE_APKG_UNREADABLE"
                            | "UPDATE.NOTETYPE_MODEL_ID_MISSING"
                            | "UPDATE.NOTE_REVISION_MISSING"
                    )
                });
        let write_identity_lockfile =
            options.write_identity_lockfile && !lockfile_evidence_unverified;
        if options.write_identity_lockfile && lockfile_evidence_unverified {
            facts.diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.LOCKFILE_WRITE_SKIPPED_UNVERIFIED"),
                severity: Severity::Warning,
                domain: None,
                stage: None,
                message: "identity lockfile was not written because baseline evidence is unverified"
                    .into(),
                source: options
                    .identity_lockfile
                    .as_ref()
                    .map(|path| SourcePath::new(path.display().to_string())),
                help: Some("restore readable baseline evidence and recover missing identities or revisions from the previous APKG before writing the lockfile".into()),
            });
        }

        facts.update_safety = Some(update_safety_summary_val);
        Ok(ReconciledBuild {
            prepared: PreparedBuild {
                artifact_workspace,
                baseline,
                normalized,
                media_source_modes,
                media_store_dir,
                writer_policy,
                build_context,
            },
            identity_index: current_identity.index,
            reconcile,
            update_mode,
            write_identity_lockfile,
            lockfile_index,
        })
    }
}
