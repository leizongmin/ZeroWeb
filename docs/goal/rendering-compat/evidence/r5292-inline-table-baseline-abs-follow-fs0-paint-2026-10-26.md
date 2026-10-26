# R5292 — inline-table 基线探针 + auto 宽补源 + abs 跟随 + fs:0 paint 放行 + run 度量镜像——table-vertical-align-baseline-009 ✗1.15→✓0.57%

日期：2026-10-26。改动：`crates/layout-engine/src/inline_finalization.rs`（+349/−47，含 R4380 原子基线探针盒参数化重构）、`crates/engine/src/paint/painter/text.rs`（+12）。新 kill-switch 4（均 default-on）：`ZW_INLINE_TABLE_BASELINE` / `ZW_REMEASURE_ABS_FOLLOW` / `ZW_PAINT_FS0_STORED_BYPASS` / `ZW_MIRROR_RUN_FONTS`。

## 案面

`table-vertical-align-baseline-009`（R5291 账本 vertical-align-baseline-009 1.15%）：test 页 `div`（`float:left;position:relative;font-size:0;background:red`）含三个子——`position:absolute;bottom:0` 的 span X、普通 span X、inline-table span（两个 table-row 各含 X），全部 span `font:50px/1 Ahem;color:green`；ref = 100px 绿方。期望几何：inline-table 基线 = 首行 cell 基线（ascent 40），单行盒高 100（ascent 40 + descent 60），容器高 100，abs span X（50px）底对齐容器底、与 row2 的 X 叠合成绿方。

## 取证链

四层，layout dump + 临时探针 eprintln（均已删）：

1. **abs y 现形**：`position:absolute;bottom:0` 的 span y=150 贴包含块底之下（应底对齐）。最小复现证明 abs bottom 数学自洽（y=108 = 8+150−50）→ 真 bug 是**行盒高 150 应 100**：div 的 remeasure IFC 产出两行 `[(0,50,40), (50,100,40)]`——inline span X 与 inline-table 之间被断行（容器 784 宽装得下，不应断），行盒 150 = 50+100，把 abs bottom 解连带推偏。
2. **基线根因**（collect_items.rs:1059）：inline-table 的 collect 期基线回退 = 底边 `h`，R4380 `baseline_overrides` 探针只填 InlineBlock；CSS2 §17.5.1：inline-table 基线 = **第一行 cell 的基线**。基线 100（底边）→ 行盒 150；基线 40 → 行盒 100。
3. **override 键错配**：probe 写 override 用的键是 LayoutBox 子盒 id（`24v1`），而 collect 期 DOM 遍历把 inline-table 收成 DOM 元素 id（`21v1`）——表管线给 inline-table 建了匿名包装盒（id 重建），键错配导致 override 永远 miss。消费端（collect_items.rs:1035）按 DOM 元素 id 查 map，命中即用不分 display。
4. **断行根因**：remeasure 期 `ib_sizes` 的 auto 宽 inline-table 宽 = **784（taffy junk 全宽拉伸）**→ X(50)+空白+784 > 784 触发 fit 断行。R5243 已有 auto 宽 inline-table 的 max-content 测量（`inline_table_auto_max_content`）可直接补源。
5. **修复显形两层 paint 缺口**：基线/断行修对后 layout 全对，但 X 仍错绘——①div `font-size:0` 触发 paint_text 早期返回门（glyph-only 时代 d4d910c14 遗留）整盒跳过 stored replay/Path B，子孙 50px X 落 16px 兜底路径；②`has_direct_paintable_text` 对该 div 判 false（inline 子树含 table-row 属三类深路径排除）→ 容器整跳 paint_text，行内子盒自绘时自身 text store 空 → 同样 16px 兜底。abs span 修对布局后位置仍偏 → remeasure 高回写后 abs 盒不跟随（taffy 首趟按旧高解析、无第二趟）。

## 实现

