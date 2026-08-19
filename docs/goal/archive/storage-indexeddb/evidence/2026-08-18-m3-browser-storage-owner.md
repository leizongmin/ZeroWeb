# M3 browser-owned IndexedDB

**日期**: 2026-08-18

## 结果

生产 renderer 不再创建自己的 IndexedDB `StorageManager`。页面同步 host callback 通过 request ID IPC 调用 browser 主进程中的唯一 storage owner；普通 tabs 使用平台数据目录中的持久 owner，private tabs 使用独立的进程内 owner。

## 行为

+ `IndexedDbRequest` / `IndexedDbResponse` 追加到 IPC enum 末尾，保持既有 bincode discriminant
+ Renderer JS worker 使用共享 stdout writer 发送请求，独立 stdin router 按 request ID 解除 waiter
+ Router 不依赖 renderer 主循环，页面脚本执行期间不会形成主线程自等待
+ Browser 从 tab URL 推导 origin，IPC request 不接受 renderer 提供的 origin 字段
+ Browser 对请求再次执行 8 MiB 上限校验
+ 所有普通 renderer 共享同一个持久 `StorageManager`
+ 每个 renderer 使用独立 transaction registry；renderer 退出时 registry 随 handler 回收
+ Private tabs 共享独立内存 owner，不读取或写入普通持久数据库
+ `ZERO_STORAGE_DIR` 覆盖产品数据目录；`ZERO_PRIVATE=1` 禁用 IndexedDB 磁盘写入
+ 持久 owner 初始化失败时普通 tab 的 IndexedDB 请求返回具名 `UnknownError`，不静默降级为空数据库；private owner 仍可用

## 验证

+ Protocol request/response serialization round-trip：Pass
+ Renderer waiter/router：matching response、non-IDB passthrough Pass
+ 真实多进程 smoke：两个普通 renderer 同 origin 写入/读回 Pass
+ 真实多进程 smoke：private renderer 同 origin 隔离 Pass
+ 真实多进程 smoke：普通 renderer 导航到另一 origin 后隔离 Pass
+ 持久 owner 初始化失败时 private owner 可用：Pass
+ 固定 IndexedDB WPT：38 文件 / 222 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass，含 V8、QuickJS、GPU adapter-only 和真实多进程测试
+ `make bench-gate`：16 / 16 microbench Pass；页面绝对预算与 retained form budget Pass

## 剩余

+ Embedded WebView 的宿主级 persistent/private owner 注入
+ 完整跨 connection transaction scheduling 与 blocked/versionchange 事件
+ Browser 导航提交的独立 storage-key authority
+ Successful database version 超过 Rust `u32`
