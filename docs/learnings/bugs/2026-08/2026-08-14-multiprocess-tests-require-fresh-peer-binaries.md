---
date: 2026-08-14
modules: protocol, browser, renderer, compositor, test tooling
---

# Multiprocess tests require fresh peer binaries

## Problem

After adding fields to `PaintSnapshotParams`, workspace tests compiled successfully but browser multiprocess tests spawned stale `target/debug/zero-renderer` and `target/debug/zero-compositor` binaries. The renderer disconnected with broken pipes, while the compositor could retain stale frames. Several tests then timed out far from the protocol change.

## Root Cause

`cargo test --workspace` builds test harness executables under `target/debug/deps`, but it does not guarantee that standalone peer binaries in `target/debug` are rebuilt. Browser unit tests resolve and spawn those standalone binaries. A prior build therefore used an older bincode schema than the current browser test process.

One scrolling test also depended on the stale renderer producing a document taller than the viewport. The fresh renderer produced a 794px document in an 814px viewport, so the requested scroll correctly clamped to zero.

## Solution

Build `zero-renderer` and `zero-compositor` through `test-guard` before the workspace test matrix. Multiprocess tests must construct their own behavioral preconditions; the scroll matrix now appends a fixed-height spacer instead of depending on platform font metrics or historical layout output.
