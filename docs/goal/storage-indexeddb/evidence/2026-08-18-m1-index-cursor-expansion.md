# M1 index and cursor WPT expansion

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 5 个上游文件：

+ `idbindex_getAll.any.js`
+ `idbindex_getAllKeys.any.js`
+ `idbobjectstore_openCursor.any.js`
+ `idbindex_openCursor.any.js`
+ `idbindex_openKeyCursor.any.js`

覆盖 index getAll/getAllKeys、multiEntry、range/count、无效 query key、object store 100 条 cursor 迭代，以及 index cursor lifecycle/exception ordering。

## 结果

+ 修复前：43 Pass / 2 Fail
+ 修复后：45 Pass / 0 Fail
+ 完整 imported 矩阵：52 文件 / 277 Pass / 0 Fail
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make test`：Pass，含 V8、QuickJS、GPU adapter 与真实多进程矩阵
+ `make bench-gate`：16 / 16 microbench；页面绝对预算与 retained-form budget Pass

两个失败均来自 native `MessageChannel` transfer 后的 detached TypedArray。旧 key 校验只识别 shim `_detached` 标记，构造 binary key view 时泄漏 V8 `TypeError`。统一 binary-key 提取边界现在捕获 native detached buffer，并按 IndexedDB 规范映射为 `DataError`。
