# M2 store metadata

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 create/deleteObjectStore、keyPath identity、DOMStringList、transaction scope、closed connection、异常优先级和 upgrade task 时序。

## 结果

- 修复前：8 Pass / 43 Fail / 0 Timeout
- 修复后：51 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：91 文件 / 471 Pass / 0 Fail / 0 Timeout / 0 NotRun / 0 empty

## 实现

- objectStoreNames/indexNames 实现排序、索引、contains/item 与动态 upgrade scope
- create/deleteObjectStore 补齐 versionchange guard、keyPath 校验和规范异常顺序
- transaction 补齐 scope 去重排序、closed/mode/empty scope 同步校验
- upgrade completion 从同一 microtask 移到后续 task
- 关闭连接冻结 schema 名称视图
- host 名称字段按需使用可逆 UTF-16 code-unit wire，支持 lone surrogate

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（91 文件 / 471 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- 定向回归：engine 20、page-runtime 15、runner 7，全 Pass
- fetch / runner / ledger 清单：91 / 91 / 91
- 首轮 `make test` 的 loopback revalidation 瞬态失败单测复验 3/3 Pass，第二轮完整矩阵 Pass
