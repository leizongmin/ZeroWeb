# Web Storage、profile、持久化与配额

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取键序、配额、浏览上下文共享、磁盘恢复及无痕隔离问题。本文区分 `WebStorage` 内存对象、`StorageManager` 持久化后端和 WebView/browser owner，不能以 StorageType::Local 名称推断已经落盘。

## Gotchas
- WebStorage 以 IndexMap 保存插入序：覆写保留位置、删除使用 shift_remove、重加移到末尾；换成 HashMap 或 swap_remove 会改变 key(n) 可见顺序。 (`crates/storage/src/local_storage.rs`)
- WebStorage 配额计 key.len()+value.len() 的 UTF-8 字节，并在覆写前减去旧项；超限不更新 used_bytes 或旧值，不能将其描述为 UTF-16 code unit 配额。 (`crates/storage/src/local_storage.rs`)
- Rust WebStorage::set 拒绝空键，但网页 Web Storage 的 JS 路径与该 Rust API 并非必然同一语义；不能从 Rust 底座测试直接推断 document.localStorage 空键兼容性。 (`crates/storage/src/local_storage.rs`, `crates/webview/src/webview.rs`)
- StorageManager::with_persistence 只开启 IndexedDB 与 CacheStorage，local_stores/session_stores 仍从空 HashMap 开始；该构造器不是所有网页存储的统一恢复入口。 (`crates/storage/src/storage_manager.rs`)
- StorageManager 的 sessionStorage 只按 origin 分组，不带 tab 键；宿主必须通过 owner 生命周期/上下文隔离决定共享范围，不能把一个 manager 随意共享给互不相关标签。 (`crates/storage/src/storage_manager.rs`)

## Architecture
- 同一浏览上下文的嵌入 WebView 应克隆同一个 IndexedDbOwner；其 persistent(path) 将 IndexedDB 放 path、CacheStorage 放 path/CacheStorage、OPFS 放 path/OPFS，和 StorageManager::with_persistence(root) 的目录布局不同。 (`crates/webview/src/indexed_db_owner.rs`, `crates/storage/src/storage_manager.rs`)
- browser 普通数据使用持久化 storage，私密数据使用另一个 StorageManager::new；IndexedDB 请求按宿主记录的标签隐私状态选择 manager，不接受 renderer 自报 origin 作为权限依据。 (`apps/browser/src/process_backend.rs`)
- ZERO_STORAGE_DIR 的默认函数并非统一追加同一子目录：default_indexed_db_dir 直接返回该值，CacheStorage/OPFS 追加各自目录，Service Worker 追加状态文件；迁移目录必须逐 owner 核对。 (`crates/storage/src/storage_manager.rs`)

## Patterns
- 数据库和缓存的持久写入应先完成候选快照落盘，再替换 live 状态；持久化测试包含重建 manager、I/O 失败旧值保留和损坏数据拒绝，不能用内存读回代替恢复检查。 (`crates/storage/src/storage_manager.rs`, `crates/storage/src/storage_manager/persistence_tests.rs`)
