//! 行内格式化上下文实现。
//!
//! 处理行内级内容的布局：文本节点、inline 元素、行换行。
//! Taffy 仅支持 Block/Flex/Grid，行内布局需要自行实现。
//! 支持文本对齐方式：left、center、right、justify。

// R342：文本度量与估计辅助抽出（2000 行规则 + Phase A 准备），通过 glob 再导出保持 API。
mod text_metrics;
pub use text_metrics::*;

// R830：行内布局核心数据类型抽出（2000 行规则 + Phase A IFC 统一 Phase 5 准备），
// 通过 glob 再导出保持 `crate::inline::TextRun` 等 API 路径不变（纯移动，零行为变化）。
mod inline_types;
pub use inline_types::*;

// Phase A §12.6 step-1：font-metric 桥接（FontLoader → IFC 真实行度量）。
// 仅 trait + FontLoader 实现 + IFC 可选字段，dormant 默认零回归（step-2 才消费）。
mod font_metrics;
pub use font_metrics::*;

// Phase 2a step-1：multicol 列碎片化上下文（IFC 把行盒碎片化到列的输入）。
// 仅数据结构 + IFC dormant 字段，默认零回归（step-2 才消费）。
mod column_fragmentation;
pub use column_fragmentation::*;

// Phase 2a step-2：multicol 列碎片化算法（纯函数，零生产调用方，net 0）。
// step-2 commit 2 在 layout 侧接线后才有生产调用方。
mod column_fragmentation_flow;
pub use column_fragmentation_flow::*;

use std::collections::HashMap;
use std::rc::Rc;

use zero_css_parser::values::{DisplayValue, LengthValue, OverflowValue, PositionValue, VerticalAlignValue};

use zero_dom::{Document, NodeId, NodeKind};

use zero_style_system::{ComputedStyle, TextAutospaceValue, TextTransformValue};

/// 读取已解析的 LengthValue（Px）为 f32，非 Px（Auto/Percentage/Calc…）返回 0。
/// 用于 inline-block margin 读取（margin 已在 compute_style 解析为 Px）。
fn length_px(lv: &LengthValue) -> f32 {
    match lv {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    }
}

/// 行内格式化上下文 — 负责将行内内容排列成行盒。
#[derive(Debug, Clone)]
pub struct InlineFormattingContext {
    /// 包含块的可用宽度。
    pub container_width: f32,
    /// 文本对齐方式。
    pub text_align: TextAlign,
    /// 末行对齐方式（CSS text-align-last）。None 表示跟随 text-align。
    pub text_align_last: Option<TextAlign>,
    /// 是否允许在单词内断行（overflow-wrap: break-word / anywhere）。
    pub break_word: bool,
    /// 是否禁止换行（white-space: nowrap / pre 时为 true）。
    pub no_wrap: bool,
    /// 是否保留空白字符序列（white-space: pre / pre-wrap 时为 true）。
    pub preserve_whitespace: bool,
    /// 是否在换行符 `\n` 处强制断行（white-space: pre-line：折叠空白序列但保留换行符为
    /// 强制断行点，CSS Text 3 §4.2）。介于 normal（折叠含 `\n`）与 pre-wrap（保留全部空白）
    /// 之间：空白序列仍折叠为单空格，但 `\n` 触发强制断行。
    pub break_at_newline: bool,
    /// CSS word-break 行为。
    pub word_break: WordBreakMode,
    /// CSS text-autospace 行为（CSS Text 4 §8，表意文字与字母/数字间 0.125em 间距）。
    /// IFC 级（容器样式），默认 `NoAutospace`（零回归）。
    pub text_autospace: TextAutospaceValue,
    /// 首行文本缩进（CSS text-indent，px）。仅影响第一行的起始 x 坐标。
    pub text_indent: f32,
    /// CSS tab-size（px）— 制表符展开宽度。默认 8 个空格宽度。
    pub tab_size: f32,
    /// 浮动排除区域 — 浮动元素占据的空间，文本需环绕排列。
    pub float_exclusions: Vec<FloatExclusion>,
    /// 生成的行盒列表。
    pub lines: Vec<LineBox>,
    /// R2431 line-clamp（CSS Overflow 4）：夹到 N 行（容器 `line-clamp: Count(n)` → `Some(n)`）。
    /// `layout()` 读容器样式设此字段，`break_items_into_lines` 后 truncate `self.lines` 到 N
    /// 并置 `clamped`。box 高度由下游从 lines 推 → 自然 = N 行。kill-switch `ZW_LINE_CLAMP=0`。
    pub line_clamp: Option<usize>,
    /// line-clamp 是否截断了行（第 N 行后有更多内容 → 第 N 行末须渲 ellipsis，slice 2 消费）。
    pub clamped: bool,
    /// 垂直书写模式（vertical-rl 或 vertical-lr）。
    ///
    /// 当为 true 时，字符沿 y 轴向下推进，"行"变为垂直列，
    /// 列沿 x 轴排列。fragment 的坐标系统不变（x=水平，y=垂直），
    /// 但"换行"的触发条件和推进方向交换。
    pub vertical: bool,
    /// 垂直模式下列排列方向：vertical-rl 时列从右到左排列。
    ///
    /// 仅当 vertical=true 时有效。当为 true 时，第一列在右侧，
    /// 后续列向左推进。fragment 的 x 坐标会相应镜像。
    pub vertical_rtl: bool,
    /// 垂直模式下容器的 **block 轴 extent**（= content_width，x 方向跨度）。
    ///
    /// R1122：`break_items_into_columns` 的 vrl 列 x 定位须用 block 轴 extent（列沿 x 排列
    /// 的可用宽度），**非 `container_width`**（vertical 下 = content_height，是 inline 深度/y 轴，
    /// 量纲错）。旧实现误用 container_width 致单列 caption col.y=784-50=734 → paint off-screen
    /// → caption-side-vrl 文本完全不绘制（R1120）。本字段由 compute_final/paint 在 vertical 时
    /// 设为 content_width；默认 = container_width（horizontal / 未设 caller 零回归）。
    pub block_extent: f32,
    /// inline-block 元素的预计算尺寸（来自 LayoutBox / taffy 布局结果）。
    ///
    /// 当 CSS 属性 width/height 为 Auto 时，inline-block 的尺寸由其内容决定，
    /// IFC 无法自行测量，需要外部布局结果提供。
    pub inline_block_sizes: HashMap<NodeId, (f32, f32)>,
    /// `<img>` 替换元素的解码固有尺寸（intrinsic w/h，px），按 img NodeId 索引。
    ///
    /// **R1578 inline>inline-IMG 固有尺寸打通**：`collect_inline_items` img 分支在
    /// HTML attr / CSS computed / 百分比 / `inline_block_sizes` 全部无法给出两侧
    /// 维度时，若 img 恰有一侧已知（显式 width 或 height），用本 map 的固有宽高比
    /// 推导缺失侧（解「inline 元素包裹 auto-width img 致父容器塌缩 h=0」，如
    /// wintertc footer `<p><a><img class="h-6 inline-block"></a></p>`）。
    /// 默认空 map = 不推导（零回归）；由 measure path `measure_text_content` 经
    /// `with_img_intrinsic_sizes` 注入（`img_intrinsic_sizes` 来自 engine 输入）。
    /// env-gated `ZW_IFC_IMG_INTRINSIC=0` 关闭（default-on R1578 slice 2）。
    pub img_intrinsic_sizes: HashMap<NodeId, (f32, f32)>,
    /// 默认字体度量 — 当 styles HashMap 中找不到元素样式时使用。
    ///
    /// 这主要用于 paint 系统的 IFC，因为 paint 系统传入空的 styles HashMap。
    /// 设置此值后，文本节点在找不到父元素样式时会使用此默认值
    /// 而非硬编码的 16px/19.2px。
    pub default_font_metrics: Option<(f32, f32)>,
    /// 包含块（行内格式化上下文的宿主块容器）自身的 font-size（px）。
    ///
    /// CSS 2.1 §10.8.1：行盒的 strut（隐式行内盒）由块容器自身的
    /// font-size/line-height 决定，与行内内容无关。strut 的 ascent 用于
    /// 行盒基线计算的下限。`apply_vertical_alignment` 用此值推导 strut ascent，
    /// 而非用行盒实测高度（行盒高度会被高大的原子行内盒撑高，导致 strut
    /// 被错误放大，进而把基线偏低的原子盒压到行盒下方）。
    /// 默认 16px；`layout()` 从容器样式中读取真实值。
    pub container_font_size: f32,
    /// 逐文本节点的字体大小覆盖（key = 文本节点的父元素 NodeId）。
    ///
    /// paint IFC 传入空的 styles HashMap，导致所有文本使用 16px 默认字体度量，
    /// 行断计算与 layout IFC 不一致。此字段存储 layout IFC 为每个文本节点
    /// 计算的实际 font_size，以父元素 ID 为键（因为 collect_inline_items
    /// 查找的是文本节点的父元素样式）。
    /// 当 styles 中找不到父元素样式且此映射有对应条目时，使用映射中的
    /// font_size 而非 16px 默认值，使字符宽度计算更准确。
    pub font_size_overrides: HashMap<NodeId, f32>,
    /// paint IFC 的父元素/inline 元素到已解析字体 ID 映射。
    pub font_id_overrides: HashMap<NodeId, u32>,
    /// 文本/inline owner 到有序 CSS face ID 列表的映射。
    pub font_ids_overrides: HashMap<NodeId, Vec<u32>>,
    /// 文本/inline run 的 `font-size-adjust` 计算值。
    pub font_size_adjust_overrides: HashMap<NodeId, zero_style_system::FontSizeAdjustValue>,
    /// 逐文本节点的 Ahem 字体标志覆盖（key = 文本节点的父元素 NodeId）。
    ///
    /// paint IFC 传入空的 styles HashMap，无法检测 Ahem 字体，
    /// 导致所有文本使用 0.55×font_size 的 ASCII 字符宽度估算。
    /// 当文本实际使用 Ahem 字体时，字符宽度应为 1.0×font_size，
    /// 此覆盖确保 paint IFC 使用正确的字符宽度。
    pub is_ahem_overrides: HashMap<NodeId, bool>,
    /// 逐文本节点的 letter-spacing 覆盖（key = 文本节点的父元素 NodeId）。
    ///
    /// paint IFC 传入空的 styles HashMap，无法获取 letter-spacing，
    /// 导致所有字符使用 0 的默认间距。此覆盖确保 paint IFC 使用正确的
    /// letter-spacing 值进行字符宽度和行断计算。
    pub letter_spacing_overrides: HashMap<NodeId, f32>,
    /// 逐文本节点的 line-height 覆盖（key = 文本节点的父元素 NodeId）。
    ///
    /// paint IFC 传入空的 styles HashMap，无法获取 line-height，
    /// 回退为 font_size * 1.2 近似值。对于使用自定义 line-height 的元素，
    /// 近似值导致行盒高度与 layout IFC 不一致，进而影响垂直定位。
    /// line-height 仅影响垂直定位（行盒高度），不影响行断（水平宽度），
    /// 因此传递此覆盖不会改变行断行为。
    pub line_height_overrides: HashMap<NodeId, f32>,
    /// 内联元素的 (font_size, line_height) 覆盖（key = 元素自身的 NodeId）。
    ///
    /// 与 font_size_overrides/line_height_overrides 不同（以文本节点的父元素为键），
    /// 此映射以内联元素自身的 NodeId 为键。供 collect_inline_items 中
    /// 处理内联元素（非文本节点）时使用。
    /// 这些属性仅影响垂直定位（行盒高度），不影响行断（水平宽度）。
    pub inline_element_metrics: HashMap<NodeId, (f32, f32)>,
    /// 行内级盒的基线覆盖（key = 元素 NodeId）。
    ///
    /// 用于 inline-flex/inline-grid 等元素，其基线应从第一个子元素的布局位置
    /// 合成，而非使用简单的 height/2 回退。由 adjust_inline_block_positions
    /// 从 LayoutBox 子元素位置计算后传入。
    pub baseline_overrides: HashMap<NodeId, f32>,
    /// 内联元素的 (margin_left, margin_right) 覆盖（key = 元素自身的 NodeId）。
    ///
    /// paint IFC 传入空的 styles HashMap，无法获取 inline 元素的水平 margin，
    /// 导致所有 margin 回退为 0。此覆盖确保 paint IFC 使用正确的 margin 值。
    /// margin 不影响行断（仅影响水平偏移），因此传递此覆盖不会改变行断行为。
    pub margin_overrides: HashMap<NodeId, (f32, f32)>,
    /// R109 §9.2.1.1 匿名块盒的片段文本节点覆盖。
    ///
    /// 当此 IFC 为匿名块盒（inline 元素被 block 子元素拆分后的一个片段）服务时，
    /// 设置此字段使 `collect_inline_items` 只收集这些节点（该片段的 inline 内容），
    /// 而非遍历 container 的全部 DOM 子节点。`None` = 正常遍历 container 子节点。
    /// 为 tree.rs 匿名块生成接线奠基（当前无调用方设值，默认 None 零回归）。
    pub fragment_node_ids: Option<Vec<NodeId>>,
    /// Phase A font-metric 提供者（可选）。
    ///
    /// `None`（默认）= `apply_vertical_alignment` 回退 `0.8·fs` 启发式（当前行为，零回归）。
    /// `Some` = 注入 `FontLoader`-backed 真实度量，供 Phase A step-2 在 strut baseline /
    /// half-leading 计算中消费（替换 `0.8`）。step-1 仅持有该字段、不读取 → 行为不变。
    pub font_metric_provider: Option<FontMetricProviderHandle>,
    /// 字符 advance 宽度源（C3 advance plumbing，R2 dormant）。
    ///
    /// `None`（默认）= 4 个 in-IFC 度量点回退 `EstimateAdvance`（= `estimate_char_width`
    /// 启发式，字节等价，零回归）。`Some` = 注入 `FontLoader`-backed 真实 advance（R3），
    /// 度量点改读 hmtx 真实 advance，与 paint 同源（解 advance-wall / R1264 layout 残余）。
    pub advance_source: Option<AdvanceSourceHandle>,
    /// CSS font-family 到字体 ID 的独立解析表。
    ///
    /// 与 `font_metric_provider` 分离，使 shaped-text 可在不启用 per-font line-height
    /// 的情况下填充 `TextRun.font_id`。
    pub font_resolver: Option<Rc<HashMap<String, u32>>>,
    /// 逐文本节点的真实 ascent ratio 覆盖（key = 文本片段 NodeId）。
    ///
    /// **Phase A §12.6 step-2 bypass 基础设施（dormant，零回归）**：R890 实证
    /// `apply_vertical_alignment` 在 paint Path B 被空 styles 重跑，provider 无法
    /// 解析 family → 单点 wiring no-op。bypass = 在 layout IFC（有 styles + provider）
    /// 算出每文本节点真实 ascent ratio（ascent / font_size），经
    /// `store_font_sizes_from_ifc` 存入 LayoutBox，paint Path B 经此 map 读取，
    /// 绕过空 styles。R990 已落地 is_ahem-gated 常数（Ahem 0.8 / 非-Ahem 0.928），
    /// 本字段是 per-font 真实值的承载——空 map（默认）回退 R990 常数（零回归），
    /// 由 `ascent_ratio_for` 消费。
    pub ascent_ratio_overrides: HashMap<NodeId, f32>,
    /// 逐父元素的 text-transform 覆盖（key = 文本节点的父元素 NodeId）。
    ///
    /// **R1012 Phase A IFC 统一首切**：text-transform 须在行断前应用（layout 用
    /// 转换后文本宽度行断），但 paint Path B 重跑 IFC 时 styles 为空 →
    /// `collect_inline_items` 读不到父元素 text-transform。paint 从
    /// `LayoutBox.text_node_text_transform`（key = 文本节点）re-key 到父元素后
    /// 填充本 map，`collect_inline_items` 据此在空 styles 下应用 transform。
    /// 空 map（默认）= None = 原文，零回归。
    pub text_transform_overrides: HashMap<NodeId, TextTransformValue>,
    ///
    /// `None`（默认）= IFC 行盒不碎片化（当前行为，零回归）。
    /// `Some` = step-2 在 `break_items_into_lines` 后按本上下文把行盒分配到列
    /// （respected 列高 budget，整行不裁断）。step-1 仅持有字段、不读取 → 行为不变。
    pub column_fragmentation: Option<ColumnFragmentationContext>,
}

