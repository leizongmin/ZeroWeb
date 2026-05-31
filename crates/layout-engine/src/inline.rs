//! 行内格式化上下文实现。
//!
//! 处理行内级内容的布局：文本节点、inline 元素、行换行。
//! Taffy 仅支持 Block/Flex/Grid，行内布局需要自行实现。

use zero_dom::{Document, NodeId, NodeKind};

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
}

/// 行内格式化上下文 — 负责将行内内容排列成行盒。
#[derive(Debug, Clone)]
pub struct InlineFormattingContext {
    /// 包含块的可用宽度。
    pub container_width: f32,
    /// 生成的行盒列表。
    pub lines: Vec<LineBox>,
}

impl InlineFormattingContext {
    /// 创建新的行内格式化上下文。
    pub fn new(container_width: f32) -> Self {
        Self {
            container_width,
            lines: Vec::new(),
        }
    }

    /// 对文档中指定节点的行内子内容执行布局。
    ///
    /// 收集文本节点和 inline 元素，将它们排列成行盒。
    pub fn layout(&mut self, doc: &Document, container: NodeId) {
        let runs = self.collect_inline_runs(doc, container);
        self.break_into_lines(runs);
    }

    /// 收集容器中所有行内级内容（文本节点 + inline 元素）。
    fn collect_inline_runs(&self, doc: &Document, container: NodeId) -> Vec<TextRun> {
        let mut runs = Vec::new();
        let children = doc.child_nodes(container);

        for &child_id in &children {
            if let Some(node) = doc.get(child_id) {
                match &node.kind {
                    NodeKind::Text(text_data) => {
                        let text = text_data.content.trim().to_string();
                        if !text.is_empty() {
                            runs.push(TextRun {
                                text,
                                node_id: child_id,
                                font_size: 16.0,   // 默认字体大小
                                line_height: 20.0, // 默认行高
                            });
                        }
                    }
                    NodeKind::Element(_) => {
                        // inline 元素的文本内容也收集进来
                        let text = doc.text_content(child_id).unwrap_or_default();
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            runs.push(TextRun {
                                text: trimmed,
                                node_id: child_id,
                                font_size: 16.0,
                                line_height: 20.0,
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
            },
            TextRun {
                text: "World".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
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
        ctx.layout(&doc, p);

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
        ctx.layout(&doc, p);

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
        ctx.layout(&doc, p);

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
        }];
        ctx24.break_into_lines(runs_24);

        // 行高 32px 的单行
        let mut ctx32 = InlineFormattingContext::new(800.0);
        let runs_32 = vec![TextRun {
            text: "Short text".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 32.0,
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
            },
            TextRun {
                text: "Large".to_string(),
                node_id: NodeId::default(),
                font_size: 24.0,
                line_height: 30.0,
            },
            TextRun {
                text: "Medium".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
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
            },
            TextRun {
                text: "Word".to_string(),
                node_id: NodeId::default(),
                font_size: 20.0,
                line_height: 24.0,
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
            },
            TextRun {
                text: "gamma delta".to_string(),
                node_id: NodeId::default(),
                font_size: 16.0,
                line_height: 20.0,
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
            },
            TextRun {
                text: "Second".to_string(),
                node_id: id2,
                font_size: 16.0,
                line_height: 20.0,
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
        }];
        ctx.break_into_lines(runs);

        // 容器宽度为 0 时，第一个单词放入第一行（即使溢出），
        // 后续每个单词都换新行
        assert!(!ctx.lines.is_empty(), "即使容器宽度为 0，也应产生行盒");
        assert!(ctx.lines.len() >= 2, "零宽度容器中多个单词应产生多行");
    }
}
