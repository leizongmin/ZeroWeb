# 待消费 clean lever 队列（code-agent hand-off）

> ⚠️ **状态：历史归档 / 队列已耗尽（R740 战略转向）**。本文为 R678-R709 时代的 clean-lever 只读分析，经 R726-R850 实证消费：7 lever LANDED（R689 / R716 / R720→R879 / R695→R842 / R699→R843 / R711→R850 / R679→R749）、3 lever 实证失效（R696 / R678 / R717）、余皆 R109 / taffy / complex structural-gated。R740 裁决「clean lever 队列实质耗尽，勿再盲消费 doc lever」。**master.md 现行结论：clean-lever surface 9 vein 全 exhaust（R2597-R2602），自主面 = plateau-guard + 文档纠偏，唯一推向 95% = 用户授权深结构。** 本文保留作历史根因参考；行号/路径经 R2623 对齐当前源码（`engine.rs` 已拆 `engine/postprocess.rs` / `engine/sizing.rs` 等子模块，`painter/mod.rs` 迁 `paint/painter/mod.rs`，`color.rs` 迁 `values/color.rs`）。

> 来源：R678–R709 LAYOUT_DUMP 脉络跨 12 目录采样（7 top-level + 5 CSS2 subdir）。
> 12 目录采样结论：**clean LAYOUT lever 全来自 box-model sizing 目录**（tables/normal-flow/position/grid/margin-padding-clear/floats-clear）；render/flow/dynamic 目录（flexbox/multicol/writing-modes/box-display/linebox/borders）均 structural/render/JS-gated，零 clean lever。
> 本文档汇总 **11 个 clean-ish lever + R678 float-width 簇**，按「effort × yield」排优先级，供 code-agent 消费。每条已 read-only 定位根因 + 代码位置（截至 R709，行号可能随主分支漂移，动手前 `grep` 核验）。
>
> ⚠️ **消费准则**：① 每个修复须 `make test` + scoped `make reftest` 零回归；② 涉及渲染/布局须额外 `make product-smoke`（DC-13 welcome 门禁，diff>20% 退出 2，R541 教训）；③ 用 chromium-Oracle A/B（`make reftest-oracle DIR=<case>`）确证 yield、排查假通过（self-source 同源 REF 会抵消）；④ 代码位置注释见各轮 master.md / archive 详记。

## 消费状态（R739 实证：执行 agent 实际修复验证，非只读分析）

> R726–R738 doc-side「极简监控态」基于**过时快照**判定 code-agent「stalled 55+ 轮」——
> 实际 table-cell overflow WIP 早已提交（`15e25a14`），后续还有 browser/HTTPS 工作。
> 「等 code-agent 消费」是误诊：**执行 agent 本身就是消费侧**。R739 起转为实际修复。

