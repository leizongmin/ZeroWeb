---
date: 2026-08-11
modules: zero-engine, zero-layout-engine, zero-render-foundation
---

# Shaped advance must preserve the paint base

## Problem

Using the absolute rustybuzz shaped width for layout made product rendering substantially worse even though the width came from the same nominal font.

## Root Cause

The shaping and paint paths did not have identical base metrics:

- rustybuzz positions were combined with fontdue floating-point metrics;
- missing glyphs used the primary face's `.notdef` width, approximately `0.6em`;
- paint used FreeType 26.6 grid-fitted advances, font fallback, and finally a `0.5em` fallback.

Absolute shaped width therefore replaced the paint base as well as adding contextual kerning. It was not layout/paint coherence.

## Solution

Keep the paint advance as the base and apply only the contextual shaping delta:

```text
layout advance = paint base + shaped advance - unshaped advance
```

Pass the same source and resolved font ID into both layout IFC and paint IFC. Reject missing-glyph, spacing, vertical, RTL, Ahem, and ruby runs until those paths have equivalent contracts.

Use a gated fragment trace to compare layout estimate, unshaped/shaped advance, fragment width, and final paint consumption before changing defaults.
