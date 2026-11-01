# M2-S201：SVG 形状 gBCR/命中注入——inert-svg-hittest 收口

- **日期**：2026-11-01
- **目标面**：`html/semantics/interactive-elements/the-dialog-element/inert-svg-hittest.html`（2 子测：inert 不达 / dialog 关闭后到达）
- **状态**：收口——子测 2 翻绿（interactive 全树 1/2 负载敏感，targeted 2/2 稳定）

## 勘根

入场：s196 挂账序「inert-svg-hittest 第 2 子测」——子测 1 恒 Pass
（inert 门 `_zwDialogModalFocusGateOk` 先于 runner 面积择优，vacuous 绿）；
子测 2（dialog close 后 `pointerMove(0,0,{origin:wrapper}).pointerDown()`
断言 `target.matches(":active")` + mousedown 到达 `<rect id=target>`）恒 Fail：
`assert_true: target is active expected true got false`。

探针四轮（zz-s201-p1–p4，语料零改动验后删）：

- p3 事件落点转储：wrapper 中心按下 `hits=[svg,wrapper,body]`——事件在
  `<svg>` 止步，`<rect>` 不达（事件 target=svg，子元素监听不触发、
  `:active` 不标记）。
- p2 offset(0,0)/(2,2)/center 全 miss——非边缘像素/包含判定问题。
- p4 gBCR 直读（施工中复跑）：`svg=8,8,784×784 target=8,8,784×0`——
  **形状布局盒是退化体（h=0）**，勘根假设「形状无布局盒」修正为「退化盒」。
- p1 双面分离：事件面与 `:active` 标记面同根同翻。

根因链：布局只为 `<svg>` 建替换盒（usvg/resvg 整体栅格化），形状子元素
（rect/circle/ellipse）盒退化（h=0，几何不参与布局）→ gBCR rect 快照对形状
不可用 → runner 最小面积命中扫描（`querySelectorAll('body, body *')` + gBCR
包含 + 最小面积，tie `<=` 文档序后者胜——s201 未改）在形状处回落 `<svg>`
本体 → 事件永不达形状。

## 施工

（rust 2 文件 + 新模块 1，净 +331/−1；runner/语料零改动）

- **新模块** `crates/engine/src/hit_test_svg.rs`（310 行；hit_test.rs 已
  2141 行超 2000 规则，新代码独立成文件）：
  - viewBox meet 映射：`scale=min(box_w/vw, box_h/vh)` + `preserveAspectRatio`
    缺省 `xMidYMid` 居中；无/非法 viewBox 回落用户单位 1:1；
  - 形状包围盒：`rect`（缺省 x/y=0，w/h ≤0 跳过）、`circle`（cx−r 等）、
    `ellipse`；百分比等非纯数值解析失败按缺省跳过；
  - 嵌套 `<svg>` 子树与 path/polygon/line/text/use/foreignObject 跳过
    （FIXME 在册）；
  - `svg_shape_rects_from_layout` 布局树绝对坐标累计复用 `child_origin`
    （hit_test.rs 内提为 `pub(crate)`）；单元测试 6 个（映射/1:1/圆椭圆/
    跳过域/零尺寸/非法 viewBox）。
- **hit_test.rs**：`HitTestCache.svg_shape_rects` 字段（键 = `node_id_to_u64`）；
  `from_document` 构建期填充；`fill_layout_rect_snapshot` **覆盖**注入——
  首版 `or_insert` 被形状退化盒遮蔽（p4 探针证伪后改覆盖，合成视觉包围盒
  为准）；`from_snapshot`（跨进程）留空 + FIXME。
- **lib.rs**：`pub mod hit_test_svg;`。

## 验证

- **单元**：hit_test_svg 6/6；zero-engine 全量 3652P+1F——1F =
  `test_ua_display_li_list_item_canvas_inline_s24_flip`，stash 基线复跑同败
  （渲染流域在册键，D2 归因），非本切片引入。
- **目标面**：inert-svg-hittest 2/2 Pass ×2 targeted（s201-targeted.log /
  s201-targeted-recheck.log）+ 探针面 p1/p2 同断言 Pass（wip4）。
- **A/B**（stash 协议 keyed diff，(key,state) 对去重）：
  - interactive（FILTER=interactive-elements，483 键恒定）：baseline
    439P/44F/0T → wip2 **440P/43F/0T，净 F→P=1（靶键）P→F=0 键漂移 0**；
    wip1 靶键 Fail（负载敏感，同二进制 targeted 2/2 绿 + wip2 绿）——
    `:active` 标记负载敏感域新样本。
  - forms（FILTER=semantics/forms，4711 键恒定）：baseline 4450P/250F/11T →
    wip2 **4452P/248F/11T，净 F→P=2 P→F=0**（F→P 均为 onblur flake 域
    顺向摆动）；wip1 的 radio-focus-navigation-checked 2 键 P→F 在 wip2
    复归 Pass——负载抖动非本切片回归；onblur 家族双向摆动持续采样。
- fmt --check 过；guarded-clippy workspace 过。

## 挂账

- 嵌套 `<svg>` viewport 级联映射、path/polygon/polyline/line/text/use/
  foreignObject 几何未建模（FIXME 在册，语料无驱动）。
- 跨进程快照路径（`from_snapshot`）未携带 svg_shape_rects——多进程
  renderer 下形状命中回落 `<svg>` 本体（FIXME 在册）。
- SVG 几何级点拾取（圆内外区分、`fill:none` 不可命中）按包围盒近似未建模。
- `position:fixed` 静态 div gBCR/命中面缺失（p1 探针对照面 p3 复现）——
  独立布局缺口，语料无驱动，留观察。
- `:active` 标记负载敏感（interactive 全树 1/2 绿）——与在册 onblur 摆动
  同族，持续采样。
- 渲染流域 `ua_display_li` 键本轮复现同败（stash 基线同败，非本切片）。

## 日志

`s201-{probe-round1,probe-round2,probe-p2,probe-p3,probe-wip1,probe-wip2,probe-wip3,probe-wip4,targeted,targeted-recheck,interactive-stash-baseline,interactive-wip,interactive-wip2,forms-stash-baseline,forms-wip,forms-wip2,engine-unittests,clippy}.log`
（evidence/logs/ gitignore，账本仅存档文件名）
