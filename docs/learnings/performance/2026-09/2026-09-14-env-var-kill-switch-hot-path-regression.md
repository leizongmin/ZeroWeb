---
date: 2026-09-14
modules: zero-layout-engine
---

# 热路径遗留 env 探针——R4332_FOLD_OFF 逐 item `std::env::var` 致 wide_tree 微基准 +67%

## 问题

CI 守护轮（dispatch 34846841872，2026-09-14）benchmarks job perf-gate FAIL，孤立单指标：`mb/zero-layout-engine/wide_tree_500_children` 475.0µs vs 基线 247.2µs（预算 333.7µs，超 +92%）。同平台其余 layout 指标仅 +1~7%（环境底噪）——按《CPU 频率窗效应》的判据「单一指标孤立漂移才值得查代码」，孤立性成立，须查代码而非归因环境。

## 根因

R4332（2cbc3539c）在 `collect_items.rs` 两处 inline margin 计算中遗留了 A/B kill-switch：

```rust
+ if std::env::var("R4332_FOLD_OFF").is_ok() { 0.0 } else { border_adv_l };
```

每 inline item 无条件执行 4 次 `std::env::var`（两处代码位 × margin_left/right），且不受 `adv0 == (0.0, 0.0)` 零边框短路保护。`std::env::var` 每次调用锁全局 ENV RwLock 并线性扫描整个 environ——成本正比于环境变量个数。两个放大因素叠加：

- **wide_tree_500_children 是唯一以 default display（Inline）铺 500 兄弟的基准**（`make_block_doc` 显式 Block，`make_wide_doc` 不设 display），恰是该热路径独占消费者；
- **CI runner 的 environ 远大于本地 shell**（GitHub Actions 注入数十个长值变量）→ 同一代价 CI +191µs vs 本地 +57µs，平台放大比与 CI/local 硬件比（1.9-2.0）叠加后表现为 +67%/+68% 双窗同向。

## 归因链（可复用协议）

1. **孤立性筛环境**：同 crate 其余指标 +1~7% 而目标指标 +66% → 查代码；
2. **双窗复现定罪**（freq 窗判据「同窗二次复现才可定罪」）：本地 c975f6e70 锚点 176µs → HEAD 229-238µs 三轮聚簇 ±2%（loadavg 1.6→3.2 稳定），CI 同平台类 284µs（09-13）→ 475µs——两平台两窗同向即定罪，绕开单窗 A/B 相位污染；
3. **bisect 收点**：`git bisect run` + criterion min-of-2（防高频窗单侧误判），边界取 good/bad 观测中点，编译失败 exit 125 跳过——12+ 提交范围 4 步收口到 2cbc3539c。

## 解决方案

移除 4 处 env 探针，保留默认开启语义（`+ border_adv_l/r`）。env 未设是 CI/生产唯一形态，行为逐位不变；修正后本地 wide_tree 168.9µs 复原（锚点带内）。

**预防**：kill-switch/调试探针若确需保留，必须用 `OnceLock::get_or_init`（本仓既有正例 `ZW_CLEAR_MT_TAFFY_GUARD`，同文件 collect_items.rs）或编译期 `cfg!` 收敛为一次求值；任何 `std::env::var` 出现在逐 item/逐帧路径即应视为性能缺陷。热路径新增代码过 bench-gate 时若遇「系统繁忙门自拒」（R4332 提交说明记载 loadavg 17>12 自拒、A/B ±25% 噪声不可判），欠账的活跑复核应在净窗补上，而非默认无回归。
