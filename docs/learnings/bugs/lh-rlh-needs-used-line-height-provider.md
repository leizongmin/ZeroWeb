# lh/rlh require a used line-height provider

- Date: 2026-08-13
- Modules: `style-system`, `layout-engine`

## Problem

A typed `lh/rlh` implementation made `rlh-in-monospace` match its self-reference exactly, but moved
the fresh Chromium Oracle difference from 0.96% to 0.98%.

## Root cause

`lh/rlh` do not behave like font aspect units. They depend on computed line-height, and CSS Values 4
resolves them against the parent when used in `font-size`, `line-height`, or another font-affecting
property to break cycles. Chrome 127 computed `font-size: 1rlh` as 19px under the default body and
23.3846px under a `font: 1.5em monospace` ancestor. ZeroWeb's stable `1.164em` normal-line fallback is
an internal layout approximation, while its per-font line-height provider is disabled by default.
It therefore cannot produce authoritative parent and root used values for these units.

## Solution

Do not derive `lh/rlh` from a fixed ratio or accept a self-source strict match as sufficient evidence.
First make `line-height: normal` expose the same used parent/root values consumed by layout and paint,
including generic-family behavior and CSS pixel rounding. Then add typed units with three contexts:
current line-height for ordinary properties, parent line-height for font-affecting properties, and
root line-height for ordinary `rlh`. Require fresh Chromium and directory-level A/B before enabling.
