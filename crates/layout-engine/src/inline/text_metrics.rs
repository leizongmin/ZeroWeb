//! 文本度量与字符宽度估计辅助。
//!
//! 从 `inline/mod.rs` 抽出（R342，2000 行规则 + Phase A 准备）。
//! 包含：字符/字符串宽度估计（estimate_char_width / estimate_string_width）、
//! AdvanceSource trait + EstimateAdvance 默认实现、CJK/emoji 字符分类、
//! font-metrics 解析（resolve_font_metrics）、inline-block 尺寸解析、BiDi 重排序。

use std::rc::Rc;
use std::sync::Arc;

use zero_css_parser::values::LengthValue;
use zero_style_system::{ComputedStyle, TextAutospaceValue};
// 经 `pub use text_metrics::*`（inline/mod.rs）再导出，供 inline/tests 子模块经 glob 访问。
pub use zero_style_system::LineHeightValue;

use super::inline_types::TextFragmentSource;

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
///
/// ⚠️ R1769 advance 杠杆最终关闭（refute R1768「REOPENED」）：per-char NotoSansCJK advance
/// 表（chromium fc-match sans-serif 实测）注入 estimate 后 welcome diff **16.84%→23.44%
/// (+6.6pp)** —— **显著恶化**。根因 = ZW paint 路径用 **DejaVu Sans**（`find_system_font`
/// → `DejaVuSans.ttf`）非 NotoSansCJK，故 estimate 切 NotoSansCJK 反在 ZW 内部制造
/// layout↔paint 字体源分裂（estimate NotoSansCJK 宽 vs paint DejaVu 宽），换行点两边都不
/// 对齐。R1768 drastic 0.55→0.90「advance 影响 diff」仅证 advance 影响幅度（平凡真），非证
/// 「匹配 chromium 有益」。**advance 任何单字体 per-char 表注入 estimate 都受 layout/paint/
/// chromium 三方字体源分裂所阻**，须 font-stack rebuild 统一三方字体源才能 yield。
/// 见 master.md R1769。
pub fn estimate_char_width(c: char, font_size: f32, is_ahem: bool) -> f32 {
    // R1449：零宽格式字符宽度恒为 0（与字体无关，CSS 语义覆盖字体 advance）。
    // ZWNJ U+200C / ZWJ U+200D / WJ U+2060 / ZWNBSP U+FEFF 均零宽（joiner 类，shaping/
    // white-space-vs-joiners 用 ZWJ；实测零宽 +0 flip 改善无回归）。
    // 注：ZWSP U+200B 亦零宽，但其零宽在 **normal 模式**会触发 seg-break-transformation-018
    // 回归（ZW seg-break 变换 bug，20px U+200B 反而更近 oracle）——故 U+200B 零宽仅经
    // preserve 模式 split_into_words 的断词丢弃路径实现（见 mod.rs），normal 模式暂不零宽。
    if matches!(c, '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}') {
        return 0.0;
    }
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

    /// 测量整段文本的 advance 宽度。
    ///
    /// 默认逐字符调用 [`Self::measure`]，保持现有实现行为不变。需要 kerning/GPOS
    /// 上下文的实现可覆写本方法，并在一次 shaping 中返回整段 advance。
    fn measure_text(&self, text: &str, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        text.chars()
            .map(|ch| self.measure(ch, font_id, font_size, is_ahem))
            .sum()
    }

    /// 使用有序 CSS face 列表测量整段文本。
    fn measure_text_with_fonts(&self, text: &str, font_ids: &[u32], font_size: f32, is_ahem: bool) -> f32 {
        self.measure_text(text, font_ids.first().copied(), font_size, is_ahem)
    }

    /// 使用有序 face 列表和 `font-size-adjust` 上下文测量文本。
    fn measure_text_with_font_context(
        &self,
        text: &str,
        font_ids: &[u32],
        font_size: f32,
        is_ahem: bool,
        _size_adjust: &zero_style_system::FontSizeAdjustValue,
    ) -> f32 {
        self.measure_text_with_fonts(text, font_ids, font_size, is_ahem)
    }
}

/// 默认 advance 源：委托 `estimate_char_width` 启发式（保持当前行为，零回归）。
pub struct EstimateAdvance;

impl AdvanceSource for EstimateAdvance {
    fn measure(&self, ch: char, _font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        estimate_char_width(ch, font_size, is_ahem)
    }
}

