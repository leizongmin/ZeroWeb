---
date: 2026-09-03
modules: engine,dom,canvas,css-parser,render-foundation,host-runtime,browser-shell
---

# bench-gate 失败指标集轮换 + 隔离复测回基线 = 负载噪声签名（非代码回归的快速判据）

## 问题

双流并行开发机上，`make bench-gate` 全量跑出现 GATE FAIL，但失败指标在不同轮次间**完全轮换、零交集**：

- run A（18:13，gate 后台有并行任务）：`tab_creation_100` / `parse_html_wide_5000` / `compositing_layer_analysis_200` / `paint_complex_page_500_elements` / `window_config_builder_chain` / `build_1000_fills` 六项 FAIL；
- run B（18:26，机器近乎空闲）：`path_ops_1000_segments` / `save_restore_100_deep` / `transform_chain_100` / `css_parse_100kb` / `css_parse_by_size/rules/500` 五项 FAIL——与 run A **零交集**。

超限幅度还很大（`transform_chain_100` 124.9µs vs 预算 52.4µs，2.4×；`css_parse_100kb` 3.74ms vs 2.12ms，1.76×），容易误判为真实回归。

## 根因分析

两个信号叠加：

1. **测量窗口污染**：perf-gate 的 suspect 标记只在测量**结束后**查 loadavg；并行进程若在测量中途结束，loadavg 已回落 → `suspect: false`，污染样本照样进入比较（policy §8.1 的保护机制覆盖不到这种时序）。
2. **共享机器基线噪声**：本机是双流并行开发环境，另一流的 WPT/make test 重 CPU 任务与 bench 测量窗口碰撞时，µs 级微基准集体虚高——即 policy §8.1 记录的「csp_parse_1000 在 WPT 运行期间 +20~40%、隔离重跑回到基线」同族。

## 快速判据（三层证据）

1. **失败指标集轮换**：两轮 GATE FAIL 失败项零交集 → 不是特定代码路径劣化（回归的失败集是稳定的）。
2. **隔离复测回基线**：对失败项用 `cargo bench -p <crate> --bench <bench> -- "<filter>"` 单独复跑，全部以 1.5~3.3× 余量回到预算内（`transform_chain_100` 124.9→38µs、`css_parse_100kb` 3.74→1.6ms、`path_ops` 14.1→6.8µs、`window_config_builder_chain` 15.96→9.1ns、`tab_creation_100` 7.3→4.9µs）。
3. **代码窗口核对**：本轮变更（js_dom_shim webaudio 语义面）不触达任何失败指标所在预算敏感路径——shim 仅在 V8 沙箱执行期注入，benches 不加载 shim。

三条同时成立 → 判负载噪声（ZRG-2026-08-22/23/24-01 同签名），不动基线、不 relax、不 re-capture。

## 解决方案

- 判噪声后**重跑定论**（policy §8「失败重跑一次再定论」）或直接记录归因，禁止为通过门禁 relax/re-capture（config_hash 与 justification 会暴露）。
- 定向测量（`ZERO_WEB_BENCH_CRATES=...`）不能替代全量，但隔离单 bench 复测（`cargo bench -- "<filter>"`）是判噪声最快的手段——秒级出结果且天然独占机器。
- 改进方向（未实施）：perf-gate 的 suspect 判定若改为「测量期间周期采样 loadavg 峰值」而非仅结束时刻，可覆盖中途结束的污染时序。
