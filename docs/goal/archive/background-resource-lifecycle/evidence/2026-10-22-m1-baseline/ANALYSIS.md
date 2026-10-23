# M1 四类场景资源基线（BG-01～04）

**运行身份**：HEAD `2c7175115`（工作区含本轮脚本/fixture/文档改动）；zero-browser sha256 `8581b795…`、zero-compositor `f731c12e…`（2026-10-22 15:31 `make browser-build` 产物，24G systemd scope 内编译）。
**环境**：Linux x86_64（6.18 xanmod）、 liberation/dejavu/noto 系统字体；headless 腿 = CDP 驱动、无窗口、viewport 800x600；GUI 腿 = X display :20、真实窗口 1920x893、GPU compositor（lavapipe 软件栈）。CPU/RSS 均为生产进程 `/proc` 真实采样（sampler 250ms；cpu_cores = jiffies 差分/时长，100 jiffies = 1 CPU·s）。
**隔离**：每腿独立 `XDG_CONFIG_HOME` profile、独立端口 fixture server + CDP、timeout 墙钟、外层 24G systemd scope；采样器只统计 timeout 包裹树的后代进程。
**协议**：`scripts/background-resource-observe.sh`（单腿编排）+ `background-resource-probe.mjs`（CDP 场景驱动）+ `background-resource-sampler.py` + `background-resource-fixture-server.py`；fixture 在 `tests/fixtures/background-resource/`。重跑命令见各腿 `run-config.txt`。

## 每腿汇总（renderer 主实例；完整数据见 summary.json）

| 腿 | wall | renderer CPU (cores) | renderer RSS 首→尾 KiB | 帧数 | 页面采样 |
|---|---|---|---|---|---|
| bg01-foreground | 30s | **1.46** | 45,608→242,380 | – | ~20 万 iter/s |
| bg01-background | 30s | **1.54** | 33,104→241,796 | – | ~20.5 万 iter/s |
| bg01-interval | 20s | 0.09 | 38,216→326,092 | – | ~1,000 iter/s（1kHz） |
| bg02-anim（headless） | 30s | 0.009 | 40,356→124,772 | –（无 compositor） | `__zw_raf`=2 恒定 |
| bg02-rafstub | 15s | 0.011 | 61,852→120,060 | – | `__zw_raf`=513 后停 |
| bg02-static（headless） | 30s | 0.008 | 44,456→112,408 | – | – |
| bg02-anim-gui | 30s | 0.004 | 108,012→114,224 | **3** | – |
| bg02-anim-compgui（ZW_COMPOSITOR_ANIMATIONS=1） | 30s | 0.007 | 109,032→115,432 | **2** | – |
| bg02-onscreen-compgui（视口内动画对照） | 30s | 0.005 | 108,136→114,344 | **2** | – |
| bg02-static-gui | 30s | 0.004 | 108,944→109,072 | **2** | – |
| bg03-nav（30 次导航） | ~45s | 0.076 | 44,056→**267,756** | – | nav_ok 30/30 |
| bg04-cycle（20 次 close/reopen） | 40s | 0.025 | 50,688→**213,980** | – | destroyed 20/20 |

browser 进程各腿 RSS 稳定（~34–48MB；GUI 腿含窗口栈升至 ~250MB 属正常）；GUI 腿有独立 zero-compositor 进程。

## 按 DC 结论

### DC-1 / BG-01 嵌套 timer 与前后台——**基线成立，无钳制、无后台策略**

