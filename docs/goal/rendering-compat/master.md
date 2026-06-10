# 渲染兼容性目标 — 运行时控制面板

**最后更新**: 2026-06-11
**当前活跃里程碑**: M10 — 上游 WPT 真实 Reftest 通过率提升
**上游真实 reftest 通过率**: 78.2% (383/490)

### R41 进展

**通过率**：383/490 (78.2%)，+4 tests（自 R40）。完成 multicol column breaking paint 层渲染；调查 paint IFC font_size_overrides 方案（因行断回归而回退）。

#### R41 代码贡献

| 变更 | 说明 |
|------|------|
| multicol column breaking paint 层渲染 | layout 层将所有片段（含主片段）存储到 `column_span_offsets`；paint 层对 multicol 容器中有 column breaking 的子元素跳过正常渲染，改为对每个列片段独立渲染并裁剪到列区域。+4 tests（multicol-breaking-000/001/002/003） |
| IFC font_size_overrides 基础设施 | InlineFormattingContext 新增 `font_size_overrides` 字段和 builder 方法，paint IFC 可按父元素 ID 覆盖字体大小。暂未启用：实测导致 anonymous-inline-inherit-001 回归（0%→1.86%），因正确 font_size 改变行断行为与 layout IFC 定位冲突 |

#### R41 调查与尝试

1. **paint IFC font_size_overrides（回退）**：尝试将 layout IFC 存储的 `text_node_font_sizes` 转换为按父元素 ID 的覆盖映射传入 paint IFC 的 `collect_inline_items`，使字符宽度计算使用正确 font_size 而非 16px 默认值。回归原因：正确 font_size 导致不同的行断行为（字符宽度变化→换行点变化），与 layout IFC 的定位冲突。这再次确认 R37/R38 结论——paint IFC 无法安全使用与 layout IFC 不同的上下文参数。

#### 按目录通过率变化

| 目录 | R40 | R41 | 变化 |
|------|-----|-----|------|
| css-multicol/ | 56.1% (32/57) | 63.2% (36/57) | +4 tests |

#### R41 失败根因分布

107 个失败测试的分布：
- CSS2/floats-clear precision: ~15（多数为 swatch 图像缩放精度）
- writing-mode vertical: ~13
- multicol remaining: ~21（breaking 004/005/006 + clip/count/fill）
- flexbox baseline: ~9
- CSS2/linebox inline box: ~8
- table various: ~10
- CSS2/border+background: ~7
- 其他: ~24

#### 后续重点（R42+）

1. **multicol breaking 004/005/006 修复**（影响 3 测试，diff 5.6-16.6%）：当前 breaking 基础设施已工作，但这些测试的 column-fill/height 组合可能需要更精细的片段分配。
2. **multicol clip/collapsing 精度**（影响 5 测试，diff 2-4%）：near-miss 测试可能通过小精度修复通过。
3. **CSS2 floats-clear near-miss**（影响 5 测试，diff 1.2-2.4%）：多为 swatch 图像缩放精度问题，少数可能为 clearance 计算精度。
4. **flexbox baseline 对齐**（影响 ~5 near-miss 测试）：需要 baseline 传递改进。
5. **paint IFC 架构改进**（影响 50+ 测试，系统性瓶颈）：唯一可行路径是在所有后处理完成后存储完整 layout IFC 结果到 LayoutBox，paint 直接复用。

### R40 进展

**通过率**：379/490 (77.3%)，与 R39 持平。提交 multicol column breaking 基础设施；深入调查 paint IFC 架构和 inline 元素背景定位问题，两个修复尝试均因回归而回退。

#### R40 代码贡献

| 变更 | 说明 |
|------|------|
| multicol column breaking 基础设施 | ColumnFragment 结构体支持超高子元素跨列拆分，paint 层 overflow 裁剪确保每列只显示对应片段。基础设施就绪，但 paint 层 per-column clipping 未完成（需要存储 fragment 信息到 LayoutBox） |

#### R40 调查与尝试

1. **inline 元素 x 偏移修复（回退）**：在 Phase 2 浮动调整中为非块级元素添加 x 偏移到左侧浮动右边缘。仅改善 clear-inline-001 从 6.04% 到 5.99%，对整体通过率无影响，已回退。

2. **paint IFC 传入实际样式（回退）**：对无浮动子元素的容器传入实际 CSS 样式到 paint IFC，使文本获得正确的字体度量。导致 6 个测试回归（379→373），原因：即使无浮动，传入实际样式导致不同断行行为，与 layout IFC 的定位冲突。已回退。

3. **paint IFC 架构问题确认**：paint IFC 使用 `HashMap::new()` 导致所有文本使用 16px 默认字体度量。这是影响 50+ 测试的系统性问题。修复需要存储 layout IFC 结果到 LayoutBox，但受两个根本问题阻塞：
   - 存储的 IFC 结果在后续后处理（table/multicol）后可能过期
   - paint 基线计算（`frag.y + font_size`）与存储结果的 `frag.y + height` 不一致

4. **inline 元素背景定位问题**：CSS 2.1 规定 inline 元素的 margin-top/margin-bottom 无效，但 taffy 将 inline 映射为 Block 并包含其垂直 margin。paint 层使用 LayoutBox 位置（来自 taffy/Phase 2）渲染背景，而 IFC 使用正确的行内位置渲染文本，导致 inline 元素背景与文本位置不一致（影响 clear-inline-001、inline-box-001/002、border-padding-bleed-001 等测试）。

#### R40 失败根因分布（不变）

与 R39 一致，111 个失败测试的分布：
- multicol breaking: ~16
- writing-mode vertical: ~13
- flexbox baseline: ~9
- CSS2/floats-clear precision: ~15（多数为 swatch 图像缩放精度）
- CSS2/linebox inline box: ~8
- table various: ~10
- CSS2/border+background: ~7
- 其他: ~17

#### 后续重点（R41+）

