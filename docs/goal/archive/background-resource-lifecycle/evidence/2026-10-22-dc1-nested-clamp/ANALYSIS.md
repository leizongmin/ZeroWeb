# DC-1：嵌套 timer 4ms 钳制修复 + 生产复测

**日期**：2026-10-22 / **构建**：`make browser-build`（652ccf97f + 本轮钳制） / **环境**：headless CDP，隔离 profile，probe 5s 采样，systemd 24G scope + 墙钟守卫

## 判定摘要

1. **嵌套零延迟 timer 最小延迟钳制落地并生产验证**。修后 BG-01 嵌套链 ~237 iter/s（5 窗稳定 231–237/s，即 spec 4ms 钳制值），M1 修前 ~20 万 iter/s——**~840× 降速**；renderer CPU 整体均值 0.05 cores（修前同场景 1.46 cores，~29× 降）。
2. **单测钉住 spec 边界**：第 7 代（nesting 6 > 5）首个被钳、顶层与前 5 代不受影响；0ms interval 前 5 tick 续 0ms、第 6 tick 起 ≥4ms；≥4ms 周期不受影响。

## 修复面（本提交）

`crates/engine/src/js_dom_shim/part01.js` setTimeout/setInterval shim：

- 新增 `_zwTimerNesting` 计数 + `_zwClampDelay(ms)`（nesting > 5 且 ms < 4 → 4）。
- nesting level 按 spec 是**任务链属性**：创建时闭包携带 level，fire 时计数绝对赋值为 level+1、结束恢复现场——host 每到期一次独立 execute，计数不随代际漂移；`__zw_fire_due_timers` 多 fire 循环（WPT runner）以 entry 时现场恢复互不污染。
- setInterval re-arm 在计数恢复前执行（spec 重复分支在 task 内重跑 initialisation，同 nesting 口径），并逐代 +1 传入下个 arm。
- 覆盖面：主页面 shim 唯一 setTimeout 定义（iframe realm 经 `globalThis.setTimeout` 转发自动继承；`__zw_test_setTimeout` runner stub 收到钳后 delay 自然生效）。

## 生产复测腿：bg01-foreground（修后）

fixture `bg01-nested-timer.html`（`setTimeout(loop, 0)` 无限链），wall=30s：

- iters（probe 5 窗）：1263 → 2451 → 3636 → 4818 → 6004，逐窗速率 **237.2 / 236.6 / 231.4 / 236.8 /s**。
- renderer CPU：整体均值 **0.05 cores**（尾窗 0.4 cores 为腿收尾采样含导航余波；首窗 0.20 含加载）。
- renderer RSS（MiB，10 分位 + 末值，n=124）：`[64.3, 139.0, 165.4, 199.5, 230.6, 257.2, 281.8, 320.7, 321.3, 321.8, 327.4]`，peak 327.4——预热爬坡后有界平台、无向上漂移（绝对值高于 M1 修前腿 243 MiB peak，跨构建环境差异，非增长形态）。

## 单测（zero-engine lib，与修复同提交）

- `test_timer_nesting_clamp_zero_delay_chain_dc1`：记录式 host stub 逐代驱动真实 `__zwResolveCallback` 时序——第 1 代顶层与第 2–6 代（nesting 1–5）传 host delay=0，第 7/8 代（nesting 6/7）钳到 4。
- `test_timer_nesting_clamp_interval_rearm_dc1`：0ms interval 创建不钳，第 1–5 tick re-arm 0ms、第 6/7 tick 钳到 4；10ms 对照 interval 全程原样。

## 已知缺口（不阻塞本判定）

- **worker 路径**（`crates/script-sandbox/src/worker.rs` `worker_timer_delay` 1ms floor + FIXME）与 service worker 域 shim 未实现 nesting 钳制——独立面，单独切片。
- **promise 续体欠钳**：计数在 wrapper 同步段结束即恢复，fn 的 microtask 续体内创建的零延迟 timer 不累计嵌套（与 Chromium task 级 tracking 相比）；对齐须 host 侧 task-end 钩子，已 FIXME 标注。
- `dom_bridge` 同步 timer stub（立即调用无延迟）与 requestIdleCallback 不在钳制域（spec 亦然）。

## 验证

- `zero-engine` lib 全量 3407 通过（含 P20/D-N1 既有 timer 行为回归）。
- `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings` 见控制面验证记录。
- 全量 `make test` 结果见控制面。
