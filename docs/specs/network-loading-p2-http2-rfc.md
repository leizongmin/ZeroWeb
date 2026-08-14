# Spec：网络加载 P2（HTTP/1.1/2 流式传输与连接预热）

**版本**：v1.0
**日期**：2026-08-14
**状态**：已确认实施（HTTP/3 明确排除）

---

## 0. 执行摘要

- **目标**：在不引入 QUIC/HTTP/3 的前提下，将现有异步 HTTP/1.1/2 传输接入流式消费者和 HTML 连接预热提示。
- **本期范围**：`HttpClient` 流式响应、协议 telemetry、`link rel=preconnect`/`dns-prefetch` 的受控执行。
- **明确排除**：HTTP/3、QUIC、新第三方网络依赖、HTTP/3 优先级帧、跨导航持久化预热状态。
- **核心约束**：预热不得写 HTTP 缓存、不得携带页面凭据或敏感头、不得阻断渲染或导航、失败仅记录匿名 telemetry。
- **推荐方案**：复用已缓存的 reqwest async client，以无响应体的 `HEAD` 请求预热可复用连接；流式 API 直接消费 chunk，不在 `HttpClient` 聚合 body。
- **首个落地步骤**：为 `zero-net` 定义预热请求/事件与 HTTP/2 Priority header 映射，并以本地 TCP fixture 验证请求方法、缓存隔离和失败降级。

## 1. 背景与目标

### 1.1 背景

P0/P1 已将页面资源统一接入 `ResourceLoader`，并已提供 `HttpClient::send_async_stream`。但 HTML 资源提示中的 `preconnect` 与 `dns-prefetch` 仅被解析，没有触发传输层动作；流式 API 也没有页面消费者。当前依赖为 `reqwest 0.12` + `tokio`，未包含 HTTP/3/QUIC 能力。

### 1.2 目标

- 页面加载可非阻塞地执行同源或跨源的受控连接预热，并在后续请求复用 async client 的连接池。
- 页面可选择流式取得大体积子资源，传输层不额外保留完整 body。
- 匿名 telemetry 能报告实际协商的 `http/1.1` 或 `h2`，不记录查询串、cookie、authorization 或响应体。

### 1.3 范围边界

- **在范围内**：HTTP/1.1/2、HEAD 预热、DNS 解析预热、流式字节消费者、RFC 9218 `Priority` HTTP header 的 HTTP/2 受控发送、单元/集成测试。
- **不在范围内**：HTTP/3/QUIC、服务端优先级帧、请求取消协议、缓存的流式写入、跨 profile 共享预热、主文档 body 的增量 HTML parser。

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|---|---|---|
| 功能需求 | 是 | 研究文档 P2 与本 RFC §3 |
| 非功能需求 | 是 | 隔离、非阻塞与可观测性，见 §4 |
| 接口需求 | 是 | `zero-net` transport API，见 §5 |
| 过渡需求 | 是 | 保持 P0/P1 `ResourceLoader` 缓存语义，见 §7 |

## 3. 功能需求

### FR-201：执行连接与 DNS 预热

- **描述**：当页面发现 `link rel=preconnect` 或 `dns-prefetch` 时，宿主必须异步提交一次预热任务；预热不得读取或写入 HTTP 缓存。
- **优先级**：必须。
- **来源**：研究文档 P2。

**验收场景**：

```text
场景: preconnect 预热后主请求复用同一 async client
  假设 本地 HTTP fixture 记录方法和连接
  当 页面声明 link rel=preconnect 且随后请求同一 origin 的资源
  那么 fixture 先收到 HEAD，后收到 GET
  并且 ResourceLoader 缓存统计不包含 HEAD
  验证: zero-net 的 preconnect_uses_head_without_cache

场景: 预热失败不阻断页面加载
  假设 preconnect 指向不可达 origin
  当 页面开始加载
  那么 主文档/其他资源调度仍继续
  并且 只产生匿名失败 telemetry
  验证: zero-webview 的 preconnect_failure_is_non_fatal
```

### FR-202：流式响应消费者

- **描述**：页面子资源的流式路径必须通过 chunk callback 交付 body，`HttpClient` 不得额外聚合完整 body。
- **优先级**：必须。
- **来源**：研究文档 P2。

**验收场景**：

