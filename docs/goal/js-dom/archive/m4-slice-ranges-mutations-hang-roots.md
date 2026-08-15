# R51b — dom/ranges mutations 族死循环三根因修复（child→parent 反向链 / pending overlay / pre-insert 校验）

**日期**: 2026-08-15
**Commit**: `aed92314`（在并行流 `b3d8feb2` R51 之上的 follow-up）
**里程碑**: M4（WPT dom 基线扩展）+ M1 方向（反链+overlay 是 L2 polyfill-live 合一的实质推进）

## 背景

R51（`b3d8feb2`）补上 `new Document()` 构造器后，`dom/common.js` 的 `setupRangeTests()` 首次完整跑通，dom/ranges 的 mutations 族用例（Range-mutations-*.html，12 文件）真正开始执行——随即触发**无限死循环**：7 个 wpt-runner 进程各 ~97% CPU 空转近 1 小时（上一轮 session 遗留，本轮开场清理）。

## 三根因（递进暴露）

### 根因 1：child→parent 反向链缺失

WPT common.js 的 identity 循环模式：

```js
function indexOf(node) {
  if (!node.parentNode) return 0;
  var i = 0;
  while (node != node.parentNode.childNodes[i]) i++;  // ← 越界后 undefined != node 恒真
}
```

- handle 子节点（createElement 产物）的 `parentNode` 走 `_parentNodeFor(null, handle)` fallback **恒猜 body**；
- `body.childNodes` 是 host 快照（`__zw_child_nodes`），同步脚本 turn 内 mutation 还是 pending 未 apply，快照不含刚插入节点；
- 假父快照永不含该节点 → 越界恒不等 → 97% CPU 自旋。

**修复**：
- `_zwNodeParent` registry（part01 声明）：`childHandle → {parentSel, parentHandle, nextSibling}`，记账挂 `_mo_notify` childList 汇流点（shim 全部 childList mutation 单一入口，R50 同款收口）；added 记链、removed 清链。
- `_parentNodeFor`（part03）handle 分支优先查反链；无链的纯 detached 节点返 **null**（spec 正确行为；旧 fallback 猜 body 本身是 bug）。

### 根因 2：childNodes 双视图割裂

- **sel 父**：host 快照无 pending overlay → 同步 turn 内 insertBefore/appendChild 的子不可见；
- **handle 父**：part04 childNodes getter 里 `_zwLocalChildNodes`（textContent= 的本地文本视图）命中即 return，**短路**了 `_handleChildren` registry（appendChild 建的子）→ `paras[0].textContent=...` 后 `paras[0].appendChild(paras[1])`，childNodes 只见文本子。

**修复**：
- `_zwOverlayPendingChildNodes`（part05）：`_childNodeList` 结果按 `_zwPendingAdded/_zwPendingRemoved`（R50 记账，`_mo_notify` 汇流点维护）修正——removed 剔除、added 按父 sel 匹配 + `nextSibling`（R47 record 字段）定位插入，无匹配 ref 保守尾插；
- part04 childNodes/firstChild/lastChild：本地文本视图 + registry 子**融合**（text 在前、handle 子在后）。

### 根因 3：pre-insert 校验缺失 → registry 自环

`appendChildTests[30]` = `["paras[0]", "paras[0]", ...]` 即 `paras[0].appendChild(paras[0])`（非法操作用例段）：
- 旧 shim 不抛、真执行 → `_handleChildren[paras0].push(paras0)` 自环 → `_zwHCCollectSubtree` 无限递归 → RangeError 栈溢出；
- ancestor 变体（`paras[0].appendChild(testDiv)`）→ host apply mutations 报「操作会导致 DOM 树中出现循环」→ **整批 mutation 丢弃**。

**修复**：`_zwIsAncestorOf(child, targetSel, targetHandle)`（part05）——**从目标上行**（child 是目标的祖先 ⟺ 目标祖先链含 child；第一版从 child 上行是方向错误，child 在目标之上永远走不到目标），handle 链走 `_zwNodeParent`、sel 链走 `__zw_parent`，64 层防环。appendChild/insertBefore（self + ancestor）/replaceChild（ancestor）抛 `HierarchyRequestError`。

## 排查方法记录（供复用）

1. 孤儿进程现象（~97% CPU 零输出）→ identity 循环假设；
2. 探针 HTML 注入（throw Error(JSON) 把断点值带出 runner）逐步缩小：identity 断点矩阵 → 单 test 函数 → doTests 管线 → 逐 params 二分；
3. spin 与 crash 的区分：`timeout N` 下 exit=124（kill）vs exit=1（正常 Fail）；
4. 栈溢出栈帧（`_zwHCCollectSubtree` 重复）直接指向自环；
5. 「字面量数组不炸 vs 共享文件数组炸」→ 内容核对发现 `[30]` 实为 self-append 用例。

## 结果

- **mutations 族：无限 spin → 全部跑完**（Range-mutations-appendChild 2.5s 34P/36F；10/12 文件 120s 内 586P/1010F）；
- **四子目录零回归**：nodes 3049P（R50 后 2508→2957→3049，反链+overlay 让 dom/nodes 共用 common.js 用例继续受益）、events 189P、collections 48P/0F、traversal 925P 全部与 R51 记录一致或更好；
- **engine v8 2138 / quickjs 1415 全绿**（+6 R51b 单测 part18），双矩阵 clippy 干净，fmt 无 diff。

## 遗留（下轮首查）

1. **三文件超时**：Range-mutations-dataChange/insertBefore/replaceData 120s/300s 不完——次级循环或每-subtest `setupRangeTests()` 全量重建的累计成本，先 profile 归因；
2. **mutations 语义面**：append 同位节点后 `range.startContainer` 期望旧 proxy（`__n17`）实得新 proxy（`__n33`）——host 重挂后 proxy 缓存或 NodeId 重建语义，下一切片；
3. 本切片的 overlay 是**保守尾插**（nextSibling 也 pending 时无精确位），Range 深比较用例（compareBoundaryPoints/set/isPointInRange/comparePoint ~30k Fail 主力）需要 insert 位置精确化——M1 L2 主线内容。

## 碰头记录（run-rules §9）

本轮 session 与并行 rally 流（另一 claude 实例）在**同一 clone** 同时工作——并行流 08:18 提交 `b3d8feb2` 时把本 session 正在编辑的工作树一并吸收。无代码丢失（增量独立 land 为 `aed92314`），但双流同 clone 违反 run-rules §8（双独立 clone）。已记入 master.md；再次发生则暂停一边。
