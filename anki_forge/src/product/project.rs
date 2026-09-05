mod counts;
mod paths;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::authoring_core::{normalize_with_options, NormalizationRequest, NormalizeOptions};
use crate::writer_core::{artifact_path_from_ref, BuildArtifactTarget, BuildContext, WriterPolicy};
use anyhow::Context;
use base64::Engine as _;
use tempfile::TempDir;

use counts::{card_count_from_inspect_or_fallback, count_phase1_cards_without_inspect};
use paths::BuildPathPlan;

use crate::build::{
    ApkgArtifact, BuildCounts, BuildError, BuildFailureCause, BuildMetrics, BuildOptions,
    BuildPolicyResult, BuildReport, BuildStatus, ComparisonStatus, MediaSourceMode, MediaSummary,
    ProjectMediaMode, ProjectNormalizeOptions,
};
use crate::diagnostics::{
    Diagnostic, DiagnosticCode, ErrorCode, ErrorCodeExt, Severity, SourcePath, ValidationReport,
};
use crate::product::lowering::ProductSourceMap;
use crate::product::{
    GenerationRule, LoweringDiagnostic, LoweringPlan, Note, NoteType, ProductDiagnostic,
    ProductDocument, ProductLoweringError, STOCK_BASIC_ID, STOCK_CLOZE_ID,
    STOCK_IMAGE_OCCLUSION_ID,
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
    product_document_source: Option<ProductDocument>,
}

enum NotetypeDuplicateFirst<'a> {
    ImplicitStock,
    Project { index: usize, name: Option<&'a str> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAddError {
    diagnostic: Box<Diagnostic>,
}

impl ProjectAddError {
    fn new(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostic: Box::new(diagnostic),
        }
    }

    pub fn diagnostic(&self) -> &Diagnostic {
        self.diagnostic.as_ref()
    }

    pub fn code(&self) -> ErrorCode {
        self.diagnostic.code.error_code()
    }
}

impl std::fmt::Display for ProjectAddError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.diagnostic.code, self.diagnostic.message)
    }
}

impl std::error::Error for ProjectAddError {}

