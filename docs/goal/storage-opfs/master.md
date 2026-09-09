# OPFS 真实化 — 运行时控制面板（master.md）

**入口文档**: [../storage-opfs.md](../storage-opfs.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-09（M1 完成——WPT 基线建立 + opfs 模块骨架）

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

**2026-09-09 实测**：`git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
活跃编辑来自 service-worker / editing / keyboard goal（非渲染流域），`part02.js` 近 14 天
无渲染流碰撞。本日两切片均落零碰撞面（WPT 导入 + storage crate）。

## 实测基线（2026-09-09 M1 收口时）

### WPT 基线（内存 shim 版 = 真实化对照起点）

- **导入**：`fs/` window 可执行 13 用例（WPT pin `315976933870b34d6ea30e3f6643403edae678ba`，
  与 storage-indexeddb 同版）；`make testharness-fs` 执行通道（test-guard 包裹，FILTER 透传）；
  `imported-testharness.txt` 记账 13 条（OPFS-M1-baseline）
- **基线成绩**：133 subtests，**48 Pass / 85 Fail（36%）**——完整报告与失败聚类见
  `evidence/2026-09-09-fs-baseline-inmemory-shim.md`
- **skip 域**（fetch 脚本头注释，不充数）：postMessage*（transferable）、
  create-sync-access-handle*（Worker，M3 评估）、move（页面级 move 语义未排期）、
  FileSystemObserver*（tentative）、IndexedDB/buckets 序列化、opaque-origin、bfcache

### 失败聚类（10 类，M2 修复队列的输入）

1. keys/values/entries 返回数组非 async iterator + 目录无 `Symbol.asyncIterator`（~25）
2. 错误类型 TypeError 应为 DOMException（NotFoundError=8/TypeMismatchError=17/
   NoModificationAllowedError=7）（~10）
3. `handle.remove({recursive})` 缺失（10）
4. `getUniqueId()` 缺失（11）
5. `resolve(child)` 缺失（5）
6. writable 流非真 WritableStream：pipeTo/getWriter/queued 命令/单 close 成功（~16）
7. Blob write 路径空 + keepExistingData 定位错误（~6）
8. getFile() Blob 元数据缺失 name/lastModified + slice（3）
9. truncate 指针钳位语义（~4）
10. postMessage 克隆句柄（3，skip 域）

### Rust 层现状

- ✅ **M1 切片 2 已落**：zero-storage `opfs` 模块（`crates/storage/src/opfs.rs` +
  `opfs/persistence.rs`）——OpfsFileSystem（路径=句柄值语义 / 字典序迭代）+ OpfsError
  （name+legacy code 对齐 WPT assert_throws_dom）+ OpfsWriteCommand 流全语义
  （truncate 钳指针 / 显式 position 不前进指针 / 单 close 成功 / abort 弃缓冲）+
  is_valid_name + generate_unique_id（UUID v4）+ OpfsPersistence（per-origin 落盘，
  照 cache_api 模式：JSON + 临时文件 + rename + fsync + 中断恢复）+ 19 个单测
- ⚠️ JS 接线未动（`part02.js` 仍是内存虚拟树）——**M2 核心**
- ⚠️ estimate 未接 usage_bytes；getFile 元数据接线未动

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | WPT 用例覆盖 + 基线报告 | ✅ M1（13 用例 / 48P-85F 基线） |
| P2 | zero-storage opfs 模块（目录树/句柄/读写流/持久化底座） | ✅ M1（骨架 + 流语义 + 持久化底座全落） |
| P3 | JS 接线（内存虚拟树 → host 命令 → Rust；async iterator + DOMException 面） | ⬜ M2 |
| P4 | 持久化 e2e（per-origin 落盘 + 跨会话 + engine 接线后） | ⬜ M3 |

## 下一步计划（M2）

1. **接线设计**：host 命令集（`__zw_opfs_*` 约定）+ origin 归属（page URL → StorageManager
   per-origin OpfsFileSystem）+ kill-switch（env/flag 切内存版/真实版，A/B 零回归）
2. **JS 面重构**（engine 共享面，动前再核对 git log）：句柄缓存路径化；keys/values/entries
   改 async iterator（`Symbol.asyncIterator` + next()）；错误改 DOMException（Rust 返回
   `name|code|message` 编码映射）；createWritable 换真 WritableStream 语义（pipeTo/getWriter/
   队列）——可先走「Rust 实现 + JS 包装」渐进路线
3. **M2 验收**：`make testharness-fs` 通过率显著抬升（聚类 1/2/3/4/5 清零为主要目标）；
   `make test` 全绿 + clippy 零警告
4. **M3**：持久化 e2e（engine 重建 → 读回一致）、estimate 真实化、sync access handle 评估定论

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + opfs 模块骨架 | ✅ 2026-09-09（13 用例导入 + 48P/85F 基线 + opfs 模块 19 单测） |
| M2 — 读写流 + JS 接线 | ⬜ 下一步（见上方计划 1-3） |
| M3 — 持久化 + 收尾 | ⬜ |

## 验证基线

- WPT OPFS 面：`make testharness-fs`——13 用例 / 133 subtests，基线 48P/85F（2026-09-09，
  内存 shim 版）；证据 `evidence/2026-09-09-fs-baseline-inmemory-shim.{md,json}`
- 测试基线：`make test` / `make reftest` 入口（test-guard 包裹；禁止裸跑 cargo test）；
  zero-storage opfs 单测 19 个全绿（2026-09-09）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  渲染相关变更（本目标预期无）才需 product-smoke / bench-gate
