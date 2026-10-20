# M4-S28 — streams flow-control 背压编排（pipeTo 泵串行读门）

**日期**: 2026-10-01
**基线**: M4-S26（2026-10-01，9290/11658 = 79.7%，streams 903/1019 = 88.6%）
**结果**: **9292/11659 = 79.7%**（streams **905/1020 = 88.7%**，flow-control
**5/5 全绿**——挂账 2 腿解锁，+2 零回归；fetch 1491/2081 / xhr 557/584 /
mimesniff 100% / url 73.0% 持平；eventsource 33/34——request-cache-control 页
全量争用超时再现，隔离复跑绿，在册族）。make test 19547 passed / 0 failed
（68 套全绿）+ clippy `-D warnings` 干净 + fmt 干净。

## 变更清单

### shim — pipeTo 泵串行读门（fetch/streams spec ReadableStreamPipeTo 逐 chunk 编排）

- **读在飞时不再入泵**（`pendingRead` 门）：S11 的 read-ahead 泵在 watchReady
  兑现/write 完成路径重入时，前一次 `_zwReadRaw` 仍在飞即再发读——并发双读超额
  消费背压余量（pump 日志探针定位：desired 2 被双读打穿 → 3 chunk 全拉，期望
  只拉 2）。
- spec pipeTo 编排为逐 chunk read→write→desiredSize 检查串行；背压余量（writer
  desiredSize）按在飞写记账，串行门使余量逐写递减（S26 trace 对照：write 后
  desired 2→1→0 → 泵止）。
- 写完成回调的续泵照旧（背压解除驱动尾部——「desires more chunks」用例在首写
  完成后继续拉 d 并 close ✓）。

## 解锁明细（+2 对照 S26，REGRESSED 0）

- streams/piping/flow-control 2：Piping from a ReadableStream to a
  WritableStream that desires more chunks before finishing with previous ones /
  Piping to a WritableStream that does not consume the writes fast enough exerts
  backpressure on the ReadableStream（M4-S11 挂账项清偿——flow-control 页 5/5）。

## 残余

- **streams 结构性挂账**：transfer/detach 族（需宿主 V8 detach——14 腿），
  JS 层不可表达，维持挂账。
- **eventsource/request-cache-control 全量争用超时**——隔离绿（在册族）。
- 至此 M4 里程碑除结构性挂账（V8 detach）外全部清偿。
