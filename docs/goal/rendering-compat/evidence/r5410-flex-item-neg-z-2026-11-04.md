# R5410 证据 — static flex item 负 z-index 下沉绘制（flex-item-z-ordering-002 ✗2.08%→✓0.00%，corpus 真态 717F→716F FIXED=1 REGRESSED=∅）

日期：2026-11-04。R5409 下一步① misc 账本续扫择轻驱动；② flexbox 残余域侦察结论：baseline-synthesis 5 案全部 vertical-rl/lr（竖排 baseline 合成域）并入 writing-mode 竖排深域记档。

## 1. 侦察与归因

- **flexbox 残余分布复核**（39F）：竖排域 ~19（writing-mode 14 + baseline-synthesis 5）＋ misc 单案（scrollbar 2、container max/min-content 2、dynamic/contain 多特性、first-line 字体墙对等）。
- **驱动案** `flex-item-z-ordering-002`（2.08%）：flex 容器两 item 各含 100×100 子盒，item2 `z-index:-10`（position:static）。css-flexbox-1 #painting「z-index applies to flex items even if they are position: static」——负 z item 应下沉到常规流内容之下（Appendix E step 2 类比）。REFTEST_DUMP 量化：10000/480000 px 差＝恰红盒整面，红上浮盖绿。
- **ZW 断点**（painter/mod.rs）：`child_paint_sort_key` 非 positioned 子恒 (1,0)/(2,0)，注释在案「非 positioned 元素的 z-index 不参与堆叠，v1 保守留 legacy 相位」；`style_creates_stacking_context` 的 z-index 臂要求 is_positioned（sizing.rs:3387）。CSS 2.1 语义正确，css-flexbox 的 flex item 特例未接。
- **家族面**：001（正 z flex items × positioned 兄弟）✓0.26% 判例不动（self-source 自洽，正 z 建 SC 域另案）；css-transforms 3d-rendering-context-and-z-ordering-001/002/003 ✓ 不涉。

## 2. 修复（painter 单点）

`child_paint_sort_key` 新增首分支：`parent_is_flex_container && !is_positioned && z_index < 0 → (0, z)`（下沉相位，负 z 升序保序）。

- 父面判定 `Painter::flex_neg_z_parent_gate`（kill-switch 折叠）：`flex_item_neg_z_enabled`（构造期单次读取，R5086/R5098 字段同型）∧ 父 style display Flex/InlineFlex。
- **三循环同门**：主子循环（paint_node_in_rect）与两处 collapse sweep（`collapse_sweep_in_flow` 加参透传）共用同一父面判定——R5265 sweep 集 ≡ 常规集不变量保持，排序序一致。
- 限定：仅负 z（正 z 建 SC 域无驱动案另案）；仅 flex 容器父（普通块流 z-index 非 positioned 不参与堆叠的 legacy 语义保持——Appendix E 本义）；负 z item 不建 SC（后代随 item 下沉，同负 z positioned 语义）。
- kill-switch `ZW_FLEX_ITEM_NEG_Z=0`（default-on）。

## 3. 常驻测试

`paint/tests/r5410_flex_item_neg_z_tests.rs` 2 针（OFF 轮由 corpus 002 ✗2.08% 精确回退覆盖，R3858 口径）：

1. `r5410_negative_z_flex_item_paints_below_inflow` ★load-bearing（002 形态）：fills 序 red idx < green idx（红先绘被绿覆盖）。
2. `r5410_block_parent_negative_z_keeps_tree_order` 非 flex 父守卫：负 margin 重叠兄弟 z:-10 恒 legacy tree order（green < red），flex 门不外溢普通块流。

## 4. 验证

- 2/2 新针；zero-engine lib **3724 绿**（5 败＝js_dom_bridge CSSOM 对端账同名单）；layout-engine 未触碰。
- kill-switch OFF 案面精确回 ✗2.08%、001 双态 ✓0.26% 不变、3d-transforms z-ordering 三案零扰动。
- corpus 全量 A/B（16594 案双跑窗口，BASE=同二进制 kill-switch OFF）：BASE 721F / FIX **719F** 账面——mover 3（insert-middle-003 / insert-block-middle-001 / table-anon-153）隔离双侧 3×3 **全绿洗清**（insert-middle + table-anon 在册抖动族，guidelines #35），真态 **FIXED={flex-item-z-ordering-002} REGRESSED=∅，717F→716F**。
- 本地集 **725/725**；fmt ✓；guarded-clippy 全工作区 EXIT 0；product-smoke 主+legacy EXIT 0；make test 唯一红腿＝js_dom_bridge CSSOM 对端账 5 案同名单其余 65 腿全绿。
- **bench-gate 定向 zero-engine**：首两跑 GATE FAIL（page/medium layout/first_paint 超预算）＋配对对照定谳——同窗 kill-switch OFF 腿同型超差且更差（layout 512 vs ON 424、paint 280 vs 230），负载污染非本改动（guidelines #11 配对洗清）；load1 220+ 对端重型负载（load5 峰 162）退避轮询至静窗（load1<2.5×2）复跑 **GATE PASS 19 指标（NEW=7，R5404 zero-engine 定向同款既有态）**。

## 5. 挂账与下一步

新增挂账：

1. **正 z-index static flex item 建堆叠上下文域**（css-flexbox #painting 后半——item 正 z 应 SC 化参与 step 7；001 现自洽过案无驱动，待独立案或 chromium oracle 对照定谳；css-grid grid items 同规则同域 FIXME）。
2. **flexbox 竖排域并账**：baseline-synthesis 5 案（001/002/003/004/vert-lr-line-under 全 vertical-rl/lr，text-orientation 基线合成位）并入 writing-mode 竖排深域（原 14 案 → 19 案口径）。
3. scrollbar 族（content-height-with-scrollbars 6.26% / cross-axis-scrollbar 5.36%）与 flexbox_first-line（::first-line 不适用 flex 容器语义 + 字体墙自洽对）维持侦察待排；align-items-stretch-3（vertical + padding-% 两阶段）、contains-size-layout-001（contain+grid+动态多特性）归多特性域。

维持：R5409 四面（单线零主尺寸候选/flexible strut FIXME/vertical 迁移缺口/strut margin）、R5408 残留四案、R5406 家族初判六面、R5405 十面及以前全维持；对端账维持。

**下一步 R5411**：①misc 账本续扫择轻——flexbox misc 单案（auto-margins-003 2.20% / align-self-016 4.17% / flex-wrap-006 2.08% 动态域 / flex-svg-no-intrinsic-column-001 4.69% 择轻）或全账本余量（避开竖排 19 深域/对端面/多特性域）②scrollbar 族两案侦察（同域成对，或特性级归档）。