impl ErrorCodeExt for ProjectAddError {
    fn code(&self) -> ErrorCode {
        ProjectAddError::code(self)
    }
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
            product_document_source: None,
        }
    }

    pub fn from_product_document(document: ProductDocument) -> Self {
        let name = document.document_id().to_string();
        let default_deck = document.default_deck_name().map(str::to_string);
        Self {
            name: name.clone(),
            stable_id: Some(name),
            default_deck,
            note_types: Vec::new(),
            notes: Vec::new(),
            media: crate::product::MediaRegistry::default(),
            deck_source: None,
            product_document_source: Some(document),
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

    pub fn add_notetype(&mut self, note_type: NoteType) -> Result<&mut Self, ProjectAddError> {
        self.validate_notetype_for_add(&note_type)?;
        self.note_types.push(note_type);
        Ok(self)
    }

    pub fn add_note(&mut self, note: Note) -> Result<&mut Self, ProjectAddError> {
        self.validate_note_for_add(&note)?;
        self.notes.push(note);
        Ok(self)
    }

    pub fn import_template_bundle(
        &mut self,
        root: impl AsRef<std::path::Path>,
    ) -> Result<&mut Self, crate::product::TemplateBundleError> {
        let loaded = crate::product::template_bundle::load_template_bundle(root)?;
        self.validate_notetype_for_add(&loaded.note_type)
            .map_err(|error| {
                crate::product::TemplateBundleError::new(
                    error.diagnostic().code.as_str(),
                    error.diagnostic().message.clone(),
                    error
                        .diagnostic()
                        .source
                        .as_ref()
                        .map(|source| std::path::PathBuf::from(source.as_str())),
                )
                .with_byte_offset(error.diagnostic().source_span().map(|span| span.byte_start))
            })?;

        let mut staged_media = self.media.clone();
        for asset in loaded.assets {
            staged_media
                .add_file(&asset.path)
                .and_then(|pending| pending.export_as(&asset.export_as))
                .map_err(|error| {
                    crate::product::TemplateBundleError::new(
                        "TEMPLATE.BUNDLE_ASSET_INVALID",
                        error.to_string(),
                        Some(asset.path.clone()),
                    )
                })?;
        }

        self.note_types.push(loaded.note_type);
        self.media = staged_media;
        Ok(self)
    }

    pub fn media_mut(&mut self) -> &mut crate::product::MediaRegistry {
        &mut self.media
    }

    pub fn validate(&self) -> ValidationReport {
        let mut diagnostics = Vec::new();
        let mut seen_stable_ids = BTreeSet::new();

        if let Some(diagnostic) = self.product_document_source_mixed_diagnostic() {
            diagnostics.push(diagnostic);
            return ValidationReport { diagnostics };
        }

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
                    domain: None,
                    stage: None,
                    message: error.to_string(),
                    source: Some(SourcePath::new("project.deck")),
                    help: Some("inspect deck notes before building".into()),
                }),
            }

            if !self.notes.is_empty() || !self.note_types.is_empty() {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.DECK_SOURCE_AUTHORING_STATE_UNSUPPORTED"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
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
                        domain: None,
                        stage: None,
                        message: "stable_id cannot be blank".into(),
                        source: Some(SourcePath::new(format!("project.notes[{index}]"))),
                        help: Some("choose a non-empty stable_id or omit it".into()),
                    });
                } else if !seen_stable_ids.insert(stable_id) {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("AFID.STABLE_ID_DUPLICATE"),
                        severity: Severity::Error,
                        domain: None,
                        stage: None,
                        message: format!("duplicate stable_id '{stable_id}'"),
                        source: Some(SourcePath::new(format!("project.notes[{index}]"))),
                        help: Some("choose a unique stable_id for each note".into()),
                    });
                }
            }

            if !is_supported_stock_notetype_id(note.note_type_id())
                && !custom_note_type_ids.contains(note.note_type_id())
            {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.UNSUPPORTED_NOTE_TYPE"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
                    message: format!(
                        "note type '{}' is not registered on the project",
                        note.note_type_id()
                    ),
                    source: Some(SourcePath::new(format!("project.notes[{index}]"))),
                    help: Some("add a matching NoteType with Project::add_notetype".into()),
                });
            }
        }

        let implicit_stock_notetype_ids = self.implicit_stock_notetype_ids();
        let mut notetype_id_counts = BTreeMap::<&str, usize>::new();
        for stock_id in &implicit_stock_notetype_ids {
            *notetype_id_counts.entry(*stock_id).or_default() += 1;
        }
        for note_type in &self.note_types {
            *notetype_id_counts.entry(note_type.id()).or_default() += 1;
        }
        let mut first_notetype_by_id = BTreeMap::<&str, NotetypeDuplicateFirst<'_>>::new();
        for stock_id in &implicit_stock_notetype_ids {
            first_notetype_by_id.insert(*stock_id, NotetypeDuplicateFirst::ImplicitStock);
        }
        for (index, note_type) in self.note_types.iter().enumerate() {
            if let Some(first) = first_notetype_by_id.get(note_type.id()) {
                let message = match first {
                    NotetypeDuplicateFirst::ImplicitStock => {
                        duplicate_implicit_stock_notetype_message(
                            note_type.id(),
                            index,
                            note_type.name_ref(),
                        )
                    }
                    NotetypeDuplicateFirst::Project {
                        index: first_index,
                        name: first_name,
                    } => duplicate_notetype_message(
                        note_type.id(),
                        *first_index,
                        *first_name,
                        index,
                        note_type.name_ref(),
                    ),
                };
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("NOTETYPE.ID_DUPLICATE"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
                    message,
                    source: Some(SourcePath::new(format!("project.note_types[{index}]"))),
                    help: Some("choose a unique id for each custom note type".into()),
                });
            } else {
                first_notetype_by_id.insert(
                    note_type.id(),
                    NotetypeDuplicateFirst::Project {
                        index,
                        name: note_type.name_ref(),
                    },
                );
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
                    domain: None,
                    stage: None,
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

            let field_names = note_type
                .fields()
                .iter()
                .map(|field| field.name())
                .collect::<Vec<_>>();
            for template in note_type.templates() {
                for (location, source) in [
                    ("front", Some(template.front_source())),
                    ("back", Some(template.back_source())),
                    ("browser_front", template.browser_front_source()),
                    ("browser_back", template.browser_back_source()),
                ] {
                    let Some(source) = source else {
                        continue;
                    };
                    for issue in
                        crate::product::TemplateEngine::validate(source.as_str(), &field_names)
                            .into_iter()
                            .filter(|issue| {
                                issue.severity == crate::product::TemplateIssueSeverity::Warning
                            })
                    {
                        diagnostics.push(Diagnostic {
                            code: DiagnosticCode::new(issue.code),
                            severity: Severity::Warning,
                            domain: None,
                            stage: None,
                            message: issue.message,
                            source: Some(SourcePath::new(
                                source.origin().map(str::to_string).unwrap_or_else(|| {
                                    format!(
                                        "{}.templates[{:?}].{location}",
                                        note_type_source,
                                        template.name()
                                    )
                                }),
                            )),
                            help: Some(format!(
                                "replace or explicitly support the filter near byte offset {}",
                                issue.byte_offset
                            )),
                        });
                    }
                }
            }

            for field in note_type.fields() {
                if field.key_auto_derived() {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("NOTETYPE.FIELD_KEY_AUTO_DERIVED"),
                        severity: Severity::Warning,
                        domain: None,
                        stage: None,
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

    fn validate_notetype_for_add(&self, note_type: &NoteType) -> Result<(), ProjectAddError> {
        let notetype_index = self.note_types.len();
        let notetype_source = format!("project.note_types[{notetype_index}]");
        let note_type_id = note_type.id();

        if note_type_id.trim().is_empty() {
            return Err(project_add_error(
                "NOTETYPE.ID_BLANK",
                "note type id cannot be blank",
                notetype_source,
                "choose a non-empty custom note type id",
            ));
        }

        if is_supported_stock_notetype_id(note_type_id) {
            return Err(project_add_error(
                "NOTETYPE.ID_RESERVED",
                format!("custom note type id '{note_type_id}' is reserved for stock notes"),
                notetype_source,
                "choose a non-stock custom note type id",
            ));
        }

        if let Some((first_index, first)) = self
            .note_types
            .iter()
            .enumerate()
            .find(|(_, existing)| existing.id() == note_type_id)
        {
            return Err(project_add_error(
                "NOTETYPE.ID_DUPLICATE",
                duplicate_notetype_message(
                    note_type_id,
                    first_index,
                    first.name_ref(),
                    notetype_index,
                    note_type.name_ref(),
                ),
                notetype_source,
                "choose a unique id for each custom note type",
            ));
        }

        let mut seen_field_keys = BTreeMap::<&str, &str>::new();
        let mut seen_field_names = BTreeSet::<&str>::new();
        let mut sort_field: Option<&str> = None;
        for field in note_type.fields() {
            let key = field.key_ref().as_str();
            if let Some(first_name) = seen_field_keys.insert(key, field.name()) {
                return Err(project_add_error(
                    "NOTETYPE.FIELD_KEY_DUPLICATE",
                    format!(
                        "custom note type '{note_type_id}' uses field key '{key}' for both '{first_name}' and '{}'",
                        field.name()
                    ),
                    format!("project.note_types[{notetype_index}].fields[{:?}]", field.name()),
                    "choose a unique key for each field",
                ));
            }

            if !seen_field_names.insert(field.name()) {
                return Err(project_add_error(
                    "NOTETYPE.FIELD_NAME_DUPLICATE",
                    format!(
                        "custom note type '{note_type_id}' uses duplicate field name '{}'",
                        field.name()
                    ),
                    format!(
                        "project.note_types[{notetype_index}].fields[{:?}]",
                        field.name()
                    ),
                    "choose a unique name for each field",
                ));
            }

            if field.is_sort() {
                if let Some(first_sort) = sort_field {
                    return Err(project_add_error(
                        "NOTETYPE.SORT_FIELD_DUPLICATE",
                        format!(
                            "custom note type '{note_type_id}' marks both '{first_sort}' and '{}' as sort fields",
                            field.name()
                        ),
                        format!("project.note_types[{notetype_index}].fields[{:?}]", field.name()),
                        "mark at most one field as the sort field",
                    ));
                }
                sort_field = Some(field.name());
            }
        }

        let field_keys = note_type
            .fields()
            .iter()
            .map(|field| field.key_ref().as_str())
            .collect::<BTreeSet<_>>();
        let field_names = note_type
            .fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>();

        if let crate::product::NoteTypeKind::Cloze { field } = note_type.kind() {
            let Some(cloze_field) = note_type
                .fields()
                .iter()
                .find(|candidate| candidate.key_ref() == field)
            else {
                return Err(project_add_error(
                    "TEMPLATE.CLOZE_FIELD_UNKNOWN",
                    format!(
                        "custom Cloze note type '{note_type_id}' references unknown field key '{}'",
                        field.as_str()
                    ),
                    notetype_source,
                    "declare the Cloze field on the custom NoteType",
                ));
            };
            if note_type.templates().len() != 1 {
                return Err(project_add_error(
                    "TEMPLATE.CLOZE_TEMPLATE_COUNT_INVALID",
                    format!(
                        "custom Cloze note type '{note_type_id}' must declare exactly one template"
                    ),
                    notetype_source,
                    "keep one card template for the custom Cloze note type",
                ));
            }
            let template = &note_type.templates()[0];
            let referenced =
                crate::product::TemplateEngine::cloze_fields(template.front_source().as_str());
            if !referenced.contains(cloze_field.name()) {
                return Err(project_add_error(
                    "TEMPLATE.CLOZE_FILTER_REQUIRED",
                    format!(
                        "custom Cloze template '{}' must render field '{}' with the cloze filter",
                        template.name(),
                        cloze_field.name()
                    ),
                    format!(
                        "project.note_types[{notetype_index}].templates[{:?}].front",
                        template.name()
                    ),
                    format!("add {{{{cloze:{}}}}}", cloze_field.name()),
                ));
            }
            if referenced.iter().any(|name| name != cloze_field.name()) {
                return Err(project_add_error(
                    "TEMPLATE.CLOZE_FIELD_MISMATCH",
                    format!(
                        "custom Cloze template '{}' renders a field other than declared Cloze field '{}'",
                        template.name(),
                        cloze_field.name()
                    ),
                    format!(
                        "project.note_types[{notetype_index}].templates[{:?}].front",
                        template.name()
                    ),
                    "use the declared Cloze field in every cloze filter".to_string(),
                ));
            }
        }

        let mut seen_template_keys = BTreeMap::<&str, &str>::new();
        let mut seen_template_names = BTreeSet::<&str>::new();
        for template in note_type.templates() {
            let key = template.key_ref().as_str();
            if let Some(first_name) = seen_template_keys.insert(key, template.name()) {
                return Err(project_add_error(
                    "NOTETYPE.TEMPLATE_KEY_DUPLICATE",
                    format!(
                        "custom note type '{note_type_id}' uses template key '{key}' for both '{first_name}' and '{}'",
                        template.name()
                    ),
                    format!(
                        "project.note_types[{notetype_index}].templates[{:?}]",
                        template.name()
                    ),
                    "choose a unique key for each template",
                ));
            }

            if !seen_template_names.insert(template.name()) {
                return Err(project_add_error(
                    "NOTETYPE.TEMPLATE_NAME_DUPLICATE",
                    format!(
                        "custom note type '{note_type_id}' uses duplicate template name '{}'",
                        template.name()
                    ),
                    format!(
                        "project.note_types[{notetype_index}].templates[{:?}]",
                        template.name()
                    ),
                    "choose a unique name for each template",
                ));
            }

            for field_key in generation_rule_field_keys(template.generation_rule()) {
                if matches!(
                    (note_type.kind(), template.generation_rule()),
                    (
                        crate::product::NoteTypeKind::Normal,
                        crate::product::GenerationRule::Cloze { .. }
                    )
                ) {
                    return Err(project_add_error(
                        "TEMPLATE.CLOZE_RULE_REQUIRES_CLOZE_NOTETYPE",
                        format!(
                            "template '{}' uses a Cloze generation rule on normal note type '{note_type_id}'",
                            template.name()
                        ),
                        format!(
                            "project.note_types[{notetype_index}].templates[{:?}]",
                            template.name()
                        ),
                        "construct the note type with NoteType::custom_cloze",
                    ));
                }
                if !field_keys.contains(field_key) {
                    return Err(project_add_error(
                        "TEMPLATE.FIELD_UNKNOWN",
                        format!(
                            "template '{}' for note type '{note_type_id}' references unknown field key '{field_key}'",
                            template.name()
                        ),
                        format!(
                            "project.note_types[{notetype_index}].templates[{:?}]",
                            template.name()
                        ),
                        "use field keys declared on the custom NoteType",
                    ));
                }
            }

            let template_source = format!(
                "project.note_types[{notetype_index}].templates[{:?}]",
                template.name()
            );
            for (location, source) in [
                ("front", Some(template.front_source())),
                ("back", Some(template.back_source())),
                ("browser_front", template.browser_front_source()),
                ("browser_back", template.browser_back_source()),
            ] {
                let Some(source) = source else {
                    continue;
                };
                if let Some(issue) =
                    crate::product::TemplateEngine::validate(source.as_str(), &field_names)
                        .into_iter()
                        .find(|issue| {
                            issue.severity == crate::product::TemplateIssueSeverity::Error
                        })
                {
                    return Err(project_add_error(
                        issue.code,
                        issue.message,
                        source
                            .origin()
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("{template_source}.{location}")),
                        format!(
                            "fix the template expression near byte offset {}",
                            issue.byte_offset
                        ),
                    ));
                }
            }
        }

        if let Some(recipe) = note_type.identity_ref() {
            for field_key in recipe.field_keys() {
                if !field_keys.contains(field_key.as_str()) {
                    return Err(project_add_error(
                        "PRODUCT.IDENTITY_FIELD_UNKNOWN",
                        format!(
                            "custom note type '{note_type_id}' identity references unknown field key '{}'",
                            field_key.as_str()
                        ),
                        notetype_source,
                        "use field keys declared on the custom NoteType",
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_note_for_add(&self, note: &Note) -> Result<(), ProjectAddError> {
        let note_index = self.notes.len();
        let note_source = format!("project.notes[{note_index}]");
        let note_type_id = note.note_type_id();

        if note_type_id.trim().is_empty() {
            return Err(project_add_error(
                "PROJECT.UNSUPPORTED_NOTE_TYPE",
                "note type id cannot be blank",
                note_source,
                "choose a stock note type id or add a matching custom NoteType",
            ));
        }

        if let Some(stable_id) = note.stable_id_ref() {
            if stable_id.trim().is_empty() {
                return Err(project_add_error(
                    "AFID.STABLE_ID_BLANK",
                    "stable_id cannot be blank",
                    format!("project.notes[{note_index}]"),
                    "choose a non-empty stable_id or omit it",
                ));
            }
        }

        if let Some(stable_id) = note.stable_id_ref() {
            if let Some((existing_index, _)) = self
                .notes
                .iter()
                .enumerate()
                .filter_map(|(index, note)| {
                    note.stable_id_ref().map(|stable_id| (index, stable_id))
                })
                .find(|(_, existing)| !existing.trim().is_empty() && *existing == stable_id)
            {
                return Err(project_add_error(
                    "AFID.STABLE_ID_DUPLICATE",
                    format!(
                        "duplicate stable_id '{stable_id}' at project.notes[{note_index}]; first definition is project.notes[{existing_index}]"
                    ),
                    format!("project.notes[{note_index}]"),
                    "choose a unique stable_id for each note",
                ));
            }
        }

        let matching_custom = self
            .note_types
            .iter()
            .find(|note_type| note_type.id() == note_type_id);

        if is_supported_stock_notetype_id(note_type_id) && matching_custom.is_some() {
            return Err(project_add_error(
                "NOTETYPE.ID_RESERVED",
                format!("stock note type id '{note_type_id}' is already registered as custom"),
                note_source,
                "remove the custom note type or use a non-stock custom note type id",
            ));
        }

        if !is_supported_stock_notetype_id(note_type_id) && matching_custom.is_none() {
            return Err(project_add_error(
                "PROJECT.UNSUPPORTED_NOTE_TYPE",
                format!("note type '{note_type_id}' is not registered on the project"),
                note_source,
                "add a matching NoteType with Project::add_notetype",
            ));
        }

        if let Some(allowed_keys) = stock_field_keys(note_type_id) {
            for field_key in note.field_keys() {
                if !allowed_keys.contains(&field_key) {
                    return Err(project_add_error(
                        "PRODUCT.FIELD_UNKNOWN",
                        format!(
                            "stock note for note type '{note_type_id}' contains unknown field key '{field_key}'"
                        ),
                        format!("project.notes[{note_index}].fields[{field_key:?}]"),
                        "use the exact Rust Product stock field name",
                    ));
                }
            }
            return Ok(());
        }

        let Some(note_type) = matching_custom else {
            return Ok(());
        };

        for field_key in note.field_keys() {
            let field_known = note_type
                .fields()
                .iter()
                .any(|field| field.key_ref().as_str() == field_key || field.name() == field_key);
            if !field_known {
                return Err(project_add_error(
                    "PRODUCT.FIELD_UNKNOWN",
                    format!(
                        "custom note for note type '{note_type_id}' contains unknown field key '{field_key}'"
                    ),
                    format!("project.notes[{note_index}].fields[{field_key:?}]"),
                    "use a field key or field name declared on the custom NoteType",
                ));
            }
        }

        if let Some(recipe) = note.identity_ref() {
            for field_key in recipe.field_keys() {
                let field_known = note_type
                    .fields()
                    .iter()
                    .any(|field| field.key_ref().as_str() == field_key.as_str());
                if !field_known {
                    return Err(project_add_error(
                        "PRODUCT.IDENTITY_FIELD_UNKNOWN",
                        format!(
                            "note identity for note type '{note_type_id}' references unknown field key '{}'",
                            field_key.as_str()
                        ),
                        format!("project.notes[{note_index}]"),
                        "use field keys declared on the custom NoteType",
                    ));
                }
            }
        }

        if note.stable_id_ref().is_none()
            && note.identity_ref().is_none()
            && note_type.identity_ref().is_none()
        {
            return Err(project_add_error(
                "PRODUCT.IDENTITY_MISSING",
                format!(
                    "custom note type '{note_type_id}' needs an explicit stable_id or identity recipe"
                ),
                format!("project.notes[{note_index}]"),
                "set Note::stable_id(...), Note::identity(...), or NoteType::identity(...)",
            ));
        }

        Ok(())
    }

    /// Lowers this project into an authoring plan for inspection or serialization.
    ///
    /// `lower()` returns a self-contained `LoweringPlan`. Product media registered
    /// with `media_mut().add_file(...)` is verified, read from disk, and embedded
    /// as inline base64 bytes in that plan, so callers do not receive hidden
    /// absolute source paths or need a media input directory.
    ///
    /// Because the self-contained form uses inline media, file-backed media must
    /// fit within the inline media limit. Larger file media can make `lower()`
    /// fail with `MEDIA.INLINE_TOO_LARGE`. The default build path uses
    /// path-backed media staging instead, and keeps `add_file(...)` assets
    /// path-backed until normalization unless `BuildOptions::self_contained()`
    /// is selected explicitly.
    pub fn lower(&self) -> anyhow::Result<LoweringPlan> {
        if self.product_document_source_mixed_diagnostic().is_some() {
            return Err(ProductLoweringError {
                product_diagnostics: vec![ProductDiagnostic {
                    code: "PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED",
                    message: "ProductDocument-backed projects cannot mix direct Project notes, note types, media, or deck sources".to_string(),
                    source_path: Some("project".to_string()),
                    byte_offset: None,
                }],
                lowering_diagnostics: vec![],
            }
            .into());
        }

        if let Some(product) = &self.product_document_source {
            return product.lower().map_err(anyhow::Error::from);
        }

        if let Some(deck) = &self.deck_source {
            let product = deck.clone().into_product_document()?;
            let mut plan = product.lower().map_err(anyhow::Error::from)?;
            self.apply_note_source_paths(&mut plan);
            self.apply_notetype_source_paths(&mut plan);
            return Ok(plan);
        }

        let mut plan = self
            .to_product_document()
            .lower()
            .map_err(anyhow::Error::from)?;
        self.apply_note_source_paths(&mut plan);
        self.apply_notetype_source_paths(&mut plan);
        plan.authoring_document
            .media
            .extend(product_media_to_authoring_media(self.media.media())?);
        record_project_media_source_paths(&mut plan, self.media.media());
        Ok(plan)
    }

    #[cfg(feature = "internal-tools")]
    pub fn normalize(&self) -> anyhow::Result<crate::authoring_core::NormalizedIr> {
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
        self.build_with_writer_stack_source(options, None)
    }

    pub(crate) fn build_with_writer_stack(
        &self,
        options: BuildOptions,
        writer_policy: WriterPolicy,
        build_context: BuildContext,
    ) -> Result<BuildReport, BuildError> {
        self.build_with_writer_stack_source(options, Some((writer_policy, build_context)))
    }

    fn build_with_writer_stack_source(
        &self,
        options: BuildOptions,
        writer_stack: Option<(WriterPolicy, BuildContext)>,
    ) -> Result<BuildReport, BuildError> {
        let started = Instant::now();
        if let Err(diagnostic) = BuildPathPlan::new(&options).validate() {
            let cause = if diagnostic.code.as_str() == "PROJECT.PATH_COLLISION" {
                BuildFailureCause::Diagnostics
            } else {
                BuildFailureCause::Io
            };
            let mut report = failure_report(
                started,
                diagnostic.code.as_str(),
                diagnostic.message.clone(),
            );
            report.status = if matches!(cause, BuildFailureCause::Io) {
                BuildStatus::Error
            } else {
                BuildStatus::Invalid
            };
            report.diagnostics = vec![*diagnostic];
            return return_report_error(&options, report, cause);
        }
        let baseline = options.compare_to.as_deref().map(|path| {
            crate::product::comparison::BaselineSnapshot::capture_with_limits(
                path,
                &options.inspect_limits,
            )
        });
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
                    media_source_modes,
                } = error;
                diagnostics.append(&mut normalize_diagnostics);
                deduplicate_diagnostics(&mut diagnostics);
                diagnostics.push(Diagnostic {
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
                            &diagnostics,
                            &media_source_modes,
                        )
                    })
                    .unwrap_or_default();
                let report = BuildReport {
                    artifact: None,
                    counts,
                    media,
                    diagnostics,
                    metrics: BuildMetrics {
                        duration: started.elapsed(),
                    },
                    inspect: None,
                    previous_inspect: None,
                    update_safety: None,
                    comparison: ComparisonStatus::NotRequested,
                    diff: None,
                    risk: None,
                    policy: BuildPolicyResult::default(),
                    status: BuildStatus::Invalid,
                };
                return return_report_error(&options, report, BuildFailureCause::Diagnostics);
            }
        };
        let mut normalized = normalized_output.normalized_ir;
        let media_source_modes = normalized_output.media_source_modes;
        diagnostics.extend(normalized_output.diagnostics);
        deduplicate_diagnostics(&mut diagnostics);

        if normalized.notes.is_empty() {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("PROJECT.EMPTY"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: "project contains no notes".into(),
                source: Some(SourcePath::new("project.notes")),
                help: Some("add at least one note before building".into()),
            });
            let report = invalid_report_without_artifact(
                diagnostics,
                &normalized,
                &media_source_modes,
                started,
            );
            return return_report_error(&options, report, BuildFailureCause::Invalid);
        }

        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            let media = MediaSummary::from_normalized_ir_with_source_modes(
                &normalized,
                &diagnostics,
                &media_source_modes,
            );
            let report = BuildReport {
                artifact: None,
                counts: BuildCounts {
                    notes: normalized.notes.len(),
                    cards: count_phase1_cards_without_inspect(&normalized),
                    media: normalized.media_bindings.len(),
                },
                media,
                diagnostics,
                metrics: BuildMetrics {
                    duration: started.elapsed(),
                },
                inspect: None,
                previous_inspect: None,
                update_safety: None,
                comparison: ComparisonStatus::NotRequested,
                diff: None,
                risk: None,
                policy: BuildPolicyResult::default(),
                status: BuildStatus::Invalid,
            };
            return return_report_error(&options, report, BuildFailureCause::Diagnostics);
        }

        let (writer_policy, build_context) = match writer_stack {
            Some((writer_policy, build_context)) => (writer_policy, build_context),
            None => {
                let (_runtime, writer_policy, build_context) =
                    crate::runtime::load_default_writer_stack().map_err(|err| {
                        let report = failure_report(
                            started,
                            "PROJECT.RUNTIME_DEFAULTS_FAILED",
                            err.to_string(),
                        );
                        match maybe_write_report_json(&options, report) {
                            Ok(report) => BuildError::new(report, BuildFailureCause::Io),
                            Err(err) => err,
                        }
                    })?;
                (writer_policy, build_context)
            }
        };
        let stable_ref_prefix = self
            .stable_id
            .as_deref()
            .map(|stable_id| format!("artifacts/{stable_id}"))
            .unwrap_or_else(|| "artifacts".into());
        let resolved_note_identities = self.resolved_note_identities();

        let update_mode = match crate::update_safety::effective_mode(&options) {
            Ok(mode) => mode,
            Err(err) => {
                diagnostics.push(Diagnostic {
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
                    &diagnostics,
                    &media_source_modes,
                );
                let report = BuildReport {
                    artifact: None,
                    counts: BuildCounts {
                        notes: normalized.notes.len(),
                        cards: count_phase1_cards_without_inspect(&normalized),
                        media: normalized.media_bindings.len(),
                    },
                    media,
                    diagnostics,
                    metrics: BuildMetrics {
                        duration: started.elapsed(),
                    },
                    inspect: None,
                    previous_inspect: None,
                    update_safety: None,
                    comparison: ComparisonStatus::NotRequested,
                    diff: None,
                    risk: None,
                    policy: BuildPolicyResult::default(),
                    status: BuildStatus::Invalid,
                };
                return return_report_error(&options, report, BuildFailureCause::Diagnostics);
            }
        };

        let project_stable_id_required =
            if matches!(update_mode, crate::update_safety::EffectiveMode::Disabled) {
                options.write_identity_lockfile
            } else {
                true
            };

        if self.stable_id.is_none() && project_stable_id_required {
            let condition =
                if options.identity_lockfile.is_some() || options.write_identity_lockfile {
                    crate::update_safety::EvidenceCondition::LockfileRequired
                } else {
                    crate::update_safety::EvidenceCondition::StrictCompareOnly
                };
            let classified = crate::update_safety::classify_project_stable_id_missing(condition);
            if let Some(code) = classified.diagnostic_code {
                diagnostics.push(Diagnostic {
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
                project_stable_id: self.stable_id.as_deref(),
                normalized: &normalized,
                writer_policy: &writer_policy,
                mode: update_mode,
                resolved_note_identities: &resolved_note_identities,
            },
        );
        diagnostics.extend(current_identity.diagnostics);
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            let media = MediaSummary::from_normalized_ir_with_source_modes(
                &normalized,
                &diagnostics,
                &media_source_modes,
            );
            let report = BuildReport {
                artifact: None,
                counts: BuildCounts {
                    notes: normalized.notes.len(),
                    cards: count_phase1_cards_without_inspect(&normalized),
                    media: normalized.media_bindings.len(),
                },
                media,
                diagnostics,
                metrics: BuildMetrics {
                    duration: started.elapsed(),
                },
                inspect: None,
                previous_inspect: None,
                update_safety: None,
                comparison: ComparisonStatus::NotRequested,
                diff: None,
                risk: None,
                policy: BuildPolicyResult::default(),
                status: BuildStatus::Invalid,
            };
            return return_report_error(&options, report, BuildFailureCause::Diagnostics);
        }

        let disabled_update_safety =
            matches!(update_mode, crate::update_safety::EffectiveMode::Disabled);

        let (reconcile, writer_guid_plan, update_safety_summary_val, lockfile_index) =
            if disabled_update_safety {
                let mut baseline_sources = Vec::new();
                if let Some(path) = options.compare_to.as_ref() {
                    diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("UPDATE.BASELINE_IGNORED_DISABLED"),
                    severity: Severity::Info,
                    domain: None,
                    stage: None,
                    message: "compare_to baseline ignored because update safety is disabled".into(),
                    source: Some(SourcePath::new(path.display().to_string())),
                    help: Some("remove update_safety(UpdateSafetyMode::Disabled) to analyze the baseline".into()),
                });
                    baseline_sources.push(
                        crate::update_safety::report::ignored_previous_apkg_source(path),
                    );
                }
                if let Some(path) = options.identity_lockfile.as_ref() {
                    baseline_sources
                        .push(crate::update_safety::report::ignored_lockfile_source(path));
                }
                let reconcile = crate::update_safety::reconcile::current_only_reconcile(
                    &current_identity.index,
                )
                .map_err(|_err| {
                    let report = BuildReport {
                        artifact: None,
                        counts: BuildCounts {
                            notes: normalized.notes.len(),
                            cards: count_phase1_cards_without_inspect(&normalized),
                            media: normalized.media_bindings.len(),
                        },
                        media: MediaSummary::from_normalized_ir_with_source_modes(
                            &normalized,
                            &diagnostics,
                            &media_source_modes,
                        ),
                        diagnostics: diagnostics.clone(),
                        metrics: BuildMetrics {
                            duration: started.elapsed(),
                        },
                        inspect: None,
                        previous_inspect: None,
                        update_safety: None,
                        comparison: ComparisonStatus::NotRequested,
                        diff: None,
                        risk: None,
                        policy: BuildPolicyResult::default(),
                        status: BuildStatus::Invalid,
                    };
                    match maybe_write_report_json(&options, report) {
                        Ok(report) => BuildError::new(report, BuildFailureCause::Diagnostics),
                        Err(err) => err,
                    }
                })?;
                let writer_guid_plan = crate::writer_core::WriterGuidPlan {
                    assignments: reconcile.assignments.clone(),
                };
                let summary = crate::update_safety::report::summary_from_disabled_mode(
                    &current_identity.index,
                    baseline_sources,
                    diagnostics
                        .iter()
                        .filter(|item| item.severity == Severity::Error)
                        .map(|item| item.code.to_string())
                        .collect(),
                );
                (reconcile, writer_guid_plan, summary, None)
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
                                    &mut diagnostics,
                                    self.stable_id.as_deref(),
                                    Some(lockfile.project_stable_id.as_str()),
                                    path.display().to_string(),
                                    update_error_severity,
                                );
                                Some(lockfile)
                            }
                            Err(err) => {
                                diagnostics.push(Diagnostic {
                                    code: DiagnosticCode::new(
                                        "UPDATE.BASELINE_LOCKFILE_UNREADABLE",
                                    ),
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
                        diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("UPDATE.BASELINE_LOCKFILE_UNREADABLE"),
                        severity: update_error_severity,
                        domain: None,
                        stage: None,
                        message: format!("identity lockfile {} does not exist", path.display()),
                        source: Some(SourcePath::new(path.display().to_string())),
                        help: Some("run with write_identity_lockfile(true) to create the first lockfile".into()),
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
                                &mut diagnostics,
                                self.stable_id.as_deref(),
                                index.project_stable_id.as_deref(),
                                path.display().to_string(),
                                update_error_severity,
                            );
                            Some(index)
                        }
                        Err(err) => {
                            diagnostics.push(Diagnostic {
                                code: DiagnosticCode::new("UPDATE.BASELINE_APKG_UNREADABLE"),
                                severity: update_error_severity,
                                domain: None,
                                stage: None,
                                message: err.to_string(),
                                source: Some(SourcePath::new(path.display().to_string())),
                                help: Some(
                                    "verify the previous APKG path and package contents".into(),
                                ),
                            });
                            crate::product::comparison::push_resource_diagnostic(
                                &mut diagnostics,
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
                        diagnostics.push(Diagnostic {
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
                    let update_safety_summary =
                        crate::update_safety::report::summary_from_reconcile(
                            update_mode,
                            &crate::update_safety::reconcile::current_only_reconcile(
                                &current_identity.index,
                            )
                            .map_err(|_err| {
                                let report = BuildReport {
                                    artifact: None,
                                    counts: BuildCounts {
                                        notes: normalized.notes.len(),
                                        cards: count_phase1_cards_without_inspect(&normalized),
                                        media: normalized.media_bindings.len(),
                                    },
                                    media: MediaSummary::from_normalized_ir_with_source_modes(
                                        &normalized,
                                        &diagnostics,
                                        &media_source_modes,
                                    ),
                                    diagnostics: diagnostics.clone(),
                                    metrics: BuildMetrics {
                                        duration: started.elapsed(),
                                    },
                                    inspect: None,
                                    previous_inspect: None,
                                    update_safety: None,
                                    comparison: ComparisonStatus::NotRequested,
                                    diff: None,
                                    risk: None,
                                    policy: BuildPolicyResult::default(),
                                    status: BuildStatus::Invalid,
                                };
                                match maybe_write_report_json(&options, report) {
                                    Ok(report) => {
                                        BuildError::new(report, BuildFailureCause::Diagnostics)
                                    }
                                    Err(err) => err,
                                }
                            })?,
                            &diagnostics,
                            baseline_sources.clone(),
                            false,
                        );
                    let risk =
                        baseline_unavailable_risk(&diagnostics, Some(&update_safety_summary));
                    let policy = crate::risk::policy_from_risk_report(options.fail_on, Some(&risk));
                    let status = BuildStatus::highest([
                        BuildStatus::Invalid,
                        policy_status(&policy),
                        diagnostics_status(&diagnostics),
                    ]);
                    let report = BuildReport {
                        artifact: None,
                        counts: BuildCounts {
                            notes: normalized.notes.len(),
                            cards: count_phase1_cards_without_inspect(&normalized),
                            media: normalized.media_bindings.len(),
                        },
                        media: MediaSummary::from_normalized_ir_with_source_modes(
                            &normalized,
                            &diagnostics,
                            &media_source_modes,
                        ),
                        diagnostics: diagnostics.clone(),
                        metrics: BuildMetrics {
                            duration: started.elapsed(),
                        },
                        inspect: None,
                        previous_inspect: None,
                        update_safety: Some(update_safety_summary),
                        comparison: ComparisonStatus::Unavailable,
                        diff: None,
                        risk: Some(risk),
                        policy,
                        status,
                    };
                    return return_report_error(&options, report, BuildFailureCause::Diagnostics);
                }

                let reconcile = crate::update_safety::reconcile::reconcile_guid_plan(
                    &current_identity.index,
                    previous_index.as_ref(),
                    lf_index.as_ref(),
                )
                .map_err(|err| {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("UPDATE.GUID_DUPLICATE_AT_RECONCILE"),
                        severity: Severity::Error,
                        domain: None,
                        stage: None,
                        message: err.to_string(),
                        source: Some(SourcePath::new("update_safety.reconcile")),
                        help: Some(
                            "choose unique stable ids or remove conflicting lockfile entries"
                                .into(),
                        ),
                    });
                    let report = BuildReport {
                        artifact: None,
                        counts: BuildCounts {
                            notes: normalized.notes.len(),
                            cards: count_phase1_cards_without_inspect(&normalized),
                            media: normalized.media_bindings.len(),
                        },
                        media: MediaSummary::from_normalized_ir_with_source_modes(
                            &normalized,
                            &diagnostics,
                            &media_source_modes,
                        ),
                        diagnostics: diagnostics.clone(),
                        metrics: BuildMetrics {
                            duration: started.elapsed(),
                        },
                        inspect: None,
                        previous_inspect: None,
                        update_safety: None,
                        comparison: ComparisonStatus::NotRequested,
                        diff: None,
                        risk: None,
                        policy: BuildPolicyResult::default(),
                        status: BuildStatus::Invalid,
                    };
                    match maybe_write_report_json(&options, report) {
                        Ok(report) => BuildError::new(report, BuildFailureCause::Diagnostics),
                        Err(err) => err,
                    }
                })?;
                diagnostics.extend(reconcile.diagnostics.clone());
                let mut model_diagnostics =
                    crate::update_safety::notetype_ids::reconcile_notetype_ids(
                        &mut current_identity.index,
                        previous_index.as_ref(),
                        lf_index.as_ref(),
                    );
                let model_id_collision = model_diagnostics.iter().any(|diagnostic| {
                    diagnostic.code.as_str() == "UPDATE.NOTETYPE_MODEL_ID_COLLISION"
                });
                if matches!(update_mode, crate::update_safety::EffectiveMode::ReportOnly) {
                    for diagnostic in &mut model_diagnostics {
                        if diagnostic.code.as_str() == "UPDATE.NOTETYPE_MODEL_ID_MISSING" {
                            diagnostic.severity = Severity::Warning;
                        }
                    }
                }
                diagnostics.extend(model_diagnostics);
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
                diagnostics.extend(revision_diagnostics);
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
                    diagnostics.extend(merge_diagnostics);
                }
                if model_id_collision
                    || revision_overflow
                    || (matches!(update_mode, crate::update_safety::EffectiveMode::Strict)
                        && diagnostics
                            .iter()
                            .any(|diagnostic| diagnostic.severity == Severity::Error))
                {
                    let update_safety_summary =
                        crate::update_safety::report::summary_from_reconcile(
                            update_mode,
                            &reconcile,
                            &diagnostics,
                            baseline_sources.clone(),
                            false,
                        );
                    let risk = crate::risk::classify_import_risk(crate::risk::rules::RiskInput {
                        diagnostics: &diagnostics,
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
                        diagnostics_status(&diagnostics),
                    ]);
                    let report = BuildReport {
                        artifact: None,
                        counts: BuildCounts {
                            notes: normalized.notes.len(),
                            cards: count_phase1_cards_without_inspect(&normalized),
                            media: normalized.media_bindings.len(),
                        },
                        media: MediaSummary::from_normalized_ir_with_source_modes(
                            &normalized,
                            &diagnostics,
                            &media_source_modes,
                        ),
                        diagnostics: diagnostics.clone(),
                        metrics: BuildMetrics {
                            duration: started.elapsed(),
                        },
                        inspect: None,
                        previous_inspect: None,
                        update_safety: Some(update_safety_summary),
                        comparison: ComparisonStatus::NotRequested,
                        diff: None,
                        risk: Some(risk),
                        policy,
                        status,
                    };
                    return return_report_error(&options, report, BuildFailureCause::Diagnostics);
                }
                let writer_guid_plan = crate::writer_core::WriterGuidPlan {
                    assignments: reconcile.assignments.clone(),
                };
                let summary = crate::update_safety::report::summary_from_reconcile(
                    update_mode,
                    &reconcile,
                    &diagnostics,
                    baseline_sources,
                    false,
                );
                (reconcile, writer_guid_plan, summary, lockfile_index)
            };

        let lockfile_evidence_unverified =
            matches!(update_mode, crate::update_safety::EffectiveMode::ReportOnly)
                && diagnostics.iter().any(|diagnostic| {
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
            diagnostics.push(Diagnostic {
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

        let candidate_dir = artifact_workspace.create_candidate_dir().map_err(|error| {
            let report = failure_report(started, "PROJECT.ARTIFACTS_DIR_FAILED", error.to_string());
            match maybe_write_report_json(&options, report) {
                Ok(report) => BuildError::new(report, BuildFailureCause::Io),
                Err(error) => error,
            }
        })?;
        let artifact_target =
            BuildArtifactTarget::new(artifacts_dir.clone(), stable_ref_prefix.clone())
                .with_media_store_dir(media_store_dir.clone());
        let apkg_target = BuildArtifactTarget::new(candidate_dir.path(), stable_ref_prefix)
            .with_media_store_dir(media_store_dir);
        let notetype_ids = current_identity
            .index
            .notetypes
            .iter()
            .filter_map(|notetype| {
                notetype
                    .anki_model_id
                    .map(|id| (notetype.note_type_id.clone(), id))
            })
            .collect::<BTreeMap<_, _>>();
        let package_build_result = crate::writer_core::build_with_identity_plan(
            &normalized,
            &writer_policy,
            &build_context,
            &artifact_target,
            &apkg_target,
            Some(&writer_guid_plan),
            Some(&notetype_ids),
        )
        .map_err(|err| {
            let report = failure_report(started, "PROJECT.WRITER_FAILED", err.to_string());
            match maybe_write_report_json(&options, report) {
                Ok(report) => BuildError::new(report, BuildFailureCause::Internal),
                Err(err) => err,
            }
        })?;

        diagnostics.extend(package_build_result.diagnostics.items.iter().map(|item| {
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
            && !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            diagnostics.push(Diagnostic {
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
                let report =
                    failure_report(started, "PROJECT.ARTIFACT_REF_FAILED", error.to_string());
                match maybe_write_report_json(&options, report) {
                    Ok(report) => BuildError::new(report, BuildFailureCause::Io),
                    Err(error) => error,
                }
            })?
            .map(|path| ApkgArtifact { path });

        let media = MediaSummary::from_normalized_ir_with_source_modes(
            &normalized,
            &diagnostics,
            &media_source_modes,
        );
        let writer_status = build_status_from_writer_result(&package_build_result.result_status);
        let mut inspect = None;
        let mut previous_inspect = None;
        let mut comparison = ComparisonStatus::NotRequested;
        let mut diff = None;
        let mut risk = None;
        let mut policy = BuildPolicyResult::default();
        let mut status = BuildStatus::highest([writer_status, diagnostics_status(&diagnostics)]);
        if let Some(artifact) = artifact.as_ref() {
            let comparison_output = crate::product::comparison::assemble_comparison_with_limits(
                crate::product::comparison::ComparisonInput {
                    current_artifact: &artifact.path,
                    previous_artifact: options.compare_to.as_deref(),
                    diagnostics: &diagnostics,
                    update_safety: Some(&update_safety_summary_val),
                    started,
                },
                baseline.as_ref(),
                &options.inspect_limits,
            );
            diagnostics = comparison_output.diagnostics;
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
                downgrade_compare_errors_to_warnings(&mut diagnostics);
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
                diagnostics_status(&diagnostics),
            ]);
        }
        let counts = BuildCounts {
            notes: normalized.notes.len(),
            cards: card_count_from_inspect_or_fallback(inspect.as_ref(), &normalized),
            media: normalized.media_bindings.len(),
        };
        let mut report = BuildReport {
            artifact,
            counts,
            media,
            diagnostics,
            metrics: BuildMetrics {
                duration: started.elapsed(),
            },
            inspect,
            previous_inspect,
            update_safety: Some(update_safety_summary_val),
            comparison,
            diff,
            risk,
            policy,
            status,
        };

        if let Err(error) = report.ensure_success() {
            // Candidate files are private and cleaned up on return. A rejected
            // report must not advertise a path that was never published.
            report.artifact = None;
            return return_report_error(&options, report, error.cause);
        }
        if let Err(diagnostic) = BuildPathPlan::new(&options).validate() {
            report.artifact = None;
            return return_path_validation_error(&options, report, *diagnostic);
        }

        let candidate = report.artifact.take().expect("successful candidate exists");
        match artifact_workspace.publish(&candidate.path, &options) {
            Ok(path) => report.artifact = Some(ApkgArtifact { path }),
            Err(error) => {
                report.status = BuildStatus::Error;
                report.diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("PROJECT.OUTPUT_WRITE_FAILED"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
                    source: Some(SourcePath::new("build.output")),
                    message: error.to_string(),
                    help: None,
                });
                return return_report_error(&options, report, BuildFailureCause::Io);
            }
        }
        // Keep any published temporary artifact valid even if a later lockfile
        // or report write fails. The separate candidate directory still drops.
        artifact_workspace.persist_if_requested();
        // New paths can only reveal case-folding aliases after creation. Keep
        // the published APKG intact and reject any conflicting follow-up write.
        if let Err(diagnostic) = BuildPathPlan::new(&options).validate() {
            return return_path_validation_error(&options, report, *diagnostic);
        }
        if write_identity_lockfile {
            if let Some(path) = options.identity_lockfile.as_ref() {
                let selected_index = crate::update_safety::reconcile::selected_identity_index(
                    &current_identity.index,
                    &reconcile,
                    lockfile_index.as_ref(),
                );
                let writer_policy_ref =
                    crate::writer_core::policy_ref(&writer_policy.id, &writer_policy.version);
                let lockfile = crate::update_safety::model::IdentityLockfile {
                    schema_version: "identity-lockfile-v1".into(),
                    project_stable_id: self
                        .stable_id
                        .clone()
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
                    report.status = BuildStatus::Error;
                    report.diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("UPDATE.LOCKFILE_WRITE_FAILED"),
                        severity: Severity::Error,
                        domain: None,
                        stage: None,
                        source: Some(SourcePath::new(path.display().to_string())),
                        message: error.to_string(),
                        help: Some("verify the lockfile path is writable".into()),
                    });
                    return return_report_error(&options, report, BuildFailureCause::Io);
                }
                if let Some(summary) = report.update_safety.as_mut() {
                    summary.lockfile_written = true;
                }
            }
        }
        report.metrics.duration = started.elapsed();
        maybe_write_report_json(&options, report)
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> Result<BuildReport, BuildError> {
        self.build(BuildOptions::new().output(path.as_ref().to_path_buf()))
    }

    pub fn diff_against_apkg(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<crate::diff::ProjectDiffReport, crate::diff::ProjectDiffError> {
        self.diff_against_apkg_with_limits(path, crate::writer_core::InspectLimits::default())
    }

    /// Compare with an explicitly selected, finite APKG inspection budget.
    pub fn diff_against_apkg_with_limits(
        &self,
        path: impl AsRef<Path>,
        limits: crate::writer_core::InspectLimits,
    ) -> Result<crate::diff::ProjectDiffReport, crate::diff::ProjectDiffError> {
        let started = Instant::now();
        let temp = tempfile::Builder::new()
            .prefix("anki-forge-project-diff-")
            .tempdir()
            .map_err(|err| {
                let report = crate::diff::ProjectDiffReport {
                    status: BuildStatus::Error,
                    comparison: ComparisonStatus::Unavailable,
                    diagnostics: vec![Diagnostic {
                        code: DiagnosticCode::new("DIFF.TEMP_DIR_FAILED"),
                        severity: Severity::Error,
                        domain: None,
                        stage: None,
                        message: err.to_string(),
                        source: Some(SourcePath::new("project.diff_against_apkg")),
                        help: Some(
                            "verify that the system temporary directory is writable".to_string(),
                        ),
                    }],
                    current_inspect: None,
                    previous_inspect: None,
                    update_safety: None,
                    diff: None,
                    risk: None,
                    metrics: crate::diff::ComparisonMetrics { duration_ms: 0 },
                };
                crate::diff::ProjectDiffError::new(report, BuildFailureCause::Io)
            })?;
        let current_path = temp.path().join("current.apkg");

        let build = self.build(
            BuildOptions::new()
                .output(&current_path)
                .inspect(true)
                .inspect_limits(limits)
                .compare_to(path.as_ref())
                .update_safety(crate::build::UpdateSafetyMode::ReportOnly),
        );
        let build_report = match build {
            Ok(report) => report,
            Err(err) => {
                let report = crate::diff::ProjectDiffReport {
                    status: err.report.status,
                    comparison: ComparisonStatus::Unavailable,
                    diagnostics: err.report.diagnostics.clone(),
                    current_inspect: err.report.inspect.clone(),
                    previous_inspect: err.report.previous_inspect.clone(),
                    update_safety: err.report.update_safety.clone(),
                    diff: None,
                    risk: err.report.risk.clone(),
                    metrics: crate::diff::ComparisonMetrics {
                        duration_ms: started.elapsed().as_millis(),
                    },
                };
                return Err(crate::diff::ProjectDiffError::new(report, err.cause));
            }
        };

        if build_report.artifact.is_none() {
            let report = crate::diff::ProjectDiffReport {
                status: BuildStatus::Invalid,
                comparison: ComparisonStatus::Unavailable,
                diagnostics: build_report.diagnostics.clone(),
                current_inspect: build_report.inspect.clone(),
                previous_inspect: build_report.previous_inspect.clone(),
                update_safety: build_report.update_safety.clone(),
                diff: None,
                risk: build_report.risk.clone(),
                metrics: crate::diff::ComparisonMetrics {
                    duration_ms: started.elapsed().as_millis(),
                },
            };
            return Err(crate::diff::ProjectDiffError::new(
                report,
                BuildFailureCause::Invalid,
            ));
        }

        let status = if build_report.comparison == ComparisonStatus::Unavailable {
            BuildStatus::Invalid
        } else {
            build_report.status
        };
        let report = crate::diff::ProjectDiffReport {
            status,
            comparison: build_report.comparison,
            diagnostics: build_report.diagnostics,
            current_inspect: build_report.inspect,
            previous_inspect: build_report.previous_inspect,
            update_safety: build_report.update_safety,
            diff: build_report.diff,
            risk: build_report.risk,
            metrics: crate::diff::ComparisonMetrics {
                duration_ms: started.elapsed().as_millis(),
            },
        };

        if report.status == BuildStatus::Success {
            Ok(report)
        } else {
            let cause = if report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error)
            {
                BuildFailureCause::Diagnostics
            } else {
                BuildFailureCause::Invalid
            };
            Err(crate::diff::ProjectDiffError::new(report, cause))
        }
    }

    fn to_product_document(&self) -> ProductDocument {
        let document_id = self.stable_id.clone().unwrap_or_else(|| self.name.clone());
        let default_deck = self
            .default_deck
            .clone()
            .unwrap_or_else(|| self.name.clone());
        let implicit_stock_notetype_ids = self.implicit_stock_notetype_ids();
        let mut note_type_id_counts = BTreeMap::<&str, usize>::new();
        for stock_id in &implicit_stock_notetype_ids {
            *note_type_id_counts.entry(*stock_id).or_default() += 1;
        }
        for note_type in &self.note_types {
            *note_type_id_counts.entry(note_type.id()).or_default() += 1;
        }
        let mut note_types = implicit_stock_notetype_ids
            .into_iter()
            .map(|stock_id| {
                crate::product::model::ProductNoteTypeV2::Stock(
                    crate::product::model::ProductStockNoteTypeV2 {
                        id: stock_id.to_string(),
                        name: None,
                        fields: Vec::new(),
                        templates: Vec::new(),
                        css: None,
                        source_path: None,
                    },
                )
            })
            .collect::<Vec<_>>();

        note_types.extend(
            self.note_types
                .iter()
                .enumerate()
                .map(|(index, note_type)| {
                    let note_type_source = if note_type_id_counts
                        .get(note_type.id())
                        .copied()
                        .unwrap_or_default()
                        > 1
                    {
                        format!("project.note_types[{index}]")
                    } else {
                        format!("project.note_types[{:?}]", note_type.id())
                    };
                    let (note_type_kind, cloze_field) = match note_type.kind() {
                        crate::product::NoteTypeKind::Normal => (Some("normal".to_string()), None),
                        crate::product::NoteTypeKind::Cloze { field } => {
                            (Some("cloze".to_string()), Some(field.as_str().to_string()))
                        }
                    };
                    let identity = note_type.identity_ref().map(|recipe| {
                        crate::product::model::ProductIdentityV2::Fields {
                            fields: recipe
                                .field_keys()
                                .iter()
                                .map(|field| field.as_str().to_string())
                                .collect(),
                        }
                    });
                    crate::product::model::ProductNoteTypeV2::Custom(
                        crate::product::model::ProductCustomNoteTypeV2 {
                            id: note_type.id().to_string(),
                            name: note_type.name_ref().map(ToOwned::to_owned),
                            note_type_kind,
                            cloze_field,
                            fields: note_type
                                .fields()
                                .iter()
                                .map(|field| crate::product::model::ProductFieldV2 {
                                    name: field.name().to_string(),
                                    key: field.key_ref().as_str().to_string(),
                                    identity: field.is_identity(),
                                    sort: field.is_sort(),
                                    required: field.is_required(),
                                    source_path: None,
                                })
                                .collect(),
                            templates: note_type
                                .templates()
                                .iter()
                                .map(|template| crate::product::model::ProductTemplateV2 {
                                    name: template.name().to_string(),
                                    key: template.key_ref().as_str().to_string(),
                                    front: template.front_source().as_str().to_string(),
                                    back: template.back_source().as_str().to_string(),
                                    browser_front: template
                                        .browser_front_source()
                                        .map(|source| source.as_str().to_string()),
                                    browser_back: template
                                        .browser_back_source()
                                        .map(|source| source.as_str().to_string()),
                                    target_deck: template.target_deck_name().map(ToOwned::to_owned),
                                    generation_rule: match note_type.kind() {
                                        crate::product::NoteTypeKind::Normal => Some(
                                            product_v3_generation_rule(template.generation_rule()),
                                        ),
                                        crate::product::NoteTypeKind::Cloze { .. } => None,
                                    },
                                    source_path: Some(format!(
                                        "{note_type_source}.templates[{:?}]",
                                        template.name()
                                    )),
                                })
                                .collect(),
                            identity,
                            css: note_type.css_ref().map(ToOwned::to_owned),
                            source_path: Some(note_type_source),
                        },
                    )
                }),
        );

        let stable_id_counts = self.note_stable_id_counts();
        let mut notes = Vec::with_capacity(self.notes.len());
        for (index, note) in self.notes.iter().enumerate() {
            let note_id =
                resolve_product_note_identity(self, note, index, &stable_id_counts).stable_id;
            let deck_name = note
                .deck_name()
                .unwrap_or(default_deck.as_str())
                .to_string();
            let fields = note.rendered_fields();
            if note.note_type_id() == STOCK_BASIC_ID {
                notes.push(product_v3_stock_note(
                    note,
                    note_id,
                    deck_name,
                    [("front", fields.get("Front")), ("back", fields.get("Back"))],
                    index,
                ));
            } else if note.note_type_id() == STOCK_CLOZE_ID {
                notes.push(product_v3_stock_note(
                    note,
                    note_id,
                    deck_name,
                    [
                        ("text", fields.get("Text")),
                        ("back_extra", fields.get("Back Extra")),
                    ],
                    index,
                ));
            } else if note.note_type_id() == STOCK_IMAGE_OCCLUSION_ID {
                notes.push(product_v3_stock_note(
                    note,
                    note_id,
                    deck_name,
                    [
                        ("occlusion", fields.get("Occlusion")),
                        ("image", fields.get("Image")),
                        ("header", fields.get("Header")),
                        ("back_extra", fields.get("Back Extra")),
                        ("comments", fields.get("Comments")),
                    ],
                    index,
                ));
            } else {
                let fields = custom_note_fields_for_product_v3(self, note)
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key,
                            crate::product::model::ProductFieldContentV2::Html { value },
                        )
                    })
                    .collect();
                notes.push(crate::product::model::ProductNoteV2::Custom(
                    crate::product::model::ProductCustomNoteV2 {
                        note_type_id: note.note_type_id().to_string(),
                        stable_id: Some(note_id),
                        deck_name,
                        fields,
                        tags: note.tags().to_vec(),
                        source_path: Some(format!("project.notes[{index}]")),
                    },
                ));
            }
        }
        ProductDocument::from_product_v3_parts(document_id, Some(default_deck), note_types, notes)
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

    fn resolved_note_identities(
        &self,
    ) -> BTreeMap<String, crate::update_safety::model::ResolvedNoteIdentity> {
        if let Some(product) = &self.product_document_source {
            if let Some(v2) = product.product_v2() {
                return product_v2_resolved_note_identities(v2);
            }
        }

        let stable_id_counts = self.note_stable_id_counts();
        self.notes
            .iter()
            .enumerate()
            .map(|(index, note)| {
                let identity = resolve_product_note_identity(self, note, index, &stable_id_counts);
                (identity.stable_id.clone(), identity)
            })
            .collect()
    }

    fn implicit_stock_notetype_ids(&self) -> Vec<&'static str> {
        supported_stock_notetype_ids()
            .iter()
            .copied()
            .filter(|stock_id| {
                self.notes
                    .iter()
                    .any(|note| note.note_type_id() == *stock_id)
            })
            .collect()
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
        let mut project_indexes_by_authoring_id = BTreeMap::<String, Vec<usize>>::new();
        for (project_index, product_note) in self.notes.iter().enumerate() {
            let identity =
                resolve_product_note_identity(self, product_note, project_index, &stable_id_counts);
            project_indexes_by_authoring_id
                .entry(identity.stable_id)
                .or_default()
                .push(project_index);
        }
        for authoring_note in &plan.authoring_document.notes {
            let Some(project_index) = project_indexes_by_authoring_id
                .get(&authoring_note.id)
                .filter(|indexes| indexes.len() == 1)
                .map(|indexes| indexes[0])
            else {
                continue;
            };
            let Some(product_note) = self.notes.get(project_index) else {
                continue;
            };
            let field_source_names = note_field_source_names_for_authoring(self, product_note);
            let note_source = match product_note.stable_id_ref() {
                Some(stable_id)
                    if !stable_id.trim().is_empty()
                        && stable_id_counts.get(stable_id).copied() == Some(1) =>
                {
                    format!("project.notes[{stable_id:?}]")
                }
                _ => format!("project.notes[{project_index}]"),
            };
            for field_name in authoring_note.fields.keys() {
                let product_field_name = field_source_names
                    .get(field_name)
                    .map(String::as_str)
                    .unwrap_or(field_name);
                plan.source_map.insert(
                    crate::product::lowering::authoring_note_field_path(
                        &authoring_note.id,
                        field_name,
                    ),
                    crate::product::lowering::product_note_field_source(
                        &note_source,
                        product_field_name,
                    ),
                );
            }
        }
    }

    fn apply_notetype_source_paths(&self, plan: &mut LoweringPlan) {
        if self.deck_source.is_some() {
            return;
        }

        let implicit_stock_notetype_ids = self.implicit_stock_notetype_ids();
        let mut id_counts = BTreeMap::new();
        for stock_id in &implicit_stock_notetype_ids {
            *id_counts.entry(*stock_id).or_insert(0usize) += 1;
        }
        for note_type in &self.note_types {
            *id_counts.entry(note_type.id()).or_insert(0usize) += 1;
        }

        let mut consumed_by_id = BTreeMap::new();
        for stock_id in &implicit_stock_notetype_ids {
            // Product custom note types start after any implicit stock notetype
            // with the same id in the lowered authoring document.
            consumed_by_id.insert((*stock_id).to_string(), 1usize);
        }
        let mut entries = Vec::new();
        for (project_index, note_type) in self.note_types.iter().enumerate() {
            if id_counts.get(note_type.id()).copied().unwrap_or_default() <= 1 {
                continue;
            }

            let consumed = consumed_by_id
                .entry(note_type.id().to_string())
                .or_insert(0usize);
            let authoring_match = plan
                .authoring_document
                .notetypes
                .iter()
                .enumerate()
                .filter(|(_, authoring_notetype)| authoring_notetype.id == note_type.id())
                .nth(*consumed);
            *consumed += 1;

            let Some((authoring_index, authoring_notetype)) = authoring_match else {
                continue;
            };
            let authoring_notetype_source = format!("authoring.note_types[{authoring_index}]");
            let project_notetype_source = format!("project.note_types[{project_index}]");

            if let Some(templates) = authoring_notetype.templates.as_ref() {
                for template in templates {
                    let authoring_template =
                        format!("{authoring_notetype_source}.templates[{:?}]", template.name);
                    let project_template =
                        format!("{project_notetype_source}.templates[{:?}]", template.name);
                    entries.push((
                        format!("{authoring_template}.front"),
                        format!("{project_template}.front"),
                    ));
                    entries.push((
                        format!("{authoring_template}.back"),
                        format!("{project_template}.back"),
                    ));
                    if template.browser_question_format.is_some() {
                        entries.push((
                            format!("{authoring_template}.browser_front"),
                            format!("{project_template}.browser_front"),
                        ));
                    }
                    if template.browser_answer_format.is_some() {
                        entries.push((
                            format!("{authoring_template}.browser_back"),
                            format!("{project_template}.browser_back"),
                        ));
                    }
                }
            }

            if authoring_notetype.css.is_some() {
                entries.push((
                    format!("{authoring_notetype_source}.css"),
                    format!("{project_notetype_source}.css"),
                ));
            }
        }

        for (authoring_path, project_source) in entries {
            plan.source_map.insert(authoring_path, project_source);
        }
    }

    fn normalize_with_dirs(
        &self,
        base_dir: impl Into<PathBuf>,
        media_store_dir: impl Into<PathBuf>,
        mut options: ProjectNormalizeOptions,
    ) -> Result<ProjectNormalizeOutput, ProjectNormalizeError> {
        if let Some(diagnostic) = self.product_document_source_mixed_diagnostic() {
            return Err(ProjectNormalizeError {
                message: format!("{}: {}", diagnostic.code.as_str(), diagnostic.message),
                diagnostics: vec![diagnostic],
                normalized_ir: None,
                media_source_modes: BTreeMap::new(),
            });
        }

        let base_dir = base_dir.into();
        let media_store_dir = media_store_dir.into();
        options.base_dir = options.base_dir.or(Some(base_dir.clone()));
        options.media_store_dir = options.media_store_dir.or(Some(media_store_dir.clone()));
        let mut lowering = self.lower_with_project_error()?;
        let product_diagnostics =
            map_product_diagnostics(std::mem::take(&mut lowering.product_diagnostics));
        let lowering_diagnostics =
            map_lowering_diagnostics(std::mem::take(&mut lowering.lowering_diagnostics));
        self.apply_note_source_paths(&mut lowering);
        self.apply_notetype_source_paths(&mut lowering);
        let media_mode = options.media_mode;
        if let Some(deck) = &self.deck_source {
            let media = deck
                .registered_media()
                .values()
                .map(|media| match media_mode {
                    ProjectMediaMode::PathBacked => media.to_authoring_media(&base_dir),
                    ProjectMediaMode::SelfContained => media.to_self_contained_authoring_media(),
                })
                .collect::<anyhow::Result<Vec<_>>>()
                .map_err(|error| ProjectNormalizeError {
                    message: "prepare deck media".into(),
                    diagnostics: vec![Diagnostic {
                        code: DiagnosticCode::new("PROJECT.DECK_MEDIA_FAILED"),
                        severity: Severity::Error,
                        domain: None,
                        stage: None,
                        message: error.to_string(),
                        source: Some(SourcePath::new("project.deck.media")),
                        help: Some("inspect deck media registrations and media paths".into()),
                    }],
                    normalized_ir: None,
                    media_source_modes: BTreeMap::new(),
                })?;
            lowering.authoring_document.media.extend(media);
            for filename in deck.registered_media().keys() {
                crate::product::lowering::record_media_source_path(
                    &mut lowering.source_map,
                    &format!("media:{filename}"),
                    filename,
                );
            }
        } else {
            let media = match media_mode {
                ProjectMediaMode::PathBacked => {
                    product_media_to_path_backed_authoring_media(self.media.media(), &base_dir)
                }
                ProjectMediaMode::SelfContained => {
                    product_media_to_self_contained_authoring_media(self.media.media())
                }
            }
            .map_err(|error| ProjectNormalizeError {
                message: error.message,
                diagnostics: error.diagnostics,
                normalized_ir: None,
                media_source_modes: BTreeMap::new(),
            })?;
            lowering.authoring_document.media.extend(media);
            record_project_media_source_paths(&mut lowering, self.media.media());
        }
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
        let source_map = lowering.source_map.clone();
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

    fn lower_with_project_error(&self) -> Result<LoweringPlan, ProjectNormalizeError> {
        if let Some(diagnostic) = self.product_document_source_mixed_diagnostic() {
            return Err(ProjectNormalizeError {
                message: format!("{}: {}", diagnostic.code.as_str(), diagnostic.message),
                diagnostics: vec![diagnostic],
                normalized_ir: None,
                media_source_modes: BTreeMap::new(),
            });
        }

        let product = if let Some(product) = &self.product_document_source {
            product.clone()
        } else if let Some(deck) = &self.deck_source {
            deck.clone()
                .into_product_document()
                .map_err(|error| ProjectNormalizeError {
                    message: "lower deck product document".into(),
                    diagnostics: vec![Diagnostic {
                        code: DiagnosticCode::new("PROJECT.DECK_LOWER_FAILED"),
                        severity: Severity::Error,
                        domain: None,
                        stage: None,
                        message: error.to_string(),
                        source: Some(SourcePath::new("project.deck")),
                        help: Some("inspect deck notes before lowering".into()),
                    }],
                    normalized_ir: None,
                    media_source_modes: BTreeMap::new(),
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
                media_source_modes: BTreeMap::new(),
            }
        })
    }

    fn product_document_source_mixed_diagnostic(&self) -> Option<Diagnostic> {
        if self.product_document_source.is_some()
            && (!self.notes.is_empty()
                || !self.note_types.is_empty()
                || self.deck_source.is_some()
                || self.media.media().next().is_some())
        {
            Some(Diagnostic {
                code: DiagnosticCode::new("PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: "ProductDocument-backed projects cannot mix direct Project notes, note types, media, or deck sources".to_string(),
                source: Some(SourcePath::new("project")),
                help: Some(
                    "build either a ProductDocument-backed Project or a builder-backed Project"
                        .to_string(),
                ),
            })
        } else {
            None
        }
    }
}

fn product_v3_generation_rule(
    rule: &crate::product::GenerationRule,
) -> crate::product::model::ProductGenerationRuleV2 {
    match rule {
        crate::product::GenerationRule::AnkiDefault => {
            crate::product::model::ProductGenerationRuleV2::AnkiDefault
        }
        crate::product::GenerationRule::All(fields) => {
            crate::product::model::ProductGenerationRuleV2::All {
                fields: fields
                    .iter()
                    .map(|field| field.as_str().to_string())
                    .collect(),
            }
        }
        crate::product::GenerationRule::Any(fields) => {
            crate::product::model::ProductGenerationRuleV2::Any {
                fields: fields
                    .iter()
                    .map(|field| field.as_str().to_string())
                    .collect(),
            }
        }
        crate::product::GenerationRule::Cloze { field } => {
            crate::product::model::ProductGenerationRuleV2::Cloze {
                field: field.as_str().to_string(),
            }
        }
    }
}

fn product_v3_stock_note<const N: usize>(
    note: &crate::product::Note,
    stable_id: String,
    deck_name: String,
    fields: [(&'static str, Option<&String>); N],
    index: usize,
) -> crate::product::model::ProductNoteV2 {
    crate::product::model::ProductNoteV2::Stock(crate::product::model::ProductStockNoteV2 {
        note_type_id: note.note_type_id().to_string(),
        stable_id: Some(stable_id),
        deck_name,
        fields: fields
            .into_iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    crate::product::model::ProductFieldContentV2::Html {
                        value: value.cloned().unwrap_or_default(),
                    },
                )
            })
            .collect(),
        tags: note.tags().to_vec(),
        source_path: Some(format!("project.notes[{index}]")),
    })
}

fn custom_note_fields_for_product_v3(
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

    let key_by_name = note_type
        .fields()
        .iter()
        .map(|field| (field.name(), field.key_ref().as_str()))
        .collect::<BTreeMap<_, _>>();
    let field_keys = note_type
        .fields()
        .iter()
        .map(|field| field.key_ref().as_str())
        .collect::<BTreeSet<_>>();

    let mut fields = BTreeMap::new();
    let mut field_priorities = BTreeMap::new();
    for (field_key_or_name, value) in rendered {
        let visible_name_key = key_by_name.get(field_key_or_name.as_str()).copied();
        let is_stable_key = field_keys.contains(field_key_or_name.as_str());
        let field_key = visible_name_key
            .unwrap_or(field_key_or_name.as_str())
            .to_string();
        let priority = if visible_name_key.is_some() {
            2
        } else if is_stable_key {
            1
        } else {
            0
        };
        if field_priorities
            .get(&field_key)
            .is_some_and(|existing| *existing > priority)
        {
            continue;
        }
        field_priorities.insert(field_key.clone(), priority);
        fields.insert(field_key, value);
    }
    fields
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

fn duplicate_implicit_stock_notetype_message(
    id: &str,
    duplicate_index: usize,
    duplicate_name: Option<&str>,
) -> String {
    format!(
        "duplicate note type id '{id}' at project.note_types[{duplicate_index}]{}; first definition is an implicit stock note type inserted for stock {id} notes",
        display_name_suffix(duplicate_name),
    )
}

const SUPPORTED_STOCK_NOTETYPE_IDS: &[&str] =
    &[STOCK_BASIC_ID, STOCK_CLOZE_ID, STOCK_IMAGE_OCCLUSION_ID];

fn supported_stock_notetype_ids() -> &'static [&'static str] {
    SUPPORTED_STOCK_NOTETYPE_IDS
}

fn is_supported_stock_notetype_id(id: &str) -> bool {
    supported_stock_notetype_ids().contains(&id)
}

fn stock_field_keys(note_type_id: &str) -> Option<&'static [&'static str]> {
    match note_type_id {
        STOCK_BASIC_ID => Some(&["Front", "Back"]),
        STOCK_CLOZE_ID => Some(&["Text", "Back Extra"]),
        STOCK_IMAGE_OCCLUSION_ID => {
            Some(&["Occlusion", "Image", "Header", "Back Extra", "Comments"])
        }
        _ => None,
    }
}

