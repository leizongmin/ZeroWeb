---
date: 2026-11-09
modules: engine
---

# 样式 carry 刀的捕获三形分账与壳级缩放口径

## 问题描述

F6b 刀（sub/sup×fontsize）需要把选区 carry 的字号在 typing/回放时恢复。14 个语料键的现场分四种失败形：A 族桶漂移、N 族 post_insert 不触发、C 族等值门吞掉重放、W/D4 尾选区空转——单一捕获口径无法同时满足，硬套会产生双包（壳内外各包一层 font）。

## 根因分析

caret 上行捕获（`__zw_ce_capture_script_carry`，`crates/engine/src/js_dom_shim/part01f.js:1311`）面临三种形态，其 px 语义不同：① inScript 且上行先遇壳内 `font[size]`——该 font 自身档位才是语义值，walk_px 级联会被壳缩放污染；② 其余 inScript——walk_px 直走（已含 0.83 缩放），值须重映射回档位；③ armed-only——walk_px 桶加 `inner: true` 旗标，消费端要知道"这个桶应该裹在内层"。而 `getComputedStyle` 对 UA 样式元素（sub/sup）盲读缺省值，16px 与 medium 在等值门里不可区分，forceValue 的"值相同即跳过"门（`crates/engine/src/js_dom_shim/part01d.js:886`）因此吞掉合法重放。

## 解决方案

- 捕获分账三形各自返回 `{bucket}` 或 `{bucket, inner}`，消费端按旗标路由：inner 走 typing 内层臂（text 节点先裹 font 再包 sub/sup），否则走 post_insert 外层臂（剥中间层 font、按桶包壳外）。
- 等值门只对**壳级**（node 本身是 sub/sup 元素）按 0.83em 缩放后比较；段文本级保持盲读——壳外裹上 font 后 text 节点已见壳值，内层自然溶解，天然防双包。
- 教训：样式 carry 刀的捕获与消费必须按现场形态分账并显式旗标路由，单一全局口径必败；等值/等价门遇到 UA 样式元素盲读时，缩放校正放最窄层级（壳级），放宽到所有节点会双包。
