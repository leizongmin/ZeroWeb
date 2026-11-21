---
date: 2026-11-21
modules: engine,testing
---

# 双错相消型 P2F：修复 bug 对一侧会揭穿配对侧同源 bug

## 问题描述

clientWidth padding-box 修复（CLIENTWIDTH-PADBOX face）入电池后，
flexbox-min-height-auto-003/004 共 5 断言 P2F，签名 `scrollWidth ==
clientWidth`（"no horizontal overflow"）失败，message
`expected 40 but got 44`。b0（修复前）该页全绿。

## 根因分析

shim 的 `scrollWidth`/`scrollHeight` 与旧 `clientWidth`/`clientHeight`
是**同一处 border-box 近似的两个消费方**（布局 rect 直返 r.w/r.h）。
测试断言 `scrollWidth == clientWidth` 在两值同错（都含 2×2px border＝44）
时恒真——两个 bug 相互抵消，页面在 WPT 上「通过」。face 单侧修正
clientWidth（44→40）后断言变成真差值检测，立即揭穿 scrollWidth 的同源
错误。这不是修复引入的回归，而是测试对配对 bug 的免费检测器被激活。

## 解决方案

修复某个 getter 语义后，同一 A/B 电池里出现的 P2F 不要先按「回归」防御，
先逐案读 message 找**与被修属性同断言配对**的量：若 P2F 断言形为
`X == Y` 且 X/Y 同源于同一近似实现，则 Y 是既有 bug 被揭出，配对修复
（scrollWidth/scrollHeight 同口径 padding-box，floor 0）应入**同一 face**
同电池重验，而非回退或门控原修复。判定顺序：读失败 message 拿
got/expected → 反查两者实现是否共享同一近似/同一布局 rect → 是则
配对同修。WPT 正确引擎（双值皆 spec 值）该类页恒绿，配对修后 5 案
F2P 与主靶增益同电池兑现。
