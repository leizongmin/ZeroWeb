//! 文本度量与字符宽度估计辅助。
//!
//! 从 `inline/mod.rs` 抽出（R342，2000 行规则 + Phase A 准备）。
//! 包含：字符/字符串宽度估计（estimate_char_width / estimate_string_width）、
//! AdvanceSource trait + EstimateAdvance 默认实现、CJK/emoji 字符分类、
//! font-metrics 解析（resolve_font_metrics）、inline-block 尺寸解析、BiDi 重排序。

use zero_css_parser::values::LengthValue;
use zero_style_system::ComputedStyle;
// 经 `pub use text_metrics::*`（inline/mod.rs）再导出，供 inline/tests 子模块经 glob 访问。
pub use zero_style_system::LineHeightValue;

/// 默认字体大小（px）。
pub(crate) const DEFAULT_FONT_SIZE: f32 = 16.0;

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
///
/// ⚠️ R224 实验回退：曾用 DejaVu Sans 实测 advance 表（W=0.99/i=0.28 等）替换此启发式，
/// 但全量 reftest 实测 **439→436 净 -3 回归**（非 Ahem 用例换行点翻转）。教训：estimate
/// 并非纯自源中性——test/ref 文本结构不同时换行点敏感度不同，单独扰动 estimate 会破
/// 同源对齐。真实修复须完整接入 FontLoader（R223 plumbing R2-R5，layout+paint+intrinsic
/// 三处同源替换 + font_id 解析），而非单点改 estimate_char_width。证据见 master.md R224。
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

/// 字符 advance 宽度源（依赖反转，解耦 layout-engine 与字体光栅化层）。
///
/// 默认实现 [`EstimateAdvance`] 回退到 `estimate_char_width` 启发式（零行为变更）；
/// `zero-engine` 可注入 FontLoader-backed 实现提供真实 advance（见
/// `docs/goal/rendering-compat/advance-width-plumbing-design.md`），以降低 R222 实测的
/// 逐字符 ±44-98% 估计误差（DC-2~5 chromium 一致率系统性噪声根因）。
pub trait AdvanceSource {
    /// 测量单字符 advance 宽度。
    ///
    /// - `font_id`：CSS font-family 解析结果（R3 起由 TextRun 携带）；None = 未知，
    ///   实现应回退到 estimate。
    /// - `is_ahem`：Ahem 测试字体（advance = font_size）。
    fn measure(&self, ch: char, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32;
}

/// 默认 advance 源：委托 `estimate_char_width` 启发式（保持当前行为，零回归）。
pub struct EstimateAdvance;

impl AdvanceSource for EstimateAdvance {
    fn measure(&self, ch: char, _font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        estimate_char_width(ch, font_size, is_ahem)
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
pub(crate) fn is_cjk_character(c: char) -> bool {
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

/// R645：判断字符是否属于「需要字典分词」的东南亚复杂文字（Thai/Lao/Myanmar/Khmer）。
///
/// 这些文字不在单词间使用空格，词边界须字典查找（如 libthai/ICU）。CSS Text 3
/// §line-break-details 要求：UA 若无此类字典分词能力，**必须**做某种 fallback 断行
/// （不允许溢出）。ZeroWeb 无 SEA 词典，按字符断行作为 fallback。
///
/// 仅用于 [`split_into_words`](super::InlineFormattingContext::split_into_words) 的
/// 断行点判定（与 CJK 同样按字符单独成词）；**不**影响 advance 宽度估计——SEA 字符
/// 非全角，[`estimate_char_width`] 仍走「其他 Unicode」0.5× 分支。
pub(crate) fn is_sea_word_script(c: char) -> bool {
    matches!(
        c,
        '\u{0E00}'..='\u{0E7F}'   // Thai（泰文）
        | '\u{0E80}'..='\u{0EFF}' // Lao（老挝文）
        | '\u{1000}'..='\u{109F}' // Myanmar（缅甸文）
        | '\u{1780}'..='\u{17FF}' // Khmer（高棉文）
    )
}

/// R645：CJK 或 SEA 词典分词文字 → 在 `split_into_words` 中按字符断行（fallback）。
/// 集中此判定避免 3 处调用点重复 `is_cjk_character(ch) || is_sea_word_script(ch)`。
pub(crate) fn is_per_char_break_script(c: char) -> bool {
    is_cjk_character(c) || is_sea_word_script(c)
}

/// 判断字符是否为 emoji 或常见符号（非 CJK）。
pub(crate) fn is_emoji_character(c: char) -> bool {
    let cp = c as u32;
    (0x1F300..=0x1FAFF).contains(&cp)
        || (0x2600..=0x26FF).contains(&cp)
        || (0x2700..=0x27BF).contains(&cp)
        || (0xFE00..=0xFE0F).contains(&cp)
        || (0x1F1E6..=0x1F1FF).contains(&cp)
}

/// 估算字符串的总宽度，按每个字符逐一计算。
pub(crate) fn estimate_string_width(text: &str, font_size: f32, is_ahem: bool) -> f32 {
    text.chars().map(|c| estimate_char_width(c, font_size, is_ahem)).sum()
}

/// 默认行高倍数（用于 line-height: normal）。
pub(crate) const NORMAL_LINE_HEIGHT_RATIO: f32 = 1.2;

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

/// 对文本进行 BiDi 重排序，返回视觉顺序的字符串。
///
/// 使用 unicode-bidi 库分析文本的嵌入层级，对 RTL 段落进行重排序。
/// 如果文本不需要重排序（纯 LTR），返回原始文本。
pub(crate) fn bidi_reorder(text: &str) -> String {
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
