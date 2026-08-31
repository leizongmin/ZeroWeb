# R385 — M4/DC-3 全量 dom sweep 快照（default-on 后）+ 按子分类通过率

**日期**: 2026-08-31
**命令**: `make testharness-dom TIME_LIMIT=2000`（test-guard 包裹；commit `90ac9c654` HEAD，js-dom R385 收官轮）
**原始输出**: [2026-08-31-r385-m4-sweep-55808P.txt](2026-08-31-r385-m4-sweep-55808P.txt)

## 总量

| 指标 | 数值 |
|------|------|
| **Pass** | **55,808** |
| Fail（subtest） | 12（分布 7 文件） |
| Timeout（文件级） | 15 |
| **subtest 通过率** | **99.95%**（55808/55820） |
| vs R380 基线（55808P/11F） | Pass 数**完全一致**——default-on + kill-switch 删除后净零漂移 |

## 按子分类

| 子分类 | Pass | Fail | Timeout | 通过率 |
|--------|------|------|---------|--------|
| dom/（根散用例） | 119 | 5* | 0 | 96.0% |
| dom/abort | 3 | 0 | 0 | 100% |
| dom/collections | 49 | 0 | 0 | 100% |
| dom/events | 598 | 1 | 10 | 98.3% |
| dom/lists | 189 | 0 | 0 | 100% |
| dom/nodes | 12,791 | 4 | 4 | 99.96% |
| dom/ranges | 40,455 | 2 | 0 | 100% |
| dom/traversal | 1,604 | 0 | 1 | 99.94% |

\* 根目录 Fail = historical 3（stale 期望不追）+ window-extends 2（EventTarget 继承域转档）。

## Fail 集合定性（与 R380/R384 定档恒等，无新增）

| 文件 | 数 | 定性 |
|------|----|------|
| dom/nodes/MutationObserver-document | 3 | parse-time MO（解析流式交错架构域，pending-apply RFC pa4） |
| dom/nodes/remove-and-adopt-thcrash | 1 | window.open 无 popup 通道（环境基建） |
| dom/events/click-on-absolute-pseudo | 1 | Chromium 专有 ::-webkit 语义，不追 |
| dom/ranges/Range-mutations-{dataChange,replaceData} | 2 | R353 游离树 mega-case 特化（L2 域挂账） |
| dom/historical | 3 | stale 期望不追（现行 spec 已回归 Node） |
| dom/window-extends-event-target | 2 | EventTarget 继承域（转档） |

**Timeout 15 文件** = R331/R355/R384 多轮记录的并发轮转族（events 跨 realm/
webkit-动画族/insertBefore-iframe-crash 等），单跑复验全 Pass（R384 evidence 表）。

## DC-3 判定

- ✅ 上游 WPT dom 真实用例已导入（`tests/wpt-runner/wpt-data/dom/`，8 子目录 + 根散用例；
  `imported-tests.txt` 账本 72+ 条目）
- ✅ 按子分类通过率报告（本文档，文本 + 表格；原始逐 subtest 输出同目录持久化）
- ✅ 基线建立且持续维护（R1 41.25% → 当前 99.95% subtest 级；55808P 净零漂移 @ default-on）
- ✅ driving 用例经 `make import-wpt` 资产化记入账本（CLAUDE.md 测试资产化规则）
