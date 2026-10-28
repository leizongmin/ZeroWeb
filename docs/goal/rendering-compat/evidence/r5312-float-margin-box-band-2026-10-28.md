# R5312 — float 排除带 margin-box 口径（CSS2 §9.5）——taffy native float border-box 坐标下 `+margin` 双计；outline-028 0.70%→0.50%（案 2/3/5/6 逐像素转精确）；余差归因 ref 侧 inline-block 背景盖字 paint-order 缺陷（R5313 候选立档）

**日期**: 2026-10-28
**驱动案**: `css/css-ui/outline-028.html`（R5311 下一步②outline-028 第二层评估命中）
**corpus**: 两处树态与三处树态各跑 A/B/C 三方：fail-list 排序口径 md5 `85c078c8a50d9c26acabdbd1d6c99661` **六腿全等**（两处树态首轮 A/B + stash 归零 C；第三处并入后三处树态复跑 A/B + 双文件 stash 归零 C，均 16592 / 839F）——零翻转零新增红（案仍红系余差属 R5313，见下）。

## 定性（诊断链）

outline-028（outline:auto × float margin 矩阵，4 列 × 8 案，fuzzy maxDifference=0-2 totalPixels=0-4）修前 ✗0.70%。PNG 逐带比对 + 逐案归因：

1. **案 2（float ml:5）**：test 渲 span 于 rel x=25 行 2（ref：rel x=15 行 1）。paint stored 片段 `frags=[(25.0,10.0)]` 反推排除带 **[10,25]**，margin box 应 **[0,15]**。
2. **根因（坐标约定断裂）**：pre-M3 taffy 把 float 当 block 堆叠（float child x/y = margin-box 原点）→ 旧式 `x: c.x + ml, y: c.y + mt` 组带正确；**M3 step(b) 起 taffy native float 激活**（converter/mod.rs:168），float child x/y 已是 **border-box 位（margin 含在坐标内）** → 再 `+margin` 即双计。compute_final own_floats、remeasure exclusions 与 postprocess R4234 float_exclusions **三处构建同病**。
3. **案 3/5/6（mr/mb/水平双 margin）**：同源但仅 ml 被双计（旧式 width 项本含 mr）——修后与案 2 一并逐像素精确。
4. **案 4/7/8（含 margin-top 矩阵）修后仍差**：test 侧渲染经 char-map 核认**已正确**（案 4：span rel (10,0) + float 字形 rel (0,5)，恰 CSS 预期）；差异全在 **ref 页侧**——ref 用 inline-block `padding-top:5px; background:white`（白 bg 压字形 trick）模拟期望图，ZW 渲 ref 时**字形缺失**（色分类 char-map：ref 仅剩容器 lime 右半 10×10）。
5. **inline-block 背景盖字 = 新缺陷（R5313 候选）**：最小 probe 五连定谳——
   - G：ib `width:10px; background:white` + 黑字 "XX" → 右 X（溢出 bg 外）可见、左 X 被白 bg 盖；
   - H：同 G 去 bg → 双 X 全见（文字位置正确）；
   - I/K：bg 白 + 单 X / `width:5px` 部分盖 → 恰 bg 矩形区域压字；
   - 机制图像：FLAT_CHILD_WALK 坑（R4312 块子门把 InlineBlock 归 inline 级下潜）→ ib 内文字被扁平进父 IFC 先画；子盒遍历后画 ib bg 盖字；ib 自身 paint_text 经 `painted_inline_nodes`（text.rs:1286）dedup 跳过。CSS2.1 App E 要求 bg 先于行内内容。修复面涉及全线共用扁平化/去重路径，**须独立轮 corpus A/B**（R5313）。
6. **float walk y 双补偿直证（既有挂账升级实证）**：RenderPipeline 快照 span 盒 `pos=(0.00,-10.00)`（float_y_offset = mt+h+mb 整段被扣）——paint Path A 用 stored 片段（容器相对）不可见，但 gBCR/hit-test/Path B 面失效域坐实。

## 实现（三处构建点，无新 kill-switch）

`compute_final_inline_layouts` own_floats、`remeasure_text_with_float_exclusions` exclusions 与 `engine/postprocess.rs` `adjust_inline_block_positions` float_exclusions（R4234）统一改 margin-box 口径。第三处由并行会话 zeroweb-2-f0 独立扫出（该构建点服务 inline-block 行位/行首 x 的 float 排除，与 R4234 remeasure 覆盖域互补），四式与本轮口径逐值一致后并入；其快速腿（r5312 回归钉 1/1、本地 720 套件 719/720 零漂移、outline-028 ✗0.50% 维持）已由对方先行验证：

```rust
y: rel_y - c.margin_top,
height: c.margin_top + c.height + c.margin_bottom,
width: c.width + c.margin_left + c.margin_right,
x: c.x - c.margin_left,
```

- 左/上缘回退 margin 得 margin-box 原点；margin 字段被后继 pass 清零时 `−0 = no-op`，新旧坐标约定同成立（RenderPipeline 端实证：清零域两式同值）。
- remeasure 侧 height 用 R3787 补测后的局部 `height` 变量（同语义）。

## 回归钉（engine 级，stash-mutant 红在案）

`crates/engine/tests/r5312_float_margin_box_band.rs`：grid > max-content 容器（font:10px/1 Ahem）内 float ml:5 + span。判别信号 = **容器 auto 高**（RenderPipeline 端双计表现为测量面多折一行：mutant 下容器 20px = 2 行盒，修复后 10px = 1 行）+ 可见契约（span 字形停在 float 字形右缘 +10px 同行）。**mutant（四式反转）红：容器高 20 断言失败；修复后绿**。

