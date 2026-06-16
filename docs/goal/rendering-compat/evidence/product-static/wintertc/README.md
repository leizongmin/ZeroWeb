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
| 800×600 | **25.11%**（120,546 / 480,000 px） |

> 注：绝对路径图片加载修复（见下）后 diff 从 22.42% **升**至 25.11%——这是**诚实上升**：修复前 14 个 logo 仅 2 个加载（站点根相对 URL `/static/x` 因 `base.join(absolute)` 替换 base → 文件系统根 → 加载失败），参与方 logo 全缺失致 diff 人为偏低（假低）；修复后 14 logo 全加载并渲染到可见区（y≈444-600），暴露参与 `flex flex-wrap` 容器的布局 bug（logo 错位/重叠），diff 真实上升。DC-14 anti-false-pass 原则：诚实测量优先于人为偏低的假通过。**下一目标 = 修复参与 flex 布局，diff 将随之下降（甚至低于 22%，因 logo 正确 + 此前改进叠加）**。

## 诊断（REFTEST_DEBUG primitive dump + 分区域像素分析 + 计算样式探针）

**布局经核实基本正确**（CSS 已加载后探针）：universal 选择器 `*,::before,::after{margin:0}` 生效（body/h1/p/ul margin 均被 reset 为 0，UA 默认 margin 正确覆盖）；Twind utility 类生效（`text-4xl`→h1 font=36px、`text-xl`→p font=20px、`flex`/`grid`/`mt-8`/`gap-*` 等结构正确）；hero `<section>`（logo 96×96 + 标题）、nav `<ul class="mt-8 grid-cols-4">`（y≈160）位置合理。

- `images: 1`（800×600）、`images: 2`（800×2000）—— 图元数随视口增高略增，证明 **SVG 栅格化已生效**（logo.svg hero + 个别被布局的 logo 渲染）。
- 但 14 个参与方 logo 中多数在 800×2000 仍未产生 ImagePrimitive——参与方容器 `flex gap-4 flex-wrap justify-evenly` 仅布局出个别 logo（疑 flex-wrap/justify-evenly 在多 item 时的布局精度，**待独立诊断**，非图片加载问题：PNG/JPEG/SVG 三类均能加载）。
- 800×600 下参与方 logo grid 在折叠线以下；22.42% 主差异来自**顶部 hero/nav/正文文本区**的 fontdue vs Skia 字体度量噪声（行高/字宽/AA 差异，与 morning.work/welcome 同源，非 clean 单点修——R174 已结论此类需字体光栅器升级）。

**结论**：wintertc 布局层无 clean bug（universal 选择器、Twind 类、img aspect 均正确）；22.42% 与 morning.work/welcome 同属 fontdue 字体噪声 plateau + 参与 flex-wrap 精度小问题。**勿再以「Twind 布局缺失」重查**（已实证布局正确）。

**组件级复核（隔离复现）**：flex-wrap（4 box 正确换行 3+1）、justify-evenly（同）、img aspect（w-28/h-12 + sm: 变体均生效，6 logo 全布局）——**组件特性均工作**。故全页 2/13 logo（仅 hero + 1 个 704×24 拉伸图）非组件 bug，而是**全页交互**（祖先约束 / Twind `sm\:` 转义类匹配 / CSS 级联在多祖先嵌套下的差异），隔离复现不出，**defer**（非 clean win，勿再单点追）。

## 本轮交付

1. **`product-smoke` 子命令**（tests/wpt-runner/src/main.rs）：通用 DC-13 产品 fixture 渲染+对比工具，输出 ZeroWeb CPU PNG 并与 chromium Oracle PNG 像素 diff。后续 welcome/morning-work/wintertc 及任意 fixture 均可复用。
2. **SVG 文件栅格化**（reftest.rs `load_svg_file`）：resvg + tiny-skia 把 `<img src="*.svg">` 栅格化为 RGBA，补齐 `build_image_cache` 此前仅 PNG/JPEG 的缺口（wintertc 14 logo 中 11 个为 SVG）。
3. **capture-wintertc.mjs**：puppeteer 录制 live 页面为已解析 fixture（含 Twind `<style>`）+ chromium Oracle 截图。
4. **wintertc fixture + 基线 22.42%**（首个图片密集真实页面 DC-13 数据点）。

## 剩余差距（下轮）

- **参与方 `flex flex-wrap justify-evenly` 多 logo 未布局**（800×2000 仅 2 个 ImagePrimitive）——疑 flex-wrap/justify-evenly 在多 item 场景的布局精度，独立子问题（待诊断是否 clean）。
- 顶部文本区 fontdue vs Skia 度量噪声（与 morning.work/welcome 同源，非单点修——需字体光栅器升级，结构性 plateau）。
- 真实 ZeroBrowser 路径（非 wpt-runner harness）的 SVG/图片加载——本轮仅 harness 侧（build_image_cache）；浏览器/webview 层 ImageCache 的 SVG 支持待同步。
- **HTML width/height 单属性 aspect 推导**（`<img width=N>` 无 height）尝试修复后致 background-001/003/328/329 回归 -4（已回退）；该分支 aspect 推导与某类 background 用例交互未明，需先厘清再重试。
