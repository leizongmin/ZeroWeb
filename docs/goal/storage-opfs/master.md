# OPFS 真实化 — 运行时控制面板（master.md）

**入口文档**: [../storage-opfs.md](../storage-opfs.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-09（M1-M3 全部落毕——DC-1~4 全满足，DONE 判定成立）

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

## 下一步计划（DONE 判定复核）

M1-M3 全部落毕，DC-1~4 逐项判定见下方「Done Criteria 判定」。剩余：全量 make test /
clippy 终验 + archive 建立。

> M3 切片 1（2026-09-09 已落）：crates/webview/src/tests/opfs_owner.rs——
> persistent_owner_opfs_file_survives_webview_rebuild（DC-3 跨会话 e2e）、
> opfs_files_are_origin_scoped（per-origin 隔离）、in_memory_owner_opfs_does_not_persist
> （kill-switch 回退不落盘）、persistent_owner_opfs_directory_tree_roundtrip（嵌套目录树）。
> M3 切片 2（已落）：test_opfs_estimate_real_usage_m3——写入 5 字节 usage 差值精确 5。
> M3 切片 3（已落）：SAH 评估定论=skip（evidence/2026-09-09-m3-skip-list-and-sah-verdict.md §1）。
> M3 切片 4（已落）：skip 清单正式化（同上 §2/§3）。

## Done Criteria 判定（2026-09-09 终验）

| DC | 判定 | 依据 |
|----|------|------|
| **DC-1 WPT 用例导入与通过率基线** | ✅ | `scripts/fetch-fs-subset.sh`（照 indexeddb 先例，pin 31597693）+ fs/ window 可执行 13 用例导入（`make testharness-fs`）+ 基线报告持久化（`evidence/2026-09-09-fs-baseline-inmemory-shim.{md,json}`：133 subtests 48P/85F）+ `imported-testharness.txt` 记账 13 条（OPFS-M1-baseline）+ window picker 域（postMessage*/SAH*/move/Observer 等）fetch 脚本头 skip 注明 |
| **DC-2 页面走真实引擎** | ✅ | zero-storage opfs 模块全 API 面（目录树/句柄/读写流，19 单测）+ `part02.js` 切换 `__zw_opfs` 宿主命令→opfs 模块（kill-switch：`__zw_opfs` 未注册→内存虚拟树同构回退，`in_memory_owner_opfs_does_not_persist` 断言其不落盘）+ 句柄身份/名称校验/错误类型语义与 spec 一致（WPT 为准：126P/7F，剩余 7 项全部归类域外——见 skip 定论） |
| **DC-3 持久化** | ✅ | per-origin 落盘（OpfsPersistence，照 cache_api 模式）+ 跨会话 e2e（`persistent_owner_opfs_file_survives_webview_rebuild`：写入→重建 WebView→读回一致；page/WebView owner 面）+ 磁盘错误不 panic（opfs_handler 打开失败降级内存态；flush 错误经 wire 层 reject）+ 中断恢复（`.tmp`/`.bak` 扫描，`test_persistence_roundtrip`） |
| **DC-4 测试与质量不可退让** | ✅ | `make test` 全绿（2026-09-09 终验，67 test binary 零 FAILED，含 quickjs clippy 门禁步）+ `cargo clippy --workspace --all-targets -- -D warnings` 零警告 + 每项修复带单测（storage 19 / page-runtime host 6 / engine bridge 6 / webview e2e 4 = 35 个新测试）+ driving WPT 用例资产化（13 用例记账） |

**判定：DC-1~4 全部满足，DONE 条件成立。**

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + opfs 模块骨架 | ✅ 2026-09-09（13 用例导入 + 48P/85F 基线 + opfs 模块 19 单测） |
| M2 — 读写流 + JS 接线 | ✅ 2026-09-09 切片 1（接线完成，95% 通过率；DC-2 语义面全满足） |
| M3 — 持久化 + 收尾 | ✅ 2026-09-09（切片 1 e2e ×4 + 切片 2 estimate 真实化断言 + 切片 3 SAH 评估定论=skip + 切片 4 skip 清单正式化——见 evidence/2026-09-09-m3-skip-list-and-sah-verdict.md） |

## 验证基线

- WPT OPFS 面：`make testharness-fs`——13 用例 / 133 subtests；基线 48P/85F（36%，
  内存 shim 版）→ M2 后 126P/7F（95%，Rust 后端）；证据 `evidence/2026-09-09-*`
- 测试基线：`make test` / `make reftest` 入口（test-guard 包裹；禁止裸跑 cargo test）；
  zero-storage opfs 单测 19 个 + page-runtime opfs host 单测 6 个 + engine bridge 测试
  4 个全绿（2026-09-09）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
  （含 quickjs feature 门禁步）；渲染相关变更（本目标预期无）才需 product-smoke / bench-gate