| lever | 实证结果 | chromium Oracle A/B | 备注 |
|---|---|---|---|
| **R689** | ✅ **LANDED** `d8f0003e` | position-static-001 **17.53%→0.90% PASS**；CSS2 twin 3.00%→1.60% | converter/mod.rs static→inset Auto；新增单测；css-position dir 无回归 |
| **R716** | ✅ **LANDED** `a559fbd7` | box-offsets-rel-pos-002 **9.18%→0.86% PASS** | engine.rs resolve_relative_inset 补 right/bottom（§9.4.3）；3 调用点共享 |
| **R720** | ✅ **LANDED R879**（真实根因 ≠ R721 假设） | background-root-{005,008,009,012a,012b} **平均 -72pp**（005 79.75→4.73 / 008/009 75→10.79 / 012a 79→4.24 / 012b 82.70→1.74） | R721「颜色变体」假设被 R721 自身证伪（color=Transparent ✓ 正确）；**R879 发现真根因 = `background-image:none`→`vec![None]` 非空致 canvas 传播 `!is_empty()` 误判有图片** → body green 不传播。修 = `painter/mod.rs` 新 helper `has_paintable_bg_image`（含非 None 图层才 true），2 处检查替换。5 案跌出 top-worst（残余 border/margin/`*{margin:1em}` 布局差未翻 PASS）。101/102/103（100%）= JS-driven 独立。 |
| **R696** | ❌ **anchor+cluster 全 inline-SVG-content 阻塞**（out-of-scope） | inline-replaced-width-009 **22.08%→22.08% 零变化** | anchor + 008/ib-007/ib-008 全用 `<svg:svg>` + `<svg:rect fill>`；ZW 不渲染内联 SVG 内容（goal line 118 排除），sizing 修不修都不 yield（内容缺失主导）。非-img 默认 300×150 sizing 修正确但因无确认 yield 已回退 |
| **R678** | ❌ **anchor 发散非 float-width** | font-size-zero-3 **33.53%→33.53% 零变化** | `box_node.float` 已正确 populate（float 字段现于 `converter/mod.rs:84` `float: match style.float` 填充），shrink-to-fit 实现正确，但 anchor 的 33.53% 不是 float 满宽所致（绿子/高度/位置他因）。float:right 定位顺序风险（x=container_w−used_w）下，无 yield 不值得保留，已回退 |
| **R717** | ❌ **decode 修对但 anchor 复合 bug 阻塞**（实证，R740） | replaced-elements-all-auto **31.66%→32.39% 反而 +0.73pp 回归** | decode_svg_bytes 经 probe 实证修对（7 SVG 全 match CSS spec：ratio-2 1000×500→300×150 等），**decode 修正确且有 probe 单测**，但 anchor 的 ~32% 由 **tree.rs §10.3.2 replaced-sizing + reftest harness img 处理**的复合 bug 主导（correct intrinsic 未转化为 chr-matching layout）；all-auto 反退 0.73pp。decode 修已回退（单独无 yield 且小回归）；**R717 须 decode 修 + tree.rs sizing 修同做**才能 yield，decode 修在 git 历史/master 可复用 |
| **R679 empty-table** | ✅ **LANDED（R749，空 table 子 facet）** | empty-table-height **30.74%→0.00% PASS**；css-tables 57→58 零回归；css-position 零变化 | `shrink_table_to_block_content` 早退（`block_indices.is_empty()`）致完全空的 display:table 保满宽。新增 `shrink_empty_table_to_padding_border` 收缩 width:auto 空 table 到 padding+border。**Auto-guard**（`matches!(s.width, LengthValue::Auto)`）跳过显式宽 table（Px/Percent/Em/Calc 由 taffy 解析）——无 guard 会回归 absolute-tables-007/012/016 + subpixel-table-width-001（全显式宽）。R679 多 facet（R681-R684 box/content/horizontal/vertical）仍 deferred |
| **R695** | ✅ **LANDED（R842，indefinite-CB %height → auto）** | height-percentage-005 **88.42%→0.67% PASS**；normal-flow 563→564 **+1/0**；visudet 16→16 **worst-15 字节同**；welcome 16.11% 不变 | 新增 `apply_indefinite_percent_height_to_auto`（engine.rs，~60 行），复用 apply_intrinsic_content_sizing 两趟 set_style+mark_dirty+重算基础设施。自上而下按样式判 CB 明确性（同 `clamp_percentage_max_height` 的 `my_definite_content_height`）：%height + CB 不明确 → taffy size.height=Auto；替换元素补 img_intrinsic_sizes 固有绝对尺寸。第二趟 taffy 自动算非替换块内容高/替换固有。「complex」标签实为 clean slice（既有级联 + 两趟基建）。详见 [`evidence/r695-indefinite-cb-percent-height-2026-06-30.txt`](./evidence/r695-indefinite-cb-percent-height-2026-06-30.txt)。两趟+definiteness 级联模式可供同区组 A（R699/R702）复用 |
| **R699** | ✅ **LANDED（R843，非 BFC 父 height:auto 忽略 float 子）** | block-non-replaced-height-011 **16.10%→0.42% PASS**；normal-flow 564→565 **+1/0**；floats-clear 53→54 **+1/0 worst-15 字节同**；welcome 16.11% 不变 | 新增 `exclude_floats_from_non_bfc_auto_height`（engine.rs 步 5.2），后序递归重算 `style.height==Auto && !establishes_bfc` 块的 content_height=in-flow 子 border-box 底 max（无 in-flow→0）。**★ has_float_child 守卫**（首轮无守卫 net -13：max(child.y+child.height) 公式对负 margin 不精确，误收缩无 float 用例如 root-box-001；守卫=仅 float 子存在时重算）。establishes_bfc 已全 BFC 条件无须 expand。详见 [`evidence/r699-non-bfc-parent-height-excludes-float-2026-06-30.txt`](./evidence/r699-non-bfc-parent-height-excludes-float-2026-06-30.txt)。同区组 A R695+R699 连续 LANDED |
| **R711** | ✅ **LANDED（R850，block-level relative top/bottom % inset 垂直 slice）** | css/CSS2/positioning Oracle **227→237/520（43.7%→45.6%）+10 case PASS 0 回归**；bottom-113 47.46→0.75 / top-113 39.50→0.75 / bottom-103 31.94→0.75 / top-103 23.82→0.75 / bottom-104 31.94→0.75 / top-104 23.82→0.75 / relpos-calcs-001/002 3.29→0.84 / relpos-calcs-007 1.81→0.84；welcome 16.11% 不变 | 新增 `apply_block_relative_percent_insets`（engine.rs ~60 行）。**★ 仅垂直轴 + 仅 definite CB**：taffy 0.7 应用 Length + left/right %（水平轴已工作）但丢弃 top/bottom %（R715 实证）；本 pass 只补 top/bottom % delta（首版含水平 % 致 left-103/104/113 等 double-count 回归，A/B 锐化为垂直 only）。CB definiteness 用 style.height==Px 判定。bottom-091/092（`ex` 单位非 %，原误标 R711 cluster）/ relpos-calcs-006（RTL 水平 overconstrained）/ right-113（水平亚像素）= 独立子问题非 R711 scope。详见 [`evidence/r850-block-relative-percent-inset-2026-06-30.txt`](./evidence/r850-block-relative-percent-inset-2026-06-30.txt)。pending-levers 全 clean slice 收官 |

