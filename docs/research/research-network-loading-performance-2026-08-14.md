# ZeroWeb 网络加载性能专项：统一调度、协议缓存与可量化验收

日期：2026-08-14
状态：P0/P1 实施中（2026-08-14 起）；P2 传输层迁移待独立 RFC
范围：`zero-net`、`zero-webview`、`zero-browser` 的页面主文档及子资源加载路径

## 实施进展（持续更新）

- 已完成：统一 `ResourceLoader` 入口、请求身份合并、全局与 origin 并发预算、同优先级 origin 轮转、请求缓存指令、缓存分区、浏览器页面 origin 分区传递、匿名加载生命周期事件（navigation/destination/origin/时序/字节/缓存结果/合并数）、`fetchpriority`（preload 与图片）及 lazy 图片低优先级。
- 已完成：本地 TCP fixture 覆盖 fresh hit、身份/Vary 隔离、缓存分区及匿名加载指标；加载器提供 cache/revalidate/network/only-if-cached、字节数与等待时长聚合计数。
- 未完成：真正 async/流式传输、连接预建、HTTP/3 与 RFC 9218 `Priority`；这些需要引入 runtime 并重构传输 API，仍按本文 P2 边界单独实施。

## 阅读指引：来源分级

| 分级 | 本文含义 |
|---|---|
| 🟢 一手事实 | 当前仓库源码或 IETF/WHATWG/Chromium 的规范、源码、设计文档 |
| 🟡 外部搜索 | Google web.dev 的工程实践文章 |
| 💡 推理 / 作者综合 | 基于上述事实针对 ZeroWeb 的设计，不代表 Chromium 的逐行实现 |
| ⚠️ 假设 | 尚需压测或产品决策验证的阈值、收益目标 |

## 0. 执行摘要

目标不是再增加一个“下载线程池”，而是把所有可缓存的 GET 资源收敛到**一个浏览器上下文级 Resource Loader**：先以完整请求身份查缓存，再以完整身份合并相同在途请求，最后按资源关键性、公平性和协议能力调度网络传输。

现有仓库已经具备可复用基础：`HttpCache`（内存 LRU + 可选磁盘、`Vary`、`ETag`/`Last-Modified` 再验证）、`PerOriginFetchScheduler`（每 origin 默认 6、优先级队列、部分去重），以及 WebView 的 preload/并行 CSS、字体、图片加载。专项应优先修正这些能力的边界并统一入口，而不是重写缓存。[1][2]

推荐分三期：

1. **P0 正确性与统一入口**：引入 `RequestIdentity`，解决当前以 URL 合并在途请求的错误；资源请求统一经缓存感知调度器；补齐请求缓存指令与失效。
2. **P1 调度质量**：把“每 origin 6”升级为“全局预算 + origin 公平队列 + 动态优先级”；接入 HTML 的 `fetchpriority`、preload、lazy 与可见性信号。
3. **P2 传输与体验**：迁移到真正 async/流式网络层、度量 HTTP/2/HTTP/3 能力并支持 preconnect；在证据充分后再发送 RFC 9218 `Priority` 头。

首批可验收成果：同一“等价 GET”只发一次网络请求；不同 `Authorization`、`Cookie`、`Accept-Language` 或 `Vary` 变体绝不串用；`Cache-Control: no-store/no-cache/max-age/must-revalidate` 和 304 合并行为通过本地 fixture；关键 CSS 在拥塞下先于图片启动；仪表盘能分别报告内存命中、磁盘命中、再验证、合并和队列等待。[3][4]

## 1. 调研范围与术语

### 1.1 5W1H

| 维度 | 结论 |
|---|---|
| What | 页面导航及子资源的并发请求、优先级、连接复用与 HTTP 私有缓存。 |
| Why | 缩短关键渲染路径、避免重复下载、降低带宽与源站压力，同时不破坏 HTTP/安全语义。 |
| Where | `crates/net` 为共享核心；`crates/webview/src/net_pool.rs` 和 `apps/browser/src/fetch_proxy.rs` 为两条宿主适配路径；`async_load.rs` 是资源发现端。 |
| When | 以 2026-08-14 的项目代码和仍有效的 HTTP/HTML 标准为基线。 |
| Who | 浏览器用户、WebView 宿主、渲染/网络/安全维护者及性能测试维护者。 |
| How | 先正确性后吞吐；每期以协议 fixture、端到端加载瀑布和稳定指标验收。 |

