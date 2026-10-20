# M4-S5 — TransformStream spec 化重做 + writable 深状态 + 构造严格校验重做

**日期**: 2026-09-29
**切片**: M4-S5（streams 底座第三片——M4-S4 遗留 transform/writable 深状态机页级
Timeout 甄别解锁 + general 严格校验重做）
**原始数据**: [fetch](2026-09-29-m4-s5-fetch.json) ·
[xhr](2026-09-29-m4-s5-xhr.json) · [url](2026-09-29-m4-s5-url.json) ·
[mimesniff](2026-09-29-m4-s5-mimesniff.json) ·
[streams](2026-09-29-m4-s5-streams.json) ·
[eventsource](2026-09-29-m4-s5-eventsource.json)

## 结果（六 corpus，对照 M4-S4）

| corpus | M4-S4 | M4-S5 | Δ pass |
|---|---|---|---|
| fetch | 1225/1745（70.2%） | 1227/1745（70.3%） | +2 |
| xhr | 240/451（53.2%） | 240/451（53.2%） | 0 |
| url | 4431/6065（73.1%） | 4431/6065（73.1%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 548/823（66.6%） | **635/910（69.8%）** | **+87** |
| eventsource | 2/32（6.2%） | 2/32（6.2%） | 0 |
| **合计** | **8344/11014（75.8%）** | **8433/11101（76.0%）** | **+89** |

streams 分母 823→910（页级 Timeout 11→1——仅 readable-streams/tee 余挂）。
页级（对照 M4-S4）：readable general 30/38→**38/38 全绿** · writable constructor
6/13→**13/13** · properties 7/8→**8/8** · transform strategies 1/10→9/10 ·
general 16/26→24/26 · errors 2/5→11/21 · backpressure 3/8→11/14 · terminate
0/6→4/6 · reentrant-strategies 0/2→7/11 · cancel 0/11→3/11 · writable aborting
5/14→22/61 · write 4/13→7/13 · byob/general 19/26→21/26。**-2 记账**：
owning-type.tentative 2/5→0/5——tentative `type: 'owning'`（非发布 enum）由宽松
忽略改 spec 严格拒绝（ReadableStreamType = {"bytes"}），spec 正确形态。

## 修齐内容（part02.js / part10.rs）

### 1. 构造严格校验重做（M4-S1 回退面收口——undefined-source 根因已解）

- WebIDL object 转换：显式 `null` source/sink → TypeError（undefined/缺省 = 无源，
  M4-S4 判例保持）；非 object 原语 → TypeError（「garbage」面）。
- `ReadableStreamType` enum：非 'bytes'（含 null/''/'asdf'/ToString 抛错）→ TypeError；
  `getReader` mode enum：非 'byob'（undefined/null 缺省）→ TypeError。
- start/pull/cancel **callback 转换**：非 null/undefined 非函数 → TypeError
  （「will not tolerate initial garbage」面；callback 类型接受 null/undefined = absent）。
- WritableStream `type` 成员存在 → RangeError（spec 保留面）。
- controller 原型面：ReadableStreamDefaultController proto own props 恰
  close/constructor/desiredSize/enqueue/error（「extensible controller」面）；
  `locked` accessor 上移 `ReadableStream.prototype`/`WritableStream.prototype`
  （Subclassing + getOwnPropertyDescriptor(...).get 品牌访问面）。
- 全局类补齐：`WritableStreamDefaultController`（不可构造 TypeError）、
  `WritableStreamDefaultWriter`（brand + locked → TypeError，getWriter 同厂）、
  `TransformStreamDefaultController`（不可构造）+ writer/controller `.constructor`
  身份（`c.constructor`/`writer.constructor` 面）。
- `sink.close()` 无参调用（properties「0 arguments」面）。

### 2. TransformStream spec 化重做（spec §6.2——页级 Timeout 5 页全解除）

- 构造：readableType/writableType → RangeError；writable/readable 策略分离
  （writable 默认 hwm 1、readable 默认 hwm 0 + size 算法透传）。
- **backpressure 机制**：[[backpressureChangePromise]]（初始 true）——SinkWrite 背压
  门控（等 change promise 再 transform + 醒后 errored 检查）；写链**逐写串行**
  （spec AdvanceQueueIfNeeded 等价）；SourcePull 置 false 返新 change promise；
  ControllerEnqueue false→true 翻转观察（HasBackpressure = !ShouldCallPull 精确形——
  `_zwRsProbe` state/closeRequested/desired/readRequests 锚）。
- **传播语义**：transform/flush/start 拒绝 → 双侧 error；readable cancel → cancel
  算法后 error writable（SinkAbort 同型）；terminate → close readable + error
  writable(TypeError)；start promise 门控写链（transform/flush 不早于 start 完成；
  start 恰调一次）。
- closeReadableSafe（已关/排空关面静默返回——terminate after cancel 面）；
  readable/writable prototype accessor **带 setter**（getter-only 下 sloppy 构造器
  `this.readable = ...` 静默失效——回归根因，探针定谳）。

### 3. writable in-flight write 语义（spec FinishErroring）

- `[[inFlightWriteRequest]]` 锚：controller.error/abort/写失败 → 拒绝 **queued**
  write 请求；**in-flight 由 sink.write 完成面收尾**（正常完成 → fulfill——writable
  constructor「controller.error() in write()」write fulfill 面）；sink.write 拒绝/
  同步抛 → 本 entry reject + 流 error（M4-S4 遗留收尾）。
- 存量引擎测试随 spec 翻新 1 处（part10 R2969 桥测：in-flight write reject → resolve）。

### 4. 内部流创建投毒免疫

- `_bodyToStream` source **null 原型**（stream-safe-creation 页——Object.prototype.type
  投毒不得触及内部流创建；fetch +2）。

## 门禁

- `make test` 全绿（EXIT=0；含 1 处 spec 翻新的存量桥测）。
- `cargo clippy --workspace --all-targets -- -D warnings` 干净；`cargo fmt --all -- --check`
  干净。
- reftest 不涉（纯 JS shim 语义——DC-4 口径延续 M4-S4 判例）。

## 残余记账（M4 后续）

- writable aborting 39 腿（erroring 态状态机：ready 拒绝时序/abort 期间 write/close
  语义——61 腿已全量注册，22 绿）。
- byob controller 深路径（byobRequest.view/respond/零长度-detached enqueue TypeError——
  bad-buffers 15 腿 + read-min 9 腿）。
- transform cancel 余量（8 腿——cancel 时序/change promise 精确形）。
- readable-streams/tee 页级 Timeout 1 页（缓存读链挂死甄别）。
- eventsource 维持 6.2%（P7 event-source.py fixture 关联——.asis 族）。
