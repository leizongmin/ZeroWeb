# R5460 证据：viv 混合内容容器块轴 extent 补偿——R5459 挂账勘误（html shrink 零可见）＋驱动案 vrl −1.86pp / vlr −1.75pp＋corpus 双窗 0 新增红净 +3 绿

日期：2026-11-08。基线 HEAD：`61f68333c`（R5459）。驱动案：`float-lft-orthog-htb-in-vrl-002`＋同构 vlr-002。

## 1. 归因：R5459 挂账勘误＋chromium oracle 定谳

R5459 证据文档第 4 节称「vrl-002 残余 3.25% 主体：html shrink-to-fit 含 float 溢出」，并把「html shrink 修一则 .vp 高即回 497 带」列为最短路径。本轮几何分解证伪其可见性判断：

- **html/body 透明零可见像素**：html w=32/junk h 与 body fill 784 的偏差不产生任何可见差异（html/body 无边框背景；.vp 右缘锚定自调整，位置全不变）。html shrink 修则 reftest **零改善**。
- **002 残差真主体**（oracle + PNG 取证）：(b) test 臂 .vp 块轴 extent 塌宽 w=16 frame-only（左橙条 776..784 vs ref 736..744，~1.9pp）＋(a) ref 臂 abs 宽 89 vs 211.48（taffy absolute 交换帧 sizing，R5455 家族，~1.0pp）。

chromium oracle 五＋五变体定谳 root/容器 sizing 规则（headless=new --window-size=800,600 --dump-dom，document.title 零流内污染输出）：

| 规则 | oracle 证据 |
|---|---|
| vertical root 行内轴 = fill ICB 高（F 空 body 亦 513，内容无关；513＝chrome 壳裁剪后真视窗高） | A2/F/G/H/I 全 513 |
| vertical root 块轴 = shrink-to-fit 含 float（A2 227.48＝float 211.48+margin 16；显式宽 D 200+16） | A2/D |
| 非脱流容器 extent＝流内内容含自身文本列，不含 float（B 纯文本 56=40+16；C float-only 16 纯 border） | B/C |
| body 块轴＝.vp margin box（56）；混合内容容器（文本+float）extent = max(own-IFC 列, Σ 流内子) | B |

ZW 对应缺口（探针矩阵）：html h=junk 链（270.4 溢出链/262.4/8，kill-switch 三连不变＝base taffy 产物）、body h=784（ICB 宽 stretch）、.vp w=16（自身列 40 不贡献）。V1 纯文本 56 / V4 块子域 taffy 语义已对。

## 2. 修复：`viv_mixed_content_block_extent`（ZW_VIV_MIXED_EXTENT default-on）

- 位置：vertical_block_flow.rs 新 walk + engine.rs 管线 **11.11**（R5459 fill 11.10 后、compute_final_inline_layouts 前）——列断深度 = content_height 依赖 fill 后可信（fill 前塌 40 时逐词断列病理，R4445 gate 前提）。
- gate：父/自身 vertical + display Block/ListItem + height Auto + **物理宽 auto**（物理属性不换轴存储，style.width=物理宽；逻辑 block-size vertical 域亦落 style.width，apply.rs:184）+ 子树含 float **或** abs/fixed 子（两形态 taffy 交换帧均不给自身列宽贡献；reftest 双臂自洽 DC-14 要求驱动案 test 臂 float / ref 臂 abs 同步域）+ 非 float/abs/fixed/multicol + 无流内非 float 元素子 + **非同 node_id 片段盒**（R109 inline-split 片段与宿主同 id、float 嵌片段内，fire 其上经 R4460 泄漏修正级联改写宿主、破坏 R5457 混排 gate 契约——r5457 gate_excludes_float_with_inflow_sibling 翻红实证后补门）。
- action：真实 IFC 测量（R4463 同源：with_vertical + with_vertical_rtl + central baselines + configure_inline_fonts，float 子由 collect_items 发 FloatAnchor 无 strut 不入列宽）→ block_extent = total_height()；width = extent + frame（min/max-width 钳制，5.5 口径），content_width = extent，**only-grow**；vertical-rl 块流起始＝右缘 x 左移 delta 保持块流位（R4428 viv restack 在 float 子树页关闭，taffy 右缘锚无人重算），vlr x 不动。float 子几何不改写——R5457 anchor（apply 12.7）按最终 content_width 重锚自动跟随。

## 3. 验证

- 探针矩阵（临时，已删）：A2 .vp w 16→56、abs x 776→736（=chromium）、float 右缘 784 不变；G/I 同构归位；H（definite height，另账）与 V1/V3/V4/V5 gate 自限不触。
- 常驻钉 2：`r5460_viv_mixed_content_block_extent.rs`（extent 56/cw 40/右缘锚保持/float 贴缘随 content_width/纯文本与 float-only 哨戒/abs 域同规则/显式宽不越权）＋`..._kill_switch.rs`（独立进程，`=0` 复原 16/0）。
- 驱动案 solo：vrl-002 **3.25% → 1.39%**（−1.86pp）；vlr-002 **2.99% → 1.24%**（−1.75pp）。ref 臂（abs+span）taffy 本已 56（span 文本形态无塌），臂对其 no-op，无双臂分裂。
- 内测 reftest **725/725**；fmt/clippy（-p zero-layout-engine --all-targets -D warnings）清零；layout-engine 全腿 **2500P/0F**（含 r5457 混排 gate 哨戒）。
- corpus 双窗（同二进制 16594）：w1(OFF) **694F** / w2(ON) **691F**；**P→F = 0**；F→P 3 案（table-anonymous-objects-129/148/151）solo×3 全 ✓✓✓（0.00% 精确）＝新增稳定红 0、**净 +3 稳定绿**（混合内容补偿惠及 table 匿名对象域）。

## 4. 残差与挂账（未了结债）

- 002/003 残余 1.24~1.39%：ref 臂 abs 宽 89 vs 211.48（taffy absolute 交换帧 sizing，R5455 家族）＋双臂共享 h 债（.vp 784 vs chromium 513/497，html root sizing 上游，双臂自洽零 reftest 影响）。
- html root sizing 链（ZW html h junk 应=fill ICB、html w 32 应=shrink 227.48 带、body fill 挂 html content）：chromium 规则已 oracle 定谳，002/003 零可见像素，**降级低优先**。
- definite-height 混合内容容器（探针 H：w=16 应 56）：本臂 height Auto 门排除，R4445 域亦不覆盖（float 子非纯 IFC），有驱动案再收。
- 块子混排 viv 容器 extent（R5457 钉文档：.vp extent = max(own col, Σ 流内子) 含块子；R109 片段域语义另账）；inline 级元素子混排（R4469 原子注入口径）另账。
- R1099 vertical advance 开门（待视觉消费驱动案）；R5457 窄门 FIXME（多列容器块流位/VerticalLr 镜像）。

## 5. 取证命令存根

```bash
# 驱动案 solo（ON 态）
./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vlr-002
# corpus 双窗（剥 ✗ 后 comm 互翻；输出在 stderr）
ZW_VIV_MIXED_EXTENT=0 ./target/release/zero-wpt-runner reftest-upstream
./target/release/zero-wpt-runner reftest-upstream
```
