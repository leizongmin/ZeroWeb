//! 行内格式化上下文实现。
//!
//! 处理行内级内容的布局：文本节点、inline 元素、行换行。
//! Taffy 仅支持 Block/Flex/Grid，行内布局需要自行实现。
//! 支持文本对齐方式：left、center、right、justify。

use std::collections::HashMap;
use zero_css_parser::values::{LengthValue, VerticalAlignValue};
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
        }
    }
}

/// 行内块盒 — inline-block 元素的原子级行内盒。
///
/// inline-block 元素参与行内格式化上下文，但自身作为一个不可分割的整体
/// （不能跨行拆分），宽度/高度由其自身的块级布局计算得出。
#[derive(Debug, Clone)]
pub struct InlineBlockBox {
    /// inline-block 的宽度（px），由自身块级布局计算。
    pub width: f32,
    /// inline-block 的高度（px），由自身块级布局计算。
    pub height: f32,
    /// 对应的 DOM 节点。
    pub node_id: NodeId,
    /// vertical-align 值。
    pub vertical_align: VerticalAlignValue,
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
pub fn estimate_char_width(c: char, font_size: f32) -> f32 {
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
fn estimate_string_width(text: &str, font_size: f32) -> f32 {
    text.chars().map(|c| estimate_char_width(c, font_size)).sum()
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
    /// 生成的行盒列表。
    pub lines: Vec<LineBox>,
}

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
            lines: Vec::new(),
        }
    }

    /// 设置文本对齐方式。
    pub fn with_text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
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
                        let text = text_data.content.trim().to_string();
                        if !text.is_empty() {
                            // 文本节点没有自己的 ComputedStyle，查找父元素
                            let style = doc.parent_node(child_id).and_then(|pid| styles.get(&pid));
                            let (font_size, line_height) = resolve_font_metrics(style);
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
                            items.push(InlineItem::Text(TextRun {
                                text,
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                                letter_spacing,
                                word_spacing,
                            }));
                        }
                    }
                    NodeKind::Element(elem_data) => {
                        // `<br>` 元素产生强制换行条目
                        if elem_data.local_name() == "br" {
                            items.push(InlineItem::Br);
                            continue;
                        }
                        // 其他 inline 元素的文本内容也收集进来
                        let text = doc.text_content(child_id).unwrap_or_default();
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            let style = styles.get(&child_id);
                            let (font_size, line_height) = resolve_font_metrics(style);
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
                            items.push(InlineItem::Text(TextRun {
                                text: trimmed,
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                                letter_spacing,
                                word_spacing,
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
    /// 和 `InlineItem::Br`（强制换行）。
    pub fn break_items_into_lines(&mut self, items: Vec<InlineItem>) {
        self.lines.clear();

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
                    // 按字符类别逐字符估算宽度，替代统一 0.6 倍近似
                    let words = self.split_into_words(&run.text);

                    for (word_idx, word) in words.iter().enumerate() {
                        // 基础宽度 + letter-spacing（每个字符追加）
                        let char_count = word.chars().count();
                        let mut word_width =
                            estimate_string_width(word, run.font_size) + run.letter_spacing * char_count as f32;
                        // 非首个单词：追加 word-spacing（单词间间距）
                        // 注意：split_into_words 在每个单词后添加空格，最后一个单词也有空格
                        // word-spacing 仅在单词之间（非最后一个）或单词内含空格时生效
                        if word_idx > 0 {
                            word_width += run.word_spacing;
                        }

                        // 检查当前行是否放得下
                        if !self.no_wrap
                            && current_x + word_width > self.container_width
                            && !current_line.runs.is_empty()
                        {
                            // 当前行放不下，开始新行
                            self.lines.push(current_line);
                            current_line = LineBox {
                                y: 0.0,
                                height: 0.0,
                                runs: Vec::new(),
                            };
                            current_x = 0.0;
                        }

                        // overflow-wrap: break-word / anywhere 或 word-break: break-all —
                        // 如果单词仍超出行宽，逐字符拆分并在行边界处断行。
                        // word-break: break-all 允许在任意字符间断行（即使单词未超出行宽），
                        // 但为了简化，仅在单词超出行宽时逐字符拆分。
                        let need_char_break = !self.no_wrap
                            && (self.break_word || self.word_break == WordBreakMode::BreakAll)
                            && current_x + word_width > self.container_width
                            && !word.is_empty();
                        if need_char_break {
                            let fragment_height = run.line_height;
                            let chars: Vec<char> = word.chars().collect();
                            let mut partial_x = current_x;

                            for (ci, ch) in chars.iter().enumerate() {
                                let ch_width = estimate_char_width(*ch, run.font_size) + run.letter_spacing;

                                if partial_x + ch_width > self.container_width && ci > 0 {
                                    // 当前行满了，开始新行
                                    self.lines.push(current_line);
                                    current_line = LineBox {
                                        y: 0.0,
                                        height: 0.0,
                                        runs: Vec::new(),
                                    };
                                    partial_x = 0.0;
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
                                });

                                partial_x += ch_width;
                                current_line.height = current_line.height.max(fragment_height);
                            }
                            current_x = partial_x;
                        } else {
                            let fragment_height = run.line_height;
                            current_line.runs.push(TextFragment {
                                x: current_x,
                                y: 0.0,
                                width: word_width,
                                height: fragment_height,
                                text: word.clone(),
                                node_id: run.node_id,
                                font_size: run.font_size,
                                vertical_align: run.vertical_align.clone(),
                            });

                            current_x += word_width;
                            current_line.height = current_line.height.max(fragment_height);
                        }
                    }
                }
                InlineItem::InlineBlock(box_info) => {
                    // inline-block 是原子盒，不可拆分
                    let box_width = box_info.width;
                    let box_height = box_info.height;

                    // 检查当前行是否放得下（当行非空时）
                    if !self.no_wrap && current_x + box_width > self.container_width && !current_line.runs.is_empty() {
                        // 当前行放不下，开始新行
                        self.lines.push(current_line);
                        current_line = LineBox {
                            y: 0.0,
                            height: 0.0,
                            runs: Vec::new(),
                        };
                        current_x = 0.0;
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
                    });

                    current_x += box_width;
                    current_line.height = current_line.height.max(box_height);
                }
                InlineItem::Br => {
                    // 强制换行：将当前行推入结果，开始新行
                    // Br 总是产生一个换行，即使当前行为空
                    self.lines.push(current_line);
                    current_line = LineBox {
                        y: 0.0,
                        height: 0.0,
                        runs: Vec::new(),
                    };
                    current_x = 0.0;
                }
            }
        }

        // 添加最后一行（非空时）
        if !current_line.runs.is_empty() {
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

        let last_idx = self.lines.len() - 1;
        for (i, line) in self.lines.iter_mut().enumerate() {
            if line.runs.is_empty() {
                continue;
            }

            // 计算行内内容的总宽度（最后一个片段的右边界）
            let content_width = line.runs.last().map(|r| r.x + r.width).unwrap_or(0.0);
            let remaining = self.container_width - content_width;

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
            // 基线近似位置：行盒高度的 80% 处（对应大多数拉丁字体的基线位置）
            let baseline_y = line_height * 0.8;

            for run in &mut line.runs {
                run.y = match run.vertical_align {
                    VerticalAlignValue::Baseline => {
                        // 片段底部对齐到基线
                        baseline_y - run.height
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

        if self.preserve_whitespace {
            // 保留空白字符序列：不折叠空格，保留换行符作为强制换行点
            // 将文本按换行符分段，每段再按空格序列切分
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
                let mut current_word = String::new();
                for ch in segment.chars() {
                    if ch == ' ' || ch == '\t' {
                        if !current_word.is_empty() {
                            result.push(format!("{current_word} "));
                            current_word.clear();
                        }
                        // 空格也作为独立片段以保留空白
                        result.push(" ".to_string());
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
            text.split_whitespace().map(|w| format!("{w} ")).collect()
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
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
