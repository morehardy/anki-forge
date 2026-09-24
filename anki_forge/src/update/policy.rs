use super::{PolicyError, PolicyEvaluation, RiskFinding};
use serde::Serialize;
use std::{collections::BTreeSet, fmt, str::FromStr};

/// Ordered severity of a completed comparison finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// An observed addition or unchanged identity fact.
    Info,
    /// A routine content change.
    Low,
    /// A change that should be reviewed before importing.
    Medium,
    /// A structural change or omission requiring explicit acceptance by default.
    High,
    /// The highest publishable risk category; hard errors are never findings.
    Critical,
}

macro_rules! risk_codes {
    ($( $variant:ident => ($code:literal, $doc:literal) ),* $(,)?) => {
        /// Registered, explicitly acceptable risk categories. Hard errors and
        /// unknown codes cannot be constructed through text parsing.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
        pub enum RiskCode { $( #[doc=$doc] #[serde(rename=$code)] $variant, )* }
        impl RiskCode {
            /// Returns the registered machine code independent of human wording.
            pub const fn as_str(self) -> &'static str { match self { $( Self::$variant => $code, )* } }
        }
        impl FromStr for RiskCode {
            type Err = PolicyError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { $( $code => Ok(Self::$variant), )* _ => Err(PolicyError::unknown(value)) }
            }
        }
    }
}
risk_codes! {
    NoteAdded => ("RISK.NOTE_ADDED", "A new or restored note is present in the candidate."),
    NoteChanged => ("RISK.NOTE_CHANGED", "An existing note's content, deck or tags changed."),
    NoteRemoved => ("RISK.NOTE_REMOVED", "A previously published note is omitted; APKG import does not delete it from a learner's collection."),
    CardAdded => ("RISK.CARD_ADDED", "An existing note generates a new card identity."),
    CardRemoved => ("RISK.CARD_REMOVED", "An existing note no longer generates a previously published card identity."),
    ModelAdded => ("RISK.MODEL_ADDED", "A new or restored note type is present."),
    ModelChanged => ("RISK.MODEL_CHANGED", "An existing model's display, rendering or configuration changed."),
    SortFieldChanged => ("RISK.SORT_FIELD_CHANGED", "The model's sort field changed, which can advance target note timestamps during import."),
    ModelRemoved => ("RISK.MODEL_REMOVED", "A previously published model is omitted from the candidate."),
    FieldAdded => ("RISK.FIELD_ADDED", "An existing model has a new or restored field, requiring a compatible model merge."),
    FieldRemoved => ("RISK.FIELD_REMOVED", "An existing model omits a field; model merging retains it in the target."),
    TemplateAdded => ("RISK.TEMPLATE_ADDED", "An existing model has a new or restored card template."),
    TemplateRemoved => ("RISK.TEMPLATE_REMOVED", "An existing model omits a card template; existing cards are not deleted by an APKG merge."),
    MaskAdded => ("RISK.MASK_ADDED", "An image-occlusion note has a new or restored mask card."),
    MaskRemoved => ("RISK.MASK_REMOVED", "An image-occlusion mask is omitted; its ordinal remains reserved."),
    MediaAdded => ("RISK.MEDIA_ADDED", "A new media filename is included."),
    MediaChanged => ("RISK.MEDIA_CHANGED", "An existing export filename refers to different bytes."),
    MediaRemoved => ("RISK.MEDIA_REMOVED", "A previously published media filename is omitted."),
}
impl fmt::Display for RiskCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A publication threshold and explicit allowances for entire risk categories.
/// Default blocks High and Critical findings, with no allowances. Evidence
/// errors cannot be accepted through any policy.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "pass this policy to BuildOptions or CompareOptions"]
pub struct UpdatePolicy {
    threshold: RiskLevel,
    allowances: BTreeSet<RiskCode>,
}
impl Default for UpdatePolicy {
    fn default() -> Self {
        Self {
            threshold: RiskLevel::High,
            allowances: BTreeSet::new(),
        }
    }
}
impl UpdatePolicy {
    /// Blocks unaccepted findings at or above this severity.
    pub fn fail_on(mut self, threshold: RiskLevel) -> Self {
        self.threshold = threshold;
        self
    }
    /// Accepts every finding in this category, preserving its original risk and
    /// evidence. Split the update when only some changes should be accepted.
    pub fn allow(mut self, code: RiskCode) -> Self {
        self.allowances.insert(code);
        self
    }

    pub(super) fn evaluate(&self, findings: &[RiskFinding]) -> PolicyEvaluation {
        let observed: BTreeSet<_> = findings.iter().map(RiskFinding::code).collect();
        PolicyEvaluation {
            threshold: self.threshold,
            allowed_codes: self.allowances.intersection(&observed).copied().collect(),
            unmatched_allowances: self.allowances.difference(&observed).copied().collect(),
            blocking_findings: findings
                .iter()
                .filter(|f| f.level() >= self.threshold && !self.allowances.contains(&f.code()))
                .cloned()
                .collect(),
        }
    }
}
