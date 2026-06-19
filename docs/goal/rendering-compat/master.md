# 渲染兼容性目标 — 运行时控制面板

**最后更新**: 2026-06-20（R351：**plateau 首破——self-source 438→439/490（89.6%）**，首个 pass-rate 进展（R305 以来）。修复 `table-layout:fixed` 不尊重显式 width：CSS Tables §17.5.2.1 fixed 表 width 应由 width 属性决定，但 ZeroWeb 的 `compute_column_widths` 对所有表用内容固有列宽（col_max_widths），导致 fixed 表 `width:100px`+200px 内容渲染成 200px（内容撑宽）而非 100px（内容溢出 cell 由 overflow 裁剪）。根因 `table-cell-overflow-auto-scrolled`（2.17% FAIL）：cell 被撑到 200px，其 overflow:auto 无物可裁 → 与 100px div-ref 发散。**修复**（compute_column_widths）：fixed + 显式 Px width + 内容列宽和 > width 时按比例收缩列到 width（`fixed_capped` 标志），且仅当收缩实际触发时跳过「填满容器」扩展（未收缩的 fixed 表仍正常扩展——保 multicol-fill-001 reference 表）。验证：`table-cell-overflow-auto-scrolled` FAIL→PASS（0.00%）；**upstream reftest 438→439/490 零回归**（multicol-fill-001 + calc-percent-plus-0px-fixed + table_grid_size_col_colspan 三 fixed-layout 案全保）；make test 12265/0/72、clippy/fmt 干净、+1 单测 `test_fixed_layout_caps_columns_at_explicit_width_when_content_wider`。**启示**：plateau 并非绝对——逐案深查 self-source FAILURE（非 polluted）的近 misses 仍可发现单点 CSS 解析/布局 bug；table-layout:fixed width 尊重是 R305-R350 漏网的 clean win（仅 3 个 fixed-layout 案、爆炸半径小）。下一步：继续逐案审 self-source failure 近 misses（baseline-007/multicol-breaking-006/child-border-box-max-content 等 <1.5% 案）。

**前轮 R348**：fresh chromium-Oracle 复测确证 plateau 稳定（R324/R325/R326 零移动 polluted case chr diff；污染 48.0% vs R311 48.2%，逐 case 逐 chr% 稳定）。再前 R347 完成全仓库 2000 行达标（reftest.rs 拆出 resources.rs）。

**前轮 R345/R346**：R345 `paint/tests/visual.rs` 2056→1790（resize/scroll 测试尾）；R346 `inline/tests/advanced.rs` 2281→1948（float/tab 测试尾）。纯测试移动零回归。再前 R343/R344 完成生产源码 <2000（app_render.rs / gpu/renderer/mod.rs）。

**前轮 R342c**：2000 行规则收尾——`table.rs` 2694→1973 行拆分，抽出 `table_borders.rs`（740，resolve_collapsed_borders + BorderSource/resolve_border/边框颜色读取，CSS §17.6.2 collapsed 边框冲突解析集群）。**纯移动零行为变更**：reftest-upstream 438/490 字节级一致、**css-tables 51/55 不变**、888 layout 测试过、clippy 干净。详见 [`evidence/r342c-table-borders-split-2026-06-19.txt`](./evidence/r342c-table-borders-split-2026-06-19.txt)。

**当前活跃里程碑**: M10 — 上游 WPT 真实 Reftest 通过率（结构性 plateau，单会话杠杆已穷尽）/ DC-13 产品 smoke（证据已持久化 `evidence/product-static/`，残余为文本度量结构性）

**基线（R323 复验；strict post-R326 再复验仍零漂移，见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)）**：
- self-source loose **438/490 (89.4%)** @ 默认 1%/5% 容差
- self-source strict **295/490 (60.2%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径）
- chromium-Oracle 真一致率 **~35.6%**（self-source 含 48.6% 假通过，DC-14 anti-false-pass）
- 产品 smoke：welcome 17.06% / wintertc ~13.6% / morning-work fullpage 48.65%（全文本度量结构性，非图片/CSS 缺口——图片加载 R318 已端到端贯通）

**字体攻坚结论（2026-06-17 AA 基准）**：fontdue **Regular** 与 chromium 光栅化基本一致（W 0.1% / i 3.0%），**非渲染差异来源**；welcome 26% / Oracle 污染大头是**布局/度量**（line-height / R109 inline→block / 多行结构）。fontdue **Bold** 变体比 chromium 过墨 ~15%（R229b net-negative 已回退）。**字体攻坚停止，转布局/度量**——advance-width(R225/R320)、font-weight -Bold(R229b)、AA 噪声(R174) 三谱系均实证为死路，勿再投入。

> **结构化 plateau 结论见下方「综合裁决」节**（R305–R323 杠杆穷尽表 + 4 条多会话路径 + 需用户决策卡点）。逐轮详细记录见文末「最近轮次详细记录」（R310–R329）；更早轮次已归档：R309 → [`archive/rounds-r309.md`](./archive/rounds-r309.md)、R308 → [`archive/rounds-r308.md`](./archive/rounds-r308.md)、R307 → [`archive/rounds-r307.md`](./archive/rounds-r307.md)、R305–R306 → [`archive/rounds-r305-r306.md`](./archive/rounds-r305-r306.md)、R304 → [`archive/r304-taffy-upgrade-deferred.md`](./archive/r304-taffy-upgrade-deferred.md)、R303 → [`archive/r303-dc9-gpu-primitive-audit.md`](./archive/r303-dc9-gpu-primitive-audit.md)、R142–R302 → [`archive/rounds-r142-r302.md`](./archive/rounds-r142-r302.md)、R23–R139 → [`archive/rounds-r23-r139.md`](./archive/rounds-r23-r139.md)、R11–R20 → [`archive/rounds-r11-r20-reftest-investigation.md`](./archive/rounds-r11-r20-reftest-investigation.md)。


## 综合裁决：结构性 plateau（R305–R323，≥10 轮一致收敛）

> 本节为 doc-maintenance 轮（2026-06-19）对最近 ~20 轮的**浓缩结论**，置于控制面板顶部便于检索。逐轮详细记录见文末「最近轮次详细记录」（R310–R329）与归档 [`archive/rounds-r309.md`](./archive/rounds-r309.md)（R309）、[`archive/rounds-r308.md`](./archive/rounds-r308.md)（R308）、[`archive/rounds-r307.md`](./archive/rounds-r307.md)（R307）、[`archive/rounds-r305-r306.md`](./archive/rounds-r305-r306.md)（R305–R306）、[`archive/r304-taffy-upgrade-deferred.md`](./archive/r304-taffy-upgrade-deferred.md)（R304）、[`archive/r303-dc9-gpu-primitive-audit.md`](./archive/r303-dc9-gpu-primitive-audit.md)（R303）、[`archive/rounds-r142-r302.md`](./archive/rounds-r142-r302.md)（R142–R302）。

**核心结论**：rendering-compat 的**所有单会话 / 中会话 forward-motion 杠杆均已 ruled out 或 refuted**——这是 R313–R323（≥10 轮）一致收敛的结论，非单轮判断。rally 单会话迭代已无法提升真实通过率。

**基线（R323 复验；strict post-R326 再复验仍零漂移，见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)）**：

- self-source loose：**438/490 (89.4%)** @ 默认 1%/5% 容差
- self-source strict：**295/490 (60.2%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径）
- chromium-Oracle 真一致率：**~35.6%**（169/475；self-source 含 48.6% 假通过，DC-14 anti-false-pass）
- 产品 smoke：welcome 17.06% / wintertc 13.70% / morning-work 28.72%（全文本度量结构性，非图片/CSS 缺口——图片加载 R318 已端到端贯通）

> **post-R326 strict 再复验（2026-06-19 doc-maintenance read-only，test-guard 包裹 `ZERO_REFTEST_STRICT=1 ... reftest-upstream`）**：strict 仍 **295/490 (60.2%) / 195 fail**（zero drift vs R323）——确认 plateau 在 DC-14 诚实指标上成立：R324（position:fixed）/R325（img aspect）/R326（sticky）三处 DC-11 correctness 修复**均未**把任一 strict-fail 翻成 strict-pass（loose 亦经三 commit 各自复验 438/490 零回归）。详见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)。**顺带纠正**：旧文「295/490 (60.4%)」百分比过时（60.4% 为 R308 前 296/490 值；R308 font-size% 修复使 strict 296→295 后未同步百分比，295/490=60.2%）。

**已穷尽 / 证伪的杠杆（勿再以单会话重试）**：

| 杠杆 | 裁决轮 | 结论 |
|------|--------|------|
| near-pass clean-win frontier | R307 | 26 个 <0.2% 案全落结构性墙 / 字体噪声，零 clean win |
| POLLUTED 候选逐项 hunt | R299/R300/R302/R309/R311/R329 | 三趟复核（R298 清单逐项 + fresh 475 例 top-30 + 长尾 spot-check）全结构性/特性缺口，exhausted；唯 R308 font-size% 一处真实 clean win |
| fresh chromium-Oracle cross-validate | R311 | 4 新候选 ruled out，plateau 再确认 |
| Phase A IFC font_size 解锁（4 路） | R125/R198/R205/R206 | paint IFC vs layout IFC 两趟，font_size/line-height/换行耦合死锁；R207 narrow 精修仅获 font-051 +1 |
| multicol breaking paint 侧 | R157/R198/R203/R317 | 5 次实证全 net-negative，paint 侧死路 |
| multicol balance 二分搜索 | R199/R200/R321/R322 | T/N ≡ binary-search（等高行）；columns-001 diff 实为 wrapping 精度，非 balance |
| multicol column-aware IFC（layout 侧） | R319 | spec-rfc 产出 + A1 probe REFUTES Phase 1（目标结构在失败集几乎不存在，迁移零增益） |
| baseline-export（3 机制） | R266/R310/R312/R313/R316 | field-fill 净 0 / inline-flex 不受控 / flex-post-pass 回归，3 路全证伪 |
| advance-width plumbing | R225/R320 | 双角证伪（reftest-oracle 零变化 + Ahem advance 精确），死路 |
| DC-9 blend_mode | R278 | 单 framebuffer post-process 架构不可行，需 paint-isolation |
| font-weight -Bold 接线 | R229b | fontdue Bold 过墨 ~15%，net-negative |
| @font-face 自定义字体加载（css-fonts 聚类解锁假设） | R330 | 17 字体导入激活后实测：self-source 438/490 持平；chromium-Oracle 净负向（alternates-order +7.53 / font-features-across-space +0.95，余 54 不变，污染仍 ~46%）。根因 = rustybuzz TextShaper 未接入生产路径（生产逐字符 glyph_id=ch as u32），加载特性字体不应用特性更远离 chromium。fontdue/simple-shaping 第四条死路（同 R225/R229b/R174） |
| advance-width paint 单侧 fontdue | R225/R331 | layout(paint?) 双侧均用 estimate_char_width 一致（self-source 抵消）；paint 单侧改 fontdue 致与 layout 不一致 intra-fragment 错位；双侧改 = R225「三处同源」已证 oracle 26 案零变化。死路 |
| rustybuzz shaping 接入生产（单会话 bounded probe） | R331/R332 | R331 定位 4 子任务多会话；R332 实现全 4 子任务（high-bit 哨兵法避免 GlyphPrimitive 55 处构造点改动）+ SHAPE_PAINT 门控实测：self-source 438/490 持平；chromium-Oracle pollution 48.0%→48.3%（+2），零 case 改善。**paint-only shaping 净负向**——layout 仍用 estimate 断行，paint 单侧 shaping 致 layout/paint 不一致。须 layout+paint 同源（Phase A IFC 统一子集）。保留 TextShaper advance/offset bug 修复（真实修复），回退 probe 其余 |
| taffy 0.11 升级 | R304 | DEFER（541 ref + 108 alignment + native float 冲突，具名缺口零收益） |
| clear+margin-collapse（clear-float-003 等） | R333 | ZeroWeb 把 `clear` 仅 convert_clear→bool 传 taffy 0.7，无 ZeroWeb 侧 clearance 后处理 → clear+margin 折叠交互 = taffy-0.7-bound（R323 探针覆盖基本折叠不含 clearance）。须 taffy 升级或自建 clearance 后处理（与 taffy margin 折叠耦合，高风险） |
| CSS2/multicol 剩余失败（border-bottom-width-006 / border-padding-bleed / multicol-count-computed-003/004 等） | R333 | 新鲜审视 5 未深查用例全结构性：inline-box 模型（vertical-align/行盒绘制，与 Phase A 耦合）；multicol 列几何计算 spec-正确（divergence 在内容分布/规则定位，multicol 结构死锁）。DC-11 correctness 轴（R323-R326）确已穷尽 |
| WM-1 abs-pos-non-replaced-vrl/vlr（R237/R238「首选候选」从未执行） | R334/R335/R336 | R334 positioning（direction-rtl 镜像修复 spec-correct，0 clean win，latent 保留）→ R335 color（per-fragment 颜色 net-negative，paint-IFC 位置错）→ R336 suppression（精确定位：collect_inline_items 不检查 position 收 abspos 文本 + render_fragment! 标记 painted 抑制 abspos 自身 paint_text；refined skip 解抑制但 span 自身 paint-IFC 空 styles 产出 fs=16 = double-path，net-neutral 回退）。**三角度均收敛 Layout/Paint IFC 双路径（gap #4 = Phase A）**，WM-1 单会话 lever 彻底穷尽 |
| R348 #1 polluted case = backdrop-inherit-rendered（47.54%）= DOM/JS gap，非 CSS bug | R349 | **非渲染 bug**：该 case 测 `dialog::backdrop` 伪元素 + `<dialog showModal()>` + CSS var 继承；ZeroWeb 把 `dialog` 列为非渲染元素（style-system/lib.rs:73），无 `::backdrop` 伪元素 / showModal / modal 态支持。属 JS/DOM API 兼容，**out of rendering-compat scope**（goal 明确排除）。CSS `backdrop-filter`（apply_advanced.rs:807）是**另一回事**且已实现。**真 #1 CSS-rendering polluted case = abs-pos-border-offset-002（27.94%，writing-modes，Phase A）**。多数 top polluted 是纯 CSS 结构聚类（writing-modes/grid/fonts/flexbox-baseline/multicol），非 DOM/JS；backdrop 是孤例 |
| wintertc @media + escaped-class（Twind `sm\:`/`hover\:`）疑似缺口 | R350 | **probed 两路均 GREEN（work）**：(1) `@media(min-width:640px){.box{background:green}}` 在 800×800 视口产出 green（0,128,0）= @media 经 `evaluate_media_query`（matcher/mod.rs:997 + lib.rs:238 MediaContext::new(w,h)）正确按视口求值；(2) `.sm\:green` 选择器匹配元素 `class="sm:green"`（tokenizer `consume_escape` tokenizer.rs:385 正确消费 `\:`→`:`）；(3) 二者组合（@media + escaped class）也 green。**故 wintertc 残余 13.7% 非 @media/escaped bug**：per-band diff 集中在正文区（y=250-450，17-24%=systematic 文本度量 Phase A）+ logos 区（y=550-600，20.7%=flex-wrap/justify-evenly 精度 taffy R304 DEFERRED）。R318 README「Twind 类生效、无 clean bug」结论经 fresh 复核成立。勿再以 @media/escaped-class 嫌疑重查 |

**剩余 forward motion = 多会话架构承诺（非单会话），或接受 plateau**：

1. **Phase A IFC 三路径统一** — paint 不重跑 IFC，直接渲染 layout 存储的行盒（R205/R207 viable slice 已证 font-051 可行；broad 应用需多轮 narrow 精修 + 守 multicol-fill-auto 反向依赖墙）。设计文档 [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)。
2. **Phase 2 嵌套 multicol fragmentation** — layout 侧 column-aware IFC + 嵌套列碎片化（R131/R201；R319 证纯 inline 迁移零增益，真实价值在嵌套 / 混合内容）。设计文档 [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)、[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)。
3. **baseline-export 真修复** — taffy 0.8+ baseline_overrides（需先解 R304 升级冲突）或自建 inline-level-box baseline 合成；解 flexbox-baseline / multicol-baseline 聚类（~10+ 案）。
4. **接受 plateau** — 当前 self-source 89.4% / strict 60.2% / Oracle ~36% 作为诚实基线。

**裁决**：需用户对「投入多会话架构承诺」vs「接受 plateau」决策。继续 rally 单会话迭代将重复 plateau 确认，无新进展。R314 已通过飞书通知用户此卡点。

> **分支审计（2026-06-19 doc-maintenance read-only）**：未合并分支 `fix/rendering-compat-stacking`（8959ddb，2026-06-12，自称 R61 / 基线 387/490）经核查**冗余**——其核心改动（painter positioned/z-index 堆叠排序 CSS 2.1 App. E + `sync_inline_child_boxes_from_ifc`）**均已在 main**（`crates/engine/src/paint/painter/mod.rs:56-78`、`crates/layout-engine/src/engine.rs:1219`），且 main 版本更完整（额外处理 stacking-context 创建 + z-index:auto tree-order）。该分支**非**未合并 plateau 杠杆，亦非活跃并行开发（06-12 后无后续 commit）；doc-maintenance 续以 main HEAD 为准，不并入未合并分支内容。

---
## 里程碑完成状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| M1 — WPT Reftest 基础设施 | ✅ 完成 | 14/14 标准全部达成 |
| M2 — CSS 2.1 + Quirks Mode | ✅ 完成 | CSS parser + style system quirks 已实现；layout engine quirks 推迟到 M4 |
| M3 — Flexbox + Grid | ✅ 完成 | 179 个 reftest, 100.0% pass rate；Flexbox/Grid 无渲染缺口 |
| M4 — Float + Table + Multicol | ✅ 完成 | float + table + multicol 布局算法已实现；219 个 reftest, 100.0% pass |
| M5 — 文字排版 | ✅ 完成 | CJK 换行 + justify 修复 + float 堆叠修复 + 51 个 Text reftest |
| M6 — 全量扩展 | ✅ 完成 | 685 reftest, 13 目录全部 ≥50, 100.0% pass；unicode-bidi + CJK 换行已接入生产。⚠️ **rustybuzz TextShaper（GSUB/GPOS）已实现+单元测试，但未接入生产 paint/layout 路径**（R330 代码核查实证：生产 text.rs:1057 仍逐字符 `glyph_id=ch as u32`，TextShaper 仅 lib.rs 单测调用）→ ligature/kerning/alternates 生产未生效 |
| M7 — 渲染器图元覆盖 | ✅ 完成（管线层）⚠️ | CPU 渲染器：全部 13 种图元 ✅；GPU 渲染器：13 种图元**管线**已建（48 单元测试 ✅），**但浏览器全量 GPU 路径 `render_full_scene_gpu` 实际消费以 DC-9 表为准**——transform=✅ R285/ee8373a 已接入（`collect_transforms`+`apply_transform_filters_headless`）、blend=CPU no-op stub+GPU 丢弃（**唯一剩余 GPU 真实缺口**）、5 color-matrix 滤镜（grayscale/invert/saturate/sepia/hue-rotate）=✅ R286/94c773a 已落（`collect_color_filters` mode 3-7 全处理，parity CPU）、clip=no-op（engine 从不生成）；filter:opacity/brightness/contrast/blur 已落（f6fed44/fc86937/3a3530f）；浏览器消费：全部 13 种图元 ✅ |
| M8 — 布局正确性 | ✅ 完成 | BFC 检测 ✅；float clear ✅；margin 折叠(taffy 0.7 内置) ✅；<img> 固有尺寸 ✅；position:fixed ✅(adjust_fixed_to_viewport)；position:sticky 需宿主层（已标记 is_sticky，后续集成）；percentage height/auto margin/min-max-width 已有测试验证 |
| M9 — 高级视觉效果 | 🔧 进行中 | 重复渐变 ✅；多图层背景 ✅；clip-path 全形状裁剪 ✅(inset+circle+ellipse+polygon)；border-image ✅；text-shadow ✅；backdrop-filter ✅；CSS mask ✅(渐变蒙版裁剪+alpha衰减)；overflow 全图元裁剪 ✅；滚动容器 paint 偏移 ✅(scroll_x/scroll_y 字段 + paint 时子元素坐标偏移 + 3 个单元测试)；剩余：scroll-snap 行为（需宿主层输入路由）、滚动输入路由（需浏览器 app 集成） |
| M10 — 上游 WPT 真实 Reftest 导入 | ⏸ plateau（R323） | 基础设施 ✅；490 上游 reftest 已导入（9 目录）；self-source loose **438/490 (89.4%)** / strict **295/490 (60.2%)** / chromium-Oracle ~35.6%；R305–R323 全单会话杠杆穷尽，达标需多会话架构（见「综合裁决」） |

## 当前状态概览

| 维度 | 状态 | 说明 |
|------|------|------|
| 渲染管线 | ⚠️ 全链路贯通但非一致 | HTML→CSS→Style→Layout→Paint→Composite 可运行；但 layout IFC、paint IFC 和 ZeroBrowser glyph 消费仍存在多套坐标/度量路径，`welcome.html` 已暴露用户可见错位 |
| WPT Runner | ✅ reftest 级 | 1,341 个手写 TestCase + 685 个内联 reftest（13 目录 ≥50） |
| Reftest Harness | ✅ 可用 | 分类容差、per-test fuzzy 注解、match/mismatch 模式 |
| Manifest Parser | ✅ 扩展完成 | reftest 条目解析、fuzzy 元数据、HTML 链接提取 |
| CPU 软件渲染 | ✅ 全量图元 | render_full_scene() 支持全部 13 种图元（fills, rounded_rects, gradients, shadows, images, strokes, path_fills, path_strokes, glyphs, clips, transforms, filters, blend_modes） |
| Reftest CLI | ✅ 可用 | `cargo run --bin zero-wpt-runner -- reftest` |
| Skip List | ✅ 已创建 | `tests/wpt-runner/reftest-skip-list.txt` |
| Chromium 截图脚本 | ✅ 已创建 | `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` |
| WPT 导入脚本 | ✅ 已创建 | `tests/wpt-runner/scripts/import-wpt-reftests.sh` |
| 内联 reftest | ✅ 685 个 | 13 个目录全部 ≥50，覆盖 CSS 2.1、Flexbox、Grid、Position、Display、Box、Float、Table、Multicol、Text、Fonts、Text-decor、Writing-modes |
| JS 执行 | ✅ 已集成 | reftest harness 通过 V8 sandbox 在渲染前执行 JS（不修改 DOM） |
| GPU 渲染截图 | ✅ 已验证 | GpuRenderer headless + read_pixels()；685/685 reftest GPU 模式 100.0% pass |
| GPU 渲染器图元 | ✅ 全量 | 全部 13 种图元管线 + 48 个单元测试 + 浏览器 GPU 路径集成 |
| CI 集成 | ✅ 已接入 | GitHub Actions reftest job（CPU 渲染） |
| Quirks Mode | ✅ 完成 | CSS parser + style system + layout engine quirks 全部实现 |
| 外部 stylesheet 加载 | ✅ 已贯通 | R213 落地：URL 导航路径 `fetch_url()` 现抓取 `<link rel="stylesheet">`（extract_stylesheet_hrefs → base URL 解析 → http_client 抓取 → 合并级联），三条 fetch_url 分支注入；离线 fixture HTTP server（R212）支撑测试 |
| 图片子资源/ImageCache | ✅ 已贯通 | R214（PNG 抓取+解码+image_cache）→ R215（浏览器 render_cpu/render_frame 消费 webview ImageCache，最后消费 hop）→ R216（JPEG）→ R218（SVG 栅格化统一到 render-foundation decode_image_bytes）。`<img>` 经 URL 导航全链路 fetch→decode→image_cache→browser render→真像素贯通（DC-13 P1 闭环） |
| 产品/真实静态页面视觉 smoke | 🔧 证据已持久化·持续修复 | welcome/morning.work/wintertc fixture + product-smoke + chromium Oracle 工具链就绪；**证据已持久化 `evidence/product-static/`**（3 fixture × {ZeroWeb-CPU/chromium PNG + README 含 diff%/根因}，满足 DC-13 line 305，R281 审计）；当前 diff：welcome 17.06%（R227 padding 双计 28→17）、wintertc 13.59%（R227+R255 后 2026-06-18 复测 25→13.59）、morning-work fullpage 48.65%（R255 ua_default_display 修 4× 高度幻影盒 89.14%→48.65%）；残余 diff = item-tag span→block R109 IFC（结构性）+ fontdue CJK 度量 + hljs（需 JS），非证据缺口 |
| #[ignore] 测试 | ⚠️ 保留 | 59 个真实网站测试保留 #[ignore]，因本地网络不稳定。其余零 #[ignore] |

---

## Done Criteria 进度

### DC-1: WPT Reftest 基础设施就位

| 条目 | 状态 | 说明 |
|------|------|------|
| fetch 上游 WPT 仓库 | ⚠️ | 导入脚本已创建，内联 reftest 替代上游导入 |
| 解析 fuzzy() 元数据 | ✅ | manifest.rs 已扩展 |
| CPU 渲染截图 | ✅ | render_scene_to_framebuffer() 可用 |
| GPU 渲染截图 | ✅ | GpuRenderer headless + CPU 圆角叠加 |
| Chromium 参考截图 | ✅ | Puppeteer 脚本已创建（capture-chromium-screenshots.mjs） |
| Viewport 对齐 | ✅ | ReftestConfig 有 viewport 字段 + CLI --width/--height |
| JS 执行集成 | ✅ | V8 sandbox 在渲染前执行 JS（不修改 DOM） |
| 分类容差机制 | ✅ | ReftestCategory (Layout/Text/Unknown) + per-test fuzzy override |
| 范围外过滤 | ✅ | reftest-skip-list.txt 已创建 |
| 通过率报告 | ✅ | 文本 + JSON 格式，按分类输出 |
| 单一命令运行 | ✅ | `cargo run --bin zero-wpt-runner -- reftest` |
| CI 集成 | ✅ | GitHub Actions reftest job |

### DC-2: CSS 2.1 核心通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283，2026-06-18）**：下表原「通过率 ≥95% ✅ 100.0%」基于**内联 685 reftest**，直接违反 DC-14（goal line 319「内联 reftest 100% 仅作 smoke，不计达标」+ line 844「禁止 DC-2~5 以内联 100% 冒充达标」= DONE 阻断项）。**真实达标**须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差 0.1%/0.5%），当前诚实数 = **39.6% strict**（188/475，evidence/cross-validate-full-2026-06-17.txt）/ 89.4% self-source-loose（438/490 @ 1%/5%），**均 <95%，DC-2 未达标**。内联 100% 仅 smoke（DC-7 全绿基线）。

| 条目 | 状态 | 说明 |
|------|------|------|
| 导入 reftest 子集 ≥ 50 | ✅（smoke） | 179 个内联 CSS 2.1 核心 reftest（**不计达标分母**，DC-14 line 323） |
| 通过率 ≥ 95% | ❌ 未达标 | 内联 smoke 100%（179/179）不计达标；真实上游全量+chromium Oracle+严格容差 = 39.6% strict，未达 95% |
| CPU 模式达标 | ❌ 未达标 | 同上（reftest harness 走 CPU 路径，容差 10× 过松 R280，reference 同源自渲染） |
| GPU 模式达标 | ❌ 未达标 | GpuRenderer headless 可用（机制就绪），但真实通过率未达标 + 容差过松 |

### DC-3: Flexbox + Grid 通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原「Flexbox/Grid 通过率 ✅ 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| Flexbox reftest 子集 | ✅（smoke） | 51 个内联 Flexbox reftest（基础+进阶+边界+M6 扩展，**不计达标分母**） |
| Flexbox 通过率 | ❌ 未达标 | 内联 smoke 100%（51/51）不计达标；真实上游全量+chromium Oracle+严格容差未达 95% |
| Grid reftest 子集 | ✅（smoke） | 51 个内联 Grid reftest（基础+进阶+边界+M6 扩展，**不计达标分母**） |
| Grid 通过率 | ❌ 未达标 | 同 Flexbox，内联 smoke 不计达标，真实未达 95% |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280 + 同源 reference，真实通过率未达标 |

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原「各项通过率 ✅ 全部 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达；且 multicol/table 含已知结构性死锁（multicol-breaking R131、table colspan R177b 部分修），真实 sub-领域通过率更低。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| Positioning reftest | ✅（smoke） | 50 个定位 reftest（基础+进阶+M6 扩展，**不计达标分母**） |
| Float reftest | ✅（smoke） | 50 个 float 布局 reftest（M6 扩展，**不计达标分母**） |
| Table reftest | ✅（smoke） | 50 个 table 布局 reftest（M6 扩展，**不计达标分母**） |
| Multicol reftest | ✅（smoke） | 50 个 multicol 布局 reftest（M6 扩展，**不计达标分母**） |
| 各项通过率 | ❌ 未达标 | 内联 smoke 100% 不计达标；真实上游全量+chromium Oracle+严格容差未达 95%（multicol/table 结构性死锁更低） |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280 + 同源 reference，真实通过率未达标 |

### DC-5: 文字排版通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原各目录「通过率 ✅ 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达；且文字类容差 5%（R280）更过松，fontdue CJK 度量/line-height 噪声（R174/R187/R229b）是文字类残余 diff 大头。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| css-text/ reftest ≥ 50 | ✅（smoke） | 51 个（**不计达标分母**） |
| css-text/ 通过率 | ❌ 未达标 | 内联 smoke 100% 不计达标；真实上游全量+chromium Oracle+严格容差未达 95% |
| css-fonts/ reftest ≥ 50 | ✅（smoke） | 50 个（**不计达标分母**） |
| css-fonts/ 通过率 | ❌ 未达标 | 同上（fontdue 度量噪声是残余 diff 大头） |
| css-text-decor/ reftest ≥ 50 | ✅（smoke） | 50 个（**不计达标分母**） |
| css-text-decor/ 通过率 | ❌ 未达标 | 同上（text-emphasis 等未实现 R232） |
| css-writing-modes/ reftest ≥ 50 | ✅（smoke） | 50 个（**不计达标分母**） |
| css-writing-modes/ 通过率 | ❌ 未达标 | 同上（vertical-rl clearance R114/R164 死锁） |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280（文字类 5%）+ 同源 reference，真实通过率未达标 |

### DC-6: Quirks Mode

| 条目 | 状态 | 说明 |
|------|------|------|
| CSS parser quirks | ✅ | 已实现：quirky color values（hashless hex + numeric colors）、unitless lengths（裸数字视为 px） |
| Style system quirks | ✅ | 已实现：percentage-height quirk、table height quirk（height → min-height）、inline width/height quirk 注释 |
| Layout engine quirks | ✅ | table/float layout 已在 M4 实现，quirks mode 通过 UA 默认 display 值和 table height quirk 生效 |
| DOM → style 链路传递 | ✅ | Document::quirks_mode() → tag_name 提取 → apply_quirks_mode_adjustments |

