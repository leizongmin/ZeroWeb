# R5335：block-in-inline 切片边框语义（CSS2 §9.2.1.1）

日期：2026-10-30
基线：R5334 交付态 799F / 15791 pass（95.2%）@ 0d4430775
驱动案：`css/CSS2/normal-flow/block-in-inline-insert-009.xht`（账本最轻非排除域 1.08%）
同域家族：insert-010（1.02%）、box-display/block-in-inline-001/002（2.62%/2.49%）、relpos-002（2.52%）、floats/become-float（2.08%）

## 现象（A/B 像素 + layout-dump + 临时诊断单测定谳）

009 结构：`body > span(blue 3px border) > span#target(cyan 3px border) > [span One/Two/Three + div×2 + span Six + div×2]`（onload 追加 Nine）。

ZW 渲染 vs nosplit-ref 三类偏差：

1. **outer blue 全边连续竖带**：outer（间接含块级后代的 inline）未被 R109 拆分（`inline_has_block_child` 只认直接块子），paint 按 box-level border-box 画全边 → blue/cyan left 竖带贯穿全部块区域（ref 语义：left 仅首 slice、right 仅末 slice、上下带每 slice）。
2. **切片内容 +3/±6px 水平偏移**：切片内容（R109 匿名块片段 + 块级子）挂在 split 壳盒子树内，几何继承壳 frame 推进；规范语义切片在块层排布（div 左缘 = body content 缘）。
3. **R109 片段边选择缺陷**：旧二态清法（first→清 right、last→清 left）把「首末合一的单片段」错清 right、「middle 片段」（首末皆非）错留全边——nosplit-ref 的 notend/notstart notend/notstart 三类 slice 构造为判定基准。

根因链（三级定谳：单案 PNG dump + 纯 zlib 手解像素轮廓 → layout-dump 树形 → layout-engine 临时诊断单测加 R109 标志位打印）：

- 诊断 dump 盒身份：outer (6v1) split=false、inner 壳 (7v1) split=true/frag=false（装饰已抑制 ✓）、anon 片段 (7v1) frag=true（边选择承载者 ✓）——**唯一主错源是 outer 未拆分的 box-level 绘制**+ 几何偏移。
- `inline_finalization.rs:1269` 空 inline 单 fragment 臂 `child.x = agg.first_x` 直接取 fragment x（已含 border-left 流内折入推进，R5248 口径），而同文件 1098 臂有 `fold_landed` 扣除——outer 盒 x +3 即此双计。
- inner 壳 x=0 ✓、div x=6 ✓ 在修复前已对齐；anon 片段 x=6 应为 3（壳 frame 双重推进）。

## 修复（5 处，布局引擎 3 + paint 侧 2）

1. `inline_finalization.rs:1269`：空 inline 单 fragment 臂补 `fold_landed` 扣除（`(agg.first_ml - raw_ml).max(0.0)`，与 any_text 臂同式）——outer 盒 x 对齐。
2. `r109.rs` 边选择三态：first∧last（单片段）全边保留；first 清 right；last 清 left；middle 双清。
3. `tree.rs:4035`：单 Inline 段也登记 `last_inline_fragments`（旧 `len > 1` 条件使单片段被当 first 清 right）。
4. `r109.rs` 新 pass `unshift_r109_slice_content_frames`：剥 split 壳 frame 对切片内容的推进——anon 片段 x 剥壳 frame_left（保留祖先 inline 链推进，nosplit-ref slice1 双左带错开 3px 实证）；块级子剥全部 inline 链推进（块级子包含块 = 外层块容器）；块容器子树累计清零；abs/fixed 不平移。
5. paint 侧 `painter/border.rs` 新增 `paint_block_in_inline_slice_borders` + `collect_block_in_inline_slices`、`painter/mod.rs` 门接线：inline 盒子树含匿名块片段盒（自身非 split 宿主）时抑制 box-level border，改按 slice 描绘（left 仅首 slice / right 仅末 slice / 上下横带每 slice，外侧锚定 slice 盒区间；横带与内层片段盒横带同位，内层后画覆盖）。

## 单案结果（make reftest-upstream FILTER=block-in-inline-insert）

- insert 家族 **37 过 1 挂**：009 ✓ (0.98%)、001a–008c/011–017 全过（多案 0.00%）
- 010 ✗ (1.11%)：010 = 009 + `:after { content: "Ten" }`；剩余差异主因 = y 维 -2.2px 行盒 strut 偏移（slice 带整体上移，属行盒 va/strut 域，R4383 谱系）——切片带语义正确但 y 锚定随 anon 盒偏移，挂账行盒域另案。
- 009 diff 区 y=2..163 已从「连续竖带 + 全局偏移」收敛到容差内。

## 全量 corpus A/B（基线 799F @ 0d4430775）

- 结果 **15792 / 16590 = 95.2%**，798F（净 -1）
- **FIXED=1**：block-in-inline-insert-009 ✓
- **REGRESSED=0**
- 家族挂案 diff% 前后对比（前 = R5334 账本 / 后 = 本轮）：001 2.62→2.61、002 2.49→2.49、relpos-002 2.52→2.52、become-float 2.08→2.08、insert-010 1.19→1.11——全部持平或改善，无恶化。
- 家族剩余挂案共性 = y 维行盒 strut 偏移（-2.2px 量级）+ 各案独立语义缺口（relpos-002 相对定位域、become-float float 化域），留待行盒域专项。

## 回归面与风险控制

- 修复 2/3/4 只在有 R109 split 壳 + 非 frame 推进时改变几何：壳无 border/padding 时平移量 0（零影响）；block-mixed 宿主（R3893/R4338 满宽语义）不进平移。
- 修复 1 只在「空 inline（零宽 TextRun）单 fragment + first_ml > raw_ml」时改变 x——无 border/margin 的空 inline fold=0 零影响。
- paint 门只对「display:inline + 子树含片段盒 + 非 split 宿主」触发；R4332 多行文本域（`has_direct_paintable_text`）不重叠（结构壳无直接文本子）。
- bg 的切片化未实现（本域案例均无背景，挂账）；复杂 border-style 逐 slice 复刻挂账（FIXME 标注）。

## 待用户决策 / 挂账

- y 维 -2.2px 行盒 strut 偏移（010 剩余主因 + 009/010 ref 对比横带错位源）：行盒 va/strut 写位域（R4383 谱系），涉全 inline 页面行位，需专项轮次。
- outer background 的切片化、复杂 border-style（dashed/double）逐 slice 复刻。
- 家族案 001/002/relpos-002/become-float 本轮前后对比见上方全量 A/B 段。
