# M2 cursor stepping expansion

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 object store/index cursor 的 `continue()`、`advance()`、overload、异常优先级、iteration mutation，以及 compound object-store key path。

## 结果

- 修复前：31 Pass / 8 Fail / 0 Timeout
- 修复后：39 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：83 文件 / 420 Pass / 0 Fail / 0 Timeout / 0 NotRun / 0 empty

## 实现

- `continue(undefined)` 按省略 key 参数处理
- `advance()` 先执行 unsigned long 转换，再检查 transaction/cursor 状态
- cursor stepping 按规范顺序检查 transaction active、deleted source 与 got-value flag
- object-store compound key path 贯通 JS shim、page-runtime wire、zero-storage schema 与持久化
- 旧字符串 object-store key path 持久化格式保持向后兼容

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（83 文件 / 420 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- 定向回归：storage 678+20、page-runtime 15、engine 19、runner 7，全 Pass
- fetch / runner / ledger 清单：83 / 83 / 83
