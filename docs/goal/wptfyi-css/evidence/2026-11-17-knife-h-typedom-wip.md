# 刀 H · typed-om shim WIP 认领落地（r1 +71 → r2 修复 +74）

2026-11-17。认领面 `crates/engine/src/js_dom_shim/part10-typed-om.js`（本流
e2 时代孤儿在途码，stash@{2} "e2 typed-om js in-flight" 同源；刀 I 碰头账
在册靶面 typedom +71）。刀 I A′↔g1 对账披露本面净增后，按认领板
（CLAIM-knife-h-typedom-round.txt）认领落地，两轮对称消去验证。

## 交付面

r1（原 WIP，181+/24−，指纹 zw-typedom-listvalued-2026-11-17）：

- `_zwOmPropName` camelCase/交替大小写属性名归一（known-syntax 表收敛）。
- `_zwOmValidateSet` 值裁决步：关联属性错配门、list-valued string 门
  （`_zwOmListValuedProps` ~40 属性）、shorthand token 门、单 token 数值族
  裁决；set/append 的 CSSUnparsedValue/var() 并置门。
- get 面：shorthand 回落基类实例（携关联属性）、list-valued 逗号逐段、
  非负属性 CSSMathSum 包装、case-insensitive delete/has/getAll。
- `_zwOmExpandShorthand` margin 1-4 值长槽展开（all/background 留 FIXME）。

r2（191+/24−，指纹 -r2）：r1 收账暴露 3 键真实面回归（见下），同根因
var() 归一未贯穿补两处——

1. `mapGetAll` shorthand 回落臂：`/(^|[^\w-])var\(/i` 命中文本返
   `new CSSUnparsedValue(_zwOmTokenizeVar(text))`（css-typed-om-1 §get 值
   归一：var() 引用值不分属性恒 unparsed 形）。
2. `_zwOmCheckShorthandTokens` 折叠占位接受臂 `if (tk === '@') continue;`
   ——折叠 regex 把 calc()/var()/url() 折为 '@' 后无接受臂（'@' 落 ident
   正则失败抛 TypeError），字符串 var()/url() shorthand set 同根错杀；
   接受臂完成折叠作者既有意图（后续 `/^(calc|var)\(/i` 臂）。语料 r1
   静默（无字符串 var() shorthand set 翻转），r2 实测零额外位移。

## r1 账（11-15:56 发射，unit zw-cssa-h-111556，HEAD=05efa0a62）

A′ bin `6e60e480`（刀面 stash 态）/ B bin `8303d45a`（181+/24−），双链
十腿全 rc=0。reconcile-h1-vs-haprime-final.txt：

| corpus | A′ | B | delta | F2P | P2F | 页集合 |
|---|---|---|---|---|---|---|
| gaps | 3792 | 3792 | +0 | 0 | 0 | 75=75 |
| multicol | 757 | 757 | +0 | 0 | 0 | 91=91 |
| typedom | 9873 | 9944 | **+71** | 74 | 3 | 359=359 |
| shapes | 5106 | 5106 | +0 | 0 | 0 | 145=145 |
| masking | 2308 | 2308 | +0 | 0 | 0 | 61=61 |
| TOTAL | 21836 | 21907 | +71 | | | |

## r1 3 键 P2F 根因定谳（r2 修复靶）

P2F 全集：properties/{all,background,margin}.html
"Can set ... to var() references: var(--A)"，失败消息
`assert_equals: expected "CSSUnparsedValue" but got "CSSStyleValue"`。

- harness 链（testsuite.js `testPropertyValid` + `gVarReferenceExamples`）：
  set 输入是 CSSUnparsedValue **实例** `[' ', CSSVariableReferenceValue('--A')]`
  → set instance 分支（kind='unparsed'）放行，不触 shorthand token 门 →
  存文本成功 → get 走 shorthand 回落臂返 opaque 基类 → 类断言失败。
  码面矛盾（token 门按理对字符串 var() 应抛）由 node 复演定谳：字符串
  形根本不进本脸；margin-top/left/right/bottom "Unsupported property"
  全家族 F 为既有缺口，双侧同相不入账。
