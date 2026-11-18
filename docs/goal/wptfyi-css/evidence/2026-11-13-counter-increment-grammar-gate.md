# counter-increment 文法门：css-lists invalid 13F/valid 2F/computed −3 桶收口

2026-11-13。承接 [trig 型别门批](2026-11-13-trig-typegate-font-unit-cycles.md) §4
残余账中的 css-lists 桶（v14 对账四小回归之 −2 桶所在套件），同日修复。

## 1. 起点形态（v14 三页 18F）

`css-lists/parsing/counter-increment-{invalid,valid,computed}.html`（results-v14-regprobe
本地对齐账）：

- **invalid 0P/13F**：全部 `expected "" but got <value>`——引擎接受非法形
  （`none chapter`/`reversed(chapter)`/`3`/`99 imagenum`/`section -1, imagenum 99`/
  `section 3.14`/css-wide 带后缀 6 形含 `default 1`）。
- **valid 8P/2F**：`chapter` → 期望 `chapter 1`、`first -1 second third 99` →
  期望 `first -1 second 1 third 99`——隐式整数位缺省 = 1（css-lists-3 canonical
  serialization），引擎 store 原文。
- **computed 7P/3F**：全部 `'…' is a supported value … got false`——数学形
  支持（`myCounter otherCounter -8` / `sign(2cqw - 10px)` 复合 2 形）被拒。

## 2. 根因（双点）

1. **解析误吞名 token**：`parse_counter_list_with_sibling`（css-parser
   parse_extended.rs）整数位 math 分支 `Some(t) =>` 无守卫——名后跟名形
   （`myCounter otherCounter -8`）把 `otherCounter` 送 `parse_math_function`
   失败整拒。
2. **写面无校验 + supports 无文法门**：counter-increment 不在
   validate_decl_value 白名单（13 属性表外放行 store 原文——invalid 13 形
   全放行）；decl_supported 走 apply oracle sibling 哨兵 (1,1)，math 整数位
   eval 需容器 ctx（`sign(2cqw - 10px)` 的 cqw 项）而 CalcContext 无
   container_width → fail-closed 误拒合法形。

## 3. 修复（五件套）

1. **math 分支守卫**（parse_extended.rs）：`Some(t) if t.contains('(') =>`——
   函数形必含括号、custom-ident 不含；名后跟名形落整数位缺省臂留给下一轮。
2. **文法门** `parse_counter_list_grammar`（css-lists-3 §counter-increment
   `[ <counter-name> <integer>? ]#`）：仅文法裁决不求值（求值延后 apply 站——
   supports/写面门无容器 ctx fail-closed 误拒）；`none` 单置返空表；名 =
   <custom-ident>（[`is_counter_name`]）；整数位 = <integer> 字面或
   <math-function> 形（型别 <number>）；顶层逗号拒（math 形括号内逗号豁免）；
   `is_counter_name` 排除集补 `default`（css-counter-styles-3 §counter-style-name
   保留，invalid 语料 `default 1` 驱动，valid 语料零 default 名形）。
3. **decl_supported counter-increment 臂**（js_dom_bridge.rs）：文法门放行
   （position-area/anchor 先例位）。
4. **validate_decl_value 白名单接入**（14th）：分派臂置于
   value_has_unimplemented_function 之前——`reversed(` 不在已实现函数名单，
   未实现放行臂会先行误放 `reversed(chapter)`；转义 ident（`a\ 8 9` 转义
   空格名）超 top-level 分词面保留既有放行（FIXME: custom-ident 转义序列
   词法挂账）；css-wide 裸形经既有早返放行（`inherit 1` 带后缀由文法拒）。
5. **fold_decl_specified counter-increment 臂**：token 流重放补隐式整数位 1、
   math 形与字面整数位原文保留单空 join（`section calc(1)` 恒等）；非法形
   None 保留原文（`a\ 8 9` 透传——该形 valid 语料序列化恒等断言保持）。

## 4. 验证

- 单测：zero-css-parser lib **3,187 passed / 0 failed**（净 +3：文法门 invalid
  13 形逐钉 + valid 面 + fold canonical）；zero-style-system lib **2,529
  passed / 0 failed**（净 +1：写面门 13 形 + math 形 + 转义豁免 + css-wide 裸形）。
  clippy 三 crate 零告警。
- css-lists 全套件本地复测（results-g27-counter，35 页 / 276 子测）同页对账：
  - invalid **13F → 13P** 全回；
  - valid **2F → 2P** 全回（canonical 补 1 逐字对齐 Chrome 期望）；
  - computed **7P → 8P**（`myCounter otherCounter -8` 回位）；
  - li-counter-increment-computed-style 1P 零漂移。
- tree-counting 回归钉：`calc-sibling-function.html`（页内
  `counter-increment: foo calc(sibling-count())`）5P/7F 与 v14-trigfix-full
  基线逐子测一致（守卫放宽 + fold math 原文保留零漂移；2F 伪元素挂账与本批
  无关）。`calc-in-counter-001.xhtml` 为 reftest 形态不在 testharness 分母。
- counter-reset/set parsing 页（套件其余 31 页为本地首跑账）：本批改动面
  与 counter-set 解析路径零交集（独立 parser、fold 无 set 臂）；counter-reset
  仅 is_counter_name default 排除（不达 shim 写面白名单——counter-reset 不在册，
  matcher 站收紧方向正确），15F/13F 为本批前同态。

## 5. 残余（挂账）

computed 页 2F：`myCounter calc(10 + (sign(2cqw - 10px) * 5))` → gCS `none`
——写面/supports 已放行（文法门），**apply 站 eval 需容器 ctx**：
apply_advanced_property_value 签名（style/property/value/sibling）无容器参数，
CalcContext.container_width 无法从 counter 臂注入（容器 100px → 2cqw = 2px →
sign = −1 → 10 + 5×(−1) = 5）。挂账为 counter eval 容器参数腿。

## 6. 对 v14 分母账

css-lists 同页 **+16P**（invalid +13、valid +2、computed +1）；套件 −2 小回归
桶（v14 对账四小之一）随之闭账。18F → 2F。
