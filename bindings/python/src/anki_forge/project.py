from __future__ import annotations

import json
import os
import re
import shutil
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from types import MappingProxyType
from typing import Mapping, Sequence

from .diagnostics import Diagnostic, RuntimeInvocationError, ValidationError
from .media import MediaRegistry
from .note import Note
from .notetype import NoteType, _validate_non_empty, _validate_optional_non_empty
from .report import BuildReport
from .runtime import RuntimeOverride, resolve_runtime, run_product_build

STOCK_NOTE_TYPE_IDS = {"basic", "cloze"}
STOCK_FIELD_KEYS = {
    "basic": {"front", "back"},
    "cloze": {"text", "back_extra"},
}
ALLOWED_FAIL_ON = {"info", "low", "medium", "high", "critical"}
CLOZE_MARKER_PATTERN = re.compile(r"\{\{[cC][1-9][0-9]*::")
DIAGNOSTIC_DOMAIN_BY_PREFIX = {
    "AFID": "identity",
    "COMPARE": "comparison",
    "DECK": "deck",
    "MEDIA": "media",
    "NOTETYPE": "notetype",
    "TEMPLATE": "notetype",
    "PRODUCT": "product",
    "PROJECT": "project",
    "RISK": "risk",
    "UPDATE": "update_safety",
}
DIAGNOSTIC_STAGE_BY_PREFIX = {
    "AFID": "validate",
    "COMPARE": "compare",
    "DECK": "validate",
    "MEDIA": "normalize",
    "NOTETYPE": "validate",
    "TEMPLATE": "validate",
    "PRODUCT": "validate",
    "PROJECT": "build",
    "RISK": "risk",
    "UPDATE": "update_safety",
}


