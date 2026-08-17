# M3 切片 R91 — shadow 树 isConnected + whenDefined 断言闭环

**日期**: 2026-08-17
**Commit**: `6fa0a6df`（rebase over 并行流）
**上游**: R90 已知限制清单的两项闭环

## 根因与修复

### ① shadow 树内 isConnected false（WPT Node-isConnected-shadow-dom 2F）

- **根因 A**：handle 容器（shadow root / fragment / handle 宿主）appendChild 只记 `_handleChildren` registry，不记 `_zwNodeParent` 反链——isConnected 的反链上行到容器即断。
- **根因 B**：反链即使到达容器 handle，也没有 shadow→host 的边界穿越。
- **修复**：
  - `_recordHandleChild`（part05）同步记反链 `{ parentSel: null, parentHandle }`；`_unrecordHandleChild` 对称清（parentHandle 匹配才删）。
  - isConnected 反链循环（part04）：到达容器 handle 查 `_shadowHandleMeta` → hostSel 直返 true / hostHandle 续链 / handle host 回落 rect。
- **spec 依据**：dom.spec.whatwg.org/#connected——shadow-including root 是 document 即 connected（host 在文档内 → 整棵 shadow 树 connected，open/closed 同判）。

### ② whenDefined 断言闭环（WC 资产断言组 4）

- 页面挂 `__futureResolved` 探针 + define 触发 pending resolve；断言经**第二次** `execute_script_with_dom` 读回（微任务 checkpoint 已跑）。

## 关键发现

- `_makeProxy` 的 target 是 `_fvTarget`（form validity 预置空对象）——isConnected 分支内不能经 `target.parentNode` 取父。容器侧记账（append 时写反链）是唯一可靠路径。

## 结果

| 项 | 前 | 后 |
|----|----|----|
| Node-isConnected-shadow-dom | 2F | **2P（100%）** |
| dom/nodes | 1336F | 净 -2（name-validation flake 波动外 per-case 一致） |
| dom/traversal | 1593P/11F | per-case 不变 |
| dom/events / collections | — | per-case 逐字节一致 |
| integration | 772 | 772（whenDefined 强化） |
| engine v8 / quickjs | 2187 / 1427 | 同 |

fmt 无 diff；clippy 双矩阵零警告；pre-commit-guard PASS。
