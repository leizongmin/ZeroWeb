---
date: 2026-10-07
modules: zero-layout-engine
---

# abspos + aspect-ratio 的 auto 轴 = 传递值 + automatic minimum（仅 min-*:auto 时），且 R1743 回填会覆盖后处理结果

## 问题描述

WPT `css/css-sizing/aspect-ratio/abspos-007/008/013/018` 四案长期在册：双 auto 塌 19×18、definite 高只传比不取内容、definite 宽被内容撑爆、max-height 钳高不回传宽。

## 根因分析

三层：

1. **taffy 0.12 对 OOF（abspos/fixed）盒的 AR 语义不完整**——不同形态给出不同错误值（塌 0、纯比值、内容高），无统一规律。
2. **语义误读风险**：abspos-012（min-height:auto）曾让我推断「abspos 无内容高地板」，实际它的地板一直由 R1743 父高回填承担。`min-*:auto ⇔ automatic minimum 生效` 这一 css-sizing-4 §4.1 判据对 OOF 与 in-flow 一致；018 的 `min-height:0` 才是禁用地板的对照案。
3. **后处理时序**：R1743 回填（`shift_siblings_after_ifc_grow` 内）在后处理 pass 之后运行，会把臂内已按传递定值的 auto 高盒重新长回内容高——已有 R3754 豁免（滚动容器）需按同一原则扩展到 OOF+AR。

## 解决方案

- `engine/abspos.rs` 新增 `fix_abspos_aspect_ratio_auto_sizes` 三臂（definite 高→宽 / definite 宽→高 / 双 auto shrink-to-fit+max 钳回传），地板统一 `min-*:auto` 门控；非替换/contain:size/insets 拉伸/fixed/%尺寸全部不触。
- `postprocess.rs` R3754 豁免扩 OOF：`is_absolute || is_fixed` 时回填不得扩展传递定值盒。

## 如何避免

- 推断「X 语义不存在」前，先检查现有绿案是不是被**另一条后处理 pass**（回填/居中/位移）间接承担——本项目布局是 taffy + 十余个后处理 pass 的叠加态，单看最终渲染值无法归因（kill-switch 逐个关是唯一可靠手段：`ZW_IFC_GROW_SHIFT=0` / `ZW_IFC_PARENT_HEIGHT_BACKFILL=0` / `ZW_ABSPOS_AR=0` 三开关定位到回填）。
- 单元测试手建 `ComputedStyle` 必须显式 `display: Block`——没有 UA sheet 兜底，div 默认 display 会让子块 inline 化塌成 18.6px 行高，floor 测量全错。
- 给后处理 pass 加臂时，同时排查 R1743 回填是否会覆盖该臂的写入（凡是「把 auto 高盒长到内容」的 pass 都会）。
