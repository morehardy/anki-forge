//! Streaming recipe decoding, with a per-asset budget before byte allocation.

use std::{cell::RefCell, collections::BTreeSet, fmt};

use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};

use super::{Asset, AssetSource, Input};
use crate::media::{MediaError, MediaLimits};

pub(super) fn input(reader: impl std::io::Read, limits: MediaLimits) -> anyhow::Result<Input> {
    let budget = Budget {
        limits,
        failure: RefCell::new(None),
    };
    let mut decoder = serde_json::Deserializer::from_reader(StopAfterFailure {
        reader,
        budget: &budget,
    });
    let decoded = MapSeed(InputVisitor(&budget)).deserialize(&mut decoder);
    let input = match decoded {
        Ok(input) => input,
        Err(cause) => {
            return Err(match budget.failure.borrow_mut().take() {
                Some(error) => anyhow::Error::new(error).context(cause),
                None => cause.into(),
            })
        }
    };
    decoder.end()?;
    Ok(input)
}

// This state belongs to one decode, never to a thread/global setting. Serde's
// error interface erases concrete sources, so retain the original media error
// here and attach the JSON location at the operation boundary.
struct Budget {
    limits: MediaLimits,
    failure: RefCell<Option<MediaError>>,
}

impl Budget {
    fn fail<E: Error>(&self, error: MediaError) -> E {
        *self.failure.borrow_mut() = Some(error);
        E::custom("inline media decoding failed")
    }
}

// Serde may probe closing delimiters/whitespace while unwinding a visitor
// error. Once a budget fails, do not let those probes read an unbounded tail.
struct StopAfterFailure<'a, R> {
    reader: R,
    budget: &'a Budget,
}
impl<R: std::io::Read> std::io::Read for StopAfterFailure<'_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.budget.failure.borrow().is_some() {
            Ok(0)
        } else {
            self.reader.read(buffer)
        }
    }
}

struct MapSeed<V>(V);
impl<'de, V: Visitor<'de>> DeserializeSeed<'de> for MapSeed<V> {
    type Value = V::Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        decoder.deserialize_map(self.0)
    }
}

fn key<'de, A: MapAccess<'de>>(
    map: &mut A,
    seen: &mut BTreeSet<String>,
) -> Result<Option<String>, A::Error> {
    let key = map.next_key::<String>()?;
    if let Some(key) = &key {
        if !seen.insert(key.clone()) {
            return Err(A::Error::custom(format!("duplicate field `{key}`")));
        }
    }
    Ok(key)
}

fn required<T, E: Error>(value: Option<T>, name: &'static str) -> Result<T, E> {
    value.ok_or_else(|| E::missing_field(name))
}

struct InputVisitor<'a>(&'a Budget);
impl<'de> Visitor<'de> for InputVisitor<'_> {
    type Value = Input;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an ankiforge project object")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Input, A::Error> {
        let (mut format_version, mut namespace, mut notes) = (None, None, None);
        let (mut name, mut default_deck) = (None, None);
        let (mut models, mut assets) = (Vec::new(), Vec::new());
        let mut seen = BTreeSet::new();
        while let Some(key) = key(&mut map, &mut seen)? {
            match key.as_str() {
                "format_version" => format_version = Some(map.next_value()?),
                "namespace" => namespace = Some(map.next_value()?),
                "name" => name = map.next_value()?,
                "default_deck" => default_deck = map.next_value()?,
                "models" => models = map.next_value()?,
                "assets" => assets = map.next_value_seed(Assets(self.0))?,
                "notes" => notes = Some(map.next_value()?),
                _ => {
                    return Err(A::Error::unknown_field(
                        &key,
                        &[
                            "format_version",
                            "namespace",
                            "name",
                            "default_deck",
                            "models",
                            "assets",
                            "notes",
                        ],
                    ))
                }
            }
        }
        Ok(Input {
            format_version: required(format_version, "format_version")?,
            namespace: required(namespace, "namespace")?,
            notes: required(notes, "notes")?,
            name,
            default_deck,
            models,
            assets,
        })
    }
}