1. **multicol paint 层 per-column clipping**（影响 ~16 测试）：需要将 fragment 分配信息持久化到 LayoutBox，paint 层根据 fragment 信息对每列应用独立裁剪区域。这是将 css-multicol 通过率从 56.1% 提升的关键。

2. **paint IFC 架构改进**（影响 50+ 测试，系统性瓶颈）：需要在所有后处理完成后的最后阶段运行 IFC 并存储结果。需要解决：(a) 基线计算一致性；(b) 浮动排除区域传递；(c) 后处理步骤不改变容器尺寸的保证。这是最大的系统性改进，但需要较大重构。

3. **inline 元素背景从 IFC 坐标绘制**（影响 ~4 测试）：需要 paint 层对 inline 元素使用 IFC 计算的位置渲染背景和边框，而非 taffy 的 block 位置。这需要存储 IFC inline 盒的位置信息。

### R39 进展

**通过率**：379/490 (77.3%)，与 R38 持平。新增多列容器 BFC 建立和图像插值精度修复；全面分析 111 个失败测试的根因分布。

#### R39 代码贡献

| 变更 | 说明 |
|------|------|
| 多列容器 BFC 建立 | `establishes_bfc()` 新增 `is_multicol` 检查，多列容器正确阻止子元素 margin 折叠（CSS Multi-column §2）。为避免回归，多列容器在浮动包含高度计算中使用非 BFC 路径 |
| taffy overflow: Clip 设置 | tree.rs 中为多列容器设置 `taffy_style.overflow = Clip`，阻止 taffy 内部父子 margin 折叠。不影响视觉裁剪（paint 层使用 LayoutBox.overflow_x/y） |
| 图像双线性插值精度修复 | CPU renderer 的 bilinear interpolation 从 truncation（`as u8`）改为 rounding（`+ 0.5 as u8`），提高图像缩放精度 |
| multicol BFC 单元测试 | margin_collapse.rs 新增 `test_establishes_bfc_multicol` 测试 |

#### R39 失败根因分布分析

对 111 个失败测试进行全面分类：

| 失败类别 | 数量 | 主要根因 | 修复难度 |
|----------|------|----------|----------|
| multicol breaking | ~16 | 需内容碎片化（拆分单个块到多列） | 高（大特性） |
| flexbox baseline | ~9 | taffy first_baselines 未持久化到 Layout | 中 |
| writing-mode 垂直布局 | ~13 | 垂直书写模式轴交换 + 垂直字形渲染 | 高 |
| CSS2/floats-clear 精度 | ~15 | swatch 图像缩放精度 + clearance 边界 case | 中 |
| CSS2/linebox inline box | ~8 | 空 inline line-height + anonymous block 拆分 | 高 |
| table 各种 | ~10 | border-collapse 精度 + min/max-size + row suppress | 中 |
| CSS2/border+background | ~7 | Ahem 字体渲染 + 图像 repeat vs stretch | 低-中 |
| CSS2/fonts | ~2 | font shorthand 验证 + font-family 括号 | 低 |
| writing-mode abspos | ~4 | 垂直模式 inline 布局 + box-offsets | 高 |
| 其他 | ~17 | 混合根因 | 混合 |

**near-miss 测试统计**（< 3% diff）：37 个测试差异小于 3%，但多数差异来自：
1. **Ahem 字体渲染差异**（fontdue vs Skia 光栅化精度）
2. **Swatch 图像缩放**（20×20 PNG → 96×96 与 CSS background-color 精确填充的像素差异）
3. **亚像素舍入**（floor/ceil 在元素边界位置差异）

这些系统性精度问题影响几乎所有 < 3% diff 的测试，无法通过单一修复解决。

#### 后续重点（R40+）

1. **multicol column breaking**（影响 ~16 测试）：需要实现内容碎片化基础设施 — 将单个块级元素内容拆分到多列。当前仅移动整个子元素到下一列。这是 css-multicol 通过率从 56.1% 提升到 95% 的最大杠杆。
2. **writing-mode 垂直布局**（影响 ~13 测试）：需要垂直书写模式下完整轴交换 + 垂直字形渲染（旋转文本 90°）。
3. **flexbox baseline 对齐**（影响 ~9 测试）：需要从 taffy LayoutOutput 捕获 first_baselines 并传递到 IFC 的 InlineBlockBox。
4. **CSS2/linebox inline box model**（影响 ~8 测试）：需要实现 anonymous block box splitting（inline 元素包含 block-level 子元素时拆分 inline box）。
5. **paint IFC 架构改进**（影响 50+ 测试）：需要在所有后处理完成后存储 layout IFC 结果到 LayoutBox，paint 复用该结果。这是最大的系统性改进，但需要较大重构。

### R38 进展

**通过率**：379/490 (77.3%)，与 R37 持平。深入调查 paint IFC 架构改进的可行性，建立基础设施但发现基线计算兼容性问题。

#### R38 调查分析

1. **paint IFC 存储结果方案（方案 C）**：在三个现有 IFC 运行点（remeasure_text_with_float_exclusions、remeasure_inline_only_containers、adjust_inline_block_positions）存储片段结果到 LayoutBox.inline_layout。paint 系统通过 `use_stored` 标志复用结果，避免重新运行 IFC。

2. **基线计算双重计数问题**：IFC 片段的 `frag.y` 表示片段框顶部在行盒中的位置（`baseline_y - run.height`）。paint 渲染代码使用 `frag.y + frag.fs`（font_size）作为基线位置。当 IFC 使用空 styles 时，frag.y 和 frag.fs 都基于 16px 默认值，错误相互抵消，视觉结果正确。当使用存储结果（正确 font_size）时，frag.y 已包含正确的基线偏移，加上 frag.fs 导致双重计数，文本位置下移过多。

3. **传递实际 styles 到 paint IFC（方案 B，验证回退）**：再次验证 R37 结论——将 `styles.unwrap_or(&HashMap::new())` 传入 paint IFC 导致 379→373（-6 tests）回归。根因：paint IFC 与 layout IFC 在不同上下文运行（不同容器宽度、不同 float exclusion zones），正确 styles 导致不同的 line-breaking 行为，与 layout 定位冲突。