1. **inline-table 基线探针**（inline_finalization.rs）：`probe_inline_block_baselines` 扩 InlineTable 分支——`find_first_table_row` → `find_first_table_cell(row).unwrap_or(row)` → `probe_atomic_box_baseline`（R4380 盒参数化 + `first_line` 参数选行盒）取首行基线；override 键 = **DOM 元素 id**（修匿名包装盒 id 错配）。kill-switch `ZW_INLINE_TABLE_BASELINE`。FIXME：`rowspan` 跨首行的 cell 基线取整行盒未精化，案面无跨行。
2. **auto 宽 inline-table 补源**：remeasure `ib_sizes` 的 filter_map 对 auto 宽 inline-table 用 `inline_table_auto_max_content`（R5243，cap 容器宽）补 max-content 源——消 784 junk 拉伸断行。
3. **abs bottom 锚跟随**：`shift_abs_children_on_cb_height_change`（kill-switch `ZW_REMEASURE_ABS_FOLLOW`）——remeasure 容器高回写（grow/shrink 两臂）后对 `is_abspos_cb` 的**直接** abs 子做 delta 平移：height auto + top/bottom 均 definite → 高补差；top auto + bottom definite → y 平移 Δ（% bottom 按 (1−p) 缩放）。锚缘（CB padding 盒底）位移恰 = Δheight，不重算 CB 基。11.7a walk（engine.rs resolve_abspos_against_nested_cb）显式跳过「本盒即 CB 的直接子」域（abspos.rs R4408 注：首轮无差别重解析 −20 回归），故增量平移而非重解析。FIXME：经 static 中间层的更深 abs 后代（11.7a cb 传递域）未跟随。
4. **fs:0 paint 门收窄**（text.rs，kill-switch `ZW_PAINT_FS0_STORED_BYPASS`）：`font_size <= 0.0` 早期返回仅当「无任何 stored run 带非零字号」——`font-size:0` + 行内子 span 非零字号正是 WPT 抑制空白的惯用法，整盒跳过会把子孙文本留给 16px 兜底错绘。store 空盒保守回退旧行为（直系 0px 文本不可见语义不变）。
5. **run 度量镜像**：`mirror_run_fonts_to_inline_child_boxes`（kill-switch `ZW_MIRROR_RUN_FONTS`）——remeasure `store_font_sizes_from_ifc` 后把 run 度量（font_size/is_ahem/family/letter_spacing/glyph verticals/ws_override）镜像进直接 inline 子盒 text store（键 = run 文本节点 + 载体元素 + 载体文本子节点），供子盒自绘 Path B 免 16px 兜底。**`is_text_run` 过滤是正确性关键**：原子盒（inline-block/inline-table）与块级子在父 IFC 中同样以元素归因 run 出现，其 font_size 是盒级量而非文本字号，写进该盒自身 store 会毒化其 Path B 字体解析（vars-font-shorthand-001 的 150px X 整体消失实证）——仅文本节点 + display:inline 载体放行。FIXME：更深 inline 后代（span>span 自绘域）未覆盖。

## 交付

- 目标案 `table-vertical-align-baseline-009` **✗1.15 → ✓0.57**（阈值 1.00%）；残余 = inline-table paint 高 110px 溢出 10px 带（约 0.1%）+ AA 噪声。
- 修复中途 3 案回归（镜像初版未过滤原子 run）：contain-layout-baseline-004 / flex-direction-with-element-insert / vars-font-shorthand-001——`is_text_run` 过滤后全复验 ✓（0.58% / 0.99% / 0.25%）。
- 全量 corpus **16590/15729P/861F** vs R5291 交付态 15728P/862F：失败集**恰移出目标案一案、零新增翻转**（两次全量跑一致；归一化 A/B：`/tmp/r5291-norm.txt` vs `/tmp/r5292-fail2.txt`）。
- 中途观察：display-contents-table-003 曾随镜像初版翻绿，原子 run 过滤后回基线失败态 7.00%（其真因非本轮域，可接受）。
- 挂账新增（本轮取证中发现、深结构不动）：anonymous-boxes-001a（R109 匿名块内原子 inline 内联化，跨 tree/paint/IFC 三层）、column-visibility-004（跨列 cell 字形级抑制/平移，R1492 级）。

## 验证

- make reftest 全量两轮一致：**716/716，16590/15729P/861F**。
- make test 84 腿：唯一 F = `image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight` 已知并行时序 flake（同轮另一腿 ok，standalone 带 test-guard 1P/1.56s，R5288 同口径；网络调度 e2e 与本轮 layout/paint 改动面无关）。
- product-smoke **6/6** struct-check PASS + welcome oracle **15.50%** 持平；bench-gate 定向 zero-layout-engine **GATE PASS 25/25**；fmt 0 diff、clippy 干净（helper 插入挤走 `#[allow(clippy::too_many_arguments)]` 的回归已修）。

## 教训

- zero-wpt-runner 逐案 ✓/✗ 结果走 **stderr**——grep 结果必须 `2>&1`，`2>/dev/null` 会把整轮二分做成全 0 的假阴性（本轮最初 kill-switch 二分因此重做）。
- 父 IFC 的元素归因 run 不能当文本度量写回该元素自身 store：原子盒在父 IFC 的 font_size 是盒级量，镜像前必须按「文本节点 / display:inline 载体」过滤，否则毒化其自绘 Path B。
- self-matching reftest 恒 0.00% 无法取像素取证——临时把 match href 指向另一案强制失败触发 `REFTEST_DUMP=1` 才拿到并排 dump。
