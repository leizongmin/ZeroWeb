# DC-13 产品静态页面视觉 smoke — welcome.html 基线证据

**日期**: 2026-06-16
**状态**: ✅ box-shadow/text-shadow rgba 带空格丢失 alpha 致实心黑——**已修复（R170, 9ade334）**。welcome.html 纯黑 132,793→0。剩余 ~50% 差距为其他渲染差异（cards 区域，Phase A inline-block/IFC ownership）。
**渲染模式**: ZeroWeb CPU 软件渲染（`render_full_scene`，800×600）vs headless Chromium（800×600）
**fixture**: `apps/browser/assets/welcome.html`（自包含：仅 favicon data-URI + anchor href，无外链 CSS/图片）

## 总体差距

| 渲染端 | 像素一致率（vs Chromium） | 着色像素数 | 内容 bbox |
|--------|--------------------------|-----------|-----------|
| ZeroWeb CPU | **48.41%**（diff **51.59%**）| 208,368 | x[74,799] y[72,599] |
| Chromium（Oracle）| — | 28,115 | x[40,759] y[36,599] |

## 根因定位（box-shadow alpha 丢失 → 实心黑）

逐区域分析 208k 着色像素：**132,793 是纯黑 (0,0,0)**，bbox x[86,793] y[265,582]（连续全宽，非 4 个 card 形块）= `.card`（4 张）+ `.bottom-row` section（4 个）区域，约 96%/行 黑色（文本浮在上面）。

**关键 bisect**：把 welcome.html 的所有 `box-shadow` 声明删除后重渲 → **纯黑从 132,793 降到 0**。即 box-shadow 是 132k 黑像素的唯一来源。

**精确机制（SHDWDBG 插桩 paint_box_shadow）**：welcome.html 的 8 个 card/section 的 box-shadow 计算值 `color=rgba(0,0,0,255)` —— **alpha=255（实心黑）**，而声明是 `box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08)`（alpha 应≈20）。`#dadce0` 等实心色阴影 alpha=255 正确。`render_shadow`（cpu/shadow.rs）逻辑本身正确（用 shadow_alpha 合成），问题在**上游：box-shadow 的 rgba alpha 在 welcome.html 的样式计算中被丢失，算成 255**。

## 触发条件（单变量排除，需 CSS bisect）

以下**单变量探针均正确**（box-shadow alpha=20，无实心黑）：
- ✅ `display:grid` 的 card（grid item 不丢 alpha）
- ✅ `rgba(0, 0, 0, 0.08)` 逗号后空格（与无空格同）
- ✅ `* { box-sizing; margin; padding }` 通用选择器
- ✅ `@media (prefers-color-scheme: dark)` 不在 light 模式应用（background 验证：dark 规则不泄漏）
- ✅ base box-shadow + `@media dark { box-shadow: rgba(...,0.4) }` 覆盖（不泄漏/不损坏 base alpha）
- ✅ 8 个 card 同页（element count 不触发）

**结论**：触发条件在 welcome.html 完整 ~240 行 CSS 级联的组合中，无法用单变量探针复现。需对 welcome.html CSS 做**系统性 bisect**（按规则减半，定位使 card box-shadow alpha 变 255 的具体规则/组合），可能是多规则级联或样式计算的状态/hash 问题。

## 证据文件

- `welcome-zeroweb-cpu.png` — ZeroWeb CPU 渲染（含 132k 实心黑）
- `welcome-chromium.png` — Chromium 参考截图

## 方法

1. `node /tmp/capture-welcome.mjs`（puppeteer-core + /usr/bin/chromium，800×600）→ chromium shot。
2. welcome.html 临时复制为 wpt-data 自源 reftest（注入 `<link rel="match">`），`REFTEST_DUMP` 渲染 ZeroWeb，**渲染后删除临时文件**（恢复 490 基线）。
3. PIL 逐像素 + 颜色 + y-band 分析；box-shadow bisect（regex 删除 box-shadow 重渲）；SHDWDBG 插桩 paint_box_shadow 打印 color.alpha。
4. 单变量探针（grid / `*` / @media / count 等）逐个排除。

## 意义

- **reftest 434/490 平台期无法捕获**此缺陷——welcome.html 非上游 reftest，其 box-shadow alpha 丢失是产品可见的渲染 bug。
- 修复后 welcome.html 差距应大幅下降（132k/208k 着色像素源自此 bug），推进 DC-13。
- 根因（box-shadow rgba alpha 在多规则页丢失）可能也影响其他含 box-shadow 的真实页面。

## 下一步

对 welcome.html `<style>` 做 CSS bisect：保留 `.card` 规则 + `*`，逐块引入其余规则，定位使 card box-shadow alpha 变 255 的规则/组合（候选：某条规则触发样式计算的 alpha 解析路径异常）。找到后修复 css-parser/style-system 的 box-shadow rgba alpha 计算。