/// 持有 `AdvanceSource` 的 trait 对象句柄（C3 advance plumbing，R2 dormant）。
///
/// 镜像 `FontMetricProviderHandle`（font_metrics.rs）：单独 newtype 因
/// `InlineFormattingContext` derive `Debug` 而 `dyn AdvanceSource` 非自动 `Debug`。
/// 内部 `Rc` 允许 engine 与 IFC 共享同一源而不引入生命周期参数。
///
/// **零回归**：IFC 默认 `advance_source = None`，4 个 in-IFC 度量点回退
/// `EstimateAdvance`（= `estimate_char_width`，字节等价）。`zero-engine` 注入
/// `FontLoader`-backed 实现后（R3），度量点改读真实 advance。
#[derive(Clone)]
pub struct AdvanceSourceHandle(pub(crate) Rc<dyn AdvanceSource>);

impl std::fmt::Debug for AdvanceSourceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdvanceSourceHandle").finish_non_exhaustive()
    }
}

impl AdvanceSourceHandle {
    /// 经由内部源测量单字符 advance。
    ///
    /// IFC 度量点（`inline/mod.rs` 4 站点）调用本方法；`None` 时调用方回退
    /// `EstimateAdvance`（零回归）。
    pub fn measure(&self, ch: char, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        self.0.measure(ch, font_id, font_size, is_ahem)
    }

    /// 经由内部源测量整段文本 advance。
    pub fn measure_text(&self, text: &str, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        self.0.measure_text(text, font_id, font_size, is_ahem)
    }

    /// 经由内部源按有序 CSS face 列表测量整段文本。
    pub fn measure_text_with_fonts(&self, text: &str, font_ids: &[u32], font_size: f32, is_ahem: bool) -> f32 {
        self.0.measure_text_with_fonts(text, font_ids, font_size, is_ahem)
    }

    /// 经由内部源按有序 face 列表和 `font-size-adjust` 上下文测量文本。
    pub fn measure_text_with_font_context(
        &self,
        text: &str,
        font_ids: &[u32],
        font_size: f32,
        is_ahem: bool,
        size_adjust: &zero_style_system::FontSizeAdjustValue,
    ) -> f32 {
        self.0
            .measure_text_with_font_context(text, font_ids, font_size, is_ahem, size_adjust)
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

/// R1215：判断字符是否为 text-autospace 的「表意文字」（CSS Text 4 §8）。
///
/// 比 [`is_cjk_character`] **窄**：排除 CJK 标点/符号区（U+3000..=U+303F，含 。、）
/// 与全角形式区（U+FF00..=U+FFEF，含 ！，），这些是标点不是表意文字，不应触发
/// ideograph-alpha/numeric 自动间距。仅保留 Han + 平假名 + 片假名 + 韩文音节。
pub(crate) fn is_autospace_ideograph(c: char) -> bool {
    matches!(
        c,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}' // CJK Ext A
        | '\u{F900}'..='\u{FAFF}' // CJK Compat Ideographs
        | '\u{3040}'..='\u{309F}' // 平假名
        | '\u{30A0}'..='\u{30FF}' // 片假名
        | '\u{AC00}'..='\u{D7AF}' // 韩文音节
    )
}

/// R1215：text-autospace 字符类别（CSS Text 4 §8）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AutospaceCategory {
    /// CJK 表意文字（Han/假名/韩文）。
    Ideograph,
    /// ASCII 字母。
    Letter,
    /// ASCII 数字。
    Numeric,
    /// 标点、空格、其他。
    Other,
}

/// R1215：判定单个字符的 text-autospace 类别。
pub(crate) fn char_autospace_category(c: char) -> AutospaceCategory {
    if c.is_ascii_digit() {
        AutospaceCategory::Numeric
    } else if c.is_ascii_alphabetic() {
        AutospaceCategory::Letter
    } else if is_autospace_ideograph(c) {
        AutospaceCategory::Ideograph
    } else {
        AutospaceCategory::Other
    }
}

