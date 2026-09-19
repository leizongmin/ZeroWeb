# R4533 计划：border-shape 属性域勘察 + 实施蓝图（css-borders-4 §7）

- 日期：2026-09-19（R4533 P 轮——R4532 下轮方向②，0 net code）
- 谱系：css-borders 域 36 fail（border-shape 18 + corner-shape 14 + 杂项）+
  background-clip 家族 border-shape ×3 ≈ 39 fail 关联域——当前最大未开发区。

## 域盘点（对 R4532 fail-list）

- **border-shape 子目录 13 fail**：geometry-box 12.56、collapsed-shape-clips-background
  24.59、background-origin-border-box 5.14、inset-shadow 6.46、polygon-miter-limit 5.31、
  inner-outer 1.82、clips-background 1.27、overflow 系 ×5（~1-2.1）、outline-double-path
  1.78、shadow 系 ×2。
- **border-shape-absolute-coords（tentative）5.27**。
- **corner-shape 子目录 14 fail**（1.0-8.3 微差簇）——superellipse 精度/AA 域，独立账
  不入本蓝图。
- background-clip 家族 border-shape ×3（R4529/R4530 记档挂账同源）。

## 规范语义定谳（css-borders-4 §7，2026-03-26 WD）

- **文法**：`border-shape: none | [ <basic-shape> <geometry-box>? ]{1,2}`，initial none，
  不影响布局（§7.5.4 纯视觉效果）。
- **Stroke mode**（单形状）：border = 沿形状路径的 **stroke**，宽度 = relevant side 的
  computed border-width；geometry-box 默认 **half-border-box**（路径两侧各延半边框宽，
  与默认边框行为对齐）。oracle 实证（clips-background：100×100 content + 10px border
  → border-box 120 → half-border 110 → circle() r=55，描边居中 → 环带 [50,60]，ref
  SVG r=55/stroke 10 逐像素吻合）。
- **Fill mode**（双形状）：border = 两路径之间的面积；第一个 = 外形状（默认 border-box）、
  第二个 = 内形状（默认 padding-box）；填充色 = relevant side 的 computed border-color。
- **Relevant side**（§7.6）：block-start → inline-start → block-end → inline-end 序中
  第一个非 none 的边（horizontal-tb LTR = top → left → bottom → right）。
- **交互**：border-radius 视为 0、corner-shape 隐式忽略（§7.5.1）；box-shadow 按内外
  形状投射（§7.5.2）；内形状裁剪 overflow 内容与背景（§7.5.3）。

## ZW 现状

border-shape 属性**未实现**（声明整体丢弃 → 普通边框盒渲染）。部分 border-shape 案
PASS（double-shape-default/half-border-box-default/circle-extent-keywords 等）= ref
页与「忽略行为」巧合同形，非真支持。

## 复用面（代码确证）

1. **基本形状解析现成**：`ClipPathValue`（css-parser values/types.rs:914）支持
   inset()/circle()/ellipse()/polygon() + closest-side/farthest-side + at position，
   解析器与测试完备——border-shape 直接复用该类型或薄包装。
2. **形状→几何解析现成**：`resolve_inset_length`（paint/helpers.rs:1162，Px/Em/Rem/%
   相对盒维）+ `circle_to_polygon`（helpers，指示器路径已在用）——形状 → 绝对坐标
   多边形的机制齐备。
3. **多边形图元现成**：`PathFillPrimitive`（CPU 扫描线 even-odd 填充，R4531 刚修
   alpha）+ R4248 形角环带「外形状填边框色 + 内形状填背景色」双填充先例。
4. **消费点挂点现成**：paint_borders（跳过常规边框）、paint_background（bg color 臂
   R4526 BorderArea 环带臂同位）、paint_box_shadow（slice 2）。

## 实施蓝图（下轮 C 轮起 slice 化）

**slice 1（首个 C 轮）**：
1. css-parser：`parse_border_shape` → `BorderShapeValue { None, Stroke(ClipPathValue,
   Option<GeometryBox>), Fill { outer, outer_box, inner, inner_box } }`（geometry-box
   = content/padding/border/margin/half-border-box 枚举；文法非法整条丢弃）。
2. style-system：computed 字段 + default None + apply_advanced/matcher/kill-switch
   `ZW_BORDER_SHAPE=0`。
3. engine paint：
   - paint_borders 早退（border-shape 活跃 → 跳过常规 4 边绘制；border-radius 归零
     语义天然满足）；
   - stroke mode = 闭合多边形 PathStroke（circle/ellipse → circle_to_polygon 64 段、
     inset/polygon → 顶点直出；width = relevant side border-width、color = relevant
     side border-color）；
   - fill mode = 双 PathFill（外形状 border 色 + 内形状背景色，R4248 同构；背景仅
     color 场景）；
   - paint_background：bg color 臂 border-shape 活跃时改涂内形状（或交由双填充覆盖，
     bg fill 抑制同 R4525 clip:text 模式）。
   - + 单测（stroke 环几何 / fill 双形状 / geometry-box 换算）。
4. **预期翻绿上限 ~5 案**：clips-background 1.27、inner-outer 1.82、geometry-box
   12.56、background-origin-border-box 5.14、collapsed-shape-clips-background 24.59。

**slice 2**：box-shadow 内外形状投射 + overflow 内形状裁剪（overflow 系 ×5 +
inset-shadow 系 + shadow 系 ≈ 8 案）。
**slice 3**：bg image 内形状裁剪（需 path-clip 能力，与 dotted/dashed 墨迹挂账同底座
——ImagePrimitive.clip 仅矩形）+ hit-test 域（非 reftest）。
**风险**：AA 边界（ref = ZW SVG 管线抗锯齿圆 vs PathStroke 二值折线描边）——
clips-background 现差 1.27% 距 1% 阈值近，64 段近似误差需实测校验；inset() round
半径与 extent keyword 组合面大，slice 1 只保 solid/circle/ellipse/polygon 主干。