/// 默认 tab-size 值（8 个空格宽度，对应浏览器默认值）。
const DEFAULT_TAB_SIZE: f32 = 8.0;

impl InlineFormattingContext {
    /// 创建新的行内格式化上下文。
    pub fn new(container_width: f32) -> Self {
        Self {
            container_width,
            text_align: TextAlign::default(),
            text_align_last: None,
            break_word: false,
            no_wrap: false,
            preserve_whitespace: false,
            break_at_newline: false,
            word_break: WordBreakMode::default(),
            text_autospace: TextAutospaceValue::NoAutospace,
            text_indent: 0.0,
            tab_size: DEFAULT_TAB_SIZE,
            float_exclusions: Vec::new(),
            lines: Vec::new(),
            line_clamp: None,
            clamped: false,
            vertical: false,
            vertical_rtl: false,
            block_extent: container_width,
            inline_block_sizes: HashMap::new(),
            img_intrinsic_sizes: HashMap::new(),
            default_font_metrics: None,
            container_font_size: DEFAULT_FONT_SIZE,
            font_size_overrides: HashMap::new(),
            font_id_overrides: HashMap::new(),
            font_ids_overrides: HashMap::new(),
            font_size_adjust_overrides: HashMap::new(),
            is_ahem_overrides: HashMap::new(),
            letter_spacing_overrides: HashMap::new(),
            line_height_overrides: HashMap::new(),
            inline_element_metrics: HashMap::new(),
            baseline_overrides: HashMap::new(),
            margin_overrides: HashMap::new(),
            fragment_node_ids: None,
            font_metric_provider: None,
            advance_source: None,
            font_resolver: None,
            ascent_ratio_overrides: HashMap::new(),
            text_transform_overrides: HashMap::new(),
            column_fragmentation: None,
        }
    }

    /// 设置匿名块盒片段的文本节点覆盖（R109 §9.2.1.1）。
    ///
    /// 使本 IFC 只收集 `node_ids`（拆分后某片段的 inline 内容），而非遍历 container
    /// 的全部 DOM 子节点。供匿名块盒（inline 被 block 子元素拆分）的 IFC 使用。
    pub fn set_fragment_node_ids(&mut self, node_ids: Vec<NodeId>) {
        self.fragment_node_ids = Some(node_ids);
    }

    /// 设置文本对齐方式。
    pub fn with_text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    /// 设置垂直书写模式。
    ///
    /// 启用后，字符沿 y 轴向下推进，"行"变为垂直列。
    pub fn with_vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// 设置垂直模式下列排列方向（vertical-rl 时列从右到左）。
    ///
    /// 仅当 vertical=true 时有效。
    pub fn with_vertical_rtl(mut self, rtl: bool) -> Self {
        self.vertical_rtl = rtl;
        self
    }

    /// 设置垂直模式容器的 block 轴 extent（content_width）。仅 vertical 时消费
    ///（break_items_into_columns vrl 列 x）。见 `block_extent` 字段文档（R1122）。
    pub fn with_block_extent(mut self, extent: f32) -> Self {
        self.block_extent = extent;
        self
    }

    /// 设置 inline-block 元素的预计算尺寸（来自 LayoutBox / taffy 布局结果）。
    pub fn with_inline_block_sizes(mut self, sizes: HashMap<NodeId, (f32, f32)>) -> Self {
        self.inline_block_sizes = sizes;
        self
    }

    /// 设置 `<img>` 替换元素的解码固有尺寸（R1578，见 `img_intrinsic_sizes` 字段文档）。
    pub fn with_img_intrinsic_sizes(mut self, sizes: HashMap<NodeId, (f32, f32)>) -> Self {
        self.img_intrinsic_sizes = sizes;
        self
    }

