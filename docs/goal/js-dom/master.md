# JS/DOM 原生化 — 主控面板（master.md）

**入口文档**: [../js-dom.md](../js-dom.md)（长期 Mission / Done Criteria / 执行协议 / 文档治理规则）
**关联 RFC**: [../../specs/p1b-v8-native-bindings-rfc.md](../../specs/p1b-v8-native-bindings-rfc.md)
**创建日期**: 2026-08-13（goal 拆分 bootstrap）
**当前轮次**: M0 未启动（本文件为首轮 bootstrap，状态待执行 agent 填充）

> **⚠️ 本文件是首版 bootstrap，内容为框架占位，不是已核实状态。**
> 执行 agent 在**第一轮进入**时，必须按入口文档「首轮进入检查清单」逐项核实并**重写本文件**：把所有 `待核实/待填充` 替换为真实仓库事实（带 commit hash / 测试命令 / 实测数字）。**未核实前不得据本文件下任何 `DONE`/收口判断。**
> 入口文档（js-dom.md）的基线事实块记录了 goal 拆分时（2026-08-13）的探查结论，可作为首轮核实的起点，但 master.md 的数字必须由执行 agent 当轮实测确认（并行双流下 main 随时漂移，run-rules §10）。

---

## 当前状态（执行 agent 首轮填充）

> **填充指引**：逐项核实，标 ✅/⚠️/❌，附证据（commit hash / 测试命令 / 文件路径）。与本节互斥矛盾的内容不允许出现在其他 section。

| 项 | 状态 | 证据（待填充） |
|----|------|----------------|
| P1a（event loop / fetch / Observer） | 待核实 | |
| P1b native bindings S0–S5 | 待核实 | |
| L1 Live Document 共享 | 待核实 | |
| L2 polyfill-live 合一 | 待核实 | |
| S6 高层 API 去字符串 | 待核实 | |
| S7 死代码清理 + shim 萎缩 | 待核实 | |
| default-on + 删 kill-switch | 待核实 | |
| 真实 SPA/WC 端到端验收 | 待核实 | |
| WPT dom 上游基线 | 待核实 | |
| `make test` / clippy / coverage | 待核实 | |
| dom_bindings 独立 coverage 口径 | 待核实 | |

**核心缺口**（本目标要消除，首轮核实并补全）：
1. 待填充
2. 待填充

---

## Active Milestone（执行 agent 首轮确认）

**当前活跃里程碑**: M0 — 基线建立 + polyfill-live 合一起刀（L2/S6 入口）（见入口文档「Single Active Milestone」）

**M0 首切片**: 待执行 agent 选定并填充（候选见入口文档；首轮必须选定并直接动手推进）

**本轮/本切片进度**: 未启动

---

## 测试基线（执行 agent 首轮填充）

| 基线 | 命令 | 当前值（待填充） |
|------|------|------------------|
| workspace 测试 | `make test` | |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | |
| 行覆盖率（全量） | `scripts/check-coverage.sh` | |
| dom_bindings 覆盖率（独立） | 待 M0 补口径 | |
| product-smoke | `make product-smoke` | |
| bench-gate | `make bench-gate` | |

---

## Coverage 矩阵（执行 agent 持续更新）

| crate/模块 | 行覆盖率 | 趋势 | 备注 |
|------------|----------|------|------|
| dom_bindings | 待 M0 补口径 | — | 新模块，本目标首个 coverage 工作 |

**覆盖率口径规则**：不缩范围伪造达标；新代码必带测试；持续提升、不退化。

---

## Latest Evidence（执行 agent 每轮追加）

| 日期 | 轮次 | 证据 | 结果 |
|------|------|------|------|
| 2026-08-13 | goal 拆分 bootstrap | 入口文档基线事实块 | 框架占位，待首轮核实 |

---

## 下一步计划（执行 agent 每轮更新）

1. （首轮）按入口文档「首轮进入检查清单」完成探索 + 重写本文件 + 建 archive/evidence + 补 dom_bindings coverage 口径 + 建 A/B 对照门骨架
2. （首轮）选定 M0 首切片（L2-read-only 候选）并直接动手推进
3. 后续按入口文档 Ordered Next Milestones（M1→M5）切片推进

---

## 待用户决策清单（深结构护栏）

> 遇需用户拍板的事项记此清单并跳过，继续轻量修复。**这些不阻塞推进，但 default-on 是本目标最后的收敛动作。**

| 事项 | 触发条件 | 状态 |
|------|----------|------|
| `ZW_NATIVE_DOM` default-on（改 Mission 级单向门，M5） | M1–M4 完成、native 路径生产就绪 | 待 M5 启动前征询 |

---

## 未解决问题（执行 agent 追加）

- （待首轮填充）

---

## 归档记录

> 已完成的 milestone/切片记录到 `archive/`。当前无已完成项。

- （无）
