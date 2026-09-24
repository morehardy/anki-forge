#![deny(unused_must_use)]

use std::borrow::Cow;
use ankiforge::{Content, Field, Media, Note, NoteType, Template};
use ankiforge::schema::FieldKey;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = NoteType::builder("vocab")
        .field(Field::new("front").name("正面"))
        .template(Template::new("card").front("{{front}}")).build()?;
    let string = String::from("front");
    let key = FieldKey::from("front");
    let notes = [
        model.note().field("front", "细胞"),
        model.note().field(string.clone(), "细胞"),
        model.note().field(&string, "细胞"),
        model.note().field(Cow::Borrowed("front"), "细胞"),
        model.note().field(Cow::Owned::<str>(string), "细胞"),
        model.note().field(key.clone(), "细胞"),
        model.note().field(&key, "细胞"),
    ];
    for note in &notes {
        assert_eq!(note, &notes[0]);
        assert_eq!(note.note_type().key(), "vocab");
    }
    drop(model);
    assert_eq!(notes[0].note_type().fields()[0].display_name(), "正面");
    assert_eq!(Note::basic("<b>cell</b>", "answer"),
        Note::basic(Content::text("<b>cell</b>"), "answer"));
    assert_ne!(Note::basic("<b>cell</b>", "answer"),
        Note::basic(Content::html("<b>cell</b>"), "answer"));
    assert_eq!(Note::cloze("<b>{{c1::cell}}</b>"),
        Note::cloze(Content::text("<b>{{c1::cell}}</b>")));
    let image = Media::file("assets/pixel.png")?;
    let audio = Media::file("assets/silence.wav")?;
    let content = Content::sequence([Content::text("question"), image.image(), audio.sound()]);
    let note = Note::basic(content, Content::html("<b>answer</b>"));
    drop(image);
    drop(audio);
    assert_eq!(note.note_type().key(), "basic");
    Ok(())
}
