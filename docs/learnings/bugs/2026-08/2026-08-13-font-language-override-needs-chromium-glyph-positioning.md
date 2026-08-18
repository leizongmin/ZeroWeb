---
date: 2026-08-13
modules: style-system, render-foundation, engine
---

# font-language-override needs Chromium-aligned glyph positioning

## Problem

A complete `font-language-override` prototype correctly selected the Libertine `TRK` language
system. It changed `fi` from one ligature glyph to two glyphs and produced a PNG byte-identical to
the WPT reference using `font-feature-settings: "liga" 0`. Despite that semantic match, the fresh
Chromium Oracle difference moved from 0.78% to 0.79%.

## Root cause

The OpenType language-system selection was correct: the lowercase `"trk"` control retained the
ligature, while uppercase `"TRK"` disabled it. The remaining difference is downstream of feature
selection. ZeroWeb rasterizes and positions the two separated glyphs differently from Chromium, so
the incorrect single ligature happened to be one hundredth of a percentage point closer in the
current screenshot metric.

## Solution

Do not enable language-system selection solely because it matches a self-rendered reference.
Revisit it after separated-glyph positioning, advances, and rasterization share the Chromium-aligned
path. Keep a real-font test covering default one-glyph `fi`, uppercase `TRK` two-glyph output, and
lowercase `trk` one-glyph output, then require fresh Chromium and directory-level A/B before landing.
