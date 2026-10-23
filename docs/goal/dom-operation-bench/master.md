# DOM 操作基准控制面

**日期**：2026-10-20
**状态**：Active（**M1 已解锁**——2026-10-23 用户决策，无人值守可启动；共享 engine/dom/script-sandbox 面与 html-semantics 流，启动前按 run-rules §9–11 错峰核对）；规划已创建，基准开发与运行 NOT_RUN。
**当前卡点**：没有统一 fixture、采样协议和语义断言。
**下一步**：执行 M1——先固定 style/geometry 两类负载并复用自动化入口。
**入口**：[目标契约](../dom-operation-bench.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 真实 DOM/Canvas 基准 fixture | 区分缓存读取、强制布局与真实绘制 | NOT_STARTED |
| 预热/操作数/样本/正反序协议 | 固定浏览器版本与环境，检测污染样本 | NOT_STARTED |
| 桥接、样式与布局热点归因 | 基线后选择数量级异常的首个调用链 | NOT_STARTED |
| 报告 schema/入门禁指标/Firefox 环境 | 复用现有消费者；缺失平台记 NOT_RUN | 待核查 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 操作/采样与语义协议 | NOT_STARTED | 无 |
| M2 | ZeroWeb/Chrome 实测与操作报告 | NOT_STARTED | 无 |
| M3 | 首热点最小修复与配对回归 | NOT_STARTED | 无 |
| M4 | 有价值指标接线与 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | BENCH-01-style-and-geometry-read | NOT_RUN | 无 |
| DC-2 | BENCH-02-replace-and-list-mutation | NOT_RUN | 无 |
| DC-3 | BENCH-03-canvas-state | NOT_RUN | 无 |
| DC-4 | BENCH-04-paired-and-semantic-check | NOT_RUN | 无 |
| DC-5 | 报告与既有门禁 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：engine/dom/script-sandbox、bench-report/perf-gate 和自动化消费者。实际占用 **NOT_CHECKED**；启动前按 run-rules §9–11 核对，不覆盖既有 capture 或基线。

本轮未运行或安装浏览器，提交、推送和通知未授权。后续按统一协议保存命令、原始样本、分位数、语义结果与清理到 `evidence/`，里程碑追加 `archive/`。当前没有运行记录。
