---
date: 2026-09-13
modules: tests/playwright-matrix
---

# Node 父进程内嵌服务 + execFileSync 子进程 = 双向死锁

## 问题描述

cdp-protocol S17 的 M5 复核脚本（ZeroWeb 侧 CDP 捕获）：父进程 spawn headless 浏览器 +
`startCaptureProxy`（HTTP/WS 代理跑在父进程），再用 `execFileSync('node', ['capture-core-flow.mjs'])`
子进程经代理驱动全核心流——子进程 `connectOverCDP` 恒 30s 超时，代理流量记录恒 0 行；
同款代理在父进程内直连却完全正常。

## 根因分析

`execFileSync` 阻塞的是**父进程的 libuv 事件循环**。捕获代理作为父进程内的 HTTP server，
其请求处理全靠事件循环驱动——同步等待子进程退出期间，父进程无法 accept/read/respond。
于是子进程的发现请求（`GET /json/version`）永远得不到服务，子进程挂起等待；
而父进程又在同步等子进程退出——双向死锁。超时杀死子进程后，父进程事件循环恢复，
但代理里没有留下任何已服务请求的痕迹（流量恒空）。

判定特征：子进程日志停在网络请求发出前、代理侧零记录、父进程内直连同端点正常——
三者同时出现即此模式。

## 解决方案

父子进程间存在「父进程内嵌服务 ← 子进程消费」关系时，必须用**异步 `spawn` + 事件回调**
等待退出（`child.on('exit')` 包成 Promise），禁止 `execFileSync`/`spawnSync`：

```js
const { code } = await new Promise((resolve) => {
  const child = spawn('node', [script], { stdio: ['ignore', 'pipe', 'pipe'] })
  child.on('exit', (code) => resolve({ code }))
})
```

已落地：`tests/playwright-matrix/scripts/probe-s17-capture.mjs`（本地调试脚本，不入 git）。
教训通用化：同步执行原语（exec*Sync）在持有任何异步服务的宿主进程里都是全局停顿，
服务面跨进程暴露时即为死锁。
