# DC-2 切片 10 sanitizer 刀⑬ 判读证据——深条目属性 ns 发射（foreign namespace 写链保真）

**日期** 2026-11-25 · **判定** 刀⑬ 落地，sanitizer 净 +63（133F→70F，basic-filtering 整页清零），五腿 join 零回归

## 刀面（4 文件，+96/−24）

1. `crates/engine/src/js_dom_bridge.rs`：新 `deep_attr_entry_json` helper——qname＋`ans`/`al`/`ap` 三字段（adjust foreign attributes 命中形才发，XHTML ns 与 null ns 省略）；`deep_entry_json_unsafe`＋`deep_entry_json` 双发射器 attr 闭包统一同源（spec: [adjust foreign attributes](https://html.spec.whatwg.org/multipage/parsing.html#adjust-foreign-attributes)）。`crates/dom/src/node.rs` re-export `markup5ever::Attribute`。
2. `crates/engine/src/js_dom_shim/part03.js`：`_zwMApplyAttrNsMeta`（带 `ans` 条目灌 R190 印记 `{ns,prefix,local}`——`_zwMAttrNS` 读面消费约定；写 DOM 字段名 namespaceURI/localName 则读面不消费＝初版根因）；`_zwMBuildDeepEntry`＋`_zwMBuildDeepTreeNode` 双调用点；`_wcRebuildAsHandle` 非 HTML ns 元素 `createElementNS`（失败回落 createElement）＋带 ns 属性 `setAttributeNS`（失败回落 qname setAttribute）。
3. `crates/engine/src/js_dom_bridge_tests/part159_s10k13_foreignns_pins.rs`（新，120 行）：四组钉＋反向假恒真守卫（handle 三形态三元组/doc 域物化/isEqualNode 配对 vs createElementNS 期望树/消毒链元素 ns）。
4. `crates/engine/src/js_dom_bridge_tests.rs`：include 接入。

## 根因链（probe1-7 七轮定谳，修正刀⑫ R7/R8/R9 旧「parser 不产 ns」结论）

- 引擎发射端完好：`__zw_parse_html_subtree_unsafe` RAW JSON 对 ctxTag div/'' 均发 SVG/MathML 元素 ns＋adjust 命中属性全三元组（probe7 RAW 实证：`"n":"xlink:href","v":"u","ans":"http://www.w3.org/1999/xlink","al":"href","ap":"xlink"`）。
- 丢失点①：shim 深臂物化器只灌 `{n,v}`——`_zwMAttrNS` 读面（attributes proxy 索引/item/iterator/getNamedItem）消费 R190 印记约定 `{ns,prefix,local}`，Attr 物化恒 bare `_zwMakeAttr` 缺省形（localName=整 qname、ns/prefix=null）。probe6 循环结构错误暴露 `attributes` accessor 真身：`_zwMEl` defineProperty 以 `_zwDescAttributes` NamedNodeMap proxy 替换裸数组——写 DOM 字段名 namespaceURI/localName 被读面静默忽略。
- 丢失点②：`_wcRebuildAsHandle`（handle 域重建）旧 `createElement(tag)` 全 HTML ns＋attr 全 `setAttribute(qname)`（null ns）——svg/math 元素与带前缀属性双双丢 ns。
- null-ns 形（HTML 属性＋表外 `xmlns:foo`）零动作即正确：解析器保持 null ns＋整 qname local＝bare 缺省形恰好 setAttribute 语义＝spec 形（adjust-foreign-attributes 只枚举 xlink:*/xml:*/xmlns/xmlns:xlink），不印记。
- setHTML 默认配置剥 `xlink:href`（probe3 W3 实证）——钉测④ 只钉元素 ns。

## 电池判读（v1，发射器 `.acceptance/dps/s10k13-launch.sh`，md5 断言 `a8d2de2f8f28489d93c7b652f2ff4c8d` 先行）

二进制 md5 `a8d2de2f8f28489d93c7b652f2ff4c8d`（工作树刀⑬ 面 release 构建）；窗 19:56:50Z–20:27:28Z；pre 锚＝k12 v2 五腿（`/tmp/k13-*-preF.txt` F 集提取源经 cmp 逐字节确认）；判读器 `.acceptance/dps/s10k13-judge.sh`（LC_ALL=C awk 提取＋comm 前双侧对账＋在飞腿 EXIT 守卫）。

| 腿 | pre (k12 v2) | post (k13 v1) | GONE | P2F | 终态 | 锚字节恒等 |
|---|---|---|---|---|---|---|
| WC | 18385P/1118F | 18166P/1118F | 0 | 0 | F 集逐键恒等；P−219＝截断落位族 | — |
| domparsing | 1313P/222F | 1313P/222F | 0 | 0 | SET-IDENTICAL | **逐字节恒等** `d8a8b37a` |
| **sanitizer** | **623P/133F** | **686P/70F** | **63** | **0** | **F2P=63 精确/P2F=0/NET +63** | — |
| TT | 1548P/542F | 1548P/542F | 0 | 0 | SET-IDENTICAL | **逐字节恒等** `cfbbfcd0` |
| dom | 57414P/496F | 57547P/496F | 0 | 0 | F 集逐键恒等；P+133＝截断落位族 | — |

**join GONE=63 / P2F=0 / 新见 F=0** —— 零附带损伤。发射席 inline 判读（账本 `.acceptance/dps/s10k13-coord-k13.md`）与本席独立复跑（`s10k13-judge.sh`）五腿终态逐项一致，双席收敛；pre-P 列取自 k12 v2 锚日志重测（`/tmp/s10k12-*-post-knife12-v2.log`）。

### SAN GONE 构成（63）

- `sanitizer-basic-filtering.html` **46F→0F 整页清零**＝主目标 D/E parser foreign namespace 簇（namespaces/0-9 ×44 跨 Element.setHTML/ShadowRoot.setHTML/setHTMLUnsafe/parseHTMLUnsafe sink＋同根 2）
- `sanitizer-javascript-url.html` 10（22→12 残：allowed/1 data: 保留＋animating 表示形＋mathml 元素 ns＋navigating 已通）
- `sethtml-safety.html` 4（13→9 残）
- `sanitizer-names.html` 3（5→2 残，Namespaced attributes # 形）

### 残 70F 分布（挂账后续刀位）

parseHTML 22（文档级骨架消毒族）、javascript-url 12（animating 行为裁决待对）、sethtml-safety 9、custom-elements-is.tentative 8、sethtml-with-declarative-shadow-root 7、sethtml-with-custom-elements 3、tree-construction 2、names 2、inert-document 2、boolean-defaults 2、html5lib-basics 1。

## 验证链

- probe7（印记版）全绿：W1 handle 三形态三元组（`xlink:href|XLINK|xlink|href`、`xml:space|XML|xml|space`、`xmlns:foo|ns=null|l=xmlns:foo`）＋W3 消毒链 svg/math ns＋W7 mock 域索引＋getNamedItem 双读面 NS 流入＋W5 handle 直读回。
- 钉测 `test_s10k13_foreign_ns_knife13_pins` 1 passed（终版复确认）；刀⑫ `test_s10k12_writechain_knife12_pins` 1 passed 零回归；fmt `--all --check` 零 diff；clippy zero-engine/zero-dom 0 warning。
- 临时探针 probe_tmp_k13.rs＋include 行已删（commit 前纪律）。
- 判读字节绑定：判读二进制 md5 `a8d2de2f8f28489d93c7b652f2ff4c8d`；提交字节＝发射时工作树刀⑬ 面（blob 恒等发射域）。
