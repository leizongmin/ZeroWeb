# OPFS 真实化 — 运行时控制面板（master.md）

**入口文档**: [../storage-opfs.md](../storage-opfs.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-09（M2 切片 1 完成——JS 接线 + Rust 真实后端，通过率 36%→95%）

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
- ✅ **M2 切片 1 已落（2026-09-09）**：
  - engine `opfs_bridge.rs`：`OpfsBridge` + `OpfsHandler` wire 契约（照 cache_storage_bridge），
    注册 `__zw_opfs`（`__zw_opfs_ok:` / `__zw_opfs_error:Name|code|message`）
  - page-runtime `opfs_host.rs`：JSON 请求 dispatch → per-origin OpfsFileSystem；
    `opfs_handler(Option<root>)`——Some=落盘 / None=内存；磁盘错误降级内存不 panic；
    6 个 host 单测（含持久化重开恢复）
  - webview 接线：`IndexedDbOwner` 增 `opfs_root`（persistent owner → `<root>/OPFS`，
    in-memory owner → None）；`WebView` 注册 `opfs_bridge`（ensure_sandbox 统一注册点）
  - part02.js navigator.storage 段重写（**kill-switch**：`__zw_opfs` 未注册 → 内存虚拟树
    回退路径，同构面）：句柄=路径+kind、async iterator 全家（keys/values/entries/句柄
    直迭代=entries 对）、真 WritableStream 子类（getWriter/pipeTo/locked + write/seek/
    truncate/close 直通）、SyntaxError 缺参面、全局 FileSystem*Handle 构造器、getFile
    Blob 元数据（name/lastModified）

## M2 后 WPT 成绩（2026-09-09）

- **13 用例 / 133 subtests：126 Pass / 7 Fail（95%）**——基线 48/85（36%），
  净 +78；证据 `evidence/2026-09-09-fs-m2-rust-backend.{md,json}`
- 剩余 7 失败：postMessage 克隆 ×3（skip 域）、上游 pinned-rev 用例签名缺陷 ×1、
  blob 失效快照 ×1（M3 深化）、fetch response.body 流 ×2（fetch 域非本 goal）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | WPT 用例覆盖 + 基线报告 | ✅ M1（13 用例 / 48P-85F 基线） |
| P2 | zero-storage opfs 模块（目录树/句柄/读写流/持久化底座） | ✅ M1（骨架 + 流语义 + 持久化底座全落） |
| P3 | JS 接线（内存虚拟树 → host 命令 → Rust；async iterator + DOMException 面） | ✅ M2 切片 1（95% 通过率，kill-switch 保留内存回退） |
| P4 | 持久化 e2e（per-origin 落盘 + 跨会话 + engine 接线后） | ⬜ M3（webview 层已就绪，缺 e2e 断言） |

## 下一步计划（M3 剩余）

1. **M3 切片 2 estimate 真实化**：estimate() 已走 hostCall usage（真实字节数）——补断言
   + quota 语义评估（静态 100MB 是否入 skip/近似清单注明）
2. **M3 切片 3 sync access handle 评估定论**：worker 环境专用 + headless worker 无真线程 →
   预期记入 skip（Support Envelope 允许「评估后决定做或记入 skip」）；记录评估理由
3. **M3 切片 4 剩余语义**：blob 失效快照检测（低优先）；skip 清单正式化（postMessage 克隆
   ×3、createSyncAccessHandle*、fetch response.body ×2——后两者属 fetch/stream 域，与上游
   用例缺陷项一并注明归属）
4. **DC 全满足判定**：make test 全绿 + clippy 零警告 + 通过率报告持久化 + master.md 自洽
   + archive 建立

> M3 切片 1（2026-09-09 已落）：crates/webview/src/tests/opfs_owner.rs——
> persistent_owner_opfs_file_survives_webview_rebuild（DC-3 跨会话 e2e）、
> opfs_files_are_origin_scoped（per-origin 隔离）、in_memory_owner_opfs_does_not_persist
> （kill-switch 回退不落盘）、persistent_owner_opfs_directory_tree_roundtrip（嵌套目录树）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + opfs 模块骨架 | ✅ 2026-09-09（13 用例导入 + 48P/85F 基线 + opfs 模块 19 单测） |
| M2 — 读写流 + JS 接线 | ✅ 2026-09-09 切片 1（接线完成，95% 通过率；DC-2 语义面全满足） |
| M3 — 持久化 + 收尾 | 🔶 切片 1 ✅ 2026-09-09（跨会话 e2e ×4：persistent 重建读回一致 / per-origin 隔离 / in-memory 不落盘 / 目录树往返）；剩：estimate 断言 + SAH 评估定论 + skip 清单正式化 |

## 验证基线

- WPT OPFS 面：`make testharness-fs`——13 用例 / 133 subtests；基线 48P/85F（36%，
  内存 shim 版）→ M2 后 126P/7F（95%，Rust 后端）；证据 `evidence/2026-09-09-*`
- 测试基线：`make test` / `make reftest` 入口（test-guard 包裹；禁止裸跑 cargo test）；
  zero-storage opfs 单测 19 个 + page-runtime opfs host 单测 6 个 + engine bridge 测试
  4 个全绿（2026-09-09）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
  （含 quickjs feature 门禁步）；渲染相关变更（本目标预期无）才需 product-smoke / bench-gate
