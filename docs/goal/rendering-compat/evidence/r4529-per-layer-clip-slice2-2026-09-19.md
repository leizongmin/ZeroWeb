# R4529 证据：per-layer clip slice-2——逐层 painting area + border-style 墨迹环带（double 翻绿）

- 日期：2026-09-19（R4529 C 轮——R4528 slice-2 挂账清偿）
- 谱系：`clip-border-area-*`（css-backgrounds-4 §2.1 border-area）。

## 实现

1. **逐层 painting area（R4528 slice-2 挂账）**：`paint_background_image` 按图层预计算
   `LayerPaintAreas { clips, rings }`（`clip_rect_for_layer(i)` = R2312/R3908 单层语义
   的逐层化，css-backgrounds-3 §3.7 多值 clip cyclic `i % len`）传入
   `paint_bg_image_in_origin`（该函数签名无元素盒几何——R4528 记档的两案中取
   「caller 预计算传入」案）。层循环内非 local 层优先取逐层 clip；None 载荷
   （画布传播/col/fixed 路径）= 标量 `clip_*` 旧行为。单值 clip 页 `i%1=0`
   全层同值 = byte-identical。
2. **border-style 墨迹环带**：`border_area_ring_strips`（R3908 4 条带的样式感知化）——
   solid/groove/ridge/inset/outset = 整带（4 条带逐位同旧几何，byte-identical 守卫
   测试锚定）；**double = 双带**（外带 + 内带，镜像 `paint_border_edge` 的
   `gap = max(t/3,1)`、`line_w = max((t−gap)/2,1)` 口径 → 8 条带）；dotted/dashed
   墨迹为 stroke pattern，矩形 clip 不可表达 → 近似整带（挂账）。上下条带走全宽、
   左右条带走 padding 盒竖向区间——与 paint_borders 角区排除一致。
3. **kill-switch** `ZW_BORDER_AREA_INK=0`（环带回退全带；复测 double 0.06→2.40% 红原样）。
   逐层 plumbing 无独立开关——`ZW_PER_LAYER_CLIP=0`（R4528 parse 层）使 vec 退化 len=1
   即整体旁路。

## A/B

- **corpus 1475→1474（15120/16594 = 91.11%）**；fail-list diff（对 R4528 基线
  /tmp/corpus-fails-v5.txt sort 后）= **唯一 `clip-border-area-double` 移除，零新红**。
- **double 2.40→0.06% 翻绿**（单层 border-area + 4 侧 double：ref 双带墨迹间白隙 =
  墨迹外区域，环带按墨迹双带裁剪后逐像素对齐）。
- multiple-backgrounds 13.32→**9.16%**（layer0 border-area 环带正确后残余 = dotted
  圆点墨迹——oracle 蓝点阵 vs 整带蓝环，stroke-pattern clip 域）；complex 4.02→**3.85%**
  （右 double 臂生效，dashed/dotted 侧残余同域）；box-decoration-break 1.81% 持平
  （inline fragmentation 域）；border-shape ×3 / blend-mode / text 域外持平。
- 全门禁：make test 19,355P/0F（+3 新单测）、fmt、clippy 双 feature 组 -D、
  product-smoke welcome 15.40% 同值 struct PASS、bench-gate 定向 zero-engine
  GATE PASS（26 指标）。
- 新单测（paint/tests/background_repeat.rs）：double 8 条带几何逐矩形断言、solid 4 条带
  byte-identical 守卫、逐层 clip（border-area + content-box 双层）端到端 5 图元断言。

## 挂账（border-area 余 8 fail 的剩余域）

- **dotted/dashed 墨迹环带**（multiple-backgrounds 9.16 / complex 3.85）：需 stroke
  pattern 形状的 clip 能力（ImagePrimitive.clip 仅矩形）——渲染器 path-clip 或
  tile-级 dot/dash 剖分，深域评估后立项。
- box-decoration-break（inline 多片段逐段环带）、border-shape ×3（corner-shape × 环带
  几何）、blend-mode / text（blend 与 mask 管线）——域外挂账维持。
