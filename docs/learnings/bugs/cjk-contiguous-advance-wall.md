# CJK per-character breaks must not change advances globally

- Date: 2026-08-13
- Modules: `layout-engine`, `engine`

## Problem

Making all CJK and Southeast Asian per-character break fragments contiguous removed synthetic spaces
that were not present in the source text, but the welcome product fixture regressed from 15.96% to
22.70% against Chromium. The full css-text corpus kept the same pass count while credible strict
matches fell from 580 to 466.

## Root cause

Per-character break opportunities and fragment advances are separate concerns. The legacy non-Ahem
path includes spacing that compensates for layout and paint using different font advance sources.
Removing it globally is spec-shaped in isolation but changes line wrapping before those advance
sources are coherent. Ahem is safe because its fixed square metrics are shared across both paths.

## Solution

Keep contiguous CJK fragments default-on only for Ahem. Preserve the ordinary-font behavior behind
`ZW_CJK_CONTIGUOUS=1` for future experiments, and test the mode through a pure helper rather than
mutating a process-wide environment variable in parallel tests. Re-enable it by default only after
layout, paint, and Chromium use compatible font advances and both product and css-text A/B are
non-negative.
