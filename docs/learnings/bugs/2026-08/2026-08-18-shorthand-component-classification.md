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

`transition` and `animation` exposed the fallback-ident variant: after specialized slots fail, the final custom-ident/name slot must still validate the token shape against its own grammar. Negative time-shaped tokens, digit-start tokens, quoted strings for transition-property, and function tokens must not be accepted merely because no earlier classifier matched them; animation-name is a different grammar and may accept quoted keyframe names.

`animation-iteration-count` exposed the divergent-parser variant: parser and longhand apply must share the same target grammar. CSS allows zero iteration counts but rejects negatives; if the parser rejects zero while longhand accepts raw negative numbers, shorthand and longhand behavior diverge in both directions.

`transition-duration` and `animation-duration` exposed the sibling-longhand variant: related longhands may share a primitive parser but not the same grammar. Duration requires non-negative time while delay accepts negative time, so duration apply must reuse the duration parser instead of the generic time parser, and delay must remain deliberately wider.

`transition-delay` and `animation-delay` exposed the partial-list variant: even when a sibling longhand intentionally accepts a wider primitive value such as negative time, comma lists must still fail atomically if any item is invalid. `filter_map` is unsafe at CSS grammar boundaries because it silently converts invalid lists into shorter valid lists.

`animation-direction`, `animation-fill-mode`, and `animation-play-state` exposed the enum-list variant of the same bug: enum-valued comma lists must reject atomically when any item is unknown. This is especially easy to miss because a `filter_map` result still looks non-empty and can overwrite the previous computed value with a shorter list.

`contain-intrinsic-size` exposed the non-comma filter-map variant: whitespace-separated component lists are just as vulnerable as comma lists. Optional keywords such as `auto` may be intentionally skipped, but every remaining token must be classified; unknown tokens must reject the whole declaration instead of being dropped while valid lengths survive.

`transform-origin` and `perspective-origin` exposed the tail-token variant outside shorthand expansion: a parser that validates the first component and defaults or ignores the rest still violates CSS declaration atomicity. Optional defaulting such as the Y origin defaulting to `50%` is only valid when the component is omitted, not when an invalid token is present; extra tokens must also reject the whole declaration.

`columns` exposed the partial-write variant: even if a parser ultimately returns `false`, writing the first successfully parsed component before validating the whole declaration still corrupts computed state. Multi-component apply paths should parse into temporary values first and only commit to `ComputedStyle` after every component in the declaration has passed validation.

`animation-name` exposed the longhand/parser-divergence variant: fixing shorthand parsing is not enough when the corresponding longhand apply path directly stores raw comma items. Longhand list properties must reuse the same item grammar as their parser, reject empty comma items, and commit the list only after every item validates.

`transition-property` exposed the sibling longhand variant of the same issue: when a shorthand helper has learned a target grammar, audit the longhand apply path too. Raw comma-splitting of custom-ident lists accepts quoted strings, time-shaped tokens, and empty items unless the longhand shares the same item validator.

`transition-timing-function` and `animation-timing-function` exposed the helper-return-type variant: returning a plain `Vec` from a list parser encourages callers to treat "some valid items" as success. List parsers at CSS grammar boundaries should return a failure state, not just a possibly shortened list, so callers cannot accidentally apply partially valid declarations.

`font-variant-alternates` exposed the argument-list variant: even inside a single CSS function, comma-separated `#` lists must reject empty items. Avoid `filter(|arg| !arg.is_empty())` at grammar boundaries; trim each item, fail on empties, then collect only after validation.

`text-shadow` and `box-shadow` exposed the shared-splitter variant: private comma split helpers are grammar boundaries too. A helper that silently drops empty leading, trailing, or repeated comma segments makes every caller accept partially valid CSS lists; return an explicit failure state instead.

`box-shadow` exposed the optional-marker variant: optional keywords such as `inset?` are still grammar components with cardinality. Extracting them with `filter_map` or unconditional removal must track whether the component has already appeared; otherwise duplicate markers become invisible instead of invalidating the value.

`clip-path: polygon()` exposed the point-list variant: list parsers must not skip malformed items just because later items are valid. Empty point segments, missing coordinates, and extra coordinates are declaration-level failures unless the target grammar explicitly supports recovery.

`clip-path: inset()` exposed the trailing-component and optional-subgrammar variant: reading only the first N components silently accepts extra tokens, and optional subgrammars such as `round <border-radius>` must fail the declaration when present but invalid rather than being downgraded to absent.

`clip-path` circle/ellipse exposed the invalid-optional-result variant: when a delimiter keyword such as `at` is present, failure to parse the following subgrammar is not the same as the subgrammar being absent. Return a failure state from split helpers so callers cannot silently continue with defaults.

`background-position` exposed the axis-classification variant: normalizing order is not a substitute for validating grammar roles. Components with axis-specific meaning must be classified before swapping or defaulting, otherwise same-axis pairs can be reshuffled into nonsensical but accepted computed values.

`overflow-clip-margin` exposed the empty-input-defaulting variant: default values are only valid after the grammar has matched at least one component. An empty token list must fail before applying defaults, otherwise an absent declaration value becomes a real computed value.

