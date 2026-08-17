# M2 object-store CRUD/query 路由

**日期**: 2026-08-17

## 结果

页面普通 `IDBTransaction` 已绑定 Rust transaction ID。Object store 的 add、put、get、delete、
range delete、clear、count、getAll、getAllKeys 通过 `zero-storage` transaction mutation/view 执行。

## 行为

+ readwrite mutation 在 Rust commit 前保持 buffered
+ readonly transaction 拒绝写入
+ abort 丢弃 Rust mutation，并恢复 JS index/cursor mirror
+ auto-complete 先 commit Rust，再派发 transaction complete
+ upgrade 中写入的 records 在 schema 成功后以单个 Rust transaction 导入
+ renderer reset JS context 后可读回 Date key、Date value 与 typed array
+ tagged value wire 支持 undefined、特殊 Number、Date、ArrayBuffer、typed view、Blob、Array、Object
+ range delete、count、getAll/getAllKeys 使用同一 Rust transaction view
+ clear 作为独立 mutation，abort 可回滚，commit 不重置 key generator

## 验证

+ page-runtime IndexedDB handler：7 Pass / 0 Fail
+ renderer 跨 document record 恢复：lib + bin 2 Pass / 0 Fail
+ engine 无 host fallback：13 Pass / 0 Fail
+ imported IndexedDB WPT：21 文件 / 166 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass（默认 V8、adapter-only GPU、QuickJS Clippy 与 QuickJS 运行测试）

## 剩余

+ index schema/query/cursor 仍读取 JS mirror
+ cyclic structured-clone graph 尚未进入 value wire
+ 跨 renderer 进程共享与落盘尚未实现
