# Cell anatomy illustration

Used by the homepage animation, interactive card previews, and the generated Anki packages.

- Generated on 2026-09-29 with the built-in image_gen tool (not the CLI).
- Saved asset: `cell-anatomy.png` (836 × 471, transparent PNG).
- Preparation: proportional downscale and 256-color PNG compression with Sharp; composition and colors are otherwise unchanged.
- The rectangular nucleus mask is expressed in source image pixels by `website_showcase.rs`; image dimensions in the preview manifest are read from the PNG.

## Final prompt

```text
Use case: scientific-educational.
Asset type: a polished illustration inside an AnkiForge image-occlusion flashcard on a developer library homepage.
Primary request: a beautiful, clearly readable cross section of one animal cell, designed to remain legible when displayed about 260 pixels wide.
Style/medium: premium educational 3D illustration, softly sculpted forms, matte translucent material with fine restrained detail, clean studio lighting, crisp silhouettes.
Composition/framing: wide landscape 16:9 canvas. One horizontally oval cell fills most of the frame, complete outline visible with a small clear margin. Almost top-down orthographic view. Broad mint-teal cytoplasm and a thin darker emerald cell membrane. The single large coral-peach nucleus is at the center, visually distinct, round with a visible darker nucleolus. Leave a small uncluttered gap around the nucleus so a rectangular HTML mask can cover just this structure. A few clearly separated mitochondria with visible inner folds and some folded teal endoplasmic reticulum around the sides provide biological context. Balanced, sparse composition.
Color palette: restrained botanical emerald and mint, warm coral for the nucleus, small amber organelle accents. It must read well on both off-white and dark green UI surfaces.
Scene/backdrop: genuinely transparent background outside the cell, no floor, no backdrop.
Constraints: scientifically recognizable stylized animal cell; one nucleus only; no labels, no words, no lettering, no arrows, no lines pointing to structures, no occlusion mask, no interface, no border, no watermark, no glow, no cute face. Keep all structures within the membrane. Prioritize clarity and elegance over dense microscopic detail.
```
