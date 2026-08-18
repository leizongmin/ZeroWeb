---
date: 2026-08-18
modules:
---

# Shorthand Component Classification

Related modules: `crates/style-system/src/shorthand`, `crates/css-parser/src/values`

## Problem

`border: begin solid red` was expanded as if it were valid. The old `looks_like_length` classified `begin` as a length because it ended with `in`; after switching to the shared length parser, the parser still accepted broad values such as `auto`, and unknown border shorthand tokens were silently ignored.

The same silent-defaulting pattern also affected `outline`, `column-rule`, and `text-decoration`: unknown tokens were ignored while the valid remaining components were expanded.

`text-decoration`, `outline`, `column-rule`, and `list-style` exposed another form of the same bug class: duplicate components were accepted and the later value silently replaced the earlier one. For example, `underline dotted dashed red` became valid with style `dashed`, `solid red blue` became valid with color `blue`, and `inside outside` became valid with position `outside`.

For repeated components that are allowed by grammar, such as multiple `text-decoration-line` keywords, the repeated set still needs grammar-specific validation. `none` is mutually exclusive with other line keywords, and each line keyword can appear at most once.

Some shorthands do not need component classification but still need value validation after token counting. `overflow` and `overscroll-behavior` accepted arbitrary one-token values and collapsed three-or-more tokens back into one invalid longhand value, instead of rejecting the shorthand before cascade.

`flex-flow` exposed the same class through ignored tokens and first-value-wins slots: unknown tokens were skipped, duplicate direction or wrap components were silently ignored, and an empty shorthand became `row nowrap`.

`gap` exposed the value-validation variant: token count alone was treated as sufficient, so unknown tokens, `auto`, border-width keywords, and negative simple lengths crossed the shorthand boundary even though `gap` only accepts `normal` or non-negative length-percentage values.

Grid placement shorthands exposed the delimiter variant: splitting at the first slash allowed extra slash-separated components and empty components to become longhand values instead of rejecting the shorthand.

`place-*` shorthands exposed the cross-longhand validation variant: a value can be valid for the justify side but invalid for the align side, so token count alone is not enough.

`border-width`, `border-style`, and `border-color` exposed the edge-quartet variant: 1-4 token expansion must still validate each mapped side against the specific target grammar.

`border-radius` exposed the radius form of the same class: 1-4 corner expansion must reject non-radius tokens and negative simple radii before emitting corner longhands.

Logical border axis shorthands exposed the axis variant: 1-2 start/end mapping must validate each mapped value against the target width, style, or color grammar before emitting logical longhands.

`padding` and logical padding exposed the box-edge validation variant: padding accepts neither `auto` nor negative lengths, so both 1-4 full expansion and 1-2 logical-axis expansion need property-specific token validation.

`margin` and logical margin exposed the permissive edge variant: margin accepts `auto` and negative lengths, but still must reject colors and border-width keywords such as `thin`.

`inset` and logical inset use the same permissive offset grammar as margin for shorthand validation: `auto` and negative offsets are valid, while colors and border-width keywords must still invalidate the entire shorthand.

`scroll-margin` and `scroll-padding` exposed the missing-expander variant: implemented longhands are not enough if the shorthand is absent from `expand_one`; the declaration will pass through as a normal property and become a no-op during apply.

`border-image` exposed the routed-longhand validation variant: after splitting a complex shorthand into source, slice, width, outset, and repeat groups, each non-empty group must still be accepted by the corresponding longhand parser before any declaration is emitted.

`grid-template` exposed both delimiter and CSS-wide keyword variants: slash-separated forms must reject empty sides and extra delimiters, and CSS-wide keywords must expand to every longhand in the shorthand, not just the row side.

`text-emphasis` exposed the atomicity variant: if one extracted longhand component is valid and another is invalid, the shorthand must reject the whole declaration instead of letting the valid component survive.

`font-variant` exposed the optional-slot overwrite variant: unknown tokens must not be ignored, and mutually exclusive longhand slots must reject repeats instead of allowing the last token to win.

`columns` exposed the helper-scope variant: a token classifier built for border/outline widths is too permissive for `column-width`; shorthand helpers must encode the target property's own range restrictions such as non-negative lengths and keyword exclusions.

`font` exposed the pre-required-component variant: before a required component such as font-size is found, unknown optional-looking tokens must reject the shorthand instead of being skipped; generic numeric parsing must not widen the target grammar to accept bare nonzero numbers.

`transition` and `animation` exposed the repeated-slot/list-item variant: parsers for comma-separated shorthand items need a failure state, not just default-filled output. Duplicate component slots, overlong time lists, and empty comma items must invalidate the whole shorthand item; otherwise the later token silently overwrites the earlier one or an empty list item disappears before validation. Ambiguous keywords such as `animation-name:none` versus `animation-fill-mode:none` still need grammar-aware positive guards.

`background` exposed the default-to-color variant: a classifier must not route every unknown token to a permissive fallback slot. Only tokens accepted by the real color grammar may fill `background-color`, repeated color tokens must reject the shorthand, and tokens that are valid only on the position side must not be silently ignored after the `/` size delimiter.

`background-size` and `background-attachment` exposed the slot cardinality variant: a token may be valid for a longhand but invalid in its shorthand position or as a repeated component. `cover`/`contain`/`auto` must be tied to the slash-delimited size side, `cover`/`contain` are exclusive, size accepts at most two simple components, and attachment accepts only one component per background layer.

## Root Cause

The shorthand layer used heuristic component classifiers and treated unrecognized tokens as absent optional components. That is wrong for CSS shorthands such as `border`, where every token must match one of the allowed component grammars. A shared parser can also be broader than the property grammar that consumes it: general length parsing may accept `auto`, intrinsic sizing keywords, or percentages that `border-width` must reject.

## Solution

Use the shared value parsers for token boundaries, then filter by the specific property grammar. For border-like shorthands:

+ accept only real length variants plus `thin`, `medium`, and `thick` as width components;
+ use the real color parser for color components;
+ reject the entire shorthand when any token is not consumed by one of the allowed component grammars.
+ reject duplicate components unless the spec grammar explicitly allows repetition, such as multiple `text-decoration-line` keywords.
+ when grammar allows repetition, validate the repeated group itself instead of accepting arbitrary token lists.
+ preserve explicit grammar exceptions while adding duplicate guards, such as `list-style: none square url(...)`, where `none` supplies default type/image values that explicit type/image tokens may override.
+ for simple 1-2 value shorthands, validate each token against the corresponding longhand grammar and reject overlong token lists.
+ for 1-4 edge shorthands, expand the rectangle mapping only after every mapped value validates against the target longhand grammar.
+ for unordered component shorthands, reject unknown tokens and duplicate component slots before applying initial defaults for omitted slots.
+ for slash-delimited shorthands, validate the full shorthand grammar before splitting into raw longhand values.
+ for shorthands that route values to different longhand grammars, validate against each target longhand before emitting any declaration.

Keep adjacent shorthand users covered with regression tests when a shared classifier changes. `columns: auto 100px` is a good guard because it depends on `auto` not being mistaken for a length.
