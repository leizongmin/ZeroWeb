# R5297 — inline 多片段 slice 背景行连续域 + 内容区锚 + 跨 inline 边界空白折叠——c42-ibx-ht-000 ✗1.73→✗1.19（真改善留红）

日期：2026-10-27。改动：`crates/engine/src/paint/painter/text.rs`（多片段 slice 整盒门解封 + owner 行连续 x 域表 + 内容区锚对多片段生效 + 行域止步盒修正 + bg-image 门全 None 层视同无图）、`crates/layout-engine/src/inline/break_lines.rs`（词臂前导空白折叠门 +1 条件）、两 crate 各 +1 回归测试文件。无新 kill-switch（单片段 kill-switch 维持默认关）。

## 案面

`css/CSS2/css1/c42-ibx-ht-000`（CSS2 §10.3.4/c42 系列：inline 盒 border/padding 垂直外溢与背景覆盖）：div 20em×10px/12px Ahem 绿底内 `<span>`（border solid 10px lime + padding 2px 0 2px 10px）跨 5 行折叠，内嵌 `<em>`（font 1em/1 红色）跨 L2→L5；ref 页 = 嵌套 span background-color lime 逐行等价形态。R5296 交付态 ✗1.73%。

## 取证链（chromium oracle 像素级画像）

1. **oracle 可全胜定谳**：chromium（非 headless + CDP，DISPLAY :101）渲染 test 页与 ref 页**逐像素相同（0px 差）**——本案地板为 0，凡 pair > 0 皆我方缺口。oracle 截图入库 `tests/wpt-runner/oracle-shots/css_CSS2_css1_c42-ibx-ht-000-ref_xht.png`（gitignored，`git add -f`）。
2. **chromium inline 装饰模型**（逐带像素画像）：行片段**连续**带/bg 贴**内容区** [B−A, B+D]（Ahem 10px/12px → 10px 带 + 2px half-leading 行间缝，带 [51,61)/[63,73)/…）；border 带 = 内容盒 ± pad ± border；**parent 片段延展域贯穿子 inline 领土**（bg/border 在嵌套 span 下连续）；行 k+1 的 BT 带绘于行 k 字形之上（双引擎同序，无残留在案）。
3. **我方旧模型**（共享根因）：逐词 run 矩形 @ `baseline−line_h`（行盒 12px 带无缝、空格除外、止步子 inline 边界、带锚旧基线域 [B−bt, B]）；test 侧对 oracle 差 9553px（1.99%）、ref 侧差 7729px（1.61%）。
4. **跨 inline 边界空白双计**：`z z <em> x` 形态旧实现保留 2 空格 advance（20px）——chromium 1 空格；em 首词全行右移 10px、5 行全错位。

## 实现（text.rs + break_lines.rs）

- **多片段 slice 整盒门解封**：`r4332_whole_box` 对多片段分支不再随单片段 kill-switch（`ZW_INLINE_SINGLE_FRAG_BOX_BORDER` 默认关）回落——R5120 重构误把多片段并入此默认关开关，横条回落基线锚+词宽。
- **owner 行连续 x 域表** `owner_line_band_extent_map`：run 归属 owner（文本节点取父元素）沿祖先链向**每个**中间 inline 祖先累计 (min_x, max_end)，键 (NodeId, line_top bits)；查表 miss 回落词 run 域。
- **内容区锚对多片段 slice 生效**：`r5120_content_box` 去 `!is_multi_line` 限制——带/bg 贴 [B−A, B+D] ± pad ± border（度量缺失回退行盒锚）。
- **行域止步盒修正**：`stop_at` 仅块容器（`style.display == Inline` 的 box_node 自绘 IFC 时不可止步于自身——span 自盒存储 IFC 的 pass 片段 owner = 自身，止步把自身表项全剔除 → 查表全 miss 回落词宽，单测实证）。
- **bg-image 门**：`background: <color>` 简写留一层 `None` 图像占位（非空 vec）——门义「无真实图像」改全 None 层视同无图（bgi=1 曾把 bg 色多行 span 挡回词宽旧轨）。
- **跨 inline 边界空白折叠**（break_lines.rs 词臂）：前导空白门增 `!last_ended_collapsible_ws`（css-text-3 §4.1：上一内容词尾随可折叠空白与后继 run 前导空白同属一个空白序列，只保留前者 advance）；纯空白 run 路径（`last_was_collapsible_ws`）与行首语义不变。
- **行域 x 基座换算 + 多列守卫**（corpus 复验追加，见交付）：宏头查表值/回退值统一乘 (content_x + tx)；`multicol_info.is_some()` 时不建表（跨列键碰撞）。

## 交付

