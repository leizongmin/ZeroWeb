# Variable font shaping must share axes with rasterization

- Date: 2026-08-14
- Modules: `render-foundation/font`, `layout-engine`, `engine/paint`, WPT runner

## Problem

`font-variation-settings` coordinates reached rustybuzz shaping and changed HVAR advances, but Chromium Oracle results regressed instead of improving.

## Root Cause

The shaping path applied variation coordinates through `rustybuzz::Face::set_variations`, while glyph rasterization still used the default font instance. The resulting run combined varied advances with unvaried outlines. A subset variable font also required grapheme-level fallback for characters outside its cmap; without that split, glyph ID 0 caused the entire run to fall back to legacy paint and hid the variation path.

## Solution

Keep the production gate off until the same ordered axis vector is carried through `GlyphPrimitive`, raster cache keys, FreeType variation coordinates, renderer IPC, browser, and compositor reconstruction. Use a variable font with a width axis to test shaping and cache isolation because weight-only axes may change outlines without changing advances.

FreeType faces are mutable and cached across glyphs. Every raster call must therefore apply the requested full coordinate vector, including an explicit reset to axis defaults when the vector is empty. Glyph bitmap and GPU atlas keys must include the ordered `(tag, value.to_bits())` vector; otherwise two instances of the same font, glyph, and size alias even though their outlines differ.

Chromium Oracle is the activation gate. A shaping-only implementation is infrastructure, not a completed visual implementation.
