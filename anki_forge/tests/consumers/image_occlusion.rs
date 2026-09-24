use std::{error::Error, fs, io::Read};
use ankiforge::{BuildOptions, Media, Note, Project};
use ankiforge::note::{ImageOcclusionBuilder, ImageOcclusionError, ImageOcclusionErrorKind as Kind, Mask, OcclusionMode};
use prost::Message;

#[derive(Clone, PartialEq, Message)]
struct NotetypeConfig {
    #[prost(int32, tag="1")] kind: i32,
    #[prost(int32, tag="9")] stock: i32,
}

fn entry(archive: &mut zip::ZipArchive<fs::File>, name: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    archive.by_name(name).unwrap().read_to_end(&mut bytes).unwrap();
    zstd::decode_all(bytes.as_slice()).unwrap()
}
fn builder(image: &Media) -> ImageOcclusionBuilder {
    Note::image_occlusion(image.clone())
        .mask(Mask::rect("nucleus", 10, 10, 30, 20))
        .mask(Mask::rect("membrane", 50, 30, 10, 20))
}
fn main() -> Result<(), Box<dyn Error>> {
    fn error_traits<T: Error + Send + Sync + 'static>() {}
    error_traits::<ImageOcclusionError>();
    let image = Media::file("assets/occlusion.png")?;
    let mut project = Project::new("io-contract")?;
    project.add("all", builder(&image).build()?.field("header", "<b>Choose</b>"))?;
    project.add("one", builder(&image).mode(OcclusionMode::HideOneGuessOne).build()?)?;
    fs::remove_file("assets/occlusion.png")?;
    let output = project.build(BuildOptions::temporary())?;
    assert_eq!(output.report().counts().notes, 2);
    assert_eq!(output.report().counts().cards, 4);
    assert_eq!(output.report().counts().media, 1);
    let mut archive = zip::ZipArchive::new(fs::File::open(output.artifact().path())?)?;
    fs::write("io.sqlite", entry(&mut archive, "collection.anki21b"))?;
    let db = rusqlite::Connection::open("io.sqlite")?;
    let config: Vec<u8> = db.query_row("select config from notetypes", [], |row| row.get(0))?;
    let config = NotetypeConfig::decode(config.as_slice())?;
    assert_eq!(config.kind, 1);
    assert_eq!(config.stock, 6, "Anki must recognize this as its image-occlusion model");
    let fields = db.prepare("select flds from notes order by id")?.query_map([], |row| row.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    let all = "{{c1::image-occlusion:rect:left=0.1:top=0.125:width=0.3:height=0.25:oi=1}}<br>{{c2::image-occlusion:rect:left=0.5:top=0.375:width=0.1:height=0.25:oi=1}}<br>";
    let one = "{{c1::image-occlusion:rect:left=0.1:top=0.125:width=0.3:height=0.25}}<br>{{c2::image-occlusion:rect:left=0.5:top=0.375:width=0.1:height=0.25}}<br>";
    assert_eq!(fields[0].split('\u{1f}').next().unwrap(), all);
    assert!(fields[0].contains("&lt;b&gt;Choose&lt;/b&gt;"));
    assert_eq!(fields[1].split('\u{1f}').next().unwrap(), one);
    let ordinals = db.prepare("select ord from cards order by nid, ord")?.query_map([], |row| row.get::<_, i64>(0))?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(ordinals, [0, 1, 0, 1]);

    let empty = Note::image_occlusion(image.clone()).build().unwrap_err();
    assert_eq!(empty.kind(), Kind::InvalidMask);
    assert_eq!(empty.code(), "NOTE.IO_MASKS_EMPTY");
    for mask in [Mask::rect("", 0, 0, 1, 1), Mask::rect("zero", 0, 0, 0, 1), Mask::rect("outside", 99, 0, 2, 1), Mask::rect("negative", -1, 0, 1, 1), Mask::rect("nan", f64::NAN, 0, 1, 1)] {
        assert_eq!(Note::image_occlusion(image.clone()).mask(mask).build().unwrap_err().kind(), Kind::InvalidMask);
    }
    let duplicate = builder(&image).mask(Mask::rect("nucleus", 0, 0, 1, 1)).build().unwrap_err();
    assert_eq!(duplicate.code(), "NOTE.IO_MASK_KEY_DUPLICATE");
    let corrupt = Media::bytes(b"\x89PNG\r\n\x1a\nnot-a-png".to_vec(), "image/png")?;
    let invalid = Note::image_occlusion(corrupt).mask(Mask::rect("mask", 0, 0, 1, 1)).build().unwrap_err();
    assert_eq!(invalid.kind(), Kind::InvalidImage);
    assert!(invalid.source().is_some());
    let wrapped = anyhow::Error::new(invalid).context("preparing lesson diagram");
    assert_eq!(wrapped.downcast_ref::<ImageOcclusionError>().unwrap().code(), "NOTE.IO_IMAGE_INVALID");

    let overridden = builder(&image).build()?.field("occlusion", "{{c99::overwrite}}");
    assert_eq!(project.add("override", overridden).unwrap_err().code(), "NOTE.IO_FIELD_RESERVED");
    let incomplete = builder(&image).build()?.note_type().note();
    assert_eq!(project.add("incomplete", incomplete).unwrap_err().code(), "NOTE.IO_STRUCTURE_REQUIRED");
    assert_eq!(project.len(), 2);
    let mut excessive = Note::image_occlusion(image.clone());
    for i in 0..501 { excessive = excessive.mask(Mask::rect(format!("m{i}"), 0, 0, 1, 1)); }
    assert_eq!(excessive.build().unwrap_err().code(), "NOTE.IO_ORDINAL_EXHAUSTED");

    let rotated = Media::file("assets/rotated.jpg")?;
    let oriented = Note::image_occlusion(rotated.clone()).mask(Mask::rect("corner", 70, 90, 10, 10)).build()?;
    let mut rotation = Project::new("io-rotation")?;
    rotation.add("rotated", oriented)?;
    let output = rotation.build(BuildOptions::temporary())?;
    let mut archive = zip::ZipArchive::new(fs::File::open(output.artifact().path())?)?;
    let png = entry(&mut archive, "0");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 80);
    assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 100);
    assert_eq!(Note::image_occlusion(rotated).mask(Mask::rect("outside", 80, 90, 1, 1)).build().unwrap_err().kind(), Kind::InvalidMask);
    Ok(())
}
