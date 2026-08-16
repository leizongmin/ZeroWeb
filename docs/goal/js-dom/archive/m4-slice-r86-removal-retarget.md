# M4 切片 R86 — detached 子树保留其子 + NodeIterator 移除 retarget（WPT 驱动）

**日期**: 2026-08-17
**基线**: R85 后（traversal 1565P/15F；removal.html 整文件崩溃）
**Evidence**: [../evidence/2026-08-17-r86-removal-retarget.json](../evidence/2026-08-17-r86-removal-retarget.json)

## 驱动用例簇

- `NodeIterator-removal.html`（整文件崩溃 → 10P/15F 执行——setup 在 remove paras[0] 后读 `paras[0].firstChild` 直接 null.parentNode 崩）
- `NodeIterator-removal-during-filtering.html`（2F——filtering 中途移除 retarget）

## 四重根因

1. **removeChild 注销注册文本视图后 detached 元素无子可读**：spec 要求 detached 子树保留其子；R51c 的防泄漏注销与可读性冲突。
2. **迭代器 order 快照对移除不感知**：spec 迭代集合 live——移除节点（含子树）退出集合。
3. **referenceNode 移除 retarget 缺失**：spec nodeiterator-remove 的双指针分支（后置→前驱/前置→后继；root inclusive-ancestor no-op）。
4. **通知点不全**：`_zwMEl.removeChild`（createElement 产物）与 detached-doc 的 doc/body/_tree 三族 removeChild 都在移除路径上但不通知。

## 修复

- **_zwMaterializeDetachedChildren**：注销注册文本**前**把融合子视图快照入 `_zwDetachedChildren`（handle 键，512 软上限）；childNodes/firstChild/lastChild 在注册表 miss 后回落——可读性与防泄漏兼得。
- **_zwMarkRemovedHandle / _zwIsRemovedNode**：sel+handle 双移除标记 + 沿父链上行的子树判定；appendChild 成功清标记（re-append 移动语义）。
- **order 扫描跳过移除节点**：nextNode 前向整子树跳（orderEnd 区间）、previousNode 逆向逐节点跳。
- **_zwIterRegistry + _zwNotifyIteratorsRemove**：pred/succ 树序计算（previousSibling 最深右端/父；nextSibling/爬升）+ inSubtree（ref 在 removed 子树内）+ isAnc（**removed 是 root 的 inclusive ancestor——沿 root 父链找 removed**，首轮方向写反过）+ 双指针分支 retarget。
- **五个 removeChild 形态全部接通知**（proxy handle-child / proxy remove / _zwMEl / detached-doc doc / body _tree），通知一律**先于** splice/parentNode 清理（pred/succ 读移除前链）。

## 过程发现

- isAnc 方向写反（沿 removed 找 root vs 沿 root 找 removed）——spec「toBeRemoved is an inclusive ancestor of **root**」是 removed 在 root 的链上。
- `removed === root` 是 no-op（inclusive ancestor 含自身）——单测首版误期望 retarget。
- sel-based 子节点（parsed 元素）的 removeChild 在 part04 proxy 只处理 handle 子——sel 子移除走 host `__zw_remove`（既有路径），单测须用 createElement 产物驱动。

## 验证

| 项 | 结果 |
|----|------|
| dom/traversal | 1565P → **1575P（+10 净）**；removal.html 0 崩溃 → 10P/15F（执行暴露） |
| dom/nodes / events / collections | 6650P / 189P / 48P-0F 零回归 |
| 单测 | part18 +2（detached 子保留 + retarget 到父 / 移除跳过 + re-append 恢复 6→5→6）；engine v8 **2181** / quickjs **1427** 全绿 |
| fmt / clippy | 无 diff / workspace 默认 + quickjs 矩阵零警告 |

## 剩余（traversal 29F）

- NodeIterator-removal 15F：root=document 跨多 k 推进的树序细节（M1 L2 live-iterator 邻域）
- removal-during-filtering 2F + cross-realm 7F + currentNode 2F + 其余 3F（R85 清单）

## 教训

1. **崩溃文件是零信号**——修崩溃让用例执行本身就是基线真实化（同 R8）；P 数与 F 数会同时涨，判定标准是「可执行子测试的 pass 是否增长」。
2. **防泄漏与 spec 可读性冲突时**：物化快照两头兼得（注销注册表 + 缓存视图），不是二选一。
3. **通知必须在树状态变化前**——pred/succ 计算读移除前的兄弟/父链；放在 splice 之后是静默错值。