- **嵌套零延迟 timer 无最小延迟钳制**：`setTimeout(loop, 0)` 嵌套循环实测 ~20 万迭代/秒（若 spec 的 ≥5 层后 ≥4ms 钳制生效应 ≤250 次/s）。spec 链：`html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timer-initialisation-steps`；源码静态核对一致（主线程 timer 路径无 nesting level，worker 路径仅 1ms 下限 + FIXME 标注）。
- **后台 tab 无 timer 节流**：同页面被 static tab 抢占 active 后，迭代速率（20.0 vs 20.5 万/s）与 renderer CPU（1.46 vs 1.54 cores）无差异。结构性原因（源码核对）：`visibilityState` 恒 `'visible'`、`visibilitychange` 永不触发、tab 切换不给 renderer 发任何消息；现存的"后台节流"仅是 browser 侧 poll-order 排空（后台 tab IPC 每 ~5 tick 排空一次 + 4ms poll 预算），不影响页面工作。
- 恢复正确性（GUI 前后台往返）：**NOT_RUN**——headless 无可见性概念，GUI 前后台切换本腿未覆盖；在可见性消息机制存在之前无恢复语义可验证。
- 附带观测：timer 高频腿 renderer RSS 增长 ~200MB（30s），interval 腿 20s 增至 326MB——高频回调伴生的引擎侧内存增长，机制待 M2 归因。

### DC-2 / BG-02 视口外动画/静止呈现——**静止免呈现成立；动画呈现链未激活（INCONCLUSIVE）**

- **静止页面按需渲染成立**：bg02-static 全腿 renderer CPU ≤0.008 cores，GUI 腿 30s 仅 2 帧（首帧 + 完成帧）。
- **CSS 动画不驱动持续呈现（视口内外一致）**：offscreen 动画 GUI 腿 3 帧、onscreen 对照腿 2 帧、`ZW_COMPOSITOR_ANIMATIONS=1` 腿 2 帧，且日志无 `compositor_animations_active`——动画导出/keep-alive 链在默认与 flag 开启形态下均未激活。资源面是"免呈现"，但 DC-2 的"合法工作不误停"维度（动画应持续时应持续）**INCONCLUSIVE**——呈现链激活条件待 M3 排查（`process_backend.rs` 的 keep-alive 依赖 `compositor_animations_active`，激活前置未满足）。
- **rAF 帧驱动依赖渲染 tick**：`ZW_RAF_FRAME_DRIVEN=1` 下 `__zw_raf` 卡在 2——渲染停止后 rAF 不再 tick，无自我驱动环路（与 GUI 帧数一致）。rAF stub 模式（默认 env）513 次预算上限后停。
- 音视频合法工作不误停：**NOT_RUN**（本组无媒体 fixture）。
- headless 腿无 compositor 进程（renderer 直接 publish），frame dump 为 GUI 腿专属——后续腿设计已区分（`*_gui` 场景名）。

### DC-3 / BG-03 重复导航与停滞资源——**导航功能正常；连接与内存随导航积累**

- 30 次 source/target 交替导航全部成功（nav_ok 30/30，document.title 就绪），文档替换功能面正常。
- **挂起 fetch 的 HTTP 连接不随导航取消**：fixture server 观测 `stall_active` 1→6（后段恒 6），`stall_closed_by_peer` 观测窗内为 0。M2 归因（调用链核对）修正解读：连接滞留至 **reqwest client 级 30s 总超时**（`crates/net/src/client.rs:60`，含 body 阶段；`ResourceRequest::get` 默认 timeout_secs=30），到期后 hyper 丢弃半读连接；6 条封顶是 per-origin 并发帽（`resource_policy.rs:18`，DEFAULT_MAX_CONNECTIONS_PER_ORIGIN=6）所致，首批连接的超时点恰在 45s 观测窗尾部，故 peer 关闭未及观测。真实缺陷是**无取消路径**：`fetch_scheduler` spawn 的 task 无句柄留存、无 CancellationToken，导航时 `inflight_fetches.clear()` 只 drop 应答端点——被放弃的 fetch 继续占用 FetchBridge launch 线程名额（gate 64，`fetch_bridge.rs:102`）与 per-origin 槽位直至超时，队头阻塞同源后续请求；页面 fetch() 路径全程无登记（`InflightIpcFetches` 只覆盖 IPC 子资源路径）。
- **renderer RSS 随导航近线性增长**：44MB → 268MB（四分位 44/165/198/216/268），30 次导航 +224MB。候选来源（待 M2 归因）：① 超长 timer（2^31-1ms）heap entry 滞留——TimerBridge 协调线程为 per-worker 单例，导航不清队列，entry 滞留至 expiry；② stall fetch 上下文与连接缓冲；③ 其他文档替换残留。**本基线不判定根因。**
- browser 进程 RSS 稳定（~36MB）——增长集中于 renderer。