`text-emphasis-position` exposed the duplicate-axis overwrite variant: paired axis keywords are slots, not assignments that can be overwritten. Empty token lists must fail, and a second token for the same axis must invalidate the declaration.

`text-emphasis-style` exposed the duplicate-component overwrite variant: `||` order independence still has per-component cardinality. Parser state variables must act like occupied slots, not mutable last-token-wins assignments.

`scroll-snap-type` exposed the duplicated-parser variant: when the same grammar is implemented in multiple modules, strictness fixes must be applied to every implementation or re-export path. Slot occupancy checks should be mirrored across copies until the parser has a single owner.

`border-image-slice` exposed the boolean-marker duplicate variant: even a marker stored as a bool has grammar cardinality. Setting a flag is not enough; a second occurrence of the marker must invalidate the declaration.

CSS gradients exposed the invalid-config-defaulting variant: once a configuration keyword such as `at` or `from` appears, the following subgrammar is present and must parse successfully. Unknown radial shape/size tokens, invalid gradient positions, and invalid conic angles must reject the whole gradient instead of falling back to default center or `0deg`.

Relative color syntax exposed the comma-channel variant: channel lists inside CSS functions are still grammar lists. A trailing or repeated comma in `rgb(from ... g, r, b,)` or `g,,b` must fail instead of filtering out empty segments and accepting the remaining three channels.

Lab-like color functions exposed the component-cardinality variant: `lab()` / `lch()` / `oklab()` / `oklch()` have exactly three main components plus optional slash alpha. Parsers must reject empty comma segments and extra main components instead of accepting the first three values and ignoring the rest.

Transform functions exposed the fixed-arity function variant: once function arguments are parsed, each transform function still has its own cardinality. Empty comma segments and extra arguments in single-axis or two-axis functions must fail instead of being filtered or ignored after the first expected values.

Timing functions exposed the function-specific numeric-constraint variant: fixed argument counts are not enough. `steps()` has step-count lower bounds, including a stricter `jump-none` lower bound, and `cubic-bezier()` only allows x coordinates in `[0,1]`; accepting the first valid-looking arguments silently widens the grammar.

`hwb()` exposed the color-function trailing-token variant: modern color functions with exactly three main components must reject extra components and must consume the entire function input. `rfind(')')` body extraction is only safe after confirming the value ends at that closing parenthesis.

`rgb()` / `hsl()` exposed the modern-alpha delimiter variant: CSS Color 4 whitespace syntax does not allow a fourth bare component as alpha. Alpha in modern syntax must be slash-delimited, and parser body slicing must still reject tokens after the closing parenthesis.

Filter functions exposed the role-specific length constraint variant: a shared `<length>` parser may be correct for offsets but too wide for blur radii. `blur()` and the third `drop-shadow()` length require non-negative values, while the first two drop-shadow offsets still allow negatives.

Filter amount functions exposed the shared-number lower-bound variant: a shared number/percentage parser for `brightness`/`contrast`/`opacity`-style functions must reject negative finite values, while still allowing greater-than-one amounts where the consuming filter grammar permits them.

Filter length and angle functions exposed the Rust-float widening variant: `f32::parse` accepts textual `inf` and `NaN`, but CSS numeric tokens do not. Filter-specific parsers must require finite values after unit conversion, not just successful Rust parsing.

Filter functions exposed the function-token-boundary variant: trimming the substring before `(` accepts `blur (5px)`, but CSS tokenization only forms a function token when the ident is immediately followed by `(`. Consumer parsers that operate on strings must preserve that boundary and reject whitespace before the opening parenthesis.

Background image URLs exposed the URL-payload token variant: after recognizing `url(...)`, the payload still has quoted-string vs unquoted-url-token grammar. Quoted payloads must be fully enclosed by matching quotes with no trailing token; unquoted payloads must reject raw whitespace, quotes, and parentheses instead of treating the entire slice as an opaque URL.

Image source properties exposed the copied-parser variant: fixing `background-image` alone leaves `border-image-source` and `list-style-image` accepting the same invalid URL payloads if each property keeps a local string-slice parser. Shared CSS image source helpers should own URL payload grammar so all consumers reject the same invalid token shapes.

Generated content URLs exposed the cross-module consumer variant: `content: url(...)` lives outside the visual image parser module, but it still consumes the same CSS URL payload grammar. After extracting a shared helper, audit non-background image consumers across modules so generated content does not keep an older `trim_matches` parser.

Font-face sources exposed the token-serialization variant: `Token::Url("a b.woff")` displayed as `url(a b.woff)` loses the fact that the original input was quoted. Downstream string consumers then cannot distinguish legal quoted whitespace from illegal unquoted whitespace. URL token display must emit a quoted, escaped form whenever the payload cannot be represented as an unquoted url-token.

Tokenizer URL parsing exposed the bad-url truncation variant: when unquoted `url()` sees raw whitespace followed by more input, or quoted `url()` sees trailing input after the closing quote, returning the prefix as `Token::Url` turns an invalid token into a valid URL. Tokenizers must return an error token and consume the bad-url remnants so consumers cannot observe a truncated "valid" URL. Resource scanners that concatenate `<style>` blocks must also preserve each block's EOF boundary, otherwise a malformed URL in one block can absorb URLs from the next block.

