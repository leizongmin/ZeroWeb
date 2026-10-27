# 编辑命令兼容 — editing/run + editing/other

**版本**: v1.0
**日期**: 2026-10-27（立项，coverage 对账驱动）
**状态**: Active——M1 语料预置未启动
**执行模式**: WPT 驱动（上游 editing corpus 为验收标尺）+ 长线实现推进
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——编辑面）

## coverage 对账（coverage-2026-10-27 基线，47 套件干跑口径，终稿待 `docs/compat/wptfyi/coverage-2026-10-27.md` 复核）

| WPT 面 | 全量分母（子测试） | 覆盖 |
|---|---:|---:|
| editing/run | 96,745 | 0/120 文件 |
| editing/other | 16,339 | 2/510 文件 |

合计 **113,084 子测试**，是 encoding 之外最大的连片零覆盖域。

## Mission

execCommand / queryCommand / queryCommandState 等编辑命令族语义面预置与收敛。
这是长线 goal：编辑命令语义面广（style/color/list/alignment/link/removeFormat
等数十命令 × HTML 片段矩阵），预期初始通过率低。策略是**先语料后引擎**：
M2 基线先把 Unsupported / Fail / Timeout 归簇摸清，再按簇立项收敛，不追求
一轮全绿。

## Done Criteria

- **DC-1（语料全量）**：33 号脚本拉全 editing/run + editing/other（deep 模式，
  include/ 共享 helper 随目录拉取）。
- **DC-2（runner 通道）**：testharness 子命令 + per-dir skip 规则 + Makefile
  target（M1 rally 轮落地）。
- **DC-3（基线分类）**：全量分类通过率落 `docs/goal/editing-compat/evidence/`；
  Unsupported 与 Fail 分列（能力缺失 ≠ 实现错误）；失败按命令族归簇；
  `docs/compat/trends/wpt-suites.csv` planned 行转数据行。
- **DC-4（收敛轮）**：按失败簇逐轮修齐，每轮零回归（逐 subtest 对照），
  evidence 落档；收敛目标与节奏按簇另行定稿。
- **DC-5（coverage 复测）**：editing 未覆盖子测试清零路径成立。

## 里程碑

- **M1**：语料预置 + runner 通道。
- **M2**：基线分类（重点：execCommand 全命令矩阵的能力现状图）。
- **M3+**：按簇收敛（每簇一轮），收口标准按簇定稿。

## 依赖约束（run-rules §9）

- 编辑命令实现与 uievents（50 号已收口，IME 挂账）、selection（23 号二期）
  共享编辑域路径——失败簇跨界时按 run-rules §9 先协调归口，不双边重复立项。
- 编辑面涉及 selection/Range 的部分与 34 号 dom-expansion（dom/ranges）有
  共享底座，收敛轮核对两域基线互不回归。