@dataclass
class Project:
    name: str
    stable_id: str | None = None
    default_deck: str | None = None
    media: MediaRegistry = field(default_factory=MediaRegistry)
    _note_types: dict[str, NoteType] = field(default_factory=dict)
    _note_type_order: list[str] = field(default_factory=list)
    _notes: list[Note] = field(default_factory=list)

    def __post_init__(self) -> None:
        self.name = _validate_non_empty(self.name, "project name")
        self.stable_id = _validate_optional_non_empty(self.stable_id, "stable id")
        self.default_deck = _validate_optional_non_empty(self.default_deck, "default deck")

    @property
    def notetypes(self) -> Mapping[str, NoteType]:
        return MappingProxyType(self._note_types)

    @property
    def notetype_order(self) -> Sequence[str]:
        return tuple(self._note_type_order)

    @property
    def notes(self) -> Sequence[Note]:
        return tuple(self._notes)

    def add_notetype(self, note_type: NoteType) -> Project:
        note_type.validate()
        if note_type.id in STOCK_NOTE_TYPE_IDS:
            raise ValidationError(f"custom note type id is reserved: {note_type.id}")
        if note_type.id in self._note_types:
            raise ValidationError(f"duplicate note type id: {note_type.id}")
        self._note_types[note_type.id] = note_type
        self._note_type_order.append(note_type.id)
        return self

    def add_note(self, note: Note) -> Project:
        if note.note_type_id not in STOCK_NOTE_TYPE_IDS and note.note_type_id not in self._note_types:
            raise ValidationError(f"unknown note type id: {note.note_type_id}")
        if note.note_type_id in self._note_types:
            self._validate_custom_note_field_keys(note, self._note_types[note.note_type_id])
        self._notes.append(note)
        return self

    def to_product_document(self) -> dict[str, object]:
        from .product_json import (
            basic_stock_notetype_json,
            cloze_stock_notetype_json,
            custom_notetype_json,
            media_to_json,
            note_to_json,
        )

        self._validate_notes_for_serialization()
        note_types: list[dict[str, object]] = []
        for note_type_id in self._stock_note_types():
            if note_type_id == "basic":
                note_types.append(basic_stock_notetype_json())
            elif note_type_id == "cloze":
                note_types.append(cloze_stock_notetype_json())
        note_types.extend(custom_notetype_json(self._note_types[note_type_id]) for note_type_id in self._note_type_order)

        return {
            "product_document_version": "product-v2",
            "document_id": self.stable_id or self.name,
            "default_deck_name": self.default_deck,
            "note_types": note_types,
            "notes": [note_to_json(note, index, self._resolve_deck(note)) for index, note in enumerate(self._notes)],
            "media": [media_to_json(item) for item in self.media.items],
        }

    def write_apkg(
        self,
        path: str | os.PathLike[str],
        *,
        compare_to: str | os.PathLike[str] | None = None,
        fail_on: str | None = None,
        report_json: str | os.PathLike[str] | None = None,
        runtime: RuntimeOverride | None = None,
    ) -> BuildReport:
        target_path = Path(path).resolve()
        compare_path = Path(compare_to).resolve() if compare_to is not None else None
        report_path = Path(report_json).resolve() if report_json is not None else None
        _validate_write_paths(target_path, compare_path, report_path, fail_on)
        resolved_runtime = resolve_runtime(runtime)

        try:
            with tempfile.TemporaryDirectory(prefix="anki-forge-product-") as temp_dir:
                product_input = Path(temp_dir) / "project.product-v2.json"
                report = self._cloze_marker_report()
                if report is not None:
                    if report_path is not None:
                        _write_report_json(report, report_path)
                    return report
                product_document = self._runtime_product_document(product_input.parent)
                product_input.write_text(
                    json.dumps(product_document, ensure_ascii=False),
                    encoding="utf-8",
                )
                report = run_product_build(
                    runtime=resolved_runtime,
                    product_input=product_input,
                    apkg_out=target_path,
                    compare_to=compare_path,
                    fail_on=fail_on,
                    report_json=report_path,
                )
        except OSError as error:
            raise RuntimeInvocationError(str(error), kind="setup_failed") from error

        return report

    def _runtime_product_document(self, runtime_dir: Path) -> dict[str, object]:
        product_document = self.to_product_document()
        media_entries = product_document.get("media")
        if not isinstance(media_entries, list):
            return product_document

        for item, entry in zip(self.media.items, media_entries, strict=True):
            if item.source_kind != "file" or item.path is None or not isinstance(entry, dict):
                continue
            source = entry.get("source")
            if not isinstance(source, dict):
                continue
            source["path"] = item.ref.export_as
            if item.path.is_file():
                shutil.copy2(item.path, runtime_dir / item.ref.export_as)
        return product_document

    def _cloze_marker_report(self) -> BuildReport | None:
        for index, note in enumerate(self._notes):
            if note.note_type_id != "cloze":
                continue
            text = note.fields.get("text")
            if text is None or text.value is None or CLOZE_MARKER_PATTERN.search(text.value):
                continue
            return BuildReport(
                status="invalid",
                comparison="not_requested",
                artifact=None,
                counts={"notes": 0, "cards": 0, "media": 0},
                media={
                    "objects": 0,
                    "bindings": 0,
                    "references": 0,
                    "missing_references": 0,
                    "unsafe_references": 0,
                    "unused_bindings": 0,
                    "unique_bytes": 0,
                },
                diagnostics=(
                    Diagnostic(
                        code="PRODUCT.CLOZE_MARKER_MISSING",
                        severity="error",
                        domain="product",
                        stage="validate",
                        message="cloze note text must contain at least one cloze marker",
                        path=f"project.notes[{index}].fields[\"text\"]",
                        suggested_fix="add a marker like {{c1::text}} to the cloze note text",
                    ),
                ),
            )
        return None

    def _stock_note_types(self) -> list[str]:
        used = {note.note_type_id for note in self._notes}
        return [note_type_id for note_type_id in ("basic", "cloze") if note_type_id in used]

    def _resolve_deck(self, note: Note) -> str:
        return note.deck_name or self.default_deck or self.name

    def _validate_notes_for_serialization(self) -> None:
        for note_type_id in self._note_type_order:
            self._note_types[note_type_id].validate()

        media_ids = {item.ref.media_id for item in self.media.items}
        seen_stable_ids: set[str] = set()
        for note in self._notes:
            if note.note_type_id not in STOCK_NOTE_TYPE_IDS and note.note_type_id not in self._note_types:
                raise ValidationError(f"unknown note type id: {note.note_type_id}")

            if note.stable_id is not None:
                if note.stable_id in seen_stable_ids:
                    raise ValidationError(f"duplicate note stable_id: {note.stable_id}")
                seen_stable_ids.add(note.stable_id)

            if note.note_type_id in self._note_types:
                note_type = self._note_types[note.note_type_id]
                self._validate_custom_note_field_keys(note, note_type)
                has_identity_fields = any(field.identity for field in note_type.fields)
                if note.stable_id is None and not has_identity_fields:
                    raise ValidationError(f"custom note type {note_type.id} needs identity fields or stable_id")
            else:
                self._validate_stock_note_field_keys(note)

            self._validate_note_media_references(note, media_ids)

    def _validate_stock_note_field_keys(self, note: Note) -> None:
        allowed = STOCK_FIELD_KEYS[note.note_type_id]
        for field_key in note.fields:
            if field_key not in allowed:
                raise ValidationError(f"unknown field key for {note.note_type_id}: {field_key}")

    def _validate_note_media_references(self, note: Note, media_ids: set[str]) -> None:
        for content in note.fields.values():
            if content.kind in {"sound", "image"} and content.media_id not in media_ids:
                raise ValidationError(f"unknown media id: {content.media_id}")

    def _validate_custom_note_field_keys(self, note: Note, note_type: NoteType) -> None:
        allowed = {field.key for field in note_type.fields}
        for field_key in note.fields:
            if field_key not in allowed:
                raise ValidationError(f"unknown field key for {note_type.id}: {field_key}")


