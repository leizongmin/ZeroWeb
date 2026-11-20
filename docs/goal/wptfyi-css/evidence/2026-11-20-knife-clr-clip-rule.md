# 刀 CLR：clip-rule 全链注册面（2026-11-20，rate-limit 断轮接管续）

## 池与根因链

GBX 收账后（11:28 提交 053cfecf0/380cdbd1a），上一轮在限额窗口内起本刀
（11:50-51 写入两清单注册即撞 5h 限额，12:15:38 重置后本轮接管）。v1 face
（registry known_properties + grid_cssom SUPPORTED_STYLE_PROPS 两清单加列）
A/B 定谳 **gained=0**：主靶 clip-rule-no-interpolation 42 子测推进一格仍全灭
——消息从 `'from' value should be supported` 翻为 `'to' value should be
supported`（`CSS.supports('clip-rule','evenodd')` 仍 false）。

v1 探针页双探针定谳两处缺口：
1. `decl_supported` 尾门＝`known_properties && apply_property_value_with_quirks`
   （js_dom_bridge.rs:6332-6336）——apply oracle 主 match 落 `_` 委托
   apply_advanced，clip-rule 无臂 → `_ => false`（validate_decl_value 的
   clip-rule 臂是写面文法单点，apply oracle 不经它）。
2. gCS 读通道 `serialize_computed_property` 主 match 落 `_ => String::new()`
   （computed_style.rs:1278）——clip-rule 零槽位零串化臂。

参照页（同框架 from [initial] 形）：mask-border-source-interpolation 294P/294
满贯——其链＝枚举槽位 + apply 臂 + 串化臂 + DISCRETE_TRANSITIONABLE 名册 +
AnimationClock::apply_to_computed_style 写回臂，本刀 v2 全链镜像。

框架语义定谳（探针实证）：`interpolation-testcommon.js` measure() 的期望值
由**引擎对期望串静态 apply**（expectedTargetContainer）算出——42 子测是
engine-vs-engine 对账；`CSS.supports(property, from/to)` 断言在每子测内硬性
前置（与 allow-discrete 方法的无条件 supportsValue 无关）。端点 'initial'
静态回显为初值 'nonzero'（mbs 静态探针 'none' 同形），非字面回显。

## 链路

css-masking-1 §the-clip-rule：`nonzero | evenodd`，初值 nonzero、inherited、
非 animatable（css-transitions-1 §animations animation-type: none——仅
allow-discrete 可过渡，css-transitions-2 §transition-behavior-property）。
六通道对账语义（expectFlip(from,to,flipAt)，7 采样点/方法 × 6 方法 = 42）：
- cssTransitions/cssTransitionAll plain（flip −Infinity）：名册外属性不启动
  过渡＝静态跳终值 'evenodd' 恒真（TRANSITIONABLE_PROPERTIES 清单外门，
  transition_registry.rs:203）。
- allow-discrete 两方法 + cssAnimations + WAAPI（flip 0.5）：离散翻转
  eased≥0.5 → to；pre-flip 端点经 canonicalize apply 往返变化判定回 from。

## 码面（10 文件 +195/−10，patch md5 7869fd7a78b07c8021f1b2b2fd6e9113）

- `crates/style-system/src/property/registry.rs`：known_properties() +=
  "clip-rule"（supports 关键字臂，v1）。
- `crates/engine/src/dom_bindings/grid_cssom.rs`：SUPPORTED_STYLE_PROPS +=
  "clip-rule"（css-conditional probe 一致性页，v1）。
- `crates/style-system/src/property/types.rs`：`ClipRuleComputedValue` 枚举
  （Nonzero/Evenodd + as_css_str）。
- `crates/style-system/src/property/computed_style.rs`：`clip_rule` 槽位。
- `crates/style-system/src/property/default_impl.rs`：初值 Nonzero。
- `crates/style-system/src/property/apply_advanced.rs`：apply oracle 值臂
  （大小写不敏感 nonzero/evenodd；v1 A/B 定谳的 CSS.supports 值门根因位）。
