# R4534 证据：border-shape slice 1（R4533 蓝图实施）——clips-background / inner-outer 双翻绿

- 日期：2026-09-19（R4534 C 轮——R4533 下轮方向①）
- 谱系：css-borders-4 §7 border-shape（36 fail 域首 slice）。

## 实现

1. **css-parser**：`parse_border_shape`（文法 `none | [ <basic-shape> <geometry-box>? ]{1,2}`）
   ——token 扫描（函数 token 配平括号 + 独立 geometry-box 关键词，形状与关键词间允许
   空白），复用 `parse_clip_path`；`BorderShapeValue{None,Stroke,Fill}` +
   `BorderShapeGeometryBox{content/padding/border/margin/half-border-box}`
   （parse_extended_border.rs）。任一段非法 → 整条丢弃。
2. **style-system**：computed `border_shape` 字段 + default None + apply_advanced 臂
   （含 `return true`）+ apply_initial_value 臂 + registry 初值/known_properties +
   @supports。
3. **engine paint**（helpers `border_shape_plan` + 两消费点）：
   - relevant side（§7.6 top→left→bottom→right 首个非 none）宽/色；
   - 形状→绝对坐标多边形（circle/ellipse 64 段近似 + closest/farthest-side +
     **closest/farthest-corner 欧氏角距**、inset（round 忽略）、polygon）；
   - geometry-box 引用盒（half-border-box = border-box 内缩半边框宽）；
   - **stroke mode** = `PathStrokePrimitive`（闭合折线、宽 = relevant border-width）；
   - **fill mode** = 外+内顶点串联单 `PathFillPrimitive`（扫描线 even-odd 天然只填环带）；
   - `paint_borders` 早退发射（§7.5.1 border-radius 归零/corner-shape 忽略）；
   - `paint_background` 背景色涂内形状（stroke mode = 路径多边形，超出内缘的带由
     居中描边覆盖——oracle 语义）；bg image 形状裁剪挂账 slice 3。
   - kill-switch `ZW_BORDER_SHAPE=0`。
4. **插错一现即修**：`ClipPathRadius` 新增 `ClosestCorner/FarthestCorner`（css-shapes-1
   radial-extent；ellipse 逐轴关键词按 csswg#14010 双轴同值）——首版遗漏致
   ellipse-corner-keywords 从绿转红 22.68%（test 页声明不可解析被丢弃而 ref 页可解析
   → 两页分叉），补角距解析后 0.00% 翻绿。clip-path 半径枚举扩展同步补齐
   js_dom_bridge 序列化 / 指示器 / clip-path 渲染三处 match。

## A/B

- **corpus 1471→1469（15125/16594 = 91.15%）**；fail-list diff（对 R4532 基线）=
  **唯一 border-shape-clips-background（1.27→0.86%）+ border-shape-inner-outer
  （1.82→0.81%）移除，零新红**（ellipse-corner-keywords 修复后维持绿）。
- 残余改进未翻案：geometry-box 12.56→9.39、collapsed-shape 24.59→16.42、
  background-origin-border-box 5.14→3.25、inset-shadow 6.46→2.32、
  outline-double-path 1.78→1.54（各案剩余机制 = overflow 内形状裁剪（slice 2）/
  bg image 形状裁剪（slice 3）/ shadow 形状跟随等）。
- 全门禁：make test 19,366P/0F（+5 新单测）、fmt、clippy 双 feature 组 -D、
  product-smoke welcome 15.40% 同值 struct PASS、bench-gate 定向
  css-parser+style-system+engine GATE PASS（41 指标）。
- 新单测：parse（circle() stroke / 双 polygon+geometry-box fill）、apply 链 ×2。

## 挂账（slice 2/3）

- slice 2：box-shadow 内外形状投射 + overflow 内容内形状裁剪（overflow 系 ×5、
  inset-shadow 系、shadow 系）。
- slice 3：bg image 内形状裁剪（path-clip 底座，与 dotted/dashed 墨迹挂账同源）；
  absolute-coords/polygon-miter-limit 勘察。
- corner-shape 子目录 14 fail（superellipse 精度/AA）独立账维持。