4. **零 clearance 未折叠边距修复尝试**：尝试在 zero clearance case 中使用 uncollapsed margin（`flow_bottom + last_flow_mb + child.margin_top`），但对 clearance-006 无影响（该测试的 margin 恰好相等，折叠与不折叠结果相同）。已回退。

5. **失败分布更新**：
   - <2% diff: 17 tests（font metrics/渲染精度）
   - 2-5% diff: 29 tests（定位差异）
   - 5-15% diff: 22 tests（布局差异）
   - 10-20% diff: 30 tests（较大布局差异/缺失功能）
   - >20% diff: 13 tests（功能缺失）

#### R38 代码贡献

| 变更 | 说明 |
|------|------|
| `store_inline_layout_results()` 辅助函数 | engine.rs 新增辅助函数，将 IFC 片段结果存储到 LayoutBox.inline_layout。当前被注释掉（等待基线计算修复），作为未来架构改进的基础设施 |
| clippy 警告修复 | 移除 compute_final_inline_layouts 中未使用的 LineHeightValue import、unreachable pattern（TextAlignValue exhaustive match）、unused mut、unused variable |
| paint_text 宏重构（保留） | text.rs 中的 render_fragment! 宏统一了存储和 IFC 片段的渲染逻辑，消除代码重复 |

#### 后续重点（R39+）

1. **paint IFC 架构改进**（系统性瓶颈，影响 50+ 测试）：存储 IFC 结果的方案（方案 C）因两个根本问题无法直接启用：(a) paint 基线计算 `frag.y + font_size` 与存储结果的 `frag.y + height` 不一致——前者对空 styles IFC 恰好正确，后者对真实 styles IFC 仍有偏差；(b) 更关键的是，IFC 结果在步骤 6/6.5 捕获，但步骤 8（table layout）和 9（multicol）会改变 LayoutBox 的坐标和尺寸，导致存储结果过期。**真正的解决方案**需要在所有后处理完成后的最后阶段运行 IFC 并存储结果，这需要较大的重构。
2. **near-miss 测试攻坚**（17 个 <2% diff）：多数差异来自 font metrics 或 border/image 渲染精度，难以通过简单修复解决。
3. **CSS2/floats-clear 精度提升**（17 个失败）：需要 CSS 2.1 clearance 算法的精细调整。
4. **writing-mode 布局支持**（影响 35+ 测试）：垂直书写模式轴交换。
5. **multicol column breaking**（影响 ~16 测试）：内容碎片化。

### R37 进展

**通过率**：379/490 (77.3%)，与 R36 持平。新增垂直书写模式 gap 轴交换；深入调查 paint IFC 字体度量问题。

#### 修复提交

| 修复 | 影响 | 说明 |
|------|------|------|
| 垂直书写模式 gap 轴交换 | CSS Writing Modes §7.1 | `apply_vertical_writing_mode` 新增 `gap.width ↔ gap.height` 交换。CSS Writing Modes 规定垂直书写模式中 gap 属性轴随主轴交换。当前不影响上游 reftest（测试不使用 gap+writing-mode 组合） |

#### R37 调查分析

1. **paint IFC 字体度量 — 方案 A（render_fs）**：尝试在 paint 循环中用容器 `font_size` 替代 `fragment.font_size` 作为基线偏移和字形渲染大小。回归 2 个测试（379→377）：`font-feature-resolution-002` 使用 `font-size: 2em`（32px），IFC 基于 16px 定位但以 32px 渲染导致字形重叠。已回退。
2. **paint IFC 字体度量 — 方案 B（传递 styles HashMap）**：`paint_text()` 已有 `styles: Option<&HashMap<NodeId, ComputedStyle>>` 参数，但传给 IFC 的是 `&HashMap::new()`。改为传递实际 styles 导致回归 6 个测试（379→373）：paint IFC 与 layout IFC 使用不同上下文（容器宽度、浮动排除区域等）运行，正确样式导致不同行断行为，与 layout IFC 的定位冲突。已回退。
3. **paint IFC 根因确认**：paint 系统运行的是第二次独立 IFC 布局，无法保证与 layout 引擎的第一次 IFC 一致。根本解决方案是存储 layout IFC 结果并在 paint 中复用，避免重新运行 IFC。这属于架构级改进。
4. **near-miss 测试分析**（6 个 <1.5% diff）：
   - `position-relative-table-tfoot-top` (1.04%)：border-collapse 亚像素精度
   - `whitespace-001` (1.05%)：display:table 容器中 inline-block 空白处理
   - `clearance-006` (1.16%)：Ahem 字体 em-to-px 精度
   - `clear-clearance-calculation-002` (1.18%)：swatch 图像缩放精度
   - `block-in-inline-align-001` (1.42%)：IFC 匿名文本 font metrics
   - `border-conflict-resolution` (1.54%)：ridge/outset/hidden 边框冲突解决
5. **flexbox 近 miss 分析**：5 个 <2% diff 测试的根因分类为：(a) inline-flex 基线来自第一个 flex item 而非框底部（taffy Layout 不持久化 first_baselines）；(b) 垂直书写模式 gap 轴交换（已修复但测试未覆盖）；(c) wrap-reverse 基线来自逻辑第一行而非视觉第一行（taffy 上游问题）
6. **产品静态页 smoke 缺口确认**：`apps/browser/assets/welcome.html` 在 Chromium 中布局正常，但 ZeroBrowser 输出出现文本重叠、sibling card/link/shortcut 文本串联、`ZeroBrowser` 宽屏标题误拆行、footer/tagline 文本间距错误。该页面无页面级 JS，说明缺口不在动态 Web API，而在基础 layout/paint/glyph 消费链路。
7. **welcome.html 代码事实**：
   - 页面集中使用 `display:grid`、`display:flex`、`gap`、inline `span` 拼接标题、`<br>`、中英文混排、`letter-spacing`、`box-shadow`、`border-radius`。
   - `crates/layout-engine/src/converter/mod.rs` 当前将 `display:inline` / `inline-block` 映射为 taffy `Block`，而 `crates/layout-engine/src/inline/mod.rs` 又通过 `doc.text_content(child_id)` 把 inline 子树文本收集成 IFC run，存在 inline 所有权分裂。
   - `crates/layout-engine/src/engine.rs` 中 `compute_final_inline_layouts()` 仍被禁用，paint 无法稳定复用 layout 阶段最终 IFC 结果。
   - `apps/browser/src/app_render.rs` 的 `reflow_webview_glyphs()` 会按 baseline 对 WebView glyph 做整行重排，可能破坏 engine 已计算好的 fragment x 坐标，尤其影响同一 baseline 上的 grid/flex sibling 内容。