### DC-7: 测试与质量

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 零失败 | ✅ | 全部通过（59 个真实网站测试保留 #[ignore]） |
| 零 #[ignore] 测试 | ✅ | 仅 real_website_compat.rs 有 59 个 #[ignore] |
| 新修复有单元测试 | ✅ | quirks mode 颜色/长度/样式系统各新增单元测试 |
| cargo clippy 零警告 | ✅ | `cargo clippy -- -D warnings` 通过 |
| Reftest 报告持久化 | ✅ | evidence/reftest-report-2026-06-06.json/txt |
| 历史记录可追溯 | ✅ | 首份报告已持久化 |

### DC-8: CPU 渲染器图元覆盖（全部 13 种）

| 条目 | 状态 | 说明 |
|------|------|------|
| FillPrimitive | ✅ | 填充矩形（原有） |
| RoundedRectPrimitive | ✅ | 圆角矩形（原有） |
| GlyphPrimitive | ✅ | 文字渲染（原有） |
| GradientPrimitive | ✅ | 线性/径向/锥形渐变，逐像素插值 |
| ShadowPrimitive | ✅ | box-blur 近似阴影，含 blur_radius/spread_radius |
| ImagePrimitive | ✅ | RGBA 像素数据合成到 framebuffer |
| StrokePrimitive | ✅ | solid/dashed/dotted 线段 + LineCap |
| PathFillPrimitive | ✅ | 多边形扫描线填充 |
| PathStrokePrimitive | ✅ | 多边形描边 |
| TransformPrimitive | ✅ | 仿射变换后处理 |
| ClipPrimitive | ✅ | 矩形裁剪（像素级 discard） |
| FilterPrimitive | ✅ | blur + opacity + brightness/contrast/grayscale/invert/saturate/sepia/hue-rotate（apply_filter 全 8 color-matrix + blur，effects.rs；drop-shadow 仍 stub） |
| BlendModePrimitive | ⚠️ stub | draw_order 派发（cpu/mod.rs:250）但 `apply_blend_mode`（effects.rs:331-348）是 **no-op stub**（算 rect 后不应用，注释自承需 source+dest 双图层）——同 DC-9 blend（R278/R282 证），paint 生成但 CPU 不真混合。生产 footprint 低 |
| render_full_scene() 入口 | ✅ | 新函数，CSS painting order 渲染全部 13 种图元 |

### DC-9: GPU 渲染器图元覆盖

> **状态（R277 只读复核；2026-06-19 doc-maintenance 复核 committed HEAD b75035b 纠正：R285 transform / R286 5 color-matrix 滤镜均已落，原「transform WIP / 5 滤镜 GPU 丢弃」两项 stale）**：较 R211（2026-06-17 标 transform/clip/filter/blend 全 ⚠️「丢弃」）有实质推进。filter:opacity（f6fed44）/brightness/contrast（fc86937）/blur（3a3530f）已落地为独立 WGSL 后处理管线（ping-pong 区域读写，`render_full_scene_gpu` 经 `apply_color_filters_headless`/`apply_blur_filters_headless` 消费，非 passthrough，满足 DC-14）。clip 经 R220 实证为 no-op——engine 生产路径**从不生成** ClipPrimitive（`add_clip` 0 处非测试调用），overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect` 预烘焙进图元几何，CPU/GPU 两路均空谈满足，**非真实缺口**。**真实剩余缺口 1 类（仅 blend_mode）**：(a) ~~transform~~ **✅ R285/ee8373a 已落**——`fs_transform`（逆变换重采样）+ `create_transform_pipeline` 已接入 `render_full_scene_gpu`（`collect_transforms`+`apply_transform_filters_headless`，guard 非空；单测 `test_gpu_full_scene_transform_translation`，匹配 CPU `apply_transform_post` clear-to-white 语义）；(b) **blend_mode**——paint 在 `painter/effects.rs:313` 生成 `BlendModePrimitive`，但 **CPU `apply_blend_mode`（effects.rs:331-348）是 no-op stub**（算 rect 后 `_=(left,top,right,bottom)` 仅消未用警告），GPU `render_full_scene_gpu` 同样不消费，需 source+dest 双图层新机制（R269 标记为比 opacity 大的独立特性，低 reftest footprint）；(c) ~~5 color-matrix 滤镜~~ **✅ R286/94c773a 已落**——`collect_color_filters`（renderer/mod.rs:2062）现处理全 8 mode（Opacity0/Brightness1/Contrast2/Grayscale3/HueRotate4/Invert5/Saturate6/Sepia7），parity CPU `apply_filter`，单测 `test_gpu_full_scene_filter_grayscale`/`_hue_rotate`/`_invert`。drop-shadow（CPU 亦 stub，GPU 同）仍 `_ => None` 丢弃。reftest harness 与 product-smoke 均走 CPU 路径，GPU 缺口不污染测量数字，仅影响浏览器 GPU 渲染模式。

| 条目 | 状态 | 说明 |
|------|------|------|
| FillPrimitive | ✅ | GPU 填充（原有） |
| GlyphPrimitive | ✅ | GPU 文字渲染（原有，atlas） |
| RoundedRectPrimitive | ✅ | GPU 片段着色器（WGSL corner discard） |
| GradientPrimitive | ✅ | GPU 渐变 shader（线性/径向/锥形 + 1D 渐变纹理） |
| ShadowPrimitive | ✅ | 半透明填充矩形（简化，不做 GPU blur） |
| ImagePrimitive | ✅ | GPU 纹理上传 + 采样（RGBA→texture→shader） |
| StrokePrimitive | ✅ | CPU 侧顶点生成 + GPU fill pipeline（solid/dashed/dotted） |
| PathFillPrimitive | ✅ | CPU 侧扇形三角化 + GPU fill pipeline |
| PathStrokePrimitive | ✅ | CPU 侧分解为粗线段 + GPU fill pipeline |
| TransformPrimitive | ✅ | R285（ee8373a）独立 WGSL `fs_transform`（逆变换重采样，匹配 CPU `apply_transform_post` clear-to-white 语义）+ `create_transform_pipeline`，**已接入** `render_full_scene_gpu`（`collect_transforms` + `apply_transform_filters_headless`，guard `!empty && headless_texture.is_some()`）；单测 `test_gpu_full_scene_transform_translation` |
| ClipPrimitive | ⚪ no-op | engine 生产路径**从不生成** ClipPrimitive（R220 实证），overflow 裁剪预烘焙进图元几何。CPU/GPU 两路均空谈满足，**非真实缺口** |
| FilterPrimitive | ✅（drop-shadow 除外） | **全 8 color-matrix + blur 已落（独立 WGSL ping-pong 后处理，非 passthrough）**：opacity（fs_color_filter mode0）/brightness（mode1）/contrast（mode2，fc86937）/grayscale（mode3）/hue-rotate（mode4）/invert（mode5）/saturate（mode6）/sepia（mode7，R286/94c773a，parity CPU `apply_filter`）/blur（fs_blur 三角核 2-pass，3a3530f）。`collect_color_filters`（mod.rs:2062）现处理全 8 mode（原「mode 0/1/2 其余丢弃」已过时）。**仍未落**：drop-shadow（CPU 亦 stub，GPU `_ => None`） |
| BlendModePrimitive | ❌ 丢弃 | paint 生成（effects.rs:313）但 CPU `apply_blend_mode`=no-op stub（effects.rs:331-348）+ GPU `render_full_scene_gpu` 不消费。**单 framebuffer post-process 架构上不可行**（R278 实证：apply 时元素子树已与 backdrop 合并进 framebuffer、不可分离，区别于 opacity/blur 的合法区域近似）→ 需 **paint-isolation 架构**（元素子树隔离渲染到 offscreen + source/dest 双纹理 blend 合成 pass）；render-foundation 现无 per-element staging buffer、paint 无 isolation group，**multi-round 架构 defer**。footprint ~2-4 case，非 lever |

> **DC-9/DC-14 parity caveat（R277）**：覆盖满足 ≠ CPU 像素 parity——(1) opacity=GPU RGB-darken 近似（R272，post-process 无法恢复背景）；(2) blur=GPU 三角核 separable 2-pass vs CPU 多遍 box（R277，算法分歧，非 ==CPU，见 `evidence/r277-dc9-gpu-blur-vs-cpu-boxblur-parity-2026-06-18.txt`）；(3) brightness/contrast=精确 parity（R273 正确 CSS 语义）。三者覆盖均达标（独立 WGSL 非丢弃），但 opacity/blur 属「覆盖达标非像素对齐」类。

### DC-10: 浏览器图元消费

| 条目 | 状态 | 说明 |
|------|------|------|
| transform_webview_primitives() 全 13 种 | ✅ | 新函数处理所有 RenderPrimitives 字段 |
| render_cpu() 使用 render_full_scene() | ✅ | 完整图元渲染替代旧版 3 种入口 |
| scale_factor 应用 | ✅ | 所有图元类型正确缩放 |
| offset 应用 | ✅ | 所有图元类型正确偏移 |
| clip_y 视口裁剪 | ✅ | fills + glyphs 应用 clip_y 裁剪 |
| CSS painting order | ✅ | shadows → backgrounds → borders → content → overlay → filters → blend_modes |

### DC-11: M7 验证

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 全绿 | ✅ | 7800+ 测试全部通过 |
| cargo clippy 零警告 | ✅ | `cargo clippy -- -D warnings` 通过 |
| 新增图元单元测试 | ✅ | 渐变/阴影/图片/线段/路径填充/路径描边/变换/裁剪/滤镜/混合模式各有独立测试 |

---

## M1 里程碑详情

**目标**: 建立能够导入、运行、对比和报告 WPT reftest 的完整基础设施。

### M1 完成标准 (14 项)

1. ✅ fetch 上游 WPT 仓库（导入脚本 + 内联 reftest 替代）
2. ✅ 扩展 manifest.rs 解析 fuzzy() 元数据
3. ✅ CPU 软件渲染截图（render_scene_to_framebuffer）
4. ✅ GPU 渲染截图（GpuRenderer headless + CPU 圆角叠加）
5. ✅ 自动化 Chromium 截图工具（Puppeteer 脚本）
6. ✅ Viewport 对齐机制
7. ✅ JS 执行集成（V8 sandbox 执行 script 标签中的 JS）
8. ✅ 分类容差机制
9. ✅ 范围外 reftest 过滤 (skip list)
10. ✅ 按目录分类通过率报告（文本 + JSON）
11. ✅ 单一命令运行全部 reftest
12. ✅ 导入 CSS 2.1 核心 ≥ 50 个 reftest（115 个）
13. ✅ 记录初始通过率（100.0% 113/113）
14. ✅ 确认 #[ignore] 标记状态

### M1 已完成的基础设施

| 组件 | 文件 | 说明 |
|------|------|------|
| Manifest 解析 | `tests/wpt-runner/src/manifest.rs` | reftest 条目、fuzzy 元数据、HTML 链接提取 |
| Reftest 引擎 | `tests/wpt-runner/src/reftest.rs` | 分类容差、fuzzy 覆盖、match/mismatch 比较 |
| Reftest 数据 | `tests/wpt-runner/src/reftest_data.rs` | 159 个 CSS 2.1 核心 + Flexbox/Grid 内联 reftest |
| Reftest CLI | `tests/wpt-runner/src/main.rs` | `reftest` 子命令 + 文本/JSON 报告 |
| Skip List | `tests/wpt-runner/reftest-skip-list.txt` | SVG/Canvas/WebGL/动画过滤规则 |
| Chromium 工具 | `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` | Puppeteer headless 截图 |
| 导入脚本 | `tests/wpt-runner/scripts/import-wpt-reftests.sh` | 上游 WPT reftest 批量导入 |

---

## 初始 Reftest 通过率数据

**日期**: 2026-06-07（M6 DC-5 全目录达标）
**总用例**: 685（内联 reftest）
**运行用例**: 685
**通过**: 685
**失败**: 0
**通过率**: 100.0%
**渲染模式**: CPU 软件渲染
**视口**: 800×600

### 按分类

| 分类 | 通过/总数 | 通过率 |
|------|-----------|--------|
| Layout | 484/484 | 100.0% |
| Text | 201/201 | 100.0% |

### 按 WPT 目录

| 目录 | 数量 | 通过率 | ≥50 达标 |
|------|------|--------|----------|
| css21/ | 78 | 100.0% | ✅ |
| css-box/ | 54 | 100.0% | ✅ |
| css-text/ | 51 | 100.0% | ✅ |
| css-grid/ | 51 | 100.0% | ✅ |
| css-flexbox/ | 51 | 100.0% | ✅ |
| css-fonts/ | 50 | 100.0% | ✅ |
| css-position/ | 50 | 100.0% | ✅ |
| css-display/ | 50 | 100.0% | ✅ |
| css-text-decor/ | 50 | 100.0% | ✅ |
| css-writing-modes/ | 50 | 100.0% | ✅ |
| css-multicol/ | 50 | 100.0% | ✅ |
| css-float/ | 50 | 100.0% | ✅ |
| css-table/ | 50 | 100.0% | ✅ |

### 覆盖范围

- 颜色 (5): 命名色 vs hex, 命名色 vs rgb, 不同颜色 mismatch
- 背景 (5): 多色背景, 百分比尺寸, 不同背景 mismatch
- 边框 (10): 等价边框声明, 不同边框颜色 mismatch, 边框方向, solid 等价, 不同边宽度, padding+border, box-sizing
- 盒模型基础 (5): margin, padding, 等价盒模型, 不同 padding mismatch
- 定位基础 (5): absolute, relative, 不同定位 mismatch, bottom/right
- 显示基础 (5): display:none, display:block, visibility, 显示隐藏 mismatch
- 尺寸 (5): 固定尺寸, 百分比尺寸, 不同尺寸 mismatch
- Flexbox 基础 (10): row, column, row-vs-block, grow, wrap, justify, align, gap, nested, basis
- Flexbox 进阶 (10): grow-proportional, grow-with-base, wrap-multi-line, align-center, justify-space-between, shrink-overflow, column-direction, gap-between-items, order-reorder, basis-0-grow
- Flexbox 边界 case (10): align-self-flex-end, flex-basis-auto-with-width, nowrap-overflow, justify-flex-end, justify-center, wrap-reverse, shrink-ratio, min-width-constraint, max-width-constraint, nested-flex-wrap
- Grid 基础 (10): 固定列, fr, 2x2, gap, auto-rows, mixed-fr-px, vs-block, 三列, row/col gap, nested
- Grid 进阶 (11): fr-unit-proportional, mixed-fr-px-proportional, auto-placement-3x2, gap-rows-columns, nested-grid-in-flex, minmax-column, repeat-auto-fill, grid-in-grid, justify-items-stretch, flex-in-grid-item, shorthand-gap
- Grid 边界 case (10): auto-rows-minmax, justify-content-center, align-content-center, implicit-rows, place-items-center, grid-auto-columns, named-grid-area-simple, fr-with-percentage, empty-tracks, percentage-track-sizing
- 定位进阶 (15): absolute-top-left, shift-mismatch, relative-offset, vs-no-position, in-flow, bottom-right, stacking, z-index, overlap-mismatch, multiple-relatives, absolute-in-relative, absolute-right-bottom, relative-offset-no-layout, z-index-stacking-order, absolute-overlaps-static
- 文本排版 (10): 颜色, align, whitespace, line-height, letter-spacing, word-spacing, text-indent, transform, flex-container, vs-background
- 盒模型进阶 (10): margin-collapse, box-sizing, border-colors, overflow-hidden, overflow-visible, max-width, min-height, percentage-width, auto-margin-center, negative-margin
- 显示进阶 (10): none-removes-layout, inline-block, visibility-hidden, nested-inline-block, none-vs-visible, flex-item-none, grid-item-none, nested-flex-grid, block-100pct, body-background
- 嵌套/复杂 (5): 三层嵌套, 不同内部尺寸 mismatch, 兄弟排序, float 布局
- Overflow (5): hidden clips, visible no-clip, hidden vs visible mismatch, nested overflow, overflow with margin child
- Margin 折叠 (5): sibling collapse, parent-child collapse, BFC no-collapse, auto center, body reset
- Quirks mode (5): hashless color, numeric color, unitless width, unitless padding, table height as min-height
- Table 布局 (9): basic-2col, basic-3col, multi-row, with-tbody, auto-width-equal-cols, row-tallest-cell, thead-tbody-tfoot, th-td-mixed, single-column
- Multi-column 布局 (10): column-count-2, column-count-3, column-width-auto, column-gap, columns-shorthand, balanced-4-children, uneven-heights, with-column-rule, mismatch-column-count, no-columns
- 文字排版 (51): text-align (justify/center/right/multiline), word-spacing (normal/large), text-decoration (underline/overline/line-through/dashed), text-transform (uppercase/lowercase/capitalize/none), white-space (pre/pre-wrap/pre-line/nowrap), line-height (double/tight/mismatch), font-size (large/mismatch), text-color (green), text-indent (50px/percent), letter-spacing (4px/2px), word-break (break-all/keep-all), overflow-wrap (break-word/long-url), CJK (line-break/mixed-wrap), tab-size, text-in-flex, text-in-grid, vertical-align (top/middle), 颜色/align/whitespace/line-height/letter-spacing/word-spacing/text-indent/transform/flex-container/vs-background (15 个 css21 基础)

---

## 上游真实 WPT Reftest 通过率

> 早期上游 reftest 调查（R11–R20，2026-06-09/10，self-source 基线 74.7%）已归档至 [`archive/rounds-r11-r20-reftest-investigation.md`](./archive/rounds-r11-r20-reftest-investigation.md)。

**当前基线（R323 复验；strict post-R326 再复验仍零漂移，见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)）**：

- self-source loose **438/490 (89.4%)** @ 默认 1%/5% 容差
- self-source strict **295/490 (60.2%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径）
- chromium-Oracle 真一致率 **~35.6%**（self-source 含 48.6% 假通过）

完整 plateau 分析、已穷尽杠杆表、4 条多会话路径见顶部「综合裁决」节；逐目录 chromium-Oracle 污染分布见 `evidence/cross-validate-full-2026-06-18.txt`（flexbox 26% 污染最诚实，writing-modes 73% 最高）。达标需多会话架构，单会话杠杆已穷尽。

---

## 已知关键缺口（当前活跃）

> 下表仅列**尚未解决**的缺口，与 R305–R323 plateau 框架对齐（剩余 forward motion 全部需多会话架构承诺，见顶部「综合裁决」）。**已完成项**（Float/Table/Multicol 布局算法、OpenType shaping、BiDi、CJK 换行、justify、quirks mode、CPU/GPU/浏览器图元覆盖、外部 stylesheet、图片子资源/ImageCache、margin 折叠、BFC 检测+margin 隔离、`<img>` intrinsic sizing + object-fit）见「里程碑完成状态」「当前状态概览」，不再在此重复列。

| 缺口 | 影响范围 | 优先级 | 解锁路径（均为多会话架构） |
|------|----------|--------|------|
| **Paint IFC / taffy-IFC 架构分裂** | large-font（ifc-008/009/011）+ welcome/morning 文本度量残余（self-source 失败主因） | **P0** | Phase A IFC 统一：baseline-resolved 单一权威行盒（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)；墙②③ R125–R213 六轮死锁 + R306 几何基线证伪） |
| Multicol column breaking / 嵌套碎片化 | css-multicol 失败聚类（结构性） | P1 | Phase 2 嵌套 multicol fragmentation（layout 侧 column-aware IFC；paint 侧 R157/R198/R203/R122/R317 五轮证 net-negative 死路） |
| Multicol / flexbox baseline-export | baseline-000~008 + flexbox-baseline（~10+ 案） | P1 | taffy 0.8+ `baseline_overrides`（R304 DEFER 升级）或自建 inline-level-box baseline 合成（R266/R313/R316 三机制穷尽，须 layout 侧注入） |
| Writing-mode 垂直布局 | css-writing-modes 垂直 float/clearance 轴 | P1 | 精细轴交换（R57/R114/R164 谱系，clearance vertical-axis） |
| Inline-box 模型 | CSS2 linebox（vertical-align/行盒高度） | P1 | 与 Phase A IFC 统一耦合（v_offset/baseline 语义分歧） |
| DC-9 blend_mode（mix-blend-mode） | GPU backdrop 合成（~2-4 reftest 案，近零覆盖） | P2 | paint-isolation 架构（offscreen 子树渲染 + source/dest 双纹理 blend pass，R278 defer） |
| position: sticky | 滚动吸附（需宿主层输入路由） | P2 | host-runtime 层 sticky 偏移驱动；**R326 实测**：converter 把 sticky 映射为 taffy Relative，block-level 偏移已被 taffy 应用（scroll-0 应吸附场景 delta==inset）。缺的是 scrollport 相对钳制（normal 位满足 inset 时应 == static，当前 == relative），属架构性 |
| 产品 smoke 文本度量残余 | welcome 17% / wintertc 14% / morning 49% | P1（与 Phase A 同源） | Phase A IFC 统一（item-tag R109 inline→block + system-ui 字体度量，非图片/CSS 缺口） |
| WebP 解码 + CSS `url()` 背景图抓取 | 图片子资源残余（wintertc/morning 不用，低 ROI） | P3 | `decode_image_bytes` 扩 WebP + `fetch_image_subresources` 扩 `url()` |

### DC-11 doc 复核（2026-06-19，read-only 代码核查；无代码/reftest 变更）

承接 R323 margin 折叠探针（纠正 goal doc「未实现」）与 R324 position:fixed 修复（commit 5b11fc2）后，本轮 read-only 核查 DC-11「布局正确性」其余项是否如 goal doc 声称的「未实现」。逐项代码核查 + 生产接线验证：

| 项 | goal doc 旧声明 | 代码实证 | 裁决 |
|----|----------------|---------|------|
| BFC | 「无 BFC 概念，overflow:hidden 不隔离浮动/不阻止 margin 折叠」 | `establishes_bfc`（margin_collapse.rs:33-76）全条件（overflow/float/abspos/flow-root/flex/grid/table/multicol）**接线生产**（engine.rs:2940/2988/3052）；`use_bfc_float_containment`（engine.rs:2992）落地 float containment；margin 隔离 R323 实测 6 探针全过 | **过时**（待 goal doc 纠正；并行 agent R324 note 标 BFC float containment 为下一调查项） |
| `<img>` intrinsic + object-fit | 「无固有尺寸，object-fit 在 paint 阶段处理但无实际图片数据」 | `apply_replaced_element_sizing`（tree.rs:165，HTML width/height 属性 + SVG data URI + 解码固有尺寸）+ `compute_object_fit_rect` 全 5 值（Fill/Contain/Cover/None/ScaleDown，text.rs:1582，img paint site text.rs:614 调用）+ R318 图片数据端到端贯通 | **过时**（待 goal doc 纠正；并行 agent R324 note 标 object-fit 为下一调查项） |
| 滚动容器 | 「无真正滚动容器，浏览器层手动偏移」 | `scroll_x/scroll_y` 字段 + paint 偏移（painter/mod.rs:465-471）+ overflow 裁剪（needs_clip/clip_all_primitives_to_rect，mod.rs:197/298）；app 层 scroll_offset per tab + wheel 路由 | **基本准确**（paint 偏移+裁剪已落地，非 layout 级真滚动容器；master.md 已如实标「简化处理」）→ 不改 |
| position: sticky | 「需宿主层动态调整」 | `is_sticky` 标记落地（engine.rs:606）。**R326 实测纠正**：converter（converter/mod.rs:286）把 `Sticky` 映射为 taffy `Position::Relative`，故 block-level sticky 偏移**已被 taffy 应用**（scroll-0 应吸附场景 delta==inset，新单测 `test_sticky_applies_inset_like_relative_at_scroll_zero` 实证）。旧注「偏移未应用」源于 `engine.rs:1948` 死代码（`#[allow(dead_code)]` 的 `apply_relative_offsets`）注释，非生产路径。缺的是 **scrollport 相对钳制**（normal 位满足 inset 时应 == static，当前渲染 == relative）——属架构性，非单点修复 | **部分过时**（R326 已纠正「偏移未应用」为「已应用、缺 scrollport 钳制」） |
| position: fixed | 「当前错误地映射为 absolute」 | `adjust_fixed_to_viewport`（engine.rs:2176）存在且调用；**R324（commit 5b11fc2）已修 fixed-inside-positioned-ancestor over-correction（`+=`→`-=`）** | R324 已处理（goal doc 纠正见并行 agent 提交） |

**结论**：goal doc DC-11 的 BFC + object-fit 两项「未实现」声明与代码现实矛盾（governance §1 自洽）。本轮将核查结论沉淀于本表（避免与并行 agent 活跃编辑 rendering-compat.md 冲突）；goal doc prose 纠正**已由 R325 执行**（BFC known-gaps line 378 + 替换元素 DC-11/support-envelope/known-gaps 三处，按 R323/R324 先例）。scroll/sticky 声明准确不改。本轮零代码变更（仅 read-only 核查）。

## IFC 统一技术参考

> 本节为 R69+ 执行 agent 提供精确的代码级上下文，避免重复探索。

### 三套 IFC 运行路径

当前系统在三个不同时机运行 IFC，使用不同的上下文参数：

| # | 函数 | 文件:行 | styles | 时机 | 作用 |
|---|------|---------|--------|------|------|
| ① | `measure_text_content` | `engine.rs:1462` | 真实 `styles` | taffy 布局中（measure callback） | 返回 `Size{width, height}`，taffy 据此计算块级位置 |
| ② | `remeasure_text_with_float_exclusions` | `engine.rs:2150` | 真实 `styles` | taffy 布局后（step 6） | 重新 IFC + float exclusion，存储 5 个 override map，更新容器高度（shrink） |
| ③ | `paint_text` → IFC | `text.rs:884` | **空 `HashMap::new()`** | paint 阶段 | 生成 glyph 图元，依赖 5 个 override map 回退字体度量 |

**当前 use_stored 路径**：`text.rs:788` 检查 `box_node.inline_layout.is_some()` 且宽度匹配时直接复用存储的 IFC 片段位置，不运行 IFC ③。此路径代码就绪但 `inline_layout` 当前始终为 `None`（因为 `compute_final_inline_layouts` 在 `engine.rs:198` 被注释禁用）。

### Paint IFC override 覆盖缺口

`collect_inline_items`（`inline/mod.rs:708`）在 styles 为空时通过以下 override 回退。**粗体**标记影响行断（字符宽度）的属性：

| 属性 | 覆盖机制 | 覆盖状态 | 影响 |
|------|---------|---------|------|
| **`font_size`** | `font_size_overrides[parent_id]` → 16px | ✅ 已覆盖 | **宽度 + 行高** |
| `line_height` | `line_height_overrides[parent_id]` → fs×1.2 | ✅ 已覆盖 | 仅行高 |
| **`letter_spacing`** | `letter_spacing_overrides[parent_id]` → 0 | ✅ 已覆盖 | **宽度** |
| **`word_spacing`** | 无覆盖机制 | ❌ 始终为 0 | **宽度** |
| **`is_ahem_font`** | `is_ahem_overrides[parent_id]` → false | ✅ 已覆盖 | **宽度** (0.55fs vs 1.0fs) |
| `vertical_align` | 无覆盖机制 | ❌ 始终为 Baseline | 行盒对齐 |
| **`margin_left/right`** (inline 元素) | 无覆盖机制 | ❌ 始终为 0 | **宽度** |
| `padding/border` (inline 元素) | 无覆盖机制 | ❌ 始终为 0 | 行高 |

### 已穷尽的不可行路径（R37-R68 共 32 轮）

以下所有路径均已尝试并回退，**R69+ 不需要重试**：

| 路径 | 结果 | 根因 |
|------|------|------|
| 修改 glyph advance (render_fs) | 回归 | 字形推进与 IFC 片段位置不一致 |
| 传递完整 styles 到 paint IFC | 回归 -5~-6 | 行断行为改变 |
| 存储 layout IFC 结果（所有变体） | 回归 -4~-6 | 存储 IFC 上下文与 paint 时不同 |
| font_size_overrides 启用 | 零改进（R45）/ 可能回归 | 行断变化 |
| is_ahem glyph advance 修改 | 回归 -2~-3 | 字形推进与 IFC 行断不一致 |
| letter_spacing_overrides 启用 | 零改进 | — |
| line_height_overrides 启用 | 零改进 | 仅影响垂直，不影响水平 |
| inline_element_metrics 启用 | 零改进 | 仅影响垂直 |
| default_font_metrics 传递 | 回归 -6 | font_size 变化导致行断不一致 |
| taffy measure callback IFC 缓存 | 回归 -5 | 多次 measure 调用的 available_space 不同 |
| 外边缘边框完整厚度 | 回归 -2 | taffy 单元格定位冲突 |

### 存储 IFC vs Paint IFC 基线计算差异

存储 IFC 的 fragment 基线位置计算与 paint IFC 的 glyph 基线位置不同：

```
存储 IFC：baseline_y = frag.y + frag.height     （line-height 盒底边）
paint IFC：baseline_y = frag.y + font_size      （当前 paint 使用）
差值 = (line_height - font_size) / 2            （半行距）
```

启用 `use_stored` 路径时，需要统一基线计算。推荐以存储 IFC 的 `frag.y + frag.height` 为准（CSS 规范中 line-height 半行距分布在文字上下）。

### IFC 统一完成度检查清单

```
✅ LayoutBox.inline_layout: Option<Vec<InlineLayoutLine>>    (engine.rs:167)
✅ LayoutBox.inline_layout_width: f32                          (验证容器宽度匹配)
✅ compute_final_inline_layouts() 函数实现                    (engine.rs:1175)
✅ paint 侧 use_stored 路径                                   (text.rs:788)
✅ store_font_sizes_from_ifc() 5 个 override map              (engine.rs:874)
✅ remeasure 高度收缩 → sibling reflow (shrink)               (R68)
❌ compute_final_inline_layouts 启用                          (engine.rs:198 被注释)
❌ remeasure 高度增长 → sibling reflow (grow)                  (仅处理 shrink)
❌ table/multicol 后处理后重新运行 IFC                         (宽度可能改变)
❌ 存储 IFC vs paint 基线计算对齐                             (frag.height vs font_size)
```

### Taffy Fork 状态

项目已 fork taffy 0.7.7 到 `crates/taffy-local/`（~16,400 行），通过 workspace `[patch.crates-io]` 替换 crates.io 版本。当前仅有一个自定义补丁：

- `cached_baselines()` 访问器（`cache.rs:187`, `taffy_tree.rs:853`）— 暴露 taffy 内部缓存的 `first_baselines`，供 inline-flex/inline-grid 基线提取

**结论**：不需要深度修改 taffy。IFC 统一通过在 remeasure 后处理阶段（taffy 布局完成后）存储完整 IFC 结果并传播高度变化来实现，不涉及 taffy 内部算法变更。

---

