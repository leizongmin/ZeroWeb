# 刀 fv1 · @font-feature-values 全链接入（2026-11-18）

## 认领与背景

- 认领：[CLAIM-knife-fv1-font-feature-values.txt](../../../.acceptance/wptfyi-css/CLAIM-knife-fv1-font-feature-values.txt)（2026-11-18 04:1x，fork 在飞码树内接手收口轮）。
- 池：master head「候选 @font-feature-values 294F」。靶面四页：
  `test_font_feature_values_parsing.html`（主靶）+ `font_feature_values_map_iteration.html`
  + `font-feature-values-map-live.html` + `font-feature-values-sameobject.tentative.html`。
- 本轮 429 断点接手：原认领 fork 与 resume-swarm 双 fork 同窗死于使用上限；本 session
  以树内在飞码（css-parser 4 文件 + engine callbacks/css_wire/part06 + style-system
  apply.rs）为基础收口，另补齐三处缺口（见修复节）。

## 码面（12 文件）

css-parser：
1. `parser/at_rules.rs`——`consume_font_feature_values_rule` 族名列表文法（通配/`--`
   前缀未引号拒、引号形放行）、畸形嵌套值 at-rule 吞至 `;`/块尾（规则存活仅值定义缺失）、
   junk 块整体吞匹配块（保后随合法值 at-rule）、值定义个数上限（styleset ≤20、
   character-variant ≤2、其余恰 1）+ 值名 case 显著窗。
2. `serialize.rs`（新）——cssom-1 §serializing：`serialize_css_ident`（§serialize-an-identifier）
   + `serialize_font_feature_values_css_text` canonical 全文（insertRule 与 stylesheet
   两通道单源裁决）。
3. `values/parse_layout.rs`——`parse_font_variant_alternates` `character_variant`
   `Option<String>`→`Vec<String>`（css-fonts-4 §font-variant-alternates-prop 文法
   `<feature-value-name>#`，wpt font-variant-alternates-valid 钉三名列表合法；同函数
   重复仍拒）。
4. `tests/font_feature_values.rs`——钉测随码（列表形合法 + 重复拒 + swash 多参拒）。

style-system：
5. `property/apply.rs`——`validate_decl_value` 白名单 58→59 接入 `font-variant-alternates`
   （var()/env()/attr() 替换函数形豁免，其余走 apply 链既有 parse 文法裁决）。
6. `font_feature_values.rs`——`character_variant` 列表形消费（逐名 lookup cv tag）。

engine：
7. `js_dom_bridge/callbacks.rs`——`__zw_parse_ffv_rule` 单规则解析 + canonical cssText +
   六类 defs JSON（空返 = 整条非法 → shim SyntaxError，cssom-1 insertRule 语义）。
8. `js_dom_bridge/css_wire.rs`——`style_rules_text` FontFeatureValues wire 条目
   （`@font-feature-values\x1e<cssText>` 标记字段，真实选择器不可能以 `@` 起头）。
9. `js_dom_shim/part06.js`——insertRule FFV 分支（`@font-feature*` 前缀整族）+
   CSSFontFeatureValuesRule/CSSFontFeatureValuesMap 接口全局 + `_makeFfvRule` 物化
   （type 14 + fontFamily + 六类 map）+ getRules wire 读面物化。

## 接手补齐三缺口（原 fork 未及）

10. `registered_properties.rs` 域外：`part06.js` `CSSFontFeatureValuesMap.prototype`
    补链 `Map.prototype`（`_makeFfvRule` 的 setPrototypeOf 截断 Map 链——entries/set/
    forEach 全丢，map_iteration/map-live 三 F 根因）+ `set` 标量→单元素列表归一化
    （css-fonts-4 §om-fontfeaturevalues maplike，map-live `set(k,42)`→`get`→`[42]` 钉）。
11. `computed_style.rs` `font_variant_alternates_str` character-variant 列表串化
    （逗号+空格 join，canonical 形与 valid 页 expected 串一致）。
12. （RP1 域文件在树同窗，另账见其节）

## 钉测

- css-parser `parses_font_variant_alternates_combination_and_rejects_duplicates` 扩：
  character-variant 列表形 + 同函数重复拒 + swash 多参拒。

## 探针定谳（bin = 树内 fv1+RP1+s419 同态，release 42s 建）

- 四 FFV 页 solo/多页两形态全绿：主靶 302P/0F、map_iteration 1P、map-live 2P、
  sameobject 1P（probe-fv1-takeover/ 工件）。
