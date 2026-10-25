# C3——崩溃检测/重启退避与 SIGKILL 注入契约

**日期**：2026-10-25
**状态**：PASS（双路检测 + 状态机/退避重启 + SIGKILL 注入 3 新场景全绿 + M1 宿主面 release 零回归）
**goal**：[media-runtime-isolation](../../master.md)
**设计**：[m3-prep-design DESIGN §11](../2026-10-24-m3-prep-design/DESIGN.md)（偏差记 §10.9 偏差 13–17）

## 1. 版本与环境

| 项 | 值 |
|---|---|
| 基线 | C2 切片 2 落地后 main（`911032196`） |
| 平台 | Linux 6.18.33.2-microsoft-standard-WSL2，AMD Ryzen 7 9700X（16 线程），39GB RAM |
| 工具链 | rustc 1.99.1 / cargo 1.99.1，edition 2024 |
| 测量档 | 契约 debug 档（真实二进制）；M1 宿主面 release 档（与切片 2 基线同口径） |
| 隔离与守卫 | 编译无阈值；执行 `./target/test-guard --per-proc-mem 4 --total-mem 8`；clippy 走 `make guarded-clippy`（24G scope，-D warnings） |

## 2. 落地面

- **状态机**（`media_proxy.rs`）：`Idle → Healthy → Restarting{attempt, next_at}（退避 1s/2s/4s，n≤3，握手成功重置）→ Healthy | Failed`；初始 spawn/握手失败直达 Failed（§11.1 纪律不变）；显式 `shutdown`/facade `clear()` 回 Idle——reload 后全新 host（DC-3「重载恢复」）。
- **崩溃检测双路**（§11.2）：(a) send 失败/读半失效即时转 `connection_lost`；(b) 泵周期 `poll_health`（facade `tick_into` 每 tick 调用）`try_wait` 兜底。失效时弃置连接前取 exit status 诊断（`exit_reason` 同构 renderer 先例）+ stderr tail（16KiB 有界，后台读线程；host stderr 从 inherit 改 piped）入 tracing，exit status 入 `MediaError.message` 面。
- **崩溃通知 one-shot**（§11.3）：命令路径检测存 pending，由 `poll_health` 下发（命令 `Err` 不触发广播——播放级错误与传输失效同为 `Err` 面，广播只认崩溃通知）；`poll` `try_wait` 路径同步返回并清 pending。facade 收通知即全活跃镜像广播 `Error{HostInternal}`（playing 清除、回卷待办作废、epoch 不变）。
- **重启语义**（§11.4）：重启 = 全新 host，源全丢失，不自动重播/重注册（重喂前 play 如实报 unknown source）；命令在 Restarting 期间恒错误面（「重启退避中」）；Failed 恒「永久错误面」。`clear()` 从「连接保持」改 teardown（切片 2 语义让位，§11.4 既定）。
- **真值口**：facade `is_errored` → registry `is_errored`（pub）→ 桥回调 `__zw_video_is_errored` + `__zwVideoBridge.isErrored`；`media_host_pid`（client/facade/registry）为诊断与注入面。shim march Error face → 元素 error 事件归后续切片（DESIGN §10.9-16）。

## 3. 验证结果

### 3.1 SIGKILL 注入契约（真实二进制，debug 档，8 场景 16.2s 全绿）

[media-host-contract-c3.log](media-host-contract-c3.log)。既有 5 场景零回归；新增 3 场景（§11.5 五场景合并面）：

| 场景 | 断言面 | 结果 |
|---|---|---|
| 崩溃广播 + 隔离（§11.5-1+2） | SIGKILL → one-shot 通知携 `media-host exited: signal: 9 (SIGKILL)` → 消费后重启面（非 Failed）；renderer（测试进程）存活 | PASS |
| 重启路径（§11.5-3） | 退避 1s 到期重连出新 pid；重喂前 play 报 unknown source（不自动重播）；register+喂入后可播（SourceReady duration=2000ms，epoch 重单调） | PASS |
| 重启耗尽 + reload 恢复（§11.5-4+5） | 假二进制路径驱动 1s/2s/4s 三次重试全败 → Failed（真二进制恢复 + 超退避窗仍无重连 + play 恒 false——无内联回落）；`clear()` teardown → 全新 pid 重喂恢复（duration 2.0、帧注入 ImageCache） | PASS |

### 3.2 门禁

- workspace `make guarded-clippy`（-D warnings）过；
- feature-on 单测（media_proxy 9 + video_registry_host 8）17/17——状态机/spawn 失败短路/tail 有界/崩溃广播；
- feature-off registry 内联套件 24/24（`is_errored` 双态编译面）；
- 全量 `make guarded-test` 过（exit 0，83 套件）。

### 3.3 M1 宿主面 release 复跑（tick 路径新增 poll_health 后零回归）

[m1baseline-host-mode-release-c3.log](m1baseline-host-mode-release-c3.log)——C3 在 `tick_into` 头部新增 `poll_health`（Healthy 态每 tick 一次 `try_wait`）后与切片 2 基线同带：

| 场景 | 切片 2 基线 | C3 复跑 | 含义 |
|---|---|---|---|
| 纯视频 60s | wall 59983ms / cpu 690ms / cores 0.012 | wall 59979ms / cpu 760ms / cores 0.013 | 呈现 1440/1440 零丢帧 |
| A/V 双轨 60s | 同量级 | wall 60072ms / cpu 810ms / cores 0.013 | 同上 |
| 时钟保真 rate | 0.99827（offset p95 49ms） | 0.99827（offset p95 50ms） | 同带（A1 收带内） |
| seek 冷跳 15s | 19.1ms | 19.0ms（warm30 1.3ms / warm45 0.19ms） | 窄化路径并源，A4 宿主界过 |
| 套件 | 5/5（226.6s） | 5/5（226.9s） | 零回归 |

## 4. 限制与未决

- **页面 error 事件链**（shim march Error face → 元素 error 事件）未接——桥 `isErrored` 真值口已备，接线归后续切片（engine shim 属 zero-web 流共享面 + WPT event 族门禁）；DC-3 页面级终验归 C4/M4。
- **场景 4 到达路径**按「重启尝试连续 3 次失败」实施（§11.3 握手成功重置计数下 kill 不消耗预算），偏差记 DESIGN §10.9-13。
- 宿主进程 CPU 计量、shm 帧面维持切片 2 未决（归 C4 资源审计/帧路径切片）。
