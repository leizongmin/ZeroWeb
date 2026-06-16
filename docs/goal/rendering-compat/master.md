# 渲染兼容性目标 — 运行时控制面板

**最后更新**: 2026-06-16
**当前活跃里程碑**: M10 — 上游 WPT 真实 Reftest 通过率提升（Phase A 部分解锁）
**上游真实 reftest 通过率**: 88.6% (434/490) R168（同源持平；**chromium Oracle 真实修复 ×2**：R168 table height-as-minimum 修复 table-grid-item-dynamic-004 chromium 差距 11.12%→2.98%——修复 table `height` 属性被完全忽略的真实 bug（CSS §17.5.3），同源零回归；R165 margin:auto 水平居中修复 html-display-table chromium 差距 33.09%→2.63%）。**434 即诚实 DC-14 基线，无需恢复 436**（R164 证否 vrl-004/008 R114b 路径：正确 vertical-rl CSS 使 4/4 vrl 变差，因同源 REF 水平渲染 vs 正确 vertical-rl 右侧块起始结构性不可对齐；chromium Oracle 证同源 REF 比 chromium 更怪异：vrl-004 同源 7.09% vs chr 5.08%，font-051 同源 8.19% vs chr 1.62%）。R163 PNG 正确 RGBA 默认启用（DC-14 anti-false-pass）。draw_order 默认启用满足 DC-10。剩余 56 同源失败（结构性多轮 + REF 怪异产物）；**优化目标已转 chromium Oracle 一致率（d16bb8e），18 真 bug 候选见 `evidence/analyze-pollution-2026-06-16.txt`**。



**可信指标口径（唯一达标判定依据）**：上游真实 reftest 通过率 **434/490 (88.6%)**（R163 起默认正确图像渲染=DC-14 anti-false-pass，消除 PNG 退化假绿；旧 436 含 garbled-image 假通过）。⚠️ 当前 reference 仍由 **ZeroWeb 自渲染 ref.html**（`reftest.rs:230-232`），衡量「ZeroWeb-test vs ZeroWeb-ref」一致性而非「ZeroWeb vs Chromium/标准」，存在**同源假通过**风险（test 与 ref 同错）；治理门禁见 **DC-14 真通过标准**（独立 chromium Oracle 交叉验证基建已就绪 72764a0）。内联 reftest 685/685 (100%) 为 smoke，**不计达标判定**。

**归档策略（约每 20 轮一次）**：约每 20 轮做一次 archive——本文件保留最近 10 轮，更早的约 10 轮移入 `archive/` 目录下的归档文档，避免随轮次无限增长。当前已归档 R139 及更早（91 轮）至 `archive/rounds-r23-r139.md`；下次归档窗口约在再增 10 轮后（届时 R155~R146 移入归档，本文件仅留最新 10 轮）。

### R177 — top 候选根因实证确认（colspan 5 部件机制 + morning.work blue-nav = inline→block 背景结构性，诊断轮，无代码提交）

本轮对 `evidence/analyze-pollution-2026-06-16-r168.txt` 18 真 bug 候选逐一实证复核，**再次确证 R118/R140「clean win 已穷尽，剩余全结构性」结论**。上游同源基线 **434/490 (88.6%) 持平**（无代码变更）。把最高价值候选（colspan 52% chr）的**精确可执行机制**钉死，供下轮直接施工，避免重复 40+ 步实证。

**colspan 实证机制（ZW_TABLE_DEBUG 插桩 + 像素逐表分析确认）**：`table_grid_size_col_colspan`（同源 0% 但 chromium 52%）真实根因 = `<col>` 元素在 `build_grid`（table.rs:1065）被 `_ =>` 分支**完全跳过**，导致：(1) **col_count 只来自单元格 colspan**——`<table><col>×3><td colspan=1>` 的 col_count=1（非 3），`<td colspan=4>` 的 col_count=4（**未钳制到 3 列**，违反「colspan 不超出 grid」断言）；(2) `<col>` 的 `width:50px` **从不读取**（仅 detect_collapsed_columns 读 visibility）；(3) `width:auto` 表格 `apply_table_size_constraints` 设 `content_width=intrinsic` 想收缩，但 `compute_column_widths` 的 `cell_used_width` 对空单元格返回 `compute_cell_intrinsic_width=char_width+padding≈11.6`，再被 line 1675 的 `(has_explicit_width || is_fixed_layout)` 扩展条件**撑到容器 784px**。chromium 实测：每表收缩到「单 cell 跨的列数 × 50px + 边框」（60/110/160/60/60px），空列（无 cell 的 `<col>`）被裁剪。

**colspan 修复 = 5 个相互耦合部件（不可单点）**：(a) build_grid 让 `<col>`/`<colgroup>` 计入 col_count；(b) colspan 钳制到 grid 列数；(c) compute_column_widths Pass 0 读 `<col>` width；(d) 裁剪无 cell 的空列；(e) width:auto 表格收缩到 grid（改 line 1675 扩展条件）。**风险分级**：实测 6 个含 `<col>` 用例中，`col-definite-size-001`/`col-definite-max-size-001`（同源 0%）test 与 ref **结构相同**（同 4×`<col width>`）→ 加 col-width 读取两侧同变仍匹配 = **安全**；`insert-after-colgroup`（动态插入）/`visibility-collapse-colspan-003`（collapse 交互）/`border-collapse-dynamic-col-001`（动态+border-collapse）/`table_grid_size_col_colspan`（需空列裁剪）= **风险**，须配套 (b)(d) 且逐用例验证。`cell_used_width` 对显式 width 单元格用 `cell_box.width`（taffy 拉伸的 784）而非 CSS 显式值——独立子 bug，但单独修会打破 colspan 同源（test 空 cell 11.6 vs ref width:40px），需与 col-width 打包。

**morning.work blue-nav（剩余 28.72% 最大块 ~55k px）= inline→block 背景结构性**：article.html 的 3 个 `.item-tag <span>`（display:inline UA，`background:#607cd2`）经 converter `DisplayValue::Inline → taffy::Block`（converter/mod.rs:265）变全宽块，**文本+背景均纵向堆叠**（3 条全宽蓝条 y=169-241），非行内小徽章。chromium 渲染为行内 badge。属 R109 IFC ownership / inline→block 架构（goal P1），非单会话。chromium 逐表宽度已量化备用。

**其余候选复核结论**：`backdrop-inherit-rendered`(47%) 需 `dialog::backdrop`+showModal JS = 范围外/过重；`baseline-block-with-overflow-001`(45%) = overflow:hidden inline-block 基线=底边（CSS2 §10.8.1）结构性；`flexbox-collapsed-item-horiz-001`(20%) = float flex 容器 shrink-to-fit（R129 仅覆盖 block 子元素）结构性；`flex-abspos-inset-nested`(18%)/`fixed-table-layout-percentage-in-flex-item`(11%) = flex item 含 table 的 definite size 结构性；writing-mode 系（box-offsets-rel-pos-vrl 等 22-25%）= R114/R142/R164 已 4 轮证伪的轴交换；multicol 17 失败 = R113/R122/R131 碎片化结构性；min-max-size-table(36%)/table-cell-width-0(30%) = R97 内在尺寸 cluster；近阈值（baseline-007 1.04%/css-flexbox-row 1.23%/block-in-inline 1.37%/child-border-box-max-content 1.52%）均经查为 flex/multicol 基线 + taffy grid 限制结构性非 clean。

**下轮最高杠杆**：colspan 5 部件修复（(a)+(c) 安全起步可独立提交保 col-definite-size/max-size chromium 一致；(b)+(d) 需含 colspan/visibility-collapse/dynamic-col 4 用例逐验证；(e) 收缩改条件须全 css-tables 51 用例回归）。或转 DC-13 WinterTC fixture 量化（R176 已备 img 固有尺寸+JPEG 解码，需录制 wintertc.org 静态 fixture）。

### R176 — `<img>` 解码固有尺寸注入布局 + reftest harness JPEG 解码（DC-11 替换元素 + DC-13 图片子资源能力，零回归，已提交）

补齐 DC-11「替换元素固有尺寸」与 DC-13「图片子资源」的关键缺口：`<img>` 无 width/height 属性时此前依赖 CSS 或默认尺寸（DC-11 标记 P1 未实现），导致图片密集真实页面（WinterTC 首页 Logo 等）的 `<img>` 布局塌缩。

**修复（layout-engine + engine + reftest harness 三层）**：
1. `RenderPipeline::build_img_intrinsic_sizes`（pipeline.rs）——从 `self.image_sizes`（URL hash 索引，由调用方从解码后 ImageCache 预解析）按 DOM NodeId 解析 `<img>` 固有尺寸。hash 在 engine 层解析（`simple_hash` 定义于 engine crate），避免把 hash 函数泄漏到不依赖 engine 的 layout-engine。所有 4 处布局入口（`render_html`/headless/缓存路径）改调 `compute_with_img_sizes`。
2. `LayoutEngine::compute_with_img_sizes`（engine.rs）——新增 pub 方法，透传 `img_intrinsic_sizes: HashMap<NodeId,(f32,f32)>` 到 `build_layout_tree`；`compute` 保持不变（传空 map）。
3. `apply_replaced_element_sizing`（tree.rs）——在「无 HTML 属性」分支回退到解码固有尺寸：仅在 `width:auto`/`height:auto` 时注入，`aspect_ratio` 未显式设置时按固有尺寸比补设（与有 HTML 属性分支对称）。
4. reftest harness JPEG 解码（reftest.rs `load_jpeg_file`）——`build_image_cache` 旧仅支持 PNG，真实页面 logo/照片多为 JPEG；新增 jpeg-decoder 解码（RGB24/L8/CMYK32/L16 → RGBA8），PNG 失败再尝试 JPEG。

**验证**：上游 reftest **434/490 持平零回归**（set-diff 零翻转——upstream 用例的 `<img>` 多带显式 width/height，无属性且 base_dir 有图的少数用例尺寸变化两侧 test/ref 同源仍匹配）；inline 686/686；make test **12189/0**（+1 单测 `test_img_intrinsic_size_from_decoded` 验无属性 `<img>` 用解码固有尺寸）；clippy/fmt clean。

**意义**：(1) DC-11 替换元素固有尺寸从「未实现」到「解码驱动」，与 DC-13 图片子资源/ImageCache 贯通闭环——为 WinterTC 等图片密集 fixture 的 `<img>` 正确布局奠基（此前无尺寸 img 被 taffy 当 0 尺寸块）；(2) `<img>` 固有尺寸与 DC-13 morning.work/wintertc 真实页面衔接，下一轮可量化 WinterTC Logo 布局改善；(3) hash 解析留在 engine 层是依赖边界正确选择（layout-engine 不依赖 engine）。

**剩余**：JPEG/SVG 栅格化渲染本身（DC-8 CPU ImagePrimitive 图元已能从 ImageCache 绘制，本轮只补布局侧尺寸）； WinterTC fixture 的产品 smoke 量化为下一轮目标。

### R175 — CSS 自定义属性继承修复（var() 不继承致 :root 变量丢失，morning.work 67.45%→28.72%，零回归，已提交）

录制 morning.work 中文文章 fixture（DC-13 首个真实外链页面 fixture，`apps/browser/assets/morning-work/`，含 4 外链 CSS + 2 图片，经 base_dir 加载），与 chromium 800×600 对比 → 初始 diff **67.45%**（页面背景/代码块背景全白、布局塌）。诊断为 **CSS 自定义属性不继承**。

**根因**：CSS 自定义属性是继承属性，但 `gather_custom_properties`（style-system/src/lib.rs）每元素只取自身级联 `--*` 声明，丢弃祖先（`:root`/`html`/`body`）定义的变量 → 后代 `var(--x)` 解析失败（`--x` 不在自身 map）→ 背景回退默认白/颜色丢失。**任何 `:root{--token}` + 子元素 `var(--token)` 的真实页面全部受影响**（现代 CSS 设计系统标配）。诊断探针证实：`--c` 定义在 `.a` 自身时 `var(--c)` 正确，定义在祖先时失败（白）——继承类 bug 须跨元素 DOM 诊断，单元素探针会漏。

**修复**：`gather_custom_properties(cascaded, inherited)` 先继承父自定义属性再自身覆盖再迭代 resolve var()；`compute_styles_recursive` 递归传 `parent_custom`（进入子树前一次性捕获供 sibling 共享）。`compute_element_style`（pub 单元素）签名不变。

**验证**：morning.work fixture diff **67.45%→28.72%**（-185,898 px，页面背景 #f9f7f4 + 代码块背景 + 设计 token 全部正确应用）；reftest **434/490 持平零回归**（同源两侧同变仍匹配；8 类目全持平）；新增 2 单测（test_custom_property_inheritance + override_inherited）；make test 12188/0；clippy/fmt clean。welcome.html 不用 var() 故未受影响。

**意义**：(1) CSS variables 是现代页面基础特性，此 bug 影响面极大——真实中文页面此前背景/颜色全丢；(2) DC-13 真实页面 fixture 轴线（morning.work）捕获了 reftest 490 平台期 + welcome 都覆盖不到的 bug；(3) fixture 证据持久化 `evidence/product-static/morning-work/`。

