# M4-S4 — values() 完整语义 + 构造 dictionary 转换序（M4-S1 两处回退的重做切片）

**日期**: 2026-09-29
**切片**: M4-S4（streams 底座第二片——M4-S1 回退注记两项专设重做 + 随基线甄别展开的
构造/控制器/pull/reader 语义簇）
**原始数据**: [fetch](2026-09-29-m4-s4-fetch.json) ·
[xhr](2026-09-29-m4-s4-xhr.json) · [url](2026-09-29-m4-s4-url.json) ·
[mimesniff](2026-09-29-m4-s4-mimesniff.json) ·
[streams](2026-09-29-m4-s4-streams.json) ·
[eventsource](2026-09-29-m4-s4-eventsource.json)

## 结果（六 corpus，对照 M4-S3）

| corpus | M4-S3 | M4-S4 | Δ pass |
|---|---|---|---|
| fetch | 1221/1745（70.0%） | 1225/1745（70.2%） | +4 |
| xhr | 240/451（53.2%） | 240/451（53.2%） | 0 |
| url | 4431/6065（73.1%） | 4431/6065（73.1%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 271/598（45.3%） | **548/823（66.6%）** | **+277** |
| eventsource | 2/32（6.2%） | 2/32（6.2%） | 0 |
| **合计** | **8063/10789（74.7%）** | **8344/11014（75.8%）** | **+281** |

streams 分母 598→823：closed-promise 全局化解除页级挂死后 runner 记满全部注册腿
（M4-S3 多页以页超时截断记账）。streams 内页级（对照 M4-S3）：
async-iterator **9/41→41/41 全绿** · bad-strategies 2/8→**8/8** · bad-underlying-sources
1/5→**21/22** · cancel 1/2→**11/11** · general 6/18→30/38 · from 15/34→39/50 ·
default-reader 1/12→17/29 · queuing-strategies 7/20→18/20 · piping/abort 0/1→20/33 ·
throwing-options 0/5→**8/8** · pipe-through 13/43→21/43 · writable bad-strategies
2/7→**7/7** · tee（readable）4/9→6/9。页级 Timeout 存量 20（前值更高）：transform
backpressure/flush/reentrant/terminate、writable aborting/start/close、byob
general/tee/templated——深状态机面归后续切片。

## 修齐内容（part02.js / part01.js）

### 1. values() / @@asyncIterator 完整语义（spec §4.2.5 + WebIDL async iterator 机制）

- `values(options)` 命名方法 + `@@asyncIterator` 同体（preventCancel 取 options）；
  getReader 锁定（locked → 同步 TypeError）。
- **[[OngoingPromise]] 串行**：next/return 经 ongoing 链排队（前序未决则链后）——
  「return(); next() [no awaiting]」resolve 序锚（WebIDL「previous calls to next() have
  settled before return is called」的机制实现）。
- next：close/error steps 均 release（exhaustive 迭代/错误后可再 getReader——
  「Acquiring a reader after exhaustively async-iterating」面）；error 后 isDone →
  后续 next 返 done（「next() that reports an error; next()」面）。
- return：preventCancel 假 → cancelInternal（**errored → reject storedError**——
  「return() rejects if the stream has errored」面）+ **同步 release**（「return() should
  unlock the stream synchronously」面）+ cancel promise 稳定后 fulfill {value, done:true}
  （「delayed cancel」面——_doCancel 升级为 await source.cancel）。
- 迭代结果 %Object.prototype% 字面量（assert_iter_result [[Prototype]] 断言）；
  原型链接 %AsyncIteratorPrototype%（引擎 async generator 原型两跳；proto 自身 own
  props 恰 next/return）。

### 2. read() 结果形态分离（author vs 内部）

- author `reader.read()` 结果改 **%Object.prototype%** 形态（spec read-request 步骤）；
  内部消费（consume-body / pipeTo / tee 双处，part01+part02 四点）改走 `_zwReadRaw`
  （null 原型保留——M2-S3 then 投毒防线不回退，response-stream-with-broken-then
  六腿维持全绿）。

### 3. reader.[[closedPromise]] 全局化（挂死根因消除）

- 旧形态：closed getter 每次访问新建**永挂 Promise**——「先取 closed、后 error/cancel」
  的腿永不 settle（bad-strategies/bad-underlying-sources/default-reader 页级 Timeout
  根因）。改 waiters 列表：closeStream resolve 全部、errorStream reject 全部、
  getReader 时已 closed/errored → 立即 settle（spec GenericInitialize）、releaseLock
  摘除并按 GenericRelease 以 TypeError 拒绝（readable 态；已发布 promise 标 handled）。

### 4. 构造 dictionary 转换序（ReadableStream / WritableStream / 策略类）

- QueuingStrategy dictionary 转换先行（成员 getter 读取 + size 非函数 TypeError，
  **先于构造步骤**与 source.type 读取）；ExtractHighWaterMark：NaN/负数 → RangeError、
  +∞ 允许、bytes 默认 0 / default 默认 1；bytes 流禁 size（RangeError）。
  source null/undefined → {}（M4-S1 回退根因面=WebIDL object 转换语义，非校验对象）。
