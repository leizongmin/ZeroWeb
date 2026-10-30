---
date: 2026-10-30
modules: zero-wpt-runner
---

# corpus 脚本分片+续跑组合：resume 跳过必须消费 idx 槽位

## 问题

`run-editing-testharness-corpus.sh`（`RESUME=1` + `EDITING_CORPUS_SHARD=k/N`
组合）续跑补案时，已完成案被 `continue` 跳过但不递增 `idx`，后续案的
`idx % shard_n` 归属整体左移。实测：shard 2/3 名册 14 案完成 13 案后续跑
最后 1 案，脚本把 shard 0/1 的案（caret-navigation-around-line-break、
fontname、hilitecolor、inserthtml 等）误判为本分片成员逐案跑入 transcript
（8627 行错账，幸好错块后缀可剔、分片 JSON 未及写出）。

## 根因

分片成员资格由「名册序位」决定（`idx % shard_n`），不是由案名决定。任何
按运行时状态（transcript 已完成集）跳过名册条目的分支，若不消费序位
计数器，等于把后续案的序位前移——分片边界从「确定性名册划分」漂移成
「依赖完成态的划分」。确定性跳过（如病态案名快筛）不影响跨 run 一致性，
运行时状态驱动的跳过才是坑。

## 解决方案

resume 跳过分支补 `idx=$((idx + 1))` 后再 `continue`（editing 脚本
2026-10-30 已修）。判断法则：**循环内凡按非确定性条件 `continue` 的分支，
都必须与正常路径一样推进序位/迭代计数器**。同构脚本
`run-css-testharness-corpus.sh` 的 resume 分支存在同款缺陷（属
css-testharness-compat 域，由该域自行修）；其 ONLY_SUBSTR 定向过滤分支
同样不消费 idx——定向单案模式通常不与分片并用，风险低但同法则适用。

## 避免方式

给「roster 划分 + 状态续跑」组合写测试：构造 3/9 名册、完成集含中间元素，
断言续跑只补缺口案且不触碰他分片案；或改用「先过滤名册再分片」的两段式
（filter-then-shard），让续跑集合在进入分片循环前就已确定。
