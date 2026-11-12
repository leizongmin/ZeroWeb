# DC-2 g23 批：min/max/clamp serialize canonical 保持串化 + 写面文法/槽族型别门 + clamp none 重写

日期：2026-11-12　刀：g22 写面文法门之上的 minmax/clamp/round-mod-rem
serialize 与 invalid 双面收口——四刀（A serialize canonical 保持串化、
B 写面 math 通用文法+槽族型别门、C clamp `none` 实参 parse 期重写 +
注释等价空白、D shim math 函数透传 + bridge 期望更新）。探针实证
**+262**（预测 +238）；首次全量跑（g23/2230）values 同账但 color -11
回归，勘根 part11-color.js 透传名单过宽（calc/min/max/clamp 旁路
`_zwCComp` canonical 折叠），收窄后重跑 **g23b** 权威账：六套件
values 3,228 → **3,553 净 +325 零回归**（25 页增益），color
10,972 ±0、transitions/animations/easing/transforms 逐位 ±0。

## 1. 实现面

- **刀 A serialize canonical 保持串化**（css-parser
  values/serialize_math.rs）：`fold_math_specified` Length 位标量塌缩
  失败（有限 %/布局依赖叶树）→ `canonical_math_specified`（§10.6
  sort）：min/max 多实参**保序**逐实参 canonical、单参 ≡ `calc(实参)`
  解包（§10.5）；和串叶项化——同单位系数合并、`-0` 归一
  （`format_number_6dp`）、排序 number → % → 维度（单位 ASCII CI
  升序，`30% + 1em + 21px`）、min/max/clamp 节点殿后保源序、可标量
  折叠节点塌缩为标量项；乘积项 number 系数前置、sum 语境括号化
  （`1px + (0.4 * min(1em, 1px))`）；除法节点/多非标量因子/trig 复合
  返 None 保原文（零回归边界）。`fold_decl_specified` 白名单接线
  margin-top/margin-left（Length 槽，g12 面 width/text-indent 同型）。
  parser 一元负号伪形零锚（`0px - x`）不落项、右臂反号——`-5%` 负叶
  canonical 幂等。
- **刀 B 写面 math 通用文法+槽族型别门**（style-system
  property/apply.rs `validate_decl_value` 白名单检查之前）：值顶层为
  math 函数形（22 名单）→ 整值过 parse（`parse_math_function`，time
  槽 `parse_time_calc`）+ `math_slot_family_valid` 槽族型别表——
  LengthOnly（border 宽度族/outline-width/column-rule-width/
  transform-origin-z，% 不可）/LengthPercentage（尺寸/边距/缩进/
  字距/left）/Number（opacity/scale，角度单位文本 token 扫描补
  provenance——角度叶 parse 期弧度 Number 化，三角实参角剥跨度不误伤）
  /rotate（容 number，bare-number-as-deg 同口径）。Invalid 二分：无
  除法拒（混族和树），含除法放行交 fold/computed 链（单位代数幂抵消
  ——getComputedStyle-calc-mixed-units-003 四 P 钉面）。parse 失败双
  豁免交 oracle：非单函数平衡跨度（`calc(50%) calc(25%)` 双 calc 拐
  角——border-radius-002 钉面）与未知函数实参（css-values-5
  inherit()）；替换函数（var/env/attr/random 族/progress/if/
  calc-size/url）与 custom property 豁免。
- **刀 C clamp `none` 重写**（css-parser values/types.rs）：
  `rewrite_none_clamps` parse 期文本级重写（§comp-func `none` = 对应
  界不限）——`clamp(none,A,B)` ≡ `min(A,B)`、`clamp(A,B,none)` ≡
  `max(A,B)`、`clamp(none,A,none)` ≡ `A`（裸值落 plain-expr 兜底臂）；
  嵌套内层先归约循环至稳定；`none` 落 val 位/实参残缺保原文（上层
  parse 拒）。`skip_whitespace` 补注释等价空白（css-syntax-3 §4.4，
  `clamp(10px, 35px /*foo*/, 30px)` 形）。
- **刀 D shim math 透传**（engine js_dom_shim/part11-color.js）：
  `_zwCCParse` 对 round/mod/rem/hypot/pow/trig/abs/sign 等 22 函数形
  放行透传（此前 null 拒——round-mod-rem-serialize spec 语料 3F 根因
  ），文法/槽族型别裁决交宿主 `__zw_validate_decl_value`（刀 B），
  canonical 折叠交读面 `fold_decl_specified`。**calc/min/max/clamp
  不入透传**——`_zwCComp` 的 calc 臂即 canonical 折叠口径
  （`calc(25% * 2)` → `calc(50%)`），旁路即丢折叠：首版透传名单过宽
  曾致 css-color -11 回归（parsing/opacity-valid），收窄后复位。
  part03.rs bridge 期望同步（margin-left `round(23px, 10px)` →
  `calc(20px)` 折叠读出，r2320 同例）。
