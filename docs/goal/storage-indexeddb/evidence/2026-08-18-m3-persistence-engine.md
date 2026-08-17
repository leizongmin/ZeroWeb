# M3 IndexedDB persistence engine

**日期**: 2026-08-18

## 结果

`zero-storage` 已具备显式单 owner 的 IndexedDB 持久化 manager。Schema、records、index 定义和 auto-increment generator 可原子落盘并在重建 `StorageManager` 后恢复；page-runtime handler 已验证写入、销毁 owner、重建和读回链路。

本阶段未把独立 renderer 直接指向磁盘目录。生产 browser 仍需新增同步 request/response IPC，由 browser 主进程持有唯一 manager，避免多个 renderer 的旧快照相互覆盖。

## 行为

+ Origin 与 database name 以 SHA-256 路径组件隔离，文件内同时保存并校验原始 identity
+ Number、Date、String、Binary、Array key 无损编码，包含 Infinity 与负零
+ Schema 保存 object store、keyPath、index compound/multiEntry/unique 定义
+ Record value 保存完整 structured-clone graph wire
+ 写入使用同目录临时文件、file sync、atomic rename 和 directory sync
+ Windows replacement 使用 backup，启动时恢复中断的 backup 并清理 orphan temp
+ Transaction 在数据库候选副本提交，持久化成功后才替换 live state
+ Schema mutation 与 deleteDatabase 传播持久化 I/O 错误
+ 损坏文件返回 `StorageError::Serialization`，不静默创建空数据库
+ page-runtime 将磁盘错误映射为 `UnknownError`，失败 transaction 不污染 live database

## 验证

+ storage round-trip：schema、records、compound/multiEntry index、typed key、generator、origin isolation Pass
+ crash recovery：backup restore 与 orphan temp cleanup Pass
+ corruption：invalid JSON 拒绝 Pass
+ I/O failure：candidate write 失败且 live database 不变 Pass
+ page-runtime restart E2E：commit → drop manager → rebuild → get/index query Pass
+ page-runtime disk failure：commit 返回 `UnknownError` 且 record 不可见 Pass
+ imported IndexedDB WPT：38 文件 / 222 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass，含 V8、QuickJS 和 GPU adapter-only 测试
+ `make bench-gate`：16 / 16 microbench Pass；页面绝对预算与 retained form budget Pass

## 剩余

+ browser 主进程唯一 storage owner 与 renderer 同步 request/response IPC
+ 跨 connection / 跨 renderer transaction scheduling
+ 默认产品数据目录与隐私模式策略
+ successful database version 超过 Rust `u32`
