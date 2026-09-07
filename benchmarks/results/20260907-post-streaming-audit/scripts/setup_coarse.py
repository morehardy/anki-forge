"""Apply the exact additional coarse probes used in this assessment."""
from pathlib import Path
import runpy
import shutil

WORK = Path(__file__).resolve().parent
state = runpy.run_path(str(WORK / 'instrument.py'))
for file, function, label in [
    ('product/project/input.rs', 'validate', 'project.validate'),
    ('product/project/input.rs', 'resolved_note_identities', 'project.resolved_identities'),
    ('product/project.rs', 'lower_product_document', 'project.lower_product'),
    ('product/project.rs', 'to_product_v3_payload', 'project.payload'),
    ('runtime/defaults.rs', 'load_embedded_writer_defaults', 'runtime.defaults'),
    ('writer_core/staging.rs', 'validate_normalized_ir', 'writer.validate'),
    ('writer_core/staging.rs', 'materialize_with_prepared_media', 'writer.staging'),
    ('authoring_core/normalize.rs', 'resolve_media_references', 'normalize.media_refs'),
]:
    state['scope']('anki_forge/src/' + file, function, label)
shutil.copy2(WORK / 'binaries/before', WORK / 'binaries/current')
