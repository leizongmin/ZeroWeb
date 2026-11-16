# shape-margin / shape-image-threshold 五站小刀（shapemargin，2026-11-16 二十三轮）

shapepos 刀（二十二轮）落地后的下一残面刀：css-shapes-1 §shape-margin-property /
§shape-image-threshold-property 两属性五站接线（style-system 类型/字段/默认/registry/
apply 解析臂 + apply.rs 写面门白名单+裁决 + css-parser decl canonical + engine gCS
串化/transition get 臂 + animation 三插值器臂）+ 双向钉两处。css-shapes corpus 净
**+187 零回归**（gained=187 lost=0 flip=0，19 页全 GAIN），护栏 css-masking
SET-IDENTICAL、css-typed-om +1（stylepropertymap/shape-margin 转真），**总账净 +188
零回归**。

## 值域双层级模型（刀前钉死）

- **shape-margin**：非负 `<length-percentage>`——字面负值 apply 期钳 0（`clamp_non_negative_length`），
  Calc 形保留（负钳在 gCS 求值层 `shape_margin_to_css`：静态求值出负 px 串 → `0px`）；
  em 按字号绝对化（`normalize_font_relative_length`），%/物理单位保留/折 px。关键字域
  拒收同 margin 族（auto/thin/medium/thick/min-content/max-content/fit-content/stretch/
  -webkit-fill-available + 解析器副产变体全拒——shape-margin 无 auto 形，css-shapes-1 文法）。
- **shape-image-threshold**：specified 级 `<number>|<percentage>` 不钳（decl canonical
  pct→num/100 折算，`-100%`→`-1`、`-7` 原样）+ computed 级钳 [0,1]（apply 期
  `parse_opacity` 语义恰同——负→0、12.5→1、80%→0.8，直接复用单点）。gCS 发射
  computed 值 number 形（f64 Display 最短形恰合语料整数期望：0/0.5/0.8/1）。

## 五站刀界（八站全落）

1. **style-system 类型**（types.rs）：`ShapeMargin(LengthValue)` 变体（ShapeOutside 后）；
   computed_style.rs 字段 `shape_margin: LengthValue` + `shape_image_threshold: f64`；
   default_impl.rs `Px(0.0)` / `0.0`；registry initial_value 两臂 + known_properties
   白名单第七/第八串。
2. **apply 解析臂**（apply_advanced.rs，shape-outside 臂后）：shape-margin 臂
   （`parse_length_or_math` + `shape_margin_length_is_valid` + em 绝对化 + 非负钳）；
   threshold 臂 = `values::parse_opacity(value)` 直存（computed 钳语义同源复用）。
3. **inline 写面门**（apply.rs）：VALIDATED_PROPERTIES 20→22 两串 + 两裁决臂（先
   `value_has_substitution_function` 放行，再 shape-margin 域裁决 / `parse_opacity`
   裁决——序陷阱教训（v1 b1v3「白名单接入≠兜底可达」）直接前置到放行臂之后）。
   helpers：`shape_margin_length_is_valid`（关键字域拒收 + 变体拒收）、
   `clamp_non_negative_length`（负 px/% 钳 0，Calc 保留）。
4. **decl canonical**（css-parser serialize_math.rs fold_decl_specified）：
   `shape-margin` `"0"`→`"0px"`；`shape-image-threshold` pct 剥后缀 `/100` finite 过滤
   （specified 级不钳）。
5. **engine 三处**：gCS 分派两臂 + `shape_margin_to_css`（Calc 静态求值负 px 钳 0）；
   transition `get_property_value` 两臂（margin 经 font 提取走 shape_margin_to_css）；
   animation 三插值器（interpolate_property_value 通用臂 margin `{:.2}px` lerp 钳 0 /
   threshold `{:.4}` lerp 钳 [0,1]；interpolate_property_value_strict 两臂（px 严格 /
   number 严格，失配回落 None = 离散）；apply_single_property 写回两臂（`Px(parse_px.max(0.0))`
   / `parse_f64.clamp(0,1)`）。

## 常驻钉（刀面三处）

- engine part03 `test_get_computed_style_shape_margin_threshold`：缺省 `0px`/`0`、
  px/%/em（16px 基 32px）/`0` 无单位折 px、负值钳 `0px`、auto/thin 拒收回落、threshold
  `0.5`/`80%`→`0.8`/`-7`→`0`/`12.5`→`1`/auto 拒收。