fn generation_rule_field_keys(rule: &GenerationRule) -> Vec<&str> {
    match rule {
        GenerationRule::AnkiDefault => Vec::new(),
        GenerationRule::All(fields) | GenerationRule::Any(fields) => {
            fields.iter().map(|field| field.as_str()).collect()
        }
        GenerationRule::Cloze { field } => vec![field.as_str()],
    }
}

fn project_add_error(
    code: &'static str,
    message: impl Into<String>,
    source: impl Into<String>,
    help: impl Into<String>,
) -> ProjectAddError {
    ProjectAddError::new(Diagnostic {
        code: DiagnosticCode::new(code),
        severity: Severity::Error,
        domain: None,
        stage: None,
        message: message.into(),
        source: Some(SourcePath::new(source.into())),
        help: Some(help.into()),
    })
}

fn display_name_suffix(name: Option<&str>) -> String {
    name.map(|name| format!(" ({name})")).unwrap_or_default()
}

fn product_v2_resolved_note_identities(
    v2: &crate::product::model::ProductDocumentV2Payload,
) -> BTreeMap<String, crate::update_safety::model::ResolvedNoteIdentity> {
    let custom_notetypes = v2
        .note_types
        .iter()
        .filter_map(|notetype| match notetype {
            crate::product::model::ProductNoteTypeV2::Custom(custom) => {
                Some((custom.id.clone(), custom))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let media_export_by_id = v2
        .media
        .iter()
        .map(|media| (media.id.clone(), media.export_as.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut identities = BTreeMap::new();

    for note in &v2.notes {
        let Some(identity) = product_v2_note_identity(note, &custom_notetypes, &media_export_by_id)
        else {
            continue;
        };
        identities.insert(identity.stable_id.clone(), identity);
    }

    identities
}

fn product_v2_note_identity(
    note: &crate::product::model::ProductNoteV2,
    custom_notetypes: &BTreeMap<String, &crate::product::model::ProductCustomNoteTypeV2>,
    media_export_by_id: &BTreeMap<String, String>,
) -> Option<crate::update_safety::model::ResolvedNoteIdentity> {
    match note {
        crate::product::model::ProductNoteV2::Stock(stock) => {
            if let Some(stable_id) = stock.stable_id.as_deref() {
                return Some(crate::update_safety::model::ResolvedNoteIdentity {
                    stable_id: stable_id.to_string(),
                    current_guid_candidate: stable_id.to_string(),
                    recipe_id: "product.explicit-stable-id.v1".into(),
                    canonical_payload_hash: None,
                    provenance: "ExplicitStableId".into(),
                    used_override: false,
                });
            }

            let stable_id = match stock.note_type_id.as_str() {
                STOCK_BASIC_ID => {
                    let front = stock
                        .fields
                        .get("front")
                        .map(|content| product_v2_identity_content(content, media_export_by_id))
                        .unwrap_or_default();
                    crate::deck::identity::derive_basic_stock_stable_id_from_front(&front).ok()?
                }
                STOCK_CLOZE_ID => {
                    let text = stock
                        .fields
                        .get("text")
                        .map(|content| product_v2_identity_content(content, media_export_by_id))
                        .unwrap_or_default();
                    crate::deck::identity::derive_cloze_stock_stable_id_from_text(&text).ok()?
                }
                _ => return None,
            };

            let recipe_id = match stock.note_type_id.as_str() {
                STOCK_BASIC_ID => crate::deck::identity::BASIC_RECIPE_ID,
                STOCK_CLOZE_ID => crate::deck::identity::CLOZE_RECIPE_ID,
                _ => return None,
            };
            Some(crate::update_safety::model::ResolvedNoteIdentity {
                stable_id: stable_id.clone(),
                current_guid_candidate: stable_id,
                recipe_id: recipe_id.into(),
                canonical_payload_hash: None,
                provenance: "InferredFromStockRecipe".into(),
                used_override: false,
            })
        }
        crate::product::model::ProductNoteV2::Custom(note) => {
            if let Some(stable_id) = note.stable_id.as_deref() {
                return Some(crate::update_safety::model::ResolvedNoteIdentity {
                    stable_id: stable_id.to_string(),
                    current_guid_candidate: stable_id.to_string(),
                    recipe_id: "product.explicit-stable-id.v1".into(),
                    canonical_payload_hash: None,
                    provenance: "ExplicitStableId".into(),
                    used_override: false,
                });
            }

            let notetype = custom_notetypes.get(&note.note_type_id)?;
            let field_by_key = notetype
                .fields
                .iter()
                .map(|field| (field.key.clone(), field))
                .collect::<BTreeMap<_, _>>();
            let identity_fields = match notetype.identity.as_ref() {
                Some(crate::product::model::ProductIdentityV2::Fields { fields }) => fields.clone(),
                Some(crate::product::model::ProductIdentityV2::Unknown(_)) => return None,
                None => notetype
                    .fields
                    .iter()
                    .filter(|field| field.identity)
                    .map(|field| field.key.clone())
                    .collect(),
            };
            if identity_fields.is_empty() {
                return None;
            }

            let selected_fields = identity_fields
                .into_iter()
                .map(|key| {
                    let field = field_by_key.get(&key)?;
                    let value = note
                        .fields
                        .get(&key)
                        .map(|content| product_v2_identity_content(content, media_export_by_id))
                        .map(|value| {
                            crate::deck::identity::normalize_field_text_for_identity(&value)
                        })
                        .unwrap_or_default();
                    Some(crate::product::identity::CustomIdentityFieldComponent {
                        key,
                        name: field.name.clone(),
                        value,
                    })
                })
                .collect::<Option<Vec<_>>>()?;

            Some(crate::product::identity::derive_custom_notetype_identity(
                &notetype.id,
                selected_fields,
            ))
        }
        crate::product::model::ProductNoteV2::Unknown(_) => None,
    }
}

fn product_v2_identity_content(
    content: &crate::product::model::ProductFieldContentV2,
    media_export_by_id: &BTreeMap<String, String>,
) -> String {
    match content {
        crate::product::model::ProductFieldContentV2::Text { value } => {
            crate::product::content::escape_html(value)
        }
        crate::product::model::ProductFieldContentV2::Html { value } => value.clone(),
        crate::product::model::ProductFieldContentV2::Sound { media_id } => media_export_by_id
            .get(media_id)
            .map(|export_as| format!("[sound:{export_as}]"))
            .unwrap_or_default(),
        crate::product::model::ProductFieldContentV2::Image { media_id } => media_export_by_id
            .get(media_id)
            .map(|export_as| {
                format!(
                    "<img src=\"{}\">",
                    crate::product::content::escape_html(export_as)
                )
            })
            .unwrap_or_default(),
        crate::product::model::ProductFieldContentV2::Unknown(_) => String::new(),
    }
}

fn resolve_product_note_identity(
    project: &Project,
    note: &crate::product::Note,
    index: usize,
    stable_id_counts: &BTreeMap<&str, usize>,
) -> crate::update_safety::model::ResolvedNoteIdentity {
    if let Some(stable_id) = note.stable_id_ref() {
        if !stable_id.trim().is_empty() && stable_id_counts.get(stable_id).copied() == Some(1) {
            return crate::update_safety::model::ResolvedNoteIdentity {
                stable_id: stable_id.to_string(),
                current_guid_candidate: stable_id.to_string(),
                recipe_id: "product.explicit-stable-id.v1".into(),
                canonical_payload_hash: None,
                provenance: "ExplicitStableId".into(),
                used_override: false,
            };
        }
    }

    if note.note_type_id() == STOCK_BASIC_ID {
        let rendered = note.rendered_fields();
        if let Ok(stable_id) = crate::deck::identity::derive_basic_stock_stable_id_from_front(
            rendered
                .get("Front")
                .map(String::as_str)
                .unwrap_or_default(),
        ) {
            return crate::update_safety::model::ResolvedNoteIdentity {
                stable_id: stable_id.clone(),
                current_guid_candidate: stable_id,
                recipe_id: crate::deck::identity::BASIC_RECIPE_ID.into(),
                canonical_payload_hash: None,
                provenance: "InferredFromStockRecipe".into(),
                used_override: false,
            };
        }
    }

    if note.note_type_id() == STOCK_CLOZE_ID {
        let rendered = note.rendered_fields();
        if let Ok(stable_id) = crate::deck::identity::derive_cloze_stock_stable_id_from_text(
            rendered.get("Text").map(String::as_str).unwrap_or_default(),
        ) {
            return crate::update_safety::model::ResolvedNoteIdentity {
                stable_id: stable_id.clone(),
                current_guid_candidate: stable_id,
                recipe_id: crate::deck::identity::CLOZE_RECIPE_ID.into(),
                canonical_payload_hash: None,
                provenance: "InferredFromStockRecipe".into(),
                used_override: false,
            };
        }
    }

    let note_type = project
        .note_types
        .iter()
        .find(|note_type| note_type.id() == note.note_type_id());

    if let (Some(note_type), Some(recipe)) = (note_type, note.identity_ref()) {
        return derive_product_note_identity(
            note_type,
            note,
            recipe,
            "custom.note-override.fields.v1",
            "InferredFromNoteFields",
            true,
        );
    }

    if let Some((note_type, recipe)) =
        note_type.and_then(|note_type| note_type.identity_ref().map(|recipe| (note_type, recipe)))
    {
        return derive_product_note_identity(
            note_type,
            note,
            recipe,
            "custom.notetype.fields.v1",
            "InferredFromNotetypeFields",
            false,
        );
    }

    let generated = format!("generated:{}", index + 1);
    crate::update_safety::model::ResolvedNoteIdentity {
        stable_id: generated.clone(),
        current_guid_candidate: generated,
        recipe_id: "product.generated-note-id.v1".into(),
        canonical_payload_hash: None,
        provenance: "unknown_baseline".into(),
        used_override: false,
    }
}

fn derive_product_note_identity(
    note_type: &NoteType,
    note: &crate::product::Note,
    recipe: &crate::product::IdentityRecipe,
    recipe_id: &str,
    provenance: &str,
    used_override: bool,
) -> crate::update_safety::model::ResolvedNoteIdentity {
    let rendered = note.rendered_fields();
    let field_by_key = note_type
        .fields()
        .iter()
        .map(|field| (field.key_ref().as_str(), field))
        .collect::<BTreeMap<_, _>>();
    let selected_fields = recipe
        .field_keys()
        .into_iter()
        .map(|key| {
            let key = key.as_str().to_string();
            let field_name = field_by_key
                .get(key.as_str())
                .map(|field| field.name().to_string())
                .unwrap_or_else(|| key.clone());
            let value = rendered
                .get(key.as_str())
                .or_else(|| rendered.get(field_name.as_str()))
                .map(|value| crate::deck::identity::normalize_field_text_for_identity(value))
                .unwrap_or_default();
            crate::product::identity::CustomIdentityFieldComponent {
                key,
                name: field_name,
                value,
            }
        })
        .collect();

    if recipe_id == "custom.notetype.fields.v1"
        && provenance == "InferredFromNotetypeFields"
        && !used_override
    {
        return crate::product::identity::derive_custom_notetype_identity(
            note_type.id(),
            selected_fields,
        );
    }

    crate::product::identity::derive_custom_identity(
        note_type.id(),
        recipe_id,
        provenance,
        used_override,
        selected_fields,
    )
}

fn note_field_source_names_for_authoring(
    project: &Project,
    note: &crate::product::Note,
) -> BTreeMap<String, String> {
    let rendered = note.rendered_fields();
    let Some(note_type) = project
        .note_types
        .iter()
        .find(|note_type| note_type.id() == note.note_type_id())
    else {
        return rendered
            .keys()
            .map(|field| (field.clone(), field.clone()))
            .collect();
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

    let mut sources = BTreeMap::new();
    let mut field_priorities = BTreeMap::new();
    for field_key_or_name in rendered.keys() {
        let is_visible_name = field_names.contains(field_key_or_name.as_str());
        let field_name = if is_visible_name {
            field_key_or_name.clone()
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
        sources.insert(field_name, field_key_or_name.clone());
    }
    sources
}

fn product_media_to_authoring_media<'a>(
    media: impl Iterator<Item = &'a crate::product::media_registry::ProductMedia>,
) -> anyhow::Result<Vec<crate::authoring::AuthoringMedia>> {
    let mut prepared = Vec::new();
    let mut diagnostics = Vec::new();

    for item in media {
        match product_media_item_to_authoring_media(item) {
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
        }
        .into())
    }
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

fn authoring_media_source_modes(
    media: &[crate::authoring::AuthoringMedia],
) -> BTreeMap<String, MediaSourceMode> {
    media
        .iter()
        .map(|item| {
            let mode = match &item.source {
                crate::authoring::AuthoringMediaSource::Path { .. } => MediaSourceMode::PathBacked,
                crate::authoring::AuthoringMediaSource::InlineBytes { .. } => {
                    MediaSourceMode::Inline
                }
            };
            (item.id.clone(), mode)
        })
        .collect()
}

fn product_media_to_self_contained_authoring_media<'a>(
    media: impl Iterator<Item = &'a crate::product::media_registry::ProductMedia>,
) -> Result<Vec<crate::authoring::AuthoringMedia>, ProductMediaPrepareError> {
    let mut prepared = Vec::new();
    let mut diagnostics = Vec::new();

    for item in media {
        match product_media_item_to_self_contained_authoring_media(item) {
            Ok(media) => prepared.push(media),
            Err(mut error) => diagnostics.append(&mut error.diagnostics),
        }
    }

    if diagnostics.is_empty() {
        Ok(prepared)
    } else {
        Err(ProductMediaPrepareError {
            message: "prepare self-contained product media".into(),
            diagnostics,
        })
    }
}

fn product_media_to_path_backed_authoring_media<'a>(
    media: impl Iterator<Item = &'a crate::product::media_registry::ProductMedia>,
    media_input_dir: &Path,
) -> Result<Vec<crate::authoring::AuthoringMedia>, ProductMediaPrepareError> {
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

fn product_media_item_to_self_contained_authoring_media(
    media: &crate::product::media_registry::ProductMedia,
) -> Result<crate::authoring::AuthoringMedia, ProductMediaPrepareError> {
    let source = match &media.source {
        crate::product::media_registry::ProductMediaSource::File { path } => {
            media
                .verify_registered_source()
                .map_err(ProductMediaPrepareError::from_source_diagnostic)?;
            let limit = crate::product::media_registry::INLINE_MEDIA_LIMIT_BYTES as u64;
            if media.observed_size_bytes() > limit {
                return Err(ProductMediaPrepareError::single(
                    "MEDIA.INLINE_TOO_LARGE",
                    format!(
                        "MEDIA.INLINE_TOO_LARGE: project.media[{filename:?}] has {} bytes, above inline limit {}",
                        media.observed_size_bytes(),
                        crate::product::media_registry::INLINE_MEDIA_LIMIT_BYTES,
                        filename = &media.export_filename,
                    ),
                    media.export_filename.clone(),
                ));
            }
            let bytes = std::fs::read(path).map_err(|err| {
                ProductMediaPrepareError::single(
                    if err.kind() == std::io::ErrorKind::NotFound {
                        "MEDIA.SOURCE_MISSING"
                    } else {
                        "MEDIA.SOURCE_READ_FAILED"
                    },
                    format!("read media source file {}: {err}", path.display()),
                    media.export_filename.clone(),
                )
            })?;
            crate::authoring::AuthoringMediaSource::InlineBytes {
                data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            }
        }
        crate::product::media_registry::ProductMediaSource::InlineBytes { data_base64, .. } => {
            crate::authoring::AuthoringMediaSource::InlineBytes {
                data_base64: data_base64.clone(),
            }
        }
    };

    Ok(crate::authoring::AuthoringMedia {
        id: media.id.clone(),
        desired_filename: media.export_filename.clone(),
        source,
        declared_mime: media.declared_mime.clone(),
    })
}

fn product_media_item_to_authoring_media(
    media: &crate::product::media_registry::ProductMedia,
) -> Result<crate::authoring::AuthoringMedia, ProductMediaPrepareError> {
    let source = match &media.source {
        crate::product::media_registry::ProductMediaSource::File { path } => {
            media
                .verify_registered_source()
                .map_err(ProductMediaPrepareError::from_source_diagnostic)?;
            if media.observed_size_bytes()
                > crate::product::media_registry::INLINE_MEDIA_LIMIT_BYTES as u64
            {
                return Err(ProductMediaPrepareError::single(
                    "MEDIA.INLINE_TOO_LARGE",
                    format!(
                        "project.media[{filename:?}] has {} bytes, above inline limit {}",
                        media.observed_size_bytes(),
                        crate::product::media_registry::INLINE_MEDIA_LIMIT_BYTES,
                        filename = &media.export_filename,
                    ),
                    media.export_filename.clone(),
                ));
            }
            let bytes = std::fs::read(path).map_err(|err| {
                let code = if err.kind() == std::io::ErrorKind::NotFound {
                    "MEDIA.SOURCE_MISSING"
                } else {
                    "MEDIA.SOURCE_READ_FAILED"
                };
                ProductMediaPrepareError::single(
                    code,
                    format!("read media source file {}: {err}", path.display()),
                    media.export_filename.clone(),
                )
            })?;
            crate::authoring::AuthoringMediaSource::InlineBytes {
                data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            }
        }
        crate::product::media_registry::ProductMediaSource::InlineBytes { data_base64, .. } => {
            crate::authoring::AuthoringMediaSource::InlineBytes {
                data_base64: data_base64.clone(),
            }
        }
    };

    Ok(crate::authoring::AuthoringMedia {
        id: media.id.clone(),
        desired_filename: media.export_filename.clone(),
        source,
        declared_mime: media.declared_mime.clone(),
    })
}

fn product_media_item_to_path_backed_authoring_media(
    media: &crate::product::media_registry::ProductMedia,
    media_input_dir: &Path,
) -> Result<crate::authoring::AuthoringMedia, ProductMediaPrepareError> {
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
                if target
                    .try_exists()
                    .with_context(|| format!("stat media input target: {}", target.display()))
                    .map_err(ProductMediaPrepareError::from_prepare_error)?
                {
                    return Err(ProductMediaPrepareError::staging_collision(
                        path.clone(),
                        target,
                        media.export_filename.clone(),
                    ));
                }
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
            crate::authoring::AuthoringMediaSource::Path {
                path: media.export_filename.clone(),
            }
        }
        crate::product::media_registry::ProductMediaSource::InlineBytes { data_base64, .. } => {
            crate::authoring::AuthoringMediaSource::InlineBytes {
                data_base64: data_base64.clone(),
            }
        }
    };

    Ok(crate::authoring::AuthoringMedia {
        id: media.id.clone(),
        desired_filename: media.export_filename.clone(),
        source,
        declared_mime: media.declared_mime.clone(),
    })
}

fn self_contain_authoring_path_media(
    media: &mut [crate::authoring::AuthoringMedia],
    base_dir: &Path,
    inline_limit: usize,
    source_map: &ProductSourceMap,
) -> Result<(), ProductMediaPrepareError> {
    let mut diagnostics = Vec::new();
    let mut canonical_base: Option<Result<PathBuf, String>> = None;

    for item in media {
        let crate::authoring::AuthoringMediaSource::Path { path } = &item.source else {
            continue;
        };
        let source_path =
            authoring_media_product_source(source_map, &item.id, &item.desired_filename);
        let base = match canonical_base
            .get_or_insert_with(|| {
                base_dir
                    .canonicalize()
                    .map_err(|err| format!("canonicalize base_dir {}: {err}", base_dir.display()))
            })
            .as_ref()
        {
            Ok(base) => base,
            Err(message) => {
                let mut error = authoring_path_media_error(
                    "MEDIA.UNSAFE_SOURCE_PATH",
                    message.clone(),
                    source_path,
                );
                diagnostics.append(&mut error.diagnostics);
                continue;
            }
        };

        match read_authoring_path_media_for_inline(ReadAuthoringPathMediaParams {
            path,
            base,
            inline_limit,
            filename: &item.desired_filename,
            source_path,
        }) {
            Ok(bytes) => {
                item.source = crate::authoring::AuthoringMediaSource::InlineBytes {
                    data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
                };
            }
            Err(mut error) => diagnostics.append(&mut error.diagnostics),
        }
    }

    if !diagnostics.is_empty() {
        return Err(ProductMediaPrepareError {
            message: "prepare self-contained authoring media".into(),
            diagnostics,
        });
    }
    Ok(())
}

fn authoring_media_product_source(
    source_map: &ProductSourceMap,
    media_id: &str,
    filename: &str,
) -> String {
    source_map
        .source_for_diagnostic_path(media_id)
        .or_else(|| source_map.source_for_diagnostic_path(filename))
        .map(str::to_owned)
        .unwrap_or_else(|| format!("project.media[{filename:?}]"))
}

struct ReadAuthoringPathMediaParams<'a> {
    path: &'a str,
    base: &'a Path,
    inline_limit: usize,
    filename: &'a str,
    source_path: String,
}

fn read_authoring_path_media_for_inline(
    params: ReadAuthoringPathMediaParams<'_>,
) -> Result<Vec<u8>, ProductMediaPrepareError> {
    let ReadAuthoringPathMediaParams {
        path,
        base,
        inline_limit,
        filename,
        source_path,
    } = params;
    let raw_path = Path::new(path);
    if path.is_empty() || raw_path.is_absolute() || path_has_parent_component(raw_path) {
        return Err(authoring_path_media_error(
            "MEDIA.UNSAFE_SOURCE_PATH",
            format!("source.path must be relative and stay below base_dir: {path}"),
            source_path,
        ));
    }

    let candidate = base.join(raw_path);
    let canonical = candidate.canonicalize().map_err(|err| {
        authoring_path_media_error(
            "MEDIA.SOURCE_MISSING",
            format!("read source.path {path}: {err}"),
            source_path.clone(),
        )
    })?;
    if !canonical.starts_with(base) {
        return Err(authoring_path_media_error(
            "MEDIA.UNSAFE_SOURCE_PATH",
            format!("source.path escapes base_dir: {path}"),
            source_path,
        ));
    }

    let metadata = std::fs::metadata(&canonical).map_err(|err| {
        authoring_path_media_error(
            "MEDIA.SOURCE_READ_FAILED",
            format!("stat media source {}: {err}", canonical.display()),
            source_path.clone(),
        )
    })?;
    if !metadata.is_file() {
        return Err(authoring_path_media_error(
            "MEDIA.SOURCE_NOT_REGULAR_FILE",
            format!(
                "media source must be a regular file: {}",
                canonical.display()
            ),
            source_path,
        ));
    }
    if metadata.len() > inline_limit as u64 {
        return Err(authoring_path_media_error(
            "MEDIA.INLINE_TOO_LARGE",
            format!(
                "MEDIA.INLINE_TOO_LARGE: project.media[{filename:?}] has {} bytes, above inline limit {inline_limit}",
                metadata.len(),
            ),
            source_path,
        ));
    }

    std::fs::read(&canonical).map_err(|err| {
        authoring_path_media_error(
            "MEDIA.SOURCE_READ_FAILED",
            format!("read media source file {}: {err}", canonical.display()),
            source_path,
        )
    })
}

fn authoring_path_media_error(
    code: &'static str,
    message: String,
    source_path: String,
) -> ProductMediaPrepareError {
    let help = product_diagnostic_help(code, Some(&source_path));
    ProductMediaPrepareError {
        message: "prepare self-contained authoring media".into(),
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new(code),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message,
            source: Some(SourcePath::new(source_path)),
            help,
        }],
    }
}

