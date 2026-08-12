//! 行内布局核心数据类型（2000 行规则 + Phase A IFC 统一 Phase 5 准备）。
//!
//! R830 从 `inline/mod.rs` 抽出：TextAlign / TextRun(+impl) / InlineBlockBox /
//! InlineItem / LineBox / TextFragment / collapse_whitespace / WordBreakMode /
//! FloatExclusion。与 `text_metrics.rs`、`inline_finalization.rs` 同属 inline
//! 子模块拆分。通过 `mod.rs` 的 `pub use inline_types::*` 再导出，保持
//! `crate::inline::TextRun` 等 API 路径不变（纯移动，零行为变化）。

use std::ops::Range;
use std::sync::Arc;

use zero_css_parser::values::VerticalAlignValue;
use zero_dom::NodeId;

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
    /// 解析后的字体 id（CSS font-family → FontLoader id）。
    ///
    /// **C3 advance plumbing（R2 dormant）**：供 `AdvanceSource::measure` 查询真实
    /// 字符 advance（替 `estimate_char_width` 启发式）。`None` = 未知（构造处尚未接线），
    /// `EstimateAdvance` 忽略本字段回退启发式 = 零回归；`FontLoader`-backed 实现启用后
    /// 按本字段查 hmtx 真实 advance。见 `advance-width-plumbing-design.md` R2-R3。
    pub font_id: Option<u32>,
    /// CSS `direction` 属性：true = rtl, false = ltr（默认）。
    /// 用于 BiDi 段落基方向（UBA paragraph level）。
    pub is_rtl: bool,
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
            font_id: None,
            is_rtl: false,
        }
    }

    /// inline 非替换元素对 line box 高度的贡献 = line-height。
    ///
    /// CSS 2.1 §10.8.1 + §8.4：inline 非替换元素的垂直 padding/border 只绘制
    /// （向 line box 上下方延伸），**不影响 line box 高度**。WPT blocks-019 明确
    /// "Top padding on inline elements has no effect on layout"，chromium 同此。
    /// 旧实现把 padding/border 加进 box_height 致 inline padding-top/border
    /// 错误撑高 line box（minimal 复现：`<span padding-top:100px>` 让 div 120px 而非 20px）。
    pub fn box_height(&self) -> f32 {
        self.line_height
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
    /// 外边距上侧（px）。inline-block margin box 参与行内格式化：margin_top 把盒
    /// 内容下移（apply_vertical_alignment 据此偏移盒 Y）；此前完全忽略致 margin 失效
    /// （flexbox_flex REF 的 span margin:1em 不偏移 → 与 flex test 不一致）。
    pub margin_top: f32,
    /// 外边距右侧（px）——推进水平位置。
    pub margin_right: f32,
    /// 外边距下侧（px）——计入行盒高度。
    pub margin_bottom: f32,
    /// 外边距左侧（px）——推进水平位置。
    pub margin_left: f32,
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
    /// 行盒基线相对行顶的 y（= max_ascent，CSS §10.8.1）。R816 linebox 度量统一 Phase 1：
    /// 由 `apply_vertical_alignment` 算出并存储，供后续 Phase 由 paint 复用（取代 is_ahem?0:font_size
    /// 启发式）。Phase 1 仅存储，paint 尚未读取（行为不变）。
    pub baseline_y: f32,
    /// 行盒 ascent（baseline 到行顶，含 half-leading 上半）。Phase 1 存储。
    pub ascent: f32,
    /// 行盒 descent（baseline 到行底，含 half-leading 下半 = height - ascent）。Phase 1 存储。
    pub descent: f32,
}

/// BiDi 重排后的文本片段源码映射。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFragmentSource {
    /// BiDi 重排前的完整逻辑文本运行。
    pub text: Arc<str>,
    /// 与片段视觉字符一一对应的逻辑 UTF-8 byte range。
    ///
    /// `None` 表示断词阶段合成、源码中没有直接对应字符的内容。
    pub visual_to_logical: Vec<Option<Range<usize>>>,
    /// 与视觉字符一一对应的 UBA resolved level 奇偶方向。
    pub visual_is_rtl: Vec<bool>,
}

