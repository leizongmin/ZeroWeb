# M2 engine wire 与共享 storage handler

**日期**: 2026-08-17

## 结果

`zero-engine` 已定义同步 `__zw_idb(requestJson)` bridge，origin 只从宿主持有的当前页面 URL
推导。`zero-page-runtime` 已提供共享 `StorageManager` handler，首批支持 factory 与 schema 操作。

## 行为

+ wire 请求上限为 8 MiB
+ 页面 request JSON 无法覆盖可信 origin
+ opaque origin 返回 `SecurityError`
+ 支持 open、deleteDatabase、databases
+ 支持 createObjectStore、deleteObjectStore、storeNames
+ 同源重开保留 schema，异源同名数据库隔离
+ 版本降级返回 `VersionError`
+ 非法请求与缺失数据库/store 返回具名错误

## 验证

+ `zero-storage` storage manager 定向测试：17 Pass / 0 Fail
+ `zero-engine` IndexedDB bridge 定向测试：3 Pass / 0 Fail
+ `zero-page-runtime` IndexedDB handler 定向测试：3 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass（默认 V8、adapter-only GPU、QuickJS Clippy 与 QuickJS 运行测试）

## 下一步

在 WebView、browser worker、renderer worker 注册共享 bridge，再把 `part02.js` 的 factory/schema
操作路由到 Rust backend。
