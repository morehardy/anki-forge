"""Target high-level Product API shape for future Python bindings.

The implemented Python runtime module is currently `anki_forge_python`; this
file documents the intended future `anki_forge` API and is not a runnable
example yet.
"""

from anki_forge import Field, GenerationRule, IdentityRecipe, Note, NoteType, Project, Template


def build_project() -> Project:
    project = Project(
        name="Japanese Core",
        stable_id="jp-core",
        default_deck="Japanese::Core",
    )

    vocab = NoteType.custom("jp-vocab", name="Japanese Vocabulary")
    vocab.field(Field("Expression", key="expr", identity=True, sort=True))
    vocab.field(Field("Meaning", key="meaning", required=True))
    vocab.template(
        Template(
            "Recognition",
            key="recognition",
            front="{{Expression}}",
            back="{{FrontSide}}<hr id='answer'>{{Meaning}}",
            generate_when=GenerationRule.all(["expr"]),
        )
    )
    vocab.identity = IdentityRecipe.fields(["expr"])

    project.add_notetype(vocab)
    project.add_note(
        Note("jp-vocab", stable_id="jp-vocab:taberu")
        .text("expr", "食べる")
        .text("meaning", "to eat")
    )
    return project


def write_example() -> None:
    report = build_project().write_apkg("jp-core.apkg")
    report.ensure_success()
