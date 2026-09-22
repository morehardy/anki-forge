from anki_forge import Note, Project
import pytest
from anki_forge import TemplateBundleError
from test_native_parity import observe


def write_bundle(root):
    root.mkdir()
    (root / "assets").mkdir()
    (root / "anki-template.yaml").write_bytes(b"""format_version: template-bundle-v1
note_type:
  id: bundle-card
  name: Bundle Card
  fields:
    - {key: prompt, name: Prompt, identity: true, required: true}
    - {key: extra, name: Extra, optional: true}
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
      browser_front_file: browser.html
      target_deck: Bundle::Cards
      generation_rule: {kind: all, fields: [prompt]}
css_file: style.css
assets:
  - {path: assets/icon.svg, export_as: icon.svg}
  - {path: assets/font.woff2, export_as: font.woff2}
""")
    (root / "front.html").write_bytes(' \r\n<section>{{Prompt}}</section>\n'.encode())
    (root / "back.html").write_bytes('{{Prompt}}<hr>{{Extra}}<img src="icon.svg">'.encode())
    (root / "browser.html").write_bytes(b"{{Prompt}}")
    (root / "style.css").write_bytes(b"\n@font-face { font-family: Bundle; src: url(font.woff2); }\n.card { background-image: url(icon.svg); }\n")
    (root / "assets/icon.svg").write_bytes(b'<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>')
    # Opaque font payload: the package pipeline preserves assets, not font rendering.
    (root / "assets/font.woff2").write_bytes(b"wOF2" + bytes(64))


def test_template_bundle_matches_independent_rust_authoring(tmp_path, monkeypatch):
    bundle = tmp_path / "bundle 中文"
    write_bundle(bundle)
    project = Project("Bundle", stable_id="native-bundle", default_deck="Bundle", base_dir=tmp_path)
    monkeypatch.chdir(bundle)
    project.import_template_bundle("bundle 中文")
    project.add_note(Note("bundle-card").text("prompt", "中 & prompt").text("extra", "Extra"))
    assert project.notetypes["bundle-card"].fields[1].optional
    report = project.build()
    report.ensure_success()
    assert report.counts["media"] == 2
    assert observe("inspect", report.artifact.path) == observe("bundle", tmp_path / "rust.apkg", bundle)


def test_invalid_template_preserves_utf8_source_location_and_allows_retry(tmp_path):
    bundle = tmp_path / "bundle"
    write_bundle(bundle)
    (bundle / "front.html").write_bytes("中{{Missing}}".encode())
    project = Project("Bundle", base_dir=tmp_path)
    with pytest.raises(TemplateBundleError) as caught:
        project.import_template_bundle("bundle")
    assert caught.value.code == "TEMPLATE.RENDER_FIELD_UNKNOWN"
    assert caught.value.path == str((bundle / "front.html").resolve())
    assert caught.value.byte_offset == 3
    assert "bundle-card" not in project.notetypes
    (bundle / "front.html").write_bytes(b"{{Prompt}}")
    project.import_template_bundle("bundle")
    project.add_note(Note("bundle-card").text("prompt", "repaired"))
    project.build().ensure_success()


def test_asset_failure_rolls_back_note_type_and_preceding_media(tmp_path):
    bundle = tmp_path / "bundle"
    write_bundle(bundle)
    project = Project("Atomic", base_dir=tmp_path)
    project.media.add_bytes(source_label="existing", data=b"different font", export_as="font.woff2")
    with pytest.raises(TemplateBundleError) as caught:
        project.import_template_bundle(bundle)
    assert caught.value.code == "TEMPLATE.BUNDLE_ASSET_INVALID"
    assert "bundle-card" not in project.notetypes
    # icon.svg was staged before the conflicting font; it must not remain registered.
    project.media.add_bytes(source_label="replacement", data=b"different icon", export_as="icon.svg")
    project.add_note(Note.basic("still usable", "answer"))
    project.build().ensure_success()