fn path_has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
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
fn metadata_file_identity(_metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    // Stable Rust does not expose a Windows volume/file index pair. The caller
    // falls back to canonical path comparison when metadata identity is absent.
    None
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

fn duplicate_notetype_media_reference_diagnostics(
    document: &crate::authoring_core::AuthoringDocument,
    source_map: &ProductSourceMap,
) -> Vec<Diagnostic> {
    let mut notetype_id_counts = BTreeMap::<&str, usize>::new();
    for notetype in &document.notetypes {
        *notetype_id_counts.entry(notetype.id.as_str()).or_default() += 1;
    }
    if !notetype_id_counts.values().any(|count| *count > 1) {
        return Vec::new();
    }

    let media_exports = document
        .media
        .iter()
        .map(|media| media.desired_filename.as_str())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();

    for (notetype_index, notetype) in document.notetypes.iter().enumerate() {
        if notetype_id_counts
            .get(notetype.id.as_str())
            .copied()
            .unwrap_or_default()
            < 2
        {
            continue;
        }

        let authoring_notetype_source = format!("authoring.note_types[{notetype_index}]");
        if let Some(templates) = notetype.templates.as_ref() {
            for template in templates {
                let authoring_template =
                    format!("{authoring_notetype_source}.templates[{:?}]", template.name);
                append_missing_media_reference_diagnostics(
                    &mut diagnostics,
                    &media_exports,
                    source_map,
                    MissingMediaReferenceScan {
                        authoring_path: &format!("{authoring_template}.front"),
                        owner_kind: "notetype",
                        owner_id: &notetype.id,
                        location_kind: "template_front",
                        location_name: &format!("{}:front", template.name),
                        value: &template.question_format,
                    },
                );
                append_missing_media_reference_diagnostics(
                    &mut diagnostics,
                    &media_exports,
                    source_map,
                    MissingMediaReferenceScan {
                        authoring_path: &format!("{authoring_template}.back"),
                        owner_kind: "notetype",
                        owner_id: &notetype.id,
                        location_kind: "template_back",
                        location_name: &format!("{}:back", template.name),
                        value: &template.answer_format,
                    },
                );
                if let Some(value) = template.browser_question_format.as_deref() {
                    append_missing_media_reference_diagnostics(
                        &mut diagnostics,
                        &media_exports,
                        source_map,
                        MissingMediaReferenceScan {
                            authoring_path: &format!("{authoring_template}.browser_front"),
                            owner_kind: "notetype",
                            owner_id: &notetype.id,
                            location_kind: "browser_template_front",
                            location_name: &format!("{}:browser_front", template.name),
                            value,
                        },
                    );
                }
                if let Some(value) = template.browser_answer_format.as_deref() {
                    append_missing_media_reference_diagnostics(
                        &mut diagnostics,
                        &media_exports,
                        source_map,
                        MissingMediaReferenceScan {
                            authoring_path: &format!("{authoring_template}.browser_back"),
                            owner_kind: "notetype",
                            owner_id: &notetype.id,
                            location_kind: "browser_template_back",
                            location_name: &format!("{}:browser_back", template.name),
                            value,
                        },
                    );
                }
            }
        }

        if let Some(css) = notetype.css.as_deref() {
            append_missing_media_reference_diagnostics(
                &mut diagnostics,
                &media_exports,
                source_map,
                MissingMediaReferenceScan {
                    authoring_path: &format!("{authoring_notetype_source}.css"),
                    owner_kind: "notetype",
                    owner_id: &notetype.id,
                    location_kind: "css",
                    location_name: "css",
                    value: css,
                },
            );
        }
    }

    diagnostics
}

struct MissingMediaReferenceScan<'a> {
    authoring_path: &'a str,
    owner_kind: &'a str,
    owner_id: &'a str,
    location_kind: &'a str,
    location_name: &'a str,
    value: &'a str,
}

