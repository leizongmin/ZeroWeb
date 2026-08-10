# 性能与资源占用预算（Performance & Resource Budget）

**版本**: v1.0
**日期**: 2026-08-08
**状态**: Active
**依据**: docs/goal/zero-web.md Done Criteria §4（回归门禁/JSON 持久化/趋势追踪）
**参考**: ~/work/ZeroUI 的三层门禁模型（gallery-performance-and-experience-budget.md）

---

## 1. 使命

防止 ZeroWeb 在长期迭代中发生**无感知的性能/资源劣化**。代码越堆越多时，性能、内存、
首屏时间会悄悄退化——本预算体系让每次劣化都能被测量、被门禁拦截、被趋势记录，
并支持「先建立门禁，后随优化收紧」。

## 2. 三层门禁模型

| 层 | 含义 | 示例 |
|---|---|---|
| **Hard Gate（硬门禁）** | 绝对预算，不随基线变化 | 页面首屏 `total_ms` p95 ≤ 2000ms |
| **Budget Gate（预算门禁）** | 相对基线，基线×因子 + 常数 | 微基准 p95 ≤ max(基线×1.35, 基线+1.0ns)（2026-08-10 校准） |
| **Trend Metric（趋势指标）** | 先记录、后晋升为预算 | startup_ms、各阶段耗时 |

趋势指标先记录一段时间（确认稳定），再根据分布收紧为正式预算。

## 3. 测量与门禁管线

```
bench-report.sh（测量）→ benchmark_${DATE}.json（gitignored 原始报告）
      ↓
perf-gate.sh（纯比较，永不融合测量逻辑）→ PASS / FAIL（退出码 0/1/2）
      ↓
record-bench-trend.sh（趋势）→ benchmark-trend.csv + 日快照（提交）
record-bench-baseline.sh（基线，手动）→ docs/perf/baselines/<platform_class>.json（提交）
```

### 3.1 测量面（scripts/bench-report.sh）

1. **criterion 微基准**：16 crate × [[bench]]（79 个基准函数）。criterion 0.5.1 无
   `--output-format json`，解析 `target/criterion/**/new/sample.json`（逐迭代
   ns = `times[i]/iters[i]`）自算 p50/p95/max。
2. **页面级首屏基准**：zero-wpt-runner `perf` 子命令，3 个 fixture 各 14 个样本
   （第 1 次 warmup 付字体加载/图片缓存成本）：
   - `welcome`（apps/browser/assets/welcome.html，自包含）
   - `medium`（tests/benchmarks/fixtures/medium.html，~4400 元素合成页，对齐
     「中等复杂度页面 < 2s」验收面）
   - `morning`（apps/browser/assets/morning-work/article.html，真实文章页离线副本，
     DC-13 产品 fixture 复用）
   输出各阶段耗时（parse/style/layout/paint/total）+ 墙钟首屏。
3. **进程资源**：峰值 RSS（Linux VmHWM / macOS getrusage / 其他平台 null）+ startup_ms。

### 3.2 门禁公式（scripts/perf-gate.sh）

| Tier | 指标 | 公式 |
|---|---|---|
| Hard | `page/*/total_ms` p95 | `≤ 2000.0` ms（绝对，对齐「首屏 < 2s」） |
| Budget | `mb/*` p95（ns） | 基线 ≥ 10ns：`≤ 基线 × 1.35`；基线 < 10ns：`≤ max(基线 × 1.35, 基线 + 1.0ns)`（2026-08-10 校准，见 §4） |
| Budget | `page/*` 各阶段 p95（ms） | `≤ 基线 × 1.15 + 40`（+40 常数吸收调度抖动） |
| Budget | `resource/peak_rss_mb` | `≤ 基线 × 1.20 + 128` MB |

