# 按字体链分组 hmtx 字符缓存

- 日期：2026-08-18
- 相关模块：`zero-render-foundation`

## 问题描述

hmtx 字符缓存原本以 `(完整字体链, 字号, 字符)` 为平面键。一次文本测量会逐字符查询缓存，因此同一 run 的每个字符都会重新哈希并比较完整字体链。medium 页面已有较高命中率，实际测量成本逐渐由缓存键本身主导。

## 根因分析

字体链和字号在一次 `measure_text_hmtx` 调用内不变，只有字符变化。平面 map 仍为每个字符 clone `Arc` 并对字体链做 hash/equality；字符命中避免了字体表读取，却没有避免复合键成本。

## 解决方案

缓存改为两级结构：外层键为 `(完整字体链, 字号)`，内层 map 为 `char → advance`。每个 run 只查一次外层字体链，逐字符热循环只查询 `char`。总字符条目仍受 4096 上限约束，fallback chain 或 `unicode-range` 改变时同时清空两级缓存。

默认启用分组路径，`ZW_HMTX_GROUPED_CACHE=0` 恢复原平面键实现；`ZW_HMTX_CHAR_CACHE=0` 仍可关闭整个字符缓存。

## 验证

固定 CPU31 的 release Criterion 临时 probe 使用四字体链和重复缓存命中。两组反序结果中，平面键为 `2.164–2.435µs`，分组键为 `0.298–0.335µs`，约快 7 倍。两轮 medium frame-pointer profile 中 `measure_text_hmtx` self 分别从 `1.53%→0.57%`、`1.84%→0.89%`，目标调用栈占比从 `2.23%→0.89%`、`2.12%→1.34%`。

页面短轮 A/B 一组改善、一组轻微反向，profile 轮次又伴随 parse/style/paint 同步漂移，因此不宣称稳定整页百分比收益。关闭/开启路径的 medium PNG SHA-256 完全相同，零容差比较为 `0/480000`。

完整 `make bench-gate` 的16个 crate 基准、页面绝对预算、RSS与 form-input 均通过，报告 `suspect=false`；medium style/layout/paint/total p95 为 `43.39/313.64/127.08/497.23ms`，RSS `157.76MiB`。reftest `687/687`、welcome `16.61%` 与完整 V8/QuickJS 测试矩阵通过。