impl TextFragmentSource {
    /// 返回片段全部视觉字符一致的 UBA resolved direction。
    pub fn uniform_resolved_rtl(&self) -> Option<bool> {
        if self.visual_is_rtl.len() != self.visual_to_logical.len() {
            return None;
        }
        let first = *self.visual_is_rtl.first()?;
        self.visual_is_rtl.iter().all(|rtl| *rtl == first).then_some(first)
    }

    /// 返回可由单个逻辑文本切片表示的源码范围。
    ///
    /// 仅接受全部 range 按视觉顺序严格相邻且方向一致的映射。混合 BiDi、
    /// 合成字符或非法 UTF-8 range 返回 `None`，避免交给单方向 shaping 路径。
    pub fn logical_range(&self) -> Option<Range<usize>> {
        let mut ranges = self.visual_to_logical.iter();
        let first = ranges.next()?.as_ref()?;
        if !valid_source_range(&self.text, first) {
            return None;
        }

        let mut start = first.start;
        let mut end = first.end;
        let mut previous = first;
        let mut ascending = None;
        for range in ranges {
            let range = range.as_ref()?;
            if !valid_source_range(&self.text, range) {
                return None;
            }
            let next_ascending = if previous.end == range.start {
                true
            } else if range.end == previous.start {
                false
            } else {
                return None;
            };
            if ascending.is_some_and(|direction| direction != next_ascending) {
                return None;
            }
            ascending = Some(next_ascending);
            start = start.min(range.start);
            end = end.max(range.end);
            previous = range;
        }
        Some(start..end)
    }

    /// 返回可由单个 shaping run 消费的逻辑文本切片。
    pub fn logical_slice(&self) -> Option<&str> {
        self.text.get(self.logical_range()?)
    }
}

fn valid_source_range(text: &str, range: &Range<usize>) -> bool {
    range.start < range.end
        && range.end <= text.len()
        && text.is_char_boundary(range.start)
        && text.is_char_boundary(range.end)
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
    /// BiDi 重排后的视觉字符到逻辑源码映射。
    ///
    /// 仅非 identity 映射携带该字段；当前绘制路径尚不消费它。
    pub source: Option<TextFragmentSource>,
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
    /// inline 元素的水平 margin（px）。文本节点为 0。
    pub margin_left: f32,
    /// inline 元素的水平 margin（px）。文本节点为 0。
    pub margin_right: f32,
    /// inline-block 的上 margin（px）—— apply_vertical_alignment 据此偏移盒的 Y
    /// （CSS：inline-block margin box 参与行盒，margin_top 把盒内容下移）。
    /// 文本/inline 元素为 0。
    pub margin_top: f32,
    /// 基线高度（px）— 从片段顶部到基线的距离。
    ///
    /// - 文本运行：baseline = font_size（ascent 近似）
    /// - inline-block：baseline = height（基线在底部边缘）
    /// - inline-flex/inline-grid：baseline 从第一个 item 合成
    pub baseline: f32,
}