struct Assets<'a>(&'a Budget);
impl<'de> DeserializeSeed<'de> for Assets<'_> {
    type Value = Vec<Asset>;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        decoder.deserialize_seq(self)
    }
}
impl<'de> Visitor<'de> for Assets<'_> {
    type Value = Vec<Asset>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an array of project assets")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut assets = Vec::new();
        while let Some(asset) = sequence.next_element_seed(MapSeed(AssetVisitor(self.0)))? {
            assets.push(asset);
        }
        Ok(assets)
    }
}

struct AssetVisitor<'a>(&'a Budget);
impl<'de> Visitor<'de> for AssetVisitor<'_> {
    type Value = Asset;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a project asset object")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Asset, A::Error> {
        let (mut asset_key, mut source, mut export_as) = (None, None, None);
        let mut seen = BTreeSet::new();
        while let Some(key) = key(&mut map, &mut seen)? {
            match key.as_str() {
                "key" => asset_key = Some(map.next_value()?),
                "source" => source = Some(map.next_value_seed(MapSeed(SourceVisitor(self.0)))?),
                "export_as" => export_as = map.next_value()?,
                _ => {
                    return Err(A::Error::unknown_field(
                        &key,
                        &["key", "source", "export_as"],
                    ))
                }
            }
        }
        Ok(Asset {
            key: required(asset_key, "key")?,
            source: required(source, "source")?,
            export_as,
        })
    }
}

struct SourceVisitor<'a>(&'a Budget);
impl<'de> Visitor<'de> for SourceVisitor<'_> {
    type Value = AssetSource;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a file or bytes asset source")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<AssetSource, A::Error> {
        let (mut kind, mut path, mut data, mut mime) = (None::<String>, None, None, None);
        let mut seen = BTreeSet::new();
        // Deserialize directly from the JSON stream even when `data` precedes
        // `kind`. Serde's derived internally tagged enum would buffer the whole
        // source as generic Content before a bounded byte visitor could run.
        while let Some(key) = key(&mut map, &mut seen)? {
            match key.as_str() {
                "kind" => kind = Some(map.next_value()?),
                "path" => path = Some(map.next_value()?),
                "mime" => mime = Some(map.next_value()?),
                "data" => data = Some(map.next_value_seed(Bytes(self.0))?),
                _ => {
                    return Err(A::Error::unknown_field(
                        &key,
                        &["kind", "path", "data", "mime"],
                    ))
                }
            }
        }
        match required(kind, "kind")?.as_str() {
            "file" => {
                if data.is_some() || mime.is_some() {
                    return Err(A::Error::custom("file source accepts only kind and path"));
                }
                Ok(AssetSource::File {
                    path: required(path, "path")?,
                })
            }
            "bytes" => {
                if path.is_some() {
                    return Err(A::Error::custom(
                        "bytes source accepts only kind, data and mime",
                    ));
                }
                Ok(AssetSource::Bytes {
                    data: required(data, "data")?,
                    mime: required(mime, "mime")?,
                })
            }
            kind => Err(A::Error::unknown_variant(kind, &["file", "bytes"])),
        }
    }
}

