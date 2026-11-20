# 刀 GBX：clip-path `[ <basic-shape> || <geometry-box> ]` 双序组合形 + 裸 `<geometry-box>`（2026-11-20）

## 池

WAC 落库后取最新 b1 JSON 池勘：css-masking clip-path 族 ~840F 为最大连片矿脉，
其中 clip-path-geometry-box-interpolation.html 0/144 整页红——根因链单点：
support probe `CSS.supports(property, from)` → shim `__zw_css_supports` →
`decl_supported` → `apply_property_value_with_quirks` → clip-path 臂
`parse_clip_path`（无盒处理）→ None → probe false → 144 子测全部在插值前死亡
（消息 `'from' value should be supported expected true got false`）。

同矿脉邻页归因（不属本刀）：clip-composition 72F 为 legacy `clip` 属性域；
interpolation-shape 218F 为既有 lerp_shape 命令插值臂 bug 域（刀 SHFN 备刀）；
xywh-rect 84F 为 rect 跨形→inset calc 面（二十一轮刀既记残面保位）；
interpolation-001 余 56F support 失败成因未明（待查）。

## 链路

css-masking-1 §the-clip-path 文法：`[ <basic-shape> || <geometry-box> ] | none`。
geometry-box 七关键字（border/padding/content/margin/fill/stroke/view-box）与
mask 侧 `<geometry-box>` 同值域——复用 `MaskGeometryBoxValue`
（parse_extended_visual）零新枚举。语义四层：
1. parse：裸盒 → `WithBox{shape: None}`（等价 border-box+该盒）；前缀/后缀
   单序组合 → `WithBox{Some(shape)}`；双盒拒收（|| 单序单现）。
2. 串化（canonical store/computed 同语义）：default border-box 双序省略
   （与既有裸形字节兼容、刀 S 剥除钉面 L91/L94 保位）；非 default 盒保形
   后缀（geometry-box 插值语料 computed oracle `circle(20px) margin-box`
   保形）；前缀形收正为后缀形。
3. 插值（css-shapes-2 §interpolating-shape 组合规则）：同盒对内层同构插值、
   盒保形发射；异盒/裸盒对恒异构 → None 离散切换。内层走原串剥盒后字符串
   递归（css-parser 串化面不含 circle/inset 族——解析-串化往返不闭合，
   故不经 clip_path_value_to_css）。
4. shape-outside 分治：css-shapes-1 `<shape-box>` 仅 content/padding/border/
   margin 四形——刀 GBX 起 parse_clip_path 亦受理 masking 七盒，apply/写门
   两臂收窄回拒（裸盒走自有 Box 槽；组合形/扩形未建模仍拒，FIXME 保位）。

## 码面（10 文件 +393/−83，patch md5 67b20260077d8e770683959aff80c92b）

- `crates/css-parser/src/values/types.rs`：`ClipPathValue::WithBox` 变体。
- `crates/css-parser/src/values/parse_extended_border.rs`：`parse_clip_path`
  盒感知重构（裸盒/前缀/后缀单序 + `parse_clip_basic_shape` 八函数分派helper）。
- `crates/css-parser/src/values/parse_basic_shape.rs`：
  `clip_path_value_to_css` WithBox 臂 + `pub geometry_box_keyword`；
  `canonicalize_strip_box_keywords` 返前缀关键词、`canonicalize_clip_path_specified`
  default-omit 保形收正。
- `crates/engine/src/js_dom_bridge/computed_style.rs`：`clip_path_to_css`
  computed 通道 WithBox 臂。
- `crates/engine/src/animation/clip_interp.rs`：同盒臂（原串剥盒递归 +
  default-omit 发射）+ `strip_geometry_box_suffix`。
- `crates/engine/src/animation.rs`：`clip_value_has_calc` WithBox 透传臂；
  WAAPI keyframe apply shape-outside 臂收窄（WithBox{None,四形}→Box 槽、
  组合形不动样式）。
- `crates/engine/src/paint/border_shape.rs`：顶点通道 WithBox 内层透传臂
  （盒移位上游建模，FIXME 残面记非 default 盒）。
- `crates/style-system/src/property/apply_advanced.rs`：shape-outside 臂收窄。
- `crates/style-system/src/property/apply.rs`：clip-path 写门收敛
  parse_clip_path 单点裁决（词法剥除通道撤除——剥单盒后解析会复活双盒形
  「border-box border-box」）；shape-outside 写门 WithBox 收窄。
