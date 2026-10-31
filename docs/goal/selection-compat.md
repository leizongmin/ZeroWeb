# Selection 兼容二期 — 语料扩面

> **二期立项（2026-10-27，coverage 对账驱动）**：一期（[归档文档](archive/selection-compat.md)）
> 收口于子集语料；coverage 对账显示 selection 面 101 文件中仍有 81 个未覆盖，
> 缺口 31,128 子测试。本 goal 承接语料扩面 + 扩面后失败簇归因。
> **一期零回归底线继续有效**。

**版本**: v2.1
**日期**: 2026-10-27（二期立项）；2026-10-31（无人值守收口定稿——33 号交接方式
与 M2 阈值落定，见 Mission / 里程碑与「待用户决策」）
**状态**: Active——M1 未启动（控制面 `selection-compat/master.md` 已建）
**执行模式**: WPT 驱动（上游 selection corpus 为验收标尺）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——编辑/选区面）

## coverage 对账

**现行账（coverage-2026-10-31 快照**，`docs/compat/wptfyi/coverage-2026-10-31.md`，
aligned union 口径）：

| WPT 面 | 全量分母（子测试） | 覆盖 | 未覆盖 |
|---|---:|---:|---:|
| selection | 34,421 | 23/194 文件 | 31,638 |

立项时干跑口径（coverage-2026-10-27，47 套件）：33,845 子测试 / 101 文件 /
20 覆盖。两口径分母差为 pin 滚动与 union 口径并集，不互换；DC-1 的
「selection 全量」以现行账 194 文件为准。

## Mission

selection 测试面从子集扩到全量（现行账 194 文件，干跑口径 101 文件），摸清
未覆盖文件的失败分布（selection/Range 与 editing 命令交织的用例占比）；扩面
后失败按「selection API 语义 / editing 依赖 / runner 基建」归因。editing
依赖簇**不在本 goal 修**：落本 goal 挂账清单标注归口 33 号 editing-compat，
由用户转登记（2026-10-31 用户决策——33 号在 ZeroWeb-3 工作区并行在跑，rally
不直接写对方控制面，避免跨工作区提交撞车），不双边重复立项。

## Done Criteria

- **DC-1（语料扩面）**：23 号脚本 DIRS 扩至 selection 全量（deep 模式续拉）。
- **DC-2（扩面基线）**：全量分类通过率落 `docs/goal/selection-compat/evidence/`；
  与一期基线逐 subtest 对照零回归（一期锚：
  `archive/selection-compat/evidence/2026-10-14-m1-baseline.md`）。
- **DC-3（归因）**：失败簇归因表落 evidence（selection/editing/runner 分列，
  editing 簇落本 goal 挂账清单标注归口 33 号，登记 master.md 待转登记表）。
- **DC-4（coverage 复测）**：selection 未覆盖文件清零路径成立。

## 里程碑

- **M1**：语料扩面 + 扩面基线 + 归因表（一轮可完成）。
- **M2**：selection API 语义簇收敛。**并入 M1 的判据（2026-10-31 定稿）**：
  失败簇 ≤300 子测试且不引入新引擎概念时就地收口（零回归门照 DC-2）；超限
  簇归簇挂账出立项建议（登记 master.md 待用户决策表）。editing 依赖簇不
  收敛，落挂账清单待用户转登记 33 号。

## Support Envelope

**范围内**：selection 语料全量拉取（deep 模式续拉）、扩面基线与失败簇归因、
M2 判据内的语义簇收敛、coverage 复测。
**排除**：editing 依赖簇修复（归 33 号，本 goal 只挂账）；runner 基建通道
缺陷（挂账登记）；>300 子测试或需新引擎概念的语义簇（出立项建议）。
**资源约束（无人值守）**：批跑限 lane 并发 + case 墙钟预算（照 37 号先例
12 lane / 600s 起步，按面实测调整）；长跑任务走既有 OOM/墙钟守卫
（`docs/engineering/quality-gates.md`），禁止裸跑；CI 发射前查在跑实例。

### 依赖约束（run-rules §9）

- selection 语义与 33 号 editing-compat、34 号 dom-expansion（dom/ranges）
  共享编辑域底座，收敛轮核对三域基线互不回归。33 号 rally 在 ZeroWeb-3
  工作区并行推进——本 goal 不写对方控制面，跨界事项挂账登记待用户转登记。
- 一期零回归底线继续有效（逐 subtest 对照，锚
  `archive/selection-compat/evidence/2026-10-14-m1-baseline.md`）。

## Final Output Protocol

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方 DONE 允许条件 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **默认输出** |
| 真正外部阻塞（依赖不可用、平台不支持） | `BLOCK: <原因>` | 罕见使用 |

**DONE 允许条件**（同时满足）：DC-1~4 全部满足有据（测试命令、报告路径、
数字证据）；验证基于上游真实 WPT 用例（无内建 inline 充数）；`cargo build`
+ `cargo test` + `cargo clippy` 全过；master.md 内部自洽、里程碑归档完成；
一期基线零回归保持；挂账/待转登记事项已落 master.md 待用户决策表（等审批
不是 BLOCK）。

## Document Control

- 每轮进展写 `selection-compat/master.md` 与 `evidence/`；入口文档仅在目标
  实质变化时修改；里程碑完成归档至 `selection-compat/archive/`（只追加）。
- 收口归档：入口 + 控制面整树迁 `docs/goal/archive/`，归档 README 登记行。
- 「待用户决策 / 待转登记」表事项不 BLOCK 推进——转零碰撞面自主推进，
  交付时集中提出。
