# Selection 兼容二期运行时控制面板

**状态**: Active——M1 未启动（2026-10-31 控制面建立；入口文档 v2.1 无人值守收口定稿）
**最后更新**: 2026-10-31

---

## 当前状态

待启动。入口文档 [`../selection-compat.md`](../selection-compat.md)（v2.1）为
唯一目标契约。

**与兄弟 goal 的边界**：

- 33 号 editing-compat——**ZeroWeb-3 工作区并行在跑**：selection/Range 的
  editing 依赖簇归 33 号，本 goal 只挂账登记、rally 不写对方控制面
  （2026-10-31 用户决策，避免跨工作区提交撞车）。
- 34 号 dom-expansion（dom/ranges）——共享 Range 底座，收敛轮核对互不回归。
- 共享底座跨界事项按 run-rules §9 先归口再动手。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| S1 | selection 语料 171/194 文件未覆盖（31,638 子测试，10-31 快照） | 待 M1 |
| S2 | 扩面后失败簇未归因（selection/editing/runner 分列） | 待 M1 |
| S3 | coverage 复测未做（未覆盖清零路径） | 待 M2 |

## 里程碑状态表

| 里程碑 | 内容 | 状态 |
|---|---|---|
| M1 | 语料扩面 + 扩面基线 + 归因表（DC-1/2/3） | 未启动 |
| M2 | 语义簇收敛（≤300 子测试且零新引擎概念判据，超限挂账）/ coverage 复测收口（DC-4） | 未启动 |

## 碰撞管理记录（run-rules §9）

- 2026-10-31：立项时无同域在跑碰撞；33 号 editing-compat 在 ZeroWeb-3 并行
  推进（编辑域底座共享，收敛轮核对三域互不回归）。
- 一期零回归底线锚：
  [`../archive/selection-compat/evidence/2026-10-14-m1-baseline.md`](../archive/selection-compat/evidence/2026-10-14-m1-baseline.md)。

## 待用户决策 / 待转登记

| # | 事项 | 状态 |
|---|------|------|
| T1 | editing 依赖簇挂账 → 用户转登记 33 号（M1 归因后） | 待 M1 产出 |
| D2 | 超限语义簇（>300 子测试或需新引擎概念）后续立项建议 | 待 M2 产出 |