fn append_missing_media_reference_diagnostics(
    diagnostics: &mut Vec<Diagnostic>,
    media_exports: &BTreeSet<&str>,
    source_map: &ProductSourceMap,
    scan: MissingMediaReferenceScan<'_>,
) {
    for candidate in crate::authoring_core::extract_media_reference_candidates(
        scan.owner_kind,
        scan.owner_id,
        scan.location_kind,
        scan.location_name,
        scan.value,
    ) {
        if candidate.skip_reason.is_some() || candidate.unsafe_reason.is_some() {
            continue;
        }
        let Some(local_ref) = candidate.normalized_local_ref.as_deref() else {
            continue;
        };
        if media_exports.contains(local_ref) {
            continue;
        }

        let source = source_map
            .source_for_diagnostic_path(scan.authoring_path)
            .map(SourcePath::new);
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new("MEDIA.MISSING_REFERENCE"),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message: missing_media_reference_summary(&candidate),
            help: product_diagnostic_help(
                "MEDIA.MISSING_REFERENCE",
                source.as_ref().map(SourcePath::as_str),
            ),
            source,
        });
    }
}

fn missing_media_reference_summary(
    candidate: &crate::authoring_core::MediaReferenceCandidate,
) -> String {
    if candidate.ref_kind == "css_url" {
        let raw_ref = candidate
            .diagnostic_ref
            .as_deref()
            .unwrap_or(candidate.raw_ref.as_str());
        format!(
            "missing media reference {} in {} {} {} {} line {}",
            raw_ref,
            candidate.owner_kind,
            candidate.owner_id,
            candidate.location_kind,
            candidate.location_name,
            candidate.source_line.unwrap_or(1)
        )
    } else {
        format!(
            "missing media reference {} in {} {} {} {}",
            candidate.raw_ref,
            candidate.owner_kind,
            candidate.owner_id,
            candidate.location_kind,
            candidate.location_name
        )
    }
}

