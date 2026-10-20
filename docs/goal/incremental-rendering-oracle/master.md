# 增量渲染对照控制面

**日期**：2026-10-20
**状态**：Active（启动门控）；规划已创建，开发与双路径验收 NOT_RUN。
**当前卡点**：尚无确定性双路径快照与失效反例。
**下一步**：明确启动后盘点完整/增量入口，固定最小 DOM/CSSOM 序列。
**入口**：[目标契约](../incremental-rendering-oracle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 完整计算参考与最终输出快照 | 包含 Taffy 后处理、溢出和命中几何 | NOT_STARTED |
| 继承/Shadow/伪元素/字体失效序列 | 建立可重放语料并人工注错验证检测器 | NOT_STARTED |
| 缓存与帧复用生命周期 | 固定重复导航、缓存释放观测 | NOT_STARTED |
| 浮点与像素容差、检查模式成本阈值 | M1 按输出契约和配对基线固定 | 待核查 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 入口/缓存盘点与同输入双路径基线 | NOT_STARTED | 无 |
| M2 | 变更回放与首差异定位 | NOT_STARTED | 无 |
| M3 | 生产帧/像素与释放验证 | NOT_STARTED | 无 |
| M4 | 检查成本、规范回归和 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | INC-01-dom-cssom-mutations | NOT_RUN | 无 |
| DC-2 | INC-02-inheritance-shadow-has | NOT_RUN | 无 |
| DC-3 | INC-03-layout-cache-inputs | NOT_RUN | 无 |
| DC-4 | INC-04-final-output-and-retention | NOT_RUN | 无 |
| DC-5 | 生产默认路径与检查成本 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：style/layout/engine/render-foundation/compositor/字体及 WPT corpus。实际占用 **NOT_CHECKED**；启动时按 run-rules §9–11 核对，保留现有 WPT pin 与其他流语料。

本轮只创建规划；提交、推送、通知和发布未授权。运行遵守统一证据与守卫协议；master 持续更新，`evidence/` 和 `archive/` 追加保存。当前没有运行记录，内部等价不能替代规范正确性验收。
