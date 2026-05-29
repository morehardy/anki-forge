from __future__ import annotations

import re
import string
from dataclasses import dataclass, field
from typing import Iterable

from .diagnostics import ValidationError

_ASCII_CONTROL = "".join(chr(value) for value in range(32)) + chr(127)
_ASCII_WHITESPACE = "".join(chr(value) for value in range(128) if chr(value) in string.whitespace)
_SLUG_PATTERN = re.compile(r"[^a-z0-9]+")


def _has_control(value: str) -> bool:
    return any(char in _ASCII_CONTROL for char in value)


def _reject_ascii_control(value: str, label: str) -> None:
    if _has_control(value):
        raise ValidationError(f"{label} must not contain ASCII control characters")


def _validate_non_empty(value: str, label: str) -> str:
    _reject_ascii_control(value, label)
    stripped = value.strip(_ASCII_WHITESPACE)
    if not stripped:
        raise ValidationError(f"{label} must not be empty")
    return stripped


def _validate_optional_non_empty(value: str | None, label: str) -> str | None:
    if value is None:
        return None
    return _validate_non_empty(value, label)


def _validate_id(value: str, label: str = "id") -> str:
    return _validate_non_empty(value, label)


def _validate_tag(value: str) -> str:
    normalized = _validate_non_empty(value, "tag")
    if any(char in _ASCII_WHITESPACE for char in normalized):
        raise ValidationError("tag must not contain whitespace")
    return normalized


def _slug(value: str) -> str:
    normalized = _validate_non_empty(value, "name").lower()
    slug = _SLUG_PATTERN.sub("_", normalized).strip("_")
    if not slug:
        raise ValidationError("name cannot be converted to an ASCII key; pass an explicit ASCII key")
    return slug


def _generation_rule_field_keys(rule: GenerationRule | None) -> tuple[str, ...]:
    if rule is None or rule.kind == "anki_default":
        return ()
    if rule.kind in {"all", "any"}:
        return rule.fields
    if rule.kind == "cloze":
        if rule.field is None:
            raise ValidationError("cloze generation rule requires a field")
        return (rule.field,)
    raise ValidationError(f"unknown generation rule kind: {rule.kind}")


@dataclass(frozen=True)
class GenerationRule:
    kind: str
    fields: tuple[str, ...] = ()
    field: str | None = None

    def __post_init__(self) -> None:
        kind = _validate_non_empty(self.kind, "generation rule kind")
        if kind not in {"anki_default", "all", "any", "cloze"}:
            raise ValidationError(f"unknown generation rule kind: {kind}")

        fields = tuple(_validate_id(field_key, "field key") for field_key in self.fields)
        field = _validate_id(self.field, "field") if self.field is not None else None

        if kind in {"all", "any"}:
            if not fields:
                raise ValidationError(f"{kind} generation rule requires at least one field key")
            if field is not None:
                raise ValidationError(f"{kind} generation rule must not set field")
        elif kind == "cloze":
            if fields:
                raise ValidationError("cloze generation rule must not set fields")
            if field is None:
                raise ValidationError("cloze generation rule requires a field")
        elif fields or field is not None:
            raise ValidationError("anki_default generation rule must not set fields or field")

        object.__setattr__(self, "kind", kind)
        object.__setattr__(self, "fields", fields)
        object.__setattr__(self, "field", field)

    @classmethod
    def anki_default(cls) -> GenerationRule:
        return cls(kind="anki_default")

    @classmethod
    def all(cls, field_keys: Iterable[str]) -> GenerationRule:
        fields = tuple(_validate_id(field_key, "field key") for field_key in field_keys)
        if not fields:
            raise ValidationError("all generation rule requires at least one field key")
        return cls(kind="all", fields=fields)

    @classmethod
    def any(cls, field_keys: Iterable[str]) -> GenerationRule:
        fields = tuple(_validate_id(field_key, "field key") for field_key in field_keys)
        if not fields:
            raise ValidationError("any generation rule requires at least one field key")
        return cls(kind="any", fields=fields)

    @classmethod
    def cloze(cls, field: str) -> GenerationRule:
        return cls(kind="cloze", field=_validate_id(field, "field"))