    /// 设置默认字体度量（font_size, line_height）。
    ///
    /// 当 styles HashMap 中找不到元素样式时，使用此默认值替代
    /// 硬编码的 16px/19.2px。主要用于 paint 系统的 IFC。
    pub fn with_default_font_metrics(mut self, font_size: f32, line_height: f32) -> Self {
        self.default_font_metrics = Some((font_size, line_height));
        self
    }

    /// 注入 paint IFC 的逐元素字体 ID。
    pub fn with_font_id_overrides(mut self, overrides: HashMap<NodeId, u32>) -> Self {
        self.font_id_overrides = overrides;
        self
    }

    /// 注入逐元素有序 CSS face ID 列表。
    pub fn with_font_ids_overrides(mut self, overrides: HashMap<NodeId, Vec<u32>>) -> Self {
        self.font_ids_overrides = overrides;
        self
    }

    /// 注入逐 run 的 `font-size-adjust` 计算值。
    pub fn with_font_size_adjust_overrides(
        mut self,
        overrides: HashMap<NodeId, zero_style_system::FontSizeAdjustValue>,
    ) -> Self {
        self.font_size_adjust_overrides = overrides;
        self
    }

    /// 注入 Phase A font-metric 提供者（`FontLoader`-backed 真实行度量）。
    ///
    /// **Phase A §12.6 step-1（零回归）**：本方法仅设置字段，`apply_vertical_alignment`
    /// 尚未读取该字段（仍走 `0.8·fs` 启发式）。step-2（三方协调）才在此消费真实
    /// ascent/descent/line_gap。调用方（`zero-engine` 构造 IFC 时）传入
    /// `Rc::new(font_loader)` 共享同一 `FontLoader`。
    pub fn with_font_metric_provider(mut self, provider: Rc<dyn FontMetricProvider>) -> Self {
        self.font_metric_provider = Some(FontMetricProviderHandle(provider));
        self
    }

    /// 注入字符 advance 宽度源（`FontLoader`-backed 真实 hmtx advance）。
    ///
    /// **C3 advance plumbing（R2 dormant）**：本方法仅设置字段。默认（未调用）=
    /// `advance_source = None`，4 个 in-IFC 度量点（`inline/mod.rs` 空格/字符定位）
    /// 回退 `EstimateAdvance` = `estimate_char_width` 启发式，行为不变（零回归）。
    /// `zero-engine` 注入 `FontLoader`-backed 实现后，度量点经
    /// `AdvanceSourceHandle::measure` 读真实 advance。
    pub fn with_advance_source(mut self, source: Rc<dyn AdvanceSource>) -> Self {
        self.advance_source = Some(AdvanceSourceHandle(source));
        self
    }

    /// 注入 CSS font-family 到字体 ID 的解析表。
    pub fn with_font_resolver(mut self, resolver: Rc<HashMap<String, u32>>) -> Self {
        self.font_resolver = Some(resolver);
        self
    }

    /// 测量单字符 advance 宽度（C3 advance plumbing，R2 dormant）。
    ///
    /// 注入源时经 `AdvanceSourceHandle`（真实 hmtx），否则回退 `estimate_char_width`
    /// 启发式（字节等价，零回归）。in-IFC 度量点（`break_items_into_lines` /
    /// `break_items_into_columns` 的空格与字符定位）统一经本方法，使 advance 源
    /// 可在不改度量点的前提下切换。
    fn advance_of(&self, ch: char, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        match &self.advance_source {
            Some(src) => src.measure(ch, font_id, font_size, is_ahem),
            None => estimate_char_width(ch, font_size, is_ahem),
        }
    }

    /// C3 advance（R223 font_id gap）：解析 ComputedStyle 的 font-family → font_id。
    ///
    /// TextRun 构造处调用以 populate `run.font_id`（当前恒 `None`，致 `advance_of`
    /// 即使注入 `AdvanceSource` 也收到 `None` 回退 estimate）。**dormant**：IFC 的
    /// `font_metric_provider` 默认 `None` → 返回 `None` = 现行为零回归；provider
    /// 注入后（U1b-wiring 激活）经 `FontMetricProvider::font_id_of` 真实解析。
    fn font_id_for_style(&self, style: Option<&zero_style_system::ComputedStyle>) -> Option<u32> {
        let s = style?;
        s.font_family
            .iter()
            .find_map(|family| {
                self.font_resolver.as_ref().and_then(|resolver| {
                    resolver.get(family).copied().or_else(|| {
                        resolver
                            .iter()
                            .find(|(name, _)| name.eq_ignore_ascii_case(family))
                            .map(|(_, id)| *id)
                    })
                })
            })
            .or_else(|| {
                self.font_metric_provider
                    .as_ref()
                    .and_then(|provider| provider.font_id_of(&s.font_family))
            })
    }

    /// 返回允许受限 shaping 的 run 字体 ID；不满足边界时保留旧 estimate 路径。
    fn shaping_font_id_for_style(
        &self,
        override_node: Option<NodeId>,
        style: Option<&zero_style_system::ComputedStyle>,
        is_ahem: bool,
        letter_spacing: f32,
        word_spacing: f32,
        is_ruby: bool,
    ) -> Option<u32> {
        if is_ahem
            || letter_spacing != 0.0
            || word_spacing != 0.0
            || is_ruby
            || !style.map_or_else(
                || override_node.is_some_and(|node| self.font_id_overrides.contains_key(&node)),
                |style| matches!(style.direction, zero_style_system::DirectionValue::Ltr),
            )
        {
            return None;
        }
        self.font_id_for_style(style)
            .or_else(|| override_node.and_then(|node| self.font_id_overrides.get(&node).copied()))
    }

    /// 测量整段文本的 advance 宽度（C3 advance plumbing，R2 dormant）。
    ///
    /// 注入源时逐字符经 `advance_of`（真实 hmtx）；否则直接委托 `estimate_string_width`
    /// （字节等价，零回归，且保持该函数生产活跃）。换行决策的 word_width
    /// （`break_items_into_lines`）/ word_height（列模式）经本方法，使 advance 源真正
    /// 驱动换行点（advance-wall / R1264 layout 残余的 root）。
    fn advance_string_width(&self, text: &str, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        match &self.advance_source {
            Some(src) => src.measure_text(text, font_id, font_size, is_ahem),
            None => estimate_string_width(text, font_size, is_ahem),
        }
    }

