# storage-opfs 目标完成归档

**归档日期**: 2026-09-09
**判定**: DC-1~4 全满足，DONE（判定详情见 [../master.md](../master.md)「Done Criteria 判定」）

## 时间线

| 日期 | 里程碑 | 内容 |
|---|---|---|
| 2026-09-07 | 立项 | 从父目标 zero-web.md Tier 2 存储线拆出（IndexedDB 2026-08-19 收口、Cache API 2026-09-06 收口，OPFS 为最后一块） |
| 2026-09-09 | M1 ✅ | WPT fs/ window 子集 13 用例导入 + 内存 shim 基线（48P/85F，36%）+ zero-storage opfs 模块（19 单测） |
| 2026-09-09 | M2 ✅ | `__zw_opfs` 宿主桥 + page-runtime dispatch + webview 接线 + shim 切换（kill-switch 保留内存回退）→ 126P/7F（95%），净 +78 |
| 2026-09-09 | M3 ✅ | 持久化跨会话 e2e ×4 + estimate 真实化断言 + SAH 评估定论=skip + skip 清单正式化 |

## 交付物

- **crates/storage/src/opfs.rs + opfs/persistence.rs**：OpfsFileSystem / OpfsError /
  OpfsWriteCommand 流全语义 / generate_unique_id / OpfsPersistence（per-origin 落盘 +
  中断恢复）；`default_opfs_dir()`（storage_manager）
- **crates/engine/src/opfs_bridge.rs**：OpfsHandler wire 契约 + `__zw_opfs` 同步回调
- **crates/page-runtime/src/opfs_host.rs**：14 op JSON dispatch + per-origin 态管理 +
  磁盘错误降级
- **crates/webview**：IndexedDbOwner.opfs_root + WebView opfs_bridge 注册 +
  tests/opfs_owner.rs e2e ×4
- **crates/engine/src/js_dom_shim/part02.js**：navigator.storage 段真实后端切换
  （kill-switch 同构回退）
- **tests/wpt-runner**：fetch-fs-subset.sh + testharness-fs 模式 + Makefile 双 target +
  imported-testharness.txt 13 条记账
- **测试**：35 个新单测/e2e（storage 19 + host 6 + bridge 6 + webview 4）

## 验证证据（evidence/）

- 2026-09-09-fs-baseline-inmemory-shim.{md,json} — M1 内存版基线（对照起点）
- 2026-09-09-fs-m2-rust-backend.{md,json} — M2 Rust 后端成绩（95%）
- 2026-09-09-m3-skip-list-and-sah-verdict.md — SAH 定论 + skip 清单正式化

## 后续（非本 goal 承诺）

- blob 失效快照检测（低优先深化项，如实保留在通过率分母内）
- Worker 真线程落地后可附带实现 createSyncAccessHandle（重启条件见 SAH 定论）
- postMessage 结构化克隆句柄（transferable 域）、fetch response.body 流（fetch 域）——
  各归其域
