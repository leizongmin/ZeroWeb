//! 行内格式化上下文实现。
//!
//! 处理行内级内容的布局：文本节点、inline 元素、行换行。
//! Taffy 仅支持 Block/Flex/Grid，行内布局需要自行实现。
//! 支持文本对齐方式：left、center、right、justify。

use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, LengthValue, VerticalAlignValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::{ComputedStyle, LineHeightValue};

/// 文本对齐方式 — 控制行内内容在行盒中的水平排列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// 左对齐（LTR 下的默认值）。
    #[default]
    Left,
    /// 右对齐。
    Right,
    /// 居中对齐。
    Center,
    /// 两端对齐 — 非最后一行时在单词间均匀分配剩余空间。
    Justify,
}

/// 文本运行 — 一段连续的、具有相同样式的文本。
#[derive(Debug, Clone)]
pub struct TextRun {
    /// 文本内容。
    pub text: String,
    /// 对应的 DOM 节点（文本节点或 inline 元素）。
    pub node_id: NodeId,
    /// 字体大小（px）。
    pub font_size: f32,
    /// 行高（px）。
    pub line_height: f32,
    /// vertical-align 值。
    pub vertical_align: VerticalAlignValue,
    /// letter-spacing（px），每个字符后追加的额外间距。
    #[doc(hidden)]
    pub letter_spacing: f32,
    /// word-spacing（px），空格字符后追加的额外间距。
    #[doc(hidden)]
    pub word_spacing: f32,
    /// inline 元素的水平 margin（px）。文本节点为 0。
    pub margin_left: f32,
    /// inline 元素的水平 margin（px）。文本节点为 0。
    pub margin_right: f32,
    /// inline 元素的上内边距（px）。文本节点为 0。
    /// CSS 2.1 规范要求 inline 元素的 padding 参与行盒高度计算。
    pub padding_top: f32,
    /// inline 元素的下内边距（px）。文本节点为 0。
    pub padding_bottom: f32,
    /// inline 元素的上边框宽度（px）。文本节点为 0。
    /// CSS 2.1 规范要求 inline 元素的 border 参与行盒高度计算。
    pub border_top: f32,
    /// inline 元素的下边框宽度（px）。文本节点为 0。
    pub border_bottom: f32,
    /// 是否使用 Ahem 字体（所有字符宽度等于 font_size）。
    pub is_ahem_font: bool,
}

impl TextRun {
    /// 创建简单的 TextRun（letter_spacing=0, word_spacing=0）。
    ///
    /// 用于测试和不需要间距的场景。
    pub fn simple(
        text: String,
        node_id: NodeId,
        font_size: f32,
        line_height: f32,
        vertical_align: VerticalAlignValue,
    ) -> Self {
        Self {
            text,
            node_id,
            font_size,
            line_height,
            vertical_align,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            margin_left: 0.0,
            margin_right: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            border_top: 0.0,
            border_bottom: 0.0,
            is_ahem_font: false,
        }
    }

    /// CSS 2.1 inline 元素对行盒高度的贡献：
    /// line-height + padding-top + padding-bottom + border-top + border-bottom。
    ///
    /// CSS 2.1 §10.8.1: inline 元素的内容区高度由 line-height 决定，
    /// 但 padding 和 border 会额外增加行盒中该元素占据的垂直空间。
    pub fn box_height(&self) -> f32 {
        self.line_height + self.padding_top + self.padding_bottom + self.border_top + self.border_bottom
    }
}

/// 行内块盒 — inline-block / inline-flex / inline-grid / inline-table 元素的原子级行内盒。
///
/// 这些元素参与行内格式化上下文，但自身作为一个不可分割的整体
/// （不能跨行拆分），宽度/高度由其自身的块级布局计算得出。
#[derive(Debug, Clone)]
pub struct InlineBlockBox {
    /// 盒的宽度（px），由自身块级布局计算。
    pub width: f32,
    /// 盒的高度（px），由自身块级布局计算。
    pub height: f32,
    /// 对应的 DOM 节点。
    pub node_id: NodeId,
    /// vertical-align 值。
    pub vertical_align: VerticalAlignValue,
    /// 基线高度（px）— 从盒顶部到基线的距离。
    ///
    /// - inline-block：基线在底部边缘，`baseline = height`
    /// - inline-flex/inline-grid：基线从第一个 flex/grid item 合成
    ///   （简化为 `height / 2` 作为回退，理想情况应从 taffy first_baselines 提取）
    /// - inline-table：基线为第一行单元格的基线
    pub baseline: f32,
}

/// 行内级条目 — 行内格式化上下文中的原子单位。
///
/// 区分文本运行、inline-block 盒和强制换行：
/// - `Text` — 可按单词拆分的文本运行
/// - `InlineBlock` — 不可拆分的原子行内级盒
/// - `Br` — 强制换行（`<br>` 元素）
#[derive(Debug, Clone)]
pub enum InlineItem {
    /// 可按单词拆分的文本运行。
    Text(TextRun),
    /// 不可拆分的 inline-block 盒（原子行内级盒）。
    InlineBlock(InlineBlockBox),
    /// 强制换行 — 由 `<br>` 元素产生。
    Br,
}

/// 行盒 — 一行中的所有行内内容。
#[derive(Debug, Clone)]
pub struct LineBox {
    /// 行盒的 y 坐标（相对于包含块的内容区域）。
    pub y: f32,
    /// 行盒的高度。
    pub height: f32,
    /// 行盒中的文本片段列表。
    pub runs: Vec<TextFragment>,
}

