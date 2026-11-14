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

## 补充（2026-09-06，ZRG-20260905T220123Z）：隔离复测的对照必须是「同日同环境的基线 SHA」，不能跨天比

隔离复测判据 2 有一个隐蔽陷阱：把污染窗口里的隔离值与**上一天的干净参照**直接比较，会得出「真实回归」的误判。本轮 `incremental_layout` 污染窗口隔离复测 2.614ms vs 前日参照 2.10ms = +24.5%，据此发了候选回归告警；随后用 `git worktree add` 在基线 SHA 建临时 worktree、同日交错 A/B（两侧各 3–5 轮，criterion 标准 warmup）推翻——基线 SHA 当日同法复测同样超预算（2.33–2.42ms vs 预算 2.399ms），成对差值 −0.9%~+5.5% 符号翻转，超限来自基线（录于 08-22）与当日环境的漂移而非代码。

修正后的判据 2 表述：**隔离复测的对照不是历史记录，而是同日建立的基线 SHA worktree**；判据 3（代码窗口核对）同理。跨天比较只用于趋势观察，不能作为回归定论。附带操作要点：worktree 里跑含 v8 的 bench 需把 `~/.cache/zero-web/rusty_v8/<v>/librusty_v8_*.a.gz` 复制为 worktree `.cargo/rusty_v8/archive` **文件**（主仓该路径是符号链接，build.rs 对目录会报 IsADirectory）；A/B 完成后 `git worktree remove --force` 清理。

## 补充（2026-11-14，R5529）：「稳定超限」可伪稳定；顺序双二进制 A/B 即便同日也会被负载漂移混淆

R5529 定期刷新 bench-gate 再踩同签名，两个新教训：

1. **判据 1 的补强**：run2 有四个指标连续两轮稳定超限 2.4-3.9× 且数值一致（15.83/15.6ns 等），几乎推翻「轮换=噪声」判据。实际同 commit 静窗复跑四项全部回到基线（4.68ns / 7.4µs / 61.7µs / 98.2µs）。「两轮数值一致」只证明两轮落在同一段负载水平内，是**伪稳定**；轮换要看跨轮失败集的全景（run1 19F → run2 5F → 复跑 5F 完全不同），不能因局部稳定而提前收敛结论。
2. **顺序 A/B 的混淆**：旧基线二进制（718e0f0e6）恰好跑在静窗得快值、HEAD 恰好跑在负载窗得慢值，据此差点向对端流发出假回归碰头信号。尽管都发生在同一天，09-06 补充的「同日交错 A/B」要求依然必要——负载在小时尺度漂移。本轮被静态证据救回：窗口 `crates/` diffstat 中 zero-dom 仅 1/4 行布尔恒等式化、Cargo.lock 零版本变化、无 profile 改动，任何代码面都解释不了纯 DOM 查询 3.5×。判据 3 的具体工具就是这条：**测量差值量级与窗口代码 diffstat 明显不匹配时，先怀疑测量环境**。
3. 门槛放行≠无争用：peer 单容器重腿把 loadavg 推到 4-8（阈值 12 以下），单线程 criterion 已膨胀 2-4×；运行中途 loadavg 4.2→10.2 跳变决定哪些指标恰好中弹。与 2026-10-18 亚秒突发穿透门槛同族：门槛是粗筛，criterion 对争用远比 loadavg 敏感。