@dataclass(frozen=True)
class Field:
    name: str
    key: str | None = None
    identity: bool = False
    sort: bool = False
    required: bool = False

    def __post_init__(self) -> None:
        name = _validate_non_empty(self.name, "field name")
        key = _validate_id(self.key, "field key") if self.key is not None else _slug(name)
        object.__setattr__(self, "name", name)
        object.__setattr__(self, "key", key)


@dataclass(frozen=True)
class Template:
    name: str
    front: str
    back: str
    key: str | None = None
    generate_when: GenerationRule | None = None

    def __post_init__(self) -> None:
        name = _validate_non_empty(self.name, "template name")
        key = _validate_id(self.key, "template key") if self.key is not None else _slug(name)
        front = _validate_non_empty(self.front, "template front")
        back = _validate_non_empty(self.back, "template back")
        generate_when = self.generate_when or GenerationRule.anki_default()
        if not isinstance(generate_when, GenerationRule):
            raise ValidationError("template generate_when must be a GenerationRule")
        object.__setattr__(self, "name", name)
        object.__setattr__(self, "key", key)
        object.__setattr__(self, "front", front)
        object.__setattr__(self, "back", back)
        object.__setattr__(self, "generate_when", generate_when)


@dataclass
class NoteType:
    id: str
    name: str | None = None
    fields: list[Field] = field(default_factory=list)
    templates: list[Template] = field(default_factory=list)
    css_value: str | None = None
    custom_value: bool = True

    def __setattr__(self, name: str, value: object) -> None:
        if name == "id" and "id" in self.__dict__:
            raise AttributeError("note type id is immutable")
        super().__setattr__(name, value)

    def __post_init__(self) -> None:
        note_type_id = _validate_id(self.id, "note type id")
        object.__setattr__(self, "id", note_type_id)
        object.__setattr__(self, "name", _validate_optional_non_empty(self.name, "note type name") or note_type_id)
        if self.css_value is not None:
            _reject_ascii_control(self.css_value, "css")

    @classmethod
    def custom(cls, note_type_id: str, name: str | None = None, css: str | None = None) -> NoteType:
        return cls(note_type_id, name=name, css_value=css, custom_value=True)

    def css(self, css: str | None) -> NoteType:
        """Set CSS for this note type, or clear CSS when passed None."""
        if css is None:
            self.css_value = None
        else:
            _reject_ascii_control(css, "css")
            self.css_value = css
        return self

    def field(self, field: Field) -> NoteType:
        if any(existing.key == field.key for existing in self.fields):
            raise ValidationError(f"duplicate field key: {field.key}")
        if any(existing.name == field.name for existing in self.fields):
            raise ValidationError(f"duplicate field name: {field.name}")
        if field.sort and any(existing.sort for existing in self.fields):
            raise ValidationError("only one sort field is allowed")
        self.fields.append(field)
        return self

    def template(self, template: Template) -> NoteType:
        if any(existing.key == template.key for existing in self.templates):
            raise ValidationError(f"duplicate template key: {template.key}")
        field_keys = {field.key for field in self.fields}
        for field_key in _generation_rule_field_keys(template.generate_when):
            if field_key not in field_keys:
                raise ValidationError(f"template generation rule references unknown field key: {field_key}")
        self.templates.append(template)
        return self

    def validate(self) -> NoteType:
        seen_field_keys: set[str] = set()
        seen_field_names: set[str] = set()
        sort_count = 0
        for current_field in self.fields:
            if current_field.key in seen_field_keys:
                raise ValidationError(f"duplicate field key: {current_field.key}")
            if current_field.name in seen_field_names:
                raise ValidationError(f"duplicate field name: {current_field.name}")
            seen_field_keys.add(current_field.key)
            seen_field_names.add(current_field.name)
            if current_field.sort:
                sort_count += 1
        if sort_count > 1:
            raise ValidationError("only one sort field is allowed")

        seen_template_keys: set[str] = set()
        for current_template in self.templates:
            if current_template.key in seen_template_keys:
                raise ValidationError(f"duplicate template key: {current_template.key}")
            seen_template_keys.add(current_template.key)
            for field_key in _generation_rule_field_keys(current_template.generate_when):
                if field_key not in seen_field_keys:
                    raise ValidationError(f"template generation rule references unknown field key: {field_key}")
        return self