struct Bytes<'a>(&'a Budget);
impl<'de> DeserializeSeed<'de> for Bytes<'_> {
    type Value = Vec<u8>;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        decoder.deserialize_seq(self)
    }
}
impl<'de> Visitor<'de> for Bytes<'_> {
    type Value = Vec<u8>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an array of bytes within the media budget")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let limit = self.0.limits.max_bytes;
        let maximum = usize::try_from(limit).unwrap_or(usize::MAX);
        let mut data = Vec::new();
        while let Some(byte) = sequence.next_element::<u8>()? {
            if data.len() as u64 == limit {
                return Err(self
                    .0
                    .fail(MediaError::exceeded(limit, limit.saturating_add(1))));
            }
            if data.len() == data.capacity() {
                // Grow incrementally, capped by the actual limit, without
                // trusting a sequence length hint or reserving the whole budget.
                let capacity = data.capacity().saturating_mul(2).max(1024).min(maximum);
                data.try_reserve_exact(capacity - data.len())
                    .map_err(|cause| {
                        self.0.fail(MediaError::io(
                            "allocate inline media",
                            std::io::Error::other(cause),
                        ))
                    })?;
            }
            data.push(byte);
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, io::Read};

    struct Counted<'a> {
        bytes: &'a [u8],
        consumed: &'a Cell<usize>,
    }
    impl Read for Counted<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let count = self.bytes.read(buffer)?;
            self.consumed.set(self.consumed.get() + count);
            Ok(count)
        }
    }

    fn recipe(source: &str) -> String {
        format!(
            r#"{{"format_version":"ankiforge-project-v1","namespace":"budget","notes":[],"assets":[{{"key":"one","source":{source}}}]}}"#
        )
    }

    #[test]
    fn inline_budget_accepts_exact_limits_and_applies_per_asset() {
        for limit in [0, 1, 4, 1025] {
            let data = vec![255u8; limit];
            for source in [
                format!(r#"{{"kind":"bytes","mime":"application/octet-stream","data":{data:?}}}"#),
                format!(r#"{{"data":{data:?},"mime":"application/octet-stream","kind":"bytes"}}"#),
            ] {
                let json = recipe(&source);
                let decoded = input(
                    json.as_bytes(),
                    MediaLimits {
                        max_bytes: limit as u64,
                    },
                )
                .unwrap();
                let AssetSource::Bytes { data: actual, .. } = &decoded.assets[0].source else {
                    panic!("bytes source")
                };
                assert_eq!(actual, &data);
                assert!(actual.capacity() <= limit);
            }
            let excess = vec![0u8; limit + 1];
            let json = recipe(&format!(
                r#"{{"data":{excess:?},"kind":"bytes","mime":"text/css"}}"#
            ));
            let error = input(
                json.as_bytes(),
                MediaLimits {
                    max_bytes: limit as u64,
                },
            )
            .err()
            .expect("over budget");
            let error = error.downcast_ref::<MediaError>().unwrap();
            assert_eq!(error.code(), "MEDIA.RESOURCE_LIMIT_EXCEEDED");
            assert_eq!(error.limit_exceeded().unwrap().observed, limit as u64 + 1);
        }
        let json = br#"{"format_version":"ankiforge-project-v1","namespace":"budget","notes":[],"assets":[{"key":"one","source":{"data":[0,1],"kind":"bytes","mime":"text/css"}},{"key":"two","source":{"data":[2,3],"kind":"bytes","mime":"text/css"}}]}"#;
        assert_eq!(
            input(json.as_slice(), MediaLimits { max_bytes: 2 })
                .unwrap()
                .assets
                .len(),
            2
        );
    }

    #[test]
    fn streaming_decode_preserves_strict_json_structure() {
        let valid = r#"{"kind":"bytes","data":[0,255],"mime":"text/css"}"#;
        for source in [
            r#"{"kind":"bytes","kind":"bytes","data":[],"mime":"text/css"}"#,
            r#"{"kind":"bytes","data":[],"data":[],"mime":"text/css"}"#,
            r#"{"kind":"bytes","data":[],"mime":"text/css","path":"x"}"#,
            r#"{"kind":"file","path":"x","data":[]}"#,
            r#"{"kind":"file","path":"x","mime":"text/css"}"#,
            r#"{"kind":"bytes","data":[],"mime":"text/css","unknown":0}"#,
            r#"{"kind":"bytes","data":null,"mime":"text/css"}"#,
            r#"{"kind":"bytes","mime":"text/css"}"#,
            r#"{"data":[],"mime":"text/css"}"#,
            r#"{"kind":null,"data":[],"mime":"text/css"}"#,
            r#"{"kind":"unknown","data":[],"mime":"text/css"}"#,
            "[]",
            "null",
        ] {
            let json = recipe(source);
            let error = input(json.as_bytes(), MediaLimits { max_bytes: 4 })
                .err()
                .expect(source);
            assert!(
                error.downcast_ref::<serde_json::Error>().is_some(),
                "{error:?}"
            );
            assert!(error.downcast_ref::<MediaError>().is_none());
        }
        for byte in ["-1", "256", "1.5", "true", "\"1\"", "null"] {
            let json = recipe(&valid.replace("0,255", byte));
            assert!(
                input(json.as_bytes(), MediaLimits { max_bytes: 4 }).is_err(),
                "{byte}"
            );
        }
        let original = recipe(valid);
        for json in [
            original.replace(
                "\"namespace\":",
                "\"name\":null,\"name\":null,\"namespace\":",
            ),
            original.replace("\"key\":", "\"export_as\":null,\"export_as\":null,\"key\":"),
            original.replace("\"assets\":", "\"assets\":[],\"assets\":"),
            original.replace("\"source\":", &format!("\"source\":{valid},\"source\":")),
            original.replace("\"notes\":[]", "\"notes\":null"),
            original.replace("\"notes\":[]", "\"assets\":null,\"notes\":[]"),
            format!("{original} {{}}"),
            format!("{original} garbage"),
        ] {
            assert!(
                input(json.as_bytes(), MediaLimits { max_bytes: 4 }).is_err(),
                "{json}"
            );
        }
        let minimal = br#"{"format_version":"ankiforge-project-v1","namespace":"minimal","name":null,"default_deck":null,"notes":[]}"#;
        let decoded = input(minimal.as_slice(), MediaLimits { max_bytes: 0 }).unwrap();
        assert!(decoded.models.is_empty() && decoded.assets.is_empty());
    }

    #[test]
    fn streaming_decode_preserves_reader_and_syntax_errors() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "original reader failure",
                ))
            }
        }
        let error = input(Broken, MediaLimits::default()).err().unwrap();
        let original = error.downcast_ref::<serde_json::Error>().unwrap();
        assert_eq!(
            original.io_error_kind(),
            Some(std::io::ErrorKind::PermissionDenied)
        );
        assert!(original.to_string().contains("original reader failure"));
        let error = input(b"{\n invalid".as_slice(), MediaLimits::default())
            .err()
            .unwrap();
        let original = error.downcast_ref::<serde_json::Error>().unwrap();
        assert!(original.is_syntax());
        assert_eq!(original.line(), 2);
    }

    #[test]
    fn inline_budget_stops_at_first_excess_byte_without_buffering_source() {
        for (kind_before_data, separator) in [(false, ","), (true, ","), (false, " "), (true, " ")]
        {
            let kind = if kind_before_data {
                "\"kind\":\"bytes\","
            } else {
                ""
            };
            let prefix = format!(
                r#"{{"format_version":"ankiforge-project-v1","namespace":"budget","notes":[],"assets":[{{"key":"one","source":{{{kind}"data":[0,1,2,3,4{separator}"#
            );
            // The malformed unread tail also proves rejection happens during
            // the sequence, before tag buffering or full recipe validation.
            let tail = if separator == " " {
                " ".repeat(8192)
            } else {
                "0,".repeat(4096)
            };
            let json = format!("{prefix}{tail}invalid tail");
            let consumed = Cell::new(0);
            let reader = Counted {
                bytes: json.as_bytes(),
                consumed: &consumed,
            };
            let error = input(reader, MediaLimits { max_bytes: 4 })
                .err()
                .expect("over budget");
            assert!(
                error.downcast_ref::<serde_json::Error>().is_some(),
                "JSON location retained"
            );
            let limit = error
                .downcast_ref::<MediaError>()
                .expect("typed media error")
                .limit_exceeded()
                .expect("resource observations");
            assert_eq!(
                (limit.resource, limit.limit, limit.observed),
                ("media_bytes", 4, 5)
            );
            assert!(
                consumed.get() <= prefix.len(),
                "decoded beyond the first excess byte"
            );
        }
    }
}
