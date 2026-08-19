# M2 cursor continuePrimaryKey

**日期**: 2026-08-18

## 结果

Index cursor `continuePrimaryKey(key, primaryKey)` 已通过 transaction host wire 在 Rust cursor snapshot 中按 `(index key, primary key)` 推进。同步异常顺序、next/prev 方向、unique cursor 拒绝和 cursor class identity 已对齐固定 revision WPT。

## 行为

+ Rust transaction cursor 新增 pair-target step，next/prev 均按复合位置严格前进
+ object-store cursor 与 nextunique/prevunique cursor 拒绝 `continuePrimaryKey()`
+ transaction、deleted source、source type、direction、iteration state、key conversion 和方向校验按规范顺序执行
+ `prevunique` 先选每个 index key 的最小 primary key，再按 index key 逆序
+ `IDBCursorWithValue` 真实继承 `IDBCursor`；key-only cursor 保持 `IDBCursor`
+ cursor 新位置继续在下一次异步 success event 发布
+ 三份 driving WPT 固定在 revision `315976933870b34d6ea30e3f6643403edae678ba`

## 验证

+ page-runtime pair stepping、反向 stepping、source/direction guard 单测：Pass
+ 新增 fixed-revision driving WPT：3 文件 / 18 Pass / 0 Fail
+ imported IndexedDB WPT：38 文件 / 222 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass，含 V8、QuickJS 和 GPU adapter-only 测试
+ `make bench-gate`：16 / 16 microbench Pass；页面绝对预算与 retained form budget Pass

## 剩余

+ 跨 connection / 跨 renderer transaction scheduling 尚未统一
+ successful database version 仍受 Rust `u32` 限制
+ per-origin 落盘与跨会话恢复尚未实现
