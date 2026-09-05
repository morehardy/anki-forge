use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use super::{Field, GenerationRule, IdentityRecipe, NoteType, Template};

const MANIFEST_NAME: &str = "anki-template.yaml";
const MANIFEST_LIMIT: u64 = 256 * 1024;
const TEXT_FILE_LIMIT: u64 = 2 * 1024 * 1024;
const ASSET_FILE_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateBundleError {
    code: String,
    message: String,
    path: Option<PathBuf>,
    byte_offset: Option<usize>,
}

impl TemplateBundleError {
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn byte_offset(&self) -> Option<usize> {
        self.byte_offset
    }

    pub(crate) fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        path: Option<PathBuf>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path,
            byte_offset: None,
        }
    }

    pub(crate) fn with_byte_offset(mut self, byte_offset: Option<usize>) -> Self {
        self.byte_offset = byte_offset;
        self
    }
}

impl std::fmt::Display for TemplateBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for TemplateBundleError {}

#[derive(Debug)]
pub(crate) struct LoadedTemplateBundle {
    pub(crate) note_type: NoteType,
    pub(crate) assets: Vec<LoadedTemplateAsset>,
    relative_paths: Vec<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct LoadedTemplateAsset {
    pub(crate) path: PathBuf,
    pub(crate) export_as: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateBundleManifest {
    format_version: String,
    note_type: BundleNoteType,
    #[serde(default)]
    css_file: Option<String>,
    #[serde(default)]
    assets: Vec<BundleAsset>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleNoteType {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default = "normal_kind")]
    kind: String,
    #[serde(default)]
    cloze_field: Option<String>,
    fields: Vec<BundleField>,
    templates: Vec<BundleTemplate>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleField {
    key: String,
    name: String,
    #[serde(default)]
    identity: bool,
    #[serde(default)]
    sort: bool,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    optional: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleTemplate {
    key: String,
    name: String,
    front_file: String,
    back_file: String,
    #[serde(default)]
    browser_front_file: Option<String>,
    #[serde(default)]
    browser_back_file: Option<String>,
    #[serde(default)]
    target_deck: Option<String>,
    #[serde(default)]
    generation_rule: Option<BundleGenerationRule>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum BundleGenerationRule {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleAsset {
    path: String,
    export_as: String,
}

fn normal_kind() -> String {
    "normal".into()
}

/// Returns the sorted, deduplicated input paths of a loadable template bundle.
///
/// Paths stay relative to the bundle root, including aliases used by the manifest.
/// Resolution uses the same containment, file-size, and UTF-8 checks as import.
/// This interface is only for repository contract packaging.
#[cfg(feature = "internal-tools")]
pub fn template_bundle_relative_paths(
    root: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, TemplateBundleError> {
    Ok(load_template_bundle(root)?.relative_paths)
}

pub(crate) fn load_template_bundle(
    root: impl AsRef<Path>,
) -> Result<LoadedTemplateBundle, TemplateBundleError> {
    let root = root.as_ref();
    let canonical_root = root.canonicalize().map_err(|error| {
        TemplateBundleError::new(
            "TEMPLATE.BUNDLE_ROOT_INVALID",
            format!("cannot open template bundle root: {error}"),
            Some(root.to_path_buf()),
        )
    })?;
    if !canonical_root.is_dir() {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_ROOT_INVALID",
            "template bundle root must be a directory",
            Some(canonical_root),
        ));
    }

    let mut relative_paths = BTreeSet::new();
    let mut resolve_file = |relative: &str, size_limit, role: &str| {
        let path = resolve_bundle_file(&canonical_root, relative, size_limit, role)?;
        relative_paths.insert(PathBuf::from(relative));
        Ok::<_, TemplateBundleError>(path)
    };
    let manifest_path = resolve_file(MANIFEST_NAME, MANIFEST_LIMIT, "manifest")?;
    let manifest_source = read_utf8(&manifest_path, "TEMPLATE.BUNDLE_MANIFEST_INVALID")?;
    let manifest: TemplateBundleManifest =
        serde_yaml::from_str(&manifest_source).map_err(|error| {
            TemplateBundleError::new(
                "TEMPLATE.BUNDLE_MANIFEST_INVALID",
                format!("invalid template bundle manifest: {error}"),
                Some(manifest_path.clone()),
            )
        })?;
    if manifest.format_version != "template-bundle-v1" {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_VERSION_UNSUPPORTED",
            format!(
                "unsupported template bundle format_version '{}'",
                manifest.format_version
            ),
            Some(manifest_path),
        ));
    }
    if manifest.note_type.fields.is_empty() {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_MANIFEST_INVALID",
            "template bundle note_type.fields must contain at least one field",
            Some(canonical_root.join(MANIFEST_NAME)),
        ));
    }
    if manifest.note_type.templates.is_empty() {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_MANIFEST_INVALID",
            "template bundle note_type.templates must contain at least one template",
            Some(canonical_root.join(MANIFEST_NAME)),
        ));
    }
    validate_manifest_semantics(&manifest, &canonical_root.join(MANIFEST_NAME))?;
    if manifest
        .note_type
        .fields
        .iter()
        .any(|field| field.required && field.optional)
    {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_FIELD_MODE_CONFLICT",
            "template bundle fields cannot be both required and optional",
            Some(canonical_root.join(MANIFEST_NAME)),
        ));
    }
    if manifest.note_type.kind == "normal" && manifest.note_type.cloze_field.is_some() {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_MANIFEST_INVALID",
            "normal template bundles must not declare note_type.cloze_field",
            Some(canonical_root.join(MANIFEST_NAME)),
        ));
    }

    let is_cloze = manifest.note_type.kind == "cloze";
    let mut note_type = match manifest.note_type.kind.as_str() {
        "normal" => NoteType::custom(&manifest.note_type.id),
        "cloze" => {
            let field = manifest
                .note_type
                .cloze_field
                .as_deref()
                .filter(|field| !field.trim().is_empty())
                .ok_or_else(|| {
                    TemplateBundleError::new(
                        "TEMPLATE.CLOZE_FIELD_REQUIRED",
                        "Cloze template bundles require note_type.cloze_field",
                        Some(canonical_root.join(MANIFEST_NAME)),
                    )
                })?;
            NoteType::custom_cloze(&manifest.note_type.id, field)
        }
        kind => {
            return Err(TemplateBundleError::new(
                "TEMPLATE.BUNDLE_KIND_INVALID",
                format!("unsupported note_type.kind '{kind}'"),
                Some(canonical_root.join(MANIFEST_NAME)),
            ));
        }
    };
    if let Some(name) = manifest.note_type.name {
        note_type = note_type.name(name);
    }

    let identity_fields = manifest
        .note_type
        .fields
        .iter()
        .filter(|field| field.identity)
        .map(|field| field.key.clone())
        .collect::<Vec<_>>();
    for field in manifest.note_type.fields {
        let mut product_field = Field::new(field.name).key(field.key);
        if field.identity {
            product_field = product_field.identity();
        }
        if field.sort {
            product_field = product_field.sort();
        }
        if field.required {
            product_field = product_field.required();
        }
        if field.optional {
            product_field = product_field.optional();
        }
        note_type = note_type.field(product_field);
    }
    if !identity_fields.is_empty() {
        note_type = note_type.identity(IdentityRecipe::fields(identity_fields));
    }

    for template in manifest.note_type.templates {
        if is_cloze && template.generation_rule.is_some() {
            return Err(TemplateBundleError::new(
                "TEMPLATE.BUNDLE_GENERATION_RULE_CONFLICT",
                "Cloze template bundles derive generation from note_type.cloze_field and must not declare generation_rule",
                Some(canonical_root.join(MANIFEST_NAME)),
            ));
        }
        let front_path = resolve_file(&template.front_file, TEXT_FILE_LIMIT, "front template")?;
        let back_path = resolve_file(&template.back_file, TEXT_FILE_LIMIT, "back template")?;
        let mut product_template = Template::new(template.name)
            .key(template.key)
            .front_with_origin(
                read_utf8(&front_path, "TEMPLATE.BUNDLE_FILE_INVALID")?,
                front_path.to_string_lossy(),
            )
            .back_with_origin(
                read_utf8(&back_path, "TEMPLATE.BUNDLE_FILE_INVALID")?,
                back_path.to_string_lossy(),
            );
        if let Some(path) = template.browser_front_file {
            let path = resolve_file(&path, TEXT_FILE_LIMIT, "browser front")?;
            product_template = product_template.browser_front_with_origin(
                read_utf8(&path, "TEMPLATE.BUNDLE_FILE_INVALID")?,
                path.to_string_lossy(),
            );
        }
        if let Some(path) = template.browser_back_file {
            let path = resolve_file(&path, TEXT_FILE_LIMIT, "browser back")?;
            product_template = product_template.browser_back_with_origin(
                read_utf8(&path, "TEMPLATE.BUNDLE_FILE_INVALID")?,
                path.to_string_lossy(),
            );
        }
        if let Some(deck) = template.target_deck {
            product_template = product_template.target_deck(deck);
        }
        if let Some(rule) = template.generation_rule {
            product_template = product_template.generate_when(bundle_generation_rule(
                rule,
                &canonical_root.join(MANIFEST_NAME),
            )?);
        }
        note_type = note_type.template(product_template);
    }

    if let Some(css_file) = manifest.css_file {
        let css_path = resolve_file(&css_file, TEXT_FILE_LIMIT, "stylesheet")?;
        note_type = note_type.css(read_utf8(&css_path, "TEMPLATE.BUNDLE_FILE_INVALID")?);
    }

    let assets = manifest
        .assets
        .into_iter()
        .map(|asset| {
            Ok(LoadedTemplateAsset {
                path: resolve_file(&asset.path, ASSET_FILE_LIMIT, "asset")?,
                export_as: asset.export_as,
            })
        })
        .collect::<Result<Vec<_>, TemplateBundleError>>()?;

    Ok(LoadedTemplateBundle {
        note_type,
        assets,
        relative_paths: relative_paths.into_iter().collect(),
    })
}

