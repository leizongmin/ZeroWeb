# M3：S1b worker canvas 桥——transfer 到真线程 worker + 排空协议落 worker 本地 registry

日期：2026-10-22 ｜ 里程碑：M3（S1b）｜ 前置：S1a 执行面（c0c026e74）、S1d threaded 腿判定
（[2026-10-22-m3-s1d-threaded-leg](../2026-10-22-m3-s1d-threaded-leg/README.md)，transfer 门 FALSE = 本批入口）
｜ RFC：[m3-slice-worker-canvas.md](../../m3-slice-worker-canvas.md) §2.2

## 判定

**comp04a probe flag on 腿 THREADED_PROBE_PASS（5/5 判定门 TRUE，S1d 的 PARTIAL 升
PASS）**；flag 关对照 SHIM_BASELINE_PASS 保持（shim 腿零行为漂移）。

[threaded 腿](comp04a-probe-threaded.json)：`transfer_ctx=TRUE`（transferControlToOffscreen
→ postMessage transfer → worker 侧 ctx ok）、`wt_gap_spanning_load_ms=51`（忙窗
3400→4300ms 内 worker 侧时钟间隔，线程隔离维持）、`lastBeatN=387`（S1d 为 0——worker
drawFrame 真实推进，`setInterval(drawFrame, 16)` 逐拍绘制进 worker 本地 registry）、
`term_silent`/`err_reported`/`control` 维持 TRUE；s2 采样条带见
[comp04a-s2-threaded.png](comp04a-s2-threaded.png)（strip-worker=ctx ok、strip-transfer=sent）。
[shim 对照腿](comp04a-probe-shim.json)：wt 跨忙窗 938ms（基线 936–949ms 带内）、
transfer probe-only、正控 OK。

## 实现（RFC §2.2 四触点，零新 IPC）

**排空协议**（worker isolate 无事件循环/宿主回调，操作经 bootstrap 队列在 worker 线程
命令循环间隙由 Rust 读回）：

- script-sandbox `WORKER_BOOTSTRAP_JS` 增 canvas 面（`_workerCanvasOps`/`_workerCanvasSeq`）：
  `_zwWorkerMakeCtx2d(cid)` 2D ctx 代理（写路径记 op，样式 setter 记 set-* op，方法名与
  `canvas_context_op` 1:1；读路径排空模型不可同步返回，FIXME(S1c) 标注）；`OffscreenCanvas`
  构造 + `getContext('2d')` 记 `["newCtx", cid, w, h]`（构造与复活路径都在首个 getContext
  记——复活路径 presetCid 先置，`if (!cid)` 守卫不能承担已分配判定，`ctx` 变量才是）；
  `_zwWorkerReviveXfer` 深走入站载荷把 `__zwXferCanvas` 标记复活为本地 OffscreenCanvas 并记
  `["bind", cid, key]`（宿主绑定表 surface 键）。`WorkerRuntime::new_with_canvas_drain`
  （v8/quickjs 双引擎同构）排空回调挂在定时器排空同点（init/timer-fire/Execute/PostMessage
  四位点）；`None` 读回即弃（service worker 等无桥路径不累积）。
- renderer `worker_host.rs`：`make_canvas_drain()` 每 worker 一份（worker 线程本地
  `CanvasRegistry`——像素不出 worker，无 paint 路径，同 realm 幻影注册语义修除）；
  `apply_worker_canvas_ops` 求值 `newCtx`/`op`/`resize`/`bind` 条目——`newCtx` 经
  `canvas_context_op(reg,"0","getContext2d")` 分配真 id 建 prov（临时 cid→真 id）映射，
  `op`/`resize` translate 后走 `canvas_context_op` 纯栅格核**零改动**复用，`bind` 记宿主
  绑定表（surface 键 → 临时 cid，S1c patch 呈现消费；terminate/Closed 随摘）。
- engine shim：`transferControlToOffscreen` flag on（`typeof __zw_worker_host_op` 探测）
  返回占位 OffscreenCanvas + `_zwXferKey` surface 键，getContext/transferToImageBitmap 抛
  InvalidStateError（HTML spec §transferred-canvas，control 已移交 worker），二次 transfer
  抛 TypeError；flag off 走 R3313 共享 handle 路径逐字节不变。`Worker.prototype.postMessage`
  宿主分支：载荷扫原 `message`（克隆体非同一对象，identity 对不上 transferList）校验
  OffscreenCanvas 列内 + 键在（否则 DataCloneError——spec OffscreenCanvas transferable 非
  Serializable），replacer 把列内项序列化为 `__zwXferCanvas` 标记；`_zwValidateTransferList`
  增已 detach 再转移拒绝（校验先于本调用 detach 步）。structured-clone OffscreenCanvas 克隆
  体随迁 `_zwXferKey`（part02——宿主分支 replacer 读克隆体键，普通画布无键不受影响）。

已知边界（记档不阻塞）：宿主分支 OffscreenCanvas 校验抛点晚于 structuredClone/transfer
消费（spec 校验应先于全部副作用，FIXME 标注）；worker ctx 读路径（getImageData 等同步
返回）排空模型暂缺（FIXME(S1c)）。

## 测试

- script-sandbox（v8 + quickjs 双腿）：构造路径排空（newCtx/setFillStyle/fillRect）、
  transfer 复活绑定（bind/newCtx/fillRect 同复活 cid——prov 映射闭环）、None drain 读回
  即弃。测试序为「先等 drain 回调再 recv/terminate」：init 排空点晚于消息事件入队，先
  terminate 会以 terminate_execution 打断排空序列丢批（初版误诊为产品缺陷，实为测试竞态）。
- engine part65：flag 关 R3313 平面对象不变；flag on 占位语义（真类型/键/尺寸、
  InvalidStateError、二次 transfer TypeError）；postMessage 宿主分支标记序列化 + 未列/
  已 detach DataCloneError。
- renderer worker_host：排空协议求值（newCtx→prov 映射、fillRect 像素落本地 registry 经
  getImageData 读回 `1:1;255,0,0,255`、bind 记绑定表）；未知 cid/畸形 JSON 防御面。

## 验证

| 门 | 结果 |
|---|---|
| comp04a probe flag on | THREADED_PROBE_PASS（5/5 门，见上） |
| comp04a probe flag off | SHIM_BASELINE_PASS（wt 938ms 基线带内） |
| cargo test（script-sandbox v8/quickjs 双 feature 组合、engine、renderer 相关面） | 绿（分项见上） |
| make test 全量 | EXIT=0 |
| testharness-workers 全量 | 693P/12F——与 S1d 批完全同数，12 项失败文件均为归档 slice3 基线既有失败，零新增 |
| cargo fmt / clippy -D warnings | 干净（v8 默认 + quickjs-only 两组合——初版 collapsible_if 仅 quickjs 组合暴露，已修） |