### DC-4 / BG-04 关闭/重开——**生命周期回执完整；renderer 进程复用且内存不随 close 回落**

- Target 生命周期回执完整：20 次 createTarget→closeTarget 全部成功，`Target.targetDestroyed` 事件 20/20、0 错误、结束时仅剩宿主 tab。
- **renderer 进程跨 tab 复用**：全程单一 renderer pid 服务 20 次 tab 开关（进程不随 close 退出），RSS 50.7MB → 214MB（+163MB）**不随 tab 关闭回落**。候选来源：跨 tab 文档残留（同 BG-03 的释放路径缺口）。进程复用本身是设计选择（省启动成本），内存不回收是缺口信号。
- GUI 前台输入/动画恢复：**NOT_RUN**（本腿为 headless CDP 驱动，GUI 交互恢复腿未覆盖）。

### DC-5 固定协议——**协议已建立，可配对复测**

run ID/profile/PID/viewport/墙钟/采样间隔均固定并落 `run-config.txt` 与 samples.jsonl；四类场景驱动与采样器均为仓库脚本，无外部状态。长期配对阈值与复测节奏待 M4（本基线为首轮 N=1，不作阈值依据）。

## 静态事实（源码核对，支撑上述解读）

- `visibilityState`/`hasFocus` 硬编码（`js_dom_shim/part06.js:5229-5237`，R2824）；windowless 文档恒 hidden（part03.js）。
- tab 激活切换无 renderer 通知：`send_to_renderer` 全部消息类型核对（process_backend.rs），无 visibility/activation 消息。
- 后台节流 = browser 侧 poll-order（`tab_manager.rs:381-389`，每 5 tick 排空后台 + 4ms 预算）。
- TimerBridge 协调线程 min-heap 为 per-worker 单例，导航不清队列（`timer_bridge.rs`、`js_worker.rs:1924-1985` reset 不涉 timer 队列）。
- 嵌套 timer 4ms 钳制：主线程路径 NOT_FOUND；worker 路径自述未实现（`script-sandbox/src/worker.rs:26-32` FIXME）。

## M2 候选增长源（按证据强度排序）

1. **挂起 fetch 无取消路径**（BG-03，已归因）：`fetch_scheduler.rs:277-341` spawn 的 task 无句柄、无 CancellationToken；修复切片 = 调度器任务句柄表 + `cancel_key`（~30 行）→ FetchBridge 登记 `id → 取消句柄` + `ResetDocumentState` 臂触发（页面 fetch 覆盖）→ runtime 三处 `clear()` 扩展（IPC 子资源覆盖）。基于 reply channel 断开检测不可行（调度器持有 Sender 克隆，channel 永不 Disconnected）。
2. **重复导航 renderer RSS 线性增长 +224MB**（BG-03）与 **tab 开关 +163MB 不回落**（BG-04）——owner：page-runtime/renderer 文档替换与 tab 生命周期。超长 timer heap entry 滞留已核对为真实但量级小（entry = expiry+seq+id 字符串，滞留至 expiry，回调被 `__zw_pending` 缺失语义兜底 no-op；无按代际清队机制），非 RSS 增长主因，待进一步归因。
3. **高频 timer 页 RSS 增长 ~200-326MB**（BG-01）——owner：js_worker/引擎堆，伴生现象待归因。

## NOT_RUN 清单

- BG-01 GUI 前后台切换恢复；BG-02 音视频合法工作；BG-04 GUI 前台输入/动画恢复；DC-5 长期配对复测与阈值（M4）。
- GUI 腿 frame dump 的 .bin 已在提交前清理（每帧 6.8MB），manifest.jsonl 保留帧计数与元数据；browser.log/http.log 按仓库 `*.log` 忽略约定不入库，复现以 run-config.txt 为准。
