# M4 切片 R89 — traversal 分散失败：惰性 currentNode setter + previousNode dig-on-accept

**日期**: 2026-08-17
**Commit**: `2d58d788`
**上游**: R88 的 CONTINUE 方向（traversal 分散 6F 清扫）

## 背景

traversal 剩 13F 中分散 6F（非 cross-realm）。逐个 probe 定位后 2 个轻量可修（TreeWalker.html Recursive filters + previousNodeLastChildReject），其余归因深结构记档。

## 根因与修复

### ① TreeWalker.currentNode setter eager materialize（TreeWalker.html "Recursive filters need to throw"）

- **根因**：setter 调 `materialize()`（执行 filter 全树）——filter 副作用窗口被提前消耗（WPT 的 depth 计数 filter 在 setter 期间就跑完），且 filter 内 `walker.firstChild()` 重入抛出的 InvalidStateError 从 setter 泄漏（WPT 期望 setter 不抛、首个遍历方法抛）。
- **修复**：setter 改**纯赋值**（spec 语义）——只记 `currentNodeVal`/`relocated`/`syncOrderPosTo`，idx 延迟到下次遍历方法物化后经 relocated 分支按需定位。filter 的 active flag 重入防护在首个遍历方法窗口内自然生效。
- **中间方案否决记录**：先试「setter materialize 包 active + 吞异常」——异常不泄漏了，但 filter 副作用窗口仍被消耗（depth==0 只触发一次），首个 `parentNode()` 不抛 → 仍 fail。根因是 eager 物化本身，纯赋值才是正解。

### ② previousNode ACCEPT 分支不 dig（previousNodeLastChildReject）

- **根因**：`previousNode` 的 sibling 循环对 ACCEPT 立即返回节点。但 TreeWalker 的 previousNode = **filtered 序前驱**（WebKit/Blink 语义）：cur=B2、sibling=B1（ACCEPT 但有子 C1、C2-rejected）——期望 C1（B1 子树内最深可见），非 B1。
- **修复**：ACCEPT 且 `node.lastChild` 存在 → `sibling = node.lastChild` 续 dig；childless 才返回。**交叉验证**：traversal-reject（B2 childless → 直返 B2，然后 B1-REJECT 跳过 → A1）两模型同果——回归零。

## 延后（深结构 4F 记档）

- TreeWalker-currentNode 2F：currentNode 在 root 外时遍历方法应从 currentNode 的**文档序位置**继续（非 root order 快照起点）——「currentNode 位置优先于 root 快照」需 order 模型本体重构。
- walking-outside-a-tree 1F：root 移除/regraft 后 root 身份跟随（Acid3 006a）。
- NodeIterator-removal 1F：paras[5].firstChild CDATA 边角。
- cross-realm 7F：iframe 深结构（html-compat 域）。

## 结果

| 目录 | 前 | 后 | 回归 |
|------|----|----|------|
| dom/traversal | 1591P/13F | **1593P/11F（+2 净）** | per-case 零 |
| dom/nodes | — | 同 | per-case 一致（flake 1 条波动） |
| dom/events / collections | — | 同 | per-case 逐字节一致 |
| dom/ranges | — | 未跑全量 | 既有 >420s 慢用例 M1 L2 归因在案；变更面限于 walker 工厂（range 不消费） |

- engine v8 **2187**（+2 单测：setter 纯赋值三断言 / dig-on-accept 链）
- engine quickjs **1427** 全绿；fmt 无 diff；clippy 双矩阵零警告；pre-commit-guard PASS
