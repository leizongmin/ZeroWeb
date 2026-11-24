# DC-2 切片 10 sanitizer 刀⑬ 判读证据——深条目属性 ns 发射（foreign namespace 写链保真）

**日期** 2026-11-25 · **判定** 刀⑬ 落地，sanitizer 净 +63（133F→70F，basic-filtering 整页清零），五腿 join 零回归

## 刀面（4 文件，+96/−24）

1. `crates/engine/src/js_dom_bridge.rs`：新 `deep_attr_entry_json` helper——qname＋`ans`/`al`/`ap` 三字段（adjust foreign attributes 命中形才发，XHTML ns 与 null ns 省略）；`deep_entry_json_unsafe`＋`deep_entry_json` 双发射器 attr 闭包统一同源（spec: [adjust foreign attributes](https://html.spec.whatwg.org/multipage/parsing.html#adjust-foreign-attributes)）。`crates/dom/src/node.rs` re-export `markup5ever::Attribute`。
2. `crates/engine/src/js_dom_shim/part03.js`：`_zwMApplyAttrNsMeta`（带 `ans` 条目灌 R190 印记 `{ns,prefix,local}`——`_zwMAttrNS` 读面消费约定；写 DOM 字段名 namespaceURI/localName 则读面不消费＝初版根因）；`_zwMBuildDeepEntry`＋`_zwMBuildDeepTreeNode` 双调用点；`_wcRebuildAsHandle` 非 HTML ns 元素 `createElementNS`（失败回落 createElement）＋带 ns 属性 `setAttributeNS`（失败回落 qname setAttribute）。
3. `crates/engine/src/js_dom_bridge_tests/part159_s10k13_foreignns_pins.rs`（新，120 行）：四组钉＋反向假恒真守卫（handle 三形态三元组/doc 域物化/isEqualNode 配对 vs createElementNS 期望树/消毒链元素 ns）。
4. `crates/engine/src/js_dom_bridge_tests.rs`：include 接入。

## 根因链（probe1-7 七轮定谳）

- 旧结论（刀⑫ 探针 R7/R8/R9）「parser 不产 foreign ns」修正：引擎发射端完好——`__zw_parse_html_subtree_unsafe` RAW JSON 对 ctxTag div/'' 均发 SVG/MathML 元素 ns＋adjust 命中属性 ans/al/ap（probe7 RAW 实证）。
- 丢失点全在 JS 消费端两处：①`_zwMApplyAttrNsMeta` 初版写 DOM 字段名（namespaceURI/localName），而读面消费的是 R190 印记约定 `{ns,prefix,local}`——字段名错配静默丢；②`_wcRebuildAsHandle` 旧 `createElement(tag)` 全 HTML ns＋attr 全 `setAttribute`。
- null-ns 形（HTML 属性＋表外 `xmlns:foo`）解析器保持 null ns＋整 qname local＝裸 `_zwMakeAttr` 缺省形恰好 spec 形，零动作正确（不印记）。
- setHTML 默认配置剥 `xlink:href`（probe3 W3 实证）——钉测④ 只钉元素 ns。

## 电池判读（v1，发射器 `.acceptance/dps/s10k13-launch.sh`，md5 断言 `a8d2de2f8f28489d93c7b652f2ff4c8d` 先行）

二进制 md5 `a8d2de2f8f28489d93c7b652f2ff4c8d`（工作树刀⑬ 面 release 构建）；窗 19:56:50Z–20:27:28Z；pre 锚＝k12 v2 五腿 F 集（`/tmp/k13-{wc,domparsing,san,tt,dom}-preF.txt`）；判读器 `.acceptance/dps/s10k13-judge.sh`（LC_ALL=C awk 提取＋comm 前双侧对账；在飞腿 EXIT 守卫）。

| 腿 | preF | postF | GONE | P2F | 终态 | 锚字节恒等 |
|---|---|---|---|---|---|---|
| WC | 1118 | 1118 | 0 | 0 | P=18166 F=1118 T=20（截断落位漂移族） | —（F 集恒等门） |
| domparsing | 222 | 222 | 0 | 0 | P=1313 F=222 T=1 | **逐字节恒等** `d8a8b37a` |
| **sanitizer** | **133** | **70** | **63** | **0** | **P=686 F=70 T=1＝净 +63** | — |
| TT | 542 | 542 | 0 | 0 | P=1548 F=542 T=64 NR=17 | **逐字节恒等** `cfbbfcd0` |
| dom | 496 | 496 | 0 | 0 | P=57547 F=496 T=12（Range-mutations 噪声族） | —（F 集恒等门） |

**join GONE=63 / P2F=0 / 新见 F=0** —— 零附带损伤。发射席 inline 判读（账本 `.acceptance/dps/s10k13-coord-k13.md`）与本席独立复跑（s10k13-judge.sh）三腿早期读数＋五腿终态逐项一致，双席收敛。

### SAN GONE 构成（63）

- `sanitizer-basic-filtering.html` **46F→0F 整页清零**＝主目标 D/E parser foreign namespace 簇（namespaces/0-9 ×44 跨 Element.setHTML/ShadowRoot.setHTML/parseHTMLUnsafe sink＋同根 2）
- `sanitizer-javascript-url.html` 10（22→12 残）
- `sethtml-safety.html` 4（13→9 残）
- `sanitizer-names.html` 3（5→2 残）

### 残 70F 分布（挂账后续刀位）

parseHTML 22（文档级骨架消毒族）、javascript-url 12、sethtml-safety 9、custom-elements-is.tentative 8、sethtml-with-declarative-shadow-root 7、sethtml-with-custom-elements 3、tree-construction 2、names 2、inert-document 2、boolean-defaults 2、html5lib-basics 1。

## 验证链

- 钉测 `test_s10k13_foreign_ns_knife13_pins` 1 passed（终版复确认）；刀⑫ `test_s10k12_writechain_knife12_pins` 1 passed 零回归；fmt `--all --check` 零 diff；clippy zero-engine/zero-dom 0 warning。
- 临时探针 probe_tmp_k13.rs＋include 行已删（commit 前纪律）。
- 判读字节绑定：判读二进制 md5 `a8d2de2f8f28489d93c7b652f2ff4c8d`；提交字节＝发射时工作树刀⑬ 面（4 文件 diff 落 commit，blob 恒等发射域）。