### 1.2 术语映射

| 用户诉求 | 业界术语 | 本专项定义 |
|---|---|---|
| 多个并发拉取资源 | resource scheduling / connection pooling | 资源发现后异步入队，受全局及 origin 配额控制。 |
| HTTP 缓存协议 | private HTTP cache / cache validation | 按 RFC 9111 复用、再验证、更新与失效。 |
| 同一资源不要重复拉取 | request collapsing / in-flight coalescing | 仅对请求身份完全等价的安全 GET 合并。 |
| 关键资源先加载 | fetch priority / critical request chain | CSS、主文档、LCP 候选等优先于非关键图片与预取。 |

> **📌 来源说明（第 1 章）**
>
> - 🟢 一手事实 [1][2][3][4]：HTTP 缓存与浏览器资源加载的职责边界。
> - 💡 推理：专项边界与术语到 ZeroWeb 模块的映射。

## 2. 当前实现盘点

### 2.1 已有能力

| 层 | 现有实现 | 结论 |
|---|---|---|
| 并发 | `PerOriginFetchScheduler` 按 `scheme://host:port` 计数，默认 6；不同 origin 并行，队列选最高 `FetchPriority`。 | 已实现 HTTP/1.x 常见的 origin 并发保护，但没有全局预算/公平策略。 |
| 去重 | `submit_shared_*` 的 `pending: HashMap<String, _>` 广播一次结果。 | 有请求合并雏形，但键仅是 URL，语义不足。 |
| HTTP 缓存 | `HttpCache` 有 LRU、内存/磁盘、`Vary` 选择、`Age`、`Expires`、`max-age`、`no-cache`、`no-store`、`ETag`/`Last-Modified`、304 元数据合并。 | 私有缓存的核心骨架已在，适合增量补齐。 |
| 资源发现 | `async_load.rs` 并行启动 stylesheet，随后并行图片/字体；`engine::preload` 识别 `preload/prefetch/preconnect/dns-prefetch`。 | 已可产生并发；`preconnect/dns-prefetch` 目前只是解析提示，尚未建立连接。 |
| 宿主适配 | `net_pool` 走共享 scheduler/cache；browser `TabFetchProxy` 有缓存查找、IPC pending 与导航 epoch。 | 两条路径分别复制了“缓存→调度→304→写回”的流程，未来易漂移。 |

### 2.2 必须先修正的边界

1. `pending` 仅以 URL 为键，忽略方法、请求头、凭据、缓存模式和顶级站点隔离。不同身份/变体请求可能错误合并；这既是缓存正确性也是隐私边界问题。RFC 9111 至少要求缓存键包含方法和目标 URI，并按 `Vary` 区分；私有缓存还可以为隐私加入 referring-site（双键）隔离。[3]
2. `submit_shared_with_priority_and_headers` 的同 URL 后到请求会复用第一个请求的条件头及优先级，无法安全合并 revalidate 与普通请求，也不能提高一个已排队资源的优先级。
3. 调度器以 `std::thread::spawn` 承载每一个请求，`HttpClient` 为 blocking reqwest；高并发页面可能有大量线程和完整响应体驻留。Chromium 的网络栈明确要求网络线程不阻塞，并将异步事件与网络日志作为基础能力。[2]
4. `max_connections_per_origin = 6` 是 HTTP/1.x 兼容保守值；对 HTTP/2/3 的一条多路复用连接不能等同为“允许 6 个请求”。HTTP 层的资源优先级仍有价值，但连接/流配额必须由协商协议能力驱动。[5]
5. 当前缓存策略仍未形成覆盖矩阵：请求侧 `no-cache/no-store/only-if-cached/max-age/min-fresh/max-stale`，响应侧字段限定的 `no-cache`、启发式新鲜度、unsafe method 失效、范围请求及离线 stale 策略均未作为专项验收项。[3]
6. `FetchPriority` 可以从内部 header 或 URL 扩展名猜测资源类型；该猜测只能作兜底，尚未形成 HTML `fetchpriority` 到调度器的可靠信号链。[7][8]

### 2.3 不应在 P0 做的事

- 不替换 DOM/CSS/渲染管线，不改变页面资源发现顺序。
- 不在未确认底层协议支持前承诺 HTTP/3、QUIC 或 RFC 9218 帧。
- 不把 URL 级去重扩展到 POST、带 body 的请求或跨安全上下文请求。
- 不实现 `stale-while-revalidate` 等 RFC 扩展，除非在 P2 另立兼容性设计。