def _validate_write_paths(
    target: Path,
    compare_to: Path | None,
    report_json: Path | None,
    fail_on: str | None,
) -> None:
    if fail_on is not None and fail_on not in ALLOWED_FAIL_ON:
        raise ValidationError(f"unknown fail_on level: {fail_on}")
    if fail_on is not None and compare_to is None:
        raise ValidationError("fail_on requires compare_to")
    if compare_to is not None and target == compare_to:
        raise ValidationError("apkg output path must differ from compare_to")
    if report_json is not None and target == report_json:
        raise ValidationError("apkg output path must differ from report_json")
    if compare_to is not None and report_json is not None and compare_to == report_json:
        raise ValidationError("compare_to path must differ from report_json")


def _write_report_json(report: BuildReport, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(_report_to_json(report), ensure_ascii=False, indent=2), encoding="utf-8")


def _diagnostic_prefix(code: str) -> str | None:
    prefix, separator, _ = code.partition(".")
    if not separator:
        return None
    return prefix


def _inferred_diagnostic_domain(code: str) -> str:
    prefix = _diagnostic_prefix(code)
    return DIAGNOSTIC_DOMAIN_BY_PREFIX.get(prefix, "unknown")


def _inferred_diagnostic_stage(code: str) -> str:
    prefix = _diagnostic_prefix(code)
    return DIAGNOSTIC_STAGE_BY_PREFIX.get(prefix, "unknown")


def _report_to_json(report: BuildReport) -> dict[str, object]:
    return {
        "kind": "anki-forge-build-report",
        "schema_version": "phase4-build-report-v2",
        "tool_version": "anki-forge-python",
        "status": report.status,
        "comparison": report.comparison,
        "artifact": report.artifact,
        "counts": dict(report.counts),
        "media": dict(report.media),
        "diagnostics": [
            {
                "code": diagnostic.code,
                "severity": diagnostic.severity,
                "domain": diagnostic.domain or _inferred_diagnostic_domain(diagnostic.code),
                "stage": diagnostic.stage or _inferred_diagnostic_stage(diagnostic.code),
                "path": diagnostic.path,
                "message": diagnostic.message,
                "suggested_fix": diagnostic.suggested_fix,
            }
            for diagnostic in report.diagnostics
        ],
        "metrics": {"duration_ms": 0},
        "policy": {
            "status": "not_evaluated",
            "threshold": None,
            "highest_risk": None,
            "blocking_findings": [],
        },
        "inspect": report.inspect,
        "previous_inspect": report.previous_inspect,
        "update_safety": report.update_safety,
        "diff": report.diff,
        "risk": report.risk,
    }