struct ProjectNormalizeOutput {
    normalized_ir: crate::authoring_core::NormalizedIr,
    diagnostics: Vec<Diagnostic>,
    media_source_modes: BTreeMap<String, MediaSourceMode>,
}

fn replace_output_atomically(
    temp_artifact: &Path,
    target: &Path,
    force_failure_for_test: bool,
) -> std::io::Result<()> {
    let parent = target.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "output path has no parent",
        )
    })?;
    std::fs::create_dir_all(parent)?;
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("deck.apkg");
    let mut temp_target = tempfile::Builder::new()
        .prefix(&format!(".{file_name}."))
        .suffix(".tmp")
        .tempfile_in(parent)?;
    std::io::copy(
        &mut std::fs::File::open(temp_artifact)?,
        temp_target.as_file_mut(),
    )?;
    temp_target.as_file_mut().sync_all()?;
    if force_failure_for_test {
        return Err(std::io::Error::other("forced output replace failure"));
    }
    temp_target.persist(target).map_err(|err| err.error)?;
    Ok(())
}

#[cfg(test)]
mod atomic_output_tests {
    use std::path::Path;

    use tempfile::tempdir;

    use crate::{
        build::{BuildError, BuildOptions, BuildReport},
        product::{Note, Project},
    };

    fn build_with_forced_replace_failure(target: &Path) -> Result<BuildReport, BuildError> {
        let mut project = Project::new("Atomic Output")
            .stable_id("atomic-output")
            .default_deck("Atomic");
        project
            .add_note(Note::basic("front", "back").stable_id("atomic-note-1"))
            .expect("add note");

        project.build(
            BuildOptions::new()
                .output(target)
                .force_output_replace_failure_for_test(true),
        )
    }

    #[test]
    fn product_build_preserves_existing_target_when_replace_fails() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("deck.apkg");
        std::fs::write(&target, b"previous").expect("seed target");

        let err =
            build_with_forced_replace_failure(&target).expect_err("forced replace should fail");

        assert_eq!(std::fs::read(&target).expect("read target"), b"previous");
        assert!(err
            .report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "PROJECT.OUTPUT_WRITE_FAILED"));
    }
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
                let report =
                    failure_report(started, "PROJECT.ARTIFACTS_DIR_FAILED", err.to_string());
                match maybe_write_report_json(options, report) {
                    Ok(report) => BuildError::new(report, BuildFailureCause::Io),
                    Err(err) => err,
                }
            })?;
        let path = temp_dir.path().to_path_buf();

        Ok(Self {
            path,
            temp_dir: Some(temp_dir),
            persist_temp: options.output.is_none(),
        })
    }

    fn create_candidate_dir(&self) -> std::io::Result<TempDir> {
        // Keep private APKG generation on the caller-selected filesystem.
        // TempDir reserves a fresh directory and removes it on every exit path.
        std::fs::create_dir_all(&self.path)?;
        tempfile::Builder::new()
            .prefix("anki-forge-candidate-")
            .tempdir_in(&self.path)
    }

    fn publish(&self, candidate: &Path, options: &BuildOptions) -> std::io::Result<PathBuf> {
        let package_path = self.path.join("package.apkg");
        if self.temp_dir.is_none() || self.persist_temp {
            replace_output_atomically(candidate, &package_path, false)?;
        }
        if let Some(output) = options.output.as_ref() {
            replace_output_atomically(
                candidate,
                output,
                options.output_replace_failure_for_test(),
            )?;
            Ok(output.clone())
        } else {
            Ok(package_path)
        }
    }

    fn persist_if_requested(self) {
        if self.persist_temp {
            if let Some(temp_dir) = self.temp_dir {
                let persisted_artifacts_dir = temp_dir.path().to_path_buf();
                std::mem::forget(temp_dir);
                let _persisted_artifacts_dir = persisted_artifacts_dir;
            }
        }
    }
}

#[derive(Debug)]
struct ProjectNormalizeError {
    message: String,
    diagnostics: Vec<Diagnostic>,
    normalized_ir: Option<Box<crate::authoring_core::NormalizedIr>>,
    media_source_modes: BTreeMap<String, MediaSourceMode>,
}

#[derive(Debug)]
pub(crate) struct ProductMediaPrepareError {
    message: String,
    diagnostics: Vec<Diagnostic>,
}

impl ProductMediaPrepareError {
    pub(crate) fn code(&self) -> crate::diagnostics::ErrorCode {
        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .or_else(|| self.diagnostics.first())
            .map(|diagnostic| diagnostic.code.error_code())
            .unwrap_or_else(|| {
                crate::diagnostics::ErrorCode::from_code("PROJECT.PRODUCT_MEDIA_FAILED")
            })
    }

    fn from_source_diagnostic(
        diagnostic: crate::product::media_registry::ProductMediaSourceDiagnostic,
    ) -> Self {
        let code = diagnostic.code;
        let source_path = diagnostic.source_path;
        let help = product_diagnostic_help(code, Some(&source_path))
            .unwrap_or_else(|| "inspect product media registrations and source files".into());
        Self {
            message: "prepare product media".into(),
            diagnostics: vec![Diagnostic {
                code: DiagnosticCode::new(code),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: diagnostic.message,
                source: Some(SourcePath::new(source_path)),
                help: Some(help),
            }],
        }
    }

