# M4 门禁接线——dom-ops 基准接入 perf-gate 消费接口

**日期**：2026-11-04　**里程碑**：M4（有价值指标接线与 DC 审计）　**协议**：dom-ops v1.1

## 结论

1. **接线完成，判定零新增**：`scripts/dom-ops-gate.sh` 编排「转换（`tests/benchmarks/dom-ops/to-bench-report.mjs`）→ 比较（原样调用 `scripts/perf-gate.sh`）」。perf-gate.sh 一行未改；domops 用独立基线文件 `docs/perf/baselines/domops-linux-x64.json`（config_hash 按测量协议隔离，不与 criterion 共写）。
2. **门禁真实工作**：干净窗口采集比较 → GATE PASS exit 0（6 指标全预算内）；污染窗口采集比较 → GATE FAIL exit 1，逐行定位 + 经负载采样日志归因为环境噪声（未录为回归、未动基线）。退出码 0/1/2 语义与既有门禁一致。
3. **协议 v1→v1.1**：thrash 行样本 12→8。12 样本 × ~610ms + 间隙 ≈ 7.6s 贴 ZeroWeb CDP `awaitPromise` 8s 悬崖，实测 reverse 超时（capture#1 forward 侥幸擦过、reverse 坠落 INCONCLUSIVE）——M3 v3 是侥幸通过。8 样本总预算 ≈5.2s 留 2s 余量；修在首次基线录制前，无兼容包袱，config_hash 随版本变化自动失效旧报告。
4. **DC-5 三条款**见下节。

## 接线设计

| 组件 | 文件 | 职责 |
|---|---|---|
| measure | `tests/benchmarks/dom-ops/run-dom-ops.mjs` | 采集（v1.1 起报告带 `config_hash` = 协议规范字段 sha256；协议/采样参数任何变更换值 → perf-gate exit 2 拒绝跨配置比较） |
| convert | `tests/benchmarks/dom-ops/to-bench-report.mjs` | dom-ops 报告 → benchmark 格式；只做格式映射与入选行筛选；入选行缺失/非 PASS/缺 config_hash → 非零退出（门禁不在坏样本上判定） |
| gate | `scripts/dom-ops-gate.sh` | 编排录制/比较；录制模式产出独立基线（预算公式沿用共享基线同源数值：mb 1.35 因子 / floor 1ns / page +40ms / rss +128MB）；比较模式 exec perf-gate.sh 退出码透传 |

### 入选行（确有价值准则：M3 归因明确 + 跨轮绝对值稳定 + 对修复面敏感）

| 报告条目 id | p95 口径 | 基线 p95 | 哨兵面 |
|---|---|---|---|
| `domops/zeroweb/style-geometry/geom.write_read.thrash` | max(fwd,rev) p95 | 13.40ms | M3 缺陷1 面：pending 读全量有效视图重建成本（10079x 异常） |
| `domops/zeroweb/style-geometry/geom.read.same` | 同上 | 24.50µs | gBCR drained 快路径成本 |
| `domops/zeroweb/mutation/mut.replace.pool` | 同上 | 6.95ms | M3 缺陷2 面：shim 逐节点结构成本（358.8x 异常） |
| `domops/chrome/*`（同 3 行） | 同上 | 1.55µs / 0.46µs / 17.65µs | 同环境哨兵：环境漂移先红哨兵行 |

未入选与理由：`style.*`/`style.write.color`（比值由 Chrome 亚 µs 分母驱动，未归因）、`geom.read.rotate`（与 same 同链冗余）、canvas 全行（29.8x 异常未归因，防神秘失败）、timer-INCONCLUSIVE 行（计时分辨率）。双浏览器都入；每 op 取正反序 p95 较大者（保守），单位转 ns（perf-gate 的 floor_ns 预算按 ns 语义设计）。

## 采集与门禁运行记录（同日四连跑，全部真实数据）

