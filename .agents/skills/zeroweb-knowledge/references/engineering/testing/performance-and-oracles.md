# 字体 oracle、基准、性能趋势与覆盖

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位测量配置、报告、基线和可信度判断；不保存可执行测量配方。`bench-report` 是测量侧，`perf-gate` 是比较侧，字体算法归 graphics；测试执行导航见 [项目验证](../../../../zeroweb-verify/SKILL.md)。

## Gotchas
- `ZERO_WEB_BENCH_QUICK=1` 只编译基准，不产生性能测量；`run-benchmarks.sh` 已是转发 bench-report 的兼容壳，不能沿旧脚本寻找采样实现。 (`scripts/run-benchmarks.sh`, `scripts/bench-report.sh`)
- 基准显式选择 16 个 crate/bench，不执行 `cargo bench --workspace`，因为本地 Taffy dummy bench 会失败；新增 benchmark 必须核对清单与配置哈希。 (`scripts/bench-report.sh`)
- perf-gate 对含测量 error 的报告返回 1，对 suspect 返回 3（不可判定）；后者是负载污染，不等于证明代码回归或证明通过。 (`scripts/perf-gate.sh`)
- 缺平台基线走 NEW/PASS+WARN，schema/config_hash 不一致返回 2；CPU 型号或核数不一致只保留绝对页面总耗时和表单硬门，不作相对微基准比较。 (`scripts/perf-gate.sh`)
- Chromium oracle 必须以 wpt-data 为 HTTP 根提供 `/fonts/ahem.css`；`file://` 曾使 Ahem 加载失败，导致正确 ZeroWeb 被错误 oracle 判成发散。 (`tests/wpt-runner/scripts/chromium-oracle-shot.mjs`)
- oracle 捕获对字体和图片只是有界等待，超时仍可继续截图；产出 PNG 不保证资源成功加载，需要检查资源与页面证据。 (`tests/wpt-runner/scripts/chromium-oracle-shot.mjs`)
- 覆盖率 workspace summary 会把 dom_bindings 合入 engine；可选子模块检查失败被 `|| true` 吸收，主脚本退出成功不能证明独立 DOM 绑定覆盖测量完成。 (`scripts/check-coverage.sh`)

## Architecture
- criterion 原始 times/iters 转为每次迭代 ns 后自行计算分位数；配置哈希包括 bench 清单、场景、视口、迭代数和 profile，不包含提交号。 (`scripts/bench-report.sh`)
- 表单保留渲染硬门要求 release、p95≤20ms、20ms 卡顿率≤5%，每输入 parse/style/full_layout 必须为 0，paint/publish 必须为 1；即使没有平台基线也检查这些计数。 (`scripts/form-input-perf-gate.py`)
- cross-validate 分别比较 ZeroWeb-test/ZeroWeb-ref 与 ZeroWeb-test/Chromium-test，只分析有三份匹配资产的案例；污染率是采样交集内的假通过上界估计，不是全仓准确率。 (`tests/wpt-runner/scripts/cross-validate.py`)

## Patterns
- benchmark 趋势快照带时分秒，避免同一天局部结果覆盖全量；auto-tighten 对已存在基线只取更低 p95，预算系数不变，无基线时可建立首次基线。 (`scripts/record-bench-trend.sh`)
- WPT 趋势分 upstream 自一致性与 oracle credible 两种口径，并记录套件 ref；filter 调试不写趋势，退出 1 且有完整报告可作为失败案例数据记录，解析总数为 0 则拒写。 (`scripts/record-wpt-trend.sh`)