- spec 依据 css-typed-om-1 §get 值归一（margin.html 头部 rel=help 即指
  §get/§set/值归一三节）。vintage 通过正因旧行为符合 spec。
- harness 自带 `specifiedAlternateExpected`（CSSWG#13792 首空格歧义）：
  token 化后 `[varref]` 段命中备选形态；computed 侧回调 no-op 不比对。

## r2 账（12:25:39 发射，unit zw-cssa-h2chain-122539，HEAD=84cb6ab00；发射时为 9b50a92e7，landing rebase 换基）

隔离窗 Stage-2 构建（12:22:36-12:24:03）：姊妹 css-parser 半成品
（serialize_shape_specified.rs + values/mod.rs，11:42-11:47 后零触碰 33+
分钟，5 编译错卡死整树）stash 隔离 → A′2 → pop 刀面复树 → B2 → pop
姊妹文件逐字节恢复；前置 Stage-1 探 zero-engine release 绿 50.01s 暖缓存。
指纹法证：r2 标记串 B2=1/A′2=0（strings 抽验）。

A′2 bin `313394d1bab12bd990e59dc1b9b10474` / B2 bin
`318b424d77fe024c96e9fd929cfd1a7c`，双链十腿全 rc=0
（log-cssa-h2chain-122539.txt）。reconcile-h2-vs-haprime2-final.txt：

| corpus | A′2 | B2 | delta | F2P | P2F | 页集合 |
|---|---|---|---|---|---|---|
| gaps | 3838 | 3838 | +0 | 0 | 0 | 75=75 |
| multicol | 958 | 958 | +0 | 0 | 0 | 91=91 |
| typedom | 9873 | 9947 | **+74** | 74 | **0** | 359=359 |
| shapes | 5123 | 5123 | +0 | 0 | 0 | 145=145 |
| masking | 2321 | 2321 | +0 | 0 | 0 | 61=61 |
| TOTAL | 22113 | 22187 | **+74** | | | |

- **P2F 归零**：三脸 r2 双侧逐键 Pass 直查（JSON 提取），3 键真实回归
  清偿；F2P 集与 r1 恒等 74（Edit-2 语料静默符合预测，零额外位移）。
- **组合态注记**：A′2 基数较 r1 A′ 抬升（gaps 3792→3838、multicol
  757→958、shapes 5106→5123、masking 2308→2321）＝未落地 peer WIP
  组合态增量（multicol repeater/columns 流 +201、K1 cssa 五文件族、J2
  shape 序列化族），双侧同态对称消去内抵消，不入本刀账；各归属流按五
  corpus 工法自行认领（master 组合态注记同口径）。
- verify 角色轮独立复算确证（master 12:38 笔）：typedom +74 逐数兑现
  ＝「申报=实测」。

## 残面/域外

- margin-top/left/right/bottom 不在 syntax 表（"Unsupported property"
  全家族 F 双侧同相）——后续刀候选。
- `_zwOmExpandShorthand` 仅 margin 展开，all/background 留 FIXME
  （WIP 既有 scope，r1/r2 零位移非回归）。
- 字符串 var()/url() shorthand set 臂（Edit-2）语料静默，行为面留待
  上游语料扩面覆盖。

## provenance

- r1：bin-cssa-haprime/bin-cssa-h1、results-cssa-{haprime,h1}-full、
  log-cssa-h-full-111556.txt（H-CHAIN-RC=0）、reconcile-h1-vs-haprime-final.txt。
- r2：bin-cssa-haprime2/bin-cssa-h2、results-cssa-{haprime2,h2}-full、
  log-cssa-h2chain-122539.txt、reconcile-h2-vs-haprime2-final.txt、
  隔离窗注记与发射回执见 CLAIM-knife-h-typedom-round.txt、草稿
  notes-knife-h-r2.md。
- 对账工具：reconcile-five-corpus.py（status 大写口径；PAGE-ONLY 零容忍
  F2P/P2F/NEW/GONE/页集合全查）。
