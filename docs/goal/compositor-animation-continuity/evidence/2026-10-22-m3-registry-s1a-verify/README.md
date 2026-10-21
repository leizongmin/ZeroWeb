# M3 canvas registry 共享修复 + S1a 生产 runtime 验证批

日期：2026-10-22。承接 [S1d 脚手架](../2026-10-22-m3-s1d-scaffold/README.md)发现的 canvas registry 孤儿缺口（DC-4 前置阻塞）与 S1a 实现批（commit c0c026e74）。

## 本批修复四项

1. **canvas registry 孤儿缺口**：renderer runtime 预建共享 `Arc<Mutex<CanvasRegistry>>`，spawn js_worker 第 4 参传入（页面域 `__zw_canvas_op` 写入）+ `WebViewConfig.canvas_registry` 注入 webview（painter 读取）——两端口同一实例，应用显示路径页面绘制可见。`None`/缺省路径自建孤儿实例（仅测试/进程内路径），workers-compat 与 reftest/WPT 进程内路径行为不变。
2. **S1a 装填时序缺陷（实现修正）**：js_worker 线程启动即注册 `__zw_worker_host_op` 回调，而 renderer 构造器 `worker_host::install` 装填桥槽在其后（runtime.rs spawn :662 vs install :749）——注册期快照槽值恒空，回调不注册，`typeof __zw_worker_host_op` 恒 false，S1a 分流在生产 runtime 全路径（窗口 + headless 均走 run_desktop_role）从未生效（part64 测试进程内自装桥未覆盖此路径）。修复：callbacks.rs 注册只按 flag 门控，闭包每次调用动态读槽（`worker_bridge::worker_host_op()`，进程级 RwLock 槽），槽空回 `"err:host-unavailable"` → shim 回落 same-realm；flag 关仍零注册。
3. **worker bootstrap `self` 全局**：threaded worker 脚本在 script-sandbox realm eval 抛 `ReferenceError: self is not defined`（bootstrap 定义 postMessage/onmessage/定时器但无 `self`；same-realm 路径由 shim `with(self){}` 包裹故不触发）。`WORKER_BOOTSTRAP_JS` 补 `var self = globalThis`（V8/QuickJS 共享，`self.onmessage`/`self.postMessage` 与全局绑定同一）。
4. **fixture/probe 对齐**：comp04a 探测名 `__zw_worker_create`→`__zw_worker_host_op`（与 S1a 实际 API 对齐）；threaded 腿 transfer 失败降级纯消息 init（`transfer=bridge-pending`，S1b canvas 桥缺口语义显式化——`_zwValidateTransferList instanceof OffscreenCanvas` 拒收 R3313 普通 object 返回值，DataCloneError 属 S1b 范围）；spawn 落点可见性（`__zwSpawnInfo` hostId/dispatchDefined）；probe verdict 分层 S1a 门（spawn_hosted/dispatch_defined/wt_liveness/term_silent/err_reported/control）与 S1b 门（transfer_ctx），判定态 `THREADED_S1A_PASS_S1B_PENDING`。

## 验证证据

### OFF 腿（flag 关，registry 修复验证）

- probe `probe-off-leg-shim.json`：`SHIM_BASELINE_PASS`——wt 跨忙窗 959ms（comp04 基线 949ms 同量级）、beats 133、transfer=probe-only、正控 OK。**canvas_idle=ANIMATED**（修复前恒 FROZEN 空白）。
- dc4 observe + analyze（`dc4-analyze-off-leg.txt`，frame_dump 150 帧）：canvas 区域 `distinct_region_hashes=57`（修复前 1 恒空白）、region_change_events=140、忙窗冻结段 (3025,3540) 515ms + (3540,4012) 472ms ≈ 987ms（两段被忙窗起止转换帧切开；像素级闭合见下）。
- 像素级闭合（分析期人工核验）：bar x=(seq*4)%232 反推绘制序号连续；marker 文本区域哈希定位 load=busy→done 转换帧（seq30=load=busy、seq31=load=done），忙窗真实区间帧时间线滞后页内钟 ~1150ms（load 延迟），忙窗内冻结合计 ~1007ms——OFF 腿「忙窗 ≥800ms 冻结段」判据成立。analyze 冻结段窗口切分缺陷记档：结构化识别按帧时间线分组，与页内钟错位时拆两段，判定时合并读。

### THREADED 腿（flag 开，S1a 生产 runtime 首次验证）

- probe `probe-threaded-s1a.json`：`THREADED_S1A_PASS_S1B_PENDING`——spawn_hosted（hostId=1，renderer 宿主线程）、dispatch_defined、wt 跨忙窗 50ms（beat 周期；worker 线程不被页面忙窗阻塞——DC-4 核心断言成立）、beats 129（≈span/50 无缺拍）、term 后零 beat 回流（beatsAfterTerm=0，lastBeatN=0）、错误路径 `dc4-expected` 回流、正控 OK；transfer_ctx=false（S1b pending，预期）。
- `threaded-s0.png`：mode=threaded host=1 dispatch=y 条带。
- wt 时钟跳变观察（记档不阻塞）：`wtGapMax=853ms` 与 `wtGapSpanLoad=50ms` 并存，wtMaxPair 相邻条目同帧送达（at=4280）且全量拍数齐——非 worker 停拍，为 worker 侧 `Date.now()`（CLOCK_REALTIME）单次正向跳变（NTP step 怀疑；at/performance.now 单调钟不受影响）。S1d 证据批如需严格 wt 判据应换单调钟。

### 质量门禁

- `make test` 全量 EXIT=0（76 套件，0 failed）。
- `make reftest` EXIT=0（Layout 508/508 + Text 204/204——canvas 语料零回归）。
- `make product-smoke` PASS（struct-check 0 issues）。
- guarded clippy `-D warnings` 干净（script-sandbox/engine/webview/renderer）。

## 产物清单

| 文件 | 内容 |
|---|---|
| `probe-off-leg-shim.json` | flag 关对照 probe verdict（registry 修复后） |
| `probe-threaded-s1a.json` | flag 开 threaded probe verdict（S1a 门全绿） |
| `dc4-analyze-off-leg.txt` | off 腿帧序列结构化分析（57 distinct hashes + 冻结段） |
| `threaded-s0.png` / `off-s2.png` | 判定条带截图 |