/// CSS Text §4.1 白空格折叠：将连续空白字符折叠为单个空格。
///
/// 与 `trim()` 不同，此函数保留首尾的空格（作为单个空格）。
/// 仅含空白的输入返回单个空格（用于 inline-block 之间的间隔）。
/// 空输入返回空字符串。
///
/// 行首/行尾空格的剥离由 IFC 的 `break_items_into_lines` 在行级别处理。
pub(crate) fn collapse_whitespace(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut result = String::with_capacity(text.len());
    let mut last_was_space = false;
    for ch in text.chars() {
        if is_collapsible_ws(ch) {
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

/// CSS Text: U+00A0 (NO-BREAK SPACE) 是 **preserved** 且 **non-breaking**——不可折叠、
/// 不可作断行点（名如其意「no-break」）。其他 Unicode White_Space（含 U+3000 IDEOGRAPHIC
/// SPACE 等）按既有行为折叠/断行（WPT 实证：5-char 窄集合回归 css-text break-spaces/control-chars/
/// shaping-arabic 等 7 案）。
///
/// 本函数 = Rust `is_whitespace` 排除 U+00A0，用于 collapse + 断行判定。
/// 旧实现 `is_whitespace()` / `split_whitespace()` 含 U+00A0 → `&nbsp;` 被折叠为普通空格再被
/// 行首尾 trim → 仅含 `&nbsp;` 的元素塌缩为 0 行盒（`line-height-applies-to` 簇 4.75%）。
pub(crate) fn is_collapsible_ws(ch: char) -> bool {
    ch.is_whitespace() && ch != '\u{00A0}'
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

#[cfg(test)]
mod tests {
    use super::*;

    /// R1085：U+00A0 (NO-BREAK SPACE) 是 preserved + non-breaking，不可折叠。
    /// 旧实现 `char::is_whitespace()` 含 U+00A0 → 仅含 `&nbsp;` 的元素塌缩为 0 行盒
    ///（line-height-applies-to 簇）。本测试守此 CSS 语义。
    #[test]
    fn nbsp_is_not_collapsible_ws() {
        assert!(!is_collapsible_ws('\u{00A0}'), "U+00A0 must not be collapsible");
        // 标准 CSS 白空格仍可折叠。
        for ch in [' ', '\t', '\n', '\x0C', '\r'] {
            assert!(is_collapsible_ws(ch), "{:?} should be collapsible", ch);
        }
        // 其他 Unicode 空白（U+3000 等）按既有行为仍可折叠（WPT 实证：窄集合回归 css-text）。
        assert!(is_collapsible_ws('\u{3000}'), "U+3000 keeps broad-whitespace behavior");
    }

    #[test]
    fn collapse_whitespace_preserves_nbsp() {
        // 单独 nbsp 不应折叠为普通空格再被行首尾 trim 掉。
        assert_eq!(collapse_whitespace("\u{00A0}"), "\u{00A0}");
        // nbsp 与普通空格混合：普通空格折叠，nbsp 保留。
        assert_eq!(collapse_whitespace("a  \u{00A0}  b"), "a \u{00A0} b");
        // 连续 nbsp 全保留（非折叠）。
        assert_eq!(collapse_whitespace("\u{00A0}\u{00A0}"), "\u{00A0}\u{00A0}");
    }

    #[test]
    fn fragment_source_recovers_ltr_logical_slice() {
        let source = TextFragmentSource {
            text: Arc::<str>::from("Aé"),
            visual_to_logical: vec![Some(0..1), Some(1..3)],
            visual_is_rtl: vec![false, false],
        };
        assert_eq!(source.uniform_resolved_rtl(), Some(false));
        assert_eq!(source.logical_range(), Some(0..3));
        assert_eq!(source.logical_slice(), Some("Aé"));
    }

    #[test]
    fn fragment_source_recovers_rtl_logical_slice() {
        let source = TextFragmentSource {
            text: Arc::<str>::from("אבג"),
            visual_to_logical: vec![Some(4..6), Some(2..4), Some(0..2)],
            visual_is_rtl: vec![true, true, true],
        };
        assert_eq!(source.uniform_resolved_rtl(), Some(true));
        assert_eq!(source.logical_range(), Some(0..6));
        assert_eq!(source.logical_slice(), Some("אבג"));
    }

    #[test]
    fn fragment_source_rejects_discontiguous_or_mixed_order() {
        let source = TextFragmentSource {
            text: Arc::<str>::from("abcd"),
            visual_to_logical: vec![Some(0..1), Some(3..4), Some(2..3)],
            visual_is_rtl: vec![false, true, true],
        };
        assert_eq!(source.uniform_resolved_rtl(), None);
        assert_eq!(source.logical_slice(), None);
    }

    #[test]
    fn fragment_source_rejects_synthetic_or_invalid_ranges() {
        let synthetic = TextFragmentSource {
            text: Arc::<str>::from("abc"),
            visual_to_logical: vec![Some(0..1), None],
            visual_is_rtl: vec![false, false],
        };
        assert_eq!(synthetic.logical_slice(), None);

        let mismatched_direction = TextFragmentSource {
            text: Arc::<str>::from("ab"),
            visual_to_logical: vec![Some(0..1), Some(1..2)],
            visual_is_rtl: vec![false],
        };
        assert_eq!(mismatched_direction.uniform_resolved_rtl(), None);

        let invalid_utf8 = TextFragmentSource {
            text: Arc::<str>::from("אב"),
            visual_to_logical: vec![Some(1..2)],
            visual_is_rtl: vec![true],
        };
        assert_eq!(invalid_utf8.logical_slice(), None);
    }
}