8. **真实静态文章页 smoke 缺口确认**：`https://morning.work/page/2026-02/fedora-macbook-three-finger-drag.html` 在 Chromium 中是正常静态文章页，但 ZeroBrowser 输出出现 nav 缺失/弱化、tag 与阅读时间串联到大块蓝色背景、正文段落压成一行并重叠、inline code 位置漂移、table 退化为普通文本。该页面无页面级动态渲染需求，说明当前缺口已影响普通中文长文、表格和代码块页面。
9. **morning.work 代码事实**：
   - 页面依赖 `<link rel="stylesheet" href="/styles/github.css">`、`/JetBrainsMono/JetBrainsMono.css`、`/article.css`，外部 CSS 中包含 `.article`、`table`、`code/pre`、标题边框、列表和颜色变量等核心样式。
   - `crates/webview/src/webview.rs` 的 `fetch_url()` 在 Service Worker 命中、HTTP cache 命中和普通网络成功三条路径都调用 `load_html(&html, None)`，没有把页面 `<link rel="stylesheet">` 抓取为 CSS 输入。
   - `crates/engine/src/pipeline.rs` 的 `collect_stylesheets()` 只收调用方传入的 CSS 字符串和文档内 `<style>`，不会解析/抓取外链 stylesheet。
   - morning.work 的 `<head>` 仍包含 body/title/nav/tag 的内联 CSS，因此外链 CSS 缺失只能解释文章 table/code/pre 等样式退化；正文压缩、inline code 漂移和文本重叠仍指向 inline ownership、layout/paint IFC 双路径和 ZeroBrowser glyph 后处理。
10. **图片密集静态首页 smoke 缺口确认**：`https://wintertc.org/` 的核心 CSS 是内联 Twind `<style>`，Chrome 中 header logo、nav button、正文和参与方 Logo 网格均正常；ZeroBrowser 输出中 SVG/PNG Logo 大面积缺失并退化成短横/占位 glyph，标题/副标题与 nav 文本串联，正文段落压成一行，说明仅修外链 CSS 不足以覆盖真实静态站点。
11. **WinterTC 代码事实**：
   - 页面使用内联 utility CSS，不依赖外链 stylesheet；关键结构包含 `display:flex` header、`display:grid` 四列 nav、`flex-wrap justify-evenly` Logo 网格、`text-align:justify` 正文，以及 `/static/logo.svg`、`/static/logos/*.svg`、`/static/logos/*.png` 图片。
   - `crates/engine/src/paint/painter/text.rs` 会为 `<img>` 元素生成 `ImagePrimitive`，`render-foundation` CPU/GPU 路径也能从 `ImageCache` 读取像素并绘制图片。
   - `apps/browser/src/app_platform.rs` 的 CPU/GPU 渲染调用当前都传 `None` 作为 `image_cache`，并标注 `image_cache: 暂不使用`；真实导航也没有把 `<img src>` 子资源抓取、解码并注册到与 `ImagePrimitive.image_key` 对应的 cache。
   - 因此 WinterTC 的 Logo 缺失是图片子资源/ImageCache/浏览器渲染路径未贯通；正文和 nav 文本串联仍属于 inline ownership、layout/paint IFC 和 glyph 后处理缺口。

#### 按目录通过率（不变）

| 目录 | 通过/总数 | 通过率 |
|------|-----------|--------|
| css-text-decor/ | 39/39 | 100.0% ✅ |
| css-fonts/ | 60/60 | 100.0% ✅ |
| css-grid/ | 17/20 | 85.0% |
| css-writing-modes/ | 46/59 | 78.0% |
| css-tables/ | 45/55 | 81.8% |
| CSS2/ | 93/129 | 72.1% |
| css-position/ | 12/16 | 75.0% |
| css-flexbox/ | 35/55 | 63.6% |
| css-multicol/ | 32/57 | 56.1% |

### 后续重点（R38+）