- `font-variant-alternates-invalid.html` 15P/0F（apply 门兑现）；
  `font-variant-alternates-valid.html` 3P/0F（列表形修复兑现）。
- 专设探针页（zzz-probe-fv1，发射前已删，页集零污染）三测：stylesheet 通道物化
  type 14 + entries 迭代 + set/get/delete/list 形。

## A/B 定谳（七语料 × b0/b1，results-knife-fv1/）

电池窗 15:49:51–16:23:12（双建 44s+45s，法证 M2 新文件往返精确还原、S2 ride-along
窗内零扰动、fp-count b0=1 b1=16）。逐腿收账（knife-fv1-account.py）：

| 语料 | b0 | b1 | ΔP | F2P | P2F | NEW/LOST |
|---|---|---|---|---|---|---|
| css-fonts | 5441P/2814F | 5744P/2501F | **+303** | 307 | 0 | LOST=10（见异常 1） |
| cssom | 1962P/1517F | 1968P/1511F | **+6** | 6 | 0 | 0/0 |
| css-masking | 4703P/1652F | 同 b0 | 0 | 0 | 0 | 0/0 |
| css-shapes | 5915P/1600F | 同 b0 | 0 | 0 | 0 | 0/0 |
| css-transitions | 823P/2388F | 823P/1828F | 0 | 0 | 0 | LOST=560（见异常 2） |
| css-backgrounds | 3490P/2756F | 同 b0 | 0 | 0 | 0 | 0/0 |
| css-animations | 516P/786F | 516P/788F | 0 | 0 | 0 | NEW=3/LOST=1（见异常 3） |

**净账（认领口径 F2P−P2F）= +313，P2F=0 全域。**

F2P 分页：主靶 `test_font_feature_values_parsing` 288（b1 302P/0F = solo 探针
canonical 全绿形）、`font-variant-alternates-invalid` 15（apply 白名单门兑现）、
`font-feature-values-map-live` 2、`CSSFontFeatureValuesRule`（cssom）6、
`font_feature_values_map_iteration` 1、`sameobject` 1。

异常三案法证：
1. 主靶 LOST=10：b0 面 312 子测（面破坏形态多发 10 个幻影子测）/ b1 面 302 =
   探针 canonical 数——LOST 为 b0 侧破碎行为产物，非 b1 收缩，判良性。
2. `css-transitions/properties-value-inherit-001` LOST=560：battery b0 输出 561
   子测（全 0P/F）后 completion 挂起；b1 脚本 90s 硬超时零子测。solo A/B 复现
   （probe-inherit001/）：**两侧同样 `Execution timeout: 90000ms`**——预存挂起形
   （HEAD 即挂，fv1 面外），battery b0 的 561 输出为窗内时序幸存形态；全程零
   pass 流失，归 transitions 预存残面（事件系统鲸面同域）不入本刀账。
3. css-animations 两 tentative 页（Element-getAnimations/display-none-dont-cancel）
   NEW/LOST 微动：双侧均带 runner 墙 synthetic 条目（60s 墙边陲页），ΔP=0，
   判 flake 级。

## 门禁

- css-parser lib 3310/0、style-system lib 2566/0、engine lib 4188/0（test-guard 包裹）。
- 本刀 hunks fmt 清、定向 clippy 零新增（在案 3 lint：es_module.rs too_many_arguments
  归 s419 在飞 WIP、animation.rs:2452/2453 unnecessary_cast 归既有提交态）。

## 残面池

css-fonts b1 残面 2501F/127 页，头部分页（census 于收账 JSON）：

| 页 | F |
|---|---|
| animations/system-fonts.html | 396 |
| animations/font-size-adjust-composition.html | 250 |
| animations/font-variation-settings-interpolation.html | 234 |
| animations/font-stretch-interpolation.html | 153 |
| animations/font-palette-interpolation.html | 114 |
| animations/font-style-interpolation.html | 109 |
| discrete-no-interpolation.html | 104 |
| variations/at-font-face-descriptors.html | 88 |
| parsing/font-computed.html | 77 |

头九页 ~1525F 占六成，animations/interpolation 族为 css-fonts 主残面（动画插值
引擎域，非 fv1 字体特性域）；其余 118 页 ~976F 长尾。另有 css-transitions
`properties-value-inherit-001` 预存挂起形（异常 2，HEAD 即挂）挂账。
