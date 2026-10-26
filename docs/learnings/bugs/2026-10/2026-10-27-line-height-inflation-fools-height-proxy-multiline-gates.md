---
date: 2026-10-27
modules: engine, layout-engine
---

# 「高度 > 1.5×fs」式多行代理判据会被 line-height 膨胀欺骗

## 问题描述

paint 层多处用「盒高是否远超字号」来猜测 inline 盒是否跨行（box-level 背景抑制、border 门、R639 fragment 门、selection 行高亮站），判据形如 `content_height > 1.5×fs`。`content-height-001`（CSS2 §10.6.2/§10.8.1）暴露：单行 span 的 `line-height: 200px`（fs 50px）把盒高膨胀到 200 > 1.5×50，被误判为多行——box-level 背景被抑制、per-fragment 路径代 firing，背景落旧 `frag.y` 位（= baseline−line_height）上溢半 leading 78.6px 盖住邻块，✗2.50%。

行盒高随 line-height（含继承、strut）增长而行数不变，是这类判据的系统性盲区，不是孤例：R4332 已在 border 门修过同形缺口（bidi-003 判例），本轮（R5296）补齐两处 bg 门与 fragment 门时 selection 行高亮站（`inline_heights > 1.5×fs`）仍留同形判据。

## 根因分析

用几何代理（盒高）近似结构判据（行数）时，阈值只对照了「字号尺度」，漏了「单行可膨胀到多高」的下界。单行盒高的真下界不是 `fs`，而是：

- 自身 computed line-height 解析值（`normal`≈1.2×fs / `<number>`×fs / `<length>`）——继承链生效，inline 自身 style 即可取到；
- 所在 run 的行高最大值（IFC owner 盒 `text_node_line_heights` 的 max）——strut 与兄弟行高的代表。

注意流内 inline 元素自身的 `text_node_line_heights` 映射为空：`store_font_sizes_from_ifc` 把行高键记在 IFC owner 盒上，inline 本身只有 computed style 可查。R639 `render_fragment!` 宏内拿到的 `box_node` 是 IFC owner 而非 inline 自己，两者不可混用。

## 解决方案

判据升级为 `content_height > 1.5×max(fs, 自身 line-height 解析值, run 行高最大值)`（R5296，kill-switch `ZW_INLINE_BG_ML_GATE=0` 回退）。要点：

1. **同一不变量的多个门必须成对改**：box-level 抑制 ⇔ fragment 路径生效要恒耦合（R638 双绘教训），三处共用同一阈值输入与同一 kill-switch。
2. **只收窄多行判定，不动另一轴**：R5171 的扩张轴（line-height < A+D）有 active-selection-027 A/B 校准在案，不随本轮扩张。
3. **独立校准域单独切片**：selection 站同形判据有自身 A/B 校准（R5052/R5054/R5270），不搭车修改。

新增门测锁两头：单行膨胀 span 门开时背景收缩进树盒（内容区锚定）、门关时复现上溢；真跨行 span 仍逐行发射背景（抑制不变量保持）。