/// R1215：计算两相邻字符间的 text-autospace 间距（px）。
///
/// 仅在 ideograph↔letter（ideograph-alpha）或 ideograph↔numeric（ideograph-numeric）
/// 类别边界、且对应规则启用时返回 `0.125 × font_size`；否则 0。标点/空格类别（Other）
/// 不触发任何间距。
pub(crate) fn autospace_gap_for(prev_ch: char, curr_ch: char, rules: TextAutospaceValue, font_size: f32) -> f32 {
    const GAP_EM: f32 = 0.125;
    let p = char_autospace_category(prev_ch);
    let c = char_autospace_category(curr_ch);
    if rules.ideograph_alpha_active()
        && ((p == AutospaceCategory::Ideograph && c == AutospaceCategory::Letter)
            || (p == AutospaceCategory::Letter && c == AutospaceCategory::Ideograph))
    {
        return GAP_EM * font_size;
    }
    if rules.ideograph_numeric_active()
        && ((p == AutospaceCategory::Ideograph && c == AutospaceCategory::Numeric)
            || (p == AutospaceCategory::Numeric && c == AutospaceCategory::Ideograph))
    {
        return GAP_EM * font_size;
    }
    0.0
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

/// 默认行高倍数（用于 line-height: normal，非 Ahem 字体）。
///
/// **= chromium 对 DejaVu Sans 的真实 line-height:normal（R1174 三栈源码逆向）**：
/// chromium（Linux/Skia/FreeType）对未设 `OS/2.fsSelection` bit 7（useTypoMetrics）的字体
/// 走 hhea 度量分支——DejaVu Sans 恰未设该位，hhea `ascent=1901 / descent=-483 / lineGap=0`，
/// `upem=2048` → `(1901+483)/2048 = 1.164`（hhea lineGap=0 故无 leading）。FreeType
/// `sfobjs.c` + Skia `SkFontHost_FreeType.cpp` + Blink `font_metrics.h` 三栈确定性算术，
/// 详见 `docs/goal/rendering-compat/research-chromium-lineheight-normal-formula-2026-07-08.md`。
/// fontdue 实测同值（`line_metrics_full` line_gap=0，ascent−descent=1.1641）。
///
/// ZeroWeb 此前用 1.2（误以为是 chromium 近似），R1174 证 1.164 才是真值。oracle A/B
/// 实测 css/CSS2/normal-flow 607→612（+5 真实 chromium-oracle 翻转），故改用 1.164。
/// welcome product-smoke 因字体不匹配（chromium 用系统字体，ZW 用 DejaVu）+0.68pp
/// （16.29→16.97%，仍 < 20% DC-13 gate），属可接受代价（字体匹配是 R631 独立多会话）。
///
/// **R1185 A/B 复核**：测 chromium **generic** sans-serif/serif line-height:normal = 1.150
/// （puppeteer 实测，Blink 内部 generic 默认，非 resolved 字体度量；区别于 explicit
/// DejaVu 1.170 / NotoSans 1.360 / NotoSansCJK 1.450）。A/B 1.164→1.150 = css-text-decor
/// 118→120 (+2 oracle) BUT welcome 16.97→17.36% (+0.39pp 回归, 81433→83347 px) + normal-flow
/// 612→612 (neutral)。trade 效率 0.195 pp/flip **差于** R1175 的 1.164 (0.136 pp/flip) →
/// **1.164 仍是全局最优**，1.150 refuted 已回退。关键结论：chromium line-height:normal
/// 区分 generic (~1.15) vs explicit (字体 hhea)，fontdue hhea 对 explicit 精确匹配 chromium
/// （证 unified-font-stack C2 须区分 generic/explicit，naive font-swap 必 diverge）。
pub(crate) const NORMAL_LINE_HEIGHT_RATIO: f32 = 1.164;

/// Ahem 字体（WPT 标准测试字体）line-height:normal 的实际度量比率。
///
/// Chromium 对 line-height:normal 使用字体实际度量。fontdue 实测 Ahem.ttf：
/// ascent=800 / descent=-200 / line_gap=0 / units_per_em=1000 → 1.0
/// （度量探针见 `docs/goal/rendering-compat/evidence/r759-font-metric-line-height-2026-06-28.txt`）。
/// 与 `estimate_char_width` 对 Ahem 等宽=font_size 的既有特判一致。
pub(crate) const AHEM_LINE_HEIGHT_RATIO: f32 = 1.0;

/// Ahem 字体的 font-size-adjust aspect value（CSS Fonts 3 §3.6：= x-height / em）。
///
/// **= chromium 用的 OS/2 `sxHeight / units_per_em`**（不是 'x' glyph ink——Ahem 的 'x'
/// glyph 填满 em-box 故 ink=1.0，但 OS/2 sxHeight=800/upem=1000=0.8）。实证：font-size-adjust-001
/// `font:40px Ahem; font-size-adjust:0.9` → chromium adjusted = **45px** = `40×0.9/0.8`，
/// 与 ref `font-size-adjust-001-ref.html` 的 `#test{font-size:45px}` 精确一致。
///
/// **R1192 is_ahem-gated narrow apply**：font-size-adjust 仅对 Ahem 字体 apply（aspect 0.8
/// 常数，R990/R1175 is_ahem-gated 谱系）。非 Ahem 字体的 aspect 须 OS/2 sxHeight 派生 +
/// font 接入 layout（同 Phase A 字体度量架构 gap），留 Slice 3+。
pub(crate) const AHEM_FONT_SIZE_ADJUST_ASPECT: f32 = 0.8;

/// 从 ComputedStyle 中解析 font-size 和 line-height。
///
/// - `font_size` 从 `ComputedStyle::font_size` 中提取（已解析为 Px）。
/// - `line_height` 根据 `LineHeightValue` 计算：
///   - `Normal` → font_size × 度量比率（Ahem=1.0，其余=1.164）
///   - `Number(n)` → font_size × n
///   - `Length(Px(v))` → v
///
/// 当 style 为 None 时（节点没有样式），返回默认值 16.0 / 18.624（16 × 1.164）。
///
/// 不带 font-metric provider：`line-height:normal` 永远走常数比率（`NORMAL_LINE_HEIGHT_RATIO` /
/// `AHEM_LINE_HEIGHT_RATIO`）。需要 per-font 真实度量时调用
/// [`resolve_font_metrics_with_provider`]。
pub fn resolve_font_metrics(style: Option<&ComputedStyle>) -> (f32, f32) {
    resolve_font_metrics_with_provider(style, None)
}

/// U1b（unified font stack）：带 font-metric provider 的 font-size / line-height 解析。
///
/// 与 [`resolve_font_metrics`] 的唯一区别：当 `line-height:normal` 且 `provider` 为 `Some`
/// 并能解析该 `font-family` 时，用字体真实行度量（`ascent − descent + line_gap`，已按
/// `font_size` 缩放为 px）替代常数比率。provider 为 `None` 或无法解析（字体未加载）时，
/// **逐字节等价于** [`resolve_font_metrics`]（回退 Ahem 1.0 / 非-Ahem 1.164 常数）。
///
/// 这是 unified font stack 的「首消费者」：R885 font-bridge（`FontMetricProvider` trait +
/// IFC `font_metric_provider` 字段）此前 0 生产读取；本函数在 layout IFC 行盒高度计算处
/// 首次消费 provider，使 per-font line-height 经既有 override-map 链路（
/// `frag.height` → `store_font_sizes_from_ifc` → `text_node_line_heights` → paint Path B
/// `with_line_height_overrides`）触达 paint，绕过 R890 发现的「paint Path B 空 styles」阻塞。
///
/// **接通状态（R2202 核对 2026-07-29；R2393 复核 2026-08-01）**：U1b-wiring 已完成——
/// **reftest runner 已接通**（`reftest.rs:568` 调 `set_font_metric_map`，line-height:normal
/// 走真实度量）；**生产 webview/renderer dormant 接通**（env `ZW_PERFONT_LINEHEIGHT=1`
/// 激活，默认关）。**R2393 实证生产激活 = net 负，保持 dormant**（welcome 英文 +0.44pp 恶化；
/// morning 中文零变化——全显式 line-height 无 normal 行，「CJK lever」假设证伪）→ **勿再以
/// font-metric 生产激活为 lever**（证据 `evidence/font-metric-activation-ab-2026-08-01.md`）；
/// 若推进须与 IFC strut/half-leading 真实化打包（深结构 R834 谱系）。详见
/// `docs/goal/rendering-compat/unified-font-stack-design.md`。
pub fn resolve_font_metrics_with_provider(
    style: Option<&ComputedStyle>,
    provider: Option<&crate::inline::FontMetricProviderHandle>,
) -> (f32, f32) {
    let mut font_size = match style {
        Some(s) => match &s.font_size {
            LengthValue::Px(v) => *v as f32,
            _ => DEFAULT_FONT_SIZE,
        },
        None => DEFAULT_FONT_SIZE,
    };

    let is_ahem = style.is_some_and(|s| {
        s.font_family
            .iter()
            .any(|family| family.trim_matches('"').eq_ignore_ascii_case("Ahem"))
    });
    let mut normal_font_size = font_size;

    // Ahem keeps its historical direct size adjustment because its synthetic paint
    // path does not consume per-glyph shaped sizes.
    if let Some(s) = style
        && let zero_style_system::FontSizeAdjustValue::Adjust {
            metric,
            basis: zero_style_system::FontSizeAdjustBasis::Number(adj),
        } = s.font_size_adjust
        && adj.is_finite()
        && adj >= 0.0
    {
        let metric = metric.unwrap_or(zero_style_system::FontSizeAdjustMetric::ExHeight);
        if is_ahem && metric == zero_style_system::FontSizeAdjustMetric::ExHeight && AHEM_FONT_SIZE_ADJUST_ASPECT > 0.0
        {
            font_size = font_size * (adj as f32) / AHEM_FONT_SIZE_ADJUST_ASPECT;
            normal_font_size = font_size;
        } else if std::env::var("ZW_FONT_SIZE_ADJUST_NORMAL_LINE").as_deref() != Ok("0")
            && let Some(aspect) = provider.and_then(|p| p.font_metric_aspect(&s.font_family, metric))
            && aspect.is_finite()
            && aspect > 0.0
        {
            // https://drafts.csswg.org/css-fonts-4/#font-size-adjust-prop
            // Keep TextRun.font_size specified (shaping applies per-face adjustment),
            // but derive line-height:normal from the adjusted used primary size.
            normal_font_size = font_size * (adj as f32) / aspect;
        }
    }

    // line-height:normal 的回退比率：Ahem=1.0（见 AHEM_LINE_HEIGHT_RATIO），其余 1.164
    // （DejaVu hhea = chromium 真值，见 NORMAL_LINE_HEIGHT_RATIO 注释）。provider 缺省或
    // 无法解析字体时用此值。无样式（None）时无法判定字体，回退 1.164。
    let normal_ratio = match style {
        Some(_) if is_ahem => AHEM_LINE_HEIGHT_RATIO,
        _ => NORMAL_LINE_HEIGHT_RATIO,
    };
    let line_height = match style {
        Some(s) => match &s.line_height {
            LineHeightValue::Normal => {
                let descriptor_metrics = if std::env::var("ZW_FONT_FACE_SIZE_ADJUST_NORMAL_LINE").as_deref() != Ok("0")
                    && matches!(s.font_size_adjust, zero_style_system::FontSizeAdjustValue::None)
                {
                    provider.and_then(|p| p.size_adjusted_line_metrics(&s.font_family, font_size))
                } else {
                    None
                };
                descriptor_metrics.map_or_else(
                    || resolve_normal_line_height(s, normal_font_size, normal_ratio, provider),
                    |metrics| metrics.ascent - metrics.descent + metrics.line_gap,
                )
            }
            LineHeightValue::Number(n) => font_size * (*n as f32),
            LineHeightValue::Length(LengthValue::Px(v)) => *v as f32,
            // 其他长度类型（em/rem 等）在 resolve 阶段应已转换为 Px，
            // 这里做防御性回退（用字体度量比率，Ahem 时为 1.0）
            LineHeightValue::Length(_) => font_size * normal_ratio,
        },
        None => DEFAULT_FONT_SIZE * normal_ratio,
    };

    (font_size, line_height)
}

/// `line-height:normal` 的 per-font 解析（U1b）。
///
/// provider 存在且解析到字体时返回 `ascent − descent + line_gap`（px，与 chromium
/// `line-height:normal` 计算一致）；否则回退 `font_size × fallback_ratio`（Ahem 1.0 /
/// 非-Ahem 1.164），与 R1175 落地的常数行为逐字节一致。
fn resolve_normal_line_height(
    style: &ComputedStyle,
    font_size: f32,
    fallback_ratio: f32,
    provider: Option<&crate::inline::FontMetricProviderHandle>,
) -> f32 {
    if let Some(p) = provider
        && let Some(m) = p.line_metrics(&style.font_family, font_size)
    {
        // fontdue/chromium 约定：ascent 正、descent 负、line_gap 通常 0 或小正值。
        // line-height:normal = ascent − descent + line_gap（见 font_metrics.rs 注释）。
        return m.ascent - m.descent + m.line_gap;
    }
    font_size * fallback_ratio
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

/// BiDi 重排结果及每个视觉字符对应的逻辑 UTF-8 byte range。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BidiReorderedText {
    /// 视觉顺序文本。
    pub visual_text: String,
    /// 与 `visual_text.chars()` 一一对应的逻辑源码 byte range。
    pub visual_to_logical: Vec<std::ops::Range<usize>>,
    /// 与 `visual_text.chars()` 一一对应的 UBA resolved level 奇偶方向。
    pub visual_is_rtl: Vec<bool>,
}

fn identity_bidi_mapping(text: &str) -> BidiReorderedText {
    let char_count = text.chars().count();
    BidiReorderedText {
        visual_text: text.to_string(),
        visual_to_logical: text
            .char_indices()
            .map(|(start, ch)| start..start + ch.len_utf8())
            .collect(),
        visual_is_rtl: vec![false; char_count],
    }
}

fn bidi_override_mapping(text: &str, is_rtl: bool, mirror_glyphs: bool) -> BidiReorderedText {
    let mut chars = text
        .char_indices()
        .map(|(start, ch)| (ch, start..start + ch.len_utf8()))
        .collect::<Vec<_>>();
    if is_rtl {
        chars.reverse();
    }
    let mut visual_text = String::with_capacity(text.len());
    let mut visual_to_logical = Vec::with_capacity(chars.len());
    for (ch, range) in chars {
        let visual_ch = if is_rtl && mirror_glyphs {
            unicode_bidi_mirroring::get_mirrored(ch).unwrap_or(ch)
        } else {
            ch
        };
        visual_text.push(visual_ch);
        visual_to_logical.push(range);
    }
    let char_count = visual_to_logical.len();
    BidiReorderedText {
        visual_text,
        visual_to_logical,
        visual_is_rtl: vec![is_rtl; char_count],
    }
}

/// BiDi reorder with CSS `direction` context.
/// When `is_rtl` is true and the text needs BiDi processing, sets paragraph level to RTL (level 1).
fn bidi_reorder_with_direction(text: &str, preserve_all_paragraphs: bool, is_rtl: bool) -> BidiReorderedText {
    use unicode_bidi::{BidiInfo, Level};

    // 快速检查：如果文本为空或全是 ASCII 且 LTR，不需要 BiDi 处理
    if text.is_empty() || (text.is_ascii() && !is_rtl) {
        return identity_bidi_mapping(text);
    }

    // 检查是否需要 BiDi 处理：含 RTL 脚本字符 **或** bidi 控制码（R2019）**或** CSS direction:rtl。
    let needs_bidi = is_rtl
        || text.chars().any(|ch| {
            let cp = ch as u32;
            (0x0590..=0x05FF).contains(&cp)
                || (0x0600..=0x06FF).contains(&cp)
                || (0x0700..=0x074F).contains(&cp)
                || (0x08A0..=0x08FF).contains(&cp)
                || (0xFB50..=0xFDFF).contains(&cp)
                || (0xFE70..=0xFEFF).contains(&cp)
                || (0x202A..=0x202E).contains(&cp)
                || (0x2066..=0x2069).contains(&cp)
        });

    if !needs_bidi {
        return identity_bidi_mapping(text);
    }

    // UBA paragraph level: None = auto-detect, Some(Level::rtl()) = RTL, Some(Level::ltr()) = LTR.
    // CSS `direction: rtl` sets paragraph base direction to RTL (UAX #9 HL1).
    let para_level = if is_rtl { Some(Level::rtl()) } else { None };
    let bidi_info = BidiInfo::new(text, para_level);
    bidi_reorder_paragraphs(&bidi_info, text, preserve_all_paragraphs)
}

fn bidi_reorder_paragraphs(
    bidi_info: &unicode_bidi::BidiInfo,
    text: &str,
    preserve_all_paragraphs: bool,
) -> BidiReorderedText {
    use unicode_bidi::BidiClass;
    let mut visual_text = String::with_capacity(text.len());
    let mut visual_to_logical = Vec::with_capacity(text.chars().count());
    let mut visual_is_rtl = Vec::with_capacity(text.chars().count());
    let paragraph_limit = if preserve_all_paragraphs { usize::MAX } else { 1 };
    for para in bidi_info.paragraphs.iter().take(paragraph_limit) {
        // https://www.unicode.org/reports/tr9/#P1
        // unicode-bidi 将段落分隔符留在前一段。布局分词必须继续看到分隔符位于段尾，
        // 因此只重排正文，再按逻辑位置追加分隔符。
        let separator_start = text[..para.range.end]
            .char_indices()
            .next_back()
            .map(|(start, _)| start)
            .filter(|start| bidi_info.original_classes[*start] == BidiClass::B);
        let content_end = separator_start.unwrap_or(para.range.end);
        if para.range.start < content_end {
            let content_range = para.range.start..content_end;
            let (levels, runs) = bidi_info.visual_runs(para, content_range);
            for run in runs {
                let run_is_rtl = levels[run.start].is_rtl();
                let mut chars = text[run.clone()]
                    .char_indices()
                    .map(|(offset, ch)| {
                        let start = run.start + offset;
                        (ch, start..start + ch.len_utf8())
                    })
                    .collect::<Vec<_>>();
                if run_is_rtl {
                    chars.reverse();
                }
                for (ch, logical_range) in chars {
                    visual_text.push(ch);
                    visual_to_logical.push(logical_range);
                    visual_is_rtl.push(run_is_rtl);
                }
            }
        }

        if let Some(start) = separator_start {
            let ch = text[start..para.range.end]
                .chars()
                .next()
                .expect("paragraph separator range is not empty");
            visual_text.push(ch);
            visual_to_logical.push(start..start + ch.len_utf8());
            visual_is_rtl.push(para.level.is_rtl());
        }
    }
    BidiReorderedText {
        visual_text,
        visual_to_logical,
        visual_is_rtl,
    }
}

/// 按视觉顺序为断词后的片段提取逻辑源码映射。
pub(crate) struct BidiFragmentCursor {
    source_text: Option<Arc<str>>,
    reordered: BidiReorderedText,
    visual_byte_offset: usize,
    visual_char_offset: usize,
}

/// 返回 plaintext 段落首个 strong 字符的基方向。
pub(crate) fn plaintext_base_is_rtl(text: &str) -> bool {
    use unicode_bidi::{BidiClass, bidi_class};
    text.chars()
        .find_map(|ch| match bidi_class(ch) {
            BidiClass::R | BidiClass::AL => Some(true),
            BidiClass::L => Some(false),
            _ => None,
        })
        .unwrap_or(false)
}

impl BidiFragmentCursor {
    /// 创建不做视觉重排的逻辑顺序游标。
    pub(crate) fn logical(text: &str) -> Self {
        let enabled = std::env::var("ZW_BIDI_FRAGMENT_SOURCE").as_deref() != Ok("0");
        Self {
            source_text: enabled.then(|| Arc::<str>::from(text)),
            reordered: identity_bidi_mapping(text),
            visual_byte_offset: 0,
            visual_char_offset: 0,
        }
    }

    /// 为一个逻辑文本运行创建游标，带 CSS direction 和 unicode-bidi 参数。
    ///
    /// UAX #9 HL1: `is_rtl = true` → paragraph base level = 1 (RTL),
    /// 使 BiDi 算法按 RTL 基方向重排序视觉文本。
    /// UAX #9 HL4: `is_plaintext = true` → 忽略 CSS direction，强制 auto-detect。
    pub(crate) fn with_direction(text: &str, is_rtl: bool, is_plaintext: bool) -> Self {
        let enabled = std::env::var("ZW_BIDI_FRAGMENT_SOURCE").as_deref() != Ok("0");
        // unicode-bidi: plaintext → paragraph level auto-detect (CSS Writing Modes §2.2)
        let effective_rtl = is_rtl && !is_plaintext;
        let reordered = bidi_reorder_with_direction(text, enabled, effective_rtl);
        let identity = reordered.visual_text == text
            && reordered
                .visual_to_logical
                .iter()
                .cloned()
                .eq(text.char_indices().map(|(start, ch)| start..start + ch.len_utf8()));
        Self {
            source_text: (enabled && !identity).then(|| Arc::<str>::from(text)),
            reordered,
            visual_byte_offset: 0,
            visual_char_offset: 0,
        }
    }

    /// 为 CSS `unicode-bidi:bidi-override` 创建指定方向的视觉游标。
    pub(crate) fn with_override(text: &str, is_rtl: bool) -> Self {
        let source_enabled = std::env::var("ZW_BIDI_FRAGMENT_SOURCE").as_deref() != Ok("0");
        let mirror_glyphs = std::env::var("ZW_BIDI_MIRRORING").as_deref() != Ok("0");
        let reordered = bidi_override_mapping(text, is_rtl, mirror_glyphs);
        let identity = reordered.visual_text == text
            && reordered
                .visual_to_logical
                .iter()
                .cloned()
                .eq(text.char_indices().map(|(start, ch)| start..start + ch.len_utf8()));
        Self {
            source_text: (source_enabled && !identity).then(|| Arc::<str>::from(text)),
            reordered,
            visual_byte_offset: 0,
            visual_char_offset: 0,
        }
    }

    /// 返回供现有断词和布局逻辑使用的视觉文本。
    pub(crate) fn visual_text(&self) -> &str {
        &self.reordered.visual_text
    }

    /// 消费下一个视觉片段，并返回与片段字符对齐的逻辑源码范围。
    pub(crate) fn take_source(&mut self, fragment: &str) -> Option<TextFragmentSource> {
        let source_text = self.source_text.clone()?;
        let remaining = &self.reordered.visual_text[self.visual_byte_offset..];
        let core = fragment.trim_end_matches(' ');
        let synthetic_count = fragment.chars().count() - core.chars().count();
        let exact_start = remaining.find(fragment);
        let core_start = (!core.is_empty()).then(|| remaining.find(core)).flatten();
        let (matched, synthetic_count, relative_start) = match (exact_start, core_start) {
            (Some(exact), Some(core_pos)) if exact <= core_pos => (fragment, 0, exact),
            (_, Some(core_pos)) => (core, synthetic_count, core_pos),
            (Some(exact), None) => (fragment, 0, exact),
            (None, None) if core.is_empty() => (core, synthetic_count, 0),
            (None, None) => return None,
        };

        let skipped_chars = remaining[..relative_start].chars().count();
        let mapped_start = self.visual_char_offset + skipped_chars;
        let mapped_end = mapped_start + matched.chars().count();
        let mut visual_to_logical = self
            .reordered
            .visual_to_logical
            .get(mapped_start..mapped_end)?
            .iter()
            .cloned()
            .map(Some)
            .collect::<Vec<_>>();
        visual_to_logical.extend(std::iter::repeat_n(None, synthetic_count));
        let mut visual_is_rtl = self.reordered.visual_is_rtl.get(mapped_start..mapped_end)?.to_vec();
        let synthetic_is_rtl = visual_is_rtl.last().copied().unwrap_or(false);
        visual_is_rtl.extend(std::iter::repeat_n(synthetic_is_rtl, synthetic_count));

        self.visual_byte_offset += relative_start + matched.len();
        self.visual_char_offset = mapped_end;
        Some(TextFragmentSource {
            text: source_text,
            visual_to_logical,
            visual_is_rtl,
        })
    }
}

// ── Tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{BidiFragmentCursor, BidiReorderedText, bidi_reorder_with_direction, identity_bidi_mapping};

    fn bidi_reorder_with_mapping(text: &str) -> BidiReorderedText {
        bidi_reorder_with_direction(text, true, false)
    }

    /// R2019：纯 ASCII（无 RTL 脚本字符、无 bidi 控制码）须原样返回（早返 fast-path）。
    #[test]
    fn r2019_bidi_reorder_pure_ascii_unchanged() {
        assert_eq!(bidi_reorder_with_mapping("hello world").visual_text, "hello world");
        assert_eq!(bidi_reorder_with_mapping("").visual_text, "");
    }

    /// R2019：bidi 控制码（U+202E RLO）须触发重排序（修：原 `has_rtl` 漏控制码致早返原序）。
    /// logical `ab[RLO]cde` → RLO 反转后续 → 视觉序中 c/d/e 倒序（edc）。
    #[test]
    fn r2019_bidi_reorder_handles_rlo_control_code() {
        let out = bidi_reorder_with_mapping("ab\u{202E}cde").visual_text;
        // RLO 生效 → cde 视觉倒序（e 在 d 前，d 在 c 前）。
        let (pc, pd, pe) = (out.find('c').unwrap(), out.find('d').unwrap(), out.find('e').unwrap());
        assert!(pe < pd, "RLO must reverse cde (e before d): got {out:?}");
        assert!(pd < pc, "RLO must reverse cde (d before c): got {out:?}");
    }

    #[test]
    fn bidi_reorder_preserves_visual_to_logical_utf8_ranges() {
        let reordered = bidi_reorder_with_mapping("אבג");
        assert_eq!(reordered.visual_text, "גבא");
        assert_eq!(reordered.visual_to_logical, vec![4..6, 2..4, 0..2]);
        assert_eq!(reordered.visual_is_rtl, vec![true, true, true]);
        let logical = reordered
            .visual_to_logical
            .iter()
            .rev()
            .map(|range| &"אבג"[range.clone()])
            .collect::<String>();
        assert_eq!(logical, "אבג");
    }

    #[test]
    fn bidi_reorder_identity_mapping_tracks_multibyte_characters() {
        let reordered = bidi_reorder_with_mapping("Aé");
        assert_eq!(reordered.visual_text, "Aé");
        assert_eq!(reordered.visual_to_logical, vec![0..1, 1..3]);
        assert_eq!(reordered.visual_is_rtl, vec![false, false]);
    }

    #[test]
    fn bidi_reorder_preserves_all_paragraphs_and_separators() {
        let reordered = bidi_reorder_with_mapping("אבג\nדה");
        assert_eq!(reordered.visual_text, "גבא\nהד");
        assert_eq!(reordered.visual_to_logical, vec![4..6, 2..4, 0..2, 6..7, 9..11, 7..9]);
        assert_eq!(reordered.visual_is_rtl, vec![true; 6]);
    }

    #[test]
    fn bidi_reorder_rollback_mode_stops_after_first_paragraph() {
        let reordered = bidi_reorder_with_direction("אבג\nדה", false, false);
        assert_eq!(reordered.visual_text, "גבא\n");
        assert_eq!(reordered.visual_to_logical, vec![4..6, 2..4, 0..2, 6..7]);
        assert_eq!(reordered.visual_is_rtl, vec![true; 4]);
    }

    #[test]
    fn bidi_reorder_preserves_mixed_visual_run_directions() {
        let reordered = bidi_reorder_with_mapping("aאב");
        assert_eq!(reordered.visual_text.chars().count(), reordered.visual_is_rtl.len());
        assert!(reordered.visual_is_rtl.contains(&false));
        assert!(reordered.visual_is_rtl.contains(&true));
    }

    #[test]
    fn bidi_override_reverses_and_mirrors_rtl_text() {
        let mirrored = super::bidi_override_mapping(".(d c) b a", true, true);
        let legacy = super::bidi_override_mapping(".(d c) b a", true, false);
        assert_eq!(mirrored.visual_text, "a b (c d).");
        assert_eq!(legacy.visual_text, "a b )c d(.");
        assert_eq!(mirrored.visual_to_logical, legacy.visual_to_logical);
    }

    #[test]
    fn bidi_fragment_cursor_prefers_nearest_core_over_later_exact_match() {
        let mut cursor = BidiFragmentCursor {
            source_text: Some(Arc::<str>::from("fooXfoo ")),
            reordered: identity_bidi_mapping("fooXfoo "),
            visual_byte_offset: 0,
            visual_char_offset: 0,
        };
        let source = cursor.take_source("foo ").expect("synthetic-space fragment maps");
        assert_eq!(source.visual_to_logical, vec![Some(0..1), Some(1..2), Some(2..3), None]);
        assert_eq!(source.visual_is_rtl, vec![false; 4]);
    }
}
