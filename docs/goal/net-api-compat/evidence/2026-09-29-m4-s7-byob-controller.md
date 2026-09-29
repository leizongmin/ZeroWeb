# M4-S7 — BYOB controller 深路径（pull-into 描述符 / byobRequest / respond / read({min})）

**日期**: 2026-09-29
**切片**: M4-S7（streams 底座第五片——ReadableByteStreamController spec 子集 + BYOB
reader read(view,{min}) 描述符化重做）
**原始数据**: [fetch](2026-09-29-m4-s7-fetch.json) ·
[xhr](2026-09-29-m4-s7-xhr.json) · [url](2026-09-29-m4-s7-url.json) ·
[mimesniff](2026-09-29-m4-s7-mimesniff.json) ·
[streams](2026-09-29-m4-s7-streams.json) ·
[eventsource](2026-09-29-m4-s7-eventsource.json)

## 结果（六 corpus，对照 M4-S6）

| corpus | M4-S6 | M4-S7 | Δ pass |
|---|---|---|---|
| fetch | 1227/1745（70.3%） | 1229/1766（69.6%） | +2 |
| xhr | 240/451（53.2%） | 240/451（53.2%） | 0 |
| url | 4431/6065（73.1%） | 4431/6065（73.1%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 774/971（79.7%） | **794/971（81.8%）** | **+20** |
| eventsource | 2/32（6.2%） | 2/32（6.2%） | 0 |
| **合计** | **8572/11162（76.8%）** | **8594/11183（76.8%）** | **+22** |

fetch 分母 1745→1766（stream-safe-creation 面分母重锚，pass +2 零回归）。streams 页级：
read-min 4/24→**16/24**（byobRequest.view/respond 流 + min 校验链 + 多 enqueue 补齐）·
bad-buffers-and-views 3/22→**11/21**（enqueue 零长度 TypeError + respondWithNewView
RangeError 校验）· respond-after-enqueue 2/3→2/3（benign byobRequest 免崩溃）·
byob/general 21/26→22/25 · tee 4/9→2/9（**-2 记账**：pull-into 条件收紧后 byob 分支
读取面变化——byte tee 分支非字节流面归 M4-S8）· 全量丢腿仅 1（non-transferable-buffers
respondWithNewView 非传递缓冲——WASM 缓冲外延面，挂账）。

## 修齐内容（part02.js）

### 1. BYOB read pull-into 描述符化（spec §4.5/§4.9.5 子集）

- read(view, {min}) 校验链：零长度视图 → TypeError、min 0/负/非法 → TypeError、
  min > 视图长（按元素计）→ RangeError。
- 队列可满足 → **跨 chunk 拷贝填充至视图容量**（余量回队 unshift）+ commit（同缓冲
  新视图 done:false——「enqueue(), then read({min})」族面）。
- 不足且 closed/closeRequested-排空 → close steps（部分填充视图 done:true）。
- 否则 pull-into 描述符入列（FIFO）→ flushPull → 源经 byobRequest 填充（**最小填充
  契约：filled >= min 才 commit**；min 按元素×elementSize 计字节目标）。

### 2. ReadableByteStreamController.byobRequest（spec §4.7.3）

- pending pull-into 非空 → BYOBRequest：view = 构造于 buffer 已填偏移后的余量视图；
  respond(n)（closed 态零写入收尾 / 越界 RangeError / filled >= min → commit 否则
  **续拉 flushPull**）；respondWithNewView（非零长校验 + **buffer 字节长一致
  RangeError** + closed 零长面）。
- flushPull 条件纳入 pull-into（spec ShouldCallPull——BYOB read-into-requests > 0；
  修「pull never fired」挂死根因）。
- autoAllocateChunkSize benign 形（>0 时 byobRequest 非空可安全 respond——
  respond-after-enqueue 3 腿免崩溃；=== 0 → TypeError 构造面）。

### 3. 字节流 enqueue 语义

- 零长度视图 → TypeError（bad-buffers「zero-length buffer/view」面）。
- pending pull-into → 填头描述符（跨描述符续填 + 余量回队）+ **尾随 CallPullIfNeeded**
  （部分填充未达 min → 续拉补齐——「multiple enqueue() up to 3 bytes」pullCount===2 面）。
- 等待默认读直接喂 chunk（FulfillReadRequest——跳过则队列无界增长的内存爆涨根因修复）。
- 字节流 close：pending 描述符 close steps（普通 close 交还已填视图；**cancel 路径
  given undefined**——spec ReadableStreamCancel 步骤 6）；errorStream：描述符 error
  steps reject（「read({min}), then error()」面）。

### 4. 全局类

- `ReadableStreamBYOBReader`（brand + 字节流校验 + locked TypeError，getReader 同厂）、
  `ReadableStreamBYOBRequest`（不可构造 TypeError）。

## 门禁

- `make test` 全绿（EXIT=0）；`cargo clippy --workspace --all-targets -- -D warnings`
  干净；`cargo fmt --all -- --check` 干净。
- reftest 不涉（纯 JS shim 语义——DC-4 口径延续）。

## 残余记账（M4-S8）

- **缓冲 transfer/detach 族**（bad-buffers 10 腿 + non-transferable 4 腿——spec
  enqueue/read 的 TransferArrayBuffer 语义需真 detach 能力：structuredClone transfer /
  postMessage 路径甄别）。
- byte tee 分支字节流身份（tee×byob 组合——tee.any.js 2 腿回归记账）。
- pipeTo read-ahead 泵（3 腿）+ sink.abort 事件序（1 腿）。
- readable-streams/tee 页级 Timeout 1 页；eventsource P7 fixture 关联面。
