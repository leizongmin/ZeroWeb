# M4 切片 R84 — traversal 兄弟链断链 + walker 语义四重修复（WPT 驱动）

**日期**: 2026-08-17
**基线**: R83 后（traversal 1527P/53F）
**Evidence**: [../evidence/2026-08-17-r84-traversal-sibling-chain.json](../evidence/2026-08-17-r84-traversal-sibling-chain.json)

## 驱动用例簇

- `dom/traversal/NodeIterator.html`（19F → 0F）：document/foreignDoc/testDetached root 的超步族
- `dom/traversal/TreeWalker.html`（18F → 8F）：filter false 剪枝 + firstChild/nextSibling 语义
- 其余 7 文件 16F → 16F 中 4F 修复（detach/重入/previousSibling 族）

## 五重根因

1. **sibling 对 text 节点独立包装**：`__zw_sibling_nodes` 的 pair 经 `_wrapNodeEntry(pair, null)` —— parentNode=null + 兄弟静态 null。WPT oracle `nextNode()` 树序遍历走 firstChild/nextSibling/parentNode 链，在首个 text 处断链（而迭代器走 childNodes 全树）→ 两侧序列分歧。
2. **CDATA（handle-less）append 不接链**：入 `_handleChildren` 但无 parentNode/兄弟 getter（common.js `paras[5].appendChild(createCDATASection)`）。
3. **detached doc headEl/docEl 无兄弟 getter**：foreignDoc/xmlDoc root 遍历到 HEAD 即停（nextNodeDescendants climb 无 nextSibling）。
4. **NodeIterator 缺 detach() + 重入守卫**：spec 历史 no-op 方法 + active flag InvalidStateError。
5. **TreeWalker filter 语义三处**：返 false(0) 应按 REJECT 剪枝（旧 dig 子树）；root 不应被 filter（iteration collection 含 root）；currentNode setter 重定位被滤节点后 effPos 误返 0（fresh 分支）。

## 关键修复

- **identity 统一**（核心）：sibling 对改经 `_childNodeList(parentSel)` 取 —— 与 `head.childNodes[i]` 同 identity（`_zwChildBaseCache` 缓存保证），`indexOf` 定位命中。probe 实证 `kids[1]===tn` 之前 false（独立包装）是断链根源。
- **两接口 REJECT 分叉**：TreeWalker 非 1/3 返回值归一 REJECT（剪枝）；NodeIterator 保持不剪（spec：迭代集合结构性，REJECT/SKIP 等价）——首轮统一归一导致 NodeIterator -12P 回归后纠正。
- **walkRoot()**：物化从 root 子节点起步，root 不入 filter 流（层级方法语义）；order-scan（nextNode）仍 check root（迭代器 fresh 返 root 语义保持）。
- **active flag + _guarded**：七个遍历方法（nextNode/previousNode/parentNode/firstChild/lastChild/nextSibling/previousSibling）finally 复位。
- **附带 unblock**：并行流 a8d5a22d 的 v8+quickjs **组合态**（workspace 单 cargo 调用 feature 并集）五处编译断（worker.rs 死分支/es_module config 双 move/quickjs_worker ambiguous re-export/webview 重复 Drop/method/tab_js_worker config 双 move）——统一 not(v8) 门控 + clone 修复。CI 矩阵单 feature 跑不暴露，`make test`（workspace 并集）必踩。

## 验证

| 项 | 结果 |
|----|------|
| dom/traversal | 1527P/53F → **1556P/24F（+29 净）**；native 1531P/49F |
| dom/nodes | 6635P → **6643P（+8 净，sibling getter 顺带）** |
| dom/events / collections | 189P / 48P-0F 零回归 |
| 单测 | part18 +3（R84 族：sibling 链/CDATA+detach+重入/filter 剪枝+root+重定位）；engine v8 **2176** / quickjs **1427** 全绿 |
| fmt / clippy | 无 diff / workspace 默认 + quickjs 矩阵 + webview/browser/script-sandbox 零警告 |
| make test | workspace 并集态编译修复后可跑；既存环境失败 = compositor dmabuf（无 GPU）+ zero-browser 3（并行流重构域，clean HEAD 同败） |

## 剩余（traversal 24F）

- TreeWalker.html 8F：document.lastChild 边角 oracle 分歧（真浏览器 document 子列表行为差异，待深查）
- cross-realm/realm 7F（跨 realm 深结构）
- currentNode 2F + walking-outside-a-tree 1F（root 外重定位 lazy-resume，M1 L2 域）
- removal-during-filtering/removal 3F（live 集合移除跟踪）
- traversal-reject 2F / previousNodeLastChildReject 1F

## 教训

1. **oracle 与实现的遍历原语分歧是断链根因**：WPT dom oracle 用 firstChild/nextSibling/parentNode 树序导航，实现用 childNodes 数组递归——两条路径对 sibling/parent 链的完整性要求不同，identity 不统一时正确实现也 fail。
2. **TreeWalker 与 NodeIterator 的 REJECT 语义天然分叉**（剪枝 vs 不剪）——同一 walker 工厂须按接口类型归一 filter 返回值。
3. **CI 单 feature 矩阵掩盖组合态编译断**：v8+quickjs 并集（workspace `cargo test` 的实际形态）从未被 CI 覆盖——cfg 双分支代码须在并集态验证。
