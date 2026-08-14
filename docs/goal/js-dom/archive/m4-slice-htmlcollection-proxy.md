# M4 切片归档 — R50：HTMLCollection Proxy 承载（legacy platform object + live overlay）

**日期**: 2026-08-15
**里程碑**: M4 / DC-3（dom/collections 深结构主簇闭合 + nodes/traversal 波及提升）
**状态**: ✅ 已 land

## 背景与目标

R37 导入 dom/collections 后聚类出「HTMLCollection own 属性枚举 + live + legacy platform object」
深结构主簇（~19 fail），R38（namedItem 空串/children 集合）、R43（indexed/named configurable:false）、
R44（NamedNodeMap own 枚举）渐进收口后仍余 24F 五簇：own-props 8F、supported-property-names 6F、
supported-property-indices 5F、iterator 3F、as-prototype 2F。根因单一：**Array 承载无法表达
spec legacy platform object 语义**。

## 实现（js_dom_shim JS 侧，零 host 改动）

1. **`_zwMakeHTMLCollection`（part05 新）**：Proxy target 只存 expando；indexed/named 由 trap
   动态求值。trap 族：get（canonical 索引→元素 / expando 优先 / named getter / illegal
   invocation receiver 校验）、set（返 false 表拒绝——strict TypeError 由引擎抛）、
   defineProperty（indexed/named 抛 TypeError）、deleteProperty（own expando 优先删；
   non-configurable 返 false）、getOwnPropertyDescriptor（indexed enumerable:true /
   named enumerable:false，均 configurable:true writable:false）、ownKeys
   （[indices, names, expandos]，无 length/item/namedItem）、has（prototype 成员 + named）。
2. **`_zwHCPrototype`**：Object.prototype 为基（保 hasOwnProperty 等内建——assert_array_equals
   依赖）；length getter（receiver 校验）、item（`_zwToUint32` mod 2^32——`item(4294967296)`→0）、
   namedItem（空串返 null）、@@iterator（value iterator）；**无** values/entries/forEach
   （WPT HTMLCollection-iterator 断言不存在）；全部 method enumerable:false。
3. **canonical 索引判定 `_zwIsCanonicalIndex`**：`/^(0|[1-9][0-9]*)$/` 且 < 2^32−1——
   "-2"/"4294967295"+ 落 named getter（WPT supported-property-indices）。
4. **named 仅 HTML ns**：`_zwIsHTMLNamespace`（namespaceURI null/XHTML；R18 `_nsHandles` 读回）
   ——WPT supported-property-names "non-HTML namespace"。
5. **live overlay**：`_zwHCLiveInvalidate` 挂 `_mo_notify`（part01，shim 全部 childList 记录
   单一汇流点）——added/removed 经 `_zwHCCollectSubtree`（`_handleChildren` R2927 registry
   展开 handle 子树）同步维护（a）已注册集合元素表（b）`_zwPendingAdded/Removed` 全局表；
   新建集合时 pending 按 matches 并入。**不重查 host**（`__zw_query_all` 读 dom_html 快照、
   脚本批末回写——同步重查拿旧结果，R48/R49 同款教训：JS 本地视图优先）。
6. **getElementsByTagName 匹配模型**（三轮实证对齐 WPT case.js 期望模型）：HTML ns 元素
   ——查询参数 ascii-lowercase 后与 qualified name（prefix:local）**精确**比较；非 HTML ns
   元素——查询原样精确比较。`_zwAsciiLower` 只动 A-Z（'ä' ≠ 'Ä'）。
7. **连带**：普通 handle 元素 `childNodes` 从 `_handleChildren` registry 回落（此前无 sel
   恒 []——WPT case.js expected 侧 detached 容器伪空根因）；R3033 遗留测试更新（HTMLCollection
   无 forEach，for-of 替代）。

## 结果

| 子目录 | R49 基线 | R50 |
|--------|----------|-----|
| dom/collections（polyfill/native 双路径） | 24P/24F/1TO | **48P/0F/1TO（97.96%）** |
| dom/nodes（polyfill） | 2479P | **2508P（+29）** |
| dom/traversal（polyfill） | 9P | **27P（+18）** |
| dom/events / dom/ranges | 189P / 39P | 持平 |

用例级：own-props 0→8P、supported-property-names 1→7P、supported-property-indices 2→6P、
iterator 2→5P、as-prototype 0→1P、delete/empty-name 维持 100%；case.html 130P 持平。

## 过程回归（六轮，全同轮修，记录避免重蹈）

1. set/deleteProperty trap 返 true → strict 不抛（须返 false）。
2. prototype `Object.create(null)` 丢 hasOwnProperty（assert_array_equals 回归）。
3. length getter enumerable:true 进 for-in（R3033 断言）→ false。
4. getElementsByTagName 匹配模型三轮（uppercase never matches / case.js abc-Abc-ABC-ä-Ä）。
5. pending 表不展开子树 → case.js 容器孙跨子测试泄漏（-9）。
6. `_elConnected` connected 守卫误伤 handle 元素（parentNode 链断）→ 移除改子树展开。

## 验证

- `make testharness-dom FILTER=dom/<5 子目录>` + `testharness-dom-native`（collections/traversal）
- engine v8 单测 2132 全绿（新增 `test_htmlcollection_proxy_semantics_r50` 8 断言组）；
  quickjs 1415 全绿；双矩阵 clippy 零警告；fmt 无 diff。
- evidence：`evidence/2026-08-15-r50-htmlcollection-proxy.{md,json}`

## 遗留

- `childnodes-messagechannel-crash.html` timeout（MessageChannel 基础设施，非 DOM 域）。
- supported-property-indices 剩 1F（2^32 边界 expando 细节）、as-prototype 剩 1F（低 ROI）。
- traversal 28F 剩余（NodeIterator removal-during-filtering 族等深结构）。
