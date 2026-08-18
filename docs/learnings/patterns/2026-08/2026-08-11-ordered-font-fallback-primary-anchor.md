---
date: 2026-08-11
modules: zero-layout-engine, zero-engine
---

# Ordered font fallback must preserve the resolved primary face

## Problem

Passing an ordered CSS face list into IFC shaping changed far more pages than the fallback cases being targeted. A css-fonts Oracle A/B run moved the aggregate diff from `862.56` to `890.90`, with `font-family-name-025` alone regressing by `19.65pp`.

## Root Cause

The new list resolver understood weight and style variants, while the established `TextRun.font_id` resolver did not use exactly the same matching algorithm. Treating the list's first entry as authoritative therefore changed the primary face in alias, weight, and style cases. The change was no longer a fallback-only extension.

## Solution

Keep `TextRun.font_id` as the primary-face contract until primary matching is migrated explicitly. Use list-aware measurement only when:

```text
font_ids.first() == TextRun.font_id
```

Otherwise, fall back to the existing singleton measurement. The painter applies the same invariant by moving the fragment's resolved face to the front of its shaping list.

After adding this guard, the css-fonts A/B returned to the bounded fallback-only result: aggregate `862.56→862.67`, with changes limited to the four previously identified cases.
