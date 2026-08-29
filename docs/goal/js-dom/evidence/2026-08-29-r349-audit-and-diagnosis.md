# R349 — register 路径审计 + events 剩余聚类归因（诊断轮）

**日期**: 2026-08-29
**命令**: `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 2400 -- ./target/release/zero-wpt-runner testharness-dom`（全量 dom sweep）+ 单文件复跑

## 1. register_dom_callbacks 全调用点审计（R348 bug 类排查）

R348 发现的「fresh Arc 重注册吞共享队列」bug 类，全仓调用点审计结果：

| 调用点 | mutations Arc | 判定 |
|---|---|---|
| webview.rs:2101（execute_script_with_dom） | `shared_mutations.clone()` | ✓ 共享 |
| webview.rs:2190（dispatch_event） | `shared_mutations.clone()` | ✓ 共享 |
| webview.rs:2720（execute 路径） | `shared_mutations.clone()` | ✓ 共享 |
| user_actions.rs:493（execute_dom_script 进程内） | ~~fresh Arc~~ → R348 已修（末尾重绑回共享 + live doc 刷新） | ✓ 已闭合 |
| apps/browser/src/tab_js_worker.rs:286 | worker 自有队列（多进程架构，消费方同体） | ✓ 语义一致 |
| apps/renderer/src/js_worker.rs:420 | 同上 | ✓ 语义一致 |

**结论**：无同类残留。

## 2. Event-dispatch-click.html pending:1 — pre-existing 确认

- R342 基线（/tmp/dom_full.txt）与当前 sweep 逐字节同态：32 Pass / 0 Fail / 文件 Timeout（pending:1 of 33）。
- 非本轮动画基建回归。缺失 subtest 属 33 个注册中的 1 个 pending（testharness 33 = 25 async_test + 4 test + 4 loop-generated async）。
- 全部可枚举候选名（正则提取 + loop 展开）均已在 recorded 集中——缺失者为难以静态枚举的注册形态，留待专项（低优先级：单 subtest，非簇）。

## 3. Event-dispatch-target-moved.html — 文件级 crash 归因

- 失败形态：`page script threw: apply mutations: insert_adjacent_sel_element: no child match for #target`（declared tests: 1 → 整文件 Fail）。
- 用例语义：事件派发过程中 listener 把 `#target` 从 `#parent` 移动到 `#table-row`（`parent.removeChild(target)` + `table_row.appendChild(...)`）——spec 要求派发路径按派发起点的快照继续。
- 断点：移动后某条 sel 锚定的 insert 类 mutation 以 `#target` 为参照，但 `#target` 的父子关系已变 → `insert_nodes_at_position` 锚点失配 → apply 硬错中止整批。
- **定性**：sel 锚定的移动语义（R334 sel 子移动 wire 的锚点在节点移动后失效）+ 派发中 mutation 的交互——非轻量可达，随 L2 主线（身份/锚点统一）处理。记档。

## 4. 全量状态

54179P/54F/16T（含本轮 r349-probe 清理噪声一条 Fail；真实态同 R348：54180P/53F/15T）。Fail/Timeout 集与 R348 差异仅为本轮探针文件与已知 crash 文件轮转。
