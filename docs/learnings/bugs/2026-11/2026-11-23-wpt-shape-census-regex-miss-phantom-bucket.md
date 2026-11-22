---
date: 2026-11-23
modules: tests/wpt-runner
---

# WPT 键名形态普查正则漏配 additive/neutral 句式产生伪桶

## 问题描述

KC-5-r6 w4 solo 判读做桶级归因时，首版把 18 条 REG、78 条 F2P-BREAK 归为
「lp→lp 形异常破面」。该簇实际不存在——真值是 Compositing additive 对 24 条
加 neutral 端点行 52 条。伪桶把真簇藏进了错误的形分类，差点导出「LP↔LP 纯增益
面被 keep 臂打落」的错误结论。

## 根因分析

键名解析正则 `from \[(.*?)\] to \[(.*?)\]` 只覆盖 `from [X] to [Y]` 一种句式。
interpolation harness 的键名还有两类变体：

- additive：`from add [X] to add [Y]`（composition 键，`from` 后紧跟 `add`）
- neutral 端点：`from neutral to [Y]`、`from [X] to neutral`

两类句式正则不命中，解析结果落 `('?', '?')`；而形态分类函数的兜底分支把
`'?'` 与真 LP 值共用同一个 `return 'lp'` 出口，沉默地并桶。更隐蔽的是：两个
脚本对 `'?'` 行处理不同（一个兜底计数、一个 `continue` 跳过），数字对不上
才暴露问题——若只跑一个脚本，伪桶会直接入账。

## 解决方案

1. 分类前先按句式分流：`additive`、`from-neutral`/`to-neutral`、`nomatch`
   是独立桶，不得落值形兜底分支。
2. 键名正则扩为 `from (add|) ?\[(.*?)\] to (add|) ?\[(.*?)\]`，neutral 单独
   判串。
3. 兜底桶必须可观测：打印 `?` 桶样本行，禁止静默并入任何值形。
4. 同一键集双口径交叉对账（本例：独立重算脚本 vs 判读脚本计数矛盾）——矛盾
   先查解析层而非数据层。
