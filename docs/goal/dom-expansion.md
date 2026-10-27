# DOM 扩面 — dom/ranges + html/dom

**版本**: v1.0
**日期**: 2026-10-27（立项，coverage 对账驱动）
**状态**: Active——M1 语料预置未启动
**执行模式**: WPT 驱动（上游 corpus 为验收标尺）+ 语料扩面 + 接口完备性收敛
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——DOM 面）

## coverage 对账（coverage-2026-10-27 基线，47 套件干跑口径，终稿待 `docs/compat/wptfyi/coverage-2026-10-27.md` 复核）

| WPT 面 | 全量分母（子测试） | 覆盖 | 说明 |
|---|---:|---:|---|
| dom/ranges | 44,720 | 0/73 文件 | Range API 完备性 |
| html/dom | 72,282 | 0/394 文件 | 40 号收口时只预置 40 文件子集 |

## Mission

两面一体推进：dom/ranges 是 Range API（DOM Standard §5-7）完备性——中小域
（73 文件）但子测试密度高，预期引擎工作为主；html/dom 是 HTML 元素与
Document 的 DOM 接口面（394 文件，含各元素 historical/idlharness 面）——
40 号 html-syntax-compat 当时按 parser 语义划界只收了子集，本 goal 承接
剩余接口面。

## Done Criteria

- **DC-1（语料全量）**：34 号脚本拉全 dom/ranges + html/dom（deep + any.js；
  html/dom 的 idlharness.any.js 为 build 期生成资产上游不存在，拉取记 1 个
  失败属预期，不追）。
- **DC-2（runner 通道）**：ranges 归 testharness-dom 通道扩语料面、html/dom
  归 testharness-html-syntax 通道扩语料面（或新增子命令，M1 rally 轮定）；
  per-dir skip 规则 + Makefile target 同步。
- **DC-3（基线）**：两面分类通过率分别落 `docs/goal/dom-expansion/evidence/`；
  接口缺失（Unsupported）与语义错误（Fail）分列；
  `docs/compat/trends/wpt-suites.csv` planned 行转数据行。
- **DC-4（coverage 复测）**：两面未覆盖子测试清零路径成立；对 40 号已收口
  面零回归（html/dom 扩面不得动 parser 语义通道既有结果）。

## 里程碑

- **M1**：语料预置 + 通道扩面 + Makefile。
- **M2**：基线分类（ranges：已实现 API 的语义正确率；html/dom：接口存在性图）。
- **M3**：可收敛簇修齐或挂账 + coverage 复测收口。

## 依赖约束（run-rules §9）

- Range 语义与 editing（33 号）共享 selection/Range 底座，收敛轮核对两域
  基线互不回归。
- html/dom 接口面与 25 号 html-semantics-compat（主攻中）的语义面划界：
  本 goal 只收 DOM 接口行为，事件/语义行为仍归 25 号；跨界用例在 M2 归口。
