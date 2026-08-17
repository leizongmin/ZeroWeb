# M2 request core event model

**日期**: 2026-08-18

## 结果

IDBRequest 与 IDBTransaction 的核心事件模型已对齐上游 WPT。请求状态、事件传播、错误取消、abort request queue 和 transaction 完成顺序均通过固定 revision 用例。

## 行为

+ pending request 读取 `result` 或 `error` 抛 `InvalidStateError`
+ IDBDatabase、IDBTransaction、IDBRequest 使用统一 EventTarget listener registry
+ request 事件按 database capture、transaction capture、request target、transaction/database bubble 路径传播
+ error event 的 `preventDefault()` 阻止默认 transaction abort
+ 未取消的 request error 以原始 DOMException 设置 `transaction.error` 并触发 abort
+ explicit abort 将所有 pending request 按队列顺序转换为 cancelable/bubbling `AbortError`
+ transaction abort/complete 后 `objectStore()` 与 `abort()` 执行 finished-state guard
+ IDBDatabase、IDBObjectStore、IDBTransaction 与 IDBIndex 绑定真实构造器，`instanceof` 使用实际原型
+ auto-commit 仅在 request queue 为空后提交，success callback 可追加请求

## 验证

+ 新增 fixed-revision driving WPT：8 文件 / 10 Pass / 0 Fail
+ imported IndexedDB WPT：32 文件 / 192 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass
+ `make bench-gate`：16 / 16 microbench Pass；绝对页面与 retained form budgets Pass

## 剩余

+ transaction active flag 与 task/microtask deactivation 时序尚未实现
+ `continuePrimaryKey()` 尚未接入
+ successful database version 仍受 Rust `u32` 限制
+ 跨 renderer 进程 ownership 与落盘尚未实现
