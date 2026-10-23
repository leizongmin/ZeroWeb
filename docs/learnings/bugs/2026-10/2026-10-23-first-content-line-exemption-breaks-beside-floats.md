---
date: 2026-10-23
modules: layout-engine
---

# 「行首内容不折行」豁免门的隐含假设在 float 缩短带下失效——下绕必须覆盖词/原子盒/断行新行三臂

## 问题描述

floats-132（container 160px、20×20 float、Ahem 词 160px）：float 把首行可用宽缩短到 140 < 160，词应整体下移到 float 带底（第二行满宽恰好容纳）。ZeroWeb 把词强放首行溢出，第二行空、container 红底露出（✗1.33%）。pre-wrap-float-001（✗2.67%）同根。

## 根因

断行 fit 门的 `!current_line.runs.is_empty()` 豁免（行首词不因 fit 折行）内含一个隐含假设：**下一行与本行同宽，行首词折过去也放不下，折了白折**。float 排斥带打破该假设——首行被缩短、下移后的行更宽，行首词放不下**当前带**时下移有意义。而 FloatAnchor 臂在行首空行上不推行、只把 current_x 预置到缩短带左缘，`current_line.runs` 保持空，豁免门恒挡住 fit 条件 → 强放缩短带。

## 解决方案

下绕（shift_new_line_below_floats）按行首内容形态分三臂补齐，语义同源（CSS2 §9.5 rule 8「shortened line box…moved down until it fits」）：

1. R4234：行首**原子盒**放不下缩短带（0 < avail < 盒宽，区别于 avail≤0 的 R3785 推压臂）；
2. R5216：fit 门断行产生的**新空行**上的原子盒（断行只保证离开前行 y 带，新行带可能仍被缩）；
3. R5240：行首**词**——`runs.is_empty() && word_width > avail_width && wrap_allowed` → 下移，`current_x = text_indent.max(left_offset)` 重算（FloatAnchor 预置的是旧带左缘，只调大会保留 stale 值）。

新增臂的检查单：无空行盒 push（不触幽灵行/clamp 预算面）；下移不改 first-line 语义（text-indent 保留）；无重叠带守卫（纯容器溢出不推，旧行为保持）；kill-switch 反证走 corpus OFF 轮（selected 快照路径下进程内 set_var 不可靠，R3858 教训，单测不设 env 相位）。

## 避免方式

给断行/行盒逻辑加「行首豁免」类守卫时，先写明其隐含的可用宽假设；凡存在 float 排斥带（可用宽逐行动态变化）的路径，豁免门都要自问「下一行是否可能更宽」。诊断特征：内容强放在一条被 float 缩短的行上溢出、而其下方有一条明显更宽的空行——先查行首豁免门是否挡了下绕。
