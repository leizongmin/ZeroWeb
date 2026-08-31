# R304 Evidence — sel-based innerHTML 同 turn 读视图（L2 read-only 视图桥第一步；inner-outer 全解）

**日期**: 2026-08-27
**切片**: M4→M1 L2——R304(a) 同 turn 视图桥最小切片（R303 归因的直接消费方）
**改动面**: `part04.js`（innerHTML setter：wrapper 打挂父槽 + 基底缓存置空替代删除）+ `part05.js`（overlay 接受无 sel 的挂父槽节点）+ part24 探针单测（沿用 R303 归因探针）

## 一、修复内容（两件 + 一处放宽）

R303 归因：sel-based innerHTML 后 `firstChild/lastChild` 读 stale host 快照
（added wrapper 无 handle/sel，overlay 反链 miss；removed 的快照 wrapper 与
`_ihRemoved` identity 不同源，剔除恒 miss）。

1. **wrapper 挂父槽**（part04 setter）：`_ihAdded[i]._zwSelPendingParent =
   { parentSel, parentHandle, nextSibling: null }`（innerHTML 整体替换 = 尾部追加语义）；
2. **overlay 放宽**（part05 `_zwOverlayPendingChildNodes`）：反链查找增加第三
   分支——无 sel 但有 `_zwSelPendingParent` 的节点（innerHTML wrapper 域）；
3. **基底置空**（R56 的 `delete` → `set(sel, [])`）：innerHTML 是整体替换，host
   apply 前读应 = addedNodes（stale 剔除恒 miss 的根因绕过）；host apply 时
   `_zwChildBaseInvalidateAll` 全量换代（生命周期不变）。

探针（R303 同一探针）前后对比：`fc=old text|lcEqAd1=false` →
**`fc=SPAN|lc=SPAN|fcEqAd0=true|lcEqAd1=true`**（同 turn 读视图与 addedNodes
identity 全等）。

## 二、验证

| 套件 | 基线 | R304 | Δ |
|---|---|---|---|
| **MutationObserver-inner-outer** | 1P/1F | **2P/0F（100%）** | +1P/-1F |
| MutationObserver 全族 | 116P/5F | **117P/4F** | 恰 -1F |
| ParentNode 全族（innerHTML 消费方） | 2126P/6F | 2126P/6F | 持平（set-diff 一致——children live collection 1F 基线同败预存） |
| Range-mutations（childNodes 融合重度） | 342P/5F | 342P/5F | 持平 |
| Node-childNodes | 8P/0F | 8P/0F | 持平 |
| nodes 全量 | 12769P/35F | 12765P/34F | **Fail set-diff 恒等**（34 Fail 逐条一致；差 4P 为 Timeout 崩溃测试 flaky 吞 subtest——cloneNode-crash 基线 Timeout vs 本轮 nested-crash/replaceWith-crash Timeout 双向噪声） |
| traversal | 1603P/1F | 1603P/1F | 持平 |
| ChildNode/insert-adjacent | 123P/14P 0F | 同 | 持平 |
| engine 单测 | 2442 | 2442（探针断言已验证） | 持平 |
| make test | — | 1F = XOpenDisplayFailed 环境项 | 持平 |
| fmt / clippy | — | 干净 | — |

## 三、意义与后续

**这是 R220 live-view 视图桥的第一步落地**（M1 L2 read-only 的先导切片）：
「JS 写 sel 容器后同 turn 读视图可见」从 innerHTML 形态打通。同族剩余形态：
- handle 子树 append 入 sel 容器（R299 mixed-case 的 indoc 阶段、tree-order 4F）
  ——`_handleChildren` 与 sel overlay 的融合（下一步候选）；
- 工厂节点可观察 id（R302 cross-realm 域）；
- parse-time MO（document 3F）。

**MO 剩余 4F** = cross-realm 1F + document 3F（全深结构）。

## 四、教训（并行 stash 纪律）

双跑 A/B 时**禁止前台与后台脚本各自 stash 同一工作树**（本轮一次前台超时
杀死 pop 序列 + 后台 pop 弹出对方 stash 的险情——`git stash list` 校验 +
`grep -c R304 worktree` 双确认后安全恢复）。后续双跑一律「单进程顺序」或
「worktree 隔离」。