    fn advance_run_width(&self, text: &str, run: &TextRun) -> f32 {
        let font_ids = self
            .font_ids_overrides
            .get(&run.node_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        match &self.advance_source {
            Some(source) if !font_ids.is_empty() && font_ids.first().copied() == run.font_id => {
                let size_adjust = self
                    .font_size_adjust_overrides
                    .get(&run.node_id)
                    .unwrap_or(&zero_style_system::FontSizeAdjustValue::None);
                let width =
                    source.measure_text_with_font_context(text, font_ids, run.font_size, run.is_ahem_font, size_adjust);
                if std::env::var("ZW_SHAPED_ADVANCE_TRACE").as_deref() == Ok("1")
                    && !matches!(size_adjust, zero_style_system::FontSizeAdjustValue::None)
                {
                    tracing::info!(
                        target: "zero_layout_engine::shaped_advance",
                        node_id = ?run.node_id,
                        font_id = ?run.font_id,
                        ?font_ids,
                        ?size_adjust,
                        width,
                        text,
                        "ZW_LAYOUT_SHAPED_ADVANCE_TRACE"
                    );
                }
                width
            }
            _ => self.advance_string_width(text, run.font_id, run.font_size, run.is_ahem_font),
        }
    }

    /// 注入逐文本节点真实 ascent ratio 覆盖（Phase A §12.6 step-2 bypass，dormant）。
    ///
    /// 空调用方（默认）= R990 is_ahem-gated 常数行为（零回归）。当 paint Path B
    /// 从 LayoutBox.text_node_ascent_ratios 填充本 map 后，`apply_vertical_alignment`
    /// 按 `ascent_ratio_for` 优先取本 map 真实值，实现 per-font ascent 而**不**经
    /// provider family 解析（绕过 R890 空 styles 墙）。
    pub fn with_ascent_ratio_overrides(mut self, overrides: HashMap<NodeId, f32>) -> Self {
        self.ascent_ratio_overrides = overrides;
        self
    }

    /// 注入逐父元素 text-transform 覆盖（R1012 Phase A IFC 统一首切）。
    ///
    /// 空 map（默认）= None = 原文，零回归。paint Path B 从
    /// `LayoutBox.text_node_text_transform` re-key 到父元素后填充本 map，
    /// `collect_inline_items` 据此在空 styles 下应用 transform，使行断用
    /// 转换后文本宽度（与 layout IFC 一致）。
    pub fn with_text_transform_overrides(mut self, overrides: HashMap<NodeId, TextTransformValue>) -> Self {
        self.text_transform_overrides = overrides;
        self
    }

    /// 查文本片段的真实 ascent ratio（Phase A §12.6 step-2 bypass 消费点）。
    ///
    /// 优先取 `ascent_ratio_overrides[node_id]`（>0 有效，由 layout IFC 经 provider
    /// 算出并存入）；否则回退 R990 is_ahem-gated 常数（Ahem 0.8 / 非-Ahem 0.928）。
    /// 空 map（默认）= 全回退 = R990 行为（零回归）。
    pub fn ascent_ratio_for(&self, node_id: NodeId, is_ahem: bool) -> f32 {
        ascent_ratio_lookup(&self.ascent_ratio_overrides, node_id, is_ahem)
    }

    /// 注入 Phase 2a multicol 列碎片化上下文（dormant）。
    ///
    /// step-1 仅持有字段、不读取 → 行为不变（零回归）。step-2 在产宽度换行行盒后
    /// 按本上下文把行盒分配到列。调用方（layout 侧，step-2 接线）对目标结构
    /// （单层 multicol + `column-fill:auto` + 明确高度 + 单一 block 子元素）构造。
    pub fn with_column_fragmentation(mut self, ctx: ColumnFragmentationContext) -> Self {
        self.column_fragmentation = Some(ctx);
        self
    }

    /// 设置逐文本节点的字体大小覆盖。
    ///
    /// key 为文本节点的父元素 NodeId，value 为 layout IFC 计算的 font_size。
    /// 当 styles HashMap 中找不到父元素样式时，使用此映射中的 font_size
    /// 替代 16px 默认值，使字符宽度计算更准确。
    pub fn with_font_size_overrides(mut self, overrides: HashMap<NodeId, f32>) -> Self {
        self.font_size_overrides = overrides;
        self
    }

    /// 设置 Ahem 字体标志覆盖（paint IFC 使用）。
    pub fn with_is_ahem_overrides(mut self, overrides: HashMap<NodeId, bool>) -> Self {
        self.is_ahem_overrides = overrides;
        self
    }

    /// 设置 letter-spacing 覆盖（paint IFC 使用）。
    pub fn with_letter_spacing_overrides(mut self, overrides: HashMap<NodeId, f32>) -> Self {
        self.letter_spacing_overrides = overrides;
        self
    }

    /// 设置逐文本节点的 line-height 覆盖（paint IFC 使用）。
    ///
    /// key 为文本节点的父元素 NodeId，value 为 layout IFC 计算的 line-height。
    /// line-height 仅影响行盒高度（垂直定位），不影响行断（水平宽度），
    /// 因此传递此覆盖是安全的。
    pub fn with_line_height_overrides(mut self, overrides: HashMap<NodeId, f32>) -> Self {
        self.line_height_overrides = overrides;
        self
    }

    /// 设置内联元素的 (font_size, line_height) 覆盖（paint IFC 使用）。
    ///
    /// key 为内联元素自身的 NodeId，value 为 (font_size, line_height)。
    /// 这些属性仅影响垂直定位（行盒高度），不影响行断。
    pub fn with_inline_element_metrics(mut self, metrics: HashMap<NodeId, (f32, f32)>) -> Self {
        self.inline_element_metrics = metrics;
        self
    }
    /// 设置内联元素的 (margin_left, margin_right) 覆盖（paint IFC 使用）。
    ///
    /// key 为内联元素自身的 NodeId，value 为 (margin_left, margin_right)。
    /// margin 不影响行断（仅影响水平偏移），因此传递此覆盖是安全的。
    pub fn with_margin_overrides(mut self, margins: HashMap<NodeId, (f32, f32)>) -> Self {
        self.margin_overrides = margins;
        self
    }
    /// 设置末行对齐方式（CSS text-align-last）。
    ///
    /// None 表示末行跟随 text_align 设置（默认行为）。
    pub fn with_text_align_last(mut self, align: Option<TextAlign>) -> Self {
        self.text_align_last = align;
        self
    }

    /// 设置首行文本缩进（CSS text-indent）。
    pub fn with_text_indent(mut self, indent: f32) -> Self {
        self.text_indent = indent;
        self
    }

    /// 设置是否允许单词内断行（overflow-wrap: break-word / anywhere）。
    pub fn with_break_word(mut self, break_word: bool) -> Self {
        self.break_word = break_word;
        self
    }

    /// 设置是否禁止换行（white-space: nowrap / pre）。
    pub fn with_no_wrap(mut self, no_wrap: bool) -> Self {
        self.no_wrap = no_wrap;
        self
    }

    /// 设置是否保留空白字符（white-space: pre / pre-wrap）。
    pub fn with_preserve_whitespace(mut self, preserve: bool) -> Self {
        self.preserve_whitespace = preserve;
        self
    }

    /// 设置是否在换行符 `\n` 处强制断行（white-space: pre-line）。
    pub fn with_break_at_newline(mut self, break_at_newline: bool) -> Self {
        self.break_at_newline = break_at_newline;
        self
    }

    /// 设置 word-break 行为。
    pub fn with_word_break(mut self, mode: WordBreakMode) -> Self {
        self.word_break = mode;
        self
    }

    /// 设置 text-autospace 行为（CSS Text 4 §8）。
    pub fn with_text_autospace(mut self, value: TextAutospaceValue) -> Self {
        self.text_autospace = value;
        self
    }

    /// 设置浮动排除区域 — 浮动元素占据的空间。
    ///
    /// 文本在排列时会自动避开这些区域，实现文本环绕浮动元素的效果。
    pub fn with_float_exclusions(mut self, exclusions: Vec<FloatExclusion>) -> Self {
        self.float_exclusions = exclusions;
        self
    }

    /// 设置 CSS tab-size（制表符展开宽度，px）。
    ///
    /// 制表符 `\t` 在 pre/pre-wrap 模式下会展开为此宽度的空格。
    pub fn with_tab_size(mut self, tab_size: f32) -> Self {
        self.tab_size = tab_size;
        self
    }

    /// 设置行内级盒的基线覆盖。
    ///
    /// 用于 inline-flex/inline-grid 等元素，其基线从第一个子元素的布局位置合成，
    /// 而非使用简单的 height/2 回退。
    pub fn with_baseline_overrides(mut self, overrides: HashMap<NodeId, f32>) -> Self {
        self.baseline_overrides = overrides;
        self
    }

    /// 计算指定 y 范围内的有效内容区域。
    ///
    /// 返回 `(left_offset, available_width)`：
    /// - `left_offset` — 左侧浮动占据的宽度（文本起始 x 坐标）
    /// - `available_width` — 扣除左右浮动后的剩余可用宽度
    fn effective_content_area(&self, line_y: f32, line_height: f32) -> (f32, f32) {
        let mut left_offset = 0.0_f32;
        let mut right_reduction = 0.0_f32;

        for excl in &self.float_exclusions {
            // 检查排除区域是否与当前行的 y 范围重叠
            let excl_bottom = excl.y + excl.height;
            let line_bottom = line_y + line_height;
            if excl.y < line_bottom && excl_bottom > line_y {
                if excl.is_left {
                    // 左浮动：累加宽度（多个左浮动堆叠）
                    left_offset += excl.width;
                } else {
                    // 右浮动：累加缩减
                    right_reduction += excl.width;
                }
            }
        }

        let available = (self.container_width - left_offset - right_reduction).max(0.0);
        (left_offset, available)
    }

    /// 从 ComputedStyle 提取 inline 元素的垂直 padding 和 border。
    ///
    /// 返回 (padding_top, padding_bottom, border_top, border_bottom)。
    fn extract_inline_box_metrics(style: Option<&zero_style_system::ComputedStyle>) -> (f32, f32, f32, f32) {
        use zero_css_parser::values::LengthValue;
        let extract = |val: &LengthValue| -> f32 {
            match val {
                LengthValue::Px(v) => *v as f32,
                _ => 0.0,
            }
        };
        match style {
            Some(s) => (
                extract(&s.padding_top),
                extract(&s.padding_bottom),
                extract(&s.border_top_width),
                extract(&s.border_bottom_width),
            ),
            None => (0.0, 0.0, 0.0, 0.0),
        }
    }

    /// 对文档中指定节点的行内子内容执行布局。
    ///
    /// 收集文本节点和 inline 元素，从 ComputedStyle 读取 font-size 和 line-height，
    /// 将它们排列成行盒。
    ///
    /// # 参数
    ///
    /// - `doc` — DOM 文档
    /// - `container` — 行内格式化上下文的容器节点
    /// - `styles` — 元素 NodeId → ComputedStyle 映射
    pub fn layout(&mut self, doc: &Document, container: NodeId, styles: &HashMap<NodeId, ComputedStyle>) {
        // 从容器自身样式读取 font-size，供 apply_vertical_alignment 计算 strut ascent。
        // CSS 2.1 §10.8.1：strut 由块容器自身的字体度量决定。paint IFC 传入空 styles
        // 时保持默认 16（仅影响行内文本片段的垂直定位，文本 font_size 通常主导 ascent）。
        if let Some(style) = styles.get(&container) {
            self.container_font_size = match &style.font_size {
                LengthValue::Px(px) => *px as f32,
                LengthValue::Em(em) => *em as f32 * 16.0,
                LengthValue::Rem(rem) => *rem as f32 * 16.0,
                LengthValue::Percentage(p) => *p as f32 * 16.0 / 100.0,
                _ => DEFAULT_FONT_SIZE,
            };
        }
        let items = self.collect_inline_items(doc, container, styles);
        self.break_items_into_lines(items);
        // R2431 line-clamp（CSS Overflow 4）：容器声明 `line-clamp: Count(n)` 时，把行夹到 N。
        if !self.vertical
            && std::env::var("ZW_LINE_CLAMP").as_deref() != Ok("0")
            && let Some(style) = styles.get(&container)
            && let Some(n) = match &style.line_clamp {
                zero_style_system::property::types::LineClampComputedValue::Count(n) => Some(*n as usize),
                _ => None,
            }
        {
            self.apply_line_clamp_cap(n);
        }
    }

    /// R2431 line-clamp：把 `self.lines` 夹到 `n` 行并置 `clamped`（n>0 且行数>n 时截断）。
    /// 从 `layout()` 抽出供单测直接驱动（break_into_lines + apply_line_clamp_cap）。
    /// 截断保留前 N 行完整几何（float exclusion / inline-box 对齐不变）；下游 height 从 lines 推。
    pub(crate) fn apply_line_clamp_cap(&mut self, n: usize) {
        if n > 0 && self.lines.len() > n {
            self.lines.truncate(n);
            self.clamped = true;
        }
    }

    /// 递归收集 `id` 子树的所有文本，跳过 `local_name` 在 `exclude` 中的元素子树。
    ///
    /// R1022：用于 `<ruby>` —— 收集 rb 文本作 inline 文本，排除 `<rt>`/`<rp>`
    /// （rt 文本在 paint 期作 zero-width annotation 上移到 rb 之上，不参与 inline 流）。
    fn collect_text_excluding(doc: &Document, id: NodeId, exclude: &[&str]) -> String {
        let mut out = String::new();
        Self::collect_text_excluding_inner(doc, id, exclude, &mut out);
        out
    }

    fn collect_text_excluding_inner(doc: &Document, id: NodeId, exclude: &[&str], out: &mut String) {
        for child_id in doc.child_nodes(id) {
            if let Some(node) = doc.get(child_id) {
                match &node.kind {
                    NodeKind::Text(data) => out.push_str(&data.content),
                    NodeKind::Element(elem) => {
                        if exclude.iter().any(|e| elem.local_name().eq_ignore_ascii_case(e)) {
                            continue;
                        }
                        Self::collect_text_excluding_inner(doc, child_id, exclude, out);
                    }
                    _ => {}
                }
            }
        }
    }

    /// R1576 inline-box-model 递归开关：inline 元素含嵌套 inline-block 后代时递归收集。
    /// default-on（`ZW_INLINE_BOX_RECURSE=0` 关闭）。
    fn inline_box_model_recurse() -> bool {
        !matches!(std::env::var("ZW_INLINE_BOX_RECURSE").as_deref(), Ok("0"))
    }

    /// R1578 img 固有宽高比推导开关：img 分支在恰一侧维度已知时按固有比推导缺失侧。
    /// default-on（`ZW_IFC_IMG_INTRINSIC=0` 关闭）——slice 2 全量 13 dir A/B 守 net +3
    /// 零真回归（normal-flow +2 / CSS2 +2 / writing-modes −1 经证为 logical-props-004
    /// 跨运行噪声 0.96↔1.33）+ 解 wintertc footer 塌缩产品 bug。kill-switch 保留可回退。
    fn ifc_img_intrinsic_enabled() -> bool {
        !matches!(std::env::var("ZW_IFC_IMG_INTRINSIC").as_deref(), Ok("0"))
    }

    /// R2156 Phase A：检测 inline 元素的**整个子树**是否含 out-of-flow 后代
    /// （position:absolute/fixed）。tree.rs 的 `ZW_INLINE_BOX_MODEL_COHERENCE` 切片在跳过 inline
    /// 的 taffy 节点前调此函数——若子树含 ooflow 后代，必须保留 taffy 子树，否则 ooflow 后代
    /// 失去 containing block（nested-inline-abspos-child 簇：`<span>...<span class=parent>
    /// <div class=inline-content>` 中 inline-content 同时是 inline-block + absolute，跳过外层
    /// `<span>` 会把整棵子树丢出 taffy 树 → abspos 丢 CB）。与 R1492 同源约束。
    pub(crate) fn inline_subtree_has_ooflow_descendant(
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        inline_id: NodeId,
    ) -> bool {
        let mut stack = doc.child_nodes(inline_id);
        while let Some(cid) = stack.pop() {
            let Some(node) = doc.get(cid) else {
                continue;
            };
            if let NodeKind::Element(_) = &node.kind
                && let Some(s) = styles.get(&cid)
                && matches!(
                    s.position,
                    zero_style_system::property::types::PositionValue::Absolute
                        | zero_style_system::property::types::PositionValue::Fixed
                )
            {
                return true;
            }
            // 继续下探全部后代（不限 inline-level——abspos 可嵌在 block-in-inline 子树里）。
            stack.extend(doc.child_nodes(cid));
        }
        false
    }

    /// R1576：检测 inline 元素是否含**嵌套 atomic inline 后代**（任意深度，限 inline 路径）。
    /// atomic = inline-block/inline-flex/inline-grid/inline-table **或 `<img>`**（IFC 把 img 作
    /// 原子行内级盒，local_name=="img" 分支）。仅沿 inline-level 后代下探（block-level 停止 =
    /// R109 block-in-inline 另路径）。驱动 `collect_inline_items` 递归收集，修复
    /// `<p><a><img></a></p>` / `<p><a><span class=inline-block></span></a></p>` 容器塌缩。
    //
    // R2156 Phase A：暴露为 pub(crate) 供 tree.rs 子循环复用同一判定（避免逻辑漂移）。
    pub(crate) fn inline_elem_has_nested_inline_block(
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        inline_id: NodeId,
    ) -> bool {
        use zero_style_system::property::types::DisplayValue;
        let is_atomic_inline = |d: &DisplayValue| {
            matches!(
                d,
                DisplayValue::InlineBlock
                    | DisplayValue::InlineFlex
                    | DisplayValue::InlineGrid
                    | DisplayValue::InlineTable
            )
        };
        let is_inline_level = |d: &DisplayValue| {
            matches!(
                d,
                DisplayValue::Inline
                    | DisplayValue::InlineBlock
                    | DisplayValue::InlineFlex
                    | DisplayValue::InlineGrid
                    | DisplayValue::InlineTable
            )
        };
        // BFS 沿 inline-level 后代下探，找 atomic inline 盒（inline-block 或 img）。
        let mut stack = doc.child_nodes(inline_id);
        while let Some(cid) = stack.pop() {
            let Some(node) = doc.get(cid) else {
                continue;
            };
            let Some(elem_data) = (match &node.kind {
                NodeKind::Element(e) => Some(e),
                _ => None,
            }) else {
                continue;
            };
            let Some(s) = styles.get(&cid) else {
                continue;
            };
            // img 是 atomic inline（IFC img 分支）**仅当 display 为 inline-level**（默认 inline）。
            // R2236：旧实现 `local_name()=="img"` 无视 display，把 `display:block` 的 img（如
            // `<span><img class="block">` block-in-inline）误判为原子行内后代 → 触发
            // collect_inline_items 无限递归（栈溢出 DoS）。block-level img 由 R109 split 处理，
            // 非 IFC 原子行内。driving: WPT css-overflow/line-clamp/line-clamp-033。
            let img_inline_atomic = elem_data.local_name() == "img" && is_inline_level(&s.display);
            if img_inline_atomic || is_atomic_inline(&s.display) {
                return true;
            }
            if is_inline_level(&s.display) {
                // 继续沿 inline 后代下探（block-level 后代停止 = R109 另路径）
                stack.extend(doc.child_nodes(cid));
            }
        }
        false
    }
}

// 行内条目收集方法（collect_inline_items）— 拆分到独立文件控制 mod.rs 体积
// （include! 模式 ≡ apps/browser/src/app.rs；零行为/可见性变更，同模块访问私有字段）
include!("collect_items.rs");

// 行盒分割方法（break_into_lines / break_items_into_lines）— 拆分到独立文件控制 mod.rs 体积
include!("break_lines.rs");

impl InlineFormattingContext {
    /// 计算行尾「悬挂」空白的总宽度（CSS Text §3.1.4 phase II）。
    ///
    /// pre-wrap 下，行尾的保留空格应 *hang*——它仍被绘制，但在计算 text-align
    /// 对齐偏移时不计入行宽（这样 Right/Center/Justify 按可见内容对齐，空白
    /// 自然挂到行内容区右缘之外）。从末片段向前累加：
    /// - 纯空格片段（split_into_words preserve 模式下的独立 " "）→ 贡献整宽；
    /// - 混合片段以空格结尾 → 贡献尾部连续空格的 advance（近似），停止；
    /// - 片段不以空格结尾 → 停止。
    fn trailing_hang_width(&self, runs: &[TextFragment]) -> f32 {
        let mut total = 0.0;
        for frag in runs.iter().rev() {
            let chars: Vec<char> = frag.text.chars().collect();
            if chars.is_empty() {
                continue;
            }
            let trailing = chars.iter().rev().take_while(|&&c| c == ' ').count();
            if trailing == chars.len() {
                total += frag.width;
            } else if trailing > 0 {
                total += trailing as f32 * self.advance_of(' ', None, frag.font_size, frag.is_ahem);
                break;
            } else {
                break;
            }
        }
        total
    }

