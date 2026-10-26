---
date: 2026-10-26
modules: apps/browser,apps/compositor
---

# compositor 位图守卫严格等值 vs Get「拉最新 front」语义系统性失配——持续发布页 GUI 首帧白屏

## 问题描述

持续发布型页面（页面脚本按定时器持续重绘并提交新帧）在 GUI 壳层窗口中首帧白屏：
loading 永不清除，8 秒后截图仍全白。`tab_snapshot` 单测全绿——位图采纳路径
`commit_compositor_frame`/`commit_compositor_frame_dmabuf` 的首守卫把响应帧按
「过期」拒收，但单测用的静态帧序列掩盖了持续发布页的动态提交节奏。

## 根因分析

守卫要求响应帧与已记录 submission **严格等值**（同 frame_id），而 compositor 侧
Get 语义是「拉最新 front」：持续发布页的 recorded submission 随每条 IPC 前进，
浏览器取到的响应帧恒落后 ≥1 帧。两者是系统性失配而非偶发竞态——每次位图采纳都
按 stale 拒收，采纳计数为 0，loading 状态永不解除。静态帧单测里「响应 == recorded」
天然成立，守卫形同虚设，故 9/9 绿无法暴露。

## 解决方案

匹配条件放宽为「同 surface、同导航代，且 frame_id ≤ 已记录帧」：滞后帧照常采纳，
保留防回退（不采纳更旧代）与跨代拒收子句。旧单测钉住的是 bug 行为
（首帧 `!commit(first)`），改写为三钉：滞后帧采纳、防回退拒收、跨导航代拒收。
修复后持续发布页 GUI 8s 截图全绘（采纳 105 次，此前 0），无 rAF 对照组不退化。

避免方式：凡「消费者持有的游标」与「生产者推进的队列」配对，守卫必须按
**序关系（≤ / 同代）**而非等值写；等值守卫只在两端游标静止的测试里成立。
单测要覆盖 recorded 在断言间隙被推进的滞后臂。