**★ 关键裁决（R739/R740 实证）**：doc-side 只读 lever 分析的**假阳性率高**——R720/R696/R678/R717 四 lever 实证零 yield 或反退（premise 错 / out-of-scope / 发散另有他因 / 复合 bug）。**「14 clean lever 待消费」叙事大部分是幻觉**；pass-rate 提升需 per-case 实际修复验证，非盲信只读根因。

**★ 剩余 lever 状态（R740 评估）——clean lever 队列实质耗尽**：R689/R716（已 landed，仅有的 2 个真正 clean）；R720/R696/R678/R717（实证失效）；**R702 = R109 territory**（collapse-through + §9.2.1.1 匿名块，doc 自标「类 R680 R109 同族」，已知结构性 deadlock）；R695（%height indefinite-CB，需 CB-definiteness cascade，complex）；R691（grid %height，taffy-gated R304）；R679（table shrink-to-fit，table-width 多 facet）；R699（float-height exclusion，medium）；R705（clearance 算术 complex）；R692（aspect-ratio，oracle 损坏需先 regen）；R680（br 行盒，R109 blast radius）；R711（%inset，**R850 垂直 slice LANDED +10 case**，水平 % 仍 taffy-deferred）。**剩余全部为 R109-gated / taffy-gated / complex / 需复合修复，无单点 clean lever**。

**★ 战略转向（R740）**：lever 盲消费已无 EV。后续轮次应：① 勿再盲消费 doc lever（premise 假阳性高 + 剩余全 structural-gated）；② 改用**直接 chr-vs-ZeroWeb 简单用例对比**找新 high-confidence 修复（R689/R716 即此类——简单 inset handling）；③ 或攻坚**结构性根因**（R109 §9.2.1.1 匿名块 / Phase-A IFC 统一 / taffy 升级 R304），这些是多会话架构工作，需按 rally 跨会话协议推进，非单点 lever。

## 优先级总览

| 优先级 | lever | 测试 / 目录 | 发散 | 复杂度 | cluster yield |
|---|---|---|---|---|---|
| **A 最高 yield** | R720 | background-root-005 / CSS2_backgrounds（canvas bg 传播不触发） | 80.7%（簇 100%） | 低-中（wiring+测试） | **~15 案 26-100%（含 3 案 100%）** |
| **S 快速赢** | R689 | position-static-001 / css-position | 17.53%+3% | 低（一行级） | 2 案（twin） |
| **S 快速赢** | R716 | box-offsets-rel-pos-002 / CSS2_visuren（relative right/bottom inset） | 9.18% | 低（一函数 ~6 行） | 2-3 案（001/002 twin + inline right/bottom） |
| **A 高 cluster** | R678 | font-size-zero-3 / css-fonts（float shrink-to-fit） | 33.53% | 中 | **≥5 案 17-34% 跨 3 目录（最高渗透）** |
| **A 高 cluster** | R696 | inline-replaced-width-009 / CSS2_normal-flow（replaced sizing） | 22.08% | 中 | **≥5 案 12-22%（unified）** |
| **A 高 cluster** | R717 | replaced-elements-all-auto / CSS2_visudet（SVG intrinsic-size viewBox-as-intrinsic） | 31.66% | 中 | **~9 案 9-32%（visudet replaced-elements-* 簇，R718 扩）** |
| ✅ **R695 LANDED R842** | R695 | height-percentage-005 / CSS2_normal-flow（%height indefinite-CB） | ~~88.44%（最高）~~ → **0.67% PASS** | ~~中~~ 实 clean | +1（anchor；visudet 字节同 0 回归） |
| B clean | R702 | margin-collapse-101 / margin-padding-clear | 49.38% | 中 | 7 案（101/105/106/155/038/110/111） |
| B clean | R691 | replaced-element-...-002 / css-grid（grid-item %height vs track） | 34.00% | 中 | +replaced-...-001 9% |
| B clean | R679 | empty-table-height / css-tables（table shrink-to-fit） | 30.74% | 中 | table-width 簇 |
| ✅ **R699 LANDED R843** | R699 | block-non-replaced-height-011 / CSS2_normal-flow（§10.5.1 float-height） | ~~16.12%~~ → **0.42% PASS** | 中（has_float_child 守卫） | +2（anchor + floats-clear 一例，0 回归） |
| C complex | R705 | margin-collapse-clear-015 / floats-clear（clearance+collapse） | 33.66% | **中高** | 4 案（015/014/012/013） |
| C caveat | R692 | nested-grid-item-block-size-001 / css-grid（aspect-ratio） | 真 64.15% | 中 | **oracle 损坏须先 regen** |
| C caveat | R680 | table-cell-width-0 / css-tables（br 行盒高度） | 20.09% | 中 | §9.2.1.1 blast radius 风险 |
| ~~C caveat~~→**✅ R711 LANDED R850** | R711 | bottom-113 / CSS2_positioning（relative 百分比 inset 垂直 slice） | ~~47.48%~~ → **0.75% PASS（+10 case 垂直 %）** | ~~中高~~ 垂直 slice clean | bottom/top-113/103/104 + relpos-calcs-001/002/007 **+10 case**；水平 %（right-113/relpos-calcs-006 RTL）仍 taffy-deferred |

