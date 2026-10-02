---
date: 2026-10-02
modules: zero-renderer
---

# 跨通道序缺陷：fire-and-forget 优先派发越过它依赖的普通通道状态设置

## 问题描述

两例同根因：①动态插入元素的 error 事件丢失——`tick_dynamic_scripts` 的快照刷新走普通通道，依赖它的元素事件派发走优先通道，派发可越过未处理的快照 → 元素尚不可见 → 匹配 miss；②`<body onload>` 反射失效——`set_dom_snapshot`（普通通道 fire-and-forget）尚未被 worker 处理时，finish_page_load 的 DCL 生命周期派发（优先通道）已执行：反射读空 body 后按页 URL 去重，整页反射被永久丢弃（r2946 单测确定性复现）。

## 根因分析

双通道 mpsc（prio + normal）没有全局 FIFO。fix#12 把事件/生命周期派发改成优先通道 fire-and-forget 后，「先设置状态、后派发」的调用方时序不再被 worker 保证——worker 若忙于 bootstrap 或旧臂，后到的优先命令先于先到的普通命令执行。生产中 worker 空闲时 recv 即醒、毫秒级处理快照，窗口≈0 掩盖了缺陷；测试在 bootstrap 期间连发两条命令把窗口撑成确定性失败。

修复两例分别用两个惯用法：
- **成对同通道**（fix#19a）：快照刷新改走优先通道，与其消费方（执行+派发）同通道 FIFO——已有 fix#10 同型判例。
- **有界回执等待**（fix#21）：`SetDomSnapshot` 增加可选 `reply`，`set_dom_snapshot` 发送后 `recv_timeout(250ms)` 等应用完成回执（同 `reset_document_state` 惯用法）；优先通道变体保持 fire-and-forget。worker 忙臂超时放行不阻塞导航，此时快照已排普通队列、后续普通往返天然保序。

## 解决方案

- 引入优先通道/抢占语义时，审计所有「A 设置状态 → B 消费状态」跨通道对——B 抢占 A 的通道序即断。
- fire-and-forget 的适用前提是「不依赖任何尚未确认完成的普通通道状态」；有依赖就必须成对同通道或加有界回执。
- 单测把生产中被时序掩盖的窗口撑成确定性失败——这类测试失败是缺陷暴露，不是测试 flaky，先查根因再考虑改测试。
