# M3 embedded WebView IndexedDB owner

**日期**: 2026-08-18

## 结果

`zero-webview` 不再把独立内存 `StorageManager` 固定为唯一 IndexedDB owner。嵌入宿主可创建并克隆 `IndexedDbOwner`，通过 `WebView::new_with_indexed_db_owner` 或 `WebViewBuilder::indexed_db_owner` 注入共享或持久 owner。

## 行为

+ `IndexedDbOwner::in_memory()` 创建不落盘 owner，适合独立 private browsing context
+ `IndexedDbOwner::persistent(path)` 加载并持久化指定目录中的 per-origin 数据库
+ 同一 owner 的 clone 在多个 WebView 间共享数据库
+ 每个 WebView 仍有独立 transaction registry，避免跨实例复用 transaction/cursor ID
+ 未显式注入时保持原行为：每个 WebView 使用独立内存 owner
+ 持久 owner 初始化失败返回 `WebViewError::Storage`，不降级为空数据库

## 验证

+ Embedded WebView bridge：共享 owner 跨实例记录读回 Pass
+ Embedded WebView bridge：同 origin 的独立内存 owner 不可见普通数据 Pass
+ Embedded WebView bridge：持久 owner 销毁并重建后记录读回 Pass
+ `cargo check -p zero-webview --tests`：Pass
+ `make test`：Pass，含 V8、QuickJS、GPU adapter-only 和真实多进程测试
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ 固定 IndexedDB WPT：38 文件 / 222 Pass / 0 Fail
+ `make bench-gate`：16 / 16 microbench Pass；页面绝对预算与 retained form budget Pass

## 剩余

+ 完整跨 connection transaction scheduling 与 blocked/versionchange 事件
+ Browser navigation commit 的独立 storage-key authority
+ Successful database version 超过 Rust `u32`
