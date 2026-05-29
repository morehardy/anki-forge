from __future__ import annotations

import base64
import copy
import json

from .diagnostics import ValidationError
from .media import MediaItem
from .note import FieldContent, Note
from .notetype import Field, GenerationRule, NoteType, Template


def _path_key(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def _field_source_path(note_type_id: str, field_key: str) -> str:
    return f"project.note_types[{_path_key(note_type_id)}].fields[{_path_key(field_key)}]"


def _template_source_path(note_type_id: str, template_key: str) -> str:
    return f"project.note_types[{_path_key(note_type_id)}].templates[{_path_key(template_key)}]"


def _note_type_source_path(note_type_id: str) -> str:
    return f"project.note_types[{_path_key(note_type_id)}]"


_BASIC_STOCK_NOTETYPE: dict[str, object] = {
    "kind": "stock",
    "id": "basic",
    "name": "Basic",
    "fields": [
        {
            "name": "Front",
            "key": "front",
            "identity": False,
            "sort": True,
            "required": True,
            "source_path": 'project.note_types["basic"].fields["front"]',
        },
        {
            "name": "Back",
            "key": "back",
            "identity": False,
            "sort": False,
            "required": False,
            "source_path": 'project.note_types["basic"].fields["back"]',
        },
    ],
    "templates": [
        {
            "name": "Card 1",
            "key": "card_1",
            "front": "{{Front}}",
            "back": "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}",
            "generation_rule": {"kind": "anki_default"},
            "source_path": 'project.note_types["basic"].templates["card_1"]',
        }
    ],
    "css": None,
    "source_path": 'project.note_types["basic"]',
}

_CLOZE_STOCK_NOTETYPE: dict[str, object] = {
    "kind": "stock",
    "id": "cloze",
    "name": "Cloze",
    "fields": [
        {
            "name": "Text",
            "key": "text",
            "identity": False,
            "sort": True,
            "required": True,
            "source_path": 'project.note_types["cloze"].fields["text"]',
        },
        {
            "name": "Back Extra",
            "key": "back_extra",
            "identity": False,
            "sort": False,
            "required": False,
            "source_path": 'project.note_types["cloze"].fields["back_extra"]',
        },
    ],
    "templates": [
        {
            "name": "Cloze",
            "key": "cloze",
            "front": "{{cloze:Text}}",
            "back": "{{cloze:Text}}<br>\n{{Back Extra}}",
            "generation_rule": {"kind": "cloze", "field": "text"},
            "source_path": 'project.note_types["cloze"].templates["cloze"]',
        }
    ],
    "css": None,
    "source_path": 'project.note_types["cloze"]',
}


def basic_stock_notetype_json() -> dict[str, object]:
    return copy.deepcopy(_BASIC_STOCK_NOTETYPE)


def cloze_stock_notetype_json() -> dict[str, object]:
    return copy.deepcopy(_CLOZE_STOCK_NOTETYPE)


def generation_rule_to_json(rule: GenerationRule) -> dict[str, object]:
    if rule.kind in {"all", "any"}:
        return {"kind": rule.kind, "fields": list(rule.fields)}
    if rule.kind == "cloze":
        if rule.field is None:
            raise ValidationError("cloze generation rule requires a field")
        return {"kind": "cloze", "field": rule.field}
    if rule.kind == "anki_default":
        return {"kind": "anki_default"}
    raise ValidationError(f"unknown generation rule kind: {rule.kind}")


def field_to_json(note_type_id: str, field: Field) -> dict[str, object]:
    if field.key is None:
        raise ValidationError("field key must not be None after validation")
    return {
        "name": field.name,
        "key": field.key,
        "identity": field.identity,
        "sort": field.sort,
        "required": field.required,
        "source_path": _field_source_path(note_type_id, field.key),
    }


def template_to_json(note_type_id: str, template: Template) -> dict[str, object]:
    if template.key is None:
        raise ValidationError("template key must not be None after validation")
    if template.generate_when is None:
        raise ValidationError("template generation rule must not be None after validation")
    return {
        "name": template.name,
        "key": template.key,
        "front": template.front,
        "back": template.back,
        "generation_rule": generation_rule_to_json(template.generate_when),
        "source_path": _template_source_path(note_type_id, template.key),
    }


def custom_notetype_json(note_type: NoteType) -> dict[str, object]:
    fields = [field_to_json(note_type.id, field) for field in note_type.fields]
    templates = [template_to_json(note_type.id, template) for template in note_type.templates]
    result: dict[str, object] = {
        "kind": "custom",
        "id": note_type.id,
        "name": note_type.name,
        "fields": fields,
        "templates": templates,
    }
    identity_fields: list[str] = []
    for field in note_type.fields:
        if not field.identity:
            continue
        if field.key is None:
            raise ValidationError("identity field key must not be None after validation")
        identity_fields.append(field.key)
    if identity_fields:
        result["identity"] = {"kind": "fields", "fields": identity_fields}
    result["css"] = note_type.css_value
    result["source_path"] = _note_type_source_path(note_type.id)
    return result


def field_content_to_json(content: FieldContent) -> dict[str, object]:
    if content.kind in {"text", "html"}:
        if content.value is None:
            raise ValidationError(f"{content.kind} field content requires a value")
        return {"kind": content.kind, "value": content.value}
    if content.kind in {"sound", "image"}:
        if content.media_id is None:
            raise ValidationError(f"{content.kind} field content requires a media id")
        return {"kind": content.kind, "media_id": content.media_id}
    raise ValidationError(f"unsupported field content kind: {content.kind}")


def note_to_json(note: Note, index: int, deck_name: str) -> dict[str, object]:
    result: dict[str, object] = {
        "kind": "stock" if note.note_type_id in {"basic", "cloze"} else "custom",
        "note_type_id": note.note_type_id,
    }
    if note.stable_id is not None:
        result["stable_id"] = note.stable_id
    result["deck_name"] = deck_name
    result["fields"] = {key: field_content_to_json(content) for key, content in note.fields.items()}
    result["tags"] = list(note.tag_values)
    if note.stable_id is not None:
        result["source_path"] = f"project.notes[{_path_key(note.stable_id)}]"
    else:
        result["source_path"] = f"project.notes[{index}]"
    return result


def media_to_json(item: MediaItem) -> dict[str, object]:
    if item.source_kind == "file":
        if item.path is None:
            raise ValidationError("file media requires a path")
        source = {"kind": "file", "path": str(item.path)}
    elif item.data is not None:
        source = {
            "kind": "inline_base64",
            "source_label": item.source_label,
            "data_base64": base64.b64encode(item.data).decode("ascii"),
        }
    else:
        raise ValidationError(f"unsupported media source kind: {item.source_kind}")

    return {
        "id": item.ref.media_id,
        "source": source,
        "export_as": item.ref.export_as,
        "source_path": f"project.media[{_path_key(item.ref.export_as)}]",
    }
