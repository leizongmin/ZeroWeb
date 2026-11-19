# 刀 TO-GRID face B：grid-template 表 + scroll-padding 简写 + track-breadth 负值/calc（2026-11-19）

## 目标

css-typed-om 残面刀 TO-GRID face A 后续：grid-template-columns-rows 54F +
scroll-padding 简写 4F + grid-auto-columns-rows 残 10F 中的声明面缺口
（face A 配方边界样本实证残形 = 负值/calc 声明 null + computed px 换算）。
computed 语义层（calc 折叠 / % 保留 / em→px 换算）为 face C 另刀，不入本刀。

## 根因（探针分诊，results-knife-faceb/）

- **shim 白名单层**：grid-template-columns/rows 在 `_zwOmKnownExtra`
  （已知无语法）——set 无裁决恒过（32 无效形静默落 CSSOM 丢弃=「did not
  throw」）、get 恒 opaque（12 分型面 + 'none' 关键字 2 形）；
  scroll-padding 简写完全未知（4 形「Unsupported property」）。
- **Rust CSSOM 层**：`grid_length_percentage_supported` 硬性 `>= 0.0` 且
  无 calc 臂——负长/负百分/calc 形 CSSOM 写面声明丢弃 → 声明面 get null
  （face A 残 10F 的 8 形）。
- testsuite 合同定谳（探针 + testsuite.js 源）：testUnsupportedValue =
  opaque 往返非 throw；负值 specified 面期望 CSSMathSum 包裹（shim 负值
  包裹集已含两族属性）；'Incompatible units' 为 shim 转换异常（face C 面）。
- 探针教训：元素未挂载文档树时 % 不解析、calc 不转换——假绿三处；挂载后
  与电池账 scroll-padding 页失败集 name+message 双向 diff = 0。

## 码面（3 tracked 文件）

1. **crates/style-system/src/property/parse.rs**：
   `grid_length_percentage_supported` 负值放宽（`is_finite`，css-grid-2
   §track-breadth 文法允许负 length-percentage——与 <flex> 显式非负不同，
   fr 门保持 `grid_non_negative_number_with_suffix`）+ calc() 臂（复用
   css-parser `parse_calc` + `calc_expr_is_length_percentage`，css-values-4
   §10.10 按结果型别裁决）。
2. **crates/engine/src/dom_bindings/grid_cssom.rs**：钉测
   `test_grid_track_negative_and_calc_accepted`（负长/负百分/calc 两族 +
   <flex> 负值恒拒 + 混型/病态 calc 恒拒 + line-name 组合不回退）。
3. **crates/engine/src/js_dom_shim/part10-typed-om.js**：
   `_zwOmPropSyntax` +grid-template-columns/rows
   `['none','<length>','<percentage>','<flex>']`（css-grid-2 §track-sizing；
   WPT runPropertyTests 全集含 <flex> specified-only）——负 flex 由通用负
   flex 门拒；`_zwOmKnownExtra` 摘二增一（'scroll-padding' 简写，opaque
   往返面）。

## 风险核查

- layout 消费方（layout-engine converter/grid.rs）独立更严二次解析门
  （`parse_non_negative_number_with_suffix`）——负值轨道到 layout 仍拒转
  auto，无渲染面越界。
- shim 侧 line-name/repeat 组合：set 侧 opaque kind 恒放行 + CSSOM parse
  期已接受（b1 账 linename/repeat 面今天已过）——白名单升级不破坏
  testUnsupportedValue 链。

## 钉测（双绿）

- style-system `parse_grid_track_list_negative_and_calc_togrid`（1 passed；
  `parse_grid` 全家 12 测零退化——border-width 关键字拒/none-line-name/
  span 任意序/fit-content 面全过）。
- engine `test_grid_track_negative_and_calc_accepted`（1 passed；decomp
  预算形态 per-proc 10/total 30）。

## A/B 电池（results-knife-faceb，b0=face 剥除、b1=face+ride，13:05→13:32）

双腿 × {typed-om, pva, masking, transitions, animations}；ride=FI
animation.rs/transition_registry.rs（已入库）+ RIDE es_module.rs/part115
测试双面恒定，md5 census S1/S3 零扰动。systemd zw-faceb-battery 单 unit
发射（会话重启风暴下电池存活实证）。构建 b0 39.13s（bin
be09352a）/b1 40.29s（bin 3295cee6）M2 区分过；钉测双绿（style togrid
3 passed rc=0、engine decomp 形 1 passed rc=0）。全语料 rc 全 1=正常。

**typed-om gained=48 lost=0 regressed=0**（pages_gain=2）：
grid-template-columns-rows 14→58（+44）/ scroll-padding 204→208（+4）；
grid-auto-columns-rows 零位移。pva/masking/transitions/animations 哨兵
四面逐页零差异（全语料 ab-compare 仅 2 靶页出现位移，regressed=0），
GATE-DONE rc=0 trap 归一 b1 树态（TRAP-RESTORE applied）。哨兵 pva/masking/transitions/animations
四面全语料级零差异（全目录 ab-compare TOTAL gained=48 lost=0
regressed=0 即四面零贡献，收账 13:31:57 独立复跑实证）。电池
GATE-DONE 13:31:57、battery rc=0、trap TRAP-RESTORE applied
(state=done rc=0) 归一 b1 树态。本节前段「13:05→13:32」时间戳与
「全语料 rc 全 1」在 13:24 预写、13:31:57 实测核账兑现一致。

## 预期偏差与残面改判（+56 → +48）

预测 grid-auto +6 未兑现：残 10F 与 grid-template 残 10F 为**同一 5 形
族 × 2 属性**——`-3.14em`、`3.14cm`、`calc(0px + 0em)`、`-3.14%`、
`calc(0% + 0%)`。全是 readback/换算形（b0=b1 逐形同集），非声明形——
face A 时的「声明 null」归因按本电池逐形比对改判：grid 声明面
（负长/负百分/calc 臂）已被本刀全清（grid-template 44 形兑现），
残面整体归 face C engine computed 序列化层（gCS 回显），grid-auto 无
独立残面。S2/S3/S4 三类（calc 同单元折叠/% 保留/em→px 换算）在此 20F
内合并收口。

## 预期与残面

- 预期：grid-template-columns-rows 54→~8F（computed px 换算/钳 0% 残），
  grid-auto-columns-rows 10→~4F（同），scroll-padding +4F。
- 残面（face C，后续刀）：S2 computed calc 同单元折叠（logical 24x+6x、
  margin-top 控制组 m5 实证未折叠）、S3 computed % 保留（scroll-padding
  16x、border-*-radius 12x）、S4 computed em/cm→px 换算（scroll-padding
  16x、logical 简写 12x、grid 8x）、arithmetic.tentative 47F（CSSUnitValue
  数学 API）。
