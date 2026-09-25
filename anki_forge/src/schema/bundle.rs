use std::{
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

use super::{
    bundle_manifest::{Generation, Manifest},
    Field, GenerationRule, NoteType, SchemaError, Template, TemplateBundleError as Error,
    TemplateBundleErrorKind as Kind, TemplateBundleLimitExceeded, TemplateSide,
};
use crate::{
    media::{MediaErrorKind, MediaLimits},
    Media,
};

const MANIFEST: &str = "anki-template.yaml";
const MANIFEST_LIMIT: u64 = 256 << 10;
const TEXT_LIMIT: u64 = 2 << 20;

impl NoteType {
    /// Loads a `template-bundle-v2` directory into an immutable model that owns
    /// its complete asset closure. Later changes to that directory have no effect.
    ///
    /// The manifest is `anki-template.yaml`. Fields and templates use stable
    /// `key` values; templates refer to field keys. Set `note_type.cloze_field`
    /// for a cloze model. Assets declare bundle-relative `path` and `export_as`.
    /// Every referenced path must resolve inside the directory. Only this new
    /// format is supported; unknown manifest fields are rejected.
    ///
    /// The manifest is limited to 256 KiB, each template or CSS file to 2 MiB,
    /// and each asset to the default [`MediaLimits`]. Errors retain their input
    /// path and actual I/O, parse, schema or media source.
    pub fn from_bundle(path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::from_bundle_with_limits(path, MediaLimits::default())
    }

    /// Loads a bundle with a per-asset budget checked before and during each
    /// snapshot import. Text budgets remain those documented on [`Self::from_bundle`].
    /// Failure drops all newly owned assets; no model is partially registered.
    pub fn from_bundle_with_limits(
        path: impl AsRef<Path>,
        limits: MediaLimits,
    ) -> Result<Self, Error> {
        load(path.as_ref(), limits)
    }
}

fn load(path: &Path, limits: MediaLimits) -> Result<NoteType, Error> {
    let root = path.canonicalize().map_err(|cause| {
        Error::new(
            Kind::Io,
            "TEMPLATE.BUNDLE_ROOT_INVALID",
            "open bundle directory",
            path,
        )
        .caused_by(cause)
    })?;
    if !root.is_dir() {
        return Err(Error::new(
            Kind::InvalidFile,
            "TEMPLATE.BUNDLE_ROOT_INVALID",
            "bundle root must be a directory",
            root,
        ));
    }
    let manifest_path = resolve(&root, MANIFEST)?;
    let source = read_text(&manifest_path, MANIFEST_LIMIT)?;
    let manifest_error = || {
        Error::new(
            Kind::InvalidManifest,
            "TEMPLATE.BUNDLE_MANIFEST_INVALID",
            "parse bundle manifest",
            &manifest_path,
        )
    };
    // Preserve YAML scalar types and duplicate-key rejection, then apply the
    // strict JSON-compatible manifest shape. Direct YAML deserialization can
    // coerce scalars into strings, and YAML Value deserialization treats null
    // as an empty sequence.
    let value: serde_yaml::Value =
        serde_yaml::from_str(&source).map_err(|cause| manifest_error().caused_by(cause))?;
    let manifest: Manifest = serde_json::to_value(value)
        .and_then(serde_json::from_value)
        .map_err(|cause| manifest_error().caused_by(cause))?;
    if manifest.format_version != "template-bundle-v2" {
        return Err(Error::new(
            Kind::UnsupportedVersion,
            "TEMPLATE.BUNDLE_VERSION_UNSUPPORTED",
            "expected template-bundle-v2",
            manifest_path,
        ));
    }
    let mut builder = NoteType::builder(manifest.note_type.key);
    if let Some(name) = manifest.note_type.name {
        builder = builder.name(name);
    }
    if let Some(cloze) = manifest.note_type.cloze_field {
        builder = builder.cloze_field(cloze);
    }
    for field in manifest.note_type.fields {
        let mut value = Field::new(field.key);
        if let Some(name) = field.name {
            value = value.name(name);
        }
        if field.required {
            value = value.required();
        }
        if field.sort {
            value = value.sort();
        }
        builder = builder.field(value);
    }
    let mut origins = Vec::new();
    for template in manifest.note_type.templates {
        let mut value = Template::new(template.key);
        if let Some(name) = template.name {
            value = value.name(name);
        }
        for (side, file) in [
            (TemplateSide::Front, Some(template.front_file)),
            (TemplateSide::Back, Some(template.back_file)),
            (TemplateSide::BrowserFront, template.browser_front_file),
            (TemplateSide::BrowserBack, template.browser_back_file),
        ] {
            let Some(file) = file else {
                continue;
            };
            let path = resolve(&root, &file)?;
            let source = read_text(&path, TEXT_LIMIT)?;
            origins.push((value.key().clone(), side, path));
            value = match side {
                TemplateSide::Front => value.front(source),
                TemplateSide::Back => value.back(source),
                TemplateSide::BrowserFront => value.browser_front(source),
                TemplateSide::BrowserBack => value.browser_back(source),
            };
        }
        if let Some(deck) = template.target_deck {
            value = value.target_deck(deck);
        }
        if let Some(rule) = template.generation_rule {
            value = value.generate_when(match rule {
                Generation::AnkiDefault => GenerationRule::AnkiDefault,
                Generation::All { fields } => GenerationRule::all(fields),
                Generation::Any { fields } => GenerationRule::any(fields),
            });
        }
        builder = builder.template(value);
    }
    if let Some(file) = manifest.css_file {
        builder = builder.css(read_text(&resolve(&root, &file)?, TEXT_LIMIT)?);
    }
    let schema_error = |cause: SchemaError| {
        let path = cause
            .location()
            .and_then(|location| {
                origins
                    .iter()
                    .find(|(key, side, _)| key == &location.template && side == &location.side)
            })
            .map_or(&manifest_path, |(_, _, path)| path);
        Error::new(
            Kind::InvalidSchema,
            cause.code(),
            "validate bundle model",
            path,
        )
        .caused_by(cause)
    };
    // Reject invalid declarations before importing potentially large assets.
    super::validation::validate(&builder).map_err(&schema_error)?;
    for asset in manifest.assets {
        let path = resolve(&root, &asset.path)?;
        let media = Media::file_with_limits(&path, limits)
            .and_then(|media| media.with_export_name(asset.export_as))
            .map_err(|cause| {
                let kind = match cause.kind() {
                    MediaErrorKind::ResourceLimit => Kind::ResourceLimit,
                    MediaErrorKind::Io => Kind::Io,
                    _ => Kind::InvalidMedia,
                };
                Error::new(kind, cause.code(), "import bundle asset", &path).caused_by(cause)
            })?;
        builder = builder.asset(media);
    }
    builder.build().map_err(schema_error)
}

fn resolve(root: &Path, relative: &str) -> Result<PathBuf, Error> {
    let path = Path::new(relative);
    if relative.is_empty()
        || relative.contains(['\\', ':'])
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(Error::new(
            Kind::UnsafePath,
            "TEMPLATE.BUNDLE_PATH_UNSAFE",
            "input must be a portable path inside the bundle",
            path,
        ));
    }
    let candidate = root.join(path);
    let canonical = candidate.canonicalize().map_err(|cause| {
        Error::new(
            Kind::Io,
            "TEMPLATE.BUNDLE_FILE_INVALID",
            "resolve bundle input",
            &candidate,
        )
        .caused_by(cause)
    })?;
    if !canonical.starts_with(root) {
        return Err(Error::new(
            Kind::UnsafePath,
            "TEMPLATE.BUNDLE_PATH_UNSAFE",
            "input resolves outside the bundle",
            candidate,
        ));
    }
    Ok(canonical)
}