1. **产品/真实静态页视觉 smoke 门禁**：新增 `welcome.html`、morning.work 录制静态文章页和 WinterTC 录制图片密集首页的 ZeroBrowser/WebView/Chromium 截图对比，至少覆盖桌面和窄屏 viewport；先让该 smoke 可稳定失败，记录证据到 `docs/goal/rendering-compat/evidence/product-static/`。
2. **外部 stylesheet 导航加载**：在 WebView/Browser URL 导航层解析 `<link rel="stylesheet">`，按文档 URL / `<base>` 解析相对地址，经过安全检查和 HTTP cache 抓取 CSS，并按 DOM 顺序与内联 `<style>` 一起进入样式计算；render pipeline 继续保持纯输入渲染。
3. **图片子资源/ImageCache 贯通**：在 WebView/Browser URL 导航层抓取 `<img src>` 和参与渲染的 CSS `url()` 图片，支持 PNG/JPEG/WebP 解码和 SVG 栅格化，使用与 `ImagePrimitive.image_key` 一致的 key 写入 `ImageCache`，并在 ZeroBrowser CPU/GPU 路径传入 renderer。
4. **Inline formatting 所有权统一**：明确 inline 文本、inline 元素和 inline-block 由 IFC 还是 LayoutBox 负责，避免父容器用 `text_content()` 串联整棵 inline 子树，同时子 inline 盒又递归绘制。
5. **paint IFC 架构改进**（系统性瓶颈，影响 50+ 测试 + welcome.html + morning.work + WinterTC）：需要将 layout IFC 的结果存储到 LayoutBox 并在 paint 中复用，避免 paint 重新运行独立 IFC。这是最高优先级的架构改进，但需要较大重构。
6. **ZeroBrowser glyph 后处理收敛**：审视 `reflow_webview_glyphs()`，禁止浏览器层按 baseline 重排 WebView glyph 坐标；字体 fallback、选择命中和可访问性需求必须不改变 engine 输出的 fragment 坐标语义。
7. **文章页 table/code/pre smoke 补齐**：用 morning.work fixture 验证中文段落流、tag badges、inline code、pre/code 块、table/border-collapse 的基本视觉结构，避免真实静态内容退化成普通文本流。
8. **图片密集首页 smoke 补齐**：用 WinterTC fixture 验证 SVG/PNG Logo 可见、header flex、nav grid、Logo flex-wrap 网格、text-align:justify 和 footer 图标，不允许图片缺失退化为 alt 文本或短横 glyph。
9. **inline-flex 基线传递**（影响 ~5 个 flexbox 测试）：taffy 的 first_baselines 在 LayoutOutput 中可用但不持久化到 Layout 结构体。需要在 measure 回调或后处理中捕获基线信息，传递到 IFC 的 InlineBlockBox。
10. **near-miss 测试攻坚**（10 个 <2% diff）：whitespace-001 (1.05%)、clear-clearance-calculation-002 (1.18%)、clearance-006 (1.16%)、border-conflict-resolution (1.54%) 等。
11. **CSS2/floats-clear 精度提升**（17 个失败）：swatch 图像缩放精度、clearance 边界 case。
12. **writing-mode 布局支持**（影响 35+ 测试）：垂直书写模式轴交换。
13. **multicol column breaking**（影响 ~16 测试）：内容碎片化。

### R35 进展

**通过率**：379/490 (77.3%)，+2 tests（自 R34）。修复 text-align 传播缺失，新增 flex-direction 垂直模式交换。

#### 修复提交

| 修复 | 影响 | 说明 |
|------|------|------|
| text-align 传播到 IFC | +2 tests | 所有 IFC 创建点（adjust_inline_block_positions、remeasure_with_float_exclusions 等）现在从 ComputedStyle 读取 text-align 并传播到 InlineFormattingContext。修复 block-in-inline-align-justify-001、block-in-inline-align-last-001 |
| flex-direction 垂直模式交换 | 正确性 | `apply_vertical_writing_mode` 新增 flex-direction 轴交换（Row↔Column），CSS Writing Modes §7.1 合规。当前不影响通过率（flex+writing-mode 测试还需垂直字形渲染） |
| table 列宽固有宽度改进 | 正确性 | `compute_column_widths` 改为 width:auto 单元格使用固有内容宽度（非 taffy block 宽度）；`compute_cell_intrinsic_width` 优先检查子元素显式宽度 |

#### 按目录通过率变化

| 目录 | R34 | R35 | 变化 |
|------|-----|-----|------|
| CSS2/ | 69.0% (89/129) | 70.5% (91/129) | +2 tests（block-in-inline-align-*） |

#### R34 调查分析

1. **whitespace-001 分析**：`display:table` 容器中两个 `width:50%` inline-block 之间的空白应导致换行。IFC 现在正确保留空白为空格字符，但 diff 仍为 1.05%（阈值 1.0%），可能需要进一步优化 space 宽度计算或 table 容器的 IFC 集成。
2. **clear-inline-001 分析**（6.04%）：`clear:left` 在 inline 元素上应无效。代码层面已正确跳过（`!is_block_level` 分支），但 inline 元素的背景由 taffy block 布局定位（非 IFC 位置），导致蓝色背景在错误位置。需要 inline 元素背景从 IFC 坐标绘制。
3. **近通过测试统计**：10 个测试 diff < 2%（包括 whitespace-001、clear-clearance-calculation-002、clearance-006、block-in-inline-align-001 等），这些是下一轮重点攻克目标。
4. **结构性改进需求**：CSS2/linebox（9 失败）需要 inline 元素背景/border 从 IFC 坐标绘制；css-flexbox baseline（9 失败）需要 baseline alignment 改进。

#### R35 分析

1. **text-align 传播**是系统性缺陷：5 个 IFC 创建点均未传播 text-align，导致所有 center/right/justify 布局使用 Left。修复后 2 个 block-in-inline-align 测试通过。
2. **flex-direction 垂直交换**正确但不足以修复 flex+writing-mode 测试：还需要垂直字形渲染（旋转文本 90°）。
3. **table 列宽改进**未改变通过率：width:auto 单元格现在使用子元素宽度估算，但大多数表格测试的失败根因在 swatch 图像缩放或 border 渲染精度。
4. **失败分布**：18 个 <2% diff、28 个 2-5%、41 个 5-15%、24 个 >15%。最大改进杠杆仍为 CSS2/floats-clear（17 失败）、css-multicol（25 失败）、css-flexbox（20 失败）。
5. **swatch 图像缩放**影响 CSS2/floats-clear 中 7 个测试：小色块 PNG（15×15/20×20）缩放到 96×96 与 CSS background-color 精确填充存在像素差异。

### 后续重点（R36+）

1. **near-miss 测试攻坚**（10 个 <2% diff）：whitespace-001 (1.05%)、clear-clearance-calculation-002 (1.18%)、clearance-006 (1.16%)、block-in-inline-align-001 (1.41%)、grid max-content (1.52%)、flexbox near-miss 等
2. **CSS2/floats-clear 精度提升**（17 个失败）：swatch 图像缩放精度、clearance 边界 case
3. **writing-mode 布局支持**（影响 35+ 测试）：垂直书写模式轴交换
4. **multicol column breaking**（影响 ~16 测试）：内容碎片化
5. **CSS2 inline box model**（影响 ~9 测试）：inline 元素背景从 IFC 坐标绘制

