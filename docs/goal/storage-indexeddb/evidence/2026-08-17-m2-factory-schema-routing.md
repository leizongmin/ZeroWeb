# M2 factory 与 object-store schema 路由

**日期**: 2026-08-17

## 结果

页面 `indexedDB` 的 factory 状态与 object-store schema 已接入 `zero-storage`。页面先用只读
`inspect` 恢复 schema，versionchange 成功后以 `sync_schema` 提交最终快照；abort 不修改 Rust。

## 行为

+ `open()` 从 Rust 恢复数据库 version、store name、keyPath、autoIncrement
+ `createObjectStore()` / `deleteObjectStore()` 在 versionchange 成功后同步到 Rust
+ versionchange abort 保持 Rust version/schema 不变
+ 同名 store 在版本升级中允许 delete 后以不同 metadata 重建
+ 同版本 schema 变更被 host 拒绝
+ `deleteDatabase()` 与 `databases()` 走 Rust per-origin registry
+ renderer 重建 JS context 后可从同一 worker 的 Rust handler 恢复 schema
+ CRUD/index/cursor records 仍由 JS Map 持有，尚未满足 DC-2

## 验证

+ page-runtime schema handler：4 Pass / 0 Fail
+ WebView factory/schema host E2E：1 Pass / 0 Fail
+ renderer 跨 document schema 恢复：lib + bin 2 Pass / 0 Fail
+ engine IndexedDB fallback：13 Pass / 0 Fail
+ imported IndexedDB WPT：21 文件 / 166 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass（默认 V8、adapter-only GPU、QuickJS Clippy 与 QuickJS 运行测试）

## 下一步

为 Rust `IdbKey` 增加 Date 类型与排序/哈希语义，再设计 transaction-scoped CRUD wire。
