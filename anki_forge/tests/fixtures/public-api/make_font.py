"""Recreate the original MIT-licensed test font with fontTools (test tooling only)."""

from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont


def outline(points):
    pen = TTGlyphPen(None)
    if points:
        pen.moveTo(points[0])
        for point in points[1:]:
            pen.lineTo(point)
        pen.closePath()
    return pen.glyph()


root = Path(__file__).parent
font = FontBuilder(1000, isTTF=True)
font.setupGlyphOrder([".notdef", "space", "A"])
font.setupCharacterMap({32: "space", 65: "A"})
font.setupGlyf({
    ".notdef": outline([(80, 0), (80, 700), (520, 700), (520, 0)]),
    "space": outline([]),
    "A": outline([(50, 0), (230, 700), (370, 700), (550, 0), (410, 0), (300, 480), (190, 0)]),
})
font.setupHorizontalMetrics({".notdef": (600, 80), "space": (300, 0), "A": (600, 50)})
font.setupHorizontalHeader(ascent=800, descent=-200)
font.setupNameTable({
    "familyName": "AnkiForge Fixture",
    "styleName": "Regular",
    "uniqueFontIdentifier": "AnkiForgeFixture-Regular-1.0",
    "fullName": "AnkiForge Fixture Regular",
    "psName": "AnkiForgeFixture-Regular",
    "version": "Version 1.0",
    "copyright": "Original AnkiForge test outlines. MIT license.",
    "licenseDescription": "MIT License; see repository LICENSE.",
})
font.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
font.setupPost()
font.font.recalcTimestamp = False
font.font["head"].created = font.font["head"].modified = 3_800_000_000
font.save(root / "labels.ttf")
font.font.flavor = "woff"
font.save(root / "labels.woff")
for filename in ["labels.ttf", "labels.woff"]:
    parsed = TTFont(root / filename, checkChecksums=2)
    assert parsed.getBestCmap() == {32: "space", 65: "A"}
    assert parsed["glyf"]["A"].numberOfContours == 1
