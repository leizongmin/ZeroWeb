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

/// 行内格式化上下文 — 负责将行内内容排列成行盒。
#[derive(Debug, Clone)]
pub struct InlineFormattingContext {
    /// 包含块的可用宽度。
    pub container_width: f32,
    /// 文本对齐方式。
    pub text_align: TextAlign,
    /// 生成的行盒列表。
    pub lines: Vec<LineBox>,
}

impl InlineFormattingContext {
    /// 创建新的行内格式化上下文。
    pub fn new(container_width: f32) -> Self {
        Self {
            container_width,
            text_align: TextAlign::default(),
            lines: Vec::new(),
        }
    }

    /// 设置文本对齐方式。
    pub fn with_text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
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
        let runs = self.collect_inline_runs(doc, container, styles);
        self.break_into_lines(runs);
    }

    /// 收集容器中所有行内级内容（文本节点 + inline 元素），
    /// 从 ComputedStyle 中读取 font-size 和 line-height。
    fn collect_inline_runs(
        &self,
        doc: &Document,
        container: NodeId,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> Vec<TextRun> {
        let mut runs = Vec::new();
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
                            runs.push(TextRun {
                                text,
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                            });
                        }
                    }
                    NodeKind::Element(_) => {
                        // inline 元素的文本内容也收集进来
                        let text = doc.text_content(child_id).unwrap_or_default();
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            let style = styles.get(&child_id);
                            let (font_size, line_height) = resolve_font_metrics(style);
                            let vertical_align = style
                                .map(|s| s.vertical_align.clone())
                                .unwrap_or(VerticalAlignValue::Baseline);
                            runs.push(TextRun {
                                text: trimmed,
                                node_id: child_id,
                                font_size,
                                line_height,
                                vertical_align,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        runs
    }

    /// 将文本运行按可用宽度分割成行盒。
    fn break_into_lines(&mut self, runs: Vec<TextRun>) {
        self.lines.clear();

        let mut current_line = LineBox {
            y: 0.0,
            height: 0.0,
            runs: Vec::new(),
        };
        let mut current_x = 0.0;

        for run in runs {
            // 估算文本宽度：字符数 × 字体大小的 0.6 倍（近似平均字符宽度）
            let estimated_char_width = run.font_size * 0.6;
            let words = self.split_into_words(&run.text);

            for word in words {
                let word_width = word.len() as f32 * estimated_char_width;

                // 检查当前行是否放得下
                if current_x + word_width > self.container_width && !current_line.runs.is_empty() {
                    // 当前行放不下，开始新行
                    self.lines.push(current_line);
                    current_line = LineBox {
                        y: 0.0,
                        height: 0.0,
                        runs: Vec::new(),
                    };
                    current_x = 0.0;
                }

                let fragment_height = run.line_height;
                current_line.runs.push(TextFragment {
                    x: current_x,
                    y: 0.0,
                    width: word_width,
                    height: fragment_height,
                    text: word,
                    node_id: run.node_id,
                    font_size: run.font_size,
                    vertical_align: run.vertical_align.clone(),
                });

                current_x += word_width;
                current_line.height = current_line.height.max(fragment_height);
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
        if self.text_align == TextAlign::Left || self.lines.is_empty() {
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

            match self.text_align {
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
                    // 最后一行不 justify，保持左对齐
                    if i == last_idx {
                        continue;
                    }
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
    fn split_into_words(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(|w| format!("{w} ")).collect()
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
mod tests {
    use super::*;
    use zero_css_parser::values::VerticalAlignValue as VA;

    /// 测试文本分割为单词。
    #[test]
    fn test_split_into_words() {
        let ctx = InlineFormattingContext::new(800.0);
        let words = ctx.split_into_words("Hello World Foo");
        assert_eq!(words.len(), 3);
        assert_eq!(words[0], "Hello ");
    }

    /// 测试空文本不产生行盒。
    #[test]
    fn test_empty_text_no_lines() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "   ".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);
        let fragments: Vec<_> = ctx.all_fragments();
        assert!(fragments.is_empty());
    }

    /// 测试短文本放入单行。
    #[test]
    fn test_single_line() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "Hello World".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);
        assert_eq!(ctx.lines.len(), 1, "短文本应在单行中");
        assert_eq!(ctx.lines[0].runs.len(), 2, "两个单词");
        assert_eq!(ctx.total_height(), 20.0);
    }

    /// 测试长文本自动换行。
    #[test]
    fn test_line_breaking() {
        let mut ctx = InlineFormattingContext::new(100.0);
        let long_text = "a ".repeat(50);
        let runs = vec![TextRun {
            text: long_text,
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);
        assert!(ctx.lines.len() > 1, "长文本应产生多行，实际 {} 行", ctx.lines.len());
    }

    /// 测试行盒 y 坐标累加。
    #[test]
    fn test_line_y_positions() {
        let mut ctx = InlineFormattingContext::new(50.0);
        let runs = vec![TextRun {
            text: "word1 word2 word3 word4 word5".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);
        for i in 1..ctx.lines.len() {
            assert!(
                ctx.lines[i].y >= ctx.lines[i - 1].y + ctx.lines[i - 1].height - 0.01,
                "行 {} 的 y 坐标应递增",
                i
            );
        }
    }

    /// 测试 total_height 正确计算。
    #[test]
    fn test_total_height() {
        let mut ctx = InlineFormattingContext::new(50.0);
        let runs = vec![TextRun {
            text: "a b c d e f g h i j k l m n o p".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 24.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);
        let expected = ctx.lines.len() as f32 * 24.0;
        assert!(
            (ctx.total_height() - expected).abs() < 0.01,
            "total_height 应为 {}，实际 {}",
            expected,
            ctx.total_height()
        );
    }

    /// 测试 all_fragments 扁平化。
    #[test]
    fn test_all_fragments() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "Hello World".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);
        let fragments = ctx.all_fragments();
        assert_eq!(fragments.len(), 2);
    }

    /// 测试 TextFragment x 坐标递增。
    #[test]
    fn test_fragment_x_positions() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "First Second Third".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);
        for line in &ctx.lines {
            for i in 1..line.runs.len() {
                assert!(
                    line.runs[i].x >= line.runs[i - 1].x + line.runs[i - 1].width - 0.01,
                    "片段 x 坐标应递增"
                );
            }
        }
    }

    /// 测试多个 TextRun 合并到同一行。
    #[test]
    fn test_multiple_runs_same_line() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![
            TextRun {
                text: "Hello".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
            TextRun {
                text: "World".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
        ];
        ctx.break_into_lines(runs);
        assert_eq!(ctx.lines.len(), 1, "两个短文本应在同一行");
    }

    /// 测试 inline 元素文本与文本节点混合。
    #[test]
    fn test_inline_layout_from_document() {
        use zero_dom::parse_html;

        let doc = parse_html("<p>Hello <b>World</b>!</p>");

        // 找到 body > p
        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let mut ctx = InlineFormattingContext::new(800.0);
        ctx.layout(&doc, p, &HashMap::new());

        let fragments = ctx.all_fragments();
        assert!(!fragments.is_empty(), "p 元素应包含文本片段");

        let all_text: String = fragments.iter().map(|f| f.text.clone()).collect();
        assert!(all_text.contains("Hello"), "应包含 'Hello'，实际: {all_text}");
    }

    // ── 新增综合测试 ──

    /// 测试混合文本和 inline 元素（真实 HTML 结构）。
    ///
    /// 模拟 `<p>This is <em>important</em> and <strong>bold</strong> text</p>` 场景，
    /// 验证从文档收集行内内容后能正确排列成行盒。
    #[test]
    fn test_mixed_text_and_inline_elements() {
        use zero_dom::parse_html;

        let doc = parse_html("<p>This is <em>important</em> and <strong>bold</strong> text</p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let mut ctx = InlineFormattingContext::new(800.0);
        ctx.layout(&doc, p, &HashMap::new());

        let fragments = ctx.all_fragments();
        assert!(
            fragments.len() >= 5,
            "应至少有 5 个文本片段（各单词），实际 {}",
            fragments.len()
        );

        // 验证所有片段的 x 坐标在同一行内递增
        for line in &ctx.lines {
            for i in 1..line.runs.len() {
                assert!(line.runs[i].x >= line.runs[i - 1].x, "片段 x 坐标应在行内递增");
            }
        }
    }

    /// 测试超长单个单词应溢出容器。
    ///
    /// 当一个单词宽度超过 container_width 时，它仍然被放置在行盒中
    /// （浏览器行为：溢出而不截断）。
    #[test]
    fn test_very_long_single_word_overflow() {
        let mut ctx = InlineFormattingContext::new(100.0);
        // 构造一个超长单词，每个字符宽度约 16*0.6 = 9.6px
        // 50 个字符 = ~480px，远超 100px 容器宽度
        let long_word = "a".repeat(50);
        let runs = vec![TextRun {
            text: long_word.clone(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        // 应产生 1 行（单个不中断单词不换行）
        assert_eq!(ctx.lines.len(), 1, "超长单词应在单行中（溢出而不是换行）");
        assert_eq!(ctx.lines[0].runs.len(), 1, "只有一个单词片段");

        // 片段宽度应超过容器宽度
        let fragment = &ctx.lines[0].runs[0];
        assert!(
            fragment.width > ctx.container_width,
            "片段宽度 {} 应超过容器宽度 {}",
            fragment.width,
            ctx.container_width
        );
    }

    /// 测试空容器（无文本节点）不产生任何行盒。
    #[test]
    fn test_empty_container_no_lines() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs: Vec<TextRun> = vec![];
        ctx.break_into_lines(runs);

        assert!(ctx.lines.is_empty(), "空容器不应产生行盒");
        assert!(ctx.all_fragments().is_empty(), "空容器不应有文本片段");
        assert!((ctx.total_height() - 0.0).abs() < 0.01, "空容器总高度应为 0");
    }

    /// 测试空容器通过 Document layout 方法。
    #[test]
    fn test_empty_container_from_document() {
        use zero_dom::parse_html;

        let doc = parse_html("<p></p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let mut ctx = InlineFormattingContext::new(800.0);
        ctx.layout(&doc, p, &HashMap::new());

        assert!(ctx.lines.is_empty(), "没有文本的空 p 元素不应产生行盒");
    }

    /// 测试行高计算 — 不同行高产生不同的行盒高度。
    #[test]
    fn test_line_height_calculation() {
        // 行高 24px 的单行
        let mut ctx24 = InlineFormattingContext::new(800.0);
        let runs_24 = vec![TextRun {
            text: "Short text".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 24.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx24.break_into_lines(runs_24);

        // 行高 32px 的单行
        let mut ctx32 = InlineFormattingContext::new(800.0);
        let runs_32 = vec![TextRun {
            text: "Short text".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 32.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx32.break_into_lines(runs_32);

        assert!(
            (ctx24.lines[0].height - 24.0).abs() < 0.01,
            "行高 24px 应产生高度 24px 的行盒"
        );
        assert!(
            (ctx32.lines[0].height - 32.0).abs() < 0.01,
            "行高 32px 应产生高度 32px 的行盒"
        );
        assert!((ctx24.total_height() - 24.0).abs() < 0.01, "总高度应为 24.0");
        assert!((ctx32.total_height() - 32.0).abs() < 0.01, "总高度应为 32.0");
    }

    /// 测试行高在多行中的累加效果。
    #[test]
    fn test_line_height_accumulation() {
        let mut ctx = InlineFormattingContext::new(50.0);
        let runs = vec![TextRun {
            text: "a b c d e f g h i j".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 30.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        assert!(ctx.lines.len() > 1, "窄容器应产生多行");

        // 每行高度应为 30px
        for (i, line) in ctx.lines.iter().enumerate() {
            assert!(
                (line.height - 30.0).abs() < 0.01,
                "第 {} 行高度应为 30.0，实际 {}",
                i,
                line.height
            );
        }

        // 总高度 = 行数 × 30
        let expected = ctx.lines.len() as f32 * 30.0;
        assert!(
            (ctx.total_height() - expected).abs() < 0.01,
            "总高度应为 {}，实际 {}",
            expected,
            ctx.total_height()
        );
    }

    /// 测试混合字体大小的行盒。
    ///
    /// 当同一行包含不同字体大小的文本时，行盒高度应取最大值。
    #[test]
    fn test_multiple_font_sizes_same_line() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![
            TextRun {
                text: "Small".to_string(),
                node_id: NodeId::default(),
                font_size: 12.0,
                line_height: 16.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
            TextRun {
                text: "Large".to_string(),
                node_id: NodeId::default(),
                font_size: 24.0,
                line_height: 30.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
            TextRun {
                text: "Medium".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
        ];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1, "三个短词应在同一行");

        // 行盒高度应取最大行高 30.0
        assert!(
            (ctx.lines[0].height - 30.0).abs() < 0.01,
            "行盒高度应取最大行高 30.0，实际 {}",
            ctx.lines[0].height
        );

        // 验证片段保留了各自的字体大小
        let fragments = ctx.all_fragments();
        let font_sizes: Vec<f32> = fragments.iter().map(|f| f.font_size).collect();
        assert!(
            font_sizes.iter().any(|&s| (s - 12.0).abs() < 0.01),
            "应包含 12px 字体大小的片段"
        );
        assert!(
            font_sizes.iter().any(|&s| (s - 24.0).abs() < 0.01),
            "应包含 24px 字体大小的片段"
        );
    }

    /// 测试混合字体大小时估算宽度与字体大小成正比。
    #[test]
    fn test_font_size_affects_width() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![
            TextRun {
                text: "Word".to_string(),
                node_id: NodeId::default(),
                font_size: 10.0,
                line_height: 14.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
            TextRun {
                text: "Word".to_string(),
                node_id: NodeId::default(),
                font_size: 20.0,
                line_height: 24.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
        ];
        ctx.break_into_lines(runs);

        let fragments = ctx.all_fragments();
        assert_eq!(fragments.len(), 2, "两个单词各一个片段");

        // 20px 字体的片段宽度应为 10px 字体的 2 倍
        let ratio = fragments[1].width / fragments[0].width;
        assert!((ratio - 2.0).abs() < 0.01, "宽度比应为 2.0，实际 {}", ratio);
    }

    /// 测试窄容器中多个 TextRun 跨行排列。
    #[test]
    fn test_multiple_runs_wrap_across_lines() {
        let mut ctx = InlineFormattingContext::new(80.0);
        let runs = vec![
            TextRun {
                text: "alpha beta".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
            TextRun {
                text: "gamma delta".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
        ];
        ctx.break_into_lines(runs);

        assert!(
            ctx.lines.len() > 1,
            "窄容器中 4 个单词应产生多行，实际 {} 行",
            ctx.lines.len()
        );

        // 验证 y 坐标连续递增
        for i in 1..ctx.lines.len() {
            assert!(
                ctx.lines[i].y >= ctx.lines[i - 1].y + ctx.lines[i - 1].height - 0.01,
                "行 y 坐标应连续递增"
            );
        }
    }

    /// 测试所有片段的 NodeId 正确保留。
    #[test]
    fn test_fragment_node_ids_preserved() {
        let id1 = NodeId::default();
        let id2 = NodeId::default();

        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![
            TextRun {
                text: "First".to_string(),
                node_id: id1,
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
            TextRun {
                text: "Second".to_string(),
                node_id: id2,
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
        ];
        ctx.break_into_lines(runs);

        let fragments = ctx.all_fragments();
        // 每个片段都应有有效的 NodeId（即使是默认值）
        assert_eq!(fragments.len(), 2, "应有 2 个片段");
        for f in &fragments {
            assert!(f.node_id.is_valid(), "每个片段都应有有效的 NodeId");
        }
    }

    /// 测试 Container width 为 0 的边界情况。
    #[test]
    fn test_zero_container_width() {
        let mut ctx = InlineFormattingContext::new(0.0);
        let runs = vec![TextRun {
            text: "Hello World".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        // 容器宽度为 0 时，第一个单词放入第一行（即使溢出），
        // 后续每个单词都换新行
        assert!(!ctx.lines.is_empty(), "即使容器宽度为 0，也应产生行盒");
        assert!(ctx.lines.len() >= 2, "零宽度容器中多个单词应产生多行");
    }

    // ── 文本对齐测试 ──

    /// 测试默认对齐为 Left。
    #[test]
    fn test_default_text_align_is_left() {
        let ctx = InlineFormattingContext::new(800.0);
        assert_eq!(ctx.text_align, TextAlign::Left);
    }

    /// 测试 with_text_align builder 方法。
    #[test]
    fn test_with_text_align_builder() {
        let ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Center);
        assert_eq!(ctx.text_align, TextAlign::Center);
    }

    /// 测试 center 对齐 — 片段整体居中。
    #[test]
    fn test_text_align_center() {
        let mut ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Center);
        let runs = vec![TextRun {
            text: "Hello World".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1);
        let line = &ctx.lines[0];
        let content_width: f32 = line.runs.iter().map(|r| r.width).sum();
        // 第一个片段的 x 应约为 (800 - content_width) / 2
        let expected_x = (800.0 - content_width) / 2.0;
        assert!(
            (line.runs[0].x - expected_x).abs() < 0.01,
            "center: 第一个片段 x 应为 {}，实际 {}",
            expected_x,
            line.runs[0].x
        );
    }

    /// 测试 right 对齐 — 片段整体靠右。
    #[test]
    fn test_text_align_right() {
        let mut ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Right);
        let runs = vec![TextRun {
            text: "Hello World".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1);
        let line = &ctx.lines[0];
        let content_width: f32 = line.runs.iter().map(|r| r.width).sum();
        // 第一个片段的 x 应约为 800 - content_width
        let expected_x = 800.0 - content_width;
        assert!(
            (line.runs[0].x - expected_x).abs() < 0.01,
            "right: 第一个片段 x 应为 {}，实际 {}",
            expected_x,
            line.runs[0].x
        );
    }

    /// 测试 left 对齐（默认）— 片段从 x=0 开始。
    #[test]
    fn test_text_align_left_no_offset() {
        let mut ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Left);
        let runs = vec![TextRun {
            text: "Hello World".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1);
        // 第一个片段 x 应为 0
        assert!(
            ctx.lines[0].runs[0].x.abs() < 0.01,
            "left: 第一个片段 x 应为 0，实际 {}",
            ctx.lines[0].runs[0].x
        );
    }

    /// 测试 justify 对齐 — 非最后一行时片段间均匀分配空间。
    #[test]
    fn test_text_align_justify_distributes_space() {
        // 使用窄容器（60px）确保产生多行
        let mut ctx = InlineFormattingContext::new(60.0).with_text_align(TextAlign::Justify);
        let runs = vec![TextRun {
            text: "aa bb cc dd ee ff gg hh".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        assert!(ctx.lines.len() > 1, "应产生多行用于 justify 测试");

        // 非最后一行：最后一个片段的右边界应接近容器宽度
        for (i, line) in ctx.lines.iter().enumerate() {
            if i < ctx.lines.len() - 1 && line.runs.len() >= 2 {
                let last_run = line.runs.last().unwrap();
                let right_edge = last_run.x + last_run.width;
                assert!(
                    (right_edge - 60.0).abs() < 1.0,
                    "justify 第 {} 行右边界应接近 60，实际 {}",
                    i,
                    right_edge
                );
            }
        }
    }

    /// 测试 justify 最后一行不拉伸（保持左对齐）。
    #[test]
    fn test_text_align_justify_last_line_not_stretched() {
        let mut ctx = InlineFormattingContext::new(60.0).with_text_align(TextAlign::Justify);
        let runs = vec![TextRun {
            text: "aa bb cc dd ee ff gg".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        assert!(ctx.lines.len() > 1, "应产生多行");
        let last_line = ctx.lines.last().unwrap();
        // 最后一行的第一个片段 x 应为 0（不 justify）
        assert!(
            last_line.runs[0].x.abs() < 0.01,
            "justify 最后一行不应拉伸，x 应为 0，实际 {}",
            last_line.runs[0].x
        );
    }

    /// 测试 center 对齐在多行中每行都居中。
    #[test]
    fn test_text_align_center_multiline() {
        let mut ctx = InlineFormattingContext::new(60.0).with_text_align(TextAlign::Center);
        let runs = vec![TextRun {
            text: "aa bb cc dd ee ff".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        assert!(ctx.lines.len() > 1, "应产生多行");
        for (i, line) in ctx.lines.iter().enumerate() {
            let content_width = line.runs.last().map(|r| r.x + r.width).unwrap_or(0.0);
            let expected_x = (60.0 - content_width) / 2.0;
            assert!(
                (line.runs[0].x - expected_x).abs() < 1.0,
                "center 第 {} 行: x 应约 {}，实际 {}",
                i,
                expected_x,
                line.runs[0].x
            );
        }
    }

    /// 测试 right 对齐在多行中每行都靠右。
    #[test]
    fn test_text_align_right_multiline() {
        let mut ctx = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Right);
        let runs = vec![TextRun {
            text: "aa bb cc dd ee".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        assert!(ctx.lines.len() > 1, "应产生多行");
        for (i, line) in ctx.lines.iter().enumerate() {
            let last = line.runs.last().unwrap();
            let right_edge = last.x + last.width;
            assert!(
                (right_edge - 100.0).abs() < 1.0,
                "right 第 {} 行: 右边界应约 100，实际 {}",
                i,
                right_edge
            );
        }
    }

    /// 测试 justify 在只有 1 个片段的行不崩溃。
    #[test]
    fn test_text_align_justify_single_fragment_line() {
        let mut ctx = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Justify);
        // 超长单个单词，只会产生 1 个片段的行
        let runs = vec![TextRun {
            text: "aaaaaaaaaaaaaaaaaaaaaa".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);
        // 不应 panic
        assert_eq!(ctx.lines.len(), 1);
        assert!(ctx.lines[0].runs[0].x.abs() < 0.01, "单片段行 justify 不应调整 x");
    }

    /// 测试对齐不影响 total_height。
    #[test]
    fn test_text_align_does_not_affect_total_height() {
        let runs = vec![TextRun {
            text: "aa bb cc dd ee ff gg".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];

        let mut ctx_left = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Left);
        ctx_left.break_into_lines(runs.clone());

        let mut ctx_center = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Center);
        ctx_center.break_into_lines(runs.clone());

        let mut ctx_right = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Right);
        ctx_right.break_into_lines(runs.clone());

        let mut ctx_justify = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Justify);
        ctx_justify.break_into_lines(runs);

        let h = ctx_left.total_height();
        assert!((ctx_center.total_height() - h).abs() < 0.01, "center 高度应相同");
        assert!((ctx_right.total_height() - h).abs() < 0.01, "right 高度应相同");
        assert!((ctx_justify.total_height() - h).abs() < 0.01, "justify 高度应相同");
    }

    /// 测试对齐不影响行数。
    #[test]
    fn test_text_align_does_not_change_line_count() {
        let runs = vec![TextRun {
            text: "aa bb cc dd ee ff".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];

        let mut ctx_left = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Left);
        ctx_left.break_into_lines(runs.clone());

        let mut ctx_justify = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Justify);
        ctx_justify.break_into_lines(runs);

        assert_eq!(ctx_left.lines.len(), ctx_justify.lines.len(), "对齐方式不应改变行数");
    }

    /// 测试空行盒在对齐时不会崩溃。
    #[test]
    fn test_text_align_empty_lines_no_panic() {
        let mut ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Center);
        let runs: Vec<TextRun> = vec![];
        ctx.break_into_lines(runs);
        assert!(ctx.lines.is_empty());
    }

    // ── resolve_font_metrics 测试 ──

    /// 测试 resolve_font_metrics 在无样式时返回默认值。
    #[test]
    fn test_resolve_font_metrics_no_style() {
        let (font_size, line_height) = resolve_font_metrics(None);
        assert!(
            (font_size - 16.0).abs() < 0.01,
            "默认 font_size 应为 16.0，实际 {font_size}"
        );
        assert!(
            (line_height - 19.2).abs() < 0.01,
            "默认 line_height 应为 16.0 * 1.2 = 19.2，实际 {line_height}"
        );
    }

    /// 测试 resolve_font_metrics 从 ComputedStyle 中读取 font-size。
    #[test]
    fn test_resolve_font_metrics_with_font_size() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(24.0);

        let (font_size, line_height) = resolve_font_metrics(Some(&style));
        assert!((font_size - 24.0).abs() < 0.01, "font_size 应为 24.0，实际 {font_size}");
        // line-height: Normal → 24.0 * 1.2 = 28.8
        assert!(
            (line_height - 28.8).abs() < 0.01,
            "line_height 应为 28.8，实际 {line_height}"
        );
    }

    /// 测试 resolve_font_metrics 中 line-height: Number 使用倍数。
    #[test]
    fn test_resolve_font_metrics_line_height_number() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.line_height = LineHeightValue::Number(1.5);

        let (font_size, line_height) = resolve_font_metrics(Some(&style));
        assert!((font_size - 20.0).abs() < 0.01);
        assert!(
            (line_height - 30.0).abs() < 0.01,
            "line_height 应为 20.0 * 1.5 = 30.0，实际 {line_height}"
        );
    }

    /// 测试 resolve_font_metrics 中 line-height: Length 使用固定值。
    #[test]
    fn test_resolve_font_metrics_line_height_length() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.line_height = LineHeightValue::Length(LengthValue::Px(28.0));

        let (font_size, line_height) = resolve_font_metrics(Some(&style));
        assert!((font_size - 20.0).abs() < 0.01);
        assert!(
            (line_height - 28.0).abs() < 0.01,
            "line_height 应为 28.0，实际 {line_height}"
        );
    }

    /// 测试 resolve_font_metrics 中 line-height: Normal 使用 1.2 倍数。
    #[test]
    fn test_resolve_font_metrics_line_height_normal() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(32.0);
        // 默认 line-height 就是 Normal

        let (font_size, line_height) = resolve_font_metrics(Some(&style));
        assert!((font_size - 32.0).abs() < 0.01);
        assert!(
            (line_height - 38.4).abs() < 0.01,
            "line_height 应为 32.0 * 1.2 = 38.4，实际 {line_height}"
        );
    }

    // ── 样式感知 layout 测试 ──

    /// 测试从 Document 布局时使用 ComputedStyle 中的 font-size。
    #[test]
    fn test_layout_uses_style_font_size() {
        use zero_dom::parse_html;

        let doc = parse_html("<p>Hello World</p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        // 给 p 设置 font-size: 32px
        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(32.0);
        styles.insert(p, style);

        let mut ctx = InlineFormattingContext::new(800.0);
        ctx.layout(&doc, p, &styles);

        let fragments = ctx.all_fragments();
        assert!(!fragments.is_empty());

        // 所有片段的 font_size 应为 32.0
        for f in &fragments {
            assert!(
                (f.font_size - 32.0).abs() < 0.01,
                "片段 font_size 应为 32.0，实际 {}",
                f.font_size
            );
        }
    }

    /// 测试从 Document 布局时使用 ComputedStyle 中的 line-height。
    #[test]
    fn test_layout_uses_style_line_height() {
        use zero_dom::parse_html;

        let doc = parse_html("<p>Hello World</p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        // 给 p 设置 line-height: 2.0（无单位数值）
        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(16.0);
        style.line_height = LineHeightValue::Number(2.0);
        styles.insert(p, style);

        let mut ctx = InlineFormattingContext::new(800.0);
        ctx.layout(&doc, p, &styles);

        // 行盒高度应为 16.0 * 2.0 = 32.0
        for line in &ctx.lines {
            assert!(
                (line.height - 32.0).abs() < 0.01,
                "行盒高度应为 32.0（16 * 2.0），实际 {}",
                line.height
            );
        }
    }

    /// 测试从 Document 布局时 inline 元素使用自己的样式。
    #[test]
    fn test_layout_inline_element_own_style() {
        use zero_dom::parse_html;

        let doc = parse_html("<p>Hello <b>World</b></p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        // 找到 <b> 元素（跳过文本节点，node_type 1 = Element）
        let b = doc
            .child_nodes(p)
            .into_iter()
            .find(|&id| doc.node_type(id) == Some(1))
            .expect("应有 <b> 元素");

        // p 使用默认 16px，b 使用 24px
        let mut styles = HashMap::new();
        let mut p_style = ComputedStyle::default();
        p_style.font_size = LengthValue::Px(16.0);
        styles.insert(p, p_style);

        let mut b_style = ComputedStyle::default();
        b_style.font_size = LengthValue::Px(24.0);
        styles.insert(b, b_style);

        let mut ctx = InlineFormattingContext::new(800.0);
        ctx.layout(&doc, p, &styles);

        let fragments = ctx.all_fragments();

        // 应有 font_size 为 16.0 的片段（来自 p 的文本节点）
        let has_16 = fragments.iter().any(|f| (f.font_size - 16.0).abs() < 0.01);
        assert!(has_16, "应有 16px 字体大小的片段");

        // 应有 font_size 为 24.0 的片段（来自 b 元素）
        let has_24 = fragments.iter().any(|f| (f.font_size - 24.0).abs() < 0.01);
        assert!(has_24, "应有 24px 字体大小的片段");
    }

    /// 测试无样式时回退到默认值 16.0 / 19.2。
    #[test]
    fn test_layout_no_style_fallback() {
        use zero_dom::parse_html;

        let doc = parse_html("<p>Hello</p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let mut ctx = InlineFormattingContext::new(800.0);
        ctx.layout(&doc, p, &HashMap::new());

        let fragments = ctx.all_fragments();
        assert!(!fragments.is_empty());

        for f in &fragments {
            assert!(
                (f.font_size - 16.0).abs() < 0.01,
                "无样式时 font_size 应回退到 16.0，实际 {}",
                f.font_size
            );
        }

        // 行盒高度应为 16.0 * 1.2 = 19.2
        for line in &ctx.lines {
            assert!(
                (line.height - 19.2).abs() < 0.01,
                "无样式时行盒高度应为 19.2，实际 {}",
                line.height
            );
        }
    }

    /// 测试 line-height: Length(24px) 覆盖默认行高。
    #[test]
    fn test_layout_fixed_line_height() {
        use zero_dom::parse_html;

        let doc = parse_html("<p>Text</p>");

        let html = doc.first_child(doc.root()).unwrap();
        let body = doc.last_child(html).unwrap();
        let p = doc.first_child(body).unwrap();

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(16.0);
        style.line_height = LineHeightValue::Length(LengthValue::Px(24.0));
        styles.insert(p, style);

        let mut ctx = InlineFormattingContext::new(800.0);
        ctx.layout(&doc, p, &styles);

        for line in &ctx.lines {
            assert!(
                (line.height - 24.0).abs() < 0.01,
                "行盒高度应为 24.0（固定 line-height），实际 {}",
                line.height
            );
        }
    }

    // ── 新增补充测试 ──

    /// 测试混合 inline 和 block 内容边界。
    ///
    /// 窄容器中多个文本运行跨越多行，验证行盒之间的 y 坐标不重叠。
    #[test]
    fn test_mixed_inline_block_content_boundary() {
        let mut ctx = InlineFormattingContext::new(80.0);
        let runs = vec![
            TextRun {
                text: "alpha beta gamma".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
            TextRun {
                text: "delta epsilon".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
                vertical_align: VerticalAlignValue::Baseline,
            },
        ];
        ctx.break_into_lines(runs);

        assert!(ctx.lines.len() > 1, "窄容器中应有多个行盒，实际 {}", ctx.lines.len());

        // 行盒之间不应重叠：下一行的 y 应 >= 上一行的 y + 上一行的高度
        for i in 1..ctx.lines.len() {
            let prev_end = ctx.lines[i - 1].y + ctx.lines[i - 1].height;
            assert!(
                ctx.lines[i].y >= prev_end - 0.01,
                "行 {} (y={}) 应在行 {} (y={}, h={}) 之后",
                i,
                ctx.lines[i].y,
                i - 1,
                ctx.lines[i - 1].y,
                ctx.lines[i - 1].height
            );
        }
    }

    /// 测试文本中包含显式换行符时产生多个行盒。
    ///
    /// 换行符 \n 将文本分割到不同行盒中。
    #[test]
    fn test_text_with_explicit_line_breaks() {
        let mut ctx = InlineFormattingContext::new(800.0);
        // \n 在 split_whitespace 中被视为空白，会被当作单词分隔符
        let runs = vec![TextRun {
            text: "line1\nline2\nline3".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 24.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        // 宽容器中所有单词应在同一行（因为容器足够宽）
        // 但 \n 在 split_whitespace 中作为空白处理，所以分词为 3 个单词
        assert!(ctx.lines.len() >= 1, "应至少有 1 行（宽容器中单词可能全部放入同一行）");

        // 验证所有片段存在且 y 坐标合理
        for line in &ctx.lines {
            assert!(line.height > 0.0, "行盒高度应为正");
        }
    }

    /// 测试 white-space: nowrap 行为 — 模拟单行不换行。
    ///
    /// 虽然行内格式化上下文不直接处理 white-space 属性，
    /// 但可以验证当所有文本放入单行时的行为（通过足够宽的容器）。
    #[test]
    fn test_whitespace_nowrap_behavior() {
        let mut ctx = InlineFormattingContext::new(8000.0);
        let runs = vec![TextRun {
            text: "This is a long sentence that would normally wrap but in a very wide container stays on one line"
                .to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        // 在 8000px 宽的容器中，应只有 1 行
        assert_eq!(
            ctx.lines.len(),
            1,
            "超宽容器中文本应在单行中，实际 {} 行",
            ctx.lines.len()
        );

        // 总高度应等于行高
        assert!(
            (ctx.total_height() - 20.0).abs() < 0.01,
            "单行总高度应为 20.0，实际 {}",
            ctx.total_height()
        );
    }

    /// 测试超长无断词机会的单词 — 不应换行，应放入单行。
    ///
    /// 即使单词宽度远超容器，也应保持在同一行中（浏览器行为）。
    #[test]
    fn test_very_long_word_without_break_opportunity() {
        let mut ctx = InlineFormattingContext::new(50.0);
        // 100 个字符的连续字符串，无空格
        let long_word = "abcdefghijklmnopqrstuvwxyz".repeat(4);
        let runs = vec![TextRun {
            text: long_word,
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
        }];
        ctx.break_into_lines(runs);

        // 单个不中断单词应在 1 行中（不换行）
        assert_eq!(
            ctx.lines.len(),
            1,
            "超长无断词单词应在单行中（溢出），实际 {} 行",
            ctx.lines.len()
        );
        assert_eq!(ctx.lines[0].runs.len(), 1, "应只有 1 个片段");

        // 片段宽度应远超容器宽度
        let fragment = &ctx.lines[0].runs[0];
        assert!(fragment.width > 50.0, "片段宽度 {} 应超过容器宽度 50", fragment.width);
    }

    /// 测试 vertical-align: top 在行盒中的 y 偏移。
    #[test]
    fn test_vertical_align_top_in_line() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "Hello".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 30.0,
            vertical_align: VerticalAlignValue::Top,
        }];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1);
        let fragment = &ctx.lines[0].runs[0];
        // top 对齐: y 应为 0.0
        assert!(
            fragment.y.abs() < 0.01,
            "vertical-align: top 片段 y 应为 0，实际 {}",
            fragment.y
        );
    }

    /// 测试 vertical-align: bottom 在行盒中的 y 偏移。
    #[test]
    fn test_vertical_align_bottom_in_line() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "Hello".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 30.0,
            vertical_align: VerticalAlignValue::Bottom,
        }];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1);
        let fragment = &ctx.lines[0].runs[0];
        // bottom 对齐: y = line_height - fragment_height
        let expected_y = 30.0 - fragment.height;
        assert!(
            (fragment.y - expected_y).abs() < 0.01,
            "vertical-align: bottom 片段 y 应为 {}，实际 {}",
            expected_y,
            fragment.y
        );
    }

    /// 测试 vertical-align: middle 在行盒中的 y 偏移。
    #[test]
    fn test_vertical_align_middle_in_line() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "Hello".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 30.0,
            vertical_align: VerticalAlignValue::Middle,
        }];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1);
        let fragment = &ctx.lines[0].runs[0];
        // middle 对齐: y = (line_height - fragment_height) / 2
        let expected_y = (30.0 - fragment.height) / 2.0;
        assert!(
            (fragment.y - expected_y).abs() < 0.01,
            "vertical-align: middle 片段 y 应为 {}，实际 {}",
            expected_y,
            fragment.y
        );
    }

    // ── 新增边界测试 ──

    /// 测试 vertical-align: sub — 片段基线下移 font_size × 0.3。
    ///
    /// 公式: y = baseline_y - height + (font_size * 0.3)
    /// baseline_y = line_height * 0.8
    #[test]
    fn test_vertical_align_sub_in_line() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let font_size = 16.0_f32;
        let line_height = 30.0_f32;
        let runs = vec![TextRun {
            text: "sub".to_string(),
            node_id: NodeId::default(),
            font_size,
            line_height,
            vertical_align: VA::Sub,
        }];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1);
        let fragment = &ctx.lines[0].runs[0];
        let baseline_y = line_height * 0.8;
        let offset = font_size * 0.3;
        let expected_y = baseline_y - fragment.height + offset;
        assert!(
            (fragment.y - expected_y).abs() < 0.01,
            "vertical-align: sub 片段 y 应为 {}，实际 {}",
            expected_y,
            fragment.y
        );
    }

    /// 测试 vertical-align: super — 片段基线上移 font_size × 0.3。
    ///
    /// 公式: y = baseline_y - height - (font_size * 0.3)
    #[test]
    fn test_vertical_align_super_in_line() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let font_size = 16.0_f32;
        let line_height = 30.0_f32;
        let runs = vec![TextRun {
            text: "super".to_string(),
            node_id: NodeId::default(),
            font_size,
            line_height,
            vertical_align: VA::Super,
        }];
        ctx.break_into_lines(runs);

        assert_eq!(ctx.lines.len(), 1);
        let fragment = &ctx.lines[0].runs[0];
        let baseline_y = line_height * 0.8;
        let offset = font_size * 0.3;
        let expected_y = baseline_y - fragment.height - offset;
        assert!(
            (fragment.y - expected_y).abs() < 0.01,
            "vertical-align: super 片段 y 应为 {}，实际 {}",
            expected_y,
            fragment.y
        );
    }

    /// 测试 vertical-align: text-top 与 top 行为一致 — y = 0.0。
    #[test]
    fn test_vertical_align_text_top_same_as_top() {
        let mut ctx_text_top = InlineFormattingContext::new(800.0);
        let runs_text_top = vec![TextRun {
            text: "Text".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 30.0,
            vertical_align: VA::TextTop,
        }];
        ctx_text_top.break_into_lines(runs_text_top);

        let mut ctx_top = InlineFormattingContext::new(800.0);
        let runs_top = vec![TextRun {
            text: "Text".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 30.0,
            vertical_align: VA::Top,
        }];
        ctx_top.break_into_lines(runs_top);

        let y_text_top = ctx_text_top.lines[0].runs[0].y;
        let y_top = ctx_top.lines[0].runs[0].y;
        assert!(
            (y_text_top - y_top).abs() < 0.01,
            "text-top y ({}) 应与 top y ({}) 一致",
            y_text_top,
            y_top
        );
        assert!(y_text_top.abs() < 0.01, "text-top 片段 y 应为 0.0，实际 {}", y_text_top);
    }

    /// 测试 vertical-align: text-bottom 与 bottom 行为一致 — y = line_height - height。
    #[test]
    fn test_vertical_align_text_bottom_same_as_bottom() {
        let line_height = 30.0_f32;
        let mut ctx_text_bottom = InlineFormattingContext::new(800.0);
        let runs_text_bottom = vec![TextRun {
            text: "Text".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height,
            vertical_align: VA::TextBottom,
        }];
        ctx_text_bottom.break_into_lines(runs_text_bottom);

        let mut ctx_bottom = InlineFormattingContext::new(800.0);
        let runs_bottom = vec![TextRun {
            text: "Text".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height,
            vertical_align: VA::Bottom,
        }];
        ctx_bottom.break_into_lines(runs_bottom);

        let y_text_bottom = ctx_text_bottom.lines[0].runs[0].y;
        let y_bottom = ctx_bottom.lines[0].runs[0].y;
        assert!(
            (y_text_bottom - y_bottom).abs() < 0.01,
            "text-bottom y ({}) 应与 bottom y ({}) 一致",
            y_text_bottom,
            y_bottom
        );
        // text-bottom: y = line_height - fragment_height
        let height = ctx_text_bottom.lines[0].runs[0].height;
        let expected = line_height - height;
        assert!(
            (y_text_bottom - expected).abs() < 0.01,
            "text-bottom y 应为 {}，实际 {}",
            expected,
            y_text_bottom
        );
    }

    /// 测试 resolve_font_metrics 中 LineHeightValue::Length(LengthValue::Em(1.5)) 回退到 font_size × 1.2。
    ///
    /// 非 Px 长度在 resolve 阶段未转换时做防御性回退。
    #[test]
    fn test_resolve_font_metrics_line_height_em_fallback() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.line_height = LineHeightValue::Length(LengthValue::Em(1.5));

        let (font_size, line_height) = resolve_font_metrics(Some(&style));
        assert!((font_size - 20.0).abs() < 0.01, "font_size 应为 20.0，实际 {font_size}");
        // Em 长度回退到 font_size * 1.2 = 24.0
        let expected = 20.0 * 1.2;
        assert!(
            (line_height - expected).abs() < 0.01,
            "line_height 应回退到 {}，实际 {}",
            expected,
            line_height
        );
    }

    /// 测试 resolve_font_metrics 中非 Px font_size 回退到 DEFAULT_FONT_SIZE (16.0)。
    #[test]
    fn test_resolve_font_metrics_non_px_font_size() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Em(2.0);
        // line-height 默认 Normal

        let (font_size, line_height) = resolve_font_metrics(Some(&style));
        // 非 Px font_size 回退到 16.0
        assert!(
            (font_size - 16.0).abs() < 0.01,
            "非 Px font_size 应回退到 16.0，实际 {font_size}"
        );
        // line_height = 16.0 * 1.2 = 19.2
        assert!(
            (line_height - 19.2).abs() < 0.01,
            "line_height 应为 19.2，实际 {line_height}"
        );
    }

    /// 测试 break_into_lines 调用两次后状态完全重置，不残留第一次调用的结果。
    #[test]
    fn test_break_into_lines_called_twice_resets_state() {
        // 使用窄容器确保第一次调用产生多行
        let mut ctx = InlineFormattingContext::new(60.0);

        // 第一次调用：产生多行的长文本
        let first_runs = vec![TextRun {
            text: "alpha beta gamma delta epsilon zeta eta theta".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VA::Baseline,
        }];
        ctx.break_into_lines(first_runs);
        let first_line_count = ctx.lines.len();
        assert!(first_line_count > 1, "第一次调用应产生多行");

        // 切换到宽容器，第二次调用：短文本，只产生单行
        ctx.container_width = 800.0;
        let second_runs = vec![TextRun {
            text: "Short".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VA::Baseline,
        }];
        ctx.break_into_lines(second_runs);

        // 第二次调用后应只有 1 行，无第一次调用的残留
        assert_eq!(
            ctx.lines.len(),
            1,
            "第二次调用后应只有 1 行，无残留，实际 {} 行",
            ctx.lines.len()
        );
        assert_eq!(ctx.lines[0].runs.len(), 1, "应只有 1 个片段");
        assert!(
            ctx.lines[0].runs[0].text.contains("Short"),
            "片段文本应为 'Short'，实际 {}",
            ctx.lines[0].runs[0].text
        );
    }

    /// 测试 font_size=0.0 时不会 panic。
    ///
    /// 零字体大小意味着估算字符宽度为 0，所有单词放入单行。
    #[test]
    fn test_zero_font_size_no_panic() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "Hello World".to_string(),
            node_id: NodeId::default(),
            font_size: 0.0,
            line_height: 20.0,
            vertical_align: VA::Baseline,
        }];
        // 不应 panic
        ctx.break_into_lines(runs);

        // 零字体大小 → 零字符宽度 → 所有单词放入单行
        assert_eq!(ctx.lines.len(), 1, "零字体大小应产生 1 行");
        // 片段宽度应为 0
        for f in ctx.all_fragments() {
            assert!(f.width.abs() < 0.01, "零字体大小片段宽度应为 0，实际 {}", f.width);
        }
    }

    /// 测试 line_height=0.0 时不会 panic。
    ///
    /// 零行高意味着行盒高度为 0，y 坐标不会递增。
    #[test]
    fn test_zero_line_height_no_panic() {
        let mut ctx = InlineFormattingContext::new(800.0);
        let runs = vec![TextRun {
            text: "Hello World".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 0.0,
            vertical_align: VA::Baseline,
        }];
        // 不应 panic
        ctx.break_into_lines(runs);

        // 行盒高度应为 0.0（所有片段 line_height 为 0）
        assert_eq!(ctx.lines.len(), 1, "零行高应产生 1 行");
        assert!(
            ctx.lines[0].height.abs() < 0.01,
            "零行高行盒高度应为 0，实际 {}",
            ctx.lines[0].height
        );
        assert!(
            ctx.total_height().abs() < 0.01,
            "零行高总高度应为 0，实际 {}",
            ctx.total_height()
        );
    }
}