    fn from_prepare_error(error: anyhow::Error) -> Self {
        Self {
            message: "prepare product media".into(),
            diagnostics: vec![Diagnostic {
                code: DiagnosticCode::new("PROJECT.PRODUCT_MEDIA_FAILED"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: error.to_string(),
                source: Some(SourcePath::new("project.media")),
                help: Some("inspect product media registrations and media paths".into()),
            }],
        }
    }

    fn single(code: &'static str, message: String, export_filename: String) -> Self {
        let source_path = format!("project.media[{export_filename:?}]");
        let help = product_diagnostic_help(code, Some(&source_path))
            .unwrap_or_else(|| "inspect product media registrations and source files".into());
        Self {
            message: "prepare product media".into(),
            diagnostics: vec![Diagnostic {
                code: DiagnosticCode::new(code),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message,
                source: Some(SourcePath::new(source_path)),
                help: Some(help),
            }],
        }
    }

    fn staging_collision(source: PathBuf, target: PathBuf, export_filename: String) -> Self {
        Self::single(
            "PROJECT.PRODUCT_MEDIA_STAGING_COLLISION",
            format!(
                "Product media staging target already exists for export filename {export_filename:?}; source {} would overwrite target {}",
                source.display(),
                target.display()
            ),
            export_filename,
        )
    }
}

impl std::fmt::Display for ProductMediaPrepareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(diagnostic) = self
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .or_else(|| self.diagnostics.first())
        {
            write!(
                f,
                "{}: {}: {}",
                self.message,
                diagnostic.code.as_str(),
                diagnostic.message
            )
        } else {
            f.write_str(&self.message)
        }
    }
}

impl std::error::Error for ProductMediaPrepareError {}

impl crate::diagnostics::ErrorCodeExt for ProductMediaPrepareError {
    fn code(&self) -> crate::diagnostics::ErrorCode {
        ProductMediaPrepareError::code(self)
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
    item: crate::authoring_core::model::DiagnosticItem,
    source_map: &ProductSourceMap,
) -> Diagnostic {
    let source = item.path.as_deref().and_then(|path| {
        source_map
            .source_for_diagnostic_path(path)
            .map(SourcePath::new)
    });
    let code = item.code;
    let message = product_diagnostic_message(&code, item.summary);
    let help = product_diagnostic_help(&code, source.as_ref().map(SourcePath::as_str));
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity: severity_from_level(&item.level),
        domain: None,
        stage: None,
        message,
        source,
        help,
    }
}

fn product_diagnostic_message(code: &str, message: String) -> String {
    if code == "MEDIA.DECLARED_MIME_MISMATCH" {
        message.replace("sniffed MIME", "observed MIME")
    } else {
        message
    }
}

fn product_diagnostic_help(code: &str, source: Option<&str>) -> Option<String> {
    let help = match code {
        "MEDIA.MISSING_REFERENCE" => {
            if source.is_some_and(|source| source.ends_with(".css")) {
                "Product CSS scanning is conservative: a local filename in url(...) is treated as packaged media. Register it with project.media_mut().add_file(...).export_as(...), change the URL to an external URL, or remove the CSS rule/import if unused."
            } else {
                "This Product field or template references packaged media that is not registered. Register it with project.media_mut().add_file(...).export_as(...) or update the local filename in the Product content."
            }
        }
        "MEDIA.UNSAFE_REFERENCE" => {
            "This Product content uses a media reference that cannot be packaged safely. Use a bare local filename for packaged media, without paths, URL escapes, or unsafe characters."
        }
        "MEDIA.UNUSED_BINDING" => {
            "This Product media registration is not referenced by packaged content. You can remove the registration or reference it from a note, template, or CSS."
        }
        "MEDIA.DECLARED_MIME_MISMATCH" => {
            "This Product media registration declares a MIME type that does not match the observed source bytes. Change the export filename/declared MIME or replace the source file with matching content."
        }
        "MEDIA.DUPLICATE_FILENAME_CONFLICT" => {
            "This Product media export filename is already bound to different content. Choose a unique export_as(...) name for one of the registrations."
        }
        "MEDIA.SOURCE_CHANGED" => {
            "This Product media source changed after registration. Restore the original bytes, re-register the current file with project.media_mut().add_file(...).export_as(...), or remove the stale registration."
        }
        "MEDIA.SOURCE_MISSING" => {
            "This Product media registration points at a source file that no longer exists. Restore the file, update the registration to the new path, or remove the unused media binding."
        }
        "MEDIA.SOURCE_NOT_REGULAR_FILE" => {
            "This Product media registration must point at a regular file. Replace the source path with a file and register directories or special files through an explicit file asset."
        }
        "MEDIA.SOURCE_READ_FAILED" => {
            "This Product media source could not be read. Check the file path, permissions, and workspace access, then retry the build."
        }
        "MEDIA.UNKNOWN_MIME" => {
            "This media source has no reliable MIME type. Use an export filename with a known extension, provide bytes that can be sniffed, or adjust the advanced media policy if the opaque file is intentional."
        }
        "MEDIA.CAS_WRITE_FAILED" => {
            "The media object could not be written into the build media store. Check workspace permissions and disk space, then retry the build."
        }
        "MEDIA.CAS_OBJECT_INTEGRITY_CONFLICT" => {
            "An existing media-store object did not match the expected content hash. Rebuild in a clean artifacts directory or remove the corrupt media-store object before retrying."
        }
        "MEDIA.INLINE_TOO_LARGE" => {
            "This inline media payload exceeds the configured limit. Use the default path-backed build path with write_apkg()/build(...), or remove self_contained() so file-backed media is staged by path instead of embedded inline."
        }
        _ => return None,
    };
    Some(help.into())
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
            domain: None,
            stage: None,
            message: diagnostic.message,
            source: Some(SourcePath::new(
                diagnostic
                    .source_path
                    .unwrap_or_else(|| "project.lower".to_string()),
            )),
            help: diagnostic
                .byte_offset
                .map(|offset| format!("fix the template expression near byte offset {offset}")),
        })
        .collect()
}

fn map_lowering_diagnostics(diagnostics: Vec<LoweringDiagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| Diagnostic {
            code: DiagnosticCode::new(diagnostic.code),
            severity: lowering_diagnostic_severity(diagnostic.code),
            domain: None,
            stage: None,
            message: diagnostic.message,
            source: Some(SourcePath::new(
                diagnostic
                    .source_path
                    .unwrap_or_else(|| "project.lower".to_string()),
            )),
            help: diagnostic
                .byte_offset
                .map(|offset| format!("fix the template expression near byte offset {offset}")),
        })
        .collect()
}

fn lowering_diagnostic_severity(code: &str) -> Severity {
    match code {
        "PHASE5A.FONT_BINDING_UNKNOWN_NOTETYPE" | "PRODUCT.MEDIA_HELPER_REFERENCE_UNREGISTERED" => {
            Severity::Error
        }
        _ => Severity::Warning,
    }
}

fn failure_report(started: Instant, code: &str, message: String) -> BuildReport {
    BuildReport {
        artifact: None,
        counts: BuildCounts::default(),
        media: MediaSummary::default(),
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new(code),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message,
            source: Some(SourcePath::new("project.build")),
            help: None,
        }],
        metrics: BuildMetrics {
            duration: started.elapsed(),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Error,
    }
}

fn invalid_report_without_artifact(
    diagnostics: Vec<Diagnostic>,
    normalized: &crate::authoring_core::NormalizedIr,
    media_source_modes: &BTreeMap<String, MediaSourceMode>,
    started: Instant,
) -> BuildReport {
    BuildReport {
        artifact: None,
        counts: BuildCounts {
            notes: normalized.notes.len(),
            cards: count_phase1_cards_without_inspect(normalized),
            media: normalized.media_bindings.len(),
        },
        media: MediaSummary::from_normalized_ir_with_source_modes(
            normalized,
            &diagnostics,
            media_source_modes,
        ),
        diagnostics,
        metrics: BuildMetrics {
            duration: started.elapsed(),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Invalid,
    }
}

fn attach_artifact_diff_risk_if_needed(
    risk: &mut Option<crate::risk::ImportRiskReport>,
    diff: Option<&crate::diff::BuildDiffSummary>,
) {
    let Some(risk) = risk.as_mut() else {
        return;
    };
    if !risk.findings.is_empty() {
        return;
    }
    let Some(first_change) = diff
        .and_then(|diff| diff.artifact_diff.as_ref())
        .and_then(|artifact_diff| artifact_diff.changes.first())
    else {
        return;
    };

    risk.findings.push(crate::risk::ImportRiskFinding {
        code: "RISK.ARTIFACT_DIFF".into(),
        level: crate::build::RiskLevel::Low,
        category: "artifact".into(),
        message: "comparison detected artifact changes not covered by a more specific risk rule"
            .into(),
        source: Some(SourcePath::new(first_change.selector.clone())),
        evidence_refs: first_change.evidence_refs.clone(),
        suggested_action: Some("review the diff before importing".into()),
    });
    risk.highest_level = risk.findings.iter().map(|finding| finding.level).max();
}

fn maybe_write_report_json(
    options: &BuildOptions,
    mut report: BuildReport,
) -> Result<BuildReport, BuildError> {
    let Some(path) = options.report_json.as_ref() else {
        return Ok(report);
    };
    if let Err(diagnostic) = BuildPathPlan::new(options).validate_report() {
        // Never overwrite a protected file while reporting an earlier failure.
        // A newly discovered alias must not silently turn a successful build
        // into one with a missing report (e.g. a new case-variant lockfile).
        if report.status.is_success() {
            return return_path_validation_error(options, report, *diagnostic);
        }
        return Ok(report);
    }

    if let Err(err) = crate::build::write_report_json_atomic(path, &report) {
        report.diagnostics.push(Diagnostic {
            code: DiagnosticCode::new("PROJECT.REPORT_JSON_WRITE_FAILED"),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message: format!("failed to write report_json: {err}"),
            source: Some(SourcePath::new(path.display().to_string())),
            help: Some("choose a writable report_json path".to_string()),
        });
        report.status = BuildStatus::highest([report.status, BuildStatus::Error]);
        return Err(BuildError::new(report, BuildFailureCause::Io));
    }

    Ok(report)
}

fn return_path_validation_error(
    options: &BuildOptions,
    mut report: BuildReport,
    diagnostic: Diagnostic,
) -> Result<BuildReport, BuildError> {
    let cause = if diagnostic.code.as_str() == "PROJECT.PATH_COLLISION" {
        report.status = BuildStatus::Invalid;
        BuildFailureCause::Diagnostics
    } else {
        report.status = BuildStatus::Error;
        BuildFailureCause::Io
    };
    report.diagnostics.push(diagnostic);
    return_report_error(options, report, cause)
}

fn return_report_error(
    options: &BuildOptions,
    report: BuildReport,
    cause: BuildFailureCause,
) -> Result<BuildReport, BuildError> {
    match maybe_write_report_json(options, report) {
        Ok(report) => Err(BuildError::new(report, cause)),
        Err(err) => Err(err),
    }
}

fn build_status_from_writer_result(status: &str) -> BuildStatus {
    match status {
        "success" => BuildStatus::Success,
        "invalid" => BuildStatus::Invalid,
        "error" => BuildStatus::Error,
        _ => BuildStatus::Error,
    }
}

fn diagnostics_status(diagnostics: &[Diagnostic]) -> BuildStatus {
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        BuildStatus::Invalid
    } else {
        BuildStatus::Success
    }
}

fn policy_status(policy: &BuildPolicyResult) -> BuildStatus {
    if matches!(policy.status, crate::build::BuildPolicyStatus::Blocked) {
        BuildStatus::Blocked
    } else {
        BuildStatus::Success
    }
}

fn baseline_unavailable_risk(
    diagnostics: &[Diagnostic],
    update_safety: Option<&crate::build::UpdateSafetySummary>,
) -> crate::risk::ImportRiskReport {
    crate::risk::classify_import_risk(crate::risk::rules::RiskInput {
        diagnostics,
        comparison: ComparisonStatus::Unavailable,
        diff: None,
        current_inspect: None,
        previous_inspect: None,
        update_safety,
    })
}

fn severity_from_level(level: &str) -> Severity {
    match level {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    }
}

fn update_safety_blocking_severity(mode: crate::update_safety::EffectiveMode) -> Severity {
    match mode {
        crate::update_safety::EffectiveMode::Strict => Severity::Error,
        crate::update_safety::EffectiveMode::ReportOnly
        | crate::update_safety::EffectiveMode::Disabled => Severity::Warning,
    }
}

fn push_project_stable_id_mismatch_if_needed(
    diagnostics: &mut Vec<Diagnostic>,
    current_project_stable_id: Option<&str>,
    baseline_project_stable_id: Option<&str>,
    source_ref: impl Into<String>,
    severity: Severity,
) {
    let (Some(current), Some(baseline)) = (current_project_stable_id, baseline_project_stable_id)
    else {
        return;
    };
    if current == baseline {
        return;
    }
    diagnostics.push(Diagnostic {
        code: DiagnosticCode::new("UPDATE.PROJECT_STABLE_ID_MISMATCH"),
        severity,
        domain: None,
        stage: None,
        message: format!(
            "project stable id {current:?} differs from baseline project stable id {baseline:?}"
        ),
        source: Some(SourcePath::new(source_ref.into())),
        help: Some(
            "use the matching lockfile for this project or choose an explicit migration path"
                .into(),
        ),
    });
}

fn downgrade_update_errors_to_warnings(diagnostics: &mut [Diagnostic]) {
    for diagnostic in diagnostics {
        if diagnostic.code.as_str().starts_with("UPDATE.") && diagnostic.severity == Severity::Error
        {
            diagnostic.severity = Severity::Warning;
        }
    }
}

fn downgrade_compare_errors_to_warnings(diagnostics: &mut [Diagnostic]) {
    for diagnostic in diagnostics {
        if diagnostic.code.as_str().starts_with("COMPARE.")
            && diagnostic.severity == Severity::Error
        {
            diagnostic.severity = Severity::Warning;
        }
    }
}

fn deck_validation_diagnostic_to_project_diagnostic(
    diagnostic: &crate::deck::ValidationDiagnostic,
) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(deck_validation_code(&diagnostic.code)),
        severity: diagnostic.severity,
        domain: None,
        stage: None,
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

fn combine_lowering_and_normalization_diagnostics(
    mut lowering_diagnostics: Vec<Diagnostic>,
    normalization_diagnostics: Vec<Diagnostic>,
) -> Vec<Diagnostic> {
    lowering_diagnostics.extend(normalization_diagnostics);
    lowering_diagnostics
}

