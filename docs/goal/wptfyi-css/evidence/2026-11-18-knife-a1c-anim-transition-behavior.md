# 刀 A1c：transition-behavior 字段建模 + allow-discrete 离散对面 + bg-position/object-position 插值臂（2026-11-18）

## 靶面

knife-a1/b 池定谳的 transition-behavior 缺面对（CLAIM-knife-a1c）：

1. **allow-discrete 离散对面**：WPT interpolation 语料 AllowDiscrete 变体
   （`CSS Transitions with transition-behavior:allow-discrete` +
   `transition-property:all and transition-behavor:allow-discrete` 两通道，
   css-transitions-2 §transition-behavior-property）——离散型属性对
   （mask-image/columns 族/column-rule-style 等）本应启动过渡并在 eased≥0.5
   翻转终值，引擎无 `transition-behavior` 字段、无离散登记门、无翻转求值臂
   → 全通道静态跳终值恒 Fail。六腿通道 F 口径 738（shape-outside 80/
   clip-002 72/shape 70/columns 60/mask-image 54/…，含本刀前 account 差
   ——旧账 54F 实为 mask-image 单页数）。
2. **bg-position/object-position 列表插值臂**：`<position>#` 逐层双分量
   文法与 mask-position 同构（css-backgrounds-3 §background-position /
   css-images-3 §object-position），mix_mask_position（parse_background_
   position_list 基座）可逐字复用；css-backgrounds 腿插值页缺口。

## 机制（style-system 6 文件 + engine 3 文件，css-parser 零编辑）

**字段建模（style-system）**：`ComputedStyle.transition_behavior:
Vec<String>`（computed_style.rs）+ default_impl 初始 `vec![]` +
inherit.rs 复位臂 + registry.rs initial_value/known_properties 两表 +
apply_advanced.rs `parse_transition_behavior_list`（`normal|allow-discrete`
大小写不敏感逗号列表，空 token/未知关键字整条拒收保旧值）+ apply 臂。

**shorthand 第 5 槽**：`transition` 简写文法扩
`<property> || <duration> || <easing> || <delay> || <transition-behavior>`
（css-transitions-2 §transition shorthand）——expand_transition 5 元组 +
behavior 关键字分支置于 property 位识别之前（两槽皆合法 ident）+
wide_keyword/`none` 形补齐；省略槽归一 `normal`。9 处既有 arity 钉
（4→5）同步更新，style-system 2565 全绿。

**写面零编辑实证**：CSSStyleDeclaration 写走通用 `_stylePropName`
camel→kebab（part03.js:31059）→ `__zw_set_style` 宿主存储；
VALIDATED_PROPERTIES 白名单只拒收已知 invalid 形、表外属性放行 store 原文
——`style.transitionBehavior = 'allow-discrete'` 本刀前已落 decl store，
仅缺 cascade apply 臂。探针页实证：写入+回读通（readback=allow-discrete），
`getAttribute('style')` 不回映是宿主化形态非写丢失。

**登记门放开（engine transition_registry.rs）**：
- `register_from_style_change` 逐 index 读 `transition_behavior` 本位
  `allow-discrete` → 显式声明绕过 TRANSITIONABLE_PROPERTIES 清单门；
- `all` 通配本位 allow-discrete 时探针集扩 DISCRETE_TRANSITIONABLE
  （mask/mask-image/font-size-adjust/font-variation-settings/column-fill/
  column-rule-style/column-span/column-wrap；未建模属性端点序列化空串自然
  滤除）；
- `register_property_if_changed` 严格门 `!allow_discrete && strict None`
  才拒登记——strict Some 对（可插值对）语义不变。
- **discrete 旗标（v2 修复，见 P2F 章）**：strict 探针 None 且
  allow-discrete 豁免登记的对，entry 记 `discrete: true`。

**求值**：strict Some 对走 lenient 真插值不变（探针证 width 10→50
±behavior 均 30px 中点）；`discrete` 旗标对纯 `eased≥0.5 → to` 翻转。
eased==1.0 echo 短路门先于求值（离散对 at(1) 级联值=to 同语义）。

**串化**：engine computed_style.rs `transition-behavior` 臂
（`string_list_to_css(…, "normal")`）+ `transition` 简写回读 behavior 槽
（非初值尾附，`all_rest_initial` 门含 behavior）。

**TRANSITIONABLE_PROPERTIES 72→74**：+background-position/
object-position（strict 委托臂同名单，mix_mask_position 复用；Some 恒真
=mask 族契约）。

**bg-position/object-position lenient 臂**：animation.rs mask-position 臂
形逐字复用（mix_mask_position + None 离散回落）；FIXME(mask-border-
discrete) 与 columns 注记更新为已随本刀接入。

## P2F 定谳修复（v1 窗捕获 → v2 落账）

