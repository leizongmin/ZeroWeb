# M2 transaction task deactivation

**日期**: 2026-08-18

## 结果

IDBTransaction active flag 与 task/microtask 生命周期已对齐固定 revision WPT。Request 改为 timer task 派发，auto-commit 在 request queue 清空且 transaction inactive 后执行。

## 行为

+ transaction 创建后及同一 task 的 Promise microtask 内保持 active
+ 下一 timer task 开始前停用所有 active transaction
+ success/error callback 及其 microtask checkpoint 内重新激活事件所属 transaction
+ listener callback 之间停用 callback 内创建的无关 transaction
+ object store 操作在 inactive transaction 上抛 `TransactionInactiveError`
+ request success/error 通过 timer task 派发，keep-alive 不再形成无界 microtask 链
+ auto-commit 等待 pending request 清空，并遵守同 connection 的 scope 冲突 completion 顺序
+ readonly transaction 之间和不相交 scope 不互相阻塞
+ 完成或 abort 后从 task registry 与 connection scheduling registry 移除
+ WPT runner 每次 probe 只执行一个 due timer，保留真实 task boundary
+ renderer 跨 document E2E 通过有界轮询等待 TimerBridge callback

## 验证

+ 新增 fixed-revision driving WPT：3 文件 / 11 Pass / 0 Fail
+ imported IndexedDB WPT：35 文件 / 204 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass
+ `make bench-gate`：16 / 16 microbench Pass；绝对页面与 retained form budgets Pass

## 剩余

+ 跨 connection / 跨 renderer transaction scheduling 尚未统一
+ `continuePrimaryKey()` 尚未接入
+ successful database version 仍受 Rust `u32` 限制
+ per-origin 落盘与跨会话恢复尚未实现
