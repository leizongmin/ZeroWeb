# M4 — Node.contains / compareDocumentPosition 全节点形态实现（R79）

**日期**: 2026-08-16
**Commit**: `3214d883`
**前置**: R78（quickjs execute 全局代码语义 `d4eb0641`）
**证据**: [evidence/2026-08-16-r79-contains-comparedocpos.json](../evidence/2026-08-16-r79-contains-comparedocpos.json)

## 背景

nodes 子目录两大 fail 簇（两引擎共有）：Node-compareDocumentPosition 1444F + Node-contains 1002F，合计 2446F ≈ nodes fail 总数的 55%。R78 CONTINUE 方向锁定。

## 根因

1. **pending 节点不可见**：旧实现走 host sel 快照查询（`__zw_contains` / `_ancestorChain` + `__zw_element_children`），而 WPT `setupRangeTests` 的 testNodes（paras/foreignPara/detachedPara 族）全是 `createElement`+`appendChild` 建的 **pending 节点**（mutation 异步 apply，快照不含）→ DISCONNECTED|IMPL(33) / contains false。
2. **非元素节点零方法**：text/comment/document/doctype/fragment 节点没有 `contains`/`compareDocumentPosition`/`hasChildNodes`（"reference.contains is not a function"）。
3. **oracle 一致性缺陷**（修复过程中逐层暴露）：WPT oracle 用 `previousSibling`/`hasChildNodes`/`childNodes` 遍历推导期望值——shim 的 handle 元素 sibling 恒 null、hasChildNodes 与 firstChild 视图矛盾、html.parentNode 不指向 document，都会使期望值与文档序矛盾。

## 实现（JS 侧统一，host 零改动）

核心（part03，挂 globalThis 供各 part 消费）：

- `_zwNodeContains(ref, other)`——沿 other.parentNode 链 identity 上行（与 spec/oracle 同构；R51b 后 parentNode 对全节点形态正确）。
- `_zwCompareDocumentPosition(ref, other)`——同 root：CONTAINS|PRECEDING / CONTAINED_BY|FOLLOWING / LCA childNodes 序；跨树：DISCONNECTED|IMPL + 方向位（35/37，root-sort key 反对称——WPT `assert_in_array([35,37])` + anticommutative 断言）。

接线面：

- element proxy get trap（part04 contains/compareDocumentPosition 改调共享实现）
- `_wrapNodeEntry` 文本/注释（part05）+ `_zwRegisterTextEl` 文本节点（part06）
- document proxy（part06）+ detached 工厂族（part03：_zwMEl/_zwMText/_zwMComment/PI/CDATA/doctype/fragment）

基础设施修复（oracle 一致性前提）：

- `html.parentNode` → document（spec）；`document.doctype` + `createDocument(ns,qn,doctype)` 附接 + `createHTMLDocument` 预置 doctype；detached doc 树链接（head/body→documentElement→doc）。
- handle 元素 `previousSibling`/`nextSibling` 经父 childNodes 融合视图派生（旧恒 null）。
- `hasChildNodes` 与 firstChild/lastChild 融合视图对齐（旧 `false` 而 `firstChild` 非 null）。
- detached 工厂节点补 `_zwMDefineSiblings`。

## 结果

| 用例 | 前 | 后 |
|------|-----|-----|
| Node-contains | 480P/1002F | **1482P/0F（100%）** |
| Node-compareDocumentPosition | 144P/1300F | **1444P/0F（100%）** |

- **四路一致**：v8 = quickjs = v8-native = quickjs-native（ZW_NATIVE_DOM=1 双引擎）。
- nodes 目录 3115P/4396F → **5568P/1943F**（+2453，对照**重建的** clean-HEAD 基线二进制——工作树 stash 后必须重编译，旧二进制含修复会假等）。
- traversal 953P→1188P（+235 顺带）；collections/events 零回归。

## 验证

- 单测 +3（part18 R79 族）；2 处旧断言按 spec 纠正更新（html.parentNode===document / detached body.parentNode=documentElement）。
- engine v8 2161 / quickjs 1424 全绿；wpt-runner 171/106；integration 765P/2F（html_compat 既存）；make test 除 zero-compositor dmabuf（clean HEAD 同败，GPU 域）全绿；fmt/clippy 干净；pre-commit-guard PASS。

## 过程教训

1. **stash A/B 必须重建二进制**——JS shim 嵌入二进制，旧二进制含修复时「基线」假等（本轮第一轮对照 5568P 假一致，重编译后真实差 +2453）。
2. **oracle 期望值与实现必须同源**：WPT oracle 用页面导航面（sibling/hasChildNodes/childNodes）推导期望，导航面不一致会使正确实现也 fail——修导航面即修一致性。
3. spec 正确 ≠ 用例过：`html.parentNode=document` 是规范要求，但用例期望还依赖 sibling/child 视图的**全套**一致性，逐层 probe 暴露（25F→17F→8F→0F 三轮）。
