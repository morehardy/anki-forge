from anki_forge import BuildOptions, BuildReport, Deck, InspectLimits, Project

BuildOptions(fail_on="severe")
InspectLimits(max_entries="unbounded")
Deck("Bad").add_basic(7, "back")
Project.from_deck(Project("Wrong object"))
Project("Bad sink").write_to(object())
parsed = BuildReport.from_json({})
if parsed.artifact is not None:
    parsed.artifact.persist_to("no-ownership.apkg")