- 报告中无基线条目的指标 → **NEW/PASS**，记录趋势，下一轮基线 capture 时纳入。
- `schema_version` / `run_config.config_hash` 与基线不匹配 → **exit 2**（测量配置变更，
  须重新 capture 基线；防止「改了场景/迭代数却拿旧基线比」）。
- 无该平台基线 → PASS+WARN 并附 capture 指引（首次使用流程）。
- 测量执行失败（任一 crate bench 非零）→ **exit 1**（bench-report.sh 同）。
- 退出码：`0` 全过 / `1` 回归或测量失败 / `2` 配置错误。

### 3.3 基线（docs/perf/baselines/，提交、硬件固定）

- 按 `platform_class` 分文件：本地 dev box（`linux-x86_64`）+ CI（`github-ubuntu-latest`，
  `GITHUB_ACTIONS=true` 自动选择）。
- **硬件固定**：基线绑定 CPU 型号/核数。更换机器/CPU → 必须重新 capture 基线，
  旧基线只作趋势参考。
- 基线是「收紧优先」的：`record-bench-baseline.sh` 发现新 p95 ≥ 旧值×1.005 且未显式
  `--relax` 时**拒绝**覆盖（防悄悄放宽掩盖回归）。

## 4. 预算变更流程

- **收紧**（降低因子/常数/绝对预算，或实测更优后 capture）：普通提交，建议附
  `--justification`。
- **放宽**（`--relax`）：必须同时满足——
  1. 政策文档（本文件）记录旧值、新值、前后测量数据、理由；
  2. `record-bench-baseline.sh --relax --justification "..."` 显式执行；
  3. 理由须解释为何不是回归 + 恢复计划（对齐 ZeroUI 纪律）。
- **自动收紧**（weekly CI `record-bench-trend.sh --auto-tighten`）：实测 p95 低于基线 →
  就地收紧（仅收紧永远合法，无需 justification）。

**2026-08-10 校准记录（放宽，用户批准）**：
- **旧值**：`mb/*` 预算 = `基线 × 1.20`（纯因子）。
- **新值**：`mb/*` 预算 = 基线 ≥10ns → `基线 × 1.35`；基线 <10ns → `max(基线 × 1.35, 基线 + 1.0ns)`（新增 `microbench_floor_ns` 预算键）。
- **前后测量数据**：基线（08-08 weekly cron 62c27520 建立）后，08-10 两轮 dispatch CI 3 指标超旧线：`ipc_deserialize_10000` 858.6→1126.4 µs（+31.2%）、`ipc_serialize_10000` 404.5→513.7 µs（+27.0%）、`damage_all` 2.19→2.82 ns（+28.8%）；同基线 08-08 4 轮 dispatch gate 全过。
- **理由（为何不是回归）**：三指标代码零改动（`protocol/src/ipc.rs`、`render-foundation/src/damage_tracker.rs` 最后提交均早于基线）——github 共享 runner 类测量方差系统性超出旧 1.20 因子；本地固定机器 damage_all 1.37ns 亦 ±10% run-to-run 波动，佐证 ns 级指标绝对抖动主导。新因子 1.35 为收紧优先下清除观察噪声带（+27~31%）的最紧可行值（1.30 仍会在 ipc_deserialize 1.312× 上红）。
- **恢复计划**：weekly `--auto-tighten` 持续收紧；1.5× 级真回归仍会被拦截（因子语义保留）；若 runner 类长期稳定可回落因子。
- **禁用逃生舱**：无 `--ignore-gate`、无环境变量跳过门禁、不许为通过门禁临时改测量
  配置（config_hash 会暴露）。违规修改视为门禁失效事故。

## 5. 执行点

| 场景 | 门禁 |
|---|---|
| **本地 rally 轮次**（主执行点） | `make bench-gate`——性能相关变更（解析/样式/布局/绘制/Canvas/JS 桥/网络关键路径）或代码量积累后必跑；run-rules.md 有对应条目 |
| **weekly CI**（ubuntu-latest） | `benchmark-trend` job：真实测量 + perf-gate（失败 = job 红）+ 趋势回写 + auto-tighten |
| **CI dispatch** | `benchmarks` job：真实测量 + perf-gate |
| **PR CI** | 不跑性能门禁（GitHub runner 硬件与基线不一致，噪声大）；仅 `ZERO_WEB_BENCH_QUICK=1` 编译检查 |