| 采集 | 结果目录（gitignored） | 窗口 | 用途 |
|---|---|---|---|
| capture#1 | `20261104_194357` | 负载衰减尾 | **暴露协议缺陷**：thrash reverse `awaitPromise` 超时 INCONCLUSIVE（fwd 12.17ms/op × 12 样本 ≈ 7.6s 撞 8s 上限）→ 触发 v1.1 |
| capture#1' | `20261104_195001` | **被污染**（zw 行抬高 1.5-3x，chrome 哨兵 thrash +50%） | 弃用：污染基线会放宽预算掩盖回归 |
| capture#2 | `20261104_200350` | 干净（load ≤1.29） | **录制基线**（6 指标，28 行语义全 PASS，exit 0） |
| capture#3 | `20261104_200712` | **被污染**（EMA 爬坡 0.74→2.22 恰覆盖 zw forward 相位） | GATE FAIL exit 1：thrash 23.5ms（+76%）、geom.read.same 45.5µs（+86%）、replace.pool 持平；chrome 哨兵全 PASS → 环境噪声非产品回归。**不录为回归证据，未动基线** |
| capture#4 | `20261104_201253` | 干净（瞬时 busy ≤19%，/proc/stat 采样） | **GATE PASS exit 0**：6 指标全预算内，zw 行贴基线（thrash 12.52 vs 13.40ms；geom.read.same 24.52 vs 24.50µs 漂移 0.1%） |

方法教训（cap3）：loadavg 是 1min EMA，30-60s 编译爬坡在 EMA 里只见 ~2.2，却足以把 zw forward 相位推高 ~80%；采集窗口判定必须用 /proc/stat 瞬时 busy%（cap4 起采用）。chrome 哨兵行的哨兵价值两次实证：#1' 中 chrome 行同步抬高（+50%）证明全局污染；#3 中 chrome 行全稳而 zw 行抬升 + 时序（仅 forward 相位）证明局部短阵发。

## 已知限制（在册）

- **dom-ops runner 无内建负载守卫**（criterion 侧 bench-report.sh 有 suspect 检测自动重试）。当前靠外编排除污染窗口 + 采样日志事后裁决；内建 suspect 检测（采样 busy% 超阈 → 报告标 suspect → perf-gate exit 3）留 dom-ops runner 后续工作。
- domops 转换报告 `suspect: false` 恒定（runner 未产出该信号）——门禁侧依赖采集纪律，不冒充有守卫。
- capture#1' / #3 两份污染数据保留在 gitignored 结果目录，仅作环境噪声证据，不进任何基线。

## DC-5 审计

| 条款 | 证据 |
|---|---|
| 修复异常后运行既有门禁 | `make bench-gate`（既有 criterion 全量门禁）于本改动树上运行，结果见控制面本轮记录（历史噪声日 FAIL 如实记录） |
| 新报告和既有报告 schema 明确 | dom-ops 报告 v1 + `config_hash`（本窗口新增字段）；转换器校验 `schema_version===1 && kind==='dom-ops-report' && config_hash` 存在，旧 runner 报告明确拒绝；benchmark 格式与 perf-gate 消费契约（`microbenches[]{id,p95,unit}`，id 无 `mb/` 前缀；空 `pages`/`resource` → SKIP）经 4 次真实运行验证；domops 基线 schema 与 budgets 公式见 `scripts/dom-ops-gate.sh` 头注 |
| 无平台基线不冒充相对 PASS | perf-gate 无基线分支 WARN + NEW/PASS exit 0（不冒充相对判定）；domops 基线仅 `domops-linux-x64.json`（Firefox NOT_AVAILABLE 如实在册）；cpu_model/cores 不匹配分支仅跑绝对门禁并 WARN |
| 优化有常驻回归 | 缺陷1：`crates/engine/src/rect_bridge.rs` `test_effective_rect_handler_reflects_pending_style_write` + `test_effective_rect_handler_layout_follows_viewport`（commit 90572cc78）；缺陷2：`crates/engine/src/js_dom_bridge_tests/part96.rs::handle_attr_read_survives_drain_via_history_inbox_m3d2` + `apps/renderer/src/js_worker.rs:6130::renderer_js_worker_handle_attr_read_survives_drain_via_history_inbox_m3d2`（commit 9bab87ae3）。自修复提交起两文件零改动（`git log 90572cc78..HEAD --` 为空），M3 轮绿色运行证据仍适用 |

## 环境与产物

- 采集主机：linux-x64 / i5-13500H 16 核；ZeroWeb `target/release/zero-browser`（git 700d6ddb7，含 M3 两缺陷修复）；Chrome headless。
- 门禁产物（入库）：`docs/perf/baselines/domops-linux-x64.json`（recorded_at 2026-11-04T12:06Z，justification 见文件）。
- 转换报告与原始采集（gitignored）：`tests/benchmarks/results/domops_bench_*.json`、`tests/benchmarks/dom-ops/results/20261104_*`。
