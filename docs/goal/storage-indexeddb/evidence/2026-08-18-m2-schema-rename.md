# M2 schema rename

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 object store/index rename、异常顺序、abort 回滚、
name scope 和 DOMStringList 排序。

## 结果

- 修复前新增切片：7 Pass / 34 Fail / 0 Timeout
- 修复后新增切片：41 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：115 文件 / 697 Pass / 0 Fail / 0 Timeout /
  0 NotRun / 0 empty

## 实现

- object store/index `name` setter 执行 DOMString 转换与 versionchange 状态校验
- rename 原子更新 schema map、transaction scope、DOMStringList 和实例缓存
- 同一 transaction 按旧名或新名查询时保持规范要求的对象身份
- schema sync 以 rename 后名称持久化，同时保留 records、indexes 和 key generator
- versionchange abort 在返回前恢复 schema 快照和既有 wrapper metadata
- 本次 upgrade 新建对象 abort 后保持最后名称，但从 schema/list 中移除

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（115 文件 / 697 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- engine IndexedDB 定向回归：23 Pass
- fetch / runner / ledger 清单：115 / 115 / 115