**剩余 morning.work 28.72%**：最大块=顶部蓝色全宽条（~55k px #607cd2，y=169-241 共 73 行 × 752px 宽）——是 3 个 `.item-tag` `<span>`（Fedora/MacBook/Linux 标签徽章，直接规则非 @media）被**渲染为全宽堆叠块**而非行内小徽章（span UA display=inline 正确，问题在布局层 inline/inline-block 元素被当作 block，属 R109 IFC/inline→block 架构范畴，非单会话修复）；+ fontdue CJK 度量噪声 + hljs 高亮缺失 + @font-face web 字体未加载。**下一轮可独立诊断**：inline `<span>` 带背景的全宽渲染（converter 是否把 inline-block 映射为 block）与 @media min-width 在外链 CSS 的评估一致性。

### R174 — box-shadow blur σ=radius/2 修复（CSS 高斯映射修正，welcome 28.72%→28.08%，零回归，已提交）

DC-13 welcome.html 剩余 ~28% 差距逐带/逐像素定位（throwaway 渲染测试 + layout snapshot + PIL）。**关键结论：welcome.html 剩余差距 96.5% 为 fontdue vs Skia 字体噪声（非 CSS bug）**——色差直方图双峰 delta≈±10（66k 像素，glyph 边缘 AA 差异）+ card-desc 文本换行行数差异致 chromium 卡片高 ~13px。结构布局经 layout snapshot 确认正确（gradient bar y=36、hero/卡片几何、卡片白底（R172 后正确）均正确）。唯一可定位的真实渲染 bug = **box-shadow blur 高斯 σ 映射错误**。

**根因**：CSS 规范 `box-shadow: ox oy blur_radius` 的 blur_radius 对应**高斯标准差 σ = blur_radius / 2**（Chromium 实现）。ZeroWeb 旧实现 `radius = blur_r.ceil()` 直接当三遍 box-blur 半宽 → blur_r=3 时 σ≈3.46（**偏大 2.3 倍**），阴影扩散过远。实测 welcome `.card`（`box-shadow: 0 1px 3px rgba(0,0,0,0.08)`）：chromium 阴影基本不可见（alpha 0.08 经 3px 模糊 <1/255，全白），ZeroWeb 在卡片下方渲染 **12px 可见阴影带**（lum 232-243）。

**修复**（render-foundation/src/cpu/shadow.rs）：`sigma = blur_r*0.5`；连续半宽 `d=(sqrt(4σ²+1)-1)/2`；按 d 小数部分在 3 遍间分配 floor/ceil 半宽（m 遍 ceil、3-m 遍 floor，`m=round((d-r_lo)*3)`）。blur_r=3→σ=1.41（规范 1.5），blur_r=6→σ=3.16（规范 3.0）。

**验证**：welcome.html diff 137,874→134,796（**28.72%→28.08%**，阴影带收紧）；reftest **434/490 持平零回归**（test+ref 同源，blur 修复两侧同变仍匹配；inline 686/686）；make test 全绿；clippy/fmt clean。

**教训**：(1) welcome.html 已无 clean structural CSS bug——剩余差距需升级字体光栅器（fontdue→更接近 Skia）才能显著下降，非单会话范围；**DC-13 杠杆转移至 morning.work（外链 CSS + CJK 真实页）/ wintertc.org（图片子资源）等能暴露未实现 P1 缺口的 fixture**。(2) 像素扫描易误判「垂直偏移」——扫描得「gradient bar y=72」实为蓝色 title 文本，须 layout snapshot 交叉验证。(3) CSS σ=radius/2 同适用于 `filter:blur()`（effects.rs:36-41 仍用 `radius.ceil()` 单遍，σ 偏大）——未修（无测试驱动，遵循精准修改）。(4) 同源 reftest 对保真类修复天然零回归，可用同源 490 验证安全性。

### R173 — 加载 Noto Sans CJK 字体 + 回退链（CJK 字符可渲染，DC-13 能力，零回归，已提交）

DC-13 welcome.html cards 区域剩余 diff 定位到 **CJK 字符完全不渲染**（探针「中文」dark=0）。根因：`create_font_loader`（reftest.rs:1114）只加载 DejaVu/Ahem，无 CJK 字体；主字体缺 CJK 字形时无回退 → 中/日/韩文本全空白。系统有 `/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc`（chromium 用它渲染 CJK）。

**修复**：create_font_loader 加载 NotoSansCJK-Regular.ttc（fontdue 支持 .ttc）并 set_fallback_chain。引擎回退逻辑已就绪（主字体缺字形时回退），此前仅缺 CJK 字体加载。CJK 探针「中文」dark 0→91（渲染）。

**验证**：reftest **434/490 持平零回归**（Ahem/ASCII 用例不受影响）；make test 12186/0；clippy/fmt clean。

**权衡**：welcome.html 像素 diff 26.15%→27.10%（+0.95%）——CJK 现可见但带 fontdue vs Skia 字体度量噪声（已知文本差异），比空白更正确；CJK 重度页面（morning.work 中文文章 fixture）从大量空白→可读，是 DC-13 关键能力。剩余 welcome 度量差需 fontdue CJK 度量调优（与 reftest 文字 fontdue/Skia 噪声同源）。

**意义**：CJK 渲染是产品必备能力（中文页面此前全空白）。fontdue .ttc 支持 + 引擎回退链已验证可用。

### R172 — border-radius 背景在 draw_order 模式被丢弃（paint_background 绕过 add_rounded_rect，真实修复，零回归，已提交）

DC-13 welcome.html cards 区域 50.45% 差距的**第二大主因**（仅次于 R170 box-shadow 实心黑）。`paint_background`（painter/mod.rs:1101）对圆角背景直接 `primitives.rounded_rects.push()`，**绕过 `add_rounded_rect`**（后者才记录 `DrawOp::RoundedRect`）。draw_order 是 R155 起的默认渲染路径，无 DrawOp 的 rounded_rect 被丢弃 → **任何带 border-radius 的元素背景都不绘制**（透显底层背景）。welcome.html 卡片（border-radius:10px）白底消失。

**根因定位过程**：DC-13 cards 区域 y240-480 85-95% diff → no-box-shadow bisect 仍透显 body-bg → 逐变量探针（grid/padding/radius）定位 **border-radius 触发** → block+grid 均复用 → 查 paint_background 圆角路径 → 发现直接 push 绕过 add_rounded_rect（draw_order bypass）。

**修复**：改用 `primitives.add_rounded_rect()` 记录 DrawOp。全仓库仅此一处绕过 add_* 的 push（grep 确认无其他）。

**验证**：border-radius 元素背景探针（was body-bg 透显）→ 正确白底；**welcome.html 差距 50.45%→26.15%**（DC-13 重大进展，本 session 从 51.59%→26.15%）；reftest 434/490 持平零回归（test+ref 同病同愈）；make test 12186/0（+1 rounded_rect draw_order 测试）；clippy/fmt clean。

**意义**：(1) draw_order（R155 基建）的 bypass 类 bug——任何图元若不通过 add_* 方法记录 DrawOp 就会在默认渲染路径丢失；(2) border-radius 是真实页面极常见属性，此 bug 影响面巨大；(3) DC-13 smoke 轴线连续产出 R170/R171/R172 三个真实修复，welcome.html 差距减半——证明产品静态 smoke 能系统捕获 reftest 平台期外的 bug。

### R171 — border/outline/column-rule/text-decor 简写 rgba 带空格丢颜色（同 R170 class，真实修复，零回归，已提交）

R170 修复 box-shadow/text-shadow 后，排查同类 `split_whitespace()+looks_like_color` 模式，发现 4 个简写解析器有相同 bug：`parse_border_shorthand` / `expand_outline` / `expand_column_rule` / `expand_text_decoration`（style-system shorthand/mod.rs）。它们用 `split_whitespace()` 拆碎标准格式 `rgba(255, 0, 0, 0.3)`（逗号后空格）→ `looks_like_color` 命中碎片或颜色退化 currentcolor（→黑）。

**修复**：把 R170 的 `split_shadow_tokens` 重命名为通用 `split_paren_aware_tokens` 并 `pub`（css-parser parse_transform.rs），4 个简写解析器改用之（括号感知，保留 rgba()/hsl()/var() 为单 token）。

**验证**：border/outline 带空格 rgba 探针（was 黑 911/804）→ 正确半透明红/绿；make test 12185/0（+1 border rgba 测试）；reftest 434/490 持平零回归；clippy/fmt clean。

**意义**：rgba-带空格丢颜色/alpha 的 bug class 现已**全量修复**（`looks_like_color` 模式 6 个解析器：box-shadow/text-shadow/border/outline/column-rule/text-decoration）。影响所有用标准带空格 rgba 的真实页面。welcome.html 不受 border 修复影响（用 box-shadow 非 border 简写），其剩余 50.45% 差距为 cards 区域 Phase A（独立后续）。

**方法论**：R170 用 SHDWDBG 定位 box-shadow 后，本轮按「同 class 模式 grep」(looks_like_color) 一次性找出全部 4 个同源 bug——per-feature grep（区别 bbox 扫描/BISECT）是 cluster-finder，R170+R171 是其产出。

### R170 — DC-13 box-shadow/text-shadow rgba 带空格丢失 alpha 致实心黑（真实修复，零回归，已提交）

DC-13 welcome.html smoke 定位到产品可见渲染 bug：`parse_box_shadow`/`parse_text_shadow`（parse_transform.rs）用 `split_whitespace()` 分割值，把标准格式 `rgba(0, 0, 0, 0.08)`（逗号后有空格）拆成碎片，颜色解析失败回退默认实心黑（alpha=255）。

**根因定位过程**：welcome.html 51.59% 差距中 132,793 纯黑像素——bisect 删 box-shadow 后纯黑→0；SHDWDBG 插桩 paint_box_shadow 证 8 个 card/section 阴影 color=rgba(0,0,0,255)（应≈20）；单变量探针排除 grid/`*`/@media/rgba-spaces(count)/count 后，**最终用 SHDWDBG alpha 对比 `.a rgba(0,0,0,0.08)`(alpha 20) vs `.b rgba(0, 0, 0, 0.08)`(alpha 255) 定位到空格触发**——split_whitespace 拆碎 rgba。

**修复**：新增括号感知分割 `split_shadow_tokens`（不在 `()` 内分割，保留 `rgba()/hsl()/var()` 为单 token），box-shadow + 同 class 的 text-shadow 均改用之。

**验证**：
- welcome.html 纯黑 **132793→0**（DC-13 实心黑主因消除）。
- 同源上游 reftest **434/490 持平零回归**（0 用例用 box-shadow rgba）。
- make test **12184/0**（+2 单测：box_shadow/text_shadow rgba-spaces 保 alpha）。
- clippy/fmt clean。
- 注：welcome.html 整体 diff 仅 51.59%→50.45%——box-shadow 实心黑消除后暴露**其他**渲染差异（cards 区域，Phase A inline-block/IFC ownership 同源），独立后续。

**意义**：(1) DC-13 首个真实修复，产品可见渲染 bug；(2) box-shadow/text-shadow 用标准带空格 rgba 在**任何真实页面**都会触发实心黑——影响面远超 welcome.html；(3) 印证 DC-13 轴线（产品静态 smoke）能捕获 reftest 平台期无法暴露的 bug——welcome.html 非 reftest，其 box-shadow 解析退化不在 56 失败用例中。

### R169 — Phase A large-font 死锁实证复核（放宽 R84 多行守卫 net -1，已回退，无提交）

按 Phase A 推进方向（解锁 large-font 集群 ~5 自源失败：font-051/ifc-008/009/011/empty-inline-002），本轮**实证复核**了 R125 记录的死锁——放宽 `compute_final_inline_layouts` 的 R84 单行守卫（engine.rs:1690），允许「多行纯 Ahem」存储 `inline_layout`（保留纯 Ahem 过滤），测全量。

**实测结果（net -1，已回退）**：434→**433/490**。
- **large-font 集群仍全失败**：font-051 8.19%（持平）、ifc-008 8.18→4.17%（改善但未过阈）、ifc-009 4.17%、ifc-011 11.24%（持平）、empty-inline-002 29.32%（持平）。**仅存 inline_layout 不够**——这些用例还需 `text_node_font_sizes` 存储（store_font_sizes_from_ifc，remeasure 路径），而该存储正是 R125 三路 net-negative 的回归源。
- **multicol 40→39（-1）**：multicol-fill-auto-001 回归（与 R125 一致——多行 inline_layout 存储改变其 paint use_stored 渲染）。

**结论**：R125/R158 记录的 Phase A large-font 死锁经本轮独立实验**再次确证**——R84 多行守卫是 load-bearing（放宽即 net -1 且不解锁集群）。large-font 修复需同时 (a) 存 inline_layout（多行）+ (b) 存 text_node_font_sizes（real font_size），而 (b) 回归 multicol-fill-auto（依赖 16px 错误默认值才通过）。**不要再单独放宽 R84 守卫或单点补存 font_size 重试**（R125 三路 + R169 共 4 轮证否）。真正解锁需 Phase A paint IFC 三路径（compute_final/remeasure/paint）font_size 解析统一 + multicol-fill-auto 在真实 font_size 下也正确（R125 前置）——属多轮架构。

**方法论**：对「已知死锁」做独立实验复核（放宽守卫→全量测→net 效果），用实测确认而非仅据记忆接力。本轮 net -1 即刻回退，434/490 持平。印证上轮结论：剩余 16 真 bug 候选 + 56 自源失败全部 Phase A 结构性 / 基础设施 / 特性，单会话零回归不可推进。

### R168 — table height-as-minimum 修复（table-grid-item-dynamic-004 chromium 11%→2.98%，真实修复，零回归，已提交）

