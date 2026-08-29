# R357 — getElementsByClassName 的 sel/handle 分支 live 集合（17 已知 Fail 集合 -1）

**日期**: 2026-08-29
**切片**: M4 轻量修复（live HTMLCollection 语义；R318/R333 同域第三次收口尝试——首次成功）
**改动面**: `part04.js`（getElementsByClassName 两分支 liveSpec）+ `part24.rs`（+1 单测）

## 1. 根因

WPT Element-getElementsByClassName "should be a live collection"（17 已知 Fail 集合成员）：
静态容器建集合 `[b.foo]` 后 `appendChild(c.foo)` 期望 length 2、`removeChild` 后回 1，
旧恒 1。两个分支都缺 liveSpec：

- **sel 分支**（静态元素）：host 快照查询后裸 `return _zwMakeCollection(..., true)`；
- **handle 分支**（createElement 容器——**WPT 用例本体形态**，探针实证 `a` 是 handle
  proxy）：`_handleQueryAll` 后同样裸返回。

R318 首版给 sel 分支加过 liveSpec 但被回退（matches 只判类名 + 候选源是**全局**
`_zwPendingAdded` → 其它容器同名类元素错并进集合，single-activation 61F）；R333 引入
**作用域桶**（`_zwPendBucket(scopeSel)`）后候选源可精确归因，但只接了
getElementsByTagName/children，getElementsByClassName 漏接。

## 2. 修复

两分支补 liveSpec（matches = 类名全含判定，`_hClassesOf` 反射读 + `_zwSplitClassList`
ASCII 分词；候选源 = R333 作用域桶；mutation 期并入经 R333 门 `mutSel === scope 容器`）：

- sel 分支：`{ matches, scopeSel: sel }`——**同时修 `if (all)` 早退结构**（快照空时
  liveSpec 丢失，handle 子全 pending 的主消费形态首版因此 len0=0）；
- handle 分支：`{ matches, scopeHandle: handle }`（探针 dbg 定位 `mutSel=null`——
  createElement 容器的 mutation 以 handle 记账，scopeHandle 才能命中 R333 门）。

## 3. 过程教训（三次尝试才收敛）

1. **容器域误判**：首版只修 sel 分支——单测通过但 WPT 仍 Fail。dbg 探针
   （`_zwHCLiveInvalidate` 注入 `mutSel/addFlat/live` 计数，跑完即删）实证 WPT 用例的
   `a` 是 **handle proxy**（`mutSel=null`），走 handle 分支。R171「探针要打到被测层
   正下方」教训的再次验证。
2. **`if (all)` 早退吞 liveSpec**：快照空（内容全在 pending 桶）时早退到无 liveSpec
   的裸集合——构建期并入（R50 机制）依赖 liveSpec 存在，主消费形态恰是空快照。
3. **children liveSpec 重引入被 R2930 哨兵拦截（回退）**：同轮顺带给 children 两分支
   也加了 scoped liveSpec（ParentNode-children 1F 单测过），但 tab `R2930
   surroundContents` 哨兵当场抓回（快照换代后新读 `#sc.children` 把 pending 桶的
   stale 条目并进集合，3≠1）——**桶条目缺「快照换代失效」语义**，须先补桶代际清理
   才可重开（记档）。按最小改动原则整组回退 children 部分，仅保留
   getElementsByClassName。

## 4. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，333 文件） | **55482P/18F/15T——真实 Fail 集合 17→16（目标件退出），零新增零回归，Pass +1 净正** |
| 目标件 | Element-getElementsByClassName 2P/1F→**3P/0F** |
| R318 回归哨兵 | single-activation **132P/0F**（R318 时代 61F 的同套件）保持 |
| R333 回归哨兵 | tab R2929+R2930、renderer js_worker 全绿（152P+1P） |
| 文件级门 | QSA 1975 / matches 669 / appendData 384 / MO-attributes 42 / getElementsByTagName 19 / Element-children 2 全持平 |
| ParentNode-children | 0P/1F 维持（children liveSpec 重引入被哨兵拦截后回退，随「桶代际清理」前置项） |
| engine 单测 | v8 2483（+1 `test_getelementsbyclassname_sel_live_collection_r357`：handle+sel 双容器形态 append/跨容器不误并/类名过滤/remove 四断言）/ quickjs 1471 全绿 |
| integration | 781P（Vue/lit/WC e2e 含）全绿 |
| make test 范围项 | browser X11 环境项（clean HEAD 同败预存）外全绿 |
| clippy / fmt | 双矩阵 `-D warnings` 零警告 / 无 diff |

## 5. 后续

- **children liveSpec 重引入前置**：pending 桶条目补「快照换代失效」语义
  （`set_dom_snapshot`/navigation 换代时清桶）——落成后可收 ParentNode-children 1F；
- d3d-r3（element/fragment 本树化重启）维持可领取状态。
