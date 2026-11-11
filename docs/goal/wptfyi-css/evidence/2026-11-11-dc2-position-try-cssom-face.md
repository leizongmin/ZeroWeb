# DC-2 @position-try CSSOM 面账（anchorbase 9,893 → 10,182 净 +289）

2026-11-11。接续 [position-area 语法面批](2026-11-11-dc2-position-area-grammar-face.md)
后的 anchor 语料 CSSOM 缺口：at-position-try-cssom 265F + at-position-try-parse
11F + at-position-try-allowed-declarations 13F + caching-002 1F。

## 批内容

1. **css-parser `PositionTry` 变体**：`Rule::PositionTry(PositionTryRule{name,
   declarations})` + `consume_position_try_rule`（css-anchor-position-1
   §position-try-fallback）。prelude 须 `--` 开头 Ident（tokenizer 已解码转义
   名，`--\{` → `--{`），非 `{` 块或非法 prelude 返回 None——链式
   `.map(Rule::PositionTry)` 让链尾 `skip_malformed_qualified_rule` 整条 extent
   消耗。**泄漏回归位**：探针实证早写法（early-return）使 `@position-try broken
   { top: 1px; }` 的 body 漏成顶层样式规则；链式臂修之（单测
   `test_parse_position_try_invalid_prelude_discarded` 钉面）。style-system
   matcher 补 `Rule::PositionTry(_) => skip` 穷尽臂（解析进级联前跳过）。
2. **css_wire wire 条目**：`style_rules_text` PositionTry 臂——selectorText 字段
   = `@position-try <name>`（真实选择器不可能 `@` 起头，shim 侧据此分派物化），
   cssText 按 CSSOM 规则序列化形（空块 `{ }` 单空格，Chrome 同形）；
   `serialize_css_ident`（cssom-1 §serialize-an-identifier）对解码 ident 重转义
   （`--{` → `--\{`，NULL → U+FFFD）。
3. **part06.js CSSPositionTryRule / CSSPositionTryDescriptors 物化**（四处）：
   接口壳 + `_zwPtAllowed` 38 键描述符白名单（margin-*/inset-*/top/left/right/
   bottom/size 族/place-self/align-self/justify-self/position-anchor/
   position-area，键集外 getPropertyValue→''、`[prop]`→undefined、setProperty
   忽略、`!important` 拒）+ `_makePtRule` **单实现双通道**（grid G1 先例形态：
   wire cssRules 读面与 insertRule 写面共用同一构造器）+ `_makeRuleStyle`
   allowedSet 过滤参数 + `_ruleFromText` insertRule 臂（块名 `--` 校验，非法抛
   `SyntaxError` DOMException）。`@-rule defer` 注记更新：cssRules 读仅
   StyleRule + position-try。

## 验证（两段跑测 + 单测）

- **ptcssom1（读面）**：anchor 语料 **9,893P → 10,158P 净 +265**——
  at-position-try-cssom 0P/265F → 264P/1F（残余 1F = setProperty allowed/
  disallowed 案，断言期望 0 得 8，fallback 布局依赖面）；position-try-rule-
  caching-002 +1 连带；css-color 10,905P 零漂移。
- **ptcssom2（写面 insertRule）**：anchor **10,158P → 10,182P 净 +24**——
  at-position-try-parse 1P/11F → 12P/0F（+11）、at-position-try-allowed-
  declarations 43P/13F → 56P/0F（+13）；对 ptcssom1 逐页零回归（三方逐页
  比对 REGRESSIONS: NONE）；css-color 10,905P 零漂移。
- **批合计**：anchorbase2 基线 9,893P → 10,182P 净 +289 逐页零回归。
- **单测**：css-parser 6 案（基本形/空块/转义名解码/无效 prelude 整条丢弃/
  important 位/与 Style 混排序）+ engine wire 4 案（条目格式/空块 `{ }`/
  转义名重转义/混排 wire 序）——`position_try_at_rule.rs` +
  `css_wire.rs::position_try_wire_tests`。
- make test：全量门禁 **3,854P/7F**（exit 101，wptfyi-ptcssom2-maketest-2.log）
  ——7F 与 master.md 碰撞记录定谳的 main 既有欠账组逐名一致
  （js_dom_bridge::tests img/svg settle、fetch/iframe/sw、response body、
  websocket 族七钉），零新增失败；本批 10 新钉（css-parser 6 + wire 4）全绿；
  fmt 门禁前置曾抓 at_rules.rs 构造体格式漂移，收敛后复跑通过。
- 证据产物：`.acceptance/wptfyi-css/results-ptcssom{1,2}/`（JSON），日志
  `logs/ptcssom{1,2}-*.log`、`logs/wptfyi-ptcssom2-maketest-*.log`。

## 挂账

anchor 桶残余归组 F 布局行为（position-area scrolling/insets/wm-dir/
align-justify/in-position-try ~329F，需 anchor 定位布局实现）；本页 1F
（setProperty 计数案）随组 F 布局面处理。