List-style shorthand exposed the late-validation variant: expanding a shorthand into three longhands and relying on the later apply layer to reject one invalid longhand breaks CSS atomicity. Shorthand expansion must validate every produced longhand before returning any declarations, especially when a component such as `url(...)` has a stricter property-specific parser.

Background shorthand exposed the early-return variant: even after adding final longhand validation to the normal path, special branches such as `rgb()/hsl()/var()` color handling can still return partially validated longhands. Build the produced longhand vector first, then run one shared validation step for every non-wide-keyword path.

Opacity exposed the duplicated numeric-parser variant: Rust `f64::parse` accepts `inf` and `NaN`, but CSS number tokens do not. Any CSS numeric parser that clamps after parsing must first require finite values, and duplicated parser modules (`parse_basic` / `parse_layout`) must be fixed together to avoid re-export path drift.

Aspect-ratio exposed the derived-number variant: even if a slash ratio rejects only denominator zero, `1 / inf` can silently derive `0` and `inf / inf` can derive `NaN`. Validate every parsed component and the computed ratio before storing layout-facing numeric values.

Animation and transition timing exposed the keyword-vs-number variant: `animation-iteration-count: infinite` is a valid property keyword, but Rust `f64::parse("inf")` is not a valid CSS number token. Keep property keywords explicit and require every parsed numeric time/count to be finite before writing computed lists.

Transform exposed the shared-numeric-entry variant: one helper can feed length, angle, percentage, matrix, scale, and perspective functions. Put the finite check in the shared numeric helper and in the percent wrapper, otherwise individual transform functions will keep accepting different `inf` / `NaN` spellings.

Border-spacing exposed the consumer-grammar negative-length variant: the shared `parse_length` correctly accepts negative lengths for properties that allow them, but table `border-spacing` does not. After shared length parsing, filter the parsed values by the specific property grammar before writing computed table spacing.

Border-image exposed the nonnegative-but-nonfinite variant: a guard like `value < 0.0` does not reject `inf` or `NaN`. For CSS grammar terms such as nonnegative number/percentage, require finite first, then apply range checks, before writing the computed border image components.

Border-image also exposed the split-branch negative-length variant: fixing number/percentage branches is not enough when a sibling branch delegates to shared `parse_length`. For properties whose grammar says nonnegative length, run the parsed length through the same consumer-grammar negative filter before accepting it.

Column-rule-width exposed the duplicated-entry consumer-grammar variant: the same property parser existed in both `parse_basic` and `parse_layout`, and both delegated directly to shared `parse_length`. When a consumer grammar is narrower than the shared parser, audit and fix every exported entry point or the rejected value can still enter through the alternate module.

Column-width exposed the longhand-vs-shorthand grammar drift variant: the `columns` shorthand had a correct local validator for `auto | <length [0,∞]>`, but the longhand parser still delegated directly to shared `parse_length`. When a shorthand contains a stricter local validator for a longhand value, mirror that consumer grammar in the longhand parser as well.

Text-decoration-thickness exposed the narrowed-unit range-check variant: even when a property only accepts one `parse_length` output arm such as `Px`, that arm can still carry values outside the consumer grammar. Apply nonnegative or finite filters after matching the accepted unit, not only when accepting broad length families.

Tab-size exposed the keyword-alias range-check variant: a property may first reject negative and percentage lengths correctly, yet still accept `thin` because shared `parse_length` aliases border-width keywords into positive `Px` values. Reject out-of-grammar keywords before calling the shared parser when the original token identity matters.

Line-height exposed the shorthand-vs-longhand negative grammar drift variant: `font` shorthand can reject negative line-height while the longhand parser still accepts negative number/length/percentage and non-finite numbers. Keep longhand consumer grammar at least as strict as shorthand prevalidation, and apply finite checks to numeric property values before writing computed style.

Perspective exposed the apply-helper grammar drift variant: longhands implemented only in `apply_*` can bypass dedicated parser tests and inherit every value accepted by `parse_length_or_math`, including percentages and keyword aliases. Treat apply-only longhands as parser boundaries too, and filter helper output before mutating computed style.

Row-gap and column-gap exposed the shorthand-only validator variant: `gap` shorthand can correctly reject negative values while its generated longhands still accept them when authored directly. Mirror shorthand component validators in the longhand apply path, including grammar keywords such as `normal` that do not pass through the shared length parser.

Flex-basis exposed the width-grammar filter variant: properties based on `<width>` may legitimately keep intrinsic sizing keywords such as `min-content`, `max-content`, and `fit-content(...)`, while still rejecting negative length/percentage values and border-width keyword aliases. Consumer grammar filters should preserve those intrinsic branches instead of reducing the property to plain lengths.

Min/max sizing exposed the cascade-vs-apply drift variant: declaration cascade may already filter invalid negative values, while direct `apply_property_value` still accepts them through shared helpers. Keep direct apply APIs as strict as cascade validation because tests, CSSOM-style mutation paths, and future callers may bypass cascade.

Width/height logical sizing exposed the alias-branch drift variant: fixing the physical property branch is insufficient when logical properties map to the same computed slots through separate match arms. Apply the same consumer grammar filter to alias branches such as `inline-size`, `block-size`, and min/max logical sizing before mutating the shared physical fields.