调试注记（后续轮参考）：RenderPipeline 与 runner 对双计的**表现面不同**（runner：paint stored 片段 x=25 折行；RenderPipeline：仅测量面高 20、glyph 仍正确位）——引擎级测试判别须取测量面信号，不能只断言 glyph 位。

## 门禁

- fmt 0 diff；clippy 定向两 crate `--all-targets -D warnings` 0 告警（三处树态复跑覆盖 postprocess.rs 改动）+ make test quickjs 腿（13 crate）0 告警；r5312 回归钉三处树态复跑 1/1 绿。
- make test 全量（三处树态）：workspace 腿 **0F**（上轮 service_worker_owner SIGSEGV 未再现，族维持记账）+ quickjs clippy 腿 0 告警；renderer --bin 0P 过；renderer --lib **350P/2F**、browser --bin **704P/2F/1I**、browser --tests cdp 三件全绿——4 个红**stash 归零 HEAD 复现定谳既有红、非 R5312**：`page_scripts::tests::{form_interaction_fixture_complete_sequence, javascript_disabled_skips_listeners_not_default_actions}` + `tab_worker::action_adapter_tests::shared_checked_and_form_actions_preserve_transaction_semantics` + `tests::form_fixture_complete_multiprocess_semantics`，同一 form fixture 族签名（`#basic` 应 checked/value "basic" 实空——表单态未达提交/状态快照），属表单态/scripting 域（对端流近期 2cced1143 s125 form 状态传播、139dd9542 同步脚本让路为嫌疑面），移交对端并记挂账。
- product-smoke：struct ×6 全 PASS；welcome oracle **15.68%（75254/480000 px）与 R5308-R5311 零位移同值**（welcome 无 float+margin 容器，无因果路径）。
- corpus：两处树态 A/B + stash C 与三处树态 A/B + 双文件 stash C 六腿 fail-set 逐案全等（839 案 md5 同值）——零翻转。
- bench-gate 定向 zero-layout-engine：首跑 **GATE FAIL 10/25 超预算**——兄弟流（ZeroWeb-3 clone）release 构建活跃窗争用（load 13.36，legs=20；medium/layout_ms 521.81 vs R5311 同窗 148.52 = 3.5× 全面虚高，典型争用签名）。**静窗复跑 GATE PASS（25 指标 NEW=0 全在预算内，接管轮 15:13 窗；medium/layout 157.20ms / 预算 181.9）**——收口完成，基线零改动零 relax。
- make test 覆盖缺口补记（对端会话二分归因）：4 红隔离复跑稳定复现（非负载 flake），**stash A/B + 引入点二分定谳：s125^（3567f902b）renderer lib form 两条双绿 → s125（2cced1143）双红**——对端流 part05 radio 组排他「净件移 checked 属性」在 IDL setter 置真时把默认选中件（#basic）的 checked 内容属性一并剥除，破坏表单 reset 默认选中恢复契约（reset_state["plan"] 期 "basic" 得 ""、reset 后 #basic 无 checked 属性）。干净 HEAD 同红，与本轮改动无关；engine/renderer 为对端流域且其会话活跃，按并行纪律移交不硬解（碰头记账，修复须连同 radio.html corpus 侧联验防回摆翻红——s125 的 radio.html 4F→0 正是该属性剥离换来）。
- 临时 probe 页（r5312-probe*.html）与全部 ZW_DBG_R5312 插桩已清零（工作树仅剩修复 + 回归钉）；ZW_TMP_DBG 调试块清零。

## 挂账

- **R5313 候选（新立）**：inline-block 背景盖字 paint-order（本证据 §5 五连 probe 证据链完整）；修复面 = FLAT_CHILD_WALK 坑/dedup 交互，outline-028 余差 0.50% 全系此缺陷（ref 侧 4/7/8 案）。
- float walk `child.y -= float_y_offset` 双补偿（快照 span y=−10 直证升级；gBCR/hit-test/Path B 域，Path A reftest 不可见）——需自带 corpus A/B 轮。
- paint Path B `collect_float_exclusions_with_styles` border-box 带（无 margin 口径）潜在 mr 差——与 R5313 同域顺带核对。
- **form fixture 族 4 既有红（新记账，移交对端流）**：`#basic` 应 checked/value "basic" 实空——renderer --lib 2 + browser --bin 2 同签名（上列），stash 归零 HEAD 复现非 R5312；引入点二分定谳 **s125（2cced1143）**（radio 排他剥 checked 内容属性破坏 reset 恢复契约），修复归表单态/scripting 域流、须连同 radio.html 联验。
- image_batch_e2e + service_worker_owner compositor SIGSEGV 两个多进程负载脆弱族维持记账（service_worker_owner 上轮 workspace 腿未再现）；fetch-wpt-data 语料 footgun 双移交对端维持。
- text-wrap:balance 9F、welcome 15.68% 对端流、R5307/R5308 项、corner-shape 族（2 案深域）维持。

## 下一步 R5313

①839F 账本续扫择轻（不与 R5313 域重叠案优先）②R5313 inline-block paint-order 修复轮（probe 证据链已备，须 corpus A/B）③bench-gate 静窗并账（R5307-R5312 树态）④balance 供给评估。
