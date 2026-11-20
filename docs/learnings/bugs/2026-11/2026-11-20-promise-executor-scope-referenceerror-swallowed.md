---
date: 2026-11-20
modules: zero-engine
---

# Promise executor 外的 settle helper 引用 executor 参数——ReferenceError 被 catch 静默吞，promise 永不决

## 问题

WAAPI shim（`crates/engine/src/js_dom_shim/part04.js`）`Animation.finished` 自决块三代同 bug（r2→r4 face 迭代复制传播未察）：`_settle` helper 定义在 `new Promise(function (_rf) { ... })` executor **外**，函数体内 `_rf(anim)` 引用的是 executor 参数——不在 `_settle` 词法作用域。调用即抛 ReferenceError，被外层 `catch (_eRf6) {}` 静默吞掉，promise 永不 resolve。

表象离根因极远：WPT 页 `await animations[0].finished` 永挂 → 页 30s 墙钟截断 → 整页 Timeout（报告显示「1 test(s) unreported」）。排查先被年龄判据「到点即触发」的事实引向墙钟/步数假设，两轮误诊后才定位到作用域。

## 根因

JS 词法作用域：executor 参数只在 executor 函数体及其嵌套函数内可见，外部 helper 无法捕获。`node --check` 与全部静态语法检查通过——作用域错误不在其覆盖面。宽 `catch {}` 把每次调用的异常全部吞掉，无日志无断言，缺陷完全静默；三代 face 复制粘贴同块，bug 随之传播。

## 解决

`_settle` 移入 executor 体内，`_rf` 直接闭包可见；`anim._resolveFinish(anim)`（part03.js Animation 构造器 16268 行的 onfinish 驱动）保留。solo 探针验证：trans-dga b1 双侧零 Timeout、失败集与 b0 逐项同形；修复随 r6 face 落库（61d17aa54）。

## 避免方式

- shim 中 promise 的 settle/resolve helper 一律定义在 executor 体内；确需抽出时把 resolve 作为参数显式传入。
- 宽 catch 只用于「失败可容忍」路径；settle 路径的 catch 至少留一条 `tracing`，开发期可临时 rethrow 验证路径真实可达。
- 「语法检查通过」不等于「路径执行到」：promise settle 类缺陷必须以探针页真实执行验证，静态检查发现不了作用域错误。
