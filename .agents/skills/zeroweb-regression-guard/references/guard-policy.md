# 防劣化判定策略

## 1. 核心门禁

每个新 SHA 至少执行以下检查：

| 门禁 | 生产证据 | 失败条件 |
|---|---|---|
| renderer/compositor 内存 | 真实窗口、多进程、GPU 路径；只统计本次启动的 browser 子进程树 | 任一绝对预算超限，或同机历史基线显著回退 |
| compositor 连通性 | 从首次 surface 登记到场景结束的完整日志 | 成功出帧后出现帧响应超时、断连或 legacy 回退；或场景要求的帧未完成 |
| 性能预算 | `make bench-gate` | 返回回归/配置错误；繁忙机器的 INCONCLUSIVE 不算回归 |
| 产品静态渲染 | `make product-smoke` | 像素、结构或命令门禁失败 |
| Chrome 生产一致性 | `zeroweb-browser-chrome-parity` 的 production evidence | 状态、事件、几何、像素或必需产物失败 |

Chrome parity 默认使用仓库已有的最小代表场景；若变更触及表单、输入、DPR、字体、布局、绘制或 compositor，则选择覆盖该模块的场景。不得用 CPU/headless 截图替代生产窗口 GPU 证据。

## 2. 内存测量契约

1. 使用 release 产物，固定页面、viewport、DPR、字体环境和 GPU 配置。
2. 从启动前开始监控，只纳入本次 browser 的后代进程；不得按进程名汇总其他会话的实例。
3. 至少记录 renderer 和 compositor 各自的：稳定首帧后驻留内存、10 秒空闲后的驻留内存、场景全程峰值。
4. Linux 优先读取 `/proc/<pid>/status` 的 `VmRSS`/`VmHWM`；Windows 使用 `Get-Process` 的 `WorkingSet64`/`PeakWorkingSet64`。报告必须写明平台和指标语义，不混用不同口径作相对比较。
5. 同机成功报告保存在 `.rally/zeroweb-regression-guard/baseline-<platform>.json`，只允许成功巡检自动收紧或补齐缺失项，不允许自动放宽。

默认绝对预算：

| 指标 | 预算 |
|---|---:|
| renderer 单进程峰值 | ≤ 192 MiB |
| compositor 单进程峰值 | ≤ 192 MiB |
| renderer + compositor 稳定空闲总量 | ≤ 256 MiB |

已有同平台、同口径、同场景基线时，还必须满足：

```text
measured <= max(baseline * 1.50, baseline + 64 MiB)
```
绝对预算与相对预算任一失败都算回归。首次运行没有历史基线时，仅使用绝对预算；不得拿另一台机器或另一种内存口径的结果冒充基线。

## 3. compositor 日志判定

以下日志在成功出帧后出现时属于硬失败：

- `帧响应超时` / `frame response timeout`
- `Compositor disconnected`
- `switched all renderers to legacy`

单独的 `AssignProcessToJobObject failed (likely already assigned or permission denied)` 不足以证明 compositor 断连，必须结合进程存活、IPC 和帧日志判断。不得通过延长 watchdog、删除日志或无条件 legacy fallback 修复假象。

## 4. 复现与噪声控制

- 性能或内存失败在相同环境重跑一次；两次均失败才进入自动修复。700 MiB 级等超过绝对预算 2 倍的失败可直接认定，无需重复消耗资源。
- GUI/GPU、Chrome、字体、依赖或显示服务缺失时判 `BLOCKED` 并飞书通知，不修改代码。
- `bench-gate` 的 busy/suspect/INCONCLUSIVE 按性能预算文档处理，等待机器空闲后再测，禁止放宽基线。
- 自动修复时间上限为单次 Rally job 的 timeout；到期前保留证据并发送卡点通知。

## 5. 报告最小字段

每次完整运行的报告至少包含：

```json
{
  "git_sha": "...",
  "platform": "...",
  "scenario": "...",
  "memory_metric": "...",
  "renderer_peak_mib": 0,
  "compositor_peak_mib": 0,
  "idle_total_mib": 0,
  "compositor_disconnects": 0,
  "bench_gate": "PASS|FAIL|INCONCLUSIVE",
  "product_smoke": "PASS|FAIL|BLOCKED",
  "chrome_parity": "PASS|FAIL|BLOCKED",
  "verdict": "PASS|REGRESSION|BLOCKED"
}
```
