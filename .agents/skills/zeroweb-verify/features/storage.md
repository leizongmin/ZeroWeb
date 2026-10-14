# F07 站点存储与离线使用

## 用户任务与预期

应用保存数据、关闭/重新创建存储 owner 后恢复内容；不同源隔离；写失败不能污染旧数据。
预期来自 `StorageManager` 持久化契约及既有测试，不以当前实现覆盖率替代网页兼容性。

## 已实跑入口

从项目根执行 [入口中的 helper](../SKILL.md)。无浏览器、身份或外部网络依赖。
helper 将 TMPDIR 和 Cargo target 隔离为本次资源，离线编译 `zero-storage --lib`；
生产测试 `TestDirectory` 准备全新的数据库路径，真正调用生产持久化 API，不用 mock。

| 操作 | 观察与独立判据 |
|---|---|
| 新建数据库 app v3、items/keys、multiEntry/复合索引，写入 Ada，删除 key 10，再重开 | 版本 3、items 数量 1、按 code 索引读 Ada；array/binary/-0 键恢复；第二 origin 不含 items；下一生成键 11 |
| 写入命名缓存 empty/assets，重建 manager，查询忽略 search 但保留 Vary | 缓存名顺序和空缓存恢复；status 201、Created、basic、Content-Type、body `cached body` 正确 |
| 删除最后一个 cache 后重新打开 | 该 origin 无缓存，不从磁盘恢复已删除缓存 |
| 用非目录阻断持久化后尝试新写入 | 返回 StorageError::Io，旧记录/缓存仍可读，新内容不存在 |
| 准备损坏 JSON | 返回 Serialization 错误，不能静默视作成功恢复 |
| 仅留下备份和 orphan tmp 后重开 | 数据库恢复、备份及 orphan 消失，目标 JSON 存在 |

副作用是真实本地 JSON/缓存文件和 live manager 状态。manager 销毁后重新读取磁盘，
不是对同一内存副本自证。正常/失败测试都由 Drop 清理自建路径；helper 检查 `tmp/` 空，
删除本次 target/guard 后重读证据。测试失败保留 FAIL，不改产品代码。

## 未覆盖入口与方法

网页 `indexedDB/caches` 的 JS 桥、renderer IPC、WebStorage/cookies、OPFS、Service Worker
注册/控制/fetch 拦截均 `NOT_RUN`。不能把上述底座证据描述为完整网站离线通过。
需要页面 CacheStorage 验证时复用
`crates/webview/src/tests/cache_storage.rs::persistent_owner_restores_page_cache_storage_after_webview_rebuild`；
该路径检查 `assets|202|Accepted|text/plain|restored body` 及另一源 missing。
它仍不是 GUI/IPC；其中 add/addAll 的 fetch_handler 测试替代了网络边界，不能证明真实网络。

网页闭环应使用本次 origin/独立 `ZERO_STORAGE_DIR`，经 live 页面写入/读取、重建 owner、
重开验证，再删除本次数据；SW 用本地 HTTP fixture 验证 respondWith 返回字节及 unregister。
配额失败观察错误传播和旧值保持。权限/安全上下文不足时 BLOCKED，不降低预期。

## 来源与证据

生产：`crates/storage/src/storage_manager.rs`、`indexed_db/persistence.rs`、
`cache_api/persistence.rs`；断言：
`crates/storage/src/storage_manager/persistence_tests.rs`（7 项）。
消费者：`crates/webview/src/indexed_db_owner.rs` 与 `apps/browser/src/process_backend/indexed_db_transactions.rs`。
知识：[存储说明](../../../../crates/storage/README.md)、
[工程方法论](../../zeroweb-guidelines/SKILL.md)。

2026-10-14，HEAD `e523b1c73b96360a6451948f0b748b7b4657750f`，macOS 26.6.2 arm64，
Rust/Cargo 1.98.1、Node 24.15.0，相关产品源码无差异；并行 AGENTS/知识资产变化不归为 HEAD。
本次 `PASS`：7/7 持久化生产 API 测试；其他入口 `NOT_RUN`。
[机器证据](../.verify-task/20261014-storage-init/result.json)、
[实际断言日志](../.verify-task/20261014-storage-init/test.log)。
自建数据库和编译资源已清理，日志/JSON 清理后仍可读；见 [初始化记录](../initialization.md)。
