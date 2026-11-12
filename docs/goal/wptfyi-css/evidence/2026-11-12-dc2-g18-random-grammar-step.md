# DC-2 g18：random() 文法刀（2026-11-12）

刀提交 `99c573d30`（css-parser values/random.rs + style-system apply.rs random
臂 + engine part03 r8 钉例换槽）。文法全量语料反推自 random-invalid 39 拒形 /
random-serialize 71 接受形（全 width 槽写面；invalid 页唯一属性）。

## 文法规则（语料反推，草案未收组合面）

- identification 节成分：`--<dashed-ident>`（至多一且必须先于任何 scoping）×
  element 域 `element-scoped`（至多一）× UA 域 `property-scoped |
  property-index-scoped | ua-<ident>`（至多一），element × UA 域任意序可共存；
  `fixed <number [0,1)>` / `auto`（WebKit shorthand）独占节。
- 拒面锚形：双 UA 域（`property-scoped ua-width-1` /
  `property-index-scoped --foo property-scoped`）、同关键字重复、裸 ident、
  ident 滞后于 scoping、节数越界、跨族混（§10.5 一致门）、`fixed` 出范围。
- 数字节 `%` 归 length 族（serialize 页 `random(--foo, 10px, 20%)` 行）；
  min>max 文法合法（求值期交换，门不裁值序）。
- 族随属性槽 `random_family_allowed`：width/margin-left/text-indent/
  transform-origin-z→Length、scale/z-index→Number、transition/animation 四属性
  →Time、rotate→Angle、其余槽放行（complex 槽无语料钉面）。

## 写面通道闭环

- `VALIDATED_PROPERTIES` 12→13（+width——invalid 页 39 形唯一属性）。
- random 臂位于 unimplemented-passthrough 之前：值内每个 `random(` 调用
  （`extract_random_args` 括号配平 + ident 边界防 `foorandom(` 误配）逐处
  parse+族槽裁决——任一 invalid 整拒，全 valid 放行 store 原文（apply 链未
  实现 random()，经 oracle 会误杀 serialize 页接受形）。
- 零交互面确认：`random-item(`（CSS Values 5 另函数，`random` 后非 `(`）不触
  extract；item-invalid 全 font-family 槽不在白名单；random-in-* 桶走样式表/
  keyframe parse 通道不经 validate_decl_value。
- `CSS.supports` 走 decl_supported（known_properties + apply_property_value_
  with_quirks，js_dom_bridge.rs:5919）不经写面门——computed 页 supports 恒
  false 形态不变。

## probe 实证（release 二进制 07:34:52，FILTER=random- 全桶）

| 页 | 刀前 | probe 后 | 漂移 |
|---|---|---|---|
| random-invalid.tentative | 39F | **39P** | **+39（收口）** |
| random-serialize.tentative | 42F/29P | 42F/29P | 零漂移 |
| random-computed.tentative | 141F | 141F | 零漂移（supports 门维持） |
| random-item-*（6 页） | ~53F 汇总 | 21P/14P/3P + 28F/14F/4F/3F | `random-item(` 零交互面 |
| random-in-*（6 页）+ unresolvable | 挂账桶 | 形态同前 | 通道不经写面门 |

probe 日志 `/tmp/zw-probe-random-g16.log`（136 Pass / 262 Fail——Fail 面全部
为既有挂账桶：canonical 串化 42F、eval/求值 141F、item/in 子桶）。

## 交叉验证

- peer 流组合二进制账（g17b，be008fdd7 批条目）：random-invalid +39、
  values face 2,917→3,032 净 +129 零损失——与本文 probe 逐页一致。
- serialize 页 42F = canonical 串化期望（WebKit shorthand 补全 `ua-*` 形如
  `random(0px,100px)`→`random(element-scoped ua-width-1, ...)`）——engine
  computed 串面，eval/串化刀挂账（master.md 卡点）。

## 单测钉面

- css-parser values/tests/random.rs：5 组（71 接受形 + 39 拒形逐钉、族槽表），
  全量 3136 绿。
- style-system t54 random_grammar_gate（width 槽 valid 放行/invalid 拒/angle
  族拒/嵌套形/非 random 值零漂移），全量 2520 绿。
- engine part03 r8 钉例 width→height（width 入表后白名单外 passthrough 钉换
  槽；height 与 apply.rs 新旧版本双向兼容——提交顺序无关中间态），t54 钉面绿。
