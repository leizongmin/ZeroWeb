# Root font-relative units need metric identity

- Date: 2026-08-13
- Modules: `css-parser`, `style-system`

## Problem

`font-size: 1rex` was rejected as an unknown length, while `font-size: 1ex` used a fixed `0.8em`
fallback instead of the parent element's first available font metrics. The WPT still passed its loose
threshold because the affected glyph occupied few pixels, but it was not a strict match.

## Root cause

Font-relative lengths lost either their unit identity or the element that owns their metric context.
For `font-size`, `ex` and `ch` use the parent font. `rex` uses the root element's computed font and
x-height even when the current element selects another family.

## Solution

Keep `rex` as a typed `LengthValue` through parsing and `calc()`. Cache the root used x-height after
the root style is computed, pass adjusted parent metrics into font-size resolution, and resolve other
`rex` lengths with the same root metric. Test both a different current family and `calc(2rex)` so a
future implementation cannot silently substitute current-font or fixed-ratio metrics.
