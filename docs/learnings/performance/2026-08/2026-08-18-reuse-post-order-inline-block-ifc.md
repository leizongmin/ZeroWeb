---
date: 2026-08-18
modules: zero-layout-engine inline finalization and postprocessing
---

# Reuse post-order inline-block IFC results

## Problem

`remeasure_inline_only_containers` and `adjust_inline_block_positions` independently ran an `InlineFormattingContext` for the same container. The medium fixture has 4,000 items with one inline-block badge each, so the second pass repeated collection, line breaking, metrics storage, and alignment only to recover final badge coordinates.

## Root cause

The first IFC runs before recursive child finalization. A badge can still have a stale Taffy height of `2px`; its own remeasure later raises that height to `13.64px`. Directly reusing the stale fragment moved badges from absolute `y=65.0px` to `72.6px` and changed `15113/480000` pixels (`3.15%`). Passing reftests alone did not expose this fixture-specific regression.

## Solution

Retain the first IFC until direct children finish recursively. Reuse it only when all targeted atomic children are ordinary horizontal baseline-aligned inline-blocks, widths are unchanged, and heights only increase. Refresh fragment height and baseline, raise line height to the final margin-box height when needed, recompute line offsets, then copy coordinates and metrics. Any vertical mode, complex atomic type, multicol container, width change, height shrink, non-baseline alignment, or incomplete fragment match falls back to the original full pass.

`ZW_IFC_REUSE_INLINE_BLOCK_POSITIONS=0` restores the original behavior.

## Validation

- Medium off/on PNG SHA-256 is identical; zero-tolerance diff is `0/480000`.
- Comparable reverse A/B: layout p50 `321.39→301.50ms`, p95 `381.55→355.46ms`.
- Frame-pointer samples containing `adjust_inline_block_positions→InlineFormattingContext::layout`: `56→0`.
- Layout tests `1394/1394`; reftest `687/687`; product smoke and full performance gates pass.