---

## Tier S — 快速赢（最低 effort，建议先做）

### R689 · position:static 元素被应用 inset（应忽略）
- **测试**：`css-position/position-static-001`（17.53%）+ `CSS2/positioning/position-static-001.xht`（3.00%，同 bug twin）
- **根因**：`converter/mod.rs:71`〔R689 审计态，现 :79 `let is_static` 已条件化〕inset **无条件**传 taffy + `:295`〔现 :434〕`PositionValue::Static => taffy::style::Position::Relative`（Static 映射 Relative，taffy Relative 应用 inset 偏移）→ static 元素被 top/left/right/bottom 偏移。
- **fix scope**：`position==Static` 时 inset 归零（converter/mod.rs:79-82 加 position 条件，Static 不传 top/left/right/bottom）。**一行级条件修复**，spec-correct（CSS：inset 仅适用于 non-static position）。
- **风险**：低（无 spec 测试期望 static 元素被 inset 偏移）。

### R716 · relative 的 right/bottom inset 丢失（resolve_relative_inset 仅读 left/top）
- **测试**：`CSS2/visuren/box-offsets-rel-pos-002`（9.18%）+ `box-offsets-rel-pos-001`（twin）+ 其他 inline right/bottom-only relative 偏移案
- **根因（R716 LAYOUT_DUMP + 代码 read-only 确证）**：`engine/postprocess.rs:1652 resolve_relative_inset`（root relative 调用点 `engine.rs:479` + inline-level relative `apply_relative_offsets_inline`〔`postprocess.rs:528`〕共用）`dx = style.left(Px)` / `dy = style.top(Px)`——**完全忽略 right/bottom**。dump 实证：`img{left:100}` x=108✓（left 应用）/ `img{right:100}` x=208✗（应 108，right 丢）/ `img{top:100}` abs_y=151✓ / `img{bottom:100}` abs_y=251✗（应 151，bottom 丢）。CSS §9.4.3：relative 的 `right`（无 left 时）应向左偏移、`bottom`（无 top 时）应向上偏移。**block-level relative 由 taffy 处理（不受此函数影响），仅 inline-level + root 受此 bug 影响。**
- **fix scope**：扩展 `resolve_relative_inset`——dx：`left` Px→`+left`，elif `right` Px→`-right`；dy：`top` Px→`+top`，elif `bottom` Px→`-bottom`（§9.4.3）。**一函数 ~6 行**，spec-correct。
- **风险**：低（仅 right/bottom-only relative offset 元素受影响；block-level relative 走 taffy 不变）。
- **次级 note**：resolve_relative_inset 对 left/top 亦 **Px-only**（Em/Rem/Percent 丢），与 R715 percent-inset taffy 限制谱系；本 lever 聚焦 right/bottom drop（confirmed），Px-only 扩展为可选 follow-up。

---

## Tier A — 高 yield（cluster 或最高单发散）

### R720 · canvas bg 传播标准 render 路径不触发（★最高 yield ~15 案含 3 案 100%，render-feature）
- **测试**：`CSS2/backgrounds/background-root-005`（80.7%）+ 101/102/103（**100%**）+ 012a/b（85/81%）+ 008/009（76.8%）+ 010/006/018/002/020/007（26-38%）+ background-attachment-applies-to-004（31%）= **~15 案 26-100%**
- **根因（R720 + R721 minimal-repro 4-case 锐化确证）**：`paint/painter/mod.rs:413-415` `html_has_bg = hs.background_color != ColorValue::Transparent || ...`。**canvas 传播对 HTML5 html-direct bg + body-via-implicit-transparent 均工作**（R721 margin 测试证红/绿填满 viewport）。**唯 explicit `html{background:transparent}` 触发失效**：`background:transparent` shorthand 产出 ColorValue ≠ `Transparent` enum variant（疑 Rgba(0,0,0,0)/Named，shorthand 路径不经 css-parser/src/values/color.rs:30 长hand 规范化）→ `!= Transparent` 误判 TRUE → html_has_bg=true → 取 html 分支（html "bg" 透明→`add_fill` 被守卫跳过）→ **body 永不传播**。隐式 initial `background_color`==Transparent（→ html_has_bg=false→body 分支→工作）。REFTEST_DEBUG（R720）对 background-root-005（explicit transparent）证无 canvas fill 正确，但 R720「传播块静默失效」归因过宽——R721 证传播块对 HTML5/implicit 工作。
- **fix scope（crisp，hand-off code-agent，低 effort）**：① 规范化 `background:transparent` shorthand → `ColorValue::Transparent`（对齐长hand css-parser/src/values/color.rs:30）；或 ② 修传播检查用 **alpha==0**（非 enum variant 比较，Rgba(0,0,0,0) 也算 transparent）。**+ 补 propagation 通过 render_html 全路径测试**（防再静默失效）。**R721 已 crisp root cause，无需 debug print**。
- **风险**：低-中（canvas 传播影响每页 body/html 背景铺满；修后须 A/B product-smoke welcome/legacy——body 背景全铺是基础视觉，或暴露其他被白底掩盖的布局问题）。
- **★ 意义**：浏览器基础视觉（每页 body/html 背景铺满 viewport），修后产品可见度极高；「逻辑存在但静默失效 + 无测试」模式（R491/R507）。

