# M2 Rust-backed cursor stepping

**日期**: 2026-08-18

## 结果

Object store 与 index cursor 由 active Rust transaction 持有。open、continue(key) 与 advance(count) 均通过 host wire 推进；JS 只保留 cursor 表面和 request success 事件派发。

## 行为

+ cursor ID 绑定 transaction，commit/abort 后随 transaction registry 回收
+ cursor snapshot 使用 Rust transaction view，可读取 buffered add/put/delete/clear
+ 支持 object store 与 index、key range、next/prev/nextunique/prevunique
+ 支持 value cursor 与 index key-only cursor
+ continue(key) 校验方向并在 Rust 中寻找下一位置
+ advance(count) 在 Rust 中推进，零值和越界输入按 WebIDL 失败
+ got-value flag 阻止同一 success callback 内重复推进
+ 新位置只在下一次异步 success event 发布，调用后同步读取仍保持旧 key/value
+ WPT fetch 脚本支持 `WPT_SOURCE`，且严格校验本地 checkout revision

## 验证

+ page-runtime transaction cursor ownership/stepping 单测：Pass
+ renderer 跨 document object-store advance 与 pending guard E2E：Pass
+ imported IndexedDB WPT：24 文件 / 182 Pass / 0 Fail
+ 新增 driving WPT：3 文件 / 16 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass
+ `make bench-gate`：16 / 16 microbench Pass；绝对页面与 retained form budgets Pass

## 剩余

+ 完整 IDBRequest success/error/readyState/auto-commit 事件模型仍在 JS
+ `continuePrimaryKey()` 尚未接入
+ successful database version 仍受 Rust `u32` 限制
+ 跨 renderer 进程 ownership 与落盘尚未实现
