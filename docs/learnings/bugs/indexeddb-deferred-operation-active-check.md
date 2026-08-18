# IndexedDB deferred operation active check

**日期**: 2026-08-18

**相关模块**: `crates/engine/src/js_dom_shim/part02.js`

## 问题描述

Transaction scope scheduler 将 index 查询延迟到前序冲突 transaction 完成后执行时，object store 操作可以正常完成，但 index 查询触发 `TransactionInactiveError` 并导致后续请求 abort。

## 根因分析

API 调用阶段已经按规范检查 transaction active flag。等待期间 transaction 会在 task 边界对页面脚本变为 inactive，但获批后执行已接受的内部 operation 不应再次执行脚本可见 active 检查。`_zwIDBIndex._entries()` 同时承担 API 校验和内部取数，deferred operation 调用它时重复执行 `_assertUsable()`，把合法的已排队请求误判为新请求。

## 解决方案

公共 index API 在调用时同步完成 lifecycle、active flag 和 query 校验；deferred closure 只执行已经接受的内部取数和 request event 派发。回归测试必须覆盖冲突 transaction 下的 object store、index 和 cursor-open 操作，并断言所有 success 晚于前序 transaction complete。
