# M3 browser-owned cross-renderer transactions

**日期**: 2026-08-18

## 结果

Browser owner 现按 regular/private partition、可信 navigation-commit origin、database、object store scope 和 transaction mode 统一调度 transaction。冲突 transaction 先获得 pending request，轮询获批 lease 后才能调用 Rust `begin_transaction`；commit、abort、navigation、renderer teardown 和 partition 切换都会释放 lease。

Browser 路径强制 `begin_transaction` 携带 owner lease，页面直接调用 `__zw_idb` 不能绕过 scheduler。Embedded WebView capability 保持关闭，继续使用原同步 host 路径。

等待 transaction 启动期间，object store 的 add/put/get/delete/clear/count/getAll、index 查询以及 store/index cursor open 均延迟执行。Cursor continue/continuePrimaryKey/advance 只能在首次 cursor success 后调用，因此发生在 transaction 已启动之后。

## 验证

+ Browser transaction owner 状态机：5 / 5 Pass
+ Engine IndexedDB 回归：16 / 16 Pass
+ Page-runtime IndexedDB host：13 / 13 Pass
+ 真实双 renderer：writer 持有 readwrite lease 时 reader 保持 pending；writer commit 后 reader 读取 `new`
+ 固定 IndexedDB WPT：47 文件 / 232 Pass / 0 Fail
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass，含 V8、QuickJS、GPU adapter 与真实多进程矩阵
+ `make bench-gate`：16 / 16 microbench；页面绝对预算与 retained-form budget Pass

## 剩余

+ 继续扩大固定 revision IndexedDB WPT 覆盖
