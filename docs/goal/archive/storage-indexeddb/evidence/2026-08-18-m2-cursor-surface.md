# M2 cursor surface

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 object store `openKeyCursor`、invalid cursor query、cursor readonly attributes、request/source identity 和 key-only interface shape。

## 结果

+ 修复前：10 Pass / 21 Fail
+ 修复后：31 Pass / 0 Fail
+ 完整 imported 矩阵：67 文件 / 360 Pass / 0 Fail / 0 Timeout / 0 NotRun

## 实现

+ `source`、`direction`、`key`、`primaryKey`、`request` 改为 prototype readonly getters
+ cursor 可变状态保存在内部下划线字段，迭代逻辑继续通过公共 getter 读取
+ `value` 仅定义在 `IDBCursorWithValue` prototype，key-only cursor 不再暴露该属性
+ object store 和 index 增加标准 `Symbol.toStringTag`

## 门禁

+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make testharness-indexeddb`：Pass（67 文件 / 360 Pass）
+ `make test`：Pass
+ fetch / runner / ledger 清单：67 / 67 / 67
