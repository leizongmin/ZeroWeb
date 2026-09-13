---
date: 2026-09-14
modules: zero-engine,perf-gate
---

# CPU 频率窗效应——bench-gate 微基准跨窗漂移可达 2-3×（compositing_layer_analysis 误判归因）

## 问题

`compositing_layer_analysis_200` 微基准在 2026-09-14 多轮 bench-gate 中持续超预算（48-55µs vs 预算 46.7µs / 基线 34.6µs），初判为代码回归。同日全量 bench-gate 在 loadavg 0.16-1.44 的**低负载**窗内 11 个指标跨 5 个 crate 齐超预算（含 render-foundation `frame_buffer/clear_1080p` 纯 memset 型 2.47×——该函数与任何近期改动无交集），且同一指标数小时内测量值在 169k ↔ 343k 间摆动（wide_tree_500_children 定向跑 vs 全量跑）。

## 根因

本机 CPU governor 虽为 `performance`，但实际频率在 **399 MHz（idle floor）↔ 4234 MHz（boost）** 间剧烈震荡（/proc/cpuinfo 采样 30 次：min 399 / max 4234 / avg 1357 MHz）。criterion 的测量窗若落在低频段，微基准读数放大 2-3×；定向跑（指标少、前序编译/测量已把核心拉上高频）与全量跑（113 指标顺序执行、每个基准起点频率不确定）因此对同一代码测出显著不同的数。

关键误导点：**loadavg 门禁对此盲**——频率塌陷不需要并发负载（idle 即 400MHz），bench-report 的 loadavg>12 防护无法拦截。

## 归因纪律结论

- `promote_compositing_layers`（bench 循环体）输入预构建、与近期渲染流 diff（marker 定位/style 缓存）零交集；漂移 = 频率窗伪影，非代码回归。
- 判据：**纯内存型指标（clear_1080p memset）与其他计算型指标同比例膨胀 = 环境信号**；单一指标孤立漂移才值得查代码。

## 如何避免

- 微基准超预算时，先看**跨 crate 一致性**（多个无关指标齐超 = 环境窗）与**纯带宽型指标**（memset/clear 类放大 = 频率/内存子系统信号），再查 diff。
- 同日定向跑与全量跑对同一指标读数差 >30% 即为窗效应证据，勿以单窗读数定罪代码。
- 可能的基建级缓解（未实施，涉测量配置须拍板）：bench-report 测量前短促 busy-spin 拉频（让每指标起点频率一致）；或 criterion 前置 warm-up 迭代数提高。禁止以放宽阈值替代。