Padding exposed the logical-shorthand parity variant: a shorthand can reject invalid values while both physical longhands and logical longhands still bypass that validator through direct apply arms. For non-negative box properties, route every physical and logical write through the same consumer grammar filter before mutating computed fields.

Border-radius exposed the shared-alias variant inside a local validator: even a shorthand-specific validator can remain too broad if it accepts any `LengthValue` produced by the shared parser. Reject aliases that belong to another grammar, such as border-width `thin`/`medium`/`thick`, before accepting radius length-percentage values.

Background-size exposed the dedicated-parser drift variant: having a property-specific parser is not enough if its internals still delegate to shared length parsing without checking the consuming grammar. Filter both single-value and two-value component paths for finite non-negative length/percentage values, and reject aliases that belong to other grammars before direct apply can persist them.

Letter-spacing exposed the inherited-text direct-apply variant: inherited text longhands can bypass parser-level assumptions through `apply_property_value` and persist values from the shared length parser that are not in the property grammar. Validate direct writes against `normal | <length>` so percentages, sizing keywords, border-width aliases, and non-finite lengths fail without mutating the previous computed value.

Text-indent exposed the duplicated text-longhand parser variant: a parser may reject one keyword such as `auto` yet still accept shared-parser aliases and non-finite numbers, while direct apply accepts even the rejected keyword. Keep duplicated parser modules and direct apply on the same consumer grammar, and preserve legitimate negative lengths and percentages when narrowing the rest.

Word-spacing exposed the keyword-bypass direct-apply variant: a longhand can reject its own valid keyword when direct apply only calls the shared length parser, while also accepting aliases that belong to other grammars. Handle property keywords before shared parsing, then filter parsed values by the property grammar and keep failed declarations atomic.

Outline-offset exposed the mutually-exclusive keyword variant: a property-specific keyword such as `inset` must remain mutually exclusive with the `<length>` branch, and invalid shared-parser values must not clear that keyword state. Validate the parsed length branch before mutating either the value slot or companion keyword flag.

Positioned offsets exposed the physical/logical alias variant: fixing `top/right/bottom/left` is incomplete if `inset-block-*` and `inset-inline-*` still write through the broader shared length parser after writing-mode mapping. Put the consumer grammar in one shared validator and call it before mutating either physical or logical alias slots.

Margins exposed the same physical/logical alias shape with a different property owner: the grammar matches `auto | <length-percentage>`, but the validator should still be named for margin so later changes do not accidentally couple box spacing to positioned-offset semantics. Reuse the invariant, keep the public helper semantically scoped, and validate logical aliases before writing the mapped physical side.

Border-width exposed the alias-is-legitimate variant: `thin|medium|thick` are invalid for many length consumers but valid for `<line-width>`, so the consumer validator must distinguish property-owned aliases from shared-parser drift. Validate both physical border widths and logical border width aliases before mutating the mapped side.

Gap exposed the legacy-field variant: even when a shorthand expands to validated longhands, any retained direct field for the shorthand must enforce the same consumer grammar. Keep `gap`, `row-gap`, and `column-gap` on one non-negative gap validator so direct apply cannot accept broader shared-parser values or reject valid `normal`.

Outline-width exposed the shared-line-width variant: properties that share `<line-width>` with border width should reuse the same consumer validator, including legitimate `thin|medium|thick` aliases and rejection of percentages, `auto`, sizing keywords, negative values, and non-finite lengths.

Font-size exposed the shorthand/direct parity variant: even after a shorthand has a local size grammar, the direct longhand must reject the same shared-parser drift. Keep keyword handling separate, then validate `<length-percentage [0,∞]>` before mutating `font_size`.

Scroll-margin exposed the `<length>`-only variant: properties stored as px can still receive broader shared `LengthValue` inputs before conversion. Validate the source value before `resolve_length_to_px`, especially to reject percentages, aliases, sizing keywords, and non-finite values while preserving valid negative lengths.

Scroll-padding exposed the px-storage length-percentage variant: a property may store resolved px while its grammar is still `auto | <length-percentage [0,∞]>`. Validate the parsed source value before conversion so percentages remain valid, negative values fail, and shared-parser aliases or non-finite values cannot overwrite the old computed value.

Contain-intrinsic-size exposed the optional-prefix longhand variant: `auto? none | <length [0,∞]>` needs a two-state consumer result, not just `Option<LengthValue>`. Strip optional `auto` only at token boundaries, preserve `none` as a real clearing value, and validate the length branch before writing any physical or logical intrinsic-size field.

Transform and perspective origins exposed the `<position>` consumer variant: accepting only shared length tokens is both too narrow for legal keywords and too wide for parser aliases. Parse the origin as horizontal/vertical roles, reject same-axis keyword pairs, and validate length-percentage tokens from the raw source before mutating either axis.

Text and box shadows exposed the per-slot length variant: a shadow uses `<length>` tokens, but offset/spread and blur have different negativity rules. Validate every raw token after shared parsing, reject percentages and parser aliases everywhere, allow negative offsets and box spread, and reject negative blur before style-system can overwrite the old shadow list.

