# Cache API 真实化 — 运行时控制面板（master.md）

**入口文档**: [../storage-cache-api.md](../storage-cache-api.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-22（M1 页面 CacheStorage 初始桥接完成）

---

## 当前状态

**专项定位**：存储方向三拆之二。把页面 `caches`（CacheStorage/Cache）从**零接线**
接到 zero-storage `cache_api.rs` 并补持久化，WPT `cache-storage`（window 面）真实用例驱动。
2026-08-22 已完成 WebView/in-process 页面侧初始桥接：
`caches.open/has/delete/keys/match` 与 `Cache.put/match/delete` 经 host callback 进入共享
`StorageManager`，origin 由宿主页面 URL 推导；WPT 基线、持久化、`add/addAll`、`matchAll/keys`
与 CacheQueryOptions/Vary 仍待后续切片。

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
  `open` 后 `put/match/delete`，并可 `has/keys/match`
- ⚠️ 无持久化：内存结构
- ⚠️ WPT `cache-storage` 未导入，无基线
- ⚠️ add/addAll 的 fetch 链路与 Response 可缓存性判定未实现

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| C1 | WPT cache-storage 用例覆盖为零 | ⬜ M1 |
| C2 | 页面 `caches` 全局缺失（零接线） | ✅ M1 初始桥接完成；全 API 语义继续归 C4 |
| C3 | 无持久化 | ⬜ M3 |
| C4 | Request/Response 集成（add/addAll/可缓存性） | ⬜ M2 |

## 下一步计划

1. **M1 剩余**：WPT `cache-storage` window 面用例导入 + 基线（当前还只有定向单元/
   WebView e2e 覆盖）
2. **M2 切片 1**：补 `Cache.matchAll()` / `Cache.keys()` 与 CacheQueryOptions 基础 wire
3. **M2 切片 2**：`add/addAll` 复用 fetch 管线，补 Response 可缓存性判定
4. **M3**：per-origin 持久化与跨会话 e2e

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT cache-storage 基线 + caches 骨架 | 🚧 页面骨架已接入；WPT 基线待导入 |
| M2 — Cache 全 API + 查询语义 | ⬜ |
| M3 — 持久化 + 剩余语义收尾 | ⬜ |

## 验证基线

- 2026-08-22 定向验证：
  - `cargo test -p zero-storage cache_storage --no-default-features`：23 passed
  - `cargo test -p zero-page-runtime cache_storage`：6 passed
  - `cargo test -p zero-engine cache_api_page_shim`：2 passed
  - `cargo test -p zero-webview cache_storage`：2 passed
- WPT cache-storage 面：无基线（未导入）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