> **📌 来源说明（第 2 章）**
>
> - 🟢 一手事实：`crates/net/src/{fetch_scheduler,http_cache,cache_policy,resource_policy,client}.rs`，`crates/webview/src/{net_pool,async_load}.rs`，`apps/browser/src/fetch_proxy.rs`；[2][3][5]。
> - 💡 推理：风险排序及 P0 范围。

## 3. 外部实践与协议约束

### 3.1 Chromium 可借鉴的边界，不应照抄的实现

Chromium 将 HTTP Cache 放在网络栈内，位于资源加载器之前：一个 active cache entry 管理 writer、readers 与等待事务，实施单写多读，从而避免同一资源并行网络下载。[1] 它的网络栈以非阻塞异步操作和 NetLog 记录为基础。[2] 这些是本专项应复用的**边界模式**：缓存与在途合并是同一个资源加载器的状态，不应该散落在 WebView/浏览器代理中。

Chromium 仍有 `ResourceScheduler` 并支持 request priority 和 reprioritize，因此优先级不是“入队时固定数字”。[6] 对 ZeroWeb 的对应设计是：优先级由资源类型、HTML hint、可见性及导航状态合成，进入等待队列后的有效优先级可以提升，但已发到网络层的请求 P1 不抢占。

### 3.2 HTTP 缓存的不可破坏约束

RFC 9111 定义浏览器为 private cache；缓存最小键为 method + target URI，`Vary` 指定的请求头需参与变体选择，private cache 可以额外按顶级站点隔离。[3] stale 响应不能任意返回；`no-cache` 和 `must-revalidate` 会要求成功验证，断网时也不能绕过 `must-revalidate`。[3] 再验证时应发送 `If-None-Match`，适用时还应发送 `If-Modified-Since`，收到 304 后更新受影响的存储元数据。[3][4]

这说明“缓存命中”必须是一个带原因的结果：`FreshMemory`、`FreshDisk`、`Revalidated304`、`Network200`、`Bypass`、`OnlyIfCachedMiss`。将它们压成 `Hit/Miss` 会使性能指标和行为排障失真。

### 3.3 优先级与资源提示

HTML 标准规定 `link rel=preload` 按 `as` 与 `fetchpriority` 提前抓取当前导航高度可能需要的资源；而 prefetch 可被 UA 延迟以让位给当前文档必要请求。[7] `fetchpriority=high|low|auto` 是对同目的资源的相对提示，不是强制指令。[8] HTTP/2/3 的 RFC 9218 提供 urgency/incremental 和 `Priority` header，但它只是向服务器表达偏好，客户端仍要实施自己的调度，服务器也不保证遵从。[5]

> **📌 来源说明（第 3 章）**
>
> - 🟢 一手事实 [1]-[8]：Chromium 缓存/调度架构、RFC 9111/9110/9218、WHATWG HTML。
> - 🟡 外部搜索 [9]：Chrome 对 fetch priority 与 LCP 场景的工程说明。
> - 💡 推理：将通用架构原则映射为 ZeroWeb 的边界；不声称与 Chromium 实现相同。

## 4. 推荐目标架构（作者综合）

```text
HTML parser / preload scanner / JS fetch / navigation
                    │ ResourceRequest { identity, intent, priority }
                    ▼
        ┌───────────────────────────────┐
        │ BrowserContext ResourceLoader │  唯一入口、可观测
        └──┬──────────────┬─────────────┘
           │              │
   Cache decision      In-flight table
  (private cache)   (同 identity 合并/提升)
           │              │
       fresh ─────► immediate response
           │ miss/stale   │
           └──────┬───────┘
                  ▼
       Fair priority scheduler
       global budget + per-origin budget
                  ▼
   async transport + connection pool
                  ▼
      response store / 304 merge / invalidate
```

### 4.1 核心数据模型（接口优先）

