# R5296 — inline bg/border 多行判据升级（自身 line-height 入阈）——content-height-001 ✗2.50→✓0.35

日期：2026-10-27。改动：`crates/engine/src/paint/painter/mod.rs`（+83/−5，两处 bg 抑制门 + `inline_owner_max_lh` 索引 + kill-switch 字段）、`crates/engine/src/paint/painter/text.rs`（+31/−1，R639 fragment 门同源升级）、`crates/engine/src/paint/tests/mod.rs`（+1）、`crates/engine/src/paint/tests/r5296_inline_bg_ml_gate_tests.rs`（新 +152）。新 kill-switch `ZW_INLINE_BG_ML_GATE=0`（两门共用）。

## 案面

`css/CSS2/visudet/content-height-001`（CSS2 §10.6.2：inline 盒 content area 高 = 字体 A+D，与 line-height 无关）：三 inline-block div（line-height 200px/30px/normal）各包同款蓝底 `<span>aa</span>`，断言三段蓝色同高成矩形。

## 取证链（layout 层全对、缺口纯 paint）

1. **layout-dump 子命令与 reftest 管线分叉**：`zero-wpt-runner layout-dump`（div1 abs_y=50 基线对齐）与 reftest PNG（蓝到 y=0 盖 `<p>` 文本）渲染结果不一致——同一 build 两条渲染路径，需以 reftest 管线内 `LAYOUT_DUMP=1`/`REFTEST_DEBUG=1` 为准（reftest.rs:829）。
2. **布局正确性核验**：div 盒 [50,250]/[135,165]/[121,179] 符合 CSS inline-block 模型（T_i = B − half_leading_i − A 逐项吻合，C=42.8）；ZW_DEBUG_GLYPHS 六字形全基线 y=171.4 ✓。
3. **paint 缺口定谳**（REFTEST_DEBUG fills）：test fills = (8,**−28.6**,44.4,**200**)/(52.4,135,44.4,30)/(96.8,121,44.4,58)；ref fills 三条全 (x,−28.6,44.4,200)——背景高随 line-height 膨胀、span1 上溢 78.6px（= half-leading）盖 `<p>`，两页背景形状不一致 → ✗2.50%。
4. **路径归因**：span1 `content_height`(200) > 1.5×fs(75) 触发 R4297 box-level bg 抑制 + R639 fragment 门（owner_h 200 > 75）双命中 → 背景落 R639 旧 `frag.y` 位（= baseline−line_height，R1441 注释明示的「字形 run 顶」语义，仅 has_bleed 时校正）；span2/3（30/58 < 75）走 box-level 且 == 树盒 ✓。单行 span 被高度代理误判多行是唯一缺口。
5. **既有修复域核对**：R4383 内容区锚定只覆盖 provider/webfont 臂（系统字体 `line_metrics` 返 None）；R5171 paint 侧 `inline_ca_paint_override` 消费 `inline_reported_rect`（dormant 臂系统字体有记录）但仅收缩轴。修复只需把单行膨胀 span 送回 box-level（R5171 收缩臂自动生效），无需动扩张轴。

## 实现

- **判据升级**（三处成对）：`content_height > 1.5×fs` → `content_height > 1.5×max(fs, 自身 line-height 解析值, run 行高最大值)`。自身 line-height 取 computed style（Normal=1.2fs / Number / Length→resolve_length，text.rs:3886 选区臂同式）；run 行高最大值 = R4332 border 门输入（`text_node_line_heights` max）。
- **`inline_owner_max_lh` 新索引**：`collect_box_heights` 同趟收集 owner 盒的 run 行高最大值（text.rs `render_fragment!` 宏内 box_node 是 IFC owner 非 inline 本身；流内 span 自身 `text_node_line_heights` 为空——`store_font_sizes_from_ifc` 键在 IFC owner 盒，探针实证）。
- **成对不变量**：painter/mod.rs 两处 box-level bg 抑制门与 text.rs R639 fragment 门输入同源同值，共用 `ZW_INLINE_BG_ML_GATE`（构造期单次读取），box-level 抑制 ⇔ fragment 生效恒成立（R638 双绘教训）。
- R5270 selection 站 `inline_heights > 1.5×fs`（text.rs:3878）**不动**（selection 域独立校准 R5052/R5054/R5270）。

