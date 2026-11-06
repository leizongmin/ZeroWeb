---
date: 2026-11-06
modules: zero-engine
---

# DBG 插桩读点必须对齐 harness 断言时点——exec 内时点读数不能反驳终态断言失败

## 问题描述

sr1 轮（选区恢复耦合刀）为调查 iol `:393`（`<ul><li>foo<ul><li>[bar]<li>baz</ul><li>quz</ul>` 的 qState-after 断言失败）做了 DBG 插桩：在 exec 实跑窗口内读 queryCommandState 得 `true` 且 innerHTML 与语料期望逐字合，据此把该行判为「切片跨窗伪影」除名出结构债。但后续 br 刀、rtl 刀两代全腿批中该行均确定性复红（expected true / got false，compare innerHTML 同案恒过）。伪影定性错误，除名被推翻。

## 根因分析

插桩读点在 `_zwListToggle` 调用栈内部（选区恢复臂完成之前），而 harness 的 after 断言读在命令完全退出之后。两者之间存在出口收钳、壳重建、fallback `(host,len)` 落点等后处理臂——sr1 本轮正在排查的嫌疑面恰是这些臂。exec 内时点读数只证明「该时点状态正确」，不证明「终态断言应通过」；用前置时点读数反驳终态断言失败，等于用嫌疑面执行前的状态为嫌疑面辩护，循环论证。失败行「伪影除名」类结论因此被一个时点错位的读数误导。

## 解决方案

- DBG 插桩的读点必须与被审断言的读点严格对齐：harness after 断言 = 命令退出后的引擎终态，插桩须放在命令调用栈完全退出之后，或直接 dump 终态 range 端点（对照语料期望选区标记位）。
- 对「失败行除名 / 伪影定性」结论，要求终态证据；exec 时点读数、中途树快照均不构成除名证据。
- 判活经验同型：确定性复红（跨两代基线恒在 fail 行集）本身就是「非伪影」的强信号——伪影族（签名 A straddle / 签名 B leak-cut）依赖窗口边界位置，逐代恒现的失败优先按真实缺陷重启归因。
