# M2 get-all options

**日期**: 2026-08-19

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 10 个上游文件，覆盖 object store/index 的 getAll/getAllKeys options
dictionary、count [EnforceRange]、direction/unique 语义与 getAllRecords/IDBRecord。

## 结果

- 修复前新增切片：6 Pass / 158 Fail / 0 Timeout
- 修复后新增切片：164 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：157 文件 / 1032 Pass / 0 Fail / 0 Timeout /
  0 NotRun / 0 empty

## 实现

- getAll/getAllKeys 区分 legacy query/count 与 IDBGetAllOptions overload
- count 使用 WebIDL [EnforceRange] unsigned long 转换
- options dictionary 支持 query/count/direction，count 0 按无上限处理
- store/index 共用 direction、unique、count 后处理顺序
- getAllRecords 返回带只读 key/primaryKey/value 的 IDBRecord
- host 路径先获取完整有序结果，再应用反向、去重和数量限制

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（157 文件 / 1032 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- engine IndexedDB 定向回归：26 Pass
- fetch / runner / ledger 清单：157 / 157 / 157