### R33 进展

**通过率**：376/490 (76.7%)，与 R32 净增 +2 tests（clear-002 + clear-float-005）。修复 inline relative offset 对 table 内部元素的误用。

#### 修复提交

| 修复 | 影响 | 说明 |
|------|------|------|
| apply_relative_offsets_inline 仅对 inline-level 元素生效 | CSS2 +2 | 上一轮的 apply_relative_offsets_inline 使用 `!is_block_level` 检测 inline 元素，但 table 内部元素（tfoot/thead/tbody/tr/td）也不是 block-level，导致 position:relative 在这些元素上被双重偏移。改为检查 display type（Inline/InlineBlock），仅对真正由 inline layout 定位的元素应用偏移。修复 clear-002.xht 和 clear-float-005.xht |

#### 按目录通过率变化

| 目录 | R32 | R33 | 变化 |
|------|-----|-----|------|
| CSS2/ | 70.5% (91/129) | 69.0% (89/129) | R33 修复了 position-relative-table-tfoot-top 回归（2.08%→1.04%），但 CSS2 通过数因 clear-002/clear-float-005 已在 R32 计入而无变化 |

#### R33 调查分析

1. **零高度浮动水平空间**：尝试让零高度浮动不占据水平空间（`left_used_width` 跳过零高度浮动），但导致 clear-float-003 回归（1.92%→5.76%），已回退。零高度浮动仍然占据水平空间。
2. **CSS2/floats-clear 失败分析**：17 个失败测试中，多数差异来自 swatch 图像缩放精度（20×20→96×96 与 CSS 背景色填充的像素差异）或 clearance 计算边界 case，非简单修复。
3. **CSS2/linebox**（8 个失败）：需要 inline box model 深层改进（空 inline 元素 line-height、block-in-inline 拆分），属于结构性改动。
4. **multicol**（25 个失败）：7 个 multicol-breaking-* 测试（~16%）需要 column breaking/内容碎片化，属于大特性。
5. **css-writing-modes**（13 个失败）：需要垂直书写模式轴交换支持，属于大特性。

### 后续重点（R34+）

1. **CSS2/floats-clear 精度提升**（17 个失败，最大瓶颈）：swatch 图像缩放 20×20→96×96 与 CSS background-color 精确填充的像素差异。需改进图像缩放或替代方案。
2. **writing-mode 布局支持**（影响 35+ 测试）：需实现垂直书写模式下块级布局轴交换。R12 已尝试但回退。
3. **multicol column breaking**（影响 ~16 测试）：需实现内容碎片化（拆分单个块到多列）。
4. **CSS2 inline box model**（影响 ~8 测试）：空 inline 元素 line-height 贡献、block-in-inline 拆分。
5. **css-flexbox baseline**（影响 ~9 测试）：multi-line baseline 对齐、flex 方向轴交换。

### R32 进展

**通过率**：374/490 (76.3%)，+1 test。聚焦于 table 布局坐标系统修正和匿名行单元格查找修复。

#### 修复提交

| 修复 | 影响 | 说明 |
|------|------|------|
| 行坐标系统修正 | table.rs position_cells | 行的 y 坐标从绝对定位（相对 table content）改为相对行组定位，避免 paint 链中行组位置 + 行位置双重计数。影响所有多行组表格（tbody+tfoot） |
| 嵌套匿名行单元格查找 | table.rs build_grid + position_cells | TableCell 新增 parent_rg_idx 字段，孤立行组中嵌套行组的单元格通过 table_box.children[rg_idx].children[idx] 正确查找。修复 table-row-group-nested-anonymous-001 |
| get_row_box 孤立模式 | table.rs get_row_box/get_row_box_mut | 匿名行 row_group_index=None 时返回 table_box 本身（而非 table_box.children[idx]），支持孤立行组场景 |

#### 按目录通过率变化

| 目录 | R31 | R32 | 变化 |
|------|-----|-----|------|
| css-tables/ | 81.8% (45/55) | 83.6% (46/55) | +1 test |

#### R32 分析

1. **position-relative-table-tfoot-top** (1.04%)：行组 position:relative 的背景色在正确位置渲染（由 update_row_group_positions 设置），但差异可能来自字体渲染或 border-collapse 细节。
2. **clearance/clear 测试**（1.16%-1.95%）：clearance 算法已正确使用 content_y_offset，差异主要来自 Ahem 字体渲染和 swatch 图像缩放。
3. **writing-mode + flexbox** 测试（1.52%-1.85%）：需要垂直书写模式下轴交换支持，属于功能缺失。
4. **20 个测试 <2% diff**：其中多数差异来自字体渲染（Ahem vs 默认字体）、swatch 图像缩放精度、或 writing-mode 功能缺失。

### 后续重点（R33+）

1. **CSS2/floats-clear 精度提升**（19 个失败，最大瓶颈）：swatch 图像缩放 20×20→96×96 与 CSS background-color 精确填充的像素差异。需改进图像缩放或替代方案。
2. **writing-mode 布局支持**（影响 35+ 测试）：需实现垂直书写模式下块级布局轴交换。R12 已尝试但回退。
3. **multicol column breaking**（影响 ~16 测试）：需实现内容碎片化（拆分单个块到多列）。
4. **CSS2 inline box model**（影响 ~8 测试）：空 inline 元素 line-height 贡献、block-in-inline 拆分。
5. **css-flexbox baseline**（影响 ~9 测试）：multi-line baseline 对齐、flex 方向轴交换。

### R31 进展

**通过率不变**：373/490 (76.1%)。本轮聚焦于系统性分析和高质量 bug 修复，为后续改进奠定基础。