```text
场景: 流式下载按 chunk 写入消费者
  假设 fixture 分两次写入响应体
  当 调用流式子资源接口
  那么 消费者按原始顺序收到全部字节，且响应头提供实际协议版本
  验证: zero-net 的 send_async_stream_delivers_body_without_response_buffer

场景: 流式下载出错
  假设 fixture 在响应体中途关闭连接
  当 调用流式子资源接口
  那么 接口返回网络错误，已交付 chunk 不被写入 HTTP 缓存
  验证: zero-net 的 streamed_response_failure_does_not_cache
```

### FR-203：HTTP/2 Priority header

- **描述**：当调用方提供资源优先级且 HTTP/2 已启用时，传输层必须映射为 RFC 9218 `Priority` header；HTTP/1.1 不发送该 header。
- **优先级**：应该。
- **来源**：研究文档 P2；用户明确排除 HTTP/3。

**验收场景**：

```text
场景: HTTP/2 模式发送 Priority
  假设 transport 配置允许 HTTP/2
  当 Critical 与 Low 资源发起请求
  那么 请求分别携带确定的 urgency 值
  验证: zero-net 的 priority_header_maps_fetch_priority

场景: HTTP/1.1 fallback 不发送 Priority
  假设 ZERO_HTTP2=0
  当 资源发起请求
  那么 fixture 不收到 Priority header
  验证: zero-net 的 priority_header_is_absent_for_http1
```

## 4. 非功能需求

### NFR-201：安全与隔离

- **描述**：预热请求必须使用 `HEAD`、无 body、无自定义页面头；不得携带 cookie、authorization、referer，且不得写入或命中 HTTP 缓存。
- **测量标准**：fixture 断言方法与头；`ResourceLoader` stats 不变。
- **优先级**：必须。

### NFR-202：资源上界

- **描述**：预热使用既有 async runtime；同一 origin 同时最多一个预热任务，队列必须使用既有上限或在满时丢弃低优先级预热。
- **测量标准**：fixture 和 unit test 验证重复 hint 只发起一次。
- **优先级**：必须。

### NFR-203：可观测性

- **描述**：事件只记录 origin、动作、协议、耗时和结果；不得记录 URL query、header value 或 body。
- **测量标准**：匿名事件单测。
- **优先级**：必须。

## 5. 接口需求

### IF-201：预热 API

- **类型**：`zero-net` Rust API。
- **规格**：`HttpClient::preconnect_async(origin: &str) -> Result<HttpPreconnectResult, NetError>`；仅接受 `http`/`https` origin URL，发送无 body 的 `HEAD`，返回匿名协议/耗时结果。
- **错误处理**：非法 URL 返回 `NetError::UrlParse`；网络错误返回现有 `NetError`；页面层记录但忽略错误。
- **默认动作**：重复 origin 或资源预算已满时跳过预热并返回 `Skipped`，不阻塞调用方。

### IF-202：优先级 API

- **类型**：`zero-net` Rust API。
- **规格**：`HttpRequest` 由 `FetchPriority` 映射为 RFC 9218 urgency；仅当 `ZERO_HTTP2` 未禁用时在请求上设置 `Priority` header。
- **错误处理**：调用方显式提供 `Priority` header 时保留该值，不覆盖。
- **默认动作**：HTTP/1.1 fallback 不设置 header。

## 6. 约束与假设

### 6.1 必须约束

- 复用 `zero-net` 的 async reqwest client 和受限 Tokio runtime；不引入 HTTP/3/QUIC crate。
- 保持 `ResourceLoader` 的缓存、分区、Vary、unsafe invalidation 和 in-flight 合并语义。
- 预热失败、重复或队列饱和必须静默降级，不能改变页面加载结果。

### 6.2 禁止约束

- 不把 HEAD 预热结果写入 HTTP cache 或暴露给 Fetch API。
- 不从 preconnect/dns-prefetch 继承页面请求的 cookie、authorization、referer 或请求 body。
- 不把协议配置开关误报为实际协商协议；实际版本只从 response version telemetry 读取。

### 6.3 已定决策

- HTTP/3 暂不处理；不修改 `Cargo.toml` 引入 QUIC/HTTP/3 依赖。
- 预热以同一 async reqwest client 的 HEAD 请求实现，目的是预热其连接池；它不是裸 TCP/TLS preconnect，需在 telemetry 中区分为 `head-preconnect`。
- 流式路径不接入 HTTP cache，直到存在原子写入与取消设计。

### 6.4 技术约束与实现来源

