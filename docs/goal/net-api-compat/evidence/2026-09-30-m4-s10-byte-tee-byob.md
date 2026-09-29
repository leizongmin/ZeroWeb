# M4-S10 — Byte tee 源侧 BYOB pull 切换（spec ReadableByteStreamTee 端口）专设 session

**日期**: 2026-09-30
**切片**: M4-S10（streams 底座第八片——tee 重做 + 源侧 byob 读 + 配套 byte 流语义修齐；
前置：V1-V4 四版回退记账见 master.md）
**原始数据**: [fetch](2026-09-30-m4-s10-fetch.json) ·
[xhr](2026-09-30-m4-s10-xhr.json) · [url](2026-09-30-m4-s10-url.json) ·
[mimesniff](2026-09-30-m4-s10-mimesniff.json) ·
[streams](2026-09-30-m4-s10-streams.json) ·
[eventsource](2026-09-30-m4-s10-eventsource.json)

## 结果（六 corpus，对照 M4-S9）

| corpus | M4-S9 | M4-S10 | Δ pass |
|---|---|---|---|
| fetch | 1229/1766（69.6%） | 1229/1766（69.6%） | 0 |
| xhr | 240/451（53.2%） | 240/451（53.2%） | 0 |
| url | 4431/6065（73.1%） | 4431/6065（73.1%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 826/993（83.2%） | **862/1020（84.5%）** | **+36** |
| eventsource | 2/32（6.2%） | 2/32（6.2%） | 0 |
| **合计** | **8626/11205（77.0%）** | **8662/11232（77.1%）** | **+36** |

streams 页级位移：`readable-byte-streams/tee.any.js` **10/40 + 页级 Timeout → 40/40 零超时**
（页级 Timeout 根因 = 'closing the original should close the branches' 腿：原 buffer-tee 的
分支 closed promise 不随源 close 传播——spec 结构的 closeSteps 双分支关流解除）；
`readable-streams/tee.any.js` 24→26；`read-min` 16→18；`bad-buffers-and-views` 11→12
（respondWithNewView spec 校验解锁）；`piping/abort` 28→29（errorStream 标记 handled）。
零 pass 回归（其余五 corpus 全量逐数一致）。

## 探针逐 hop 根因链（Node 复算环境，shim streams 段直载）

1. **分支 byob 读 PEND 根因**：分支 byob read 挂 pull-into 描述符后调 tee 分支 pull，
   原 buffer-tee 只会走源**默认读** → 源 pull 的 `c.byobRequest` 为 null
   （WPT leg 4 'TypeError: Cannot read properties of null' 同形复现）。
2. **V1-V4 回退真因**：spec 的双 reader 切换（releaseLock + getReader({mode:'byob'})）在
   本实现触发 M4-S8 的 `reader.closed` 拒绝前向（release 即 readable 态拒绝）→ 双分支被
   误 error。**解法：不切换 reader**——byob read 原语上移构造器级（`_zwByobReadRaw`），
   tee 直接以分支 byobRequest 视图发起源 pull-into；单 reader 下无假拒绝面。
3. **错误前向越过已读 chunk 投递（'errors in the source' 双块队列源根因）**：promise 形读
   （`readRaw().then(h)`）的投递反应在**整个同步体之后**注册，而源 pull throw → errorStream
   → closedPromise 前向在同一同步体内入队 → 前向恒先于投递执行，已 dequeue 的 chunk 投递
   分支时分支已 errored、被静默丢弃。spec 无此问题：read request steps 在 PullSteps 内同步
   执行、投递微任务先于 CallPullIfNeeded 的 throw 入队。**解法：steps 形内部读 API**
   （`_zwReadRawSteps` + byobReadInto 第三参 request），tee 投递微任务在 dequeue/commit
   同步步内排队，恢复 spec 时序。

## 修齐内容（part02.js）

- **tee() 重做为 spec ReadableByteStreamTee 结构**（替换 buffer-based 简化模型）：`reading`
  串行门 + `readAgainForBranch1/2`（chunk/close steps 开头复位、末尾按旗标续拉——pull 计数面）；
  `pullWithDefaultReader`（steps 形源默认读，双分支 enqueue：branch1 换新缓冲保偏移视图 +
  branch2 内容克隆——无 TransferArrayBuffer 下的 spec 等价形）；`pullWithBYOBReader`（**源侧
  BYOB 读**：分支 byobRequest 视图直入源 pull-into，源 enqueue/respond 物理写分支缓冲后
  `respondWithNewView(chunk)` 回提交 byob 分支 + enqueue 克隆给对侧分支——源 pull 的
  byobRequest.view 非空面）；closeSteps 双分支关流 + cancelPromise 兑现。
- **分支 cancel 单路化**：`desc.cancel = teeCancelBranch`（spec 分支 cancelAlgorithm 即
  cancel1/2Algorithm）——stream.cancel 与 reader.cancel 同经 cancelInternal→_doCancel→srcCancel；
  撤 M4-S8 的 stream.cancel 实例包装（reader.cancel 不走包装的挂账腿根因）。
- **`_doCancel` 两处 spec 修齐**：① closeStream(true)——cancel 的 pull-into close steps
  given undefined（read 兑现 value undefined）；② errored 态 reject storedError（spec
  ReadableStreamCancel 步骤 3——composite cancelPromise 兑现值采用 rejection，
  'erroring a teed stream should properly handle canceled branches' 双页解锁）。
- **respondWithNewView 校验升 spec**（steps 7-9）：写入偏移匹配 + **缓冲字节长**匹配
  （原对描述符视图长比较——整缓冲分支视图回提交被误拒）+ 容量校验（'byobRequest.view.
  subarray(1,2)' RangeError 腿解锁）。
- **errorStream 标记 closedP handled**（spec ReadableStreamError 步骤 7）——页面未访问
  .closed 的 reader 不产生 unhandled rejection（双 tee 页 'Unhandled rejection' 失败根因）。

## 门禁

- `make test` 全绿；`cargo clippy --workspace --all-targets -- -D warnings` 干净；
  `cargo fmt --all -- --check` 干净。
- 纯 JS shim 语义切片，不触渲染——reftest 不涉（DC-4 口径延续）。

## 残余记账（M4 收尾）

- transfer/detach 族结构性挂账不变（需宿主 V8 detach，14 腿）。
- pipeTo read-ahead 泵（3 腿）+ sink.abort 事件序（1 腿）——未涉本片。
- eventsource P7 fixture 关联面（.asis/event-source.py）——并轨推进。
