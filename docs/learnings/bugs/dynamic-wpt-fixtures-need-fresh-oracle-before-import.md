# Dynamic WPT fixtures need a fresh Oracle before import

- Date: 2026-08-13
- Modules: `tests/wpt-runner`, `engine`

## Problem

Five `font-variant-*` reftests were reported as exact or near-exact Chromium matches while their
shared JavaScript generators and GSUB test font were missing. Both Chrome capture and ZeroWeb
rendered nearly empty pages, so the apparent passes did not exercise font-variant behavior.

## Root cause

The fixture needs two JavaScript files, a shared stylesheet, and `gsubtest-lookup3.otf`. Restoring
the resource closure generated large feature tables and invalidated the old screenshots. Against
fresh Chrome captures, the missing-resource ZeroWeb baseline totaled 13.01 percentage points,
while the complete fixture totaled 28.09 percentage points. All five cases regressed. The complete
self-source results also included mismatches for ligatures (7.08%) and numeric variants (5.52%).

## Solution

For script-generated WPT pages, verify the complete transitive resource closure before treating an
Oracle shot as evidence. After restoring resources, recapture Chrome first, then compare ZeroWeb
with resources disabled and enabled against that same fresh Oracle. Do not permanently import the
fixture until the generated DOM, shaping features, and test font path are jointly net-positive.
