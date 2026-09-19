# R4525 证据：background-clip:text 实底彩字 v1——corpus 1476（9 案翻绿）

- 日期：2026-09-19（R4525 C 轮——painter 2 文件）
- 谱系：`css-backgrounds/background-clip` 29 fail 中 **clip-text-\* 子族**（canonical 模式
  `background-color + background-clip:text + color:transparent`）。

## 实现（实底彩字 v1，painter 2 文件零新原语）

css-backgrounds-4 §background-clip:text：背景仅绘于文本字形区域内。**实底色等价渲染**：
bg 色字形 + bg fill 抑制 ≡ chromium 对实底 clip:text 的渲染（canonical 模式逐像素等价；
background-image 非空不触发——渐变/图片 clip 需 mask 管线，域外）。

1. `Painter.bg_clip_text_color: Vec<Color>` 上下文栈：paint_node_inner push（clip:text +
   实底 + 无 image）/ 单一出口 pop（同 counter 作用域模式）。
2. `paint_background`：clip:text + 实底 → **bg fill 抑制**。
3. `paint_text` 字形色两路（stored / paint-ifc）override：栈顶色 → owner 样式自身
   clip:text（inline span 字形由宿主块 IFC 承载、栈未及 push 的形态——transform 包裹页
   clip-text-scaled 实证）→ 原色。
4. kill-switch `ZW_BG_CLIP_TEXT=0`。paint_node_in_rect（dirty-rect 路径）v1 未接线
   （早返多，栈语义易漏；全帧路径已盖，dirty-rect gap 记账）。

## 中间迭代

- 栈式 v1 首测：scaled 页全白（inline 字形被宿主块 IFC 吸收，栈 push 晚于宿主
  paint_text）→ 补 owner 样式判定后 0.00 ✓。
- 声明误植 paint_node_in_rect（fmt 塌缩形态致锚点串函数）→ 移正 paint_node_inner；
  clippy -D 抓漏。

## A/B（终态）

- **corpus 1485→1476（15118/16594 = 91.11%）；翻绿 8 案**：clip-text-scaled（17.56）、
  -ellipsis、-inline-block-child、-on-body-not-propagated、-out-of-flow-child、
  -relative-child、-text-decorations、-text-emphasis（+box-shadow-003 flake 相位）。
- background-clip 族内余 23 fail：clip-text 多行/descendants/stacking（需 mask 管线或
  祖先穿透深化）、blend-mode 族、border-area 族（独立特性）。
- 全门禁：make test 19,352P/0F、fmt、clippy 双 feature 组 -D、product-smoke welcome
  15.40% 同值 struct PASS、bench-gate 定向 zero-engine GATE PASS（26 指标）。
