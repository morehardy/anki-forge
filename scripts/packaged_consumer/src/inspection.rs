//! Independent ZIP/protobuf/SQLite observations; no AnkiForge internals.
use anyhow::{ensure, Context};
use prost::Message;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, io::Read, path::Path};

#[derive(Clone, PartialEq, Message)]
struct MediaEntries {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<MediaEntry>,
}
#[derive(Clone, PartialEq, Message)]
struct MediaEntry {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(uint32, tag = "2")]
    size: u32,
    #[prost(bytes, tag = "3")]
    sha1: Vec<u8>,
}
#[derive(Clone, PartialEq, Message)]
struct TemplateConfig {
    #[prost(string, tag = "1")]
    front: String,
    #[prost(string, tag = "2")]
    back: String,
}
#[derive(Debug)]
pub struct Observed {
    pub fields: Vec<String>,
    pub guids: Vec<String>,
    pub ordinals: Vec<u16>,
    pub models: BTreeMap<i64, Value>,
    pub assets: BTreeMap<String, Vec<u8>>,
    pub identity: Value,
}
fn decoded(archive: &mut zip::ZipArchive<fs::File>, name: &str) -> anyhow::Result<Vec<u8>> {
    let mut bytes = vec![];
    archive.by_name(name)?.read_to_end(&mut bytes)?;
    Ok(zstd::stream::decode_all(bytes.as_slice())?)
}
pub fn inspect(path: &Path) -> anyhow::Result<Observed> {
    let mut archive = zip::ZipArchive::new(fs::File::open(path)?)?;
    let collection = decoded(&mut archive, "collection.anki21b")?;
    let temp = tempfile::tempdir()?;
    let db_path = temp.path().join("collection.db");
    fs::write(&db_path, collection)?;
    let db = Connection::open(db_path)?;
    ensure!(
        db.query_row("pragma integrity_check", [], |r| r.get::<_, String>(0))? == "ok",
        "SQLite integrity"
    );
    let fields = db
        .prepare("select flds from notes order by guid")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    let guids = db
        .prepare("select guid from notes order by guid")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    let ordinals = db
        .prepare("select ord from cards order by nid,ord")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<u16>, _>>()?;
    let mut models = BTreeMap::new();
    for row in db
        .prepare("select id,name from notetypes order by id")?
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
    {
        let (id, name) = row?;
        let names = db
            .prepare("select name from fields where ntid=? order by ord")?
            .query_map([id], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let mut templates = vec![];
        for row in db
            .prepare("select name,config from templates where ntid=? order by ord")?
            .query_map([id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
            })?
        {
            let (name, config) = row?;
            let config = TemplateConfig::decode(config.as_slice())?;
            templates.push(json!({"name":name,"front":config.front,"back":config.back}));
        }
        models.insert(
            id,
            json!({"name":name,"fields":names,"templates":templates}),
        );
    }
    let map = MediaEntries::decode(decoded(&mut archive, "media")?.as_slice())?;
    let mut assets = BTreeMap::new();
    for (ordinal, entry) in map.entries.into_iter().enumerate() {
        let bytes = decoded(&mut archive, &ordinal.to_string())?;
        ensure!(
            bytes.len() == entry.size as usize,
            "media length differs for {}",
            entry.name
        );
        ensure!(
            assets.insert(entry.name, bytes).is_none(),
            "duplicate media filename"
        );
    }
    let identity: Value = serde_json::from_reader(archive.by_name("ankiforge-identity.json")?)?;
    ensure!(
        identity["format_version"] == "ankiforge-identity-v1",
        "complete identity evidence missing"
    );
    let notes = identity["identity"]["notes"]
        .as_object()
        .context("identity notes")?;
    let active: Vec<_> = notes.values().filter(|n| n["active"] == true).collect();
    ensure!(
        active.len() == fields.len(),
        "actual note count and evidence differ"
    );
    for note in active {
        let guid = note["guid"].as_str().context("identity guid")?;
        ensure!(
            guids.iter().any(|g| g == guid),
            "actual database missing evidenced GUID"
        );
    }
    Ok(Observed {
        fields,
        guids,
        ordinals,
        models,
        assets,
        identity,
    })
}
