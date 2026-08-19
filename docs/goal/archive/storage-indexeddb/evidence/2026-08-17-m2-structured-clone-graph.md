# M2 cyclic structured-clone graph wire

**日期**: 2026-08-17

## 结果

页面写入包含循环引用或共享引用的值时切换到 graph wire。跨 document 读回后保留对象身份，Rust index 通过独立的无环投影提取普通与 compound keyPath。

## 行为

+ 普通树值继续使用既有紧凑 JSON/tagged wire
+ graph wire 先登记节点再编码引用，支持 object、array、Date、Blob、ArrayBuffer 与 typed view
+ decode 先分配全部节点，再连接引用
+ `record.self === record` 与 `record.left === record.right` 跨 document 成立
+ `indexProjection` 对每条路径独立展开，共享对象可重复投影，路径内循环不可索引
+ Rust 保存完整 graph envelope，仅从 `indexProjection` 提取 index key
+ 普通与 compound index 均可查询 graph record

## 验证

+ Rust graph projection 单测：Pass
+ renderer 跨 document graph identity/index E2E：Pass
+ imported IndexedDB WPT：21 文件 / 166 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass
+ `make bench-gate`：16 / 16 microbench Pass；绝对页面与 retained form budgets Pass

## 剩余

+ cursor stepping 与 request event 时序仍在 JS
+ successful database version 仍受 Rust `u32` 限制
+ 跨 renderer 进程 ownership 与落盘尚未实现