Background-position exposed the role-based `<position>` length variant: the parser must validate raw length tokens before assigning them to single, two-value, or edge-offset roles. Preserve finite positive and negative lengths plus finite percentages, but reject shared-parser aliases and non-finite values before style-system resolves them to computed coordinates.

Clip-path exposed the basic-shape slot variant: one property can contain both signed coordinate slots and non-negative radius slots. Route every shared parsed length through a slot-aware validator, preserving negative inset/polygon/position values while rejecting negative radii, parser aliases, sizing keywords, and non-finite values before the computed clip-path is mutated.

Border-spacing exposed the inherited-table length-only variant: inherited table spacing still needs the parser boundary to enforce `<length [0,∞]>` before direct apply writes computed px fields. Reject percentages, parser aliases, sizing keywords, negative values, and non-finite values at the shared parser edge so invalid declarations preserve the previous spacing pair.

Border-image width and outset exposed the suffix-gated length variant: checking only `px|em|rem` before shared parsing narrows a valid `<length>` grammar and rejects viewport or font-metric units. Parse candidate length tokens first, then apply a raw-token non-negative length validator before falling back to unitless number parsing.

Overflow-clip-margin exposed the optional visual-box length variant: once a parser has consumed the optional box keyword, the remaining value still needs the property's non-negative `<length>` grammar rather than the broader shared length parser. Validate the raw length token before filling the defaulted length slot so percentages, aliases, keywords, negative values, and non-finite values cannot overwrite the previous computed margin.

Text-underline-offset exposed the inherited text-decoration offset variant: a property may allow signed lengths and percentages while still needing to reject shared-parser keyword aliases and non-finite values. Keep `auto` on its keyword branch, then validate the raw `<length-percentage>` token before writing the inherited computed offset.

Text-decoration-inset exposed the two-value text-decoration inset variant: even when both slots allow signed offsets and percentages, each slot still needs the same raw-token validation before expansion. Validate every token before cloning or assigning the start/end pair so one invalid alias cannot partially overwrite the computed decoration inset.

Text-decoration-thickness exposed the used-value text-decoration thickness variant: a parser that stores only px both rejects legal relative/percentage/calc values and can still admit shared-parser aliases such as `thin`. Store the specified `<length-percentage>` as `LengthValue`, validate the non-negative grammar at parse time, then resolve against font size only at paint/CSSOM used-value boundaries.

Background-size exposed the suffix-gated image sizing variant: a helper can already call the shared length parser yet still narrow the accepted grammar by matching only `px|em|rem` afterwards. Keep the raw-token non-negative validator, but enumerate every real length unit the shared parser can return before mapping into the current computed size representation.

Translate exposed the shared-number transform variant: a helper reused by angle, scale, matrix, and translate cannot be widened with extra length units without admitting invalid angles. Give translate its own length parser, then keep angle/scale/matrix on the narrower numeric parser.

Translate3d exposed the sibling transform variant: after creating a dedicated translate length parser for 2D translate, route the 3D translate length slots through the same helper instead of leaving them on the generic number parser. Keep the model distinction explicit because z accepts length, not percentage.

Perspective exposed the positive-length transform variant: transform helpers often share numeric parsing, but perspective needs a positive length grammar rather than an angle/scale/matrix number grammar. Reuse the transform length parser, then layer the strict positivity check at the perspective boundary.

Filter exposed the px-only helper variant: a function can be named for the computed storage unit yet still must accept every real `<length>` unit at parse time. Reuse the shared length parser, then reject percentages and shared-parser aliases at the filter boundary before collapsing into the existing numeric representation.

Background-position exposed the partial-enum length variant: after switching a property to store `LengthValue`, every downstream validator must enumerate all real units the shared parser can return. A comment that says "any length unit" is not enough; add representative tests for metric-relative units such as `ex` and root-relative variants such as `rch`.

Shadow apply exposed the computed-conversion variant: parser-level grammar parity is incomplete if the style-system collapses every non-px `LengthValue` to zero while building computed values. Reuse the same length resolver at the apply boundary so legal relative units survive into paint-facing numeric storage.

Border-spacing exposed the inherited table-spacing instance of the same computed-conversion variant: once a converter exists for effect lengths, route other paint-facing numeric storages through it instead of repeating `Px else 0.0` matches.

Border-image exposed the weak-direct-apply-test variant: an apply test that only checks `true` can miss lossy conversion from a parsed `LengthValue` into a default computed number. For properties with tagged computed components, assert the exact computed variant as well as the success boolean.

Scroll offsets exposed the shared-helper variant: a helper named generically enough to serve multiple properties can still encode an old `Px else 0.0` shortcut. When widening it, keep context-free values such as percentages at the existing behavior unless the computed storage can represent them.

Perspective paint exposed the split-consumer variant: apply can preserve a correct `LengthValue` while a later paint gate still pattern-matches only `Px`. Audit both the outer "should paint" predicate and the inner used-value computation; either one can silently drop legal relative units.

Word-spacing exposed the metric-override variant: layout can compute the right run metric while paint Path B loses it when rerunning IFC with empty styles. Any spacing-like metric used by IFC must have both a layout-side resolved value and a paint-side override map, not just a style-system field.

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