- **c42 ✗1.73→✗1.19**（阶段：1.59 解封门 → 1.63 行域带（pair 反弹：ref 侧等量误差不抵消）→ 1.44 内容区锚+行域几何 → 1.21 空白折叠 → 1.19 行域 x 基座换算；ref 侧对 oracle 7729→4469px（1.61%→0.93%））。**残余 = 混合 line-height 行距 bug**（见挂账），非本臂缺口。
- 全量 corpus **16591/15732P/859F**（含本轮自带 gitignored 诊断 probe 1 案；真实 corpus 16590 计 **15732P/858F**）：**inline-formatting-context-022 ✗1.36→✓0.00**（新绿，x 基座换算后像素全同）；multicol-basic-001..004 通过态**原地改善至 ✓0.00**；翻转 = {−IFC-022, +hanging-punct}。
- **行域 x 基座换算**（第 6 修，corpus 复验抓出）：表值/回退值均容器相对 x，绘制须乘 (content_x + tx) 同基座——缺 content_x 时整带左移出内容盒（c42 带左移 body margin 8px 被 pair 对消掩盖；multicol-basic-001..004 边框带实证 2.33% 翻红→修复后 0.00%）。
- **多列路径不建行域表**：键 (owner, line_top) 跨列碰撞会把不同列行域焊成横跨列缝的一条带（multicol 翻红第一因）；回退词 run 域，列内连续域留待键扩列索引再启。
- **hanging-punctuation-inline-bound-001 ✓3.44（阈值 5%）→ ✗10.27 翻红**（机制定谳，非回退）：REFTEST_DEBUG + 门控 trace 证明 test 侧（wrap span，mf=true）已进整盒臂连续带，ref 侧三 span（`<br>` 分行 + owner_h=0 orphan fire）**仍走单片段 legacy 词宽轨**——pair 度量惩罚「单侧前进」。chromium 两侧皆内容区带，我方 ref 是滞后侧；修复方向 = R5120 单片段 kill-switch 转正（独立轮 + 独立 sweep，风险面 border-width-applies-to-008 15.53 既有红与 bidi-003 族）。
- 敏感族复验（全部修正后复跑）：bidi-003 ✓0.81、line-breaking-bidi-003 ✓0.00、unicode-bidi-003 ✓0.10、content-height-001 ✓0.35、border-width-applies-to-008 ✗15.53（R5120 既有红不变）、blocks-019 ✓0.00、multicol-basic-001..004 ✓0.00。

## 回归测试

- `r5297_ml_span_bg_line_contiguous_across_child_inline`（engine paint）：4 行折叠 span 内嵌 em 跨 L2→L3，逐行连续带（w=50 贯穿子 inline 领土）、组内全等、行序 L1→L4。竖直锚在单测进程无字体库度量时回退行盒域（12k, 12）——内容区锚判例由 corpus c42 reftest 对 + oracle 截图锁定。**本测试抓出两个真缺陷**（止步盒取自身、bgi 占位层），非事后补钉。
- `r5297_leading_ws_collapses_with_prev_trailing_ws_across_runs` / `..._kept_when_prev_word_has_no_trailing_ws` / `..._pure_ws_run_path_unchanged`（layout-engine）：跨边界双空白折叠单空格、无尾随空白时前导空白照常计、纯空白 run 原门保留。

## 门禁

- make reftest 716/716；product-smoke 6/6 struct PASS；bench-gate 定向 `ZERO_WEB_BENCH_CRATES="zero-engine zero-layout-engine"` **GATE PASS**（19 指标 NEW=0；前两跑 medium/first_paint_wall 366.9/397.33 超预算 1.3–9.7% 系对端流 test-all 并行负载（loadavg 13），机器空闲复跑 PASS——R5296 同形先例）。
- fmt 0 diff；clippy（zero-engine + zero-layout-engine --all-targets）0 告警；engine lib **3543P(+1)**、layout-engine lib **2171P(+3)**。
- 全量 corpus 终态 `r5297-reftest-full3`：16591/15732P/859F（真实 corpus 858F 净零翻转）。

## 挂账（新矿井）

1. **混合 line-height 行距 bug（c42 残余 1.21% 的直接原因，下一步首选）**：小行高 run（`font:1em/1` em）参与行时 IFC strut/基线放置错——我方 test 页基线 59/70/81/93/105（行距 11）vs chromium 59/71/83/95/107（行距 12）；R1442 `box_height() = line_height` 不变量已排除，属行盒 strut 模型独立缺口。带下游全行位移。
2. **单片段/orphan inline 装饰 legacy 轨转正**（R5120 kill-switch 默认开）：hanging-punct 翻红的滞后侧；需独立 sweep（敏感：border-width-applies-to-008 既有红、bidi-003 族单行 lh:3em 形态、content-height 族）。
3. **selection fill 逐词 run 域**（R5270 域 FIXME 已注）：chromium 按行片段连续域，代码内 FIXME 在案。
4. **c43-rpl-ibx-000（1.50%）/ c44-ln-box-001（1.60%）**：replaced-element 入行盒域，不同矿井（oracle ref 截图已入库）。

## 教训

- **pair 度量奖励两页一致、不奖励对齐 chromium**：单侧修复可翻红（hanging-punct 3.44→11.12）。定谳须回 oracle（两侧对 chromium 的误差各自归零才可交付）；「test 前进 / ref 滞后」型翻红要查滞后侧是否同轮可达，不可达则挂账并明确转正路径。
- **宏卫生（hygiene）使调用点局部不可见**：`render_fragment!` 宏内访问不到调用点局部变量，新增输入必须作宏参传入（本轮 band map 第 14 参）；查表 miss 的静默回落会掩盖调用点/宏参失配——诊断期 eprintln hit/miss 比读代码快。
- **`background: <color>` 简写非空 `background_image`**：样式系统留 None 层占位，「无图」判据写成 `is_empty()` 会把所有简写 bg 色元素误判有图；语义判据应写「全 None 层」。
