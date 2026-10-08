#!/usr/bin/env python3
"""Python P1 workflow control, using the native SDK without intervening edits."""
import json
import sys
import time

from ankiforge import BuildOptions, CompareOptions, Note, Project

input_path, destination, mode, baseline = sys.argv[1:]
started = time.perf_counter()
with open(input_path) as stream:
    document = json.load(stream)
project = Project("publication-node")
for index, note in enumerate(document["notes"]):
    project.add(f"note-{index}", Note.basic(note["front"], note["back"]))
input_ms = (time.perf_counter() - started) * 1000
operation_started = time.perf_counter()
options = BuildOptions.to(destination).update_from(baseline)
if mode == "compare-build":
    comparison = project.compare(CompareOptions.against(baseline)).snapshot()
    output = project.build(options)
else:
    prepared = project.prepare_publication(options)
    comparison = prepared.report.comparison.snapshot()
    output = prepared.publish()
assert output.report.counts.notes == document["note_count"]
operation_ms = (time.perf_counter() - operation_started) * 1000
output.artifact.close()
print(json.dumps(dict(input_ms=input_ms, operation_ms=operation_ms,
                     comparison=comparison, report=output.report.snapshot())))
