---
date: 2026-08-18
modules: zero-layout-engine inline finalization
---

# Skip redundant final IFC after post-order finalization

## Problem

The post-order inline-block pass had already run IFC and stored final font metrics for 4,000 containers in the medium fixture. The final inline pass did not consume that completion state and repeated child collection, line breaking, and metric storage for the same containers.

## Root cause

Pipeline phases treated the presence of stored `inline_layout` as the only reusable IFC result. Generic text containers intentionally do not retain that full layout, but the post-order pass still leaves two authoritative signals: successful-finalization membership and complete final metric maps.

## Solution

Pass the successful-finalization set into the final inline traversal and skip IFC only when both signals are present. Keep the policy fail-closed for floats, multicol, line clamp, font-size adjustment, custom or multiple fonts, Phase A orphan backfill, and containers that retain a full inline layout.

`ZW_FINAL_IFC_REUSE_REMEASURED=0` restores the original final IFC pass.

## Validation

- Medium off/on PNG SHA-256 is identical.
- Comparable reverse A/B: layout p50 `246.85→214.69ms`, p95 `294.02→262.70ms`.
- DWARF samples for `compute_final_inline_layouts→InlineFormattingContext::layout`: `65→0`.
- Layout tests `1397/1397`; reftest `687/687`; full V8/QuickJS/GPU, product, clippy, and performance gates pass.
