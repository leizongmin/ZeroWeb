# Cache API 真实化 — 运行时控制面板（master.md）

**入口文档**: [../storage-cache-api.md](../storage-cache-api.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-22（M2 Cache.add WPT 扩面）

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
2026-08-22 已接入 9 个上游 CacheStorage `.any.js` window 面 WPT 基线，WebIDL
brand、缺参 TypeError、`CacheStorage.keys()` 创建顺序、Vary 匹配、delete-dooming
生命周期、DOMString code-unit name wire、`Cache.matchAll()` 查询矩阵与 `CacheStorage.match()`
跨 cache/cacheName 查询、`Cache.match()` URL/fragment/opaque Vary/MIME/fetched response URL
矩阵、`Cache.put()` 可缓存性/响应体消费矩阵、`Cache.add()`/`Cache.addAll()` 矩阵修复后
双跑稳定为 136 subtest / 136 Pass /
0 Fail。
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
`zero-storage::CacheStorage`。
更大范围 WPT 导入与完整
`basic`/`cors`/`opaque`/`opaqueredirect` filtered response 生成矩阵仍待后续切片。

**与兄弟 goal 的边界**：
- [storage-indexeddb](../archive/storage-indexeddb.md)（已归档）— IDB 归其管
- service-workers — SW 环境的 cache 用例（cache-storage/sw 类）归其验收；本目标只收
  window 环境可执行面
- js-dom（DOM API 反射面）— 仅 host 回调注册段可能共享，run-rules §9 碰头管理

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ Rust 层：`crates/storage/src/cache_api.rs`（976 行 / 67 函数）——CacheStorage/Cache/
  CacheQueryOptions 全 API 面 + 单测
- ✅ JS 页面层初始表面：`part07.js` 暴露 `CacheStorage`/`Cache`/`caches`，WebView 页面可
  `open` 后 `put/match/matchAll/delete/keys`，并可 `has/keys/match`
- ✅ 持久化首片：page/WebView `StorageManager` owner 已支持 per-origin CacheStorage 落盘；
  SW registration-local CacheStorage 已随 active registration snapshot/restore 验证
- ✅ WPT `cache-storage` window 基线已导入：9 case / 136 subtest，136 Pass / 0 Fail
- 🚧 add/addAll 的页面 fetch 链路、Cache API 返回对象 brand、缺参 TypeError、
  `CacheStorage.keys()` 创建顺序、Vary 匹配、delete-dooming 与 DOMString name wire
  已完成；`Cache.put` 非 GET/非 HTTP(S)/206/`Vary: *` 可缓存性拒绝、used body 拒绝、
  empty body 不消费、opaque 内部 206/`Vary: *` 可缓存、cached `Response.type`/`Response.url`
  读回保真、`Response.error()` 可缓存读回、`Response.redirect()`/Blob/FormData response
  路径、`Cache.matchAll()`、`Cache.match()`、`CacheStorage.match()`、`Cache.add()` 与
  `Cache.addAll()` 扩面已完成，完整 `basic`/`cors`/`opaque`/`opaqueredirect` filtered
  response 生成矩阵未实现

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| C1 | WPT cache-storage 用例覆盖为零 | ✅ M1 window 首批基线已接入 |
| C2 | 页面 `caches` 全局缺失（零接线） | ✅ M1 初始桥接完成；全 API 语义继续归 C4 |
| C3 | 无持久化 | ✅ M3 完成：page/WebView owner per-origin 落盘、跨 WebView 重建读回、磁盘错误 reject；SW registration-local CacheStorage snapshot/restore 与 normal profile 持久化 dirtying 已验证 |
| C4 | Request/Response 集成（add/addAll/可缓存性） | 🚧 M2 页面 `add/addAll` GET + `Response.ok` 路径、返回对象 brand、缺参 TypeError、Vary 匹配、delete-dooming、DOMString name wire、`Cache.put` 非 GET/非 HTTP(S)/206/`Vary: *` 拒绝、used body 拒绝、empty body 不消费、cached `Response.type`/`Response.url` 读回保真、`Response.error()` 可缓存读回、opaque response 忽略 Vary 与内部 206/`Vary: *` 可缓存、`Response.redirect()`/Blob/FormData response 路径、`addAll` 失败不部分落库、undefined entry 拒绝与 Vary-aware duplicate 判定完成；完整 `basic`/`cors`/`opaque`/`opaqueredirect` filtered response 生成矩阵待补 |
| C5 | Cache.matchAll/Cache.keys 页面桥接 | ✅ M2；`ignoreSearch`/`ignoreMethod`/`ignoreVary` 已接线 |

## 下一步计划

1. **M2 切片 9**：继续导入 dynamic-server / cross-origin CacheStorage WPT case，补完整 filtered response 生成矩阵
2. **service-workers 后续**：补 SW cache-storage WPT 验收，持久化能力已由 registration-local snapshot/restore 覆盖

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT cache-storage 基线 + caches 骨架 | ✅ 页面骨架 + 9-case window WPT 基线已接入 |
| M2 — Cache 全 API + 查询语义 | 🚧 `Cache.matchAll()` / `Cache.keys()`、`Cache.match()`、`CacheStorage.match()`、`ignoreSearch`/`ignoreMethod`/`ignoreVary`、页面 `add/addAll` GET fetch→store、返回对象 brand、缺参 TypeError、`CacheStorage.keys()` 创建顺序、delete-dooming、DOMString name wire、`Cache.put` 核心可缓存性拒绝/body 消费语义、cached `Response.type`/`Response.url` 读回保真、`Response.error()` 可缓存读回、opaque response 忽略 Vary 与内部 metadata 可缓存、`Response.redirect()`/Blob/FormData response 路径、`addAll` 原子失败、undefined entry 拒绝、Vary-aware duplicate 判定、SW runtime `Cache.delete()` 与 `CacheStorage.delete/has/keys` 已接入；完整 filtered response 生成与更大 WPT 覆盖待完成 |
| M3 — 持久化 + 剩余语义收尾 | 🚧 page/WebView owner per-origin 持久化与 SW registration-local CacheStorage 持久化已完成；剩余 filtered response 矩阵待补 |

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
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