fn read_text(path: &Path, limit: u64) -> Result<String, Error> {
    let io_error = |cause| {
        Error::new(
            Kind::Io,
            "TEMPLATE.BUNDLE_FILE_INVALID",
            "read bundle text",
            path,
        )
        .caused_by(cause)
    };
    let metadata = path.metadata().map_err(&io_error)?;
    if !metadata.is_file() {
        return Err(Error::new(
            Kind::InvalidFile,
            "TEMPLATE.BUNDLE_FILE_INVALID",
            "bundle text must be a regular file",
            path,
        ));
    }
    let file = File::open(path).map_err(&io_error)?;
    let metadata = file.metadata().map_err(&io_error)?;
    if !metadata.is_file() {
        return Err(Error::new(
            Kind::InvalidFile,
            "TEMPLATE.BUNDLE_FILE_INVALID",
            "opened bundle text is not a regular file",
            path,
        ));
    }
    let exceeded = |observed| {
        Error::new(
            Kind::ResourceLimit,
            "TEMPLATE.BUNDLE_RESOURCE_LIMIT_EXCEEDED",
            "bundle text exceeds its byte budget",
            path,
        )
        .caused_by(TemplateBundleLimitExceeded { limit, observed })
    };
    if metadata.len() > limit {
        return Err(exceeded(metadata.len()));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > limit {
        return Err(exceeded(bytes.len() as u64));
    }
    String::from_utf8(bytes).map_err(|cause| {
        Error::new(
            Kind::InvalidFile,
            "TEMPLATE.BUNDLE_FILE_INVALID",
            "bundle text must be UTF-8",
            path,
        )
        .caused_by(cause)
    })
}
