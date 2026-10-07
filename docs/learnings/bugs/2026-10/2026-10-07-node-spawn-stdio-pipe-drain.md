---
date: 2026-10-07
modules: 验收工具链/probe
---

# node spawn 子进程 stdio pipe 无消费者导致浏览器假死（CDP 永不起）

## 问题描述

t8d 诊断中自制探针（`b1-errcap-journey.mjs`、`t8d-legC2-playwright-args.mjs`）spawn zero-browser 时用 `stdio: ['ignore', 'ignore', 'pipe']`——stderr 接了 pipe 但**没有任何 data 监听器**。浏览器被 tracing 日志写满 pipe 缓冲（~64KB）后阻塞在 stderr 写入上，CDP HTTP 服务永远起不来；探针在 `/json/version` 轮询处无限等待，表现为随机 wedge。同型探针三连失败（r9/r9b/r9c），且一度误诊为 profile 父目录缺失（mkdir 修复无效）。

## 根因分析

node `child_process.spawn` 的 pipe 流若无消费者，内核缓冲写满后**子进程**阻塞在该流写入上——不是探针侧问题，是子进程假死。ZeroWeb 启动即打大量 fetch scheduler/tracing INFO 日志（stderr），秒级写满缓冲。原版 b1 探针健康是因为它对 stdout/stderr 都挂了 `.on('data', () => {})` 消费者。

## 解决方案

```js
const browser = spawn(BIN, args, { stdio: ['ignore', 'pipe', 'pipe'] });
browser.stderr.on('data', () => {});   // 必须 drain，否则浏览器写满缓冲假死
browser.stdout.on('data', () => {});
```

不需要日志内容时直接 `stdio: ['ignore', 'ignore', 'ignore']` 更省事；要落盘就 `fs.createWriteStream` 接管。

## 如何避免

- spawn 长跑子进程时，凡用了 `'pipe'` 必须同步挂 drain（data 监听器/目标流），启动后立即检查。
- 探针 wedge 诊断顺序：先 `ps` 看子进程是否存活 + `curl` CDP 端口，再查 stdio 配置——「进程在但端口永不开」优先怀疑 pipe 满阻塞，其次才是启动参数/profile 路径。