- engine part02 supports 双向钉五串（R2785 backdrop 同款第九块）：`shape-margin 10px`=T、
  `auto`=F、`shape-image-threshold 0.5`=T、`80%`=T、`bogusxyz`=F。

## 门禁与构建链

- 定向钉测试（test-guard --compile-first per-proc 6G/total 12G/wall 1200s）：
  part03 新钉 + shape_outside 存量钉 + part02 supports 钉 **3 passed**（编译 30.86s）。
- fmt 三 crate 净；clippy style-system+css-parser `-D warnings` 零告警；engine
  `--all-targets` 我文件零告警（collapsible-if 自修一轮；唯一余量 warning 在
  es_module.rs:2217 too_many_arguments——peer s419+ WIP 在飞面，避让不代修，
  make test 拆腿账既有）。

## A/B 语料账（B0 = shapepos 提交态 vs B1 = 本刀）

B0 = bin-shapepos-b1v8 md5 `84815f65a818fe07c9be279a55153412`（20:31 快照，shapepos
提交态，css-shapes 账复用 results-shapepos-b1v8/css-shapes.json 4535P/2979F）；B1 =
bin-shapemargin-b1 md5 `5c639a1a93c225fe0c23b086632090bb`（release 重建 48.99s）。
五腿 21:05-21:13（launch-shapemargin-ab.sh，systemd-run user unit zw-shapemargin-ab，
test-guard per-proc 4G/total 8G/wall 900s，CASE_WALL_S=90）全腿 LEG-EXIT=0。
**es_module 漂移切片记录**：peer WIP mtime 20:42 晚于 B0 快照 20:31——B0/B1 携
不同 vintage 的 peer es_module WIP；css-shapes/masking/typed-om 语料纯 CSS
testharness 页不触 ES module 通道（护栏双腿 0/0/0 亦旁证），不入计分归因。

| 套件 | B0 | B1 | Δ | census |
|---|---|---|---|---|
| css-shapes | 4535P/2979F | **4722P/2792F** | **+187/0/0 净 +187** | 145 页双侧，子测分母 7515 恒 |
| css-masking | 1818P/4522F | 1818P/4522F | **SET-IDENTICAL**（逐子测 0/0/0） | 61 页双侧 |
| css-typed-om | 9872P/2487F | **9873P/2486F** | **+1/0/0** | 359 页双侧，分母 12359 恒 |

- **css-typed-om +1**：`the-stylepropertymap/properties/shape-margin.html` 单子测转
  Pass（shape-margin 入白名单后 StylePropertyMap 已知属性面）——增益侧，非回归。

**css-shapes 19 页全 GAIN 零回归**（reconcile-shapemargin.py 逐案逐子测对账，
gained=187 lost=0 flip=0）：

| 页 | Δ |
|---|---|
| animation/shape-image-threshold-interpolation | +62 |
| animation/shape-margin-interpolation | +56 |
| animation/shape-margin-composition | +15 |
| values/shape-margin-001 | +10 |
| values/shape-image-threshold-000 | +5 |
| values/shape-margin-004 | +4 |
| values/shape-image-threshold-001 | +4 |
| parsing/shape-image-threshold-computed | +6 |
| values/shape-margin-002 | +3 |
| values/shape-margin-003 | +3 |
| parsing/shape-margin-computed | +3 |
| parsing/shape-image-threshold-valid | +3 |
| values/shape-image-threshold-003 | +2 |
| values/shape-margin-000 | +2 |
| values/shape-margin-005 | +2 |
| parsing/shape-image-threshold-invalid | +2 |
| parsing/shape-margin-invalid | +2 |
| parsing/shape-margin-valid | +1 |
| inheritance | +2 |

census timeout=1 未产生任何子测状态翻转（分母与 Pass 账双侧恒等）。

## 残面队列（B1 剩余 F 构成）

- **interpolation 混单位端点对**（margin 页余 64F/threshold 页余 58F）：`from px` ↔
  `to %` 等单位失配对 strict 解析回落 None = 离散面（css-transitions 非同构对不启动
  语义），与 margin `%` 项布局依赖求值面同域——需 % 端点插值器（布局期解算）后置。
- **composition add 累加组**（margin 页余 25F 全 add-composite）：引擎无 additive
  accumulate 机制（got "50px" = underlying echo），与全属性 add 语料共性缺口，非本刀
  属性特有。
- **shape-margin-001 物理单位组部分**（余 3F）：`10in` 形 parse 期折 px 发射 `960px`
  vs 期望串化微差（roundResultStr 归一边界），1-2 案级后置。