    /// 根据当前 text_align 设置，调整每行中片段的 x 坐标。
    ///
    /// - Left: 不做调整（默认行为）。
    /// - Center: 整行居中于 container_width。
    /// - Right: 整行右对齐。
    /// - Justify: 非最后一行在单词间均匀分配剩余空间。
    fn apply_text_alignment(&mut self) {
        if (self.text_align == TextAlign::Left && self.text_align_last.is_none()) || self.lines.is_empty() {
            return;
        }

        // R1338：pre-wrap 行尾保留空格 hang（default-on，kill-switch ZW_PREWRAP_HANG=0 关闭）。
        let hang_prewrap = self.preserve_whitespace && std::env::var("ZW_PREWRAP_HANG").as_deref() != Ok("0");

        // 预计算每行的有效内容区域（避免在 iter_mut 中借用 self）
        let line_areas: Vec<(f32, f32)> = self
            .lines
            .iter()
            .map(|line| self.effective_content_area(line.y, line.height))
            .collect();

        // R1338：预计算每行行尾悬挂空白宽度（pre-wrap），避免在 iter_mut 中借用 self。
        let hang_widths: Vec<f32> = if hang_prewrap {
            self.lines
                .iter()
                .map(|line| self.trailing_hang_width(&line.runs))
                .collect()
        } else {
            Vec::new()
        };

        let last_idx = self.lines.len() - 1;
        for (i, line) in self.lines.iter_mut().enumerate() {
            if line.runs.is_empty() {
                continue;
            }

            // 计算行内内容的总宽度（最后一个片段的右边界）
            // R1338：pre-wrap 扣除行尾悬挂空白，使对齐按可见内容计算。
            let raw_right = line.runs.last().map(|r| r.x + r.width).unwrap_or(0.0);
            let hang = if hang_prewrap { hang_widths[i] } else { 0.0 };
            let content_width = (raw_right - hang).max(0.0);

            // 使用预计算的有效可用宽度
            let (left_offset, avail_width) = line_areas[i];
            let line_limit = left_offset + avail_width;
            let remaining = line_limit - content_width;

            // 确定本行使用的对齐方式
            // 最后一行：使用 text_align_last（如果设置了），否则 text-align: justify 回退到 Left
            let align = if i == last_idx {
                if let Some(tal) = self.text_align_last {
                    tal
                } else if self.text_align == TextAlign::Justify {
                    // justify 的最后一行默认回退到左对齐（标准行为）
                    TextAlign::Left
                } else {
                    self.text_align
                }
            } else {
                self.text_align
            };

            match align {
                TextAlign::Left => { /* 默认，无需调整 */ }
                TextAlign::Center => {
                    let offset = remaining / 2.0;
                    for run in &mut line.runs {
                        run.x += offset;
                    }
                }
                TextAlign::Right => {
                    let offset = remaining;
                    for run in &mut line.runs {
                        run.x += offset;
                    }
                }
                TextAlign::Justify => {
                    // 只在有 2 个及以上片段时才能分配空间
                    if line.runs.len() < 2 {
                        continue;
                    }
                    // 在片段之间均匀分配剩余空间
                    let gap_count = line.runs.len() - 1;
                    let extra_per_gap = remaining / gap_count as f32;
                    let mut accumulated = 0.0;
                    for j in 0..line.runs.len() {
                        line.runs[j].x += accumulated;
                        if j < gap_count {
                            accumulated += extra_per_gap;
                        }
                    }
                }
            }
        }
    }