fn deduplicate_diagnostics(diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = BTreeSet::new();
    diagnostics.retain(|diagnostic| {
        seen.insert((
            diagnostic.code.as_str().to_string(),
            diagnostic.severity as u8,
            diagnostic
                .domain
                .as_ref()
                .map(|domain| domain.as_str().to_string()),
            diagnostic
                .stage
                .as_ref()
                .map(|stage| stage.as_str().to_string()),
            diagnostic
                .source
                .as_ref()
                .map(|source| source.as_str().to_string()),
            diagnostic.message.clone(),
        ))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::{Field, Template};

    fn diagnostic(code: &str, severity: Severity) -> Diagnostic {
        Diagnostic {
            code: DiagnosticCode::new(code),
            severity,
            domain: None,
            stage: None,
            message: code.into(),
            source: None,
            help: None,
        }
    }

    fn project_with_private_state(name: &str) -> Project {
        Project::new(name).stable_id(name).default_deck(name)
    }

    fn diagnostics_include_code(report: &ValidationReport, code: &str) -> bool {
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == code)
    }

    #[test]
    fn validate_still_reports_duplicate_stable_ids_for_internal_invalid_state() {
        let mut project = project_with_private_state("duplicate-stable-id");
        project
            .notes
            .push(Note::basic("front 1", "back 1").stable_id("same"));
        project
            .notes
            .push(Note::basic("front 2", "back 2").stable_id("same"));

        let report = project.validate();

        assert!(diagnostics_include_code(
            &report,
            "AFID.STABLE_ID_DUPLICATE"
        ));
    }

    #[test]
    fn lower_keeps_later_note_source_after_required_note_is_skipped() {
        let note_type = NoteType::custom("source-card")
            .field(Field::new("Prompt").key("prompt").required())
            .field(Field::new("Extra").key("extra").optional())
            .template(
                Template::new("Card")
                    .key("card")
                    .front("{{Prompt}}")
                    .back("{{Extra}}"),
            );
        let mut project = Project::new("Shifted Notes").stable_id("shifted-notes");
        project.add_notetype(note_type).expect("add note type");
        project
            .add_note(
                Note::new("source-card")
                    .stable_id("missing-required")
                    .text("extra", "first"),
            )
            .expect("add invalid note for lowering diagnostic");
        project
            .add_note(
                Note::new("source-card")
                    .stable_id("later-note")
                    .text("prompt", "valid")
                    .html("extra", "<img src=missing.png>"),
            )
            .expect("add later note");

        let plan = project.lower().expect("lower project");

        assert!(plan
            .product_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PRODUCT.REQUIRED_FIELD_MISSING"));
        assert_eq!(
            plan.source_map.source_for_authoring_path(
                &crate::product::lowering::authoring_note_field_path("later-note", "Extra")
            ),
            Some("project.notes[\"later-note\"].fields[\"extra\"]")
        );

        let temp = tempfile::tempdir().expect("tempdir");
        let error = match project.normalize_with_dirs(
            temp.path(),
            temp.path().join("media-store"),
            ProjectNormalizeOptions::default(),
        ) {
            Ok(_) => panic!("missing media and required field should fail normalization"),
            Err(error) => error,
        };
        assert_eq!(
            error
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
                .and_then(|diagnostic| diagnostic.source.as_ref())
                .map(SourcePath::as_str),
            Some("project.notes[\"later-note\"].fields[\"extra\"]"),
            "{:?}",
            error.diagnostics
        );
    }

    #[test]
    fn lower_keeps_index_source_when_colliding_identity_note_is_skipped() {
        let note_type = NoteType::custom("colliding-source-card")
            .field(Field::new("Identity").key("identity").optional())
            .field(Field::new("Prompt").key("prompt").required())
            .field(Field::new("Extra").key("extra").optional())
            .template(
                Template::new("Card")
                    .key("card")
                    .front("{{Prompt}}")
                    .back("{{Extra}}"),
            )
            .identity(crate::product::IdentityRecipe::fields(["identity"]));
        let mut project =
            Project::new("Colliding Shifted Notes").stable_id("colliding-shifted-notes");
        project.add_notetype(note_type).expect("add note type");
        project
            .add_note(
                Note::new("colliding-source-card")
                    .text("identity", "shared")
                    .text("extra", "first"),
            )
            .expect("add invalid note for lowering diagnostic");
        project
            .add_note(
                Note::new("colliding-source-card")
                    .text("identity", "shared")
                    .text("prompt", "valid")
                    .html("extra", "<img src=missing.png>"),
            )
            .expect("add later note");

        let plan = project.lower().expect("lower project");
        let authoring_note = plan
            .authoring_document
            .notes
            .first()
            .expect("later note should survive lowering");

        assert!(plan
            .product_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PRODUCT.REQUIRED_FIELD_MISSING"));
        assert_eq!(
            plan.source_map.source_for_authoring_path(
                &crate::product::lowering::authoring_note_field_path(&authoring_note.id, "Extra",)
            ),
            Some("project.notes[1].fields[\"extra\"]")
        );

        let temp = tempfile::tempdir().expect("tempdir");
        let error = match project.normalize_with_dirs(
            temp.path(),
            temp.path().join("media-store"),
            ProjectNormalizeOptions::default(),
        ) {
            Ok(_) => panic!("missing media and required field should fail normalization"),
            Err(error) => error,
        };
        assert_eq!(
            error
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
                .and_then(|diagnostic| diagnostic.source.as_ref())
                .map(SourcePath::as_str),
            Some("project.notes[1].fields[\"extra\"]"),
            "{:?}",
            error.diagnostics
        );
    }

    #[test]
    fn validate_still_reports_blank_stable_id_for_internal_invalid_state() {
        let mut project = project_with_private_state("blank-stable-id");
        project
            .notes
            .push(Note::basic("front", "back").stable_id("   "));

        let report = project.validate();

        assert!(diagnostics_include_code(&report, "AFID.STABLE_ID_BLANK"));
    }

    #[test]
    fn validate_still_reports_implicit_stock_collision_for_internal_invalid_state() {
        let mut project = project_with_private_state("implicit-stock-collision");
        project
            .notes
            .push(Note::basic("front", "back").stable_id("basic:1"));
        project.note_types.push(
            NoteType::custom(STOCK_BASIC_ID)
                .field(Field::new("Prompt").key("prompt"))
                .template(Template::new("Card").front("{{Prompt}}").back("{{Prompt}}")),
        );

        let report = project.validate();

        assert!(diagnostics_include_code(&report, "NOTETYPE.ID_DUPLICATE"));
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
    fn duplicate_diagnostics_with_the_same_source_are_collapsed() {
        let mut first = diagnostic("TEMPLATE.FILTER_UNKNOWN", Severity::Warning);
        first.help = Some("replace the unsupported filter".into());
        let mut second = diagnostic("TEMPLATE.FILTER_UNKNOWN", Severity::Warning);
        second.help = Some("fix the template expression".into());
        let mut diagnostics = vec![first, second];

        deduplicate_diagnostics(&mut diagnostics);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn product_helper_wiring_lowering_diagnostics_map_to_errors() {
        let diagnostics = map_lowering_diagnostics(vec![
            LoweringDiagnostic {
                code: "PHASE5A.FONT_BINDING_UNKNOWN_NOTETYPE",
                message: "missing notetype".into(),
                source_path: None,
                byte_offset: None,
            },
            LoweringDiagnostic {
                code: "PRODUCT.MEDIA_HELPER_REFERENCE_UNREGISTERED",
                message: "missing asset".into(),
                source_path: None,
                byte_offset: None,
            },
            LoweringDiagnostic {
                code: "PHASE5A.ADVISORY",
                message: "advisory".into(),
                source_path: None,
                byte_offset: None,
            },
        ]);

        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.severity))
                .collect::<Vec<_>>(),
            vec![
                ("PHASE5A.FONT_BINDING_UNKNOWN_NOTETYPE", Severity::Error),
                (
                    "PRODUCT.MEDIA_HELPER_REFERENCE_UNREGISTERED",
                    Severity::Error
                ),
                ("PHASE5A.ADVISORY", Severity::Warning),
            ]
        );
    }

    #[test]
    fn product_v2_normalize_preserves_unknown_media_source_path() {
        let document = serde_json::from_str::<crate::product::ProductDocument>(
            r#"{
              "product_document_version": "product-v2",
              "document_id": "invalid-media-source",
              "default_deck_name": "Invalid",
              "note_types": [],
              "notes": [],
              "media": [{
                "id": "media:future",
                "source": {"kind": "future_source", "uri": "asset://future"},
                "export_as": "future.bin",
                "source_path": "project.media[\"future.bin\"]"
              }]
            }"#,
        )
        .expect("parse product-v2 document");
        let temp = tempfile::tempdir().expect("tempdir");
        let err = match Project::from_product_document(document).normalize_with_dirs(
            temp.path(),
            temp.path().join(".anki-forge-media"),
            ProjectNormalizeOptions::default(),
        ) {
            Ok(_) => panic!("product diagnostics should fail normalization"),
            Err(error) => error,
        };

        assert_eq!(
            err.diagnostics
                .iter()
                .find(|diagnostic| {
                    diagnostic.code.as_str() == "PRODUCT.MEDIA_SOURCE_KIND_UNSUPPORTED"
                })
                .and_then(|diagnostic| diagnostic.source.as_ref())
                .map(SourcePath::as_str),
            Some("project.media[\"future.bin\"]")
        );
    }

    #[test]
    fn product_v2_normalize_preserves_required_field_source_path() {
        let document = serde_json::from_str::<crate::product::ProductDocument>(
            r#"{
              "product_document_version": "product-v2",
              "document_id": "invalid-basic-required",
              "default_deck_name": "Invalid",
              "note_types": [{
                "kind": "stock",
                "id": "basic",
                "name": "Basic",
                "fields": [
                  {"name": "Front", "key": "front", "required": true},
                  {"name": "Back", "key": "back", "required": false}
                ],
                "templates": [],
                "css": null
              }],
              "notes": [{
                "kind": "stock",
                "note_type_id": "basic",
                "deck_name": "Invalid",
                "fields": {"back": {"kind": "text", "value": "Back only"}},
                "source_path": "project.notes[0]"
              }],
              "media": []
            }"#,
        )
        .expect("parse product-v2 document");
        let temp = tempfile::tempdir().expect("tempdir");
        let err = match Project::from_product_document(document).normalize_with_dirs(
            temp.path(),
            temp.path().join(".anki-forge-media"),
            ProjectNormalizeOptions::default(),
        ) {
            Ok(_) => panic!("product diagnostics should fail normalization"),
            Err(error) => error,
        };

        assert_eq!(
            err.diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code.as_str() == "PRODUCT.REQUIRED_FIELD_MISSING")
                .and_then(|diagnostic| diagnostic.source.as_ref())
                .map(SourcePath::as_str),
            Some("project.notes[0]")
        );
    }

    #[test]
    fn lower_maps_duplicate_custom_notetype_sources_to_project_indices_when_stock_is_implicit() {
        let mut project = Project::new("Shifted Note Types")
            .stable_id("shifted-notetypes")
            .default_deck("Shifted Note Types");
        project
            .notes
            .push(Note::basic("front", "back").stable_id("basic:1"));
        project.note_types.push(
            NoteType::custom("dup")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Recognition")
                        .key("recognition")
                        .front("{{Prompt}}")
                        .back("{{Prompt}}")
                        .browser_back("{{Prompt}}"),
                ),
        );
        project.note_types.push(
            NoteType::custom("dup")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Recall")
                        .key("recall")
                        .front("{{Prompt}}")
                        .back("{{Prompt}}")
                        .browser_front("{{Prompt}}"),
                ),
        );

        let plan = project.lower().expect("lower project");

        assert_eq!(
            plan.source_map.source_for_authoring_path(
                "authoring.note_types[1].templates[\"Recognition\"].front"
            ),
            Some("project.note_types[0].templates[\"Recognition\"].front")
        );
        assert_eq!(
            plan.source_map.source_for_authoring_path(
                "authoring.note_types[1].templates[\"Recognition\"].browser_back"
            ),
            Some("project.note_types[0].templates[\"Recognition\"].browser_back")
        );
        assert_eq!(
            plan.source_map
                .source_for_authoring_path("authoring.note_types[1].css"),
            Some("project.note_types[0].css")
        );
        assert_eq!(
            plan.source_map
                .source_for_authoring_path("authoring.note_types[2].templates[\"Recall\"].back"),
            Some("project.note_types[1].templates[\"Recall\"].back")
        );
        assert_eq!(
            plan.source_map.source_for_authoring_path(
                "authoring.note_types[2].templates[\"Recall\"].browser_front"
            ),
            Some("project.note_types[1].templates[\"Recall\"].browser_front")
        );
        assert_eq!(
            plan.source_map
                .source_for_authoring_path("authoring.note_types[2].css"),
            Some("project.note_types[1].css")
        );
    }

    #[test]
    fn lower_rejects_custom_notetype_stock_collision_before_authoring() {
        let mut project = Project::new("Implicit Duplicate")
            .stable_id("implicit-duplicate")
            .default_deck("Implicit Duplicate");
        project
            .notes
            .push(Note::basic("front", "back").stable_id("basic:1"));
        project.note_types.push(
            NoteType::custom("basic")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Custom Basic")
                        .front("{{Prompt}}")
                        .back("{{Prompt}}"),
                ),
        );

        let plan = project.lower().expect("lower project");

        let diagnostic = plan
            .product_diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "PRODUCT.RESERVED_ID_KIND_MISMATCH")
            .expect("reserved custom id diagnostic");
        assert_eq!(
            diagnostic.source_path.as_deref(),
            Some("project.note_types[0]")
        );
        assert_eq!(plan.authoring_document.notetypes.len(), 1);
    }

    #[test]
    fn build_rejects_stock_collision_before_scanning_custom_template_media() {
        let mut project = Project::new("Implicit Duplicate Media")
            .stable_id("implicit-duplicate-media")
            .default_deck("Implicit Duplicate Media");
        project
            .notes
            .push(Note::basic("front", "back").stable_id("basic:1"));
        project.note_types.push(
            NoteType::custom("basic")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Custom Basic")
                        .front(r#"<img src="missing-custom-template.png"> {{Prompt}}"#)
                        .back("{{Prompt}}"),
                )
                .css(r#".card { background: url("missing-custom-css.png"); }"#),
        );

        let error = project
            .build(BuildOptions::new().inspect(false))
            .expect_err("implicit stock collision and missing custom media fail build");

        let duplicate = error
            .report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.ID_DUPLICATE")
            .expect("duplicate notetype diagnostic");
        assert_eq!(
            duplicate.source.as_ref().map(|source| source.as_str()),
            Some("project.note_types[0]")
        );
        assert!(duplicate.message.contains("implicit stock"));

        let reserved = error
            .report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code.as_str() == "PRODUCT.RESERVED_ID_KIND_MISMATCH")
            .expect("reserved custom id diagnostic");
        assert_eq!(
            reserved.source.as_ref().map(|source| source.as_str()),
            Some("project.note_types[0]")
        );
        assert!(!error
            .report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE"));
    }

    #[test]
    fn build_maps_missing_media_reference_to_index_source_for_blank_and_duplicate_stable_ids() {
        let mut project = project_with_private_state("invalid-note-ids-media");
        project.notes.push(
            Note::new(STOCK_BASIC_ID)
                .stable_id("   ")
                .text("Front", "blank stable id")
                .html("Back", r#"<img src="blank.png">"#),
        );
        project.notes.push(
            Note::new(STOCK_BASIC_ID)
                .stable_id("dup")
                .text("Front", "duplicate stable id 1")
                .html("Back", r#"<img src="dup-one.png">"#),
        );
        project.notes.push(
            Note::new(STOCK_BASIC_ID)
                .stable_id("dup")
                .text("Front", "duplicate stable id 2")
                .html("Back", r#"<img src="dup-two.png">"#),
        );

        let error = project
            .build(BuildOptions::new().inspect(false))
            .expect_err("invalid stable ids and missing media references fail build");

        assert!(error
            .report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "AFID.STABLE_ID_BLANK"));
        assert!(error
            .report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "AFID.STABLE_ID_DUPLICATE"));

        let sources = error
            .report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
            .map(|diagnostic| {
                diagnostic
                    .source
                    .as_ref()
                    .map(|source| source.as_str())
                    .expect("missing media diagnostic source")
            })
            .collect::<Vec<_>>();

        assert!(
            sources.contains(&"project.notes[0].fields[\"Back\"]"),
            "{sources:?}"
        );
        assert!(sources.contains(&"project.notes[1].fields[\"Back\"]"));
        assert!(sources.contains(&"project.notes[2].fields[\"Back\"]"));
    }

    #[test]
    fn build_reports_duplicate_notetype_template_and_css_media_sources_by_index() {
        let mut project = Project::new("Duplicate Note Type Media")
            .stable_id("duplicate-notetype-media")
            .default_deck("Duplicate Note Type Media");
        project.note_types.push(
            NoteType::custom("dup")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Recognition")
                        .front(r#"<img src="missing-first-template.png"> {{Prompt}}"#)
                        .back("{{Prompt}}"),
                )
                .css(r#".card { background: url("missing-first-css.png"); }"#),
        );
        project.note_types.push(
            NoteType::custom("dup")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Recall")
                        .front(r#"<img src="missing-second-template.png"> {{Prompt}}"#)
                        .back("{{Prompt}}"),
                )
                .css(r#".card { background: url("missing-second-css.png"); }"#),
        );

        let error = project
            .build(BuildOptions::new().inspect(false))
            .expect_err("duplicate notetype id and missing media references fail build");

        assert!(error
            .report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.ID_DUPLICATE"));

        let sources = error
            .report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
            .map(|diagnostic| {
                assert_eq!(diagnostic.severity, Severity::Error);
                diagnostic
                    .source
                    .as_ref()
                    .map(|source| source.as_str())
                    .expect("missing media diagnostic source")
            })
            .collect::<Vec<_>>();

        assert!(sources.contains(&"project.note_types[0].templates[\"Recognition\"].front"));
        assert!(sources.contains(&"project.note_types[0].css"));
        assert!(sources.contains(&"project.note_types[1].templates[\"Recall\"].front"));
        assert!(sources.contains(&"project.note_types[1].css"));
    }

    #[test]
    fn writer_result_status_maps_to_typed_build_status() {
        assert_eq!(
            build_status_from_writer_result("success"),
            BuildStatus::Success
        );
        assert_eq!(
            build_status_from_writer_result("invalid"),
            BuildStatus::Invalid
        );
        assert_eq!(build_status_from_writer_result("error"), BuildStatus::Error);
        assert_eq!(
            build_status_from_writer_result("unexpected"),
            BuildStatus::Error
        );
    }

    #[test]
    fn product_diagnostic_help_covers_actionable_media_storage_codes() {
        for code in [
            "MEDIA.SOURCE_CHANGED",
            "MEDIA.SOURCE_MISSING",
            "MEDIA.SOURCE_NOT_REGULAR_FILE",
            "MEDIA.SOURCE_READ_FAILED",
            "MEDIA.UNKNOWN_MIME",
            "MEDIA.CAS_WRITE_FAILED",
            "MEDIA.CAS_OBJECT_INTEGRITY_CONFLICT",
            "MEDIA.INLINE_TOO_LARGE",
        ] {
            assert!(
                product_diagnostic_help(code, Some("project.media[\"asset.bin\"]")).is_some(),
                "{code} should have Product-facing help"
            );
        }
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

    #[test]
    fn private_candidates_follow_the_artifact_workspace_and_are_cleaned_up() {
        for explicit_artifacts in [true, false] {
            let root = tempfile::tempdir().unwrap();
            let mut options = BuildOptions::new().output(root.path().join("output.apkg"));
            if explicit_artifacts {
                options = options.artifacts_dir(root.path().join("new/artifacts"));
            }
            let workspace = ArtifactWorkspace::new(&options, Instant::now()).unwrap();
            let candidate_path = {
                let candidate = workspace.create_candidate_dir().unwrap();
                assert_eq!(candidate.path().parent(), Some(workspace.path.as_path()));
                assert!(!candidate.path().join("package.apkg").exists());
                std::fs::write(candidate.path().join("package.apkg"), b"candidate bytes").unwrap();
                candidate.path().to_path_buf()
            };
            assert!(!candidate_path.exists());
            assert!(!workspace.path.join("package.apkg").exists());
        }
    }

    #[test]
    fn output_only_publication_does_not_copy_package_into_disposable_workspace() {
        let root = tempfile::tempdir().unwrap();
        let candidate = root.path().join("candidate.apkg");
        std::fs::write(&candidate, b"candidate bytes").unwrap();
        let output = root.path().join("output.apkg");
        let options = BuildOptions::new().output(&output);
        let workspace = ArtifactWorkspace::new(&options, Instant::now()).unwrap();

        let published = workspace.publish(&candidate, &options).unwrap();

        assert_eq!(published, output);
        assert_eq!(std::fs::read(published).unwrap(), b"candidate bytes");
        assert!(
            !workspace.path.join("package.apkg").exists(),
            "output-only builds must not allocate a second temporary APKG"
        );
    }

    #[test]
    fn publication_keeps_packages_when_the_workspace_is_retained() {
        for (explicit_artifacts, explicit_output) in [(true, true), (true, false), (false, false)] {
            let root = tempfile::tempdir().unwrap();
            let candidate = root.path().join("candidate.apkg");
            std::fs::write(&candidate, b"candidate bytes").unwrap();
            let output = root.path().join("output.apkg");
            let mut options = BuildOptions::new();
            if explicit_artifacts {
                options = options.artifacts_dir(root.path().join("artifacts"));
            }
            if explicit_output {
                options = options.output(&output);
            }
            let workspace = ArtifactWorkspace::new(&options, Instant::now()).unwrap();

            let published = workspace.publish(&candidate, &options).unwrap();

            assert_eq!(std::fs::read(&published).unwrap(), b"candidate bytes");
            let package = workspace.path.join("package.apkg");
            assert_eq!(std::fs::read(&package).unwrap(), b"candidate bytes");
            assert_eq!(published, if explicit_output { output } else { package });
        }
    }
}