/// 文本片段 — 文本运行在行盒中的布局结果。
#[derive(Debug, Clone)]
pub struct TextFragment {
    /// 片段在行盒中的 x 坐标。
    pub x: f32,
    /// 片段在行盒中的 y 坐标（相对于行盒顶部）。
    pub y: f32,
    /// 片段的宽度。
    pub width: f32,
    /// 片段的高度。
    pub height: f32,
    /// 文本内容。
    pub text: String,
    /// 对应的 DOM 节点。
    pub node_id: NodeId,
    /// 字体大小。
    pub font_size: f32,
    /// vertical-align 值。
    pub vertical_align: VerticalAlignValue,
    /// 是否使用 Ahem 字体（影响字形宽度：Ahem 为 1.0×font_size）。
    pub is_ahem: bool,
    /// letter-spacing（px），每个字符后追加的额外间距。
    pub letter_spacing: f32,
    /// 基线高度（px）— 从片段顶部到基线的距离。
    ///
    /// - 文本运行：baseline = font_size（ascent 近似）
    /// - inline-block：baseline = height（基线在底部边缘）
    /// - inline-flex/inline-grid：baseline 从第一个 item 合成
    pub baseline: f32,
}

/// 默认字体大小（px）。
const DEFAULT_FONT_SIZE: f32 = 16.0;

/// 根据字符类别估算单个字符的宽度。
///
/// 不同类别的字符具有不同的典型宽度比例：
/// - CJK 字符（中日韩统一表意文字）：全宽，约等于 font_size
/// - ASCII 字母：约 font_size × 0.55
/// - 空格：约 font_size × 0.25
/// - 标点符号：约 font_size × 0.4
/// - 数字：约 font_size × 0.5
/// - 其他字符（默认）：约 font_size × 0.5
///
/// Ahem 字体特殊处理：所有字符宽度等于 font_size（WPT 标准正方形字体）。
pub fn estimate_char_width(c: char, font_size: f32, is_ahem: bool) -> f32 {
    if is_ahem {
        // Ahem 字体：所有字符（包括空格）宽度等于 font_size
        return font_size;
    }
    if c.is_ascii_whitespace() {
        // 空格类字符：较窄
        font_size * 0.25
    } else if is_cjk_character(c) {
        // CJK 全角字符：宽度约等于字体大小
        font_size
    } else if is_emoji_character(c) {
        // Emoji 通常占一个全角宽度
        font_size
    } else if c.is_ascii_punctuation() {
        // ASCII 标点：比字母窄
        font_size * 0.4
    } else if c.is_ascii_digit() {
        // 数字：略窄于字母
        font_size * 0.5
    } else if c.is_ascii_alphabetic() {
        // ASCII 字母
        font_size * 0.55
    } else {
        // 其他 Unicode 字符（非 CJK）：默认宽度
        font_size * 0.5
    }
}

/// 判断字符是否属于 CJK（中日韩）范围。
///
/// 覆盖常见 CJK Unicode 区块：
/// - U+4E00..=U+9FFF — CJK 统一表意文字（基本区）
/// - U+3400..=U+4DBF — CJK 统一表意文字扩展 A
/// - U+F900..=U+FAFF — CJK 兼容表意文字
/// - U+3000..=U+303F — CJK 符号和标点
/// - U+FF00..=U+FFEF — 半角及全角形式
/// - U+2E80..=U+2EFF — CJK 部首补充
/// - U+3040..=U+309F — 平假名
/// - U+30A0..=U+30FF — 片假名
/// - U+AC00..=U+D7AF — 韩文音节
fn is_cjk_character(c: char) -> bool {
    matches!(
        c,
        '\u{4E00}'..='\u{9FFF}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{3000}'..='\u{303F}'
        | '\u{FF00}'..='\u{FFEF}'
        | '\u{2E80}'..='\u{2EFF}'
        | '\u{3040}'..='\u{309F}'
        | '\u{30A0}'..='\u{30FF}'
        | '\u{AC00}'..='\u{D7AF}'
    )
}

/// 判断字符是否为 emoji 或常见符号（非 CJK）。
fn is_emoji_character(c: char) -> bool {
    let cp = c as u32;
    (0x1F300..=0x1FAFF).contains(&cp)
        || (0x2600..=0x26FF).contains(&cp)
        || (0x2700..=0x27BF).contains(&cp)
        || (0xFE00..=0xFE0F).contains(&cp)
        || (0x1F1E6..=0x1F1FF).contains(&cp)
}

/// 估算字符串的总宽度，按每个字符逐一计算。
fn estimate_string_width(text: &str, font_size: f32, is_ahem: bool) -> f32 {
    text.chars().map(|c| estimate_char_width(c, font_size, is_ahem)).sum()
}

/// CSS Text §4.1 白空格折叠：将连续空白字符折叠为单个空格。
///
/// 与 `trim()` 不同，此函数保留首尾的空格（作为单个空格）。
/// 仅含空白的输入返回单个空格（用于 inline-block 之间的间隔）。
/// 空输入返回空字符串。
///
/// 行首/行尾空格的剥离由 IFC 的 `break_items_into_lines` 在行级别处理。
fn collapse_whitespace(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut result = String::with_capacity(text.len());
    let mut last_was_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }
    result
}

/// 默认行高倍数（用于 line-height: normal）。
const NORMAL_LINE_HEIGHT_RATIO: f32 = 1.2;

/// 从 ComputedStyle 中解析 font-size 和 line-height。
///
/// - `font_size` 从 `ComputedStyle::font_size` 中提取（已解析为 Px）。
/// - `line_height` 根据 `LineHeightValue` 计算：
///   - `Normal` → font_size × 1.2
///   - `Number(n)` → font_size × n
///   - `Length(Px(v))` → v
///
/// 当 style 为 None 时（节点没有样式），返回默认值 16.0 / 19.2。
pub fn resolve_font_metrics(style: Option<&ComputedStyle>) -> (f32, f32) {
    let font_size = match style {
        Some(s) => match &s.font_size {
            LengthValue::Px(v) => *v as f32,
            _ => DEFAULT_FONT_SIZE,
        },
        None => DEFAULT_FONT_SIZE,
    };

    let line_height = match style {
        Some(s) => match &s.line_height {
            LineHeightValue::Normal => font_size * NORMAL_LINE_HEIGHT_RATIO,
            LineHeightValue::Number(n) => font_size * (*n as f32),
            LineHeightValue::Length(LengthValue::Px(v)) => *v as f32,
            // 其他长度类型（em/rem 等）在 resolve 阶段应已转换为 Px，
            // 这里做防御性回退
            LineHeightValue::Length(_) => font_size * NORMAL_LINE_HEIGHT_RATIO,
        },
        None => DEFAULT_FONT_SIZE * NORMAL_LINE_HEIGHT_RATIO,
    };

    (font_size, line_height)
}

