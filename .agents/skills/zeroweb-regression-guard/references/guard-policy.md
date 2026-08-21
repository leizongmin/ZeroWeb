# 防劣化判定策略

## 1. 核心门禁

每个新 SHA 至少执行以下检查：

| 门禁 | 生产证据 | 失败条件 |
|---|---|---|
| CPU、页面性能与整体内存 | `make bench-gate` + `docs/perf/baselines/<platform_class>.json` | `perf-gate` 报告任何 FAIL；配置错误单独判 BLOCKED |
| renderer/compositor 分进程内存 | 真实窗口、多进程、GPU 路径；只统计本次启动的 browser 子进程树 | 任一生产路径绝对预算超限 |
| compositor 连通性 | 从首次 surface 登记到场景结束的完整日志 | 成功出帧后出现帧响应超时、断连或 legacy 回退；或场景要求的帧未完成 |
| 产品静态渲染 | `make product-smoke` | 像素、结构或命令门禁失败 |
| Chrome 生产一致性 | `zeroweb-browser-chrome-parity` 的 production evidence | 状态、事件、几何、像素或必需产物失败 |

Chrome parity 默认使用仓库已有的最小代表场景；若变更触及表单、输入、DPR、字体、布局、绘制或 compositor，则选择覆盖该模块的场景。不得用 CPU/headless 截图替代生产窗口 GPU 证据。

## 2. 已有性能与资源基线

`make bench-gate` 是 CPU、页面阶段耗时和整体峰值 RSS 的权威相对基线入口。它生成报告后由 `scripts/perf-gate.sh` 与 `docs/perf/baselines/<platform_class>.json` 比较，包括：

- `mb/*` CPU 微基准；
- `page/*` parse/style/layout/paint/total 与首屏墙钟；
- `resource/peak_rss_mb`，预算沿用 `baseline × 1.20 + 128 MB`。

巡检必须直接采用该脚本的指标、平台选择、配置哈希、阈值和退出码，不复制公式到新脚本，不自动执行 baseline capture/relax。**唯一例外**：goal master 控制面已记录用户明确放行的一次性 re-capture（如 zero-web master GB-20260821 用户放行块，2026-08-19 批复 + 2026-08-21 放行 bench 基线重建）——按批复 JUSTIFICATION 执行 `make bench-capture` 并在完成后记入控制面，不属违规 relax/覆盖。`NEW/PASS`、无匹配平台基线或硬件不匹配只能说明缺少可比基线；报告中必须标明，不能宣称“相对基线无劣化”。busy/suspect 导致的 exit 3 为 `INCONCLUSIVE`，按性能预算文档等待空闲后重测。

## 3. 生产分进程内存补充门禁

现有 `resource/peak_rss_mb` 不区分 renderer 与 compositor，也不覆盖真实窗口多进程 GPU 生命周期，因此补充以下生产路径绝对预算：

1. 使用 release 产物，固定页面、viewport、DPR、字体环境和 GPU 配置。
2. 从启动前开始监控，只纳入本次 browser 的后代进程；不得按进程名汇总其他会话的实例。
3. 至少记录 renderer 和 compositor 各自的：稳定首帧后驻留内存、10 秒空闲后的驻留内存、场景全程峰值。
4. Linux 优先读取 `/proc/<pid>/status` 的 `VmRSS`/`VmHWM`；Windows 使用 `Get-Process` 的 `WorkingSet64`/`PeakWorkingSet64`。报告必须写明平台和指标语义，不混用不同口径作相对比较。

默认绝对预算：

| 指标 | 预算 |
|---|---:|
| renderer 单进程峰值 | ≤ 192 MiB |
| compositor 单进程峰值 | ≤ 192 MiB |
| renderer + compositor 稳定空闲总量 | ≤ 256 MiB |

这些分进程数据写入本次报告并用于趋势诊断，但不得伪装成仓库已有 `perf-gate` 基线。绝对预算与 `perf-gate` 相对预算任一失败都算回归；不得拿另一台机器或另一种内存口径的结果比较。

## 4. compositor 日志判定

以下日志在成功出帧后出现时属于硬失败：

- `帧响应超时` / `frame response timeout`
- `Compositor disconnected`
- `switched all renderers to legacy`

单独的 `AssignProcessToJobObject failed (likely already assigned or permission denied)` 不足以证明 compositor 断连，必须结合进程存活、IPC 和帧日志判断。不得通过延长 watchdog、删除日志或无条件 legacy fallback 修复假象。

## 5. 复现与噪声控制

- 性能或内存失败在相同环境重跑一次；两次均失败才进入自动修复。700 MiB 级等超过绝对预算 2 倍的失败可直接认定，无需重复消耗资源。
- GUI/GPU、Chrome、字体、依赖或显示服务缺失时判 `BLOCKED` 并飞书通知，不修改代码。
- `bench-gate` 的 busy/suspect/INCONCLUSIVE 按性能预算文档处理，等待机器空闲后再测，禁止放宽基线。
- 自动修复时间上限为单次 Rally job 的 timeout；到期前保留证据并发送卡点通知。

## 6. 专项修复记录

以下任一条件成立时停止自动修复：

- 需要跨进程协议、所有权或生命周期重构；
- 需要修改/放宽性能政策、阈值或既有基线；
- 无法在 job 时间内完成根因测试和全部质量门禁；
- 两次最小修复尝试仍未让原门禁通过；
- 修复风险或范围需要用户决策。

停止后只撤销本轮尚未提交的实验性代码，不触碰运行前已有改动。将问题写入对应控制面：renderer/compositor、字体、布局、绘制或 Chrome parity 问题写入 `docs/goal/rendering-compat/master.md`；其他性能/资源问题写入 `docs/goal/zero-web/master.md` 的“待用户决策/专项修复”清单。记录至少包含：

- 唯一问题 ID、发现日期、被测 SHA 和首个可疑提交范围；
- 失败指标、仓库基线文件、基线值、预算和两次实测值；
- 复现命令、证据目录及环境口径；
- 已确认事实、根因假设、尝试过的修复及失败原因；
- 建议的专项方案、风险、预计涉及模块和需要人工确认的事项。

按文档提交规则和 `lei-pre-commit-guard` 提交、普通推送这份记录到 `main`，不得夹带未验证代码。推送后把记录提交的 `HEAD`、问题 ID 和控制面路径写入 `.rally/zeroweb-regression-guard.last-deferred`；同一 HEAD 且该问题仍为打开状态时不重复运行或告警。任何后续新提交都会重新触发巡检。

## 7. 报告最小字段

每次完整运行的报告至少包含：

```json
{
  "git_sha": "...",
  "platform": "...",
  "scenario": "...",
  "perf_baseline": "docs/perf/baselines/<platform_class>.json|null",
  "perf_regressions": [],
  "memory_metric": "...",
  "renderer_peak_mib": 0,
  "compositor_peak_mib": 0,
  "idle_total_mib": 0,
  "compositor_disconnects": 0,
  "bench_gate": "PASS|FAIL|INCONCLUSIVE",
  "product_smoke": "PASS|FAIL|BLOCKED",
  "chrome_parity": "PASS|FAIL|BLOCKED",
  "verdict": "PASS|REGRESSION|BLOCKED|DEFERRED",
  "deferred_issue": null
}
```
