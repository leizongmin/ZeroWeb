# IndexedDB 事务、游标与连接事件

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取数据库升级 blocked、连接 versionchange、跨 renderer 事务竞争、游标变更与持久化失败。底座 `indexed_db/types.rs` 和 `cursor.rs` 保存数据语义，`indexed_db_host` 执行页面事务，browser owner 仲裁跨进程连接和租约。

## Gotchas
- Number 键拒绝 NaN 但允许 Infinity，Date 键要求有限数；JSON wire 用字符串保存数值以保留 Infinity 和负零，不能改成普通 JSON number 而丢掉键身份。 (`crates/page-runtime/src/indexed_db_host.rs`, `crates/storage/src/indexed_db/types.rs`)
- 简化 Rust 游标 advance(0) 会重置位置，而页面宿主另有完整方向和参数语义；不能直接使用底座游标方法推断 JS IDBCursor 行为。 (`crates/storage/src/indexed_db/cursor.rs`, `crates/page-runtime/src/indexed_db_host/cursor.rs`)
- Rust 游标推进使用 saturating_add，极大 count 只令游标结束；继续推进不能回绕成首项，也不能用裸加法重新引入 debug panic。 (`crates/storage/src/indexed_db/cursor.rs`)
- browser 在导航提交前拒绝 IDB，限制请求正文为 8 MiB，并用 indexed_db_origins 中的宿主 origin 执行；renderer handler 的 _origin 参数不作为授权来源。 (`apps/browser/src/process_backend.rs`, `apps/renderer/src/ipc_indexed_db.rs`)
- 连接升级请求需等待 versionchange 事件 ack，再判断旧连接是否关闭；Pending 与 Blocked 不是同一状态，不能在未 ack 时就报告 blocked 或启动升级。 (`apps/browser/src/process_backend/indexed_db_connections.rs`)

## Architecture
- 连接队列按 private、origin、database 隔离，request_scope/status 校验 requester renderer；知道 request id 并不足以轮询别人的升级请求。 (`apps/browser/src/process_backend/indexed_db_connections.rs`)
- 跨 renderer begin_transaction 必须持有 owner 调度的 lease；缺 lease 返回 SecurityError，开始失败会取消 lease，commit/abort 完成时释放调度占用。 (`apps/browser/src/process_backend/indexed_db_transactions.rs`)
- 页面游标保存 index key、primary key、方向、query 与 mutation_generation，具备 nextunique/prevunique；它与仅保存键列表的简化底座游标是不同层，修复排序应追真实页面入口。 (`crates/page-runtime/src/indexed_db_host/cursor.rs`)
- 持久写入使用 create_new 临时文件、sync_all、替换目标及目录同步；恢复清理 tmp 并处理 bak，加载还验证路径与元数据 origin/name 对应并拒绝重复数据库。 (`crates/storage/src/indexed_db/persistence.rs`)
- IDB 回包路由器同时协作 Service Worker 路由，IPC 断开时 fail_all 解除等待；不能把所有非 IDB 消息吞掉，否则会阻塞其他宿主命令。 (`apps/renderer/src/ipc_indexed_db.rs`)
