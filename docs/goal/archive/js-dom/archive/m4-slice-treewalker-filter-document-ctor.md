# M4 切片归档 — R51：TreeWalker NodeFilter 语义 + Document 构造器解锁 dom/common.js

**日期**: 2026-08-15
**里程碑**: M4 / DC-3（traversal/ranges/nodes mega-case 大释放）
**状态**: ✅ 已 land

## 切片一：NodeFilter callback 语义（TreeWalker-acceptNode-filter 5P/7F→12P/100%）

- filter 以 **callback 对象**保存（旧实现构造时一次性解绑 `filter.acceptNode`）：
  函数直接调用；对象每次 traverse 经 `Get(filter, "acceptNode")`——getter 每次执行、
  抛错原样传播、`this` = filter 对象、acceptNode 缺失/非 callable 抛 TypeError。
- filter 抛错**原样重抛**（旧 `catch → return ACCEPT` 吞错）+ currentNode 不动 +
  物化失败可重试（accepted/walked 清零，下次 traverse 重新物化）。
- **nextNode lazy 步进**：结构序（pre-order 全节点 + 子树 exclusive-end）物化只读
  childNodes（构造零异常、时序符合 spec「filter 只在遍历方法调用时执行」）；步进时
  才对候选调 filter（WPT "performs Get on every traverse"：两次 nextNode = 恰两次 Get）；
  REJECT 跳子树区间；fresh 首候选 = root（iteration order 含 root，R2803 语义）；
  previousNode 以 currentNode 实际位置续接（旧 idx 在 accepted 未物化时误导）。
- NodeIterator 专有 referenceNode + pointerBeforeReferenceNode（WPT NodeIterator.html 全程断言）。

## 切片二：`new Document()` + detached doc 工厂族（解锁 dom/common.js）

WPT dom/* mega-case 共享 `dom/common.js` 的 `setupRangeTests()`——`new Document()` /
`createCDATASection` / `xmlDoc.createProcessingInstruction` / `foreignDoc.createDocumentFragment` /
`ownerDoc.createRange` / `xmlDoc.appendChild` 缺失 → setup 中途崩 → `testNodes` undefined →
大量用例零 subtest 或顶层 ReferenceError（从未真正跑过）。

- `globalThis.Document` 构造器（`_makeDetachedDocument('')`，prototype→Node.prototype）
- detached doc：createCDATASection(4)/createProcessingInstruction(7，spec 校验)/createComment(8)/
  createDocumentFragment(11，本地可变容器)/createRange/文档级 appendChild/childNodes
- ownerDocument 链：detached 工厂产物 + `_wrapNodeEntry` parsed 文本/注释 +
  `_zwRegisterTextEl` 本地文本（`rangeFromEndpoints` 的 `ownerDocument(node).createRange()`）

## 切片三：runner WPT long-timeout + Makefile TIME_LIMIT

- `<meta name=timeout content=long>` → CASE_TIMEOUT 10s→60s（上游 normal/long 标准）。
- `make testharness-dom TIME_LIMIT=` 透传。
- 已知边界：`run_page_scripts_strict` 同步执行期 deadline 不介入（mutations 族 4 用例
  单跑 >120s 纯算非死循环）——follow-up。

## 结果

| 子目录 | R50 | R51 |
|--------|-----|-----|
| dom/traversal（polyfill） | 36P/19F | **925P/655F（+889）** |
| dom/traversal（native） | 8P | **893P（+885）** |
| dom/nodes | 2508P | **2957P（+449）** |
| dom/ranges（非 mutations 43 用例） | 39P | **1847P（+1808）** |
| dom/collections / dom/events | 48P/0F / 189P | 持平 |

engine v8 2132（R2803/R3257 断言随 lazy 语义更新）/ quickjs 1415 全绿；clippy 零警告；fmt 无 diff。

## 遗留（下轮候选）

1. CDATA/普通对象节点 append 进 handle 容器（part04 appendChild 要求 `__zwHandle`，普通对象
   静默丢弃）——detached 普通对象树 × handle 树融合 = M1 L2 身份桥邻域；ranges 剩余大部根因。
2. Range-mutations 族 4 用例同步执行 >120s（执行期超时介入缺失）。
3. traversal cross-realm 5F（iframe realm）、removal-during-filtering 4F、currentNode 2F。
