# M2 Date key 与 key wire

**日期**: 2026-08-17

## 结果

Rust `IdbKey` 已新增独立 Date 类型，保持 IndexedDB 的 `Number < Date < String < Binary < Array`
类型秩。`zero-page-runtime` 已定义递归 key JSON wire，为 transaction/store CRUD 接线提供无损边界。

## 行为

+ Date 以 Unix epoch 毫秒存储，NaN/Infinity Date 被拒绝
+ Date 与相同数值的 Number 是不同主键
+ Date 的负零哈希与正零一致
+ Number wire 支持 Infinity、-Infinity 与负零
+ Array wire 递归支持 Number、Date、String、Binary、Array
+ 非法数值 wire 返回 `DataError`

## 验证

+ `zero-storage`：671 unit + 5 cursor + 2 type + 13 coverage，全部通过
+ page-runtime key wire 定向测试：1 Pass / 0 Fail
+ imported IndexedDB WPT：21 文件 / 166 Pass / 0 Fail
+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass（默认 V8、adapter-only GPU、QuickJS Clippy 与 QuickJS 运行测试）

## 下一步

定义 transaction-scoped store CRUD wire，把 add/put/get/delete/clear/count/getAll 从 JS Map
迁移到 Rust。
