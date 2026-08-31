# M4 切片 R87 — NodeIterator removal 恢复段二次周期 retarget + previousNode before-flip

**日期**: 2026-08-17
**Commit**: `237a6d7e`（rebase over `873c509e`）
**上游**: R86（detached 子树保留其子 + 移除 retarget）的 CONTINUE 方向

## 背景

R86 把 NodeIterator-removal.html 从整文件崩溃推到 10P/15F。上轮 session 在 429 限额中断时留下未提交的 R87 WIP（恢复段 insertBefore 清标记 / detached doc insertBefore / registry 保尾 / previousNode before 半边 / 主文档 removeChild-insertBefore doctype）。本轮接手：**重建二进制 A/B 对照发现 WIP 含 1 个真回归（NodeIterator.html 766P→758P——主文档 firstChild/lastChild getter 被 R87 注释块误删）**，修复后继续把 removal 簇推到收口。

## 七重根因与修复

| # | 根因 | 修复 |
|---|------|------|
| ① | 恢复段 `oldParent.insertBefore(node, oldSibling)` 不清移除标记——只 appendChild 清 | insertBefore 对插入子树统一 `_zwUnmarkRemovedHandle`（part04） |
| ② | detached doc / body 代理 / 主 document 缺 insertBefore；主文档缺 removeChild（doctype 子测试 TypeError 崩） | 三处补齐（part03 detached-doc/body；part06 主文档 removeChild/insertBefore + `_docDtorRemoved` 本地标记使 childNodes 视图剔除 doctype） |
| ③ | **R87 首版 guard 误用 child 查注册表**——`_zwRegisterTextEl` 的键是父 el proxy（`_makeProxy(sel,handle)`），child 恒 miss → 文本子 remove 静默 no-op | guard 改查父（`_r87Self`），注销/物化同传父 |
| ④ | 元素 remove/restore 周期后 `_zwUnregisterTextSubtree` 已注销注册文本——二次 remove 时子视图来自物化缓存，③ 的 guard 又 miss | guard 放宽：注册命中 ∨ 物化缓存含该子（`_zwDetachedChildrenOf`）都走通知路径；新增 `_zwDetachChildFromCache` 剔除缓存中的 removed（父视图 spec 正确） |
| ⑤ | previousNode 的 pointer-before=false 半边直接返 ref 不过 filter | 返 ref 前过 `check()`（REJECT/SKIP 则继续前驱）——"Recursive filters need to throw" 对 previousNode 的断言恢复 |
| ⑥ | registry 容量压实 `reg.length = 0` 全清——大用例中途在档迭代器被清 → retarget 静默丢失 | 65536 上限 + 保最近 1024（`splice` 保尾） |
| ⑦ | retarget 后继未约束 root 子树；主文档 firstChild/lastChild getter 被误删（A/B 捕获的真回归） | `inRootOf` 判定（后继跨 root 边界 → pred 分支）；getter 恢复（`_docDtorRemoved` 时 firstChild=html） |

## probe 方法论（本轮新增沉淀）

1. **A/B 必须重建二进制**（R79 教训第三次实证）：stash → checkout → rebuild → 跑基线 → pop → rebuild → 跑对照。本轮 A/B 抓到 WIP 自带的 8F 真回归，不重建就 land 了。
2. **跨子测试状态必须复现完整前序周期**：standalone probe 全绿 ≠ 全量 suite 绿——removal.html 的失败依赖前序 remove/restore 周期累积的树破坏。probe 用「元素周期 → 文本周期」两段复现才命中 guard miss。
3. **testharness assert message 是最强定位信号**：`expected "__n1" but got "[object Object]"` 直接给出期望/实际的对象形态（handle proxy vs 本地 plain text node），比计数差更快锁定分支。

## 结果

| 目录 | 前 | 后 | 回归 |
|------|----|----|------|
| dom/traversal | 1575P/29F | **1589P/15F（+14 净）** | per-case 零（comm 对照） |
| NodeIterator-removal.html | 10P/15F | **23P/1F** | — |
| dom/nodes | 6560P | 6560P | per-case 逐字节一致 |
| dom/events | 190P/138F/3TO | 同 | per-case 逐字节一致 |
| dom/collections | 48P/1TO | 同 | per-case 逐字节一致 |

- engine v8 **2183**（+2 R87 单测：二次周期 retarget / previousNode before-flip 四断言）
- engine quickjs **1427** 全绿
- fmt 无 diff；clippy v8+quickjs 双矩阵零警告；pre-commit-guard PASS

## 剩余（dom/traversal 15F）

- cross-realm 7F（TreeWalker-acceptNode-filter-cross-realm 5F + TreeWalker-realm 2F——iframe 深结构）
- removal-during-filtering 2F（filter 执行中移除 in-flight 节点——通知时机的 spec 细节）
- 分散 5F（currentNode 2F / TreeWalker 1F / walking-outside-a-tree 1F / previousNodeLastChildReject 1F）
- removal.html 残 1F（paras[5].firstChild CDATA 边角）