/// 从 CSS LengthValue 解析 inline-block 元素的尺寸（宽度或高度）。
///
/// 支持 Px、Em、Rem 等绝对长度单位。Auto、Percentage、MinContent 等返回 0.0
/// （inline-block 在行内格式化上下文测量阶段无法确定这些值，需要 taffy 布局后回填）。
pub fn resolve_inline_block_dimension(value: &LengthValue, style: &ComputedStyle, _is_width: bool) -> f32 {
    match value {
        LengthValue::Px(v) => *v as f32,
        LengthValue::Em(v) => {
            let base = match &style.font_size {
                LengthValue::Px(fs) => *fs as f32,
                _ => 16.0,
            };
            *v as f32 * base
        }
        LengthValue::Rem(v) => *v as f32 * 16.0, // 假设 root em = 16px
        _ => 0.0,                                // Auto、Percentage、MinContent 等暂不支持
    }
}

/// CSS word-break 行为 — 控制单词内的断行规则。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordBreakMode {
    /// normal — 标准断行规则。
    #[default]
    Normal,
    /// break-all — 允许在任意两个字符间断行（包括非 CJK 文本）。
    BreakAll,
    /// keep-all — 禁止在 CJK 字符间断行（CJK 文本视为单词）。
    KeepAll,
}

