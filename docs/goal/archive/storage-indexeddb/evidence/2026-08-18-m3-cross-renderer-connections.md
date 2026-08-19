# M3 browser-owned cross-renderer connections

**日期**: 2026-08-18

## 结果

Browser owner 现统一维护 regular/private partition 内的 IndexedDB connection registry。Registry key 为 browser 可信的 renderer ID 与 realm-local connection ID；origin 来自 navigation commit，database/version 由 browser storage 校验。

版本升级和删除流程：

1. requester 向 browser owner 申请 connection change
2. browser 向同 partition/origin/database 的 connection renderer 发送 `versionchange`
3. renderer 在 JS worker 串行执行事件后回 ack
4. 全部 ack 后 requester 才观察到 `blocked`
5. connection `close()` 回报 browser；全部关闭后 requester 进入 upgrade/delete

同 scope 的 connection change 按 browser owner FIFO 串行执行。排队请求成为队首时重新读取 browser storage version，并按此 fresh `oldVersion` 选择当前 connection targets 和派发 `versionchange`。

导航、renderer teardown 和 regular/private partition 切换都会撤销该 renderer 的 connection 与 pending request。

## 验证

+ Browser owner 状态机：5 / 5 Pass，覆盖 scope FIFO 与队首 fresh `oldVersion`
+ Protocol event/ack bincode roundtrip：Pass
+ Renderer JS worker / runtime：194 / 194 Pass
+ Page-runtime embedded fallback：44 / 44 Pass
+ 真实双 renderer upgrade：`1>2 → blocked → close → success:2`
+ 真实双 renderer delete：`1>null → blocked → close → success:1`
+ 固定 IndexedDB WPT 保持：47 文件 / 232 Pass / 0 Fail
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass，含 V8、QuickJS、GPU adapter 与真实多进程矩阵
+ `make bench-gate`：16 / 16 microbench；页面绝对预算与 retained-form budget Pass

## 剩余

+ 将 transaction scheduler 提升到 browser owner，覆盖跨 renderer scope 冲突
+ 将 deferred execution 扩展到全部 object store、index 和 cursor operation
+ 继续扩大固定 revision IndexedDB WPT 覆盖
