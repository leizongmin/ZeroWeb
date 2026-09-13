---
date: 2026-09-13
modules: apps/browser,tests/playwright-matrix
---

# Playwright session.send() 对桥 miss 不 throw——CDP 自动化错误走 exceptionDetails 而非协议错误

## 问题描述

cdp-protocol goal S25 补测（`runtime.releaseObjectGroup` e2e 步）时：先经
`Runtime.evaluate`（带 `objectGroup`）建立对象句柄，再 `Runtime.releaseObjectGroup`
释放整组，随后断言「被释放句柄不可再用」。按 JSON-RPC 客户端习惯，预期失效句柄的后续
`Runtime.callFunctionOn` 会**抛异常或返回带 `error` 字段的协议错误**——实际
`session.send()` 既不 throw，响应里也没有 `error` 字段，断言初版无从下手。

## 根因分析

ZeroWeb headless 把 **automation 层失败**（objectId 桥 miss、句柄失效等）译为 CDP
`Runtime` 域的 `exceptionDetails`——即 **200 形响应**
（`{result: {...}, exceptionDetails: {...}}`），而非 JSON-RPC 协议错误
（`{error: {code, message}}`）。这不是 bug，而是协议形状的自然延伸：`Runtime.evaluate`/
`callFunctionOn` 对**页面内脚本异常**本就返回 exceptionDetails（V8 异常面，CDP 规范形状），
句柄失效与页面异常同面处理；协议错误码保留给传输/参数/未实现命令层
（-32700/-32601/-32602，见 goal DC-3）。

Playwright 的 `CDPSession.send()` 只把**协议错误**译为 rejection；200 形响应原样 resolve
返回，所以异常自然不会发生——「不 throw」是客户端对这种响应形状的正确行为。

## 解决方案

断言「自动化语义失效」（句柄失效、脚本异常）时，**兼容双形状**：exceptionDetails 或
协议错误任一出现即算失效（账本 `runtime.releaseObjectGroup` 步实测形态）：

```js
let invalidated = false
try {
  const r = await session.send('Runtime.callFunctionOn', { objectId: stale, functionDeclaration: 'function() { return 1 }' })
  invalidated = !!r?.exceptionDetails
} catch {
  invalidated = true // 协议错误路径
}
```

ZeroWeb 侧语义分界（扩展 CDP 域时沿用）：**协议错误码** = 传输/参数/未实现命令；
**exceptionDetails** = 页面/V8/句柄桥层失败。raw-CDP 测试按同一分界写断言。
