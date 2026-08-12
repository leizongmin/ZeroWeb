# 性能基线的平台分类必须区分 CPU

- 日期：2026-08-13
- 相关模块：`scripts/bench-report.sh`、`scripts/perf-gate.sh`、`docs/perf/baselines/`

## 问题描述

在 Xeon 8260 KVM 环境运行 `make bench-gate` 时，`form_input` 的 p95、jank 和语义计数全部通过，但 102 个互不相关的微基准和页面指标同时超预算，普遍比基线慢 3-7 倍。

## 根因分析

报告和基线都使用 `linux-x86_64` 作为 `platform_class`，配置哈希也相同，因此门禁直接比较二者；但当前测量机器是 Xeon 8260，基线由 i5-13500H 记录。平台分类没有包含 CPU 性能等级，导致不同硬件的结果被当作同一平台回归。

当 DOM、CSS parser、Canvas、网络、WebView 等无关指标同时按相近倍数退化，而变更只触及单一模块时，应先核对报告与基线的 `cpu_model`，不能通过放宽或重录基线掩盖分类错误。

## 解决方案

1. 先读取报告与基线的 `cpu_model`、`cpu_cores` 和 `platform_class`，确认比较对象是否同类。
2. 对当前变更使用最贴近风险面的子门禁作为有效证据，例如 HTML 输入路径使用 `form-input-perf-gate.sh` 的时延和语义计数。
3. 不在硬件不匹配时更新共享基线。
4. 后续应让平台分类包含稳定的 CPU 性能等级，或在 CPU 型号不兼容时将结果判为 `INCONCLUSIVE`。
