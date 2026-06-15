# DC-13 产品静态页面视觉 smoke — welcome.html 基线证据

**日期**: 2026-06-16
**渲染模式**: ZeroWeb CPU 软件渲染（`render_full_scene`，800×600）vs headless Chromium（800×600）
**fixture**: `apps/browser/assets/welcome.html`（自包含：仅 favicon data-URI + anchor href，无外链 CSS/图片）

## 总体差距

| 渲染端 | 像素一致率（vs Chromium） | 着色像素数 | 内容 bbox |
|--------|--------------------------|-----------|-----------|
| ZeroWeb CPU | **48.41%**（diff **51.59%**）| 208,368 | x[74,799] y[72,599] |
| Chromium（Oracle）| — | 28,115 | x[40,759] y[36,599] |

**关键发现**：ZeroWeb 着色像素是 chromium 的 **~7.4 倍**（208k vs 28k，占页面 43% vs 6%）。即 ZeroWeb 把大量本应为空白的区域填上了背景/内容色——典型症状是大面积背景填充错位、section 重叠、或 hero 背景渲染成全页。这是 **reftest 通过率平台期（434/490）无法捕获的产品可见缺陷**——welcome.html 不是上游 reftest，其渲染退化不在 56 个失败用例中。

## 证据文件

- `welcome-zeroweb-cpu.png` — ZeroWeb CPU 渲染
- `welcome-chromium.png` — Chromium 参考截图

## 方法

1. `node /tmp/capture-welcome.mjs`（puppeteer-core + /usr/bin/chromium，viewport 800×600，networkidle0）→ `welcome-chromium.png`。
2. welcome.html 临时复制为 wpt-data 自源 reftest（注入 `<link rel="match">`），`REFTEST_DUMP` 渲染 ZeroWeb CPU 输出 → `welcome-zeroweb-cpu.png`，渲染后**删除临时文件**（恢复 490 上游 reftest 基线，未污染统计）。
3. PIL 逐像素对比（max channel diff > 5 判异）。

## 与 DC-13 验收标准的差距

- ❌ welcome.html 与 Chromium 在相同 viewport 下的参考截图对比：**51.59% 差距**（远未通过）。
- 待自动检查（DC-13）：文本不重叠、sibling card/link/shortcut 文本不串联、`ZeroBrowser` 标题宽屏不拆行、`<br>` 后 tagline 两行——需进一步逐区域分析（本轮仅总体像素对比）。

## 下一步（DC-13 推进）

1. 逐区域（hero / feature cards / 快捷键 / 快速访问 / footer）定位 208k 着色像素的来源（大面积背景填充 vs 重叠 vs 布局塌缩）。
2. welcome.html 是自包含页，差距根因在 ZeroWeb 基础排版/绘制链路（IFC/背景/盒模型），与上游 reftest 的 Phase A 结构性缺口同源。
3. 建立 welcome.html smoke 为定期门禁（capture-welcome.mjs + ZeroWeb dump + PIL 对比，阈值待定）。
