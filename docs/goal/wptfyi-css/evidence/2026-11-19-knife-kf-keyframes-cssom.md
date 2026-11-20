# 刀 KF：@keyframes CSSOM cssRules 读/写面（2026-11-19）

## 目标与定谳

残面候选（NOTES 17:4x 段 pre-survey）：keyframes-name-valid 39F「未深挖，
face=放宽名 token 前置校验」——recon 改判定谳：**非 token 过严，是
CSSOM 读面整层缺席**。

- `keyframes-name-valid.html` 39/39 全败（含 `@keyframes example { }`
  最简形），`keyframes-name-invalid.html` 20/20 全过——invalid 侧 vacuous
  过：harness（parsing-testcommon.js `test_keyframes_name_valid`）断言
  `sheet.cssRules.length === 1`，@keyframes 规则不入 cssRules 即两侧同
  形虚过/空 Rao。
- 根因：`css_wire.rs` `style_rules_text` wire 只发 StyleRule/
  position-try/font-feature-values/property 四类，keyframes 显式 defer
  （尾注「其余 @-rule / keyframes / @media defer」——fv1/ATP 两刀扩面后
  注记已陈旧）。规则解析层本来就能产 `Rule::Keyframes`（动画回放走独立
  通道），仅 CSSOM 读面断链。
- 附靶 `keyframes-rule-caching.html` 1F：`sheet.cssRules[0].cssRules[1]
  .style.color` 嵌套写面，同属 CSSKeyframesRule 接口缺席。

## face（10 文件 3 crate；全冷域，animation.rs hunk 精确拆分）

1. `css-parser/parser/at_rules.rs`：`consume_keyframes_rule` 名文法门——
   <custom-ident> 排除 CSS-wide 关键字 + none（大小写不敏感，counter-style
   名引用排除同型），<string> 排除空串；`name_is_string` 保形记录。
   无效名 None 走 dispatch 共享残余消耗（body 未消费，同非 Ident/String
   臂）。
2. `css-parser/ast.rs`：`KeyframesRule.name_is_string` 字段——CSSOM 串化
   保形（string 形名带引号重排，`"none"` 重解析归 String token 绕
   custom-ident 排除——r0 solo 7F 定谳后补）。
3. `css-parser/serialize.rs`：`serialize_keyframes_css_text` canonical
   cssText + `serialize_keyframe_selectors`（from/to 保词、百分比整数去尾）。
4. `engine/js_dom_bridge/callbacks.rs`：`__zw_parse_keyframes_rule` 宿主
   单源回调（`__zw_parse_property_rule` 同型，ATP 单实现双通道先例）。
5. `engine/js_dom_bridge/css_wire.rs`：`style_rules_text` Keyframes wire
   臂（selectorText 字段 = `@keyframes` 标记，shim 据此分派物化）。
6. `shim part06.js`：CSSKeyframesRule（type 7）/CSSKeyframeRule（type 8）
   接口壳 + `_makeKfRule` 物化（嵌套 cssRules 经 `_makeRuleStyle` 嵌套
   style 写面、`_kfFlush` 重算父 cssText 上 sheet、flushToOwner join 回写
   `<style>` 文本引擎侧再解析归 Rule::Keyframes 往返稳定）+
   appendRule/findRule/deleteRule + cssRules 读臂 + insertRule 写臂
   （宿主解析空返 ⇒ SyntaxError DOMException；`@keyframes-` 前缀异名
   at-rule 经宿主解析非 Keyframes 规则同臂拒）。
7. `animation.rs` + style-system 三测试件：`KeyframesRule` 字段适配
   （animation.rs 仅 make_keyframes 测试 helper 1 行，与 computed_font
   流在飞 WIP hunks 域零交叠，patch hunk 精确拆分）。

## r0→r1 solo 探针（results-knife-kf r0bin/kf-runner）

r0（bin md5 `e0ff784773`，无 name_is_string）：name-valid 7F——`"none"`/
`"default"` 等 string 形 CSS-wide 名：canonical 串化裸排 ident 形 → 宿主
再解析撞新文法门 → 条目弃 → cssRules 恒 0。定谳 = 串化丢 token 形，补
`name_is_string`（face ②）。

r1（bin md5 `ec826b2fea`，shim include_str 换装入 bin）：**name-valid
39/39 全绿 + name-invalid 20/20 持位 + keyframes-rule-caching 1/1**。
animations 语料净 +40。

## A/B 电池（results-knife-kf，zw-kf-battery-2337，23:36:22 发射）

occupancy gate 通过（OCCUPIED=0）后重发；b0 构建 52.43s、b1 构建 53.39s，
12 腿全完成（transitions 单侧 ~8 分钟为该语料正常耗时）。

- M2 双 bin 异 md5：b0-runner `1f0a2a9cd4a0e020`（face 剥除）≠ b1-runner
  `ec826b2fea3fc1d7`（face 应用）——b1 与 solo r1 bin 同 md5，solo 探针与
  电池 b1 侧二进制连续。
- S1/S2/S3 ride census 三代文件逐字节一致（19 文件）——窗内零扰动。
- RECONCILE（ab-compare b0→b1）：
  - `keyframes-name-valid.html` 0/39 → 39/39（+39 主靶）
  - `keyframes-rule-caching.html` 0/1 → 1/1（+1 嵌套写面）
  - `keyframe-selectors.html` 0/14 → 7/14（+7 扩面红利；残留 7F =
    `entry`/`exit` scroll-driven + `calc()` 形 keyframe selector，
    css-animations-2 新文法独立面，候选池候选）
  - `keyframes-name-invalid.html` 20/20 → 20/20 持位——语义升级：b0 侧为
    vacuous 过（规则不入 cssRules 断言空真），b1 侧真过（invalid 名经文法
    门真拒绝、`cssRules.length===1` 断言两侧成真检查）
- `== TOTAL: gained=47 lost=0 regressed=0 pages_gain=3 pages_loss=0`；
  fonts/pva/shapes/typed-om/transitions 五哨兵语料零页差异（RECONCILE 无
  条目）；P2F=0 判据达成。
- TRAP-NOOP state=done rc=0：树留 b1 face 态即提交态。

## 发射窗记

23:30 首射未闸 shapes-b2 在飞（zw-shapes-b2-232652 23:26:52）——queue
违规自查让位（unit stop + 手动归位 face；shapes-b2 构建相先完成、face
零交叠、S3 census 先于本流 strip，其账无损）。重发待其 GATE-DONE。
