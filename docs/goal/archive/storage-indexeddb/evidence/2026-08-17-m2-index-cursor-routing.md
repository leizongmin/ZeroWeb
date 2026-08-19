# M2 index schema/query 与 cursor entries 路由

**日期**: 2026-08-17

## 结果

Index schema 已进入 Rust database schema。Index get/getKey/count/getAll/getAllKeys 与 object-store/index
cursor 的 entries 均来自 Rust transaction view；JS cursor 仅保留事件派发与 continue stepping。

## 行为

+ index name、keyPath、unique、multiEntry 随 database schema 同步与恢复
+ schema upgrade 前预检 unique index，避免失败后留下部分 version/schema
+ 支持 string、empty string 与 compound keyPath
+ compound keyPath 与 multiEntry 组合返回 `InvalidAccessError`
+ value wire 对普通 JSON 保持原形，Date/特殊 Number/Binary 等继续使用 tagged value
+ transaction index view 包含 buffered add/put/delete/clear
+ index entries 按 `(index key, primary key)` 排序
+ get/getKey/count/getAll/getAllKeys 与 cursor query 使用同一 Rust view
+ next/prev/nextunique/prevunique 由 Rust entries 驱动
+ empty keyPath 按规范提取 record value 本身

## 验证

+ `zero-storage`：671 Pass / 0 Fail
+ page-runtime IndexedDB handler：8 Pass / 0 Fail
+ renderer 跨 document index/compound/cursor 恢复：lib + bin 2 Pass / 0 Fail
+ imported IndexedDB WPT：21 文件 / 166 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass（默认 V8、adapter-only GPU、QuickJS Clippy 与 QuickJS 运行测试）

## 剩余

+ cursor stepping 与 request event 时序仍在 JS
+ cyclic structured-clone graph 尚未进入 value wire
+ 跨 renderer 进程 ownership 与落盘尚未实现
