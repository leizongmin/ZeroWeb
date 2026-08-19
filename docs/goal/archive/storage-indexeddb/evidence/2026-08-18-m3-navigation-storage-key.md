# M3 browser navigation storage-key authority

**日期**: 2026-08-18

## 结果

Browser 不再从可提前变化的 `TabSnapshot.url` 推导 IndexedDB origin。Renderer 通过 append-only `NavigationStarted` / `NavigationCommitted` IPC 报告导航生命周期，Browser 使用 renderer ID、navigation epoch 和 URL 维护 pending/committed storage key。

## 行为

+ 导航开始立即撤销旧 committed origin 和对应 transaction/cursor registry
+ 导航提交仅在 renderer ID、navigation epoch 和 URL 与 pending navigation 完全匹配时生效
+ 导航提交前的 IndexedDB 请求返回 `SecurityError`，不会写入旧 origin 或待导航 origin
+ Mismatched/stale commit 不启用 IndexedDB
+ Browser fetch proxy 把主文档最终 response URL 传给 renderer，redirect 后以最终 URL 作为 document base URL 和 committed storage key
+ Renderer 退出、替换或 tab 关闭时同步清理 pending/committed origin

## 验证

+ Committed origin 与 UI snapshot URL 解耦：Pass
+ Redirect requested origin → final origin：Pass
+ Mismatched commit 不写入任一 origin：Pass
+ Navigation start 撤销旧 origin 和 transaction registry：Pass
+ Protocol start/commit bincode round-trip：Pass
+ `make test`：Pass，含 V8、QuickJS、GPU adapter-only 和真实多进程测试
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ 固定 IndexedDB WPT：38 文件 / 222 Pass / 0 Fail
+ `make bench-gate`：16 / 16 microbench Pass；页面绝对预算与 retained form budget Pass

## 测试隔离修复

新增 authority 单测暴露了 `ProcessTabBackend::Drop` 与真实多进程测试竞争全局 compositor 的既有问题。所有构造该 backend 的单测现与 GUI 多进程测试共用 `MULTIPROCESS_TEST_LOCK`；无 instrumentation 的最终全矩阵已通过。

## 剩余

+ 完整跨 connection transaction scheduling 与 blocked/versionchange 事件
+ Successful database version 超过 Rust `u32`
+ 扩大固定 revision 上游 IndexedDB WPT 导入范围
