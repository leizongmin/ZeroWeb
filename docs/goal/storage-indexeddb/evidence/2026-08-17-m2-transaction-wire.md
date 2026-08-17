# M2 transaction-scoped CRUD wire

**日期**: 2026-08-17

## 结果

`zero-page-runtime` 已建立每个 IndexedDB handler 独立的 transaction registry，支持跨同步 callback
保存 Rust `IdbTransaction`，并通过真实 mutation buffer 执行 begin/add/put/get/delete/commit/abort。

## 行为

+ transaction ID 单调生成并绑定 origin 与 database
+ 跨 origin 使用 transaction ID 返回 `SecurityError`
+ readonly transaction 写操作返回 `ReadOnlyError`
+ add/put/delete 在 commit 前只进入 Rust mutation buffer
+ transaction get 可读到同事务 buffered mutation
+ abort 丢弃 mutation 与局部 key generator
+ commit 复用 storage 层原子预检与写入
+ Date、Infinity、Binary、Array 等 key 统一走递归 key wire

## 验证

+ page-runtime IndexedDB handler：6 Pass / 0 Fail
+ transaction 定向场景覆盖 buffered read、abort、autoIncrement commit、readonly、origin isolation
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass（默认 V8、adapter-only GPU、QuickJS Clippy 与 QuickJS 运行测试）

## 下一步

给 JS `IDBTransaction` 绑定 host transaction ID，将 object store add/put/get/delete 路由到该状态机。
