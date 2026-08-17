# M2 三宿主 IndexedDB bridge 注册

**日期**: 2026-08-17

## 结果

WebView 进程内 sandbox、browser Tab JS worker、renderer JS worker 均已注册
`IndexedDbBridge`，并共用 `zero-page-runtime::indexed_db_handler` 实现。

## 行为

+ WebView 在 sandbox 创建时注册一次 bridge，导航只更新可信 `page_url`
+ WebView 生命周期内复用同一 `StorageManager`
+ browser 与 renderer worker 在持久 sandbox 初始化时注册 bridge
+ 三条路径均从宿主 URL 推导 origin，不接受页面自报 origin
+ 当前 storage 生命周期限于单个 WebView/worker；跨进程与跨会话共享留待 M3

## 验证

+ WebView QuickJS callback 可达性：1 Pass / 0 Fail
+ browser worker QuickJS callback 可达性：lib + bin 2 Pass / 0 Fail
+ renderer worker QuickJS callback 可达性：lib + bin 2 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass（默认 V8、adapter-only GPU、QuickJS Clippy 与 QuickJS 运行测试）

## 下一步

修改 `part02.js`，把 factory 与 schema 操作路由到 `__zw_idb`，保留尚未迁移的 CRUD、
transaction、index 与 cursor JS 实现。
