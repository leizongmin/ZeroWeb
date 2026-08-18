# Shorthand Component Classification

Date: 2026-08-18

Related modules: `crates/style-system/src/shorthand`, `crates/css-parser/src/values`

## Problem

`border: begin solid red` was expanded as if it were valid. The old `looks_like_length` classified `begin` as a length because it ended with `in`; after switching to the shared length parser, the parser still accepted broad values such as `auto`, and unknown border shorthand tokens were silently ignored.

The same silent-defaulting pattern also affected `outline`, `column-rule`, and `text-decoration`: unknown tokens were ignored while the valid remaining components were expanded.

`text-decoration`, `outline`, `column-rule`, and `list-style` exposed another form of the same bug class: duplicate components were accepted and the later value silently replaced the earlier one. For example, `underline dotted dashed red` became valid with style `dashed`, `solid red blue` became valid with color `blue`, and `inside outside` became valid with position `outside`.

For repeated components that are allowed by grammar, such as multiple `text-decoration-line` keywords, the repeated set still needs grammar-specific validation. `none` is mutually exclusive with other line keywords, and each line keyword can appear at most once.

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

Keep adjacent shorthand users covered with regression tests when a shared classifier changes. `columns: auto 100px` is a good guard because it depends on `auto` not being mistaken for a length.
