# M4-S6 — WritableStream spec 状态机重做（erroring/abort/写队列/close sentinel）

**日期**: 2026-09-29
**切片**: M4-S6（streams 底座第四片——writable 侧 spec §5.2/5.4/5.5 状态机整体重做）
**原始数据**: [fetch](2026-09-29-m4-s6-fetch.json) ·
[xhr](2026-09-29-m4-s6-xhr.json) · [url](2026-09-29-m4-s6-url.json) ·
[mimesniff](2026-09-29-m4-s6-mimesniff.json) ·
[streams](2026-09-29-m4-s6-streams.json) ·
[eventsource](2026-09-29-m4-s6-eventsource.json)

## 结果（六 corpus，对照 M4-S5）

| corpus | M4-S5 | M4-S6 | Δ pass |
|---|---|---|---|
| fetch | 1227/1745（70.3%） | 1227/1745（70.3%） | 0 |
| xhr | 240/451（53.2%） | 240/451（53.2%） | 0 |
| url | 4431/6065（73.1%） | 4431/6065（73.1%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 635/910（69.8%） | **774/971（79.7%）** | **+139** |
| eventsource | 2/32（6.2%） | 2/32（6.2%） | 0 |
| **合计** | **8433/11101（76.0%）** | **8572/11162（76.8%）** | **+139** |

streams 分母 910→971；**六 corpus 全量零 pass 回归**（逐腿 diff 零丢失）。
页级（对照 M4-S5）：writable aborting 22/61→**65/65 全绿** · close 5/8→**26/26 全绿** ·
write 7/13→12/13 · constructor 13/13 维持 · general 9/11→12/16 · piping
close-propagation-forward 1/2→29/30 · error-propagation-backward 29→29（+6 新录腿）·
abort 20/33→29/33 · multiple-propagation 6/9→9/9 · transform terminate/general/cancel
维持+。

## 修齐内容（part02.js / part10.rs）

### 1. WritableStream 状态机 spec 化（slot 对应 §5.2）

- 状态：writable / **erroring** / closed / errored + `[[queue]]`（close 哨兵）+
  `[[writeRequests]]` + `[[inFlightWriteRequest]]` + `[[closeRequest]]` +
  `[[inFlightCloseRequest]]` + `[[pendingAbortRequest]]` + `[[backpressure]]` +
  `[[storedError]]`。
- **写队列串行推进**（AdvanceQueueIfNeeded——sink.write 逐个调用，前一写稳定后续推——
  「large queue of writes processed completely」面）；**close 哨兵排队**（writer.close 不
  再立即关——in-flight 写排空后 ProcessClose）。
- erroring 状态机：StartErroring（ready 同步拒绝）/ FinishErroring（queued 写拒绝 +
  abort 请求收尾）/ FinishInFlightWrite(±Error)（**DequeueValue** + ClearAlgorithms 仅
  writable 态——erroring 保留 abort 算法）/ FinishInFlightClose(±Error)/
  DealWithRejection / RejectCloseAndClosedIfNeeded。
- **WritableStreamAbort**：同步 signal abort（`controller.signal`——AbortSignal 挂靠 +
  「the abort signal is signalled synchronously」4 腿）；pendingAbortRequest 与
  close-in-flight **共存**（close 成功 → FinishInFlightClose resolve abort 请求——
  「ignore the abort attempt」面；close 失败 → 以 close 错误 reject——「preferred
  rejection」面）；动作集顺序执行 + abort 拒绝优先；stream 公开 abort/close 的
  locked TypeError 面。

### 2. pipeTo spec 条件优先序（条件 1-4）

- 首查按序：① 源 errored（WritableStreamAbort 动作——erroring dest 以其 storedError
  反噬为 pipeTo 拒绝值）② dest errored ③ 源 closed（WriterCloseWithErrorPropagation
  ——dest closing/closed → resolved、errored → storedError、否则 writer.close()）
  ④ dest closed/closing/erroring（TypeError/storedError + 源 cancel）——
  multiple-propagation 全组合 + 「abort signal takes priority」族面。
- shutdown-with-action 时序：abort 动作集**先排空 in-flight 写**再执行
  （「abort should not be called while write is in-flight」面）+ shuttingDown 停读；
  dest error 中途传播（writer.closed 拒绝 → 源 cancel + 收尾）。
- signal null → TypeError（AbortSignal 非空型）。

### 3. writer 语义补全

- release 后 write/close/abort → TypeError（`[[stream]]` undefined 面）；desiredSize
  erroring → null；close-on-erroring 不就地拒（closeRequest 经 FinishErroring reject）；
  stream.close() 公开方法补齐（locked → TypeError）。

### 4. 存量测试随 spec 翻新

- part10 R2969 桥测 1 处：sink.abort 异步调用面（abort promise then 内读取——M4-S5
  已翻新 in-flight write resolve，本片翻新 abort 时序断言）。

## 门禁

- `make test` 全绿（EXIT=0）；`cargo clippy --workspace --all-targets -- -D warnings`
  干净；`cargo fmt --all -- --check` 干净。
- reftest 不涉（纯 JS shim 语义——DC-4 口径延续）。

## 残余记账（M4-S7）

- pipeTo **read-ahead 泵**（spec 背压读取——dest hwm ∞ 时前读，'all pending writes
  should complete on abort' 3 腿 + flow-control 余量）。
- 'a rejection from underlyingSink.abort() should be preferred'（sink.abort 先于
  source.cancel 的事件序与 started 门的交互——需 spec 精确形甄别）。
- 'pipeTo on a teed readable byte stream'（tee 分支组合 abort）。
- byob controller 深路径（bad-buffers 15 腿 / read-min 9 腿——byobRequest.view/
  respond/零长度-detached enqueue TypeError）。
- readable-streams/tee 页级 Timeout 1 页。
