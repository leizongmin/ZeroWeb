# M2 key range and binary keys

**日期**: 2026-08-19

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 IDBKeyRange 构造与 includes、BufferSource conversion、
detached/round-trip binary key、key conversion exceptions 和 object store key order。

## 结果

- 修复前新增切片：55 Pass / 24 Fail / 0 Timeout
- 修复后新增切片：79 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：131 文件 / 795 Pass / 0 Fail / 0 Timeout /
  0 NotRun / 0 empty

## 实现

- KeyRange 构造器统一执行必填参数、key 合法性、边界顺序和空区间校验
- lowerBound/upperBound 的缺失侧按规范设置 open flag
- key conversion 经 typed wire round-trip 生成独立 canonical key
- DataView/TypedArray 转为按 view 范围复制的 ArrayBuffer
- array key getter 异常保持原对象向上传播
- IDBFactory.cmp 首个 key 失败后不再读取第二个 key
- getAll/getAllKeys 不再把 getter 异常错误转换为 DataError

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（131 文件 / 795 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- engine IndexedDB 定向回归：24 Pass
- fetch / runner / ledger 清单：131 / 131 / 131
