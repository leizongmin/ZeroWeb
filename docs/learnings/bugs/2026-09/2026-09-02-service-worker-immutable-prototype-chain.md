---
date: 2026-09-02
modules: script-sandbox, service-worker, wpt-runner
---

# Service Worker Global Prototype Chain Must Reject Prototype Mutation

## 问题描述

`service-workers/service-worker/immutable-prototype-serviceworker.https.html` 会把
`MessagePort` 传给 Service Worker，worker 内遍历 `self` 的 prototype chain 并对每一层调用
`Object.setPrototypeOf(object, {})`。ZeroWeb 的 Service Worker global/prototype chain 原来由
普通 JavaScript 对象拼出，前四层可以被重新设置原型，WPT 因此收到
`mutable, mutable, mutable, mutable, immutable` 而失败。

## 根因分析

WebIDL 的 global object 与相关 prototype object 属于 immutable prototype exotic object：
改变这些对象的 `[[Prototype]]` 必须失败。单纯用 `Object.create()` 和
`Object.setPrototypeOf(globalThis, ServiceWorkerGlobalScope.prototype)` 拼出继承链，只能建立
`instanceof` 关系，不能自动得到 immutable prototype 语义。

不能直接 `Object.preventExtensions(globalThis)` 或冻结整个 global：Service Worker 脚本仍需要
声明全局变量、安装事件处理器和更新普通全局属性。冻结会把 prototype 保护扩大成错误的全局写入
限制。

## 解决方案

在 Service Worker bootstrap 完成 prototype chain 建立后，只记录需要保护的对象：
`globalThis`、`ServiceWorkerGlobalScope.prototype`、`WorkerGlobalScope.prototype`、上层
global 原型和 `Object.prototype`。随后包裹 `Object.setPrototypeOf()` 与
`Reflect.setPrototypeOf()`：

- 目标对象在保护集合中，且新 prototype 不等于当前 prototype：`Object.setPrototypeOf()` 抛
  `TypeError`，`Reflect.setPrototypeOf()` 返回 `false`。
- 新 prototype 等于当前 prototype 时保持幂等成功。
- 普通对象仍走原生实现，避免影响页面/worker 脚本自己的对象模型。

后续新增 worker/global 类 shim 时，不要用冻结对象替代 immutable prototype；应按 mutation API
边界最小拦截，并用 WPT 或单测同时覆盖受保护对象和普通对象。
