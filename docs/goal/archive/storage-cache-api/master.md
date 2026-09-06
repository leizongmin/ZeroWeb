# Cache API 真实化 — 运行时控制面板（master.md）

> **▶ 已归档（2026-09-06，模式 A 整树归档）**：本控制面随 goal 完成整体迁入
> `docs/goal/archive/storage-cache-api/`，只读保留，不再更新。终态：DC-1~DC-4 全满足，
> window 面 39 case / 449 subtest 全绿。

**入口文档**: [storage-cache-api.md](../storage-cache-api.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-06（DC 收口审计：testharness 账本补齐 5 条 + 449/449 复验 + 全工作区门禁；同日模式 A 归档）

---

## 当前状态

**专项定位**：存储方向三拆之二。把页面 `caches`（CacheStorage/Cache）从**零接线**
接到 zero-storage `cache_api.rs` 并补持久化，WPT `cache-storage`（window 面）真实用例驱动。
2026-08-22 已完成 WebView/in-process 页面侧初始桥接：
`caches.open/has/delete/keys/match` 与 `Cache.put/match/matchAll/delete/keys` 经 host
callback 进入共享 `StorageManager`，origin 由宿主页面 URL 推导；`Cache.add()` /
`Cache.addAll()` 已复用页面 `fetch()` 与 `Cache.put()` 完成 GET 成功响应的 fetch→store
链路。`ignoreSearch`/`ignoreMethod` 查询选项已在页面 Cache API 与 Service Worker runtime
Cache API 中接入；`ignoreVary` 已基于请求头快照和响应 `Vary` 匹配语义落地。
共享 `zero-storage::Cache::put()` 已拒绝非 GET request、非 HTTP(S) URL、206 Partial
Content response 与 `Vary: *` response，页面 `Cache.addAll()` 已改为先完成全部 fetch/
cacheability 校验再串行写入，避免失败批次部分落库。
随后补齐 page Cache API 读回链路的 `Response.type` 元数据保真：CacheStorage host match/
matchAll 结果改用 Cache 专属 `__zwcr:` wire payload 携带 `response_type`，页面 shim 兼容
旧 `__zwfr:` fixture 并将 type 恢复到返回的 `Response`，`Response.clone()` 也保留非 error
type；page-runtime host 会拒绝未知 response type 字符串。
WPT 扩面时发现上游 `simple_entries` 会把 `Response.error()` 作为普通缓存条目预置，
因此已按 WPT 调整为允许 `Cache.put(..., Response.error())` 写入并读回 `type == "error"`；
Service Worker `FetchEvent.respondWith()` 仍保持 200..599 响应结算限制，CacheStorage
传输层单独允许 status 0 的 error filtered response。
`cache-put.https.any.js` 扩面补齐页面 `Response.bodyUsed` / body stream 锁定、
`Response.redirect()`、Blob/FormData response body 序列化，以及 opaque filtered response
保留内部 206 / `Vary: *` 元数据写入 CacheStorage 的共享语义。
`cache-add.https.any.js` 扩面补齐 body-less `Request.text()` 不应置位 `bodyUsed`、
`Cache.addAll()` 对 `undefined` request list entry 的 TypeError、以及根据 fetch 后
response `Vary` 头判定批量请求重复的语义。
2026-08-23 已接入 33 个上游 CacheStorage window runner WPT 基线，WebIDL
brand、缺参 TypeError、`CacheStorage.keys()` 创建顺序、Vary 匹配、delete-dooming
生命周期、DOMString code-unit name wire、`Cache.matchAll()` 查询矩阵与 `CacheStorage.match()`
跨 cache/cacheName 查询、`Cache.match()` URL/fragment/opaque Vary/MIME/fetched response URL
矩阵、`Cache.put()` 可缓存性/响应体消费矩阵、`Cache.add()`/`Cache.addAll()` 矩阵修复后
继续补入 `common.https.window.js`、`cache-api-nested-worker.https.html` 与 9 个
`worker/*.https.html` Dedicated Worker wrapper case，补齐 `cache-abort` 的 window 与
Dedicated Worker 覆盖，并新增 9 个 `window/*.https.html` wrapper case；
`window/sandboxed-iframes.https.html` 已固定 sandbox iframe 无 `allow-same-origin`
时的 CacheStorage `SecurityError`、有 `allow-same-origin` 时可访问语义；另已加入
`cache-storage-buckets.https.any.js`，页面 shim 暴露最小
`navigator.storageBuckets` / `StorageBucket.caches`，以 bucket 名 UTF-16 code unit 前缀隔离
同名 cache，bucket 删除后旧 `bucket.caches` 操作按 WPT 期望 reject `UnknownError`。基线
固定 Window / Dedicated Worker / nested Dedicated Worker 共享同一 CacheStorage owner 的路径，
双跑稳定为 431 subtest / 431 Pass / 0 Fail。
2026-08-24 补入上游 `common.https.html` HTML wrapper，与已导入的
`common.https.window.js` 共同固定 Window 读回 Dedicated Worker 写入 cache 条目的共享 owner
路径，当前 CacheStorage window runner 基线为 34 case / 432 subtest / 432 Pass / 0 Fail。
2026-08-31 补齐页面 `fetch()` 的 `basic`/`cors`/`opaque`/`opaqueredirect` filtered
response 生成矩阵：JS shim 将 request `mode` / `redirect` 经 `__zw_fetch` wire 传给 host，
同源 fetch 生成 `basic`，跨源 CORS fetch 生成 `cors`，跨源 `no-cors` fetch 生成 `opaque`，
`redirect: "manual"` 的 30x response 生成 `opaqueredirect`；`opaque` 与 `opaqueredirect`
均保留隐藏内部 status/header/body 供 CacheStorage 写入，页面可见 response 仍投影为 status 0、
空 body、隐藏 headers。CacheStorage window runner 加入 1 个 ZeroWeb 回归 fixture 固定
fetch→CacheStorage round-trip，当时基线为 35 case / 436 subtest / 436 Pass / 0 Fail。
随后补入上游 top-level `sandboxed-iframes.https.html`，与既有
`window/sandboxed-iframes.https.html` 共同固定 sandbox iframe CacheStorage 安全边界，当前
CacheStorage window runner 基线为 36 case / 438 subtest / 438 Pass / 0 Fail。
随后补入上游 top-level `credentials.https.html`，复用已通过的 Service Worker
`credentials-worker.js` / `credentials-iframe.html` 资源闭包，固定受控 iframe XHR
credentialed request URL 作为 Cache key 在页面 runner 下也可经 worker fetch interception、
`Cache.put()`、`Cache.match()` / `Cache.matchAll()` / `CacheStorage.match()` 与
`Cache.keys()` 往返保真，当前 CacheStorage window runner 基线为 37 case / 439 subtest /
439 Pass / 0 Fail。
随后补入上游 top-level `cache-abort.https.any.js`，复用已通过的 abort 动态 fetch fixture，
把 `Cache.put()` / `Cache.add()` / `Cache.addAll()` 的 already-aborted、same-task abort
与 headers-received abort 行为从 wrapper 覆盖提升到 `.any.js` window 入口，当前
CacheStorage window runner 基线为 38 case / 448 subtest / 448 Pass / 0 Fail。
随后补入上游 `crashtests/cache-response-clone.https.html` no-harness crashtest，
runner 对无 testharness 页改为等待 `<html class="test-wait">` 清除后才判 pass，并收紧
脚本抛错不再假绿；同时模块脚本转换支持顶层 `await` 的 async IIFE 执行路径。该用例固定
`Cache.add("")` / `Cache.match("")` 后对 cached `Response.body` 打开 reader、再 `clone()`
并继续 read 的不崩溃路径，当前 CacheStorage window runner 基线为 39 case / 449 subtest /
449 Pass / 0 Fail。
M3 首片已补齐 page/WebView `StorageManager` owner 的 per-origin CacheStorage 持久化：
CacheStorage 以 origin hash `.cache` 文件落盘，请求/响应元数据和 body bytes JSON 保真，
写入采用临时文件 + sync + 原子替换，并在启动时清理 `.tmp` / 恢复 `.bak`；页面 host 的
`caches.open()`、`Cache.put()`、`Cache.delete()` 与 `caches.delete()` 均改为候选状态写盘
成功后再替换 live state，I/O 错误映射为 Promise `UnknownError`。Browser normal profile
使用既有 IndexedDB 目录加 sibling CacheStorage 目录，`ZERO_PRIVATE` 仍保持纯内存；
embedded `IndexedDbOwner::persistent(path)` 保持旧 IndexedDB root 布局兼容，同时新增
`path/CacheStorage`。Service Worker active registration-local `CacheStorage` 也已纳入
`ServiceWorkerPersistentRegistration` snapshot/restore；normal profile 的 SW cache mutation
会触发现有 Service Worker persistence writer，private profile 继续只保留内存态。
Service Worker runtime 已补齐 `Cache.delete()` 与 `CacheStorage.delete()/has()/keys()` 到
同一 typed host bridge，entry 删除和命名 cache 删除复用 registration-local
`zero-storage::CacheStorage`；同时已对齐 worker bootstrap 中 `Response.error().clone()` 的
error filtered response 保真与 `Cache.delete()` 缺参 TypeError，支撑 service-workers 目标的
6-case / 68-subtest SW CacheStorage wrapper baseline。随后 `cache-match.https.html`
扩面补齐 worker runtime 的最小 Blob/FileReader、cached `Response.url` 往返、response
guard 隐藏 `Set-Cookie` 与内部 `X-Zero-*` 元数据，以及 cross-origin no-cors opaque
filtered response 投影；`cache-put.https.html` 扩面补齐 worker runtime 的 `Request.bodyUsed`
初值、`Response.redirect()`、Blob response body 序列化，以及 `URL.hostname` mutation 后
`new Request(url, {mode: 'no-cors'})` 经 worker `fetch()` 生成 opaque filtered response 的
路径；`cache-add.https.html` 扩面补齐 worker runtime `Cache.addAll()` 同 request / response
`Vary` duplicate 检查和失败原子性；`cache-abort.https.html` 扩面补齐 SW runtime
AbortController/AbortSignal 与 aborted `Cache.put/add/addAll()` 的 `AbortError` rejection，
`cache-keys-attributes-for-service-worker.https.html` 扩面补齐 SW iframe reload/history
navigation request 标志经 `Cache.put()` / `Cache.keys()` 的保真；`credentials.https.html`
扩面继续验证 credentialed request URL 作为 Cache key 经 iframe XHR、worker fetch
interception、`Cache.put()`、`Cache.match()` / `Cache.matchAll()` / `CacheStorage.match()`
和 `Cache.keys()` 往返保真，支撑 service-workers 目标的 12-case / 157-subtest
SW CacheStorage wrapper baseline。
更大范围 WPT 导入仍待后续切片。（2026-09-06 复核：pinned revision 下 window 可执行面
39 case 已全部导入并全绿，SW 环境面归已归档 service-workers 目标，`cross-partition`
记 gated——window 面扩面空间已用尽，见「下一步计划」§3。）
CacheStorage window asset manifest 已补充逐 asset `source_revision`，恢复脚本会按每行
revision 下载缺失资产，避免 33-case baseline 中后续 wrapper/support 资产依赖某个本地
WPT checkout 状态。
Service Worker `fetch-event-within-sw.https.html` 扩面进一步固定 iframe
`contentWindow.caches` 页面表面：iframe window 现在暴露 `CacheStorage`/`Cache`/`caches`，
`Cache.add()` 的相对 URL、fetch client id 与 referrer 使用 iframe 文档上下文，因此受控
iframe 的 `cache.add('sample.txt')` 会经 SW fetch 事件拦截，而 SW 内部 `fetch()` /
`Cache.add()` 仍保持不自拦截。
Service Worker `fetch-event-respond-with-custom-response.https.html` 扩面进一步复用并固定
页面 `Response` body 读回能力：page-side `Response` 支持 ArrayBuffer /
ArrayBufferView body 字节化，`Response.formData()` 支持最小 multipart/form-data 文本字段解析；
worker-global 最小 `FormData` 序列化出的 multipart response 可被受控 iframe subresource fetch
与 navigation 路径读回。该切片服务 service-workers fetch baseline，不改变本目标的 window
CacheStorage 分母。
Service Worker `uncontrolled-page.https.html` 扩面固定 scope 外 uncontrolled 页面绕过 SW fetch
handler 的边界；`claim-fetch.https.html` 继续固定 message-time `clients.claim()` 控制既有
iframe client 后的 fetch interception；`claim-not-using-registration.https.html` 固定
`clients.claim()` 不抢占更长匹配 registration client 的边界。这些切片服务 service-workers
fetch baseline，不改变本目标的 window/SW CacheStorage 分母。

**与兄弟 goal 的边界**：
- [storage-indexeddb](../storage-indexeddb.md)（已归档）— IDB 归其管
- service-workers — SW 环境的 cache 用例（cache-storage/sw 类）归其验收；本目标只收
  window 环境可执行面
- js-dom（DOM API 反射面）— 仅 host 回调注册段可能共享，run-rules §9 碰头管理

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ Rust 层：`crates/storage/src/cache_api.rs`（976 行 / 67 函数）——CacheStorage/Cache/
  CacheQueryOptions 全 API 面 + 单测
- ✅ JS 页面层初始表面：`part07.js` 暴露 `CacheStorage`/`Cache`/`caches`，WebView 页面与
  iframe `contentWindow` 可 `open` 后 `put/match/matchAll/delete/keys`，并可 `has/keys/match`
- ✅ 持久化首片：page/WebView `StorageManager` owner 已支持 per-origin CacheStorage 落盘；
  SW registration-local CacheStorage 已随 active registration snapshot/restore 验证
- ✅ WPT `cache-storage` window runner 基线已导入：39 case / 449 subtest，449 Pass / 0 Fail
- ✅ add/addAll 的页面 fetch 链路、Cache API 返回对象 brand、缺参 TypeError、
  `CacheStorage.keys()` 创建顺序、Vary 匹配、delete-dooming、DOMString name wire 与
  Storage Buckets cache namespace
  已完成；`Cache.put` 非 GET/非 HTTP(S)/206/`Vary: *` 可缓存性拒绝、used body 拒绝、
  empty body 不消费、opaque 内部 206/`Vary: *` 可缓存、cached `Response.type`/`Response.url`
  读回保真、`Response.error()` 可缓存读回、`Response.redirect()`/Blob/FormData response
  路径、`Cache.matchAll()`、`Cache.match()`、`CacheStorage.match()`、`Cache.add()` 与
  `Cache.addAll()` 扩面已完成；页面 `fetch()` 的 `basic`/`cors`/`opaque`/`opaqueredirect`
  filtered response 生成矩阵已实现并经 CacheStorage round-trip 固定（2026-09-06
  复核：上游 window 可执行面 39 case 全绿，C4 收口）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| C1 | WPT cache-storage 用例覆盖为零 | ✅ M1 window 首批基线已接入 |
| C2 | 页面 `caches` 全局缺失（零接线） | ✅ M1 初始桥接完成；全 API 语义继续归 C4 |
| C3 | 无持久化 | ✅ M3 完成：page/WebView owner per-origin 落盘、跨 WebView 重建读回、磁盘错误 reject；SW registration-local CacheStorage snapshot/restore 与 normal profile 持久化 dirtying 已验证 |
| C4 | Request/Response 集成（add/addAll/可缓存性） | ✅ M2 页面 `add/addAll` GET + `Response.ok` 路径、返回对象 brand、缺参 TypeError、Vary 匹配、delete-dooming、DOMString name wire、`Cache.put` 非 GET/非 HTTP(S)/206/`Vary: *` 拒绝、used body 拒绝、empty body 不消费、cached `Response.type`/`Response.url` 读回保真、`Response.error()` 可缓存读回、opaque response 忽略 Vary 与内部 206/`Vary: *` 可缓存、`Response.redirect()`/Blob/FormData response 路径、ArrayBuffer/ArrayBufferView response body 与 multipart `Response.formData()` 文本字段读回、`addAll` 失败不部分落库、undefined entry 拒绝与 Vary-aware duplicate 判定、`basic`/`cors`/`opaque`/`opaqueredirect` filtered response 生成矩阵完成；上游 window 可执行面 39 case 全绿（2026-09-06 复核收口） |
| C5 | Cache.matchAll/Cache.keys 页面桥接 | ✅ M2；`ignoreSearch`/`ignoreMethod`/`ignoreVary` 已接线 |

## 下一步计划

1. **M2 切片 18**：本轮已复核剩余 window 面 CacheStorage WPT；`cross-partition.https.tentative.html` 仍需 dispatcher/popup/SharedWorker/partitioned-storage 支撑，继续留在 gated，不纳入当前 Cache API 语义 baseline；`cache-keys-attributes-for-service-worker.https.html` 已归 service-workers 目标收口
2. **service-workers 后续**：继续扩展 SW cache-storage / fetch-cache WPT 验收；fetch runner 已覆盖 uncontrolled-page scope bypass、message-time `clients.claim()` iframe control、claim longest-match boundary 与 ReadableStream pull-source chunk serialization，持久化能力已由 registration-local snapshot/restore 覆盖
3. **DC 收口（2026-09-06 审计后）**：上游分母核对完成——pinned revision `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83` 下 `cache-storage` 目录 17 个 source 中，window 可执行面 39 case 全部纳入 runner 并补齐 `imported-testharness.txt` 账本（46→51 条，revision 与 asset manifest 精确一致）；`serviceworker/` 子目录已由已归档 service-workers 目标收口（25 case / 318 subtest 全绿），`cross-partition.https.tentative.html` 记 gated。DC-1~DC-4 证据链已闭合，见下方验证基线 2026-09-06 条目。

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT cache-storage 基线 + caches 骨架 | ✅ 页面骨架 + 39-case / 449-subtest window runner WPT 基线已接入（449 Pass / 0 Fail，double-run deterministic） |
| M2 — Cache 全 API + 查询语义 | ✅ `Cache.matchAll()` / `Cache.keys()`、`Cache.match()`、`CacheStorage.match()`、`ignoreSearch`/`ignoreMethod`/`ignoreVary`、页面 `add/addAll` GET fetch→store、iframe `contentWindow.caches` + `Cache.add()` iframe fetch context、返回对象 brand、缺参 TypeError、`CacheStorage.keys()` 创建顺序、delete-dooming、DOMString name wire、Storage Buckets cache namespace、`Cache.put` 核心可缓存性拒绝/body 消费语义、cached `Response.type`/`Response.url` 读回保真、`Response.error()` 可缓存读回、opaque response 忽略 Vary 与内部 metadata 可缓存、`Response.redirect()`/Blob/FormData response 路径、ArrayBuffer/ArrayBufferView response body 与 multipart `Response.formData()` 文本字段读回、`addAll` 原子失败、undefined entry 拒绝、Vary-aware duplicate 判定、cached `Response.body` reader + `Response.clone()` 不崩溃路径、`basic`/`cors`/`opaque`/`opaqueredirect` filtered response 生成、Window/Dedicated Worker/nested Dedicated Worker owner 共享路径、`cache-abort` window/worker abort 语义、sandboxed iframe top-level/window CacheStorage 安全边界、credentialed request URL cache key top-level/SW wrapper、SW runtime `Cache.delete()` 与 `CacheStorage.delete/has/keys` 已接入；上游 window 可执行面 39 case 全绿，剩余窗口面无可执行用例（2026-09-06 复核） |
| M3 — 持久化 + 剩余语义收尾 | ✅ page/WebView owner per-origin 持久化与 SW registration-local CacheStorage 持久化已完成；剩余语义已由 39-case 全绿 window 基线覆盖（2026-09-06 收口） |

## 验证基线

- 2026-08-22 定向验证：
  - `cargo test -p zero-storage cache_storage --no-default-features`：23 passed
  - `cargo test -p zero-page-runtime cache_storage`：6 passed
  - `cargo test -p zero-engine cache_api_page_shim`：2 passed
  - `cargo test -p zero-webview cache_storage`：2 passed
- 2026-08-22 M2 `Cache.matchAll()` / `Cache.keys()` 定向验证：
  - `cargo test -p zero-storage cache_api -- --nocapture`：52 passed
  - `cargo test -p zero-page-runtime cache --no-default-features --features quickjs -- --nocapture`：8 passed
  - `cargo test -p zero-webview cache_storage --no-default-features --features quickjs -- --nocapture`：2 passed
  - `cargo test -p zero-engine test_cache_api_page_shim_host_roundtrip -- --nocapture`：1 passed
  - 证据：[M2 Cache.matchAll and Cache.keys](evidence/2026-08-22-m2-cache-matchall-keys.md)
- 2026-08-22 M2 `ignoreSearch` / `ignoreMethod` 定向验证：
  - `cargo test -p zero-storage cache_query_options -- --nocapture`：3 passed
  - `cargo test -p zero-page-runtime cache_storage_handler_applies_query_options --no-default-features --features quickjs -- --nocapture`：1 passed
  - `cargo test -p zero-engine test_cache_api_page_shim_query_options_wire -- --nocapture`：1 passed
  - `cargo test -p zero-webview page_cache_api_query_options_match_delete_and_keys -- --nocapture`：1 passed
  - 证据：[M2 CacheQueryOptions ignoreSearch and ignoreMethod](evidence/2026-08-22-m2-cache-query-options.md)
- 2026-08-22 M2 页面 `Cache.add()` / `Cache.addAll()` fetch→store 定向验证：
  - `cargo test -p zero-engine test_cache_api_page_shim_add_and_add_all_wire -- --nocapture`：1 passed
  - `cargo test -p zero-webview page_cache_api_add_and_add_all_fetch_then_store -- --nocapture`：1 passed
  - 证据：[M2 Cache.add and Cache.addAll Page Fetch Path](evidence/2026-08-22-m2-cache-add-addall.md)
- 2026-08-22 M1 WPT `cache-storage` window 首批基线：
  - `cargo test -p zero-engine test_response_request_constructors_r2968 -- --nocapture`：1 passed
  - `cargo test -p zero-wpt-runner cache_storage -- --nocapture`：2 passed
  - `bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：8 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：4 cases / 35 subtests / 29 Pass / 6 Fail，double-run deterministic
  - 资产 manifest：[CacheStorage window assets](evidence/2026-08-22-cache-storage-window-assets.tsv)
  - 通过率 evidence：[CacheStorage window WPT baseline](evidence/2026-08-22-cache-storage-window-baseline.md)
- 2026-08-22 M2 Cache API WebIDL brand + required arguments：
  - `cargo test -p zero-engine test_cache_api_page_shim_host_roundtrip -- --nocapture`：1 passed
  - `cargo test -p zero-engine test_cache_api_page_shim_required_arguments_reject -- --nocapture`：1 passed
  - `make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：4 cases / 35 subtests / 29 Pass / 6 Fail，double-run deterministic
- 2026-08-22 M2 CacheStorage 创建顺序：
  - `cargo test -p zero-storage cache_storage -- --nocapture`：24 passed
  - `cargo test -p zero-page-runtime cache_storage_handler_lists_and_deletes_caches_and_entries -- --nocapture`：1 passed
  - `make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：4 cases / 35 subtests / 30 Pass / 5 Fail，double-run deterministic
- 2026-08-22 M2 Vary/`ignoreVary` 匹配：
  - `cargo test -p zero-storage cache_api -- --nocapture`：59 passed
  - `cargo test -p zero-page-runtime cache_storage -- --nocapture`：11 passed
  - `cargo test -p zero-page-runtime fetch_handler_cache_storage_respects_vary_request_headers -- --nocapture`：1 passed
  - `make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：4 cases / 35 subtests / 33 Pass / 2 Fail，double-run deterministic
- 2026-08-22 M2 delete-dooming + DOMString name wire：
  - `cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`：10 passed
  - `cargo test -p zero-engine test_cache_api_page_shim -- --nocapture`：5 passed
  - `cargo test -p zero-webview cache_storage -- --nocapture`：6 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release --bin zero-wpt-runner`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：4 cases / 35 subtests / 35 Pass / 0 Fail，double-run deterministic
- 2026-08-22 M2 Cache.put/addAll 核心可缓存性矩阵：
  - `cargo test -p zero-storage cache_api::tests:: -- --nocapture`：56 passed
  - `cargo test -p zero-storage cache -- --nocapture`：75 passed
  - `cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`：11 passed
  - `cargo test -p zero-engine test_cache_api_page_shim -- --nocapture`：7 passed
  - `cargo test -p zero-webview cache_storage -- --nocapture`：7 passed
  - `cargo fmt --all -- --check`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo clippy -p zero-storage -p zero-page-runtime -p zero-engine -p zero-webview --all-targets -- -D warnings`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`：passed
  - `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`：passed
  - 证据：[M2 Cache.put/addAll cacheability](evidence/2026-08-22-m2-cache-cacheability.md)
- 2026-08-22 M2 `Response.type == "error"` 可缓存性 guard（历史中间态；后续 WPT
  扩面已校正为 CacheStorage 允许保存/读回 error filtered response，FetchEvent 响应结算仍拒绝 status 0）：
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo check -p zero-storage -p zero-page-runtime -p zero-script-sandbox -p zero-protocol -p zero-renderer -p zero-browser -p zero-webview --all-targets`：passed
  - `cargo fmt --all -- --check`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_api::tests::test_cache_put_rejects_uncacheable_requests_and_responses -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`：11 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-engine test_cache_api_page_shim -- --nocapture`：8 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox cache_put_rejects_error_response_before_host_write -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox cache -- --nocapture`：5 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_protocol -- --nocapture`：19 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-webview cache_storage -- --nocapture`：7 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-renderer service_worker_host -- --nocapture`：12 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-browser service_worker_owner -- --nocapture`：52 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy -p zero-storage -p zero-page-runtime -p zero-engine -p zero-script-sandbox -p zero-protocol -p zero-renderer -p zero-browser -p zero-webview --all-targets -- -D warnings`：passed
  - `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`：passed
  - 证据：[M2 Cache.put Response.error cacheability](evidence/2026-08-22-m2-cache-response-type-error.md)
- 2026-08-22 M2 cached `Response.type` 读回保真：
  - `cargo fmt --all -- --check`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`：12 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_page_shim_host_roundtrip -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview page_cache_api_match_preserves_cached_response_type -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo clippy -p zero-page-runtime -p zero-engine -p zero-webview --all-targets -- -D warnings`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`：passed
  - `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`：passed
  - 证据：[M2 Cache Response Type Readback](evidence/2026-08-22-m2-cache-response-type-readback.md)
- 2026-08-22 M2 CacheStorage window WPT 扩面：
  - 新增 WPT：`cache-matchAll.https.any.js`、`cache-storage-match.https.any.js`、`cache-match.https.any.js`、`cache-put.https.any.js`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_put -- --nocapture`：11 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_page_shim_puts_error_response -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage_handler_preserves_error_response_type -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_host_fetch_command_and_event_round_trip -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox cache_put_sends_error_response_to_host_storage -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview page_cache_api_rejects_uncacheable_put_and_atomic_add_all -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_vary_ignored_for_opaque_response`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage_handler_preserves_response_url`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_fetch_ --features v8`：7 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-match.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：25 subtests / 25 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- cargo test -p zero-storage cache_ -- --nocapture`：77 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo test -p zero-engine test_response_body_used_redirect_and_blob_formdata_cache_put_support -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo test -p zero-engine test_cache_api_page_shim_put_response_validation_and_opaque_internal_response -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-put.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：27 subtests / 27 Pass
  - `bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：16 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release --bin zero-wpt-runner`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：8 cases / 114 subtests / 114 Pass / 0 Fail，double-run deterministic
  - `cargo fmt --all -- --check`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`：passed
  - `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`：passed
  - 证据：[M2 CacheStorage Window WPT Expansion](evidence/2026-08-22-m2-cache-window-expansion.md)
- 2026-08-22 M2 `Cache.add()` / `Cache.addAll()` WPT 扩面：
  - 新增 WPT：`cache-add.https.any.js`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-add.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：22 subtests / 22 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_request_null_body_text_does_not_mark_body_used -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_page_shim_add_all_validates_entries_and_vary_duplicates -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-engine test_cache_api_page_shim -- --nocapture`：10 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`：1 passed
  - `bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：17 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release --bin zero-wpt-runner`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：9 cases / 136 subtests / 136 Pass / 0 Fail，double-run deterministic
  - `cargo fmt --all -- --check`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo clippy -p zero-engine -p zero-wpt-runner --all-targets -- -D warnings`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`：passed
  - `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`：passed
  - 证据：[M2 Cache.add WPT Expansion](evidence/2026-08-22-m2-cache-add-wpt-expansion.md)
- 2026-08-22 M2 CacheStorage Window/Worker sharing WPT 扩面：
  - 新增 WPT：`common.https.window.js` + `resources/common-worker.js`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage common.https.window.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：1 subtest / 1 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_dedicated_worker_uses_window_cache_storage_bridge -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：19 assets matched pinned manifest
  - `cargo test -p zero-wpt-runner cache_storage_window_manifest_has_ten_unique_cases`：1 passed
  - `cargo test -p zero-wpt-runner cache_storage_runner_reports_every_case_when_harness_is_missing`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release -p zero-wpt-runner`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：10 cases / 137 subtests / 137 Pass / 0 Fail，double-run deterministic
  - 证据：[M2 CacheStorage Worker Sharing WPT Expansion](evidence/2026-08-22-m2-cache-worker-sharing-wpt-expansion.md)
- 2026-08-22 M2 CacheStorage nested Worker WPT 扩面：
  - 新增 WPT：`cache-api-nested-worker.https.html` + `cache-api-nested-worker1.js` + `cache-api-nested-worker2.js`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_dedicated_worker_nested_worker_resolves_against_parent_script_url -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-api-nested-worker.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：1 subtest / 1 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：22 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release --bin zero-wpt-runner`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：11 cases / 138 subtests / 138 Pass / 0 Fail，double-run deterministic
  - 证据：[M2 CacheStorage Nested Worker WPT Expansion](evidence/2026-08-22-m2-cache-nested-worker-wpt-expansion.md)
- 2026-08-22 M2 CacheStorage Dedicated Worker wrapper WPT 扩面：
  - 新增 WPT：`worker/cache-storage.https.html`、`worker/cache-storage-keys.https.html`、`worker/cache-delete.https.html`、`worker/cache-keys.https.html`、`worker/cache-matchAll.https.html`、`worker/cache-storage-match.https.html`、`worker/cache-match.https.html`、`worker/cache-put.https.html`、`worker/cache-add.https.html`
  - 后续已补：`worker/cache-abort.https.html` 依赖 AbortController 与 dynamic slow-response/stash fixture，见下一条 cache-abort 切片
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`：40 assets restored
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_dedicated_worker_uses_window_cache_storage_bridge -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_dedicated_worker_imported_self_property_is_bare_global -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/ --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：9 cases / 135 subtests / 135 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：20 cases / 273 subtests / 273 Pass / 0 Fail，double-run deterministic
  - 证据：[M2 CacheStorage Dedicated Worker Wrapper WPT Expansion](evidence/2026-08-22-m2-cache-worker-wrapper-expansion.md)
- 2026-08-22 M2 CacheStorage `cache-abort` WPT 扩面：
  - 新增 WPT：`window/cache-abort.https.html`、`worker/cache-abort.https.html`
  - 新增 support：`script-tests/cache-abort.js`、`common/utils.js`、
    `fetch/api/resources/infinite-slow-response.py`、`fetch/api/resources/stash-take.py`、
    `fetch/api/resources/stash-put.py`
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`：47 assets restored
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：47 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage window/cache-abort.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：1 case / 9 subtests / 9 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/cache-abort.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：1 case / 9 subtests / 9 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage -- --nocapture`：2 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_fetch_abort_signal_r3044 -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：22 cases / 291 subtests / 291 Pass / 0 Fail，double-run deterministic
  - 证据：[M2 Cache Abort WPT Expansion](evidence/2026-08-22-m2-cache-abort-wpt-expansion.md)
- 2026-08-22 M2 Storage Buckets CacheStorage WPT 扩面：
  - 新增 WPT：`cache-storage-buckets.https.any.js`
  - 新增 support：`storage/buckets/resources/util.js`
  - `./target/test-guard --time-limit 120 -- cargo fmt --all -- --check`：passed
  - `./target/test-guard --time-limit 180 -- cargo test -p zero-engine test_cache_api_storage_buckets_namespace_and_delete -- --nocapture`：1 passed
  - `./target/test-guard --time-limit 180 -- cargo test -p zero-wpt-runner cache_storage_window_manifest_has_expected_unique_cases -- --nocapture`：1 passed
  - `./target/test-guard --time-limit 180 -- bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：49 assets matched pinned manifest
  - `./target/test-guard --time-limit 240 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-storage-buckets.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：2 subtests / 2 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：23 cases / 293 subtests / 293 Pass / 0 Fail，double-run deterministic
  - 证据：[M2 Storage Buckets CacheStorage WPT Expansion](evidence/2026-08-22-m2-storage-buckets-wpt-expansion.md)
- 2026-08-23 CacheStorage window manifest source revision 固化：
  - `bash -n tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`：passed
  - `./target/test-guard --time-limit 120 -- bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：49 assets matched pinned manifest
  - 单 asset 临时数据根恢复验证：`worker/cache-add.https.html` 按 `24197a11e8c5bd29a5cb7bdf18135a82be8a8546` 下载/恢复，361 bytes，blob `2658e1e50f9ebfe8ac5971a16af9e26b02d140a8`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest_has_expected_unique_cases -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：23 cases / 293 subtests / 293 Pass，double-run deterministic
  - 证据：[CacheStorage Window Manifest Source Revisions](evidence/2026-08-23-cache-storage-window-manifest-revisions.md)
- 2026-08-23 M2 CacheStorage window wrapper WPT 扩面：
  - 新增 WPT：`window/cache-storage.https.html`、`window/cache-storage-keys.https.html`、`window/cache-delete.https.html`、`window/cache-keys.https.html`、`window/cache-matchAll.https.html`、`window/cache-storage-match.https.html`、`window/cache-match.https.html`、`window/cache-put.https.html`、`window/cache-add.https.html`
  - `bash -n tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`：passed
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`：58 assets restored
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：58 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest_has_expected_unique_cases -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo run -p zero-wpt-runner -- testharness-cache-storage window/ --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：10 cases / 145 subtests / 145 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：32 cases / 429 subtests / 429 Pass，double-run deterministic
  - 证据：[M2 CacheStorage Window Wrapper WPT Expansion](evidence/2026-08-23-m2-cache-window-wrapper-expansion.md)
- 2026-08-23 M2 CacheStorage sandboxed iframe WPT 扩面：
  - 新增 WPT：`window/sandboxed-iframes.https.html` 与 helper `resources/iframe.html`
  - 页面 shim 将 iframe `sandbox` token 传入 `contentWindow`，无 `allow-same-origin` 时 iframe `caches.open/has/delete/keys/match` reject `SecurityError`
  - `fetch()` shim / WebView `__zw_fetch` / async `FetchBridge` 透传 Request credentials，修复 `cache-add` Vary duplicate fixture 在 `credentials: "omit"` 下的判定
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo test -p zero-engine test_iframe_sandbox_without_same_origin_denies_cache_storage -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo test -p zero-engine test_fetch_passes_request_credentials_to_host -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo test -p zero-engine fetch_bridge_preserves_credentials_wire -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo test -p zero-wpt-runner vary_py_handler_respects_request_credentials -- --nocapture`：1 passed
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make testharness-cache-storage FILTER=sandboxed-iframes.https.html`：1 case / 2 subtests / 2 Pass
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- make testharness-cache-storage FILTER=cache-add`：3 cases / 66 subtests / 66 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：33 cases / 431 subtests / 431 Pass，double-run deterministic
  - 证据：[CacheStorage window assets](evidence/2026-08-22-cache-storage-window-assets.tsv)、[CacheStorage window WPT baseline](evidence/2026-08-22-cache-storage-window-baseline.md)
- 2026-08-24 M2 CacheStorage common HTML wrapper WPT 扩面：
  - 新增 WPT：`common.https.html`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`：61 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest_has_expected_unique_cases -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage common.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`：1 case / 1 subtest / 1 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：34 cases / 432 subtests / 432 Pass，double-run deterministic
  - 证据：[CacheStorage window assets](evidence/2026-08-22-cache-storage-window-assets.tsv)、[CacheStorage window WPT baseline](evidence/2026-08-22-cache-storage-window-baseline.md)
- 2026-08-31 M2 CacheStorage filtered response 生成矩阵：
  - 新增 ZeroWeb 回归 fixture：`service-workers/cache-storage/zeroweb-filtered-response-types.https.any.js`
  - 页面 `fetch()` shim 透传 request `mode` / `redirect` 到 host，并将 host response 投影为
    `basic` / `cors` / `opaque` / `opaqueredirect` filtered response；CacheStorage 写入侧对
    `opaque` 与 `opaqueredirect` 使用内部响应元数据
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_fetch_filtered_response_type_matrix -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_page_shim_put_response_validation_and_opaque_internal_response -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage test_cache_put_accepts_opaque_like_internal_uncacheable_metadata -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_ -- --nocapture`：5 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo check -p zero-storage -p zero-engine -p zero-webview -p zero-wpt-runner --all-targets`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- make testharness-cache-storage FILTER=zeroweb-filtered`：4 subtests / 4 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-31-m2-filtered-response-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-31-m2-filtered-response-baseline.md`：35 cases / 436 subtests / 436 Pass，double-run deterministic
  - `cargo fmt --all -- --check`：passed
  - `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`：passed
  - `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`：passed（首次因 `tests/wpt-runner/wpt-data/fonts` 缺失失败，运行 `tests/wpt-runner/scripts/sync-imported-resources.sh` 恢复 WPT 字体后复跑通过）
  - 证据：[M2 CacheStorage Filtered Response Baseline](evidence/2026-08-31-m2-filtered-response-baseline.md)
- 2026-08-31 M2 CacheStorage sandboxed iframe top-level WPT 扩面：
  - 新增 WPT：`service-workers/cache-storage/sandboxed-iframes.https.html`
  - 与既有 `window/sandboxed-iframes.https.html` 共同固定 sandbox iframe 无
    `allow-same-origin` 时拒绝 CacheStorage、带 `allow-same-origin` 时允许访问的安全边界
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`：62 assets restored
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make testharness-cache-storage FILTER=sandboxed-iframes.https.html`：2 cases / 4 subtests / 4 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-31-m2-cache-sandboxed-iframes-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-31-m2-cache-sandboxed-iframes-baseline.md`：36 cases / 438 subtests / 438 Pass，double-run deterministic
  - 证据：[M2 CacheStorage Sandboxed Iframes Baseline](evidence/2026-08-31-m2-cache-sandboxed-iframes-baseline.md)
- 2026-08-31 M2 CacheStorage credentials top-level WPT 扩面：
  - 新增 WPT：`service-workers/cache-storage/credentials.https.html`
  - 复用 support：`service-workers/service-worker/resources/test-helpers.sub.js`、
    `service-workers/cache-storage/resources/credentials-worker.js`、
    `service-workers/cache-storage/resources/credentials-iframe.html`
  - 固定受控 iframe 发起的 credentialed XHR request URL 作为 Cache key，在页面
    CacheStorage runner 下经 Service Worker `fetch` 拦截和 `Cache.put()` /
    `Cache.match()` / `Cache.matchAll()` / `CacheStorage.match()` / `Cache.keys()`
    往返保真
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- target/release/zero-wpt-runner testharness-cache-storage --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root credentials.https.html --json`：1 case / 1 subtest / 1 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-31-cache-storage-window-credentials-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-31-cache-storage-window-credentials-baseline.md`：37 cases / 439 subtests / 439 Pass，double-run deterministic
  - 证据：[M2 CacheStorage Credentials Baseline](evidence/2026-08-31-cache-storage-window-credentials-baseline.md)
- 2026-09-02 M2 CacheStorage response clone crashtest 扩面：
  - 新增 WPT：`service-workers/cache-storage/crashtests/cache-response-clone.https.html`
  - runner 对无 testharness 的 `test-wait` crashtest 改为等待根元素清除 `test-wait`
    后才判 pass，并收紧脚本抛错不再作为 crash 用例通过
  - `<script type="module">` 转换支持顶层 `await`，用于执行该 crashtest 的 module body
  - 固定 `Cache.add("")` / `Cache.match("")` 读回 cached `Response.body` 后，先打开
    reader、再 `Response.clone()`、再继续 read 的不崩溃路径
  - `./target/test-guard --time-limit 90 --total-mem 4 --per-proc-mem 3 -- cargo test -p zero-script-sandbox test_top_level_await_wraps_module_in_async_iife -- --nocapture`：1 passed
  - `./target/test-guard --time-limit 90 --total-mem 4 --per-proc-mem 3 -- cargo test -p zero-webview test_module_top_level_await_completes_r3083 -- --nocapture`：1 passed
  - `./target/test-guard --time-limit 90 --total-mem 4 --per-proc-mem 3 -- cargo test -p zero-wpt-runner no_harness -- --nocapture`：3 passed
  - `./target/test-guard --time-limit 120 --total-mem 4 --per-proc-mem 3 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-response-clone --wpt-data /tmp/zw-wpt-cache-storage --json`：1 case / 1 subtest / 1 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-09-02-m2-cache-response-clone-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-09-02-m2-cache-response-clone-baseline.md`：39 cases / 449 subtests / 449 Pass，double-run deterministic
  - 证据：[M2 CacheStorage Response Clone Crashtest](evidence/2026-09-02-m2-cache-response-clone-crashtest.md)、[M2 CacheStorage Response Clone Baseline](evidence/2026-09-02-m2-cache-response-clone-baseline.md)
- 2026-09-02 Service Worker fetch stream body error：
  - `service-workers/service-worker/fetch-error.https.html` 纳入 Service Worker
    fetch/message runner，覆盖 `respondWith(new Response(stream))` 的 body stream
    先产生进展、后续 error 时，页面 `response.text()` body 消费 reject。该行为同
    `Cache.add()` / worker `fetch()` 共享 Response body 桥接路径，避免将 errored stream
    错误快照为成功文本 body。
  - `make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.md`：26 cases / 70 subtests / 70 Pass，double-run deterministic
  - 证据：[Service Worker Fetch WPT Baseline](../service-workers/evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.md)
- 2026-09-02 Service Worker secure-context surface：
  - `ServiceWorkerGlobalScope/isSecureContext.https.html` 纳入 Service Worker core runner，
    补齐 `WorkerGlobalScope.prototype.isSecureContext === true`。该切片不改变本目标的
    CacheStorage window/SW 分母，但解除后续 worker-harness Cache/SW 交叉用例的一个基础
    global surface 缺口。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
    40 cases / 166 subtests / 166 Pass，double-run deterministic
  - 证据：[M3 Service Worker WorkerGlobalScope.isSecureContext](../service-workers/evidence/2026-09-02-m3-worker-secure-context.md)
- 2026-09-02 Service Worker install event type：
  - `install-event-type.https.html` 纳入 Service Worker core runner，补齐 install 事件
    `bubbles === false` 基础属性并复用已有 worker-testharness 结果通道。该切片不改变
    CacheStorage window/SW 分母，但继续收敛 Cache/SW worker-harness 依赖的基础 lifecycle
    event surface。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
    41 cases / 167 subtests / 167 Pass，double-run deterministic
  - 证据：[M3 Service Worker InstallEvent Type](../service-workers/evidence/2026-09-02-m3-install-event-type.md)
- 2026-09-02 Service Worker global close absence：
  - `ServiceWorkerGlobalScope/close.https.html` 纳入 Service Worker core runner，确认 SW
    global 不暴露 `close()`。该切片不改变 CacheStorage window/SW 分母，但继续收敛
    worker-harness 依赖的 Service Worker global 接口面。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
    42 cases / 169 subtests / 169 Pass，double-run deterministic
  - 证据：[M3 Service Worker Global Close Absence](../service-workers/evidence/2026-09-02-m3-worker-close.md)
- 2026-09-02 Service Worker interface requirements：
  - `interface-requirements-sw.https.html` 纳入 Service Worker core runner，`FetchEvent`
    constructor 现在拒绝缺失/非法 `FetchEventInit.request`，并继续确认 SW global 不暴露
    `XMLHttpRequest` / `URL.createObjectURL`。该切片不改变 CacheStorage window/SW 分母，
    但收敛 CacheStorage serviceworker wrapper 依赖的 worker-harness 基础接口。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
    43 cases / 173 subtests / 173 Pass，double-run deterministic
  - 证据：[M3 Service Worker Interface Requirements](../service-workers/evidence/2026-09-02-m3-worker-interface.md)
- 2026-09-02 FetchEvent historical interface：
  - `historical.https.any.js` 纳入 Service Worker core runner，确认
    `FetchEvent.prototype.targetClientId` 不暴露。该切片不改变 CacheStorage window/SW
    分母，只收敛 CacheStorage serviceworker wrapper 依赖的 FetchEvent 接口面。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
    44 cases / 175 subtests / 175 Pass，double-run deterministic
  - 证据：[M3 FetchEvent Historical Interface](../service-workers/evidence/2026-09-02-m3-fetch-event-historical.md)
- 2026-09-02 Classic Service Worker dynamic import rejection：
  - `no-dynamic-import.any.js` 纳入 Service Worker core runner，确认 classic Service Worker
    global 中动态 `import(url)` 返回 rejected promise。该切片不改变 CacheStorage window/SW
    分母，只收敛 CacheStorage serviceworker wrapper 依赖的 worker script 负面能力面。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
    45 cases / 176 subtests / 176 Pass，double-run deterministic
  - 证据：[M3 Classic Service Worker Dynamic Import Rejection](../service-workers/evidence/2026-09-02-m3-no-dynamic-import.md)
- 2026-09-02 Module Service Worker dynamic import rejection：
  - `no-dynamic-import-in-module.any.js` 纳入 Service Worker core runner，确认
    `serviceworker-module` wrapper 以 module 类型注册 worker，且 module worker 动态
    `import(url)` 返回 rejected `TypeError` promise。该切片不改变 CacheStorage window/SW
    分母，只补齐 CacheStorage serviceworker wrapper 依赖的 module worker 负面能力面。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
    46 cases / 183 subtests / 183 Pass，double-run deterministic
  - 证据：[M3 Module Service Worker Dynamic Import Rejection](../service-workers/evidence/2026-09-02-m3-module-no-dynamic-import.md)
- 2026-09-02 Service Worker global self identity：
  - `global-serviceworker.https.any.js` 纳入 Service Worker core runner，确认 worker
    global 中只读 `self.serviceWorker`、install/activate 事件期 registration slot，以及
    启动期 self-message。该切片不改变 CacheStorage window/SW 分母，只补齐 CacheStorage
    serviceworker wrapper 依赖的 worker global 身份面。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json TIME_LIMIT=900`：
    47 cases / 188 subtests / 188 Pass，double-run deterministic
  - 证据：[M3 Service Worker Global Self Identity](../service-workers/evidence/2026-09-02-m3-global-serviceworker.md)
- 2026-09-02 Service Worker immutable prototype chain：
  - `immutable-prototype-serviceworker.https.html` 纳入 Service Worker core runner，确认
    worker global prototype chain 的 `Object.setPrototypeOf()` / `Reflect.setPrototypeOf()`
    不可变语义。该切片不改变 CacheStorage window/SW 分母，只补齐 CacheStorage
    serviceworker wrapper 依赖的 worker global 对象模型面。
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json TIME_LIMIT=900`：
    48 cases / 189 subtests / 189 Pass，double-run deterministic
  - 证据：[M3 Service Worker Immutable Prototype](../service-workers/evidence/2026-09-02-m3-immutable-prototype.md)
- 2026-08-22 M3 CacheStorage 持久化首片：
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo check -p zero-storage -p zero-page-runtime -p zero-webview --all-targets`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_storage_persistence -- --nocapture`：3 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`：15 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview cache_storage -- --nocapture`：9 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview indexed_db_owner -- --nocapture`：4 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo clippy -p zero-storage -p zero-page-runtime -p zero-webview -p zero-browser --all-targets -- -D warnings`：passed
  - `cargo fmt --all -- --check`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`：passed
  - `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`：passed
  - `make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`：8 cases / 114 subtests / 114 Pass / 0 Fail，double-run deterministic
  - 证据：[M3 CacheStorage Persistence](evidence/2026-08-22-m3-cache-storage-persistence.md)
- 2026-08-22 M3 Service Worker registration-local CacheStorage 持久化：
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime persistent_registration_round_trips_cache_storage -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-browser persistent_owner_restores_registration_cache_storage -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_storage -- --nocapture`：27 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage -- --nocapture`：19 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-browser service_worker_owner -- --nocapture`：53 passed
  - 证据：[M3 Service Worker CacheStorage Persistence](../service-workers/evidence/2026-08-22-m3-registration-cache-storage-persistence.md)
- 2026-08-22 Service Worker Cache delete/listing 支撑：
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox cache -- --nocapture`：6 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 12 --time-limit 600 -- cargo test -p zero-renderer service_worker_host -- --nocapture`：13 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-page-runtime cache_storage -- --nocapture`：20 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-browser service_worker_owner -- --nocapture`：54 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_protocol -- --nocapture`：19 passed
  - 证据：[M2 Service Worker Cache Delete And Listing](../service-workers/evidence/2026-08-22-m2-worker-cache-delete-listing.md)
- 2026-08-23 Service Worker CacheStorage serviceworker `cache-match` WPT 扩面：
  - 新增 WPT：`service-workers/cache-storage/serviceworker/cache-match.https.html`
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- make fetch-wpt-service-workers-cache-storage-wave`：22 assets restored
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- make testharness-service-workers-cache-storage FILTER=cache-match.https.html`：26 entries Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox service_worker::tests::worker_global_fetch_ -- --nocapture`：3 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 360 -- cargo test -p zero-page-runtime service_worker -- --nocapture`：49 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-cache-storage OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`：7 cases / 94 subtests / 94 Pass，double-run deterministic
  - 证据：[Service Worker CacheStorage WPT Baseline](../service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md)
- 2026-08-23 Service Worker CacheStorage serviceworker `cache-put` WPT 扩面：
  - 新增 WPT：`service-workers/cache-storage/serviceworker/cache-put.https.html`
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- make fetch-wpt-service-workers-cache-storage-wave`：25 assets restored
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- make testharness-service-workers-cache-storage FILTER=cache-put.https.html`：27 entries Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox service_worker -- --nocapture`：50 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner service_worker_cache_storage -- --nocapture`：2 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- make audit-wpt-service-workers-cache-storage-wave`：25 assets verified
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-cache-storage OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`：8 cases / 121 subtests / 121 Pass，double-run deterministic
  - 证据：[Service Worker CacheStorage WPT Baseline](../service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md)
- 2026-08-23 Service Worker CacheStorage serviceworker `cache-add` WPT 扩面：
  - 新增 WPT：`service-workers/cache-storage/serviceworker/cache-add.https.html`
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- make fetch-wpt-service-workers-cache-storage-wave`：27 assets restored
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- make testharness-service-workers-cache-storage FILTER=cache-add.https.html`：23 entries Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox service_worker -- --nocapture`：52 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-cache-storage OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`：9 cases / 144 subtests / 144 Pass，double-run deterministic
  - 证据：[Service Worker CacheStorage WPT Baseline](../service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md)
- 2026-08-23 Service Worker CacheStorage serviceworker `cache-abort` WPT 扩面：
  - 新增 WPT：`service-workers/cache-storage/serviceworker/cache-abort.https.html`
  - 新增 support：`common/utils.js`、`service-workers/cache-storage/script-tests/cache-abort.js`
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- make fetch-wpt-service-workers-cache-storage-wave`：30 assets restored
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- make testharness-service-workers-cache-storage FILTER=cache-abort.https.html`：10 entries Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-cache-storage OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`：10 cases / 154 subtests / 154 Pass，double-run deterministic
  - 证据：[Service Worker CacheStorage WPT Baseline](../service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md)
- 2026-08-23 Service Worker CacheStorage serviceworker request navigation attributes WPT 扩面：
  - 新增 WPT：`service-workers/cache-storage/serviceworker/cache-keys-attributes-for-service-worker.https.html`
  - 新增 support：`service-workers/cache-storage/resources/cache-keys-attributes-for-service-worker.js`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- make testharness-service-workers-cache-storage FILTER=cache-keys-attributes-for-service-worker.https.html`：2 entries Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-cache-storage OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`：11 cases / 156 subtests / 156 Pass，double-run deterministic
  - 证据：[Service Worker CacheStorage WPT Baseline](../service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md)
- 2026-08-23 Service Worker CacheStorage serviceworker credentials WPT 扩面：
  - 新增 WPT：`service-workers/cache-storage/serviceworker/credentials.https.html`
  - 新增 support：`service-workers/cache-storage/resources/credentials-worker.js`、`credentials-iframe.html`
  - `make testharness-service-workers-cache-storage FILTER=credentials.https.html`：1 entry Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- python3 tests/wpt-runner/scripts/run-service-workers-cache-storage-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --output docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json --summary docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`：12 cases / 157 subtests / 157 Pass，double-run deterministic
  - 证据：[Service Worker CacheStorage WPT Baseline](../service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md)
- 2026-08-23 Service Worker fetch `fetch-event-within-sw` WPT 扩面：
  - 新增 WPT：`service-workers/service-worker/fetch-event-within-sw.https.html`
  - 新增 support：`service-workers/service-worker/resources/fetch-event-within-sw-worker.js`、`resources/sample.txt`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- make testharness-service-workers-fetch FILTER=fetch-event-within-sw.https.html`：2 entries Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-within-sw-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-within-sw-baseline.md`：7 cases / 12 subtests / 12 Pass，double-run deterministic
  - 证据：[Service Worker Fetch WPT Baseline](../service-workers/evidence/2026-08-23-m2-fetch-within-sw-baseline.md)
- 2026-08-23 Service Worker fetch custom-response 共享 Response/FormData 支撑：
  - 新增 WPT：`service-workers/service-worker/fetch-event-respond-with-custom-response.https.html`
  - 新增 support：`service-workers/service-worker/resources/fetch-event-respond-with-custom-response-worker.js`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_response_body_used_redirect_and_blob_formdata_cache_put_support -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox fetch_event_respond_with_serializes_buffer_source_and_form_data_response -- --nocapture`：1 passed
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-custom-response-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-custom-response-baseline.md`：8 cases / 23 subtests / 23 Pass，double-run deterministic
  - 证据：[Service Worker Fetch WPT Baseline](../service-workers/evidence/2026-08-23-m2-fetch-custom-response-baseline.md)
- 2026-08-23 Service Worker fetch claim-fetch 扩面：
  - 新增 WPT：`service-workers/service-worker/claim-fetch.https.html`
  - 新增 support：`service-workers/service-worker/resources/claim-worker.js`、`blank.html`
  - `WPT_SOURCE=$HOME/github/others/wpt make testharness-service-workers-fetch FILTER=claim-fetch.https.html`：1 Pass
  - `WPT_SOURCE=$HOME/github/others/wpt make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-baseline.md`：11 cases / 26 subtests / 26 Pass，double-run deterministic
  - 证据：[Service Worker Fetch WPT Baseline](../service-workers/evidence/2026-08-23-m2-fetch-baseline.md)
- 2026-08-23 Service Worker fetch claim registration-boundary 扩面：
  - 新增 WPT：`service-workers/service-worker/claim-not-using-registration.https.html`
  - 新增 support：`service-workers/service-worker/resources/empty.js`、`empty-worker.js`
  - `WPT_SOURCE=$HOME/github/others/wpt make testharness-service-workers-fetch FILTER=claim-not-using-registration.https.html`：2 Pass
  - `WPT_SOURCE=$HOME/github/others/wpt make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-baseline.md`：12 cases / 28 subtests / 28 Pass，double-run deterministic
  - 证据：[Service Worker Fetch WPT Baseline](../service-workers/evidence/2026-08-23-m2-fetch-baseline.md)
- 2026-08-23 Service Worker fetch claim active-state 扩面：
  - 新增 WPT：`service-workers/service-worker/claim-using-registration.https.html`
  - 复用 support：`service-workers/service-worker/resources/claim-worker.js`、`empty.js`、`blank.html`
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make testharness-service-workers-fetch FILTER=claim-using-registration.https.html`：2 Pass
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-fetch-baseline.md`：13 cases / 30 subtests / 30 Pass，double-run deterministic
  - 证据：[Service Worker Fetch WPT Baseline](../service-workers/evidence/2026-08-23-m2-fetch-baseline.md)
- 2026-09-06 DC 收口审计（账本闭包 + 门禁复验）：
  - 上游分母核对：pinned revision `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83` 下
    `service-workers/cache-storage/` 目录共 17 个 source——window/Dedicated Worker
    可执行面 39 case 已全部在 runner manifest；`serviceworker/` 子目录 5 case 归
    已归档 service-workers 目标（CacheStorage lane 25 case / 318 subtest 全绿）；
    `cache-keys-attributes-for-service-worker.https.html` 同归 SW 目标；
    `cross-partition.https.tentative.html` 记 gated（需 dispatcher/popup/
    SharedWorker/partitioned-storage）。skip list 归属成立，无不充数、无误排除。
  - `imported-testharness.txt` 账本补齐 5 条缺失条目（`common.https.window.js`、
    `cache-api-nested-worker.https.html`、top-level/window `sandboxed-iframes.https.html`、
    `zeroweb-filtered-response-types.https.any.js`——前三者/前三者/`24197a11`/`04067ce9`
    revision 与 asset manifest 精确一致）：cache-storage 条目 46→51，39/39 window
    case 全部入账；SW disposition 审计 `python3
    tests/wpt-runner/scripts/audit-service-worker-disposition.py`：PASS
    （294 sources / 331 URLs，core=78 defer=22 fetch=3 gated=149 skip=42）。
  - `make baseline-wpt-cache-storage`（恢复 68 assets 后）复验：39 cases / 449
    subtests / 449 Pass / 0 Fail，double-run deterministic——证据
    [2026-09-06 CacheStorage Window Baseline](evidence/2026-09-06-cache-storage-window-baseline.md)
  - 单元测试复验：`cargo test -p zero-storage cache --no-default-features`：81 passed；
    `cargo test -p zero-engine cache_api_page_shim`：10 passed；
    `cargo test -p zero-page-runtime cache_storage`：20 passed；
    `cargo test -p zero-webview cache_storage`：9 passed
  - `make test`（干净全工作区，test-guard 包裹）：首轮
    `navigator_skip_waiting_activates_replacement_version` 1F（组合态负载
    timeout——隔离复跑 0.16s 绿、同树二跑全绿，定性 flake 非语义回归，
    与 2026-09-05 e55038a2a deadline 60s 放宽后残余负载敏感性一致）；
    二跑 18954 Pass / 0 Fail / exit 0（v8 主矩阵 + adapter-only GPU +
    QuickJS 矩阵 + QuickJS clippy 并行编译全过）
  - `cargo fmt --all -- --check`：passed；`git diff --check`：passed
  - `cargo clippy --workspace --all-targets -- -D warnings`：exit 0 零警告；
    `cargo clippy --no-default-features --features quickjs --workspace
    --all-targets -- -D warnings`：exit 0 零警告
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
