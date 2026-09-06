"""Frozen project-owned synthetic corpus; selection uses SHA-256, not a PRNG."""
import hashlib
import json
import statistics
import unicodedata
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SIZES = (200, 500, 1000, 10000)
SEED = 20260906
PROFILE = "basic-mixed-text-v1"
QFMT = "{{Front}}"
AFMT = "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
# Original phrases written for this repository; no scraped or third-party deck data.
ENGLISH = (
    "A quiet garden records the changing seasons.",
    "Describe how sunlight reaches the river valley.",
    "Careful observation connects a question to evidence.",
    "A small workshop keeps its tools beside the window.",
    "The morning train carries letters across the hills.",
    "Compare the two examples and explain your reasoning.",
    "The map marks a narrow path through the old forest.",
    "Practice recalling the idea before reading the answer.",
    "A blue notebook contains sketches of distant islands.",
    "Every measurement needs a clearly stated unit.",
    "Write a short explanation using familiar words.",
    "The sequence preserves the order of each observation.",
    "A patient reader notices details in the final sentence.",
    "Imagine a different example with the same relationship.",
    "Fresh water flows beneath the stone bridge in spring.",
    "The next exercise asks for a concrete comparison.",
)
MIXED = (
    "观察河流与山谷的关系，再用 English 描述一个例子。",
    "把问题写在笔记中，compare 两种解释需要哪些证据。",
    "清晨的花园很安静，remember 光线与季节的变化。",
    "每次练习之后，record 一个能够帮助理解的新细节。",
    "沿着地图中的小路前进，describe 周围的树木和石桥。",
    "用熟悉的词语解释规律，check 每个测量值使用的单位。",
    "书架旁放着蓝色笔记本，review 昨天记录的观察结果。",
    "先回忆概念再阅读答案，connect 文字与具体的场景。",
    "不同例子可以共享同一结构，explain 它们之间的联系。",
    "在一段简短的说明中，include 条件、过程和最终结果。",
    "雨后的森林有新的颜色，notice 远处小屋窗边的工具。",
    "保持每条记录的顺序，compare 已知事实与新的问题。",
)
ESCAPING = ' 字面 <b>literal</b> &amp; < & > "quote" \'single\' '


def pick(pool, index, field, step):
    digest = hashlib.sha256(f"{SEED}:{index}:{field}:{step}".encode("ascii")).digest()
    return pool[int.from_bytes(digest[:8], "big") % len(pool)]


def field_text(index, field, category):
    digest = hashlib.sha256(f"{SEED}:{index}:{field}:length".encode()).digest()
    low, high = (40, 100) if field == "front" else (120, 300)
    target = low + int.from_bytes(digest[:4], "big") % (high - low + 1)
    prefix = f"[BF-{index + 1:05d}] " if field == "front" else ""
    if category == "escaping":
        prefix += ESCAPING
        target = max(target, len(prefix) + 10)
    pool = ENGLISH if category == "english" else MIXED
    content = prefix
    step = 0
    while len(content) < target:
        content += pick(pool, index, field, step) + " "
        step += 1
    return content[:target]


def corpus():
    assert all(unicodedata.normalize("NFC", s) == s for s in ENGLISH + MIXED + (ESCAPING,))
    return [{
        "id": f"BF-{i + 1:05d}",
        "category": "english" if i % 20 < 10 else "mixed" if i % 20 < 18 else "escaping",
        "front": field_text(i, "front", "english" if i % 20 < 10 else "mixed" if i % 20 < 18 else "escaping"),
        "back": field_text(i, "back", "english" if i % 20 < 10 else "mixed" if i % 20 < 18 else "escaping"),
    } for i in range(10000)]


def document(notes, count):
    return {"schema": "basic-apkg-v1", "profile": PROFILE, "seed": SEED,
            "deck_name": "Basic Benchmark", "genanki_deck_id": 1607392319,
            "note_count": count, "notes": notes[:count]}


def serialize(value):
    return (json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n").encode("utf-8")


def digest_bytes(value):
    return hashlib.sha256(value).hexdigest()


def distribution(values):
    return {"min": min(values), "median": statistics.median(values), "max": max(values),
            "mean": statistics.mean(values)}


def characterize(doc, raw):
    notes = doc["notes"]
    return {
        "sha256": digest_bytes(raw), "input_bytes": len(raw), "notes": len(notes),
        "categories": {c: sum(n["category"] == c for n in notes) for c in ("english", "mixed", "escaping")},
        "fields": {field: {"codepoints": distribution([len(n[field]) for n in notes]),
                           "utf8_bytes": distribution([len(n[field].encode()) for n in notes])}
                   for field in ("front", "back")},
        "compression_diagnostic": {"algorithm": "zlib", "version": zlib.ZLIB_RUNTIME_VERSION,
                                   "level": 9, "compressed_raw_ratio": len(zlib.compress(raw, 9)) / len(raw)},
    }


def generate(*, freeze=False):
    notes = corpus()
    target = ROOT / "inputs"
    target.mkdir(exist_ok=True)
    evidence = {"profile": PROFILE, "seed": SEED, "pool_provenance": "Original project-owned synthetic phrases",
                "serialization": "UTF-8, LF, no BOM, compact insertion-ordered JSON; SHA-256 selection",
                "qfmt": QFMT, "afmt": AFMT, "cases": {}}
    hashes = {}
    for count in SIZES:
        doc = document(notes, count)
        raw = serialize(doc)
        hashes[str(count)] = digest_bytes(raw)
        evidence["cases"][str(count)] = characterize(doc, raw)
        path = target / f"basic-{count}.json"
        if not path.exists() or path.read_bytes() != raw:
            path.write_bytes(raw)
    golden = ROOT / "workload-golden.json"
    if freeze:
        if golden.exists():
            raise ValueError("golden hashes already frozen; version the profile before changing it")
        golden.write_text(json.dumps(hashes, indent=2) + "\n")
    if json.loads(golden.read_text()) != hashes:
        raise ValueError("fixture differs from frozen golden hashes")
    return evidence