## 回归测试

- `r5296_single_line_inflated_lh_span_bg_box_level_content_anchored`：Ahem 100px + div inline-block lh 200px 单行 span。门开：恰 1 蓝矩形、h=100（dormant Ahem content 1.0fs）、w=200、收缩进树盒（⊆ 判据）。门关（`ZW_INLINE_BG_ML_GATE=0`，双相位单函数 env 竞态纪律）：h=200 行盒域、y 上溢出树盒（复现修复前）。
- `r5296_true_multiline_span_bg_still_per_fragment`：45px 窄容器 3 词折 3 行 → 逐行 3 条背景（真跨行仍 per-fragment，抑制不变量保持）。教训：初版 `XXXXXX` 单词不可断溢出不折行，改 `XX XX XX` 词间断行。

## 交付

- 目标案 ✗2.50→✓0.35（阈值 1% 内；残余 = span2/3 扩张轴差异带 ~0.14%——R5171 FIXME 扩张轴挂账不动）。
- 全量 corpus **16590/15732P/858F**，失败集 vs R5295 交付态（逐案同 R5294 859 集）归一化 diff **恰 {−content-height-001}**、零新增翻转。
- kill-switch 判别：`ZW_INLINE_BG_ML_GATE=0` 定向单案 → ✗2.50% 逐字节（11994px 同值）回修复前，OFF 轮归因成立。

## 门禁

- make reftest 716/716；engine lib **3538P(+2)**、layout-engine lib 2168P、style-system lib 2450P；fmt 0 diff；clippy（zero-engine + zero-layout-engine --all-targets）0 告警。
- product-smoke 6/6 struct PASS + welcome oracle 15.50% 持平；bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-engine` **GATE PASS**（19 指标 NEW=7）。首跑 `page/medium/first_paint_wall_ms` 367.48 超预算 362.35 达 1.4%——**复跑 311.47 PASS**（wall 时钟噪声；layout/paint/total 各轴全 PASS 无同向漂移）。

## 挂账

- R5171 **扩张轴**（line-height < A+D 时 inline bg 保持行盒域）——active-selection-027 A/B 校准域，须与 fragment/selection 站同切片校准（既有 FIXME）。
- R5270 selection 站同形判据（`inline_heights > 1.5×fs`）未动。
- R639 旧 `frag.y` 位（baseline−line_height）对**真跨行** bg-only 无 padding/border 的定位仍是近似（R1441/R1442 bleed 校正域之外）——本案修复不触碰真跨行路径。
- anonymous-boxes-001a（4.17%）：本轮 triage **SKIP**——R109 匿名块结构域（匿名片段内 inline 级子按 taffy block 子布局 = 位半），R4112/R4113 已修尺寸半（百分比高 penetration + 替换宽 ratio），R1982 probe（WITH-BLOCK-CHILD span height=100）成功信号在案；位半 = 伪盒实体化，spec-rfc gated。
- before-after-table-parts-001（14.97%）深结构挂账维持；跨流红移交维持（image_batch 37≠36 / page_scripts 滞留窗时序，对端流）。

## 教训

- **「高度 > 1.5×fs」式多行代理判据会被 line-height 膨胀欺骗**：行盒高随 line-height（含继承/strut）增长而行数不变——结构判据（行数）用几何代理（盒高）必须把「单行可膨胀到多高」的下界一并入阈。R4332 在 border 门已修同形缺口（bidi-003 判例），本轮补齐 bg 门 + fragment 门；selection 站仍有同形判据（独立校准域）。
- **同 build 双渲染路径取证须锚定消费管线**：`layout-dump` 子命令与 reftest 管线渲染可分叉，诊断以被比较的那条管线内建 dump（reftest.rs `LAYOUT_DUMP`/`REFTEST_DEBUG`）为准。
