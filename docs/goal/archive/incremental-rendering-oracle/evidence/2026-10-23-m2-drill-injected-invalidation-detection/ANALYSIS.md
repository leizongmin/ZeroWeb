# M2 收口：人工注入失效演练——对照器检出能力实证

**运行身份**：基线 HEAD `1839c5d58`（本轮 M1 布局切片提交后）。**演练性质：临时注入 → 观测 → 还原，注入不入库**（`git status` 还原后 clean，oracle 复跑全绿）。
**目的**：兑现 M2 验收项"人工注入失效错误能被检测"（goal 契约 M2 行）——对照器不能只对已知开放缺口（反例 A、撤用臂边界）报分歧，还必须对**回归型注入**（已修复缺口被撤回）逐项报警。

## 注入与检出

对 `crates/style-system/src/lib.rs` 同时注入三处失效（一行级，带 `DRILL-*` 标记），跑 oracle：

| 注入 | 撤回的修复 | 检出测试 | 断言消息要点 |
|---|---|---|---|
| DRILL-C：链走容器压栈条件改恒假 | 反例 C（2026-10-23 修复） | `inc01_container_context_preserved_on_chain_recompute` | 步骤「p1 inline margin 变更触发链走（@container 应按 .card 求值，p1 保持 green）」双路径分歧，首差 `p#p1.text` |
| DRILL-B：链走 `attach_pseudo_elements` 调用短路 | 反例 B（2026-10-23 修复） | `inc01_ancestor_pseudo_preserved_on_chain_recompute` | 步骤「p1 inline style 变更触发祖先链重算（body::before 应保留）」双路径分歧，首差 `body` 伪样式字段 |
| DRILL-COUNTER：回退分支计数改 no-op | DC-2 计数（本轮 goal 内新增） | `inc02_has_selector_fallback_matches_full` | `left: 0, right: 1`——计数器断言精确捕获"回退发生但未计数" |

**无误报**：其余 8 序列（与注入点无关的路径）全部照常通过——检出是定位的，不是"全线飘红"式的假灵敏。

结果：`8 passed; 3 failed`，3 失败与 3 注入一一对应。还原后备 `lib.rs` 复跑：`11/11 passed`。

## 布局面检出能力说明

layout 侧（`layout_incremental_oracle`）的检出能力由撤用臂 Divergent 测试实证（几何级边界，含全量基线自检排除非确定性）；冷温 Equal 序列无"已修复缺口"可撤回（该切片为纯基线建立，无历史修复），故不做注入演练——若未来温路径检出真实分歧即为其价值时刻（护栏性质，见 m1-layout 证据）。

## 边界

- 演练只覆盖 style 层 oracle 的三处已修复缺口；反例 A（开放缺口）的检出由其 Divergent 断言常态守护。
- 注入为撤回修复（一行级短路），未测试"全新失效模式"的检出——那是对照器语义覆盖问题，由序列集扩张持续补强，不属于本次验收项。
