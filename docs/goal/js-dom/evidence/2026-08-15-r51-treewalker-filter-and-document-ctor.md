# R51 Evidence — TreeWalker NodeFilter 语义 + Document 构造器解锁 dom/common.js mega-case

**日期**: 2026-08-15
**轮次**: R51（js-dom M4 / DC-3 traversal/ranges/nodes 大释放）
**测试命令**: `make testharness-dom FILTER=dom/<subdir> [TIME_LIMIT=…]`（test-guard 包裹）

## 切片一：TreeWalker/NodeIterator NodeFilter callback 语义（spec `callbackdef-nodefilter`）

1. **filter 以 callback 对象形态保存**（不一次性解绑）：函数直接调用；对象每次 traverse 经
   `Get(filter, "acceptNode")`（getter 副作用/抛错传播、this=filter 对象、每次 Get）；
   acceptNode 缺失/非 callable → 调用时 TypeError。
2. **filter 抛错原样重抛**（旧实现 `catch → return ACCEPT` 吞错）+ currentNode 不动 +
   物化失败可重试（accepted/walked 清零）。
3. **nextNode lazy 步进**：结构序（pre-order 全节点数组 + 子树 exclusive-end）物化只读
   childNodes（构造零异常）；步进时才对候选调 filter（每 traverse 恰一次 Get）；REJECT 跳
   子树区间；SKIP 继续。fresh（未步进）首候选 = root 自身（iteration order 含 root）。
   previousNode 以 currentNode 实际位置续接。
4. **NodeIterator 专有属性**：referenceNode + pointerBeforeReferenceNode（getter，随步进更新）。

WPT：TreeWalker-acceptNode-filter **12/12 全过**（旧 5P/7F）。

## 切片二：`new Document()` 构造器 + detached doc 工厂族 → 解锁 dom/common.js

**根因**：WPT dom/* mega-case（NodeIterator.html、Range-isPointInRange.html、
Range-comparePoint.html 等）共享 `dom/common.js` 的 `setupRangeTests()`，其中
`new Document()` / `createCDATASection` / `xmlDoc.createProcessingInstruction` /
`foreignDoc.createDocumentFragment` / `ownerDoc.createRange` / `xmlDoc.appendChild`
此前全部缺失/undefined → setup 中途崩 → `testNodes` undefined → **整个用例零 subtest 或
顶层 ReferenceError**——dom/nodes、dom/ranges、dom/traversal 的大量用例从未真正跑过。

修复（detached doc = `_makeDetachedDocument`，R2815 基础设施）：

- `globalThis.Document` 构造器（返 `_makeDetachedDocument('')`，prototype→Node.prototype）
- createCDATASection（nodeType 4）/ createProcessingInstruction（nodeType 7，spec 命名校验）/
  createComment（nodeType 8）/ createDocumentFragment（nodeType 11，本地可变容器）/
  createRange（`_makeRange`）/ 文档级 appendChild/removeChild/childNodes
- 产物 ownerDocument 语义：detached 工厂产物 + `_wrapNodeEntry` parsed 文本/注释节点 +
  `_zwRegisterTextEl` 本地文本节点（`rangeFromEndpoints` 的
  `ownerDocument(node).createRange()` 链）

## 切片三：WPT long-timeout 支持（runner）

- `<meta name=timeout content=long>` → CASE_TIMEOUT 10s→60s（WPT 上游标准 normal/long）。
- Makefile `testharness-dom TIME_LIMIT=` 透传（mega-case 子目录墙钟放宽）。
- 已知边界：`run_page_scripts_strict` 同步执行期无超时介入（mutations 族 4 用例单跑 >120s
  纯算非死循环），本轮不处理，记 follow-up。

## 结果（vs R50 基线）

| 子目录 | R50 | R51 | Δ |
|--------|-----|-----|---|
| dom/traversal（polyfill） | 36P/19F | **925P/655F** | **+889** |
| dom/traversal（native） | 8P | **893P/687F** | **+885**（用例解锁同源） |
| dom/nodes（polyfill） | 2508P | **2957P** | **+449** |
| dom/ranges（非 mutations 43 用例） | 39P | **1847P** | **+1808** |
| dom/collections | 48P/0F | 48P/0F | 持平 |
| dom/events | 189P | 189P | 持平 |

用例级：TreeWalker-acceptNode-filter 5P→**12P（100%）**；NodeIterator-removal-during-filtering
部分解锁；Range-isPointInRange 24P→36P（CDATA-append 融合为剩余主因）。

- engine v8 单测 **2132 全绿**（R2803/R3257 遗留断言随 lazy 语义更新）；quickjs **1415 全绿**；
  双矩阵 clippy 零警告；fmt 无 diff。

## 遗留（下轮候选）

1. **CDATA/普通对象节点 append 进 handle 容器**：`paras[5].appendChild(cdataSection)`
   （普通对象）被 part04 appendChild trap 丢弃（要求 `__zwHandle`）——detached 普通对象树与
   handle 树融合 = M1 L2 身份桥邻域。isPointInRange 剩 5697F、ranges 32454F 大部源于此族。
2. Range-mutations 族 4 用例同步执行 >120s（性能/断言密度，非死循环）——
   `run_page_scripts_strict` 无执行期超时介入。
3. traversal 655F 剩余：cross-realm 5F（iframe realm 测试设施）、removal-during-filtering 4F、
   NodeIterator-removal.html 等。
4. TreeWalker-currentNode 2F（root 外节点续走语义）。
