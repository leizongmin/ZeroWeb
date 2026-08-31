# M4 切片 R88 — NodeIterator filter 执行中移除的 in-flight retarget

**日期**: 2026-08-17
**Commit**: `aa368299`
**上游**: R87（removal 恢复段收口）的 CONTINUE 方向

## 背景

dom/traversal 剩余 15F 中 NodeIterator-removal-during-filtering 2F 是最后一块非 cross-realm、非分散的可收敛簇。spec `concept-nodeiterator-traverse`：filter 执行中移除节点时，pre-remove 步须对 **in-flight 遍历位置**（正在被 filter 的候选）生效——返回值仍是被 filter 节点，referenceNode 已 retarget 到存活节点。

## 五重根因与实现

| # | 根因 | 修复 |
|---|------|------|
| ① | pre-remove 只作用于 referenceNode，in-flight 候选无登记 | registry entry 增 `getInFlight`/`getInFlightBefore`；`check()` 执行 filter 前后登记/恢复 `inFlightVal`（方向态：nextNode→before=false / previousNode→before=true 由遍历方法入口设置） |
| ② | 遍历 wrapper 在 filter 返回后无条件 `refNodeVal = 返回值`——覆盖 retarget 已落的存活节点 | wrapper 读 `flightRetargeted` 抑制回写（返回值=被 filter 节点，reference=存活节点） |
| ③ | 抑制回写对「already-visited」场景（filter 内移除无关已访问节点）是错的 | `retargetInFlight` vs `retarget` 双 fire 路径：仅 in-flight 命中（`subj`）走前者置标记；常规 reference 命中走后者，filter 返回后 ref 正常落新 accepted |
| ④ | previousNode 的 before=false 早返返回 `refNodeVal`——filter 内 retarget 改掉后返回值成了 retarget 目标 | ref 快照先于 `check()`，返回快照 |
| ⑤ | 清理时序：check finally 先清 `flightRetargeted` → wrapper 读到已清零值（首版实测复现） | 清理移到 wrapper 读之后（`flightRetargeted = false` 在 wrapper 尾部） |

## probe 方法论

- 「ref 被谁覆盖」类 bug：notify hook 打印 registry entry 状态即可分离「retarget 没执行」vs「执行后被覆盖」——本轮一次 probe 直接定位 ②。
- 抑制类机制按「成因路径」分流（retarget 因 in-flight 而起才抑制）——一刀切抑制在同文件第 2 个用例立即暴露（already-visited 回归），A/B per-case 对照捕获。

## 结果

| 目录 | 前 | 后 | 回归 |
|------|----|----|------|
| dom/traversal | 1589P/15F | **1591P/13F（+2 净）** | per-case 零 |
| NodeIterator-removal-during-filtering | 2P/2F | **4P/0F（100%）** | — |
| dom/nodes | 6560P | 同 | per-case 一致（name-validation flake 1 条波动） |
| dom/events / dom/collections | — | 同 | per-case 逐字节一致 |

- engine v8 **2185**（+2 单测：filter 移除候选 retarget / previousNode filter 内移除祖先）
- engine quickjs **1427** 全绿
- fmt 无 diff；clippy 双矩阵零警告；pre-commit-guard PASS

## 剩余（dom/traversal 13F）

- cross-realm 7F（iframe 深结构，TreeWalker-realm 族）
- 分散 6F（currentNode 2F / TreeWalker 1F / walking-outside-a-tree 1F / previousNodeLastChildReject 1F / NodeIterator-removal CDATA 边角 1F）