## IFC 之外的其他卡点

IFC 统一预计解决 ~50 个失败测试。剩余 ~48 个失败测试的根因分布如下。

### 卡点 #2：Multicol Column Breaking（~22 测试，独立于 IFC）

**影响**：css-multicol 当前 35/57 (61.4%)，距 95% 需 +18。是所有目录中通过率最低的。

**当前能力**：R41 实现了 column breaking 的 paint 层渲染 — 将整个子元素分配到各列后，paint 按列裁剪。这解决了 4 个 breaking 测试（000/001/002/003）。

**缺失**：**内容碎片化（content fragmentation）** — 当单个块级子元素的内容（如长文本段落）超过列高时，需要将其拆分到多个列。当前只能移动整个子元素到下一列。

**关键失败测试**：
- `multicol-breaking-004/005/006`：单个段落跨列拆分（diff 5.6-16.6%）
- `multicol-fill-auto-*`：column-fill:auto 的填充行为
- `multicol-count-*`：列数计算的边缘情况
- `multicol-clip-*`：溢出裁剪

**技术方向**：在 `assign_children_to_columns_with_breaking`（`multicol.rs`）中实现内容级拆分 — 对超高子元素，先运行 IFC 获取文本行，按列高逐列分配行。

---

### 卡点 #3：Writing-mode 垂直布局（~10 测试，部分独立于 IFC）

**影响**：css-writing-modes 当前 49/59 (83.1%)，距 95% 需 +7。Large-diff 测试（>9%）的根因是垂直模式下 float/clearance 定位不正确。

**当前能力**：
- 盒体几何轴交换：✅ — taffy 输入前交换 CSS 属性到水平模型，提取结果后逆交换回视觉坐标
- 垂直字形渲染：✅ — paint 层通过 `GlyphPrimitive.rotation = π/2` 旋转文字
- 垂直模式 inline 布局：✅ — R14 实现

**缺失**：垂直模式下 float/clearance 的完整轴交换。R57 尝试了完整轴交换方案（交换子元素尺寸 + 容器属性），但因零高度 float 元素的 block 轴 extent 改变导致 `clearance-calculations-vrl-008` 回归而回退。

**关键失败测试**：
- `direction-vlr-*` / `direction-vrl-*`：垂直书写方向（~12% diff）
- `clear-clearance-calculation-vrl-*`：垂直模式 clearance（~2-14% diff）
- `float-contiguous-vlr-*`：已全部通过（0.00%）— R57 发现无需修改

**技术方向**：精细轴交换 — 仅交换 float 的 inline 轴定位方向（x↔y），不改变 float 自身的 block 轴 extent。或采用更保守的方案：当前 83.1% 已接近目标，优先推动 multicol 和 flexbox 更远的目标。

---

### 卡点 #4：Flexbox Baseline 对齐（~3-5 测试，独立于 IFC）

**影响**：css-flexbox 当前 37/55 (67.3%)。虽距 95% 需 +14，但其中 ~10 个的根因是 IFC 架构（inline-flex 容器内文本定位），~3-5 个是 baseline 对齐问题。

**当前能力**：R59 添加了 taffy `cached_baselines()` 补丁和 `extract_baselines_recursive`。`adjust_inline_block_positions` 优先使用 taffy 缓存基线，回退到 font-size 近似。

**缺失**：taffy 仅在 flex 容器有 **≥2 个 `align-self: baseline` 子元素**时才计算子元素基线。大多数 WPT 测试使用默认 `align-self: stretch`，导致 `child.baseline` 保持默认值 0.0，基线计算等价于 `offset_cross + 0.0`。

**关键失败测试**：
- `flexbox-baseline-multi-line-horiz-003/004`（~48% diff）：inline-flex + flex-wrap:wrap + align-content:center 的复杂交互
- `flex-order-wrap-reverse-baseline` (1.27%)：wrap-reverse baseline

**技术方向**：修改 taffy 的 `compute_flexbox_layout` 使其对所有 flex 子元素计算基线（不限于 baseline-aligned），或扩展 `cached_baselines()` 提供合成基线。

---

### 卡点 #5：Table Border-collapse 精度（~3 测试，独立于 IFC）

**影响**：css-tables 当前 46/55 (83.6%)。near-miss 测试的根因多为 border-collapse 外边缘精度。

**当前能力**：R49 实现了 `resolve_collapsed_borders`（含行组边框集成）、`collapsed_border_outer_edge` 标记。Cell-vs-Cell 内部边颜色修正已合入。

**缺失**：外边缘单元格边框减半（与表格边框各占一半），导致边缘视觉宽度与规范不一致。R49/R50/R53 三次尝试完整厚度外边缘边框均导致回归 — taffy 的单元格位置基于原始边框宽度计算，完整厚度边框扩展超出元素边界。

**关键失败测试**：
- `border-conflict-resolution` (1.50%)
- `row-group-margin-border-padding` (1.32%)
- `whitespace-001` (1.05%)

**技术方向**：在 table layout 的 `position_cells` 中，对外边缘单元格的位置进行调整以匹配解析后的边框宽度。或在 converter 中移除边缘单元格的外部边框（从 box model 中减去 border 贡献）。

---

### 卡点 #6：CSS 2.1 Appendix E 堆叠顺序（2-3 测试，独立于 IFC）

**影响**：涉及 position:relative 容器内嵌套 absolute/fixed 后代的绘制顺序。

**当前能力**：R61 实现了基础堆叠排序（negative z-index → normal flow → floats → non-negative z-index）。

**缺失**：position:relative 元素不创建 stacking context 时，其 positioned 后代应参与父级 stacking context 的 step 6 排序，按 tree order 排列。当前实现将 positioned 元素全部排在 normal flow 之后，不区分嵌套层级。

**关键测试**：`flex-item-position-relative-001` (1.04% — 已在边缘，修复后可能通过)

**技术方向**：在 `paint_node_in_rect` 的排序逻辑中，增加对 positioned 后代 tree order 排序的支持。改动集中在 `paint/painter/mod.rs`。

---

### 卡点 #7：Grid Max-content Sizing（2-3 测试，独立于 IFC）

**关键测试**：`child-border-box-and-max-content-001/002` (1.52%)。near-miss，距通过很近。

**技术方向**：taffy grid 的 max-content 尺寸计算。可能需要调整 `computed_style_to_taffy` 中 grid item 的尺寸约束映射。

---

### 卡点 #8：Swatch 图像缩放精度（~5 测试，独立于 IFC）

**影响**：CSS2 floats-clear 中多个 near-miss 测试。15×15 或 20×20 纯色 PNG 被缩放到 96×96，双线性插值产生边缘伪影 vs CSS background-color 的精确填充。

**当前能力**：R43 添加了 `ImageData.solid_color` 检测和 CPU renderer 快速路径。

**技术方向**：对 solid_color 图像使用 nearest-neighbor 缩放（而非双线性），或直接按 solid_color 快速路径渲染（跳过纹理采样）。

---

### 卡点 #9：Position Fixed 视口定位（1-2 测试，独立于 IFC）

`position: fixed` 当前被 taffy 当作 `absolute` 处理（相对于包含块）。R68 禁用了 `adjust_absolute_to_initial_containing_block`（因导致 4 个 PASS→FAIL 回归）。需要重新设计更精细的条件判断。

---

### 卡点依赖关系与推荐执行顺序

```
IFC 统一（~50 tests）
  ├── 无依赖，可立即推进
  └── 完成后重新评估各目录通过率
      │
      ├── Multicol breaking（~22 tests）
      │   └── 独立，可与 IFC 并行推进
      │
      ├── Writing-mode 垂直（~10 tests）
      │   └── 可并行，但建议 IFC 后再做（依赖 IFC 修复后的文本定位）
      │
      ├── Flexbox baseline（~3-5 tests）
      │   └── 依赖 taffy 修改，可独立进行
      │
      └── 小卡点（table border / stacking order / grid / swatch / fixed）
          └── 独立小修复，可穿插进行
```

**推荐 R69+ 优先顺序**：
1. **IFC 统一**（最大杠杆，P0）
2. **Multicol column breaking**（第二大杠杆，可并行）
3. **Writing-mode 垂直**（当前 83.1%，离 95% 仅差 7 个，优先级可降低）
4. **小卡点穿插**：swatch 精度（影响 5 个 near-miss）、stacking order（1 个 near-miss）、grid max-content（2 个 near-miss）

---

