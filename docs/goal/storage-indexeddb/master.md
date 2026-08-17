# IndexedDB 真实化 — 运行时控制面板（master.md）

**入口文档**: [../storage-indexeddb.md](../storage-indexeddb.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-17（M2 factory/schema routing）

---

## 当前状态

**专项定位**：从 zero-web 父目标拆出的存储方向三拆之一（另两个：storage-cache-api /
service-workers）。把页面 `indexedDB` 从 in-memory 近似接到 zero-storage 真实引擎（~10k 行）
并补持久化，WPT `IndexedDB` 真实用例驱动。

**与兄弟 goal 的边界**：
- storage-cache-api — Cache API/CacheStorage 归其管；本目标不碰 `caches`
- service-workers — SW 环境的 IDB 使用（self.indexedDB）归其集成验收；本目标只保证
  window 面 `indexedDB` 真实
- js-dom（DOM API 反射面）— 仅 host 回调注册段可能共享，run-rules §9 碰头管理

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ Rust 引擎：`crates/storage/src/indexed_db/`（~10k 行）——IdbKey/IdbKeyRange/
  IdbCursor/事务/object store CRUD 全 API 面 + 单测（types_coverage 5 轮 +
  tests_basic/advanced/edge）
- ✅ JS 近似：part02.js:2564 `globalThis.indexedDB` in-memory 实现（为 5 个 storage
  WPT smoke 不抛 not defined）
- ⚠️ 页面零接线：JS 近似与 Rust 引擎零关联
- ⚠️ 无持久化：indexed_db 无落盘路径
- ✅ WPT 首批：factory/global/event 9 文件、50 subtest，50 Pass / 0 Fail（100.00%）
- ✅ `IDBFactory.cmp`：真实 WPT 12/12（基线 4/12，净增 8）
- ✅ `IDBRequest` EventTarget：14 个基础设施失败归零，净增 5 Pass
- ✅ `indexedDB.open` version 转换：15 个同步校验 WPT 全绿
- ✅ 版本状态与 `IDBVersionChangeEvent`：delete/versionchange WPT 全绿，净增 7 Pass
- ✅ versionchange transaction：complete/abort/error 顺序与回滚，最后 9 Fail 全灭
- ✅ Object Store CRUD 首批：6 文件、54 subtest，54 Pass / 0 Fail（100.00%）
- ✅ Object Store getAll/getAllKeys：2 文件、34 subtest，34 Pass / 0 Fail（100.00%）
- ✅ Index get/getKey/count：3 文件、20 subtest，20 Pass / 0 Fail（100.00%）
- ✅ Index cursor continue：1 文件、8 subtest，8 Pass / 0 Fail（100.00%）
- 🟨 M2 页面接线：factory 与 object-store schema 已走 Rust；records CRUD/index/cursor 待迁移

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| I1 | WPT IndexedDB 用例覆盖为零 | 🟨 M1 已导入 21 文件 |
| I2 | 页面→Rust 引擎零接线 | 🟨 factory/schema 已接，records 仍在 JS |
| I3 | 无持久化（重启即失） | ⬜ M3 |
| I4 | IDBRequest 事件模型（success/error/readyState/auto-commit）非 spec | ⬜ M2/M3 |

## 下一步计划

1. **M2**：补 Rust Date key 与 IndexedDB key wire
2. **M2**：把 transaction/store CRUD 切到 Rust backend
3. **M2**：把 index/cursor 切到 Rust backend
4. **M3**：接入 per-origin 落盘与跨会话读回

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT IndexedDB 基线建立 | 🟨 imported 166/166 |
| M2 — JS↔Rust 接线（核心通路） | 🟨 factory/schema routed |
| M3 — 索引 + 事件模型 + 持久化 | ⬜ |

## 验证基线

- 测试基线：storage crate 既有单测全绿（立项时点）；clippy 零警告
- WPT IndexedDB factory 首批：9 文件 / 50 subtest / 50 Pass / 0 Fail / 100.00%
- WPT Object Store CRUD 首批：6 文件 / 54 subtest / 54 Pass / 0 Fail / 100.00%
- WPT Object Store getAll：2 文件 / 34 subtest / 34 Pass / 0 Fail / 100.00%
- WPT Index get/getKey/count：3 文件 / 20 subtest / 20 Pass / 0 Fail / 100.00%
- WPT Index cursor continue：1 文件 / 8 subtest / 8 Pass / 0 Fail / 100.00%
- imported 合计：21 文件 / 166 subtest / 166 Pass / 0 Fail / 100.00%
- 当前 100% 仅覆盖 imported 21 文件，不代表上游 IndexedDB 目录整体通过率
- 证据：`evidence/2026-08-17-m1-factory-baseline.{md,json}`、
  `evidence/2026-08-17-m1-cmp-fix.{md,json}`、
  `evidence/2026-08-17-m1-request-eventtarget-fix.{md,json}`、
  `evidence/2026-08-17-m1-open-version-fix.{md,json}`、
  `evidence/2026-08-17-m1-version-state-fix.{md,json}`、
  `evidence/2026-08-17-m1-factory-first-slice-final.{md,json}`、
  `evidence/2026-08-17-m1-crud-baseline.{md,json}`、
  `evidence/2026-08-17-m1-crud-range-fix.{md,json}`、
  `evidence/2026-08-17-m1-crud-store-guards.{md,json}`、
  `evidence/2026-08-17-m1-crud-key-validation.{md,json}`、
  `evidence/2026-08-17-m1-crud-cursor-continuation.{md,json}`、
  `evidence/2026-08-17-m1-crud-first-slice-final.{md,json}`、
  `evidence/2026-08-17-m1-getall-index-cursor-expansion.{md,json}`、
  `evidence/2026-08-17-m1-index-queries.{md,json}`、
  `evidence/2026-08-17-m1-second-slice-final.{md,json}`、
  `evidence/2026-08-17-m2-per-origin-registry.{md,json}`、
  `evidence/2026-08-17-m2-engine-wire-handler.{md,json}`、
  `evidence/2026-08-17-m2-three-host-registration.{md,json}`、
  `evidence/2026-08-17-m2-factory-schema-routing.{md,json}`
- 回归门禁：`make test` 全绿；期间修复 DMA-BUF 测试缺失 scroll-transform 前提、
  renderer idle-drain 启动期计数假设、QuickJS-only 测试 feature-union 门控
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
