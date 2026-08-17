# M2 per-origin IndexedDB registry

**日期**: 2026-08-17

## 结果

`StorageManager` 现按 origin 和数据库名称持有真实 `IdbDatabase`，为 WebView、browser worker、
renderer worker 共用同一 bridge handler 提供唯一后端。

## 行为

+ 同源同名数据库重开复用 schema 与数据
+ 不同源的同名数据库完全隔离
+ version=0 与版本降级返回错误
+ 版本升级保留 schema 与数据
+ delete、clear_origin、clear_all_indexed_db 按预期清理数据库
+ 数据库名称返回稳定排序

## 验证

`zero-storage` 新增 2 个定向测试，覆盖重开、版本、隔离、删除和清理。

## 下一步

在 zero-engine 定义同步 JSON wire 契约，在三条页面宿主路径注册共享 handler。