    /// 垂直书写模式的行内布局 — 字符沿 y 轴向下推进，"列"沿 x 轴排列。
    ///
    /// 与水平模式的对应关系：
    /// - 水平模式的 `x` 推进 → 垂直模式的 `y` 推进（字符向下排列）
    /// - 水平模式的换行增加 `y` → 垂直模式的换列增加 `x`（新列向右）
    /// - 水平模式的 `container_width` 限制行宽 → 垂直模式的 `container_width` 限制列高
    /// - 片段的 `width`（水平跨度）→ 片段的 `height`（垂直跨度）
    /// - 片段的 `height`（line-height，行高）→ 片段的 `width`（列宽）
    fn break_items_into_columns(&mut self, items: Vec<InlineItem>) {
        // 垂直模式下 container_width 表示内容可向下推进的最大高度
        let max_depth = self.container_width;
        let _default_line_height = 20.0_f32;

        // 当前列的状态
        let mut current_column = LineBox {
            y: 0.0,
            height: 0.0,
            runs: Vec::new(),
            baseline_y: 0.0,
            ascent: 0.0,
            descent: 0.0,
        };
        // 当前深度（字符沿 y 向下推进的位置）
        let mut current_depth = self.text_indent;

        for item in items {
            match item {
                InlineItem::Text(run) => {
                    let mut source_cursor = BidiFragmentCursor::new(&run.text);
                    let words = self.split_into_words(source_cursor.visual_text(), run.is_ahem_font);

                    // 空 inline 元素
                    if words.is_empty() && run.text.is_empty() {
                        let col_width = run.line_height;
                        if col_width > current_column.height {
                            current_column.height = col_width;
                        }
                        if run.margin_left > 0.0 {
                            current_depth += run.margin_left;
                        }
                        continue;
                    }

                    if run.margin_left > 0.0 {
                        current_depth += run.margin_left;
                    }

                    for (word_idx, word) in words.iter().enumerate() {
                        let char_count = word.chars().count();
                        // 垂直模式下，单词的"高度" = 水平模式的宽度
                        let mut word_height =
                            self.advance_run_width(word, &run) + run.letter_spacing * char_count as f32;
                        if word_idx > 0 {
                            word_height += run.word_spacing;
                        }

                        // 检查当前列是否放得下（深度方向）
                        if !self.no_wrap && current_depth + word_height > max_depth && !current_column.runs.is_empty() {
                            self.lines.push(current_column);
                            current_column = LineBox {
                                y: 0.0,
                                height: 0.0,
                                runs: Vec::new(),
                                baseline_y: 0.0,
                                ascent: 0.0,
                                descent: 0.0,
                            };
                            current_depth = 0.0;
                        }

                        // overflow-wrap / word-break: break-all
                        let need_char_break = !self.no_wrap
                            && (self.break_word || self.word_break == WordBreakMode::BreakAll)
                            && current_depth + word_height > max_depth
                            && !word.is_empty();

                        if need_char_break {
                            let char_col_width = run.line_height;
                            let chars: Vec<char> = word.chars().collect();
                            let mut partial_depth = current_depth;

                            for (ci, ch) in chars.iter().enumerate() {
                                let ch_height = self.advance_of(*ch, run.font_id, run.font_size, run.is_ahem_font)
                                    + run.letter_spacing;
                                let text = ch.to_string();
                                let source = source_cursor.take_source(&text);

                                if partial_depth + ch_height > max_depth && ci > 0 {
                                    self.lines.push(current_column);
                                    current_column = LineBox {
                                        y: 0.0,
                                        height: 0.0,
                                        runs: Vec::new(),
                                        baseline_y: 0.0,
                                        ascent: 0.0,
                                        descent: 0.0,
                                    };
                                    partial_depth = 0.0;
                                }

                                current_column.runs.push(TextFragment {
                                    x: 0.0,
                                    y: partial_depth,
                                    width: char_col_width,
                                    height: ch_height,
                                    text,
                                    source,
                                    node_id: run.node_id,
                                    font_size: run.font_size,
                                    vertical_align: run.vertical_align.clone(),
                                    is_ahem: run.is_ahem_font,
                                    letter_spacing: run.letter_spacing,
                                    margin_left: run.margin_left,
                                    margin_right: run.margin_right,
                                    margin_top: 0.0,
                                    baseline: run.font_size,
                                });

                                partial_depth += ch_height;
                                current_column.height = current_column.height.max(char_col_width);
                            }
                            current_depth = partial_depth;
                        } else {
                            let col_width = run.line_height;
                            current_column.runs.push(TextFragment {
                                x: 0.0,
                                y: current_depth,
                                width: col_width,
                                height: word_height,
                                text: word.clone(),
                                source: source_cursor.take_source(word),
                                node_id: run.node_id,
                                font_size: run.font_size,
                                vertical_align: run.vertical_align.clone(),
                                is_ahem: run.is_ahem_font,
                                letter_spacing: run.letter_spacing,
                                margin_left: run.margin_left,
                                margin_right: run.margin_right,
                                margin_top: 0.0,
                                baseline: run.font_size,
                            });

                            current_depth += word_height;
                            current_column.height = current_column.height.max(col_width);
                        }
                    }

                    if run.margin_right > 0.0 {
                        current_depth += run.margin_right;
                    }
                }
                InlineItem::InlineBlock(box_info) => {
                    // 垂直模式下 inline-block 的 height 变为向下推进量，width 变为列宽
                    let box_depth = box_info.height;
                    let box_col_width = box_info.width;

                    if !self.no_wrap && current_depth + box_depth > max_depth && !current_column.runs.is_empty() {
                        self.lines.push(current_column);
                        current_column = LineBox {
                            y: 0.0,
                            height: 0.0,
                            runs: Vec::new(),
                            baseline_y: 0.0,
                            ascent: 0.0,
                            descent: 0.0,
                        };
                        current_depth = 0.0;
                    }

                    current_column.runs.push(TextFragment {
                        x: 0.0,
                        y: current_depth,
                        width: box_col_width,
                        height: box_depth,
                        text: String::new(),
                        source: None,
                        node_id: box_info.node_id,
                        font_size: 0.0,
                        vertical_align: box_info.vertical_align.clone(),
                        is_ahem: false,
                        letter_spacing: 0.0,
                        margin_left: 0.0,
                        margin_right: 0.0,
                        margin_top: 0.0,
                        baseline: box_info.baseline,
                    });

                    // 垂直模式：margin_top/bottom 沿 inline（depth）方向推进，
                    // margin_left/right 沿 block（列宽）方向计入列宽。
                    current_depth += box_depth + box_info.margin_top + box_info.margin_bottom;
                    current_column.height = current_column
                        .height
                        .max(box_col_width + box_info.margin_left + box_info.margin_right);
                }
                InlineItem::Br => {
                    self.lines.push(current_column);
                    current_column = LineBox {
                        y: 0.0,
                        height: 0.0,
                        runs: Vec::new(),
                        baseline_y: 0.0,
                        ascent: 0.0,
                        descent: 0.0,
                    };
                    current_depth = 0.0;
                }
            }
        }

        // 添加最后一列（非空时）
        if !current_column.runs.is_empty() {
            self.lines.push(current_column);
        }

        // 计算每列的 x 坐标（沿 x 轴排列）
        // 垂直模式中 LineBox.y 表示 x 坐标，LineBox.height 表示列宽
        if self.vertical_rtl {
            // vertical-rl：第一列在右侧，后续列向左排列。
            // ★ R1122：从 block 轴 extent（content_width）开始递减，**非 container_width**。
            // container_width 在 vertical = content_height（inline 深度/y 轴），用它当 block 右端
            // 致单列 col 落到 x=container_width-col_w（caption 784-50=734），paint off-screen，
            // caption-side-vrl 文本完全不绘制（R1120）。block_extent 是真实 block 轴（x）跨度，
            // caption content_width=50 → col 0 → paint content_x+0 正确。
            let mut x = self.block_extent; // 从 block 轴右端开始
            for col in &mut self.lines {
                x -= col.height; // col.height 在垂直模式表示列宽
                col.y = x;

                // 修正每个片段的 x 为列起始位置
                for run in &mut col.runs {
                    run.x = col.y;
                }
            }
        } else {
            // vertical-lr 或默认：列从左到右排列
            let mut x = 0.0;
            for col in &mut self.lines {
                col.y = x;
                x += col.height; // col.height 在垂直模式表示列宽

                // 修正每个片段的 x 为列起始位置
                for run in &mut col.runs {
                    run.x = col.y;
                }
            }
        }

        // 垂直模式下不应用水平文本对齐和 vertical-align
    }

