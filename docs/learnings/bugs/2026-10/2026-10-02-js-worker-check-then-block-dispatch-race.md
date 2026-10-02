---
date: 2026-10-02
modules: zero-renderer
---

# JS worker 分派环 check-then-block 竞态：优先派发可被普通命令越过

## 问题描述

renderer JS worker 分派环在阻塞于 `cmd_rx.recv_timeout()` **之前**检查优先通道（prio_rx）。单测探针断言「提交顺序即执行顺序」（DCL → img → window load 后探针应读到完整生命周期状态），四例齐失败；`+300ms` settle 后通过曾误导为「纯测试时序问题」。

## 根因分析

分派环每轮先 `prio_rx.try_recv()`，空则 `cmd_rx.recv_timeout(50ms)` 阻塞。竞态窗：阻塞期间优先命令到达 → worker 仍睡在 recv 上，而普通命令（探针/页面后续脚本）在同一毫秒到达 → worker 醒来先取到普通命令执行，优先命令反而延后。fix#12 把生命周期派发改为 fire-and-forget 后，提交与执行解耦，该窗口从「理论」变为「必现」——普通命令读到的是生命周期派发前的旧状态。

修复：recv 醒来后再查一次 prio；查到则把刚取到的普通命令搁入 `held_normal` 槽，下一轮循环 prio 清空后补跑。`ResetDocumentState` 清队臂同步清 `held_normal`（搁置命令同属旧文档队列）。修复后测试无需 settle 即过——回归钉 `normal_probe_cannot_leapfrog_priority_lifecycle_dispatch` 常驻。

## 解决方案

- 任何「先查 A 通道再阻塞等 B 通道」的循环，醒来后必须重查 A——check-then-block 是竞态模板，不是一次性问题。
- 单测 settle/加延时掩盖的「时序问题」要先问：产品路径同样的提交序列会怎样？settle 通过 ≠ 产品正确。
- fire-and-forget 提交使顺序契约从「调用方可见」退化为「分派器保证」——分派器是实现顺序契约的唯一地点，测试应直接钉分派器行为。
