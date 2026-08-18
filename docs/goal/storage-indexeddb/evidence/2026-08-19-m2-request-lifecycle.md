# M2 request lifecycle 80% milestone

**日期**: 2026-08-19

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 11 个上游文件，覆盖 request success/error/upgradeneeded listener 异常、abort
request 顺序、transaction 捕获/冒泡与队列、blocked 生命周期及 upgrade 中 close。

## 结果

- 修复前新增切片：11 Pass / 30 Fail / 0 Timeout
- 修复后新增切片：41 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：168/210 文件（80.00%）/ 1073 Pass / 0 Fail /
  0 Timeout / 0 NotRun / 0 empty

## 实现

- IndexedDB listener 异常统一报告，并继续派发后续 listener
- 未提交 transaction 在 listener 抛错后以 AbortError abort
- committing/finished transaction 不因后续 callback 异常回退
- upgrade listener 异常 abort versionchange transaction 与 open request
- upgrade 中 close 先完成 transaction，再以 AbortError 结束 open request
- close-in-upgrade 保留已提交 version/schema，后续连接可重新打开

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（168 文件 / 1073 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- engine IndexedDB 定向回归：28 Pass
- fetch / runner / ledger 清单：168 / 168 / 168