- `crates/engine/src/js_dom_bridge/computed_style.rs`：串化臂
  `style.clip_rule.as_css_str()`（gCS 读通道）。
- `crates/engine/src/js_dom_bridge/transition_registry.rs`：
  DISCRETE_TRANSITIONABLE 16→17 加列 "clip-rule"（allow-discrete 启动门；
  plain 通道清单外不启动语义不变）。
- `crates/engine/src/animation.rs`：`AnimationClock::apply_to_computed_style`
  clip-rule 写回臂——**v1 两残域共同根因**：canonicalize 往返要求 apply 后
  串化变化（ser1≠ser0），无臂时 pre-flip discrete echo（eased<0.5 回 from）
  与 ≥0.5 翻转位双双恒等返 None 落级联（探针 6F pre-flip got evenodd +
  8F flip 位 got nonzero 实证）；补臂后探针页 84P/2F（余 2F 为探针垃圾
  期望串本体）。
- 常驻钉测 ×2（与修复同提交）：style-system `knife_r2_expansion_gate_verdicts`
  扩 apply oracle 断言（值臂真/假 + 槽位落位）；engine
  `test_clip_rule_discrete_flip_end_to_end`（登记→咨询→canonicalize 全链，
  at=0.3 回 from / at=0.6 翻 to；终值巧合 None 按设计两形态皆认）。

## A/B 电池（knife-clr-battery.sh v2，log knife-clr-battery-v2-1345.log）

b0 = HEAD（face 剥除）md5 `a2456cba604142be710d7aba51c0ad41`；b1 = HEAD +
face md5 `8256f15c35952a84e9835f17ca5f6308`（M2 distinct ✓）。build/leg 全经
test-guard（build per-proc 6/total 14/1800s；leg per-proc 4/total 8/3000s +
CASE_WALL_S=60）。12:53:21→12:58:17 全程 4 分 56 秒。

**主电池（css-masking + css-conditional）：gained=45 lost=0 regressed=0
pages_gain=3 pages_loss=0**
- clip-rule-no-interpolation.html **P 0→42**（42 子测满贯 F2P）。
- parsing/clip-rule-computed.html **P 0→2**。
- inheritance.sub.html P 25→26（+1——clip-rule inherited 语义随槽位落位
  兑现，非预判收益）。
- css-conditional SET-IDENTICAL（probe 一致性页 clip-rule 对零扰动）。
- css-masking 页级账：P 5427→5472（F 928→883）。

**补哨兵腿（clip-rule 用例页扫描：css-masking 外 4 文件）**：
- css-cascade 79 页 SET-IDENTICAL（all-prop-revert-layer 系零扰动）。
- css-typed-om 359 页 SET-IDENTICAL（the-stylepropertymap/properties/
  clip-rule.html 33P 恒）。
- css-variables 56 页 **gained=1 lost=0**：variable-presentation-attribute.html
  P 12→13（'Testing clip-rule' 子测——呈现属性经变量间接通道兑现）。

**净 +46 零回归**（主电池 +45、补腿 +1；lost=0 regressed=0 全腿）。

## 残面

- `CSS.supports('clip-rule', <替换函数>)` 形与组值串化：apply 臂仅收关键字
  两形（语料无此驱动，validate 写门替换函数豁免语义在 store 原文透传层）。
- clip-rule 渲染联动（SVG fill-rule 语义对 HTML 裁剪）未建模——types.rs
  FIXME 保位；clip-rule-001/002 渲染页两侧恒等（b0/b1 各 2P 恒）。
- 上一轮 v1 草稿（两清单注册）与本轮 v2 全链同提交落库；v1 A/B zero-gain
  证据存 knife-clr-battery-1219.log。

[控制面](../master.md)