- **规范锚**：CSS Values 4 §10.6 sort/§10.9/§comp-func/§10.5
  calc-type-checking/§round-func、css-syntax-3 §4.4。

## 2. 探针（results-g23probe，六 pass /tmp/zw-g23-r2-*.log）

- minmax/clamp/round-mod/calc-mix/calc-infinity/serialize 六 pass 恰集
  对 g22 基线逐断言 diff：**净 +262 零回归**（judge-g23probe.py
  exit=0，跨面一致性零违例）——超出刀前预测 +238（signs-abs-invalid
  +43 与 calc-unit-analysis +6 为预测外页）。

## 3. 权威账（results-g23b，systemd zw-g23b-full-2252，22:17-22:2x）

- **链**：三 crate 单测 → 定向 clippy → 六套件全量（唯一路径
  transcript/JSON，test-guard 包裹）。首轮 g23（stamp 2230）values
  3,553 已达但 **css-color 10,961（-11）**——透传名单收窄后重跑
  g23b（stamp 2252）为权威账。
- **六套件逐断言对 g22**（analyze-g23-full.py，regression=NO）：
  values **3,228 → 3,553 净 +325**（flip=325/reg=0）、color
  10,972 → 10,972 ±0、transitions 200/animations 238/easing 51/
  transforms 229 全部逐位 ±0；页级 new=0 missing=0 六套件全绿。
- **values 25 页增益**：minmax-length-invalid +48 / clamp-length-
  serialize +45 / signs-abs-invalid +43 / minmax-number-invalid +42 /
  round-mod-rem-invalid +32 / clamp-length-invalid +23 / clamp-partial-
  serialize +16 / hypot-pow-sqrt-invalid +10 / clamp-length-computed
  +9 / minmax-length-percent-serialize +8 / calc-unit-analysis +6 /
  minmax-percentage-invalid +6 / clamp-integer-computed +5 /
  minmax-percentage-serialize +5 / round-mod-rem-serialize +5 / 其余
  9 页 +1~+4。invalid 桶（minmax/clamp/round-mod-rem/signs-abs，
  master 在账 ~155F+52F）全数兑现；timeout 桶 7 页与 g22 同集
  （cap/rcap/rch/rex/ric/rlh-invalidation + dynamic-viewport-units
  rule-cache，非本批伪影）。
- **单测**：css-parser lib **3,177 全绿**（g22 3,173 +4：none 重写
  margin 表/clamp margin serialize 表/minmax canonical margin 表/
  round-mod-rem opacity serialize 表，共 ~90 断言）；style-system lib
  **2,527 全绿**（+1：`math_grammar_slot_family_gate` 恰集矩阵 ~120
  断言）；engine lib 3,909 过（7F 逐名核对全为在册既有组：img settle
  2/fetch 1/iframe-sw 1/response-body 1/websocket 2，与 g22 基线
  一致，零新增）。
- **clippy**（css-parser/style-system/engine 定向 -D warnings）
  exit=0 零告警；**fmt** 三 crate clean（首轮 apply.rs 两处漂移
  机械收敛，收敛后 style-system lib 复跑 2,527/0F）。
- provenance：g23b 链单 systemd 单元单写者（results-g23b 唯一路径
  transcript/JSON），树在跑期间零编辑；fmt 收口在跑后（空白等价，
  style-system 复验通过）。

## 4. 残余挂账（g23b 口径 values 残 2,040F/199 页）

- **serialize 桶新暴露**：calc-dimension-serialization-order 44F、
  calc-serialization-002 22F、minmax-length-percent-serialize 残 29F、
  minmax-percentage-serialize 残 21F（维度串化排序/`mod-rem` 维度
  形）。
- **在册桶不动**：signs-abs-computed 94F（%-CB 形）、
  calc-infinity-nan-computed 31F（rotate matrix 化桶）、serialize-
  resolution 29F（image-set parse 前置刀，deferred）。
- **独立桶**：line-break-ch-unit 194F / attr-all-types 105F /
  viewport-units 84F+24F / calc-size 族 158F / position-computed 55F /
  progress-serialize 55F+progress-computed 36F（progress() 函数未实
  现）/ random 残 46F+23F / if()-cycle 40F / url-request-modifiers
  60F（css-values-5 未接线函数族）/ typed_arithmetic 32F。
- 五套件（color/transitions/animations/easing/transforms）本批零
  触碰零漂移；color 挂账桶（系统色 27F/长尾 ~39F）与 anchor 组 F 残
  桶不变。
