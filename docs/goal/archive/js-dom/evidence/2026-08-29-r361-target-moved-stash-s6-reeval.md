# R361 — 批内 detach→insert 移动语义（detached-stash；14→13）+ M2/S6 前置重估

**日期**: 2026-08-29
**切片**: M4 轻量修复（Event-dispatch-target-moved 转绿）+ M2 前置评估（零源码结论）
**改动面**: `js_dom_bridge.rs`（apply 批内 detached-stash）+ `dom_bridge_tests.rs`（+1 单测）

## 1. M2（S6 去字符串）前置重估（零源码，评估结论）

ZW_NATIVE_DOM=1 实测探针（testharness-dom-native）：

| 面 | 实测 |
|----|------|
| native 工厂面（`__zw_native_query_selector`） | function——native 绑定已安装可达 |
| 页面 `document.createElement` | **shim 函数**（`function(tag) { tag = String(tag); …`）——页面可见 DOM 仍 polyfill 所有（R9 结论维持） |
| MutationObserver / fetch 面 | shim 提供、工作正常 |

**结论**：S6 原定「shim 高层 API 改调 native node 方法」在 default-on 前不可达——页面
DOM 仍是 polyfill 表示，改造 shim 内部指向 native 对象 = 在**将被 M5/M7 删除的路径**上
建一次性桥（M5 切片 2 明确删除 polyfill 桥死代码）。S6 的架构目标（单一权威表示）正是
default-on 本身的交付物；**S6 记「被 default-on 取代（superseded-by-default-on）」，
随 M5/M7 用户点名后一并达成**，不再独立切片。M2 里程碑以此评估收口。

## 2. 已知 Fail 巡检续 + Event-dispatch-target-moved 修复（14→13）

巡检定域：remove-and-adopt-thcrash（window.open popup 通道=环境基建不追）、
Node-isConnected iframe 语义（R360 转档专项）、MO parse-time 3F（深结构维持）——余下
target-moved 为本轮轻量件。

**根因**（R349 归因的落地修复）：同一 dispatch 内 listener 执行
`parent.removeChild(target); table_row.appendChild(target)` 产生两条 wire——①
`Remove{'#target'}` 使 target 脱离文档；② `InsertAdjacentSelElement{child_selector:
'#target'}` 的 `find_by_selector` 在 target **已 detach** 时失配 → 硬错中止整批（
`apply mutations: insert_adjacent_sel_element: no child match for #target` 文件级 crash）。

**修复**：apply 批内 **detached-stash**——`DomMutation::Remove` 应用时记账
`(selector → NodeId)`；`InsertAdjacentSelElement` 的 child 解析失配时查 stash 复用该
NodeId（`insert_nodes_at_position` 自带 reparent = 移动语义）。spec：removeChild +
appendChild 引用**同一节点对象**，insert 复用即移动。stash 生命周期 = 一批 mutation
（与 ephemeral handles 同级），跨批 stale 引用不复活。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，333 文件） | **55484P/15F/16T——真实 Fail 集合 14→13（target-moved 退出），零新增零回归**（Timeout +1 轮转） |
| 目标件 | Event-dispatch-target-moved 文件级 crash→**1P/0F** |
| 变异路径消费方 | Node-removeChild 28P / Node-appendChild 11P / Node-insertBefore 40P 全持平 |
| host 级单测 | +1 `test_apply_detached_stash_move_semantics_r361`（dom_bridge_tests：Remove→stash→insert 复用→target 落 #other 子树全链断言） |
| engine 单测 | v8 2487 / quickjs 1472 全绿 |
| tab/renderer/integration | tab 38P、renderer 152P、integration 781P 全绿 |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

## 4. 后续

- 已知 Fail 集合余 13：全部深结构/基建域（realm 族 5、MO parse-time 3、Node-isConnected
  iframe 专项、sel/pseudo/replacement 3）——无轻量可达面，逐项需专项立项。
- 主线剩余：M5/M7 default-on（待用户点名）；M3 已达成（R100/R339）；M4 基线持续维护。
