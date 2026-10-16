---
date: 2026-10-16
modules: layout-engine, wpt-runner
---

# 自源 reftest diff 幅度跳变：先补 oracle-shot 对照再立根因域

## 问题

R5147 行度量整数化使 `multicol-span-all-fieldset-001/002/003` 从 PASS 0.35-0.49% 变
FAIL 1.51-1.59%（越 1% 阈），自然归因写成了「spanner 平衡决策对行高敏感」并挂账
域核。若按该假设开工，会去调 spanner 平衡/预算逻辑——真因根本不在那里。

## 根因

corpus reftest 是 **self-source**（test 与 ref 两页都由 ZW 渲染），diff 幅度只度量
「ZW 两页自洽度」，不度量「ZW vs chromium」。幅度跳变有两种成因，像素数字分不清：

1. 真新缺陷（ZW 相对 chromium 变差）；
2. 既有缺陷的两页**相位差被放大**（两页都错但错得一样 → 阈内；错相 → 阈外）。

本轮是 2：两页都把 legend 画进列流（都错、同相 → 0.35% 阈内）；整数化后 fieldset
路径与 div 路径的行高相位分开（同错不同相 → 1.5%+ 越阈）。真缺陷其实是
「fieldset 自身 multicol 容器时 legend 不该入列流」（chromium oracle-shot 一眼定谳），
与 R5147 无因果。

## 解决方案

对「幅度跳变型」新红立案前走三步，前两步都是分钟级：

1. **kill-switch/flag-off 复核**：旧行为下两页是否也失配（0px 同 → 是相位放大，
   失配早已存在；否 → 真回归）。
2. **oracle-shot 对照**：`./scripts/run-oracle-capture.sh --category <dir> --filter <name>`
   抓 chromium 渲染，与 ZW 逐像素比——直接看正确模型长什么样，别从 diff 数字反推。
3. 只有 oracle 证实「结构模型错」才立根因域；幅度数字本身不立案。

判据：self-source 数字回答「自洽吗」，oracle-shot 回答「对吗」；跳变归因必须用后者。
