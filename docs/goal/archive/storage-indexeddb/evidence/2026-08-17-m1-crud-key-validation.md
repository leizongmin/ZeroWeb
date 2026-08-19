# M1 CRUD key 校验与生成

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| Pass | 32 / 54 | 45 / 54 | +13 |
| Fail | 22 | 9 | -13 |
| 通过率 | 59.26% | 83.33% | +24.07pp |

add/put/get 的 11 个 key/DataError 用例与 2 个非法 index key request 用例全部转绿。

## 修复

+ inline store 传入显式 key 时同步抛 DataError
+ out-of-line store 缺 key、inline key 缺失或 key 非法时同步抛 DataError
+ get 使用非法 key 时同步抛 DataError
+ autoIncrement 维护独立 key generator，并向结构化克隆值注入 inline key
+ 普通 get/delete 使用 IndexedDB key 比较，支持 Date 等值键
+ 真实 request 暴露为 IDBRequest/IDBOpenDBRequest 构造器

## 下一步

实现 cursor continuation，消除 6 个 autoIncrement/cursor 失败。