- UnderlyingSource 成员（start/pull/cancel）**构造时一次读取**（定义序，getter 抛错
  同步传播出构造器）+ 算法缓存（「second pull does not result in a second get」面）+
  回调 this = underlyingSource（「start should be called with the proper thisArg」面）；
  start 同步抛错冒出构造器（spec §4.2.3 re-throw 面）。
- CountQueuingStrategy/ByteLengthQueuingStrategy：QueuingStrategyInit required
  highWaterMark（非对象/缺失 → TypeError）；unrestricted double 转换存储；size 为
  **全局共享**箭头函数（instances 同一函数 + 无 prototype 属性 + 不可 new + name/length
  显式定）；ByteLength size = GetV(chunk, 'byteLength')（null/undefined → TypeError）。

### 5. 控制器语义（enqueue/close/cancel）+ pull 机制

- enqueue：CanCloseOrEnqueue（readable + 非 closeRequested）假 → TypeError；
  有等待 read → FulfillReadRequest（跳过 size）；size 抛错 → error 流 + 重抛；
  size 非法返回（NaN/负数/±∞）→ RangeError + error 流；末尾 CallPullIfNeeded。
  close：queue 非空 → closeRequested（drain 后真关——read/byob 两读路径补排空关），
  假 → TypeError（close twice / after cancel / after error 面）。
- ReadableStreamCancel 语义化（cancelInternal）：closed → resolved、errored →
  reject storedError、否则 close + await source.cancel（fulfillment → undefined、
  rejection 传播——cancel 回调抛错/返回 promise 面）；reader.cancel 改走其上
  （spec GenericCancel——旧路径 reader.cancel 恒被 locked TypeError 拒绝）。
- pull 机制 spec 化：start **微任务**完成（started 置位 + 首拉恒微任务——异步 start
  「cancelling before start finishes」面 + recording 工厂构造后补字段面）；CallPullIfNeeded
  条件 = started + readable + 非 closeRequested +（有等待 read 请求或 desiredSize>0）；
  pulling → pullAgain（**完成后续拉**——「next(); return() [no awaiting]」timesPulled===2
  语义锚）；pull 同步完成的重拉走微任务（零 size 源同步递归栈爆防护）。

### 6. pipeTo options / signal（piping 残余簇首片 + OOM 根因修复）

- StreamPipeOptions 序贯读取（**preventAbort → preventCancel → preventClose → signal**
  ——throwing-options touched 序锚）：pipeTo 在 rejected-promise 路径、pipeThrough
  同步 throw；提取值传 impl（getter 恰触一次）。
- signal 校验（非 AbortSignal 形态 → TypeError，事件零触碰）+ aborted signal →
  abortAlgorithm（preventAbort/preventCancel 门控 dest.abort + 源 cancel，reject
  signal.reason）+ abort 监听中途生效；preventClose（done 后不关 dest）/
  preventAbort（源 error 不 abort dest）/ preventCancel（dest error 不 cancel 源）门控；
  dest 首查（已 closed/errored → 源 cancel + 拒绝——「closing/error propagated
  backward」面）+ writer.closed 中途拒绝传播。**OOM 根因修复**：tee×无限源×pipeTo
  （signal 被忽略 → 泵永转分配）4.2GB test-guard 拦截——pre-aborted signal 即拒即收。
- WritableStream：start thenable 拒绝 → 流 error（「starts errored」面）；sink.write
  拒绝/同步抛 → 本 write 请求拒绝 + 流 error（旧形态仅 error 不 reject entry → write 永挂）。

## 门禁

- `make test` 全绿（EXIT=0，0 failures，engine 桥测零翻新——read 值形态等价）。
- `cargo clippy --workspace --all-targets -- -D warnings` 干净；`cargo fmt --all -- --check`
  干净。
- reftest 不涉（纯 JS shim 语义，无渲染面变更——DC-4 口径延续 M4-S1/M4-S3 判例）。

## 残余记账（M4 后续）

- transform-streams 深状态机：backpressure（backpressureChangePromise）/
  flush/errors/reentrant/terminate——页级 Timeout 5 页。
- writable-side 状态机：aborting（abort 期间 write/close 语义）/ start / close /
  reentrant-strategy / bad-underlying-sinks——页级 Timeout 6 页。
- byob 深路径：controller 语义（byobRequest.view / respond / 零长度与 detached
  buffer enqueue TypeError）/ general / tee / templated——页级 Timeout 4 页。
- readable-streams/general 余量：start/pull/cancel **callback 可调用校验**（'potato' →
  TypeError——M4-S1 回退的严格校验面，现 undefined-source 根因已解，可安全重做）+
  type enum 校验 + getReader mode enum 校验。
- fetch +4 已甄别：pipeTo preventCancel 面解锁（request/response pipe 腿）。