d16bb8e 方向（优化 chromium Oracle 一致率）下第二个真实修复。修复 table 的 `height` 属性被完全忽略的真实 bug —— 被「同源假通过」掩盖的缺口（table-grid-item-dynamic-004 同源 0.00% 通过，但 chromium 差 11.12%；18 真 bug 候选之一）。

**根因**：`apply_table_size_constraints`（table.rs）把表格高度设为 `intrinsic_height`（行内容驱动）+ min/max-height 约束，但**完全忽略 CSS `height` 属性**（Px 和 % 均不读取，只赋值 `final_height = intrinsic_height`）。CSS 2.1 §17.5.3 规定 table 的 `height` 是内容高度的「下限」（min 语义）：表格至少这么高，内容更高则增长。ZeroWeb 此前对任何带 `height`（如 `height:200px`、`height:100%`）的表格都按内容尺寸渲染，与 chromium 大幅不一致。

**修复**：在 `clamp_percentage_max_height`（engine.rs:1378，自上而下 pass，已有 cb_content_height「明确高度」语义）新增 §1.5 段——对 `display:table/inline-table` 盒，把 `style.height`（Px 直接 / 百分比按 cb_content_height 解析）解析为内容高度下限，与已计算 `content_height` 取 max 后回写 `content_height`+`height`（+padding_border）。复用 R119 的 CB 明确性语义：百分比仅当 CB 明确时解析，否则忽略（§10.5）。在此 pass 处理而非 apply_table_size_constraints，是因为后者无 CB 上下文（LayoutBox 无父指针），而本 pass 已自上而下传递明确高度。

**验证（chromium Oracle + 严谨禁用对照）**：
- table-grid-item-dynamic-004: table 500×116（intrinsic，height 被忽略）→ 500×200（fixed：100 内容 + 100 padding-top），chromium 500×222。chromium 差距 **11.12% → 2.98%**（< 5% 布局阈值 → chromium Oracle 实际通过）。
- **禁用对照**：临时把增长条件改 `false`，004 回到 116（unfixed）；启用→200。确认修复是 004 改善的**唯一原因**，排除巧合。
- 同源 upstream reftest **434/490 持平，set-diff 零翻转**（004 同源本就 0.00% 通过=polluted case，修复不改同源通过数，只降 chromium 差距 = DC-14 真实改进）。
- 18 个含 table `height` 用例**零回归**：min-height-table 0.00%、max-height-table 0.02%、subpixel-table-width-001 0.00%、multicol-fill-001 0.16%、border-spacing-vrl-002 4.17%、min-max-size-table-content-box 36.34%、table-cell-width-0 29.99%、baseline-vertical 16.66%、fixed-table-layout-… 11.20% 全部 diff 持平（min/max-height 语义不变，因取 max 只增不减且多数用例 content≥指定高）。
- make test **12182 passed / 0 failed**；clippy 零警告；fmt clean。
- +2 单测：`test_table_height_as_minimum_px`（engine.compute 验 Px 分支）+ `test_table_percentage_height_resolves_as_minimum`（直接调用 clamp 验百分比分支——engine.compute 路径中 table 匿名包装盒打断直接父子 CB 传递致百分比不触发，故直接测函数；百分比端到端正确性由 004 reftest 覆盖）。

**chromium Oracle 全量交叉验证（DC-14 交替推进 step 2）**：重跑 `cross-validate.py`（`evidence/cross-validate-full-2026-06-16-r168.txt`）+ `analyze-pollution.py`（`evidence/analyze-pollution-2026-06-16-r168.txt`）。污染总数 153→154（基本持平，污染率 46.5%→46.7%——R168 把 004 的 chr 差从 11% 降到 2.98% 但仍 >1% oracle 阈，故污染计数未翻转）。但**真 bug 候选（>5% 布局目录）从 18 降到 16**：R165 移除 html-display-table（33%→<5%），**R168 移除 table-grid-item-dynamic-004（11.12%→2.98%）**，并**把 003 从 29.18% 降到 23.79%**（height 修复部分生效，剩余=宽度 grid-stretch）。印证 R165+R168 是 18 候选中仅有的可单点修复真实缺口，剩余 16 项确认结构性/基础设施/特性（grid-stretch / flex baseline·collapse / multicol-abspos / border-collapse×vrl / dialog·iframe 基础设施 / font-fallback 特性）。

**剩余**：
- **003（29%）同源通过但 chromium 仍高**——差异主因是 table 作为 grid item 的**宽度 shrink-to-fit vs grid-stretch**（ZW table 327 shrink-to-fit，chromium 800 拉伸到 grid 轨道），与高度修复独立；003 仅 height 修复后 table 仍 327 宽，与 chromium 800 宽差距主导。需 grid-item-table 拉伸单独处理（grid item width:auto 应拉伸到 track，但 table shrink-to-fit 覆盖——类似 R138 但需 grid 感知）。
- **004 剩余 2.98%** = chromium intrinsic 行高 ~122 vs ZW ~100（cell padding/border-spacing 细节），独立小差异。

**方法论**：chromium Oracle 像素对比（REFTEST_DUMP ZeroWeb vs /tmp/oracle-shots-all）定位「同源通过但 chr 不一致」的真 bug → 探针（TBLH_DBG）定位 `style.height` 在 apply_table_size_constraints 被忽略 → 严谨禁用对照（条件改 `false`）确认修复是改善唯一原因。**印证 d16bb8e 转向正确**：18 真 bug 候选中仍有可单点修复的真实缺口（R165 margin:auto、R168 table height），非全部结构性；下一轮最高杠杆=重审 R111/R130/R138/R124 等「同源修但 chr 仍高」用例的分歧点，或 003 的 grid-item-table 拉伸。

### R166 — 真 bug 候选再甄别（table_grid 非单点 + float-006 paint 层，诊断，持平，无提交）

R165 之后继续按 chromium Oracle 真实缺口推进，逐个甄别几个 self+chr 均高（=真 bug 非 REF 产物）的候选：

- **table_grid_size_col_colspan (chr 52%)**：根因 = build_grid 的 col_count 仅从单元格 colspan 计算，**忽略 `<col>` 元素**（3 个 col 算 1 列）；且 fixed-layout auto-width 错误扩展到容器。实现修复（col_count 计 col + 收集 col_widths + 去除 fixed 扩展）后**内部全部正确**（col_widths=[50,50,50]、table 宽 150、cell w=50），但**渲染仍全宽/重叠**——直接 `<td>`+`<col>`+匿名行+fixed 结构有多个叠加问题（含表格垂直堆叠），非单点修复，已回退。
- **flexbox-collapsed-item-horiz-001 (chr 20.5%)**：flex visibility:collapse 内部（R111 territory），多 cell 差异，flex 内部复杂。
- **multicol-breaking-005 (self 21.7%/chr 22.3%)**：嵌套 multicol 平衡（外 3 列 × 内 2 列 balance），最深水区。
- **float-006 (self 7.5%/chr 8.4%，已精确定位=需 2 处协同修复)**：零高度空 float 测试。**实测（修正 R166 初判）**：绿色 abspos `x=0 w=224` **渲染正确**（X[8..231] 与 chromium 一致），红色 float 在 `x=288`（X[296..519] 可见，应被绿覆盖）。需两处协同修复：(1) **layout**：零高度（margin-box）空 float 不应占据水平空间（engine.rs adjust_float_positions_with_context `left_used_width += child_outer_width` 须加 `child_outer_height > 0` 守卫，使红色 float 回到 x=0）；(2) **paint**：z-index:auto 的 abspos 须画在 float 之上（CSS App. E step 6，painter/mod.rs child_paint_sort_key 现按 (1,0)=step3 画 positioned 即在 float(2,0) 之前，仅做 (1) 会使红覆盖绿）。**(2) 实测 net -1**：把 z-index:auto positioned 改 (3,0) 画在 float 之后 → regress `css-flexbox/flex-item-position-relative-001`（嵌套 positioned 用例，原 (1,0) 排序是为其修复），fix 0。CSS App.E 的「z-index:auto after floats」与 painter 的嵌套 positioned 近似**结构性冲突**，需完整 stacking-context 实现（CSS App.E 全量）才能两全，属多轮架构改造。float-006 非单点、非双点，需架构层 stacking-context。**精确机制（R167 复查）**：flex-item-position-relative-001 的 green abspos 嵌套在**正常流** `#flex` 内，flat 按容器排序使其随 `#flex`(step3) 一起画、早于兄弟 red abspos(step6) → 红覆盖绿。**z-index:auto positioned 嵌套在正常流中必须「逃逸」到祖先 stacking-context 的 step6**——flat sort 结构性做不到，须按 stacking-context 收集所有 positioned 后代再按 App.E 7 步渲染。

**结论**：除 R165（margin:auto，单点）外，本轮再甄别的真 bug 候选全部结构性/多子系统/需多修复协同。

### R165 — margin:auto 水平居中修复（html-display-table chromium 33%→2.63%，真实修复，零回归，已提交）

**d16bb8e 方向（优化 chromium Oracle 一致率）下首个真实修复**。修复 `margin:auto` 水平居中缺失——这是一个被同源假通过「掩盖」的真实渲染 bug（html-display-table 同源 0.00% 通过，但 chromium 差 33.09%）。

**根因（两处）**：
1. **根元素**（如 `<html style="width:280px;margin:auto">`）：CSS §10.3.3 规定 width<视口 + margin-left/right 均 auto 应水平居中。taffy 对**嵌套** block 正确居中（实测 nested div margin:auto → x=260 ✓），但对**根节点**不应用（根无父级提供居中上下文，左对齐 0）。实测 root_x=0（应 260）。
2. **display:table 容器**（width:auto 收缩后）：R138 的 `shrink_table_to_block_content` 收缩了宽度但未重新居中——taffy 在收缩前（table 填满 CB）已把 auto margin 解析为 0，收缩后未补。

**修复（两处补丁）**：
1. `compute()`（engine.rs）根居中补丁：水平书写模式下，若根 margin-left/right 均 auto 且边框盒宽度 < 视口，把 root.x 居中（跳过 display:table 避免双重居中）。
2. `shrink_table_to_block_content`（table_shrink.rs）收缩后补居中：若 margin-left/right 均 auto 且新宽度 < 旧宽度，table.x 居中（子元素 x 相对 table 内容盒，paint 累积偏移 offset_x+box.x，改 table.x 即整树居中，无需逐子元素平移）。

**验证（chromium Oracle /tmp/oracle-shots-all + cross-validate.py）**：
- html-display-table: chromium 差距 **33.09% → 2.63%**（同源 0.00% 持续通过，test 与 ref 均居中）。
- 同源 upstream reftest **434/490 持平，set-diff 零翻转**（stash 前后 56 失败完全一致）。
- make test **12180 passed/0 failed**（59 ignored = 真实网站）；smoke reftest 686/686；clippy 零警告；fmt clean。
- +2 单测 `test_root_block_margin_auto_centers` / `test_display_table_margin_auto_centers`。

**方法论**：用 chromium Oracle 像素对比（`REFTEST_DUMP` ZeroWeb 渲染 vs `/tmp/oracle-shots-all`）定位「同源通过但 chromium 不一致」的真实缺口；用根 vs 嵌套 margin:auto 探针界定 taffy 行为（嵌套正确/根缺失），把修复精准限制在 taffy 缺口（根 + 收缩后 table），避免触碰 taffy 已正确的嵌套 block 居中路径（零回归保证）。

**意义**：印证 d16bb8e 转向的正确性——html-display-table 是 R138「同源修」但 chromium 仍差 33% 的典型案例；本修复根因（margin:auto 居中缺失）使其真正对齐 chromium，而非匹配怪异同源 REF。剩余 18 候选中 R124/R130/R138/R111 等「同源修但 chr 仍高」的用例是下一轮最高杠杆（代码位置已知，只需找与 chromium 的分歧点）。

### R164b — 18 真 bug 候选逐项甄别：5 项已查全部结构性（诊断，持平，无提交）

按 d16bb8e 方向（优化目标=chromium Oracle 一致率）逐项甄别 `evidence/analyze-pollution-2026-06-16.txt` 的 18 真 bug 候选。用 `REFTEST_DUMP` ZeroWeb 渲染 + `/tmp/oracle-shots-all` chromium Oracle 像素对比定位差异。**已查 5 项全部确认为结构性多轮**（非单会话 clean win），逐项根因：

1. **backdrop-inherit-rendered (47.5%)**：需 `<dialog>` + `::backdrop` + `showModal()` JS + CSS 变量继承 + `inset` 基础设施，ZeroWeb 无 dialog/modal/backdrop 渲染管线——基础设施级，非渲染 bug。
2. **baseline-block-with-overflow-001 (45%)**：`inline-block` + `overflow:hidden` 基线=底边（CSS §10.8.1）+ 5 section 复合，inline-block 基线×overflow 交互，影响面广。
3. **multicol-contained-absolute (16%, R124 同源「修」但 chr 仍 16%)**：ZeroWeb 渲染 abspos 绿块**仅 1 列** X[8,399]，chromium **2 列** X[8,391]+[408,791]——multicol+abspos containing-block 语义（abspos 是否跨列/分列碎片化），CSS multicol 规范深水区，非 R124 的 overflow 裁剪层面。
4. **collapsed-border-vertical-rtl-overflow (6.1%)** ×3：`border-collapse` 冲突解析 × vertical-rl × direction:rtl × will-change overflow，三重复合，border-collapse 冲突已是 CSS 最难区之一。
5. **position-absolute-semi-replaced-stretch-input (23%)**：测试名暗示 abspos stretch，**实测 ZeroWeb 已正确 stretch**（窄 CB 绿框 147px、宽 347px 与 chromium 一致）。真实差异=**inline-block CB 的行距/基线/空白**：ZeroWeb 行距 ~100px、框间 gap ~7px，chromium 行距 ~116px、gap ~23px——inline formatting 基线×line-height×whitespace 子问题（非 abspos）。

