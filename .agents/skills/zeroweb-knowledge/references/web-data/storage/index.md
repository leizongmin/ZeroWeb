# 存储后端与宿主所有权

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 Web Storage、IndexedDB、CacheStorage、OPFS、持久 owner 和页面/IPC 信任边界。任务先按数据类型选主题，再查嵌入或 browser owner；协议消息结构由 runtime 领域维护。

### 验证入口

[存储功能地图](../../../../zeroweb-verify/features/storage.md) 提供底座恢复判据及页面未覆盖项；本次没有运行产品验证。

## Architecture
- StorageManager 不包含 OPFS 或 SW runtime；数据库、缓存、文件树和注册状态的持久 owner 有各自格式和生命周期，不能以统一清理或统一恢复的假设修改其中一个。 (`crates/storage/src/storage_manager.rs`, `crates/page-runtime/src/opfs_host.rs`, `crates/page-runtime/src/service_worker_manager.rs`)
- 页面 IndexedDB、CacheStorage、OPFS 各有 handler，嵌入 owner 提供具体实现；browser 的 IDB 权限与事务仲裁另在主进程，不能直接把 renderer wire 参数视为可信存储上下文。 (`crates/webview/src/indexed_db_owner.rs`, `apps/browser/src/process_backend.rs`)

## Child Knowledge Nodes
- [Web Storage、profile、持久化与配额](web-storage-and-persistence.md) — 键顺序、配额、目录布局与浏览上下文共享。
- [IndexedDB 事务、游标与连接事件](indexeddb.md) — versionchange ack、blocked、租约、键身份与恢复。
- [Cache API、OPFS 与跨进程存储桥](cache-api-and-opfs.md) — 响应类型、缓存对象身份、文件写流与持久化边界。
