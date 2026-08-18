---
date: 2026-08-14
modules: crates/protocol, apps/browser, apps/compositor
---

# Compositor IPC 合法帧超过通用管道上限

## 问题描述

Compositor 正常启动后收到约 20.8 MiB 的 UI 消息，但通用管道只允许 16 MiB。提高上限后，高频 UI 上传和 present 回读又会造成 10 秒响应超时，Browser 随后回退到 legacy frame publishing。

## 根因分析

16 MiB 上限建立时 IPC 只承载较小的控制消息。后续 `CompositorUiFrame` 开始携带完整物理像素位图：3024×1802×4 本身约 20.8 MiB，但协议上限没有随消息模型更新。发送端也没有执行同一上限检查，因此无效帧会先写入管道，再由接收端断开连接。

提高上限后暴露出第二个问题：browser worker 会先发送整个命令 batch，再读取响应。当 browser 写入大型 UI 位图、compositor 同时写回大型 present 位图时，两端可能同时堵在管道写操作上，没人读取对方的数据，最终被看门狗误判为 compositor 无响应。

## 解决方案

将仍然有界的管道消息上限提高到 64 MiB，以容纳当前 UI 位图和单张 4K RGBA 帧；发送端和接收端复用同一校验，避免不对称断连。worker 改为每条命令完成请求—响应后再处理下一条，消除双向大帧同时写入导致的背压死锁。测试锁定实际观测到的 21,797,029 字节合法帧、超过 64 MiB 的拒绝行为，以及命令的 lockstep 交换顺序。

更大的像素载荷不应继续通过抬高管道上限解决；需要支持 8K 或大量图片时，应将图片像素迁移到共享内存，只在 IPC 消息中传递描述符。
