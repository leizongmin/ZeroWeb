# M3 JavaScript safe-integer database versions

**日期**: 2026-08-18

## 结果

IndexedDB database version 的 Rust 类型链从 `u32` 提升为 `u64`，与页面已实现的 WebIDL `[EnforceRange] unsigned long long` 和 JavaScript safe-integer 限制一致。`Number.MAX_SAFE_INTEGER`（9,007,199,254,740,991）可完成 open、upgrade event、查询和持久化重建，不再在 `2^32` 截断。

## 覆盖链路

+ `StorageManager::open_indexed_db`、`IndexedDbInfo` 和 `IdbDatabase`
+ Transaction snapshot 的 `db_version`
+ Page-runtime `open` / `sync_schema` request wire
+ JSON persistence DTO 与旧 `u32` 数值文件的向后读取
+ `IDBVersionChangeEvent.newVersion` 和 `IDBDatabase.version`

## 验证

+ Storage manager open/list：`Number.MAX_SAFE_INTEGER` Pass
+ Page-runtime host response：`Number.MAX_SAFE_INTEGER` Pass
+ Persistent manager rebuild：`Number.MAX_SAFE_INTEGER` Pass
+ WebView `upgradeneeded.newVersion`：`Number.MAX_SAFE_INTEGER` Pass
+ WebView `IDBDatabase.version`：`Number.MAX_SAFE_INTEGER` Pass
+ `make test`：Pass，含 V8、QuickJS、GPU adapter-only 和真实多进程测试
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ 固定 IndexedDB WPT：38 文件 / 222 Pass / 0 Fail
+ `make bench-gate`：16 / 16 microbench Pass；页面绝对预算与 retained form budget Pass

## 剩余

+ 完整跨 connection transaction scheduling 与 blocked/versionchange 事件
+ 扩大固定 revision 上游 IndexedDB WPT 导入范围
