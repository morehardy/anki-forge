from pathlib import Path
from typing import assert_type
import io

from anki_forge import (
    ApkgArtifact, BasicIdentityOverride, BuildOptions, BuildReport, Content, Deck,
    Field, IdentityRecipe, InspectLimits, Note, NoteType, Project, ProjectDiffReport,
    Template, Versions, versions,
)

project = Project("Typed")
project.add_notetype(NoteType.custom("word").field(Field("Prompt", key="prompt", optional=True))
    .template(Template("Card", key="card", front="{{Prompt}}", back="{{FrontSide}}"))
    .identity(IdentityRecipe.fields(["prompt"])))
project.add_note(Note("word").field("prompt", Content.text("Hello")).identity(["prompt"]))
report = project.build(BuildOptions(inspect_limits=InspectLimits(max_entries=100)))
assert_type(report, BuildReport[ApkgArtifact])
if report.artifact is not None:
    assert_type(report.artifact.path, Path)
    assert_type(report.artifact.persist_to("copy.apkg"), ApkgArtifact)
assert_type(project.diff_against_apkg("previous.apkg"), ProjectDiffReport)
assert_type(project.write_to(io.BytesIO()), int)
deck = Deck("Typed", basic_identity=["front"])
deck.add_basic("front", "back", identity_override=BasicIdentityOverride(["back"], "answer"))
assert_type(Project.from_deck(deck), Project)
assert_type(versions(), Versions)
