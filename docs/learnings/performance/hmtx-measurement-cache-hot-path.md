# hmtx 文本测量缓存热路径

- 日期：2026-08-17
- 相关模块：`zero-render-foundation`、`zero-layout-engine`、`zero-engine`

## 问题描述

布局改用真实 hmtx advance 后，medium 页面 layout 从约 88ms 恶化到 384-850ms。已有实现具备 run 级结果缓存和 face 缓存，但在 8,000 个文本节点的页面上仍有大量重复字体解析接线成本。

## 根因分析

face 缓存查找前会为每个字符重新扫描字体前 4KiB 计算 FNV hash，并 clone 字体字节 `Arc`、借用 thread-local cache。布局内部多个阶段还会重复收集全文档 font overrides；taffy 每次文本测量回调又忽略已收集的 node-to-font 映射，重新 clone/hash `font-family` 并执行 face matching。

run 级缓存只能命中完全相同的 `(字体链, 字号, 文本)`，不能抵消这些逐字符和逐回调固定成本。

## 解决方案

face cache 改用进程唯一 `font_instance_id`，整段文本只借用一次 thread-local cache并批量读取 glyph advance。一次完整布局只收集一份 font overrides，后续 taffy、table、float、multicol 和 final IFC 阶段共享同一 `Rc`。TextRun 与匿名 flex/grid 文本优先借用预计算的 node-to-font slice，仅在未注入 overrides 的测试路径回退旧 resolver。

性能开关在 `ShapedAdvanceSource` 构造时读取一次，避免逐字符调用 `getenv`。

## 验证

同机定向 A/B 的 medium layout p95 从约 546ms 降至约 411ms；完整性能门下为 504ms，对比巡检最坏 850ms 下降约 41%。`make bench-gate` 的 16 个微基准、绝对页面预算、RSS 与 form-input 均通过；相对门因 Xeon 与 i5 基线硬件不匹配仅告警。reftest 687/687，welcome product smoke 16.61%，换行基准与 hmtx/shaping 数值测试均通过。
