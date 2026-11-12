# DC-2 g20 批：random() <random-key> parse 期 canonical 串化刀（Face 1）

日期：2026-11-12　刀：g18 random 文法刀之上的指定值 canonical 化——CSS
Values 5 §9.4.1 Simplification of `<random-key>`：写面 fold 入口单点接线，
random-serialize 页 42F 收口 39F（A key 展开/替换 28、B 单位归一 4、C 实参
lite 折叠 3、E 显式 ua-ident 序归一 4），余 3F 为 color 通道嵌套（D 桶
挂账）。探针 **175P/223F 恰中预测**（对 g19 基线 136P/262F 净 +39，余 19
页 ±0）。

## 1. 实现面

- **canonical 机器**（values/random.rs）：`parse_key_section` 结构化
  `<random-key>`（`RandomKey{dashed, element_scoped, ua: UaDomain, fixed,
  is_auto, omitted}`；fixed 形单 token 直拆或 `calc(` 包裹 lite 求值，
  [0,1) 无单位门）+ `canonical_section` 数字节 canonical 化（单 token 单位
  归一 ms→s/turn·grad·rad→deg，余单位原样保往返；多 token calc-sum lite
  求值器 `eval_lite_expr`——括号/乘除链/同单位和差，维度×维度、跨单位
  和差、number/维度混除、除零拒）+ `canonical_key_text` 序归一（§9.4.1：
  auto/省略 → `element-scoped ua-<property>-<index>` 展开；
  `property-scoped`→`ua-<property>`、`property-index-scoped`→
  `ua-<property>-<index>` 替换（index = 声明内 random+random-item 1 基序、
  canonical 重排前文本序）；显式 `--ident`/`element-scoped`/`ua-<ident>`
  保留仅序归一；`fixed <expr>` 独占，`calc(2 / 4)`→`calc(0.5)` 钉）+
  `parse_random_parts`（omitted 判定改双路：identification 解析成功→
  key 形，失败而数字节 canonical 化成功→省略形——`random(10 * 100px,
  200em / 2)` 语料形首节即多 token 数字）+ `canonicalize_random_keys`
  全值扫描（引号状态机含转义、词边界、invalid 不改写、random-item 原样
  透传仅计 ordinal）。
- **写面单点接线**（values/serialize_math.rs）：`fold_decl_specified`
  头部插入 `canonicalize_random_keys`（custom property `--*` 跳过——§9.4
  substitution 语义 random 在未注册自定义属性内不识别）。native setter
  css_style_declaration.rs:284/:546 + bridge replay js_dom_bridge.rs:5491、
  callbacks.rs:3157/:4877 共 5 写面站自动覆盖，engine 侧零改动。
- **validate 面升级**（委派）：`parse_random_args` 委派
  `parse_random_parts`，写面门自动获得 calc-sum-lite 接受面（C 桶三形
  `10 * 100px`/`200em / 2`/`fixed calc(2 / 4)`）。

## 2. 探针（results-g20probe，filter=random 20 页）

对 g19 基线（136P/262F）：

- random-serialize 29P/42F → **68P/3F（+39）**——A/B/C/E 四桶全收口；
  残 3F = `rgb(random(...))/color-mix(rgb(random(...)))` color 通道嵌套
  （D 桶，color parser 求值挂账 Face 2+）
- 其余 19 页（random-computed 13P/141F、random-item-serialize 14P/1F、
  random-invalid 39P/0F、random-item-*、random-in-*）逐位 ±0
- **175P/223F 恰中预测**（+39 零回归）；二进制 provenance epoch
  1794454551（zwg20-build.log，探针/全量账同轮二进制）

## 3. 验证面

- 单测：css-parser lib 3,152 全绿（random 组 11——既有 5 + 新增 6：
  key 展开/替换/序归一 16 形、单位归一与 lite 折叠 10 形、不变形/invalid
  保形、字符串字面与 random-item 免疫、多调用 ordinal、calc-sum 写面门
  接受面/fixed 门矩阵、fold 入口接线/custom property 跳过）；
  style-system 2,525 + 5 全绿；engine 3,903 过（10 失败经 g19 基线法证
  为既有失败：websocket 2/fetch 2/execcommand 2/img settle 2/cssom t49/
  iframesw 1，与本批无关）。
- workspace guarded-clippy 零告警；rustfmt 零漂移。

## 4. 权威账（results-g20，六套件全量）

- values（对 results-g19 基线）：3,072 → **3,111 净 +39，零回归**——
  增益全在 random-serialize 页（29P/42F → 68P/3F）。对账文件
  results-g20/diff-values-vs-g19.txt。
- 五套件零漂移（对 results-g19 基线）：color 10,972 ±0、transitions 200
  ±0、animations 238 ±0、easing 51 ±0、transforms 229 ±0——页级零回归。
  对账文件 results-g20/diff-{color,transitions,animations,easing,
  transforms}-vs-g19.txt（g19 五套件对 g16 已 ±0）。
- 全量账 service zwg20-full 六套件连跑 14m01s 墙钟收官（journal
  11:50:36，exit=0 六套件）。

## 5. 挂账桶更新

- D 桶 color 通道嵌套 random 3F（rgb()/color-mix() 实参槽——color
  parser 未接 random 求值/canonical）+ random-item color 1F
- Face 2：random() engine 求值（sha256 种子 + element/property 上下文 +
  supports 门 + computed 数值化）——random-computed 141F +
  random-item-computed 28F
- random-in-custom-function 8F / random-item-invalid 14F / unresolvable
  6F：custom function 通道与 unresolvable 拒面
- 既有挂账不变：tree-counting 家族残 89F、anchor 953F、独立大桶
  （line-break-ch 194F/attr-all-types 103F/viewport-units 84F/
  calc-size 62F 等）、serialize-resolution 29F
