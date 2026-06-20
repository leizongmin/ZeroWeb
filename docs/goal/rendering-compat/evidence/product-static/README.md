# DC-13 产品静态页面视觉 smoke — welcome.html 基线证据

> **R370 复测（2026-06-20，post-R368）**：welcome `z_vs_chr`（ZeroWeb CPU vs chromium Oracle，DC-14 口径）= **14.68%**，较 R227 的 17.06% 改善 ~2.4pp。归因：R368（commit 8db89df，inline-block 文本内容 shrink-to-fit 经 intrinsic 测量）修正了 welcome 的两处 `display:inline-block; width:auto` 含文本元素——`.version` 徽章（`<head>` CSS line 53）与 `.shortcut kbd`（line 149）——此前被 taffy 当 Block 拉伸到容器满宽（满宽灰条），现收缩到文本宽，与 chromium 一致。welcome-zeroweb-cpu.png 已更新为当前渲染。**残余 14.68% 仍为 fontdue 字体 AA 噪声 + item-tag span→block R109 IFC（结构性，Phase A）**，非 inline-block 缺口。复测方法：临时把 welcome.html 包成 reftest（注入 match link）+ REFTEST_DUMP + cross-validate.py vs welcome-chromium.png（已清理临时文件）。

> **R227 更新（2026-06-17）**：welcome diff **28.08% → 17.06%**。下方「剩余 28.08% = fontdue 字体噪声、结构布局正确」的结论已被**证伪**——R226/R227 定位到真实布局 bug：taffy `Layout::location`（border-box 相对，已含父 padding+border）被 painter 当作内容盒相对再次叠加 padding+border → **padding 双计**，致 hero-accent 渲染于 y=72（应 36，整页下移 36px 级联）。修复（`extract_layout` 把块级子节点换算为内容盒相对）后 hero-accent 回到 y=36。**剩余 17% 仍含 fontdue 字体噪声，但原 28% 的大头是此布局 bug，非字体**。详见 `evidence/r227-welcome-padding-doublecount-fix-2026-06-17.txt`。下文为 R174 时点的历史记录。

**日期**: 2026-06-16
**状态**: welcome.html 差距演化 **51.59%→26.15%（R170/R171/R172）→ 28.72%（R173 CJK 文本可渲染，fontdue 度量噪声推高）→ 28.08%（R174 box-shadow blur σ 修复）**。本 session 累计 5 修复 + 1 能力：(1) R170 box-shadow rgba 带空格丢 alpha 致实心黑（132k 纯黑→0）；(2) R171 border/outline/column-rule/text-decor 简写同 class；(3) **R172 border-radius 背景在 draw_order 模式被丢弃**（卡片白底消失，50.45%→26.15% 主因）；(4) R173 加载 Noto Sans CJK 字体（中/日/韩可渲染）；(5) **R174 box-shadow blur σ=radius/2 修复**（旧实现 σ≈radius 偏大 2.3 倍，卡片阴影扩散 12px→收紧）。
**渲染模式**: ZeroWeb CPU 软件渲染（`render_full_scene`，800×600）vs headless Chromium（800×600）
**fixture**: `apps/browser/assets/welcome.html`（自包含：仅 favicon data-URI + anchor href，无外链 CSS/图片）

## 总体差距

| 渲染端 | 像素一致率（vs Chromium） | 着色像素数 | 内容 bbox |
|--------|--------------------------|-----------|-----------|
| ZeroWeb CPU（R174 后）| **71.92%**（diff **28.08%**）| — | x[38,799] y[36,599] |
| Chromium（Oracle）| — | 28,115 | x[40,759] y[36,599] |

## 剩余 28.08% 根因（已穷尽定位，非 CSS bug）

经 throwaway 渲染测试 + layout snapshot + PIL 逐带/逐像素分析，welcome.html 剩余差距 **96.5% 为 fontdue vs Skia 字体噪声**，结构布局经确认正确（gradient bar y=36、hero/卡片几何、卡片白底均正确）：

- **glyph 边缘 AA 噪声（~50%）**：色差直方图双峰 delta≈−10（33k px）与 +10（33k px），fontdue 与 Skia 子像素覆盖位置不同致同一 (x,y) 一边亮一边暗。
- **文本换行致卡片高度差**：card-desc 长文本（如 `DOM / CSS / layout / paint in Rust · Rust 原生渲染管线`）fontdue 量宽与 Skia 不同→换行行数不同→chromium 卡片比 ZeroWeb 高 ~13px。
- **box-shadow 残差**：R174 已收紧 blur 扩散，但 box-blur 近似与 chromium 精确高斯仍有边缘残差。
- **CJK fontdue 度量噪声**：R173 后 CJK 字符可见，但 fontdue CJK 度量与 Skia 差异。

**结论**：welcome.html 已无 clean structural CSS bug 可修。剩余差距需升级字体光栅器（fontdue→更接近 Skia 的实现）才能显著下降，非单会话范围。下一步 DC-13 杠杆转移至 morning.work（外链 CSS + CJK 真实页）/ wintertc.org（图片子资源）等能暴露未实现 P1 缺口的 fixture。

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