### R678 · float 0-content / width:auto 取满宽非 shrink-to-fit（★cluster≥5 案跨 3 目录，最高渗透 lever）
- **测试**：`css-fonts/font-size-zero-3`（33.53%，R678 pin）+ `floats-clear/float-non-replaced-width-007`（21.98%，R704）+ `floats-clear/floats-125`（28.80%，R706）+ `floats-clear/floats-124`（28.24%，推同族）+ `CSS2/positioning/abspos-008`（17.64%，R714：`.control` red float w=784 应 ~185；abspos §10.3.7 shrink-to-fit `.outer` w=185 已正确，发散纯 .control float）
- **根因**：`float_positioning.rs:14` 注释明言「taffy 将 float 当普通 block（按正常流）」→ float width:auto 被当 in-flow block 拉伸到满宽（784）。ZW `float_positioning.rs` 仅 `shrink_vertical_blocks_to_content`（:47，仅垂直 WM）+ `shrink_inline_blocks_to_content`（:111，仅 inline-block R180），**无水平 WM float shrink 后处理**。
- **fix scope**：加水平 WM float shrink-to-fit 后处理（扩展 `shrink_inline_blocks_to_content` R180 模式到 float，或 taffy 0-content float 测量 clamp）。ZW-side post-process 可行（float 位置由 float_positioning.rs 独立设，与父 height 解耦）。
- **风险**：中（须 A/B 排查 float-heavy 测试，但 spec 上现「满宽」行为错）。

### R696 · svg/canvas replaced 元素无 sizing（★unified cluster≥5 案）
- **测试**：`CSS2/normal-flow/inline-replaced-width-009`（22.08%）+ 008/ib-007/ib-008 + `css-tables/percent-height-replaced-in-percent-cell-003`（R683，canvas 满宽）
- **根因**：`tree.rs:366 apply_replaced_element_sizing`（img/canvas gate 现 ~:398-405）——仅 img/canvas 走 §10.3.2 sizing；`engine.rs:1081` 已把 `img|video|iframe|embed|object|svg|canvas` 全标 `is_replaced=true`，**但 svg/canvas/object/iframe/video early-return 无 sizing**→taffy 当普通元素（满宽 / content 高 0）。
- **fix scope**：扩展 `apply_replaced_element_sizing`（tree.rs:366）超越 img——对 svg/canvas/object/iframe/video：① 读 width/height HTML attr；② 无 intrinsic width 且无 intrinsic ratio 时 used width 默认 **300px**（§10.3.2），无 intrinsic height 默认 **150px**；③ 有 intrinsic 时按 intrinsic+ratio。**code-located 一处 + 簇 yield≥5**。
- **风险**：中（现有 img sizing 不变，新增 svg/canvas 路径，须 A/B）。

### R717 · `<img src=*.svg>` SVG intrinsic-size 用 viewBox 当固有尺寸（★cluster 5 案 30%，与 R696 正交）
- **测试**：`CSS2/visudet/replaced-elements-all-auto`（31.66%）+ min-height-20/40 + min-width-40/80（5 案 30-32%）+ height-20/max-height-20/max-width-40/width-40（4 案 9%，R718 确证 explicit-width height 推导同根）= **~9 案 9-32%，同 7 SVG + 约束变体，同根因**
- **根因（R717 LAYOUT_DUMP + SVG 文件 + 代码 read-only 确证）**：`image_cache.rs:565-566`（`decode_svg_bytes` :560 内）`let w = size.width().ceil()/h = size.height().ceil()`（注释「按 SVG 内在尺寸（width/height 属性或 viewBox）栅格化」）——**usvg 的 size.width()/height() 在 width/height attr 缺失时回落到 viewBox 维度**，ZW 直接当固有 size。dump 实证：`<img src=height-25-ratio-2.svg>`（height:25 viewBox:1000×500 无 width）→ img **w=1000**（应 width=height×ratio=50；viewBox width 当固有）；`ratio-2.svg`（仅 viewBox 1000×500）→ img **1000×500**（应默认 300×150）；no-ratio 缺维 → img **100**（应 300/150 默认）。**混淆 viewBox（定义 ratio）与 width/height attr（定义固有 size）。**
- **fix scope**：image_cache.rs:565-566 SVG intrinsic-size 提取须 ① 仅 width/height attr 作固有 size（usvg 区分 attr-size vs viewBox-fallback）；② 无 attr 时报「无固有 size」让 tree.rs §10.3.2 默认/ratio 逻辑处理（传 viewBox ratio）；③ 默认 100→**300×150**。**2 sub-bug**（viewBox-as-intrinsic + wrong-default），中复杂度。
- **风险**：中（影响所有 `<img src=*.svg>`，须 A/B；与 R696 正交——R696 = 非-img 替换 TAG 无 sizing；R717 = img SVG 源固有 size 提取错，不同 code path 可独立修）。
- **关联**：R685/R686 image_cache.rs:560 vein（彼判 mix；本案 viewBox-as-intrinsic 子 facet clean 且 high-yield 5 案）。

