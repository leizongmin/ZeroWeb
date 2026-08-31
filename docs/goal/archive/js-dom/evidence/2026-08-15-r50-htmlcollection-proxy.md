# R50 Evidence — HTMLCollection Proxy 承载（legacy platform object 语义 + live overlay）

**日期**: 2026-08-15
**轮次**: R50（js-dom M4 / DC-3 dom/collections + dom/nodes + dom/traversal 波及提升）
**测试命令**: `make testharness-dom FILTER=dom/<subdir>` / `make testharness-dom-native FILTER=dom/<subdir>`（经 test-guard 包裹）

## 切片内容

HTMLCollection 从 Array 承载升级为 **Proxy 承载**（spec legacy platform object，
https://dom.spec.whatwg.org/#interface-htmlcollection + WebIDL legacy platform objects），
闭合 R37 聚类的「HTMLCollection own 枚举 + live + legacy platform object」深结构主簇。

### 修复面（对应 WPT 失败簇）

| # | 语义 | 实现 |
|---|------|------|
| 1 | own 枚举 `[indices…, names…, expandos…]`（无 length/item/namedItem） | ownKeys trap（namespace 过滤 + canonical 数字 name 去重） |
| 2 | 无 values/entries/forEach（非 iterable 接口成员） | prototype 只挂 length/item/namedItem/@@iterator/toString（Object.prototype 为原型基） |
| 3 | indexed/named 拒绝 set/defineProperty/delete（loose no-op / strict TypeError） | set 返 false / defineProperty 抛 / deleteProperty 返 false（non-configurable expando 也正确拒绝） |
| 4 | canonical 索引边界（"-2"/"4294967295"+ 落 named；item(2^32) ToUint32→0） | `_zwIsCanonicalIndex`（0 ≤ n < 2^32−1）+ `_zwToUint32`（WebIDL mod 2^32） |
| 5 | illegal invocation（collection 作 prototype，base object 读 length 抛 TypeError） | get trap `recv !== proxy` 检查 |
| 6 | live 语义（同步脚本 appendChild 后 c[0] 可见） | `_zwHCLiveInvalidate`（挂 `_mo_notify` childList 单一汇流点）+ `_zwPendingAdded/Removed`（handle 子树展开经 `_handleChildren` registry）+ `_zwMakeCollection` 构建时 pending 并入 |
| 7 | named getter 仅 HTML ns 元素（non-HTML namespace name 不暴露） | `_zwIsHTMLNamespace`（namespaceURI 读 R18 `_nsHandles`） |
| 8 | expando 优先于 named getter（shadow later + delete 后 named 重新可见） | get trap hasOwnProperty 优先 + deleteProperty own 优先 |
| 9 | getElementsByTagName 匹配模型 | HTML ns：查询 ascii-lowercase + qualified name 精确；非 HTML ns：查询原样精确（`_zwAsciiLower` 只动 A-Z，'ä'≠'Ä'） |

### 连带修复

- `childNodes`：普通 handle 元素（createElement 容器）从 R2927 registry 读子（此前无 sel 恒 []）——
  WPT case.js expected 侧 detached 容器 childNodes 伪空根因。
- R3033 遗留测试更新：HTMLCollection 无 forEach（旧 Array 泄漏语义），for-of（@@iterator）替代。

## 结果（vs R49 基线）

| 子目录 | R49 基线 | R50 | Δ |
|--------|----------|-----|---|
| dom/collections（polyfill） | 24P/24F/1TO | **48P/0F/1TO** | **+24（全灭）** |
| dom/collections（native） | 24P/24F/1TO | **48P/0F/1TO** | **+24（对等 0pp）** |
| dom/nodes（polyfill） | 2479P | **2508P** | **+29** |
| dom/traversal（polyfill） | 9P/46F | **27P/28F** | **+18**（live overlay 让 detached 树遍历可见） |
| dom/traversal（native） | 8P/47F | 8P/47F | 持平（用例侧同源 polyfill） |
| dom/events（polyfill） | 189P | 189P | 持平 |
| dom/ranges（polyfill） | 39P | 39P | 持平 |

- 用例级：HTMLCollection-own-props 0P/8F→**8P/0F**、supported-property-names 1P/6F→**7P/0F**、
  supported-property-indices 2P/5F→**6P/1F**、iterator 2P/3F→**5P/0F**、as-prototype 0P/2F→**1P/1F**、
  delete/empty-name 维持 100%。case.html 130P 持平（getElementsByTagName 匹配模型三轮实证对齐）。
- zero-engine v8 单测 **2132 全绿**（新增 R50 单测 8 断言组 + R3033 更新）；quickjs 矩阵 **1415 全绿**；
  双矩阵 clippy 零警告；fmt 无 diff。

## 过程回归与修正（记录避免重蹈）

1. set/deleteProperty trap 初版返 true——strict 赋值/删除不抛（Proxy 语义：返 false 才触发
   strict TypeError）。HTMLCollection-delete 两个子测试回归同轮修。
2. `_hcProto` 初版 `Object.create(null)` 丢 Object.prototype 内建——`collection.hasOwnProperty`
   不存在（getElementsByClassName-32 assert_array_equals 回归）→ 改 `Object.create(Object.prototype)`。
3. length getter enumerable:true 进 for-in（R3033 断言回归）→ enumerable:false（ZeroWeb 旧行为一致，
   WPT 无 for-in length 断言）。
4. getElementsByTagName 匹配模型三轮实证（uppercase never matches → case.js abc/Abc/ABC/ä/Ä 全族）：
   最终模型 = HTML ns 查询 ascii-lower + 元素 qualified name 精确；非 HTML ns 查询原样精确。
   中间版曾致 case.html -9/-5/-3，均同轮修正恢复 130P。
5. live overlay pending 表初版不展开子树——case.js 容器孙节点跨子测试泄漏（-9）→
   `_zwHCCollectSubtree` 经 `_handleChildren` 展开 added/removed。
6. connected 守卫（`_elConnected`）误伤 handle 元素（append 后 parentNode 链断）→ 移除，
   改由子树展开 + pending removed 正确清理。
