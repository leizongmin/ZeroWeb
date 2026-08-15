# ZeroWeb runtime environment variables

This document is the user-facing index for runtime environment variables. Their
names, defaults, and common parsing helpers are owned by
[`zero-runtime-config`](../crates/runtime-config/src/lib.rs). Add a product-level
switch there before consuming it in an application or library crate.

## Product switches

| Variable | Values | Default | Effect |
| --- | --- | --- | --- |
| `ZEROWEB_RENDERER` | `auto`, `gpu`, `cpu` | `auto` | Select the rendering backend; auto prefers GPU then falls back to CPU. |
| `ZERO_BROWSER_MULTIPROCESS` | `0`/`false` disables | enabled | Run each tab in the renderer process. |
| `ZERO_RENDERER_PATH` | executable path | auto-discovery | Override the `zero-renderer` executable. |
| `ZERO_PRIVATE` | `1`/`true` enables | disabled | Do not write the HTTP disk cache. |
| `ZERO_CACHE_DIR` | directory path | platform cache directory | Override the HTTP disk-cache directory. |
| `ZERO_HTTP2` | `0`/`false` disables | enabled | Force HTTP/1.1 and omit HTTP/2 priority behaviour. |
| `ZERO_NOPROXY` | `1`/`true` enables | disabled | Bypass system and `HTTP[S]_PROXY` environment proxies. |
| `ZERO_MAX_CONNECTIONS_PER_ORIGIN` | positive integer | `6` | Per-origin connection limit. |
| `ZERO_MAX_CONNECTIONS_TOTAL` | positive integer | `24` | Global concurrent-request limit. |
| `ZERO_BROWSER_COLOR_SCHEME` | `dark`, `light` | system | Override `prefers-color-scheme`. |
| `ZERO_BROWSER_UI_LANG` | locale beginning `zh` or `en` | locale, then English | Override browser UI language. |
| `ZERO_SCROLL_BLIT` | `0` disables | enabled | Disable retained-frame scroll blitting. |
| `ZW_RENDER_THREAD` | `0` disables | enabled | Disable the persistent CPU rendering worker. |
| `ZW_IMAGE_DECODER_PROCESS` | `0`/`false` disables | enabled | Decode raster images in `zero-image-decoder`. |
| `ZW_IMAGE_DECODER_BIN` | executable path | auto-discovery | Override the image-decoder executable. |
| `ZW_COMPOSITOR_PROCESS` | `0` disables | enabled | Enable the compositor client process path. |
| `ZW_COMPOSITOR_BIN` | executable path | auto-discovery | Override the compositor executable. |
| `ZW_COMPOSITOR_ASYNC_SCROLL` | `0`/`false` disables | enabled | Enable compositor asynchronous scrolling. |
| `ZW_COMPOSITOR_UI_FRAMES` | `0`/`false` disables | enabled | Submit browser UI frames to compositor. |
| `ZW_COMPOSITOR_PRESENT` | `0`/`false` disables | enabled | Disable compositor Viz present. |
| `ZW_COMPOSITOR_OWNED_PRESENT` | `0`/`false` disables | enabled | Disable compositor-owned final window present. |
| `ZW_COMPOSITOR_GPU` | `0`/`false` disables | enabled on Linux | Compositor headless GPU rasterization. |
| `ZW_COMPOSITOR_GPU_IMAGE` | `0`/`false` disables | enabled on Linux | GPU shared-image metadata channel. |
| `ZW_COMPOSITOR_GPU_TEXTURE_EXPORT` | `0`/`false` disables | enabled on Linux | GPU dma-buf texture export. |
| `ZW_BROWSER_GPU_DMABUF_IMPORT` | `0`/`false` disables | enabled on Linux | Browser GPU dma-buf import. |
| `ZW_COMPOSITOR_SHM` | `0`/`false` disables | enabled on Linux | Linux POSIX shared-memory frame transport. |
| `ZW_COMPOSITOR_GPU_ZERO_COPY` | `0`/`false` disables | enabled on Linux | Linux shared-image mmap zero-copy consumption. |
| `ZW_COMPOSITOR_SCROLL_TRANSFORM` | `1`/`true` enables | disabled | Apply scrolling in compositor (pixel bake; scrolls larger than one frame exceed the baked content). |
| `ZW_RENDERER_SECCOMP` | `0`/`false` disables | enabled | Enable renderer seccomp sandbox hook. |
| `ZW_COMPOSITOR_SANDBOX` | `0`/`false` disables | enabled | Enable compositor environment sanitization. |
| `ZW_COMPOSITOR_SECCOMP` | `0`/`false` disables | enabled | Enable compositor seccomp filtering on Linux. |
| `ZW_COMPOSITOR_LANDLOCK` | `0`/`false` disables | enabled | Enable compositor Landlock on Linux. |

## Compatibility and test switches

The `ZW_*` switches in CSS, layout, painting, and text modules are temporary
compatibility kill-switches used to bisect WPT/reftest changes. They are not a
stable host-facing configuration API. Existing semantics are preserved:

- most switches are default-on and use `=0` to return to the older path;
- opt-in experiments use `=1`;
- `REFTEST_*`, `ORACLE_*`, `LAYOUT_DUMP`, `R109_DBG`, and `INTRINSIC_DBG` are
  test or diagnostic controls only.

When an experimental switch becomes a supported runtime option, move its
parsing and registry entry into `zero-runtime-config` and add it to the table
above. Do not add new product-level `std::env::var` calls outside that crate.

## Platform variables

`WINIT_UNIX_BACKEND`, `WAYLAND_DISPLAY`, `DISPLAY`, and
`WINIT_X11_SCALE_FACTOR` are supplied by the windowing platform. Standard proxy
variables such as `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and `NO_PROXY` are
honoured unless `ZERO_NOPROXY=1` is set.
