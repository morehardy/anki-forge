"""Target high-level Product media API shape for future Python bindings.

The implemented Python runtime module is currently `anki_forge_python`; this
file documents the intended future `anki_forge` API and is not a runnable
example yet.
"""

from anki_forge import Field, GenerationRule, IdentityRecipe, Note, NoteType, Project, Template


TINY_PNG = bytes(
    [
        137,
        80,
        78,
        71,
        13,
        10,
        26,
        10,
        0,
        0,
        0,
        13,
        73,
        72,
        68,
        82,
        0,
        0,
        0,
        1,
        0,
        0,
        0,
        1,
        8,
        6,
        0,
        0,
        0,
        31,
        21,
        196,
        137,
        0,
        0,
        0,
        12,
        73,
        68,
        65,
        84,
        120,
        156,
        99,
        248,
        15,
        4,
        0,
        9,
        251,
        3,
        253,
        167,
        102,
        129,
        94,
        0,
        0,
        0,
        0,
        73,
        69,
        78,
        68,
        174,
        66,
        96,
        130,
    ]
)


def tiny_wav(sample: int) -> bytes:
    return bytes(
        [
            ord("R"),
            ord("I"),
            ord("F"),
            ord("F"),
            37,
            0,
            0,
            0,
            ord("W"),
            ord("A"),
            ord("V"),
            ord("E"),
            ord("f"),
            ord("m"),
            ord("t"),
            ord(" "),
            16,
            0,
            0,
            0,
            1,
            0,
            1,
            0,
            0x40,
            0x1F,
            0,
            0,
            0x40,
            0x1F,
            0,
            0,
            1,
            0,
            8,
            0,
            ord("d"),
            ord("a"),
            ord("t"),
            ord("a"),
            1,
            0,
            0,
            0,
            sample,
        ]
    )


def build_project() -> Project:
    project = Project(
        name="Spanish Media",
        stable_id="spanish-media",
        default_deck="Spanish::Media",
    )

    media = project.media
    audio = media.add_bytes(
        source_label="hola-source.wav",
        data=tiny_wav(128),
        export_as="hola.wav",
    )
    picture = media.add_bytes(
        source_label="hola-picture-source.png",
        data=TINY_PNG,
        export_as="hola.png",
    )
    media.add_bytes(
        source_label="unused-hint-source.wav",
        data=tiny_wav(127),
        export_as="unused-hint.wav",
    )

    vocab = NoteType.custom("spanish-vocab", name="Spanish Vocabulary")
    vocab.field(Field("Expression", key="expression", identity=True, sort=True))
    vocab.field(Field("Meaning", key="meaning", required=True))
    vocab.field(Field("Audio", key="audio", optional=True))
    vocab.field(Field("Picture", key="picture", optional=True))
    vocab.template(
        Template(
            "Recognition",
            key="recognition",
            front='<img class="deck-logo" src="hola.png" alt=""> {{Expression}}',
            back=(
                "{{FrontSide}}<hr id=\"answer\">{{Meaning}}"
                '<div class="media">{{Audio}}{{Picture}}</div>'
            ),
            generate_when=GenerationRule.all(["expression"]),
        )
    )
    vocab.css = (
        '.card { font-family: Arial, sans-serif; background-image: url("hola.png"); }\n'
        ".deck-logo { width: 32px; height: 32px; }\n"
        ".media img { max-width: 120px; }"
    )
    vocab.identity = IdentityRecipe.fields(["expression"])

    project.add_notetype(vocab)
    note = Note("spanish-vocab", stable_id="es:hola")
    note.text("expression", "hola")
    note.text("meaning", "hello")
    note.sound("audio", audio)
    note.image("picture", picture)
    project.add_note(note)
    return project


def write_example() -> None:
    report = build_project().write_apkg("spanish-media.apkg")
    print(report.pretty_report())
    report.ensure_success()
    assert report.media.unused_bindings == 1
    assert "MEDIA.UNUSED_BINDING" in report.diagnostic_codes()