```rust
struct ResourceRequest {
    identity: RequestIdentity,
    url: Url,
    method: SafeCacheableMethod, // P0 只允许 GET/HEAD
    headers: HeaderMap,
    cache_mode: CacheMode,
    credentials_mode: CredentialsMode,
    partition: CachePartitionKey,
    destination: ResourceDestination,
    priority: RequestPriority,
    initiator: Initiator,
}

struct RequestIdentity {
    method: SafeCacheableMethod,
    normalized_url: UrlWithoutFragment,
    partition: CachePartitionKey,
    credentials_scope: CredentialsScope,
    // 在确定 Vary 前不可只靠 URL 合并：保存所有会影响响应的请求语义。
    request_fingerprint: RequestFingerprint,
}

enum CacheDecision {
    Fresh(CachedResponse, CacheSource),
    Revalidate { cached: CachedResponse, conditional: HeaderMap },
    Network,
    FailOnlyIfCached,
}
```

**Decision D-01**：P0 合并键采用完整 `RequestIdentity`，而非 URL。应包含规范化 URL（无 fragment）、GET/HEAD、缓存分区、凭据范围、请求中可能影响响应的头的 fingerprint；存储后按 `Vary` 选择候选。没有足够身份信息时，宁可不合并。

**Decision D-02**：一个 `BrowserContext` 持有一个 `ResourceLoader`。普通 profile 共享一个 private persistent cache；隐私 profile 使用独立内存 cache 和独立分区。`zero-webview` 与 `TabFetchProxy` 只实现宿主/IPC adapter，不能再自行拼接 cache + scheduler 流程。

**Decision D-03**：P0 保留 `HttpClient` 传输实现、先提供无阻塞 API 外观；P2 再把实现替换为 async 流式传输。这样不会把高风险运行时迁移和缓存语义修复捆绑在一起。

### 4.2 调度算法

每次接收请求执行如下伪代码：

```text
request = normalize_and_validate(input)
if cache_mode == OnlyIfCached:
    return cache.lookup_or_504(request)

decision = cache.lookup(request)
if decision is Fresh: return decision.response

key = request.identity
if in_flight[key] exists:
    subscribe(request); raise_queued_priority_if_needed(request); return

create in_flight[key]
enqueue(decision.network_request, effective_priority(request))

on_network_complete:
    if 304: cache.merge_304_and_reply_all()
    else: cache.store_or_invalidate_and_reply_all()
```

队列选择采用“优先级优先、同优先级按 origin 轮转”的 two-level policy：

- 全局 `max_in_flight` 是防线程/内存失控的硬上限；P0 以配置值实现，初始值仅作 ⚠️ 假设，必须通过基准确定。
- 每 origin 上限在 HTTP/1.x 保留 6；HTTP/2/3 使用较高**逻辑请求**上限，但同时受全局上限控制，且不得假设一个连接能无限承载 body。
- `Critical`（主文档、render-blocking CSS）优先；`High`（脚本、字体、明确 high hint）；`Medium`（默认图片）；`Low/Idle`（prefetch、低 hint）只在当前导航的关键队列没有饥饿时运行。
- 同一请求后续出现更高优先级时只提升排队任务；已运行任务不取消重发。导航取消只取消订阅者/回调，P0 不要求终止共享网络传输。

### 4.3 HTTP 缓存语义矩阵

| 场景 | P0 行为 | 验收 |
|---|---|---|
| `max-age`/`Expires` fresh | 从内存或磁盘直接返回，生成正确 `Age`。 | 服务器请求数不增加。 |
| stale + `ETag`/`Last-Modified` | 合并相同 revalidation；发送条件 GET；304 合并元数据后回同一 body。 | N 个并发消费者仅 1 次条件请求。 |
| response `no-store` | 不写内存/磁盘/在途完成后的可复用缓存。 | 二次请求重新到网络。 |
| response/request `no-cache` | 可存储但再次使用前必须成功验证；请求 `no-cache` 不直接给 fresh 条目。 | 二次请求带 validator。 |
| `must-revalidate` | 失效后验证失败或离线时返回网络错误，不能返 stale。 | fixture 断网断言失败。 |
| `Vary: Accept-Language` | 仅匹配相同变体；不匹配为 miss。 | 两语言 body 不串用。 |
| unsafe method 成功 | write-through 后使目标 URI 及已知关联 URI 的缓存变为需验证/删除。 | 后续 GET 不使用旧 body。 |
| `only-if-cached` miss | 返回 504 语义结果，不访问网络。 | server 请求数为 0。 |

范围请求/206、启发式 freshness、RFC 5861 stale 扩展、跨进程缓存事务恢复列为 P2；它们不能被当前 P0 的“HTTP cache 已完成”表述掩盖。

### 4.4 观测与性能指标

