use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use super::{
    ComparisonEvidence, ComparisonReport, RiskCode as Code, RiskFinding, RiskLevel as Level,
    UpdatePolicy,
};
use crate::{
    build::{
        identity::{IdentityEnvelope, SymbolIdentity},
        BuildCounts,
    },
    diagnostics::{Diagnostic, Severity},
};

impl ComparisonReport {
    pub(crate) fn analyze(
        before: &IdentityEnvelope,
        after: &IdentityEnvelope,
        baseline_counts: BuildCounts,
        candidate_counts: BuildCounts,
        policy: &UpdatePolicy,
    ) -> Self {
        let mut findings = Vec::new();
        for key in keys(&before.identity.notes, &after.identity.notes) {
            let old = before.identity.notes.get(key);
            let new = after.identity.notes.get(key).filter(|n| n.active);
            let path = format!("notes[{key:?}]");
            match (old, new) {
                (Some(old), None) if old.active => push(&mut findings, Code::NoteRemoved, Level::High, path, Some(json!({"guid":old.guid})), None,
                    "Note omitted from this distribution. Importing the APKG does not delete an existing learner note."),
                (None, Some(new)) => push(&mut findings, Code::NoteAdded, Level::Info, path, None, Some(json!({"guid":new.guid})), "New or restored note."),
                (Some(old), Some(new)) => {
                    if !old.active { push(&mut findings, Code::NoteAdded, Level::Info, path.clone(), None, Some(json!({"guid":new.guid})), "Historical note restored; its prior schema and cards still require comparison."); }
                    for key in keys(&old.cards, &new.cards) {
                        // Mask identities have their own risk categories. A
                        // history of IO must not hide ordinary cloze card
                        // changes, including transitions into or out of IO.
                        if key.starts_with("mask:") { continue; }
                        // Schema membership changes already have their own
                        // high-risk category, avoiding duplicate allowances.
                        if let Some(template) = key.strip_prefix("template:") {
                            let old_model = &before.identity.models[&old.model];
                            let new_model = &after.identity.models[&new.model];
                            if old_model.templates.get(template).is_none_or(|t| t.ordinal.is_none()) ||
                                new_model.templates.get(template).is_none_or(|t| t.ordinal.is_none()) { continue; }
                        }
                        let card_path = format!("{path}.cards[{key:?}]");
                        match (old.cards.get(key), new.cards.get(key)) {
                            (Some(ord), None) => push(&mut findings, Code::CardRemoved, Level::High, card_path, Some(json!(ord)), None,
                                "A previously generated card is omitted. Card counts alone do not describe identity changes or learner scheduling."),
                            (None, Some(ord)) => push(&mut findings, Code::CardAdded, Level::Info, card_path, None, Some(json!(ord)), "A new card identity is generated for this note."),
                            _ => {}
                        }
                    }
                    if old.content_hash != new.content_hash {
                        push(&mut findings, Code::NoteChanged, Level::Low, path.clone(), Some(json!(old.content_hash)), Some(json!(new.content_hash)),
                            "Content, deck or tags changed. Acceptance still depends on the target Anki import settings and local modification times.");
                    }
                    for mask in keys(&old.masks, &new.masks) {
                        let old = old.masks.get(mask).filter(|m| m.active);
                        let new = new.masks.get(mask).filter(|m| m.active);
                        let path = format!("{path}.masks[{mask:?}]");
                        match (old, new) {
                            (Some(old), None) => push(&mut findings, Code::MaskRemoved, Level::High, path, Some(json!(old.ordinal)), None,
                                "Mask omitted; its cloze ordinal remains reserved. Anki may retain an empty card until the learner removes it."),
                            (None, Some(new)) => push(&mut findings, Code::MaskAdded, Level::Medium, path, None, Some(json!(new.ordinal)), "New or restored mask card."),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        for key in keys(&before.identity.models, &after.identity.models) {
            let old = before.identity.models.get(key);
            let new = after.identity.models.get(key).filter(|m| m.active);
            let path = format!("models[{key:?}]");
            match (old, new) {
                (Some(old), None) if old.active => push(&mut findings, Code::ModelRemoved, Level::High, path, Some(json!(old.id)), None, "Model omitted; APKG import does not remove the model from an existing collection."),
                (None, Some(new)) => push(&mut findings, Code::ModelAdded, Level::Info, path, None, Some(json!(new.id)), "New or restored model."),
                (Some(old), Some(new)) => {
                    if !old.active { push(&mut findings, Code::ModelAdded, Level::Info, path.clone(), None, Some(json!(new.id)), "Historical model restored; its prior schema still requires comparison."); }
                    if old.content_hash != new.content_hash {
                        push(&mut findings, Code::ModelChanged, Level::Medium, path.clone(), Some(json!(old.content_hash)), Some(json!(new.content_hash)), "Model display, rendering or configuration changed; review the resulting cards.");
                    }
                    symbols(&mut findings, &format!("{path}.fields"), &old.fields, &new.fields, Code::FieldAdded, Code::FieldRemoved);
                    if old.sort_field != new.sort_field {
                        push(&mut findings, Code::SortFieldChanged, Level::High, format!("{path}.sort_field"), Some(json!(old.sort_field)), Some(json!(new.sort_field)),
                            "Changing the sort field can advance target note timestamps before content import. IfNewer may skip incoming content; verify the target Anki import profile, including same-second updates.");
                    }
                    symbols(&mut findings, &format!("{path}.templates"), &old.templates, &new.templates, Code::TemplateAdded, Code::TemplateRemoved);
                }
                _ => {}
            }
        }
        let old_history = media_history(before);
        let new_history = media_history(after);
        let active_names = |envelope: &IdentityEnvelope| {
            envelope
                .media
                .keys()
                .map(|name| crate::media::assets::filename_identity(name))
                .collect::<BTreeSet<_>>()
        };
        let old_active = active_names(before);
        let new_active = active_names(after);
        for key in old_active.union(&new_active) {
            let old = old_history.get(key);
            let new = new_history.get(key).filter(|_| new_active.contains(key));
            let filename = new.or(old).expect("verified active media has history").0;
            let path = format!("media[{filename:?}]");
            let evidence = |(filename, content): &(
                &String,
                &crate::build::identity::MediaIdentity,
            )| json!({"filename": filename, "content": content});
            match (old, new) {
                (Some(old), None) => push(&mut findings, Code::MediaRemoved, Level::Low, path, Some(evidence(old)), None, "Media omitted; import does not delete the previous learner copy."),
                (None, Some(new)) => push(&mut findings, Code::MediaAdded, Level::Info, path, None, Some(evidence(new)), "New exported media filename."),
                (Some(old), Some(new)) if old.1 != new.1 => push(&mut findings, Code::MediaChanged, Level::Medium, path, Some(evidence(old)), Some(evidence(new)), "A previously published portable filename now refers to different bytes, including across omitted releases; review Anki media conflict handling."),
                (Some(_), Some(new)) if !old_active.contains(key) => push(&mut findings, Code::MediaAdded, Level::Info, path, None, Some(evidence(new)), "Previously omitted media is restored with its last published bytes."),
                _ => {}
            }
        }
        let policy = policy.evaluate(&findings);
        let diagnostics = policy
            .unmatched_allowances()
            .iter()
            .map(|code| Diagnostic {
                code: "UPDATE.UNMATCHED_ALLOWANCE".into(),
                severity: Severity::Warning,
                message: format!("allowance {code} matched no finding"),
                source: None,
                help: Some("remove allowances that are not needed for this update".into()),
            })
            .collect();
        Self {
            findings,
            policy,
            diagnostics,
            baseline_counts,
            candidate_counts,
        }
    }
}

fn media_history(
    envelope: &IdentityEnvelope,
) -> BTreeMap<unicase::UniCase<String>, (&String, &crate::build::identity::MediaIdentity)> {
    envelope
        .identity
        .media_history
        .iter()
        .map(|(name, content)| {
            (
                crate::media::assets::filename_identity(name),
                (name, content),
            )
        })
        .collect()
}

fn keys<'a, T, U>(
    before: &'a BTreeMap<String, T>,
    after: &'a BTreeMap<String, U>,
) -> BTreeSet<&'a String> {
    before.keys().chain(after.keys()).collect()
}

fn symbols(
    findings: &mut Vec<RiskFinding>,
    path: &str,
    before: &BTreeMap<String, SymbolIdentity>,
    after: &BTreeMap<String, SymbolIdentity>,
    added: Code,
    removed: Code,
) {
    for key in keys(before, after) {
        let old = before.get(key).filter(|s| s.ordinal.is_some());
        let new = after.get(key).filter(|s| s.ordinal.is_some());
        let path = format!("{path}[{key:?}]");
        match (old, new) {
            (Some(old), None) => push(findings, removed, Level::High, path, Some(json!(old)), None,
                "Schema member omitted. Anki's model merge preserves target-only members; this does not synchronize deletion."),
            (None, Some(new)) => push(findings, added, Level::High, path, None, Some(json!(new)),
                "Schema member added or restored. Updating existing notes requires compatible Anki model-merge settings; test the target import profile."),
            _ => {}
        }
    }
}

fn push(
    findings: &mut Vec<RiskFinding>,
    code: Code,
    level: Level,
    selector: String,
    before: Option<Value>,
    after: Option<Value>,
    message: &str,
) {
    findings.push(RiskFinding {
        code,
        level,
        message: message.into(),
        evidence: vec![ComparisonEvidence {
            selector,
            before,
            after,
        }],
    });
}