    /// 根据每个片段的 vertical-align 值，计算其在行盒内的 y 偏移量。
    ///
    /// 对齐规则（基于行盒高度 line_height 和片段高度 fragment_height）：
    ///
    /// - **baseline** — 片段底部对齐行盒基线。基线位置 = line_height × 0.8（近似）。
    ///   y = baseline_y - fragment_height
    /// - **top** — 片段顶部紧贴行盒顶部。y = 0.0
    /// - **middle** — 片段垂直居中于行盒。y = (line_height - fragment_height) / 2
    /// - **bottom** — 片段底部紧贴行盒底部。y = line_height - fragment_height
    /// - **text-top** — 与 top 行为一致（简化：按字体度量等同于 top）。
    /// - **text-bottom** — 与 bottom 行为一致（简化）。
    /// - **sub** — 基线向下偏移 font_size × 0.3。
    /// - **super** — 基线向上偏移 font_size × 0.3。
    fn apply_vertical_alignment(&mut self) {
        for line in &mut self.lines {
            let line_height = line.height;

            // CSS 2.1 §10.8.1: 行盒基线由所有 baseline 对齐的 inline 级盒的 ascent 最大值决定。
            // - 文本运行：ascent ≈ font_size（字体 ascent 的近似）
            // - inline-block：baseline 在底部边缘，ascent = height
            // - strut 由块容器自身的 font-size 决定（而非行盒实测高度）
            //
            // strut ascent 的来源按行内是否有文本区分：
            // - 文本行：沿用 line_height*0.8（line_height 来自文本 line-height，含 leading，
            //   近似 leading 对半分布的基线位置）。
            // - 仅原子行内盒的行（如 inline-flex 容器被块级化后独占一行）：strut 基于容器
            //   font-size。否则 line_height 被高大的原子盒撑高，strut ascent 被错误放大，
            //   把合成 baseline 偏低的原子盒压到行盒下方，与同容器其它盒错位。
            let strut_ascent = if line.runs.iter().any(|r| r.font_size > 0.0) {
                // R800/R990：strut baseline = half-leading + ascent（CSS §10.8.1）。原 line_height*0.8
                // 随 line-height 线性增长过快（line-height 1.5→1.2em baseline，chromium ~1.05em），
                // 致文本基线偏低累积（welcome ~17% 主因之一）。正确：half-leading=(line_height-em)/2，
                // baseline = half-leading + ascent（em≈font_size）。
                //
                // R990：ascent ratio 按 is_ahem 区分——Ahem=0.8（精确，upem 1000/ascent 800）；
                // 非-Ahem 真实字体（system-ui/DejaVuSans）ascent≈0.928（R885 FontMetricProvider
                // 实测）。旧 0.8 对非-Ahem 偏低致行盒偏矮、基线偏低。is_ahem_font 在 layout
                // 由 style.font_family 定、在 paint 由 is_ahem_overrides 定，两侧一致（不受
                // paint Path B 空 styles 影响，区别于 R889/R890 provider 单点 no-op）。
                let (dominant_fs, dominant_is_ahem, dominant_node) = line
                    .runs
                    .iter()
                    .filter(|r| r.font_size > 0.0)
                    .max_by(|a, b| {
                        a.font_size
                            .partial_cmp(&b.font_size)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|r| (r.font_size, r.is_ahem, r.node_id))
                    .unwrap_or((0.0, false, NodeId::default()));
                // R1004：优先取 ascent_ratio_overrides 真实 per-font ratio（dormant，
                // 空 map 回退 R990 常数）。用自由函数 + 字段访问绕开 &mut self.lines 借用冲突。
                let dominant_ratio = ascent_ratio_lookup(&self.ascent_ratio_overrides, dominant_node, dominant_is_ahem);
                (line_height - dominant_fs).max(0.0) / 2.0 + dominant_fs * dominant_ratio
            } else {
                self.container_font_size * 0.8
            };
            let mut max_ascent = strut_ascent;
            for run in &line.runs {
                if matches!(
                    run.vertical_align,
                    VerticalAlignValue::Baseline | VerticalAlignValue::Sub | VerticalAlignValue::Super
                ) {
                    if run.font_size > 0.0 {
                        // 文本运行：ascent = font_size × 字体真实 ascent ratio（R990：Ahem 0.8 / 非-Ahem 0.928；
                        // R1004：优先取 ascent_ratio_overrides 真实 per-font ratio，空 map 回退 R990 常数）。
                        let run_ratio = ascent_ratio_lookup(&self.ascent_ratio_overrides, run.node_id, run.is_ahem);
                        max_ascent = max_ascent.max(run.font_size * run_ratio);
                    } else {
                        // 原子行内级盒（font_size==0 标识）：
                        // 使用 baseline 字段决定 ascent
                        // inline-block: baseline = height（底部边缘）
                        // inline-flex/inline-grid: baseline 从第一个 item 合成
                        max_ascent = max_ascent.max(run.baseline);
                    }
                }
            }
            let baseline_y = max_ascent;
            // R816 Phase 1：存储行盒度量供后续 Phase paint 复用。baseline_y=行顶到基线，
            // ascent=同，descent=行高-ascent（含 half-leading）。Phase 1 仅存储，不改变 run.y 计算。
            line.baseline_y = baseline_y;
            line.ascent = max_ascent;
            line.descent = (line_height - max_ascent).max(0.0);

            for run in &mut line.runs {
                run.y = match run.vertical_align {
                    VerticalAlignValue::Baseline => {
                        if run.font_size > 0.0 {
                            // 文本运行：保持原有计算方式（片段底部对齐到基线）
                            baseline_y - run.height
                        } else {
                            // 原子行内级盒：使用 baseline 字段定位
                            // baseline 表示从顶部到基线的距离
                            baseline_y - run.baseline
                        }
                    }
                    VerticalAlignValue::Top | VerticalAlignValue::TextTop => 0.0,
                    VerticalAlignValue::Middle => (line_height - run.height) / 2.0,
                    VerticalAlignValue::Bottom | VerticalAlignValue::TextBottom => line_height - run.height,
                    VerticalAlignValue::Sub => {
                        // 下标：基线下移 font_size × 0.3
                        let offset = run.font_size * 0.3;
                        baseline_y - run.height + offset
                    }
                    VerticalAlignValue::Super => {
                        // 上标：基线上移 font_size × 0.3
                        let offset = run.font_size * 0.3;
                        baseline_y - run.height - offset
                    }
                };
                // R1720：inline-block 垂直 margin 不下移 border box（chromium 行为：margin 扩
                // line-box 高度，border box 仍按 valign 定位）。旧 `run.y += run.margin_top`（R536）
                // 把 border box 下移 margin_top，致 valign:middle + margin over-shift（R1719 vspace
                // 根因）。A/B 实证该 shift **inert**：css-flexbox 309/497 + css-multicol 179/452
                // with/without 全等（R536 的 +52 oracle 全来自 flex-basis em 修，非此 shift）。
                // 文本运行 margin_top=0 无影响；行盒高度已含 margin box（break_lines.rs:430）。
            }

            // R822：line-box 高度 = strut ∪ valign 偏移 inline box（CSS §10.8.1）。text-top/bottom/
            // sub/super 把 inline box 移出 strut 范围，line-box 须扩展容纳（block 高度随之增长）。
            // 旧 line.height = strut only（break_into_lines 在 valign 前算 max run.box_height），
            // 致 va-117a ZW line-box 130 而 REF/chromium 175（text-bottom span box 越过 strut 顶 45px）。
            // 扩展方向：text-bottom/super 向上扩（strut 下移），text-top/sub 向下扩。baseline_y/ascent
            // 随 top_extend 下移（strut 在更高 line-box 内位置下移），line.y/run.y 不动（绝对字形位
            // 置由 paint baseline_y_abs 决定，见 Phase 3 storage）。
            let strut_fs_hl = if line.runs.iter().any(|r| r.font_size > 0.0) {
                line.runs
                    .iter()
                    .filter(|r| r.font_size > 0.0)
                    .map(|r| r.font_size)
                    .fold(0.0f32, f32::max)
            } else {
                self.container_font_size
            };
            let half_leading_hl = ((line_height - strut_fs_hl) / 2.0).max(0.0);
            let mut top_extend = 0.0f32;
            let mut bot_extend = 0.0f32;
            for run in &line.runs {
                let (up, down) = match run.vertical_align {
                    VerticalAlignValue::TextBottom => (half_leading_hl, 0.0),
                    VerticalAlignValue::TextTop => (0.0, half_leading_hl),
                    VerticalAlignValue::Super => (0.3 * run.font_size, 0.0),
                    VerticalAlignValue::Sub => (0.0, 0.3 * run.font_size),
                    _ => (0.0, 0.0),
                };
                if up > top_extend {
                    top_extend = up;
                }
                if down > bot_extend {
                    bot_extend = down;
                }
            }
            if top_extend > 0.0 || bot_extend > 0.0 {
                line.height += top_extend + bot_extend;
                line.baseline_y += top_extend;
                line.ascent += top_extend;
                line.descent += bot_extend;
            }
        }
    }

    /// 将文本按空白字符分割成单词。
    ///
    /// - `preserve_whitespace` 模式：保留空白字符序列和换行符。
    /// - `keep-all` 模式：CJK 文本不按字符拆分，而是保持为连续的"单词"。
    /// - 默认模式：按空白字符分割，每个单词追加尾部空格。
    ///   CJK 字符每个单独作为一个"单词"（CSS 规范要求 normal 模式下 CJK 允许任意断行）。
    fn split_into_words(&self, text: &str, is_ahem: bool) -> Vec<String> {
        // word-break: keep-all — CJK 字符不被视为断行点，
        // 将连续的 CJK 文本保持为一个单词（类似拉丁文本的行为）
        if self.word_break == WordBreakMode::KeepAll {
            let mut result = Vec::new();
            let mut current = String::new();
            for ch in text.chars() {
                if ch.is_ascii_whitespace() {
                    // 空白字符处可以断行
                    if !current.is_empty() {
                        result.push(format!("{current} "));
                        current.clear();
                    }
                } else {
                    current.push(ch);
                }
            }
            if !current.is_empty() {
                result.push(format!("{current} "));
            }
            return result;
        }

        // 默认模式（normal）：CJK 字符每个单独作为"单词"以允许任意断行点。
        // 非 CJK 字符按空白分割保持原有行为。
        if self.preserve_whitespace {
            // 保留空白字符序列：不折叠空格，保留换行符作为强制换行点
            let mut result = Vec::new();
            for (i, segment) in text.split('\n').enumerate() {
                if i > 0 {
                    // 换行符处产生强制换行标记（空字符串表示换行）
                    result.push(String::new());
                }
                if segment.is_empty() {
                    continue;
                }
                // 在保留空白模式下，按连续空格切分，保留空格作为独立"单词"
                // 制表符展开为 tab_size 个空格。
                // R1338：词本身**不带尾部空格**——词间间距完全由独立的 " " 片段承载
                //（这样多空格序列 "a  b" 才能正确保留为两个空格）。旧实现给每个词追加
                // `format!("{current_word} ")` 尾随空格 + 又 push 独立 " " → 词间双空格，
                // 致 pre-wrap 换行点偏早 + 行宽虚高（pre-wrap-align-* 失败根因）。
                let mut current_word = String::new();
                for ch in segment.chars() {
                    if ch == '\t' {
                        // R1447：保留制表符为独立 "\t" 片段（不展开为固定 tab_size 空格），
                        // 由 break_items_into_lines 按 tab stop（CSS Text 3 §4.1.3）推进到
                        // 下一个 tab_size 倍数。旧实现展开为 tab_size 空格 → 固定宽度，
                        // 仅在光标恰处于 tab stop 边界时正确，否则过度推进（如 "a b c\t"
                        // 在 6 字符处应进到 8，旧实现进到 14）→ text-align-justify-tabs /
                        // pre-wrap-tab / tab-position 簇失败。
                        if !current_word.is_empty() {
                            result.push(current_word.clone());
                            current_word.clear();
                        }
                        result.push("\t".to_string());
                    } else if ch == ' ' {
                        if !current_word.is_empty() {
                            result.push(current_word.clone());
                            current_word.clear();
                        }
                        // 空格也作为独立片段以保留空白（承载词间间距与多空格序列）
                        result.push(" ".to_string());
                    } else if is_per_char_break_script(ch) {
                        // CJK / R645 SEA 词典分词文字单独作为一个单词
                        if !current_word.is_empty() {
                            result.push(current_word.clone());
                            current_word.clear();
                        }
                        result.push(ch.to_string());
                    } else if ch == '\u{200B}' {
                        // R1449：ZWSP U+200B 是零宽断行机会（CSS Text 3 §5.4）——在此断词
                        //（产生换行机会），但不插入可见空格（零宽、不可见）。U+200B 本身丢弃。
                        // 与空格不同：空格作独立 " " 片段承载词间间距；ZWSP 零宽无间距。
                        // 例 "xx\u{200B}x" → ["xx","x"]，break_items_into_lines 可在两者间断行。
                        if !current_word.is_empty() {
                            result.push(current_word.clone());
                            current_word.clear();
                        }
                    } else {
                        current_word.push(ch);
                    }
                }
                if !current_word.is_empty() {
                    // 段末词不带尾部空格：源行以换行符（强制断行）或文本结尾收尾，
                    // 行尾空白由 phase II 的 hanging 处理，无需在此追加。
                    result.push(current_word.clone());
                }
            }
            if result.is_empty() {
                result.push(format!("{text} "));
            }
            result
        } else if self.break_at_newline {
            // pre-line 模式（CSS Text 3 §4.2）：空白序列折叠为单空格（同 normal），但换行符
            // `\n` 触发强制断行（同 pre-wrap 的强制断行语义）。按 `\n` 切段，每段走 normal
            // 折叠分割（collapse_split_words），段间插入空串强制断行标记——break_items_into_lines
            // 消费空串为强制断行（gate 已扩到 break_at_newline，见 break_lines.rs）。
            // 旧实现 PreLine 派生 preserve_whitespace=false → 走 normal 分支 text.split(is_collapsible_ws)
            // 把 `\n` 当普通词界折叠，丢断行 identity（content-white-space-001 簇）。
            let mut result = Vec::new();
            for (i, segment) in text.split('\n').enumerate() {
                if i > 0 {
                    // 段间强制断行标记（空串，语义同 preserve_whitespace 分支）
                    result.push(String::new());
                }
                if segment.is_empty() {
                    continue;
                }
                result.extend(self.collapse_split_words(segment, is_ahem));
            }
            if result.is_empty() {
                result.push(format!("{text} "));
            }
            result
        } else {
            self.collapse_split_words(text, is_ahem)
        }
    }

    /// normal 模式折叠分割：按可折叠白空格分割 + CJK per-char 断词。
    ///
    /// split_into_words 的 normal 分支与 pre-line 分支（每个 `\n` 段）共用此逻辑。
    fn collapse_split_words(&self, text: &str, is_ahem: bool) -> Vec<String> {
        // 标准 normal 模式：按可折叠白空格分割（R1085：is_collapsible_ws 排除 U+00A0 nbsp，
        // 保 non-breaking；split_whitespace 含 nbsp 致 nbsp-only 元素 0 行盒）。
        // R1214：is_ahem（cjk_contiguous）时 CJK per-char 连续无词间空格——Ahem 精确
        // 1em == chromium Ahem 1em，修 text-autospace ideograph-numeric/alpha-001 2em 发散
        // （旧 post-loop 给同一 step-1 词内 CJK per-char 也加 1em 词间距 → 2em）。非-Ahem 保留
        // 旧词间空格：estimate 1em ≠ chromium real font，CJK reflow 发散（welcome +7.39pp
        // 回归），须 advance-wall 解后才可对非-Ahem 启用连续。A/B 实测 +2 strict（11→13
        // oracle-pass / 4→6 strict，零 PASS→FAIL）。
        let cjk_contiguous = is_ahem;
        let mut result = Vec::new();
        let words: Vec<&str> = text.split(is_collapsible_ws).filter(|s| !s.is_empty()).collect();
        for (word_idx, word) in words.iter().enumerate() {
            let is_last_step1 = word_idx + 1 == words.len();
            // 检查单词中是否包含 CJK / R645 SEA 词典分词文字
            let has_cjk = word.chars().any(is_per_char_break_script);
            if has_cjk && self.word_break != WordBreakMode::KeepAll {
                // 将单词拆分为：连续非 CJK + 单个 CJK/SEA 交替
                let mut current_latin = String::new();
                for ch in word.chars() {
                    if is_per_char_break_script(ch) {
                        // 先推入累积的拉丁字符（cjk_contiguous 时不加尾部空格——连续）
                        if !current_latin.is_empty() {
                            if cjk_contiguous {
                                result.push(current_latin.clone());
                            } else {
                                result.push(format!("{current_latin} "));
                            }
                            current_latin.clear();
                        }
                        // CJK / R645 SEA 字符单独作为"单词"（不带尾部空格，不需要词间距）
                        result.push(ch.to_string());
                    } else {
                        current_latin.push(ch);
                    }
                }
                if !current_latin.is_empty() {
                    result.push(current_latin);
                }
            } else {
                result.push(word.to_string());
            }
            // cjk_contiguous：仅 step-1 词之间（原文本真实空格处）加尾部空格作 word-spacing
            if cjk_contiguous && !is_last_step1 {
                let needs_space = result.last().is_some_and(|last| !last.ends_with(' '));
                if needs_space {
                    if let Some(last) = result.last_mut() {
                        last.push(' ');
                    }
                }
            }
        }
        // 非-cjk_contiguous：保留旧 post-loop（给所有非末尾 result 词加尾部空格）
        if !cjk_contiguous {
            let len = result.len();
            for (i, item) in result.iter_mut().enumerate() {
                if i < len - 1 && !item.ends_with(' ') {
                    item.push(' ');
                }
            }
        }
        result
    }

    /// 获取所有行盒的总高度。
    pub fn total_height(&self) -> f32 {
        self.lines.iter().map(|line| line.height).sum()
    }

    /// 获取所有文本片段（扁平化所有行盒）。
    pub fn all_fragments(&self) -> Vec<&TextFragment> {
        self.lines.iter().flat_map(|line| line.runs.iter()).collect()
    }

    /// 获取所有文本片段，将行的 y 坐标加到每个片段的 y 上。
    ///
    /// 与 `all_fragments()` 不同，此方法返回的片段的 y 坐标是相对于容器的，
    /// 包含了行盒的累积 y 偏移量。
    pub fn all_fragments_with_line_y(&self) -> Vec<TextFragment> {
        self.lines
            .iter()
            .flat_map(|line| {
                let line_y = line.y;
                // R1456：垂直模式下 line.y 是**列 x 坐标**（break_items_into_columns
                // vertical_rtl 轴交换把列 x 存进 col.y/line.y），已在 run.x（= 列 x）中体现，
                // 不可再加到片段 y（深度 = run.y）。与 painter/text.rs:1015 stored 路径同修。
                // horizontal 仍 run.y + line_y（line.y 是行盒 y 偏移）。WM gate 零回归。
                let vert_y = self.vertical;
                line.runs.iter().map(move |run| TextFragment {
                    x: run.x,
                    y: if vert_y { run.y } else { run.y + line_y },
                    width: run.width,
                    height: run.height,
                    text: run.text.clone(),
                    source: run.source.clone(),
                    node_id: run.node_id,
                    font_size: run.font_size,
                    vertical_align: run.vertical_align.clone(),
                    is_ahem: run.is_ahem,
                    letter_spacing: run.letter_spacing,
                    margin_left: run.margin_left,
                    margin_right: run.margin_right,
                    margin_top: run.margin_top,
                    baseline: run.baseline,
                })
            })
            .collect()
    }
}

/// `ascent_ratio_for` 的自由函数内核（供 `apply_vertical_alignment` 在 `&mut self.lines`
/// 循环内调用，绕开方法调用对整个 `self` 的不可变借用——Rust 允许不相交字段借用：
/// `&self.ascent_ratio_overrides` 与 `&mut self.lines` 不冲突）。
///
/// 优先取 `overrides[node_id]`（>0 有效，由 layout IFC 经 provider 算出并存入）；
/// 否则回退 R990 is_ahem-gated 常数（Ahem 0.8 / 非-Ahem 0.928）。空 map = 全回退 = 零回归。
fn ascent_ratio_lookup(overrides: &HashMap<NodeId, f32>, node_id: NodeId, is_ahem: bool) -> f32 {
    if let Some(ratio) = overrides.get(&node_id) {
        if *ratio > 0.0 {
            return *ratio;
        }
    }
    if is_ahem { 0.8 } else { 0.928 }
}

#[cfg(test)]
mod tests;
