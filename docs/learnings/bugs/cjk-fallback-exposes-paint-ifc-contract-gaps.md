# CJK fallback exposes paint IFC contract gaps

Date: 2026-08-14

Related modules: `layout-engine`, `engine::paint`, `render-foundation::font`

## Problem

Installing a real CJK fallback font made Chinese text visible, but glyphs were shifted upward, CJK text contained synthetic gaps, and text following native checkbox/radio controls overlapped the controls.

## Root Cause

Three independent paint IFC contracts had been hidden while CJK glyphs were unavailable:

1. The non-stored paint IFC passed an em-box top coordinate as `GlyphPrimitive.y`, while the rasterizer consumes it as a baseline.
2. Per-character CJK line-break opportunities synthesized spaces that were not present in the source text.
3. The paint IFC intentionally runs without styles. It received atomic element sizes, but atomic detection still required a computed style, so native controls contributed zero inline width. Leading collapsed whitespace after the control was also discarded.

## Solution

Use the full text-fragment height to convert the fragment top to a glyph baseline. Keep per-character CJK break opportunities contiguous by default. Treat stored inline-block sizes as sufficient atomic-element evidence in the style-free paint IFC, and preserve one collapsed leading space after a preceding inline item.

## Prevention

Test fallback-font pages with visible CJK glyphs. Include `<label><input> text</label>` and line-height greater than font-size so baseline, atomic width, and whitespace contracts are exercised together.
