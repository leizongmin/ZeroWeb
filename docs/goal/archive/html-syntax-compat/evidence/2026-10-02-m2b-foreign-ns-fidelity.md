# M2 树构造簇（a）— sel 代理层 foreign content 命名空间保真

**日期**: 2026-10-02 | **对应缺口**: P2 解析树一致性（树构造边缘簇第一片）

## 根因（单点，四 getter）

zero-dom 解析树完全正确（探针实证：SVG `<font>` = SVG ns + tag `font`、`<math>` =
MathML ns）——丢失在 shim 的 **sel-based 元素代理身份 getter**（part04）：

1. `namespaceURI` 硬编码 XHTML（R185 perf 让路只探 clone 路径）；
2. `tagName`/`nodeName` 经 `_realTag` 无条件 ASCII 大写；
3. `localName` 对 SVG 大小写修正表产物（linearGradient 等）错误小写化。

连锁：`_zwFilterByTagNameNS`（R120 的 DOM 语义匹配核心——HTML ns ASCII 折叠 /
foreign ns 精确比较）逐元素读 `el.namespaceURI` 判臂 → 全部误判 HTML ns →
foreign 元素被折叠匹配（`getElementsByTagName("altGlyph")` 命中 SVG `altglyph`
2 个、`"FONT"` 命中 SVG `font`、math 的 ns/nodeName 断言错）。

## 修复

- part03 新增 `_zwSelNs(sel, handle)`：host `__zw_get_ns` 权威值（R185 修正的
  NS_MEMO epoch memo 已使重复读零重算）+ shim 侧 gen 印章 memo（`_zwTagCache`
  同款——gET 过滤器逐元素读 ns 的 R-baidu3 风暴路径预防）；HTML ns 归一 ''。
- part04 四身份 getter（tagName/localName/nodeName/namespaceURI）接 foreign 臂：
  non-HTML ns 返回 host 原值 local（qualifiedName 原样，非大写）+ 真实 ns。
- goals/40 fetch 脚本补拉 `html/syntax/parsing/resources`（html5lib 三案的
  common.js/template.js/test.js——首轮 script fetch failed 记账后补）。

## 结果（全通道复跑逐案对账，分母恒 113 案）

| 案 | 修复前 | 修复后 |
|---|---|---|
| Document.getElementsByTagName-foreign-01 | 1/37 | **37/37** |
| Document.getElementsByTagName-foreign-02 | 0/2 | **2/2** |
| Element.getElementsByTagName-foreign-01/02 | 0/2 ×2 | **2/2 ×2** |
| math-parse01 / math-parse03 | 6/9 · 13/19 | 8/9 · 15/19 |
| **全通道** | 46122/61304 = 75.23% | **46168/61304 = 75.31%** |

回归案数：**0**。

## 记账（本片未触及的相邻面）

- html5lib_write/url/write_single 三案：资源补拉后可加载，但 30s 案预算内跑不完
  （Timeout）——html5lib 全量语料 × document.write 面需要更长案预算或切片跑法，随
  M2 续评估。
- cdata-in-integration-point-fragment（9）/ zero NUL-in-charref（6）/ quotes-in-meta
  + meta-inhead（2）：tokenizer/fragment 模式边缘，独立片。
- the-end（4）/ DOMContentLoaded-defer（1）：页面生命周期事件时序（defer 脚本
  DCL/load 序），脚本执行管线面。

## 门禁

- `make test` 68 suites 全绿（首跑 1 案 SW 时序失败为与全通道并跑抖动，串行复跑
  68/68 销案）；定向 bench-gate（zero-engine 26 指标）**GATE PASS**。