### R695 · %height 对 indefinite-CB-height 解析到 CB width（应 compute-to-auto）（★最高单发散 88%）
- **测试**：`CSS2/normal-flow/height-percentage-005`（**88.44%，全 lever 最高**）
- **根因**：CSS §10.5——%height 仅当 CB height **显式 specified** 时解析，否则 **compute to auto**。本案 CB 链 grandparent(0,definite)→parent(auto,**indefinite**)→child(100%,**应 auto**)→img(100%,**应 auto→intrinsic 96**)。ZW 把 %height 传 taffy 作 Percent，taffy 0.7.7 在 indefinite-CB-height 时 fallback 到 CB width(784)；**ZW 无 normal-flow %height 后处理**（`engine/postprocess.rs:1426 clamp_percentage_max_height` 仅处理 max/min-height Percentage，不处理 height Percentage）。
- **fix scope**：仿 `clamp_percentage_max_height`（engine/postprocess.rs:1426）加 §10.5 处理：遍历布局树，对 height:Percentage 元素，若其 CB height indefinite（auto / percent-of-indefinite）则把 %height compute-to-auto（→ block 取内容高 / replaced 取 intrinsic；本案 img→96）。须传播 CB-height-definiteness（cascade）；blast radius 须 A/B（definite-CB %height 案如 min-height-094/095 须仍 pass）。
- **风险**：中（非一行，须 CB-definiteness 传播，但 spec 明确、本案 88%→~0%）。

---

## Tier B — clean single / cluster（中 yield）

### R702 · margin collapse-through 丢 inline content height
- **测试**：`CSS2/margin-padding-clear/margin-collapse-101`（49.38%）+ 105（identical）+ 106/155/038/110/111（cluster 7 案）
- **根因**：div.b（块子空 .red **+** inline "B" 文本）被算 h=0——margin collapse-through 嵌套块时 zeroing 父 content height，丢了 inline "B" 匿名块行高（§8.3.1 collapse-through + §9.2.1.1 匿名块）。`margin_collapse.rs:83 collapse_two_margins` + `:33 establishes_bfc` 基础已有，gap 在 collapse-through 路径保 inline 高度。
- **fix scope**：collapse-through 须保留父的 inline/匿名块内容高度（类 R680 §9.2.1.1 同族）。

### R691 · grid-item %height 解析到 grid 容器而非 track
- **测试**：`css-grid/replaced-element-percentage-height-in-grid-nested-in-flex-002`（34.00%）+ replaced-...-001（9.00%）
- **根因**：grid-item `height:100%` 解析到 grid 容器高度（400）而非其 row track（1fr=100）。违反 css-grid §11（grid item %size 应对 its grid area/track 解析）。
- **fix scope**：grid-item 百分比 size 解析须对 item 的 grid track（row height for %height / column width for %width）而非 grid 容器。

### R679 · display:table 0-content 取满宽非 shrink-to-fit
- **测试**：`css-tables/empty-table-height`（30.74%）
- **根因**：`display:table; padding:155px` 空 table 取满宽（784），应 shrink-to-fit（CSS 2.1 §17.5.2：table width = min(CB width, max-content)；空→~padding+border≈312）。table width:auto sizing（table.rs R177 territory）。
- **fix scope**：定位 0-content shrink-to-fit fallback（与 R678 float 同 symptom 簇，可能共享根因或独立 table path）。**注意**：R681/R682/R683/R684 已证 table-width 是多 facet 簇（box/content/horizontal/vertical），修须 writing-mode-aware shrink-to-fit + 子节点约束两层级。
- **★ 状态（R749）**：**empty-table 子 facet 已 LANDED**（`shrink_empty_table_to_padding_border`，width:auto 空 table 收缩到 padding+border，Auto-guard 跳过显式宽）。R679 anchor empty-table-height **30.74%→0% PASS**。R681-R684 的 box/content/horizontal/vertical 多 facet 仍 deferred（需 col-sizing 路径完整 trace）。

### R699 · §10.5.1 非-BFC 块父 height 计入 float 子（应排除）
- **测试**：`CSS2/normal-flow/block-non-replaced-height-011`（16.12%）
- **根因**：`float_positioning.rs:14`（taffy 把 float 当 in-flow block，**计入父 content height**）→ 非-BFC 父（overflow:visible）继承 float-inclusive height。CSS §10.5.1：block + overflow:visible + height:auto → height = in-flow 子顶到底距离，**floating box 显式 ignored**。
- **fix scope**：加 ZW parent-height post-process（类 `clamp_percentage_max_height` / R695 §10.5）：对非-BFC 块父（`establishes_bfc` margin_collapse.rs:33 / `is_flow_root` engine.rs:1146），重算 height = 仅 in-flow 子的 bottom（排除 float 子）。**次级 issue**：`is_flow_root` 仅查 FlowRoot|InlineBlock，未含 overflow:hidden/clip/auto/scroll BFC——若一并修须 expand BFC 检测。与 R108b（float flow_bottom margin）/R145（flex item float 归零）正交（那些是 float 子**位置**，R699 是**父 height**）。