Line-height exposed the canonicalization fallback variant: even when the normal cascade computed pass usually converts relative `<length-percentage>` values to `Px`, layout used-value helpers must still handle legal residual `LengthValue` variants. Direct apply, tests, and alternate style entry points can bypass early canonicalization; falling back to `normal` silently loses valid `em` and percentage line-height values.

Letter-spacing exposed the same split at paint/layout fallback boundaries: the normal cascade may canonicalize `em` to `Px`, but direct paint fallback, IFC collection, and inline-block remeasure can still see the original `LengthValue`. Spacing-like consumers should resolve legal residual lengths against the current fragment font-size instead of pattern-matching only `Px`.

Column-rule-width exposed the paint-helper variant: even when style-system computed has a special canonicalization pass for an enum-wrapped length, direct paint helpers can still receive residual `LengthValue` variants. Paint should resolve the enum payload at the draw boundary instead of assuming computed storage is always `Px`.

Outline exposed the naked paint-helper variant: even for plain `LengthValue` fields with computed canonicalization, direct paint and residual computed storage can still reach paint as `em`/`ch`/viewport units. Effects and borders should resolve outline-like lengths at the draw boundary instead of routing them through px-only convenience helpers.

Border-radius exposed the box-dependent variant: a paint helper can correctly handle percentages because it has box geometry, yet still drop residual real lengths by falling back to a px-only convenience helper. Geometry-aware resolvers should combine box-relative percentage handling with font-relative length resolution at the same paint boundary.

Multicol exposed the duplicate-resolver variant: layout and paint both had a `length_to_px` helper for column geometry, but paint supported fewer real length units. When paint consumes geometry-sensitive CSS values for adornments such as column rules, keep its resolver unit set byte-aligned with layout or route through the layout helper directly.

Clip-path exposed the context-length variant: a shape resolver may handle `em` correctly only if the font-size context itself was resolved correctly. Audit both the value being resolved and every contextual basis such as font-size, viewport, or box dimension.

Transform-origin exposed the geometry transform variant: helpers that already have the border-box size can still lose residual font-relative origins if they special-case only `%` and `px`. Resolve the origin length at the matrix construction boundary, using the current `font-size` as context and keeping invalid residual keywords on the existing center fallback.

Clip rect exposed the legacy-property variant: deprecated paint paths are still real consumers of computed CSS values. When a legacy property accepts `<length>`, its paint boundary needs the same residual length resolution as modern properties; keep unsupported grammar residues on the historical fallback instead of broadening the property.

Radial gradients exposed the helper-context variant: a generic conversion helper may look value-only, but CSS Images length values still depend on the element font context. Keep a default-context wrapper for tests and pure callers, then route production paint through a context-aware entry point.

Gradient color stops exposed the nested-helper-context variant: fixing a gradient geometry resolver does not automatically fix nested stop positions. When adding a context-aware paint entry point, audit every helper it calls for residual length conversion instead of stopping at the outer geometry fields.

Text-indent exposed the shared-inline-resolver variant: even when layout and paint share one resolver, that resolver can still lag behind the accepted grammar. For `<length-percentage>` text metrics, keep the percentage basis explicit and send every real length unit through the same style-system used-value resolver.

Inline visual metrics exposed the box-expansion variant: geometry sync code for inline backgrounds and borders may consume padding/border widths outside the normal Taffy conversion path. Resolve residual real lengths at that sync boundary with the element font context, and leave percentages unresolved when the helper has no containing-block basis.

Absolute positioning exposed the postprocess-CB variant: fixes that correct Taffy containing-block coordinates must not assume the inset is already `Px`. When a postprocess path reinterprets CSS offsets against a viewport or root CB, resolve residual real lengths with the element font context at that same boundary while leaving existing percentage and auto branches explicit.

Root abspos exposed the parallel-postprocess variant: a viewport-CB fix can leave a root-CB sibling path behind. When two postprocess functions reinterpret the same CSS inset semantics against different containing blocks, share the real-length resolver and add one direct test per CB path.

Fixed positioning exposed the stretch-postprocess variant: position correction and size stretch can live in separate postprocess functions for the same CSS inset semantics. After adding a real-length resolver for fixed or absolute coordinates, audit opposing-inset auto-size stretch paths before considering the CB family covered.

Viewport abspos exposed the coordinate-vs-stretch split: resolving `left` and `top` used values does not cover `width:auto` or `height:auto` stretch from opposing insets. For positioned layout, audit coordinates and auto-size equations as separate consumers even when they share the same containing block.

Root self positioning exposed the top-level-special-case variant: root elements often bypass child traversal helpers. When a child-positioning bug is fixed in viewport or root-CB postprocess code, audit root self helpers separately for both position and auto-size equations.

Abspos auto-margin centering exposed the equation-consumer variant: positioned layout does not only consume inset values for coordinates and stretch. Any pass that re-solves a CSS positioning equation, such as vertical `margin:auto` centering, must resolve residual real lengths with the same font and viewport context while keeping percentage basis explicit.

Root-CB abspos stretch exposed the sibling-consumer variant inside a single postprocess function: fixing coordinates in a containing-block correction pass does not fix auto-size stretch in that same pass. Audit every branch that consumes the same inset pair, not just every function.