fn validate_manifest_semantics(
    manifest: &TemplateBundleManifest,
    manifest_path: &Path,
) -> Result<(), TemplateBundleError> {
    let invalid_identifier = manifest.note_type.id.trim().is_empty()
        || manifest
            .note_type
            .name
            .as_deref()
            .is_some_and(|name| name.trim().is_empty())
        || manifest
            .note_type
            .fields
            .iter()
            .any(|field| field.key.trim().is_empty() || field.name.trim().is_empty())
        || manifest.note_type.templates.iter().any(|template| {
            template.key.trim().is_empty()
                || template.name.trim().is_empty()
                || template
                    .target_deck
                    .as_deref()
                    .is_some_and(|deck| deck.trim().is_empty())
        });
    if invalid_identifier {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_MANIFEST_INVALID",
            "note type, field, template, and target deck identifiers must be non-blank",
            Some(manifest_path.to_path_buf()),
        ));
    }

    let field_keys = manifest
        .note_type
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<BTreeSet<_>>();
    let field_names = manifest
        .note_type
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    let template_keys = manifest
        .note_type
        .templates
        .iter()
        .map(|template| template.key.as_str())
        .collect::<BTreeSet<_>>();
    let template_names = manifest
        .note_type
        .templates
        .iter()
        .map(|template| template.name.as_str())
        .collect::<BTreeSet<_>>();
    if field_keys.len() != manifest.note_type.fields.len()
        || field_names.len() != manifest.note_type.fields.len()
        || template_keys.len() != manifest.note_type.templates.len()
        || template_names.len() != manifest.note_type.templates.len()
        || manifest
            .note_type
            .fields
            .iter()
            .filter(|field| field.sort)
            .count()
            > 1
    {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_MANIFEST_INVALID",
            "field keys/names and template keys/names must be unique, with at most one sort field",
            Some(manifest_path.to_path_buf()),
        ));
    }

    if manifest.note_type.templates.iter().any(|template| {
        let fields = match template.generation_rule.as_ref() {
            Some(BundleGenerationRule::All { fields })
            | Some(BundleGenerationRule::Any { fields }) => fields,
            _ => return false,
        };
        fields
            .iter()
            .any(|field| !field_keys.contains(field.as_str()))
    }) {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_MANIFEST_INVALID",
            "template generation_rule references an unknown field key",
            Some(manifest_path.to_path_buf()),
        ));
    }
    Ok(())
}

