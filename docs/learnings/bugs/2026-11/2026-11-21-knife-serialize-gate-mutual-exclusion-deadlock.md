---
date: 2026-11-21
modules: build,testing
---

# knife 电池序列化门互排除设计的互指死锁

## 问题描述

2026-11-21 晚同一 clone 上三个电池队列链两两互等、全程挂起：`zwv3-compile-wait`（yield 循环等 `zw-knife-*` unit 静默）与 `knife-g1a-relaunch`（busy 集显式含 `zw-v3wait-*`）互指；新发射的电池 unit 又与队列中已有 unit 互指（math↔V3-r2，22:25），一方被外部 stop 后另一方（V3-r2）放行。整个晚间窗口出现三连死锁形态，其中一次需要人工 stop 才解。

## 根因分析

各电池脚本的序列化门采用**互排除**语义：每门用 `systemctl list-units 'zw-knife-*' | grep -qv '<自家字面>'` 检测他流 unit，只排除自家字面、不排除对方。该设计在「双方都在门循环中挂起」时无解——A 等 B 退出、B 等 A 退出，谁都不是在跑电池（都在 `SERIALIZE-WAIT` 挂起），却互相视为 busy。触发条件只是**第二个挂起 unit 的出现**：新 unit 发射时队列里已有挂起 unit（它在等更早的链），两者立即互指。门循环无超时、无队位序号、无全局账本，死锁只能靠外部 stop 或 launch-timeout 退出。

同形第二因子：发射器（waiter/relaunch）把「等待中的 unit」也算 busy。等待 unit 与在跑 unit 在 `list-units` 的 running 态无法区分（systemd 不暴露脚本内部状态），导致排队链的长度被放大成永久 busy。

## 解决方案

1. **即时解环**：后到方撤让——stop 后发射的 unit，让先手先跑（cwcv/H 与 math/V3-r2 两次同款处置）；若互等双方分属不同使命且一方使命已被取代（v3wait 的 ZeroWeb-2 发射被「移 ZeroWeb-3 代发」声明取代），停僵尸链。
2. **结构性规避**（后续电池门候选设计）：
   - 全局队位锁：发射前 `flock` 一把全局队列锁排队，锁内写 FIFO 队列文件登记自家 unit 名与状态（waiting/running/done），门只等**队首之后**的 running 态——挂起等待不占 running 语义位；
   - 或发射器在发射前把等待声明写入 `.acceptance` 共享账本（类似 NOTES 仲裁），门检测账本而非 systemd 状态。
3. **发射前普查**：发射挂起型 unit 前查全量活跃 unit（含 waiter/relaunch 形态），若已有挂起 unit 排队，直接串到队尾挂起（同款门）或延迟发射，不制造第二个互指点。

## 经验点

- systemd unit running 态不区分「在跑」与「挂起等待」，以 unit 状态为 busy 判据的门在多链排队时必然互指；busy 判据必须能区分这两态。
- 互排除门每加一条新字面排除只是延迟死锁（新增 unit 即新互指点），不是解决。
- 挂起型发射（先发射后等门）比先等后发射更容易制造死锁：挂起 unit 本身成为他流门的 busy 信号。
