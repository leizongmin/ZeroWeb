# M2 key semantics

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 keyPath extraction、合法/非法 key、排序、长 keyPath、特殊平台属性、File、Proxy 与 sparse array。

## 结果

- 修复前：103 Pass / 33 Fail / 0 Timeout
- 修复后：136 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：99 文件 / 607 Pass / 0 Fail / 0 Timeout / 0 NotRun / 0 empty

## 实现

- object store/index 共用 keyPath 正规化与 own-property extraction
- 空 keyPath、compound keyPath 和 auto-increment 注入按规范处理
- 缺失路径通过 own data property 注入，避免 prototype accessor 副作用
- File wire 保留 name/type/lastModified
- sparse array wire 逐索引定义 own property，不触发继承 getter
- Proxy、sparse array 和 cyclic array 在 key conversion 边界拒绝
- IDBRequest 补齐 WebIDL class tag

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（99 文件 / 607 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- 定向回归：engine 21、page-runtime 15、runner 7，全 Pass
- fetch / runner / ledger 清单：99 / 99 / 99