fn bundle_generation_rule(
    rule: BundleGenerationRule,
    manifest_path: &Path,
) -> Result<GenerationRule, TemplateBundleError> {
    match rule {
        BundleGenerationRule::AnkiDefault => Ok(GenerationRule::AnkiDefault),
        BundleGenerationRule::All { fields } | BundleGenerationRule::Any { fields }
            if fields.is_empty()
                || fields.iter().any(|field| field.trim().is_empty())
                || fields.iter().collect::<BTreeSet<_>>().len() != fields.len() =>
        {
            Err(TemplateBundleError::new(
                "TEMPLATE.BUNDLE_MANIFEST_INVALID",
                "template generation_rule fields must be non-empty, unique field keys",
                Some(manifest_path.to_path_buf()),
            ))
        }
        BundleGenerationRule::All { fields } => Ok(GenerationRule::all(fields)),
        BundleGenerationRule::Any { fields } => Ok(GenerationRule::any(fields)),
    }
}

fn resolve_bundle_file(
    root: &Path,
    relative: &str,
    size_limit: u64,
    role: &str,
) -> Result<PathBuf, TemplateBundleError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_PATH_UNSAFE",
            format!("{role} path must stay within the template bundle"),
            Some(relative_path.to_path_buf()),
        ));
    }
    let candidate = root.join(relative_path);
    let canonical = candidate.canonicalize().map_err(|error| {
        TemplateBundleError::new(
            "TEMPLATE.BUNDLE_FILE_INVALID",
            format!("cannot open {role}: {error}"),
            Some(candidate.clone()),
        )
    })?;
    if !canonical.starts_with(root) {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_PATH_UNSAFE",
            format!("{role} resolves outside the template bundle"),
            Some(candidate),
        ));
    }
    let metadata = canonical.metadata().map_err(|error| {
        TemplateBundleError::new(
            "TEMPLATE.BUNDLE_FILE_INVALID",
            format!("cannot inspect {role}: {error}"),
            Some(canonical.clone()),
        )
    })?;
    if !metadata.is_file() || metadata.len() > size_limit {
        return Err(TemplateBundleError::new(
            "TEMPLATE.BUNDLE_FILE_INVALID",
            format!("{role} must be a regular file no larger than {size_limit} bytes"),
            Some(canonical),
        ));
    }
    Ok(canonical)
}

fn read_utf8(path: &Path, code: &'static str) -> Result<String, TemplateBundleError> {
    std::fs::read_to_string(path).map_err(|error| {
        TemplateBundleError::new(
            code,
            format!("file is not readable UTF-8: {error}"),
            Some(path.to_path_buf()),
        )
    })
}