每个请求产生结构化事件，字段至少包含 `navigation_id`、资源 destination、origin、queue_wait_ms、network_ms、bytes、priority、protocol、cache_outcome、coalesced_subscriber_count`。日志禁止记录 cookie、authorization、完整查询参数或响应体。

| 指标 | 定义 | P0 基线/门槛 |
|---|---|---|
| cache hit ratio | `FreshMemory + FreshDisk` / cacheable lookup | 建立分页面、分资源类型基线；不先设虚假百分比目标。 |
| revalidation savings | `304 bytes_saved` 与 304 次数 | 必须可从 fixture 和 telemetry 对账。 |
| request coalescing ratio | `(subscribers - network_transactions) / subscribers` | 重复 preload/消费场景必须大于 0。 |
| queue wait p50/p95 | 入队到开始传输的时长 | Critical 的 p95 不得因 Low/Idle 饥饿而恶化。 |
| critical-start order | CSS/主文档先于非关键图的比例 | 受控拥塞 fixture 中为 100%。 |
| peak in-flight / thread count | 每导航及全局峰值 | 不得超过配置上限；P0 记录并设测试断言。 |

> **📌 来源说明（第 4 章）**
>
> - 🟢 一手事实 [1][2][3][5][6][7][8]：缓存合并、私有缓存键、HTTP 优先级及 HTML hint 的约束。
> - 💡 推理 / 作者综合：`ResourceLoader`、数据模型、队列策略、指标与分期；阈值标为待验证，不把推测当作 Chromium 行为。

## 5. 可实施 Spec + RFC

### 5.1 需求与约束

| 编号 | 类型/优先级 | 需求 |
|---|---|---|
| FR-001 | 功能 / Must | 对所有页面 GET 子资源，先执行私有 HTTP 缓存决策，再执行网络调度；缓存新鲜命中不得占用网络槽位。 |
| FR-002 | 功能 / Must | 只合并 `RequestIdentity` 完全等价的在途安全请求；不同分区、凭据范围、方法或影响响应的头不得合并。 |
| FR-003 | 功能 / Must | 实现 §4.3 P0 矩阵中的缓存行为，并保留 `Vary` 和 304 元数据合并。 |
| FR-004 | 功能 / Must | 调度器必须同时执行全局与 origin 并发预算、优先级和同优先级 origin 公平性。 |
| FR-005 | 功能 / Should | 支持 HTML preload/prefetch 及 `fetchpriority` 映射到请求 destination 与优先级；hint 只能影响排序，不能绕过安全或缓存语义。 |
| FR-006 | 可观测 / Must | 产生不含敏感数据的请求生命周期指标和可聚合的 cache/scheduler 计数。 |
| NFR-001 | 安全 / Must | 缓存和在途合并不得跨隐私 profile、顶级站点分区、凭据范围或 `Vary` 变体泄露响应。 |
| NFR-002 | 可靠性 / Must | 导航取消不得错误投递旧 navigation 的结果；同一消费者完成一次且仅一次。 |
| NFR-003 | 性能 / Must | 所有并发/队列/线程数均有上界；P0 压测在上界内完成，不产生无界线程。 |

### 5.2 验收场景

**FR-001：新鲜缓存不占网络槽位**

```gherkin
场景: 从 fresh cache 同时满足两个资源请求
  假设 fixture 为 GET /a.css 返回 Cache-Control: max-age=60
  并且 /a.css 已缓存
  当 页面同时请求 /a.css 两次
  那么 fixture 服务器不收到新请求
  并且 scheduler 的 in-flight 计数不增长
  验证: zero-integration-tests 的 cache_fresh_hit_bypasses_scheduler
```

**FR-002：在途合并保护隔离**

```gherkin
场景: 同身份请求合并而不同变体不合并
  假设 /greeting 响应 Vary: Accept-Language
  当 两个 en-US GET 并发且一个 zh-CN GET 并发
  那么 en-US 只产生一次网络事务且两个消费者收到同一响应
  并且 zh-CN 产生独立网络事务且不收到 en-US body
  验证: zero-integration-tests 的 coalesce_respects_request_identity_and_vary
```

**FR-003：再验证与禁缓存**

```gherkin
场景: stale ETag 条目收到 304
  假设 /app.js 已有 stale body 和 ETag
  当 两个消费者并发请求 /app.js 且服务器返回 304 和新 Cache-Control
  那么 服务器只收到一次 If-None-Match 请求
  并且 两个消费者收到旧 body 与更新后的缓存元数据
  验证: zero-net 的 stale_etag_revalidation_is_coalesced

