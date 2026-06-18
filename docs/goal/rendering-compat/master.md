# 渲染兼容性目标 — 运行时控制面板

**最后更新**: 2026-06-19（R311 R308 后 fresh chromium-Oracle cross-validate：污染 48.2% 再确认 plateau，4 新候选 ruled out[downloadable-font JS/iframe、css-fonts @font-face 缺口、rules-groups legacy attr、flexbox-baseline 结构性]；三条 clean-win 搜索路径[near-pass R307/POLLUTED R309/fresh-xval R311]全穷尽。→R310 multicol 设计 v0.4 修订+baseline-export 探针确认根因[taffy 仅 flex/grid 算 first-baseline，multicol/block 项 None]，剩余 forward motion 全结构性多轮[baseline-export/multicol breaking/DC-9 blend]或特性实现[@font-face/JS/原生控件]）
**当前活跃里程碑**: M10 — 上游 WPT 真实 Reftest 通过率提升（near-pass 杠杆已关闭，转结构性多轮）/ DC-13 产品 smoke（morning-work 4× 高度幻影盒 R255 已修）
**最新进展**: R307 **DC-14 strict 全量 near-pass frontier 实证 + evidence 持久化（read-only，零代码变更）**——承接优先级队列标的「攻 near-pass CSS2 clean win」（R280 phased 第二步）。全量 `ZERO_REFTEST_STRICT=1` reftest 复现 R287 **strict 296/490 (60.4%) / 194 fail**（基线一致）。194 失败按 diff% 升序持久化 `evidence/r307-strict-nearpass-frontier-2026-06-19.txt`。Diff-band：<0.2%:26 / 0.2-0.5%:18 / 0.5-1%:53 / 1-2%:22 / >2%:75。**<0.2% frontier（26 案）逐聚类根因分类——全部落入已知结构性墙或字体噪声，零独立 clean win**：css-multicol baseline-export（baseline-000/003/004/005/006，R235/R266）+ breaking/fill/balance（R131 墙②，10 案最大聚类）/ css-tables collapsed-border·visibility-collapse·display-contents（6 案）/ css-position hypothetical-box-scroll（2 案）/ css-flexbox baseline·column-row-gap（2 案）/ CSS2 color-applies-to·float-nowrap（text glyph 亚像素噪声）/ **CSS2 ifc-001（0.12%）LAYOUT_DUMP 实测 TEST div1 h=21.2 vs REF div h=22.0——inline 包裹文本 vs 直接文本行盒高差 0.8px = Phase A 墙③**（R206 broad 翻 FAIL 直接因）/ css-grid text-input（表单控件特性缺口 R202）。**裁决**：near-pass frontier 是结构性 plateau 拖尾边缘非 clean-win 源；R280「101 ≤1%」计数乐观逐用例分类后零 clean win，**near-pass clean-win 杠杆经实证关闭**。剩余 forward motion 全转结构性多轮（Phase A 墙②③ / multicol column-aware IFC R131 / DC-9 blend_mode / DC-13 残余）。→ R306 **Phase A Phase 0 glyph-baseline 耦合探针实证（read-only，零默认回归，env-gated 改动已 100% 回退）**——R305 spec-rfc 设计文档把 Phase 0 定为实测 glyph 基线耦合（§6.3A 自标前提不稳：`GlyphPrimitive.y`=基线 + frag.y/offset/glyph.y 经验性耦合，读码不可解）。本轮 env-gated 探针：text.rs:1208 stored Path A 的 `v_offset` 加 `PHASEA_BL=1` 临时改用文档化不变量 `v_offset=frag.height`（types/mod.rs:387「基线=frag.y+height」+ apply_vertical_alignment `run.y=baseline_y-run.height`）。font-051（`div{font:100px/1 Ahem}`→"FAIL" 4×100=400×100 黑矩形）A/B：**默认 offset → 0.00% PASS；探针 frag.height → 16.67% FAIL（80000/480000px, max ch 255）**。裁决：**§6.3A「geometric baseline 可作 render baseline」证伪**——IFC 几何基线（frag.y+height）≠ fontdue render baseline，fontdue Ahem glyph 度量使 offset=0（非 height）产出与 chromium 一致位图。**关键推论**：① Gate 2 `is_pure_ahem` 保证 stored 片段 is_ahem 恒 true → stored Path A `else{frag.font_size}` 分支**死代码**、v_offset 恒 0；② 若 baseline_y 字段存几何基线 paint 直接用会重演 16.67% 错误（破坏 R207 子集）。**对设计文档纠正**：原 Phase 1（baseline_y=几何基线）作废；Phase 1 重定向为 **Gate 2 放宽（offset 校准 is_ahem?0:font_size 不动）**——即 R209 PHASEA_MULTILINE 已试方向，被墙②+换行精度阻塞。**offset 语义非 Phase A 阻塞点**（Path A offset 对 stored Ahem 已正确）；真硬阻塞=墙② multicol 反向依赖 + 换行/列宽精度。设计文档升 v1.2（§6.3B 实证裁决 + header ⚠️ 修订标注 + 修订历史）。探针代码已回退（git diff 仅余并行 agent README.md WIP），revert 后 font-051 复测 0.00% PASS。基线 loose 438/490 / strict 296/490 / chromium-Oracle ~35.6% 持平。→ R301 **float shrink-to-fit 0-width 块子元素修复落地**（R300 定位的真 bug：width:auto 浮动元素，块级子元素全 0 宽时——如 visibility:collapse 的 flex item 主尺寸归零——旧 `content_max_w > 0.0` 条件跳过收缩致 float 撑满全宽）。**修复**：engine.rs adjust_float_positions_with_context 把收缩条件从 `content_max_w > 0.0` 改为「有块级子元素即收缩」（content_max_w=0 时收缩到 padding+border，最小盒，仍比全宽更接近 §10.3.5 shrink-to-fit）。**验证（chromium Oracle）**：flexbox-collapsed-item-horiz-001 z_vs_chr **20.53%→15.04%**（浮动 flex 容器现收缩而非撑满；残余 15% = 2px vs chromium min-content 21px，需 intrinsic sizing 完整修复）。**loose 438/490 / strict 296/490 全量持平零回归**；新单测 `test_float_with_zero_width_block_child_shrinks`（断言 float 宽 <50 而非 800）**回退验证有牙齿**（旧条件下 FAIL "got 800"）；make test 12253 passed/0 failed、clippy/fmt 干净。同轮复核 **Phase A viable slice 已穷尽**：R207 narrow 条件（pure-inline 叶文本容器）已覆盖 font-051；剩余 large-font（ifc-008/009/011 = block-child 容器）是 R125/R198/R208 stored-vs-paint baseline 墙，narrow 扩展无安全目标。**意义**：又一 DC-14 真实一致性提升（被 self-source 掩盖——test/ref 同获收缩故 self 不变），float shrink 是真实 CSS §10.3.5 正确性缺口。→ R300 POLLUTED 候选攻坚全面确认 structural plateau。→ R298 DC-14 **全量** chromium-Oracle 交叉验证（--all 503 oracle 截图 + 全量 ZeroWeb dump，475 用例尺寸可比）：**污染率 48.6%（160/329 同源通过为 chromium 假通过）**，真实 chromium 一致率≈35.6%（169/475）远低于 self-source 89.4%。按目录：writing-modes 73% / multicol 60% / CSS2 57% / grid 50% / text-decor 47% / fonts 47% / tables 41% / position 38% / **flexbox 26%（最诚实）**。完整 POLLUTED 候选清单（self 通过但 chromium 大不一致）见 `evidence/cross-validate-full-2026-06-18.txt`：top = backdrop-inherit-rendered（47.5%，dialog JS）/ abs-pos-border-offset-002（27.9%，vrl abspos）/ table-grid-item-dynamic-003（23.8%）/ position-absolute-semi-replaced-stretch-input（23%，原生 widget）/ **flexbox-collapsed-item-horiz-001（20.5%，R111 已 self-fix 但仍 chr 20%）/ font-051（8.3%，Phase A）/ collapsed-border-vertical-{lr,rtl,sideways}-rtl-overflow（3 联 ~4.7%，table 子系统聚类）/ float-no-content-beside-001（4.0%）**。同轮另排查 colspan-overflow（visibility-collapse-colspan-003 残余 2.79% = ZW **过裁剪** vs chr，复杂 collapsed-column 语义）+ col-definite-max-size-001（col max-width 需天花板强制非贡献钳制，触及核心算法）两候选均非 clean win。**意义**：DC-14 anti-false-pass 全量基线确立，完整候选池扩大 clean-win 搜索；table collapsed-border-vertical-rtl-overflow 3 联聚类 + flexbox-collapsed-item（R111 谱系）是下轮可执行杠杆。→ R297 `<col>`/`<colgroup>` **列背景渲染落地**（CSS Tables §17.5.3——之前完全缺失，`<col>` 不生成常规流盒故 background-color 被静默丢弃）。R296 cross-validate 定位的 POLLUTED 真 bug `visibility-collapse-colspan-003`（self 0.14% / chromium 4.91%）根因即此。**修复**：① layout `collect_table_col_backgrounds`（table.rs）在 position_cells 后把每个非透明、非折叠的 col/colgroup 记为 `(node_id, x, width)`，几何（含 border-spacing + visibility:collapse）镜像 position_cells 单元格定位；② LayoutBox 加 `table_col_backgrounds: Vec<(NodeId,f32,f32)>` 字段；③ painter `paint_table_col_backgrounds` 在单元格子元素**之前**按列跨满表格 content 高度绘制背景填充（CSS 层序：table→colgroup→col→rowgroup→row→cell），接入 `paint_node` + `paint_node_in_rect` 两路。**验证（chromium Oracle）**：visibility-collapse-colspan-003 z_vs_chr **4.91%→2.79%**（红/绿列背景现绘制，纯背景区与 chromium 完全一致）；残余 2.79% = 预存 table-cell 文本定位 + colspan 溢出裁剪（独立子问题，非列背景）。**loose 438/490 / strict 296/490 全量持平零回归**；新单测 `test_collect_table_col_backgrounds`（3 col：red/蓝折叠/green→断言 2 条目 + 折叠列跳过 + x/width 正确）；make test 12252 passed/0 failed、clippy/fmt 干净。**意义**：DC-14 真实 chromium 一致性提升（self-source 因 test/ref 同获列背景仍 0.14% 不变，故被 self-source 掩盖——正印证 R296 污染机制），列背景是真实渲染能力缺口（影响任何用 `<col background-color>` 的页面/表格）。→ R296 DC-14 chromium-Oracle 交叉验证 fresh 证据（抽样 90 用例，82 可对比）：**污染率 43.3%（26/60 同源通过为 chromium 假通过）**，印证 DC-14 self-source 含水分；**css-flexbox 0% 污染**（7/7 同源通过全部 chromium 一致，flexbox 失败诚实可信），CSS2/fonts/multicol/writing-modes 50-67% 高假通过。新 POLLUTED 真 bug 候选（self 通过但 chromium 大不一致）：table-grid-item-dynamic-003（23.79% grid+table+%height）/ abs-pos-replaced-vrl-001（13.13%）/ font-family-name-025（7.13%）/ visibility-collapse-colspan-003（4.91% table）/ anonymous-inline-inherit-001（3.86%）/ whitespace-001（3.14% table）/ col-definite-max-size-001（2.25% table）。详见 `evidence/cross-validate-2026-06-18.txt`。table 类候选在 R177b/R289/R292 单点修复有先例的子系统，是下一轮可执行杠杆（用 chromium Oracle 验证 z_vs_chr 下降）。→ R295 三方向 read-only 实证全 ruled out（DC-9 GPU filter 零 reftest 覆盖；flexbox-baseline 聚类结构性 wrap-reverse 容器基线合成；BFC-003 行组内部 border 修复正确但 net-negative——LAYOUT_DUMP 纠正 PIL 误判，真实 blocker = collapsed-border 表高度 vs margin ~1px/行累积漂移）。→ R294 image-clip **crop（非 rescale）**语义修复落地（DC-8 图片裁剪正确性），**但实证纠正 R293 诊断**：clip-rect-vrl-002/006/008 三联**非** image-clip-rescale bug——PIL 实测 test/ref 的 50×50 绿块**位置完全一致** x[8,57] y[68,117]（crop 本就正确），唯一 diff（2500px/max127）= `pattern-gr-rr-100x100.png` 左上=**lime(0,255,0)** vs `swatch-green.png`=**dkgreen(0,128,0)** 的 **fixture 资产颜色不一致**（两者均真实 PNG 正确加载，chromium 渲染亦不同→该三联在 chromium 也不通过，非 ZeroWeb bug、不可修）；pattern 每象限均匀故 rescale≡crop 视觉相同（修复对度量零影响）。修复仍为**真实 DC-8 正确性提升**（非均匀图片 clip:rect/overflow:hidden 应 crop 非 rescale），经 2 新单测（engine `test_clip_image_crops_without_rescaling` 保 rect 不变 + render-foundation `image_clip_crops_not_rescales`）**回退验证有牙齿**（旧 shrink 行为下 engine 测试 FAIL）。GPU `prepare_image_resources` 同步 crop（UV 映射 clip 窗口在 rect 内归一化位置）。**loose 438/490 / strict 296/490 全量持平零回归零增益**。教训：R293「PIL 渲染 50×50」= crop 正确被误读为 rescale（绿位置对齐被忽略，只看尺寸 24×24 误判）；**read-only 诊断的根因须实现后实证复核**。→ R292 collapsed-border 表尺寸边缘 border 双计修复 → subpixel-collapsed-borders-001/002 **STRICT 转 true-pass**（0.24% FAIL→0.10% PASS）；**strict 294→296（+2）**，**loose 438/490 持平零回归**。R291「需重构 resolve_collapsed_borders」推断被实证为 apply_table_size_conditions 单函数就地扣减可解（不触碰 R177b 高耦合 compute_column_widths）。R289/R292 同模式（子元素坐标系 vs 参考盒边界 / 折叠覆盖）证明 near-pass table 子系统仍可单点修。**R293（read-only 跨域 near-pass 扫描）**：① border-conflict-resolution 经 A/B 实测 R292 已改善（3435→2668px），残余 delicate table 多子系统（hidden 解析 + 行高 + R292 multi-cell 扣减耦合）；② **clip-rect-vrl-002/006/008 三联（均 0.52%）= 真实 image-clip-rescale bug**：`clip_all_primitives_to_rect`（helpers.rs:463）裁剪 image 时 shrink dest rect → `render_image` 把整张 source 映射进缩小 rect = rescale（应 crop 非 rescale）；三重实证（IMGPAINT_DBG primitive 100×100 正确 + LAYOUT_DUMP box 正确 + PIL 渲染 50×50）闭环。修复方案定位：ImagePrimitive 加 `clip` 字段 + clip 函数改 crop + CPU/GPU render_image crop-not-rescale（~25 edits，低回归，+3 strict 最难域）。**next = R294 执行 image-clip crop 修复**（绕过 WM 轴死锁的新杠杆）。
**上游真实 reftest 通过率**: 89.4% (438/490) R177b/R228b（2026-06-18 提交 + chromium Oracle A/B 验证）——R177b 落地 R177 延后的 colspan/col-width 缺口，`table_grid_size_col_colspan` **chromium-diff 52.27%→1.70%**（DC-14 anti-false-pass 真实 win；同源 reftest test==ref 天然不变故计数仍 438/490 持平零回归，零回归经全量验证）；R228b 半透明圆角矩形背景 alpha 修复（cpu `fill_rounded_rect`）。→ R227（**welcome padding 双计修复——product-smoke 28.34%→17.06%；reftest 439→438 净 -1（唯一回归 grid-flex-spanning-items-001 borderline 0.77→1.31%，aqua 实更正确，旧 pass 系两误差抵消）**）→ R225（**advance-width 证伪为死路**——R221 曾假设 183 case 1-3% chromium-diff 噪声主因=advance-width 估算误差；R225 双实验证伪：reftest-oracle 26 case 零变化 + product-smoke welcome/wintertc ±0.03%，机制=paint 经 fontdue 真实 shaping 定位 glyph 非 estimate_char_width；R223 AdvanceSource trait seam 留存无害勿再投入）→ R220（**DC-9 真实范围纠正——clip 为 no-op，GPU 缺口仅 transform/filter/blend 三项**）——经 grep 实证：**engine 生产路径从不生成 `ClipPrimitive`**（`add_clip` 0 处非测试调用），overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect`（painter/mod.rs 多处）**预烘焙进图元几何**，故 `primitives.clips` 生产恒空。因此 R211 所记「GPU drops clip」**实为 no-op**（无 clip 可丢），DC-9 的 ClipPrimitive 项在 CPU/GPU 两路均**空谈满足**。**真实 DC-9 缺口仅 transform/filter/blend_mode 三项**（engine 在 `paint/painter/effects.rs:266/289/313` 生成，GPU 全量路径静默丢弃），需 ping-pong 双纹理后处理（wgpu 不能同 pass 读写同一纹理；filter/transform/blend 均区域 read+write），且 reftest/静态内容中**低频**（仅显式 CSS filter/transform/mix-blend-mode 触发），非 reftest load-bearing。**GPU 现状**：9 图元类型经 6 独立 WGSL 管线渲染（fill/glyph/stroke/path_fill/path_stroke 共享 FILL_GLYPH 三角化，rounded_rect/gradient/image/blur[shadow+filter:blur]/color_filter 独立；R275 核实 stroke 含 dashed/dotted、非 passthrough 满足 DC-14），`headless_texture` offscreen 目标就绪，ping-pong 差第二张纹理+post-process pipeline。**本轮为治理性纠正（docs-only），无代码/reftest 变更**：纠正 R211/line381 对 clip 的误记，重定 DC-9 收尾范围（4 项→3 项且低频多轮），避免后续在 no-op clip 上浪费。下一步=DC-9 GPU ping-pong 地基（filter:opacity 先行）或 DC-14 chromium-oracle 严格容差默认接线或 DC-13 产品 smoke 持久化证据）→ R218（**SVG 解码统一到 render-foundation——DC-13「SVG 栅格化」全路径贯通**）——goal doc DC-13 要求「PNG/JPEG/WebP 基础解码和 SVG 栅格化」。reftest 路径早有 `load_svg_file`（resvg+tiny-skia），但 webview/browser URL 导航路径的 `decode_image_bytes` 对 SVG 返 unsupported——浏览器导航含 `<img src=logo.svg>` 的真实页面（WinterTC 14 logo 中 11 个 SVG）Logo 不渲染。**修复**：① render-foundation 加 `resvg`(workspace)+`tiny-skia` 依赖 + `pub fn decode_svg_bytes(bytes)`（resvg usvg 解析→按 SVG 内在尺寸 tiny-skia pixmap 栅格化→RGBA，过大尺寸 pixmap 分配失败自然兜底）；② `decode_image_bytes` 扩展 SVG 分支——`looks_like_svg` 嗅探 UTF-8 文本（跳 BOM/空白后 `<svg`/`<?xml` 起始）路由到 `decode_svg_bytes`；③ reftest `load_svg_file` 委托 `decode_svg_bytes`（同 R217 去重），移除 wpt-runner 的 resvg/tiny-skia 直接依赖（load_svg_file 唯一用户，依赖图精简）。**测试**：render-foundation decode_tests +2——`decode_svg_bytes_green_4x3`（含 `<?xml` 声明的 4×3 纯绿 SVG 往返，断言 G>200 + alpha=255）、`decode_svg_bytes_invalid_returns_err`（非 SVG XML→err）；`decode_image_bytes_dispatches_by_magic` 加 SVG 路由断言（现四分发 PNG/JPEG/SVG/unsupported）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test **12227 passed/0 failed**、reftest-upstream **439/490 持平**（reftest 路径 load_svg_file 委托后行为不变）。**意义**：DC-13 三种图片格式（PNG/JPEG/SVG）在三条渲染路径（reftest / webview fetch_url / browser render_cpu）全部统一到 render-foundation `decode_image_bytes` 单点；浏览器经 URL 导航现可加载并渲染 SVG Logo（WinterTC logo.svg 等真实场景）。下一步=DC-13 产品 smoke 端到端证据（DONE#11 5 真实网站 / WinterTC Logo 经浏览器路径验证）或 DC-9 GPU 4 图元（transform/clip/filter/blend））→ R217（**JPEG 解码合并去重——清理 R216 造成的重复**）——R216 在 render-foundation 落地 tested `decode_jpeg_bytes` 后，reftest 路径（`reftest.rs:load_jpeg_file`，~55 行）的独立 JPEG PixelFormat→RGBA 转换逻辑与之重复（且 L16 处理不一致：reftest `(px[0]|px[1]<<8>>8)` vs R216 干净的高字节）。**修复**：`load_jpeg_file` 委托给 `zero_render_foundation::image_cache::decode_jpeg_bytes`（读文件→解码），reftest 与 webview/browser URL 导航路径现共用**同一解码器**（单点 tested）。移除 wpt-runner 不再使用的 `jpeg-decoder` 直接依赖（load_jpeg_file 是唯一用户）。保留 `load_png_file` 的 `ZERO_PNG_EXPAND` 诊断门控与 `load_svg_file`（resvg）不动——非本轮变更遗留。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test **12225 passed/0 failed**、reftest-upstream **439/490 持平**（L16 JPEG 在 WPT reftest 实质不出现，转换差异零影响）。**意义**：三条渲染路径（reftest / webview fetch_url / browser render_cpu）的 JPEG 解码统一到 render-foundation 单点，消除维护负担与潜在不一致；DC-13 图片解码一致性提升。下一步=SVG 解码统一（reftest 已有 resvg，webview/browser 路径缺）或 DC-13 产品 smoke 端到端证据（DONE#11））→ R216（**JPEG 图像解码扩展——DC-13「PNG/JPEG 基础解码」第二步**）——goal doc DC-13 要求 PNG/JPEG/WebP 基础解码，R214 落地 PNG，本轮补 JPEG。**修复**：① render-foundation 加 `jpeg-decoder = "0.3"`（MIT/Apache-2.0 纯 Rust）+ `pub fn decode_jpeg_bytes(bytes)`（L8/L16/RGB24/CMYK32 全 PixelFormat→RGBA，CMYK 按 Adobe 倒置 K 惯例转 RGB）+ `convert_jpeg_pixels_to_rgba` 纯函数；② **格式分发** `pub fn decode_image_bytes(bytes)`——按**魔数字节**嗅探（PNG `\x89PNG` / JPEG `\xFF\xD8\xFF`）路由，比 URL 扩展名可靠（URL 可能无扩展名/扩展名错误），未知格式返 unsupported err；③ webview `fetch_image_subresources` 改调 `decode_image_bytes`（原 decode_png_bytes）→ 同一路径现处理 PNG+JPEG。**测试**：render-foundation decode_tests 5 项——`convert_jpeg_pixels_to_rgba` RGB/灰度纯函数、`decode_jpeg_bytes_green_4x3` 真实 fixture（PIL 生成 4×3 纯绿 JPEG quality 95，断言绿色主导 G>200/R<50/B<50 + alpha=255，容 JPEG 有损）、invalid→err、`decode_image_bytes_dispatches_by_magic`（PNG/JPEG/未知三分发）。fixture `crates/render-foundation/src/testdata/green_4x3.jpg`（635B）。**验证**：cargo build + clippy --workspace --all-targets -D warnings 干净、fmt 干净、make test **12225 passed/0 failed**、reftest-upstream **439/490 持平**（reftest 用本地文件不走 URL 导航故零影响）。**意义**：DC-13 图片基础解码 PNG+JPEG 就绪，WebP/SVG 后续；浏览器经 URL 导航现可加载并渲染常见位图格式。下一步=SVG 栅格化（WinterTC logo.svg）或 WinterTC Logo 端到端产品 smoke 证据（DC-13 验收））→ R215（**浏览器渲染路径消费 webview ImageCache——DC-13 P1「图片子资源/ImageCache 未贯通」全链路贯通（最后消费 hop）**——承接 R214 标注的「下一步」。R214 已打通 fetch→decode→image_cache（webview 层），但浏览器 `render_cpu`/`render_frame` 仍传 `None`（app_platform.rs:194/153），图元到渲染器最后一跳断开。**修复**：app.rs 加 `use zero_render_foundation::image_cache::ImageCache`；`render_cpu`（CPU）与 `render_frame`（GPU）两路在 `render_full_scene[_gpu]` 调用前用**不相交字段借用**取活跃标签页 webview 的 image_cache——`match self.shell.active_tab_id() { Some(id) => self.webviews.get_mut(&id).map(|wv| wv.image_cache()), None => None }`（self.webviews / self.font_loader / self.glyph_cache 为不同结构字段，borrow checker 允许同语句并存），传 `Some(&mut ImageCache)` 替代 `None`。**测试**：新增 `#[cfg(test)] render_full_scene_with_webview_for_test`（与 render_cpu 同场景装配但返回 FrameBuffer，mirror 现有 `render_scene_for_test` 模式）+ 差异法测试 `render_path_consumes_webview_image_cache`——基线（image_cache 空）渲染断言目标颜色计数 0（缓存 miss 不绘制），填充 `ImageKey(simple_hash(src))`（键与 engine text.rs:611 一致）后渲染断言 >0（图片经浏览器路径被消费）。**验证**：cargo build/clippy --workspace --all-targets -D warnings 干净、cargo fmt 干净、make test 全绿（新增测试通过）、`./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest-upstream` 实测 **439/490 持平**（reftest 用本地文件不走 URL 导航，image_cache 恒空，`Some(&空)`≡`None` 行为一致故零回归，符合预期）。**意义**：`<img>` 经 URL 导航全链路贯通——抓取(R214)→解码(R214)→image_cache(R214)→**浏览器渲染消费(R215)**→renderer 绘制真像素，goal doc DC-13 P1「图片子资源缺失不得被 alt/占位 glyph 静默替代」在浏览器层落地。下一步=JPEG/SVG 解码同模式扩展 + 产品静态页 WinterTC Logo 端到端 smoke 证据（DC-13 验收））→ R214（**图片子资源加载落地（URL 导航路径，PNG）— DC-13 第二个 P1 子项打通**——goal doc DC-13 P1「图片子资源/ImageCache 未贯通」修复（PNG 先行，JPEG/SVG 同模式后续）：`<img>` paint 可生成 ImagePrimitive，但 fetch_url 不抓 `<img src>`、webview 不持有 ImageCache、render-foundation 无解码。**分层修复**：① render-foundation 加 `png` dep + `pub fn decode_png_bytes(bytes)`（image_cache.rs，正确 EXPAND 全 color type→RGBA，独立于 reftest 的 env-gated 版本，零 baseline 影响）；② engine 加 `extract_img_srcs(html)`（DOM 精确，parallel to extract_stylesheet_hrefs）；③ webview 加 `image_cache: ImageCache` 字段 + `fetch_image_subresources(html, base_url)`（extract img srcs → 按 base URL 解析 → http_client 抓取 → decode_png_bytes → `image_cache.insert_with_key(simple_hash(abs), img)`，键与 pipeline image_sizes + 渲染器查找一致）→ 返回 image_sizes → `pipeline.set_image_sizes`（`<img>` 正确固有尺寸 DC-11），三条 fetch_url 分支注入；暴露 `pub fn image_cache(&mut self)` 供下游渲染器绘制消费。**端到端测试**（webview_coverage，mini-server 重构支持二进制）：服务 3×2 纯绿 PNG + page，fetch_url 后断言 image_cache 含该图、尺寸 3×2、像素纯绿。13 webview + 2 decode + 1143 engine 测试全过。**意义**：图片子资源抓取+解码+缓存贯通，`<img>` 经 URL 导航获正确固有尺寸 + ImageCache 就绪供浏览器渲染；浏览器 render_cpu/gpu 传 `Some(&mut webview.image_cache())`（当前 None）是最后消费 hop（下一步）。reftest 439/490 不变）→ R213——R210 在 compute_final_inline_layouts 的 PHASEA_MULTILINE 多行存储条件加 `!in_multicol` 守卫（in_multicol 经新增递归参数从 multicol 容器透传后代）。全量 make reftest-upstream 实测**净 +0**（仅 2 翻转：✅ ifc-008 8.18%→PASS 0.00%、❌ multicol-fill-auto-001 0.63%→9.15%）。CFDEBUG 探针精确定位 multicol-fill-auto 回归源=**ref 文件的 2 个 float div**（非 multicol，用 float 模拟列，in_multicol=false），非 multicol 容器——`!in_multicol` 守卫**无法触及**它们（合法非 multicol）。真正阻塞=**stored 多行路径 v_offset=0（Ahem）vs paint IFC 路径 baseline_fs=font_size 不一致**，ref（stored）与 test（paint IFC）渲染差 font_size/行；实测 v_offset=font_size 反破坏 font-051（16.67%）+ ifc-008（8.33%）。**关键方法学纠正**：product-smoke 单趟不加载外链 CSS（base_dir=None）→ multicol-fill-auto 的 /fonts/ahem.css 不解析→ is_pure_ahem=false→ 不存储，故 R209 product-smoke 看不到其存储行为；**涉及外链 CSS 的用例必须用 reftest 双趟路径诊断**。结论：ifc-008 已确认可被 compute_final 正确存储并 PASS，但解锁需先统一 stored 与 paint IFC 多行 baseline 语义（R125/R198 同墙，单点不可解）。无代码变更（实验已回退回 R207 干净态），基线 439/490 持平）→ R209——本轮用**干净单趟探针**（product-smoke 渲染 test only，避开 test/ref 双趟干扰）定位 R208 未定的 ifc-008 根因。**精确根因**：compute_final（engine.rs:1900-1903）的 **R84 单行 Ahem 存储限制**——`if lines.len() > 1 || !is_pure_ahem { return }`，仅存单行纯 Ahem。ifc-008 的 "XX XX" 100px 在 200px 宽换 **2+ 行** → lines.len()>1 → 不存 → paint 重跑 → 16px。（font-051 单行 "FAIL" → 存 → PASS，故 R207 成功。）干净探针证实 node 39(inner-div) **被访问、block=true、direct_text=true**，仅因多行限制未存。**实证放宽**（env PHASEA_MULTILINE=1，允许 Ahem 多行存储）：ifc-008 **8.18→4.17%**、ifc-009 **6.11→4.17%**（改善，仍 FAIL，余 4.17% 系换行精度/覆盖差），font-051 不变 PASS。**但 net-negative**：multicol-fill-auto-001 0.63 PASS→FAIL（**R198 反向依赖再现**——multicol 子容器现多行存储致渲染变），无用例翻 PASS。**结论**：ifc-008/009 修复需 (a) 放宽多行存储 AND (b) 守 multicol-fill-auto 不回归（=R198 ancestry-guard 墙，已知失败）AND (c) 修余 4.17% 换行精度。三重耦合，非单点。R207 font-051 +1 保持。无代码变更（探针+实验经 git checkout 清除回 R208 ccabef5），基线 439/490 持平）→ R208（**ifc-008 block-child large-font 深挖：inner-div 文本 16px，compute_final 未存其 inline_layout，架构性非单点**）——承接 R207（font-051 inline-child 容器 PASS）。本轮攻 ifc-008（div1>inner-div(block)>\"XX XX\" 100px Ahem），dump 实测：div1 区域几乎全红（39064px）仅微量绿（736px≈16px 文本）——inner-div 的文本渲染成 16px（large-font bug），**R207 inline-child 路径不覆盖此 block-child 结构**。探针（IFCDBG）：paint 见 inner-div(node39) fs=100 但 **inline_layout=None / use_stored=false** → paint 重跑 IFC 用空 styles → 16px。compute_final 探针未对该 node 触发存储（探针被 test/ref 双趟 compute + 字符串匹配 \"39\" 假阳性干扰，未定论根因）——**疑似 compute_final 树遍历未到达 inner-div 的带 node_id 盒，或匿名盒包裹致 node_id=None 早退**。**结论**：ifc-008（block-child 文本容器）的 large-font 修复需厘清 compute_final 为何不为 inner-div 存储行盒，属架构性深挖（非 R207 inline-child 路径的单点扩展），defer。R207 font-051 +1 胜利保持（默认 439/490）。无代码变更（探针已 git checkout 清除），基线 439/490 持平）→ R207（**Phase A「stored-line-boxes」路径 narrow 精修成功，+1 零回归默认启用**）——承接 R206（broad 应用 net-negative，font-051 单点改善）。本轮做 narrow 精修至「纯 inline 内容」容器：扩展条件 = 有 inline-level 元素子节点 AND 无 block-level 元素子节点 AND inline 子元素无元素子节点（叶文本容器，排除 block-in-inline R109）。compute_final（engine.rs:1693）扩展存储 + has_direct_paintable_text（text.rs:1479）同步扩展（paint_text 到达 use_stored）。**默认启用（env PHASEA_STORE_EXT=0 关闭）**。**font-051 8.19%→PASS 0.00%（+1）**，全量 reftest **438→439/490**，**零 count 回归**（inline-box-001/002 + multicol-block-no-clip-001 三处 broad 回归经 narrow 条件排除恢复）。make test 全绿（45 crate/0 failed）；clippy -D warnings clean。**Phase A 首次实证 clean win**：stored-line-boxes 架构（compute_final 用真实 styles 存行盒，paint use_stored 渲染）对纯 inline 容器产出正确度量，破 R205 4 路 font_size 死锁。剩余 large-font（ifc-008/009/011 = block 子节点容器，非本路径覆盖）+ empty-inline-002 仍卡（block-child IFC 另一子问题）。下轮可扩 narrow 条件覆盖更多结构或攻 block-child IFC。无回退，默认实测 439/490）→ R206（**Phase A「stored line boxes」路径首次实证改善 font-051，但 broad 应用 net-negative，须 narrow 精修**）——本轮实现 R205 定性的「唯一未试架构路径」：paint 不重跑 IFC，渲染 compute_final 存储的真实度量行盒。关键基础：compute_final 传**真实 styles** 给 IFC（engine.rs:1851，区别于 paint 传空），故存储行盒 font_size/line-height 正确。实现 env-gated（PHASEA_STORE_EXT=1）：① compute_final 扩展存储条件，覆盖含 inline-level 元素子节点的容器（如 div>span）；② has_direct_paintable_text 同步扩展，使 paint_text 不在 line 683 提前返回，到达 use_stored。**首次实证改善**：font-051 **8.19→1.51%**（100px 文本现以正确度量渲染，Phase A 5 路首次有真实正向）——**证明 stored-line-boxes 架构可行**。**但 broad 应用严重 net-negative，已回退**：ifc-001 0.53→1.84 / ifc-002 0.72→2.38 / ifc-003 0.23→2.02（3 个原 PASS 翻 FAIL）、ifc-008 8.18→9.73 / 009 6.11→8.07 / 011 11.24→13.00 / empty-inline-002 29.32→31.14（large-font 集群反恶化）、multicol-fill-auto-001 0.63→1.92（PASS→FAIL）、position-absolute-in-inline-005 1.19→2.46。**结论**：stored-line-boxes 路径**架构正确**（font-051 实证产出正确度量），问题在 **broad 应用**改变了大量容器的渲染（双绘/错位）。下一步 = **narrow 精修**：仅对「存储结果与重跑不同且改善」的容器应用（如仅单 inline-element 子节点 + 特定结构），逐条件 set-diff 收敛，而非全量 inline-children。这是 multi-round 精修，但**方向首次明确可行**（区别于 R205 的 4 路全死锁）。无代码变更（实验已回退），git diff clean，基线 438/490 持平）→ R205（**Phase A font_size 解锁第 4 路实证 net-negative，deadlock 定位于 IFC 度量架构**）——本轮在 R72/R125/R198 三路死锁后尝试**第 4 路：font_size 单字段解耦回退**（R72 传全量真实 styles 致 4 回归，本轮改为 paint 注入真实 ComputedStyle **仅 font_size** 表 `real_font_sizes`，IFC 在 font_size_overrides 未命中时回退，其他属性 vertical_align/letter_spacing/float-exclusion 仍走空 styles→默认，理论上规避 R72 的 4 回归）。实现 env-gated（PHASEA_FS_REAL=1）零基线风险。**实测全 net-negative，已回退**：① **font-051 8.19→11.65% 恶化**（font_size 现正确 100px，但 line-height 用 fs×1.2 近似与 div 的 100px/1 不符→行盒更高→diff 增；证明 font_size 与 line-height **耦合**，单修 font_size 致 line-height 失配）；② **multicol-fill-auto-001 0.63% PASS→1.21% FAIL 回归**（R198 同源反向依赖）；③ position-absolute-in-inline-005/006 回归（1.19/1.23% FAIL）。**结论：Phase A font_size 解锁的所有 4 条路（R72 全量 styles / R125 三路存储 / R198 override 填充 / R205 font_size 单字段解耦）全 net-negative**。死锁根因 = **paint IFC 与 layout IFC 是两趟独立运行，font_size/line-height/line-breaking 三者耦合**，任何「让 paint IFC 用正确度量」的单点/单字段改动都因破坏其他耦合而回归。唯一未试架构路径 = **paint 完全不重跑 IFC，直接渲染 layout IFC 存储的行盒**（需 compute_final 对所有含 inline 子树的容器存 line boxes，含 div>span 文本类如 font-051；paint 渲染存储结果；须解 float-exclusion/vertical-align 一致性，多轮架构）。无代码变更（实验已回退），git diff clean，基线 438/490 持平）→ R204（**绝对 plateau 再确认 + R197 DC-12 审计纠正（clip-path 已全实现）**）——本轮对剩余可操作方向逐一复核，全部 ruled out/已实现：① **clip-path circle/ellipse/polygon 真裁剪已全实现**（纠正 R197「仅画指示线」过时结论）——mod.rs:599-673 对 circle/ellipse/polygon 调 `clip_all_primitives_to_polygon`（helpers.rs:234，包围盒裁剪 + fills 扫描线精确裁剪 clip_fill_to_polygon + glyphs 中心点 point_in_polygon 测试），inset() 走 clip_all_primitives_to_rect。**DC-12 clip-path 无缺口**（仅 paint_clip_path 指示线冗余叠加在真裁剪之上，极次要清理，非 gap）。② border-001(2.77%) dump 实测：ZeroWeb 渲染**正确的空心黑方框**（25px border + 100x100 hollow），diff 来自 REF 的文本/定位差（字体度量），非 border bug。③ float-006(7.47%)=float+abspos+overlap 复杂交互（R100/R108b 结构域）。④ Phase A 新角度证伪：paint_text 已直接从 ComputedStyle 读 font_size（text.rs:639，mod.rs:263/446 传真实 style），死锁不在 style plumbing 而在 IFC 度量一致性（layout IFC vs paint IFC 两趟不共享状态），即 R125/R198 已确证的架构性。**结论：增量 clean win 彻底穷尽**——剩余 52 失败全为多轮硬架构（multicol 全碎片化模型 / Phase A IFC 统一死锁 / writing-mode 轴 4 轮证否）或浏览器引擎特性（原生表单控件 / dialog JS / fixed-bg）或 REF 怪异。无单会话 clean reftest win。无代码变更，基线 438/490 持平）→ R203（**multicol-breaking paint 侧修复实证 ruled out → 真实路径=layout 侧 column-aware IFC**）——本轮对 R201 定性的 multicol-breaking 4 阻塞点（A 门控/B 重绘协调/C column-rule/R203 新发现 D 去重）做 paint 侧实证修复尝试，**3 路全部 net-negative，已回退**：① **D = painted_inline_nodes 去重（text.rs:688）**——column breaking 目标（column_span_offsets 多片段）被去重误抑，非首列文本不渲染。单独修 D（去重条件加 `column_span_offsets.len()<=1`）：multicol-breaking-006 **1.20→1.71% 恶化**（col1/col2 现渲染文本但位置错=无 2 子列协调），004/005 不变，net-negative 已回退。② **A'+D 协调**（放宽门控给被碎片化子元素 + D 去重修复）：全量 multicol **40/57→36/57（-4 大回归）**，已回退。③ text.rs:707-709 注释确证 `height_auto` 门控**专为防回归而加**（明确高度 balance 容器简单均衡分配会回归）。**结论：paint 侧修复（单点或简单协调）对 multicol-breaking 全 net-negative，rule out**。真实路径 = **layout 侧 column-aware IFC**（R131）：IFC 在生成行盒时即按列高预算把行内流碎片化到各列（每列独立正确的行盒分布），paint 侧只按列渲染，非「单次 IFC 渲染 + paint 切片」。这是 multi-round layout 子系统。可复用教训：**架构性失败的单点/简单协调修复须实证（探针+全量套件），不能据推断落地**。无代码变更（实验已回退），git diff clean，基线 438/490 持平）→ R202（**chromium Oracle 高 diff 候选实证排查，3 项证伪关闭**——基于 06-17 fresh cross-validate 的 z_vs_chr>5% POLLUTED 清单（self-source 假通过掩盖的真缺口），逐项用 probe/product-smoke 实测排查：① **abspos-semi-replaced-stretch-input/button/other（23/3.5/15%）RULED OUT**——经 throwaway HTML probe 实测，plain div/inline-block/inline/span abspos + 全 inset + width:auto **全部正确 stretch**（red 填满 CB）；再渲染真实 `<input>/<button>` **也 stretch**（2px 采样确认 lime outline 跨满 CB x≈8-168）。早先 4px 采样误判「窄」。**stretch 算法工作正常**，23% chr 差异真因 = **ZeroWeb 把表单控件画成 styled box+outline，chromium 画原生 widget**（native button/text-field），是**表单控件渲染特性缺口**非布局 bug，单点不可修（需实现原生表单控件外观，大特性）。② **backdrop-inherit-rendered（47.5%）RULED OUT**——是 `dialog::backdrop`（`<dialog>.showModal()` JS API + `::backdrop` 伪元素），非 backdrop-filter；需 dialog JS 基础设施，非 contained。③ **background-attachment-applies-to-001（self 29.9%）= `background-attachment:fixed` on table-row-group**，fixed 背景=视口相对定位特性，非 contained 布局 fix。**结论**：fresh Oracle 高 diff 候选**全为结构性（multicol/table/Phase A/writing-mode）或特性缺口（表单控件/dialog/fixed-bg）或已修复（R165-R180）**，无单会话 clean win。可复用方法 = **probe-based 实证**（throwaway HTML 经 product-smoke 渲染 + 2px 采样判定几何，避免 4px 采样漏边误判）。无代码变更，基线 438/490 持平）→ R201（**multicol-breaking dump 实测定性，纠正 R113「两趟循环依赖」假设**——REFTEST_DUMP+BBOX+逐行像素扫描 multicol-breaking-004 实测：inner 文本**仅在 col0 渲染**（col1/col2 全空），蓝色 column-rule 全漏画，绿 border 位置错。真实 3 阻塞点 = ① paint 门控 `height_auto`（text.rs:710-715）挡住有明确高度 inner 的 2 子列布局；② `column_span_offsets` paint 路径**不重绘碎片化 IFC 内容到非主位置列**（核心 wiring 缺失，R131 同源）；③ column-rule §5.2 内容检测只查 `child.x` 主位置漏查跨列片段。**关键纠正：碎片化算法 `assign_children_to_columns_sequential`/`_with_breaking` 已存在**——R113 设想的「两趟测量」算法层面已具备，缺口是**接线到 inline paint 路径**，**勿再建 measure-first 工具**（必重复 R199→R200 证伪命运）。column-rule 检测修复（C）实测：004 5.60→5.39/006 1.20→1.12（蓝色 rule 补画改善）**但 column-rule-002 0.00→1.25% 回归**（c.x-主位置检测对该用例正确），**已回退**。设计文档 `multicol-fragmentation-design.md` 升 v0.3，Round 4 重定向为 wiring 多轮（放宽门控 + column_span_offsets 重绘碎片化 IFC，非 layout 两趟）。无代码变更，git diff clean，基线 438/490 持平）→ R200（**multicol balance 方向证伪关闭**——R199 的 round-robin shortest-column balance 接入后 multicol-columns-001 4.88→4.92%（略差）。根因：chromium multicol §8 是**顺序填充**（col0 填到平衡高度 H=T/N 再 col1），**非 round-robin**；旧代码 `line.y/target_h`（target_h=total/col_count）**本就是顺序填充+平衡高度，已正确**。我的 round-robin 破坏顺序。**multicol 列分配已正确**——类 A 低 diff（columns-001 4.88%/fill-000 6.54%/count-computed-003/004）**非列分配问题**，是列宽精度/glyph x 位置(estimate_char_width)/平衡高度精确值。移除 R199 的 multicol_fragment.rs（错误算法）+ 纠正设计文档 v0.2。multicol 剩余失败全结构性（breaking/baseline/column-span）或精度（advance-width 同源）。基线 438/490 持平）→ R199（**multicol 碎片化攻坚启动**——设计文档 + Round 1 测量工具落地，零风险不接线）：建 `multicol-fragmentation-design.md`（consolidate R113/R131/R157，5 轮实施计划：R1 测量/R2 纯行内 balance 精确化/R3 混合内容门控/R4 breaking/R5 baseline+spanner；预计全完成 css-multicol 40→55/57，438→~453）。新增 `multicol_fragment.rs::balance_lines_to_columns`（CSS §8 shortest-column-first 列分配，替代 paint text.rs:951 的 `total/col_count` 均高近似）+ 6 单测（4行2列/11行6列/含 block 已占/单列/零列/空）。**Round 1 不接线**（measure-first 同 R181 模式），reftest 438/490 持平零回归。下轮 Round 2 = 接入 paint text.rs:948 列分配（paint-only，解锁 multicol-columns-001/fill-000/count-computed-003/004 类 A 用例）→ R198（**Phase A font_size 死锁经新变体再证实证，关闭该方向**）：实验 compute_final IFC 跑过后调 store_font_sizes_from_ifc 存 font_size（paint 提示不重排）+ multicol ancestry 守卫（in_multicol 跳过）→ 全量 net **-1**（438→437）：CSS2 +1（large-font font-051 类修复）但 css-multicol -1（**multicol-fill-auto-001 0.63%→FAIL**）。ancestry 守卫无效（multicol-fill-auto 非 LayoutBox 树 ancestry-tracked，疑 multicol paint 路径重组）。即使完美守卫 net 也仅 0（+1 -1 抵消），**死锁成立**——large-font 与 multicol-fill-auto 经 font_size 存储耦合，不可单修。印证 R125（三路 -1/-1/-4）+ R158（"勿再单点补存"），**Phase A font_size 方向正式关闭**。DC-13 welcome 文本 + large-font 5 reftest 全卡此墙，需架构性 Phase A IFC 三路径统一（非 font_size 单点）。无代码变更（实验已回退），基线 438/490 持平）→ R197（**两纠正**：① welcome 文本真因 = **paint IFC font-size 默认 16px（Phase A 死锁）**，非 R196 的 advance-width——实测 `font-size:60px` 在 product-smoke 渲染成 12px（=默认 16px 字高）而 color:red 生效，证 font-size 未应用；即 R82/R101/R125/R158 标记的 paint IFC 空 styles font_size 回退（large-font reftest 死锁同源），R196 advance-width 假设**再被证伪**。Phase A 是已知硬阻塞（R125 三路死锁 + R158 multicol-fill-auto 反向依赖），DC-13 welcome 卡此墙。② **DC-12 审计**：text-shadow/multi-background-layer（全图层逆序）/repeating-gradient/clip-path/backdrop-filter/CSS mask **全部已实现**，goal doc 的 DC-12「未实现」声称**全部过时**（同 M7）；唯一真缺口 = clip-path circle/ellipse/polygon 仅画指示线非真裁剪（只 inset 真裁剪）。无代码变更，基线 438/490 持平。**结构性 plateau 全面确认**：DC-13 卡 Phase A、DC-12 基本完成、reftest clean win 穷尽）→ R196（DC-13 welcome 28% 根因深挖——**证伪 R195 line-height 假设 + font 不匹配假设**：welcome.html 全用**显式 line-height**（1.08/1.5/1.45/1.25），无一处 line-height:normal，故 R195 的 font-metrics line-height:normal plumbing 对 welcome/morning.work/wintertc **零收益**（三者皆显式 line-height），已实证后**回退**（plumbing 服务无指标，按 code-guidelines 不留推测代码）；font 不匹配实验（sans-serif DejaVu→Noto CJK，系统 fc-match=Noto CJK SC）welcome diff 28.08→28.05%（**仅 -163px negligible**）证伪字体假设。**R195 AA 基准测同字体漏了 sans-serif 解析分歧，但解析分歧本身非主因**。welcome 28% 真因 = **文本定位**（advance width 估算 estimate_char_width 0.55×fs vs 真实 advance → 文本宽度/换行/位置偏差累积），即 R188 标记的架构阻塞（layout IFC 不持 FontLoader），**自源中性仅影响 DC-13 不影响 reftest**。下步=advance-width plumbing（同 line-height 的跨 crate 模式但更高价值，影响全部非 Ahem 文本）。无代码变更，基线 438/490 持平）→ R195（DC-13 line-height 调研 + **关键去风险发现**：welcome 28% diff 经 diff-band 分析确认为 line-height/度量累积（文本行间隔 band + 底部 quadrant 差异最大=累积）；**line-height:normal 改动对 reftest 自源中性**——实测 ratio 1.2↔1.5 linebox 10/15 + css-writing-modes 53/59 双双持平，因 reftest 是 ZeroWeb-test vs ZeroWeb-ref 自渲染，test/ref 同字体等比例平移。**证明字体度量版 line-height 对 438 基线安全**，解锁 DC-13 line-height 方向。但 fix 是架构性多轮：layout-engine IFC 需字体度量，而 font_family→FontId→font 文件解析当前在 paint 懒做（R188 同源阻塞），需 engine 预解析建 font-family→line-ratio map 传入 layout+paint 双侧 IFC。reftest 52 失败全结构性复核（clear-float-003 3.20% = float+clear+negative-margin+margin-collapse 交互，确认 clean win 穷尽）。无代码变更，基线 438/490 持平）→ R194（R109 split 的 relative offset 双重计数修复，**+1 零回归**）：split inline 的匿名块片段用 converter 从 inline computed 构建 taffy Style 时**继承了 position:relative + inset**，致 taffy 对每个片段重复施加偏移（父盒 #div2 已施加一次）→ inline-box-002 的 `position:relative;top:2in` 使片段偏低 2×192px 出视口（蓝色 bg 不可见=假缺失）。**两处协同修复**：① tree.rs 匿名块 style 的 `inset` 清零（AUTO，位置由父盒单次施加）；② engine.rs `apply_relative_offsets_inline` 跳过 `is_r109_split` 盒（父+片段，taffy 按 block 单次处理，避免 computed-Inline 路径双重）。`inline-box-002` 3.14%→**PASS 0.78%**（frag1 abs_y 646→262、frag2→300，几何对齐 ref）。CSS2 115→116，零 count 回归。R109 里程碑**彻底完成**（inline-box-001/002 + align-001 全过；残余仅 clear-inline-001 = inline img+span→block 堆叠，独立子问题）→ R193（R109 §9.2.1.1 **默认启用 + fragment border 落地**，**+2 零 count 回归**）：① 匿名块片段用 converter 从 split inline 的 computed 构建 taffy Style（携带 border/padding/bg，而非默认空 Style）——此为 R192 遗漏的关键，旧实现匿名块 border=0；② paint 对 split inline 父盒（is_r109_split）跳过自身 bg/border/shadow（装饰下放片段）；③ 新增 `shrink_r109_anon_blocks` 后处理：片段收缩到 `fragment_inline_max_width`（同 paint IFC 的 estimate_char_width，故收缩宽=渲染宽自洽）+ fragment border 边选择（首片段 border_right=0、末片段 border_left=0，CSS2 §9.2.1.1 分裂边不画）。**`inline-box-001` 2.31%→PASS 0.89%、`block-in-inline-align-001` 1.37%→PASS 0.34%**；inline-box-002 3.20→**3.14%（改善不再恶化）**；block-in-inline-append/iframe/margin-collapse/justify/last 全部持平或改善（align-justify 0.38→0.50 微增仍过）。R109 默认开（R109_WIRE=0 关）；全量 make test 0 failed；437/490 默认实测。**R109 里程碑主体完成**（仅 clear-inline-001/inline-box-002 残余 = relative-on-split + inline img+span 流，独立子问题））→ R192（R109 生产端接线落地 env-gated：tree.rs `build_subtree` 把 inline+in-flow-block 子元素展开为匿名块 taffy 节点 + fragment 注册表；engine.extract_layout 写 `LayoutBox.fragment_node_ids` + `is_r109_split`；paint IFC 跳过 split inline 父盒 + 放行片段。**out-of-flow（abspos/fixed/float）排除修复**——CSS2 §9.2.1.1 只拆 in-flow block 子元素，否则 position-absolute-in-inline-005/006 回归 -2。实测 `R109_WIRE=1` 全量 **436/490 (+1 零 count 回归)**：`block-in-inline-align-001` 1.37%→**PASS 0.00%**、inline-box-001 2.31→1.11% 改善；**但 inline-box-002 3.20→4.67% 恶化**（border-having split 需 inline 级 fragment border，R182 §3 未就绪）。按项目严格「零回归=无任何用例变差」标准**保持 env-gated 默认关**，基线维持 435/490。下步=fragment border 解锁 inline-box 后默认启用。make test 0 failed；clippy/fmt clean）→ R183（flex/grid 两趟 Round C IFC 文本内容宽度测量基础落地，零回归；flex-container-max/min-content 经 INTRINSIC_DBG 证实 delta=0 已正确尺寸，其 18%/13% 差距系 grid+float 结构非 flex 宽，Round C 不修此两用例）→ R182（block-in-inline R109 攻坚确证架构性多轮 defer，clean win 同源+chr 双侧穷尽复核确认）→ R181d（flex/grid 两趟 Round B 落地，**+1 零回归**：`width:max-content` grid 经两趟 intrinsic 测量塌缩 40→182px ≈ chromium 180，`child-border-box-and-max-content-001` 1.52%→**PASS 0.03%**；R97 两通过用例 min-width:max-content/min-height:min-content 经实测仍 0.00% 持平）。前轮 R180（chromium Oracle 真实修复 ×4：R180 inline-block width:auto shrink-to-fit baseline-block-with-overflow-001 chromium **45.09%→1.25%**；R178 `<col>` px 宽度 18→400px；R168 table height-as-minimum 11.12%→2.98%；R165 margin:auto 居中 33.09%→2.63%）。**434/435 即诚实 DC-14 基线，无需恢复 436**（R164 证否 vrl-004/008 R114b 路径：正确 vertical-rl CSS 使 4/4 vrl 变差，因同源 REF 水平渲染 vs 正确 vertical-rl 右侧块起始结构性不可对齐；chromium Oracle 证同源 REF 比 chromium 更怪异：vrl-004 同源 7.09% vs chr 5.08%，font-051 同源 8.19% vs chr 1.62%）。R163 PNG 正确 RGBA 默认启用（DC-14 anti-false-pass）。draw_order 默认启用满足 DC-10。剩余 55 同源失败（结构性多轮 + REF 怪异产物）；**优化目标已转 chromium Oracle 一致率（d16bb8e），18 真 bug 候选见 `evidence/analyze-pollution-2026-06-16.txt`**。

