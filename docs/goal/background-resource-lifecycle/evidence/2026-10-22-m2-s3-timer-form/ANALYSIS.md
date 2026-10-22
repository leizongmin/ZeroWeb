# M2-S3：BG-01 长程 RSS 形态判定 + DC-3 文档残留计数生产验证

**日期**：2026-10-22 / **构建**：`make browser-build`（ac86fa8ea + 本轮诊断计数） / **环境**：headless CDP，隔离 profile，sampler 250ms，systemd 24G scope + 墙钟守卫

## 判定摘要

1. **BG-01 高频 interval timer RSS 形态 = 爬坡 + 有界平台**（120s 腿）。M1 提出的第三个也是最后一个增长源候选（timer 堆/回调高频注册）收口：三个候选全部有界，M1「RSS 增长」读数均为预热爬坡。
2. **DC-3 文档存活计数落地并生产验证**。复位诊断行（31 条）证明：旧文档挂起 fetch 每次复位即放弃取消（15/15）、旧文档命令零残留（31/31 行 retained=0）、timer entry 是唯一滞留面——每次 source 访问 +1（0→15），受 24.8 天 expiry 上界，回调对象已随 context 重建释放。

## 腿 1：bg01-interval-120（BG-01 长程形态）

- run-config：`scenario=bg01-interval wall=120s`，fixture `bg01-interval-timer.html`（`setInterval(step,0)`，shim 1ms floor）。计数：~99,974 次 / 120s ≈ 833 iters/s 实效。
- renderer RSS（MiB，10 分位 + 末值，n=484）：
  `[46.7, 232.0, 218.4, 235.2, 236.5, 236.8, 237.6, 239.4, 242.1, 242.5, 242.6]`，peak 243.7。
- 判定：~12s 内完成预热爬坡（46.7→232），其余 ~108s 在 218–244 MiB 有界振荡，无向上漂移。与 M2-S2 导航腿（200–273 MiB 平台）同形态。

**结论**：高频 timer 注册/触发不构成无界增长。M1 三个增长源候选全部收口——①导航文档替换（S2 判有界平台）、②停滞 fetch 连接滞留（S1 修复）、③高频 timer（本轮判有界平台）。DC-3 第二句「RSS 不随替换次数无界增长」在 headless 生产面成立。

## 腿 2：bg03-nav-residue（残留计数生产验证）

新增诊断（本提交）：`TimerBridge::pending_stats()`（挂起条数 + 最近到期 ms，带单测）+ renderer `js_worker` ResetDocumentState 收口处 `document reset residue stats` 日志行（seq、timers_pending、timer_nearest_expiry_ms、fetch_abandoned、commands_retained）。

- run-config：`scenario=bg03-nav wall=45s` 默认 15 cycles = 30 次导航；30/30 `nav_ok`，nav_ms 526/592/654（min/中位/max）。fixture server 终态：stall_total=16、closed_by_peer=15、active=1（末位 source 页在测，属活态）。
- 残留序列（31 行 = 初始加载 + 30 次导航复位）：

| 行类型 | 次数 | timers_pending | fetch_abandoned | commands_retained |
|---|---|---|---|---|
| 初始加载（seq=0） | 1 | 0 | 0 | 0 |
| source→target 复位 | 15 | 不变 | **1** | 0 |
| target→source 复位 | 15 | **+1** | 0 | 0 |

- 解读：
  - **fetch 释放面**：source 页挂起的 stall fetch 在每次导航复位时登记放弃并取消（15/15 精确对应 source→target 复位），连接对端关闭已由 S1 腿 `stall_closed_by_peer` 独立佐证（本腿 15/15）。
  - **命令释放面**：31/31 行 `commands_retained=0`——复位收口后旧文档命令零滞留。
  - **timer 残留面**：唯一非零残留。source 页注册的 24.8 天 timer entry 随导航滞留协调线程队列（0→15，+1 精确对应每次 source 访问），`timer_nearest_expiry_ms` 全部在 2,147,466,277–2,147,483,121 ms（≈24.8 天，FIFO 队首=最早注册）。滞留的是 `(expiry, seq, id)` 堆 entry（每条几十字节量级）；**回调对象已释放**——context 重建丢弃 `__zw_pending`，到期 resolve 走 shim 缺失语义 no-op，无页面可见效应。

## DC-3 验收面更新

「已替换文档、回调与资源可释放；RSS 不随替换次数无界增长」三面齐证（headless 生产面）：连接面 = S1（61 次 create/cancel、对端 15/15 关闭）；RSS 面 = S2 + 本腿（爬坡+有界平台）；对象/回调面 = 本腿计数（fetch 15/15 放弃、命令 0 残留、timer entry 有界滞留且回调已释放）。

## 已知残留与未解项

- **timer entry 不随导航清除**（expiry 前滞留）：有界、量级小、无行为效应。可选卫生修复（复位时按代际清队）记录为延后项，不影响 DC-3 判定——优先级低于 DC-1 钳制（行为正确性）。
- 单进程 browser 内 `tab_js_worker` 路径无同款计数与 fetch 登记（同病不同址），随 DC-1/后续修复面一并处理。
- GUI 腿 NOT_RUN（与 M1/M2 各轮同口径缺口，归 M4 门禁轮补）。

## 验证

- 单测：`zero-engine` lib `timer_bridge::pending_stats_tests::pending_stats_counts_registered_timers_and_reports_nearest_expiry` 通过（0 注册 → (0,None)；注册 500ms + 24.8d → count=2 且 nearest ≤500ms）。
- `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings` 干净；全量 `make test` 结果见控制面碰撞记录。
