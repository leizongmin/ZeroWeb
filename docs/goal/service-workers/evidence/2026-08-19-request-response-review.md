# Request、Response 与 Resource Timing WPT 裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零 runtime 源码改动）
**逐案裁决**：[request/response review TSV](2026-08-19-request-response-review.tsv)
**关键资源**：[request/response resources TSV](2026-08-19-request-response-resources.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 17 source / 83 subtest | 一手事实 | 高 |
| worker/handler/fixture 闭包 | 34 个关键资源、blob SHA | 一手事实 | 高 |
| Goal support envelope | 基础 fetch/response/Cache 消费边界 | 项目契约 | 高 |
| decision 分类 | defer/gated/skip 实施边界 | 作者综合 | 待 runtime 验证 |

## 0. 执行摘要

- 从 navigation 批次后的 55 个逻辑 `review` 中，审计 17 个 request/response/timing source。
- 17 案实际产生 **83 个 subtest**：
  - **7 source / 19 subtest**：目标内 fetch/response/XHR/performance defer；
  - **9 source / 63 subtest**：cookie、跨源、TAO、dynamic server、H2、Cache/timing gated；
  - **1 source / 1 subtest**：DataTransfer/File/form platform skip。
- Resource body-size 通过三层循环生成 24 项；body-accessed response 生成
  setup + cleanup + 18 项矩阵，共 20；Performance Timeline 还包含 2 个 worker-side test。
- 全量 inventory 的逻辑剩余 review 从 55 降为 **38**。

机器清单 SHA-256：

- review：`e5813db4ced69a3f65bcaad7b01b0197ba462401db6e43ce807c2a0a7ffaf25f`
- resources：`e4ad457823aa84aae9614a17ab0d52ceb972df124cf32a662ed71cc8941979db`

## 1. 分母核算

| Case | Subtest | 核算方式 |
|------|--------:|----------|
| `resource-timing-bodySize` | 24 | 3 mode × 2 TAO × 4 response variant |
| `respond-with-body-accessed-response` | 20 | setup + runtime cleanup + 18 case |
| `performance-timeline` | 4 | service_worker_test wrapper + 2 worker test + page test |
| `xhr-content-length` | 5 | setup + 4 header case |
| `xhr-response-url` | 6 | setup + 4 response case + cleanup |
| `credentials` | 5 | cookie setup + classic/import/module matrix |
| 其余 11 source | 19 | 显式/helper 声明 |
| **合计** | **83** | |

worker testharness 结果会并入页面，但 wrapper 自身仍计一个 subtest；运行时创建的 cleanup
`promise_test` 也必须计入分母。

## 2. 目标内 fetch/response defer

### Response validation（3 source / 3 subtest）

- NUL `Blob.type`
- NUL response header value
- ISO-8859-1 response header

三案使用静态 worker 合成 Response，由 controlled iframe 的 XHR 验证结果。它们直接驱动
Response→network response 转换和 header validation。

### FetchEvent.request projection（1 subtest）

`request-end-to-end` 把 navigation Request 的 URL、method、referrer、mode、credentials、
redirect、headers 与 immutability 结果序列化进合成 Response。它是 M2 Request 投影的核心用例。

### XHR integration（11 subtest）

- Content-Length：absent、larger、duplicate、bogus 与 ProgressEvent 语义。
- responseURL/responseXML：网络 fetch response 与 synthetic text/document response。

这些用例要求页面 XHR 穿过 SW fetch，但不依赖动态 WPT handler。

### Worker Performance Timeline（4 subtest）

worker-side 两项验证 User Timing 与 Resource Timing/buffer-full；页面项通过 busy wait 比较
普通和 slow SW fetch 的 duration。属于目标内 worker runtime + Performance API 接线。

## 3. 高阶 gate

### Script credentials/update（5 subtest）

动态 handler读取 Cookie，并用 server stash 改变每次脚本字节以触发 update；classic main/
importScripts 必须带 credential，module main/static import 不带。不能静态化。

### Resource Timing（39 subtest）

| Source | Subtest | 关键依赖 |
|--------|--------:|----------|
| body size | 24 | TAO、CORS/no-cors、stream、cross-origin image |
| cross-origin | 2 | timing allow 过滤 |
| Server-Timing | 1 | dynamic header、TAO、CORS、PerformanceObserver |
| fetch variants | 5 | dynamic redirect 与 SW 内前后 delay |
| mixed resource timing | 2 | redirect、ORB-compatible 404、跨源 |
| nextHopProtocol | 2 | H1/H2、synthetic/fallback/cache |
| opaque preload | 2 | preload cache、opaque no-cors、XHR |
| body accessed/cache | 20 | 与上表重叠计入下节，不计本行合计 |

前五项共 34；加 nextHop/opaque 为 38。它们分别依赖协议栈、跨源 timing policy 或动态 server，
不能作为基础 fetch 首批。

### Response body/cache/cross-origin（20 subtest）

`respond-with-body-accessed-response` 覆盖 basic/opaque/default Response，0/1/2 clone，以及
CacheStorage round trip 前后多次访问 `.body`。opaque 分支要求跨源 no-cors，因此整文件 gated。

## 4. Platform skip

`data-transfer-files` 创建 DataTransfer/File，赋给 file input，再 multipart POST 到 HTML form
submission handler；SW 只做 network fallback。被测核心是 File/input/form submission 跨 SW
链路，超出基础 fetch/Cache 目标，记 `Unsupported(form-file-platform)`。

## 5. 关键资源

资源清单共 34 个对象、30,853 bytes，覆盖：

- cookie/stash script handlers
- invalid header/blob 与 XHR workers
- opaque preload fixtures
- worker performance/testharness
- cross-origin/stream/resource timing workers
- dynamic Server-Timing/TAO handler
- XHR responseURL/Content-Length workers

关键事实：

- `echo-cookie-worker.py` 同时读取 Cookie 和递增 stash counter。
- `fetch-response.js` 生成 constructed/forward/stream/pass-through 四类 Response。
- `server-timing.py` 动态设置 Server-Timing、TAO 与 CORS headers。
- `respond-with-body-accessed-response-worker.js` 在 clone/cache 前后多次访问 body。

## 6. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 本批为 17 source / 83 subtest | 页面/worker/helper | review TSV | 一致 | 高 |
| 7 source 属目标内 defer | 静态 worker 闭包 | Goal fetch/response 范围 | 一致 | 高 |
| 9 source 需环境 gate | handler/cross-origin/protocol | 页面 timing/header 断言 | 一致 | 高 |
| File/form 案超出范围 | 页面 DataTransfer/form | SW 仅 fallback | 一致 | 高 |
| 逻辑剩余 review 为 38 | 前序 55 | 55 - 17 | 一致 | 高 |

## 7. 后续输入

1. M2 fetch 主路径落地后优先恢复 Request projection 和三项 response validation。
2. XHR bridge 接线后恢复 Content-Length/responseURL 11 项。
3. Performance API 接线后恢复 worker timeline 4 项。
4. dynamic server、cross-origin timing 与 Cache 集成后恢复 9 个 gated source。
5. 继续裁决剩余 38 个 media/security/context/controller review。

## 8. 质量审查

- [x] 17/17 source 正文已读，manifest blob SHA 匹配。
- [x] 34 个关键 worker/handler/fixture 已读并记录 blob SHA。
- [x] worker/helper/loop/cleanup 已展开，83 个 subtest 无文本计数低估。
- [x] fetch core、environment gate 与 form/File skip 已分开。
- [x] 未修改 runtime 源码、WPT 数据或既有 inventory 初筛记录。
