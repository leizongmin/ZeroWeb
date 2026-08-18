---
date: 2026-08-13
modules: render-foundation, browser
---

# wgpu 30 Window Surface Requires a Display Handle

## Problem

The browser accepted `--renderer=gpu`, but every frame logged a surface creation failure and retried GPU initialization. CPU and headless GPU tests did not expose the problem.

## Root Cause

After upgrading to wgpu 30, the window renderer still created `InstanceDescriptor` with `display: None`. Windowed GLES presentation requires the winit display handle to be supplied when the instance is created. Headless rendering does not.

## Solution

Pass an owned clone of the winit window as the instance display handle in `GpuRenderer::new_for_window`. Keep the headless instance display handle unset.

GPU smoke tests must fail when window surface initialization or GPU readback fails. They must not silently accept CPU fallback.

## Verification

Run a real windowed browser under an X11 display and require both:

1. `GPU renderer initialized` before page capture.
2. A strict GPU readback screenshot from the browser production scene.

Headless-only GPU tests are insufficient because they never create a window surface.
