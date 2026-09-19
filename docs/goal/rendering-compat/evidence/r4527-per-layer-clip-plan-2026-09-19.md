# R4527 计划：background-clip 逐层（per-layer）clip——border-area 余族根因 P 轮

- 日期：2026-09-19（R4527 P 轮——0 net code）
- 谱系：clip-border-area 余 11 fail（multiple-backgrounds 13.32 / border-on-top 4.17 /
  double 2.40 / complex 4.02 / box-decoration-break 1.81 / border-shape ×3 / blend-mode /
  text）共同根因定位。

## 根因（代码确证）

`background-clip` 多值声明（`border-area, border-box, content-box`）被**整体丢弃**：
`parse_background_clip` 只匹配单关键词（逗号列表 → None）→ `apply_advanced` 的
"background-clip" 臂不命中 → 计算值保持 initial BorderBox → R3908/R4526 环带全不触发。
（`background_image` 已是 `Vec<BackgroundImageComputedValue>` 逐层 ✓，clip 为单枚举 ✗。）

## 实施蓝图（下轮 C 轮）

1. `computed_style.background_clip: Vec<BackgroundClipComputedValue>`（小 Vec，典型 1 元）。
2. `parse_background_clip_layers`（镜像 parse_background_image_layers：逗号分段逐个
   parse_background_clip；层数 < 层图层数时按 CSS 规则循环补齐）。
3. apply_advanced / shorthand background.rs（shorthand 单值 → vec![..]）/ default_impl
   （vec![BorderBox]）/ inherit（Vec 派生）/ matcher（适配）。
4. painter 消费点语义：
   - `paint_bg_image_in_origin` 逐层 clip = clips[i % len]（R3908 环带仅对
     border-area 层生效；border-box/padding/content 层各按己值）；
   - `paint_background` 实底色 = **最后一层**的 clip（css-backgrounds-3）；
   - effects.rs（box-shadow/bg-image canvas 传播路径）与 text.rs
     （bg_clip_text_solid_style：任一层 Text 即触发，取其 bg 色）= clips[0] 等价或
     any-layer 判定；
   - js_dom_bridge getComputedStyle 序列化逗号列表。
5. 计 21 消费点（11 文件：painter/mod 6、effects 3、bridge 3、style-system 5、
   parser 1、text 1、matcher 1）；单值页（clip 一元）经由 `[0]` 等价 → 零回归预期。
6. kill-switch `ZW_PER_LAYER_CLIP=0`（parse 层回退单值丢弃旧行为）。

## 预期

border-area 余族 5-6 案（multiple-backgrounds/border-on-top/double/complex/
box-decoration-break）翻绿；border-shape ×3 与 blend-mode 需再叠加 corner-shape 环带
几何/blend 管线（域外挂账）。