- `crates/css-parser/src/values/tests/clip_path.rs`：GBX 钉测 3 组
  （双序/裸盒 parse、串化通道约定面、canonical 保形收正）。

## 顺修（同 commit 分离提交，归因刀 shapes-canonical 残面）

HEAD 上 zero-css-parser 存量 4 红（stash 探针实证）：shape-outside 钉测 2 红
（`circle(at center top)`/`inset round 非等值组`——刀 shapes-canonical 2fd814691
把 shape-outside canon circle 臂换刀 clip 族 canon 后缺 identity guard，join-
fallback 恒发 identity Some）+ path_valid_forms（双引号规范化 vs 旧 passthrough
钉面）+ font_slot_math_write_gate（style-system，成因未追）+ advanced
break-before-after。本刀仅顺修前 2（canonicalize_shape_outside_specified 补
`body == v → None` identity guard——None=原文保形与 identity 输出字节等价，
通道行为零漂移）；后 2 属异域残面不越界，master.md 记账待归因刀清偿。

## 单测

- zero-css-parser 3321 passed / 2 failed（2 红即上述异域残面）；GBX 3 组新钉全绿。
- zero-engine 4233 passed / 0 failed；GBX 同盒/异盒/前缀等价钉全绿。
- zero-style-system 2581 passed / 1 failed（异域残面）。
- workspace `cargo check` 0 error；`cargo fmt --all` 已过。

## 电池（.acceptance/wptfyi-css/results-knife-gbx，配方承 mb3）

- b0 = HEAD(b6de4585a) 剥面，runner md5 5a70b32940d4620ad52a78ab91cf4a88；
  b1 = HEAD+面，runner md5 8a51667cc1e1a7936100e731ee949fdd。M2 异 md5 ✓；
  face patch 字节绑定断言入批（md5 67b20260077d8e770683959aff80c92b，发射后
  换字节即 ABORT）；RIDE 空（face 即树内全部 10 修改件，S1 快照锚定）。
- 腿 {css-masking 主靶, css-shapes, css-transitions, css-animations 哨兵} ×
  {b0,b1}，CASE_WALL_S=60，test-guard 包裹（build 6/14/1800s，leg 4/8/3000s）。
- 对账：gained=168 lost=16 regressed=16 pages_gain=2 pages_loss=1。

### REG 定谳：getAnimations 窗口伪影，不转抄

唯一损失页 css-animations/Element-getAnimations.tentative.html P 17→1：双侧
页面级 Timeout（CASE_WALL_S=60 截断深度不同）。240s 高墙双臂复跑（保存的
b0/b1 runner）：17P/3F + 页面 Timeout 双侧恒等，PASS-SET-IDENTICAL=True——
窗口伪影，不转抄（b6de4585a 学习案同款）。

### 哨兵定谳：零真漂移

- css-shapes：b0/b1 145 页 P=6026/F=1489 恒等，逐页 PASS-SET-IDENTICAL
  （shape-outside 收窄 + canon identity guard 零漂移实证；语料系本 clone
  缺失，自 ZeroWeb-2 clone 拷贝补腿 546 文件 tally 对锚）。
- css-transitions：P 集恒等 1027；diff 页 2（color-mix F 4→3、event-dispatch
  F 30→15+Timeout）均 Fail/Timeout 窗内翻转，PASS-SET-IDENTICAL=True。
- css-animations：diff 页 2（getAnimations 见上、event-dispatch F 10→16）
  PASS-SET-IDENTICAL=True。

## 终账

gained=+168 lost=0 regressed=0 pages_gain=2 pages_loss=0——
clip-path-geometry-box-interpolation 0/144 → 144/0 满贯 +
clip-path-interpolation-discrete 111/57 → 135/33（+24，geometry-box 离散域
probe 翻真后离散翻转语义逐位命中）。

## 探针记录

正式电池前单页 probe（runner 8a51667c，CASE_WALL_S=60）：目标页 144/0、
discrete 135/33、001/002/shape/xywh-rect/shape-control-points/svg-geometry-box
六邻页逐页与 WAC b0/b1 基线恒等——增益构成与电池对账逐位吻合。
