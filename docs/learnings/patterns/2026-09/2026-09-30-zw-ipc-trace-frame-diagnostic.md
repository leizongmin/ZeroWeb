---
date: 2026-09-30
modules: zero-protocol,apps/browser,apps/renderer
---

# ZW_IPC_TRACE：跨进程 IPC 消息级的帧级诊断

## 问题描述

html5test 页面脚本被 15s 看门狗整体终止（t5/P7），所有微任务回调丢失。console、
CDP exception 事件都只见"Execution timeout"，看不到是哪一个 IPC 消息有去无回：
多进程（browser 主进程 ↔ renderer）之间的请求/应答散落在 mux 线程、router 线程、
JS worker 线程，常规日志无法把"请求发出"与"应答未达"对上号。

## 根因分析

需要一种**消息级追踪**手段：按消息 id 串起发送方与接收方，才能证明
"IndexedDbRequest（id=2^63，kind 0x34）发出后无任何 IndexedDbResponse"。
`zero-protocol` 的 `IpcMessage` 有 8 字节 id + 4 字节 kind 的固定帧头，天然可作
追踪主键，但默认无落盘路径。

## 解决方案

编译期环境开关 `ZW_IPC_TRACE=<path>` 启用帧日志（零默认开销，不进产品路径）。
帧格式（一行一帧）：

```text
<W|R> <pid> tid=<thread> len=<hex: 8B id(LE) + 4B kind(LE) + params>
```

- 前缀 `W`/`R` 区分写入方向（发送）与读取方向（接收）；同一消息的 W/R 行 id 相同。
- kind 为小端 u32：`0x34 = IndexedDbRequest`、`0x35 = IndexedDbResponse`（判别值见
  `crates/protocol/src/message.rs`，append-only 纪律）。
- 诊断手法：对目标 id 做 `grep`，若只有 `W` 行没有配对的 `R`/应答行 → 接收方
  catch-all 静默丢弃（本次 P7 的直接证据）；配合时间戳差值可量化阻塞时长。

经验证可用于定位：消息被 catch-all 吞（t5）、看门狗终止前最后的消息序列
（"Execution timeout: 15000ms" 帧）、以及修复后的快速应答对照。同类问题
（导航/存储/Service Worker 代理的 IPC 链路悬案）可先开此追踪再插桩。

## 如何避免

多进程消息"有去无回"类 bug，优先用帧级追踪对 id，而不是在收发两端各加
`tracing::debug!`（后者改动面大且需要重编译两端）；catch-all 臂新增消息类型时，
同步确认该类型在所有消费路径都有显式臂或显式错误应答。