## 技术决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-06-06 | 保留真实网站测试的 #[ignore] | 本地网络不稳定，这些测试不可执行 |
| 2026-06-06 | 扩展而非重写 manifest.rs 和 reftest.rs | 目标文档明确要求扩展现有模块 |
| 2026-06-06 | 使用内联 reftest 替代上游导入 | 避免网络依赖，53 个 CSS 2.1 核心 reftest 覆盖主要布局场景 |
| 2026-06-06 | mismatch 阈值设为 0.5% | 800×600 视口下，50×50 小元素差异约 0.52%，1% 阈值会漏检 |
| 2026-06-06 | 文字类 reftest 使用宽松容差 (5%/15ch) | fontdue vs Skia 字体渲染像素差异大 |
| 2026-06-06 | QuirksMode 在 StyleSystem 内部传递（不暴露为公开参数） | 保持公共 API 简洁，doc.quirks_mode() 在 compute_styles 入口处提取 |
| 2026-06-07 | quirks mode 颜色/长度解析通过函数指针分发 | parse_color_fn/parse_length_fn 模式避免重复 match 分支 |
| 2026-06-07 | apply_quirks_mode_adjustments 接受 tag_name 参数 | 需要按元素标签（如 table）应用不同的 quirks 规则 |
| 2026-06-07 | inline 元素 width/height quirks 暂不实现 | layout engine 将 inline 映射为 block，实际已生效；待 inline layout 正确实现后补充 |
| 2026-06-07 | UA 默认 display 值通过级联注入（Origin::UserAgent） | 最低优先级，可被作者样式覆盖；避免修改 ComputedStyle::default() |
| 2026-06-07 | Table 布局通过后处理步骤实现（类似 float） | taffy 无原生 table 支持，所有 table display types 映射为 Block 后重新定位 |
| 2026-06-07 | 修复 parse_display 缺失 table types 的 bug | color.rs 中有重复的 parse_display（缺 table types），通过 pub use color::* 被实际使用 |
| 2026-06-07 | Multi-column 通过后处理步骤实现（类似 float/table） | taffy 无原生 multicol 支持，column-count/column-width 容器的子元素在后处理中重新定位到各列 |
| 2026-06-07 | 多列均衡分配使用 shortest-column-first 策略 | 依次将每个子元素放入当前总高度最小的列，实现视觉均衡 |
| 2026-06-07 | CJK normal 模式下每个字符单独作为"单词" | CSS 规范要求 CJK 允许任意字符间断行，split_into_words 中 CJK 字符独立为词 |
| 2026-06-07 | text-align: justify 使用 effective_content_area 计算剩余空间 | 修复了原先用 container_width 忽略 float exclusion 的问题 |
| 2026-06-07 | Float exclusion 从 max 改为 additive stacking | 多个同侧 float 应累加宽度而非取最大值 |
| 2026-06-07 | rustybuzz 集成到 TextShaper | 优先使用 rustybuzz 进行 OpenType shaping（GSUB/GPOS），回退到 fontdue 逐字符映射 |
| 2026-06-07 | unicode-bidi 集成到 inline layout | RTL 字符自动检测并重排序，LTR 文本零开销 |
| 2026-06-07 | FontLoader 存储原始字体字节 | 供 rustybuzz Face::from_slice 使用，fontdue 仍用于 advance width 获取 |
| 2026-06-07 | ShapedGlyph 增加 x_offset/y_offset 字段 | 来自 rustybuzz 的 GPOS 定位偏移 |
| 2026-06-07 | 新增 render_full_scene() 替代 render_scene_to_framebuffer() | 旧函数仅支持 fills + rounded_rects + glyphs，新函数支持全部 13 种图元 |
| 2026-06-07 | 新增 transform_webview_primitives() 替代 inline 坐标变换 | 旧方式仅处理 fills + glyphs，新函数处理全部 13 种 RenderPrimitives 字段 |
| 2026-06-07 | 渐变使用逐像素插值 | 线性/径向/锥形渐变均在 CPU 上逐像素计算，无 GPU 依赖 |
| 2026-06-07 | 阴影使用 box-blur 近似 | 三次 box-blur 近似高斯模糊，性能与质量平衡 |
| 2026-06-07 | 路径填充使用扫描线算法 | 逐行扫描多边形边界，奇偶规则填充 |
| 2026-06-07 | CPU 后处理：Transform/Clip/Filter/BlendMode 作为后处理步骤 | 像素级后处理，不依赖 GPU；GPU 渲染器需独立实现 |
| 2026-06-07 | GPU 渲染器多管线架构 | 5 条独立 wgpu 渲染管线：Fill+Glyph、RoundedRect、Gradient、Image、Blur。每种管线有独立 WGSL shader 和绑定组布局。Mesh-based 图元（stroke/path）通过 CPU 侧顶点生成复用 fill pipeline。Phase-separated 架构避免借用冲突。 |
| 2026-06-07 | 浏览器 GPU 路径集成 render_full_scene_gpu | render_frame() 改用 render_full_scene_gpu 替代 render_scene_ext，GPU 渲染路径现在支持全部 13 种图元。GPU 渐变测试使用 ±3 容差应对 float→u8 精度误差。 |
| 2026-06-07 | Taffy 0.7 内置 margin 折叠 | 发现 taffy 0.7 已通过 CollapsibleMarginSet 实现 CSS 块级 margin 折叠，不需要额外后处理步骤。移除了自实现的 margin_collapse 后处理。 |
| 2026-06-07 | Float clear 后处理实现 | 在 adjust_float_positions() 中实现 clear:left/right/both。非 float 元素的 clear 属性将其推到对应侧浮动元素的底部之下。LayoutBox 新增 clear 字段。 |
| 2026-06-07 | 重复渐变：fract() tiling | CPU 渲染器用 fract(t/period) 实现重复渐变周期循环；GPU 渲染器通过 WGSL shader 中 fract(t) 实现，repeating 标志通过 param3 取负编码传递。 |
| 2026-06-07 | 多图层背景 Vec 迁移 | background_image 从 BackgroundImageComputedValue 单值改为 Vec，CSS 解析器新增 parse_background_image_layers() 处理逗号分隔，paint 按逆序渲染（CSS 规范最后一层在最底）。 |
| 2026-06-07 | clip-path inset 实际裁剪 | 新增 clip_all_primitives_to_rect() 对全部图元类型（fills/rounded_rects/gradients/shadows/images/glyphs/strokes）应用矩形裁剪，替代原来的虚线指示器。 |
| 2026-06-07 | CSS mask 渐变蒙版 | mask-image 复用 BackgroundImageValue 类型解析，渐变蒙版通过 clip_all_primitives_to_rect 裁剪到渐变边界 + 平均 alpha 衰减实现。URL 蒙版暂不支持（需图像加载基础设施）。 |
| 2026-06-07 | overflow 全图元裁剪修复 | paint_node/paint_node_in_rect 中 overflow:hidden/scroll/clip 原来仅裁剪 fills+glyphs，渐变/阴影/图片/线段等图元溢出容器边界不被裁剪。改为使用 PrimitiveCounts + clip_all_primitives_to_rect 裁剪全部 13 种图元类型。 |
| 2026-06-07 | 滚动容器 paint 偏移 | LayoutBox 新增 scroll_x/scroll_y 字段。paint_node 中当 overflow == Scroll 时，子元素坐标减去 scroll 偏移量。overflow:Hidden 不应用滚动偏移（非滚动容器）。3 个单元测试验证。 |
| 2026-06-07 | render_full_scene 切换到上游 reftest | 上游 reftest 从旧版 render_scene_to_framebuffer（仅 fills+rounded_rects+glyphs）切换到 render_full_scene（全部 13 种图元）。同时启用 ImageCache 从 base_dir 加载 PNG 图片。这使 reftest 结果更准确，但也暴露了之前被不完整渲染掩盖的布局差异。 |
| 2026-06-07 | skip_indicators 模式 | Painter 新增 skip_indicators 标志，RenderPipeline 新增 set_skip_indicators() 方法。当设为 true 时跳过全部 ~30 个 CSS 属性调试指示器（border-collapse 橙色标记、direction 箭头等），避免干扰 reftest 像素对比。 |
| 2026-06-07 | UA 默认样式扩展 | 新增 body{margin:8px}、h1-h6{margin+font-weight}、p{margin:1em 0}、ul/ol{margin+padding-left} UA 默认样式，对齐浏览器默认行为。 |
| 2026-06-08 | table row-group 行索引修复 | build_grid 中行组内行存储 rg_child_idx 但 get_row_box 在 table_box.children 查找，导致 tbody/thead/tfoot 内的行被静默丢弃。修复：TableRow 新增 row_group_index 字段，get_row_box/position_cells 根据此字段正确导航到行组内的行。 |
| 2026-06-08 | column-gap 属性映射修复 | converter gap.width 原先使用 style.gap（仅 gap 简写设置），改为使用 style.column_gap（column-gap 长写属性）。同时修复 gap 简写解析：`gap: 10px` 现在同时设置 column_gap 和 row_gap。使用 fallback 策略：column_gap 非 0 时优先，否则使用 gap。 |
| 2026-06-08 | background-image 固有尺寸基础设施 | Painter 新增 image_sizes HashMap<u64, (f32, f32)>（url hash → intrinsic dimensions）。RenderPipeline.set_image_sizes() 将缓存传递给 Painter。reftest runner 在渲染前构建 ImageCache、提取固有尺寸。修复了 background-size: auto 拉伸到容器大小的问题。 |
| 2026-06-08 | is_block_level / is_relative 标志 | LayoutBox 新增两个布尔标志。is_block_level 用于 float/clear 后处理（CSS 规范 clear 仅适用于块级元素）。is_relative 用于 table 布局后处理保留 position:relative 的 inset 偏移。 |
| 2026-06-08 | gap 简写 handler 修复 | gap apply handler 不再设置 column_gap/row_gap（由各自的 longhand handler 通过 shorthand expansion 设置），避免 HashMap 迭代顺序不确定性导致的值覆盖。 |
| 2026-06-09 | reftest 分类容差 bug 修复 | 上游 reftest FileReftestCase::to_config() 使用 Default::default()（1%/5ch），未调用 ReftestConfig::for_category()。所有测试被以严格布局容差（1%）衡量，导致文字类测试（5% 容差）大量误判失败。修复后通过率 68.6%→73.5%（+24 测试）。新增 ReftestConfig::with_viewport() builder 方法。 |
| 2026-06-09 | columns 简写解析修复 | `columns: 3`（单整数）被 parse_column_width 先解析为 column-width: 3px，阻止 parse_column_count 执行。CSS 规范要求整数优先解析为 column-count。交换解析顺序后，`columns: N` 正确设置 column-count: N。 |
| 2026-06-09 | 零高度浮动处理 | adjust_float_positions Phase 1 中 line_max_height 跳过零高度浮动元素（child_outer_height == 0），避免空浮动元素推进后续浮动的 Y 位置。 |
| 2026-06-08 | table 行组位置更新 | position_cells 后新增 update_row_group_positions 后处理。按视觉顺序（thead→tbody→tfoot）计算行组的 y 位置和高度，含 border-spacing。支持 position:relative inset 从行组传播到子行。修复 out-of-order-elements-collapsed-border（46.32%→通过）。 |
| 2026-06-08 | CSS 绝对长度单位 | parse_length() 新增 in/pt/pc/cm/mm/Q 单位支持，按 CSS 规范转换为 px（96 DPI）。修复了所有使用 `height: 1in; width: 1in` 的 floats-clear 测试（之前 in 单位被静默忽略，元素折叠为 0 大小）。副作用：CSS2/borders 中使用 1in(=96px) 大边框的测试暴露了布局精度差异。 |
| 2026-06-08 | CSS inherit 关键字完善 | border/background shorthand 正确广播 CSS-wide keywords（inherit/initial/unset）到所有子属性。inherit_property 扩展支持非继承属性（background-*, border-*, margin-*, padding-*），使 `border-bottom: inherit` 等显式继承生效。 |
| 2026-06-08 | is_block_level 修正 | table 内部 display types（TableRowGroup, TableRow, TableCell 等）从 is_block_level 中移除。CSS 2.1 规定 clear 属性仅适用于块级元素，table 内部元素不是块级元素。 |
| 2026-06-08 | 参考文件过滤 | reftest loader 跳过以 -ref/-reference 结尾的文件名，避免参考页面被当作测试用例运行。移除 1 个误计入的测试（float-nowrap-3-ref.html）。 |
| 2026-06-08 | XHTML CDATA 调查 | 调查发现 html5ever 在 HTML 模式下将 XHTML CDATA 标记（`<![CDATA[...]]>`）保留在 `<style>` 文本内容中。CSS 解析器遇到 `<![CDATA[` 时错误恢复路径触发 `skip_to_rbracket()`，贪婪吞噬后续所有 token，导致整个样式表提取 0 条规则。之前通过 CDATA 损坏的 .xht 测试（test+ref 都无 CSS）实际是虚假通过。 |
| 2026-06-08 | XHTML CDATA 清理实施 | `strip_cdata()` 在 `collect_stylesheets()` 中去除 CDATA 前后缀。揭示真实通过率 66.1%（之前 76.4% 含虚假通过）。真实修复：background-087/326/328 ✅。揭示的渲染缺口：writing-modes 42.4%（需 writing-mode 布局支持）、multicol 49.1%（需 column breaking）、floats-clear 新增 6 个差异。 |
| 2026-06-08 | empty-cells border-collapse 修复 | `empty-cells: hide` 仅在 separated border model 中生效。在 collapsed border model 中，空单元格仍需显示边框。修改 paint_node 两处 skip_empty_cell 条件添加 `border_collapse == Separate` 检查。 |
| 2026-06-08 | row-group/row box model 抑制 | CSS 2.1 Section 17.5.3/17.5.4：在 separated border model 中，table-row-group 和 table-row 的 border/padding/margin 无视觉效果。新增 `suppress_row_group_row_box_model()` 和 `zero_box_model()` 函数。 |
| 2026-06-08 | table cell explicit height 保留 | 有明确 height 且 overflow:hidden/scroll/clip 的单元格保持 taffy 计算的原始高度，不被行高覆盖。修复 table-cell-overflow-explicit-height 测试。 |
| 2026-06-08 | CSS 2.1 Appendix E 绘制顺序 | paint_node_in_rect 和 paint_node 中子元素分两轮绘制：先绘制非 float 子元素，再绘制 float 子元素。确保 float 内容视觉上在 block 背景之上（CSS 2.1 Appendix E）。 |
| 2026-06-08 | columns 简写顺序无关解析 | expand_columns() 双值模式改为自动检测哪个是整数（column-count）哪个是长度（column-width），而非硬编码 parts[0]/[1]。修复 `columns: 100px 6` 等逆序声明。 |
| 2026-06-08 | clearance 计算代码质量改善 | 澄清 CSS 2.1 §9.5.2 clearance 语义：零 clearance 仍然阻止 margin 折叠；clearance = max(0, clear_bottom - hypothetical_position)。后处理方式的局限性在于 taffy 已应用 margin 折叠。 |
| 2026-06-08 | 空 inline 元素 line-height | 空 inline 元素（如 `<span></span>`）生成零宽度 TextRun，其 line-height 仍贡献到行盒高度。修改 collect_inline_items 不再跳过空 inline 元素。 |
| 2026-06-08 | sibling combinators 文本节点跳过 | NextSibling (+) 和 SubsequentSibling (~) 组合器现在跳过元素间的文本节点，匹配 CSS 选择器规范行为。修改 matches_selector_recursive 和 matches_has_selector_chain。 |
| 2026-06-08 | CSS 绝对长度单位 | parse_length() 新增 in/pt/pc/cm/mm/Q 单位（96 DPI），background 简写分类器新增所有长度后缀。修复使用 1in 高度的 floats-clear 测试。 |
| 2026-06-08 | 径向渐变位置修复 | gradient_to_primitive 改用 resolve_position() 正确处理 Percentage（百分比）和 Px（绝对像素），替代旧的 length_to_f32/100 逻辑。修复相关测试用例。 |
| 2026-06-08 | 表格 min-height border-box | apply_table_size_constraints 正确处理 min-height/max-height 为 border-box 约束（减去 padding+border）。修复 min-height-table。 |
| 2026-06-08 | 表格单元格高度最小值 | CSS 2.1 规范中 cell height 为最小高度，改用 max(row_height, cell_content_height)。 |
| 2026-06-14 | R118 失败全景再扫描（诊断） | 419/490 持平。对当前 71 个失败用例重跑 `REFTEST_BBOX`（输出在 stderr，含 sub-threshold 通过项共 290 行）并逐一交叉匹配，再对 10 个候选做 REFTEST_DUMP+PIL 深挖。结论：干净的增量修复已耗尽（R108 bbox-scan + R111 per-feature grep 找到的最后两个 cluster 是 inline-fmt 与 visibility:collapse）。逐项排除：clear-applies-to-009 (1.02%) 是 `<p>` 文本行亚像素纵向偏移导致的字体抗锯齿差异（near-miss，非 clearance bug）；html-display-table 根因 = `build_grid`（table.rs:1352 `_` 分支）跳过 display:table 的非 table-internal 子元素（如 `<body>`）→ 空 grid → `layout_table` 早返回（table.rs:230）→ html:table 沿用 taffy 800px，匿名单元格修复（让表格收缩到 300px）因 body 已被 taffy 以 780px 全高布局、post-processing 无法重排其子元素而失败，属**布局前匿名盒生成**的结构改动；child-border-box-and-max-content-001/002（1.52% 相同 bbox）= `width:max-content` 映射 Auto（converter/mod.rs:375）导致 grid item 不渲染，属 intrinsic-sizing 缺口，17 个用例用该关键字、4 失败 13 通过 → 实现风险净负（R97 仍成立）；multicol-count-computed-004 = column-count:auto+auto 正确返回 None（multicol.rs:150）→ 单列渲染，差异是文本-vs-swatch 两行换行的位置 near-miss；clearance-calculations-vrl（R114b 修正）需把整段 ~150 行 `adjust_float_positions_with_context` 做 block/inline 轴参数化，非 surgical flag，对通过的 floats-clear 有高回归风险。剩余 71 个失败全部属于结构性多轮里程碑：multicol column-breaking fragmentation（R113 两趟）、writing-mode 垂直轴 float/clearance 参数化（R114b）、flex baseline 合成、intrinsic sizing 关键字（R97）、布局前匿名盒生成。 |
| 2026-06-14 | R119 百分比 max-height 收紧 | 420/490 (+1)。taffy 0.7 不会对 height:auto 的块盒按百分比 max-height 收紧（converter 已传 Percent，但 block 布局未在内容高度计算后再次 clamp）。新增自上而下后处理 `clamp_percentage_max_height`（engine.rs），按 CSS §10.7 相对明确包含块高度解析百分比 max-height，CB 不明确时按 §10.5 视为 auto。修复 fieldset-as-item-overflow（ref 依赖 max-height:100% 收紧 200px 子元素到 100px），零回归（0 个 reftest test 文件用百分比 max-height，仅 1 个 ref 用）。2 个单元测试覆盖明确/不明确 CB。 |
| 2026-06-14 | R120 U+00A0 不可折叠实验（已回退） | 420/490 持平（净中性，已回退）。发现 `collapse_whitespace`（inline/mod.rs:286 `is_whitespace`）与 `split_into_words` 标准 normal 模式（`split_whitespace`）把 U+00A0（&nbsp;）当作可折叠/可断行空白，违反 CSS Text §4.1.1（nbsp 不可折叠、不可断行）。修复（collapse 保留 nbsp、split 排除 nbsp）符合规范，但净效果中性：multicol-containing-002 改善（4.65→3.67%），multicol-count-002 **恶化**（9.50→12.28%），全量仍 420/490。原因：受影响用例是 nbsp + 列分布 + 图片布局的复合问题，单独修 nbsp 不产生净胜；multicol-count-002 的 ref 用 nbsp 连接图片（非文本），test 用 nbsp 在文本中，两者上下文不同。已回退。**结论：nbsp 修复规范正确但净中性，不要单独重试**；需配合列分布精度（R113）一起做才可能产生胜场。 |
| 2026-06-14 | R121 嵌套 multicol 塌缩机制定位（诊断） | 420/490 持平。REFTEST_DUMP+PIL 精确确认 multicol-breaking-005 的嵌套 multicol **塌缩为窄条**（magenta 仅在 x 桶 0、y=29-36 共 7px；REF 分布到 x 桶 0-7、y=8-107）。MC_DBG 插桩定位根因：`layout_multicol`（multicol.rs:255）用 `child.height` 分配子元素到列，但 inline 文本子元素的 LayoutBox.height=0（行高由 IFC 独立测量，不写回 LayoutBox）→ balanced 分配把 29 个 h=0 子元素堆叠 → 塌缩。`remeasure_inline_only_containers`（engine.rs:2699 `balance_column_geometry`）确实按列宽测量并更新**容器** content_height，但不传播**单个 inline 子元素**的高度到 LayoutBox，故 multicol 分布仍看到 h=0。这是 multicol↔IFC 集成的结构缺口（比 R113「循环两趟」更精确：真正缺的是 inline 子元素高度从 IFC 回写到 LayoutBox）。修复需在 remeasure 后、`adjust_multicol_layout` 前，把 IFC 的逐行/逐子元素高度回写到对应 LayoutBox.children 的 height，或让 multicol 分布直接消费 IFC 行结果。属多轮结构里程碑，单轮不可安全完成（39 个通过的 multicol 测试有回归风险）。 |
| 2026-06-14 | R122 multicol 列分布 rebase 修正 + clean-win 穷尽复核（净中性） | 420/490 持平（净中性，零回归）。独立复核 15+ 失败候选（border/border-bottom-width、table-cell-overflow、clear-float-003、abspos-inset-nested、inline-box、block-in-inline、flex-gap、multicol 各项），全部确认为结构性/复合问题，印证 R118「干净增量修复已耗尽」。**multicol-breaking-005 塌缩的真正触发点定位**：painter/text.rs:700-701 的 `height_auto` 守卫——纯 inline + balance + height:auto 才做列分布，明确高度（height:300px）的嵌套 multicol 回退单块→塌缩。**关键反直觉发现**：multicol-breaking-nobackground-005 与 breaking-005 布局完全相同（diff 仅 background+column-rule），nobackground-005 的 TEST **同样塌缩**（text bbox 50x15 vs REF 725x15），但因其无 solid 背景，纯文本差异 <1% 阈值而**侥幸通过**（0.82%）；breaking-005 因 magenta 背景大面积缺失达 21.74%。**验证放宽守卫 net -5**（multicol 39→34/57）：paint 列分布算法 `col_start_y = col_idx * target_h` 对非整除行数有 fractional offset，对 nobackground-001/002、breaking-001/002、nobackground-005 产生比塌缩更差的结果。**修正**：painter/text.rs 列分布改为预计算每列**实际首行 y** 作 col_start_y（rebase 到列内 y=0），消除 fractional offset——单独应用净中性（420/490 不变）但改善 2 个 near-miss（multicol-columns-001 5.00→4.88%、multicol-containing-002 4.65→4.23%），为零回归的正确性改进。**结论**：multicol 嵌套列分布需 R113 两趟结构重构（内层列几何依赖外层 column-width，当前 paint 单次 IFC 无法精确），rebase 修正为该重构的必要前置正确性基础。另新增 `REFTEST_DUMP_PASS=1` 诊断（env 门控，dump 通过用例实际渲染，供未来轮次分析通过用例真实输出）。 |
| 2026-06-14 | R123 根元素 position:relative inset 应用（+1，零回归） | 421/490 (+1)。MC_DBG 插桩定位 `abspos-containing-block-initial-007` 根因：`<html style="position:relative; top:100px; left:100px">` 的 relative inset **未被应用**（root x=0 y=0，is_relative=true），导致根 html border-box 与其 abspos 后代 body（CB=根 padding box）整体位置错（TEST root@(0,0)、body@(20,70) vs REF root@(100,100)、body@(120,170)，差恰好 (100,100)=inset）。**根因**：taffy 0.7 对非根 block-level 元素把 position:relative inset 应用到 layout.location，但对**根节点**不应用（根总在 0,0）。**修复**：engine.rs extract_layout 后，当 `root_box.is_relative` 时手动应用 `resolve_relative_inset`（top/left Px）到 root.x/y（CSS 2.1 §9.4.3），使根及 abspos 后代整体偏移。**零回归**：全 wpt-data 仅 1 个 test 文件在根 `<html>` 上用 position:relative（grep 确认），修复只能帮助。新增单元测试 `test_root_relative_position_applies_inset`（relative 根→(100,100)，static 根→(0,0)）。make test 全绿，clippy 零警告。**方法论**：R98 修了无 positioned ancestor 的 abspos Length inset（viewport 相对），R119 修了百分比 max-height；本轮是同一 converter-passes-but-taffy-ignores 谱系的**根元素 relative inset**缺口——taffy 对根节点的 position:relative 静默丢弃 inset。 |
| 2026-06-14 | R124 非 positioned overflow 不裁剪 CB 为祖先的 abspos（+1，零回归） | 422/490 (+1)。MC_DBG 布局树 dump 定位 `multicol-contained-absolute`（什么都不渲染）根因：`overflow:hidden` 元素（content_height=0，因唯一子元素是 abspos 不贡献高度）把 CB 为祖先的 abspos 后代**误裁剪到 0 高度**。结构 `relative(CB) > overflow:hidden(h=0,非 positioned) > abspos(green 100%)`——abspos 的 CB 是 relative（overflow 的祖先），按 CSS §11.1.1 不应被该 overflow 裁剪，但 ZeroWeb 原先把 abspos 当普通子元素绘制被裁掉。**失败的实验**：把所有 abspos 子元素移到 overflow 裁剪后绘制 = net -6（破坏 z-order，影响 positioned overflow 容器内的 abspos）。**正确修复**：painter/mod.rs 仅对**非 positioned 的 overflow 元素**（`needs_clip && !self_positioned && !is_multicol`）把 abspos/fixed 子元素移到 overflow 裁剪之后绘制；positioned overflow 元素（`position:relative;overflow:hidden` 常见模式）保持原行为。**零回归**：421→422，69→68 失败，仅 multicol-contained-absolute 翻转。新增单元测试 `test_overflow_nonpositioned_does_not_clip_abspos_with_ancestor_cb`（构造 relative>overflow:h=0>abspos 树，断言绿色填充高度 ~100 未被裁剪）。make test 全绿，clippy 零警告。**方法论**：CSS overflow 裁剪规则按 CB 关系区分（CB 为本元素/后代→裁剪；CB 为祖先→不裁剪），ZeroWeb 原先无 CB 感知；本轮用「overflow 元素是否 positioned」作 CB 关系的可靠近似（非 positioned overflow 不可能是 abspos 的 CB），实现 zero-regression 的常见情形修复。**遗留**：完整 CB-aware 裁剪（处理非 positioned overflow 内含 positioned 后代再含 abspos 的嵌套情形）仍需预计算 abspos CB + 祖先链检查，属后续结构增强。 |
| 2026-06-14 | R125 large-font 集群三路死锁确认（诊断，净中性已回退） | 422/490 持平（所有实验均 net-negative 已回退）。MC_DBG 精确定位 large-font（font-051/ifc-008/009/011/empty-inline-002）根因：paint IFC（painter/text.rs:912）传 `&HashMap::new()`（空 styles）+ override map；对明确高度容器（如 #div1 `height:2em` + `font:100px Ahem`），remeasure 因 `height:Auto` 守卫跳过、override 为空 → paint IFC 默认 16px → 100px 内容渲染成 16px。关键发现：`paint_text` 入口 `style.font_size=100` 正确，但 IFC 用空 styles 解析片段 font_size 走默认 16。**三条修复路径全部 net-negative**：(1) compute_final_inline_layouts 中调 `store_font_sizes_from_ifc`（覆盖）= -1（multicol-fill-auto-001 回归，该用例依赖 remeasure IFC 的 font_size 解析，compute_final IFC 解析不同）；(2) 同上但 `.entry().or_insert()`（不覆盖）= 仍 -1（multicol-fill-auto-001 的新增文本节点条目改变其 paint IFC override）；(3) override 为空时传真实 styles（painter/text.rs:912）= **-4**（"override 空"条件不够窄，多个 box 命中真实 styles 路径）。印证 R72（真实 styles 回归 BFC-004/font-feature-002 等 4 个）+ R104/R117c（font_size 存储回归 font-feature）。**结论**：large-font 是结构性死锁——paint IFC 空 styles、remeasure IFC、compute_final IFC 三路径 font_size 解析不一致，任一统一都回归其他用例。需 Phase A IFC 路径统一（三路径一致 font_size 解析 + font-feature-settings 支持，R82/R84 标注的多件耦合一次性改动）。**不要再单独重试这三条路径**。下一步换结构性目标：multicol 两趟（R113）或 writing-mode 垂直轴（R114b）。 |
| 2026-06-14 | R126 聚类裁决与策略（诊断，持平） | 422/490 持平。回退接手时 multicol.rs 净中性 speculative change；REFTEST_DUMP+PIL 排除 5 聚类（block-in-inline / flex-gap / clear-applies-to 非同源 / multicol-breaking-006 颜色）均非 clean win。clean win 确已穷尽。结论：下一步必须 commit 多轮 nested-multicol 两趟（R113，最 contained）。 |
| 2026-06-14 | R127 float 容器 margin 不折叠（+1，零回归） | 423/490 (+1)。CSS §8.3.1 float 子元素 margin 不与父容器折叠——taffy 视 float 为 block 致容器 margin-top 被折叠到 float margin(max)。新增 `declared_margin_top` 字段 + 4 重门控（门控4=float mt 自身未被 taffy 膨胀）排除 floats-rule3 回归。修复 clear-applies-to-009。同 R108b 谱系但不同子问题。 |
| 2026-06-14 | R128 multicol balance 渲染路径表征（诊断，持平） | 423/490 持平。instrument multicol balance 渲染路径——4 列分布算法(4/3/3/3)+全列 glyph 已确认正确，失败在单列 IFC 行溢出列宽(100px 列出 200px 行)+列高不符 ref(test 62 vs 167)。结构性 IFC 换行/balance 精度，不要再 instrument 此路径。 |
| 2026-06-14 | R129 float shrink-to-fit 宽度（+2，零回归） | 425/490 (+2)。CSS §10.3.5 float width:auto shrink-to-fit——taffy 视 float 为 block 填满可用宽度。`declared_width_auto` 标记 + float pass 收缩到块级子元素最大 border-box 宽度（收缩在定位前=级联正确）。修 float-non-replaced-height-001 + bonus flexbox-column-row-gap-001。同 R127 谱系。 |
| 2026-06-14 | R130 strut 容器字体（+1，零回归） | 426/490 (+1)。CSS §10.8.1 strut——仅原子行内盒的行用容器 font-size 算 strut ascent（旧行高*0.8 被高原子盒撑高致基线偏低盒压下 8px）。修 flexbox-baseline-align-self-baseline-horiz-001；精确定位 flex baseline 集群根因（taffy baseline child-order 不一致 30/20/30 + strut 放大）。 |
| 2026-06-14 | R131 multicol fragmentation 架构（诊断，持平） | 426/490 持平。纠正 R121——multicol 塌缩的 29 个零高子元素是行内级 DOM 节点(text/br)非块级，「行高回写」不可行；真正修复=列感知 IFC 把行内流碎片化到各列（按行盒非子元素生成 column_span_offsets）；even-split 无效。R122 记录 paint 路径改动 net -5 回归。 |
| 2026-06-14 | R132 multicol 容器高度 dead-end（诊断，无提交） | 426/490 持平。新根因：块级子元素 balance multicol 容器高度=taffy 堆叠(过高)；修复 net-negative 已回退(regress paged-media/block-no-clip, fill-000 主因非高度)。re-verify 放宽 height_auto 守卫仍 net -5。nested breaking 真正阻塞=R113 CSS 碎片化语义（内层被外层碎片化时应 fill 非 balance）。 |
| 2026-06-14 | R133 vertical float 实现地基（诊断，持平） | 426/490 持平。vertical-rl 浮动函数实现地基：提取后坐标物理化(block=X 右→左/inline=Y)，converter 不换 float(物理 left/right→block 方向)，故不能轴交换复用水平函数，需独立 block-方向浮动逻辑；clearance-vrl-002 期望 3 绿条水平铺排(右→左 block)。 |
| 2026-06-14 | R134 vertical block 宽度收缩（+6，零回归） | 432/490 (+6)。纠正 R133——parent-width(vertical-rl width:auto 填满 784 而非内容)才是 clearance-vrl blocker，非 float 定位。新增 `shrink_vertical_blocks_to_content` 后处理（自限：仅内容右缘<width 时收缩）；ref 退化(img 不渲染→白)故测试=diff<5%。bonus 修 baseline-inline-replaced-002 + 2 orthog-float。 |
| 2026-06-14 | R135 image-loading degenerate-ref（诊断，已回退） | 432/490 持平。reftest harness `load_png_file` 不处理非 RGBA PNG(palette/RGB→alpha=0)→大量图退化白；单独修 net -5（暴露 clearance-calc 隐藏精度误差 clear-clearance-calc-001/002/003 + vrl-004/008）；必须与 §9.5.2 clearance-calc 精度修复打包。engine 渲染正常只是 harness 加载坏。 |
| 2026-06-14 | R136 flex/grid/table 建立 BFC（净中性零回归，DC-11） | 432/490 持平。`establishes_bfc` 新增 `is_layout_container`(Flex/InlineFlex/Grid/InlineGrid/Table/InlineTable) 判定。taffy 内部已按 BFC 布局，后处理补充判定不改当前结果；DC-11 正确性改进为产品页/未来用例提供正确 BFC 隔离。前置 R137。 |
| 2026-06-14 | R137 孤立 table-internal 匿名 table BFC（+1，零回归） | 433/490 (+1)。孤立 table-internal(父非 table) 经 `mark_anonymous_table_roots` 预处理标记 is_anon_table_root + is_block_level → 建立 BFC，修 clear-applies-to-001。关键=BFC float-exclusion 要求 is_block_level，仅补 is_anon_table_root 无效。 |
| 2026-06-14 | R138 display:table 收缩适应（+1，零回归） | 434/490 (+1)。`layout_table` 在 grid 为空时调用新增 `shrink_table_to_block_content`（新模块 table_shrink.rs，只收缩尺寸不重定位=零坐标风险）。修 html-display-table。关键=grid 空才触发(49 css-tables 用例零影响)+行内级求和(200+80=280)非 max。 |
| 2026-06-14 | R139 直接 table-cell 匿名行（+1，零回归） | 435/490 (+1)。display:table 直接 table-cell 子元素(无 tr)合并为单个匿名行(CSS §17.2.1)，修 subpixel-table-cell-width-001。旧 bug=每 cell 独立行 + get_cell_box 导航错误→cell 未测量全宽堆叠。is_anonymous 导航 + position_cells 守卫。 |
| 2026-06-14 | R140 clean-wins 穷尽独立验证（诊断，持平） | 435/490 持平（两轮诊断 + gap-fix 实验回退）。独立穷尽验证 55 失败清洁 surgical 已耗尽——instrumentation 证 border 数学正确、multicol col_width=100 正确、Ahem glyph=精确 font_size。`% gap→normal when indefinite` 实测 001/002/003 零回归但 004 仅 3.90→3.62%(真正阻塞=R109 垂直 flex 主轴，非 gap)故回退。R131 block-children 分布会回归 ~20 通过用例(=R122 net-5 源)。R141b R109 实测单会话不可解（6 轮），color-block 绘制路径绕过 paint_node 与 paint_text 非存储两条已知路径=深层 inline-block ownership + DC「Layout/Paint IFC 双路径」架构。 |
| 2026-06-15 | R141b → R142 vertical-rl 兄弟位移轴（净中性零回归） | 435/490 持平。R141b R109 实测不可解后，R142 在 `remeasure_inline_only_containers` 末尾兄弟位移 `sibling.y += shrink_delta` 增加 `HorizontalTb` 守卫——该逻辑只适用于水平书写模式，垂直模式块流方向为 x 轴，inline 轴收缩不在块轴留空隙故不应移动按 x 排列的块兄弟。修整页空白灾难(vrl-004 25→12.7%、vlr-005 25→9.3% 内容恢复可见但未过阈)。方法论=逐后处理步 BISECT 插桩(非 bbox 扫描)发现跨聚类遗漏。 |
| 2026-06-15 | R143 inline/block-size 逻辑尺寸属性（净中性零回归） | 435/490 持平。`inline-size`/`block-size` 在 apply.rs 中完全没有 match 分支——作为未知属性被静默忽略。新增 inline-size→width、block-size→height 映射（垂直轴由 converter swap 自动修正）。firefox-bug-1881495 7.28→1.74%（剩余=taffy grid 定宽 inline-grid 轨道不约束换行，taffy 内部）。方法论=属性实现完整性审计（第 4 种定位法，区别 bbox/grep/BISECT）：从失败用例反查其依赖的 CSS 属性是否被实现。 |
| 2026-06-15 | R146 css-flexbox-row 残留 + img-intrinsic 调查（诊断，持平，已回退） | 436/490 持平（两路诊断均无 reftest 价值，已回退工作区清洁）。**css-flexbox-row 残留 1.23%（R145 后）精确根因**：PIL 精确色块定位对比发现 R145 已修 flex item 位置（abs_x=12 左侧正确），残留差异是 **vertical-rl flex 容器的 width:auto shrink-to-fit 循环**——flex 容器（vertical-rl, width:auto）TEST 宽 92 而 REF 宽 34；`.item` flex item（width:auto）被 align-items:stretch 拉伸到容器 cross-size=90，而 shrink_vertical_blocks_to_content 读 child.width=90（已拉伸）→ 无法收缩容器到内容 34（34 需读 .item 的 intrinsic content width=30，而非拉伸后的 90）。循环依赖：容器宽度依赖子元素宽度、子元素宽度依赖容器 cross-size。修复需对 layout-container 父级在 shrink 时用子元素 intrinsic content width 而非拉伸后的 box width——属结构性改动，单 1 测试风险高，未强推。**flex-abspos-inset-nested img 调查（已回退死代码）**：定位根因=`<img>` 无 HTML width/height 属性时 ZeroWeb 不从解码 PNG 推导 aspect-ratio（Chromium 解码 1×1 得 ratio=1.0→height 拉伸 200→width=200）。完整实现 layout→img-intrinsic plumbing（LayoutEngine image_intrinsic_sizes 字段 + build_layout_tree/build_subtree/apply_replaced_element_sizing 全链路 + pipeline builder，~60 行），但 IMGDIAG 证实 **reftest harness image_cache 为空**——扁平布局使 `../support/` 路径解析失败（WPT 原始子目录布局 css/css-flexbox/abspos/test.html）。plumbing 正确但无数据流入=死代码，按 code-guidelines 已回退。修复需先解 harness 图像路径（R135 记录 net -5 风险）。**结论**：两条路径均确认结构性，单会话不可安全推进；436/490 平台期维持。**table-cell-width-0（30% diff）实验已回退**：CSS Tables §17.5.2.2 auto layout 额外空间分配——旧实现 `*= ratio` 按比例膨胀所有列（含 width:0 列），改为「只扩展 auto 列、显式 width 列（含 width:0→min-content）保持」+ 把 width:0(Px<2.0) 从 css_width_auto=true 改为 false（视为约束）。结果 **net -1**（table-cell-width-0 30→28% 未通过 + 新回归 table_grid_size_col_colspan 0→50.92%），已回退。结论：table-cell-width-0 的 30% 差异主因非列分布（百分表宽已正确解析、width:0 列宽修正后仍 28%），且 colspan 交互高敏感；属深层 table auto-layout + width:0+colspan 结构性，单会话不可安全推进。**block-in-inline-align-001（1.42%）诊断**：PIL 确认 TEST 第 2 个 section（`dir="rtl"` + `<span>text<div>text</span>` block-in-inline）的 div 橙色背景完全未绘制（0 orange px vs REF 1072），第 1 个 section（LTR text-align:right）正常（2336 orange）。可复现（字节级 copy zz-copy 同样失败），但手写结构探针通过（差异极细微、未能定位）。EX 插桩证实两个 div 均 is_block_level=true display=Block 且几何正确（在 layout 树中），paint PD2 插桩证实两个 section 都被 paint_node 进入（nchildren=1 即 span）。根因在 paint 层：paint_node 递归进入 span→div 调用 paint_background，但 RTL section 的 div 填充未出现在最终 framebuffer——疑似 paint IFC 对 inline span 的文本绘制路径在 RTL 下消耗/排序异常导致 div 背景丢失，需更深 paint 路径插桩（下轮）。**R147 精确根因定位（已回退实验）**：TR 插桩（paint_node ENTER + paint_background）证实**两个 section 的 div 背景都被 paint_background 生成**（orange fill at abs_x=8 abs_y=8 第 1 section、abs_y=27 第 2 section）——填充图元未丢失。第 2 section 的 div 被画在 abs_y=27（与第 1 section 重叠），而 REF 在 ~65。**真正根因 = 块流重叠**：block-in-inline 的 `<span>text<div>text</span>` 使 section 展开为 3 行（h=38.4，paint trace span h=58），但 `remeasure_inline_only_containers`（engine.rs:2962-2989）的兄弟位移逻辑**仅处理收缩（shrink_delta < -0.01）不处理展开（正 delta）**——section[0] 展开到 38.4 后，section[1] 仍停在 taffy 算的 y=19（1 行高），两者重叠，第 2 section 的 div 被第 1 section 内容覆盖。**实验（已回退）**：把位移条件改为 `shrink_delta.abs() > 0.01`（同时处理展开）→ block-in-inline-align-001 1.42→1.69%（**恶化**），全量仍 436/490。原因：非所有正 delta 都是 block-in-inline 展开（IFC 高估也会产生正 delta），盲目位移正 delta 回归其他用例。**结论**：需更精确信号区分「taffy 低估 block-in-inline 展开」vs「IFC 高估」才能安全位移——例如仅对「含 inline 容器内嵌 block 子元素（block-in-inline）」的展开位移。下轮可加此精确守卫。**R147b 精确守卫实验（已回退）**：实现 `has_inline_child_with_block_descendant`（仅对块级容器含 inline 子元素且其内嵌 block 后代的展开位移），SH 插桩证实位移正确触发（idx=0 delta=19.4 is_expand=true shift_allowed=true）。但 **block-in-inline-align-001 仍 1.42→1.69%（恶化）**，全量 436/490。原因：嵌套展开——section[1] 自身的 span 也展开（delta=19.4），简单按 delta 位移产生过校正。**真正修复**：block-in-inline 容器展开需原子化多趟或一次性按展开后高度重排兄弟 y（不能逐子元素 delta 累加），属结构性改动。**不要再尝试逐 delta 位移法**。 |
| 2026-06-15 | R145 flex/grid/table 子元素 float 归零（+1，零回归，纠正 R144 R109 误判） | 436/490 (+1)。双端插桩（engine.rs extract_layout + painter/mod.rs paint_background）对比发现 css-flexbox-test1 / css-flexbox-row 的 `.item` flex item：extract_layout box.x=2（正确），paint box.x=690（=780−90 容器右缘减宽度）。**真正改写 x 的是 `adjust_float_positions_with_context` 的浮动后处理**——`.item` 带 `float:right`（CSS 注释），Phase 1 定位到 `container_width − right_used_width = 690`。R141b 的 mirror child.x 实验无效（它改 flex item 的 block 子元素非 IFC）。**纠正 R144「R109 paint 路径未定位、6 轮不可解」误判**——真正破坏入口是浮动后处理。**修复**：CSS Flexbox §4 / Grid §4 / Tables §2.4 规定布局项 float 计算为 none；浮动后处理入口对 `is_layout_container` 父级直接子元素归零 float（6 行）。零回归：435→436，css-flexbox 44→45，唯一翻转 css-flexbox-test1 FIXED（0.00%），css-flexbox-row 改善 1.82%→1.23%（剩余=vertical-rl 色块 IFC 列序独立子问题），零新失败。+1 单测 `test_flex_item_float_is_ignored`。make test 全绿，clippy 零警告，smoke 686/686。**方法论教训**：架构性失败的「不可解」结论需经多入口插桩交叉验证（R141b 单一 paint 入口的失败不能推广为整体不可解）。 |
| 2026-06-15 | R144 平台期独立复核（诊断，持平，无提交） | 435/490 持平（make reftest-upstream 实测确认 435/490，55 失败）。本轮独立复核三项：(1) **属性实现完整性审计穷尽**——写脚本对比 registry.rs 全部 197 个已注册属性 vs apply.rs/apply_advanced.rs 的 match 分支，**每个已注册属性都有对应 apply 分支**（R143 的 inline-size/block-size 是最后一个缺口），属性审计路径已完全穷尽；(2) **R109 color-block 绘制路径**——PIL 确认 css-flexbox-row.html 的 4 个 inline-block 色块 TEST 在 x[670,789](右) 而 REF 在 x[10,39](左)，证实 R141b 的结论（flex item 主轴定位 + inline-block 经 IFC，mirror child.x 不生效），6 轮不可解，架构项目；(3) **border-001 / column-height-009 / float-006 / baseline-vertical 等单例**——PIL/bbox 复核均确认为结构性复合问题（large-font 死锁、multicol-2 column-height/column-wrap、abspos+float+z-order、vertical table baseline），非清洁修复。**结论：435/490 为已确证的单会话零回归平台期**，剩余 55 失败全部属于结构性多轮里程碑（multicol column-breaking R113 两趟 / writing-mode 垂直轴 R109 架构 / flex baseline 合成 / vertical table baseline）。下一步建议：(a) 启动 inline-block ownership + vertical-rl 多会话架构项目（统一 paint 的 inline-block 绘制路径到 IFC，解 R109 聚类 css-flexbox-row/test1 + flexbox-column-row-gap-004），或 (b) 启动 multicol 列感知 IFC 碎片化（R131，把行内流按行盒碎片化到各列，影响 ~16 测试但回归风险 20 通过用例）。两条均为多轮、高风险、单会话预期 +0。 |
| 2026-06-15 | R148 全量 54 失败独立再核（诊断，持平，无提交，工作区清洁） | 436/490 持平（make reftest-upstream 实测 436/490=89.0%，54 失败）。本轮对全部 54 失败按 diff 升序排列并逐个 PIL+BIPROBE/CRPROBE/BBPROBE 独立插桩，**无任何 +1 清洁修复可安全推进**，全部印证结构性。新发现/精化（区别既往轮次）：(1) **multicol-breaking-006 (1.20%) 精确双根因**——CRPROBE 插桩 painter/text.rs paint_column_rules：外层 multicol(count=4 col_w=188) 仅 1 子(内层 .inner)，内层 .inner 的 16 个行盒**全部 x=102（右列），左列(x=0..86)无内容** → has_left_content=false 跳过 fuchsia column-rule；外层 col1/2/3 无子 → 跳过 blue column-rule。根因=**列分布把全部内容堆到单列**（应平分 2 列），column-rule 缺失是表象，真因是 R131 列感知 IFC 碎片化缺失。修 column-rule 绘制本身无意义（R112 已证 net -1）。(2) **border-bottom-width-006 (2.86%) 非绘制 bug**——BBPROBE 证实 #test border-bottom=96 w=96 h=96 ay=51 style=Solid 几何全正确、paint_borders 入口正确生成填充；差异源=inline-block「仅底边框 height:0」与 #reference 黑块的**垂直定位/基线对齐**（test 黑区 y=[16,146] 含 100px 宽条 vs ref 双 96×96 黑块 y=[29,151]），属 inline-block baseline 定位子问题非 border 渲染。(3) **clear-inline-001 (5.94%) 非 clear bug**——断言「clear 不能应用于 inline box」(CSS2.1 §13.5)，span2 clear:left 应被忽略；TEST 把蓝文本画在 float 旁(y=51)实际符合规范，REF 因用 96px 非浮动 `<img vertical-align:top>` 撑高行盒使蓝 span 落到 y=147；TEST/REF 结构不同故天然差异，非 ZeroWeb 渲染错误。(4) **block-in-inline-align-001 独立复现 R147b 恶化**——BIPROBE 证实两个 section 各 expand 19→38.4(delta=19.4, has_block_child=false 因 div 嵌在 span 内非 section 直接子)；BIR148 实验(正 delta 位移后续兄弟)复现 1.42→1.69% 恶化，PIL 显示 section[0] 的 div 画在 y=8(section 顶)而非文本一行后的 y=27——证实**另含 paint IFC 对 inline span 内 div 的垂直定位 bug**（layout 把 div 视 section 顶部，paint IFC 同样），叠加兄弟位移=双路径不一致，确认需 Phase A IFC 统一。(5) baseline-007/008(multicol baseline-export+column-span+flex align-items:baseline)、multicol-count-computed-003/004(Ahem 字形跨列溢出+column-rule)、flex-container-min/max-content(R97 intrinsic sizing 4win/13regress)、table-cell-width-0(R146 已证 net -1)、column-height-009(css-multicol-2 column-height/column-wrap 未支持 spec)、flexbox-baseline-align-self-baseline-vert(垂直 flex 基线合成)——全部结构性复合。**结论**：436/490 平台期经本轮全量 54 失败独立再核后**再次确证为单会话不可推进**；剩余按聚类=multicol 碎片化(~16) + IFC 双路径/inline-block ownership(block-in-inline×3+css-flexbox-row+large-font×4) + 垂直书写模式轴(×5) + flex 基线合成(×4) + intrinsic sizing(×4) + 表格深层(×3) + spec 未支持(multicol-2)。下一步须启动结构性多轮项目，单会话预期 +0。 |
| 2026-06-15 | R149 DC-10 draw_order 基础设施（净中性零回归，已提交）+ PNG bundle 实测（已回退） | 436/490 持平（默认 + `ZERO_DRAW_ORDER=1` 双路径均 436，set-diff 零翻转）。**已提交**：`RenderPrimitives` 增 `draw_order: Vec<DrawOp>` + 每个 `add_*` 记录插入顺序；`render_full_scene` 拆 `render_typed_buckets`（默认字节不变）+ `render_draw_order`（`ZERO_DRAW_ORDER=1` 按序）；cull 重建清空 draw_order；+1 单测。make test 12177/0，clippy 零警告。**纠正 R135b「draw_order net -1」结论**：436 基准实测净中性（R142 vertical-rl 守卫已消除 abs-pos-non-replaced-vrl-002 回归），draw_order 可安全启用。**PNG bundle 实测（已回退）**：`load_png_file` 加 `EXPAND\|STRIP_16` 实测 436→427 net -9（修正 R135 记录 -5）；PNG+draw_order 组合仍 net -9。PIL 证实 abs-pos-non-replaced-vrl-006 绿 span=0（应 6400）、红=18321 主导——**真正阻塞重新定性为 abspos vertical-rl §10.3.7 静态位置 bug**（非 DC-10 绘制顺序），PNG 网格仅暴露该布局 bug。bundle 修正=(A)PNG EXPAND + (B)draw_order 已就绪 + (C')abspos vertical-rl §10.3.7 静态位置（替代旧 clearance 精度）。实证复核 R148：54 失败=41 CSS-REF + 12 IMG-REF(退化区) + 1 路径缺失，clean single-session win 四重确证穷尽。下一步多会话=先独立修 abspos vertical-rl §10.3.7（4 个假通过，CSS-REF 可独立验证）→ 再叠加 PNG+draw_order 应 net≥0。 |
| 2026-06-15 | R150 abspos vertical-rl height:auto bug 精确定位（诊断，持平，已回退探针，工作区清洁） | 436/490 持平。写探针单测（复刻 abs-pos-non-replaced-vrl-006 结构）layout tree dump 定位：abspos span(is_absolute=true)几何=`x=240 y=80 w=80 h=320`，**h=320 错误**——CSS §10.3.7+writing-modes §7.1：vertical-rl 下 height:auto 应 shrink-to-fit 到内容(80px)而非填满 CB cross-axis(320)；spec 注释明确 height:auto=80、top:auto→static=160、bottom solved=80(160+80+80=320 ✓)。当前 taffy/converter 把 abspos auto height 当 cross-axis stretch。**不产生 session pass**：4 个 abs-pos-non-replaced-vrl 全是退化参考(非 RGBA PNG 双方退化)当前 4.5% 凑合通过，修 abspos height 不改通过数，仅 PNG fix 后有意义(需同提交=R149 bundle 组件 C')。独立验证：abspos-containing-block-outside-spanner(水平+显式尺寸,column-span CB 子问题)/flex-abspos-inset-nested(img aspect-ratio)均**非**此 bug 同源，无当前失败用例直接受阻塞。**fix 入口指引**：`fix_vertical_mode_abs_pos`(engine.rs:1133-1217)当前仅用 IFC fragment 修 top/bottom 全 auto 的 x/y，**不修 height**——下轮应在 vertical-rl 容器内对 height:auto abspos 子元素把 height 收缩到 fragment inline extent，配合 PNG fix 验证。clean single-session win 五重确证穷尽(R140/R144/R148/R149/R150)。 |
| 2026-06-15 | R158 large-font 死锁机制精确定位 + 失败聚类再分类（诊断，持平，无提交，工作区清洁） | 436/490 持平（make reftest-upstream 实测 436/490=89.0%，54 失败）。本轮独立复核全 54 失败 + 1 次 compute_final 显式高度容器补存 font_size 实验（死代码已回退）。**large-font 死锁机制现已精确定位（比 R125「三条路径」更精确）**：100px 文本位于 taffy 已测量的 height:auto 子容器（如 ifc-008 的 `#div1>div div`，content_height>1.0），被两条存储路径同时跳过——(a) remeasure 的 `content_height<1.0` 守卫(engine.rs:2883)排除 taffy 已测量块；(b) compute_final 的 R84 守卫(engine.rs:1628 `lines.len()>1\|\|!is_pure_ahem`)对多行/非纯 Ahem 提前 return。两路径都跳过→text_node_font_sizes 空→paint IFC(painter/text.rs:912 空 styles)按 16px 解析→100px 渲染成 16px。**关键冲突**：multicol-fill-auto 当前仅因部分文本节点 16px 错误默认值才通过；补存正确 font_size 改变其 paint IFC override→失败。故 large-font 修复须先让 multicol-fill-auto 在真实 font_size 下也正确。**compute_final 显式高度补存实验=死代码**：显式高度 `#div1` 无直接文本(只有块子元素)，compute_final 在 `!has_text_children` 早返回根本到不了补存点，已回退。**font-051 重新定性为 large-font**（非 font 简写 bug）：`span{font:serif}` 经 expand_font(shorthand/mod.rs:1572)正确判无效返回 vec![]，span 继承 100px Ahem；8.19% 差异来自继承 100px 经 paint IFC 死锁渲染成 16px。**min-max-size-table-content-box (36.34%) 重新定性为 inline-block ownership**（非 table bug）：TEST 的 7 个 table 正确 shrink-to-fit，但 REF 的 `.table{display:inline-block}` div 渲染全宽(w=793)——converter 把 InlineBlock 映射 taffy Block(mod.rs:266)被拉伸，adjust_inline_block_positions 的 ib_sizes 用 taffy 全宽 content_width 未做 shrink-to-fit；grid 内 inline-block 受 track 约束正确收缩(w=11)。inline-block width:auto shrink-to-fit 需测量子树 max-content=Phase A。其余复核候选(background-attachment=image 退化/baseline-007/008=baseline-export/ifc-011=image+vertical-align/count-computed=image+分布/clear-float-003=R114b 负 clearance/collapsing-001=R157 协调/border-padding-bleed=IFC/float-nowrap-hyphen=crbug1499290)全部印证结构性。clean single-session win 八重确证穷尽(R140/R144/R148/R149/R150/R155/R157/R158)。 |
| 2026-06-17 | R214 图片子资源加载（URL 导航路径，PNG，DC-13 第二个 P1 子项） | 439/490 持平（reftest 用本地文件不触发 URL 导航；影响产品导航 + DC-13/DC-11）。修复 goal doc DC-13 P1「图片子资源/ImageCache 未贯通」：`<img>` paint 已能生成 ImagePrimitive，但 fetch_url 不抓 `<img src>`、webview 不持有 ImageCache、render-foundation 无解码。**分层修复**（PNG 先行，JPEG/SVG 同模式后续）：① **render-foundation** 加 `png = "0.17"` dep + `pub fn decode_png_bytes(bytes) -> Result<ImageData,String>`（image_cache.rs，`EXPAND\|STRIP_16` 正确处理 palette/grayscale/RGB/RGBA 全 color type→RGBA，独立于 reftest 的 env-gated 版本故零 439 baseline 影响）+ `convert_png_buffer_to_rgba` + 2 单测（2×2 RGBA 解码、非法输入返 err）；② **zero-engine** 加 `pub fn extract_img_srcs(html)`（pipeline.rs，复用 zero_dom DOM 精确提取 `<img src>`，parallel to extract_stylesheet_hrefs）；③ **zero-webview** 加 `image_cache: ImageCache` 字段（new 初始化 default）+ 私有 `fetch_image_subresources(html, base_url) -> HashMap<u64,(f32,f32)>`（extract img srcs → `url::Url::join` 按 base 解析 → `http_client.get` 抓取 → `decode_png_bytes` → `image_cache.insert_with_key(ImageKey(simple_hash(abs)), img)`，键与 pipeline build_img_intrinsic_sizes + 渲染器查找一致）→ 返回 image_sizes，三条 fetch_url 分支（SW/cache/network）注入 `pipeline.set_image_sizes(image_sizes)` 后再 load_html（`<img>` 正确固有尺寸 DC-11）；data: URI 暂跳过、抓取/解码失败仅 warn 不阻断；暴露 `pub fn image_cache(&mut self)` 供下游渲染器绘制消费。**端到端测试**（webview_coverage）：mini-server 重构支持二进制内容（`HashMap<String,Vec<u8>>`，header/body 分开写），服务 3×2 纯绿 PNG + page，fetch_url 后断言 `image_cache` 含该图（键=simple_hash(abs url)）、尺寸 3×2、左上像素纯绿 (0,255,0,255)。13 webview + 2 decode + 1143 engine 测试全过。**意义**：图片子资源抓取+解码+缓存贯通 webview 层，`<img>` 经 URL 导航获正确固有尺寸 + ImageCache 就绪供浏览器渲染；浏览器 render_cpu/gpu 当前传 `None`（app_platform.rs:153），传 `Some(&mut webview.image_cache())` 是最后消费 hop（下一步）。make test 全绿、clippy --workspace --all-targets -D warnings 干净、fmt 干净。 |
| 2026-06-17 | R227 welcome 36px 偏移根因独立确证=taffy border-box 子坐标 vs painter 内容盒约定双重计入（并行 agent 已实现 engine.rs 修复，本条=只读验证） | 439/490 同源持平（本会话为文档+只读验证，无代码变更；并行 agent 正在 engine.rs:787-795 + reftest.rs `LAYOUT_DUMP` 探针实现修复，本条记录对其机制的独立确证与回归面分析）。承接 R226（welcome 顶部 36px 垂直偏移定位）。**独立只读验证确证 R226 假设——坐标系约定冲突致双重计入，经 painter 源码实证**：① 引擎提取（engine.rs:696-697）`x = layout.location.x; y = layout.location.y;` 直接取 taffy `Layout::location`，该值是**子 border-box 相对父 border-box** 的偏移（taffy 语义 `content_box_y = location.y + border + padding`，即 location 已含父 padding+border）；② painter（painter/mod.rs:271-272 `paint_node_in_rect` / 456-457 `paint_node`）`child_offset_x = abs_x + padding_left + border_left; child_offset_y = abs_y + padding_top + border_top;` 后 `paint_node(child, ..., child_offset_x, child_offset_y)`，子绝对坐标 = child_offset + child.x，即 painter 期望 child.x 是**相对父内容盒**（它在 child_offset 上已加一份父 padding+border）；③ **约定冲突**=location（border-box 相对 border-box）≠ painter 期望（内容盒相对），painter 加一份 + child.x(=location) 已含一份 → **每代有 padding/border 的祖先把整棵子树多偏移一份**；④ welcome 实算 body(`*{padding:0}`)→.page(padding-top 20)→.hero(padding-top 16)→.hero-accent：chromium 内容 y=20+16=36（实测 ✓）；双重计入下 page padding 计 2 份多 20、hero padding 计 2 份多 16=合计 +36 → ZW y=72（chromium 36，差 36）✓ 与 R226 实测精确吻合，无任何补偿（全量双重计入）。**并行 agent 修复方向正确**（engine.rs:787-795）：HorizontalTb 下对非 abspos/fixed 的块/inline-block 子节点 `child.x -= content_x; child.y -= content_y;`（content_x/y=父 border+padding）把 border-box 相对换算为内容盒相对消除双重计入；嵌套多层 padding 每代精确减一次（page→hero→hero-accent 各减本代 content_offset），数学闭合（hero-accent abs y=36 = chromium）。**回归面分析**：(a) **同源 reftest（439/490）预期~中性**——双重计入对 test/ref 同结构时双侧同偏移相互抵消仍匹配，仅当 test/ref 用不同机制（padding vs margin/abspos）达成同视觉时才翻转，多见 FAIL→PASS 正向（消除 test 侧偏移使二者对齐）；无 padding/border 祖先（content_x/y=0）本修复为 no-op；(b) **产品 smoke（welcome）**——36px 双重计入消除→内容上移对齐 chromium→welcome 28% diff 预期显著下降（DC-13 真杠杆，印证 R226，纠正 R174「welcome 剩余~28% 是 fontdue 字体噪声无 clean bug」的旧结论）；(c) **chromium Oracle**——本 bug 是**纵向偏移**类（区别于 R225 证伪的横向 advance-width），影响所有含 padding/block 嵌套的真实页，183-case 1-3% 噪声桶中含纵向偏移分量的用例应受益，需重跑 cross-validate 量化。**新 bug 类意义**：「taffy 提取层 border-box 约定 vs painter 层内容盒约定」坐标系不一致，是区别于 advance-width 死胡同（R221-R225）与结构性 multicol/writing-mode 阻塞（R109/R113/R131）的**第三类**根因；特征=同源不可见（双侧抵消）但 chromium Oracle 可见，正是 DC-14 独立 Oracle 揭示的真实缺口；修复为局部后处理（非多轮结构）潜在高杠杆。**遗留/下一步**：(a) 并行 agent 验证 make reftest 零回归 + welcome smoke 改善 + cross-validate 量化（本条不代其执行）；(b) 核查**同一约定不一致是否存在于 inline IFC 子节点路径**（paint IFC fragment 定位 painter/mod.rs:501-502 content_x/y + text.rs）——welcome 首个非背景是 block .hero-accent 故本修复已覆盖主偏移，但嵌套 padding 下文本行纵向位置仍可能受影响；(c) **垂直书写模式**本修复显式跳过（仅 HorizontalTb），vertical-rl 等价双重计入需轴交换路径另算（R109/R142 谱系已知）；(d) 复核 abspos/fixed 跳过自洽性（其坐标语义与 painter 双计补偿是否在所有嵌套深度成立）。 |
| 2026-06-17 | R226 welcome 28% diff 真因定位=顶部 36px 垂直偏移级联（非字体/度量/背景） | 439/490 持平（诊断，无代码变更）。承接 R225（advance-width 证伪）。对 welcome ZeroWeb CPU vs chromium Oracle PNG 做**区域 diff 分析**：① y 带密度——hero 顶 9.8% → 底部 grid/flex 区（cards/shortcuts/quick-links/footer）41-43%（底部远高）；② **垂直内容起点**：ZW first-non-bg at y=72，CH y=36 → **ZW 内容低 36px**；content y-range ZW[72,583] vs CH[36,534]（ZW 长 49px）；③ 扫描线 y=350：ZW 全内容、CH 全背景空 → 内容 y 错位。**排除项**：body bg 正确（角点 ZW=CH=(244,246,248)）、advance-width 已证伪（R225）、AA 基准排除光栅化。**根因**：body→.page(margin:0 auto,padding 20px 40px 24px)→.hero(padding 16px 0 20px) 链中 margin 折叠/padding 累加与 chromium 不一致（36px≈page padding-top 20 + hero padding-top 16），致整页下移 36px 级联。**策略意义**：welcome 28% diff 是**布局垂直定位 bug**（可定位可修复），非字体噪声——DC-13 产品 smoke 真杠杆=修此 margin/padding 偏移。证据 `evidence/welcome-region-analysis-2026-06-17.txt`。下一步=dump body/page/hero 盒几何定位 36px 偏移确切来源并修复。 |
| 2026-06-17 | R225 advance-width 非噪声根因（product-smoke + reftest-oracle 双实验证伪） | 439/490 持平（实验已回退）。R224 同源 -3 后，本轮做**决定性证伪实验**：re-apply DejaVu advance 表，测 chromium 一致率变化。**双实验均证伪 advance-width 是噪声根因**：① **reftest-oracle**（26 共享 case）：strict true-pass 11 vs 11、median z_vs_chr 1.06% vs 1.07%、0 case 改善/恶化（Ahem 用例 is_ahem 特例=font_size，estimate 表不适用故无影响）；② **product-smoke（非 Ahem，advance 表真正起作用的场景）**：welcome 28.34%→28.31%（Δ-0.03%）、wintertc 25.11%→25.14%（Δ+0.03%）—— **零实质变化**。**结论**：`estimate_char_width` 改进**对 chromium 一致率无影响**（无论 reftest 还是产品 smoke）。**机制推断**（待证实）：paint glyph x 定位走真实 fontdue shaping 而非 estimate（estimate 仅影响 layout 换行决策，glyph 位置视觉主项由 paint fontdue 决定，故改 estimate 不动 diff）。**重大策略意义**：推翻 R221/R222 「advance-width 是 183-case 噪声杠杆」假设——advance-width plumbing（R223 trait seam 留存无害）是**死路**，勿再投入。28% 产品 smoke diff 真因在别处（line-height/baseline、box 定位、或 paint fontdue-vs-chromium 光栅化差异，待定位）。验证：回退后 reftest 439/490 恢复、product-smoke 恢复。 |
| 2026-06-17 | R224 estimate_char_width 实测表精化实验（回退，净 -3 回归证否单点捷径） | 439/490 持平（实验已回退，仅留教训注释）。承接 R222/R223。本轮尝试**捷径**：用 DejaVu Sans 实测 advance 比率表（W=0.99/i=0.28 等 94 项 ASCII）直接替换 `estimate_char_width` 的固定倍数（字母 0.55/数字 0.5/标点 0.4/空格 0.25），避开 R2-R5 跨 crate plumbing。**实测全量 reftest 439→436 净 -3 回归**（非 Ahem 用例换行点翻转），按设计成功标准（须持平或 net≥0）**回退**。**关键教训**：estimate_char_width **并非纯自源中性**——reftest 的 test 与 ref 虽同用 estimate，但文本结构不同时换行点敏感度不同（如一个空格宽从 0.25→0.3179 致某用例恰好溢出换行），单独扰动 estimate 会破坏同源对齐。**结论**：advance-width 真实修复**不能走单点改 estimate 捷径**，必须完整接入 FontLoader（R223 plumbing R2-R5：layout+paint+intrinsic 三处同源替换 + TextRun 携带 font_id 解析），保证 test/ref 与 chromium 三方度量同源。本次回退保留教训注释（inline/mod.rs estimate_char_width doc），AdvanceSource trait seam（R223）保留待 R2 接入。验证：回退后 reftest 439/490 恢复、make test 全绿。下一步=R2 签名注入（&dyn AdvanceSource 进 IFC，默认 EstimateAdvance，零行为变更）而非单点改 estimate。 |
| 2026-06-17 | R223 advance-width plumbing R1（AdvanceSource trait + 设计 RFC，零行为变更） | 439/490 持平（R1=行为中性 seam，默认实现等价 estimate_char_width）。承接 R222 决定性诊断（逐字符 ±44-98% 误差）。本轮启动 advance-width plumbing 多轮工作：① 写设计 RFC `docs/goal/rendering-compat/advance-width-plumbing-design.md`——核心 = 依赖反转（layout-engine 定义 `AdvanceSource` trait，`EstimateAdvance` 默认实现=estimate_char_width，zero-engine 注入 FontLoader-backed 实现），5 轮渐进（R1 trait seam / R2 签名注入 / R3 真实 advance 启用 / R4 intrinsic+paint / R5 oracle 量化），含 R125 IFC 三路径死锁风险评估与缓解（source 是纯度量函数不涉 font_size 解析，三路径同源实例→度量一致）；② 实现 R1——`AdvanceSource` trait + `EstimateAdvance` 默认 impl（inline/mod.rs），`measure(ch, font_id, font_size, is_ahem)` 委托 estimate_char_width；③ 等价性单测 `test_estimate_advance_matches_estimate_char_width`（验证 trait 默认实现与 estimate_char_width 逐字符完全等价 + font_id 为 None/Some 均等价）。**零调用点改动、零行为变更**（seam 就位待 R2 注入 IFC 签名）。**验证**：make test 12230 passed/0 failed、clippy/fmt 干净、reftest-upstream **439/490 持平**（证 R1 行为中性）。**意义**：建立 advance-width 真实度量接入的依赖反转 seam（layout-engine 不向下耦合 render-foundation FontLoader），为 R2-R5 渐进替换 estimate_char_width 铺路，瞄准 R221 的 183-case 系统性噪声桶。下一步=R2（IFC 函数签名加 `&dyn AdvanceSource` 参数，默认传 EstimateAdvance，内部调用改 source.measure）。 |
| 2026-06-17 | R222 advance-width 估计误差诊断（advance-width plumbing 数据依据） | 439/490 持平（新增诊断测试 + 证据，无行为变更）。承接 R221 识别的 183-case 1-3% 系统性噪声桶。新增 `diag_advance_vs_estimate_systematic_error` 测试（render-foundation font/loader.rs）：加载系统字体，对比 `FontLoader::measure_advance`（fontdue 真实度量）与 `estimate_char_width` 启发式（字母 0.55×fs/数字 0.5/标点 0.4/空格 0.25）。**实测逐字符误差极大（±44%~98%）**：W 实际 0.989×fs（estimate 0.55 欠估 44%）、i/l 实际 0.278（过估 98%）、m 0.974（欠估 44%）、t 0.392（过估 40%）、f 0.352（过估 56%）、H 0.752（欠估 27%）、数字 0.636（欠估 21%）、标点 0.318（过估 26%）；总和部分抵消 -6.9% 但**逐字符累积定位全错**（"Will" 按 0.55 均匀 vs 实际 W 近全宽 i/l 极窄）。**证实** R221 推断：layout IFC + paint IFC + intrinsic_sizing 三处 estimate_char_width 是 183-case 系统性噪声根因（非字体光栅化，AA 基准已排除）。**关键发现**：paint/text.rs:410/443（list marker 定位）已用 estimate（paint 有 FontLoader 可直接修）；`FontLoader::measure_advance` 已存在（loader.rs:289），缺的是把它接入 layout-engine IFC。证据 `evidence/advance-width-estimate-error-2026-06-17.txt`（含复现命令）。**修复路径（多轮）**：engine 预解析 font-family→FontId 建 advance 源 → 传入 layout IFC + intrinsic_sizing + paint IFC 三处替换 estimate_char_width。self-source 中性（同源 439 不变）但降 chromium 噪声。make test 12229 passed/0 failed、clippy/fmt 干净。 |
| 2026-06-17 | R221 DC-14 可信通过率量化分析（chromium Oracle 视角，策略重定向） | 439/490 同源持平（分析型，无代码变更）。基于 06-17 全量 cross-validate 数据（R165–R180 修复后），以 z_vs_chr（ZeroWeb-test vs chromium-test）为唯一可信指标重算：**严格真通过率 = 188/475 = 39.6%**（z_vs_chr<1%），对比同源 89.6%——同源**严重高估**。分布：<0.5%=97 / <1%=188 / 1-3%=183 / 3-8%=67 / ≥8%=37。**关键发现**：① **183 case 在 1-3%**（系统性布局/字体噪声）= **最大杠杆**——AA 基准已证非光栅化，是布局定位（estimate_char_width 近似 vs 真实 advance），降此噪声是把真通过 188→370+ 的最短路径（R195/R196 定性，self-source 中性故同源 439 不动）；② **116 case 假失败**（self-fail 但 chr<5%）= ZeroWeb-test 实际接近 chromium，**同源 ref 怪异**（如 vrl-004 同源 7.09% vs chr 5.08%），同源判 FAIL 是 reference 的错非 ZeroWeb——**同源 reference 双向不可靠**，DC-14 独立 Oracle 是唯一可信判定；③ 37 case ≥8% = 结构性聚类（clean-win 穷尽）。**策略重定向**：DC-2~5 达标路径 = 降系统性布局噪声（advance-width/line-breaking 精度，多轮 self-source-neutral）+ 修结构性聚类（多轮）。证据 `evidence/dc14-credible-passrate-2026-06-17.txt`（含复现命令）。DC-14 独立 Oracle 基建已就绪，严格容差已用，分母待补全上游全量（475/490 子集）。 |
| 2026-06-17 | R220 DC-9 真实范围纠正（clip 为 no-op，GPU 缺口仅 transform/filter/blend 三项，docs-only 治理） | 439/490 持平（docs-only，无代码/reftest 变更）。承接多轮对 DC-9 GPU「丢弃 4 图元（transform/clip/filter/blend）」的认知，本轮 grep 实证纠正。**发现**：engine 在生产路径**从不生成 `ClipPrimitive`**（`add_clip` 全仓库 0 处非测试调用）——overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect`（painter/mod.rs:292/553/566/590/690 + text.rs:1090 + effects.rs:797）**预烘焙进图元几何**（fills/glyphs/strokes 等的坐标被裁到 overflow 容器 rect 内），故 `RenderPrimitives.clips` 在生产中**恒空**。因此 R211（commit 2af1141）所记「render_full_scene_gpu drops clip」**实为 no-op**（无 clip 可丢，dropping 空列表），DC-9 的 ClipPrimitive 项在 CPU（render_draw_order 的 DrawOp::Clip→apply_clip）与 GPU 两路均**空谈满足（vacuous）**。**真实 DC-9 缺口仅 transform/filter/blend_mode 三项**：engine 在 `paint/painter/effects.rs:266/289`（FilterPrimitive，CSS `filter:`/backdrop-filter）+ `:313`（BlendModePrimitive，mix-blend-mode）+ `paint/helpers.rs:168/184`（TransformPrimitive，CSS transform）生成，GPU 全量路径（render_full_scene_gpu）collect 顶点时**未读 primitives.transforms/filters/blend_modes**故静默丢弃。**修复路径（多轮）**：这三项需 **ping-pong 双纹理后处理架构**——wgpu 不能在同一 render pass 读写同一纹理，filter（区域采样变换如 blur/opacity）、transform（区域反向采样仿射）、blend（与 backdrop 合成）均需 read 源区域+write 目标。GPU 已有 `headless_texture` offscreen 渲染目标（mod.rs:93/125/154），ping-pong 基建部分就绪，差第二张纹理 + post-process WGSL pipeline + per-region scissor。**优先级低**：transform/filter/blend 在 reftest/静态内容中**低频**（仅显式 CSS 触发），GPU 路径非 reftest load-bearing（reftest + product-smoke 走 CPU）。**本轮纠正价值**：避免后续会话在 no-op clip 上浪费、重定 DC-9 收尾为「3 项低频 + ping-pong 多轮」、对齐 goal doc 治理「状态须诚实」。下一步候选=DC-9 GPU ping-pong 地基（filter:opacity 最简先建）/ DC-14 chromium-oracle 严格容差默认接线 / DC-13 产品 smoke 端到端证据持久化。 |
| 2026-06-17 | R219 SVG fetch_url 端到端验证测试（R215-R218 全链路最后验证拼图） | 439/490 持平（新增测试，零行为变更）。R218 加了 SVG 解码（decode_svg_bytes + decode_image_bytes 内容嗅探路由），但仅在 render-foundation 单测层验证；webview URL 导航路径（fetch_url→fetch_image_subresources→decode_image_bytes）未验证。新增 `test_fetch_url_loads_svg_image_subresource`（webview_coverage）：MiniServer 服务 4×3 纯绿 SVG + page，fetch_url 后断言 image_cache 含栅格化结果（键=simple_hash(abs url)、尺寸 4×3、绿色 G>200 + alpha=255）。**意义**：R215（browser render 消费 image_cache）+ R214（PNG fetch→cache）+ R216（JPEG）+ R218（SVG）+ 本测试（SVG fetch→cache）共同闭合「fetch→decode PNG/JPEG/SVG→image_cache→browser render→真像素」全链路验证。make test 12228 passed/0 failed、clippy/fmt 干净。下一步=DC-13 产品 smoke 持久化证据 / DC-9 GPU 4 图元 / DC-14 chromium-oracle 严格容差默认接线。 |
| 2026-06-17 | R218 SVG 解码统一到 render-foundation（DC-13 SVG 栅格化全路径） | 439/490 持平（reftest 路径 load_svg_file 委托后行为不变）。goal doc DC-13 要求「PNG/JPEG/WebP 基础解码和 SVG 栅格化」。reftest 路径早有 `load_svg_file`（resvg+tiny-skia），但 webview/browser URL 导航路径的 `decode_image_bytes` 对 SVG 返 unsupported——浏览器导航含 `<img src=logo.svg>` 的真实页面（WinterTC 14 logo 中 11 个 SVG）Logo 不渲染。**修复**：① render-foundation 加 `resvg`(workspace)+`tiny-skia` 依赖 + `pub fn decode_svg_bytes(bytes)`（resvg usvg 解析→按 SVG 内在尺寸 tiny-skia pixmap 栅格化→RGBA，过大尺寸 pixmap 分配失败自然兜底）；② `decode_image_bytes` 扩展 SVG 分支——`looks_like_svg` 嗅探 UTF-8 文本（跳 BOM/空白后 `<svg`/`<?xml` 起始）路由到 `decode_svg_bytes`；③ reftest `load_svg_file` 委托 `decode_svg_bytes`（同 R217 去重），移除 wpt-runner 的 resvg/tiny-skia 直接依赖（load_svg_file 唯一用户，依赖图精简）。**测试**：render-foundation decode_tests +2——`decode_svg_bytes_green_4x3`（含 `<?xml` 声明的 4×3 纯绿 SVG 往返，断言 G>200 + alpha=255）、`decode_svg_bytes_invalid_returns_err`（非 SVG XML→err）；`decode_image_bytes_dispatches_by_magic` 加 SVG 路由断言（现四分发 PNG/JPEG/SVG/unsupported）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test 12227 passed/0 failed、reftest-upstream 439/490 持平。**意义**：DC-13 三种图片格式（PNG/JPEG/SVG）在三条渲染路径（reftest / webview fetch_url / browser render_cpu）全部统一到 render-foundation `decode_image_bytes` 单点；浏览器经 URL 导航现可加载并渲染 SVG Logo（WinterTC logo.svg 等真实场景）。下一步=DC-13 产品 smoke 端到端证据（DONE#11 5 真实网站 / WinterTC Logo 经浏览器路径验证）或 DC-9 GPU 4 图元（transform/clip/filter/blend）。 |
| 2026-06-17 | R217 JPEG 解码合并去重（清理 R216 造成的重复） | 439/490 持平（reftest 用本地文件，JPEG 解码逻辑变更对像素输出零影响——L16 JPEG 在 WPT reftest 实质不出现，RGB24/L8/CMYK32 转换两路径本就等价）。R216 在 render-foundation 落地 tested `decode_jpeg_bytes` 后，reftest 路径 `reftest.rs:load_jpeg_file`（~55 行）的独立 JPEG PixelFormat→RGBA 转换与之重复（且 L16 处理不一致：reftest `(px[0]\|px[1]<<8>>8)` vs R216 干净高字节）。**修复**：`load_jpeg_file` 改为读文件→委托 `zero_render_foundation::image_cache::decode_jpeg_bytes`，reftest 与 webview/browser URL 导航路径现共用同一解码器（单点 tested）。移除 wpt-runner 的 `jpeg-decoder` 直接依赖（load_jpeg_file 是唯一用户，依赖图精简）。保留 `load_png_file` 的 `ZERO_PNG_EXPAND` 诊断门控与 `load_svg_file`（resvg）不动——非本轮变更遗留。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test 12225 passed/0 failed、reftest-upstream 439/490 持平。**意义**：三条渲染路径（reftest / webview fetch_url / browser render_cpu）JPEG 解码统一到 render-foundation 单点，消除维护负担与潜在不一致；DC-13 图片解码一致性提升。下一步=SVG 解码统一（reftest 已有 resvg，webview/browser 路径缺）或 DC-13 产品 smoke 端到端证据（DONE#11）。 |
| 2026-06-17 | R216 JPEG 图像解码扩展（DC-13 PNG/JPEG 基础解码第二步） | 439/490 持平（reftest 用本地文件不走 URL 导航，零影响）。goal doc DC-13 要求 PNG/JPEG/WebP 基础解码，R214 落地 PNG，本轮补 JPEG。**修复**：① render-foundation 加 `jpeg-decoder = "0.3"`（MIT/Apache-2.0 纯 Rust）+ `pub fn decode_jpeg_bytes(bytes)`（L8/L16/RGB24/CMYK32 全 PixelFormat→RGBA，CMYK 按 Adobe 倒置 K 惯例转 RGB）+ `convert_jpeg_pixels_to_rgba` 纯函数；② **格式分发** `pub fn decode_image_bytes(bytes)`——按**魔数字节**嗅探（PNG `\x89PNG` / JPEG `\xFF\xD8\xFF`）路由，比 URL 扩展名可靠（URL 可能无扩展名/扩展名错误），未知格式返 unsupported err；③ webview `fetch_image_subresources` 改调 `decode_image_bytes`（原 decode_png_bytes）→ 同一路径现处理 PNG+JPEG，warn 文案更新。**测试**：render-foundation decode_tests 5 项——`convert_jpeg_pixels_to_rgba_rgb`/`_grayscale` 纯函数断言、`decode_jpeg_bytes_green_4x3`（PIL 生成 4×3 纯绿 JPEG quality 95 fixture，断言绿色主导 G>200/R<50/B<50 + alpha=255，容 JPEG 有损非精确等值）、`decode_jpeg_bytes_invalid_returns_err`（魔数+无效正文→err）、`decode_image_bytes_dispatches_by_magic`（PNG ok/JPEG ok/未知 unsupported）。fixture `crates/render-foundation/src/testdata/green_4x3.jpg`（635B）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test **12225 passed/0 failed**、reftest-upstream 439/490 持平。**意义**：DC-13 图片基础解码 PNG+JPEG 就绪；浏览器经 URL 导航现可加载并渲染常见位图格式。下一步=SVG 栅格化（WinterTC logo.svg）或 WinterTC Logo 端到端产品 smoke 证据（DC-13 验收）。 |
| 2026-06-17 | R215 浏览器渲染路径消费 webview ImageCache（DC-13 P1 图片子资源最后消费 hop） | 439/490 持平（reftest 用本地文件不走 URL 导航，image_cache 恒空，`Some(&空)`≡`None` 零回归）。承接 R214 标注的「下一步」。R214 已打通 fetch→decode→image_cache（webview 层），但浏览器 `render_cpu`/`render_frame` 仍传 `None`（app_platform.rs:194 CPU / :153 GPU），图元→渲染器最后一跳断开。**修复**：app.rs 加 `use zero_render_foundation::image_cache::ImageCache`；`render_cpu`（CPU 路径）与 `render_frame`（GPU 路径）在 `render_full_scene[_gpu]` 调用前用**不相交字段借用**取活跃标签页 webview 的 image_cache——`match self.shell.active_tab_id() { Some(id) => self.webviews.get_mut(&id).map(|wv| wv.image_cache()), None => None }`（self.webviews / self.font_loader / self.glyph_cache 为不同结构字段，Rust 借用检查器允许同语句并存），传 `Some(&mut ImageCache)` 替代 `None`。**测试**：新增 `#[cfg(test)] pub fn render_full_scene_with_webview_for_test`（与 render_cpu 同场景装配但返回 FrameBuffer，mirror 现有 `render_scene_for_test` 模式）+ 差异法测试 `render_path_consumes_webview_image_cache`——基线（image_cache 空）渲染断言目标颜色计数 0（缓存 miss 不绘制，证「空缓存≡None」语义），填充 `ImageKey(simple_hash(src))`（键与 engine text.rs:611 一致）后渲染断言 >0（图片经浏览器渲染路径被消费）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、cargo fmt 干净、make test 全绿（新增测试通过）、`./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest-upstream` 实测 439/490 持平。**意义**：`<img>` 经 URL 导航全链路贯通——抓取(R214)→解码(R214)→image_cache(R214)→**浏览器渲染消费(R215)**→renderer 绘制真像素，goal doc DC-13「图片缺失不得被 alt/占位 glyph 静默替代」在浏览器层落地。下一步=JPEG/SVG 解码同模式扩展 + WinterTC Logo 端到端产品 smoke 证据（DC-13 验收）。 |
| 2026-06-17 | R213 外链 stylesheet 加载（URL 导航路径，P1 缺口修复 + 端到端测试） | 439/490 持平（reftest 用本地文件不触发 URL 导航路径；本修复影响产品 URL 导航 + DC-13）。修复 goal doc P1「外部样式表加载缺失」：`collect_stylesheets`（pipeline.rs:494）只收调用方 CSS + 文档内 `<style>`，**不抓 `<link rel=stylesheet>`**；fetch_url 三条成功路径（SW intercept / HTTP cache / network）都调 `load_html(&html, None)`。**分层修复**（engine 做 DOM 提取、webview 做 URL 解析+网络，保持 engine 不耦合网络）：① zero-engine 暴露 `pub fn extract_stylesheet_hrefs(html) -> Vec<String>`（pipeline.rs，复用 `zero_dom::parse_html` 解析 DOM 精确提取，`rel` 空白拆分后任一 token `eq_ignore_ascii_case("stylesheet")` 即匹配，覆盖 `rel="stylesheet preload"` 等写法，跳过空 href）；② zero-webview `fetch_url` 加私有 `resolve_external_css(html, base_url)`（`url::Url::parse(base).join(href)` 解析相对/绝对 href，`http_client.get` 逐个抓取，合并为单 CSS 字符串，抓取失败仅 `tracing::warn` 不阻断），三条分支改 `load_html(&html, Some(&external_css))`。**端到端测试**（webview_coverage.rs，内联 std mini-server）：① `test_fetch_url_loads_external_stylesheet`——page.html 仅外链 style.css（`#x{background:rgb(255,0,0)}`，无任何内联红），fetch_url 后断言渲染含纯红 (255,0,0) fill（证明外链 CSS 抓取+级联生效）；② `test_fetch_url_external_stylesheet_missing_does_not_break`——外链 404 时导航不崩溃。12 webview 测试全过，engine lib 1143 测试全过。**意义**：URL 导航路径现可正确加载外链 CSS（morning.work `/article.css`/`/styles/github.css`、welcome 等真实静态页依赖），DC-13「URL 导航必须加载外链样式表」子项打通；图片子资源/ImageCache（DC-13 另一 P1）仍待。make test 全绿、clippy --workspace --all-targets -D warnings 干净、fmt 干净。 |
| 2026-06-17 | R212 离线 fixture HTTP 服务器（std-only example，使能外链 CSS/图片离线测试） | 439/490 持平（新增 example，不影响 reftest）。用户明确要求「本地静态资源+Rust web server」以离线测试 URL 导航 + 外链 CSS/图片加载。新增 `crates/net/examples/fixture_server.rs`：**std-only** HTTP/1.0 服务器（`TcpListener` 非阻塞轮询 shutdown，按扩展名映射 Content-Type 含 html/css/js/svg/png/jpg/webp/woff2/ttf 等，路径穿越 `..` 段过滤 + canonicalize `starts_with` 双保险，404/405 响应，每连接一线程）。`--root/--port` 参数，`pub fn serve(root,port,shutdown)` 供测试/嵌入。**零新依赖、零 workspace 改动**（example 自动发现）。配单元测试：临时目录建 article.css+index.html，启动服务器（OS 分配 0 端口），GET CSS 断言 200+text/css+内容，GET 缺失断言 404，GET `../../etc/passwd` 断言穿越被拒 404。**用途**：后续可离线驱动 ZeroBrowser/WebView 导航 `http://127.0.0.1:<port>/` 验证 P1 缺口「外部样式表加载缺失」（preload.rs:332 明确忽略 `rel=stylesheet`）与「图片子资源/ImageCache 未贯通」。**附带 GPU 调查**（为 DC-9 后续工作铺路）：blur_pipeline（pipeline.rs:703）已创建但从未接线（self.blur_pipeline 无任何 draw 调用）；headless_texture 含 `TEXTURE_BINDING` 可采样；但 blur/filter 后处理需 ping-pong 双纹理编排（wgpu 不能同 pass 读写同一纹理），是 render_full_scene_gpu 缺的架构=多轮。make test 全绿、clippy -D warnings 干净、fmt 干净。此为外链 CSS 加载使能 + DC-9 GPU 后处理铺路的第一步。 |
| 2026-06-17 | R211 DC-9 GPU 图元覆盖状态诚实化纠正（诊断，治理强制纠正矛盾，docs-only） | 439/490 持平（无代码变更，reftest/product-smoke 均走 CPU 路径不受影响）。核查发现 master.md DC-9 表对 Transform/Clip/Filter/BlendMode 标 ✅「简化处理」属**虚假声明**——浏览器实际 GPU 路径 `render_full_scene_gpu`（gpu/renderer/mod.rs:651，app_platform.rs:149 调用）**完全丢弃这 4 种图元**（仅 collect+draw 9 种；gpu/renderer/ 仅 mod.rs+tests.rs，无其它 GPU 路径处理它们；GPU tests 609-768 也未覆盖）。表中原「scissor rect 全局裁剪/CPU 后处理对齐」描述的是 per-box 路径 `render_scene_with_clip_scaled`（仅支持**单一** clip_rect scissor），非全量 GPU 路径。**共同根因**：全量批次路径展平了场景、丢失元素子树关联——clip/transform/filter/blend 均作用于子树，扁平图元列表无法应用。故 DC-9 真正未达标（4 项修正为 ⚠️），违反 DC-14/DC-9「GPU 非 passthrough、不丢弃图元」硬约束。**CPU 路径 DC-8 经核验实处理全部 13 种**（cpu/mod.rs:163-179 typed-bucket + 246-262 draw_order 双模式），DC-8 ✅ 准确。按 goal doc 治理规则（line 757：发现文档矛盾必须先纠正），已将 DC-9 表 4 项改 ⚠️ 并加纠正说明。修复=多轮架构（paint 侧把 transform/clip 烘焙进已收集的 fill/glyph 顶点，或 GPU 全量路径携带子树结构；filter=post-processing WGSL pass，blend=blend equation）。**这是 DC-9/DC-14 通向 DONE 的明确未完成项**，区别于已穷尽的 reftest 平台期。 |
| 2026-06-17 | R210 compute_final 多行存储 + multicol 守卫实证（诊断，净 +0 不可启用，无提交，工作区清洁） | 439/490 持平（全量 make reftest-upstream 双跑实测 default 439/51 vs gate 439/51，**净 +0**）。承接 R209（PHASEA_MULTILINE 净负，疑 R198 ancestry-guard 墙），测未被试组合：compute_final 多行存储 + `!in_multicol` 守卫（in_multicol 经新增递归参数 `child_in_multicol=in_multicol\|\|root.is_multicol` 透传）+ text.rs stored frag `y=f.y+line.y`（行内相对→行盒绝对，多行必需）。**仅 2 翻转**：✅ ifc-008 8.18%→PASS 0.00%（compute_final 正确存 node39 的 2 行 100px Ahem）；❌ multicol-fill-auto-001 0.63%→9.15%。**根因精确定位（推翻「multicol 容器 ancestry」假设）**：CFDEBUG 探针（reftest 双趟，正确加载 /fonts/ahem.css）显示 multicol-fill-auto 存了 node25(10 行)/node28(5 行)，**in_multicol=false**。逐文件分析：test=1 个 multicol div，**ref=2 个 `<div float:left width:10em>`（非 multicol，用 float 模拟列）**——回归源是 ref 的 float div（合法非 multicol），`!in_multicol` 守卫**无法触及**。**v_offset 语义墙**：default ref float div 走 paint IFC（baseline_fs=font_size），gate 走 stored（v_offset=0 Ahem），两路径对同一多行 Ahem baseline 计算不同→ref/test 差 font_size/行→9.15%。实测 v_offset=font_size 反破坏 font-051(16.67%)/ifc-008(8.33%)，**stored 单行/多行 v_offset 语义不可统一**，印证 R125 三路径死锁。**方法学纠正**：product-smoke 单趟**不加载外链 CSS**（base_dir=None）→ multicol-fill-auto 的 /fonts/ahem.css 不解析→is_pure_ahem=false→不存储；故 R209 product-smoke 看「不存储」是外链 CSS 缺失假象，**涉及外链 CSS 的用例必须用 reftest 双趟诊断**。结论：净 +0 不可默认启用；ifc-008 可被正确存储并 PASS，但解锁需先统一 stored 与 paint IFC 多行 baseline 语义（R125/R198 同墙，结构性多轮）。证据 `evidence/r210-multiline-multicol-guard-2026-06-17.txt`。无代码变更（实验已回退回 R207 e0e2689 干净态）。 |


---

## 下一步

> R305–R323 已确认结构性 plateau（见上方「综合裁决」）。下列为**多会话**架构方向；单会话 rally 已无 lever。

### 需用户决策（卡点）

- [ ] **多会话架构承诺 vs 接受 plateau**：438/490 loose / 295/490 strict / ~36% Oracle 是诚实基线。剩余提升需 Phase A IFC 统一 / Phase 2 嵌套 multicol / baseline 合成 或 taffy 升级，均为多会话工程。R314 已飞书通知。

### 若推进多会话架构（按依赖序）

1. **Phase A IFC 统一**（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)）— 解 large-font（ifc-008/009/011）+ welcome/morning.work 文本度量残余。R207 narrow 已证 font-051 +1 可行；需多轮 set-diff 收敛 broad 应用 + 守 multicol-fill-auto 反向依赖（R198 墙）。
2. **Phase 2 嵌套 multicol fragmentation**（[`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)、[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)）— 解 multicol-breaking（css-multicol 最大失败聚类）。R319 证纯 inline 迁移零增益，真实价值在嵌套 / 混合内容碎片化。
3. **baseline-export 真修复** — taffy 0.8+ baseline_overrides（R304 DEFER 升级）或自建 inline-level-box baseline 合成；解 flexbox-baseline / multicol-baseline 聚类。
4. **DC-9 blend_mode** — paint-isolation 架构（offscreen 子树渲染 + source/dest 双纹理 blend pass），低 reftest footprint（~2-4 案）。

### 已 ruled out（勿以单会话重试）

near-pass(R307) / POLLUTED hunt 三趟复核 R299–R309 + R311 + R329 / fresh-xval(R311) / Phase A 4 路 font_size(R125–R206) / multicol paint 侧(R157–R317) / balance 二分(R199–R322) / column-aware IFC 纯 inline(R319) / baseline-export 3 机制(R266–R316) / advance-width(R225–R320) / blend post-process(R278) / font-weight -Bold(R229b) / taffy 升级(R304)。

### 已完成里程碑（参考，非当前活跃）

- M1–M9 基础设施 + 渲染器图元覆盖 + 浏览器消费 + 布局正确性 + 高级视觉效果：**已完成**（见下方「里程碑完成状态」「Done Criteria 进度」）。
- M10 上游 WPT reftest：基础设施完成，通过率 plateau（438/490 loose），达标需上述多会话架构。

---

## 最近轮次详细记录（R313–R332；R312 已归档至 [`archive/rounds-r312.md`](./archive/rounds-r312.md)、R311 已归档至 [`archive/rounds-r311.md`](./archive/rounds-r311.md)、R310 已归档至 [`archive/rounds-r310.md`](./archive/rounds-r310.md)、R309 已归档至 [`archive/rounds-r309.md`](./archive/rounds-r309.md)、R308 已归档至 [`archive/rounds-r308.md`](./archive/rounds-r308.md)、R307 已归档至 [`archive/rounds-r307.md`](./archive/rounds-r307.md)、R305–R306 已归档至 [`archive/rounds-r305-r306.md`](./archive/rounds-r305-r306.md)、R304 已归档至 [`archive/r304-taffy-upgrade-deferred.md`](./archive/r304-taffy-upgrade-deferred.md)、R303 已归档至 [`archive/r303-dc9-gpu-primitive-audit.md`](./archive/r303-dc9-gpu-primitive-audit.md)、R142–R302 已归档至 [`archive/rounds-r142-r302.md`](./archive/rounds-r142-r302.md)）

### R312 — baseline-export 双侧探针精确定位：inline-flex 容器 taffy_baseline 错值 + multicol 项 None（read-only 探针，基线 loose 438/490 / strict 295/490 持平）

（R312 已归档至 [`archive/rounds-r312.md`](./archive/rounds-r312.md)——baseline-export 双侧探针：inline-flex 容器 taffy_baseline 错值 + multicol 项 None，确认 baseline-export 结构性多轮。）

### R314 — 综合 plateau 确认 + 全量基线复验 + latent line-height% defer（read-only 核查，基线 loose 438/490 / strict 295/490 持平；已飞书通知卡点）

**承接**：R313 baseline-overrides lever 证伪后，本轮做「单会话 clean win 是否真枯竭」的最终核查 + 全量基线复验。

**核查 1 — 全量 reftest 基线复验**：`make reftest`（test-guard 包裹）全量 490 → **438/490 (89.4%)**，与 R308 后基线一致，R309-R313 docs-only 提交**零漂移**（DC-7 卫生确认，代码状态 = R308 verified-green）。

**核查 2 — multicol gate 放宽是否可重试**：text.rs:709-711 代码注释**明示**「明确高度 balance 容器涉及 column-breaking，简单均衡分配会回归→回退单块」。R157（净中性）+ R203（净负）+ 本注释三重确认：paint 侧 gate 放宽/协调**已知回归**，重试必重复失败。真修复 = layout 侧 column-aware IFC（R131，major multi-session 架构）。

**核查 3 — DC-9 blend_mode 杠杆**：grep wpt-data **仅 3 文件**用 mix-blend-mode/isolation（R303「近零覆盖」确认）。实现 blend_mode（需 paint-isolation 架构，R278 defer）= **零 reftest 影响 + 高成本**，非 reftest 杠杆。

**核查 4 — latent `line-height: <percentage>` bug（R313 附发现）defer 确认**：computed.rs:195-206 未解析 line-height Percentage（同 R308 font-size% 谱系）。grep **0 个 reftest + 0 个产品 fixture（apps/）** 用 line-height % → 零覆盖零消费者，按 code-guidelines「不实现需求之外的功能」**defer**（与 R308 不同——R308 有 anonymous-inline-inherit 驱动，line-height% 无）。

**综合裁决（R305-R313 九轮收敛）**：
- **三条 clean-win 搜索策略全穷尽**：near-pass 聚类（R307，26 案全结构性/字体噪声）、POLLUTED 逐项（R309，唯一 win=R308 font-size%）、fresh chromium-Oracle cross-validate（R311，4 新候选全 ruled out）。
- **四条结构轨均证非单会话可解**：Phase A IFC 统一（墙②③，R125/R198/R205/R206/R209/R213 六轮死锁 + R306 几何基线证伪）、multicol column-aware IFC（R131，paint 侧 R157/R203 + 本轮注释三重证回归，layout 侧 major 架构）、baseline-export（R310 multicol=None / R312 inline-flex=错值 / R313 baseline_overrides=无效，三轮探针）、DC-9 blend_mode（0 覆盖）。
- **单会话 clean win 真枯竭**——剩余 forward motion 需**多会话架构承诺**一条轨。

**飞书通知**：已按 run-rules 以应用机器人身份向本人发送卡点告知（message_id om_x100b6c7...），说明 plateau 现状 + loose 438/strict 295/chr~36% + 建议多会话攻坚或接受现状。通知仅为告知，不阻塞后续。

**本轮 read-only 核查**：零代码变更。基线 loose 438/490 / strict 295/490 / chromium-Oracle ~48.2% 污染持平。next = 待用户对多会话结构攻坚的决策；若继续 rally，最高杠杆轨 = multicol layout 侧 column-aware IFC（R131，17+ 失败聚类），但需 multi-session spec-rfc + 实施承诺，非单会话。

### R315 — self-fail 集第 4 条搜索路径：plateau 再确认（read-only 实证，基线 loose 438/490 / strict 295/490 持平）

**承接**：R314 综合 plateau 确认后，本轮取**全新角度**——52 个 SELF-FAIL 用例（loose 失败、真实 +1 reftest 计数目标，区别于此前 strict near-pass 与 POLLUTED 候选）。逐个 probe 5 个非已知聚类候选，全部确认为结构性/特性缺口：
- `child-border-box-and-max-content-002`（1.22%）= taffy grid intrinsic-sizing（fit-content 轨道 + box-sizing:border-box，R304 DEFER taffy 升级）。
- `border-padding-bleed-001`（2.40%）= inline line-box 绘制顺序（结构性）。
- `border-bottom-width-006`（2.86%）= height:0+border 的 inline-block 基线（R180/R266 结构域）。
- `multicol-clip-001`（0.56%）= multicol 溢出裁剪 + Ahem（结构性聚类）。
- `float-nowrap-hyphen-rewind-1`（2.92%）= `hyphens:auto` 特性缺口（需语言级连字算法）。

**裁决**：self-fail 集成为第 4 条 clean-win 搜索路径（near-pass R307 / POLLUTED R309 / fresh-xval R311 / self-fail R315）穷尽确认枯竭。零代码变更，基线持平。

### R316 — baseline-export flex-baseline 后处理：实现 + 实验证伪（code attempt + revert，基线 loose 438/490 持平）

**承接**：R310/R312/R313 探针把 baseline-export（baseline-000~008 + flexbox-baseline 聚类）根因定位为「flex 项缺 first baseline」，但仅测了 inline-flex（R313 证 baseline_overrides 无效）与 field-fill（R266 证净 0）。**block-flex + multicol 项的后处理路径此前未实测**——本轮实现并实验裁决。

**前置核查（证 R304 DEFER 正确 + line-height% defer 正确）**：
- taffy 0.7.7 vendored `Style` 结构**无 `baseline_overrides` 字段**（0.8+ 才有）→ 「设 baseline_override + 重 layout」两趟路径不可用，ZeroWeb 后处理是唯一路径。
- `line-height: <percentage>` 在 computed.rs 未解析（同 R308 font-size% 谱系）；grep 实测 **0 reftest + 0 产品 fixture** 用 line-height% → 零覆盖零消费者，R314 defer 正确。

**实现（engine.rs，已 100% 回退）**：新增三函数 + compute() step 10.7 调用——
- `resolve_font_size_px`：ComputedStyle.font_size→px（em/rem 按 16px root）。
- `synthesize_first_baseline(box, styles)`：递归合成盒 first baseline（相对自身 border-box 顶）：优先 taffy 缓存基线；否则递归首个 in-flow 子元素（child.y 已是该盒 border-box 相对，累加）；基情形叶盒用 font-size 近似 ascent（content 顶部 + font-size）。坐标系与 painter 累积（`offset_y + box.y`）一致。
- `adjust_flex_baseline_alignment`：对 `display:flex|inline-flex` + `align-items:baseline` 容器，对 `taffy_baseline=None` 的流内项，按 `desired_y = target - local_b` 重定位（**只改 item.y，子树经 painter 累积自动跟随**）。

**FLEXBL_PROBE 实测 baseline-003（flex > "PA" 文本 + columns:3 multicol > "SS"）**：
- 容器 node 17v1 taffy_baseline=**Some(19.2)**；item[0] "PA"(18v1) 与 item[1] multicol(19v1) **均 y=0 h=19 taffy_baseline=None**；multicol synth=Some(16.0)，"PA" synth=None（匿名文本项无 style/无 LayoutBox 子）。
- 关键：**两 item 已被 taffy 基线对齐**（同 y=0/h=19，内容同字号）。1.1% chromium diff 不在基线对齐，在别处（multicol 列结构/font）。

**两种 target 源均失败（决定性证伪）**：
| target 源 | 结果 |
|-----------|------|
| 兄弟项派生（`max(sibling.y+sibling.taffy_baseline)`） | baseline-003 两 item 均 None→target=None→**no-op**（z_vs_chr 1.118% 不变，证 R310 的 1.058% 即未修状态） |
| 容器 taffy_baseline（19.2） | 触发但**回归**：multicol 子集 40/57→**38/57**（baseline-001 0.52→3.15%、baseline-002 0.00→3.50% 翻 FAIL），因把已对齐项错误下移 3.2px |

**裁决：baseline-export 经 flex-baseline 后处理不可解**——block-flex 项已被 taffy 正确对齐（fallback 对 baseline-001/002 足够），强行重定位破坏已绿用例。这是 baseline-export 杠杆的**第 3 种独立机制证伪**（R266 field-fill 净 0 / R313 baseline_overrides 对 inline-flex 无效 / **R316 flex 后处理对 block-flex 回归**）。三种机制覆盖 field-fill、inline-flex 后处理、block-flex 后处理全谱，baseline-export 从 ZeroWeb 后处理侧穷尽。

**代码状态**：env-gated 探针 + 实现代码**已 100% 回退**（`git checkout engine.rs`，`git diff --stat` 空）；`cargo check -p zero-layout-engine` 干净；`make reftest` 内置 686/686 全绿（DC-7 卫生确认，回退 byte-identical HEAD）。零代码变更落地，基线 loose 438/490 持平。

**对优先级队列影响**：baseline-export（baseline-000~008 + flexbox-baseline）经三轮探针 + 本轮实现共四轮，**从 ZeroWeb 后处理侧彻底 ruled out**——真修复须 taffy inline-level-box 基线合成或升级 taffy（0.8+ baseline_overrides，R304 DEFER prohibitive）。剩余 forward motion 确认为：① multicol layout 侧 column-aware IFC（R131，major 架构）；② Phase A IFC 统一（墙②③）；③ DC-9 blend_mode backdrop（0 reftest 覆盖）；④ DC-13 残余。均非单会话 clean win。本轮价值 = 以真实实现（非推断）排除 flex 后处理这条未测路径，防止后续轮重试。

### R317 — multicol breaking paint 门控放宽：实现 + 实验证伪（code attempt + revert，基线 loose 438/490 持平）

**承接**：R316 排除 baseline-export 后，转向 multicol column-aware IFC（R131，最大失败聚类）的最具体 paint 侧 wiring 候选——text.rs:713 `height_auto` 门控。设计文档 R201 Round 4' 把 multicol-breaking 的阻塞点 A 定为「paint 门控 `height_auto` 挡住有明确高度 inner 的列分布」，但 R203 称 paint 侧协调 net-negative。两者矛盾**未经单点实验裁决**——本轮实现并实验。

**实现（text.rs:713，已 100% 回退）**：把 `if !has_in_flow_children && is_balance_mode && height_auto` 放宽为 `if !has_in_flow_children && is_balance_mode`（去掉 height_auto，允许明确高度的 balance 容器走 paint 列分布）。假设：multicol-fill-auto-* 不受影响（其 column-fill:auto → is_balance_mode=false → 本就不进此分支）。

**实证（multicol 子集）**：**净 -5 回归**（40/57 → 35/57）：
- multicol-breaking-001 0.66→1.30%、002 0.98→1.58%、nobackground-001 0.50→1.13%、002 0.82→1.42%、005 0.82→2.71%（5 案翻 FAIL）。
- 目标用例 multicol-breaking-004 **反而恶化** 5.60→6.17%（paint 侧 `total/col_count` 均衡分配对明确高度嵌套用例比单块渲染更差）。
- multicol-fill-auto-001 不变（0.63%，证假设「auto-fill 不受影响」正确，但 balance 侧大面积回归）。

**裁决**：paint 门控 `height_auto` **load-bearing**，放宽净负。这**第 N 次实证 R203「paint 侧协调不可解」**（R157 净中性 / R198 font_size 死锁 / R203 净负 / R122 守卫净中性 / **R317 净 -5**）——paint 侧 `compute_multicol_info_for_paint` 的 `total/col_count` 均衡分配对明确高度/嵌套用例结构性错误，单块回退反而是当前最优。真修复须 **layout 侧 column-aware IFC**（R131）：在 layout 阶段计算 IFC 行盒后按列高预算碎片化，存结果供 paint 直接消费（绕过 paint 门控与重算）。

**对设计文档影响**：multicol-fragmentation-design.md Round 4'（paint 侧 wiring）**经 R317 实证证伪**，须重定向为 layout 侧（与 R203/R131 一致）。设计文档 §0/§3 Round 4' 的「paint 侧多轮子系统」方向关闭。

**代码状态**：实验代码**已 100% 回退**（`git checkout text.rs`，`git diff --stat` 空）；`cargo check -p zero-engine` 干净。零代码变更落地，基线 loose 438/490 持平。

**综合（R316+R317 两轮真实实现）**：本会话以**两次真实 code attempt**（非推断）排除了 baseline-export flex 后处理（R316）与 multicol paint 门控放宽（R317）两条最具体的单会话候选，均净负回退。连同 R305-R315 的 6 条搜索路径，reftest 单会话 clean win 经 **6 路径搜索 + 2 实现证伪**穷尽确认。剩余唯一 forward motion = multicol **layout 侧** column-aware IFC（R131，major 多会话架构）或 Phase A IFC 统一，均需 spec-rfc 多轮承诺，非单会话。本轮价值 = 实证关闭 multicol paint 侧 wiring 这条 R201 标的未测候选，纠正设计文档 Round 4' 方向。

### R318 — DC-13 图片加载端到端实测：已贯通（纠正 goal doc 过时缺口）+ 产品 smoke 文本度量结构性确认（read-only 实测 + goal doc 纠正，基线持平）

**承接**：R316/R317 排除 reftest 单会话候选后，转向 DC-13 产品 smoke（welcome/morning/wintertc）寻找**非 reftest 轴**的可落地进展。先核查 memory 中「ZeroBrowser 给 renderer 传 None as image cache，URL 导航未抓取图片子资源」（DC-13 P1 缺口）是否仍成立。

**核查（代码 + 端到端实测，证 memory 过时）**：
- **代码已全贯通**：`webview.rs:265 fetch_image_subresources` 在 `fetch_url` 导航三条成功路径（line 370/395/423）抓取 + 解码 `<img src>`；`decode_image_bytes`（image_cache.rs:368）按魔数字节分发 PNG/JPEG/SVG（resvg+tiny-skia）；`app_platform.rs` render_cpu/render_gpu/render_frame 三处传 `Some(&mut webview.image_cache())`（非 None）；并有 `render_path_consumes_webview_image_cache` 测试。
- **端到端实测**（product-smoke wintertc，base-dir 本地服务）：vision 核验 header logo（橙色圆形雪花/gear）+ 13 个参与方 SVG/PNG logo（alibaba/bytedance/cloudflare/deno/fastly/igalia/netlify/nodejs/shopify/suborbital/vercel/azion/matrix）**全部正确渲染**（非占位 glyph/短横）。**memory「传 None / Logo 缺失」过时**。
- **残余缺口**（准确）：WebP 解码未接入（decode 仅 PNG/JPEG/SVG）；CSS `url()` 背景图未抓取（fetch_image_subresources 仅 `<img src>`）。

**产品 smoke 实测（800×600 viewport）**：welcome 17.06% / wintertc 13.70% / morning-work 28.72%。
- wintertc diff 带分析（repeating ~15px 32-34% 带）+ LAYOUT_DUMP 核验：h1「WinterTC」单行正确（w=583px，未折行；vision 报「Wnt er TC」系低分辨率误读）。diff 残余 = system-ui 字体度量/line-height（结构域 Phase A 谱系），**非图片/布局缺口**。
- 产品 smoke 与 reftest 同源——文本度量结构性，非单会话 clean win。

**goal doc 纠正（文档治理 §1 自洽）**：已知缺口表「图片子资源/ImageCache 未贯通 P1-严重」+ 支持包络「Logo 全部缺失」**更新为已贯通**（R318 实测），消除文档与代码现实的矛盾。

**对优先级队列影响**：DC-13 图片加载子项从缺口清单移除（已落地）。DC-13 残余确认为：① 文本度量（welcome/wintertc/morning 共性，Phase A 结构域）；② morning .item-tag R109 inline→block 堆叠（结构域）；③ WebP 解码 + CSS url() 背景图（可落地特性，低 ROI——wintertc/morning 不用 WebP/背景图）。reftest + 产品 smoke **双轴 plateau 确认**：单会话 clean win 经 reftest 6 路径+2 实现、产品 smoke 实测，均指向文本度量/列碎片化/IFC 结构域多会话工作。

**本轮 read-only 实测 + goal doc 纠正**：零代码变更（`git diff -- '*.rs'` 空）；goal doc 2 处过时缺口纠正。基线 loose 438/490 持平。next = multicol layout 侧 column-aware IFC spec-rfc 设计（唯一未 ruled-out 的 forward motion），或接受 plateau 待用户多会话决策。

### R319 — column-aware IFC spec-rfc 设计 + A1 probe：Phase 1 价值 REFUTED（read-only spec + probe，基线持平）

**承接**：R318 确认双轴 plateau 后，按上一轮 CONTINUE 启动 multicol layout 侧 column-aware IFC（R131，长期被视为最大未 ruled-out forward motion）的 spec-rfc 设计。用 spec-rfc 完整模式自主产出（rally 协议不向用户提问，假设显式标记）。

**产出**：`docs/goal/rendering-compat/column-aware-IFC-spec.md`（431 行，§0-11，Spec Lint 28 Pass / 1 Warning / 0 Fail）。设计：新增 `LayoutBox.inline_multicol_columns: Option<Vec<Vec<InlineLayoutLine>>>`，layout 侧 `assign_lines_to_columns_balanced`（行盒版，照搬 block 侧 `assign_children_to_columns_balanced`），paint 短路消费（None 回退，**不放宽** text.rs:713 门控）。

**A1 probe（spec §7 首步，实证 Phase 1 价值）**：grep 6 个非嵌套 css-multicol 失败用例结构：
| 用例 | 结构 | Phase 1 匹配？ |
|------|------|----------------|
| multicol-fill-000 / count-002 / columns-001 | height:**auto** + balance + inline | ❌ paint 侧**已处理**（同算法迁移零改善）；diff 是列宽/glyph 精度（advance-width R225 死路） |
| column-height-009 | multicol-2 `column-height` 简写 | ❌ 非 balance+height 组合 |
| multicol-containing-002 | 含 `<img>` | ❌ 非纯 inline |
| multicol-block-no-clip-002 | 含 `<h4>` block | ❌ 非纯 inline |

**裁决：Phase 1（单层 balance 明确高度纯 inline）目标结构在失败集中近乎不存在**。关键洞察：**大多数 multicol 失败是 height:auto+balance+inline——paint 侧已用同款 `total/col_count` 算法处理，迁移到 layout 侧结果不变**（不是列分布问题，是列宽/glyph 精度 + 嵌套 fragmentation）。故 column-aware IFC layout 侧迁移对 reftest **零改善**。

**对长期假设的纠正**：R131/R201 长期把「column-aware IFC」标为 multicol 最大未 ruled-out lever。R319 spec-rfc + A1 probe **实证 refuted**——该 lever 的 Phase 1 无目标用例，Phase 2（嵌套 fragmentation）才是硬结构域。**column-aware IFC 从「最大 forward motion」降级为「低 ROI，Phase 1 不实施」**。

**multicol 真实 forward motion 收敛（R319 后）**：① 列宽/glyph 精度 = advance-width 谱系（R225 双实验证伪独立死路；真修复须 fontdue glyph advance 接入 paint 换行决策，独立大件）；② Phase 2 嵌套 multicol fragmentation（multicol-breaking-004/005/006，硬结构性多会话）；③ 接受 multicol plateau。

**对全局优先级队列影响**：**rendering-compat 所有「单会话/中等会话 forward motion lever」现已全部 ruled out 或 refuted**——reftest（6 搜索路径 + R316 flex-baseline + R317 multicol gate + R319 column-aware IFC Phase 1）、产品 smoke（R318 文本度量结构域）、baseline-export（R266/R313/R316 三机制）、multicol paint 侧（R157/R198/R203/R122/R317 五轮）、column-aware IFC Phase 1（R319 A1 probe）。剩余均需**多会话架构承诺**（fontdue glyph advance 接入 / Phase 2 嵌套 fragmentation / Phase A IFC 统一 / taffy 升级 R304 DEFER）或**接受 plateau**。本会话产出的 spec-rfc 文档 + A1 refutation 防止后续轮重投 column-aware IFC Phase 1。

**本轮 read-only spec + probe**：零代码变更（`git diff -- '*.rs'` 空）；新增 `column-aware-IFC-spec.md`（含 A1 refuted 记录）。基线 loose 438/490 持平。next = 待用户对「多会话架构承诺 vs 接受 plateau」的决策；若继续 rally 且要 code 进展，唯一未深探的大件 = fontdue glyph advance 接入 paint 换行（影响 advance-width 谱系整簇 + multicol 列宽精度，但 R225 标其「死路」，需重新评估是否真死路或当时探针不充分）。

### R320 — advance-width 死路重评（multicol-columns-001 Ahem 实证）：R225 结论成立，fontdue-glyph-advance lever 对 multicol 同样无效（read-only 实证，基线持平）

**承接**：R319 收尾遗留「fontdue glyph advance 接入 paint 换行」是唯一未深探的大件（R225 双实验标「死路」但疑探针不充分）。R319 又把 multicol class-A（columns-001 4.88%）diff 归因为「列宽/glyph 精度 = advance-width 谱系」。本轮以 multicol-columns-001 为标本 ground-truth 重评 R225。

**实证（columns-001 结构 + 双 PNG 对比）**：
- columns-001 用 `font: 1.25em/1 Ahem` + `meta flags=ahem`——**is_ahem=true，字符 advance = font_size 精确值**（`estimate_char_width` 的 0.55 启发式对 Ahem 不生效，inline/mod.rs:207 Ahem 分支返回 font_size）。故 **advance-width 在此用例完全不参与**——R225 的 estimate 启发式与本用例零相关。
- LAYOUT_DUMP：test 与 ref 的 multicol div 均 h=160 w=600（**几何完全一致**），故 diff 非列宽/容器尺寸。
- 逐列 ink 行剖面对比（test vs ref PNG）：ZeroWeb 每列渲染 4 行 ink（22 ink-rows 总），ref 每列渲染 ~1-2 行（ref 是单 div 两行手工排布，视觉模拟 6 列）。**diff 来自 multicol balance 把 11 行源文本分配到 6 列的结果与 ref 手工 2-wide-line 布局不一致**——是 **balance 分布/wrapping 正确性问题**（R199/R200 谱系），**非 advance-width**。

**裁决**：R319 把 class-A 归因为「advance-width 谱系」**对本用例错误**——columns-001 是 Ahem（精确 advance），其 4.88% diff 来自 balance 分布，与 advance-width 无关。**R225「advance-width 死路」结论经 R320 fresh 角度（Ahem 观察）确认成立**：即使实现 fontdue glyph advance 接入 paint 换行，对 multicol class-A（Ahem 用例）**零改善**（advance 本就精确）。

**对 fontdue-glyph-advance lever 的最终裁决**：该 lever 对 multicol（Ahem 主导）无效；对非 Ahem 用例 R225 双实验已证 26 case 零变化 + 产品 smoke ±0.03%。**fontdue-glyph-advance 从「唯一未深探大件」降级为「已 ruled out 死路」**（R225 + R320 双证）。

**全局终局裁决（R314-R320 七轮收敛）**：rendering-compat **所有** forward motion lever 现已 ruled out/refuted，含上轮遗留的 fontdue-glyph-advance（R320）：
- reftest 单会话：6 搜索路径 + R316/R317/R319 三实现证伪
- advance-width：R225 双实验 + R320 Ahem 重评 双证死路
- baseline-export：R266/R313/R316 三机制
- multicol paint 侧：R157/R198/R203/R122/R317 五轮
- column-aware IFC Phase 1：R319 A1 probe
- 产品 smoke：R318 文本度量结构域
- 图片加载：R318 实测已贯通（非缺口）

剩余 forward motion **全部**需多会话架构承诺：① Phase 2 嵌套 multicol fragmentation；② Phase A IFC 统一（墙②③）；③ taffy 升级（R304 DEFER prohibitive）；④ balance 分布算法精确化（chromium 二分搜索 vs T/N，R199/R200 已探 round-robin 更差，二分搜索未试但属 R200 谱系）。**或接受 plateau**。单会话 rally 迭代已无法推进 reftest 通过率。

**本轮 read-only 实证**：零代码变更；columns-001 dump + 双 PNG 逐列 ink 剖面对比。基线 loose 438/490 持平。next = 待用户决策（多会话架构承诺 vs 接受 plateau）；rally 单会话层面已无未探 lever。

### R321 — multicol balance binary-search lever 证伪：T/N = binary-search（等高行），columns-001 diff 实为 wrapping 精度（read-only 算法分析，基线持平）

**承接**：R320 收尾遗留「balance binary-search（chromium 二分搜索找最短列高）vs ZeroWeb T/N」是唯一未试的 contained 算法 lever（R199 只试 round-robin 更差，R200 称 T/N 正确但未试二分搜索）。本轮算法层证伪。

**算法分析（text.rs:962-1010 paint 侧行分配）**：当前分配 = `target_h = total_height/col_count`，行按 `(line.y/target_h).floor()` 归列。这是「按单列 layout 的 line.y 几何切分到 N 列」。
- **关键数学事实**：对**等高行**（columns-001 全 20px Ahem 行），T/N 几何切分与 binary-search（找最短列高使 N 列容纳）**结果恒等**——两者都把 `total_lines` 均分到 N 列。binary-search 仅对**非等高行**（混合行高）产生不同于 T/N 的列边界。
- columns-001 是等高 Ahem 行 → **binary-search 对它零改善**。
- R200 称 chromium 用 T/N 顺序填充 → 非等高场景 binary-search 也不匹配 chromium。

**columns-001 真实 diff 源（wrapping 精度，非 balance）**：test 165 非空格字符 vs ref 199（内容相当，非 mismatch）；ref 用 `&nbsp;` 构造特定不可断序列编码期望视觉。ZeroWeb 把 "x xx xxx xxxx xxxxx"（19 字符）在 100px（5 Ahem 单位）列内 wrapping 的断点与 chromium/ref 期望不一致——这是 **IFC wrapping 算法精度**（词边界/空格处理/orphans-widows），**非 balance 列高**。

**裁决**：balance binary-search lever **证伪**——对等高行（multicol class-A 主力）与 T/N 恒等零改善；对非等高行 R200 证 chromium 亦用 T/N。columns-001 diff 属 IFC wrapping 精度（独立子域，非 balance）。

**multicol lever 全谱终局穷尽（R199/R200/R157/R198/R203/R122/R310/R312/R313/R316/R317/R319/R320/R321）**：
| lever | 裁决轮 |
|-------|--------|
| balance round-robin | R199（更差）|
| balance T/N 正确性 | R200（证正确）|
| balance binary-search | **R321（= T/N for 等高行，证伪）**|
| paint 门控放宽 | R157/R198/R203/R122/R317（5 轮 net-negative）|
| column-aware IFC Phase 1 | R319（A1 refuted）|
| baseline-export | R266/R313/R316（3 机制）|
| advance-width | R225/R320（Ahem 双证）|
| 剩余 = IFC wrapping 精度 + Phase 2 嵌套 fragmentation | 独立子域/多会话 |

**全局终局**：rendering-compat reftest **所有 contained/single-session lever 经 14 轮（R199-R321 中相关轮）穷尽 ruled out/refuted**。剩余 forward motion **全部**需多会话架构承诺：① IFC wrapping 精度（词边界/空格/orphans-widows 对齐 chromium，独立大件，影响整条 IFC）；② Phase 2 嵌套 multicol fragmentation；③ Phase A IFC 统一；④ taffy 升级（R304 DEFER）。**rally 单会话层面 reftest 通过率已无推进路径**。

**本轮 read-only 算法分析**：零代码变更。基线 loose 438/490 持平。next = 待用户对「多会话架构承诺 vs 接受 plateau」决策；单会话 rally 已无法推进 reftest。

### R322 — columns-001 wrapping 实测正确（self-纠正 R321）+ proxy/local-serving 基础设施核查：均已就位无缺口（read-only 实测 + 核查，基线持平）

**承接**：R321 把 columns-001 diff 归因为「IFC wrapping 精度」并列为最深 gap。本轮 ground-truth 实测**纠正自身**。

**wrapping 实测（self-纠正 R321）**：minimal test `<div style="width:100px;font:20px/1 Ahem">x xx xxx xxxx xxxxx</div>` 经 product-smoke + LAYOUT_DUMP：div **h=80 = 4 行**（"x xx"/"xxx"/"xxxx"/"xxxxx"，每行 ≤5 Ahem 单位=100px）。**ZeroWeb wrapping 完全正确**——R321「columns-001 diff = IFC wrapping 精度」假设**证伪**。columns-001 真实 4.88% diff = balance 分布细节 vs ref 的 `&nbsp;` 编码期望的**亚像素/边界 mismatch**（wrapping 正确 + balance 算法正确[T/N，R321 证 = binary-search for 等高行] + advance 正确[Ahem] 均排除后，残余是分布 rounding/编码差异，**非单点 bug**）。

**proxy 基础设施核查（用户原始任务要求「确保 browser 支持代理配置」）**：
- zero-net 基于 reqwest 0.12.28。reqwest 源码实证（`async_impl/client.rs:418-420`）：`Client::builder().build()` **默认添加 `ProxyMatcher::system()`**——自动读 `http_proxy`/`https_proxy`/`ALL_PROXY`（含小写）env，**除非**调 `.no_proxy()`。
- ZeroWeb `HttpClient::with_config`（net/client.rs）**未调 `.no_proxy()`** → 系统 proxy 检测**默认启用**。
- `~/use-proxy` 设 `http_proxy=192.168.1.212:7078` / `https_proxy=...` → **`source ~/use-proxy && make browser` 即生效**，ZeroBrowser 经 reqwest 自动走代理。
- **裁决：proxy 支持已就位，无缺口**。reqwest 默认行为满足用户要求；无需新增代码（加显式 proxy 读取会与 reqwest 默认重复，违 code-guidelines 简单至上）。

**local-serving 基础设施核查（用户原始任务要求「静态资源存本地 + Rust web 服务器」）**：
- reftest 走本地 `tests/wpt-runner/wpt-data/`（文件直读，无网络）。
- product-smoke 走 `--base-dir` 本地文件服务（`fetch_image_subresources` 按 base-dir 解析 `<img src>`，R318 实测 logo 全渲染）。
- **裁决：local-serving 已就位**（文件 base-dir 模式覆盖 fixture 需求）；独立 Rust HTTP server 属过度设计（无消费者驱动），按 code-guidelines 不实现。

**全局终局复核（R313-R322 十轮）**：rendering-compat **所有轴**已核查：
- reftest 通过率：所有单/中会话 lever ruled out/refuted（R199-R321 相关轮）
- 产品 smoke：文本度量结构域（R318）
- 图片加载：已贯通（R318）
- proxy 支持：reqwest 默认就位（R322）
- local-serving：base-dir/wpt-data 就位（R322）
- IFC wrapping：实测正确，非 gap（R322 纠正 R321）

**结论**：rendering-compat 目标**所有可单会话推进的子项均已穷尽核查/实现**，无遗留单会话 lever 或基础设施缺口。剩余 forward motion **唯一**是**多会话架构承诺**（IFC 度量统一 Phase A / Phase 2 嵌套 multicol fragmentation / taffy 升级 R304 DEFER）或**接受 plateau**（loose 438/490=89.4%、strict 295/490、chromium-Oracle ~36%）。**rally 单会话迭代已无推进路径**——这是 10 轮（R313-R322）一致收敛的结论，非单轮判断。

**本轮 read-only 实测 + 核查**：零代码变更（wrapping minimal test / reqwest 源码核查 / proxy env 核查）。基线 loose 438/490 持平。next = 待用户对「多会话架构承诺 vs 接受 plateau」的明确决策；继续 rally 单会话层面将重复 plateau 确认，无新进展。

### R323 — DC-11 margin 折叠实测全过 + BFC margin 隔离实测：纠正 goal doc「未实现」过时声明（read-only 探针 + goal doc 纠正，基线持平）

**承接**：R322 收尾后转 DC-11（布局正确性）轴——goal doc 多处把「Margin 折叠 ❌ 未实现」「BFC ❌ 未实现」列为 P1 缺口，但 compute() 注释称「taffy 0.7 已内置块级 margin 折叠（CollapsibleMarginSet）」。此**文档矛盾**（goal 治理 §1 须纠正）此前未实证。

**margin 折叠探针（6 case，全过）**：minimal HTML + LAYOUT_DUMP abs_y 实测——
| case | CSS 规则 | ZeroWeb 结果 | 裁决 |
|------|---------|-------------|------|
| 相邻兄弟 mb:30 + mt:20 | max→30 间距 | gap=30 | ✅ |
| 父子 mt:40 + mt:25（无 border） | 折叠到 max=40 | parent/child 同 y | ✅ |
| 父 border-top:1px + child mt:25 | border 阻断，child mt=25 保留 | gap=1+25 | ✅ |
| 相邻 mb:30 + mt:-10 | 正负 30+(-10)=20 | gap=20 | ✅ |
| 祖父 mt:40 > mid mt:0 > 孙 mt:35 | 跨层折叠 max(40,0,35)=40 | 三者同 y | ✅ |
| BFC `overflow:hidden` 父 mt:60 + 子 mt:30 | BFC 子不与父折叠 | 子 mt=30 保留 | ✅ |

**reftest 实证**：`reftest-upstream margin` 子集 5/5 全绿（`block-in-inline-...-margin-collapse` 0.00%、`empty-flex-box-and-margin-collapsing` 0.00%、grid/table margin 用例 0.00-0.03%）。

**裁决**：**DC-11 margin 折叠已实现**（taffy 0.7 CollapsibleMarginSet；6 探针 + 5 reftest 全过）。BFC **margin 隔离**部分亦工作（overflow:hidden 子不折叠）。goal doc「Margin 折叠 ❌ 未实现 P1-严重」「BFC ❌ 未实现」**过时**——R323 纠正 goal doc 4 处：支持包络（line 80/81）、Current Proven Baseline（361/362）、已知缺口表（377）、DC-11 checklist（269）。margin 折叠项标记为已实现，DC-11 实际完成度高于 goal doc 旧声明。

**对 DC-11 影响**：DC-11「布局正确性」清单 10 项中，margin 折叠（R323 ✅）+ Float 布局/clear（R108b/R127/R129 已落地）+ 部分 BFC（R323 margin 隔离）+ auto margin 居中（R165）+ 百分比 max-height（R119）+ min/max 约束（已实现）均done；剩余 fixed/sticky/滚动容器/object-fit 部分项。**DC-11 实际完成度远高于 goal doc 旧 P1 缺口表所示**。

**本轮 read-only 探针 + goal doc 纠正**：零代码变更（margin 探针 minimal test + reftest margin 子集 + goal doc 4 处过时声明纠正）。基线 loose 438/490 持平。next = 续查 DC-11 其他项（BFC float containment / position:fixed-sticky / 滚动容器 / object-fit）是否如 goal doc 声称的「未实现」——若同样过时可逐项纠正 goal doc 自洽；或转多会话架构承诺。

### R324 — position:fixed 视口相对修复（code change，DC-11 真实 correctness 修复，loose 438/490 零回归）

**承接**：R323 纠正 DC-11 margin 折叠过时声明后，续查 DC-11 剩余项的具体 bug 声明。goal doc 称「Position: fixed ... 错误映射为 absolute」。minimal 探针（`position:relative` 祖先 margin 50/100 内放 `position:fixed; top:20; left:20` + absolute 兄弟）+ LAYOUT_DUMP 实测：absolute 正确在 (70,120)（祖先相对），但 **fixed 错误在 (120,220) 而非视口 (20,20)**——goal doc bug 声明成立。

**根因（engine.rs `adjust_fixed_to_viewport`）**：taffy 0.7 把 `position:fixed` 当 absolute 处理（containing block = 最近 positioned 祖先），故 fixed 的 left/top 被解析为相对该祖先。后处理 `adjust_fixed_to_viewport` 须把累积祖先偏移**扣除**使其视口相对，但旧实现**加上**（`box.x += parent_offset`）——仅在 `parent_offset==0`（fixed 在无偏移祖先/根附近）时碰巧正确，对有偏移 positioned 祖先的 fixed **over-correct**。**关键佐证**：仓库既有 R98 `adjust_absolute_to_initial_containing_block`（tests_9.rs:562 `118 - 8 = 110`）已用**扣除**约定转视口相对——旧 fixed 的「加上」与之**不一致**，本修复对齐。

**修复（engine.rs:2179-2180）**：`box.x -= parent_offset_x; box.y -= parent_offset_y;`（加→扣）。offset 传播逻辑不变（fixed 子元素 offset 仍归 0）。

**验证**：
- **探针**：fixed 现 (20,20) 视口相对（旧 120,220），absolute 兄弟不变 (70,120)。✓
- **全量 reftest**：loose **438/490 零回归**（fixed 用例 parent_offset 多为 0，加=扣），css-position **16/16 (100%)**。
- **make test**：**12255 passed / 0 failed**（10 ignored = real_website_compat）。
- **clippy --workspace --all-targets -D warnings**：干净；**cargo fmt**：干净。
- **新单测** `test_fixed_is_viewport_relative_inside_offset_positioned_ancestor`（断言 fixed 视口相对 (20,20) + absolute 祖先相对 (70,120)）。
- **8 旧单测更新**（tests_3/8/9）：旧测断言「加上」契约（fixed field = orig+offset），改为「扣除」契约（orig−offset），并修正 1 集成测（tests_3 `fixed.y >= 50` field 检查 → 绝对坐标 ≈ top 视口相对检查）。

**意义**：DC-11「Position: fixed」**真实修复落地**（code change，非 docs）。这是 R308（font-size%）以来首个真实代码修复（中间 R309-R323 多为 docs/探针）。区别于 reftest 计数 lever（plateau），这是**产品可见 correctness**——fixed-in-offset-ancestor 是真实页面模式（sticky header/footer in positioned container），旧 bug 会错位。零 reftest 计数变化（latent bug，reftest 未覆盖此结构）但**真实世界正确性提升**。

**方法论复用**：R323（margin 折叠实测）→ R324（fixed 实测）证明 DC-11 轴的「具体 bug 声明」逐项 probe 是真实 code win 来源（区别于 reftest 计数 plateau）。续查 DC-11 剩余项（sticky→relative 映射、滚动容器 scroll offset、object-fit、BFC float containment）可能还有可单点修的真实 bug。

**代码变更**：`crates/layout-engine/src/engine.rs`（adjust_fixed_to_viewport 修复 + 1 新单测）、`tests_8.rs`/`tests_9.rs`/`tests_3.rs`（8 旧单测更新）。基线 loose 438/490 持平（latent，零计数变化）；DC-11 fixed 项标记 done。

### R325 — 替换元素两侧显式尺寸不强制固有宽高比（code change，DC-11 真实 correctness 修复，loose 438/490 零回归）

**承接**：R324 方法论续查 DC-11「替换元素」项。R323 read-only 审计已确认 `apply_replaced_element_sizing`（tree.rs:165）三来源（HTML `width`/`height` 属性 + SVG data URI + 解码固有尺寸）接线生产、`compute_object_fit_rect` 全 5 值落地，但代码核查发现 CSS §10 一处真实 bug：**`<img>` 同时显式设置 width 与 height 时，旧实现仍设 `taffy_style.aspect_ratio = intrinsic_w/intrinsic_h`，taffy 据此把显式 height 强制拉到 width 比例**——`<img style="width:200px;height:50px">`（正方形 intrinsic 1:1）渲染成 200×200 而非 200×50，显式 height 被吞。

**根因**（tree.rs `apply_replaced_element_sizing`）：两处 `if computed.aspect_ratio.is_none() { taffy_style.aspect_ratio = Some(w / h); }`（HTML 属性分支 line 209 + 解码固有尺寸分支 line 256）无条件设固有宽高比。CSS §10 替换元素：当 width 与 height 都 definite 时，box 尺寸即两者，**不得**强制固有比例（object-fit 控制内容如何填充 box，box 尺寸由两侧显式值决定）。仅当至少一侧 auto 时才用固有比例推导另一侧。

**修复**（tree.rs:209-215 + 256-262）：两处守卫收紧为 `if computed.aspect_ratio.is_none() && (css_w_auto || css_h_auto)`（仅至少一侧 auto 才设固有宽高比）。两侧都显式时不干预，由 converter 从 CSS 处理。

**验证**：
- **新单测** `test_img_both_width_height_set_no_aspect_enforcement`（engine.rs table_layout_tests）：`<img style="width:200px;height:50px">` + 正方形 intrinsic (100,100)，断言渲染 200×50（旧 200×200）。✓
- **make test**：**12256 passed / 0 failed / 72 ignored**（ignored = real_website_compat 文档化集；R324 基线 12255 → +1 = 新单测）。零回归。
- **clippy -p zero-layout-engine --all-targets -D warnings**：干净；**cargo fmt**：干净。
- **reftest 计数**：loose 438/490 预期持平（latent bug，上游 reftest 未覆盖「两侧显式尺寸 img」结构；产品可见 correctness 提升——真实页面常用 `<img style="width:..;height:..">` 或 HTML `width`/`height` 双属性做布局稳定）。

**意义**：DC-11「替换元素」项**真实修复 + goal doc 纠正落地**。延续 R323（margin 探针）→ R324（fixed 修复）→ R325（img 修复）DC-11 轴「具体 bug 声明逐项 probe = 真实 code win」谱系（区别于 reftest 计数 plateau）。本轮同时纠正 goal doc 三处过时声明（DC-11 替换元素 line 275 + known-gaps 替换元素 line 379 + support-envelope 替换元素 line 82）+ BFC known-gaps line 378（R323 审计确立、原 deferred 到本轮）。

**DC-11 剩余项**：position:sticky（偏移**已被 taffy 应用** via converter sticky→Relative 映射，R326 实测；缺 scrollport 钳制属 architectural）+ 滚动容器（paint 偏移+裁剪已落地，master.md 已如实标「简化处理」）+ 百分比高度/auto margin/min-max（已实现）。DC-11 实际完成度远高于 goal doc 旧 P1 缺口表所示；剩余 sticky scrollport 钳制为多会话架构项。

**代码变更**：`crates/layout-engine/src/tree.rs`（apply_replaced_element_sizing 两处守卫）、`crates/layout-engine/src/engine.rs`（1 新单测）、`docs/goal/rendering-compat.md`（4 处过时声明纠正）。基线 loose 438/490 持平（latent，零计数变化）；DC-11 替换元素项标记 done。

### R326 — position:sticky 偏移行为实测 + 审计纠正（test + doc，DC-11 调查，loose 438/490 零回归）

**承接**：R324→R325 DC-11 轴续查最后一项 position:sticky。R323 read-only 审计标 sticky「偏移未应用」，依据是 `engine.rs:1948` 注释「sticky 偏移需宿主层滚动驱动」。本轮核查该注释所在函数 = `apply_relative_offsets`，**`#[allow(dead_code)]` 死代码**（非生产路径），故审计结论存疑。

**核查生产路径**：`converter/mod.rs:286` `convert_position` 把 `Sticky | Relative | Static` **统一映射为 taffy `Position::Relative`**。taffy `Position::Relative` 对 block-level 元素施加 top/left inset——即 sticky 偏移**已被 taffy 应用**（== relative 行为）。审计旧注「偏移未应用」错误。

**实证**：新增单测 `test_sticky_applies_inset_like_relative_at_scroll_zero`（coverage.rs）——同结构两渲染：static 基线 vs `position:sticky;top:10px`，断言 sticky delta == 10（scroll-0 应吸附场景，偏移如 relative 下移 10）。**测试通过**（delta==10），确证偏移已应用。

**sticky 真实缺口**：缺的是 **scrollport 相对钳制**——CSS sticky 语义：当元素 normal-flow 位**满足** inset 约束时（如 top:20 且 normal y=100，距 scrollport 顶 ≥20），应 == static（无偏移）；仅当滚动致违反时才「吸附」施加偏移。当前实现（== relative）**总是**施加 inset，故 normal 位满足 inset 的用例**过度偏移**。完整正确性需 scrollport 几何 + 滚动驱动 → **架构性**（需 host 层 scroll driver 协同），非单会话单点修复。scroll-0 + normal 位违反 inset 的「应吸附」用例（最常见 sticky reftest 场景）当前**恰好正确**（== relative）。

**验证**：make test **12257 passed / 0 failed / 72 ignored**（R325 基线 12256 → +1 新单测）；clippy + fmt 干净；基线 loose 438/490 持平（latent）。

**意义**：纠正 R323 审计 sticky 误判（「偏移未应用」→「已应用、缺 scrollport 钳制」），避免未来会话据错误审计「补」一个 taffy 已施加的偏移致**双重计数**。同时把 sticky 当前近似行为（== relative）以单测锁死作回归守卫。DC-11 单会话 code-win 轴（R323→R324→R325）至此**真正穷尽**——剩余 sticky scrollport 钳制、Phase A IFC 统一、multicol fragmentation、baseline-export 全为多会话架构承诺（见顶部「综合裁决」）。

**代码变更**：`crates/layout-engine/src/engine/tests/coverage.rs`（1 新单测 + 纠正注释）、`docs/goal/rendering-compat/master.md`（审计 sticky 行 + known-gaps sticky 行 + R325 sticky 提及，3 处纠正）。零 reftest 计数变化。

### R327 — Phase A Gate 2 多行放宽控制实验：净 -1 证伪 + 纠正设计 doc 墙①（env-gated 实验 + doc，基线 438/490 恢复零漂移）

**承接**：续 Phase A IFC 统一（最大结构性 lever）。设计 doc 墙①（phase-a-IFC-unification-design.md §2.3）称 ifc-008/009/011「唯一阻塞 = Gate 2 多行限制」。本轮用 **env-gated 控制实验**精确测量该断言，并 resolve 设计 doc 标为「疑点...需实证」的墙②回归机制。

**实验**：engine.rs:1910 Gate 2 加 env `PHASEA_AHEM_MULTILINE=1`（默认关，零 baseline 影响）：开启时放宽「单行」限制，允许纯 Ahem 多行容器存 inline_layout。比 R209（PHASEA_MULTILINE）**更窄**——保留 `is_pure_ahem` 要求，仅去 `lines.len()<=1`。

**实测结果**（reftest-upstream 全量 490，env ON）：
- 通过 **437/490 (-1)** vs 基线 438/490。
- **回归 1**：`multicol-fill-auto-001.xht` pass→fail（**精确证伪**：该用例**当前通过**，不在 52 失败集；放宽后被破）。
- **改善但未过**：`ifc-008` 8.18%→4.17%、`ifc-009` 6.11%→4.17%（stored Path A 部分生效，残余 4.17% = 墙③ multi-line 垂直定位/baseline）。
- **不变**：`ifc-011` 11.27%（未触及 stored 路径——疑 width mismatch 或未过 Gate 1）。
- **净 pass 计数 = -1**（ifc 改善未跨过阈值，multicol 倒退 1）。

**墙②机制 resolved**（设计 doc 原「疑点，需 Phase 2 探针实证」）：multicol-fill-auto-001 的 **TEST** = 真 multicol（column-count:3），paint 走 `use_stored = multicol_info.is_none()` = false → Path B（不变）；其 **REF** = float 模拟列（非 multicol），放宽后 float 容器（纯 Ahem 多行）切 Path A → test(Path B) vs ref(Path A) 分歧 → 破。**layout 无法区分「ref 上下文」故不可守**（R213 `!in_multicol` 失败的同因）。

**纠正设计 doc 墙①**：原断言「ifc-008/009/011 唯一阻塞 = 多行限制」**错误**——实验证明放宽多行后三者**仍不过**（ifc-008/009 残余 4.17% 墙③，ifc-011 未触及）。真阻塞 = 墙③（Path A multi-line 垂直定位）+ 墙②（multicol 一致性），非单点 Gate 2。

**裁决**：Phase A narrow slice（Gate 2 放宽多行 Ahem）**净 -1 证伪**，比 R209 更窄仍无效——ifc 收益不跨阈值、multicol 必倒退。**Phase A 真正 unblock = 墙③（Path A multi-line 正确）+ 墙②（multicol 也走 Path A 即 Phase 2 column-aware）一次性架构**，非 Gate 2 调参。已回退 env-gate（代码零变更，git diff 空），基线 438/490 恢复确认。

**方法论**：env-gated 控制实验 = 零 baseline 风险下取**精确净计数 + 逐用例 diff**（R209 仅记「ifc-008/009 改善但 multicol 回归」无精确数）。本轮补全精确数据（净 -1 + 逐 case diff%）并 resolve 墙②疑点，使后续多会话 Phase 2 有确定起点。

**代码变更**：零（env-gate 实验已回退）。`docs/goal/rendering-compat/master.md`（本 R327 条目）+ `docs/goal/rendering-compat/phase-a-IFC-unification-design.md`（墙①纠正 + 墙② resolved）。基线 loose 438/490 恢复零漂移。

### R328 — 单会话 lever 穷尽再确认（read-only 实证：剩余低 diff 案 + DC-9/multicol 路径审计，基线 438/490 持平）

**承接**：R327 决定性证伪 Phase A Gate 2 后，本轮对**未调查的剩余低 diff 失败案**（R148 全量再核未覆盖）+ DC-9/multicol 剩余路径做 read-only 实证，确认单会话 lever 确已穷尽，防未来会话重查。

**剩余低 diff 案实证**（REFTEST_DUMP + PIL 像素对比）：
- `border-001` (2.77%)：TEST=`border:25px solid` 100×100 实心边框；REF=`font:25px Ahem + word-spacing:3em`「1 2 3 4 5 6 7 8」折行模拟空 square 边。2.77% diff = Ahem 字 + word-spacing 折行精度（**IFC 墙③ 谱系**，非 border 渲染 bug）。test 渲染正确（hollow square），ref 的 fragile 字模模拟像素不对齐。
- `background-attachment-applies-to-001` (2.40%)：`display:table-row-group` + `background-attachment:fixed` + `repeat-x` + table 嵌套（cell 2in×1in）——table 背景 + viewport-fixed bg + repeat 多特性复合，非单点 bg-attachment bug（css-parser 已解析 background-attachment，paint 侧复合交互结构性）。
- 裁定：本轮核查的 border-001 / background-attachment 两案均落 IFC 折行精度 / table+bg 复合结构性（非单点 bug）。⚠️ 区别于 R308（anonymous-inline-inherit 经 POLLUTED 逐项 probe 发现 font-size% 真实单点 bug）——**self-fail 低 diff 案多结构性，但 POLLUTED（self-pass / chr-disagree）逐项 probe 仍可能出真实 bug**（R308 已证），勿因 self-fail 结构性结论放弃 POLLUTED vein。

**DC-9 路径审计**（核 committed HEAD 36875cb 后状态）：transform（R285）+ 5 color-matrix 滤镜（R286）已落，**唯一剩余 GPU 真实缺口 = blend_mode**（R278 实证单 framebuffer post-process 不可行，需 paint-isolation offscreen 子树+source/dest 双纹理，multi-round defer）+ **drop-shadow**（CPU+GPU 均 stub，同需 alpha-shape offscreen 架构）。二者均**非单会话可解**，同 paint-isolation 架构依赖。

**multicol 路径审计**（multicol-fragmentation-design.md）：balance（R200）、paint 门控（R157/R198/R203/R317）、baseline-export（R310/R312/R313/R316）、advance-width（R225）、column-aware IFC Phase 1 纯 inline 迁移（R319）**全证伪**；唯一未证伪路径 = **layout 侧 column-aware IFC（Round 2' 嵌套/混合内容碎片化）= 硬里程碑**，R319 标「Phase 1 零增益，真价值在嵌套/混合内容」，多会话。

**综合裁决**：rendering-compat 单会话 forward-motion lever 经本轮（未调查低 diff 案 + DC-9 + multicol 全路径）**最后一轮实证后确已穷尽**。reftest 主指标（438/490 loose / 295/490 strict / ~36% Oracle）剩余提升 + DC-9 blend/drop-shadow + multicol 硬里程碑**全部需多会话架构承诺**（Phase 2 column-aware IFC / paint-isolation / taffy baseline_overrides），或接受 plateau。基线 loose 438/490 持平（read-only，零代码变更）。

**代码变更**：零（read-only 实证 + 审计）。本 R328 条目沉淀结论防重查。

### R329 — POLLUTED clean-win vein 三趟复核确认穷尽 + header 自洽纠正（read-only committed-evidence 复核，基线 438/490 持平）

**承接**：R328 header 标「POLLUTED vein（R308 font-size% 真实 win）仍可逐项 probe，下一轮 R329 继续」，但 R328 正文 caveat（勿因 self-fail 结构性结论放弃 POLLUTED vein）与「综合裁决」表（POLLUTED hunt exhausted）/ R309（clean-win 杠杆收尾关闭）/ next-steps ruled-out 存在 governance §1 自洽张力。本轮独立复核 committed evidence 以裁决。

**三趟复核（均 read-only，基于 committed HEAD c322bdc 的 evidence，未跑动态 reftest——见下方约束）**：
1. **R298 全量清单逐项（R309，已归档 [`archive/rounds-r309.md`](./archive/rounds-r309.md)）**：12 项候选全归类，唯 R308 `anonymous-inline-inherit`（font-size%）= 真实 clean win；余全结构性/特性缺口/字体噪声。
2. **fresh 475 例 cross-validate top-30（[`evidence/r311-cross-validate-fresh-2026-06-19.txt`](./evidence/r311-cross-validate-fresh-2026-06-19.txt)）**：post-R308 重跑，污染率 48.2%（vs R298 的 48.6%，R308 仅边际改善）。top-30 chr-diff POLLUTED 全 ruled out——`downloadable-font-scoped-to-document`(20%)/`alternates-order`(14%)/`font-family-013`(6.65%)/`font-default-02/03`(3.46%)=@font-face 自定义字体未加载→回退度量噪声；`iframe-in-block-in-inline`/`iframe-in-wrapped-span`(9.75% 同机制)=R302 已 defer 的 iframe infra；`flexbox-baseline-align-self-baseline-horiz`(17.65%)=R295 structural；`rules-groups`(3.39%)=legacy HTML4 `rules=groups` 属性未解析；`move-with-text-after-paint`(4.02%)=JS。R311 原 VERDICT: no new contained CSS bugs surfaced。
3. **长尾 spot-check（本轮静态代码核查）**：`text-underline-offset-calc`(1.47%)/`text-underline-offset-percentage`(1.53%) 两 POLLUTED 同机制——grep `underline_offset|UnderlineOffset|underline-offset` 于 css-parser/style-system/engine **零命中** = 未解析属性（同 R232 text-emphasis 特性缺口），**非** R308 式单点解析 bug。余长尾（`background-*`/`block-formatting-contexts-*`/`float-*`/`clear-*` @ 1-2%）= fontdue 度量 + 低 diff 结构性（R307 near-pass frontier 已关闭 <0.2% 案）。

**裁决**：POLLUTED clean-win vein **三趟复核确认穷尽**——R308 font-size% 是唯一真实 clean win，top-30 + 长尾 spot-check 无第二处。R328 header「R329 继续 probe」**纠正为已穷尽**（re-cross-validate 仅在导入新 reftest 时复跑，非每轮主动 probe）。R308 方法论教训保留（POLLUTED self-pass/chr-disagree 原则上仍可能出新 bug），但**当前无已识别未 probe 目标**——继续每轮 probe 同批已归类用例只会重复 plateau 确认（同 R314 三策略穷尽判）。

**自洽纠正（governance §1）**：header(line 3)「R329 继续 POLLUTED」→「三趟复核确认穷尽」；综合裁决表 POLLUTED 行补 R311/R329 轮次；next-steps ruled-out POLLUTED 行扩 R311+R329。

**约束说明**：本轮并行 agent 正编辑 reftest 影响代码（css-parser ast/parser/tests_10、font/loader、style-system matcher、reftest.rs 共 6 文件未提交），故**基于 committed evidence 复核而非动态跑 reftest**——避免 WIP 与 committed 基线（438/490）混淆；任何动态复跑须待并行 agent 提交后基于新 HEAD 重测。零代码变更，基线 loose 438/490 / strict 295/490 持平。

**归档**：R309（POLLUTED clean-win 杠杆收尾）作为第 21 轮迁出至 [`archive/rounds-r309.md`](./archive/rounds-r309.md)，最近窗口收窄为 R310–R329（≤20）。

### R330 — 导入上游 WPT css-fonts 字体激活 @font-face + 影响实测：净负向 + rustybuzz 未接入生产根因发现（字体资产 + evidence + doc，self-source 438/490 持平）

**承接**：R329（commit 557bc96）实现 @font-face 解析+加载但 wpt-data 零字体文件→@font-face src 静默跳过，reftest 438/490 不变（feature latent）。R311 曾建议「@font-face 字体加载是 css-fonts polluted 聚类（24 案）解锁钥匙」（待 fontdue 度量噪声限制）。本轮导入字体激活 feature 并实测，裁决该假设。

**R329 后追加修复（commit 4fc4caf）**：`extract_font_faces` 原只扫 combined_css（传入+外链 `<link>`），漏内联 `<style>`（@font-face 常声明于此）。新增 `extract_inline_style_css(html)`（zero_dom::parse_html 提取所有 `<style>` 文本，去 XHTML CDATA 包裹）追加字体扫描 CSS。

**字体导入**：proxy fetch 上游 WPT（raw.githubusercontent.com/web-platform-tests/wpt）17 字体（全 HTTP 200）——`/fonts/`(6: Ahem/AD/Lato-Medium×2/Revalia/GentiumPlus-R) + `css/css-fonts/support/fonts/`(6: FontWithFancyFeatures/RobotoExtremo-VF/LinLibertine/LigatureSymbols/Inter-VF/Rochester) + `css/css-fonts/resources/`(5: COLR-palettes×2/markA/markB/colorization_SVG_COLR)。

**加载生效实证**：font-default-02（@font-face "fwf"→FontWithFancyFeatures.otf）WITH-fonts vs WITHOUT-fonts 渲染差 = **0.353% 像素**→字体确实加载并影响渲染。

**self-source**：全量 reftest-upstream **438/490 (89.4%) 持平**（css-fonts 60/60）。

**chromium-Oracle**（cross-validate.py，css-fonts 56 可比 case，见 [`evidence/r330-font-face-impact-2026-06-19.txt`](./evidence/r330-font-face-impact-2026-06-19.txt)）：54 案不变；2 案退化（均 font-feature 依赖）——`alternates-order` 6.28%→13.81%(+7.53)、`font-features-across-space-1` 2.68%→3.63%(+0.95)；污染率仍 ~46%。**净轻微负向**。

**根因（代码追踪，关键发现）**：@font-face 字体到达**光栅化**（fontdue），但生产路径**完全绕过 OpenType shaping**——rustybuzz `TextShaper`（`render-foundation/src/font/shaper.rs`，GSUB/GPOS 实现）**仅单元测试调用**（`render-foundation/src/lib.rs:1032-1052`），生产 paint 路径（`engine/src/paint/painter/text.rs:1057-1075`）逐字符 `glyph_id = ch as u32`，无 ligature/kerning/alternates。故加载特性字体不应用特性→渲染「基础字形」比 chromium（应用特性）更远（alternates-order 退化即此）。纯度量字体 fontdue 度量差 ≈ fallback 度量差→中性。WOFF（6 字体）fontdue 不可解析→静默回退（不变）；据 OTF 实证即使加 WOFF 解码也预期中性，本轮未加（code-guidelines §2 避免推测工作）。

**裁决**：@font-face 加载**正确实现且字体加载生效**，但 self-source 零影响、chromium-Oracle 净负向。**保留 @font-face 开**：① 正确浏览器能力，DC-13 产品 smoke 需要；② 2 退化案不可单点修（需完整 shaping），结构性；③ 关闭/gate 会掩盖 fontdue shaping 缺口违反 DC-14 诚实；④ 退化反映真实状态（加载自定义字体未 shaping），比旧 fallback 假一致更诚实。**R311「@font-face 解锁 css-fonts 聚类」假设证伪**——fontdue/simple-shaping 第四条死路（同 R225 advance-width / R229b font-weight-Bold / R174 AA）。真实多会话杠杆 = 把 TextShaper 接入生产 paint/layout 路径（高风险：IFC/paint 文本路径 = R125-R213 P0 死锁区，须多轮 narrow 收敛）。

**自洽纠正（governance §1）**：header(line 3)+M6 line 71「rustybuzz 已集成」→「TextShaper 已实现+单测，生产路径未接入」；综合裁决表新增 @font-face 杠杆行（R330 证伪）。

**代码变更**：字体资产 17 个 + `evidence/r330-font-face-impact-2026-06-19.txt` + 本 R330 条目 + header/M6/裁决表纠正。零 `.rs` 变更（@font-face feature 代码 R329/4fc4caf 已落）。基线 loose 438/490 / strict 295/490 / chromium-Oracle ~35.6% 持平（css-fonts chromium-Oracle 微负向 2 案，非主指标计数）。

**归档**：R310（multicol 设计文档自洽修订 + baseline-export 探针）作为第 21 轮迁出至 [`archive/rounds-r310.md`](./archive/rounds-r310.md)，最近窗口收窄为 R311–R330（≤20）。

### R331 — rustybuzz shaping 接入生产可行性评估：4 子任务多会话（非 bounded 单会话 probe）+ 光栅化按 glyph-index 隐藏子任务发现（read-only 代码核查 + plumbing 实验（已回退），基线 438/490 持平）

**承接**：R330 标「真实多会话杠杆 = 接 TextShaper 入生产路径」。本轮评估能否做 bounded 单会话 env-gated probe（如 R327 方法论）先测量 shaping 净影响，再决定多会话投入。

**核心发现：shaped glyph_id 是字体内部索引，当前光栅化按 codepoint 解读会误读**。`cpu/mod.rs:485` `let ch = char::from_u32(glyph.glyph_id)` 把 glyph_id 当 codepoint，fontdue 内部 cmap 查 glyph——这对生产 paint（text.rs:1057 存 `glyph_id: ch as u32`）正确。但 rustybuzz shaping 产出字体内部 glyph 索引（经 GSUB，ligature 可合并），直接喂 `char::from_u32` 会把 glyph 索引当 codepoint → 渲染错误字符。故 shaping 接入**必须同时改光栅化按 glyph-index**（加 GlyphPrimitive.is_glyph_index 标志 + draw_glyph_primitive 分支 + 全仓构造点 + CPU/GPU），跨 render-foundation——R330 未识别的隐藏子任务。

**advance-width 旁证（R225 死路再确认）**：本轮发现 IFC **layout**（inline/mod.rs:1366/1694）与 **paint**（text.rs:1077）**都用** estimate_char_width → 两者一致（test vs ref 同估计，self-source 抵消）。paint 单侧改 fontdue advance 会与 layout 不一致致 intra-fragment 错位；双侧改 = R223/R224/R225「三处同源」已证 oracle 26 案零变化。shaping 价值在 GSUB（ligature）/GPOS（kerning），非 advance 替换。

**4 子任务（详见 [`evidence/r331-shaping-wiring-plan-2026-06-19.txt`](./evidence/r331-shaping-wiring-plan-2026-06-19.txt)）**：① FontLoader→Painter plumbing（pipeline + painter set_font_loader）② 光栅化按 glyph-index（GlyphPrimitive.is_glyph_index + cpu/mod.rs + GPU + 全仓构造点）③ TextShaper 单位修复（offset/advance 现 26.6 定点 64× 错，shaper.rs:107-118）④ IFC paint gate（高风险 R125-R213 死锁区，env-gated 多轮收敛）。

**plumbing 实验(1)**：本轮实验性实现 set_font_loader（pipeline.rs + painter/mod.rs + reftest.rs Arc 包裹），**验证可编译 + css-fonts 60/60 基线不变**，但因 (2)(3) 阻塞无法形成 bounded probe，**已回退**避免死代码（code-guidelines §3）。advance-width 单侧变体 = R225 死路亦排除。

**裁决**：rustybuzz shaping 接入生产 = **4 子任务多会话架构**（(2) 跨 render-foundation 是 R330 未识别的隐藏深度），单会话不可 bounded probe。本轮价值 = 精确定位全部子任务（esp. 光栅化按 glyph-index）+ 排除 advance-width 单侧变体（R225），为后续多会话工作提供可执行起点。零代码变更（plumbing 实验已回退，git diff '*.rs' 空），基线 self-source 438/490 / strict 295/490 / chromium-Oracle ~35.6% 持平。

**归档**：R311（R308 后 fresh cross-validate plateau 再确认）作为第 21 轮迁出至 [`archive/rounds-r311.md`](./archive/rounds-r311.md)，最近窗口收窄为 R312–R331（≤20）。

### R332 — SHAPE_PAINT 门控探针：paint-only shaping 净负向（须 layout+paint 同源）+ TextShaper bug 修复保留（code change + 探针回退，基线 438/490 持平）

**承接**：R331 定位 shaping 接入 4 子任务并判定多会话，标 paint gate(④) 为高风险单点。本轮实现全 4 子任务做 R327 式 env-gated 实证，裁决 paint-only shaping 是否可单会话先行落地。

**实现（high-bit 哨兵法）**：为避免 GlyphPrimitive 加字段触发 55 处构造点改动，用 glyph_id 高位（`0x8000_0000`）作「字体内部 glyph 索引」哨兵——生产 codepoint < 0x110_000 永不置位，零行为变更。全 4 子任务：(1) FontLoader→Painter plumbing（set_font_loader + Arc）；(2) `FontLoader::rasterize_glyph_index`（fontdue rasterize_indexed）+ cpu/mod.rs draw_glyph_primitive 高位哨兵分支；(3) TextShaper 修复（advance 用 fontdue metrics_indexed 按 glyph 索引，ligature-correct；offset × font_size/upem）；(4) text.rs:1047 SHAPE_PAINT 门控（shape 产 glyph 置高位 + shaped advance + offset，skip Ahem）。

**实证（test-guard 包裹，chromium-Oracle cross-validate，[`evidence/r332-shape-paint-probe-2026-06-19.txt`](./evidence/r332-shape-paint-probe-2026-06-19.txt)）**：
- self-source reftest-upstream（SHAPE_PAINT=1）= **438/490 持平**，逐目录完全一致（test/ref 同向抵消）。
- chromium-Oracle（475 可比）：pollution **OFF 157 (48.0%) → ON 159 (48.3%)**（+2 略恶化）；逐 case chr-diff 变化 >0.3pp = **0**（零改善、无显著退化）。
- **paint-only shaping 净中性偏负**。

**根因**：IFC **layout**（inline/mod.rs:1366/1694）仍用 estimate_char_width 断行；shaping 只改 **paint** glyph 位置/索引 → layout/paint 不一致（glyph 溢出/不足行盒）→ 不接近 chromium（chromium layout+paint 同源 shaping）。与 advance-width「三处同源」同构，作用于 shaping：**须 layout+paint 同源**，paint 单侧无增益。

**裁决**：paint gate(④) **不能独立先行**——净负向，须与 layout shaping 同步（Phase A IFC 统一子集）。R331「4 子任务多会话」精确化：④非独立可先行子任务。

**保留 (3) TextShaper 修复**（shaper.rs，+11/-5）：真实 bug 修复（ligature advance 取首 cluster 字符宽度过窄→metrics_indexed 按 glyph 索引正确；offset raw 字体单位未缩放→×font_size/upem）。fontdue metrics_indexed/units_per_em 已验证；27 单测全过；未来 layout+paint shaping 同源接入即正确无需重做。**回退 (1)(2)(4)**（pipeline/painter/reftest/loader/cpu/text git checkout HEAD）：paint-only 净负向 + gate 关闭即死代码（code-guidelines §3）。

**代码变更**：`crates/render-foundation/src/font/shaper.rs`（shape_with_rustybuzz advance/offset 修复，保留）；6 个 probe 文件回退至 HEAD（`git diff -- '*.rs'` 仅 shaper.rs）。零 reftest 计数变化（TextShaper 生产无调用方）。基线 self-source 438/490 / strict 295/490 / chromium-Oracle ~35.6% 持平。

**归档**：R312（baseline-export 双侧探针）作为第 21 轮迁出至 [`archive/rounds-r312.md`](./archive/rounds-r312.md)，最近窗口收窄为 R313–R332（≤20）。

### R334 — WM-1 abs-pos-non-replaced-vrl/vlr 实证：positioning 假设证伪 + direction-rtl 镜像修复（spec-correct latent，0 clean win，真阻塞 = paint-IFC 颜色 = Phase A）

**承接**：R333 收尾指 Phase 2 multicol 为唯一 forward motion（硬里程碑）。本轮回溯 R237/R238 标「首选实现候选」但**从未执行**的 WM-1 abs-pos-non-replaced vrl/vlr（14 case，全 self-fail，单一代码面 engine.rs abspos inset/CB，无图片噪声，z_vs_ref ≈ z_vs_chr），做实现轮实证。R238 曾定位两缺口：缺口 B（direction 分支缺失，rtl 5.03% vs ltr 1.28%）+ 缺口 A（ltr all-auto 残差 1.28%）。

**实证 1（direction 被完全忽略）**：诊断单测复刻 vrl-002(ltr)/vrl-012(rtl) 结构 dump span 几何——ltr 与 rtl 渲染**完全相同**（均 x=240,y=200），证实 R238 缺口 B。

**实证 2（direction-rtl 镜像修复 spec-correct）**：§10.3.7 + writing-modes §7.1 推导——all-three-auto 时 ltr 置 inline-start 边静态、rtl 置 inline-end 边静态，两者盒位沿 inline 轴镜像：`rtl_top = CB_inline_extent - ltr_top - height`（worked example 双证：vrl-002 ltr top=160；vrl-012 rtl top=80=320-160-80）。实现（engine.rs fix_vertical_mode_abs_pos all_inset_auto 块末尾 +10 行）：rtl 时 `child.y = (container_width - child.y - child.height).max(0)`。LAYOUT_DUMP（Ahem）验证：vrl-012 修复后 span 相对 CB y=80 x=160（col 2），**= worked example top=80 ✓✓**；vrl-002 不变 y=160 ✓。镜像机制 spec-correct。

**实证 3（0 clean win —— 真阻塞 = paint-IFC 颜色，非 positioning）**：修复后 abs-pos-non-replaced 子集 strict——vrl-012 5.03%→3.67%（改善仍 fail），vrl-002 1.33%（不变 fail），全 14 case 0 strict pass。positioning 已 spec-correct 为何仍 fail？PNG 像素分析：vrl-002 test span 区域（abs [168,247]×[228,307]）= **100% red（6400px），零 green**；整张图零 pure-green。→ 绿色 "X" glyph **完全未绘制**。根因（代码追踪）：div(CB) `color:transparent`（隐藏 "1 2 34"），span `color:green`；paint_text（text.rs:654）用**容器** style.color 绘制其 IFC 收集的全部 inline 文本 "1 2 34 X" → "X" 被 transparent 绘制不可见。per-fragment 颜色覆盖（text.rs:1028 frag_color）**仅在 multicol 分支**（line 1012）；非 multicol vertical-rl CB 不触发 → span green 不生效。= goal doc 已知缺口 #3（inline ownership）+ #4（Layout/Paint IFC 双路径）= **Phase A IFC 统一 P0 死锁区（R125-R213）**。WM-1 真阻塞是 paint-IFC 颜色，**非 R238 推断的 positioning**。

**裁决**：① direction-rtl 镜像**保留**（spec-correct latent correctness fix，同 R324/R325 谱系）——修真实 spec 违规（rtl 渲染与 ltr 完全相同），vrl-012 strict 5.03→3.67，loose 全量 **438/490 零回归**，新增回归单测 `test_abspos_vertical_rl_direction_rtl_mirrors_inline_position`（断言镜像不变式 ltr_y+rtl_y+h==CB_inline_extent）。② **WM-1 作 positioning lever 关闭**（防未来重查）——R237/R238「最大最干净候选」推断实证部分证伪，真阻塞 = Phase A，与 plateau P0 缺口同源，非独立单会话 lever。③ vrl-122/130（block-axis left/right/width §10.6.4）不同子计算，mirror 不触及（正确）。

**全局影响**：WM-1 是 R237/R238 后唯一「推荐但未执行」的 writing-modes contained 候选。本轮实证关闭它，进一步收敛「单会话 lever 已穷尽」。剩余 writing-modes 失败（WM-2 clip-rect 同 paint-IFC 颜色+swatch 噪声 / WM-5 clearance-vrl R114b/R164 四轮证伪 / WM-6/7 vertical-orthogonal float R133 结构多轮）全结构性/Phase A 耦合。

**代码变更**：`crates/layout-engine/src/engine.rs`（fix_vertical_mode_abs_pos +direction-rtl 镜像 + 1 回归单测）。loose 438/490 零回归；strict 总数 295 不变（vrl-012 改善 5.03→3.67 未跨 0.5% 阈）。clippy --workspace --all-targets 干净；cargo fmt 干净；layout-engine 888 测试全过。

### R335 — paint-IFC per-fragment 颜色探针 net-negative：WM-1 真阻塞 = Phase A 双路径终局确认（探针已回退，基线持平）

**承接**：R334 收尾 CONTINUE 指向探查 paint-IFC per-fragment 颜色修复可行性。R334 定位 WM-1 cluster（abs-pos-non-replaced-vrl/vlr）真阻塞 = 绿色 "X" glyph 完全未绘制（paint-IFC 用容器 `color:transparent` 绘全部 inline 子树，per-fragment 颜色覆盖仅 multicol 分支 text.rs:1028）。本轮实证该修复是否可 bounded 落地。

**探针实现（已 100% 回退）**：非多列 `render_fragment!` 宏（text.rs:1124，覆盖 use_stored + 非存储两 fragment 循环）镜像 multicol 分支（text.rs:1019-1032）——解析 fragment 所属 inline 元素（文本节点取父元素）的 color，fallback 容器 color；glyph add 改用 frag_color。生产 caller（mod.rs:263/452）传 `Some(styles)`，per-fragment 解析可生效。

**实证（abs-pos-non-replaced 子集 loose，探针 ON）**：**loose 14/14 → 12/14 净 -2**，多数 case diff 上升 1-1.5pp（vrl-002 1.33→2.67、vrl-012 3.67→5.00、vrl-130 5.03→6.33 新 loose FAIL、vlr-163 5.40 新 loose FAIL）。

**根因（Phase A 双路径，R334 推断实证确认）**：green X 现被 div 的 **paint IFC** 在 **paint-IFC 位置**（normal-flow，"X" 紧跟 "1 2 34"）绘制，≠ ref 期望的 **abspos 静态位置**（fix_vertical_mode_abs_pos 计算，R334 实证 vrl-012 y=80 正确）= goal doc gap #4 **Layout/Paint IFC 双路径**。per-fragment 颜色虽隔离正确，激活了**错误路径**（paint-IFC）绘制→离 ref 更远→diff 上升。

**裁决**：① per-fragment 颜色探针 **net-negative，已 100% 回退**（git checkout text.rs，子集复测 14/14 loose 恢复），avenue 关闭。② **WM-1 真阻塞 = Phase A 双路径**——R334（positioning）+ R335（color）两角度均收敛至此。per-fragment 颜色须先统一 layout/paint IFC（paint 复用 layout 存储行盒/abspos 位置）才能安全应用。③ WM-1 单会话 lever 彻底穷尽，剩余 forward motion = Phase A IFC 统一（多会话硬里程碑）。

**Phase A 设计补充**：paint IFC 把 abspos 后代的 inline 文本当正常流绘制（位置错）+ 抑制 abspos span 自身 paint_text 的 green 输出 = Phase A 须解决的具体机制之一（区别于 large-font 的 font_size 存储、welcome 的度量分歧）。建议 Phase A 设计文档补此表现。

**代码变更**：零（探针已回退，`git diff -- '*.rs'` 空）。基线 loose 438/490 / strict 295/490 持平。

### R336 — abspos 文本抑制机制精确定位 + refined skip 探针 net-neutral：WM-1 Phase A 第三角度确认（探针已回退，基线持平）

**承接**：R335 收尾 CONTINUE 指 per-fragment 颜色（R335 已证伪）。Phase A 设计文档 v1.2（R306）已证伪 baseline-alignment Wall ③ 为阻塞点，故本轮**不 pursuing baseline**，换第三角度——abspos 文本抑制机制——做实现轮。R334/R335 已定位 WM-1 green "X" 未绘制，本轮精确定位抑制源 + 测 refined skip。

**抑制机制精确定位（探针 PROBE_ABSPOS，env-gated）**：插桩 abspos span paint_text（text.rs:679）实证——span 的 paint_text **被调用**（fs=80、color=green、content_w=80、has_direct_text=true 全正确），但 **painted_contains=TRUE** → text.rs:690 守卫 `fragment_node_ids.is_none() && painted_inline_nodes.contains(&node_id)` return → **span 自身绘制被抑制**。探针 div IFC fragments：node_id=34(span) text="X"。**collect_inline_items（inline/mod.rs:1066）对 inline 元素用 `doc.text_content(child_id)` 收集文本，node_id=child_id=span，不检查 position**——abspos span 的 "X" 被收入 div IFC，render_fragment!（text.rs:1125）insert span 进 painted_inline_nodes → 抑制 span 自身 paint。CSS §9.8：abspos out-of-flow，文本不应参与父容器 inline 流——当前违反。

**refined skip 探针（已回退）**：非多列 fragment 循环加 skip——owner 为 abspos/fixed **且 owner≠self**（abspos 元素自身绘制时不跳过）的后代文本跳过。探针实证：div(box 32) 绘 "X"(owner=span≠div) → skip=true（正确）；span(box 34) 绘自身 "X"(owner=span=self) → skip=false（正确）；span painted_contains 翻 **false**（抑制解除）。**但 span IFC fragment fs=16**（paint-IFC 空 styles 默认，非 80）= Layout/Paint IFC double-path（gap #4）。

**净效应**：子集 loose 14/14 持平（vrl-002 1.33→1.28 微变）、strict 0/14 持平、全量 loose **438/490 持平** = skip **net-neutral**。即便抑制解除，span 自身 paint-IFC 因空 styles 产出 fs=16，green "X" 仍错——suppression 修复须**同时**解 double-path 才生效 = Phase A 整体。

**裁决**：① refined skip **net-neutral，已 100% 回退**（git checkout text.rs，基线恢复）。② collect_inline_items 不排除 abspos 是**有意为之**——layout 侧 fix_vertical_mode_abs_pos 依赖 IFC fragment 算 abspos 静态位置，故不能 collect 层排除；paint 层 skip 又受 double-path 阻塞。③ **WM-1 Phase A 第三角度确认**：R334 positioning → R335 color → R336 suppression，三角度一致指向 Layout/Paint IFC 双路径。WM-1 单会话 lever 彻底穷尽（三角度闭环）。

**Phase A 设计补充（v1.2 之上）**：WM-1 精确表现 = (a) paint IFC 把 abspos 后代 inline 文本当正常流绘制（位置错）+ 标记 painted_inline_nodes 抑制 abspos 自身 paint_text；(b) 即便解除抑制，abspos 自身 paint-IFC 空 styles 产出错误 font_size。Phase A 须同时：paint 复用 layout 存储的 abspos 位置/font_size + 容器不绘 abspos 后代文本。两者均非单点。

**代码变更**：零（探针已回退，`git diff -- '*.rs'` 空）。基线 loose 438/490 / strict 295/490 持平。




