# Multi-process GUI tests must rebuild the renderer child

- Date: 2026-08-12
- Modules: `apps/browser`, `apps/renderer`, GUI integration tests

## Problem

The form interaction integration test reported that IME preedit rendered but a Chinese commit never published a frame. The same preedit-to-commit sequence passed inside the renderer.

## Root cause

`cargo test -p zero-browser` builds the browser test harness, but the multi-process backend launches `target/debug/zero-renderer.exe`. A prior `cargo check -p zero-renderer` or renderer unit-test build does not refresh that production executable. The test therefore exercised an older renderer that did not understand the new IME protocol message.

## Solution

Use `scripts/test-form-interaction.ps1`. It builds `zero-renderer` first, then runs the guarded browser integration test. The test covers physical pointer coordinates at 1.0, 1.25, 1.5, and 2.0 scale factors, two text controls, a button, IME preedit, and Chinese commit.

## Prevention

Any browser test that spawns a production child process must explicitly build that child executable in its test entry point. Do not treat `cargo check` or a child crate's unit-test harness as an equivalent artifact.