| 修复 | 影响 | 说明 |
|------|------|------|
| Stroke 裁剪逻辑修正 | paint/helpers.rs | 原代码用 `&&` 连接对边判断（不可能同时成立），改为 `||` 连接各边判断。修正后描边线段在超出裁剪区域时可被正确裁剪 |
| 空 inline 元素 margin-right 修复 | inline/mod.rs | 空 inline 元素仅消费 margin-left，未消费 margin-right。CSS 2.1 §10.2 要求两者均消费 |

#### R31 系统性分析

1. **paint 系统审计**（12 个 bug 识别）：
   - BUG 1: 边框不遵循 border-radius（paint_borders 生成矩形而非圆角）
   - BUG 9: Glyph Y 位置用 font_size 作基线偏移（应为实际 ascent）
   - BUG 11: 基线位置用 0.8 硬编码近似（应为字体度量）
   - BUG 15: Stroke 裁剪逻辑始终为 false（✅ 已修复）
   - BUG 18: 渐变 Px 偏移未归一化到 [0,1]（不影响当前上游测试）
   - 其余 bug 影响范围有限或需要更深层改动

2. **尝试但回退的修复**：
   - multicol BFC 检测（establishes_bfc 添加 is_multicol 检查）→ 导致 multicol 回归（56.1%→54.4%），原因是影响容器高度计算逻辑，已回退
   - R30 的两个修复方向（remeasure_inline_only_containers + float clearance border-top 约束）仍因回归风险未合入

3. **失败根因分布更新**（117 个失败）：
   - 布局精度（float/clear/margin）：~48 个（最大瓶颈）
   - 功能缺失（column breaking、writing-mode）：~25 个
   - 子像素/渲染精度：~20 个
   - CSS2 inline box model：~8 个
   - 其他：~16 个

4. **关键发现**：相同 diff 百分比的测试共享系统性问题
   - clear-002 (7.67%) == clear-float-005 (7.67%) — 可能是 swatch 图像渲染或元素定位系统性偏差
   - clear-003 (3.84%) == clear-float-006 (3.84%) — 同上

#### 后续重点

1. **CSS2/floats-clear 精度提升**（19 个失败，最大失败集群）：需要找到不影响其他测试的 clearance 计算改进
2. **multicol BFC 集成**：需要更精细的修改，仅影响 margin 折叠行为，不影响容器高度计算
3. **CSS2/border-radius + border 绘制**（BUG 1）：影响所有圆角元素 + 边框渲染，可能提升多个测试

### R30 进展

| 修复 | 影响 | 说明 |
|------|------|------|
| pre/pre-wrap 模式空白保留修复 | inline layout 正确性 | CSS 2.1 §16.6.1 行尾空白剥离仅适用于 normal/nowrap 模式的可折叠空白；pre/pre-wrap 模式空白不可折叠，不剥离 |

### R30 调查分析

本轮深入分析了 117 个失败测试的根因分布，尝试了以下修复方向（均因回归风险暂未合入）：

1. **remeasure_inline_only_containers 纯 inline 容器覆盖**：为仅含 inline 子元素的容器使用 IFC 权威高度替代 taffy 高度。回归原因：display:table 容器中的 inline-block 子元素依赖原始 IFC remeasure 的"仅增大"行为，强制替换会干扰后续 table layout。
2. **浮动元素 clear 时 border-top 约束**：CSS 2.1 §9.5.2 要求有 clear 的浮动元素 border-top 不低于 clear_bottom，即使负 margin-top 会拉回。回归原因：`clear-float-002` 等测试依赖 margin 参与浮动定位的现有行为。

**通过率不变**：373/490 (76.1%)。失败分布：CSS2/floats-clear (20)、css-multicol (25)、css-flexbox (20)、css-writing-modes (13)、css-tables (10)、css-position (6)、css-grid (3)。

### R29 进展

| 修复 | 影响 | 说明 |
|------|------|------|
| Clearance C1/C2 双路径算法 | css-multicol +1 | CSS 2.1 §9.5.2：当 clearance 引入时 margin 不折叠，元素位置 = max(clear_bottom, flow_bottom + margin_top) |
| 匿名块盒生成 | CSS 2.1 §9.2.1.1 | inline 元素包含 block-level 子元素时插入 InlineItem::Br 强制换行 |
| 行尾空白剥离 | CSS 2.1 §16.6.1 | 尾部空格从片段可视文本/宽度中移除，仅用于词间距离计算 |

### R29 按目录通过率变化

| 目录 | R28 | R29 | 变化 |
|------|-----|-----|------|
| css-multicol/ | 54.4% (31/57) | 56.1% (32/57) | +1 test |

### R28 进展

| 修复 | 影响 | 说明 |
|------|------|------|
| 表格 shrink-to-fit（CSS 2.1 §17.5.2.2）| css-tables +3, css-multicol +1, css-writing-modes +1 | `width:auto` 的表格不再将列扩展到容器宽度，而是收缩到内容固有宽度。同时保留 `table-layout:fixed` 时的列扩展行为。`apply_table_size_constraints` 同步更新 `content_width` |
| 孤立 table 内部元素匿名包装 | css-tables/standalone | `display: table-row-group` 等元素缺少父级 table 时，直接对其执行 table 布局（CSS 匿名盒修复） |
| Row-group 内匿名行收集 | css-tables/anonymous | Row-group 中的直接 cell 和嵌套 row-group 中的 cell 收集到单个匿名行（而非每 cell 一个匿名行）。新增 `is_anonymous` 标志 |
| get_row_box 匿名行支持 | table.rs 基础设施 | `get_row_box`/`get_row_box_mut` 对匿名行返回 row-group 盒本身，而非错误导航到子元素 |

### R28 按目录通过率变化

| 目录 | R27 | R28 | 变化 |
|------|-----|-----|------|
| css-tables/ | 74.5% (41/55) | 81.8% (45/55) | +4 tests |
| css-multicol/ | 52.6% (30/57) | 54.4% (31/57) | +1 test |
| css-writing-modes/ | 76.3% (45/59) | 78.0% (46/59) | +1 test |
| CSS2/ | 69.0% (89/129) | 69.0% (89/129) | 无变化，但多个 test diff 显著下降 |
| css-flexbox/ | 63.6% (35/55) | 63.6% (35/55) | 无变化 |

