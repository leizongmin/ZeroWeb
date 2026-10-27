# Selection 兼容二期 — 语料扩面

> **二期立项（2026-10-27，coverage 对账驱动）**：一期（[归档文档](archive/selection-compat.md)）
> 收口于子集语料；coverage 对账显示 selection 面 101 文件中仍有 81 个未覆盖，
> 缺口 31,128 子测试。本 goal 承接语料扩面 + 扩面后失败簇归因。
> **一期零回归底线继续有效**。

**版本**: v2.0
**日期**: 2026-10-27（二期立项）
**状态**: Active——M1 未启动
**执行模式**: WPT 驱动（上游 selection corpus 为验收标尺）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——编辑/选区面）

## coverage 对账（coverage-2026-10-27 基线，47 套件干跑口径，终稿待 `docs/compat/wptfyi/coverage-2026-10-27.md` 复核）

| WPT 面 | 全量分母（子测试） | 覆盖 |
|---|---:|---:|
| selection | 33,845 | 20/101 文件 |

## Mission

selection 测试面从 20 文件子集扩到全量 101 文件，摸清未覆盖 81 文件的失败
分布（selection/Range 与 editing 命令交织的用例占比）；扩面后失败按
「selection API 语义 / editing 依赖 / runner 基建」归因，editing 依赖簇
归口 33 号 editing-compat，不双边重复立项。

## Done Criteria

- **DC-1（语料扩面）**：23 号脚本 DIRS 扩至 selection 全量（deep 模式续拉）。
- **DC-2（扩面基线）**：全量分类通过率落 `docs/goal/selection-compat/evidence/`；
  与一期基线逐 subtest 对照零回归。
- **DC-3（归因）**：失败簇归因表落 evidence（selection/editing/runner 分列，
  editing 簇标注归口 33 号）。
- **DC-4（coverage 复测）**：selection 未覆盖文件清零路径成立。

## 里程碑

- **M1**：语料扩面 + 扩面基线 + 归因表（一轮可完成）。
- **M2**：selection API 语义簇收敛（如量级小可并入 M1）；editing 依赖簇
  转登记至 33 号。

## 依赖约束（run-rules §9）

- selection 语义与 33 号 editing-compat、34 号 dom-expansion（dom/ranges）
  共享编辑域底座，收敛轮核对三域基线互不回归。
