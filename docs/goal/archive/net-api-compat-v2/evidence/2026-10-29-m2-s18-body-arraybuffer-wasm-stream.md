# M2-S18 — Body mixin arrayBuffer() spec 形态 + wasm streaming 2F 挂账清偿（2026-10-29）

**切片**: wasm-compat goal M3 挂账移交（[收口档](../../../wasm-compat/evidence/2026-10-29-m3-close.md)
「残差 2F 挂账」）——`wasm_stream_compile/instantiate_test.any.js :: using
ReadableStream with ArrayBuffer chunk` 两腿，net-api goal 域承接。
**2F → 0F**：wasm/webapi 全子集 15 文件 **272/272 全绿**（M3 基线 270/272）。

## 红腿分型（2F）

`promise_rejects_js(t, TypeError, WebAssembly.compileStreaming(response))`
两腿（compile/instantiate 各一）报
`assert_unreached: Should have rejected: undefined`——compileStreaming
**成功 resolve**，预期 TypeError 拒绝未发生。

测试 setup（[语料](https://streams.spec.whatwg.org/#read-loop 注释钉规范））：

```js
const buffer = await v.arrayBuffer();          // Response 消费 fetch 体
const stream = new ReadableStream({ start(c) { c.enqueue(buffer); c.close(); } });
const response = new Response(stream, { headers: { 'Content-Type': 'application/wasm' } });
await promise_rejects_js(t, TypeError, WebAssembly.compileStreaming(response));
```

## 根因

Body mixin `arrayBuffer()` 返 **Uint8Array** 而非 spec ArrayBuffer
（R2978/R2982 旧形态，`part01.js` 两处「已知限制」注记在案）。旧形态下
测试 enqueue 的 chunk 是合法 ArrayBufferView → 体消费读循环的 chunk 类型门
（`_zwConsumeBodyBytes` 泵，`r.value instanceof Uint8Array`，非 Uint8Array
→ TypeError——streams spec read-loop「非 ArrayBufferView chunk 须拒」的
既有实现）不触发 → compileStreaming 消费成功 → 期望的 TypeError 缺席。

上轮 429 中断前探针（zwm2s18_probe.any.js，临时语料已删）实证：
`stream body arrayBuffer gate err=null`——Uint8Array chunk 过门零错误，
与根因一致；引擎侧类型门本身无缺口。

## 修复（spec 形态对齐，引擎 2 处 + 单测 2 文件）

- `Response.arrayBuffer` / `Request.arrayBuffer`（`part01.js` 两处）：
  返 `cp.buffer`（fresh 拷贝的精确尺寸 ArrayBuffer，零拷贝外露），
  Blob.arrayBuffer 同款先例（R3011/M2-P1）。`_zwBytesThenableSafe` 保留
  （自有 `then:undefined` 截断原型链投毒，ArrayBuffer 面同防御）。
  https://fetch.spec.whatwg.org/#dom-body-arraybuffer
- 引擎单测同步 spec 断言：`part10.rs`（Response）/`part11.rs`（Request，
  含无体腿 `length` → `byteLength`）——旧断言钉 Uint8Array 形态，随
  「已知限制」解除一并翻转，spec 链接在案。

## 消费方核对（返回类型变更 blast radius）

| 消费方 | 形态 | 兼容性 |
|---|---|---|
| `dom_bridge.rs` wasm 桥 compileStreaming/instantiateStreaming | `new Uint8Array(buffer)` 包裹 | ArrayBuffer/视图双形态兼容 |
| runner `testharness.rs` streaming 两件 | `new W.Module(buffer)` | BufferSource 直收 ArrayBuffer |
| `part03.js` script fetch charset 面 | `TextDecoder.decode(_ab122)` | decode 显式 ArrayBuffer 分支（part02） |
| FileReader readAs* | Blob.arrayBuffer 路径 | 不经本次修改面 |

## 验证

- 探针（`make testharness-wasm-core FILTER=wasm_stream`）：**26/26**，
  两目标腿 Pass（`wasm-m2s18-r3-201222.log`）。
- wasm/webapi 全子集（15 文件，缺盘 13 案经 raw 按 pin 补齐后）：
  **272 Pass / 0 Fail**（`wasm-m2s18-webapi-full-201750.log`），M3 基线
  270/272 → 272/272，其余 270 子测试零回归。
- wasm/jsapi 回归底线（`make testharness-wasm`）：**878/878**（
  `wasm-m2s18-jsapi-201258.log`），DC-4 只增不降维持。
- 全通道 A/B（`netapi-m2s18-r1.json` vs `netapi-m2s17-r1.json`，
  `abdiff-m2s18-r1.txt`）：**REGRESSIONS 0 / MISSING 0**，pass
  11039 → 11040（分母 12077 持平）。唯一 GAINS 为基线漂移伪影（见
  记账 3）——**本切片净账 0 增益 / 0 回归 / 0 MISSING**，fetch/api
  consume 族（arrayBuffer 主要消费面）零扰动。
- 范围引擎测试（test-guard 包裹 `cargo test -p zero-engine`）：
  **3592 过 / 11 败**（`m2s18-engine-tests-r2-*.log`）。名单对账：9 个
  about:blank 基址 Invalid URL 族 + `test_details_name_exclusivity_template_deep_entry`
  （S17 在册并行流红）不变；**+1 `fetch_sync_wire_preserves_response_contract`**
  经 stash 对照（无本切片改动同败，两次复现）为 rebase 基座预存红，
  归并行流域核对（run-rules §10，共享面不越界不代修）。本切片更新的
  `test_blob_stream_response_body_readers_r2978`（Response arrayBuffer
  spec 断言）**绿**；`test_request_body_readers_r2982`（Request 同形）
  断言因预存 about:blank 族红在 308 行 `new Request('/api')` 构造处未达
  （Request.arrayBuffer spec 语义由语料面 request-consume 族 A/B 零回归
  覆盖）。守卫备忘：冷编译在 8G envelope 内超限被杀（编译阶段无界是
  run-rules 既有约定）——先 `cargo test -p zero-engine --no-run` 预编译
  再包裹执行阶段。

## 记账

1. wasm/webapi 语料 13 案缺盘根因：上轮（429 中断前）FORCE=1 语料重拉
   被 GitHub trees API 403 限流打断，webapi 目录先清后拉残缺（2/15 案）。
   本轮经 raw.githubusercontent.com（独立于 API 限流）按 pin
   `315976933870` 逐文件补齐；32 脚本 FORCE 全量重拉仍 403（大仓递归列树
   触发次级限流），语料滚动结算按 S13 巡检结论走 pin 单点，不为本片重开。
2. `Response.arrayBuffer()` 返真 ArrayBuffer 后，enqueue 该结果的语料腿
   （wasm_stream_* 两件）走既有类型门拒绝——引擎「非 ArrayBufferView chunk
   须 TypeError」语义由 M4 既有泵实现承接，无新增门。
3. **S17 基线 JSON 漂移伪影定谳**：A/B 唯一 GAINS
   （cors-filtering「CORS filter on Content-Type header」，
   S17 期 `text/html, text/plain` 双 CT 拼接 → 本轮单值 `text/html`）
   经三重对照**与本切片无关**——① stash WIP 重建后该腿仍 Pass；②
   精确回 S17 基树（f0e69f2fd）重建后仍 Pass；③ S17 基线 JSON 采于
   15:54:28，晚于 M3 runner 静态头去重修复（d7798559a，15:44:10）落地
   10 分钟——S17 采数所用 runner 二进制早于该修复入树（上一 session
   并行流窗口内未重取 main 即采数）。该 +1 为**基线修正**（M3 既有
   修复的正确显影），pass 基线 11039 → 11040 含此修正；后续轮次以
   `netapi-m2s18-r1.json` 为最新全通道参考。教训：并行流共享 runner
   二进制面，采全通道基线前先 pull --rebase + 重建（run-rules §8/§13
   本轮实际已守，唯上一 session 采数窗口踩中）。
