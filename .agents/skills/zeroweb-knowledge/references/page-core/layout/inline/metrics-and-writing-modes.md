# 度量、基线、竖排、ruby 与替换元素

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位字体行度量、advance、atomic inline、ruby 和竖排。IFC 的行高与字宽是两个依赖接口；字体加载与 shaping 细节沿 [字体知识](../../../graphics/render-foundation/fonts/index.md) 查，不在这里复制图形底座实现。

## Gotchas
- `LineMetrics.descent` 使用 fontdue 的负值约定，normal 行高为 ascent−descent+gap；run 的 glyph_descent 却保存非负幅长，两种表示不可直接互加。 (`crates/layout-engine/src/inline/font_metrics.rs`, `crates/layout-engine/src/inline/inline_types.rs`)
- `resolve_font_metrics` 没有 provider 时回退 Ahem 1.0、其他 1.164 的 normal 比率；带 provider 版本才可能读真实度量，显式 line-height 不因注入 provider 自动改变。 (`crates/layout-engine/src/inline/text_metrics.rs`)
- `font-size-adjust` 对非 Ahem 保留 TextRun 的 specified 字号，让 shaping 做 per-face 调整，只用 adjusted primary size 推 normal 行高；Ahem 的合成字形路径另行直接调字号。 (`crates/layout-engine/src/inline/text_metrics.rs`)
- 字体 family 解析对空/全失败列表先 serif 再 sans-serif，quoted generic 名称不走 generic resolver；仅按第一个字符串查表会漏 face 列表和 weight/style/stretch fallback。 (`crates/layout-engine/src/font_resolution.rs`)
- normal 模式的 U+200B 并未在 estimate 通用入口归零，而 ZWJ/ZWNJ/WJ/FEFF 恒零；preserve 模式通过断词丢弃 ZWSP，不能用“所有零宽字符同处理”概括实现。 (`crates/layout-engine/src/inline/text_metrics.rs`)
- 竖排 fragment 的 width 是列宽/line-height，height 是 inline advance；向 paint 的 line-height 槽存 frag.height 会让列位置以 advance 作为行高。 (`crates/layout-engine/src/inline_metric_storage.rs`)
- vertical IFC 可用深度取 content_height，而 block extent 取 content_width；有 text-decoration/emphasis 的子树仍有旧宽度 gate，不能无条件应用轴交换。 (`crates/layout-engine/src/inline_finalization.rs`)
- `vertical_block_flow.rs` 头部 dormant 状态和 inline/block 轴命名已落后；现行函数明确物理 width 为 block-size，且协调器有 sizing 与位置接线，应按当前实现判读。 (`crates/layout-engine/src/vertical_block_flow.rs`, `crates/layout-engine/src/engine.rs`)
- R5118 历史报告把 border-width-applies-to-008 的 inline 盒 h=90、Chrome 期望270归入非替换 inline 垂直边框几何缺口；这不是给行高直接加上下 border 的新契约，须分别核对行盒、装饰盒与基线。break_lines.rs 的水平 padding 推进和空 inline 注释不能证明此分支完整；该轮没有代码修复，本次只核对来源，未复现。 (`docs/goal/rendering-compat.md`, `crates/layout-engine/src/inline/break_lines.rs`, `git:3b548ed7804ecf0e050da00557ee8adb654e615e`)

## Architecture
- AdvanceSource 分单字符、整文本、ordered face list 和含 size-adjust/variation context 的接口；默认整文本只是逐字相加，实现需覆写才能提供 shaping 上下文。 (`crates/layout-engine/src/inline/text_metrics.rs`)
- `ruby_rt_ascent` 与 fallback 字体 glyph ascent/descent 作为 run 独立贡献进入行盒；未注册 fallback 回调或门控关闭时 glyph 度量为零，而非拿 primary font 猜测。 (`crates/layout-engine/src/inline/inline_types.rs`)
- vertical block-flow 的位置与尺寸使用同一 Σ 子 outer-width/max outer-height 模型；父子均竖排的放开分支默认开启但排除 float 子树，避免与浮动原几何冲突。 (`crates/layout-engine/src/vertical_block_flow.rs`)
- vertical atomic inline 终化注入真实 LayoutBox 宽高和 central baseline；水平路径不套用此注入，尺寸由前置多个重测 pass 承担。 (`crates/layout-engine/src/inline_finalization.rs`)

## Decisions
- FontMetricProvider 使用 owned Rc trait 句柄而非借用 FontLoader，避免给 owned IFC 增加生命周期，并允许桩度量；它和 advance provider 分离，不能用行高桥修换行宽度。 (`crates/layout-engine/src/inline/font_metrics.rs`)
- 单点替换 estimate 字宽的历史回退原因是 layout/paint 字体源分裂和 test/ref 换行敏感差异；现行真实 advance 必须与 face 解析和 intrinsic 同源，历史测量数字不是本次实跑。 (`crates/layout-engine/src/inline/text_metrics.rs`)