## 6. 既有测试断言的补充关系

- **首屏 < 2s**：`tests/integration/src/real_website_compat.rs`
  `test_performance_python_org`（`#[ignore]`，网络型）+ 本体系 `page/*/total_ms` Hard Gate。
- **增量渲染 < 全量 20%**：`tests/integration/src/e2e_rendering.rs`
  `test_incremental_render_performance_criterion`（`make test` 内断言，harness 不重复）。
- **test/build OOM 保护**：`scripts/test-guard.rs`（Makefile 全目标包裹）——资源**安全**
  门禁（防 OOM 杀 rally），本体系是性能**预算**门禁，二者互补。

## 7. 新增场景/指标的流程

1. 在 `bench-report.sh` 的 `PAGE_SCENARIOS`（页面）或 crate 的 `benches/*.rs`（微基准）添加；
2. `make bench-gate` 确认新指标以 NEW/PASS 出现；
3. `record-bench-baseline.sh --justification "新增 X 场景"` 纳入基线（config_hash 变化 →
   旧基线作废，一次性重 capture 全部指标）；
4. 记录到本文件 + master.md。

## 8. 已知边界

- GitHub ubuntu-latest 为共享 runner，存在硬件抖动 → 门禁仅 weekly/dispatch（非 PR），
  失败重跑一次再定论；公式偏 +constant。
- macOS/Windows 本地无 VmHWM 测量（rusage 仅 macOS），`peak_rss_mb` 为 null 时该指标
  SKIP 不门禁。
- 首次 CI 基线：由首个成功的 weekly/dispatch benchmark-trend 运行 capture 并提交回 main
  （justification「初始 CI 基线」）。

### 8.1 共享机器（双流并行）的测量保护（2026-08-08）

本机为双流并行开发（见 run-rules.md 并行规则），另一条流会不定期跑 WPT 全量
（reftest，重 CPU，~10 分钟级）——µs 级微基准在其运行窗口内集体超标
（实测 csp_parse_1000 在 WPT 运行期间 +20~40%、隔离重跑回到基线），
全量 bench-gate 与 WPT 全量会频繁碰撞。保护机制：

1. **负载守卫**（bench-report.sh）：loadavg 1min 超阈值（默认逻辑核数×0.75，
   `ZERO_WEB_BENCH_BUSY_THRESHOLD` 可调）→ 快速失败 exit 3 并提示重试；
   `ZW_BENCH_ALLOW_BUSY=1` 强制运行（不推荐）。
2. **suspect 标记**（bench-report.sh + perf-gate.sh）：测量**结束后** loadavg 再超阈值 →
   报告 `suspect: true`，perf-gate 判定 **INCONCLUSIVE（exit 3）** 不比较——防止
   中途叠加的 WPT 污染产出假 FAIL / 假收紧。
3. **定向测量**：`ZERO_WEB_BENCH_CRATES=zero-css-parser,zero-dom` 只测指定 crate
   （~2 分钟窗口，碰撞概率小），供局部优化验证 / 忙时小窗口测量；
   基线更新经 `record-bench-trend.sh --auto-tighten` 逐指标 min 合并（仅收紧）。
4. **不因噪声放宽基线**：µs 级基准的噪声用上述机制回避，禁止用 --relax 吸收
   「对方在跑测试」类噪声（会掩盖真实回归）；确认为环境噪声的指标保持原基线。

**经验**：共享机器的长时间测量（>5 分钟）天然脆弱，优先短窗口定向测量 +
  逐指标收紧；权威门禁以 dedicated runner 的 weekly CI 为准。
