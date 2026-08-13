# Root font-relative units need metric identity

- Date: 2026-08-13
- Modules: `css-parser`, `style-system`

## Problem

`font-size: 1rex`, `1rch`, `1cap`, and `1rcap` were rejected as unknown lengths, while
`font-size: 1ex` used a fixed `0.8em` fallback instead of the parent element's first available font
metrics. The WPTs still passed their loose thresholds because the affected glyphs occupied few
pixels, but they were not strict matches.

## Root cause

Font-relative lengths lost either their unit identity or the element that owns their metric context.
For `font-size`, `ex`, `cap`, and `ch` use the parent font. Their root variants use the root element's
computed font and its x-height, cap-height, or U+0030 advance even when the current element selects
another family.

## Solution

Keep each unit as a typed `LengthValue` through parsing and `calc()`. Cache the root used x-height,
cap-height, and U+0030 advance after the root style is computed, pass adjusted parent metrics into
font-size resolution, and resolve other root-relative lengths with the same root metrics. Test a
different current family and calc/dimension consumers so a future implementation cannot silently
substitute current-font or fixed-ratio metrics.