v1 电池（bin-knife-a1c `6b86e915`，基线 results-knife-p-b2）multicol 腿
**REG 36**（GAIN 20，净 −16）全在 column-count/column-width interpolation
页：`column-count from [auto] to [20] at (0.5)` 期望翻转 `"20 "`，引擎产
`"10 "`。根因：allow-discrete 豁免登记的 strict-None 对在求值端走 lenient
插值，宽松解析臂把关键字端点 `auto` 0 数值化 → lerp 伪中值。plain 面
（本刀前）该对不启动过渡静态跳终值恒过——allow-discrete 开门后才现形。

修复：`TransitionEntry.discrete` 旗标（登记期 strict None ⇔ true），
求值端离散对绕过 lenient 直落翻转。v2 单腿 spot-check（bin-knife-a1c-v2
`26153f17`）：multicol REG 36→**0**，GAIN 20→**54**。

v1 其余五腿零 REG（typed-om/gaps/fonts/shapes/masking REG=0，逐子测
reconcile）。v1 typed-om +1125 为刀 P 已落账增益混窗（knifen2-b1 基线早
于刀 P）——v2 基线换 results-knife-p-b2 后归零，账面不重复计入。

## bin provenance

- b0 六腿=results-knife-p-b2（bin `2faea6ed`，刀 P 落账后最新在册基线）
- b0 css-backgrounds 腿=bin-knifen2 `8833233e` 现跑补齐（knife-p-b2 无此腿；
  首轮 battery 窗 b0bg 空产物 60s 定谳为并发窗首例挂起伪影，diag 重跑
  1.7MB 完整、v2 串行重跑为准）
- b1=bin-knife-a1c-v2 `26153f17`（HEAD a88822ea0 + 本刀全机制 + fv1 在飞
  font-feature-values 面：apply.rs whitelist/css-parser preserve_decl_case
  窗口/serialize 模块/shim/callbacks/css_wire/es_module——claim 域=fonts
  腿，fonts 腿增益按窗口法证归 fv1 系）

## 电池账本（v3，results-knife-a1c-b2，b1=bin-knife-a1c-v2）

七腿逐子测 reconcile（六腿 vs results-knife-p-b2；css-backgrounds 靶腿 b0
缺腿，对账走 RECONCILE-BG vs bin-knifen2 现跑 b0bg）：

| 腿 | GAIN | LOST | REG | 归因 |
|---|---|---|---|---|
| css-backgrounds | +5 | 0 | 0 | 本刀（bg-position/object-position 臂 + 登记清单 72→74；background-color-interpolation 151→156）|
| css-typed-om | 0 | 0 | 0 | 零漂移（刀 P 已落账面，基线换 knife-p-b2 后归零）|
| css-fonts | +334 | −6 | 6 | **fv1 窗口**（净 +328；REG 6 全在 font-feature-values serialization round-trip ×4 + font-variant-alternates ×2——本刀 13 码文件 diff 对 font-variant/feature-values/alternates 零触及（grep=0），归 fv1 在飞面，其落账时自我裁决）|
| css-gaps | +24 | 0 | 0 | 本刀（rule-color/rule-width repeaters 两页 allow-discrete 离散对面全清）|
| css-multicol | +54 | 0 | 0 | 本刀（column-count/column-width/columns 三页各 +18；v1 REG 36→0、GAIN 20→54，discrete 旗标修复兑现）|
| css-shapes | +96 | 0 | 0 | 本刀（shape-outside 三页 +24/+30/+42）|
| css-masking | +276 | 0 | 0 | 本刀（clip-path 五页 + mask-image +54 + clip-discrete +24）|

**本刀归账净增 +455（零 P2F 零 LOST）**；fonts 腿 GAIN 334/LOST 6（净
+328）按窗口法证归 fv1 系不入本刀账。v1 窗 multicol REG 36 伪中值缺陷
已在 v2 bin 修复，v3 全腿 REG=0 定谳。六腿逐子测合计 pass delta +778
（含 fv1 净 +328）。

v3 页级明细（head-100 截断外的 shapes/multicol 页已用逐子测脚本补全核）：
shapes=shape-outside-interpolation +24 / shape-outside-path +30 /
shape-outside-shape +42；multicol=column-count/column-width/columns
三页各 +18；gaps=rule-color-repeaters-001 +12 / rule-width-repeaters +12。

v1→v3 附注：v1 css-backgrounds 腿 b1 失败为脚本双前缀 bug（非机制），
v3 命令形 `testharness-$c` 修正后满产物；v1 b0bg 空产物定谳并发窗挂起
伪影，v3 串行重跑 1724243 字节与 diag 一致。

## 引擎钉测（debt：fv1 在飞 part116 include! 悬空）

`test_allow_discrete_registers_discrete_pair`（display block→none
plain 位不登记/allow-discrete 位 progress 0.25→from、0.75→to）+
`test_bg_object_position_interp`（pct 对/px 对/多层 LCM/strict 委托）已
入册；engine test harness 编译被 fv1 在飞 part116 删件（include! 行悬空）
阻塞，待其落账后 `cargo test -p zero-engine --lib` 补跑。机制已由探针页
+ v2 电池 e2e 实证（探针页发射前已删，零残留）。
