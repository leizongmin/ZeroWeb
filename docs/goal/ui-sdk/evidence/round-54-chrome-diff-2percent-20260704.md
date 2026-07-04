# Round 54 evidence — DC-14 chrome pixel diff ≤2%

**Date**: 2026-07-04

## Changes

1. **Window control icons (min/max/close)** — BrowserTabStripWidget 窗口控制按钮区添加 3 图标:
   - minimize: 10px 横线 fill_rect
   - maximize: 10px 空心方块（4 条边 fill_rect）
   - close: X 图标 (SVG, WC_ICON_CLOSE ImageRef(6), draw_image 着色)
   - Added `ChromeTabColors.window_icon` field
   - chrome diff 2.17% → 2.14%

2. **Trailing spacer (address bar width parity)** — DesktopBrowserShell toolbar 加 88×44 trailing spacer:
   - 匹配手绘 trailing_reserved (padding 10 + gap 8 + download 32 + gap 8 + theme 32 + gap 8 = 98; menu=42, total=130)
   - SDK only had menu(42), missing 88px → address flex 88px over width → addr-r diff main cause
   - Unregistered empty container node with min_width/max_width/min_height/max_height constraints
   - chrome diff 2.14% → **1.99%** (crossed ≤2% threshold)

## Key bug fix

`child_constraints_from_props` in `ui/runtime/src/host.rs` reads `"min_width"`/`"max_width"` (not `"child_min_width"`/`"child_max_width"`). Initial implementation used wrong prop names → constraints not applied → spacer filled available 1106×760 → toolbar height 760 → viewport height 0. Fixed by using correct prop names.

## Test results

| Test | Result |
|------|--------|
| `cargo build --workspace` | ✅ |
| `cargo clippy -D warnings` (chrome + browser sdk-chrome) | ✅ |
| `cargo fmt --check` | ✅ |
| chrome crate tests | 87/87 ✅ |
| browser sdk-chrome tests | 205/205 ✅ |
| browser default tests | 191/191 ✅ |
| `dc14_chrome_region_pixel_diff_baseline` | chrome 1.99%, page 0.00% ✅ ≤2% |

## Pixel diff baseline

| Region | R53 | R54 (window icons) | R54 (trailing spacer) |
|--------|-----|---------------------|----------------------|
| chrome (y<112) | 2.17% (3106/143360) | 2.14% (3075/143360) | **1.99% (2847/143360)** |
| page (y≥112) | 0.00% | 0.00% | 0.00% |

## Commits

- `7e8635c2` — DC-14 window control icons (min/max/close): chrome diff 2.17% → 2.14%
- `1819d697` — DC-14 trailing spacer: address bar width parity, chrome diff 2.14% → 1.99% (≤2%)

## Remaining items

- DC-11 text path unification (SDK draw_text vs hand-drawn GlyphDraw — placeholder text + tab labels)
- DC-15 mobile platform first frame (needs device/HarmonyOS/Android)
- DC-2 real EventLoop::run blocking shell (needs GUI)
- DC-8 real platform a11y backend (needs platform)
