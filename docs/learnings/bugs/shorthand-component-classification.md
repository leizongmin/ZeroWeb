# Shorthand Component Classification

Date: 2026-08-18

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
+ for unordered component shorthands, reject unknown tokens and duplicate component slots before applying initial defaults for omitted slots.
+ for slash-delimited shorthands, validate the full shorthand grammar before splitting into raw longhand values.

Keep adjacent shorthand users covered with regression tests when a shared classifier changes. `columns: auto 100px` is a good guard because it depends on `auto` not being mistaken for a length.
