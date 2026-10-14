# 兼容性与回归证据

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 WPT/testharness/reftest、跨 crate 与产品测试、性能/oracle 的判据和边界，不维护页面语义实现。已存在 [项目验证入口](../../../../zeroweb-verify/SKILL.md)，按功能读取实际执行资产；本轮只核验源码来源。

## Gotchas
- testharness、同源 reftest、Chromium oracle 和窗口 GPU parity 的分母与真值来源不同，新增通过数不能抵消旧案例回归或缺证据。 (`tests/wpt-runner/src/testharness.rs`, `tests/wpt-runner/src/reftest.rs`, `tests/wpt-runner/scripts/cross-validate.py`)
- lib/bin 测试目标与独立产品 executable 不同，测试编译成功不会自动刷新 IPC peer；Makefile 单独补 renderer lib 两引擎腿，不能只看 bin 结果。 (`Makefile`, `tests/integration/Cargo.toml`)

## Architecture
- 像素成绩要经过资源/字体就绪、分类容差与退化审计；benchmark 先固定测量配置和平台，再对报告判门禁。 (`tests/wpt-runner/scripts/chromium-oracle-shot.mjs`, `tests/wpt-runner/src/reftest.rs`, `scripts/bench-report.sh`, `scripts/perf-gate.sh`)

## Child Knowledge Nodes
- [WPT、testharness、reftest 与资产账本](wpt-and-reftest.md) — 查发现、变体、多个参考页、资产恢复、容差与分母。
- [跨 crate、产品、多进程与协议测试](integration-and-product.md) — 查真实页面脚本、测试接线、旧 peer 和 CDP 双跑有效性。
- [字体 oracle、基准、性能趋势与覆盖](performance-and-oracles.md) — 查测量配置、硬件基线、负载污染、资源失真与覆盖率边界。