| 能力 | 实现来源 |
|---|---|
| HTTP/1.1/2 请求和连接池 | 现有 `reqwest 0.12` async `Client` |
| 流式 chunk | 现有 `reqwest::Response::chunk` 与 `HttpClient::send_async_stream` |
| 异步任务上界 | `zero-net::client::async_runtime`（4 worker、32 blocking bridge） |
| 提示扫描 | `zero-engine::preload::scan_html_resource_hints` |
| 页面接线 | `zero-page-runtime::AsyncFetchHost` 与 webview/renderer host adapter |

### 6.5 假设

- 已验证：当前 `reqwest` 依赖未提供 HTTP/3/QUIC 能力。
- 待验证：同一 async reqwest client 在目标平台对 HEAD 后续 GET 的连接复用率；以 fixture 的连接标识测试，不把复用率写成硬性跨平台承诺。

## 7. RFC 设计

```text
HTML hint scanner
       │ preconnect / dns-prefetch
       ▼
AsyncFetchHost (non-blocking fire-and-forget)
       ▼
HttpClient async client pool ── HEAD / DNS lookup
       │                         │
       └── anonymous event ◄─────┘

resource fetch ── FetchPriority ──► request header mapping
       │
       └── stream consumer ◄── send_async_stream(chunk callback)
```

1. `AsyncPageLoad::begin_preload_hints` 解析所有 hint；preload 保持现有行为，preconnect/dns-prefetch 调用 host 的非阻塞方法。
2. in-process host 复用 `HttpClient` 的 async pool 提交预热；IPC host 将请求交给 browser process，且不创建页面可见 response。
3. async transport 完成时记录匿名结果；没有任何缓存写入或页面错误事件。
4. Priority header 映射只在配置已启用 HTTP/2 时生效，保留调用方显式 header；实际 response protocol 用现有 `HttpResponseHead::protocol` 记录。

## 8. 实施交接

| 顺序 | 文件/模块 | 职责 | 验证 |
|---|---|---|---|
| 1 | `crates/net/src/client.rs` | HEAD/DNS 预热、protocol/priority 映射、匿名结果 | `zero-net` TCP fixture |
| 2 | `crates/page-runtime/src/lib.rs` | 扩展 `AsyncFetchHost` 非阻塞预热契约 | trait adapter 编译测试 |
| 3 | `crates/webview/src/{async_load,net_pool}.rs` | 发现并提交 hint、流式消费入口 | webview async-load unit |
| 4 | `apps/renderer/src/ipc_fetch.rs`、`apps/browser/src/fetch_proxy.rs` | IPC preconnect 请求与无页面结果回传 | browser proxy unit |
| 5 | `tests/integration` | 本地 TCP 连接/缓存/失败验收 | guarded integration test |

首批提交：

1. `net: add async preconnect telemetry`。
2. `webview: execute connection hint preloads`。
3. `integration: cover preconnect isolation and failure`。

## 9. 风险与回滚

| 风险 | 缓解 | 回滚 |
|---|---|---|
| HEAD 被非标准服务器拒绝 | 失败静默，不影响正常 GET | 关闭 hint executor |
| 预热消耗连接预算 | 使用低优先级、dedupe 和队列上限 | 只保留 dns-prefetch |
| Priority header 兼容性 | 仅配置启用 HTTP/2 时发送，保留显式 header | 关闭 header 映射 |
| 流式写入半途失败 | 不写 HTTP cache；消费者自行丢弃不完整数据 | 回退全量体路径 |

## 10. Spec Lint 报告

### 结构完整性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 执行摘要存在性 | ✅ Pass | §0 定义范围、排除项和首步。 |
| 场景存在性 | ✅ Pass | FR-201～203 均有正常与异常场景。 |
| 测试绑定 | ✅ Pass | 每个场景绑定 zero-net 或 zero-webview 测试。 |
| TBD 清零 | ✅ Pass | HTTP/3 已明确排除；连接复用率仅为非阻塞测量假设。 |
| 实施交接完备 | ✅ Pass | §8 包含模块、顺序与验证。 |

### 一致性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 范围冲突 | ✅ Pass | §1.3 与 §6.2 均排除 HTTP/3/QUIC。 |
| 方案漂移 | ✅ Pass | §6.4 仅复用现有 reqwest/Tokio/ResourceLoader。 |
| 实现来源闭合 | ✅ Pass | §6.4 指明每个能力的仓内或依赖来源。 |
| 未验证细节泄漏 | ⚠️ Warning | HEAD 后续连接复用率依赖平台，验收只验证方法与非阻塞降级。 |

**汇总**：8 Pass / 1 Warning / 0 Fail / 0 Skip。**门禁判定**：允许实施。
