# JS/DOM 原生化 — 主控面板（master.md）

**入口文档**: [../js-dom.md](../js-dom.md)（长期 Mission / Done Criteria / 执行协议 / 文档治理规则）
**关联 RFC**: [../../specs/p1b-v8-native-bindings-rfc.md](../../specs/p1b-v8-native-bindings-rfc.md)
**创建日期**: 2026-08-13（goal 拆分 bootstrap）
**当前轮次**: M0 未启动（本文件为首轮 bootstrap，状态待执行 agent 填充）

> **⚠️ 本文件是首版 bootstrap，内容为框架占位，不是已核实状态。**
> 执行 agent 在**第一轮进入**时，必须按入口文档「首轮进入检查清单」逐项核实并**重写本文件**：把所有 `待核实/待填充` 替换为真实仓库事实（带 commit hash / 测试命令 / 实测数字）。**未核实前不得据本文件下任何 `DONE`/收口判断。**
> 入口文档（js-dom.md）的基线事实块记录了 goal 拆分时（2026-08-13）的探查结论，可作为首轮核实的起点，但 master.md 的数字必须由执行 agent 当轮实测确认（并行双流下 main 随时漂移，run-rules §10）。

---

> **📥 接收登记（2026-08-13 用户决策）**：canvas-2d goal 的 `html/canvas/element/path-objects`
> 剩余工作**合并入本 goal 统一执行**（JS/DOM API 语义面）。canvas 流已完成并提交的部分
> （commit `d0874c28`：roundRect 角对半径/比例缩放/非有限守卫/16 段椭圆弧 + 新子路径）可复用。
> **待接手项**（详见 canvas-2d master.md「交接记录」）：
> 1. **roundRect 批量运行 panic**（NaN 排序，scale 归一化后复现——单用例未定位，wpt-runner
>    崩溃级）——**接手第一优先级**（定位后解阻塞剩余 roundrect 用例）
> 2. roundRect DOMPoint 断言（~26 用例：fill 扫描线与椭圆弧交点/精度）
> 3. arc 形状精度（~16）、arcTo/quadratic/bezier/isPointIn* 等 JS 侧 API 语义
> 4. roundrect 语义校验（badinput/negative/toomany 抛异常、winding/zero）
> **运行入口**：`zero-wpt-runner testharness-canvas path-objects`。
> **⚠️ 用例需重新导入**：canvas-2d master.md 交接记录称「用例已导入 205 文件」，但 2026-08-13 实测
> `tests/wpt-runner/wpt-data/html/canvas/element/path-objects/` 目录**为空**（canvas 流从
> `CANVAS_TEST_SUBDIRS` 移除该目录后用例未留在仓库）——本流接手时**须先重新导入** path-objects
> 用例并重新加入 `CANVAS_TEST_SUBDIRS`，不能假设用例已在。详见入口文档 v1.2 说明块 + DC-8 + M8。

## 当前状态（执行 agent 首轮填充）

> **填充指引**：逐项核实，标 ✅/⚠️/❌，附证据（commit hash / 测试命令 / 文件路径）。与本节互斥矛盾的内容不允许出现在其他 section。

| 项 | 状态 | 证据（待填充） |
|----|------|----------------|
| P1a（event loop / fetch / Observer） | 待核实 | |
| P1b **V8** native bindings S0–S5 | 待核实 | |
| L1 Live Document 共享（V8） | 待核实 | |
| L2 polyfill-live 合一（V8） | 待核实 | |
| S6 高层 API 去字符串（V8） | 待核实 | |
| **QuickJS 原生 DOM 绑定（DC-7, v1.1）** | 待核实（预期真空） | |
| S7 死代码清理 + shim 萎缩 | 待核实 | |
| **双引擎** default-on + 删 kill-switch | 待核实 | |
| 真实 SPA/WC 端到端验收 | 待核实 | |
| WPT dom 上游基线 | 待核实 | |
| **Canvas path-objects JS 侧 API（DC-8, v1.2 接手）** | 待核实（用例需重新导入，目录实测为空） | |
| `make test` / clippy / coverage（含 `--features quickjs` 矩阵） | 待核实 | |
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

1. （首轮）按入口文档「首轮进入检查清单」完成探索（含核实 QuickJS 页面引擎路径 = native 真空）+ 重写本文件 + 建 archive/evidence + 补 dom_bindings coverage 口径 + 建**双 feature 可参数化** A/B 对照门骨架
2. （首轮）选定 M0 首切片（V8 L2-read-only 候选）并直接动手推进
3. 后续按入口文档 Ordered Next Milestones（M1→M8）切片推进：V8 先行（M1–M5），QuickJS native 镜像（M6），双引擎 default-on + 收尾（M7）；M8 canvas path-objects 可与主线并行穿插推进

---

## 待用户决策清单（深结构护栏）

> 遇需用户拍板的事项记此清单并跳过，继续轻量修复。**这些不阻塞推进，但 default-on 是本目标最后的收敛动作。**

| 事项 | 触发条件 | 状态 |
|------|----------|------|
| V8 `ZW_NATIVE_DOM` default-on（改 Mission 级单向门，M5） | M1–M4 完成、V8 native 路径生产就绪 | 待 M5 启动前征询 |
| QuickJS `ZW_NATIVE_DOM` default-on（改 Mission 级单向门，M7） | M6 QuickJS native 移植完成 | 待 M7 启动前征询 |

---

## 未解决问题（执行 agent 追加）

- （待首轮填充）

---

## 归档记录

> 已完成的 milestone/切片记录到 `archive/`。当前无已完成项。

- （无）
