# DC-13 产品静态 smoke — WinterTC 首页 fixture（图片密集）

**日期**: 2026-06-16
**源页面**: `https://wintertc.org/`（Ecma WinterTC 技术委员会首页）
**fixture**: `apps/browser/assets/wintertc/`（index.html = chromium 已解析 DOM 含内联 Twind `<style>` + static/ 下 14 个 logo 资源）
**渲染模式**: ZeroWeb CPU 软件渲染（800×600，base_dir 加载图片）vs headless Chromium（800×600）
**工具**: `zero-wpt-runner product-smoke <html> --base-dir <dir> --oracle <png> --out <png>`（本轮新增，可复用 DC-13 渲染+对比）

## 录制资源

| 资源 | 类型 | 状态 |
|------|------|------|
| index.html | chromium 已解析 DOM（含 Twind 生成的 `<style>`） | ✅ 经 capture-wintertc.mjs 录制 |
| static/logo.svg | hero WinterTC logo | ✅ |
| static/logos/{cloudflare,deno,fastly,netlify,nodejs,shopify,suborbital,vercel,azion,matrix}.svg | 10 个 SVG 参与方 logo | ✅（SVG 栅格化本轮新增） |
| static/logos/{alibaba,bytedance,igalia}.png | 3 个 PNG 参与方 logo | ✅ |

## 像素差距（vs Chromium）

| 视口 | 像素 diff |
|------|-----------|
| 800×600 | **22.42%**（107,604 / 480,000 px） |

## 诊断（REFTEST_DEBUG primitive dump）

- `images: 1`（800×600）、`images: 2`（800×2000）—— 图元数随视口增高略增，证明 **SVG 栅格化已生效**（logo.svg hero + 个别被布局的 logo 现渲染）。
- 但 14 个 logo 中绝大多数仍未渲染——根因**非图片加载**（PNG/JPEG/SVG 三类均能在 build_image_cache 加载），而是 **Twind 驱动的 logo 网格布局**未被 ZeroWeb 完整布局（flex/grid 工具类 + gap 等），多数 `<img>` 元素未被布局到可见区域 → 不产生 ImagePrimitive。
- 800×600 视口下参与方 logo grid 多在折叠线以下，主要差异来自**顶部 hero/nav/title 文本区**（Twind 工具类布局 + fontdue vs Skia 字体度量噪声，与 morning.work/welcome 同源）。

## 本轮交付

1. **`product-smoke` 子命令**（tests/wpt-runner/src/main.rs）：通用 DC-13 产品 fixture 渲染+对比工具，输出 ZeroWeb CPU PNG 并与 chromium Oracle PNG 像素 diff。后续 welcome/morning-work/wintertc 及任意 fixture 均可复用。
2. **SVG 文件栅格化**（reftest.rs `load_svg_file`）：resvg + tiny-skia 把 `<img src="*.svg">` 栅格化为 RGBA，补齐 `build_image_cache` 此前仅 PNG/JPEG 的缺口（wintertc 14 logo 中 11 个为 SVG）。
3. **capture-wintertc.mjs**：puppeteer 录制 live 页面为已解析 fixture（含 Twind `<style>`）+ chromium Oracle 截图。
4. **wintertc fixture + 基线 22.42%**（首个图片密集真实页面 DC-13 数据点）。

## 剩余差距（下轮）

- **Twind 工具类布局**（logo grid 的 flex/grid/gap 不完整）——ZeroWeb 对 Tailwind utility class 驱动的布局支持不足，致多数 logo 未被布局。需诊断具体缺哪个布局特性（同 R109 IFC/inline→block 谱系，或 flex/grid gap）。
- 顶部文本区 fontdue vs Skia 度量噪声（与 morning.work/welcome 同源，非单点修）。
- 真实 ZeroBrowser 路径（非 wpt-runner harness）的 SVG/图片加载——本轮仅 harness 侧（build_image_cache）；浏览器/webview 层 ImageCache 的 SVG 支持待同步。
