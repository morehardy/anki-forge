//! Serialize identity checksum bytes directly in canonical key order.
use std::collections::BTreeMap;

use serde::{ser::SerializeMap, Serialize, Serializer};

use super::{
    MaskIdentity, MediaIdentity, ModelIdentity, NoteIdentity, PackageIdentity, SymbolIdentity,
};

struct Canonical<'a, T>(&'a T);
struct CanonicalMap<'a, T>(&'a BTreeMap<String, T>);

impl<T> Serialize for CanonicalMap<'_, T>
where
    for<'a> Canonical<'a, T>: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            map.serialize_entry(key, &Canonical(value))?;
        }
        map.end()
    }
}

// Field lists deliberately match serde_json::Value's recursive lexical order.
// The ordinary sidecar serializer and its public wire shape are unchanged.
macro_rules! canonical {
    ($ty:ty, $v:ident, {$($key:literal => $entry:expr),* $(,)?}) => {
        impl Serialize for Canonical<'_, $ty> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let $v = self.0;
                let mut map = serializer.serialize_map(Some([$($key),*].len()))?;
                $(map.serialize_entry($key, &$entry)?;)*
                map.end()
            }
        }
    };
}

canonical!(PackageIdentity, v, {
    "media_history" => CanonicalMap(&v.media_history),
    "models" => CanonicalMap(&v.models),
    "namespace" => v.namespace,
    "notes" => CanonicalMap(&v.notes),
});
canonical!(ModelIdentity, v, {
    "active" => v.active,
    "content_hash" => v.content_hash,
    "field_high_water" => v.field_high_water,
    "fields" => CanonicalMap(&v.fields),
    "id" => v.id,
    "kind" => v.kind,
    "mtime_secs" => v.mtime_secs,
    "sort_field" => v.sort_field,
    "template_high_water" => v.template_high_water,
    "templates" => CanonicalMap(&v.templates),
});
canonical!(NoteIdentity, v, {
    "active" => v.active,
    "cards" => v.cards,
    "content_hash" => v.content_hash,
    "guid" => v.guid,
    "mask_high_water" => v.mask_high_water,
    "masks" => CanonicalMap(&v.masks),
    "model" => v.model,
    "mtime_secs" => v.mtime_secs,
});
canonical!(SymbolIdentity, v, {
    "id" => v.id,
    "ordinal" => v.ordinal,
    "slot" => v.slot,
});
canonical!(MaskIdentity, v, {
    "active" => v.active,
    "ordinal" => v.ordinal,
});
canonical!(MediaIdentity, v, {
    "sha1" => v.sha1,
    "size" => v.size,
});

pub(super) fn bytes(identity: &PackageIdentity) -> anyhow::Result<Vec<u8>> {
    Ok(serde_json::to_vec(&Canonical(identity))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_wire_bytes_match_recursive_canonicalization() {
        let mut project = crate::Project::new("canonical-身份").unwrap();
        project
            .add("z", crate::Note::basic("front", "back"))
            .unwrap();
        project
            .add("a", crate::Note::cloze("{{c1::text}}"))
            .unwrap();
        let mut identity = PackageIdentity::create(&project).unwrap();
        identity.notes.get_mut("z").unwrap().active = false;
        identity.notes.get_mut("a").unwrap().masks.insert(
            "遮罩\"\n\\".into(),
            MaskIdentity {
                ordinal: u16::MAX,
                active: true,
            },
        );
        identity.media_history.insert(
            "cafe\u{301}.bin".into(),
            MediaIdentity {
                sha1: "a".repeat(40),
                size: u64::MAX,
            },
        );
        for model in identity.models.values_mut() {
            model.mtime_secs = i64::MAX;
            model.fields.values_mut().next().unwrap().ordinal = None;
        }
        let reference = crate::writer_core::canonical_json::to_canonical_json(&identity).unwrap();
        assert_eq!(bytes(&identity).unwrap(), reference.as_bytes());
        let reference_shape = serde_json::to_value(&identity).unwrap();
        let actual: serde_json::Value = serde_json::from_slice(&bytes(&identity).unwrap()).unwrap();
        assert_eq!(actual, reference_shape);
    }
}
