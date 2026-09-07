"""Exact media verification layered over the original canonical-row checks."""
import hashlib
import html
import json
import zipfile

from verify import InvalidArtifact, MAX_BYTES, require, zstd_decode


def _varint(data, offset):
    value = 0
    for shift in range(0, 70, 7):
        require(offset < len(data), "truncated media protobuf")
        byte = data[offset]
        offset += 1
        value |= (byte & 127) << shift
        if byte < 128:
            return value, offset
    raise InvalidArtifact("oversized media protobuf varint")


def _fields(data):
    offset = 0
    while offset < len(data):
        tag, offset = _varint(data, offset)
        number, wire = tag >> 3, tag & 7
        require(number > 0, "invalid protobuf field")
        if wire == 0:
            value, offset = _varint(data, offset)
        elif wire == 2:
            length, offset = _varint(data, offset)
            require(length <= len(data) - offset, "truncated media protobuf field")
            value = data[offset:offset + length]
            offset += length
        else:
            raise InvalidArtifact("unsupported media protobuf wire type")
        yield number, value


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate media map field")
        result[key] = value
    return result


def media_map(raw, modern):
    if not modern:
        mapping = json.loads(raw, object_pairs_hook=unique_pairs)
        require(isinstance(mapping, dict), "media map must be an object")
        return {key: {"filename": value} for key, value in mapping.items()}
    result = {}
    for index, (number, item) in enumerate(_fields(zstd_decode(raw))):
        require(number == 1 and isinstance(item, bytes), "invalid MediaEntries")
        fields = unique_pairs(_fields(item))
        name, size, sha1 = fields.get(1), fields.get(2, 0), fields.get(3)
        require(isinstance(name, bytes) and isinstance(size, int) and isinstance(sha1, bytes), "invalid MediaEntry")
        result[str(index)] = {"filename": name.decode("utf-8"), "size": size, "sha1": sha1.hex()}
    return result


def check_media(path, expected):
    media = {item["filename"]: item for item in expected.get("media", [])}
    require(len(media) == len(expected.get("media", [])), "duplicate fixture media filename")
    with zipfile.ZipFile(path) as archive:
        entries = archive.infolist()
        names = [entry.filename for entry in entries]
        require(len(names) == len(set(names)), "duplicate archive entry")
        require(len(names) <= len(media) + 4, "unexpected archive entries")
        require(sum(entry.file_size for entry in entries) <= MAX_BYTES, "archive expansion budget")
        modern = "meta" in names and archive.read("meta") == b"\x08\x03"
        mapping = media_map(archive.read("media"), modern)
        require(len(mapping) == len(media), "media count mismatch")
        require({item["filename"] for item in mapping.values()} == set(media), "media filenames mismatch")
        require(set(mapping).isdisjoint({"meta", "media", "collection.anki2", "collection.anki21", "collection.anki21b"}), "media key overlaps metadata")
        require(all(key.isdigit() and str(int(key)) == key for key in mapping), "noncanonical media entry key")
        metadata = {"meta", "media", "collection.anki2", "collection.anki21", "collection.anki21b"}
        require(set(names) - metadata == set(mapping), "unaccounted archive payload")
        decoded_total = 0
        for key, info in mapping.items():
            raw = archive.read(key)
            payload = zstd_decode(raw) if modern else raw
            decoded_total += len(payload)
            require(decoded_total <= MAX_BYTES, "decoded media expansion budget")
            fixture = media[info["filename"]]
            require(hashlib.sha256(payload).hexdigest() == fixture["sha256"], "media payload hash mismatch")
            if modern:
                require(len(payload) == info["size"], "media size mismatch")
                require(hashlib.sha1(payload).hexdigest() == info["sha1"], "media SHA-1 mismatch")
        return len(mapping)


def field_suffix(note, field, expected):
    media = {item["id"]: item for item in expected.get("media", [])}
    result = ""
    for identity in note.get(field + "_media", []):
        item = media[identity]
        if item["kind"] == "image":
            result += '\n<img src="' + html.escape(item["filename"], quote=True) + '">'
        elif item["kind"] == "audio":
            result += '\n[sound:' + item["filename"] + ']'
        else:
            raise InvalidArtifact("unsupported fixture media kind")
    return result