**模式印证**（与 419c5a8 一致）：rally 曾「修」的用例同源通过但 chr 仍高——R124(multicol-contained-absolute)、R130(flexbox-baseline-align-self-baseline-horiz=17.65%)、R138(html-display-table=33%)、R111(flexbox-collapsed-item-horiz=20.5%)——证实「刷同源通过率」跑偏，这些「修」匹配了怪异同源 REF 而非 chromium。

**剩余 13 候选**（未查，按可修性排序）：position-absolute-semi-replaced-stretch-other(15%,同 #5 inline-block)、table-grid-item-dynamic-003/004(29/11%,grid+table)、table_grid_size_col_colspan(52%,table colspan)、stretch-grid-item-button-overflow(8%,grid+button)、font-family-013(6.65%,字体噪声嫌疑)、iframe-in-block-in-inline/wrapped-span(9.75%,iframe infra)、flexbox-collapsed-item-horiz-001(20.5%,R111 重审)、flexbox-baseline-align-self-baseline-horiz-001(17.65%,R130 重审)。**最高杠杆=重审 R111/R130/R138/R124 的「同源修」为何不匹配 chromium**（代码位置已知，只需找分歧点）。

### R164 — 否决 vrl-004/008 R114b x 轴 clearance：正确 CSS 与同源水平 REF 结构性不可对齐（诊断，持平，已回退）

**434/490 持平（无提交，实验代码已回退，工作区清洁）**。本轮以**实验证伪**了 R114b「对 vertical-rl/lr 容器实现 x 轴 float 定位 + clearance」这条接力路径——这是上轮 CONTINUE 指定的下一步，也是「PNG bundle 双阻塞」的理论解。

**实验**：新增 `adjust_float_positions_vertical`（~130 行），对 vertical-rl/lr 容器把块流从 taffy 的 Y 轴重排到物理 X 轴（block-start=右/左），复制 float 定位 + clearance（正/负/零）+ margin 折叠语义；入口按 writing_mode 分流（水平路径字节不变）。在 `adjust_float_positions_with_context` 顶部加守卫调用。

**结果（已回退）**：4 个 vrl clearance 用例**全部变差**——vrl-002 2.72→11.15%、vrl-004 7.09→14.05%、vrl-006 4.38→10.33%、vrl-008 6.42→16.83%（4/4 FAIL，含 002/006 从 PASS 翻 FAIL）。即 **实现正确 vertical-rl CSS 反而使全部用例失败更严重**。

**根因（结构性，非实现 bug）**：这些 reftest 的 **reference 是水平渲染**（`<img>` 绿块在**左侧** X[8,87]）。正确 vertical-rl 的 block-start 在**右侧**（X=高值），块流右→左；当前「错误」的 Y 轴堆叠**恰好**把绿块留在左侧（X[8,87]），偶然对齐水平 REF 的左侧绿。改为正确 X 轴后绿块移到右侧（实测 X[672,791]），与水平 REF 左侧绿结构性背离。

**chromium Oracle 印证同源 REF 比 chromium 更怪异**（DC-14 全量交叉验证 b18b7ae / `evidence/cross-validate-full-2026-06-16.txt`）：
- vrl-004 同源 7.09% vs **chromium 仅 5.08%**——ZeroWeb 渲染离 chromium 更近，离自己的怪异 REF 更远。
- vrl-008 同源 6.42% vs chromium 3.15%。
- **font-051 同源 8.19% vs chromium 仅 1.62%**——ZeroWeb 渲染几乎完美匹配 chromium，但「失败」于自己的怪异同源 REF（large-font 100px 文本，REF 侧 16px 默认值退化）。

**结论**：
1. **434/490 即诚实 DC-14 基线，无需恢复 436**。436 含 garbled-image 假通过（R163 已证）；vrl-004/008 的「双阻塞」是同源 REF 怪异产物，非真实渲染 bug。修它们 = 匹配怪异 REF，非对齐标准。
2. **不要再以「正确 vertical-rl CSS」重试 vrl-004/008**（本轮 + R133/R153/R154 共 4 轮一致证伪）。
3. **同源失败用例中相当部分是 REF 怪异产物**（vrl-004/008、font-051 等），同源通过率不是可信指标（DC-14）。后续应以 **chromium Oracle 交叉验证** 识别真实 bug（POLLUTED=同源通过但 chromium 不一致=隐藏真实 bug）。
4. **真实最大杠杆**（chromium Oracle 视角）：fontdue vs chromium 字体度量噪声是 46.5% 污染主因（b18b7ae 注）；次为结构性 multicol/table/flex-baseline。均为多轮大改。

**方法论**：对「已知诊断」做独立实验复核——R133/R153/R154 推断「正确 vertical-rl CSS 应改善 vrl-004/008」，但均未实际实现验证；本轮实际实现 ~130 行后实测全负，证伪推断。**架构性推断需实验落地，不能据推断结论接力多轮**。

### R163 — PNG 正确 RGBA 转换默认启用（DC-14 anti-false-pass，436→434 真实）（已提交）

**变更（已提交）**：`load_png_file`（reftest.rs）把「EXPAND+正确 RGBA 转换」从 env-gated（ZERO_PNG_EXPAND=1）改为**生产默认**——所有非 RGBA PNG（palette/grayscale/RGB）正确展开为 RGBA8（修正 alpha=0 退化=图像类 reftest 假通过根因）。`ZERO_PNG_EXPAND=0` 作为逃生舱回退旧的「按 RGBA 直读」garbled 路径（诊断/回归对比）。

**意义**：直接落实 **DC-14 anti-false-pass**——此前所有 palette/RGB PNG（support/swatch-*.png、pass-cdts 等）因缓冲错位渲染成透明/乱码（alpha=0），使图像类 reftest 的 reference 退化，产生「garbled-test vs garbled-ref 凑合匹配」的假通过。正确渲染后 reference 真实可见，消除该类假通过。与项目并行的 chromium 独立 Oracle 基建（72764a0）+ DC-14 门禁（c4d5863）方向一致。

**实测影响**：默认 reftest **436→434/490（net -2）**——仅 vrl-004(7.09%)/vrl-008(6.42%) 翻转为 FAIL（writing-modes 55→53），其余 488 用例零翻转（image 类用例要么本就 RGBA，要么 garbled 双方凑合匹配在正确渲染后仍匹配）。这两个是须修复的**真实 vertical-rl clearance 失败**（R114b），非应隐藏的假通过。make test 全绿（45 result 行 0 failed）。

**为何现在默认启用**：DC-14 明确「不满足 anti-false-pass 的通过率不构成达标证据」——436 含 garbled-image 假通过属非合规。正确的图像渲染是 anti-false-pass 的前提。逃生舱 ZERO_PNG_EXPAND=0 保留旧 436 供对比。

### R161 — PNG EXPAND 诊断门控 + 正确 RGBA 转换（修复 alpha=0 退化，bundle 真实 net -2）（已提交，默认净中性）

**变更（已提交，默认净中性零回归）**：`load_png_file`（reftest.rs）新增 `ZERO_PNG_EXPAND` 环境变量门控——启用后 `set_transformations(EXPAND|STRIP_16)` 并用 `output_buffer_size`+`convert_png_buffer_to_rgba` 按 `OutputInfo.color_type` 正确转换为 RGBA8。**关键 bug 修复**：EXPAND 不保证输出 RGBA（palette 无 tRNS / RGB 输入 → 输出 RGB=3 字节/像素），原先按 4 字节解释会错位→ alpha=0 退化透明（swatch-green.png 实测 [0,128,0,0]）。正确转换后 swatch-green → [0,128,0,255]。`convert_png_buffer_to_rgba` 处理 Rgb(补 alpha=255)/Grayscale(复制+255)/GrayscaleAlpha/Rgba(原样)。

**bundle 真实测量（纠正本会话早先 net -1 误判）**：`ZERO_PNG_EXPAND=1` 正确转换后全量实测 **434/490（net -2）**——**vrl-004 (7.09%) 与 vrl-008 (6.42%) 双双 FAIL**（R156 的 net -2 测量正确）。本会话早先用「错位转换」测得的 net -1 / vrl-008 4.06% 通过是 **bug 假象**（错位 buffer 产生的乱码 ref 凑巧接近）。clear-clearance-calc-001~005 全 0.00% 通过。**bundle 唯一阻塞仍是 vrl-004/008 双（R114b x 轴 clearance）**，未收窄。

**vrl-004 精确分析（EXPAND 后）**：REF image-based（2 swatch-green 60×100+20×100）；TEST 三 green 块 vertical-rl 下因 clearance 代码全 y 轴(engine.rs:2510)错位→红背景暴露。**这是 R114b x 轴 clearance 工作**（~150 行参数化块轴），对 59 水平 floats-clear 零风险（严格 writing_mode 门控）但高实现复杂度，单会话不可安全完成。

**解锁路径**：修 vrl-004/008（R114b x 轴 clearance）→ bundle net 0 → 可默认启用 EXPAND（消除 IMG-REF 假通过=DC-14 anti-false-pass，让图像 multicol REF 可见从而可正确评估/修复分布）。当前 EXPAND 默认关闭保 436/490；诊断用 `ZERO_PNG_EXPAND=1`（→434 真实）。

### R158 — large-font 死锁机制精确定位 + 失败聚类再分类（诊断，持平，工作区清洁）

**436/490 持平（make reftest-upstream 实测确认 436/490=89.0%，54 失败）**。本轮独立复核全 54 失败，重点深挖 large-font 死锁与若干「疑似 discrete bug」候选，全部确认为已知结构性聚类。新/精化的理解（区别既往轮次）：

**(1) large-font 死锁机制现已精确定位（比 R125「三条路径」更精确）**。完整链条：large-font 文本（font-051/ifc-008/009/011/empty-inline-002 的 100px 内容）位于 **taffy 已测量的 height:auto 子容器**（如 ifc-008 的 `#div1>div div{color:green}` 子 div，taffy 给了正确 content_height>1.0）。该子容器被两条存储路径同时跳过：(a) `remeasure_inline_only_containers` 的 `content_height < 1.0` 守卫（engine.rs:2883 `needs_dom_text_remeasure`）排除 taffy 已测量的块——守卫注释明确这是为避免 font-feature/multicol-fill-auto/abspos 回归（R105）；(b) `compute_final_inline_layouts` 的 R84 守卫（engine.rs:1628 `lines.len()>1 || !is_pure_ahem`）对多行/非纯 Ahem 提前 return，不存储 inline_layout 也不存 font_size。两条路径都跳过 → `text_node_font_sizes` 为空 → paint IFC（painter/text.rs:912 传 `&HashMap::new()` 空 styles + 空 override）按默认 16px 解析 → 100px 渲染成 16px。**关键冲突（R125 本质）**：multicol-fill-auto-001 当前**仅因部分文本节点的 16px 错误默认值才通过**；补存正确 font_size（即使 `or_insert` 不覆盖、仅新增条目）会改变其 paint IFC override 集合 → 文本尺寸变化 → 失败。故 large-font 修复必须**先**让 multicol-fill-auto 在真实 font_size 下也正确（或查清其 ref 为何依赖 16px），否则任一补存路径都 net-negative。**不要再从 compute_final/remeasure 单点补存 font_size 重试**（本轮 compute_final 显式高度容器补存实验=死代码已回退——显式高度 `#div1` 无直接文本只有块子元素，compute_final 在 `!has_text_children` 处早返回根本到不了补存点）。

**(2) font-051 重新定性为 large-font（非 font 简写解析 bug）**。`span{font:serif}`（裸 family 无 size）经 `expand_font`（shorthand/mod.rs:1572）正确判为无效返回 `vec![]`（声明丢弃），span 继承 div 的 `font:100px/1 Ahem`——解析正确。8.19% 差异来自继承的 100px 经上述 paint IFC 死锁渲染成 16px，与 font 简写无关。**纠正潜在误判**：勿再从 css-parser font 简写验证角度修 font-051。

**(3) min-max-size-table-content-box (36.34%) 重新定性为 inline-block ownership（非 table sizing bug）**。PIL 像素分析：TEST 的 7 个 `<table>` 正确 shrink-to-fit（窄蓝带 w=19/59/85 堆叠），但 REF 的 `.table{display:inline-block}` div 渲染成全宽（蓝带 w=793）。根因=converter 把 InlineBlock 映射为 taffy `Display::Block`（converter/mod.rs:266），作为 block 子元素被 taffy 拉伸到容器全宽；`adjust_inline_block_positions`（engine.rs:684）的 `ib_sizes` 直接用 taffy 的 `child.content_width`（全宽）作 IFC 尺寸回填，未做 shrink-to-fit。grid 内的 inline-block（受 grid track 约束）则正确收缩（w=11）。**inline-block width:auto 的 shrink-to-fit 需测量子树 max-content**（非读子元素 box width——子元素也被 taffy 拉伸），属 DC「Inline formatting 所有权分裂」P1 架构，同 css-flexbox-row/block-in-inline/large-font 的 paint↔layout IFC 双路径统一（Phase A）。

**(4) 其余复核候选全部印证结构性**：background-attachment-applies-to-001 (29.92%) = image-based 退化参考区（support/blue96x96.png + swatch-blue.png，受 PNG bundle vrl-004/008 阻塞）；baseline-007/008 (1.04/1.46%) = multicol baseline-export（flex+column-span / inline-block+column-fill 复合）；ifc-011 (11.24%) = image-based + vertical-align:top 定位；multicol-count-computed-003/004 (2.06/2.50%) = image-based + 列分布精度（R131 碎片化）；clear-float-003 (3.20%) = clear:right + 负 margin-top + margin collapsing（R114b 负 clearance）；collapsing-001 (1.68%) = R157 协调（paint IFC 文本分布 vs multicol.rs 块分配不协调）；border-padding-bleed-001 (2.40%) = inline padding/border 跨行 bleed（IFC paint）；float-nowrap-hyphen-rewind-1 (2.89%) = white-space:pre + hyphens:auto + 负 margin float + 窄容器（crbug 1499290 极端 case）。

**结论**：436/490 经本轮全量 54 失败独立再核 + 1 次 compute_final 显式高度补存实验（死代码已回退）后**再次确证为单会话不可推进**。剩余 54 失败按精化聚类=multicol 碎片化/协调(17) + IFC 双路径/large-font/inline-block ownership(8+1) + flex 基线合成/intrinsic sizing(10+3) + 垂直书写模式轴(4) + 表格深层(4) + image-based 退化参考区(vrl-004/008 阻塞)。最高杠杆=Phase A IFC 统一，但其前置死锁（multicol-fill-auto 依赖 16px bug）需先解。单会话预期 +0。

### R155 — draw_order 默认启用（满足 DC-10，净中性零回归，已提交）

**变更（已提交，零回归）**：`render_full_scene`（cpu/mod.rs:70-82）把 draw_order 从 env-gated（`ZERO_DRAW_ORDER=1` 启用）改为**生产默认**——draw_order 非空时默认按插入序渲染（满足 CSS painting order），`ZERO_DRAW_ORDER=0` 作为逃生舱回退类型分桶（旧行为，诊断/回归对比）。draw_order 为空（旧代码路径未填充）自动回退。

**意义**：直接满足 **DC-10「图元渲染顺序遵循 CSS painting order（background→borders→content→outline）」**。此前的类型分桶（所有 images 画在所有 fills 之后）违反 painting order，导致父背景图覆盖子内容——这是产品页（DC-13，如 WinterTC 首页 Logo、morning.work 文章图）渲染缺陷的根因之一。默认启用 draw_order 后，背景图正确画在子内容之下。

**零回归验证**：
- upstream reftest **436/490 持平**（draw_order 与类型分桶在该基准下输出一致——reftest 无「父背景图覆盖子内容」case，故不可区分）
- make reftest smoke **686/686**
- make test **12178 passed/0 failed**
- clippy 零警告，fmt clean

**为何 reftest 不区分**：490 上游 reftest 无「父元素 background-image:url() + 子内容需可见」的非退化 case（含此结构的 abs-pos-non-replaced-vrl 系列是退化参考，PNG fix 前背景退化透明=问题不可见）。draw_order 的正确性收益主要体现在产品页（DC-13）和未来含真实背景图的用例。PNG fix 后（vrl-004/008 修复时）draw_order 使 clear-clearance-calc-001/002/003 真通过（R152 已验证）。

**逃生舱**：`ZERO_DRAW_ORDER=0` 回退类型分桶。draw_order 基础设施（R149）+ cull 重建（R152）+ 默认启用（R155）三步完成 DC-10。


### R154 — vertical-rl clearance skip-guard 实验（诊断，持平，已回退，确认 R114b 多会话）

**背景**：R153 探针发现 vertical-rl 容器内 float/clear 子元素被 y 轴后处理移位（简单块堆叠探针证实 taffy 本身正确按 x 堆叠：a/b/c 在 x=0/50/100）。假设：vertical-rl 模式跳过 y 轴 float/clearance 后处理（taffy 已正确按 x 定位）即可修复 vrl-004/008。

**实验**：`adjust_float_positions_with_context` 入口加 `writing_mode` 守卫——VerticalRl/VerticalLr 直接 return（水平模式 HorizontalTb 完全不受影响，59 个 floats-clear 通过用例零风险）。

**结果（已回退）**：436→**435/490（net -1）**。两测试对 clearance 需求**相反**：
- vrl-004 改善 3.33%→2.08%（仍假通过，但更接近 ref）
- **vrl-008 恶化 2.08%→14.42%（假通过→真失败）**

**结论**：简单跳过不可行——vrl-004 不需 clearance（跳过=正确），vrl-008 需 clearance（跳过=错误）。两测试结构差异（vrl-008 测试注释明确：「clearance + margin-right of clearing-left = 50px，clearance = 50-75 = -25px」负 clearance 边缘 case）。vertical-rl clearance 需**真正 x 轴实现**（按 writing_mode 选块轴：HorizontalTb→y 现逻辑，VerticalRl/Lr→x 新逻辑，含负 clearance、margin 折叠边缘 case），= R114b territory ~150 行参数化，对 59 个 floats-clear 高回归风险 + vrl 退化参考无法独立验证，**单会话不可安全推进**。已回退，436/490 持平。

**方法论**：write_guard+skip 是验证「bug 是否可受守卫隔离」的标准快速实验——零风险（守卫只影响 vertical 模式，水平模式字节不变），单次 reftest 即可判 net 效果。本轮 net -1 即刻回退，确认需完整实现。


### R153 — vertical-rl clearance vrl-004 几何探针定位（诊断，持平，已回退探针，工作区清洁）

**背景**：R152 PNG bundle 实测 net -2，唯一剩余阻塞=clearance-calculations-vrl-004/008（baseline 假通过 3.33/2.08%，PNG fix 后暴露 7.09/6.42%）。本轮探针精确定位 vertical-rl clearance 几何。

**探针**：复刻 vrl-004 结构（`writing-mode:vertical-rl` 容器 + preceding-sibling[mg-left:4em] + floated-left[float:left,w:2em] + clearing-left[clear:left,mg-right:3em]，均 height:5em=100px），layout tree dump 元素坐标。

**几何发现**：vertical-rl 容器(parent w=160=块轴)内子元素**按 y（inline 轴）堆叠**——prec(x=0,y=0)、float(x=0,y=100)、clear(x=60,y=200)。CSS writing-modes §7.1：vertical-rl 块轴=水平(x)，子块应按 x 从右到左堆叠，clear:left 应在 x 轴 clearance（推到 float 块轴之后）。当前 clearance 代码（engine.rs:2520-2555 `adjust_float_positions_with_context`）**全按 y 轴**计算（hypothetical_y、active_left_float_bottom 等），不感知容器 writing_mode，故 vertical-rl 下 clear/float 在错误轴定位。

**为何不安全推进**：这是 R114b territory（~150 行 clearance 轴参数化），对 59 个通过 floats-clear 测试**高回归风险**——这些测试全是水平书写模式，现有 y 轴 clearance 代码对它们正确；参数化引入 axis 判断分支需逐用例验证。且 vrl-004/008 是退化参考（PNG 双方退化），baseline 假通过（3.33/2.08%<5%），无法独立验证修复正确性（需 PNG fix 后 ref 可见）。单会话强推 net 风险高（可能回归 >2 个 floats-clear 通过用例）。

**fix 入口指引（多会话）**：`adjust_float_positions_with_context`（engine.rs:2180）+ clearance 段（2520-2555）参数化——按 `box_node.writing_mode` 选择块轴（HorizontalTb→y 轴现有逻辑，VerticalRl/Lr→x 轴新逻辑）。R133 已建实现地基（converter 不换 float，物理 left/right→block 方向；需独立 block-方向 clearance 计算）。修后四组件（PNG EXPAND + draw_order ON + abspos-vrl + vertical-rl clearance）同提交应 **net≥0**，可安全启用 PNG fix。


### R152 — cull_invisible 重建 draw_order（draw_order 生产可用，PNG bundle 净效果 -9→-2）（净中性零回归，已提交）

**关键发现：R149 draw_order 基础设施此前从未在生产路径生效**。DODBG 探针（render_full_scene 加 eprintln）证实 harness 运行 clearance-001 时 `draw_order.len()=0 use=false`——draw_order 总是空。根因：pipeline.rs:219 `primitives.cull_invisible(viewport)`（每个 HTML 渲染都调用）的旧实现用 `.iter().filter().cloned().collect()` 重建 typed Vec，draw_order 设为空（索引失效）。故 R149 的 env-gated `render_draw_order` 路径在生产中从未被进入，只在直接单测中有效。

**变更（已提交，零回归）**：`cull_invisible`（ops.rs）重写为对每个 typed Vec 用 `enumerate()` 记录保留元素的 `旧索引→新索引` 重映射（`*_remap: Vec<Option<usize>>`），cull 后按重映射重建 draw_order（被剔除的 DrawOp 丢弃，clips/glyphs/blend_modes 全保留索引不变）。

**验证（净中性零回归）**：全量 reftest **436/490 持平**（env off + env ON 双路径均 436，set-diff 零翻转）。make test **12178 passed/0 failed**，clippy 零警告，fmt clean。draw_order 现在生产路径可用（cull 后仍非空）。

**PNG bundle 全量实测进展（关键）**：临时叠加 PNG EXPAND + ZERO_DRAW_ORDER=1 实测——
- R149 PNG-only = 427 (-9)
- R151 PNG+abspos = 431 (-5)
- **R152 PNG+abspos+draw_order ON（cull 修复后 draw_order 真生效）= 434 (-2)**
- clear-clearance-calculation-001/002/003 三测试从 1.25/1.67/1.62% → 0.00%（draw_order 修复父背景图覆盖子内容的 painting-order 缺陷，真通过）
- **剩余 net -2 = vertical-rl clearance vrl-004/008**（7.09/6.42%），baseline 假通过（3.33/2.08% < 5%），PNG fix 后暴露 vertical-rl §9.5.2 clearance 垂直轴精度 bug（R114b territory，最难的 clearance 边缘 case）

**结论**：draw_order 生产可用后，PNG bundle 从 net -9 改善到 **net -2**。唯一剩余阻塞=vertical-rl clearance（2 个测试）。下轮修 vrl-004/008 后，PNG EXPAND + draw_order=on + abspos 四组件应 **net≥0**，可安全提交 PNG fix（解锁 12 IMG-REF 退化用例的真实渲染，虽 R149 实测 IMG-REF 无新增真通过，但消除假通过=真实正确性）。


### R151 — abspos vertical-rl height:auto 收缩修复（净中性零回归，已提交，PNG bundle 组件 C' 就位）

**变更（已提交，零回归）**：`fix_vertical_mode_abs_pos`（engine.rs:1205-1224 内）在 `all_inset_auto` 分支新增 height 收缩——对 vertical-rl 容器内 height:auto 的 abspos 子元素，把 `child.height` 从 taffy 给的 cross-axis stretch（320=CB 高）收缩到内容 inline 跨度（`fragment.width.max(fragment.font_size)`，垂直 IFC 下 fragment.width=单行/字形视觉竖向高度）。+1 单测 `test_abspos_vertical_rl_height_auto_shrink_to_fit`（断言 h≈80px 非 320）。

**IFC fragment 语义定位**（探针 ABSDBG）：vertical-rl abspos span fragment=`x=240 y=80 w=80 h=44 fs=80`，child=`x=0 y=0 w=80 h=320`。fragment.width=80 是 inline 跨度（单 80px 字形），fragment.height=44 是水平模式 line-height 残留（垂直模式不用）。故 height 收缩读 fragment.width（非 fragment.height）。

**零回归验证**：全量 reftest **436/490 持平**，set-diff 零翻转。4 个 abs-pos-non-replaced-vrl（006/012/122/130）退化参考测试从 **4.50%→1.83%** 改善（绿 span 现定位正确，仍因退化 ref 非零但远低于阈）。make test **12178 passed/0 failed**（+1 新单测），clippy 零警告，fmt clean，smoke 通过。

**PNG bundle 实测（关键进展）**：临时叠加 PNG EXPAND 修复实测——
- PNG-only（R149）= 427/490（net -9）
- **PNG + abspos-vrl 修复 = 431/490（net -5）**——abspos-vrl 修复恢复 4 个 abs-pos-non-replaced-vrl 回归（从假通过→真通过）
- 剩余 net -5 = clearance 集群（clear-clearance-calculation-001/002/003 [1.25/1.67/1.62%] + clearance-calculations-vrl-004/008 [7.09/6.42%]）修前假通过、修后暴露真实 clearance 像素误差

**结论**：abspos-vrl 修复是 PNG bundle 组件 C' 的**已验证零回归基础**。bundle 推进到 net -5（从 -9），剩余阻塞=clearance 精度（独立可修，CSS2 §8.3.1 + §9.5.2）。下轮修 clearance 后，PNG EXPAND + draw_order + abspos-vrl + clearance 四组件应 net≥0，且 12 个 IMG-REF 退化用例中部分可能真实通过（需实测）。



**背景**：R149 PIL 分析（PNG fix + draw_order）显示 abs-pos-non-replaced-vrl-006 test 绿 span=0（ref 6400），推断 abspos vertical-rl 静态位置 bug。本轮写探针单测精确定位。

**探针**：复刻 abs-pos-non-replaced-vrl-006 结构（html `writing-mode:vertical-rl` + `#cb` 320×320 position:relative + 内含文本 + `<span position:absolute top:auto bottom:auto height:auto>`），layout tree dump 定位每个 box 的 position/display/writing-mode/is_absolute/几何。

**精确发现**：abspos span（is_absolute=true, pos=Absolute, writing-mode 继承 vertical-rl）几何 = `x=240 y=80 w=80 h=320`。**h=320（= 完整 CB cross-size）错误**——CSS §10.3.7 + writing-modes §7.1：vertical-rl 下 top/height/bottom 映射到 inline 轴，`height:auto` 应 shrink-to-fit 到内容（单 80px glyph = 80px），不应填满 CB 的 320。spec 注释明确：`height: auto (based on the content) = 80px`，`top: auto → static position = 160px`，`bottom: solved = 80px`（160+80+80=320 ✓）。当前实现把 height:auto 当作填满 cross-axis（320），是 taffy/converter 把 abspos 的 auto height 当 cross-axis stretch 而非 inline-axis shrink-to-fit。

**为何不产生 session pass**：4 个 abs-pos-non-replaced-vrl（006/012/122/130）全是**退化参考**测试——test 侧 `background: red url(bg-red-3col-3row-320x320.png)` + ref 侧 `swatch-green.png`/`pass-cdts-abs-pos-non-replaced.png` 都是非 RGBA PNG，受 harness PNG bug（R135）影响 → 双方退化 → 当前 4.5% 凑合通过（< 5% writing-modes 容差）。修 abspos height 不改变当前通过数（已凑合通过）；仅在 PNG fix 后（ref 变可见）才有意义，且需与 PNG bundle 同提交。**这与 R149 结论一致**：PNG bundle 组件 (C')=abspos vertical-rl §10.3.7 是多会话 groundwork，非单会话 pass lever。

**独立验证：abspos height:auto bug 是否影响其他 CSS-REF 失败**？检查 abspos-containing-block-outside-spanner（4.30%, CSS-REF）——水平书写模式 + 显式 top/left/height/width（非 auto），是 column-span:all 建立 CB 子问题，**非** height:auto-vertical-rl 同源。flex-abspos-inset-nested 是 img aspect-ratio（R146 已定位）。**无当前失败用例受此 bug 直接阻塞**。

**fix 入口指引（下轮）**：`fix_vertical_mode_abs_pos`（engine.rs:1133-1217）当前用 IFC fragment 坐标修正 abspos 静态位置（仅 top/bottom 全 auto 时），但**不修 height**——height 仍由 taffy 给的 320（cross-axis stretch）。下轮应在该函数内：对 vertical-rl 容器内 height:auto 的 abspos 子元素，把 height 收缩到内容 inline 轴跨度（fragment 的 inline extent）。需配合 PNG fix 验证（退化 ref 下无法判正确性）。

### R149 — DC-10 draw_order 基础设施（净中性零回归，已提交）+ PNG bundle 实测（已回退）

**变更（已提交，零回归）**：`RenderPrimitives` 新增 `draw_order: Vec<DrawOp>` 字段（primitive/mod.rs），每个 `add_*` 方法记录插入顺序（DrawOp 枚举指向 typed Vec 索引）。`render_full_scene`（cpu/mod.rs）拆分为 `render_typed_buckets`（默认，字节不变）+ `render_draw_order`（`ZERO_DRAW_ORDER=1` 启用，按 draw_order 顺序渲染）。`cull_invisible` 重建时清空 draw_order（索引失效）。+1 单测 `test_draw_order_records_insertion_order`。make test **12177 passed/0 failed**，clippy 零警告，fmt clean，smoke 686/686，upstream **436/490 持平**（默认 + `ZERO_DRAW_ORDER=1` 双路径均 436，set-diff **零翻转**）。

**关键发现：纠正 R135b「draw_order net -1」结论**。R135b 在 432 基准测得纯 DrawRecord（DOM 顺序）regress `abs-pos-non-replaced-vrl-002`（4→5.33%）。本轮在 436 基准实测 `ZERO_DRAW_ORDER=1` = **净中性**（54 失败完全一致，零翻转）。原因：R142 的 vertical-rl 兄弟位移轴修正（HorizontalTb 守卫）已消除该回归。**draw_order 基础设施可安全启用**——是 PNG bundle 组件 (B) 的已验证零回归基础。

**PNG bundle 实测（PNG fix 已回退，draw_order 保留）**：`load_png_file` 加 `EXPAND|STRIP_16` + 按实际 samples 分配 + RGB/grayscale 补 alpha=255。实测 **436→427 net -9**（修正 R135 记录的 -5，基准不同）。PNG fix + `ZERO_DRAW_ORDER=1` 组合**仍 net -9**——draw_order **未能**修复 abs-pos-non-replaced-vrl 回归。PIL 分析 abs-pos-non-replaced-vrl-006：test 绿(0,128,0)=0（ref 6400）、红(255,0,0)=18321 主导。**真正阻塞重新定性**：不是 DC-10 绘制顺序，而是 **abspos vertical-rl 静态位置 bug**——`position:absolute; top:auto; bottom:auto` + `direction:ltr` + `writing-mode:vertical-rl` 下 CSS §10.3.7 静态位置计算错误，绿 span 子元素定位错位；PNG 网格背景只是**暴露**了该布局 bug（修前 PNG 退化=绿 span 侥幸显示在红底上凑合通过）。完整 bundle 修正 = (A) PNG EXPAND + (B) draw_order 已就绪 + (C') **abspos vertical-rl §10.3.7 静态位置**（替代旧 (C) clearance 精度——clearance 精度是次要的，abspos-vrl 才是 4 回归的真正主因）。

**本轮实证复核（独立验证 R148）**：54 失败 = **41 CSS-REF + 12 IMG-REF（退化区，含 8 个非 RGBA support PNG）+ 1 firefox-bug 路径缺失**。逐个 PIL+BBox 分析 10+ 失败全部印证 R148 结构性结论，clean single-session win 四重确证穷尽（R140/R144/R148/R149）。

**下一步明确指引（多会话）**：PNG bundle 推进顺序——(1) 先独立修 **abspos vertical-rl §10.3.7 静态位置**（影响 4 个 abs-pos-non-replaced-vrl 假通过，CSS-REF 可独立验证）；(2) 再叠加 PNG EXPAND + draw_order=on（此时 abspos 已正确，应 net ≥0）；(3) clearance 精度收尾。draw_order 基础设施已就位无回归，PNG fix 仍必须与 abspos-vrl 修复同提交。

### R145 — flex/grid/table 容器子元素 float 归零（+1，零回归，纠正 R144 R109 误判）

**变更**：`crates/layout-engine/src/engine.rs` 的 `adjust_float_positions_with_context` 入口（container_width 计算后），当 `box_node.is_layout_container`（Flex/InlineFlex/Grid/InlineGrid/Table/InlineTable）时，对所有直接子元素置 `child.float = FloatValue::None`。+1 单测 `test_flex_item_float_is_ignored`（writing_mode_tests 模块，断言 flex 容器内带 float:right 的子元素 x 不被推到右缘）。

**根因（精确插桩定位）**：R144 把 R109 标为「color-block 绘制路径未定位、6 轮不可解」。本轮 engine.rs extract_layout + painter/mod.rs paint_background 双端插桩对比，发现 css-flexbox-test1 / css-flexbox-row 的 `.item` flex item：extract_layout 时 box.x=2（正确），paint 时 box.x=690（=780−90，容器右缘减宽度）。**真正改写 x 的是 `adjust_float_positions_with_context` 的浮动后处理**——测试的 `.item` 带 `float:right`（CSS 注释「make sure UA that doesn't support writing mode and flexbox fails」），Phase 1 把它定位到 `container_width − right_used_width = 690`。R141b 的 mirror child.x 实验对此无效（它改的是 flex item 的 block 子元素，非 IFC 重定位）。**R144 的「paint 路径未定位」与「adjust_inline_block_positions vertical_rtl」判断均错**——x 来自浮动后处理。

**修复**：CSS Flexbox §4（Display 类型）/ Grid §4 / Tables §2.4 规定：flex/grid/table 容器的流内子元素（布局项）其 `float` 与 `clear` 不产生浮动或清除效果，`float` 计算为 `none`。在浮动后处理入口对 is_layout_container 父级的直接子元素归零 float，使后处理（含 Phase 1 定位 + Phase 2 BFC 排斥 + paint 的 float 绘制）一致忽略它。taffy 内部已据此布局（flex item 位置正确），旧后处理是唯一的破坏源。

**零回归**：全量 reftest 435→**436/490**，css-flexbox 44→45/55。set-diff 验证：唯一翻转 = css-flexbox-test1 FIXED（0.00%），css-flexbox-row 改善 1.82%→1.23%（仍未过阈，剩余=vertical-rl 色块 IFC 列序，独立子问题），零新失败、零其他类别变化（CSS2 113/multicol 40/grid 17/writing-modes 55/tables 51 全持平）。make test 全绿（含新单测：移除归零→a.x≈500 右缘 FAILED、加回→PASS），clippy 零警告，fmt clean，smoke 686/686。

**方法论**：纠正了 R141b「R109 单会话不可解」的过早结论——R141b 针对的 paint/inline-block 路径是错的方向；本轮双端插桩（extract vs paint 的 box.x 对比）锁定**真正的破坏入口是浮动后处理**，而修复只需 6 行（is_layout_container 守卫 + 归零循环）。教训：架构性失败（R109）的「不可解」结论需经多入口插桩交叉验证，单一入口的失败不能推广为整体不可解。见 [[r144-plateau-verified-property-audit-exhausted]]（被本轮纠正）、[[r109-writing-mode-flex-arch]]。

### R144 — 平台期独立复核（诊断，持平，无提交）

### R143 — 实现 inline-size / block-size 逻辑尺寸属性（净中性零回归，缺失 CSS 属性补全）

**变更**：`crates/style-system/src/property/apply.rs` 新增 `inline-size`→`width`、`block-size`→`height` 映射分支（CSS Logical Properties §1）。+1 单测 `test_apply_inline_block_size_logical`（apply_coverage.rs，断言 inline-size→width、block-size→height）。

**根因**：`inline-size` / `block-size` 在 apply.rs 中**完全没有 match 分支**——作为未知属性被静默忽略。`firefox-bug-1881495` 用 `inline-size:1em/2em/3em` 控制 6 个 inline-grid 的内联尺寸（1em 时 2 个 Ahem X 不应放入→换行），属性被忽略→所有 grid 退化为内容尺寸（~60px）→ 1em/2em 用例布局错。

**修复**：inline-size→width、block-size→height（水平书写模式物理等价）。垂直书写模式的轴正确性由 converter 已有的 `swap_writing_mode_axes`（width↔height 互换）自动保证，无需在 apply 层感知 writing-mode。margin-inline-end 等逻辑边距/内边距**已**映射到水平物理等价（apply_advanced.rs），同样经 swap 自动修正垂直轴——本轮无需改。

**净中性零回归**：全量 reftest **435/490 持平**，set-diff 验证零 pass/fail 翻转。8 个 inline-size 用例：7 个原本通过仍通过（dynamic-isize-change-001 / position-absolute-in-inline-005/006 / table-cell-inline-size-box-sizing-quirks / abs-pos-border-offset-001/002/003——其中 005/006 因现应用声明尺寸 diff 微增 0.64→0.76 / 0.76→0.81 但仍远低于阈），firefox-bug-1881495 **7.28%→1.74%**（剩余 1.74% = taffy grid 对定宽 inline-grid 的 auto 轨道按内容尺寸而非 grid 宽约束子元素，导致 1em grid 的 "X X" 不换行——taffy grid 内部，非 ZeroWeb 侧可修）。make test 全绿（含新单测），clippy 零警告，fmt clean，smoke 686/686。

**为何保留 +0**：与 R140 回退的 gap-fix 不同——本轮实现的是一个**命名的、规范定义的缺失 CSS 属性**（CSS Logical Properties），属目标范围内「CSS 属性解析」核心能力，非推测性规范符合；有单测覆盖；零 pass-count 回归；并将 firefox-bug 推至过阈边缘（1.74%，剩余为独立 taffy grid 子问题）。方法论：从「失败用例反查其依赖的 CSS 属性是否被实现」发现整类属性缺失（grep wpt-data 命中 19 文件用 inline-size），而非逐像素 BISECT。见 [[r142-vertical-rl-sibling-shift-axis]]、[[r97-intrinsic-sizing-rootcause]]（同类「实现缺失 CSS 关键字/属性」模式，但 intrinsic sizing 4win/13regress 风险远高于本轮 inline-size 的零回归）。

### R142 — 垂直书写模式兄弟位移轴修正（净中性零回归，整页空白→可见）

**变更**：`crates/layout-engine/src/engine.rs` 的 `remeasure_inline_only_containers` 末尾「inline-only 容器收缩后上移后续兄弟」逻辑（`sibling.y += shrink_delta`）增加守卫 `matches!(box_node.writing_mode, WritingModeValue::HorizontalTb)`。+1 单测 `test_vertical_rl_block_sibling_not_pushed_offscreen`（复刻 box-offsets-rel-pos-vrl-004 body，断言垂直模式无非流内盒子被推到负 y）。

**根因**：该兄弟位移逻辑只适用于**水平书写模式**（块流方向=y 轴，容器 inline 重测量使高度收缩后，下方块兄弟应上移合拢空隙）。在**垂直书写模式**中块流方向为**水平（x 轴）**，「高度」是 inline 轴跨度——inline 轴收缩**不会**在块轴留空隙，故不应移动按 x 排列的块兄弟。旧代码无条件 `sibling.y += shrink_delta`（负值），把垂直模式的兄弟推到负 y（屏幕外）。典型表现：`writing-mode:vertical-rl` 根页面（box-offsets-rel-pos-vrl-004）整页渲染为 100% 空白——`<p>` inline 重测量收缩 -448px，后续静态盒 + 4 个蓝块全部被推到 y=-448 及以下。

**定位过程**：BISECT 插桩逐后处理步跟踪 300×300 静态盒的 y：adjust_float=0、shrink_vertical=0、remeasure_float=0、**remeasure_inline=-448**（罪魁）。PROBE_FINAL 对比前后树确认静态盒 y 0→-448，恰等于 `<p>` 高度 752→304 的收缩量。

**修复**：仅在父容器 writing_mode 为 HorizontalTb 时执行兄弟 y 位移；垂直模式跳过（块兄弟按 x 排列，inline 轴收缩不影响其块轴位置）。对全部水平用例行为不变（守卫恒真），仅改变垂直模式行为。

**净中性零回归**：全量 reftest **435/490 持平**（vrl-004 25.24%→12.72%、vlr-005 25.12%→9.34% 内容恢复可见但未过阈——剩余差异是 4 个相对定位蓝块按**左→右**堆叠（应右→左），属垂直块流方向 R109 谱系，非本轮范围）。set-diff 验证零 pass/fail 翻转。make test 全绿（含新单测：移除守卫→FAILED「off-screen」、加回→PASS），clippy 零警告，fmt clean，smoke 686/686。

**方法论**：与 R140 gap-fix 实验不同——本轮有**单测可证缺陷**（移除守卫即空白/负 y），且修复的是**用户可见的灾难性缺陷**（整页空白）而非细微规范符合性，并显著改善 2 个失败用例（25%→10%），是垂直书写模式渲染正确性的必要前置。独立穷尽验证了 R140「清洁 surgical 路径耗尽」结论后，转向**按失败聚类重新插桩定位**（非 bbox 扫描）发现此「垂直模式兄弟位移轴」bug——一个被 7 轮 +0 章节遗漏的、独立于 multicol/IFC/flex 结构性聚类的 writing-mode 后处理缺陷。见 [[r142-vertical-rl-sibling-shift-axis]]、[[r140-cleanwins-exhausted-verified]]（前置清洁路径穷尽结论）、[[r109-writing-mode-flex-arch]]（剩余垂直块流方向阻塞）。

## 里程碑完成状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| M1 — WPT Reftest 基础设施 | ✅ 完成 | 14/14 标准全部达成 |
| M2 — CSS 2.1 + Quirks Mode | ✅ 完成 | CSS parser + style system quirks 已实现；layout engine quirks 推迟到 M4 |
| M3 — Flexbox + Grid | ✅ 完成 | 179 个 reftest, 100.0% pass rate；Flexbox/Grid 无渲染缺口 |
| M4 — Float + Table + Multicol | ✅ 完成 | float + table + multicol 布局算法已实现；219 个 reftest, 100.0% pass |
| M5 — 文字排版 | ✅ 完成 | CJK 换行 + justify 修复 + float 堆叠修复 + 51 个 Text reftest |
| M6 — 全量扩展 | ✅ 完成 | 685 reftest, 13 目录全部 ≥50, 100.0% pass；rustybuzz + unicode-bidi 已集成 |
| M7 — 渲染器图元覆盖 | ✅ 完成 | CPU 渲染器：全部 13 种图元 ✅；GPU 渲染器：全部 13 种图元管线 ✅ + 48 个单元测试 ✅；浏览器消费：全部 13 种图元 ✅；浏览器 GPU 路径集成 ✅ |
| M8 — 布局正确性 | ✅ 完成 | BFC 检测 ✅；float clear ✅；margin 折叠(taffy 0.7 内置) ✅；<img> 固有尺寸 ✅；position:fixed ✅(adjust_fixed_to_viewport)；position:sticky 需宿主层（已标记 is_sticky，后续集成）；percentage height/auto margin/min-max-width 已有测试验证 |
| M9 — 高级视觉效果 | 🔧 进行中 | 重复渐变 ✅；多图层背景 ✅；clip-path 全形状裁剪 ✅(inset+circle+ellipse+polygon)；border-image ✅；text-shadow ✅；backdrop-filter ✅；CSS mask ✅(渐变蒙版裁剪+alpha衰减)；overflow 全图元裁剪 ✅；滚动容器 paint 偏移 ✅(scroll_x/scroll_y 字段 + paint 时子元素坐标偏移 + 3 个单元测试)；剩余：scroll-snap 行为（需宿主层输入路由）、滚动输入路由（需浏览器 app 集成） |
| M10 — 上游 WPT 真实 Reftest 导入 | ⏸ 阶段性封顶 | 基础设施 ✅；490 个上游 reftest 已导入（9 个目录）；当前稳定基线 **393/490 (80.2%)**（R71 确认）；内联 reftest **685/685 (100%)**；R37-R71 共 35 轮已穷尽所有增量改进路径；后续提升需进入专项架构改造周期：**(1) taffy-IFC 架构统一**、**(2) multicol inline 内容跨列拆分**、**(3) writing-mode 垂直布局完整实现**；执行依据：[`post-r71-architecture-spec.md`](./post-r71-architecture-spec.md)；R71 已完成 **margin override** 基础设施铺设，且 **零回归** |

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
| 外部 stylesheet 加载 | ❌ 缺失 | URL 导航路径未抓取 `<link rel="stylesheet">`；`fetch_url()` 使用 `load_html(&html, None)`，`collect_stylesheets()` 只收调用方 CSS 和内联 `<style>` |
| 图片子资源/ImageCache | ❌ 缺失 | `<img>` 可生成 `ImagePrimitive`，但 URL 导航未抓取/解码图片子资源，ZeroBrowser CPU/GPU 渲染路径传 `None` image cache；WinterTC 首页 Logo 因此缺失 |
| 产品/真实静态页面视觉 smoke | ❌ 缺失 | `apps/browser/assets/welcome.html`、morning.work 录制静态文章页和 WinterTC 录制图片密集首页尚未纳入 ZeroBrowser/WebView/Chromium 截图对比门禁；当前 ZeroBrowser 已出现文本重叠、sibling 文本串联、正文压缩、table/code 退化、Logo 缺失 |
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

| 条目 | 状态 | 说明 |
|------|------|------|
| 导入 reftest 子集 ≥ 50 | ✅ | 179 个内联 CSS 2.1 核心 reftest |
| 通过率 ≥ 95% | ✅ | 100.0% (179/179) |
| CPU 模式达标 | ✅ | 全部通过 CPU 软件渲染 |
| GPU 模式达标 | ✅ | GpuRenderer headless 可用（GPU fills/glyphs + CPU rounded rects） |

### DC-3: Flexbox + Grid 通过率 ≥ 95%

| 条目 | 状态 | 说明 |
|------|------|------|
| Flexbox reftest 子集 | ✅ | 51 个内联 Flexbox reftest（基础+进阶+边界+M6 扩展） |
| Flexbox 通过率 | ✅ | 100.0% (51/51) |
| Grid reftest 子集 | ✅ | 51 个内联 Grid reftest（基础+进阶+边界+M6 扩展） |
| Grid 通过率 | ✅ | 100.0% (51/51) |
| CPU 模式达标 | ✅ | 全部通过 CPU 软件渲染 |

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%

| 条目 | 状态 | 说明 |
|------|------|------|
| Positioning reftest | ✅ | 50 个定位 reftest（基础+进阶+M6 扩展） |
| Float reftest | ✅ | 50 个 float 布局 reftest（M6 扩展） |
| Table reftest | ✅ | 50 个 table 布局 reftest（M6 扩展） |
| Multicol reftest | ✅ | 50 个 multicol 布局 reftest（M6 扩展） |
| 各项通过率 | ✅ | 全部 100.0% |
| CPU 模式达标 | ✅ | 全部通过 CPU 软件渲染 |

### DC-5: 文字排版通过率 ≥ 95%

| 条目 | 状态 | 说明 |
|------|------|------|
| css-text/ reftest ≥ 50 | ✅ | 51 个 |
| css-text/ 通过率 | ✅ | 100.0% |
| css-fonts/ reftest ≥ 50 | ✅ | 50 个 |
| css-fonts/ 通过率 | ✅ | 100.0% |
| css-text-decor/ reftest ≥ 50 | ✅ | 50 个 |
| css-text-decor/ 通过率 | ✅ | 100.0% |
| css-writing-modes/ reftest ≥ 50 | ✅ | 50 个 |
| css-writing-modes/ 通过率 | ✅ | 100.0% |
| CPU 模式达标 | ✅ | 全部通过 CPU 软件渲染 |

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
| FilterPrimitive | ✅ | blur + opacity |
| BlendModePrimitive | ✅ | normal/multiply/screen |
| render_full_scene() 入口 | ✅ | 新函数，CSS painting order 渲染全部 13 种图元 |

### DC-9: GPU 渲染器图元覆盖

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
| TransformPrimitive | ✅ | 简化处理（像素级后处理，与 CPU 渲染器对齐） |
| ClipPrimitive | ✅ | 简化处理（scissor rect 全局裁剪） |
| FilterPrimitive | ✅ | 简化处理（CPU 后处理对齐） |
| BlendModePrimitive | ✅ | 简化处理（CPU 后处理对齐） |

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


---

## 下一步

1. ~~验证 cargo test 全绿~~ ✅
2. ~~扩展 manifest.rs 添加 fuzzy 元数据解析~~ ✅
3. ~~扩展 ReftestConfig 添加分类容差和 per-test fuzzy 注解~~ ✅
4. ~~创建 reftest skip list 和过滤机制~~ ✅
5. ~~创建 Chromium 截图脚本~~ ✅
6. ~~实现 reftest runner CLI~~ ✅
7. ~~导入 CSS 2.1 核心 ≥ 50 个 reftest~~ ✅ (159 个)
8. ~~运行初始 reftest 基线测试~~ ✅ (100.0%)
9. ~~实现 JS 执行集成~~ ✅
10. ~~实现 GPU 截图~~ ✅
11. ~~CI 集成~~ ✅
12. ~~M1 完成~~ ✅
13. ~~M2 — Quirks Mode 全部可执行项~~ ✅ (CSS parser + style system quirks)
14. ~~M3 — Flexbox + Grid 基础+进阶 reftest~~ ✅ (21 个新 reftest, 100.0% pass)
15. ~~M3 — Flexbox/Grid edge case reftest~~ ✅ (20 个边界 case reftest, 100.0% pass)
16. ~~M3 — Flexbox/Grid 渲染缺口修复~~ ✅ (无缺口，全部通过)
17. ~~M4 — Table display types 添加~~ ✅ (10 个 table display variant)
18. ~~M4 — 基础 float 布局实现~~ ✅ (float left/right 定位 + 垂直堆叠)
19. ~~M4 — Float 布局 reftest~~ ✅ (10 个 reftest, 100.0% pass)
20. ~~M4 — Float exclusion zone 连接~~ ✅ (remeasure_text_with_float_exclusions)
21. ~~M4 — UA 默认 display 值~~ ✅ (ua_default_display 为 HTML 元素注入正确的 display type)
22. ~~M4 — parse_display 修复~~ ✅ (color.rs 中补全 11 个 table display types)
23. ~~M4 — Table 布局算法实现~~ ✅ (table grid 构建 + auto layout + colspan + border-spacing)
24. ~~M4 — Table 布局 reftest~~ ✅ (9 个 reftest, 100.0% pass)
25. ~~M4 — Multi-column 布局算法~~ ✅ (shortest-column-first 均衡分配 + column-count/column-width/column-gap)
26. ~~M4 — Multi-column 布局 reftest~~ ✅ (10 个 reftest, 100.0% pass)
27. ~~M5 — CJK normal-mode 逐字符换行~~ ✅ (split_into_words 中 CJK 字符单独作为单词)
28. ~~M5 — text-align: justify 修复~~ ✅ (使用 effective_content_area 计算剩余空间)
29. ~~M5 — Float exclusion 堆叠修复~~ ✅ (max → additive stacking)
30. ~~M5 — 文字排版 reftest~~ ✅ (10 个新 reftest, 229 总, 100.0% pass)
31. ~~M5 — 文字排版扩展 reftest~~ ✅ (51 个 Text reftest, 260 总, 100.0% pass)
32. ~~修复 ReftestCategory::from_path 路径匹配~~ ✅ (添加 starts_with 模式)
33. ~~更新 DC-3~DC-6 完成状态~~ ✅ (DC-3~DC-5 全部达标, DC-6 完成)
34. ~~M5 完成~~ ✅ (CJK 换行 + justify + float 堆叠 + 51 Text reftest)
35. ~~M6 — Flexbox+Grid 扩展到 ≥50~~ ✅ (各 51 个 reftest, 296 总, 100.0% pass)
36. ~~M6 — 扩展剩余目录到 ≥50~~ ✅ (535 总, 10 个目录全部 ≥50, 100.0% pass)
37. ~~M6 — 拆分 reftest_data.rs 为目录模块~~ ✅ (reftest_data/ 目录, 每个分类独立文件)
38. ~~M6 — DC-5 文字排版全目录达标~~ ✅ (新增 css-fonts/css-text-decor/css-writing-modes, 685 总, 100.0% pass)
39. ~~M6 — 引入 rustybuzz（OpenType shaping）~~ ✅ (GSUB/GPOS 连字+kerning，fontdue 回退)
40. ~~M6 — 引入 unicode-bidi（RTL 文本）~~ ✅ (BiDi 重排序，RTL 字符自动检测)
41. ~~M7 — CPU 渲染器全量图元~~ ✅ (render_full_scene() 支持全部 13 种图元)
42. ~~M7 — 浏览器图元消费~~ ✅ (transform_webview_primitives() 处理全部 13 种 + render_cpu() 使用 render_full_scene())
43. ~~M7 — 验证~~ ✅ (cargo test 7800+ 全绿, clippy 零警告)
44. ~~M7 — GPU 渲染器全量图元管线~~ ✅ (5 个 WGSL shader + 4 条管线 + mesh 生成 + render_full_scene_gpu())
45. ~~M7 — GPU 渲染器单元测试~~ ✅ (48 个 GPU 单元测试，覆盖 fills/rounded_rect/gradient/shadow/stroke/empty scene)
46. ~~M7 — 浏览器 GPU 路径集成~~ ✅ (app_platform.rs render_frame() 使用 render_full_scene_gpu)
47. ~~M8 — BFC 检测~~ ✅ (establishes_bfc() 检测 overflow/float/position 建立的 BFC)
48. ~~M8 — Float clear 支持~~ ✅ (clear:left/right/both 后处理 + 7 个集成测试)
49. ~~M8 — Margin 折叠~~ ✅ (发现 taffy 0.7 已内置 CollapsibleMarginSet，无需额外后处理)
50. ~~M8 — 替换元素布局~~ ✅ (<img> 固有尺寸注入 + 2 个集成测试)
51. ~~M8 — Position: sticky 标记~~ ✅ (layout 引擎已标记 is_sticky，需宿主层滚动集成时实现动态偏移)
52. ~~M9 — 重复渐变~~ ✅ (GradientPrimitive.repeating 字段 + CPU fract() tiling + GPU WGSL shader)
53. ~~M9 — 多图层背景~~ ✅ (background_image 改为 Vec + 逗号分隔解析 + 逆序渲染)
54. ~~M9 — clip-path inset 裁剪~~ ✅ (clip_all_primitives_to_rect() 处理全部图元类型)
55. ~~M9 — 非矩形 clip-path~~ ✅ (circle/ellipse/polygon 扫描线裁剪 + 点在多边形内检测)
56. ~~M9 — backdrop-filter~~ ✅ (复用 FilterComputedValue + 在元素绘制前应用滤镜)
57. ~~M9 — CSS mask~~ ✅ (mask-image 解析 + mask-mode 解析 + 渐变蒙版裁剪 + alpha 衰减 + 3 个单元测试)
58. ~~M9 — overflow 全图元裁剪~~ ✅ (修复 overflow:hidden/scroll/clip 仅裁剪 fills+glyphs 的问题，改用 clip_all_primitives_to_rect 裁剪全部 13 种图元)
59. ~~M9 — 滚动容器 paint 偏移~~ ✅ (LayoutBox scroll_x/scroll_y 字段 + paint 时 overflow:Scroll 子元素坐标偏移 + 3 个单元测试)
60. M9 — scroll-snap 行为（已解析存储，需宿主层输入路由实现吸附逻辑）
61. M9 — 滚动输入路由（需浏览器 app 集成：嵌套滚动容器识别、逐元素 scroll 事件分发）
62. ~~M10 — FontLoader 修复~~ ✅ (render_to_framebuffer_with_base 使用 create_font_loader()，启用文本渲染；揭示真实通过率 65.0%)
63. ~~M10 — support image 补充~~ ✅ (为缺失的 swatch 颜色/尺寸 PNG 生成文件)
64. ~~M10 — border conflict resolution 基础设施~~ ✅ (resolve_collapsed_borders + resolve_border + border_style_priority)
65. ~~M10 — 调查表格 border CSS 未生效问题~~ ✅ (经调试验证 border-top-width=Px(5.0) 正确设置；原始报告的问题可能是特定测试场景的布局差异)
66. ~~M10 — 实现 paint 系统 font-family 解析~~ ✅ (OpenType name 表解析 + FontLoader.build_font_resolver() + Painter.resolve_font_id() + RenderPipeline.set_font_resolver())
67. ~~M10 — CSS border-width zeroing~~ ✅ (border-style 为 none/hidden 时强制 width=0，符合 CSS 规范)
68. M10 — writing-mode 布局支持（影响 css-writing-modes 40.7% + css-flexbox 部分测试；需实现 vertical-rl/lr 布局方向）
69. M10 — float 布局精度提升（CSS2/floats-clear 20/30 失败；根因分析：swatch 图像缩放 20×20→96×96 与 CSS background-color 精确填充的像素差异，非 float 定位错误）
70. M10 — 失败分布分析：37个<2%、45个2-5%、40个5-15%、40个15-30%、11个>30%；最大改进杠杆为 writing-mode（影响 35 个测试）和 column breaking（影响 32 个测试）
71. ~~M10 — flex-flow 简写展开~~ ✅ (shorthand/mod.rs 新增 "flex-flow" 分支，解析 flex-direction || flex-wrap；修复 3 个 flexbox 测试)
72. ~~M10 — font-family 非法字符验证~~ ✅ (parse_font_family 验证未引用名称仅含有效字符，含非法字符时整个声明无效；修复 2 个 CSS2/fonts 测试)
73. ~~M10 — font 简写验证~~ ✅ (expand_font 检查 size_found，缺少 font-size 的声明无效；更新测试用例匹配 CSS 规范)
74. M10 — column breaking 实现（影响 css-multicol                  28/57 (49.1%)；当前 multicol 仅分配整个子元素到列，不拆分溢出内容；需实现 fragmentation 基础设施）
75. M10 — 浮动清除算法改进（影响 CSS2/floats-clear 20 个测试；当前 max(normal_y, clear_bottom) 未正确实现 CSS 2.1 clearance 对 margin 折叠的阻断）
76. ~~M10 — CSS 2.1 Appendix E 绘制顺序~~ ✅ (float 子元素在非 float 子元素之后绘制，paint_node_in_rect 和 paint_node 各分两轮遍历)
77. ~~M10 — columns 简写顺序无关解析~~ ✅ (双值模式自动检测整数/长度，修复逆序声明如 `columns: 100px 6`)
78. ~~M10 — clearance 代码质量改善~~ ✅ (澄清零 clearance 阻止 margin 折叠；后处理方式局限：taffy 已应用 margin 折叠)
79. M10 — 分析 CSS2 border/background 失败根因（6 个 border 测试 + 5 个 background 测试失败，需定位具体渲染差异）
80. ~~M10 — float Y 位置修复~~ ✅(float 在含 inflow 子元素容器中尊重 taffy Y)
81. ~~M10 — clearance 计算修复~~ ✅(flow_bottom + margin 折叠替代简单 offset 扣除)
82. ~~M10 — inline img 替换元素~~ ✅(InlineFormattingContext 识别 img 固有尺寸)
83. M10 — inline formatting context 改进（影响 CSS2/linebox ~8 个测试；空 inline 元素 line-height 贡献、inline 元素 margin 处理）
84. M10 — writing-mode 布局支持（影响 35 个测试：12 个 abs-pos-non-replaced 21.33% + direction 12.49% + float-orthog 3% 等；需实现垂直书写模式下轴交换）
85. M10 — multicol column breaking（影响 31 个测试；需实现内容跨列拆分）
86. M10 — CSS2 inline box model 改进（empty-inline-002/003 + inline-box-001/002 + inline-formatting-context-008/009/011 等 ~8 个测试）
87. ~~M10 — CSS 绝对长度单位~~ ✅(parse_length 新增 in/pt/pc/cm/mm/Q，background 简写分类器更新)
88. ~~M10 — 表格单元格 vertical-align~~ ✅(top/middle/bottom 支持)
89. ~~M10 — 径向渐变位置解析~~ ✅(resolve_position 正确处理 Percentage/Px)
90. ~~M10 — 表格 min-height border-box~~ ✅(min-height-table 通过)
91. ~~M10 — 表格单元格高度最小值~~ ✅(cell height = max(row_height, content_height))
92. ~~M10 — 图像双线性插值~~ ✅(CPU render_image 从最近邻改为双线性插值)
93. ~~M10 — writing-mode 轴交换回退~~ ✅(禁用不完整的轴交换和隐式继承，避免回归)
94. ~~M10 — CSS 负值 border-width 拒绝~~ ✅(负值视为无效，回退到初始值 medium；修复 border-bottom-width-001.xht)
95. ~~R13 — inline-block 行内定位~~ ✅(adjust_inline_block_positions 后处理 + IFC InlineBlockBox + baseline 修正；CSS2 59.7%→62.8%)
96. ~~R13 — border-collapse 样式覆盖~~ ✅(collapsed_border_style_overrides + 更多命名颜色)
97. ~~R14 — writing-mode 垂直 inline 布局~~ ✅(break_items_into_columns + 布局引擎接入 + 绘制层垂直字形渲染；6 个单元测试)
98. ~~R20 — reftest 分类容差 bug 修复~~ ✅(for_category 替代 Default::default()；+24 测试通过)
99. ~~R20 — columns 简写解析修复~~ ✅(整数优先 column-count 而非 column-width)
100. ~~R20 — 零高度浮动处理~~ ✅(line_max_height 跳过零高度浮动)
101. ~~R21 — font 简写负 line-height 拒绝~~ ✅(CSS Fonts §3.7)
102. ~~R21 — background-position 简写双值捕获~~ ✅(+1 upstream test: background-329.xht)
101. R20 — multicol column breaking（影响 ~16 个测试：需实现内容碎片化）
102. R20 — CSS2 float/clear 精度提升（影响 ~19 个测试：clearance 算法 + image scaling）
103. R20 — CSS2 inline box model（影响 ~8 个测试：空 inline line-height + block-in-inline）
104. R20 — Flexbox baseline + writing-mode（影响 ~9 个测试：flex 方向轴交换）
105. R20 — CSS 表格子像素修复（影响 ~9 个测试：border 精度 + image scaling）

### R144 后续路径（结构性多轮，单会话预期 +0）

经 R140（独立穷尽验证）+ R141b（R109 6 轮不可解）+ R144（属性审计穷尽）三重确证，**435/490 为单会话零回归平台期**。剩余 55 失败全属结构性多轮里程碑，按预期收益/风险排序的候选路径：

1. **R109 inline-block ownership 架构项目**（多轮，高风险，潜力 +2 集群 css-flexbox-row/test1 + flexbox-column-row-gap-004）
   - 根因：vertical-rl flex 容器中 inline-block 色块（经 IFC 绘制）的 x 坐标不受 `mirror_vertical_rl_block_children` 后处理影响（R141b 已证 6 轮单兵不可解）。PIL 确认 css-flexbox-row.html 4 色块 TEST x[670,789] 右 vs REF x[10,39] 左。
   - 真正修复需统一 paint 的 inline-block 绘制路径到 IFC（DC「Layout/Paint IFC 双路径」），消除绕过 paint_node 与 paint_text 非存储的两条已知路径。属多次会话架构重构。
   - 见 [[r109-writing-mode-flex-arch]]、[[r142-vertical-rl-sibling-shift-axis]]、[[r140-cleanwins-exhausted-verified]]。

2. **R131 multicol 列感知 IFC 碎片化**（多轮，高风险，潜力 ~16 测试但回归 ~20 通过用例）
   - 根因：multicol 塌缩的零高子元素是行内级 DOM 节点(text/br)，「行高回写」不可行；真正修复=列感知 IFC 按行盒（非子元素）碎片化到各列。R122 记录 paint 路径改动 net -5（multicol 39→34/57）。
   - 前置：需先确认 R131 block-children 分布会回归的 ~20 个单列回退恰好通过的用例（multicol-breaking-000/001/002/003、baseline-000~006 等）。
   - 见 [[r131-multicol-fragmentation-arch]]、[[r113-nested-multicol-twopass-plan]]。

3. **R114b writing-mode 垂直轴 float/clearance 参数化**（多轮，中等风险）
   - 需把 ~150 行 `adjust_float_positions_with_context` 做 block/inline 轴参数化，非 surgical flag，对通过的 floats-clear 有高回归风险。R133 已建实现地基（converter 不换 float，物理 left/right→block 方向）。
   - 见 [[r114-writing-mode-characterization]]、[[r133-vertical-float-impl-ground]]。

**对后续会话的明确指引**：
- 不要重试 R125 large-font 三路径死锁、R140 gap-fix、R141b R109 单兵镜像——均已验证 net-negative 或 +0。
- 不要再做属性实现完整性审计——R144 已穷尽（197 注册属性全有 apply 分支）。
- 单会话若需推进，唯一现实路径是启动上述 3 条结构性多轮项目之一的**第一个安全子步骤**（如 R109 的「定位 inline-block 背景 fill 实际绘制入口」插桩诊断，不改逻辑、零回归），并明确标注为多轮项目的第 N 步。