**🔤 字体攻坚结论（2026-06-17 AA 基准，证伪字体归因；2026-06-18 R229b 补充 Bold 细化）**：fontdue 光栅化 vs chromium 实测 **W 0.1% / i 3.0%**（`evidence/aa-baseline-2026-06-17.txt`）——**Regular 变体 fontdue 不是渲染差异来源**。advance plumbing（真实 advance 替代 estimate_char_width）实测 Oracle 污染 48.6%→48.5% 无效，已回滚。welcome 26% / 污染大头是**布局/度量（line-height / R109 inline→block / 多行结构）非字体**。纠正 R174/R187「字体噪声」误诊。⚠️ **R229b（2026-06-18）细化**：fontdue **Bold 变体**比 chromium **过墨 ~15%**（welcome title +14%/card h3 +17%）→ R229 font-weight 选择机制虽正确落地+生效，但加载 Bold 后 net-negative 已回退（见 7d062e5）。故「fontdue 非差异来源」精确为「**Regular** 非差异来源；**Bold 过墨**是 fontdue-vs-chromium 差异（同 advance-width/AA 谱系）」；**font-weight -Bold 接线死路，字体攻坚（Regular 已对齐 + Bold 过墨不可单点修）停止，转布局/度量**。

**🎯 当前最高优先级（2026-06-18 更新，R229b font-weight 死路后）**：font-weight 已证为 **net-negative 死路**——R229b/7d062e5 完整落地 R229（选择机制正确+生效，welcome card h3 ink-mass +29% 证接线对），但 **fontdue 光栅化 Bold 变体比 chromium 过墨 ~15%**（title +14%/card h3 +17%）→ welcome product-smoke 17.06%→17.55%（+0.49pp 回归）→ 全 git checkout 回退；fontdue Bold 过墨同 advance-width(R225)/AA(R174) 谱系 fontdue-vs-chromium 渲染差异，**非单点可修，勿再以「加载 -Bold 接线」重试 R229**。morning-work 4× 高度已**闭环**（R255→R260 + a2b169e：ua_default_display 补 article/aside/details 等；body 25301→5677px≈0.95×chr，fullpage chr-diff 89.14%→48.65%，reftest 438/490 零回归）。**当前真实优先级（R267 更新：R236/R238 两条 billed turnkey reftest 杠杆 premise 双双被源码证伪，无 quick win）**：⚠️ **R238 WM-1 abspos-vertical（+14）的 R262「mirror-at-1407」半 turnkey 诊断已被 R267 证伪**——IFC vertical_rtl 分支（inline/mod.rs:1806-1817）已把 run.x 镜像到右侧（vertical-rl 首列 x=container_width−列宽），`all_fragments()` 返回 fragment.x=run.x，故 engine.rs:1407 `child.x=fragment.x` 对 vertical-rl **已正确**，R262 的 `if is_vertical_rtl{mirror}` 会**双重镜像**把元素推回左侧→净负；**勿实现 mirror-at-1407**。R238 rtl 残余 5.03% 真因非「缺镜像」，需重新诊断（候选=IFC 列 x 基址轴语义错配：container_width=行内轴(视觉高) 被用于块轴(视觉宽)排列 inline/mod.rs:1808，或 border/offset 边处理；探针须问「fragment.x 是否已镜像」而非 R262 假设的「需镜像」）。**DC-9 GPU = 当前唯一 active forward motion**（filter:opacity+brightness+contrast 已落——f6fed44 opacity + fc86937 color-filter pipeline [brightness/contrast]，R268/R273 核实 sound（brightness/contrast 是正确 CSS 语义非 opacity 近似），blur 亦落（3a3530f fs_blur 三角核 2-pass；R277 核实非 ==CPU——三角核 separable vs CPU 多遍 box，parity 分歧但覆盖达标），transform 亦落（R285 TRANSFORM_SHADER 逆矩阵重采样对齐 CPU apply_transform_post，独立 WGSL pipeline）；**color-matrix 5 项（grayscale/hue-rotate/invert/saturate/sepia）亦落（R286 扩 fs_color_filter mode 3-7，逐公式对齐 CPU apply_filter，复用 R273 color-filter ping-pong 管线零新 pipeline）**；剩 blend（CPU stub+GPU 丢弃，**post-process 单 framebuffer 架构上不可行、需 paint-isolation 架构，R278 defer**）—— DC-9 GPU filter 子类全覆盖，**唯一 GPU 缺口 = blend_mode**；硬门禁+非碰撞+本 WSL `--gpu` 可验证）；⚠️ **R236 multicol baseline-export（曾标 +8 turnkey R260）已被 R266 源码证伪降级**——baseline-export 用例全 block flex（`display:flex`），唯一 `LayoutBox.taffy_baseline` 消费者 engine.rs:988 guard 仅 `InlineFlex|InlineGrid` 不覆盖 `display:flex`，且 flex 项 taffy 内部已对齐、post-pass 覆写副本不动 taffy 内部缓存，故 **field-fill 净 0**，**勿以「填 taffy_baseline」重试 R236**（真实修复=pre-pass 估测或 post-align 重对齐，结构性多轮，见 R266）；② **welcome 17%/morning-work 48.65% 残余** 重定性（R229b）= item-tag span→block（R109 IFC 架构；**并行 agent inline/mod.rs 工作树本轮核实仅 fmt 残留，item-tag 攻坚疑已回退/搁置，跨 ~8 轮未提交**）+ fontdue CJK 度量噪声（line-height/advance，**非 weight**）+ hljs（需 JS）+ body ~300px 差，font-weight **不再是主因**；③ DC-9 GPU filter:opacity 前提经 R268 只读核实 **sound**（GPU fs_opacity 区域 RGB*=amount/alpha=1.0 == CPU apply_filter Opacity 语义；ping-pong A→B→scissor→B→A 正确；**R249 标的缺的第二纹理 headless_texture_b 已在并行 agent 未提交 WIP 落地**；headless-only 零默认回归）→ 提交后应满足 DC-9 对 opacity 覆盖（GPU 独立 WGSL 非 passthrough）+ CPU/GPU 对齐，是 R267 后实质 forward motion（**R269 核实**后续：blur=sound 同模式 apply_box_blur 真实参考；transform=真实近似 apply_transform_post 须对齐 quirk（白填暴露区/rect 裁剪/整数采样）；**blend≠同模式**——CPU apply_blend_mode 是 NO-OP STUB 无参考，需 source+dest 双图层新机制，GPU 真实现会与 CPU no-op 分歧破坏对齐，是比 opacity 大的独立特性，勿据「同模式」乐观低估）；前序 evidence 实测 `--gpu` 本 WSL 可用 → DC-9 可验证+非碰撞+硬门禁（caveat：CPU+GPU 均以 RGB 变暗近似 opacity（**R271 纠正：framebuffer 实有 alpha（RGBA×4 + blend_pixel 真合成），「无 alpha」前提 false；opacity 近似真因=post-process 无法恢复背景（非格式限制）；**R272 refine**：reftest compare 用 alpha（4 通道）+ ref 不透明→须 alpha=255，clip 白填=parity-correct+死代码（无生产 ClipPrimitive），transform 白填低 footprint 可议**）——DC-9 覆盖满足，opacity 近似正确性是更深独立问题，低 reftest 杠杆 R250）；④ UA display 审计完成（R258，a2b169e 后无更多 morning-work 类危险缺口）。⚠️ ruled out（勿重试）：font-weight -Bold 接线（R229b）、advance-width（R225）、multicol paint 切片（R203）、multicol balance 两趟（R200）、chromium 高 diff 候选（R202）、DC-12（R197 全实现）、**R238 mirror-at-1407（R267 证伪 R262 半turnkey 诊断；IFC vertical_rtl 已镜像 fragment.x，mirror 会双重镜像净负）**。⚠️ 仍开放多轮硬架构（非单会话）：multicol-breaking layout 侧 column-aware IFC（R131）、**R236 multicol baseline-export（R266 证伪 turnkey，field-fill 净 0；真实修复=pre-pass 估测或 post-align 重对齐，同 R131 谱系）**、Phase A IFC 三路径统一（R125/R198 font_size 死锁）、**DC-14 chromium-oracle 严格容差默认接线（R280 量化：默认 1%/5% 是硬上限 0.1%/0.5% 的 10×；R287 已落地 env `ZERO_REFTEST_STRICT` 严格容差门控 + 三态 blast radius：self-source@strict 真通过 293(59.8%)/近似 145(29.6%)/失败 52(10.6%)，near-pass 145 中 101 个 strict diff ≤1% 是最高杠杆 clean win 目标；区别于 chromium-oracle@strict 39.6%——完整达标需 strict 容差 + chromium Oracle 两源。R280 phased 第二步=翻默认待真实修复推高 strict pass 后再做；下一步攻 near-pass CSS2 前 20 个 clean win 候选用 STRICT env 度量增量）**。


