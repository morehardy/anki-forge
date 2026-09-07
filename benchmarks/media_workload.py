"""Versioned synthetic PNG/WAV workloads, generated outside measured commands.

v2 is a new deterministic recipe. Historical v1 inputs remain usable through
media_bench --inputs; their hashes are never replaced by this generator.
"""
import hashlib
import json
import struct
import zlib
from pathlib import Path

import workload

PROFILES = (workload.PROFILE, "basic-image-unique-v2", "basic-audio-unique-v2",
            "basic-mixed-unique-v2", "basic-mixed-shared-v2")


def png(index):
    width, height = 144, 148
    pixels = hashlib.shake_256(f"anki-forge-image-v2:{index}".encode()).digest(width * height * 3)
    rows = b"".join(b"\0" + pixels[row * width * 3:(row + 1) * width * 3] for row in range(height))

    def chunk(kind, payload):
        return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload))

    # Stored DEFLATE blocks make exact bytes independent of zlib versions.
    blocks = []
    for start in range(0, len(rows), 65535):
        data = rows[start:start + 65535]
        blocks.append(bytes([int(start + len(data) == len(rows))]) +
                      struct.pack("<HH", len(data), len(data) ^ 0xffff) + data)
    compressed = b"\x78\x01" + b"".join(blocks) + struct.pack(">I", zlib.adler32(rows))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)) +
            chunk(b"IDAT", compressed) + chunk(b"IEND", b""))


def wav(index):
    # A one-second integer triangle waveform plus quiet deterministic dither.
    # No platform-dependent floating point or external encoder is involved.
    samples = bytearray()
    period = 41 + index % 173
    for position in range(16000):
        phase = (position + index) % period
        triangle = 16000 - (64000 * abs(phase * 2 - period) // (period * 2))
        dither = int.from_bytes(hashlib.sha256(struct.pack("<II", index, position)).digest()[:2], "little") % 1024 - 512
        samples.extend(struct.pack("<h", triangle + dither))
    return (b"RIFF" + struct.pack("<I", 36 + len(samples)) + b"WAVEfmt " +
            struct.pack("<IHHIIHH", 16, 1, 1, 16000, 32000, 2, 16) +
            b"data" + struct.pack("<I", len(samples)) + samples)


def generate(destination, *, freeze=False):
    destination = Path(destination)
    notes = workload.corpus()
    hashes = {}
    payloads = {}
    for profile in PROFILES:
        inputs = destination / profile / "inputs"
        assets = inputs / "media"
        assets.mkdir(parents=True, exist_ok=True)
        for size in workload.SIZES:
            document = workload.document(notes, size)
            if profile != workload.PROFILE:
                document["schema"] = "basic-media-apkg-v1"
                document["profile"] = profile
                document["notes"] = [dict(note) for note in document["notes"]]
                media = {}
                for index, note in enumerate(document["notes"]):
                    if "image-unique" in profile:
                        kind = "image"
                    elif "audio-unique" in profile:
                        kind = "audio"
                    else:
                        kind = "text" if index % 10 < 3 else "image" if index % 10 < 7 else "audio"
                    if kind == "text":
                        continue
                    ordinal = index % 70 if "shared" in profile else index
                    identity = f"{kind}-{ordinal:04d}"
                    filename = identity + (".png" if kind == "image" else ".wav")
                    if identity not in media:
                        if identity not in payloads:
                            payloads[identity] = png(ordinal) if kind == "image" else wav(ordinal)
                        payload = payloads[identity]
                        path = assets / filename
                        if not path.exists():
                            path.write_bytes(payload)
                        if path.read_bytes() != payload:
                            raise ValueError(f"existing fixture changed: {path}")
                        media[identity] = {"id": identity, "kind": kind, "filename": filename,
                                           "path": "media/" + filename, "bytes": len(payload),
                                           "sha256": hashlib.sha256(payload).hexdigest()}
                    note["front_media" if kind == "image" else "back_media"] = [identity]
                document["media"] = list(media.values())
            raw = workload.serialize(document)
            (inputs / f"{size}.json").write_bytes(raw)
            hashes[f"{profile}/{size}"] = hashlib.sha256(raw).hexdigest()
    golden = Path(__file__).with_name("media-workload-v2-golden.json")
    if freeze:
        if golden.exists():
            raise ValueError("golden hashes are already frozen")
        golden.write_text(json.dumps(hashes, indent=2) + "\n")
    if json.loads(golden.read_text()) != hashes:
        raise ValueError("media v2 workload differs from frozen hashes")
    return hashes
