---
date: 2026-10-29
modules: engine
---

# template content 物化重入 + 栈耗尽吞 RangeError 导致 details 互斥 flake

## 问题描述

`test_details_name_exclusivity_template_deep_entry`（`crates/engine/src/js_dom_bridge_tests/part60.rs`）间歇失败：断言期望 `2:1,0`（树序第一 open 胜出），失败轮实测 `2:0,0`（tp0/tp1 open 属性双被摘）。触发率随进程形态漂移：solo 稳定通过，与任意前驱测试同进程串行时 40%-70% 失败（r4100 对 6/15、无关 t59 对 14/20），加日志插桩后 solo 也 5/10 失败——插桩本身翻转结果。

## 根因分析

三层叠加：

1. **重入物化**：`_r145SelKids`（`js_dom_shim/part04.js`）构建 `_r145Built` 后**先跑 `__zwDetailsScan4100` 再写 `proxy._zwTplContentKids` 缓存**。scan 的片外分支 `_zwDetailsCollect4100(_zwNodeRootOf(el))` 沿 `_tplContent` 的 `childNodes` getter 又回到 `_r145SelKids`——彼时缓存未写，逐层重建 + 嵌套 scan，无收敛条件。
2. **栈耗尽 + 静默吞错**：递归由 V8 栈耗尽（RangeError）终止，被物化/收集各层 try/catch 静默吞掉。失败轮剖面：contentForm≈1910 次重入、scan≈1295 层、closeOpen≈1848 次；**通过轮剖面相同**——递归必然发生，只是哪层「幸存完成物化」不同。
3. **幸存层身份错位**：完成物化的层的重建对象与外层 scan 的 frag 不是同一批对象，`_zwDetailsIsInFrag4100`（按对象身份比对）误判同名 open 成员在片外 → 关闭 tp0 → (0,0)。

**非确定性载体是栈布局**，不是时间/哈希种子/句柄复用：前驱测试与插桩代码改变 Rust 栈深度 → V8 isolate 栈限（isolate 创建时按当前 SP 定）偏移 → 幸存层漂移 → 通过率漂移。这解释了「solo 稳过、配对随机挂、插桩翻结果」三个表象。

排查中先排除了 host 侧（tcjd 每次输出含 open，QDC 单次 PARSE）与 shim 构建段（attrs 工厂直灌确定性）；失败形态「name/id 在、唯 open 被摘」把嫌疑收敛到 R4100 定向 removeAttribute，最终靠 closeOpen 的 `new Error().stack` 定位调用链。

## 解决方案

`js_dom_shim/part04.js` `_r145SelKids`：`_r145Built` 构建后**先置 `proxy._zwTplContentKids` 缓存，再跑 `__zwDetailsScan4100`**。重入读命中同一活数组（对象身份稳定），scan 单次收敛（全绿：solo×20 + t59 对×20 + r4100 对×10 = 50/50）。

## 避免方式

- 物化面「构建 → 自洽性裁决 → 缓存落位」的顺序，凡裁决路径可能经 getter 回读本物化面的，缓存必须先于裁决落位；判据是存在「读 childNodes 会触发重建」的自引用视图。
- 深层 JS 递归调用栈上的 try/catch 会吞 RangeError——间歇失败且插桩能翻转结果时，优先怀疑栈耗尽，用 `new Error().stack` 在可疑写路径（如 removeAttribute）抓调用链，而不是只加状态打点。
- 测试插桩（每次调用过 V8↔Rust 边界的日志桥）会改变栈布局，足以翻转 flake 通过率；对照实验必须用同一插桩版本跑红绿两侧。
