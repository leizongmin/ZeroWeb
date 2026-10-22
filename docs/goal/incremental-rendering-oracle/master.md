# 增量渲染对照控制面

**日期**：2026-10-23
**状态**：Active（已启动：2026-10-23 Rally 运行解除启动门控）。M1 style 层双路径基线完成，反例 A/B 固化；布局/帧/成本面未动。
**当前卡点**：反例 B（祖先链重算丢伪元素，生产增量臂可达）待修——`compute_styles_incremental` 祖先链走 `compute_element_style_internal` 不含伪元素计算，修复后需 style-system 全量回归 + 两个生产消费者行为核对。
**下一步**：M2 切片 1：修反例 B（祖先链重算补伪元素），oracle 步骤改 Equal；随后补 `:has()` 全量回退计数（DC-2）。
**入口**：[目标契约](../incremental-rendering-oracle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 完整计算参考与最终输出快照 | style 层已建（`ComputedStyle` PartialEq 精确契约）；布局/最终输出快照待建 | **PARTIAL**（style 层完成，[证据](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md)） |
| 继承/Shadow/伪元素/字体失效序列 | 序列已固化于 `crates/style-system/src/tests/incremental_oracle.rs`（8 步）；反例 B 修复 + `:has()` 回退计数待做 | **PARTIAL** |
| 缓存与帧复用生命周期 | 固定重复导航、缓存释放观测 | NOT_STARTED |
| 浮点与像素容差、检查模式成本阈值 | style 层已定：精确相等不设容差（见证据"比较契约"节）；几何/像素容差待布局切片 | **PARTIAL** |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 入口/缓存盘点与同输入双路径基线 | **IN_PROGRESS**（style 层完成：入口/缓存盘点 + 8 序列基线；layout-engine→engine 最终输出快照未做） | [2026-10-23-m1-style-dualpath-baseline](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md) |
| M2 | 变更回放与首差异定位 | NOT_STARTED | 无 |
| M3 | 生产帧/像素与释放验证 | NOT_STARTED | 无 |
| M4 | 检查成本、规范回归和 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | INC-01-dom-cssom-mutations | **IN_PROGRESS**（style 层 3 序列 Equal PASS + 反例 A 固化；确定性回放/差异报告工具化待 M2） | 同上 |
| DC-2 | INC-02-inheritance-shadow-has | **IN_PROGRESS**（继承序列 PASS、`:has()` 回退一致；Shadow/slot/字体序列与回退计数未做） | 同上 |
| DC-3 | INC-03-layout-cache-inputs | NOT_RUN | 无 |
| DC-4 | INC-04-final-output-and-retention | NOT_RUN | 无 |
| DC-5 | 生产默认路径与检查成本 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：style/layout/engine/render-foundation/compositor/字体及 WPT corpus。本轮实际占用：`crates/style-system`（property 两文件 + 新测试模块）+ 本 goal 文档；未触碰 engine/layout/WPT 语料，无冲突。保留现有 WPT pin。

启动后执行记录：
- 2026-10-23：启动门控解除；M1 style 切片落地（盘点 + 双路径 oracle 8/8 + 反例 A/B 固化）；`make target/test-guard` 定向测试通过。运行遵守统一证据与守卫协议；`evidence/` 与 `archive/` 追加保存。内部等价不能替代规范正确性验收。
