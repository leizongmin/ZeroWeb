# Cache API 真实化 — 运行时控制面板（master.md）

**入口文档**: [../storage-cache-api.md](../storage-cache-api.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-22（M2 CacheStorage window baseline 全绿）

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
2026-08-22 已接入首批 4 个上游 CacheStorage `.any.js` window 面 WPT 基线，WebIDL
brand、缺参 TypeError、`CacheStorage.keys()` 创建顺序、Vary 匹配、delete-dooming
生命周期与 DOMString code-unit name wire 修复后双跑稳定为 35 subtest / 35 Pass /
0 Fail。持久化、完整 Response 可缓存性矩阵和更大范围 WPT 导入仍待后续切片。

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
- ⚠️ 无持久化：内存结构
- ✅ WPT `cache-storage` window 首批已导入：4 case / 35 subtest，35 Pass / 0 Fail
- 🚧 add/addAll 的页面 fetch 链路、Cache API 返回对象 brand、缺参 TypeError、
  `CacheStorage.keys()` 创建顺序、Vary 匹配、delete-dooming 与 DOMString name wire
  已完成；Response 可缓存性完整判定未实现

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| C1 | WPT cache-storage 用例覆盖为零 | ✅ M1 window 首批基线已接入 |
| C2 | 页面 `caches` 全局缺失（零接线） | ✅ M1 初始桥接完成；全 API 语义继续归 C4 |
| C3 | 无持久化 | ⬜ M3 |
| C4 | Request/Response 集成（add/addAll/可缓存性） | 🚧 M2 页面 `add/addAll` GET + `Response.ok` 路径、返回对象 brand、缺参 TypeError、Vary 匹配、delete-dooming、DOMString name wire 完成；完整可缓存性矩阵待补 |
| C5 | Cache.matchAll/Cache.keys 页面桥接 | ✅ M2；`ignoreSearch`/`ignoreMethod`/`ignoreVary` 已接线 |

## 下一步计划

1. **M2 切片 5**：补 `add/addAll` 完整 Response 可缓存性判定
2. **M2 切片 6**：扩大 `cache-storage` window 面 WPT 导入范围并同步 `imported-tests.txt`
3. **M3**：per-origin 持久化与跨会话 e2e

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT cache-storage 基线 + caches 骨架 | ✅ 页面骨架 + 首批 window WPT 基线已接入 |
| M2 — Cache 全 API + 查询语义 | 🚧 `Cache.matchAll()` / `Cache.keys()`、`ignoreSearch`/`ignoreMethod`/`ignoreVary`、页面 `add/addAll` GET fetch→store、返回对象 brand、缺参 TypeError、`CacheStorage.keys()` 创建顺序、delete-dooming 与 DOMString name wire 已接入；完整可缓存性待完成 |
| M3 — 持久化 + 剩余语义收尾 | ⬜ |

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
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