Abspos max-width clamp exposed the constraint-consumer variant: after stretch is fixed, min/max constraints and auto-margin redistribution can still re-read the original inset and size values. Constraint passes must resolve residual real lengths for every operand in the equation, including `max-width`, before computing leftover margins.

Vertical table sizing exposed the direct-helper variant: full DOM+style pipelines may canonicalize author lengths before layout, while table or layout unit tests can pass residual `LengthValue` directly into a helper. If a helper documents that it consumes a CSS property as a used value, give it a local resolver instead of relying on upstream canonicalization.

Collapsed border conflict exposed the source-context variant: a single equation can compare lengths from table, row group, row, and cell styles. Resolve each residual real length with the font context of the style that owns that border, not a hard-coded root font size or the table's font context.

Table shrink-to-fit exposed the constraint-replay variant: post-layout shrink helpers may replay `width`, `min-width`, `max-width`, and `height` constraints after taffy. Direct helper paths need their own used-value resolver for residual real lengths while leaving percentage values in the original upstream/taffy path.

Regular table sizing exposed the paired-row-and-wrapper variant: table postprocess can consume the same sizing properties both for row height distribution and for final wrapper min/max constraints. A resolver added for shrink-to-fit tables must be mirrored in the normal table constraint path, with percentages kept on the existing taffy/upstream route.

Table column sizing exposed the explicit-column variant: column width calculation replays `width`, `min-width`, and `max-width` from cells, cols, and colgroups before final table constraints run. Fixing wrapper min/max is not enough; explicit column freezing and auto-column redistribution need the same residual real-length resolver.

Table col prepasses exposed the sibling-prepass variant: a later column sizing helper can have an earlier collection pass that consumes the same CSS width semantics. When fixing a main table width resolver, audit pre-pass collectors for independent `Px/%` branches so col/colgroup declarations do not diverge from cell declarations.

Table min-width redistribution exposed the post-constraint-sync variant: expanding the table wrapper to `min-width` does not automatically expand the column vector consumed by cell positioning and painting. When a layout object has both wrapper constraints and internal track/column arrays, resolve residual real lengths at both consumers.

Intrinsic sizing exposed the shared-measurement variant: leaf max-content fallbacks are consumed by flex, grid, table-cell, and shrink-to-fit callers. These helpers must resolve residual real lengths locally rather than assuming direct `ComputedStyle` callers always pass canonical `Px` widths.

Flex intrinsic sizing exposed the priority-preservation variant: `flex-basis` can override `width`, so adding residual length support only to the width fallback is insufficient. Each higher-priority sizing source needs the same resolver or direct callers silently fall through to lower-priority values.

Table-float avoidance exposed the geometry-decision variant: a residual real length may not directly paint, but it can decide whether a BFC/table fits beside a float. Definite-size fit tests must resolve the declared source value before comparing against available float-side space, or the element is silently treated like auto-size.

BFC float avoidance exposed the cached-used-declaration variant: postprocess stages sometimes need the original definite source width even after taffy has produced a fallback geometry. Store a local used-value cache on the layout box for the specific decision instead of globally changing converter semantics for every consumer.

Right-float BFC avoidance exposed the entry-guard variant: adding a used-value cache is not enough if the branch guard still asks the post-taffy fallback width whether the box overlaps the float. Every gate that decides whether avoidance runs must consume the same declared used width as the later fit/pushdown logic.

Multi-float coordination exposed the feasible-region variant: candidate placement solvers must use the used border-box width when computing x intervals, not only when finally restoring geometry. A raw residual length can make an infeasible gap look feasible and lock in the wrong width.

Inline-block float avoidance exposed the atomic-inline variant: terminal geometry passes that adjust an inline-block position must also restore its declared used width and content width. Fixing only the x/y coordinate can leave the atomic box visually shifted but still sized from a raw residual unit.

Infeasible multi-float placement exposed the no-op trap: "do not move the box" must not mean "leave every fallback geometry untouched." If a branch intentionally preserves position, it still needs to restore cached used dimensions before paint and hit-test consumers read the final layout tree.

Replaced-element intrinsic ratio sizing exposed the pre-taffy variant: if the tree builder computes an auto side from a specified side before final layout, it must resolve the specified side's real length first and write that used size back to taffy. Otherwise both the explicit side and the ratio-derived side inherit the converter's raw residual number.

Ratio-only replaced sizing exposed the containing-size variant: fallback/default object sizing may still consult a definite parent size. That parent value is another CSS used-value boundary and must be resolved with the parent style context, not treated as usable only when already stored as px.

Flex transferred auto-min exposed the cross-axis-replay variant: replaced flex item minimum sizing can replay parent cross size, item cross constraints, cross padding, and specified main size after the main style conversion path. Resolve each definite real length with the style that owns that operand before feeding the transferred-size equation; otherwise residual padding can make a content-box suggestion too large.

Strict classic script execution exposed the wrapper-scope trap: catching page-script throws by wrapping raw classic code in `try { ... }` changes top-level function and lexical declaration visibility. Use a sentinel plus indirect global `eval` when strict runners need exception reporting without changing classic script scope.

Flex aspect-ratio post-fixup exposed the second-pass replay variant: a first-pass taffy result can contain raw residual-unit geometry, and later fixups may reuse parent cross size or min-size gates from CSS values. Resolve those operands again in the post-layout pass instead of trusting either the raw taffy geometry or `Px`-only style matches.

