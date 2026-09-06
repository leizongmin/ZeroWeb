# OPFS 真实化 — 运行时控制面板（master.md）

**入口文档**: [../storage-opfs.md](../storage-opfs.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（立项——M1 待启动）

---

## 当前状态

**专项定位**：存储三件套之三（IndexedDB 2026-08-19 收口、Cache API 2026-09-06 收口）。
把页面 `navigator.storage.getDirectory()` 的 OPFS 从 JS shim 内存虚拟树（`part02.js`
行 2755–2972）升级为 zero-storage 真实实现 + per-origin 持久化。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域，本流与其 crate 域（css-parser/style-system/layout-engine/
  render-foundation）零重叠；engine 共享面（`js_dom_shim/part02.js`）按 run-rules §9
  `git log` 核对后再动
- 已归档 storage-indexeddb / storage-cache-api — 持久化模式参照实现（只读参照）
- page-wasm — 无共享面

## 实测基线（2026-09-07 立项时）

### 现有实现

- ✅ JS shim 内存近似版：`part02.js` navigator.storage 段——getDirectory/getFileHandle/
  getDirectoryHandle/createWritable（write/seek/truncate/close/abort）/removeEntry/keys/
  entries/values/isSameEntry/名称校验 `_zwFsValidName`；bridge 测试 part16.rs
  （R3314/R3315/R3254 系列）
- ✅ 持久化模式可复用：`storage_manager.rs` 根目录机制（`ZERO_STORAGE_DIR` env）+
  indexed_db/persistence.rs、cache_api/persistence.rs 的「JSON + 临时文件 + replace_file +
  sync_directory + 中断恢复」模式
- ⚠️ zero-storage 无任何 OPFS/File System 对应物（grep 零命中）——**核心缺口**
- ⚠️ 内存版不持久化（跨页/进程丢失）
- ⚠️ WPT 上游 `fs` 用例未导入（无 fetch 脚本、无 skip list、`imported-testharness.txt`
  无条目）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | WPT 用例覆盖为零（fetch 脚本 + 导入 + 基线报告） | ⬜ M1 |
| P2 | zero-storage opfs 模块（目录树/句柄/读写流）不存在 | ⬜ M1-M2 |
| P3 | JS 接线（内存虚拟树 → host 命令 → Rust） | ⬜ M2 |
| P4 | 持久化（per-origin 落盘 + 跨会话 e2e + 中断恢复） | ⬜ M3 |

## 下一步计划

1. **M1 切片 1**：`scripts/fetch-fs-subset.sh`（照 fetch-cache-storage-window-subset.sh）
   + 用例导入 + 分类通过率基线（内存版表现即验收清单）
2. **M1 切片 2**：zero-storage `opfs` 模块骨架——目录树数据结构 + 名称校验 +
   getDirectoryHandle/getFileHandle + 单测
3. **M1 切片 3**：失败聚类 → 修复队列

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对渲染流域活跃面；有活跃编辑则先做零碰撞面（Rust opfs 模块、WPT 导入、storage 单测）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + opfs 模块骨架 | ⬜ 待启动 |
| M2 — 读写流 + JS 接线 | ⬜ |
| M3 — 持久化 + 收尾 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）
- OPFS 用例面：无基线（未导入/未建）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  渲染相关变更（本目标预期无）才需 product-smoke / bench-gate
