# M2-S1 挂起 fetch 取消路径——修复与生产复测

**运行身份**：HEAD `ac86fa8ea`（修复提交）；zero-browser sha256 `364b0ebf…`、zero-compositor `f731c12e…`（2026-10-22 `make browser-build` 产物，24G systemd scope 内编译）。
**环境**：与 M1 同机同协议（Linux x86_64 headless、viewport 800x600、sampler 250ms、一次性 profile、独立端口）。
**修复提交**：`fix(net): 挂起 fetch 随文档卸载取消——调度器取消路径 + 页面 fetch 登记 + IPC 子资源 cancel_tab 扩展`。

## 修复面（对应 M1 基线归因）

M1 归因（见 [M1 基线](../2026-10-22-m1-baseline/ANALYSIS.md) §DC-3）：`fetch_scheduler` spawn 的任务无句柄留存，导航只 drop 应答端点，连接滞留至 client 级 30s 超时。本切片在三个层面补取消路径：

1. **zero-net 调度器**（`crates/net/src/fetch_scheduler.rs`）：`running_cancels` 登记（fragment 归一化 URL → [(身份键, Notify)]，仅 shared API）+ `cancel_by_url`——排队任务立即回错；在途任务经 `tokio::select!` 打断网络 await，连接随 future drop 关闭，槽位随收尾臂释放。`ResourceLoader::cancel_inflight_url` 为统一入口。
2. **renderer 页面 fetch**（`apps/renderer/src/js_worker.rs`）：`default_fetch_handler` 按 URL 登记在途计数（同 URL 并发共享），`ResetDocumentState` 臂 drain 后逐 URL 取消——覆盖 M1 实测泄漏的页面 `fetch()` 路径。
3. **browser IPC 子资源**（`apps/browser/src/fetch_proxy.rs`）：`cancel_tab` 扩展为按 URL 取消底层在途加载（导航经 `on_navigate` 必达）。设计位置与 M1 计划草案的差异：原拟 renderer 侧三处 `clear()` 扩展，实际落点在 browser `cancel_tab`——IPC 子资源加载发生在 browser 进程，renderer 侧 clear() 无法触达，`cancel_tab` 是既有导航钩子，零协议改动。已知边界：同 URL 跨 Tab 合并订阅一并取消（调度器合并语义无法区分发起方）；流式图片路径无句柄机制不在覆盖内。

观测资产配套：fixture server stall hold 循环改为轮询对端 FIN（原实现只在写响应时才可能计数 `stall_closed_by_peer`，取消生效与否在服务端不可直接观测）。

## 生产复测（bg03-nav 腿，同 M1 协议 30 次导航）

| 指标 | M1 基线 | M2-S1 复测 |
|---|---|---|
| nav_ok | 30/30 | 30/30（nav_ms ~530–640ms） |
| stall_total（run 末） | 6（per-origin 封顶） | **16**（= 1 初始 + 15 次 source 重访，逐次新建） |
| stall_closed_by_peer | 0（观测窗内） | **15**（每次导航离开 source 即 +1） |
| stall_active 轨迹 | 单调 1→6 恒 6 | **0↔1 振荡**（离开 source 取消归零，重访回 1） |

解读：取消路径在生产面生效——被放弃 fetch 的 TCP 连接随导航立即关闭（服务端逐次观测 EOF）；per-origin 帽不再被滞留连接占满（M1 中第 7 条起 fetch 客户端排队、服务端不再收到请求），30 次导航全程 fetch 均可新建连接。原始序列：`bg03-nav-recheck/probe.json` `nav_series[].stats`。

## renderer RSS——负结果（候选 #2 排除一项）

renderer RSS 37.7MB → 259MB（+216MiB，四分位 37/165/222/222/259 MiB；主实例 pid 单一）——与 M1 的 +224MB 基本不变。**连接取消修复后 RSS 线性增长依旧**，排除 M1 候选来源清单中的「② stall fetch 上下文与连接缓冲」为主因；增长主体仍指向文档替换残留面（候选 ①超长 timer entry 已核对量级小），归因继续（M2-S2）。

## 回归钉与验证

- 单测/端到端 ×4：调度器排队取消（回错 + 不发连接）、调度器在途取消（对端 EOF + per-origin 槽位释放）、renderer 端到端（LoadHtml→导航→对端 EOF，真实 handler + 真实 ResourceLoader）、browser `cancel_tab`（对端 EOF）。端到端两侧均以 15s 观测窗判别旧行为（短于 30s client 超时）；实现侧禁用取消调用实测 FAILED（15.00s 窗口超时）、恢复后毫秒级 PASS。
- `make guarded-test` 矩阵：workspace / renderer lib（含新 e2e）/ browser --tests（EXIT=0）/ quickjs 各腿（clippy + workspace + renderer lib 311P + browser 679P）全绿；fmt、clippy（v8 + quickjs）零告警。browser --bin 腿三次全量轮中各遇 1 例共租负载敏感瞬态红（`cross_renderer_transactions_wait_for_conflicting_scope`、`test_t8g_npo_iframe_scan_no_collection_leak` 墙钟断言、`service_worker_runtime::navigator_update/skip_waiting` 激活轮询）——同树首轮对应腿全绿 + 隔离复跑绿 + 与改动面零交集，同 rendering-compat R5085/R4971 判例归因（机器当时 load 34，兄弟流测试/bench 并行）。

## 残余缺口（后续切片）

- **M2-S2（下一步）**：renderer RSS 随导航 +216MB 增长主因归因（连接面已排除；沿 page-runtime/renderer 文档替换路径查）。
- 流式图片路径（`start_image_stream` spawn_network_task）无取消句柄，导航后任务自跑至完成（30s 超时兜底）；tab 关闭 `remove_tab` 不取消 pending fetch（M1 未测得 browser 侧增长，暂缓）。
- browser 单进程侧（`tab_js_worker::default_fetch_handler`）无在途登记——同病不同址，独立小切片。
- DC-3 验收面（RSS 无界增长）在 M2-S2 归因修复前维持 PARTIAL。
