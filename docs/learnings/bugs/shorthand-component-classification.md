# Shorthand Component Classification

Date: 2026-08-18

Related modules: `crates/style-system/src/shorthand`, `crates/css-parser/src/values`

## Problem

`border: begin solid red` was expanded as if it were valid. The old `looks_like_length` classified `begin` as a length because it ended with `in`; after switching to the shared length parser, the parser still accepted broad values such as `auto`, and unknown border shorthand tokens were silently ignored.

## Root Cause

The shorthand layer used heuristic component classifiers and treated unrecognized tokens as absent optional components. That is wrong for CSS shorthands such as `border`, where every token must match one of the allowed component grammars. A shared parser can also be broader than the property grammar that consumes it: general length parsing may accept `auto`, intrinsic sizing keywords, or percentages that `border-width` must reject.

## Solution

Use the shared value parsers for token boundaries, then filter by the specific property grammar. For border shorthand:

+ accept only real length variants plus `thin`, `medium`, and `thick` as width components;
+ use the real color parser for color components;
+ reject the entire shorthand when any token is not width, style, or color.

Keep adjacent shorthand users covered with regression tests when a shared classifier changes. `columns: auto 100px` is a good guard because it depends on `auto` not being mistaken for a length.
