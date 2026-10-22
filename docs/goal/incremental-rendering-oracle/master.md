# 增量渲染对照控制面

**日期**：2026-10-23
**状态**：Active。M2 切片 1 完成：反例 B（链走丢伪元素）与反例 C（链走丢容器上下文）修复收口——伪元素块提为 `attach_pseudo_elements` 共享 helper，增量链走补伪元素挂载与容器条目压栈；oracle 9/9，style-system 2445/2445。
**当前卡点**：反例 A（`:only-child` 类兄弟失效）保持开放——属增量 API 的 changed 集完备性契约，生产现路由（单目标 paint-only mutation）不可达；不在增量入口内修，避免给全量路径加无用重算。
**下一步**：M2 切片 2：`:has()` 全量回退计数（DC-2"全量回退有计数"，style-system 内增量 instrumentation + oracle 断言）与 Shadow/slot/字体失效序列固化；随后 M1 布局剩余切片（layout-engine→engine 最终输出双路径快照）。
**入口**：[目标契约](../incremental-rendering-oracle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 完整计算参考与最终输出快照 | style 层已建；布局/最终输出快照待建 | **PARTIAL**（style 层完成） |
| 继承/Shadow/伪元素/字体失效序列 | B/C 已修复收口（[证据](evidence/2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md)）；`:has()` 回退计数 + Shadow/slot/字体序列待做 | **PARTIAL** |
| 缓存与帧复用生命周期 | 固定重复导航、缓存释放观测 | NOT_STARTED |
| 浮点与像素容差、检查模式成本阈值 | style 层精确相等已定；几何/像素容差待布局切片 | **PARTIAL** |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 入口/缓存盘点与同输入双路径基线 | **IN_PROGRESS**（style 层完成；layout-engine→engine 最终输出快照未做） | [m1-style-dualpath-baseline](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md) |
| M2 | 变更回放与首差异定位 | **IN_PROGRESS**（切片 1 完成：B/C 根因修复 + oracle 回归；切片 2：回退计数 + Shadow/字体序列） | [m2-s1-chain-walk-context-fix](evidence/2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md) |
| M3 | 生产帧/像素与释放验证 | NOT_STARTED | 无 |
| M4 | 检查成本、规范回归和 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | INC-01-dom-cssom-mutations | **IN_PROGRESS**（style 层 4 序列 Equal + 反例 A 固化；B/C 修复回归入账；回放工具化待 M2） | 两份证据同上 |
| DC-2 | INC-02-inheritance-shadow-has | **IN_PROGRESS**（继承/`:has` 回退一致 + B/C 收口；Shadow/slot/字体序列与回退计数未做） | 同上 |
| DC-3 | INC-03-layout-cache-inputs | NOT_RUN | 无 |
| DC-4 | INC-04-final-output-and-retention | NOT_RUN | 无 |
| DC-5 | 生产默认路径与检查成本 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：style/layout/engine/render-foundation/compositor/字体及 WPT corpus。本轮实际占用：`crates/style-system`（lib.rs 提取 + 增量链走修复 + oracle 测试）+ 本 goal 文档；未触碰 engine/layout/WPT 语料，无冲突（729e858ad 为 zero-web 流 engine 提交，无文件交叠）。保留现有 WPT pin。

启动后执行记录：
- 2026-10-23：启动门控解除；M1 style 切片（盘点 + oracle 8/8 + 反例 A/B 固化）。
- 2026-10-23：M2 切片 1：反例 B/C 根因修复（共享 helper + 链走容器上下文），oracle 9/9、style-system 全量 2445/2445、定向 bench-gate PASS；反例 A 记为 API 契约保持开放。`evidence/` 与 `archive/` 追加保存；内部等价不能替代规范正确性验收。