---

## Tier C — complex 或 caveated（高 effort / 有前置）

### R705 · clearance + margin-collapse 算术（COMPLEX）
- **测试**：`CSS2/floats-clear/margin-collapse-clear-015`（33.66%）+ 014/012/013（cluster 4 案）
- **根因**：`#clear-left{clear:left} > div{margin-top:140}`——ZW 做 clear（clear-left 在 float 下）+ collapse（父子同位），但 **clearance + collapse-through-clearance 算术多出 ~110px**（#next-yellow 低 352 vs CHR ~240），子 mt=140 未完全 collapse 进 clearance（§8.3.1 + §9.5.1）。
- **fix scope**：修 clearance + collapse-through-clearance 算术。**medium-high 复杂**（clearance 多步算术），与 R702 margin-collapse 同 area 可一并评估。

### R692 · replaced img CSS aspect-ratio 被固有比覆盖（★oracle 损坏，须先 regen）
- **测试**：`css-grid/nested-grid-item-block-size-001`（committed oracle 20.41% / **真 64.15%**）
- **根因**：`tree.rs` replaced-sizing 分支（~270-285）用 `Length(ch * w/h)` 固有比推 auto 侧，**忽略 `computed.aspect_ratio`**——CSS aspect-ratio 应胜过固有比（本案 width 应 height×2=880，非固有 height×0.5=220）。
- **⚠️ 前置**：committed chromium oracle 损坏（blank broken-img placeholder，抓取 race，R388/R692 class）——**须先 regenerate 本案 oracle**（puppeteer 重抓 + `img.complete`/`naturalWidth` 等待），或建议 `chromium-oracle-shot.mjs` 加 img-load 等待防 race 复发，再 A/B 修复。

### R680 · br-between-block-siblings 匿名块行盒高度（§9.2.1.1，blast radius 风险）
- **测试**：`css-tables/table-cell-width-0`（20.09%）
- **根因**：`<br>` 作 block 容器直系子、与 block 兄弟混排时应被匿名块盒包裹（§9.2.1.1），匿名块 IFC 为 br 生成 line-height 行盒（~19px）；ZW br h=0（`inline/mod.rs:1122` br→InlineItem::Br IFC 内正确，但 br-between-block-siblings 匿名块 strut 缺失）。
- **fix scope**：补 br-between-block-siblings 匿名块行盒高度。**R109 territory，须评 blast radius**（br-between-blocks 模式常见，修后或惠及多 case 或致位移；依赖当前 h=0 br 行为的 case 可能受影响）。

---

### R711 · position:relative 百分比 inset 未应用（✅ R850 垂直 slice LANDED；水平 % 仍 taffy-deferred）
- **测试**：`CSS2/positioning/bottom-113`（~~47.48%~~→**0.75% PASS**）+ top-113（~~39.50%~~→**0.75%**）+ bottom/top-103/104（~~31.96/23.82%~~→**0.75%**，identical 对）+ relpos-calcs-001/002/007（→**0.84% PASS**）= **+10 case**；bottom/top-091/092（16.10/15.77%）实为 **`ex` 单位**测试（非 %，原误标 cluster，R512/R544/R547 font-metric 谱系，非 R711 scope）
- **R715 decisive disambig（minimal repro）**：隔离 `#parent{height:200px}` > `.len{relative;top:50px}` + `.pct{relative;top:100%}` LAYOUT_DUMP → `.len abs_y=58`（length offset 50 **应用✓**）/ `.pct abs_y=48`（percent offset 200 **未应用**，应 248）。**裁决：LENGTH relative inset 工作，PERCENT 不工作（definite CB 亦然）= percent-specific taffy 0.7 限制**。
- **R850 fix（LANDED，clean slice）**：R715「complex」实为「垂直 % slice clean」——新增 `apply_block_relative_percent_insets`（engine.rs ~60 行），仅补 **top/bottom % delta**（CSS §9.4.3 top 优先否则 bottom 取负），CB definiteness 用 style.height==Px 判定（根 CB=视口）。**★ 仅垂直轴**：R850 A/B 实证 taffy 0.7 已应用 left/right %（水平轴），首版含水平 % 致 left-103/104/113/right-103/104/relpos-calcs-003/004/005 double-count 回归（0.46%→4.28%），锐化为垂直 only。Px 已由 taffy 处理无 double-count。详见 [`evidence/r850-block-relative-percent-inset-2026-06-30.txt`](./evidence/r850-block-relative-percent-inset-2026-06-30.txt)。
- **裁决**：垂直 % slice 已 LANDED（+10 case / 0 回归 / welcome 16.11% 不变）。**水平 %（right-113 2.34% / relpos-calcs-006 RTL overconstrained 3.02%）仍 taffy-deferred**（taffy 0.7 已应用水平 %，残余是 RTL overconstrained resolution / 亚像素，须 taffy 升级 R304 或独立 RTL 解析）。

