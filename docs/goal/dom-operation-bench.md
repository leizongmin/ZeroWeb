# 跨浏览器 DOM 与脚本桥接操作基准

**版本**：v1.0 / 2026-10-20
**状态**：Active（启动门控：文档/脚本本轮创建；基准开发与运行待后续启动）
**父目标**：[zero-web](zero-web.md) / **控制面**：[master](dom-operation-bench/master.md)

## 0. 执行摘要与 Mission

固定同一页面与操作协议，比较 ZeroWeb/Chrome 的 DOM、样式/几何查询和 Canvas 状态成本，定位数量级异常并验证优化后的语义。Firefox 可在环境具备时补测，缺失不伪造数据。

依据：2026-10-20 用户要求按 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md)立项。先建立小操作基准，不自行重写 JS 引擎。

## 基线与 Support Envelope

静态基线：已有 `scripts/bench-report.sh`、`scripts/perf-gate.sh` 与 [性能预算](../specs/performance-and-resource-budget.md)；DOM 原生入口在 `crates/engine/src/dom_bindings/mod.rs`，shim 在 `js_dom_shim`。新跨浏览器基准 **NOT_RUN**。

范围：`getComputedStyle`、`getBoundingClientRect`、`replaceChildren`、列表插删、Canvas fillStyle 等状态操作；预热、操作数、样本与语义断言；桥接/样式/布局计数。

边界：criterion 既有基准不替换；不能将 V8 单独性能当浏览器 DOM 性能；不是 Speedometer 全量集成任务。性能修复仍走已有对应规范回归。

碰撞：按 [run-rules §9–11](../rally/run-rules.md)协调 engine/dom/script-sandbox 与 bench 脚本消费者，不覆盖既有 capture 或修改基线让门禁通过。

## Done Criteria / 验收场景（P1）

| ID | 条件与动作 | 期望与验证 |
|---|---|---|
| DC-1 / BENCH-01 | 同 fixture 反复查样式和几何 | 结果正确；区分读缓存与触发布局两类成本；拟新增 DOM 操作 runner |
| DC-2 / BENCH-02 | 固定规模替换 children、插删列表 | 节点顺序/数量与身份正确；记录耗时及桥接次数；不以 stub 绕过真实 DOM |
| DC-3 / BENCH-03 | 循环切换 Canvas 状态并绘制 | 状态解析与绘制结果正确；测量包含真实消费者而不是赋值空操作 |
| DC-4 / BENCH-04 | 同环境 A/B、正反序采样、引入错误结果 | 分位数/版本/操作数可追溯；错误语义使测试失败，污染样本为 INCONCLUSIVE |
| DC-5 | 修复异常后运行既有门禁 | 新报告和既有报告 schema 明确；无平台基线不冒充相对 PASS；优化有常驻回归 |

## 里程碑与实施交接

| 阶段 | 文件/职责与依赖 | 产物与验证 |
|---|---|---|
| M1 | tests/playwright-matrix 现有自动化 + 本地 fixture | 固定 JS 负载、预热/样本协议；先 style/geometry 双场景 |
| M2 | 拟新增 `tests/benchmarks` 操作报告与采集脚本 | 真实 ZeroWeb/Chrome 数据、语义检查、异常比值；缺测明确 |
| M3 | engine/dom/native/shim 真实热点 | 首差异剖析→最小修复→配对性能与行为回归 |
| M4 | bench-report/perf-gate 消费接口 | 仅确有价值的指标入既有门禁；不修改既有配置掩盖回归 |

实现来源：复用 CDP/WebDriver/Playwright 矩阵与原生 DOM；新增测量器放现有测试脚本目录，JSON/分位数优先标准库和已装依赖。建议批次：采集协议、基准资产、热点修复。可撤销本批优化或新增场景接线，旧基线保留。

## 执行与 Document Control

`./scripts/rally-goal-dom-operation-bench.sh` 直接执行启动 Rally，加 `--dry-run` 仅预览；统一 [CLI/守卫/证据](ladybird-followups/README.md)。本轮未启动或安装浏览器。

入口稳定，master/evidence 更新，archive 只追加；启动后 `CONTINUE` / `BLOCK`，全部 DC 有证并满足适用门禁才 `DONE`。
