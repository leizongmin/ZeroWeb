# 盒树、尺寸、定位、浮动与 margin

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位盒树生成、固有尺寸、浮动排斥和包含块问题。入口为 `build_layout_tree_with_r109`、`LayoutEngine::compute`；尺寸初算、Taffy 重算和 LayoutBox 后处理是不同阶段，最终行内几何见 [IFC 终化](inline/finalization-and-reuse.md)。

## Gotchas
- `margin_collapse.rs` 开头“不需要后处理”只适用于 Taffy 基础折叠；协调器另有 clearance containment、min-height 穿透剥离和兄弟位移，不能据旧注释删除这些步骤。 (`crates/layout-engine/src/margin_collapse.rs`, `crates/layout-engine/src/engine.rs`)
- float 表格必须在浮动定位前先完成表尺寸求解，否则 specified width 小于 min-content 时会误判还能贴排；普通全量表格布局仍在浮动之后运行。 (`crates/layout-engine/src/engine.rs`)
- inline-block 排斥浮动的 x 修正放在终末阶段以免被后续重定位覆盖；多个浮动一起计算可行区间，不能逐浮动独立收缩宽度。 (`crates/layout-engine/src/float_positioning.rs`)
- `ZW_FLOAT_PUSH_BELOW=1` 才启用 declared-width inline-block 在左右浮动夹空不可行时下推的分支，默认关闭；不能把注释里的预期效果当默认行为。 (`crates/layout-engine/src/float_positioning.rs`)
- `is_fixed_cb_containment` 识别真实 layout/paint containment，却不让 `container-type` 隐含 containment 捕获 fixed 后代；视觉 filter/transform/perspective 又有独立包含块谓词。 (`crates/layout-engine/src/engine/abspos.rs`)
- 根节点缺少父布局上下文，水平根的 inset、auto margin 居中和固定 margin 分别补算；table 根跳过普通根居中，留给表格收缩后的居中路径。 (`crates/layout-engine/src/engine.rs`)
- intrinsic 模块“仅诊断、不参与布局”的头注释已落后：协调器调用 `apply_intrinsic_content_sizing`；定位尺寸实际行为须跟调用链，而不是照抄模块简介。 (`crates/layout-engine/src/intrinsic_sizing.rs`, `crates/layout-engine/src/engine.rs`)

## Architecture
- `LayoutBox.width/height` 表示 border-box，而 `content_*` 单独存储；abspos 百分比重解析同时回写两个口径，避免纠正包含块后留下旧内容尺寸。 (`crates/layout-engine/src/engine/abspos.rs`)
- R109 用注册表保存匿名块片段所属 DOM 节点；测量闭包只布局该片段，并沿 context 上行找拆分宿主，不能以宿主全部文本代替片段文本。 (`crates/layout-engine/src/engine.rs`, `crates/layout-engine/src/tree.rs`)
- BFC 判定还依赖 `is_layout_container`、匿名 table 根、多列和 caption 标记，不仅是 overflow/float/position；这些标记供浮动后处理隔离 margin 与包含浮动。 (`crates/layout-engine/src/margin_collapse.rs`)

## Decisions
- 非 BFC 的 auto-height 在浮动定位之后自底向上排除浮动贡献，再按最终收缩宽度做 plain block aspect-ratio 传高，阶段顺序防止旧宽度推导错误高度。 (`crates/layout-engine/src/engine.rs`)
- intrinsic advance 与字体 resolver 从布局线程发布到线程本地上下文；未注入时回落字符估算，`ZW_INTRINSIC_REAL_ADVANCE=0` 阻止发布，独立布局测试和生产字体口径不能混为一谈。 (`crates/layout-engine/src/intrinsic_sizing.rs`)

## Patterns
- 基础尺寸纠正先分别求值各 pass，再合并 changed 标记重跑 Taffy；直接用 `||` 串执行有副作用的 pass 会让前一成功跳过后一。 (`crates/layout-engine/src/engine.rs`)
- intrinsic 测量对 inline 贡献求和、block 贡献取最大，叶盒显式宽度作为回退；这与只看子内容的 `table_shrink` 辅助函数不同。 (`crates/layout-engine/src/intrinsic_sizing.rs`)
- 纯 inline 上下文的 br 交给 IFC；混合上下文还判断其父块是否有后续 in-flow 兄弟，避免零高父块使后续兄弟重叠，OOF 兄弟不算触发条件。 (`crates/layout-engine/src/tree.rs`)
