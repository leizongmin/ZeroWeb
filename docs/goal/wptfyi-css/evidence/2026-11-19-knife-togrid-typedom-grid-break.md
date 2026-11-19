# 刀 TO-GRID：typed-om grid/logical+break 写门扩面（face A）（2026-11-19）

## 目标

css-typed-om 残余 Fails（post-decomp residue survey 候选 #1）grid/logical
族 face A：break-after/before 关键字面 + grid-auto-columns/rows + grid
行列放置四长手 + opaque 往返链。靶面四页（the-stylepropertymap/properties/）：
break.html、grid-auto-columns-rows.html、grid-start-end.html（+ pva 哨兵）。

## 根因（probe-b0 分诊，results-knife-togrid/probe-b0.json）

- typed-om 写门 `_zwOmPropSyntax` 白名单缺 8 属性（break-after/before、
  grid-auto-columns/rows、grid-{row,column}-{start,end}）——set 恒
  TypeError「Unsupported property」、get 恒 opaque 基类。
- `computedStyleMap` 面 gCS 读数本身在册（p1/p3 gCS 'auto' 实证）——
  分型缺口全在声明面语法表，表扩面后 computed 面自愈。
- CSSOM 写面 `grid_cssom::parse_grid_line_token` 假设 span 居首
  （'5 somegridarea span' 形整值拒绝——css-grid-2 §grid-line 的 &&
  组合无序约束未落）。
- css-parser `BreakValue` 缺 left/right/recto/verso（css-break-3
  §break-between 关键字），`BreakInsideValue`/style-system 双侧枚举缺
  avoid-region/region（本刀前半 11:27 落）。

## 码面（7 tracked 文件）

1. **crates/css-parser/src/values/parse_layout.rs**：BreakValue +Region/
   Left/Right/Recto/Verso 五变体（spec 注记在档）；parse_break_before/
   after 各 +4 关键字臂。
2. **crates/style-system/src/property/types.rs**：BreakValue +Region/
   Left/Right/Recto/Verso、BreakInsideValue +AvoidRegion（doc 注明
   css-break-3 章节）。
3. **crates/style-system/src/property/apply_advanced.rs**：break-inside/
   before/after 三处 css-parser→style-system 映射臂扩面。
4. **crates/engine/src/js_dom_bridge/computed_style.rs**：
   break_value_to_css/break_inside_to_css 序列化臂（kebab-case）。
5. **crates/style-system/src/property/parse.rs**（前半 11:27）：auto 族
   track list fit-content() 解耦（grid_single_track_supported(token,
   true)——css-grid-2 §track-sizing）；parse_grid_line span 任意序
   （position 扫描替代首 token 判断）。
6. **crates/engine/src/dom_bindings/grid_cssom.rs**：CSSOM 写面
   parse_grid_line_token span 任意序（canonical 'span 5 somegridarea'
   形）；双整数/双 span 恒拒。
7. **crates/engine/src/js_dom_shim/part10-typed-om.js**：
   `_zwOmPropSyntax` +8 属性（break-after/before 12 关键字、
   grid-auto-columns/rows 六语法含 `<flex>`、grid 四长手 ['auto']）；
   `_zwOmListValuedProps` +grid-auto-columns/rows（css-grid-1 §7.4
   `<track-size>#`，_zwOmTopSplit 括号感知实证 minmax 逗号安全）；
   `_zwOmValidateSet` 负 flex 门（css-grid-1 §7.2.3 fr 非负——
   testsuite `<negative-flex>` 面 TypeError，表无 `<negative-flex>`
   语法即拒；现有表零 `<flex>` 条目=零存量面影响）。

## 钉测（先行 DOA 预核，1 passed 各一枚）

- style-system `parse_grid_line_span_any_order_togrid`（span 任意序 +
  auto 族 fit-content/minmax/fr/3fr + 恒拒形）。
- style-system `parse_break_fragmentation_keywords_togrid`（五关键字 +
  avoid-region + 近似拼写拒）。
- engine `test_grid_line_span_any_order_canonical`（canonical 序列化 +
  拒绝形）。

## 探针（zz-togrid-probe.html 12 测，results-knife-togrid/）

- probe-b0（HEAD bin d0d532df，11:17 构建）：6P/6F——F=p2/p4（computed
  分型缺表）、p9/p10（'3fr'/'auto' 分型缺表）、p11（set 未知属性抛形
  基线）、p12（'5 somegridarea span' 写面拒）。
- probe-b1b（face bin f37636313）：11P/1F——p2/p4/p9/p10/p12 全兑现；
  残 F=p11=基线断言过期形（prop 入白名单后 set 不抛=正确后形态，
  非缺陷）。

## A/B 电池（results-knife-togrid，b0=face 剥除/ride 恒定、b1=face+ride）

11:59 发射 zw-togrid-battery systemd unit（日志
knife-togrid-battery-1158.log），12:27 GATE-DONE rc=0，trap 归一 b1
树态成功。双腿 × {typed-om, pva, masking, transitions, animations}；
ride=FI（animation.rs/transition_registry.rs 刀 FI font 三通道）+
RIDE（es_module.rs/part115 测试）双面恒定，md5 census S1/S3 零扰动。

**css-typed-om b1−b0：gained=99 lost=0 regressed=0（净 +99）**，页级：

- break.html P 87→120（**+33**）
- grid-auto-columns-rows.html P 32→62（**+30**）
- grid-start-end.html P 104→140（**+36**）

哨兵四面（pva/masking/transitions/animations）逐页零差异（ab-compare
仅列上述三页）——face 不越界、ride 中性双面实证。

钉测：battery 内 style 腿 2 passed；engine 腿 battery 窗 rc=124
（test-guard 总内存超限杀树——engine 测试二进制重编译峰值 >20GB，
窗口后重跑 `--lib` 形 + per-proc 10/total 30 预算 rc=0 **1 passed**
实证；预算教训记 NOTES）。三钉最终态全绿：parse_grid_line_span_
any_order_togrid / parse_break_fragmentation_keywords_togrid /
test_grid_line_span_any_order_canonical。

## 预期与残面

- 预期 vs 实际：break.html ≈35 → 实 +33、grid-auto-columns-rows.html
  ≈28 → 实 +30、grid-start-end.html ≈25 → 实 +36；pva 哨兵净 0 → 实
  逐页零差异。
- 残面（face B/C，后续刀）：grid-template-columns-rows 54F（CSSKeywordValue
  'none' + track-list throws 面）、logical.html 58F（percent compute）、
  scroll-padding 重ify 36F、arithmetic.tentative 47F（CSSUnitValue 数学
  API）。
