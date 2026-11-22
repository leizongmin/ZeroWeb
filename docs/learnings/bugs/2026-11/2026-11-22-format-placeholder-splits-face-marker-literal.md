---
date: 2026-11-22
modules: engine, wpt-runner
---

# format! 占位符在二进制里切分面标记——字面量 FM 门假阴性

## 问题描述

刀 FAM-F 电池的 face 标记门（FM gate）原设计用
`strings b1-runner | grep '<face 字面量>'` 校验 face 烙入，恒 0 命中，形似
「电池自拒（face 未烙入）」。而 M2（b0/b1 bin md5 异）已独立证明 face 确实烙入
b1——单门假阴性险些弃跑整块电池。

## 根因分析

源码 `format!("{prefix}{} / 1", …)` 的格式串在编译期被拆成 pieces：占位符之间的
字面量各自独立存放，`"{} / 1"` 在二进制里不存在连续字节串（只有 piece ` / 1`），
strings + 字面 grep 永远找不到。与
[二进制签名串折行切分](../2026-11-09-binary-sig-string-line-wrap-split.md)
同族不同因：那是运行时折行切分，这是编译期 piece 切分；共同前提「面标记＝二进制
可检出的连续字节串」在这两种机制下都不成立。

## 解决方案

FM 门设计两条腿：

1. 优先**符号门**——face 新增函数名（如 `interpolate_aspect_ratio`）在 b1
   符号表可检出、b0 为零（实测 2/0 稳定），不依赖字面量连续性。
2. 若必须字面量门，先看源码 format! 占位符切分再拼 grep 串（逐 piece），
   或选不经 format! 的常量作标记。

偏差发生时先做三角验证（M2 md5 异＋符号门＋face 在树 grep）再定
「自拒 vs 毒化」，不因单门假阴性弃跑；偏差记入 NOTES（同 M2 字面量门→MD5 门
先例）。
