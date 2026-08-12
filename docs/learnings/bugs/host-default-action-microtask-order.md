# Host 默认动作与 Microtask 顺序

- 日期：2026-08-13
- 相关模块：`apps/renderer`、`crates/engine`、`crates/script-sandbox`

## 问题描述

renderer 将 cancelable `reset` 事件派发、UA 默认状态恢复和结果发布拆成多次
JavaScript execute。reset listener 排入的 microtask 在第一次 execute 结束时被
V8 checkpoint 提前执行，因此观察到 reset 前的 live form state。

## 根因分析

浏览器规范要求 listener 与默认动作位于同一 task，microtask checkpoint 在默认动作
完成后执行。V8 embed 则在每次宿主 execute 返回前主动执行 checkpoint，宿主分段事务
破坏了这个边界。

此外，默认动作产生的 DOM mutations 应用到 live document 后，worker 的 DOM snapshot
仍是提交前版本。若 flush 前直接清除 pending mutations，microtask 中的 reflected
property getter 会回退到旧 snapshot。

## 解决方案

宿主默认动作事务暂缓 `queueMicrotask` callback 和 Promise reaction；事件取消判定完成
后先 commit 或 rollback，再把已提交的 HTML snapshot 同步给 worker，最后 flush
microtask 并应用其 mutations。

回归测试必须同时覆盖 text live value、checkbox/radio checkedness、`queueMicrotask`
和 `Promise.then`，并在真实多进程 fixture 中验证最终状态。