## 消费顺序 & 交互 note（R732，grounded 于 R730/R731 已验代码位置）

> 多 lever 共享 code path 或同区 post-process，消费时须防冲突/重复应用。按「同区→统一 pass」+「正交→可并行」分组。

- **同区组 A — engine.rs parent content-height / sizing post-process（model on `clamp_percentage_max_height` engine/postprocess.rs:1426）**：**R695**（%height indefinite-CB → auto）+ **R699**（非-BFC 父 height 排除 float 子）+ **R702**（collapse-through 保 inline 高度，margin_collapse.rs:83）三者**都改 parent content-height 算法**。**建议作为统一 parent-sizing post-process pass 的不同 step 实现**（非三个独立 pass），顺序 R702（collapse 保 inline）→ R695（%height→auto）→ R699（排除 float），每步幂等、避免 double-apply；R699 附带次级 `is_flow_root` BFC 检测 expand（engine.rs:1146 现仅 FlowRoot|InlineBlock）。
- **正交组 B — replaced sizing，可并行**：**R696**（tree.rs:366 tag-level，svg/canvas/object/iframe/video）与 **R717**（image_cache.rs:565 img SVG source-intrinsic）不同 code path，独立可并行（hand-off 已注「正交」）。
- **同症组 C — shrink-to-fit，不同 path 可并行**：**R678**（float，float_positioning.rs）与 **R679**（table，table.rs）同 0-content shrink-to-fit symptom 但独立 path；**注意** R681-R684 已证 table-width 是多 facet 簇（writing-mode-aware），R679 修须 writing-mode-aware。
- **包含组 D — relative inset**：**R716**（resolve_relative_inset engine/postprocess.rs:1652，Px right/bottom，clean LANDED）+ **R711 垂直 % slice**（apply_block_relative_percent_insets，R850 LANDED +10 case）皆已落地；**R711 水平 %**（right-113/relpos-calcs-006 RTL）仍 taffy-deferred（R304）；R711 fix 的 CB-height-definiteness 判定与组 A 的 R695 §10.5 同族（共享 definiteness 逻辑）。
- **独立可并行**：R720（painter/mod.rs canvas bg）、R689（converter/mod.rs static-inset）、R691（grid track）、R705（clearance 算术）、R692（aspect-ratio，须先 oracle regen）、R680（br 行盒，R109 territory 评 blast radius）——各 lever 独立 code path，无强交互。

## 已识 structural-gated 区（非 clean lever，多会话架构，勿单点重试）

- **css-flexbox**（inline-as-IFC，R109/R255）：inline-box-blockification（converter inline→taffy::Block）。
- **css-multicol**（column-flow，Phase A）：无 column distribution（内容不跨列），column-span 不 fragment。
- **css-writing-modes**（axis-swap，R114）：vertical/sideways 模式轴旋转未实现（5.9% pass）。
- **css-grid** taffy track-sizing（R304 deferred）：隐式 auto-column 不扩展、grid intrinsic sizing。
- **原生 form-control 渲染**（R688）：platform-specific，out-of-scope。
- **box-display insert 簇**：dynamic-JS DOM mutation，须 JS/DOM-bridge 成熟。
- **linebox IFC**（R109 Phase-A deadlock，R247/R125）：line-box/leading/baseline/vertical-align。

## dormant / masked lever 候选（ZW-side 可修但暂无 clean harvest case）

- **calc() in margin/padding → 0**（R694 次级发现，grid-calc-margin 被 taffy w=0 masked）：`converter/mod.rs:601 convert_length_to_lpa` 的 `LengthValue::Calc(_) => length(0.0)` 把 `calc()` margin/padding 直接归零，对比 `convert_length_to_dimension`（width/height，:500）有 `extract_calc_percentage`（:1397）。**ZW-side 可修**（extend `extract_calc_percentage` 到 lpa 路径），但全语料 12 个 calc-margin 测试全 dynamic/JS 或被 taffy/grid masked，**无 clean block-context harvest case**——须先有 taffy w=0 修复（R304）或找到 calc-margin 为唯一 issue 的静态 case 才能验证 yield。低优先，待 harvest case 出现再激活。

## 工具链（read-only 复现 / A/B）

- `make reftest-oracle DIR=<dir|case>` — chromium-Oracle 真一致率 + top-15（DC-14 anti-false-pass）。
- `LAYOUT_DUMP=1 make reftest-oracle DIR=<case>` — 布局树 abs_y/height/margin/padding dump（裁决 clean vs structural）。
- `REFTEST_DUMP=1 make reftest-oracle DIR=<case>` — ZW vs self-source PNG（注：self-source 同源 REF 会抵消，须 chromium-Oracle 证）。
- PIL（`python3 + PIL`，无 numpy）— oracle validity / 像素 bbox / 颜色定位。
- `make product-smoke` — DC-13 welcome 回归门禁（渲染/布局变更必跑）。