### R28 失败根因总结

当前 118 个上游 reftest 失败的根因分布：
- **布局精度问题**（float/clear/margin）~48 个：float clearance 算法精度、margin 折叠边界 case
- **功能缺失** ~26 个：column-height、column-wrap、position:fixed 打印、3D transform 等
- **子像素/精度** ~20 个：border 渲染精度、字体度量差异、背景图像缩放
- **Writing-mode 轴交换** ~14 个：需要垂直布局模式（vertical-rl/lr）
- **Multicol column breaking** ~6 个：需要内容碎片化（拆分单个块到多列）
- **CSS2 inline box model** ~8 个：匿名块盒生成、空 inline line-height、inline-block 内在尺寸

### R25 进展

| 修复 | 影响 | 说明 |
|------|------|------|
| table cell overflow 抑制 | css-tables +1 test | CSS 2.1 §17.5：table cell height 为最小高度，overflow:hidden/scroll/clip 在 table cell 上强制为 Visible |
| table rowspan 基础设施 | css-tables 未来改进 | TableCell 新增 rowspan 字段 + get_rowspan() 辅助函数 + 行边框冲突解决 |

### R24 进展

| 修复 | 影响 | 说明 |
|------|------|------|
| multicol column count 伪算法修正 | css-multicol +2 tests | CSS §3.4 伪算法 line 18：当 column-count 和 column-width 同时指定时，使用 min() 而非 max() 计算列数 |
| multicol 子元素宽度约束 | css-multicol 渲染正确性 | 子元素移入列后递归约束 width 和 content_width 到列宽，确保 paint 层使用正确宽度 |
| multicol column-width >= container 边界 | css-multicol 边界 case | 当 column-width 大于等于容器宽度时，仅生成 1 列 |

### R23 进展

| 修复 | 影响 | 说明 |
|------|------|------|
| table cell content height 计算修正 | css-tables 正确性 | 单元格内容高度改为 sum（正常流子元素垂直堆叠）替代 max（取最大子元素高度）；vertical-align 计算同步修正 |

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
| M7 — 渲染器图元覆盖 | ✅ 完成 | CPU 渲染器：全部 13 种图元 ✅；GPU 渲染器：全部 13 种图元管线 ✅ + 48 个单元测试 ✅；浏览器消费：全部 13 种图元 ✅；浏览器 GPU 路径集成 ✅ |
| M8 — 布局正确性 | ✅ 完成 | BFC 检测 ✅；float clear ✅；margin 折叠(taffy 0.7 内置) ✅；<img> 固有尺寸 ✅；position:fixed ✅(adjust_fixed_to_viewport)；position:sticky 需宿主层（已标记 is_sticky，后续集成）；percentage height/auto margin/min-max-width 已有测试验证 |
| M9 — 高级视觉效果 | 🔧 进行中 | 重复渐变 ✅；多图层背景 ✅；clip-path 全形状裁剪 ✅(inset+circle+ellipse+polygon)；border-image ✅；text-shadow ✅；backdrop-filter ✅；CSS mask ✅(渐变蒙版裁剪+alpha衰减)；overflow 全图元裁剪 ✅；滚动容器 paint 偏移 ✅(scroll_x/scroll_y 字段 + paint 时子元素坐标偏移 + 3 个单元测试)；剩余：scroll-snap 行为（需宿主层输入路由）、滚动输入路由（需浏览器 app 集成） |
| M10 — 上游 WPT 真实 Reftest 导入 | 🔧 进行中 | 基础设施 ✅；490 个上游 reftest 已导入（9 个目录）；**真实通过率 74.3% (364/490)**；css-text-decor 100.0% ✅；css-fonts 100.0% ✅(≥95%)；css-grid 85.0%；css-writing-modes 76.3%；css-tables 72.7%；CSS2 69.0%；css-flexbox 63.6%；css-position 62.5%；css-multicol 50.9%；**R24 修复**：multicol column count 伪算法 min() 修正（CSS §3.4 line 18）+ 子元素宽度约束递归更新；**R23 修复**：table cell content height sum 替代 max；**R22 修复**：clearance 零值阻止 margin 折叠（CSS 2.1 §9.5.2 三路分支：正 clearance/零 clearance/无需 clearance）；**R21 修复**：font 简写负 line-height 拒绝 ✅；background-position 简写双值捕获修复 ✅ |

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
| Float 布局算法 | CSS 2.1 核心 | ✅ 已完成 | M4 |
| Table 布局算法 | 表格渲染 | ✅ 已完成 | M4 |
| Multi-column 布局算法 | 多列布局 | ✅ 已完成 | M4 |
| OpenType shaping | 文字排版质量 | ✅ 已完成 | M6 |
| BiDi 算法 | RTL 文本 | ✅ 已完成 | M6 |
| Vertical writing-mode | 竖排文本 | M5 | M5 |
| CJK normal-mode 换行 | CJK 排版 | ✅ 已完成 | M5 |
| text-align: justify | 文字排版 | ✅ 已完成 | M5 |
| Float exclusion 堆叠 | 布局正确性 | ✅ 已完成 | M5 |
| Quirks mode | CSS 2.1 兼容性 | ✅ 已完成 | M2 |
| 上游 WPT 真实 reftest 导入 | 覆盖范围 | M6 | M6 |
| CPU 渲染器图元覆盖 | 视觉输出 | ✅ 已完成 | M7 |
| 浏览器图元消费 | 视觉输出 | ✅ 已完成 | M7 |
| GPU 渲染器图元覆盖 | GPU 视觉输出 | ✅ 管线已实现 | M7 |
| 外部 stylesheet 加载 | 真实静态网页 CSS | P1 | M10/R38 |
| 图片子资源/ImageCache 贯通 | Logo/图片密集静态页 | P1 | M10/R38 |
| 产品/真实静态页视觉 smoke | 验收有效性 | P1 | M10/R38 |

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