场景: no-store 响应
  假设 /token 返回 Cache-Control: no-store
  当 连续两次请求 /token
  那么 服务器收到两次请求且缓存中不存在该条目
  验证: zero-net 的 no_store_never_persists
```

**FR-004/005：拥塞时关键资源优先**

```gherkin
场景: 关键 CSS 先于低优先级图片开始
  假设 同一 origin 的 fixture 将首批请求阻塞并记录到达顺序
  当 页面包含 8 个低优先级图片、一个 stylesheet 和 fetchpriority=high 的 preload
  那么 stylesheet 与 preload 在任何图片之前获得可用槽位
  并且 每个 origin 的运行数及全局运行数不超过配置
  验证: zero-integration-tests 的 critical_resources_win_under_contention
```

### 5.3 实施交接

| 顺序 | 模块/文件 | 改动职责 | 验证 |
|---|---|---|---|
| 1 | `crates/net`（新增 `resource_loader.rs`） | 定义请求身份、缓存决策、在途表、调度 API 与 telemetry trait。 | unit：identity/merge/priority。 |
| 2 | `crates/net/{fetch_scheduler,http_cache,cache_policy}.rs` | 迁移现有实现，补请求指令、unsafe invalidation 与 global/fair budget。 | raw TCP fixture unit tests。 |
| 3 | `crates/webview/src/net_pool.rs` | 退化为 `ResourceLoader` adapter，删除本地 cache/scheduler 编排。 | webview async-load 回归。 |
| 4 | `apps/browser/src/fetch_proxy.rs` | 同样只负责 IPC、安全 context/取消映射；把 request headers、method、partition 明确传入。 | browser proxy integration。 |
| 5 | `crates/engine/src/preload.rs`、`async_load.rs` | 读取 `fetchpriority`，传递 destination/initiator；不改变安全检查顺序。 | HTML hint/unit + controlled waterfall。 |
| 6 | `tests/integration`、`tests/wpt-runner` | 协议 fixture、竞争/取消、性能瀑布断言；有合适上游用例则按账本导入。 | `make test`、scoped reftest、`make bench-gate`。 |

首批提交建议：

1. `net: define request identity and coalescing contract`（无行为变更的类型与单测）。
2. `net: route cache-aware GETs through one resource loader`（替换双宿主复制流程）。
3. `net: honor request cache directives and safe invalidation`（协议 fixture）。
4. `webview: propagate resource destination and fetch priority`（HTML/加载测试）。
5. `integration: add network loading performance fixtures and metrics`（性能门禁基线）。

### 5.4 风险、回滚及决策点

| 风险 | 缓解 | 回滚点 |
|---|---|---|
| 统一入口遗漏调用方 | 先 `rg HttpClient::send/get` 建立迁移清单，过渡期埋点 report bypass。 | feature gate 回退到旧 adapter。 |
| 错误合并造成数据泄露 | P0 默认“不确定不合并”，identity 测试覆盖 cookie/auth/vary/profile。 | 禁用 in-flight coalescing，不影响缓存。 |
| 优先级导致低优先级饥饿 | origin 轮转 + aging，记录 queue wait p95。 | 恢复 FIFO 同时保留上限。 |
| async 迁移扩大范围 | P0 不迁移 transport；P2 独立 RFC/基准。 | 保持 blocking adapter。 |

待用户确认的非阻塞产品决策：是否在 P0 即启用按顶级站点分区的 disk cache（安全优先，可能降低跨站缓存命中）；是否把 telemetry 导出到开发者工具或只保留 `tracing`/测试接口。

> **📌 来源说明（第 5 章）**
>
> - 🟢 一手事实 [1]-[8]：设计中所有协议及浏览器行为约束。
> - 💡 推理 / 作者综合：ZeroWeb FR/NFR、接口、测试和实施顺序。

## 6. 方案比较与结论

| 方案 | 优点 | 缺点 | 裁决 |
|---|---|---|---|
| A. 仅把每 origin 上限调大 | 改动最小。 | 放大线程/内存，无法保证关键资源、缓存和隔离正确。 | 不采用。 |
| B. 保留现有部件、增加统一 ResourceLoader | 重用已验证缓存与队列能力；先修安全边界，再优化。 | 需迁移两条调用路径与补 fixture。 | **推荐 P0/P1。** |
| C. 先全面改 async + HTTP/3 再做缓存 | 长期传输上限更高。 | 风险和调试面太大，推迟即时收益。 | 作为独立 P2。 |

> ### 💡 推理分析：为什么选 B
>
> **观察**：本仓已经有 HTTP 缓存、资源优先级和异步资源发现，但编排复制在两个宿主路径，且在途合并键只有 URL。Chromium 把 cache/合并置于共享网络事务边界；RFC 9111 对变体、验证和复用有严格约束。[1][3]
>
> **推理**：先统一这一边界既能立即修正跨路径重复下载，也把协议正确性集中到一个可测试模块；不需要一次引入新的 runtime 或传输协议。
>
> **结论**：以方案 B 启动专项，P0 的完成定义是“正确且可观察的统一加载器”，不是“并发数更大”。

> **📌 来源说明（第 6 章）**
>
> - 🟢 一手事实 [1][3]：共享缓存/合并与 RFC 缓存约束。
> - 💡 推理：方案裁决。

## 7. 设计质量检查（Spec Lint）

| 检查 | 裁决 | 依据 |
|---|---|---|
| 执行摘要存在 | ✅ Pass | §0。 |
| 每个 Must FR 有验收场景 | ⚠️ Warning | FR-001 至 FR-005 有场景；FR-006 需在实施时将 telemetry schema 写成可断言快照测试。 |
| 异常路径覆盖 | ✅ Pass | §4.3 / §5.2 覆盖 only-if-cached、no-store、must-revalidate、Vary。 |
| 测试绑定 | ✅ Pass | §5.2 为每个场景指定 crate/测试名。 |
| 无阻塞 TBD | ✅ Pass | §5.4 的产品问题均不阻塞 P0 的安全默认行为。 |
| 约束覆盖 | ✅ Pass | NFR-001 至 NFR-003 分别由 §5.2/§5.4 覆盖。 |
| 实施交接与首步 | ✅ Pass | §5.3 的顺序 1 及验证明确。 |
| 模糊性能描述 | ⚠️ Warning | 绝对吞吐/延迟数值必须以本仓基线和目标设备测量后确定；本文刻意未编造阈值。 |
| 实现来源闭合 | ✅ Pass | P0 复用 `zero-net` / `reqwest`；P2 async/HTTP3 被明确延后。 |
| 方案漂移 | ✅ Pass | §2.3 明确排除 P0 的 HTTP/3 与大规模重构。 |

## 8. 参考资料

| # | 来源 | 类型 | 使用章节 |
|---|---|---|---|
| [1] | [Chromium HTTP Cache](https://www.chromium.org/developers/design-documents/network-stack/http-cache/) | 🟢 Chromium 设计文档 | §0、§3、§4、§6 |
| [2] | [Chromium Network Stack](https://www.chromium.org/developers/design-documents/network-stack/) | 🟢 Chromium 设计文档 | §0、§2、§3 |
| [3] | [RFC 9111: HTTP Caching](https://www.rfc-editor.org/rfc/rfc9111.html) | 🟢 IETF 标准 | §0、§2-§6 |
| [4] | [RFC 9110: HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110.html) | 🟢 IETF 标准 | §3、§4、§5 |
| [5] | [RFC 9218: Extensible Prioritization for HTTP](https://www.rfc-editor.org/rfc/rfc9218.html) | 🟢 IETF 标准 | §2-§4 |
| [6] | [Chromium ResourceScheduler source](https://chromium.googlesource.com/chromium/src/%2B/HEAD/services/network/resource_scheduler/resource_scheduler.cc) | 🟢 Chromium 源码 | §3、§4 |
| [7] | [WHATWG HTML: link preload](https://html.spec.whatwg.org/dev/links.html) | 🟢 HTML 标准 | §3、§4 |
| [8] | [WHATWG HTML: fetch priority](https://html.spec.whatwg.org/multipage/urls-and-fetching.html) | 🟢 HTML 标准 | §3-§5 |
| [9] | [web.dev: Fetch Priority API](https://web.dev/articles/fetch-priority) | 🟡 Google 工程实践 | §3 |
| [10] | [Chromium Preconnect](https://www.chromium.org/developers/design-documents/network-stack/preconnect/) | 🟢 Chromium 设计文档 | §2、P2 背景 |
