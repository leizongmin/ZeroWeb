# WPT Review 分母收口

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 review complete（零 runtime 源码改动）
**最终逐案裁决**：[final remaining review TSV](2026-08-19-final-remaining-review.tsv)
**Runner disposition**：[294-source contract](2026-08-19-wpt-disposition.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 最后 38 source | 一手事实 | 高 |
| 页面/helper/worker test 生成逻辑 | 270 个 subtest | 一手事实 | 高 |
| 前序 evidence 资源闭包 | partition/fetch/redirect/import 等复用资源 | 一手事实 | 高 |
| Goal support envelope | runtime defer、environment gate、明确 skip | 项目契约 | 高 |

## 0. 执行摘要

- 对初始 inventory 最后 38 个 `review` 完成逐案裁决，共 **270 个 subtest**：
  - **14 source / 45 subtest**：目标内 runtime defer；
  - **8 source / 26 subtest**：MIME/CSP/secure/request-header environment gate；
  - **16 source / 199 subtest**：multi-context/partition/cookie/media/WebSocket/worker-client skip。
- 至此初始 inventory 的 **152/152 个 review source 全部完成人工传递审计**，逻辑剩余
  review 为 **0**。
- “review 清零”只证明 WPT 分母已分类，不代表 Service Worker runtime、fetch interception
  或 WPT 通过率完成；源码实现仍受 RFC 审批门禁。
- runner 可直接按统一 contract 的 `core` / `defer` / `gated` / `skip` lane 选择 source；
  `make audit-wpt-service-workers-disposition` 会从原始 inventory 和十批账本重建并逐字节校验。

TSV SHA-256：
`238904601e0c1a87b4a7a787e7ace8614d8f80f6af696bacd06154e111ef3900`。

## 1. 分母核算

浅层 test 声明需作两处关键校准：

- 三个 CSP source：每个是 1 个 `service_worker_test` wrapper + worker 内 4 项，共 **5**
- `worker-interception-redirect`：
  - 4 个 redirect scenario
  - 2 个 Worker type × 2 个 classic/module type
  - 每个组合 1 parent promise test + 3 个运行时注册的 sync test
  - 加 setup/cleanup，共 **66**

其他 helper/loop：

- same-site cookie matrix：66
- async waitUntil：14
- worker interception：16
- WebVTT cross-origin：10
- local URL controller inheritance：9

逐文件总和为 270。

## 2. 目标内 defer（14 source / 45 subtest）

### Controller/context/lifecycle（7 source / 16 subtest）

- active、controller load/reload/disconnect、waiting、skipWaiting
- srcdoc controller/getRegistration/postMessage
- detached iframe global 的 container/registration/worker 对象生命周期

这些均属于 M1/M3 长期目标，需要每 Document container、controller 和真实生命周期接线。

### Event/runtime/storage（5 source / 27 subtest）

- ExtendableEvent waitUntil 的 install/activate fulfilled/rejected、promise precedence
- task/microtask 与 respondWith dispatch 边界
- ServiceWorkerGlobalScope prototype chain immutability
- IndexedDB 在 SW global 中创建/写入，再由 window 读取

它们要求 typed event loop、MessageChannel 结果通道和已完成的 IndexedDB bridge。

### URL/CSS（2 source / 4 subtest）

- relevant/incumbent global 的 scriptURL/scope 解析
- SW 用同源 Response 替换跨源 CSS request 后 CSSOM 不应 taint

## 3. Environment gate（8 source / 26 subtest）

| 类别 | Source | Subtest | Gate |
|------|-------:|--------:|------|
| MIME sniffing | 1 | 1 | navigation response sniff + HTML parser |
| Opaque script | 1 | 4 | cross-origin/no-cors script loading |
| Sandboxed iframe | 1 | 4 | sandbox flags 与 secure container exposure |
| Secure context | 1 | 1 | HTTP top-level + HTTPS iframe + popup |
| Service Worker CSP | 3 | 15 | dynamic CSP header、cross-origin import/fetch/redirect |
| Service-Worker request header | 1 | 1 | dynamic handler 捕获 main/import/update 请求头 |

CSP Python handler 为每个 directive 生成不同 CSP header 和 4 个 worker tests，不能把 handler
源码作为 JavaScript 返回。

关键隐藏资源：

| Resource | Bytes | Blob SHA |
|----------|------:|----------|
| `resources/service-worker-csp-worker.py` | 6,035 | `35a46964a7871a7ab85d4b0b181e5ff2e3f496fc` |
| `resources/immutable-prototype-serviceworker.js` | 409 | `d8a94ad46befb085791d13f21e6870e09284abe0` |
| `resources/indexeddb-worker.js` | 1,685 | `9add47683884c04fc22c34337714fd2a356dcac7` |

## 4. 明确 skip（16 source / 199 subtest）

### Multi-context/client（6 source / 102 subtest）

- about:blank/srcdoc nested iframe 与 popup client identity
- blob/data URL iframe、DedicatedWorker、SharedWorker controller inheritance
- nested blob workers
- worker main/subresource interception
- worker redirect 在 2 Worker type × 2 module type × 4 scenario 的 client/controller 语义
- data iframe opaque-origin exposure

这些超出单页面环境和“多客户端逐 client 控制”边界。

### Partition/cookie（4 source / 70 subtest）

- partitioned SW identity、clients.matchAll 与 partitioned cookie
- same-site cookie 的 origin/site/nested/redirect/GET/POST/navigation-preload 66 项矩阵

它们依赖 third-party window/iframe、partition key 与 cookie policy。

### Media/platform（6 source / 27 subtest）

- embed/object document/image/navigation
- multipart image + canvas/CORS
- WebVTT track + cross-origin/redirect
- WebSocket handshake 与 SW global WebSocket
- XSLT base URL

这些被测核心分别属于 embed/object、media、WebSocket 和 XSLT 平台，不是基础 SW fetch。

## 5. 全量 Review 账本

| Batch | Source |
|-------|-------:|
| M1 no-signal | 14 |
| M1 iframe | 11 |
| M1 message-channel | 7 |
| M1 final | 25 |
| Static Routing（不含已在 M1 final 的 1 案） | 11 |
| Worker global/import | 13 |
| IDL harness | 1 |
| Navigation/redirect | 15 |
| Request/response/timing | 17 |
| Final remaining | 38 |
| **合计** | **152** |

所有 batch 路径互斥；初始 inventory 本身保留原始启发式分类，当前 disposition 以 evidence
账本为准。

## 6. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 最终批为 38 / 270 | 页面/helper/worker | final TSV | 一致 | 高 |
| CSP 每文件 5 项 | 页面 wrapper | worker 动态生成 4 项 | 一致 | 高 |
| worker redirect 为 66 | 4×2×2 parent | 每 parent 3 子项 + setup/cleanup | 一致 | 高 |
| review 为 152/152 | inventory 初始 review | 十批路径并集 | 一致 | 高 |
| 逻辑剩余 review 为 0 | 初始 152 | 已裁决 152 个唯一路径 | 一致 | 高 |

## 7. 后续输入

1. RFC 批准后按 Tier A → next-wave → static-wave → defer family 顺序建立 red baseline。
2. Dynamic WPT server adapter 落地时，从 gated 清单恢复完整上游 source。
3. 只有 support envelope 扩大时才恢复 partition/multi-client/media/platform skip。

## 8. 质量审查

- [x] 最后 38/38 source 正文已读，manifest blob SHA 匹配。
- [x] test-generating worker/helper 已读，270 个 subtest 已展开。
- [x] defer、environment gated、support-envelope skip 已分开。
- [x] 十批路径互斥并覆盖初始 review 152/152。
- [x] 294 source / 331 URL 已转为唯一、可重建的 runner disposition contract。
- [x] 12 个 core source 与 runner 导入账本、三批 case asset 及 blob SHA 精确对应。
- [x] 未修改 runtime 源码、WPT 数据或既有 inventory 初筛记录。
