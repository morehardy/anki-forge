use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetSource {
    InlineTemplateStatic {
        namespace: String,
        filename: String,
        mime: String,
        data_base64: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontBinding {
    pub note_type_id: String,
    pub family: String,
    pub filename: String,
}

impl AssetSource {
    pub(super) fn namespace(&self) -> &str {
        match self {
            AssetSource::InlineTemplateStatic { namespace, .. } => namespace,
        }
    }

    pub(super) fn lowered_filename(&self) -> String {
        match self {
            AssetSource::InlineTemplateStatic {
                namespace,
                filename,
                mime: _,
                data_base64,
            } => {
                let short_hash = short_hash(&format!("{namespace}\n{filename}\n{data_base64}"));
                let namespace = sanitize_identifier(namespace);
                let extension = std::path::Path::new(filename)
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .map(|extension| format!(".{extension}"))
                    .unwrap_or_default();
                format!("_{namespace}_{short_hash}{extension}")
            }
        }
    }

    pub(super) fn mime(&self) -> &str {
        match self {
            AssetSource::InlineTemplateStatic { mime, .. } => mime,
        }
    }

    pub(super) fn data_base64(&self) -> &str {
        match self {
            AssetSource::InlineTemplateStatic { data_base64, .. } => data_base64,
        }
    }

    pub(super) fn filename(&self) -> &str {
        match self {
            AssetSource::InlineTemplateStatic { filename, .. } => filename,
        }
    }

    pub(super) fn product_id(&self) -> String {
        match self {
            AssetSource::InlineTemplateStatic {
                namespace,
                filename,
                ..
            } => format!("{namespace}/{filename}"),
        }
    }

    pub(super) fn identity(&self) -> String {
        format!("{}/{}", self.namespace(), self.filename())
    }
}

fn short_hash(input: &str) -> String {
    hex::encode(Sha1::digest(input.as_bytes()))[..12].to_string()
}

fn sanitize_identifier(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    if sanitized.is_empty() {
        "asset".into()
    } else {
        sanitized
    }
}