/// 浮动排除区域 — 描述一个浮动元素占据的空间。
///
/// 浮动元素（float: left/right）会占据行内内容的一部分空间，
/// 导致文本在浮动元素周围环绕排列。
#[derive(Debug, Clone)]
pub struct FloatExclusion {
    /// 排除区域的起始 y 坐标（相对于容器内容区域顶部）。
    pub y: f32,
    /// 排除区域的高度。
    pub height: f32,
    /// 排除区域占据的宽度（px）。
    pub width: f32,
    /// 浮动方向：true = 左浮动，false = 右浮动。
    pub is_left: bool,
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
    /// CSS word-break 行为。
    pub word_break: WordBreakMode,
    /// 首行文本缩进（CSS text-indent，px）。仅影响第一行的起始 x 坐标。
    pub text_indent: f32,
    /// CSS tab-size（px）— 制表符展开宽度。默认 8 个空格宽度。
    pub tab_size: f32,
    /// 浮动排除区域 — 浮动元素占据的空间，文本需环绕排列。
    pub float_exclusions: Vec<FloatExclusion>,
    /// 生成的行盒列表。
    pub lines: Vec<LineBox>,
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
    /// inline-block 元素的预计算尺寸（来自 LayoutBox / taffy 布局结果）。
    ///
    /// 当 CSS 属性 width/height 为 Auto 时，inline-block 的尺寸由其内容决定，
    /// IFC 无法自行测量，需要外部布局结果提供。
    pub inline_block_sizes: HashMap<NodeId, (f32, f32)>,
    /// 默认字体度量 — 当 styles HashMap 中找不到元素样式时使用。
    ///
    /// 这主要用于 paint 系统的 IFC，因为 paint 系统传入空的 styles HashMap。
    /// 设置此值后，文本节点在找不到父元素样式时会使用此默认值
    /// 而非硬编码的 16px/19.2px。
    pub default_font_metrics: Option<(f32, f32)>,
    /// 逐文本节点的字体大小覆盖（key = 文本节点的父元素 NodeId）。
    ///
    /// paint IFC 传入空的 styles HashMap，导致所有文本使用 16px 默认字体度量，
    /// 行断计算与 layout IFC 不一致。此字段存储 layout IFC 为每个文本节点
    /// 计算的实际 font_size，以父元素 ID 为键（因为 collect_inline_items
    /// 查找的是文本节点的父元素样式）。
    /// 当 styles 中找不到父元素样式且此映射有对应条目时，使用映射中的
    /// font_size 而非 16px 默认值，使字符宽度计算更准确。
    pub font_size_overrides: HashMap<NodeId, f32>,
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
    /// 行内级盒的基线覆盖（key = 元素 NodeId）。
    ///
    /// 用于 inline-flex/inline-grid 等元素，其基线应从第一个子元素的布局位置
    /// 合成，而非使用简单的 height/2 回退。由 adjust_inline_block_positions
    /// 从 LayoutBox 子元素位置计算后传入。
    pub baseline_overrides: HashMap<NodeId, f32>,
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
            word_break: WordBreakMode::default(),
            text_indent: 0.0,
            tab_size: DEFAULT_TAB_SIZE,
            float_exclusions: Vec::new(),
            lines: Vec::new(),
            vertical: false,
            vertical_rtl: false,
            inline_block_sizes: HashMap::new(),
            default_font_metrics: None,
            font_size_overrides: HashMap::new(),
            is_ahem_overrides: HashMap::new(),
            letter_spacing_overrides: HashMap::new(),
            baseline_overrides: HashMap::new(),
        }
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

    /// 设置 inline-block 元素的预计算尺寸（来自 LayoutBox / taffy 布局结果）。
    pub fn with_inline_block_sizes(mut self, sizes: HashMap<NodeId, (f32, f32)>) -> Self {
        self.inline_block_sizes = sizes;
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

    /// 设置 word-break 行为。
    pub fn with_word_break(mut self, mode: WordBreakMode) -> Self {
        self.word_break = mode;
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
        let items = self.collect_inline_items(doc, container, styles);
        self.break_items_into_lines(items);
    }

    /// 收集容器中所有行内级内容（文本节点 + inline 元素 + `<br>` 元素），
    /// 从 ComputedStyle 中读取 font-size 和 line-height。
    fn collect_inline_items(
        &self,
        doc: &Document,
        container: NodeId,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> Vec<InlineItem> {
        let mut items = Vec::new();
        let children = doc.child_nodes(container);

        for &child_id in &children {
            if let Some(node) = doc.get(child_id) {
                match &node.kind {
                    NodeKind::Text(text_data) => {
                        // CSS Text §4.1: 白空格折叠 — 将连续空白字符折叠为单个空格，
                        // 但不在此阶段去除（行首/行尾空格由 IFC break_items_into_lines 处理）。
                        // 保留仅含空白的文本节点为单个空格（用于 inline-block 之间的间隔）。
                        let text = collapse_whitespace(&text_data.content);
                        if !text.is_empty() {
                            // 文本节点没有自己的 ComputedStyle，查找父元素
                            let parent_id = doc.parent_node(child_id);
                            let style = parent_id.and_then(|pid| styles.get(&pid));
                            let (font_size, line_height) = if style.is_some() {
                                resolve_font_metrics(style)
                            } else if let Some(pid) = parent_id {
                                // paint IFC 传入空 styles：使用 layout IFC 存储的 font_size 覆盖
                                // 替代 16px 默认值，使字符宽度和行高计算更准确
                                if let Some(&fs) = self.font_size_overrides.get(&pid) {
                                    (fs, fs * NORMAL_LINE_HEIGHT_RATIO)
                                } else {
                                    self.default_font_metrics
                                        .unwrap_or((DEFAULT_FONT_SIZE, DEFAULT_FONT_SIZE * NORMAL_LINE_HEIGHT_RATIO))
                                }
                            } else {
                                self.default_font_metrics
                                    .unwrap_or((DEFAULT_FONT_SIZE, DEFAULT_FONT_SIZE * NORMAL_LINE_HEIGHT_RATIO))
                            };
                            let vertical_align = style
                                .map(|s| s.vertical_align.clone())
                                .unwrap_or(VerticalAlignValue::Baseline);
                            let letter_spacing = style
                                .map(|s| match &s.letter_spacing {
                                    LengthValue::Px(v) => *v as f32,
                                    _ => 0.0,
                                })
                                .unwrap_or_else(|| {
                                    // paint IFC（空 styles）：使用覆盖映射获取 letter-spacing
                                    parent_id
                                        .and_then(|pid| self.letter_spacing_overrides.get(&pid).copied())
                                        .unwrap_or(0.0)
                                });
                            let word_spacing = style
                                .map(|s| match &s.word_spacing {
                                    LengthValue::Px(v) => *v as f32,
                                    _ => 0.0,
                                })
                                .unwrap_or(0.0);
                            let is_ahem_font = style
                                .map(|s| s.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem")))
                                .unwrap_or_else(|| {
                                    // paint IFC（空 styles）：使用覆盖映射检测 Ahem 字体
                                    parent_id
                                        .and_then(|pid| self.is_ahem_overrides.get(&pid).copied())
                                        .unwrap_or(false)
                                });
                            items.push(InlineItem::Text(TextRun {
                                text,
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                                letter_spacing,
                                word_spacing,
                                margin_left: 0.0,
                                margin_right: 0.0,
                                padding_top: 0.0,
                                padding_bottom: 0.0,
                                border_top: 0.0,
                                border_bottom: 0.0,
                                is_ahem_font,
                            }));
                        }
                    }
                    NodeKind::Element(elem_data) => {
                        // `<br>` 元素产生强制换行条目
                        if elem_data.local_name() == "br" {
                            items.push(InlineItem::Br);
                            continue;
                        }

                        // CSS 2.1 §9.2.1.1 匿名块盒生成：
                        // 当 inline 元素包含 block-level 子元素时，inline 元素
                        // 被拆分为匿名块盒。这里简化处理：如果子元素是 block-level
                        // display，强制换行（与 <br> 类似），跳过其文本内容。
                        // block-level 子元素由 taffy 正常布局为独立的块盒。
                        let style = styles.get(&child_id);
                        let is_block_level = style.is_some_and(|s| {
                            matches!(
                                s.display,
                                DisplayValue::Block
                                    | DisplayValue::Flex
                                    | DisplayValue::Grid
                                    | DisplayValue::Table
                                    | DisplayValue::ListItem
                                    | DisplayValue::FlowRoot
                            )
                        });
                        if is_block_level {
                            // 强制换行：inline 内容在此中断
                            items.push(InlineItem::Br);
                            continue;
                        }

                        // 检查该元素是否为原子行内级盒（inline-block / inline-flex / inline-grid / inline-table）。
                        // 这些元素参与行内格式化上下文，作为不可拆分的原子盒。
                        let style = styles.get(&child_id);
                        let is_inline_block = style.is_some_and(|s| {
                            matches!(
                                s.display,
                                DisplayValue::InlineBlock
                                    | DisplayValue::InlineFlex
                                    | DisplayValue::InlineGrid
                                    | DisplayValue::InlineTable
                            )
                        });

                        if is_inline_block {
                            let s = style.unwrap();
                            // 从 CSS 计算样式提取尺寸（仅支持绝对长度单位）
                            let mut w = resolve_inline_block_dimension(&s.width, s, /* is_width */ true);
                            let mut h = resolve_inline_block_dimension(&s.height, s, /* is_width */ false);
                            // 仅当 CSS 属性为 Auto 时，使用 LayoutBox 预计算尺寸回退。
                            // Percentage、MinContent 等其他值不回退——它们有自己的解析逻辑。
                            let need_lb = w <= 0.0 || h <= 0.0;
                            if need_lb && let Some(&(lw, lh)) = self.inline_block_sizes.get(&child_id) {
                                if matches!(s.width, LengthValue::Auto) {
                                    w = lw;
                                }
                                if matches!(s.height, LengthValue::Auto) {
                                    h = lh;
                                }
                            }
                            if w > 0.0 && h > 0.0 {
                                let vertical_align = s.vertical_align.clone();
                                // 计算基线：
                                // - inline-block：基线在底部边缘
                                // - inline-flex/inline-grid：基线从第一个子元素合成
                                //   优先使用 baseline_overrides（由 adjust_inline_block_positions
                                //   从 LayoutBox 子元素位置计算），回退到 height/2
                                let baseline = if let Some(&b) = self.baseline_overrides.get(&child_id) {
                                    b
                                } else {
                                    match s.display {
                                        DisplayValue::InlineFlex | DisplayValue::InlineGrid => h * 0.5,
                                        _ => h, // inline-block, inline-table: 基线在底部
                                    }
                                };
                                items.push(InlineItem::InlineBlock(InlineBlockBox {
                                    width: w,
                                    height: h,
                                    node_id: child_id,
                                    vertical_align,
                                    baseline,
                                }));
                                continue;
                            }
                            // 无有效尺寸的 inline-block 降级为零宽度 TextRun
                        }

                        // `<img>` 替换元素：使用 HTML width/height 属性作为固有尺寸，
                        // 创建 InlineBlock 条目（原子盒，不可拆分）。
                        if elem_data.local_name() == "img" {
                            let w = elem_data
                                .get_attribute("width")
                                .and_then(|v| v.parse::<f32>().ok())
                                .unwrap_or(0.0)
                                .max(0.0);
                            let h = elem_data
                                .get_attribute("height")
                                .and_then(|v| v.parse::<f32>().ok())
                                .unwrap_or(0.0)
                                .max(0.0);
                            if w > 0.0 && h > 0.0 {
                                let vertical_align = styles
                                    .get(&child_id)
                                    .map(|s| s.vertical_align.clone())
                                    .unwrap_or(VerticalAlignValue::Baseline);
                                // img 替换元素的基线在底部边缘
                                items.push(InlineItem::InlineBlock(InlineBlockBox {
                                    width: w,
                                    height: h,
                                    node_id: child_id,
                                    vertical_align,
                                    baseline: h,
                                }));
                                continue;
                            }
                            // 无有效尺寸的 img 降级为零宽度 TextRun
                        }

                        // 其他 inline 元素的文本内容也收集进来
                        let text = doc.text_content(child_id).unwrap_or_default();
                        let trimmed = collapse_whitespace(&text);
                        let style = styles.get(&child_id);
                        let (font_size, line_height) = if style.is_some() {
                            resolve_font_metrics(style)
                        } else {
                            self.default_font_metrics
                                .unwrap_or((DEFAULT_FONT_SIZE, DEFAULT_FONT_SIZE * NORMAL_LINE_HEIGHT_RATIO))
                        };
                        let vertical_align = style
                            .map(|s| s.vertical_align.clone())
                            .unwrap_or(VerticalAlignValue::Baseline);
                        let letter_spacing = style
                            .map(|s| match &s.letter_spacing {
                                LengthValue::Px(v) => *v as f32,
                                _ => 0.0,
                            })
                            .unwrap_or(0.0);
                        let word_spacing = style
                            .map(|s| match &s.word_spacing {
                                LengthValue::Px(v) => *v as f32,
                                _ => 0.0,
                            })
                            .unwrap_or(0.0);
                        // 提取 inline 元素的水平 margin
                        let margin_left = style
                            .map(|s| match &s.margin_left {
                                LengthValue::Px(v) => *v as f32,
                                _ => 0.0,
                            })
                            .unwrap_or(0.0);
                        let margin_right = style
                            .map(|s| match &s.margin_right {
                                LengthValue::Px(v) => *v as f32,
                                _ => 0.0,
                            })
                            .unwrap_or(0.0);
                        let is_ahem_font = style
                            .map(|s| s.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem")))
                            .unwrap_or(false);
                        // CSS 2.1: inline 元素的 padding 和 border 参与行盒高度计算
                        let (padding_top, padding_bottom, border_top, border_bottom) =
                            Self::extract_inline_box_metrics(style);
                        if !trimmed.is_empty() {
                            items.push(InlineItem::Text(TextRun {
                                text: trimmed,
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                                letter_spacing,
                                word_spacing,
                                margin_left,
                                margin_right,
                                padding_top,
                                padding_bottom,
                                border_top,
                                border_bottom,
                                is_ahem_font,
                            }));
                        } else {
                            // CSS 规范：空 inline 元素仍需通过 line-height + padding + border 影响行盒高度
                            // 生成零宽度 TextRun，贡献 line-height + padding + border
                            items.push(InlineItem::Text(TextRun {
                                text: String::new(),
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                                letter_spacing: 0.0,
                                word_spacing: 0.0,
                                margin_left,
                                margin_right,
                                padding_top,
                                padding_bottom,
                                border_top,
                                border_bottom,
                                is_ahem_font,
                            }));
                        }
                    }
                    _ => {}
                }
            }
        }

        items
    }

    /// 将文本运行按可用宽度分割成行盒。
    ///
    /// 便捷方法：将 `Vec<TextRun>` 包装为 `InlineItem::Text` 后调用 [`break_items_into_lines`]。
    pub fn break_into_lines(&mut self, runs: Vec<TextRun>) {
        let items: Vec<InlineItem> = runs.into_iter().map(InlineItem::Text).collect();
        self.break_items_into_lines(items);
    }

    /// 将行内级条目按可用宽度分割成行盒。
    ///
    /// 支持 `InlineItem::Text`（按单词拆分行）、`InlineItem::InlineBlock`（原子盒，不可拆分）
    /// 和 `InlineItem::Br`（强制换行）。浮动排除区域会缩小每行的可用宽度。
    pub fn break_items_into_lines(&mut self, items: Vec<InlineItem>) {
        self.lines.clear();

        if self.vertical {
            self.break_items_into_columns(items);
            return;
        }

        // 追踪当前行的 y 偏移量（用于计算浮动排除区域）
        let mut current_y = 0.0_f32;
        // 估算默认行高（用于初始浮动排除计算）
        let default_line_height = 20.0_f32;

        let mut current_line = LineBox {
            y: 0.0,
            height: 0.0,
            runs: Vec::new(),
        };
        // text-indent 仅作用于首行
        let mut current_x = self.text_indent;

        for item in items {
            match item {
                InlineItem::Text(run) => {
                    // 应用 BiDi 重排序（RTL 文本需要视觉顺序）
                    let visual_text = bidi_reorder(&run.text);
                    // 按字符类别逐字符估算宽度，替代统一 0.6 倍近似
                    let words = self.split_into_words(&visual_text);

                    // 空 inline 元素：文本为空但 line-height + padding + border 仍需贡献到行盒高度
                    if words.is_empty() && run.text.is_empty() {
                        if run.box_height() > current_line.height {
                            current_line.height = run.box_height();
                        }
                        // 即使空元素也要消费 margin（CSS 2.1 §10.2：inline 元素的 margin 水平方向有效）
                        if run.margin_left > 0.0 {
                            current_x += run.margin_left;
                        }
                        if run.margin_right > 0.0 {
                            current_x += run.margin_right;
                        }
                        continue;
                    }

                    // 在第一个词之前添加 margin-left
                    if run.margin_left > 0.0 {
                        current_x += run.margin_left;
                    }

                    for (word_idx, word) in words.iter().enumerate() {
                        // CSS 2.1 §16.6.1：normal/nowrap 模式下行尾空格不渲染，不计入行宽。
                        // 将尾部空格从内容宽度中分离，仅作为词间距离使用。
                        // pre/pre-wrap 模式（preserve_whitespace）空格不可折叠，不剥离。
                        let (content_word, trailing_space_width) = if !self.preserve_whitespace && word.ends_with(' ') {
                            let trimmed = word.trim_end_matches(' ');
                            let space_count = word.len() - trimmed.len();
                            let space_w =
                                estimate_char_width(' ', run.font_size, run.is_ahem_font) * space_count as f32;
                            (trimmed, space_w)
                        } else {
                            (word.as_str(), 0.0f32)
                        };

                        // CSS 2.1 §16.6.1：行首空格不渲染。
                        // 当前行首的第一个词如果以空格开头，去除前导空格。
                        let content_word = if current_line.runs.is_empty()
                            && !self.preserve_whitespace
                            && content_word.starts_with(' ')
                        {
                            content_word.trim_start_matches(' ')
                        } else {
                            content_word
                        };
                        // 全空格词在行首不产生任何渲染
                        if content_word.is_empty() {
                            continue;
                        }

                        // 基础宽度 + letter-spacing（仅基于内容字符，不含尾部空格）
                        let content_char_count = content_word.chars().count();
                        let mut word_width = estimate_string_width(content_word, run.font_size, run.is_ahem_font)
                            + run.letter_spacing * content_char_count as f32;
                        // 非首个单词：追加 word-spacing（单词间间距）
                        if word_idx > 0 {
                            word_width += run.word_spacing;
                        }

                        // 计算当前行的有效可用宽度（扣除浮动排除区域）
                        let est_height = if current_line.height > 0.0 {
                            current_line.height
                        } else {
                            run.line_height.max(default_line_height)
                        };
                        let (left_offset, avail_width) = self.effective_content_area(current_y, est_height);

                        // 调整 current_x 到浮动排除区域之后（仅在行首且无 text-indent 时）
                        if current_line.runs.is_empty() && self.text_indent >= 0.0 && current_x < left_offset {
                            current_x = left_offset;
                        }

                        // 检查当前行是否放得下（使用有效可用宽度）
                        if !self.no_wrap
                            && current_x + word_width > left_offset + avail_width
                            && !current_line.runs.is_empty()
                        {
                            // 当前行放不下，开始新行
                            self.lines.push(current_line);
                            current_y += est_height;
                            current_line = LineBox {
                                y: 0.0,
                                height: 0.0,
                                runs: Vec::new(),
                            };
                            // 新行重新计算浮动偏移
                            let (new_left, _) = self.effective_content_area(current_y, run.box_height());
                            current_x = new_left;
                        }

                        // 计算当前有效宽度（可能在换行后更新）
                        let (_, avail_w) =
                            self.effective_content_area(current_y, current_line.height.max(run.box_height()));

                        // overflow-wrap: break-word / anywhere 或 word-break: break-all
                        let need_char_break = !self.no_wrap
                            && (self.break_word || self.word_break == WordBreakMode::BreakAll)
                            && current_x + word_width > current_x + avail_w
                            && !content_word.is_empty();
                        if need_char_break {
                            let fragment_height = run.line_height;
                            let chars: Vec<char> = content_word.chars().collect();
                            let mut partial_x = current_x;

                            for (ci, ch) in chars.iter().enumerate() {
                                let ch_width =
                                    estimate_char_width(*ch, run.font_size, run.is_ahem_font) + run.letter_spacing;

                                let (_, avail) =
                                    self.effective_content_area(current_y, current_line.height.max(run.box_height()));
                                let line_limit = current_line.runs.first().map_or(partial_x, |r| r.x) + avail;

                                if partial_x + ch_width > line_limit && ci > 0 {
                                    // 当前行满了，开始新行
                                    self.lines.push(current_line);
                                    current_y += fragment_height;
                                    current_line = LineBox {
                                        y: 0.0,
                                        height: 0.0,
                                        runs: Vec::new(),
                                    };
                                    let (new_left, _) = self.effective_content_area(current_y, fragment_height);
                                    partial_x = new_left;
                                }

                                current_line.runs.push(TextFragment {
                                    x: partial_x,
                                    y: 0.0,
                                    width: ch_width,
                                    height: fragment_height,
                                    text: ch.to_string(),
                                    node_id: run.node_id,
                                    font_size: run.font_size,
                                    vertical_align: run.vertical_align.clone(),
                                    is_ahem: run.is_ahem_font,
                                    letter_spacing: run.letter_spacing,
                                    baseline: run.font_size,
                                });

                                partial_x += ch_width;
                                // 行盒高度需容纳 inline 元素的完整盒体（含 padding+border）
                                current_line.height = current_line.height.max(run.box_height());
                            }
                            current_x = partial_x;
                        } else {
                            let fragment_height = run.line_height;
                            // word_width 已不含尾部空格（在上方剥离），直接用作可视宽度
                            // 尾部空格作为词间距离添加到 current_x
                            current_line.runs.push(TextFragment {
                                x: current_x,
                                y: 0.0,
                                width: word_width,
                                height: fragment_height,
                                text: content_word.to_string(),
                                node_id: run.node_id,
                                font_size: run.font_size,
                                vertical_align: run.vertical_align.clone(),
                                is_ahem: run.is_ahem_font,
                                letter_spacing: run.letter_spacing,
                                baseline: run.font_size,
                            });

                            current_x += word_width + trailing_space_width;
                            // 行盒高度需容纳 inline 元素的完整盒体（含 padding+border）
                            current_line.height = current_line.height.max(run.box_height());
                        }
                    }

                    // 在最后一个词之后添加 margin-right
                    if run.margin_right > 0.0 {
                        current_x += run.margin_right;
                    }
                }
                InlineItem::InlineBlock(box_info) => {
                    // inline-block 是原子盒，不可拆分
                    let box_width = box_info.width;
                    let box_height = box_info.height;

                    let est_height = if current_line.height > 0.0 {
                        current_line.height
                    } else {
                        box_height.max(default_line_height)
                    };
                    let (left_offset, avail_width) = self.effective_content_area(current_y, est_height);

                    // 调整 current_x 到浮动排除区域之后
                    if current_line.runs.is_empty() && current_x < left_offset {
                        current_x = left_offset;
                    }

                    // 检查当前行是否放得下（当行非空时）
                    if !self.no_wrap
                        && current_x + box_width > left_offset + avail_width
                        && !current_line.runs.is_empty()
                    {
                        // 当前行放不下，开始新行
                        self.lines.push(current_line);
                        current_y += est_height;
                        current_line = LineBox {
                            y: 0.0,
                            height: 0.0,
                            runs: Vec::new(),
                        };
                        let (new_left, _) = self.effective_content_area(current_y, box_height);
                        current_x = new_left;
                    }

                    // inline-block 片段不使用 font_size，设为 0
                    current_line.runs.push(TextFragment {
                        x: current_x,
                        y: 0.0,
                        width: box_width,
                        height: box_height,
                        text: String::new(),
                        node_id: box_info.node_id,
                        font_size: 0.0,
                        vertical_align: box_info.vertical_align.clone(),
                        is_ahem: false,
                        letter_spacing: 0.0,
                        baseline: box_info.baseline,
                    });

                    current_x += box_width;
                    current_line.height = current_line.height.max(box_height);
                }
                InlineItem::Br => {
                    // 强制换行：将当前行推入结果，开始新行
                    // Br 总是产生一个换行，即使当前行为空
                    let est_height = if current_line.height > 0.0 {
                        current_line.height
                    } else {
                        default_line_height
                    };
                    self.lines.push(current_line);
                    current_y += est_height;
                    current_line = LineBox {
                        y: 0.0,
                        height: 0.0,
                        runs: Vec::new(),
                    };
                    let (new_left, _) = self.effective_content_area(current_y, default_line_height);
                    current_x = new_left;
                }
            }
        }

        // 添加最后一行（非空时）
        // CSS 2.1 §10.8.1：空 inline 元素的 line-height 仍贡献到行盒高度，
        // 即使没有文本片段，行盒高度 > 0 时也需要保留。
        if !current_line.runs.is_empty() || current_line.height > 0.0 {
            self.lines.push(current_line);
        }

        // 计算每行的 y 坐标
        let mut y = 0.0;
        for line in &mut self.lines {
            line.y = y;
            y += line.height;
        }

        // 应用文本对齐
        self.apply_text_alignment();

        // 应用 vertical-align 对齐
        self.apply_vertical_alignment();
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

        // 预计算每行的有效内容区域（避免在 iter_mut 中借用 self）
        let line_areas: Vec<(f32, f32)> = self
            .lines
            .iter()
            .map(|line| self.effective_content_area(line.y, line.height))
            .collect();

        let last_idx = self.lines.len() - 1;
        for (i, line) in self.lines.iter_mut().enumerate() {
            if line.runs.is_empty() {
                continue;
            }

            // 计算行内内容的总宽度（最后一个片段的右边界）
            let content_width = line.runs.last().map(|r| r.x + r.width).unwrap_or(0.0);

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
        };
        // 当前深度（字符沿 y 向下推进的位置）
        let mut current_depth = self.text_indent;

        for item in items {
            match item {
                InlineItem::Text(run) => {
                    let visual_text = bidi_reorder(&run.text);
                    let words = self.split_into_words(&visual_text);

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
                        let mut word_height = estimate_string_width(word, run.font_size, run.is_ahem_font)
                            + run.letter_spacing * char_count as f32;
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
                                let ch_height =
                                    estimate_char_width(*ch, run.font_size, run.is_ahem_font) + run.letter_spacing;

                                if partial_depth + ch_height > max_depth && ci > 0 {
                                    self.lines.push(current_column);
                                    current_column = LineBox {
                                        y: 0.0,
                                        height: 0.0,
                                        runs: Vec::new(),
                                    };
                                    partial_depth = 0.0;
                                }

                                current_column.runs.push(TextFragment {
                                    x: 0.0,
                                    y: partial_depth,
                                    width: char_col_width,
                                    height: ch_height,
                                    text: ch.to_string(),
                                    node_id: run.node_id,
                                    font_size: run.font_size,
                                    vertical_align: run.vertical_align.clone(),
                                    is_ahem: run.is_ahem_font,
                                    letter_spacing: run.letter_spacing,
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
                                node_id: run.node_id,
                                font_size: run.font_size,
                                vertical_align: run.vertical_align.clone(),
                                is_ahem: run.is_ahem_font,
                                letter_spacing: run.letter_spacing,
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
                        };
                        current_depth = 0.0;
                    }

                    current_column.runs.push(TextFragment {
                        x: 0.0,
                        y: current_depth,
                        width: box_col_width,
                        height: box_depth,
                        text: String::new(),
                        node_id: box_info.node_id,
                        font_size: 0.0,
                        vertical_align: box_info.vertical_align.clone(),
                        is_ahem: false,
                        letter_spacing: 0.0,
                        baseline: box_info.baseline,
                    });

                    current_depth += box_depth;
                    current_column.height = current_column.height.max(box_col_width);
                }
                InlineItem::Br => {
                    self.lines.push(current_column);
                    current_column = LineBox {
                        y: 0.0,
                        height: 0.0,
                        runs: Vec::new(),
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
            // vertical-rl：第一列在右侧，后续列向左排列
            let mut x = self.container_width; // 从容器右端开始
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
            // - 空行使用 strut ascent（0.8 × line_height）作为回退
            //
            // 修正：从实际内容计算基线，而非仅使用全局 strut 近似。
            // 这修复了 Ahem 等字体（ascent ≈ font_size）的基线位置问题，
            // 同时保持对正常字体（ascent ≈ 0.8 × font_size）的兼容性。
            let strut_ascent = line_height * 0.8;
            let mut max_ascent = strut_ascent;
            for run in &line.runs {
                if matches!(
                    run.vertical_align,
                    VerticalAlignValue::Baseline | VerticalAlignValue::Sub | VerticalAlignValue::Super
                ) {
                    if run.font_size > 0.0 {
                        // 文本运行：ascent = font_size（字体度量的近似值）
                        max_ascent = max_ascent.max(run.font_size);
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
            }
        }
    }

    /// 将文本按空白字符分割成单词。
    ///
    /// - `preserve_whitespace` 模式：保留空白字符序列和换行符。
    /// - `keep-all` 模式：CJK 文本不按字符拆分，而是保持为连续的"单词"。
    /// - 默认模式：按空白字符分割，每个单词追加尾部空格。
    ///   CJK 字符每个单独作为一个"单词"（CSS 规范要求 normal 模式下 CJK 允许任意断行）。
    fn split_into_words(&self, text: &str) -> Vec<String> {
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
                // 制表符展开为 tab_size 个空格
                let mut current_word = String::new();
                for ch in segment.chars() {
                    if ch == '\t' {
                        // 制表符展开为 tab_size 个空格
                        if !current_word.is_empty() {
                            result.push(format!("{current_word} "));
                            current_word.clear();
                        }
                        let tab_spaces = " ".repeat(self.tab_size.max(1.0) as usize);
                        result.push(tab_spaces);
                    } else if ch == ' ' {
                        if !current_word.is_empty() {
                            result.push(format!("{current_word} "));
                            current_word.clear();
                        }
                        // 空格也作为独立片段以保留空白
                        result.push(" ".to_string());
                    } else if is_cjk_character(ch) {
                        // CJK 字符单独作为一个单词
                        if !current_word.is_empty() {
                            result.push(format!("{current_word} "));
                            current_word.clear();
                        }
                        result.push(ch.to_string());
                    } else {
                        current_word.push(ch);
                    }
                }
                if !current_word.is_empty() {
                    result.push(format!("{current_word} "));
                }
            }
            if result.is_empty() {
                result.push(format!("{text} "));
            }
            result
        } else {
            // 标准 normal 模式：按空白分割，CJK 字符每个单独作为"单词"
            let mut result = Vec::new();
            for word in text.split_whitespace() {
                // 检查单词中是否包含 CJK 字符
                let has_cjk = word.chars().any(is_cjk_character);
                if has_cjk && self.word_break != WordBreakMode::KeepAll {
                    // 将单词拆分为：连续非 CJK + 单个 CJK 交替
                    let mut current_latin = String::new();
                    for ch in word.chars() {
                        if is_cjk_character(ch) {
                            // 先推入累积的拉丁字符
                            if !current_latin.is_empty() {
                                result.push(format!("{current_latin} "));
                                current_latin.clear();
                            }
                            // CJK 字符单独作为"单词"（不带尾部空格，不需要词间距）
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
            }
            // 为非末尾单词添加尾部空格（表示词间距 advance width）
            // 最后一个单词不加——它后面没有下一个单词，不应有额外间隙
            let len = result.len();
            for (i, item) in result.iter_mut().enumerate() {
                if i < len - 1 && !item.ends_with(' ') {
                    item.push(' ');
                }
            }
            result
        }
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
                line.runs.iter().map(move |run| TextFragment {
                    x: run.x,
                    y: run.y + line_y,
                    width: run.width,
                    height: run.height,
                    text: run.text.clone(),
                    node_id: run.node_id,
                    font_size: run.font_size,
                    vertical_align: run.vertical_align.clone(),
                    is_ahem: run.is_ahem,
                    letter_spacing: run.letter_spacing,
                    baseline: run.baseline,
                })
            })
            .collect()
    }
}

/// 对文本进行 BiDi 重排序，返回视觉顺序的字符串。
///
/// 使用 unicode-bidi 库分析文本的嵌入层级，对 RTL 段落进行重排序。
/// 如果文本不需要重排序（纯 LTR），返回原始文本。
fn bidi_reorder(text: &str) -> String {
    use unicode_bidi::BidiInfo;

    // 快速检查：如果文本为空或全是 ASCII，不需要 BiDi 处理
    if text.is_empty() || text.is_ascii() {
        return text.to_string();
    }

    // 检查是否包含 RTL 字符
    let has_rtl = text.chars().any(|ch| {
        let cp = ch as u32;
        // Hebrew: 0x0590–0x05FF, Arabic: 0x0600–0x06FF, Syriac: 0x0700–0x074F
        // Arabic Extended: 0x08A0–0x08FF, Arabic Presentation Forms: 0xFB50–0xFDFF, 0xFE70–0xFEFF
        (0x0590..=0x05FF).contains(&cp)
            || (0x0600..=0x06FF).contains(&cp)
            || (0x0700..=0x074F).contains(&cp)
            || (0x08A0..=0x08FF).contains(&cp)
            || (0xFB50..=0xFDFF).contains(&cp)
            || (0xFE70..=0xFEFF).contains(&cp)
    });

    if !has_rtl {
        return text.to_string();
    }

    // 运行 BiDi 算法
    let bidi_info = BidiInfo::new(text, None);
    if bidi_info.levels.is_empty() {
        return text.to_string();
    }

    // 查找段落信息
    let para = unicode_bidi::ParagraphInfo {
        range: 0..text.len(),
        level: unicode_bidi::Level::ltr(),
    };

    // 对整个文本段落进行重排序
    let reordered = bidi_info.reorder_line(&para, 0..text.len());
    reordered.into_owned()
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
