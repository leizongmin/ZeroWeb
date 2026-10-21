# M3：S1d threaded 腿判定批——S1a 真线程执行面验证 + worker bootstrap `self` 缺口修复

日期：2026-10-22 ｜ 里程碑：M3（S1d 判定批）｜ 前置：S1a 执行面（推进流
c0c026e74）、canvas registry 修复（[2026-10-22-m3-canvas-registry-fix](../2026-10-22-m3-canvas-registry-fix/README.md)）

## 判定

**S1a 执行面验证成立（THREADED_PROBE_PARTIAL，4/5 判定门 TRUE；transfer 门 = S1b
入口条件，预期 FALSE）**。附带修复 worker bootstrap 缺 `self` 全局的规范缺口
（script-sandbox 一行 + 回归测试）。

### bootstrap `self` 缺口（修前 → 修后）

`ZW_THREADED_WORKERS=1` 首跑（[comp04a-probe-before-self-fix.json](comp04a-probe-before-self-fix.json)、[comp04a-s2-self-error-before-fix.png](comp04a-s2-self-error-before-fix.png)）：
mode=threaded 探测正确、宿主桥 dispatch 往返成立（error worker 事件回流），
但主 worker 脚本首行 `self.onmessage` 即抛
`ReferenceError: self is not defined at <anonymous>:12:1`，beats=0——WORKER_BOOTSTRAP_JS
只定义裸 `onmessage`/`postMessage`/定时器/`close`，无 `self`。而 WorkerGlobalScope.self
返回 worker 全局对象自身（https://html.spec.whatwg.org/multipage/workers.html#dom-workerglobalscope-self），
`self.onmessage`/`self.postMessage` 是规范 worker 脚本通用访问面，WPT worker 语料
全覆盖，缺口在 script-sandbox 面不在 fixture。

修复：`WORKER_BOOTSTRAP_JS` 增 `var self = globalThis;`（带规范链接；V8/QuickJS
共享 bootstrap，两引擎均有 globalThis；bootstrap `var` 即全局属性，故
`self.onmessage` 赋值与 `_dispatchMessage` 读 `onmessage` 同一绑定，shim prologue
对 `postMessage` 的重包裝经全局属性同样对 `self.postMessage` 可见）。
回归测试 `test_worker_self_global_alias`（worker.rs，self.onmessage/self.postMessage
往返）。script-sandbox 为推进流 S1a 面——S1a 批已收口（其 master.md 条目记完成），
本批按碰头规则在 master.md 记一行说明后取该一行修复，未动 S1a 其他面。

### threaded 腿判定（修复后，[comp04a-probe.json](comp04a-probe.json)、[comp04a-s2-threaded-strips.png](comp04a-s2-threaded-strips.png)）

| 判定门 | 结果 | 证据 |
|---|---|---|
| wt_liveness（≤200ms） | **TRUE：wt 跨忙窗间隔 50ms** | 忙窗 3409→4309ms（900ms，页面侧 busy_ms=901）；at 侧跨忙窗 943ms（主线程同载停摆），worker 侧 wt 仅断 50ms——**线程隔离语义成立（DC-4 (a) 核心）** |
| transfer_ctx | FALSE（= S1b 入口） | `transferControlToOffscreen()` 成功，但 shim `postMessage` 传输列表校验抛 "Value at index 0 does not have a transferable type"——OffscreenCanvas 结构化克隆未实现，init 未达 worker，ctxOk=null |
| term_silent | TRUE | terminate @6908ms 后 `beatsAfterTerm=0`（beats 总数 129 冻结） |
| err_reported | TRUE | error worker `Error: dc4-expected at <anonymous>:2:7` 经宿主桥回流页面 onerror |
| control | TRUE | s3 正控条带换色 `ctrl=on` |

beats 0→129：worker 顶部 `setInterval(50ms)` 经 Rust 定时器泵驱动，beat 携带
worker 侧 `Date.now()`（wt）与页面收到时刻（at）双钟；`lastBeatN=0` 为预期——
beat n 字段承载绘制进度，transfer 不可达则无 ctx 恒 0，S1b 后启用。

## 结论与边界

- DC-4 判定面推进：**「worker 线程不被主线程忙窗阻塞」在真线程执行面成立**
  （50ms vs 943ms 同窗对照）；「terminate 即冻结」「错误回流」同批成立。
- transfer 门与呈现像素连续性（dc4-observe on 腿区域动画 + terminate 终末冻结段）
  均系于 S1b canvas 桥：transfer 不可达则 worker 无绘制内容，observe on 腿顺延至
  S1b 批跑（本批不产空区域证据）。
- 探针 verdict 语义：THREADED_PROBE_PARTIAL 即 S1a 范围的通过态——S1b 落地后
  transfer_ctx 门转 TRUE，verdict 升 THREADED_PROBE_PASS。

## 验证

| 门 | 结果 |
|---|---|
| cargo test -p zero-script-sandbox | 343 passed（含新增 self 别名回归） |
| cargo fmt / clippy -D warnings（script-sandbox） | 干净 |
| make test（guard 包裹全量） | EXIT=0 |
| testharness-workers 全量（part61 语料） | 693 Pass / 12 Fail——12 项名称级逐一比对均为 workers-compat 归档 slice3 基线（29F，本批前状态）既有失败，**零新增**；`unexpected-self-properties.worker.js`（self 属性存在性簇）全 Pass——bootstrap 增行对 runner 同 realm 路径零影响实证 |
| comp04a-probe flag on | THREADED_PROBE_PARTIAL（本 README 表） |
| comp04a-probe flag 关（回归，[comp04a-probe-flag-off-regress.json](comp04a-probe-flag-off-regress.json)） | SHIM_BASELINE_PASS；wt 跨忙窗 936ms（同 realm 基线语义维持）、draws_main=372——bootstrap 增行对 shim 腿零影响 |