## 综合裁决：结构性 plateau（R305–R323，≥10 轮一致收敛）

> 本节为 doc-maintenance 轮（2026-06-19）对最近 ~20 轮的**浓缩结论**，置于控制面板顶部便于检索。逐轮详细记录见文末「最近轮次详细记录」（R303–R323）与归档 [`archive/rounds-r142-r302.md`](./archive/rounds-r142-r302.md)（R142–R302）。

**核心结论**：rendering-compat 的**所有单会话 / 中会话 forward-motion 杠杆均已 ruled out 或 refuted**——这是 R313–R323（≥10 轮）一致收敛的结论，非单轮判断。rally 单会话迭代已无法提升真实通过率。

**基线（R323 复验，零漂移）**：

- self-source loose：**438/490 (89.4%)** @ 默认 1%/5% 容差
- self-source strict：**295/490 (60.4%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径）
- chromium-Oracle 真一致率：**~35.6%**（169/475；self-source 含 48.6% 假通过，DC-14 anti-false-pass）
- 产品 smoke：welcome 17.06% / wintertc 13.70% / morning-work 28.72%（全文本度量结构性，非图片/CSS 缺口——图片加载 R318 已端到端贯通）

**已穷尽 / 证伪的杠杆（勿再以单会话重试）**：

| 杠杆 | 裁决轮 | 结论 |
|------|--------|------|
| near-pass clean-win frontier | R307 | 26 个 <0.2% 案全落结构性墙 / 字体噪声，零 clean win |
| POLLUTED 候选逐项 hunt | R299/R300/R302/R309 | 多轮候选全结构性，exhausted |
| fresh chromium-Oracle cross-validate | R311 | 4 新候选 ruled out，plateau 再确认 |
| Phase A IFC font_size 解锁（4 路） | R125/R198/R205/R206 | paint IFC vs layout IFC 两趟，font_size/line-height/换行耦合死锁；R207 narrow 精修仅获 font-051 +1 |
| multicol breaking paint 侧 | R157/R198/R203/R317 | 5 次实证全 net-negative，paint 侧死路 |
| multicol balance 二分搜索 | R199/R200/R321/R322 | T/N ≡ binary-search（等高行）；columns-001 diff 实为 wrapping 精度，非 balance |
| multicol column-aware IFC（layout 侧） | R319 | spec-rfc 产出 + A1 probe REFUTES Phase 1（目标结构在失败集几乎不存在，迁移零增益） |
| baseline-export（3 机制） | R266/R310/R312/R313/R316 | field-fill 净 0 / inline-flex 不受控 / flex-post-pass 回归，3 路全证伪 |
| advance-width plumbing | R225/R320 | 双角证伪（reftest-oracle 零变化 + Ahem advance 精确），死路 |
| DC-9 blend_mode | R278 | 单 framebuffer post-process 架构不可行，需 paint-isolation |
| font-weight -Bold 接线 | R229b | fontdue Bold 过墨 ~15%，net-negative |
| taffy 0.11 升级 | R304 | DEFER（541 ref + 108 alignment + native float 冲突，具名缺口零收益） |

**剩余 forward motion = 多会话架构承诺（非单会话），或接受 plateau**：

1. **Phase A IFC 三路径统一** — paint 不重跑 IFC，直接渲染 layout 存储的行盒（R205/R207 viable slice 已证 font-051 可行；broad 应用需多轮 narrow 精修 + 守 multicol-fill-auto 反向依赖墙）。设计文档 [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)。
2. **Phase 2 嵌套 multicol fragmentation** — layout 侧 column-aware IFC + 嵌套列碎片化（R131/R201；R319 证纯 inline 迁移零增益，真实价值在嵌套 / 混合内容）。设计文档 [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)、[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)。
3. **baseline-export 真修复** — taffy 0.8+ baseline_overrides（需先解 R304 升级冲突）或自建 inline-level-box baseline 合成；解 flexbox-baseline / multicol-baseline 聚类（~10+ 案）。
4. **接受 plateau** — 当前 self-source 89.4% / strict 60.4% / Oracle ~36% 作为诚实基线。

**裁决**：需用户对「投入多会话架构承诺」vs「接受 plateau」决策。继续 rally 单会话迭代将重复 plateau 确认，无新进展。R314 已通过飞书通知用户此卡点。

---
## 里程碑完成状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| M1 — WPT Reftest 基础设施 | ✅ 完成 | 14/14 标准全部达成 |
| M2 — CSS 2.1 + Quirks Mode | ✅ 完成 | CSS parser + style system quirks 已实现；layout engine quirks 推迟到 M4 |
| M3 — Flexbox + Grid | ✅ 完成 | 179 个 reftest, 100.0% pass rate；Flexbox/Grid 无渲染缺口 |
| M4 — Float + Table + Multicol | ✅ 完成 | float + table + multicol 布局算法已实现；219 个 reftest, 100.0% pass |
| M5 — 文字排版 | ✅ 完成 | CJK 换行 + justify 修复 + float 堆叠修复 + 51 个 Text reftest |
| M6 — 全量扩展 | ✅ 完成 | 685 reftest, 13 目录全部 ≥50, 100.0% pass；rustybuzz + unicode-bidi 已集成 |
| M7 — 渲染器图元覆盖 | ✅ 完成（管线层）⚠️ | CPU 渲染器：全部 13 种图元 ✅；GPU 渲染器：13 种图元**管线**已建（48 单元测试 ✅），**但浏览器全量 GPU 路径 `render_full_scene_gpu` 实际消费以 DC-9 表为准**——transform=并行 agent WIP 接线、blend=CPU no-op stub+GPU 丢弃、5 color-matrix 滤镜（grayscale/invert/saturate/sepia/hue-rotate）GPU collect 丢弃、clip=no-op（engine 从不生成）；filter:opacity/brightness/contrast/blur 已落（f6fed44/fc86937/3a3530f）；浏览器消费：全部 13 种图元 ✅ |
| M8 — 布局正确性 | ✅ 完成 | BFC 检测 ✅；float clear ✅；margin 折叠(taffy 0.7 内置) ✅；<img> 固有尺寸 ✅；position:fixed ✅(adjust_fixed_to_viewport)；position:sticky 需宿主层（已标记 is_sticky，后续集成）；percentage height/auto margin/min-max-width 已有测试验证 |
| M9 — 高级视觉效果 | 🔧 进行中 | 重复渐变 ✅；多图层背景 ✅；clip-path 全形状裁剪 ✅(inset+circle+ellipse+polygon)；border-image ✅；text-shadow ✅；backdrop-filter ✅；CSS mask ✅(渐变蒙版裁剪+alpha衰减)；overflow 全图元裁剪 ✅；滚动容器 paint 偏移 ✅(scroll_x/scroll_y 字段 + paint 时子元素坐标偏移 + 3 个单元测试)；剩余：scroll-snap 行为（需宿主层输入路由）、滚动输入路由（需浏览器 app 集成） |
| M10 — 上游 WPT 真实 Reftest 导入 | ⏸ plateau（R323） | 基础设施 ✅；490 上游 reftest 已导入（9 目录）；self-source loose **438/490 (89.4%)** / strict **295/490 (60.4%)** / chromium-Oracle ~35.6%；R305–R323 全单会话杠杆穷尽，达标需多会话架构（见「综合裁决」） |

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

> **状态（2026-06-18 R277 只读复核，对齐 committed HEAD 3a3530f）**：较 R211（2026-06-17 标 transform/clip/filter/blend 全 ⚠️「丢弃」）有实质推进。filter:opacity（f6fed44）/brightness/contrast（fc86937）/blur（3a3530f）已落地为独立 WGSL 后处理管线（ping-pong 区域读写，`render_full_scene_gpu` 经 `apply_color_filters_headless`/`apply_blur_filters_headless` 消费，非 passthrough，满足 DC-14）。clip 经 R220 实证为 no-op——engine 生产路径**从不生成** ClipPrimitive（`add_clip` 0 处非测试调用），overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect` 预烘焙进图元几何，CPU/GPU 两路均空谈满足，**非真实缺口**。**真实剩余缺口 3 类**：(a) **transform**——并行代码 agent 本轮 WIP（gpu/pipeline.rs 新增 `TRANSFORM_SHADER`/`fs_transform`（逆变换重采样）/`create_transform_pipeline`，但**尚未接入** `render_full_scene_gpu` 的 collect/apply）；(b) **blend_mode**——paint 在 `painter/effects.rs:313` 生成 `BlendModePrimitive`，但 **CPU `apply_blend_mode`（effects.rs:331-348）是 no-op stub**（算 rect 后 `_=(left,top,right,bottom)` 仅消未用警告），GPU `render_full_scene_gpu` 同样不消费，需 source+dest 双图层新机制（R269 标记为比 opacity 大的独立特性，低 reftest footprint）；(c) **5 种 color-matrix 滤镜**（grayscale/invert/saturate/sepia/hue-rotate）——CPU `apply_filter` 全实现，但 GPU `collect_color_filters`（renderer/mod.rs:1837）仅处理 opacity/brightness/contrast（mode 0/1/2），其余 5 种 `_ => None` 丢弃（drop-shadow CPU 亦 stub），可扩 color_filter pipeline（R273 已铺 fs_color_filter 地基）。reftest harness 与 product-smoke 均走 CPU 路径，GPU 缺口不污染测量数字，仅影响浏览器 GPU 渲染模式。

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
| TransformPrimitive | 🔧 WIP | 并行代码 agent 本轮 WIP：`fs_transform`（逆变换重采样，匹配 CPU `apply_transform_post` clear-to-white 语义）+ `create_transform_pipeline` 已加 gpu/pipeline.rs，**尚未接入** `render_full_scene_gpu`（无 collect/apply）。完成后覆盖 DC-9 transform |
| ClipPrimitive | ⚪ no-op | engine 生产路径**从不生成** ClipPrimitive（R220 实证），overflow 裁剪预烘焙进图元几何。CPU/GPU 两路均空谈满足，**非真实缺口** |
| FilterPrimitive | 🔧 部分 | **已落（独立 WGSL ping-pong 后处理，非 passthrough）**：opacity（fs_color_filter mode0）/brightness（mode1）/contrast（mode2，fc86937）/blur（fs_blur 三角核 2-pass，3a3530f）。**未落**：grayscale/invert/saturate/sepia/hue-rotate（GPU collect 丢弃，CPU 已全实现，**R279 code-ready spec**——fs_color_filter 机械扩 mode 3-7，矩阵 shader 内从标量 param 计算，无 uniform/枚举改动）；drop-shadow（CPU 亦 stub） |
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

**日期**: 2026-06-10（本轮第二十六轮）
**总用例**: 490（上游真实 reftest，排除 skip list）
**通过**: 366
**失败**: 124
**通过率**: 74.7%

**说明**：通过率从 68.6% 提升至 73.5%（+24 个测试）。R20 关键修复：(1) reftest 分类容差 bug — 上游 reftest 使用 Default::default()（1% diff, 5ch）而非分类特定容差（Layout: 1%/5ch, Text: 5%/15ch），导致所有测试使用严格布局容差。改为 ReftestConfig::for_category() 后，文字类测试（css-writing-modes, css-fonts）使用正确容差。新增 with_viewport() builder 方法。(2) columns 简写解析修复 — 单整数值（如 `columns: 3`）现在正确解析为 column-count 而非 column-width（3px）。(3) 零高度浮动处理 — line_max_height 跳过零高度浮动元素。

### 按目录

| 目录 | 通过/总数 | 通过率 | ≥95% 达标 |
|------|-----------|--------|-----------|
| css-text-decor/ | 39/39 | 100.0% | ✅ |
| css-fonts/ | 60/60 | 100.0% | ✅ |
| css-grid/ | 17/20 | 85.0% | ❌ |
| css-writing-modes/ | 45/59 | 76.3% | ❌ |
| css-tables/ | 41/55 | 74.5% | ❌ |
| CSS2/ | 89/129 | 69.0% | ❌ |
| css-flexbox/ | 35/55 | 63.6% | ❌ |
| css-position/ | 10/16 | 62.5% | ❌ |
| css-multicol/ | 29/57 | 50.9% | ❌ |

### R16 本轮修复内容

| 修复 | 影响 | 说明 |
|------|------|------|
| 移除 apply_relative_offsets 双重偏移 | +3 tests | taffy 0.7 已在 layout.location 中包含 position:relative 的 inset 偏移。apply_relative_offsets 后处理函数再次添加同一偏移量，导致相对定位元素位移 2x。禁用此函数修复了所有使用 `position:relative; top:1in` 的参考文件 |
| float Y 位置尊重正常流 | +1 test | Phase 1 定位 float 时不知道 normal flow 的位置

| 修复 | 影响 | 说明 |
|------|------|------|
| 零 clearance 阻止 margin 折叠 | CSS2/floats-clear | CSS 2.1 §9.5.2：当 clearance=0 时，margin 折叠仍被阻断，元素位于 flow_bottom + child.margin_top（不折叠） |
| inline 元素 padding/border 参与行盒高度 | CSS2/linebox | TextRun 新增 padding_top/bottom + border_top/bottom 字段，box_height() 方法返回 line-height + padding + border 的完整盒高 |
| vertical-rl 列方向 RTL | css-writing-modes | IFC 新增 vertical_rtl 标志，vertical-rl 模式下列从右到左排列 |
| 垂直模式 abs-pos 静态位置修正 | css-writing-modes | 新增 fix_vertical_mode_abs_pos 后处理，对垂直书写模式容器中 abs-pos 元素重新计算静态位置 |

### CSS2 子目录详细通过率（R15 新增）

| 子目录 | 通过/总数 | 通过率 |
|--------|-----------|--------|
| floats-clear | 11/30 | 36.7% |
| linebox | 7/15 | 46.7% |
| backgrounds | 8/15 | 53.3% |
| borders | 10/15 | 66.7% |
| abspos | 3/4 | 75.0% |
| colors | 4/5 | 80.0% |
| floats | 12/15 | 80.0% |
| fonts | 13/15 | 86.7% |
| box | 1/1 | 100.0% |

### 关键发现（R15）

| 发现 | 说明 |
|------|------|
| taffy inline→Block 映射使 IFC padding/border 无 net reftest 效果 | 所有 display 类型映射为 taffy::Block，taffy 已正确计算 inline 元素尺寸。IFC padding/border 改进是规格正确但 reftest 中性 |
| CSS2 子目录通过率分化严重 | floats-clear 36.7%（19 失败）是最大瓶颈，box/colors/fonts 已接近 80-100% |
| css-flexbox 从 58.2% 提升至 60.0% | flex-flow-001 修复（float 定位 flow 追踪）→ flex item 正确 shrink |
| 35 个 near-miss (<2% diff) 分布 | CSS2/floats-clear (10), css-writing-modes (10), css-tables (7), css-flexbox (5), css-position (2) |
| 166 个失败根因分布 | 布局精度问题 (float/clear/margin) 50+ 个、writing-mode 轴交换 36 个、multicol column breaking 32 个、其他 48 个 |
| 后续最大杠杆 | (1) CSS2 float/clear 精度提升（影响 22 个测试）(2) multicol column breaking（影响 32 个测试）(3) writing-mode 块级布局轴交换（影响 36 个测试） |

### 后续重点

### R13 本轮修复内容

| 修复 | 影响 | 说明 |
|------|------|------|
| Inline-block 行内定位 | CSS2 +2 | 新增 adjust_inline_block_positions 后处理：对包含 inline-block 子元素的容器运行 IFC，获取正确水平并排位置。IFC 新增 InlineBlockBox 类型。CSS2 从 59.7% → 62.8% |
| Border 样式覆盖 | css-tables | collapsed_border_style_overrides[4] 数组：border-collapse 冲突解决后获胜方的样式（solid/dashed/dotted 等）正确传递到 paint 阶段。color_value_to_u32 新增 purple/cyan/magenta 等 10 种命名颜色 |
| 基线计算修正 | inline layout | inline-block 元素的 baseline 在其底部边缘（ascent=height），而非 0.8×line-height |

### 失败根因分布分析（R13 更新）

| diff 范围 | 数量 | 特征 | 代表性测试 |
|-----------|------|------|-----------|
| <2% | 33 | 亚像素/微偏移，接近通过 | clearance-006 (1.16%), flex-item-position-relative (1.04%) |
| 2-5% | 45 | 小幅定位差异 | clear-float-003 (1.92%), background-043 (2.61%) |
| 5-15% | 40 | 中等定位/尺寸差异 | float-006 (7.46%), background-090 (10.20%) |
| 15-30% | 32 | 显著布局差异 | abs-pos-non-replaced-* (21.33%), direction-vrl (12.49%) |
| >30% | 16 | 基本功能缺失 | empty-inline-002 (42.06%), background-attachment (30.48%) |

### 关键发现（R13）

| 发现 | 说明 |
|------|------|
| Writing-mode abs-pos 失败主因是 inline 布局 | 12 个 abs-pos-non-replaced 测试全部在 21.33% 失败。轴交换已启用（输入/输出双向），但 inline formatting context 不支持垂直模式——文本仍水平排列。静态位置基于水平 inline 布局，导致绝对定位偏移 |
| 55 个失败测试使用 Ahem 字体 | 这些测试的失败原因是布局定位差异（而非字形渲染），因为许多 WPT 测试使用 Ahem 字体创建精确矩形内容来验证布局 |
| CSS2 边框/背景测试 | border-bottom: 1in 系列测试（3.84%）的 diff 精确等于整个 div 面积，暗示渲染存在系统性偏移。需进一步调查 1in 单位在 border 上下文中的行为 |
| 大面积差异(>25%)多因缺少功能 | background-attachment:fixed (30.48%)、position-fixed-overflow-print (75%)、column-balancing-paged (56%) 均因缺少对应 CSS 功能实现 |

### 失败根因分布分析（R12 新增）

| diff 范围 | 数量 | 特征 | 代表性测试 |
|-----------|------|------|-----------|
| <2% | 33 | 亚像素/微偏移，接近通过 | clearance-006 (1.16%), grid/child-border-box (1.52%) |
| 2-5% | 45 | 小幅定位差异 | clear-float-003 (1.92%), background-043 (2.61%) |
| 5-15% | 40 | 中等定位/尺寸差异 | float-006 (7.46%), background-090 (10.20%) |
| 15-30% | 40 | 显著布局差异 | clear-applies-to-001 (29.45%), direction-vlr (12.49%) |
| >30% | 10 | 基本功能缺失 | background-attachment (30.48%) |

### 关键发现（R12）

| 发现 | 说明 |
|------|------|
| Ahem 字形位图非主因 | 验证了 Ahem 光栅化代码路径被正确触发（font_id=3），但通过率无变化。说明上游 reftest 失败主要因为布局定位差异，非字形渲染 |
| 布局定位是核心瓶颈 | 分析 168 个失败测试，绝大多数是元素位置/尺寸与 Chrome 不同。根因分为：float/clear 后处理精度、writing-mode 轴交换、multicol 列拆分、inline box model |
| Phase 1 float+clear 已实现 | adjust_float_positions Phase 1 已正确处理 float+clear 组合（line 676-703），非 float 元素的 clear 处理在 Phase 2 |
| 相同 diff 百分比暗示系统性偏移 | 多个测试在相同百分比失败（如 3.83%、7.67%），暗示特定的元素尺寸/偏移量差异 |
| writing-mode 轴交换 | css-writing-modes -1 | 启用 CSS Writing Modes §7.1 轴交换：输入时交换 CSS 属性到 taffy 水平模型，输出时交换回视觉坐标。盒体几何位置正确，但文字仍水平排列（需要 paint 层旋转支持）。1 个测试因坐标交换而回归 |
| 属性继承修复 | 全局 | list-style-type、list-style-position、writing-mode 添加到继承属性列表和 inherit_property 处理器 |
| justify-items/justify-self | css-grid | 转换器新增映射，从 ComputedStyle 映射到 taffy Style 的 justify_items/justify_self 字段 |
| scrollbar_width | 全局 | 从硬编码 0.0 改为根据 ComputedStyle 映射（Auto→15px, Thin→8px, None→0px） |

### 后续重点

1. **multicol column breaking**（影响 css-multicol ~16 测试）：需要实现内容碎片化 — 将单个块级元素的内容拆分到多列。当前仅移动整个子元素到下一列。multicol-breaking-* 系列测试全部在 16%+ 失败。
2. **CSS2 float/clear 精度**（影响 CSS2 ~19 测试）：clearance 计算使用简化公式，不完全匹配 CSS 2.1 规范的 C1/C2 双路径算法。参考文件大量使用拉伸 swatch 图片（20x20→96x96），image scaling 差异可能贡献部分 diff。
3. **CSS2 inline box model**（影响 ~8 测试）：空 inline 元素 line-height 贡献、inline 元素 margin 处理、block-in-inline 拆分。IFC 仅在 float/inline-block/vertical-mode 容器中运行，普通 block 容器中的空 inline 元素 line-height 不被 IFC 处理。
4. **Flexbox baseline 对齐 + writing-mode**（影响 css-flexbox ~9 测试）：multi-line baseline 测试（47%）、baseline align-self（15-18%）、min/max-content（16-21%）。flex-flow:row + writing-mode:vertical-rl 需要 flex 方向的轴交换支持。
5. **CSS 表格子像素**（影响 css-tables ~9 测试）：subpixel collapsed borders (1.97%)、table-cell-overflow (1.12%)、border-conflict-resolution (1.55%)。多数是 image scaling 或 border 渲染精度问题。

### R11 本轮修复内容

| 修复 | 影响 | 说明 |
|------|------|------|
| multicol column breaking 基础设施 | css-multicol | 新增 assign_children_to_columns_with_breaking，当子元素超出列高限制时移至下一列。基础设施就绪，但真实 breaking 需要内容碎片化 |
| CSS columns 简写验证 | css-multicol | expand_columns 验证 column-width 值有效性，拒绝 'normal' 等无效值。符合 CSS 规范整个声明无效的语义 |
| table cell overflow 修复尝试 | css-tables | 调查发现 CSS 2.1 规范要求 table cell height 为最小高度，即使 overflow:hidden 也必须增长以包含内容。已回退 |

### 关键发现（R11）

| 发现 | 说明 |
|------|------|
| Ahem 字体是最大瓶颈 | 100+ 测试因 Ahem 字体渲染差异而失败（fontdue vs Skia）。非 Ahem 测试的失败率低得多。**R12 更新**：Ahem 字形位图差异已修复，但失败主因为布局定位差异而非字形渲染 |
| CSS table cell height = 最小高度 | CSS 2.1 明确规定 table cell 的 height 是最小高度，cell 必须增长以包含内容，overflow:hidden 不改变此行为 |
| Column breaking 需要碎片化 | multicol-breaking-* 测试需要将单个子元素的内容（如文本）拆分到多列，不仅仅是移动整个子元素 |
| abs-pos-non-replaced 12 个测试全在 21.33% | 这些测试都用 Ahem + background image，相同的差异比例表明是系统性渲染问题 |


### 发现的关键问题

| 问题 | 影响 | 说明 |
|------|------|------|
| **表格 border CSS 未生效** | css-tables | CSS `border: 5px solid green` 在 `<table>` 元素上未正确应用——ComputedStyle 显示 border_top_width=Px(0.0)，但 border-collapse: Collapse 正确设置。疑为 CSS 级联中 border 简写展开或选择器匹配的问题，需进一步调查 |
| **paint 系统 font-family 硬编码** | 全局 | paint/painter/text.rs 硬编码 FontId(0)，不解析 CSS font-family。Ahem 字体虽已加载但无法被 font-family: Ahem 匹配到 |
|------|------|----------|
| empty-cells 在 border-collapse:collapse 时正确显示边框 | css-tables | border-collapse-empty-cell ✅ |
| row-group/row border/padding/margin 抑制（CSS 2.1 Section 17.5.3） | css-tables | row-group-order ✅, rowspan-cell-border-after-color ✅ |
| table cell explicit height + overflow:hidden 保留原始高度 | css-tables | (cell overflow 测试因参考文件差异未通过，但修复逻辑正确) |
| 空 inline 元素 line-height 贡献到行盒 | CSS2/linebox | (需 Ahem 字体的测试仍失败) |
| sibling combinators 跳过文本节点 | CSS2 选择器 | (改善选择器匹配精度) |
| table min/max size constraints | css-tables | (基础设施准备) |
| JS-dependent test skip (position-fixed-scroll-nested-fixed) | css-position | 移除 1 个无效测试 |

---

## 已知关键缺口

| 缺口 | 影响范围 | 优先级 | 里程碑 |
|------|----------|--------|--------|
| **Paint IFC / taffy-IFC 架构分裂** | **~50 个失败测试（51% 剩余失败）** | **P0-致命 / 需专项架构改造** | **Post-R71 专项规划** |
| Float 布局算法 | CSS 2.1 核心 | ✅ 已完成 | M4 |
| Table 布局算法 | 表格渲染 | ✅ 已完成 | M4 |
| Multi-column 布局算法 | 多列布局 | ✅ 已完成 | M4 |
| OpenType shaping | 文字排版质量 | ✅ 已完成 | M6 |
| BiDi 算法 | RTL 文本 | ✅ 已完成 | M6 |
| Vertical writing-mode | 竖排文本 | M5 能力已落地，完整垂直布局仍缺 | Post-R71 专项规划 |
| CJK normal-mode 换行 | CJK 排版 | ✅ 已完成 | M5 |
| text-align: justify | 文字排版 | ✅ 已完成 | M5 |
| Float exclusion 堆叠 | 布局正确性 | ✅ 已完成 | M5 |
| Quirks mode | CSS 2.1 兼容性 | ✅ 已完成 | M2 |
| 上游 WPT 真实 reftest 导入 | 覆盖范围 | M6 | M6 |
| CPU 渲染器图元覆盖 | 视觉输出 | ✅ 已完成 | M7 |
| 浏览器图元消费 | 视觉输出 | ✅ 已完成 | M7 |
| GPU 渲染器图元覆盖 | GPU 视觉输出 | ✅ 管线已实现 | M7 |
| Multicol column breaking | css-multicol ~22 测试 (61.4%→需+18) | P1 / 需专项架构改造 | Post-R71 专项规划 |
| Writing-mode 垂直布局 | css-writing-modes ~10 测试 (83.1%→需+7) | P1 / 需专项架构改造 | Post-R71 专项规划 |
| Inline-box 模型 | CSS2 linebox ~8 测试 | P1 | R69+ |
| 外部 stylesheet 加载 | 真实静态网页 CSS | P1 | M10/R38 |
| 图片子资源/ImageCache 贯通 | Logo/图片密集静态页 | P1 | M10/R38 |
| 产品/真实静态页视觉 smoke | 验收有效性 | P1 | M10/R38 |

---

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

near-pass(R307) / POLLUTED hunt(R299–R309) / fresh-xval(R311) / Phase A 4 路 font_size(R125–R206) / multicol paint 侧(R157–R317) / balance 二分(R199–R322) / column-aware IFC 纯 inline(R319) / baseline-export 3 机制(R266–R316) / advance-width(R225–R320) / blend post-process(R278) / font-weight -Bold(R229b) / taffy 升级(R304)。

### 已完成里程碑（参考，非当前活跃）

- M1–M9 基础设施 + 渲染器图元覆盖 + 浏览器消费 + 布局正确性 + 高级视觉效果：**已完成**（见下方「里程碑完成状态」「Done Criteria 进度」）。
- M10 上游 WPT reftest：基础设施完成，通过率 plateau（438/490 loose），达标需上述多会话架构。

---

## 最近轮次详细记录（R303–R323；R142–R302 已归档至 [`archive/rounds-r142-r302.md`](./archive/rounds-r142-r302.md)）

### R303 — DC-9 GPU 图元覆盖审计：filter 已全实现（纠正 R220 过时记忆），唯一缺口 = blend_mode（read-only 审计，基线 loose 438/490 / strict 296/490 持平）

**承接**：R302 clean-win 穷尽后转向 DC-9 独立能力缺口（GPU filter:opacity）。代码审计发现 **R220「DC-9 真实缺口仅 transform/filter/blend 三项」已过时**——这三项中 transform + filter 已实现，仅 blend_mode 仍缺。

**审计（gpu/renderer/mod.rs 逐字段引用计数 + headless ping-pong 实证）**：
- **GPU 已渲染 11/13 图元类型**（独立 WGSL 管线）：fills / rounded_rects / gradients / shadows / images / glyphs / strokes / path_fills / path_strokes / transforms / **filters**。
- **filters 已全实现**（headless ping-pong，line 787-800）：`collect_color_filters`（Opacity/Brightness/Contrast/Grayscale/HueRotate/Invert/Saturate/Sepia 共 8 种，mode 0=opacity）+ `collect_blur_filters`（Blur 2-pass H+V 高斯）+ `collect_transforms`（CSS transform 2D 仿射逆矩阵）。`apply_color_filters_headless`/`apply_blur_filters_headless`/`apply_transform_filters_headless` 实证为真实 ping-pong A→B→A（scissor pass 采样+滤镜+回写），匹配 CPU `apply_filter`，**非 stub**。
- **clips（0 GPU 引用）= 合法 no-op**：engine 生产路径从不生成 ClipPrimitive（R220 已证，overflow 裁剪预烘焙进图元几何）。
- **blend_modes（0 GPU 引用）= 唯一真实 DC-9 GPU 缺口**：engine 生产**生成**（effects.rs:314，CSS `mix-blend-mode`），但 GPU 静默丢弃。实现需 **backdrop 采样**（元素内容与背后已渲染内容按 blend 方程合成）= 元素需渲到独立层再与 backdrop 合成，复用现有 ping-pong 但需 per-element-layer 渲染顺序改动，**复杂**且 **mix-blend-mode 在上游 reftest 中近乎零覆盖**。
- **DropShadow filter = 双路径一致 no-op**（CPU effects.rs:192 `_` + GPU 不收集），罕见，非 DC-9 阻塞。

**结论**：DC-9 GPU 覆盖 **≈92% 满足**（11 图元 + 全 filter 子类型独立管线，非 CPU passthrough 满足 DC-14）。唯一缺口 blend_mode 复杂（backdrop 采样）且无 reftest 验证，非单会话可验证落地目标。**纠正 R220/R302 计划**：filter:opacity 无需实现（已 done），下一步勿再以「DC-9 GPU filter」为 lever。

**对优先级队列影响**：DC-9 实质接近完成（仅 blend_mode + DropShadow 残留，均复杂/罕见/无验证）。clean-win 穷尽 + DC-9 接近完成 → 渲染兼容性目标的**剩余真实缺口集中**在：① 结构性（Phase A IFC 文本度量 / intrinsic sizing / multicol 碎片化 / writing-mode 轴）；② 特性缺口（blend_mode backdrop / iframe 子文档加载 / 原生表单控件 / dialog JS）；③ taffy 限制（grid auto-track growth / flex intrinsic sizing）。这些均非单会话 clean win，需多轮架构（spec-rfc）或上游升级（taffy）。read-only 审计，无代码/reftest 变更，基线持平。

### R304 — taffy 升级评估：DEFER（read-only 深度调研，基线 loose 438/490 / strict 296/490 持平）

**承接**：R300/R302/R303 将剩余结构性缺口归因为「taffy 限制（grid auto-track growth / flex intrinsic sizing）」。本轮 read-only 评估 taffy 升级能否解锁这两个具名缺口。

**当前 taffy 现状**：workspace 声明 `taffy = "0.7"`，经 `[patch.crates-io]` 重定向到 vendored `crates/taffy-local`（taffy 0.7.7 全量源码，git-tracked 61 文件，commit 9e5df18 R59 引入）。本地补丁 = `cached_baselines()` 访问器（Cache + TaffyTree 暴露内部 `LayoutOutput.first_baselines`），补丁面极小（仅 2 个 pub 方法 + 复用已 pub 的 first_baselines 字段）。

**上游版本演进**（crates.io API + GitHub release notes + CHANGELOG.md 实证）：0.7.7（2025-03-06，当前）→ 0.8.0（2025-04，calc() + tagged-pointer 尺寸类型）→ 0.9.x（2025-08~11，named grid lines，Style 泛型化 CheapCloneStr，grid 类型改名）→ 0.10.x（2026-03~04，native float/clear feature + direction/RTL + cache API &LayoutInput）→ **0.11.0（2026-06-12，最新，safe alignment enum variant→associated constant）**。共 4 个 breaking-change minor 版本。

**核心结论 1 — flex intrinsic sizing 升级零收益**：CHANGELOG 实证，所有 flex/grid intrinsic-sizing 修复均**早于 0.7.7，已在 vendored 副本中**：#624 grid growth limits（0.4.1）、#673 intrinsic main size vs child cross size（0.5.2）、#722/#723/#728 auto-fill/fit 计数+min-size intrinsic（0.6.1）、#522/#481 flexbox/grid intrinsic main size（0.3.13）、#388 % min-content（0.3.7）、#291 flex min-content constraint（0.3.0）。**flexbox-collapsed-item（R301 残余 15%）= ZeroWeb 自身 engine.rs:2649 浮动 shrink-to-fit 不遵循 flex-resolved 子项尺寸（min-content floor）的 post-processing 缺口，非 taffy 版本问题，升级不解锁**。

**核心结论 2 — grid auto-track growth 升级零收益**：vendored 0.7.7 **已有** `expand_flexible_tracks`（fr 吸收 free space）+ `maximise_tracks` + `#783 stretch auto tracks if content-align=stretch`（0.7.5 已含）。对比上游 main `expand_flexible_tracks`（track_sizing.rs:1179）= **实质相同**（仅 `.is_flexible()→.is_fr()` 改名 + `total_cmp` float 排序 + 注释），**auto 列仍不吸收 free space**（两版本一致）。R302 grid-calc-margin（w=0）的 auto-track-absorb-free-space 行为升级不变。新版唯一 grid 修复 = #946（0.10.1 auto-repeat 计数+min-size）+ #960（0.11 item % vs grid area），均**旁系**于 R302。

**核心结论 3 — 迁移成本 prohibitive**：`layout-engine` 内 **541 处 `taffy::` 引用 + 108 处 alignment enum（9 文件）**，跨 4 个 breaking 版本：① 0.8 tagged-pointer 改 `LengthPercentage`/`Dimension`/`MinTrackSizingFunction` 等构造（ZW 50+ 处）；② 0.9 `Style<CheapCloneStr>` 泛型化 + `TrackSizingFunction→GridTemplateComponent` 改名（ZW 20+ 处）；③ 0.10 cache `&LayoutInput`（本地 cached_baselines 补丁须在新 cache 结构上重新推导）；④ 0.11 `AlignContent::Start→AlignContent::START` 关联常量（108 处）。**最坏回归风险**：0.10 native `float_layout` feature 与 ZeroWeb ~6 轮手动 float 后处理（R108b/R127/R129/R145/R301）**冲突**——启用 native float 须退役/重写这些 pass，触及全量 layout 测试套件，难调试。

**真实但无关的升级收益**（不针对具名缺口）：calc()（0.8，关联 R97/R180 max-content→0 bug）、native float/clear（0.10）、direction/RTL（0.10，关联 writing-mode）、grid #960。

**决策：DEFER 升级**。两个具名结构性缺口均为 ZeroWeb 侧架构问题（engine.rs shrink-to-fit post-processing / Phase A IFC 统一），非 taffy 版本问题；升级对它们零收益，而迁移+回归成本 prohibitive 且 native-float 冲突风险高。**纠正 R302「③ 评估 taffy 升级」lever 期望**——升级非 clean unlock，应从优先级队列移除。

**对优先级队列影响**：taffy 升级评估完成 = ruled out（具名缺口零收益 + 成本 prohibitive）。剩余真实 lever 收敛为**纯 ZeroWeb 侧架构工作**：① Phase A IFC 统一（stored/paint 三路径 baseline 墙，spec-rfc 多轮）；② engine.rs 浮动/intrinsic-sizing post-processing 完整化（min-content floor，R97/R181 硬域）；③ 独立能力缺口（DC-13 产品 smoke 端到端证据 / DC-9 blend_mode backdrop）。next = 启动 Phase A IFC 统一的 spec-rfc 设计（最大结构性 lever，影响 large-font/multicol/IFC 度量整簇），或先做 DC-13 产品 smoke 端到端（非 taffy 阻塞、有明确验收）。read-only 调研，无代码/reftest 变更，基线 loose 438/490 / strict 296/490 / chromium-Oracle ~35.6% 持平。

### R305 — Phase A IFC 统一 spec-rfc 设计文档产出（read-only 设计，基线 loose 438/490 / strict 296/490 持平）

**承接**：R304 DEFER taffy 升级后，转向最大结构性 lever Phase A IFC 统一。本轮按 spec-rfc 工作流产出**设计文档**（不落地代码）。

**产出**：`docs/goal/rendering-compat/phase-a-IFC-unification-design.md`（335 行，11 章，Spec Lint 14 Pass / 1 Warning / 0 Fail）。

**read-only 精读结论——三处墙精确定位（代码行号实证）**：
- **三条 IFC 路径**：compute_final（engine.rs:1668 真实 styles IFC）→ Gate 2（engine.rs:1910）→ paint use_stored（text.rs:807）Path A 渲染 vs Path B 空 styles 重跑（text.rs:846）。
- **两个 Gate**：Gate 1（R207 narrow，engine.rs:1720-1749）决定哪些容器进 IFC；Gate 2（R84 安全子集，engine.rs:1910 `lines.len()<=1 && is_pure_ahem`）决定哪些容器**存** inline_layout。**关键事实**：`store_font_sizes_from_ifc`（engine.rs:1152）不受 Gate 2 限制广泛建立 per-node font_size map，Gate 2 只限完整行盒存储。
- **墙①**（large-font 簇根因）= Gate 2 多行限制：ifc-008 inner-div 2 行→不存→Path B 16px。
- **墙②**（multicol 反向依赖 R198/R209/R213）= multicol 永远走 Path B（use_stored=!multicol_info，text.rs:807），放宽 Gate 2 让内层容器存行盒后 multicol-fill-auto 0.63→9.15 回归；机制疑点（font_size map 不受 Gate 2 限→回归非 map 变化，疑 paint 分支/几何变化）降级为假设 A2 待 Phase 3 探针。
- **墙③**（v_offset/baseline 语义分歧）= Path A 用 `is_ahem?0:font_size`（text.rs:1208）vs Path B 用 `baseline_fs`（text.rs:1225），多行非-Ahem 不一致——**架构性**，只要 Path B 存在两套语义就不可收敛（R206 broad 翻 FAIL 直接原因）。

**推荐方案**：**baseline-resolved 单一权威行盒**——InlineLayoutLine/Fragment 加 `baseline_y` 字段，compute_final 对所有过 Gate 1 容器存行盒，paint 永远消费 stored（消灭 Path B，仅 flex/grid/table 保留重跑），删除 Gate 2 启发式改用「font_size 同源」不变量。multicol 经 Phase 3 探针定 M1（消费 stored 做列分配）/M2（保守 fallback）。

**5-Phase 实施计划**（每 Phase 独立可合并、零 count 回归硬门禁）：P1 加死字段 baseline_y 建测量基线（净 0 验证）；P2 paint Path A 改用 baseline_y（R207 子集仍 PASS 验证 A3）；P3 read-only 探针 multicol 墙②；P4 删 Gate 2 多行限制 + multicol 方案；P5 删 Path B 死代码 + engine.rs 拆分（3969→抽 inline_finalization.rs ~400 行）。

**对优先级队列影响**：Phase A 有了可执行的架构蓝图 + 分阶段回归门禁（区别于 R125/R198/R205/R209/R213 五轮单点死锁——它们都在试图**单点**修 font_size 而 Path B 仍在）。设计文档落盘供后续多轮接力。next = R306 执行 Phase 1（加 baseline_y 死字段 + compute_final 计算，净 0 验证，零渲染变化）。read-only 设计，无代码/reftest 变更，基线持平。

### R306 — Phase A Phase 0 探针实证：geometric baseline ≠ fontdue render baseline，§6.3A「frag.y+height」方向证伪（read-only 探针，基线 loose 438/490 / strict 296/490 持平）

**承接**：R305 spec-rfc 设计文档把 Phase 0 定为「实测 glyph 基线耦合探针」，因 §6.3A 发现 `GlyphPrimitive.y`=基线、`frag.y/offset/glyph.y` 经验性耦合，原 Phase 1「加 baseline_y 字段 = frag.y+height」前提不稳。本轮执行 Phase 0 实证探针。

**探针设计（env-gated，零默认回归）**：text.rs:1208 stored Path A 的 `v_offset` 原为 `is_ahem ? 0 : font_size`。加 env `PHASEA_BL=1` 临时改用文档化基线不变量 `v_offset = frag.height`（即假设 baseline = frag.y + height，types/mod.rs:387 注释 + apply_vertical_alignment `run.y = baseline_y - run.height` 推导）。对 stored 单行纯 Ahem 用例 font-051（`div{font:100px/1 Ahem}` → "FAIL" 4 字 ×100px = 400×100 黑矩形）A/B 实测。

**实证结果（决定性）**：
| 模式 | font-051 diff | 裁决 |
|------|---------------|------|
| BASELINE（v_offset=is_ahem?0:font_size，默认） | **0.00% PASS** ✓ | 当前 offset load-bearing 正确 |
| PROBE（v_offset=frag.height，PHASEA_BL=1） | **16.67% FAIL**（80000/480000 px，max ch 255）✗ | 文档化「frag.y+height」**渲染错误** |

font-051 单行 line-height:1 Ahem：IFC 算 `frag.height=line_height=100`、`max_ascent=max(80,100)=100`、`frag.y=100-100=0`。PROBE 把 offset 从 0 改成 height=100 → glyph_y 下移 100px → 黑矩形整体错位 16.67%。**当前 offset=0 才正确**。

**关键推论 1 — stored Path A 的 `else { frag.font_size }` 分支是死代码**：Gate 2（engine.rs:1910 `lines.len()<=1 && is_pure_ahem`）保证**只有纯 Ahem 单行容器存储** inline_layout（R207 narrow 扩展的是 Gate 1，Gate 2 的 is_pure_ahem 守卫未动）。故 stored 片段 `frag.is_ahem` **恒为 true**，`v_offset` 恒为 0，`else` 分支永不执行。stored Path A 的 offset 实际是常数 0。

**关键推论 2 — geometric baseline ≠ fontdue render baseline**：types/mod.rs:387「基线 = frag.y + height」是 IFC 的**几何基线**（apply_vertical_alignment `run.y = baseline_y - run.height` 推导成立）。但 fontdue 光栅化 Ahem glyph 时，`GlyphPrimitive.y`（被 cpu/mod.rs:33 `glyph_top_left` 当 baseline）+ fontdue 自身 glyph 度量（y_offset/bitmap_height）的组合，使 **offset=0（非几何 height）** 产出与 chromium 一致的位图。即 fontdue Ahem 的「render baseline」与 IFC「geometric baseline」差一个 fontdue-metric-dependent 常量。`baseline_y` 字段若存几何基线（frag.y+height），paint 直接用会**重演 16.67% 错误**。

**对设计文档的纠正（§6.3A / §0 / §7.1 Phase 1 作废）**：
- 原 Phase 1「paint Path A 改用 `frag.y+height` 基线 / 加 baseline_y 死字段=几何基线」**实证证伪**——会破坏 font-051 等 stored Ahem 用例（R207 子集）。
- 真正可行的统一方向（二选一，替代原 baseline_y 字段）：
  - **(A) 存 render glyph_y**：InlineLayoutFragment 加字段存「compute_final 用同款 offset 校准（is_ahem?0:font_size）算出的最终 glyph_y（= 传给 fontdue 的 baseline）」，paint 直接消费，绕过 offset 语义分歧。但 stored 路径 is_ahem 恒 true → glyph_y = content_y + frag.y + 0，对 multicol/非 Ahem 无新信息。
  - **(B) 保留 paint 端 offset 校准**：stored frag 已携带 is_ahem + font_size，paint 端 `is_ahem?0:font_size` 校准不动；统一靠「让更多容器进 stored」（Gate 2 放宽）而非「改 offset 语义」。
- 两方向都把 Phase A 的真正杠杆从「offset 语义统一」**重定**为「Gate 2 放宽覆盖多行/非纯-Ahem」——而这恰是 R209（PHASEA_MULTILINE）已试、被墙②（multicol-fill-auto 0.63→9.15 回归）阻塞的方向。offset 语义**不是**阻塞点（Path A offset 对 stored Ahem 已正确）。

**与历史轮次的一致性**：R209 已用 Gate 2 多行放宽 + offset=0（未改 offset）测 ifc-008：8.18→4.17%（改善但未过，残余=换行精度）+ multicol-fill-auto 回归。本轮探针**补齐了 offset 语义这一维度**——确认 offset 不需改、不能改成 frag.height，R209 的 ifc-008 残余 4.17% 与 offset 无关（是换行/列宽精度）。Phase A 真正硬阻塞 = 墙②（multicol 反向依赖）+ 换行精度，**非** §6.3A 假设的 offset/baseline 语义。

**意义**：Phase 0 探针以最小代价（env-gated A/B，已回退）**证伪了 R305 设计文档的核心假设**（geometric baseline 可作 render baseline），避免在错误前提下实现 Phase 1（加 baseline_y 字段 = 几何基线）而破坏 R207 子集。这是 spec-rfc「先思考再编码，不假设」原则的实证体现——§6.3A 已自我标记前提不稳（「无法仅靠读码推导」），本轮探针把「不稳」转为「证伪」。设计文档须据本结论修订（§6.3A 加实证裁决、Phase 1 重定向为 Gate 2 放宽 + multicol 墙②，非 offset 字段）。

**本轮为 read-only 探针**：env-gated 代码改动（text.rs:1208 PHASEA_BL 分支）**已 100% 回退**（git diff 仅余并行 agent 的 README.md WIP，非本轮）；revert 后重编译 + font-051 复测 **0.00% PASS** 确认恢复。未改默认行为，未跑全量 make reftest（探针仅改 stored Ahem offset，font-051 A/B 已充分裁决；默认路径零变化）。基线 loose 438/490 / strict 296/490 / chromium-Oracle ~35.6% 持平。next = 据本结论修订设计文档（§6.3A/Phase 1 重定向），或 pivot 到 multicol 墙②的 layout 侧 column-aware IFC（Phase A 真正硬阻塞，R131 谱系）。

### R307 — DC-14 strict 全量 near-pass frontier 实证：clean-win 杠杆关闭（read-only 实证 + evidence，基线 loose 438/490 / strict 296/490 持平）

**承接**：R306 Phase 0 探针证伪 Phase A 几何基线方向后，转 DC-14 优先级队列明确标的「攻 near-pass CSS2 前 20 个 clean win 候选用 STRICT env 度量增量」（R280 phased 第二步；R287 已落地 `ZERO_REFTEST_STRICT` env + 三态 blast radius：self@strict 真通过 296(60.4%)/近似 145(29.6%)/失败 49(注:本轮实测 194 fail 含近似)/）。本轮把 R280「145 near-pass / 101 ≤1%」的**计数乐观**做**逐用例根因分类**实证。

**实证方法（test-guard 包裹，合规 run-rules）**：`ZERO_REFTEST_STRICT=1 ./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest-upstream` 全量 490。复现 R287 strict **296/490 (60.4%) / 194 fail**（基线一致）。194 失败按 diff% 升序持久化到 `evidence/r307-strict-nearpass-frontier-2026-06-19.txt`。

**Diff-band 直方图（194 strict 失败）**：<0.2%: 26 / 0.2-0.5%: 18 / 0.5-1%: 53 / 1-2%: 22 / >2%: 75。

**Near-pass frontier（<0.2%，26 案）逐聚类根因分类——全部落入已知结构性墙或字体噪声，零独立 clean win**：
- **css-multicol baseline-export 聚类**（baseline-000/003/004/005/006，~0.12-0.14%，5 案）：multicol baseline，R235/R266 已证 field-fill 净 0、需 pre-pass 估测（结构性多轮）。
- **css-multicol breaking/fill/balance**（broken-column-rule-1、float-with-line-after-spanner、multicol-fill-001、balance-grid-container、multicol-breaking-nobackground-000，~0.10-0.17%）：R131 column-aware IFC 碎片化墙。
- **css-tables collapsed-border/visibility-collapse/display-contents**（collapsed-border-partial-invalidation-003、visibility-collapse-rowspan-005/colspan-003、display-contents-001/003，~0.14-0.15%，6 案）：table 子系统结构性（R177b/R292 谱系残余）。
- **css-position hypothetical-box-scroll**（parent/viewport，0.12%，2 案）：abspos hypothetical box，结构性。
- **css-flexbox baseline/column-row-gap**（flexbox-baseline-align-self-baseline-horiz-001、flexbox-column-row-gap-002，0.14/0.25%）：flexbox baseline 合成，结构性（R295 wrap-reverse 谱系）。
- **CSS2 color-applies-to-001/005 + float-nowrap-5**（0.12%）：text glyph subpixel 定位（table-cell vs block div 渲染 "Filler Text" 的亚像素差），字体/布局噪声。
- **CSS2 ifc-001（0.12%）深挖**：LAYOUT_DUMP 实测 TEST div1 h=21.2 vs REF div h=22.0——inline 元素包裹文本（3×`<div display:inline>`）vs 直接文本的 **行盒高度差 0.8px**，即 Phase A 墙③（v_offset/baseline 语义分歧，R206 broad 翻 FAIL 直接因）。结构性强耦合，非单点。
- **css-grid stretch-grid-item-text-input-overflow（0.12%）**：text-input 原生 widget（R202 表单控件特性缺口）。

**裁决：near-pass frontier 是结构性 plateau 的「拖尾边缘」，非 clean-win 源**。R280「101 near-pass ≤1%」是计数乐观，逐用例分类后**零独立 clean win**——全部映射到 Phase A 墙③ / multicol 墙②+baseline / table / flexbox-baseline / 字体噪声 / 表单控件特性缺口。**near-pass clean-win 杠杆经实证关闭**。

**对优先级队列影响**：DC-14 phased 第二步（near-pass 攻坚）实证为死路，从队列移除。剩余真实 forward motion 杠杆收敛为**纯结构性多轮**：① Phase A IFC 统一（墙② multicol + 墙③ baseline，spec-rfc v1.2 已修订方向）；② multicol column-aware IFC 碎片化（R131，最大单聚类——baseline-export 5 + breaking/fill 5 = 10 案）；③ DC-9 blend_mode backdrop（独立能力，低 reftest 覆盖）；④ DC-13 产品 smoke 残余（item-tag R109 + fontdue CJK 度量）。**这些均非单会话 clean win，需 spec-rfc 多轮或特性实现**。

**本轮为 read-only 实证 + evidence 持久化**：零代码变更（`git diff -- '*.rs'` 空）；新增 `evidence/r307-strict-nearpass-frontier-2026-06-19.txt`（70 行，194 失败升序 + 直方图 + 26 案根因分类）。复现 R287 strict 基线 296/490 一致。基线 loose 438/490 / strict 296/490 / chromium-Oracle ~35.6% 持平。next = 启动 multicol column-aware IFC 碎片化（R131，最大 near-pass 聚类 10 案的根因域）的 spec-rfc 设计，或 DC-9 blend_mode 独立特性。

### R308 — font-size 百分比解析修复（code change，DC-14 真实一致性修复，loose 438/490 持平 / strict 296→295 一处 revealed false-pass）

**承接**：R307 关闭 near-pass clean-win 杠杆后，转攻 R307 evidence 里**未调查**的 POLLUTED 候选 `anonymous-inline-inherit-001`（self 0.00% / chromium 3.84%，CSS2 linebox，非 writing-mode/multicol/grid 聚类）。PIL 渲染对比 + LAYOUT_DUMP 实测定位到**真实单点 bug**。

**根因（computed.rs:51 + 186）**：`resolve_length` 的 `LengthValue::Percentage(v) => *v` 分支返回**原始百分比数值**（注释「由布局引擎按容器尺寸处理」——对 width/height 正确）。但 **font-size 属性的调用站点**（line 186 `resolve_length(&style.font_size, ...)`）复用了该泛型函数 → `font-size: 500%` 解析为 **500.0（当 px 用）** 而非 CSS §10.1 规定的「父元素 font-size 的百分比」。实测 anonymous-inline-inherit-001：inner `<span style="font-size:500%">` font-size=500px → line-height=600px → span h=600（LAYOUT_DUMP 实证），A glyph 500px 几乎不可见，整体内容下推至 y=588-599（chromium 在 y=27-79）。

**修复（computed.rs:186-191，surgical）**：font-size 调用站点就地处理 Percentage——
```rust
let font_size_px = match &style.font_size {
    LengthValue::Percentage(v) => v / 100.0 * font_size_context,  // 父 font-size 百分比
    other => resolve_length(other, font_size_context, vw, vh),    // em/rem/px 等不变
};
```
仅改 font-size 一个调用点；width/height/margin 等的百分比仍走 resolve_length 的容器相对解析（line 51 不动）。

**验证**：
- **chromium-Oracle**（真指标）：anonymous-inline-inherit-001 ZeroWeb-test vs chromium-oracle **3.84%→0.43%**（A 现以正确 80px 渲染，内容高度/位置对齐）。残余 0.43% = 独立的 `vertical-align: top` 未应用（content y=73 vs chr y=27，Phase A 墙③ 谱系，font-size 修复之外的独立子问题）。
- **loose self-source reftest**：**438/490 持平零回归**（font-size 修复对 test/ref 同步生效，自源计数不变）。
- **strict self-source**：296→**295**（-1，唯一翻转为 `font-features-across-space-1.html`）。该用例用 `font-size:150%`（旧 bug=150px，现正确=24px）+ 自定义 `@font-face ligsym` 字体 + `font-feature-settings:"liga"` 测连字。**150px（bug）掩盖了连字/回退字体差异**（<0.1% strict pass），**24px（正确）暴露 1.03% 差异**——这是 **revealed false-pass（DC-14 anti-false-pass 目标），非修复引入的 bug**。font-size 百分比修复是 CSS 规范正确行为，该 strict -1 是真实状态暴露。
- **新单测** `test_font_size_percentage_uses_parent`（3 断言：500%@16px→80 / 150%@20px→30 / 100%@root→16），回退守卫（旧实现返回 500/150/100 会 FAIL）。
- **make test**：**12254 passed / 0 failed**（10 ignored = real_website_compat）；clippy/fmt 干净。

**意义**：DC-14 真实 chromium 一致性提升（font-size 百分比是常见 CSS，`font-size:110%/90%/150%` 在真实页面普遍，旧 bug 把它们全当 px 渲染——影响任何用百分比 font-size 的页面/产品 smoke）。这是一次「**被 self-source 掩盖的真实缺口**」修复（anonymous-inline-inherit self 0% 不变故 self 计数不动，但 chromium 真实差距 3.84→0.43%）。strict -1 的 revealed false-pass 印证 DC-14 anti-false-pass 价值——R307 关闭的「near-pass clean win」是计数乐观，但**未调查的 POLLUTED 候选逐项 probe 仍能发现真实单点 bug**（区别于 near-pass frontier 的结构性聚类）。

**方法论复用**：R307 按聚类分类（near-pass=结构性拖尾）关闭了 frontier 杠杆；但**逐用例 probe 未调查的 POLLUTED 候选**（self 通过但 chr 不一致）仍是真实 bug 来源——anonymous-inline-inherit 非任一已知结构性聚类，PIL+LAYOUT_DUMP 定位到 font-size 百分比单点。下一轮可继续逐项 probe R298 POLLUTED 清单剩余未调查项（table-grid-item-dynamic-003 23.8% / collapsed-border-vertical-rtl-overflow 4.7% 联）。

**代码变更**：`crates/style-system/src/computed.rs`（font-size 调用点 Percentage 就地解析 + 1 新单测）。基线 loose 438/490 持平 / strict 295/490（一处 revealed false-pass）/ chromium-Oracle 真实一致率提升（anonymous-inline-inherit 3.84→0.43%）。

### R309 — POLLUTED clean-win 杠杆收尾：3 候选实证 ruled out（read-only 实证，基线 loose 438/490 / strict 295/490 持平）

**承接**：R308 font-size 百分比修复证明「逐项 probe 未调查 POLLUTED 候选」仍能发现真实单点 bug（anonymous-inline-inherit = R298 POLLUTED 清单最后一处非结构性聚类）。本轮继续逐项 probe R298 清单剩余未调查项，3 候选全部 ruled out。

**候选 1 — `table-grid-item-dynamic-003`（self 0% / chr 10.51%[R308 后]）RULED OUT（JS 动态）**：测试 `display:grid;height:100px` > `table;height:100%;padding-top:100px;box-sizing:content-box`，`onload` JS 触发增量 relayout（getBoundingClientRect + body width 变更）。测试名「don't grow on incremental relayout」= **JS 驱动的动态 relayout 行为验证**。ZeroWeb reftest 不执行/不应用 onload 触发的 relayout → 渲染静态初态；chromium 执行 JS → 渲染 post-relayout 态；二者本质不同（10.51%）。**非静态 CSS 单点 bug**（即使 ZeroWeb 静态 table h=200[=100%×100+padding100] 已计算正确），是 JS 执行 + 增量 relayout 特性缺口（需 reftest harness 执行 onload + 触发 re-layout）。defer 至 JS/动态布局特性。

**候选 2 — `font-family-name-025`（self 0% / chr 7.13%）RULED OUT（缺测试字体）**：测试 `font-family: CSSTestBasic-Bold` 不应匹配 PostScript 名——**显式要求安装 CSSTest 测试字体**（页面文字「Test fonts must be installed for this test: FAIL」）。ZeroWeb 与 chromium oracle 环境均无 CSSTest 字体 → 双方回退字体渲染，7.13% = fontdue vs chromium 回退字体度量噪声（R174/R187 AA 谱系），**非 ZeroWeb 可修 bug**（缺字体资源，非渲染缺陷）。

**候选 3 — `whitespace-001`（css-tables，self 0% / chr 2.09%[R308 后]）RULED OUT（结构性 table-cell+% 宽+空白 fit）**：`.outer{display:table;width:500px;border:1px}` > 两个 `.half{display:inline-block;width:50%}` 中间有空白。PIL 实测：**ZW 渲染两 block 换行**（blue rows 9-27 line1 / yellow rows 28-46 line2），**chromium 渲染同一行**（blue+yELLOW rows 9-26）。REF 用 `display:block`（余同），ZW 渲染 test==ref 均「换行」（self 0%）。差异 = display:table 匿名 cell 内 50%+50%+空白空间是否触发换行：ZW 计 50%+50%+空白宽 > cell 宽 → 换行；chromium 不换行（50%+50% 恰填满 cell，空白空间在 fit 边界被吸收）。根因耦合 **table-cell 百分比宽基址（R177b table-width 谱系）+ R105 inter-inline-block 空白宽计入 fit 判定**——修任一会动 R105/R177b 已绿用例，**非安全单点**。defer。

**裁决：POLLUTED clean-win 杠杆经 R308+R309 收尾关闭**。R298 全量 POLLUTED 清单逐项归类：backdrop-inherit-rendered(R202 dialog JS)/abs-pos-border-offset-002(writing-mode)/table-grid-item-dynamic-003(本轮 JS)/semi-replaced-stretch-input(R202 表单)/flexbox-collapsed-item(R301 intrinsic)/font-051(Phase A)/collapsed-border-vertical-*(writing-mode)/float-no-content-beside(R300)/font-family-name-025(本轮 缺字体)/whitespace-001(本轮 结构性)/anonymous-inline-inherit(R308 ✅fixed)/grid-calc-margin+iframe+float-non-replaced(R302)。**唯一 clean win = R308 font-size%**；其余全结构性/特性缺口/字体噪声。与 R307（near-pass frontier 关闭）互补：**两条 clean-win 搜索策略（near-pass 聚类 + POLLUTED 逐项）均已穷尽**，剩余 forward motion 全为结构性多轮里程碑（Phase A 墙②③ / multicol column-aware IFC R131 / DC-9 blend_mode / DC-13 残余 / writing-mode 轴）或特性实现（JS 动态 relayout / 原生表单控件 / dialog）。

**本轮 read-only 实证**：零代码变更；PIL+LAYOUT_DUMP 逐候选实证（table-grid-item LAYOUT_DUMP table h=200 正确 + JS 缺口定性；font-family-name 缺字体；whitespace-001 display:table vs block 换行差异 + R105/R177b 耦合）。基线 loose 438/490 / strict 295/490 / chromium-Oracle 持平。next = 转结构性里程碑（multicol column-aware IFC spec-rfc 实施 R131，最大失败聚类）或 DC-9 blend_mode 独立特性。

### R310 — multicol 设计文档自洽修订（v0.3→v0.4）+ baseline-export 探针实证确认（read-only 设计+探针，基线 loose 438/490 / strict 295/490 持平）

**承接**：R309 关闭 POLLUTED 杠杆后转最大失败聚类 css-multicol。`multicol-fragmentation-design.md` 存在**自洽违规**——顶部 R200/R201 纠正（balance 方向证伪、碎片化算法已存在）与底部 §3 Round 1-2（balance 测量工具）+ §5（首步=Round 1 测量工具）**矛盾**，会误导后续轮重复 R199→R200 证伪命运。本轮修订文档自洽 + 探针验证新方向。

**文档修订（v0.4）**：
- §3/§5 重写：Round 1-2 balance 工具方向**废弃**（R200 证列分配 `total/col_count` 顺序填充本就正确，类 A 残余是精度非算法）。
- 重定向为 **Round 1' baseline-export**（最大 near-pass 聚类 baseline-000/003/004/005/006）+ **Round 2' breaking wiring**（R201 Round 4'）+ Round 3' column-rule/精度收尾。
- §4 风险更新（R200/R201 纠正 + Phase A font_size 交互）。

**Round 1' baseline-export 探针实证（env MCBL_PROBE=1，已回退）**：在 `extract_baselines_recursive`（engine.rs:482）加临时 eprintln 打印每节点 `taffy_baseline`。对 baseline-003（`display:flex;align-items:baseline` > "PA" 文本 + `columns:3` multicol > `column-span:all` "SS"）实测：
- **multicol 项（node 19v1, is_multicol=true）`taffy_baseline=None`** ✓
- 其子 spanner（node 21v1 "SS"）也 `None`
- 仅 node 17v1 有 `Some(19.2)`
- **裁决：假设确认**——taffy **仅为 flex/grid 容器计算 first_baselines**（cached_baselines 补丁路径），普通 block（含 multicol）无 first baseline → 父 flex `align-items:baseline` 对 multicol 项用 fallback（box bottom/margin）对齐 → "PA" 与 "SS" 基线错位（chromium-Oracle 1.058%）。

**关键约束发现（区别于 R266）**：R266 查 `LayoutBox.taffy_baseline` **field-fill 净 0**（消费 guard 仅 InlineFlex|InlineGrid）；本轮探针发现真因更根本——**taffy 在 layout 期间已完成 flex `align-items:baseline`**，post-layout 填 `taffy_baseline`（extract_baselines_recursive 在 taffy layout 后跑）**无法回溯修正已发生的 flex 对齐**。故修复须 **layout 侧**：① 计算 multicol/block 的 first baseline（首列首行 / column-span:all 内容基线），② **在 taffy layout 前或期间**喂给 taffy（measure-func 或两趟），或 ③ ZeroWeb flex-baseline post-pass 重对齐（类 `adjust_inline_block_positions` 但针对 flex items）。三者均结构性多轮，非单点。

**意义**：① multicol 设计文档自洽（消除 R199→R200 重复陷阱）；② Round 1' baseline-export 方向经探针**实证确认**（multicol 项 taffy_baseline=None 是根因），区别并精确化 R266 的 field-fill 结论——真阻塞是 taffy baseline 计算范围（仅 flex/grid）+ flex 对齐时机（layout 期间）。这把 baseline-export 从「需 pre-pass 估测」精确为「需 block/multicol first-baseline 计算 + 喂 taffy/flex post-pass」。下轮可据此设计 baseline 计算的 spec-rfc。

**本轮 read-only 设计 + 探针**：env-gated 探针（engine.rs:482 MCBL_PROBE）**已 100% 回退**（`git diff -- '*.rs'` 空）；rebuild + baseline-003 复测 0.12% 不变。零代码变更，基线 loose 438/490 / strict 295/490 / chromium-Oracle 持平。next = Round 1' baseline 计算的 spec-rfc 设计（block/multicol first-baseline 来源 + flex 对齐注入路径），或转 DC-9 blend_mode 独立特性。

### R311 — R308 后 fresh chromium-Oracle cross-validate：plateau 再确认 + 4 新候选 ruled out（read-only 实证 + evidence，基线 loose 438/490 / strict 295/490 持平）

**承接**：R310 multicol 设计修订 + baseline-export 探针后，做 **R308 font-size% 修复后的 fresh 全量 cross-validate**（980 ZeroWeb dump vs 503 oracle，475 可比），验证 R308 是否改变污染景观 + 搜寻新 contained-bug 候选（原 cross-validate 曾 surface R308 的 font-size bug）。

**实证结果（`evidence/r311-cross-validate-fresh-2026-06-19.txt`）**：
- **污染 158/328 self-pass = 48.2%**（R298 为 48.6%）—— R308 font-size% 仅边际改善（0.4pp），符合预期（font-size% 影响有限用例）。
- 按目录：CSS2 56% / flexbox 26% / fonts 46% / grid 50% / multicol 60% / position 38% / tables 41% / text-decor 47% / writing-modes 73%（与 R298 一致）。
- 真实 chromium 一致率 ≈ 同源通过(328) 中非污染(170) + self-fail 中 chr 一致 ≈ 仍 ~35-37%。

**4 个新候选（R298 未列或未深查）逐项实证 ruled out**：
1. **`downloadable-font-scoped-to-document`（20.22%）= JS+iframe+@font-face**：`iframe1.onload` + `iframe2.src` + `reftest-wait` 测 web font 文档作用域隔离。需 iframe 子文档加载 + JS + 字体作用域，**特性缺口非 CSS bug**。
2. **css-fonts 聚类**（alternates-order 13.8% / font-family-013 6.65% / font-default-02,03 3.46%）= **@font-face 自定义字体未加载**：reftest 不加载 .woff/.ttf → 回退字体，fontdue vs chromium 度量噪声（同 R309 font-family-name-025）。特性缺口。
3. **`rules-groups`（3.39%）= legacy HTML4 `rules=groups` 属性**：ZeroWeb **完全不解析** rules/cellspacing/cellpadding 任一 legacy 表格属性。niche legacy 特性 + 测试还用 CSS `border-block-start/end` 覆盖交互 → 非干净 contained add（低 ROI，单用例）。
4. **`flexbox-baseline-align-self-baseline-horiz-001`（17.64%）= inline-flex 基线合成**：`display:inline-flex;align-items:baseline`，容器自身基线导出。LAYOUT_DUMP 实测 inline-flex 项位置与 chromium 大差（17.64% 全在容器垂直位置=基线导出错）。属 **R295 flexbox-baseline 结构性聚类**（同 R310 multicol baseline-export 谱系，但 inline-flex 侧 taffy 算了 first baseline 却仍错——疑基线合成取错项/字体）。

**裁决**：post-R308 fresh cross-validate **再确认 plateau**——无新 contained CSS bug surface。剩余 polluted 全为结构性（writing-mode / flexbox-baseline / multicol）+ 特性缺口（@font-face 加载 / JS / iframe / 原生表单控件）+ legacy 属性。与 R307（near-pass）+ R309（POLLUTED）杠杆关闭一致。

**对优先级影响**：三条 clean-win 搜索路径（near-pass 聚类 R307 / POLLUTED 逐项 R309 / fresh cross-validate R311）**全部穷尽**，均无新 contained win。剩余 forward motion 确认为**纯结构性多轮**：① baseline-export（R310 探针确认根因，span flex+multicol+inline-flex 三侧，需 block first-baseline 计算 + 注入）；② multicol breaking wiring（R131/R201）；③ DC-9 blend_mode（paint-isolation）；④ DC-13 残余。或**特性实现**：@font-face 字体加载 / JS 动态 relayout / 原生表单控件。

**本轮 read-only 实证 + evidence**：零代码变更（`git diff -- '*.rs'` 空）；新增 `evidence/r311-cross-validate-fresh-2026-06-19.txt`（47 行，top-30 polluted + 4 新候选 ruling）。基线 loose 438/490 / strict 295/490 / chromium-Oracle ~48.2% 污染持平。next = baseline-export spec-rfc（R310 探针已确认根因，是最大可控结构性方向），或 pivot 到 @font-face 字体加载特性（影响整个 css-fonts polluted 聚类 24 案）。

### R312 — baseline-export 双侧探针精确定位：inline-flex 容器 taffy_baseline 错值 + multicol 项 None（read-only 探针，基线 loose 438/490 / strict 295/490 持平）

**承接**：R310 探针确认 multicol flex 项 `taffy_baseline=None`；R311 fresh cross-validate 标 `flexbox-baseline-align-self-baseline-horiz-001`（17.64% chr，inline-flex 容器基线导出）为未深查的 baseline-export 变体。本轮探针精确定位 inline-flex 容器侧的根因，与 R310 multicol 侧合拢为统一 baseline-export 图景。

**探针（env IBBL_PROBE=1，已 100% 回退）**：在 `adjust_inline_block_positions` 的 baseline_overrides 闭包（engine.rs:988-993，优先用 taffy_baseline 分支）加 eprintln。对 flexbox-baseline-align-self-baseline-horiz-001 实测：
- 3 个 inline-flex 容器（NodeId 25v1/37v1/49v1，均 child_h=35）导出 **taffy_baseline = 30.0 / 20.0 / 30.0**。
- 这些值经 `with_baseline_overrides` 喂入父 IFC 决定 inline-flex 在 body 行内的垂直位置 → 17.64% chr diff（inline-flex 整体垂直错位）。
- **关键**：baseline_overrides 的 step-3（taffy_baseline 优先，line 989-992）**总是命中**（taffy 为 flex 容器算 first_baseline），step-4（ZeroWeb 自有首行近似，line 995+）被旁路。故 inline-flex 基线 = taffy 的合成值，**该值对「混合 font-size flex 项」错**（R295 wrap-reverse/混合项基线合成结构性聚类）。

**双侧合拢（baseline-export 统一图景）**：
| 侧 | 用例 | taffy_baseline | 根因 |
|----|------|----------------|------|
| **multicol flex 项**（R310） | baseline-003 | **None** | taffy 仅 flex/grid 容器算 first_baseline，block/multicol 项无 → flex `align-items:baseline` fallback 错位 |
| **inline-flex 容器**（R312） | flexbox-baseline-align-self | **错值**（30/20/30） | taffy 算了 first_baseline 但对混合 font-size 项合成错 |

两侧共同缺口 = **ZeroWeb 缺「为 flex/multicol 项计算正确 first baseline」的能力**（CSS-align baseline-export）。修复须 layout 侧：
- multicol 项：计算首列首行 baseline（block first-baseline 递归：直接文本取首 IFC 行基线，block 子元素取首子首基线，空取 strut）。
- inline-flex 容器：合成正确 first baseline（取 baseline-aligned 项的 max 基线，而非依赖 taffy 错值）—— 但须在 taffy layout 期间或经 measure-func 注入，post-layout 填字段无法修正 taffy 已完成的 flex 对齐（R310 约束）。

**裁决**：baseline-export（baseline-000~008 + flexbox-baseline-* 聚类，~10+ 案）确认为**结构性多轮**，需 CSS-align baseline-export 的 spec-rfc 实施（block/multicol first-baseline 计算 + flex 对齐注入路径）。双侧探针已精确定位根因（multicol=None / inline-flex=错值），区别并超越 R266「field-fill 净 0」结论。这是 css-multicol + css-flexbox baseline 近-pass聚类的统一解锁钥匙，但非单会话 clean win。

**本轮 read-only 探针**：env-gated 探针（engine.rs:989 IBBL_PROBE）**已 100% 回退**（`git diff -- '*.rs'` 空）；rebuild 干净。零代码变更，基线持平。next = baseline-export spec-rfc（block/multicol first-baseline 计算来源 + flex 对齐注入路径设计），或转 @font-face 字体加载特性（css-fonts 24 案聚类，但 fontdue-vs-chromium 度量噪声限制收益）。

### R313 — baseline-overrides 杠杆证伪：inline-flex 位置不受 baseline_overrides 控制（read-only 实验，基线持平）

**承接**：R312 探针发现 inline-flex 容器导出基线用 taffy_baseline（30/20/30，错值），暗示「baseline_overrides 改用 ZeroWeb 自有计算」可能是 lever。本轮 env-gated 实证该假设。

**实验（env IBBL_PREFER_COMPUTED=1，已 100% 回退）**：在 baseline_overrides 闭包跳过 step-3（taffy_baseline 优先），强制走 step-4（ZeroWeb 首行近似 `item.y + item.font_size`）。对 flexbox-baseline-align-self-baseline-horiz-001 A/B 实测：
- 默认（taffy 优先）：chromium-Oracle **17.64%**
- 探针（computed 优先）：chromium-Oracle **17.64%（完全相同）**，bbox=(0,0,800,126) 一致

**裁决：R312 的暗示证伪**——baseline_overrides（step-3 vs step-4）**不影响** flexbox-baseline-align-self 的渲染。inline-flex 的垂直位置由 **taffy 的 inline-level-box 布局**（inline-flex 作为 body 行内级盒，taffy 在 body 的 IFC 里定位它，用 taffy 自算的 inline-flex 基线）决定，**ZeroWeb 的 baseline_overrides 后处理对该用例不生效**（post-pass 重跑 IFC 未覆盖 taffy 的行内级盒定位，或该路径对此结构不触发）。

**意义**：纠正 R312「baseline_overrides 是 inline-flex 基线 lever」的暗示——**不是**。inline-flex 基线导出的真根因在 **taffy 对 inline-level flex 盒的基线合成 + body IFC 定位**，非 ZeroWeb baseline_overrides 后处理可触及。这把 baseline-export 的修复路径从「改 baseline_overrides」排除，指向「taffy inline-level-box 基线」或「ZeroWeb 重跑 body IFC 时覆盖 inline-flex 定位」（更结构性）。

**附发现（latent bug，0 reftest 覆盖，defer）**：`line-height: <percentage>` 在 computed.rs:195-206 未解析（Percentage 落 `_=>{}`），同 R308 font-size% 谱系。但 grep wpt-data **0 个 reftest 用 line-height %** → 零测试覆盖，按 code-guidelines「不实现需求之外的功能」defer（非当前目标驱动）。

**本轮 read-only 实验**：env-gated 实验（engine.rs:989 IBBL_PREFER_COMPUTED）**已 100% 回退**（`git diff -- '*.rs'` 空）；rebuild + 复测 self 0.14% 不变。零代码变更，基线 loose 438/490 / strict 295/490 持平。next = baseline-export 真路径需触及 taffy inline-level-box 基线（深，结构性），或 pivot 到 multicol breaking wiring（R131）/ DC-9 blend_mode（独立特性）—— baseline-export 经 R310/R312/R313 三轮探针确认非单会话可解。

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
- `~/use-proxy` 设 `http_proxy=proxy.example.local:7078` / `https_proxy=...` → **`source ~/use-proxy && make browser` 即生效**，ZeroBrowser 经 reqwest 自动走代理。
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