Late min/max block-size clamps exposed the sibling-flow variant: if a post-taffy pass changes an in-flow block child's outer extent, later normal-flow block siblings must be shifted by the same delta. Resolving residual `min-height` without propagating that delta leaves the box size correct but the rendered flow order stale.

Relative inset postprocessing exposed the raw-delta variant: when taffy already applied a residual real length as its raw numeric value, a later used-value fix must add only `resolved_px - raw_value`. Reapplying the full resolved length would double-count the raw offset, while touching inline relative without a dedicated test risks crossing into the separate IFC offset path.

Root compensation paths are also used-value boundaries. When the engine manually replays root-level behavior that taffy cannot model, such as viewport-relative fixed margins, resolve real lengths against the root style before applying the offset; do not leave those root-only fixups as `Px`-only exceptions.

Form-control layout fixups can hide font-size used-value boundaries. If a postprocess pass estimates native-control ascent/descent from `font-size`, resolve residual real lengths first; otherwise direct `ComputedStyle` callers get the fallback constant even though the same style would be definite after normal cascade.

IFC font metrics are also a used-value boundary. Helpers that derive both the run font size and normal line height from `ComputedStyle::font_size` must resolve residual real lengths before computing metrics; otherwise direct style callers silently fall back to the default 16px and every dependent line box becomes too small.

IFC container struts are a separate font-size used-value boundary from text run metrics. Even after run font-size resolution is fixed, the block container font-size used for atomic-only line baselines must reuse the same resolver; otherwise residual units such as `ch` leave the strut at 16px and make baseline alignment too shallow.

IFC atomic inline-block dimensions are another independent used-value boundary. When layout sizes are not pre-seeded, `collect_inline_items` consumes CSS `width`/`height` directly; its `em` base must be the resolved element font-size, and intrinsic or percentage values must keep the old fallback until a containing-block basis is available.

Empty leaf measurement fallback is another residual length consumer. When `measure_text_content` has no inline content and falls back to CSS `width`/`height`, it must resolve real lengths against the element font size; otherwise flex/grid measurement callbacks lose explicit non-`px` sizes even though the normal taffy style path can represent them.

Inline multicol auto-fill height budgets are another post-IFC used-value boundary. When stored column fragmentation replays `height`/`max-height` to decide the per-column fill budget, resolve definite real lengths against the container font size; otherwise residual units fall back to full-width content height and prevent expected column breaks.

`tab-size` has two unit domains: numeric values are space-count multipliers, while length values are already tab-stop distances. Keep that distinction in the IFC configuration; passing a resolved length through a field interpreted as a multiplier silently applies the space advance a second time, and Path A/Path B must share the same conversion.

Table intrinsic helpers can duplicate font-size used-value logic outside the IFC path. Column min-content floors and vertical table growth that estimate text width must resolve residual `font-size` values through the style-system resolver; otherwise direct styles with `em`/`ch` silently fall back to 16px and under-size table tracks.

Flex intrinsic sizing has its own container-level spacing boundary. When summing flex row max-content widths, resolve the container `gap` with the same definite real-length helper used for item bases; otherwise direct residual `gap` values disappear even though item sizes are correct.

Flex intrinsic aspect-ratio transfer has a separate item-level size boundary. When `width:auto` items derive base width from `height` or `min-height`, resolve those definite real lengths before applying the ratio; otherwise residual cross sizes fall back to content max-content and erase the transferred base.

Grid intrinsic sizing has the same container spacing used-value boundary as flex, but through `column_gap`. Keep grid/flex intrinsic gap resolution on the shared definite real-length helper so direct residual gaps do not disappear in max-content calculations.

Multicol intrinsic sizing also consumes author lengths outside the normal multicol layout pass. Resolve `column-width` and `column-gap` through the shared definite real-length helper before computing column-driven max-content, or direct residual units will either disappear or be replaced by child content widths.

Multicol column-info parsing has a font-size used-value boundary separate from its length conversion helper. Even when `length_to_px` supports `em`, the font-size basis itself must be resolved first; otherwise residual `font-size` units make `column-gap:normal/1em` and `column-width:<length>` use the root fallback.

Inline item collection must resolve inline margins at the IFC boundary. Text runs, inline-blocks, and inline replaced elements can still carry residual real lengths in direct `ComputedStyle`; assuming margins are already `Px` drops `em/ch/rem` spacing before line breaking and paint overrides are stored.

Abspos table recenter has a separate definite-inset gate after table layout. Treating only `Px` offsets as definite skips the CSS auto-margin centering equation for residual real lengths; reuse the table used-length resolver for the gate while keeping `auto`, percentages, and intrinsic sizing as non-definite there.

Inline-block metric reuse can re-enter margin math after the initial IFC item collection. When refreshing reused atomic inline boxes, resolve vertical margins through the same IFC used-value helper as collection; otherwise the second pass can erase residual `em/ch/rem` margin contribution to baseline and line height.

Flex base sizing has a width-before-content used-value boundary. In the `flex-basis:auto` path, a definite `width` must be resolved before falling back to max-content; otherwise residual real widths can be skipped and wide text content incorrectly becomes the flex base.
