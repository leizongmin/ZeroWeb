//! 列表标记（list marker）渲染 + 计数器格式化 helper。
//!
//! R1694 从 painter/text.rs 抽离（text.rs 减负，单文件超 2000 行 guideline）。
//! 计数器格式化（Roman / Latin 字母序号）+ `<li>` 的 paint_list_marker Painter 方法 +
//! compute_list_item_index 兄弟索引。paint_content（CSS content 计数器）通过
//! `use super::text_list::{format_counter_alpha, format_counter_roman}` 复用格式化函数。

use zero_css_parser::values::{ContentListItem, LengthValue, ListStyleTypeValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::LayoutBox;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{GlyphPrimitive, ImagePrimitive, RoundedRectPrimitive};
use zero_style_system::{ComputedStyle, ContentComputedValue, DirectionValue, LineHeightValue, WritingModeValue};

use crate::paint::color::color_value_to_render;
use crate::paint::helpers::image_resource_key;

const MAX_COUNTER_PAD_WIDTH: usize = 1024;

/// 将正整数转为大写罗马数字（lowercase 由调用方 `to_lowercase()`）。
fn to_roman(mut num: usize) -> String {
    if num == 0 {
        return "0".to_string();
    }
    let pairs = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for (value, symbol) in &pairs {
        while num >= *value {
            result.push_str(symbol);
            num -= value;
        }
    }
    result
}

/// R2445：lower-greek 计数器表示（CSS Counter Styles 3 §6 预定义）。
///
/// 现代希腊小写字母 α-ω（24 个，U+03B1-U+03C9，σ 用 ς？——CSS spec 用 σ）。值 1→α、24→ω；
/// 超出 24 循环重复（αα=25，spec 对 >24 实现定义，取循环近似）。
fn to_greek(value: usize) -> String {
    const GREEK: &[char] = &[
        'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'σ', 'τ', 'υ', 'φ', 'χ',
        'ψ', 'ω',
    ];
    if value == 0 {
        return "0".to_string();
    }
    let mut result = String::new();
    let mut v = value;
    while v > 0 {
        // 1-based：余 0 → ω（最后一个），否则对应字母。
        let idx = (v - 1) % GREEK.len();
        result.insert(0, GREEK[idx]);
        v = if v <= GREEK.len() { 0 } else { (v - 1) / GREEK.len() };
    }
    result
}

/// R2445：persian 计数器表示（CSS Counter Styles 3 §6 预定义）。
///
/// 波斯-印度数字 ۰-۹（U+06F0-U+06F9）。把十进制各位数字替换为对应波斯数字。
fn to_persian(value: usize) -> String {
    const PERSIAN: &[char] = &['۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹'];
    value
        .to_string()
        .chars()
        .map(|c| PERSIAN[(c as u8 - b'0') as usize])
        .collect()
}

/// R2451：arabic-indic 计数器表示（CSS Counter Styles 3 §6.1 预定义，numeric system）。
///
/// 阿拉伯-印度数字 ٠-٩（U+0660-U+0669，core Arabic block；区别 persian 的 extended U+06F0+）。
/// numeric system = 各位数字替换（同 persian 算法）。ground-truth 验证：arabic-indic-101（1=١..9=٩）。
fn to_arabic_indic(value: usize) -> String {
    value
        .to_string()
        .chars()
        .map(|c| char::from_u32(0x0660 + (c as u8 - b'0') as u32).expect("arabic-indic digit block"))
        .collect()
}

/// R2471：通用 numeric system 计数器表示（CSS Counter Styles 3 §6.1）。
///
/// 十进制位数字替换：把 value 的各位 ASCII 数字替换为 `base` 起的连续 10 字形块中对应数字
///（同 arabic-indic 算法，仅 digit 字形块不同）。`base` = 该脚本 DIGIT ZERO 的 Unicode 码点。
fn to_digit_script(value: usize, base: u32) -> String {
    value
        .to_string()
        .chars()
        .map(|c| char::from_u32(base + (c as u8 - b'0') as u32).expect("digit script block contiguous 0-9"))
        .collect()
}

/// R2472：cjk-decimal 计数器表示（CSS Counter Styles 3 §6.1 预定义 numeric system）。
///
/// CJK ideographic digits——非连续码点（0=〇 U+3007，1-9=U+4E00/U+4E8C/U+4E09/U+56DB/
/// U+4E94/U+516D/U+4E03/U+516B/U+4E5D），须用 lookup table（同 to_persian 模式）。
/// ground-truth：cjk-decimal-004（10=一〇, 101=一〇一, 1002=一〇〇二）。
fn to_cjk_decimal(value: usize) -> String {
    const CJK: &[char] = &['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    value
        .to_string()
        .chars()
        .map(|c| CJK[(c as u8 - b'0') as usize])
        .collect()
}

/// R3835：CSS Counter Styles 3 §6.2 limited CJK/日/韩计数器表示。
///
/// 9 个 numeric 家族共用同一千进制合成算法，差异仅在符号表与三条家族规则：
/// - `omit_one`：数字 1 在位符前省略（`'all'` = 十/百/千全省——japanese-informal、
///   korean-hanja-informal；`'tens'` = 仅十位省——simp/trad-chinese-informal；其余家族
///   恒写 digit+unit——formal 系与 korean-hangul-formal）。
/// - `zero_mid`：中间补零（中文系 101=一百**零**一；日韩系无补零 101=百一）。
/// - 负值前缀各家族不同（マイナス/负/負/마이너스）。
///
/// range 语义（§6.2）：日/中 6 家族 10000+ 继续按位合成（WPT 044 期望 10000=一〇〇〇〇）；
/// korean 3 家族 range 1-9999，越界由调用方走 decimal fallback。ground-truth：全部 9 家族
/// 的 WPT css3-counter-styles-\* title/text 对（含 0/9999/负值/越界）算法级验证通过。
#[derive(Clone, Copy)]
struct CjkNumSymbols {
    /// 数字符号 0-9。
    digits: &'static [char],
    /// 位符 [千, 百, 十]。
    units: [&'static str; 3],
    /// 1 省略规则。
    omit_one: OmitOne,
    /// 中间补零（中文系）。
    zero_mid: bool,
    /// 零填充字符（中文系「零」；日系无补零不用）。
    zero_char: char,
    /// 零值表示（japanese-informal = 〇，其余 = 零/영）。
    zero_value: &'static str,
    /// 负值前缀。
    negative: &'static str,
}

/// 数字 1 在位符前的省略规则。
#[derive(Clone, Copy)]
enum OmitOne {
    /// 恒写 digit+unit（formal 系）。
    Never,
    /// 十/百/千前全省（japanese-informal、korean-hanja-informal）。
    AllUnits,
    /// 仅十位前省（simp/trad-chinese-informal：10=十 而 100=一百）。
    TensOnly,
}

const SIMP_CHINESE_DIGITS: &[char] = &['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
const SIMP_CHINESE_FORMAL_DIGITS: &[char] = &['零', '壹', '贰', '叁', '肆', '伍', '陆', '柒', '捌', '玖'];
const TRAD_CHINESE_FORMAL_DIGITS: &[char] = &['零', '壹', '貳', '參', '肆', '伍', '陸', '柒', '捌', '玖'];
const JAPANESE_FORMAL_DIGITS: &[char] = &['零', '壱', '弐', '参', '四', '伍', '六', '七', '八', '九'];
const KOREAN_HANGUL_DIGITS: &[char] = &['영', '일', '이', '삼', '사', '오', '육', '칠', '팔', '구'];

/// R3835：builtin 计数器样式的 marker suffix（CSS Counter Styles §2 suffix 描述符——
/// predefined 样式 suffix 默认 "." + 空格；§6.2 CJK/日 = "、"（U+3001）、韩 = ","）。
/// 返回不含尾随空格的标点部分；空格间隔由调用方按需补（inside advance）。
fn counter_suffix(t: &ListStyleTypeValue) -> &'static str {
    match t {
        ListStyleTypeValue::JapaneseInformal
        | ListStyleTypeValue::JapaneseFormal
        | ListStyleTypeValue::SimpChineseInformal
        | ListStyleTypeValue::SimpChineseFormal
        | ListStyleTypeValue::TradChineseInformal
        | ListStyleTypeValue::TradChineseFormal
        // §6.1/§6.2 假名与天干地支（WPT ref：ア、/ 甲、）。
        | ListStyleTypeValue::CjkEarthlyBranch
        | ListStyleTypeValue::CjkHeavenlyStem
        | ListStyleTypeValue::Hiragana
        | ListStyleTypeValue::HiraganaIroha
        | ListStyleTypeValue::Katakana
        | ListStyleTypeValue::KatakanaIroha => "、",
        ListStyleTypeValue::KoreanHangulFormal
        | ListStyleTypeValue::KoreanHanjaInformal
        | ListStyleTypeValue::KoreanHanjaFormal => ",",
        _ => ".",
    }
}

/// §6.2 九个 numeric 家族符号/规则表（ground-truth 见 [`to_cjk_num`] 文档）。
const JAPANESE_INFORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: SIMP_CHINESE_DIGITS,
    units: ["千", "百", "十"],
    omit_one: OmitOne::AllUnits,
    zero_mid: false,
    zero_char: '零',
    zero_value: "〇",
    negative: "マイナス",
};
const JAPANESE_FORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: JAPANESE_FORMAL_DIGITS,
    units: ["阡", "百", "拾"],
    omit_one: OmitOne::Never,
    zero_mid: false,
    zero_char: '零',
    zero_value: "零",
    negative: "マイナス",
};
const SIMP_CHINESE_INFORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: SIMP_CHINESE_DIGITS,
    units: ["千", "百", "十"],
    omit_one: OmitOne::TensOnly,
    zero_mid: true,
    zero_char: '零',
    zero_value: "零",
    negative: "负",
};
const SIMP_CHINESE_FORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: SIMP_CHINESE_FORMAL_DIGITS,
    units: ["仟", "佰", "拾"],
    omit_one: OmitOne::Never,
    zero_mid: true,
    zero_char: '零',
    zero_value: "零",
    negative: "负",
};
const TRAD_CHINESE_INFORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: SIMP_CHINESE_DIGITS,
    units: ["千", "百", "十"],
    omit_one: OmitOne::TensOnly,
    zero_mid: true,
    zero_char: '零',
    zero_value: "零",
    negative: "負",
};
const TRAD_CHINESE_FORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: TRAD_CHINESE_FORMAL_DIGITS,
    units: ["仟", "佰", "拾"],
    omit_one: OmitOne::Never,
    zero_mid: true,
    zero_char: '零',
    zero_value: "零",
    negative: "負",
};
const KOREAN_HANGUL_FORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: KOREAN_HANGUL_DIGITS,
    units: ["천", "백", "십"],
    omit_one: OmitOne::Never,
    zero_mid: false,
    zero_char: '영',
    zero_value: "영",
    negative: "마이너스 ",
};
const KOREAN_HANJA_INFORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: SIMP_CHINESE_DIGITS,
    units: ["千", "百", "十"],
    omit_one: OmitOne::AllUnits,
    zero_mid: false,
    zero_char: '零',
    zero_value: "零",
    negative: "마이너스 ",
};
const KOREAN_HANJA_FORMAL: CjkNumSymbols = CjkNumSymbols {
    digits: &['零', '壹', '貳', '參', '四', '五', '六', '七', '八', '九'],
    units: ["仟", "百", "拾"],
    omit_one: OmitOne::Never,
    zero_mid: false,
    zero_char: '零',
    zero_value: "零",
    negative: "마이너스 ",
};

/// 按家族符号表合成 CJK 数字文本（`value` ≥ 1；0/负值由调用方按家族表处理）。
fn cjk_compose(value: usize, sym: &CjkNumSymbols) -> String {
    let th = value / 1000;
    let rem1 = value % 1000;
    let hu = rem1 / 100;
    let rem2 = rem1 % 100;
    let te = rem2 / 10;
    let ones = rem2 % 10;

    let mut out = String::new();
    for (d, unit, div, lower) in [
        (th, sym.units[0], 1000, rem1),
        (hu, sym.units[1], 100, rem2),
        (te, sym.units[2], 10, ones),
    ] {
        if d == 0 {
            continue;
        }
        let omit = d == 1
            && match sym.omit_one {
                OmitOne::Never => false,
                OmitOne::AllUnits => true,
                OmitOne::TensOnly => div == 10,
            };
        if omit {
            out.push_str(unit);
        } else {
            out.push(sym.digits[d]);
            out.push_str(unit);
        }
        // 中间补零：当前位以下有非零剩余且剩余 < div/10（中间位全空）——101 → 一百零一。
        if sym.zero_mid && div > 1 && lower > 0 && lower < div / 10 {
            out.push(sym.zero_char);
        }
    }
    if ones > 0 {
        out.push(sym.digits[ones]);
    }
    out
}

/// §6.2 家族完整入口（含 0/负值/越界处理）。返回 `None` = 超出该家族 range，调用方
/// 走 decimal fallback（korean 系）。
fn to_cjk_num(value: i64, sym: &CjkNumSymbols, fallback_above: bool) -> Option<String> {
    if fallback_above && value > 9999 {
        return None;
    }
    if value < 0 {
        return Some(format!("{}{}", sym.negative, cjk_compose((-value) as usize, sym)));
    }
    if value == 0 {
        return Some(sym.zero_value.to_string());
    }
    // ≥10000（无 range limit 的日/中家族）：chromium 按位逐字映射（cjk-decimal 式，
    // 10000 = 一〇〇〇〇；WPT 044/078 ground-truth），不进入千进制合成。
    if value > 9999 {
        return Some(to_cjk_decimal(value as usize));
    }
    Some(cjk_compose(value as usize, sym))
}

/// R3835：§6.1/§6.2 fixed alphabetic 假名 + cyclic 天干地支符号表。
const HIRAGANA: &str =
    "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわゐゑをん";
const HIRAGANA_IROHA: &str =
    "いろはにほへとちりぬるをわかよたれそつねならむうゐのおくやまけふこえてあさきゆめみしゑひもせす";
const KATAKANA: &str =
    "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヰヱヲン";
const KATAKANA_IROHA: &str =
    "イロハニホヘトチリヌルヲワカヨタレソツネナラムウヰノオクヤマケフコエテアサキユメミシヱヒモセス";
const CJK_EARTHLY_BRANCH: &str = "子丑寅卯辰巳午未申酉戌亥";
const CJK_HEAVENLY_STEM: &str = "甲乙丙丁戊己庚辛壬癸";

/// alphabetic 通用：符号表 base-N 取值（同 format_counter_alpha 语义，表长泛化）。
/// CSS Counter Styles §3.1.4 system: alphabetic——1→第 1 符号、N→第 N 符号、N+1→"第1第1"。
fn to_symbol_alpha(value: i64, table: &str) -> Option<String> {
    if value <= 0 {
        return None;
    }
    let chars: Vec<char> = table.chars().collect();
    let mut v = value as usize;
    let mut result = Vec::new();
    while v > 0 {
        v -= 1;
        result.push(chars[v % chars.len()]);
        v /= chars.len();
    }
    result.reverse();
    Some(result.into_iter().collect())
}

/// R3835：cyclic 通用（CSS Counter Styles §3.1.4 system: cyclic）——值循环重复符号表：
/// 1→第 1 符号、N→第 N 符号、N+1→**第 1 符号**（与 alphabetic 的 base-N 进位不同，
/// 地支 13=子 而非「子子」）。WPT 201：1=子 … 12=亥。
fn to_symbol_cycle(value: i64, table: &str) -> Option<String> {
    if value <= 0 {
        return None;
    }
    let chars: Vec<char> = table.chars().collect();
    let idx = (value - 1) % chars.len() as i64;
    Some(chars[idx as usize].to_string())
}

/// R2447：armenian 计数器表示（CSS Counter Styles 3 §6.1 预定义，≡ upper-armenian）。
///
/// 传统亚美尼亚数字系统——纯加法（无减法形式，区别于 Roman）。大写亚美尼亚字母块
/// U+0531-U+0554（36 个字母）按十进制位映射，每 9 个字母一组对应 个/十/百/千位：
/// 个位 1-9 → U+0531+0..=8 (Ա..Թ)；十位 10-90 → U+0531+9..=17 (Ժ..Ղ)；
/// 百位 100-900 → U+0531+18..=26 (Ճ..Ջ)；千位 1000-9000 → U+0531+27..=35 (Ռ..Ք)。
/// range 1-9999；0 或 ≥10000 走 decimal fallback（CSS spec range；driving: armenian-008
/// 10000→"10000"）。ground-truth 验证：armenian-006 (1-9) / 007 (43=ԽԳ, 7865=ՒՊԿԵ, 9999=ՔՋՂԹ)。
fn to_armenian(value: usize) -> String {
    if value == 0 || value > 9999 {
        return value.to_string();
    }
    // (位基数, 该位首字母相对 U+0531 的偏移)：个=0、十=9、百=18、千=27。
    const OFFSETS: [(usize, usize); 4] = [(1000, 27), (100, 18), (10, 9), (1, 0)];
    let mut num = value;
    let mut result = String::new();
    for (base, offset) in OFFSETS {
        let digit = num / base;
        if digit > 0 {
            let code = 0x0531 + offset + (digit - 1);
            result.push(char::from_u32(code as u32).expect("armenian uppercase block"));
            num -= digit * base;
        }
    }
    result
}

/// R2449：georgian 计数器表示（CSS Counter Styles 3 §6.1 预定义）。
///
/// 传统格鲁吉亚数字——纯加法（无减法形式）。spec `@counter-style georgian { system: additive;
/// range: 1 19999; additive-symbols: ... }`。码点取自 spec（非连续，含扩展区字母 ჱ/ჲ/ჳ/ჴ/ჵ），
/// 37 对按值降序。range 1-19999；0 或 ≥20000 走 decimal fallback（driving: georgian-014）。
/// ground-truth + spec 双验证：georgian-010/011（1-9, 43=მგ, 7865=ჴყჲე, 9999=ჰშჟთ, 10000=ჵ, 10001=ჵა）。
fn to_georgian(value: usize) -> String {
    if value == 0 || value > 19999 {
        return value.to_string();
    }
    let pairs: [(usize, char); 37] = [
        (10000, '\u{10F5}'),
        (9000, '\u{10F0}'),
        (8000, '\u{10EF}'),
        (7000, '\u{10F4}'),
        (6000, '\u{10EE}'),
        (5000, '\u{10ED}'),
        (4000, '\u{10EC}'),
        (3000, '\u{10EB}'),
        (2000, '\u{10EA}'),
        (1000, '\u{10E9}'),
        (900, '\u{10E8}'),
        (800, '\u{10E7}'),
        (700, '\u{10E6}'),
        (600, '\u{10E5}'),
        (500, '\u{10E4}'),
        (400, '\u{10F3}'),
        (300, '\u{10E2}'),
        (200, '\u{10E1}'),
        (100, '\u{10E0}'),
        (90, '\u{10DF}'),
        (80, '\u{10DE}'),
        (70, '\u{10DD}'),
        (60, '\u{10F2}'),
        (50, '\u{10DC}'),
        (40, '\u{10DB}'),
        (30, '\u{10DA}'),
        (20, '\u{10D9}'),
        (10, '\u{10D8}'),
        (9, '\u{10D7}'),
        (8, '\u{10F1}'),
        (7, '\u{10D6}'),
        (6, '\u{10D5}'),
        (5, '\u{10D4}'),
        (4, '\u{10D3}'),
        (3, '\u{10D2}'),
        (2, '\u{10D1}'),
        (1, '\u{10D0}'),
    ];
    let mut num = value;
    let mut result = String::new();
    for (val, sym) in &pairs {
        while num >= *val {
            result.push(*sym);
            num -= val;
        }
    }
    result
}

/// R2450：hebrew 计数器表示（CSS Counter Styles 3 §6.1 预定义）。
///
/// 传统希伯来数字——纯加法。spec `@counter-style hebrew { system: additive; range: 1 10999;
/// additive-symbols: ... }`。37 对（码点取自 spec，部分符号为 2 字符：千位 = 字母+geresh U+05F3，
/// 15-19 用特殊形 טו/טז/יז/יח/יט 避免神圣名）。range 1-10999；0 或 ≥11000 走 decimal fallback。
/// ground-truth + spec 双验证：hebrew-015/016/016a（1=א, 15=טו, 16=טז, 17=יז, 10999=י׳תתקצט）。
fn to_hebrew(value: usize) -> String {
    if value == 0 || value > 10999 {
        return value.to_string();
    }
    let pairs: [(usize, &str); 37] = [
        (10000, "\u{5D9}\u{5F3}"), // י׳
        (9000, "\u{5D8}\u{5F3}"),  // ט׳
        (8000, "\u{5D7}\u{5F3}"),  // ח׳
        (7000, "\u{5D6}\u{5F3}"),  // ז׳
        (6000, "\u{5D5}\u{5F3}"),  // ו׳
        (5000, "\u{5D4}\u{5F3}"),  // ה׳
        (4000, "\u{5D3}\u{5F3}"),  // ד׳
        (3000, "\u{5D2}\u{5F3}"),  // ג׳
        (2000, "\u{5D1}\u{5F3}"),  // ב׳
        (1000, "\u{5D0}\u{5F3}"),  // א׳
        (400, "\u{5EA}"),          // ת
        (300, "\u{5E9}"),          // ש
        (200, "\u{5E8}"),          // ר
        (100, "\u{5E7}"),          // ק
        (90, "\u{5E6}"),           // צ
        (80, "\u{5E4}"),           // פ
        (70, "\u{5E2}"),           // ע
        (60, "\u{5E1}"),           // ס
        (50, "\u{5E0}"),           // נ
        (40, "\u{5DE}"),           // מ
        (30, "\u{5DC}"),           // ל
        (20, "\u{5DB}"),           // כ
        (19, "\u{5D9}\u{5D8}"),    // יט
        (18, "\u{5D9}\u{5D7}"),    // יח
        (17, "\u{5D9}\u{5D6}"),    // יז
        (16, "\u{5D8}\u{5D6}"),    // טז
        (15, "\u{5D8}\u{5D5}"),    // טו
        (10, "\u{5D9}"),           // י
        (9, "\u{5D8}"),            // ט
        (8, "\u{5D7}"),            // ח
        (7, "\u{5D6}"),            // ז
        (6, "\u{5D5}"),            // ו
        (5, "\u{5D4}"),            // ה
        (4, "\u{5D3}"),            // ד
        (3, "\u{5D2}"),            // ג
        (2, "\u{5D1}"),            // ב
        (1, "\u{5D0}"),            // א
    ];
    let mut num = value;
    let mut result = String::new();
    for (val, sym) in &pairs {
        while num >= *val {
            result.push_str(sym);
            num -= val;
        }
    }
    result
}

struct CounterRepresentation {
    text: String,
    symbol_count: usize,
}

fn text_marker_baseline_offset(style: &ComputedStyle, font_size: f32) -> f32 {
    let line_height = match &style.line_height {
        LineHeightValue::Normal => {
            let ratio = if is_ahem_marker_font(style) { 1.0 } else { 1.164 };
            font_size * ratio
        }
        LineHeightValue::Number(n) => font_size * (*n as f32),
        LineHeightValue::Length(length) => {
            zero_style_system::computed::resolve_length(length, font_size as f64, None, None) as f32
        }
    };
    let ascent_ratio = if is_ahem_marker_font(style) { 0.8 } else { 0.928 };
    // https://www.w3.org/TR/CSS22/visudet.html#line-height
    // Match inline layout's strut baseline: half-leading + font ascent.
    (line_height - font_size).max(0.0) / 2.0 + font_size * ascent_ratio
}

fn is_ahem_marker_font(style: &ComputedStyle) -> bool {
    style
        .font_family
        .iter()
        .any(|family| family.trim_matches('"').eq_ignore_ascii_case("Ahem"))
}

/// https://drafts.csswg.org/css-counter-styles-3/#disclosure-open
/// CSS Counter Styles 3 defines `disclosure-open` as block-end and `disclosure-closed`
/// as inline-end; the concrete triangle therefore depends on writing-mode/direction.
fn disclosure_symbol(open: bool, style: Option<&ComputedStyle>) -> &'static str {
    let writing_mode = style.map(|s| &s.writing_mode);
    let direction = style.map(|s| &s.direction);
    if open {
        return match writing_mode {
            Some(WritingModeValue::VerticalRl) => "◂",
            Some(WritingModeValue::VerticalLr) => "▸",
            _ => "▾",
        };
    }

    match (writing_mode, direction) {
        (Some(WritingModeValue::HorizontalTb), Some(DirectionValue::Rtl)) => "◂",
        (Some(WritingModeValue::VerticalLr | WritingModeValue::VerticalRl), Some(DirectionValue::Rtl)) => "▴",
        (Some(WritingModeValue::VerticalLr | WritingModeValue::VerticalRl), _) => "▾",
        _ => "▸",
    }
}

/// R2392/R2394：按 `@counter-style` 的 system 算法生成计数器表示（marker body，不含 prefix/suffix）。
/// CSS Counter Styles 3 §3.1.4。`None` = 该值无法表示（超出 range / 系统不支持）→ 调用方走 fallback。
/// R2394 注：additive 应用经 A/B 量证为 net-negative（driving WPT 全 font-wall dice/
/// triangle 字形 + system-additive ref 依赖 document.write JS），故仍 defer；extends 只解析到
/// 已注册 counter-style / built-in 系统并保留当前 rule affix。
#[cfg(test)]
fn counter_style_body(rule: &zero_css_parser::ast::CounterStyleRule, value: i64) -> Option<String> {
    counter_style_body_with_registry(rule, value, None, None)
}

fn counter_style_marker_text(
    rule: &zero_css_parser::ast::CounterStyleRule,
    value: i64,
    registry: Option<&std::collections::HashMap<String, zero_css_parser::ast::CounterStyleRule>>,
    style: Option<&ComputedStyle>,
) -> String {
    let body = counter_style_body_with_registry(rule, value, registry, style).unwrap_or_else(|| value.to_string());
    let suffix = match &rule.system {
        zero_css_parser::ast::CounterSystem::Extends(name)
            if rule.suffix == ". " && matches!(name.as_str(), "disclosure-open" | "disclosure-closed") =>
        {
            " "
        }
        _ => rule.suffix.as_str(),
    };
    format!("{}{}{}", rule.prefix, body, suffix)
}

fn counter_style_body_with_registry(
    rule: &zero_css_parser::ast::CounterStyleRule,
    value: i64,
    registry: Option<&std::collections::HashMap<String, zero_css_parser::ast::CounterStyleRule>>,
    style: Option<&ComputedStyle>,
) -> Option<String> {
    use zero_css_parser::ast::CounterSystem;
    if rule.range.as_ref().is_some_and(|ranges| {
        !ranges
            .iter()
            .any(|(lower, upper)| (*lower as i64) <= value && value <= (*upper as i64))
    }) {
        return None;
    }

    let is_negative = value < 0 && !matches!(rule.system, CounterSystem::Cyclic);
    let body_value = if is_negative { value.checked_abs()? } else { value };
    let body = counter_style_raw_body(rule, body_value, registry, style, 0)?;
    let (negative_prefix, negative_suffix): (&str, &str) = if is_negative {
        (rule.negative.0.as_str(), rule.negative.1.as_str())
    } else {
        ("", "")
    };
    let negative_symbol_count = usize::from(!negative_prefix.is_empty()) + usize::from(!negative_suffix.is_empty());
    let representation_symbol_count = body.symbol_count.saturating_add(negative_symbol_count);
    let pad = rule
        .pad
        .as_ref()
        .map(|(width, symbol)| {
            let width = usize::try_from(*width).unwrap_or(0).min(MAX_COUNTER_PAD_WIDTH);
            let needed = width.saturating_sub(representation_symbol_count);
            symbol.repeat(needed)
        })
        .unwrap_or_default();
    Some(format!("{negative_prefix}{pad}{}{negative_suffix}", body.text))
}

fn counter_style_raw_body(
    rule: &zero_css_parser::ast::CounterStyleRule,
    value: i64,
    registry: Option<&std::collections::HashMap<String, zero_css_parser::ast::CounterStyleRule>>,
    style: Option<&ComputedStyle>,
    depth: usize,
) -> Option<CounterRepresentation> {
    use zero_css_parser::ast::CounterSystem;
    if depth > 16 {
        return None;
    }
    let syms = &rule.symbols;
    let len = syms.len();
    match rule.system {
        // R2394：cyclic 用数学取模（rem_euclid），表示任意整数（含 0/负数）；CSS §3.1.4 cyclic
        // 不限值域。旧 `value < 1 → None` 致 disclosure-* 等 cyclic value 0 永远 fallback。
        CounterSystem::Cyclic if len > 0 => Some(CounterRepresentation {
            text: syms[(value - 1).rem_euclid(len as i64) as usize].clone(),
            symbol_count: 1,
        }),
        // fixed [N]：symbols[value - first]；超出 symbols 范围走 fallback。
        CounterSystem::Fixed(first) if len > 0 => {
            let first = first.unwrap_or(1) as i64;
            if value < first {
                return None;
            }
            let offset = value - first;
            if (offset as usize) < len {
                Some(CounterRepresentation {
                    text: syms[offset as usize].clone(),
                    symbol_count: 1,
                })
            } else {
                None
            }
        }
        // symbolic：symbols[(value-1) % len] × ceil(value/len) 次（value >= 1）。
        CounterSystem::Symbolic if len > 0 => {
            if value < 1 {
                return None;
            }
            let idx = ((value - 1) as usize) % len;
            let reps = ((value - 1) as usize) / len + 1;
            Some(CounterRepresentation {
                text: syms[idx].repeat(reps),
                symbol_count: reps,
            })
        }
        // alphabetic：双射 base-len（无零位，类似 Excel 列名）。value >= 1。
        CounterSystem::Alphabetic if len > 0 => {
            if value < 1 || len < 2 {
                // len < 2 无法表示 > 1 的值（spec：alphabetic 须 ≥2 symbols）。
                return if len == 1 && value == 1 {
                    Some(CounterRepresentation {
                        text: syms[0].clone(),
                        symbol_count: 1,
                    })
                } else {
                    None
                };
            }
            let mut n = value as usize;
            let mut digits = Vec::new();
            while n > 0 {
                n -= 1;
                digits.push(n % len);
                n /= len;
            }
            digits.reverse();
            Some(CounterRepresentation {
                text: digits.iter().map(|&d| syms[d].as_str()).collect(),
                symbol_count: digits.len(),
            })
        }
        // numeric：标准 base-len（含零位）。value >= 0；value 0 → symbols[0]。
        CounterSystem::Numeric if len > 0 => {
            if value < 0 || len < 2 {
                return if len == 1 && (0..=1).contains(&value) {
                    Some(CounterRepresentation {
                        text: syms[0].clone(),
                        symbol_count: 1,
                    })
                } else {
                    None
                };
            }
            let mut n = value as usize;
            if n == 0 {
                return Some(CounterRepresentation {
                    text: syms[0].clone(),
                    symbol_count: 1,
                });
            }
            let mut digits: Vec<usize> = Vec::new();
            while n > 0 {
                digits.push(n % len);
                n /= len;
            }
            let text: String = digits.iter().rev().map(|&d| syms[d].as_str()).collect();
            Some(CounterRepresentation {
                text,
                symbol_count: digits.len(),
            })
        }
        // R3743：descriptor-negative/pad WPT 依赖 `extends decimal` / `extends upper-roman`。
        // 先只闭合这两个预定义基样式，additive 与其他 extends 应用仍 defer。
        CounterSystem::Extends(ref name) if name == "decimal" => {
            let text = value.to_string();
            Some(CounterRepresentation {
                symbol_count: text.len(),
                text,
            })
        }
        CounterSystem::Extends(ref name) if name == "upper-roman" => {
            if value == 0 {
                return None;
            }
            let text = to_roman(value.max(0) as usize);
            Some(CounterRepresentation {
                symbol_count: text.chars().count(),
                text,
            })
        }
        CounterSystem::Extends(ref name) if name == "disclosure-open" || name == "disclosure-closed" => {
            Some(CounterRepresentation {
                text: disclosure_symbol(name == "disclosure-open", style).to_string(),
                symbol_count: 1,
            })
        }
        CounterSystem::Extends(ref name) => registry
            .and_then(|rules| rules.get(name))
            .and_then(|base| counter_style_raw_body(base, value, registry, style, depth + 1)),
        // additive / most extends：应用 defer（R2394 A/B 量证 net-negative，见函数注释）→ None（fallback）。
        CounterSystem::Additive => None,
        _ => None,
    }
}

/// R2392：从 stylesheets 收集 `@counter-style` 定义为注册表（name → rule，大小写敏感保留）。
/// 镜像 `animation::register_from_stylesheets` 的 @keyframes 收集模式。
pub(crate) fn build_counter_style_registry(
    rules: &[zero_css_parser::ast::Rule],
) -> std::collections::HashMap<String, zero_css_parser::ast::CounterStyleRule> {
    use zero_css_parser::ast::Rule;
    let mut map = std::collections::HashMap::new();
    for rule in rules {
        if let Rule::CounterStyle(cs) = rule {
            // CSS Counter Styles 3：计数器名大小写敏感（counter-name-case-sensitive）。
            map.entry(cs.name.clone()).or_insert_with(|| cs.clone());
        }
    }
    map
}

/// 将计数器值格式化为字母序列（a/b/.../z/aa/ab/...）。
pub(super) fn format_counter_alpha(value: i64, upper: bool) -> String {
    if value <= 0 {
        return value.to_string();
    }
    let mut v = value as u32;
    let mut result = String::new();
    while v > 0 {
        v -= 1;
        let ch = (b'a' + (v % 26) as u8) as char;
        result.push(ch);
        v /= 26;
    }
    let s: String = result.chars().rev().collect();
    if upper { s.to_uppercase() } else { s }
}

/// 将计数器值格式化为罗马数字。
pub(super) fn format_counter_roman(value: i64, upper: bool) -> String {
    let s = to_roman(value.max(0) as usize);
    if upper { s } else { s.to_lowercase() }
}

/// 按计数器样式格式化整数值为 content 文本。
fn format_counter_text(
    value: i64,
    counter_style: &Option<String>,
    style: &ComputedStyle,
    registry: &std::collections::HashMap<String, zero_css_parser::ast::CounterStyleRule>,
) -> String {
    match counter_style.as_deref() {
        Some("lower-alpha") | Some("lower-latin") => format_counter_alpha(value, false),
        Some("upper-alpha") | Some("upper-latin") => format_counter_alpha(value, true),
        Some("lower-roman") => format_counter_roman(value, false),
        Some("upper-roman") => format_counter_roman(value, true),
        Some("disclosure-open") => disclosure_symbol(true, Some(style)).to_string(),
        Some("disclosure-closed") => disclosure_symbol(false, Some(style)).to_string(),
        Some(name) => registry
            .get(name)
            .and_then(|rule| counter_style_body_with_registry(rule, value, Some(registry), Some(style)))
            .unwrap_or_else(|| value.to_string()),
        _ => value.to_string(),
    }
}

impl super::super::Painter {
    /// 解析 CSS `content` 的字符串与计数器项。
    pub(super) fn resolve_generated_content_text(
        &self,
        content: &ContentComputedValue,
        computed_style: &ComputedStyle,
    ) -> Option<String> {
        match content {
            ContentComputedValue::String(s) => Some(s.clone()),
            ContentComputedValue::Counter { name, style } => Some(format_counter_text(
                self.get_counter(name).unwrap_or(0),
                style,
                computed_style,
                &self.counter_styles,
            )),
            ContentComputedValue::Counters { name, separator, style } => {
                let scopes: Vec<i64> = match self.get_counter_scopes(name) {
                    Some(scopes) if !scopes.is_empty() => scopes.to_vec(),
                    _ => vec![0],
                };
                Some(
                    scopes
                        .iter()
                        .map(|&value| format_counter_text(value, style, computed_style, &self.counter_styles))
                        .collect::<Vec<_>>()
                        .join(separator),
                )
            }
            ContentComputedValue::List(items) => {
                let mut text = String::new();
                for item in items {
                    match item {
                        ContentListItem::Str(value) => text.push_str(value),
                        ContentListItem::Counter { name, style } => {
                            text.push_str(&format_counter_text(
                                self.get_counter(name).unwrap_or(0),
                                style,
                                computed_style,
                                &self.counter_styles,
                            ));
                        }
                        ContentListItem::Counters { name, separator, style } => {
                            let scopes: Vec<i64> = match self.get_counter_scopes(name) {
                                Some(scopes) if !scopes.is_empty() => scopes.to_vec(),
                                _ => vec![0],
                            };
                            text.push_str(
                                &scopes
                                    .iter()
                                    .map(|&value| {
                                        format_counter_text(value, style, computed_style, &self.counter_styles)
                                    })
                                    .collect::<Vec<_>>()
                                    .join(separator),
                            );
                        }
                    }
                }
                Some(text)
            }
            _ => None,
        }
    }

    /// 绘制列表项标记（disc / circle / square / decimal / alpha / roman / list-style-image）。
    pub(crate) fn paint_list_marker(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };

        let node = match doc.get(node_id) {
            Some(n) => n,
            None => return,
        };

        match &node.kind {
            NodeKind::Element(elem) if elem.local_name() == "li" => {}
            _ => return,
        }

        // list-style-image 优先
        match &style.list_style_image {
            zero_style_system::ListStyleImageComputedValue::Url(url) => {
                let font_size: f32 = match style.font_size {
                    LengthValue::Px(s) => s as f32,
                    _ => 16.0,
                };
                let img_size = font_size;
                let marker_x = abs_x + box_node.border_left - img_size * 1.5;
                let marker_y = abs_y + box_node.border_top + box_node.padding_top;
                self.primitives.add_image(ImagePrimitive {
                    rect: Rect::new(marker_x, marker_y, img_size, img_size),
                    image_key: ImageKey::new(image_resource_key(url, self.document_url.as_deref())),
                    clip: None,
                });
                return;
            }
            zero_style_system::ListStyleImageComputedValue::None => {}
        }

        if style.list_style_type == ListStyleTypeValue::None {
            return;
        }

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => 16.0,
        };
        if font_size <= 0.0 {
            return;
        }

        let color = color_value_to_render(style.marker_pseudo.as_deref().map(|m| &m.color).unwrap_or(&style.color));
        let default_font_id = self.resolve_style_font_id(&style.font_family, style).0;
        let variation_style = style.marker_pseudo.as_deref().unwrap_or(style);
        let variations = crate::text_metrics::paint_font_variations(&variation_style.font_variation_settings);
        let font_variation_id = self.primitives.intern_font_variations(&variations);
        let marker_size = font_size * 0.4;
        let marker_x = abs_x + box_node.border_left;
        let marker_y = abs_y + box_node.border_top + box_node.padding_top;

        let actual_marker_x = match style.list_style_position {
            zero_css_parser::values::ListStylePositionValue::Outside => marker_x - marker_size * 2.5,
            zero_css_parser::values::ListStylePositionValue::Inside => marker_x + marker_size * 0.5,
        };
        let text_marker_x = match style.list_style_position {
            zero_css_parser::values::ListStylePositionValue::Outside => actual_marker_x,
            zero_css_parser::values::ListStylePositionValue::Inside => marker_x,
        };
        let text_marker_font_size = font_size;
        let text_marker_baseline_y = marker_y + text_marker_baseline_offset(style, text_marker_font_size);

        // ::marker 伪元素 content 覆盖（CSS Lists 3）：marker_pseudo 存在时，content 决定标记
        // 文本——`none` 抑制标记；具体生成内容（String/Counter/Counters/List）替代默认
        // list-style-type 标记；`normal`/Attr/Url 落默认标记。color 已由上方 marker_pseudo.color
        // 覆盖。marker_pseudo 默认 None → 整块跳过，默认 marker 零行为变更。
        if let Some(marker_style) = style.marker_pseudo.as_deref() {
            match &marker_style.content {
                // content: none → 抑制标记。
                ContentComputedValue::None => return,
                // 具体生成内容 → 解析为文本，按 text marker 同位绘字。
                ContentComputedValue::String(_)
                | ContentComputedValue::Counter { .. }
                | ContentComputedValue::Counters { .. }
                | ContentComputedValue::List(_) => {
                    if let Some(text) = self.resolve_generated_content_text(&marker_style.content, style) {
                        let mut char_x = text_marker_x;
                        let char_y = text_marker_baseline_y;
                        for ch in text.chars() {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: char_x,
                                y: char_y,
                                font_size: text_marker_font_size,
                                color,
                                glyph_id: ch as u32,
                                font_glyph_index: None,
                                source: None,
                                font_id: default_font_id,
                                font_variation_id,
                                bitmap_width: None,
                                bitmap_height: None,
                                rotation: 0.0,
                                synthetic_italic: false,
                            });
                            char_x += self.measure_char_cached(default_font_id.0, ch, text_marker_font_size, false);
                        }
                        return;
                    }
                }
                // normal/Attr/Url → 用默认 list-style-type 标记（color 已覆盖）。
                _ => {}
            }
        }

        match &style.list_style_type {
            ListStyleTypeValue::Disc => {
                // R1882：disc 是实心圆（CSS §12.5 / chromium），非方块。用圆角矩形
                //（radius = marker_size/2 = 正方形四角全圆 → 圆）近似实心圆 marker。
                self.primitives.add_rounded_rect(RoundedRectPrimitive::uniform(
                    Rect::new(
                        actual_marker_x,
                        marker_y + font_size * 0.3 - marker_size / 2.0,
                        marker_size,
                        marker_size,
                    ),
                    color,
                    marker_size / 2.0,
                ));
            }
            ListStyleTypeValue::Circle => {
                // R1883：circle 是空心圆 outline（CSS §12.5 / chromium），非水平线胶囊。
                // 旧 add_stroke（length=width=marker_size + Round cap）实为 2:1 胶囊（椭圆），
                // 非圆。改用 PathStroke 多边形（24 点圆周）描真圆，line_width 细（~0.2em）。
                let cx = actual_marker_x + marker_size / 2.0;
                let cy = marker_y + font_size * 0.3;
                let radius = marker_size / 2.0;
                let line_w = (marker_size * 0.2).max(1.0);
                let steps = 24;
                let mut verts: Vec<f32> = Vec::with_capacity(steps * 2);
                for i in 0..steps {
                    let theta = (i as f32) * (2.0 * std::f32::consts::PI / steps as f32);
                    verts.push(cx + radius * theta.cos());
                    verts.push(cy + radius * theta.sin());
                }
                self.primitives.add_path_stroke(verts, color, line_w, true);
            }
            ListStyleTypeValue::Square => {
                self.primitives.add_fill(
                    Rect::new(
                        actual_marker_x,
                        marker_y + font_size * 0.3 - marker_size / 2.0,
                        marker_size,
                        marker_size,
                    ),
                    color,
                );
            }
            ListStyleTypeValue::Decimal | ListStyleTypeValue::DecimalLeadingZero => {
                // 优先使用 CSS counter "list-item"，回退到兄弟索引
                let index = self
                    .get_counter("list-item")
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                let text = if matches!(style.list_style_type, ListStyleTypeValue::DecimalLeadingZero)
                    && (0..10).contains(&index)
                {
                    format!("0{index}.")
                } else {
                    format!("{index}.")
                };
                let mut char_x = text_marker_x;
                let char_y = text_marker_baseline_y;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: text_marker_font_size,
                        color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic: false,
                    });
                    char_x += self.measure_char_cached(default_font_id.0, ch, text_marker_font_size, false);
                }
            }
            ListStyleTypeValue::LowerAlpha | ListStyleTypeValue::UpperAlpha => {
                let index = self
                    .get_counter("list-item")
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                let ch = if index > 0 && index <= 26 {
                    let base = if matches!(style.list_style_type, ListStyleTypeValue::LowerAlpha) {
                        b'a'
                    } else {
                        b'A'
                    };
                    (base + (index as u8 - 1)) as char
                } else {
                    '?'
                };
                let text = format!("{ch}.");
                let mut char_x = text_marker_x;
                let char_y = text_marker_baseline_y;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: text_marker_font_size,
                        color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic: false,
                    });
                    char_x += self.measure_char_cached(default_font_id.0, ch, text_marker_font_size, false);
                }
            }
            ListStyleTypeValue::LowerRoman | ListStyleTypeValue::UpperRoman => {
                let index = self
                    .get_counter("list-item")
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                let text = if index <= 0 {
                    format!("{index}.")
                } else {
                    let roman = to_roman(index as usize);
                    if matches!(style.list_style_type, ListStyleTypeValue::LowerRoman) {
                        format!("{}.", roman.to_lowercase())
                    } else {
                        format!("{roman}.")
                    }
                };
                let mut char_x = text_marker_x;
                let char_y = text_marker_baseline_y;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: text_marker_font_size,
                        color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic: false,
                    });
                    char_x += self.measure_char_cached(default_font_id.0, ch, text_marker_font_size, false);
                }
            }
            // R2445：lower-greek / persian 预定义计数器样式（CSS Counter Styles 3 §6）。
            // R2447：+ armenian（§6.1 additive）。R2448：+ lower-armenian（小写）。R2449：+ georgian。R2450：+ hebrew。R2451：+ arabic-indic。
            // R2471：+ 11 numeric scripts（devanagari/bengali/gujarati/gurmukhi/kannada/malayalam/
            // tamil/telugu/lao/khmer/myanmar），均 numeric system = to_digit_script(value, base)。
            ListStyleTypeValue::LowerGreek
            | ListStyleTypeValue::Persian
            | ListStyleTypeValue::Armenian
            | ListStyleTypeValue::LowerArmenian
            | ListStyleTypeValue::Georgian
            | ListStyleTypeValue::Hebrew
            | ListStyleTypeValue::ArabicIndic
            | ListStyleTypeValue::Devanagari
            | ListStyleTypeValue::Bengali
            | ListStyleTypeValue::Gujarati
            | ListStyleTypeValue::Gurmukhi
            | ListStyleTypeValue::Kannada
            | ListStyleTypeValue::Malayalam
            | ListStyleTypeValue::Tamil
            | ListStyleTypeValue::Telugu
            | ListStyleTypeValue::Lao
            | ListStyleTypeValue::Khmer
            | ListStyleTypeValue::Myanmar
            | ListStyleTypeValue::CjkDecimal
            | ListStyleTypeValue::JapaneseInformal
            | ListStyleTypeValue::JapaneseFormal
            | ListStyleTypeValue::SimpChineseInformal
            | ListStyleTypeValue::SimpChineseFormal
            | ListStyleTypeValue::TradChineseInformal
            | ListStyleTypeValue::TradChineseFormal
            | ListStyleTypeValue::KoreanHangulFormal
            | ListStyleTypeValue::KoreanHanjaInformal
            | ListStyleTypeValue::KoreanHanjaFormal
            | ListStyleTypeValue::CjkEarthlyBranch
            | ListStyleTypeValue::CjkHeavenlyStem
            | ListStyleTypeValue::Hiragana
            | ListStyleTypeValue::HiraganaIroha
            | ListStyleTypeValue::Katakana
            | ListStyleTypeValue::KatakanaIroha => {
                let index = self
                    .get_counter("list-item")
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                // lower-armenian = armenian 算法输出 + Unicode to_lowercase（Armenian 双层壳，
                // U+0531→U+0561 等；Rust 用 Unicode case folding，ground-truth 验证 1=ա/9999=քջղթ）。
                // R2471 numeric scripts：base = 该脚本 DIGIT ZERO 码点（U+0966/09E6/0AE6/0A66/
                // 0CE6/0D66/0BE6/0C66/0ED0/17E0/1040）。
                let body = match style.list_style_type {
                    ListStyleTypeValue::LowerGreek if index > 0 => to_greek(index as usize),
                    ListStyleTypeValue::Persian if index >= 0 => to_persian(index as usize),
                    ListStyleTypeValue::Armenian if index > 0 => to_armenian(index as usize),
                    ListStyleTypeValue::LowerArmenian if index > 0 => to_armenian(index as usize).to_lowercase(),
                    ListStyleTypeValue::Georgian if index > 0 => to_georgian(index as usize),
                    ListStyleTypeValue::Hebrew if index > 0 => to_hebrew(index as usize),
                    ListStyleTypeValue::ArabicIndic if index >= 0 => to_arabic_indic(index as usize),
                    ListStyleTypeValue::Devanagari if index >= 0 => to_digit_script(index as usize, 0x0966),
                    ListStyleTypeValue::Bengali if index >= 0 => to_digit_script(index as usize, 0x09E6),
                    ListStyleTypeValue::Gujarati if index >= 0 => to_digit_script(index as usize, 0x0AE6),
                    ListStyleTypeValue::Gurmukhi if index >= 0 => to_digit_script(index as usize, 0x0A66),
                    ListStyleTypeValue::Kannada if index >= 0 => to_digit_script(index as usize, 0x0CE6),
                    ListStyleTypeValue::Malayalam if index >= 0 => to_digit_script(index as usize, 0x0D66),
                    ListStyleTypeValue::Tamil if index >= 0 => to_digit_script(index as usize, 0x0BE6),
                    ListStyleTypeValue::Telugu if index >= 0 => to_digit_script(index as usize, 0x0C66),
                    ListStyleTypeValue::Lao if index >= 0 => to_digit_script(index as usize, 0x0ED0),
                    ListStyleTypeValue::Khmer if index >= 0 => to_digit_script(index as usize, 0x17E0),
                    ListStyleTypeValue::Myanmar if index >= 0 => to_digit_script(index as usize, 0x1040),
                    ListStyleTypeValue::CjkDecimal if index >= 0 => to_cjk_decimal(index as usize),
                    // R3835：§6.2 limited CJK/日/韩 + 假名 + 天干地支。越界（korean 系
                    // range 1-9999、假名/循环 value ≤ 0）→ decimal fallback。
                    ListStyleTypeValue::JapaneseInformal => {
                        to_cjk_num(index, &JAPANESE_INFORMAL, false).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::JapaneseFormal => {
                        to_cjk_num(index, &JAPANESE_FORMAL, false).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::SimpChineseInformal => {
                        to_cjk_num(index, &SIMP_CHINESE_INFORMAL, false).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::SimpChineseFormal => {
                        to_cjk_num(index, &SIMP_CHINESE_FORMAL, false).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::TradChineseInformal => {
                        to_cjk_num(index, &TRAD_CHINESE_INFORMAL, false).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::TradChineseFormal => {
                        to_cjk_num(index, &TRAD_CHINESE_FORMAL, false).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::KoreanHangulFormal => {
                        to_cjk_num(index, &KOREAN_HANGUL_FORMAL, true).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::KoreanHanjaInformal => {
                        to_cjk_num(index, &KOREAN_HANJA_INFORMAL, true).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::KoreanHanjaFormal => {
                        to_cjk_num(index, &KOREAN_HANJA_FORMAL, true).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::CjkEarthlyBranch => {
                        to_symbol_cycle(index, CJK_EARTHLY_BRANCH).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::CjkHeavenlyStem => {
                        to_symbol_cycle(index, CJK_HEAVENLY_STEM).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::Hiragana => {
                        to_symbol_alpha(index, HIRAGANA).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::HiraganaIroha => {
                        to_symbol_alpha(index, HIRAGANA_IROHA).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::Katakana => {
                        to_symbol_alpha(index, KATAKANA).unwrap_or_else(|| index.to_string())
                    }
                    ListStyleTypeValue::KatakanaIroha => {
                        to_symbol_alpha(index, KATAKANA_IROHA).unwrap_or_else(|| index.to_string())
                    }
                    _ => index.to_string(),
                };
                // R3835：suffix 按家族（见 counter_suffix）。
                let suffix = counter_suffix(&style.list_style_type);
                let text = format!("{body}{suffix}");
                let mut char_x = text_marker_x;
                let char_y = text_marker_baseline_y;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: text_marker_font_size,
                        color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic: false,
                    });
                    char_x += self.measure_char_cached(default_font_id.0, ch, text_marker_font_size, false);
                }
            }
            ListStyleTypeValue::None => {}
            ListStyleTypeValue::DisclosureOpen | ListStyleTypeValue::DisclosureClosed => {
                let text = format!(
                    "{} ",
                    disclosure_symbol(
                        matches!(style.list_style_type, ListStyleTypeValue::DisclosureOpen),
                        Some(style)
                    )
                );
                let mut char_x = text_marker_x;
                let char_y = text_marker_baseline_y;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: text_marker_font_size,
                        color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic: false,
                    });
                    char_x += self.measure_char_cached(default_font_id.0, ch, text_marker_font_size, false);
                }
            }
            // list-style-type: <string>（CSS Lists 3）：固定字符串标记（非计数器，每个 li 同值）。
            // 按文本 marker 同位绘字（≡ Decimal/script arm 字形循环）；空串 → 无标记。
            ListStyleTypeValue::String(s) => {
                let mut char_x = text_marker_x;
                let char_y = text_marker_baseline_y;
                for ch in s.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: text_marker_font_size,
                        color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic: false,
                    });
                    char_x += self.measure_char_cached(default_font_id.0, ch, text_marker_font_size, false);
                }
            }
            // R2392：自定义计数器样式（@counter-style）。查注册表 → 按 system 生成 body
            // → prefix+body+suffix。未定义 / 超出 range → fallback（decimal "N."）。
            // R2394 注：additive/extends 应用 defer（A/B net-negative，见 counter_style_body 注释）。
            ListStyleTypeValue::Custom(name) => {
                let value = self
                    .get_counter("list-item")
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                let text = match self.counter_styles.get(name) {
                    Some(rule) => counter_style_marker_text(rule, value, Some(&self.counter_styles), Some(style)),
                    // 未定义的自定义名 → fallback decimal（CSS Counter Styles 3 §3.1.3）。
                    None => format!("{value}."),
                };
                let mut char_x = text_marker_x;
                let char_y = text_marker_baseline_y;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: text_marker_font_size,
                        color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic: false,
                    });
                    char_x += self.measure_char_cached(default_font_id.0, ch, text_marker_font_size, false);
                }
            }
        }

        // R3835：inside **counter 型** marker → 记录 marker 步进宽度，paint_text 首片段
        // 右移（CSS Lists 3 §list-style-position：inside marker 是首行行盒第一个 inline，
        // 内容排其后。旧实现 marker 与内容同 x 重叠——css-counter-styles 024 等 `1AAA`）。
        // 从样式源独立重推 marker 文本测宽（marker 绘制臂各自局部 char_x，不外传）；
        // marker 绘制不在热路径，重复格式化可接受。outside 几何 marker 不记录
        //（outside 内容位置由布局 margin 让位，现状正确）。
        // String / Custom（::marker content / list-style-type:"..."）不记录：其 marker
        // 文本即 inline 流的开头，内容紧随其后无后缀间隔（css-pseudo marker-content-018：
        // content "1 " 尾随空格语义 = 文本自身，偏移会破坏 white-space 用例；ZW 内容起点
        // 不动 + marker 画在起点 ≡ chromium 紧贴排布）。
        if matches!(
            style.list_style_position,
            zero_css_parser::values::ListStylePositionValue::Inside
        ) && !matches!(
            style.list_style_type,
            ListStyleTypeValue::None | ListStyleTypeValue::String(_) | ListStyleTypeValue::Custom(_)
        ) {
            let index = self
                .get_counter("list-item")
                .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
            let text: String = match &style.list_style_type {
                ListStyleTypeValue::Decimal => format!("{index}."),
                ListStyleTypeValue::DecimalLeadingZero if (0..10).contains(&index) => format!("0{index}."),
                ListStyleTypeValue::LowerAlpha if index > 0 && index <= 26 => {
                    format!("{}.", (b'a' + (index as u8 - 1)) as char)
                }
                ListStyleTypeValue::UpperAlpha if index > 0 && index <= 26 => {
                    format!("{}.", (b'A' + (index as u8 - 1)) as char)
                }
                ListStyleTypeValue::LowerRoman => {
                    if index <= 0 {
                        format!("{index}.")
                    } else {
                        format!("{}.", to_roman(index as usize).to_lowercase())
                    }
                }
                ListStyleTypeValue::UpperRoman => {
                    if index <= 0 {
                        format!("{index}.")
                    } else {
                        format!("{}.", to_roman(index as usize))
                    }
                }
                ListStyleTypeValue::LowerGreek if index > 0 => format!("{}.", to_greek(index as usize)),
                ListStyleTypeValue::Persian if index >= 0 => format!("{}.", to_persian(index as usize)),
                ListStyleTypeValue::Armenian if index > 0 => format!("{}.", to_armenian(index as usize)),
                ListStyleTypeValue::LowerArmenian if index > 0 => {
                    format!("{}.", to_armenian(index as usize).to_lowercase())
                }
                ListStyleTypeValue::Georgian if index > 0 => format!("{}.", to_georgian(index as usize)),
                ListStyleTypeValue::Hebrew if index > 0 => format!("{}.", to_hebrew(index as usize)),
                ListStyleTypeValue::ArabicIndic if index >= 0 => format!("{}.", to_arabic_indic(index as usize)),
                ListStyleTypeValue::CjkDecimal if index >= 0 => format!("{}.", to_cjk_decimal(index as usize)),
                ListStyleTypeValue::Devanagari if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x0966)),
                ListStyleTypeValue::Bengali if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x09E6)),
                ListStyleTypeValue::Gujarati if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x0AE6)),
                ListStyleTypeValue::Gurmukhi if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x0A66)),
                ListStyleTypeValue::Kannada if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x0CE6)),
                ListStyleTypeValue::Malayalam if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x0D66)),
                ListStyleTypeValue::Tamil if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x0BE6)),
                ListStyleTypeValue::Telugu if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x0C66)),
                ListStyleTypeValue::Lao if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x0ED0)),
                ListStyleTypeValue::Khmer if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x17E0)),
                ListStyleTypeValue::Myanmar if index >= 0 => format!("{}.", to_digit_script(index as usize, 0x1040)),
                // R3835：§6.2 家族（与绘制臂同源，越界 → decimal）。
                ListStyleTypeValue::JapaneseInformal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &JAPANESE_INFORMAL, false).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::JapaneseFormal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &JAPANESE_FORMAL, false).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::SimpChineseInformal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &SIMP_CHINESE_INFORMAL, false).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::SimpChineseFormal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &SIMP_CHINESE_FORMAL, false).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::TradChineseInformal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &TRAD_CHINESE_INFORMAL, false).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::TradChineseFormal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &TRAD_CHINESE_FORMAL, false).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::KoreanHangulFormal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &KOREAN_HANGUL_FORMAL, true).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::KoreanHanjaInformal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &KOREAN_HANJA_INFORMAL, true).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::KoreanHanjaFormal => {
                    format!(
                        "{}{}",
                        to_cjk_num(index, &KOREAN_HANJA_FORMAL, true).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::CjkEarthlyBranch => {
                    format!(
                        "{}{}",
                        to_symbol_cycle(index, CJK_EARTHLY_BRANCH).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::CjkHeavenlyStem => {
                    format!(
                        "{}{}",
                        to_symbol_cycle(index, CJK_HEAVENLY_STEM).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::Hiragana => {
                    format!(
                        "{}{}",
                        to_symbol_alpha(index, HIRAGANA).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::HiraganaIroha => {
                    format!(
                        "{}{}",
                        to_symbol_alpha(index, HIRAGANA_IROHA).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::Katakana => {
                    format!(
                        "{}{}",
                        to_symbol_alpha(index, KATAKANA).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::KatakanaIroha => {
                    format!(
                        "{}{}",
                        to_symbol_alpha(index, KATAKANA_IROHA).unwrap_or_else(|| index.to_string()),
                        counter_suffix(&style.list_style_type)
                    )
                }
                ListStyleTypeValue::String(s) => s.clone(),
                ListStyleTypeValue::Custom(name) => match self.counter_styles.get(name) {
                    Some(rule) => counter_style_marker_text(rule, index, Some(&self.counter_styles), Some(style)),
                    None => format!("{index}."),
                },
                // disc/circle/square 几何 marker：宽度 = marker_size（同 paint 臂）。
                ListStyleTypeValue::Disc | ListStyleTypeValue::Circle | ListStyleTypeValue::Square => {
                    self.list_inside_marker_advance
                        .insert(node_id, font_size * 0.4 + font_size * 0.1);
                    return;
                }
                // disclosure 文本臂（含尾随空格）。
                ListStyleTypeValue::DisclosureOpen | ListStyleTypeValue::DisclosureClosed => {
                    format!(
                        "{} ",
                        disclosure_symbol(
                            matches!(style.list_style_type, ListStyleTypeValue::DisclosureOpen),
                            Some(style)
                        )
                    )
                }
                ListStyleTypeValue::None => return,
                // 其余（guard 未命中的 index 越界面 / 未列举面）与绘制臂同走 decimal
                // fallback 或空 advance：alpha>26 绘 '?'，scripts 越界走 index.to_string()。
                ListStyleTypeValue::DecimalLeadingZero
                | ListStyleTypeValue::LowerAlpha
                | ListStyleTypeValue::UpperAlpha
                | ListStyleTypeValue::LowerGreek
                | ListStyleTypeValue::Persian
                | ListStyleTypeValue::Armenian
                | ListStyleTypeValue::LowerArmenian
                | ListStyleTypeValue::Georgian
                | ListStyleTypeValue::Hebrew
                | ListStyleTypeValue::ArabicIndic
                | ListStyleTypeValue::Devanagari
                | ListStyleTypeValue::Bengali
                | ListStyleTypeValue::Gujarati
                | ListStyleTypeValue::Gurmukhi
                | ListStyleTypeValue::Kannada
                | ListStyleTypeValue::Malayalam
                | ListStyleTypeValue::Tamil
                | ListStyleTypeValue::Telugu
                | ListStyleTypeValue::Lao
                | ListStyleTypeValue::Khmer
                | ListStyleTypeValue::Myanmar
                | ListStyleTypeValue::CjkDecimal => format!("{index}."),
            };
            let mut advance: f32 = text
                .chars()
                .map(|ch| self.measure_char_cached(default_font_id.0, ch, font_size, false))
                .sum();
            // chromium inside counter marker 与内容间的间隔 = 计数器样式 suffix 的尾随
            // 空格（predefined suffix = ". "，WPT ref "X. X"）。本块只处理 counter 类
            // marker（String/Custom 已在入口排除），间隔恒补。
            advance += self.measure_char_cached(default_font_id.0, ' ', font_size, false);
            self.list_inside_marker_advance.insert(node_id, advance);
        }
    }

    /// 计算当前列表项在其兄弟中的 1-based 索引。
    fn compute_list_item_index(&self, doc: &Document, node_id: NodeId) -> i64 {
        list_item_counter(doc, node_id)
    }
}

/// R1701：计算 `<li>` 的列表序号（counter），尊重 HTML4 `<ol start=N>` 起始值
/// 与 `<li value=N>` 重置值（后续 li 从 value+1 继续）。无属性时等价 1-based 兄弟
/// 位置（向后兼容）。fixture 22 `<ol start="3" type="A">` → C/D/J(`value=10`)/K。
pub(super) fn list_item_counter(doc: &Document, node_id: NodeId) -> i64 {
    let parent_id = match doc.parent_node(node_id) {
        Some(id) => id,
        None => return 1,
    };
    // <ol start=N>：默认 1；CSS list-item counter 允许负数，非数字忽略。
    let start: i64 = doc
        .get_attribute(parent_id, "start")
        .and_then(|v| v.trim().parse::<i64>().ok())
        .unwrap_or(1);
    let mut counter = start;
    let mut found = false;
    for child_id in doc.child_nodes(parent_id) {
        if child_id == node_id {
            found = true;
            break;
        }
        if is_li(doc, child_id) {
            // 该 li 兄弟消耗一个序号；若带 value=N，先把 counter 重置为 N。
            if let Some(v) = doc
                .get_attribute(child_id, "value")
                .and_then(|s| s.trim().parse::<i64>().ok())
            {
                counter = v;
            }
            counter += 1;
        }
    }
    if !found {
        return 1;
    }
    // 目标 li 自身 value= 设其序号（其上方兄弟的循环已从 value+1 继续）。
    if let Some(v) = doc
        .get_attribute(node_id, "value")
        .and_then(|s| s.trim().parse::<i64>().ok())
    {
        return v;
    }
    counter
}

fn is_li(doc: &Document, id: NodeId) -> bool {
    doc.get(id)
        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == "li"))
}

#[cfg(test)]
mod tests {
    use super::CJK_EARTHLY_BRANCH;
    use super::JAPANESE_INFORMAL;
    use super::KOREAN_HANJA_FORMAL;
    use super::SIMP_CHINESE_FORMAL;
    use super::counter_style_body;
    use super::counter_style_marker_text;
    use super::counter_suffix;
    use super::list_item_counter;
    use super::to_arabic_indic;
    use super::to_armenian;
    use super::to_cjk_decimal;
    use super::to_cjk_num;
    use super::to_digit_script;
    use super::to_georgian;
    use super::to_hebrew;
    use super::to_symbol_cycle;
    use zero_dom::parse_html;
    use zero_style_system::{ComputedStyle, DirectionValue, WritingModeValue};

    /// R2471：numeric system 计数器（CSS Counter Styles 3 §6.1）ground-truth 对齐 WPT ref。
    /// 验证值取自 css-counter-styles/{devanagari,bengali,...}/css3-counter-styles-NNN 真实期望。
    #[test]
    fn digit_script_counter_matches_wpt_ground_truth() {
        // devanagari ०-९（U+0966-U+096F）
        assert_eq!(to_digit_script(0, 0x0966), "०");
        assert_eq!(to_digit_script(9, 0x0966), "९");
        assert_eq!(to_digit_script(10, 0x0966), "१०");
        assert_eq!(to_digit_script(123, 0x0966), "१२३");
        // bengali ০-৯（U+09E6+）
        assert_eq!(to_digit_script(1, 0x09E6), "১");
        // tamil ௦-௯（U+0BE6+）
        assert_eq!(to_digit_script(0, 0x0BE6), "௦");
        assert_eq!(to_digit_script(9, 0x0BE6), "௯");
        // myanmar ၀-၉（U+1040+）
        assert_eq!(to_digit_script(0, 0x1040), "၀");
        assert_eq!(to_digit_script(42, 0x1040), "၄၂");
        // 与 arabic-indic 同算法（ground-truth 一致性）
        assert_eq!(to_digit_script(123, 0x0660), to_arabic_indic(123));
    }

    /// R2472：cjk-decimal（CJK ideographic digits，非连续 lookup）ground-truth 对齐
    /// css-counter-styles/cjk-decimal/css3-counter-styles-004 ref。
    #[test]
    fn cjk_decimal_matches_wpt_ground_truth() {
        assert_eq!(to_cjk_decimal(0), "〇");
        assert_eq!(to_cjk_decimal(9), "九");
        assert_eq!(to_cjk_decimal(10), "一〇");
        assert_eq!(to_cjk_decimal(101), "一〇一");
        assert_eq!(to_cjk_decimal(1002), "一〇〇二");
    }

    fn li(doc: &zero_dom::Document, n: usize) -> zero_dom::NodeId {
        doc.get_elements_by_tag_name("li")[n]
    }

    /// R2447：armenian 计数器（CSS Counter Styles 3 §6.1）ground-truth 对齐 WPT ref。
    /// 验证值取自 css-counter-styles/armenian/css3-counter-styles-006/007/008 真实期望输出。
    #[test]
    fn armenian_counter_matches_wpt_ground_truth() {
        // 单位 1-9（armenian-006）
        assert_eq!(to_armenian(1), "Ա");
        assert_eq!(to_armenian(9), "Թ");
        // 十位（armenian-007：10=Ժ, 11=ԺԱ, 43=ԽԳ, 99=ՂԹ）
        assert_eq!(to_armenian(10), "Ժ");
        assert_eq!(to_armenian(11), "ԺԱ");
        assert_eq!(to_armenian(43), "ԽԳ");
        assert_eq!(to_armenian(99), "ՂԹ");
        // 百/千混合（armenian-007：7865=ՒՊԿԵ, 9999=ՔՋՂԹ）
        assert_eq!(to_armenian(7865), "ՒՊԿԵ");
        assert_eq!(to_armenian(9999), "ՔՋՂԹ");
        // range 外走 decimal fallback（armenian-008：0→"0", 10000→"10000"）
        assert_eq!(to_armenian(0), "0");
        assert_eq!(to_armenian(10000), "10000");
    }

    /// R2448：lower-armenian = to_armenian + to_lowercase（ground-truth 对齐 lower-armenian-111/114）。
    #[test]
    fn lower_armenian_counter_matches_wpt_ground_truth() {
        assert_eq!(to_armenian(1).to_lowercase(), "ա");
        assert_eq!(to_armenian(10).to_lowercase(), "ժ");
        assert_eq!(to_armenian(43).to_lowercase(), "խգ");
        assert_eq!(to_armenian(9999).to_lowercase(), "քջղթ");
    }

    /// R2449：georgian 计数器（CSS Counter Styles 3 §6.1）ground-truth + spec 双对齐。
    /// 验证值取自 css-counter-styles/georgian/css3-counter-styles-010/011/014 真实期望输出。
    #[test]
    fn georgian_counter_matches_wpt_ground_truth() {
        // 单位（georgian-010）
        assert_eq!(to_georgian(1), "ა");
        assert_eq!(to_georgian(8), "ჱ"); // 扩展区 U+10F1
        assert_eq!(to_georgian(9), "თ");
        // 十/百/千（georgian-011：10=ი, 43=მგ, 7865=ჴყჲე, 9999=ჰშჟთ）
        assert_eq!(to_georgian(10), "ი");
        assert_eq!(to_georgian(43), "მგ");
        assert_eq!(to_georgian(7865), "ჴყჲე");
        assert_eq!(to_georgian(9999), "ჰშჟთ");
        // 10000-19999 仍 in range（georgian-011：10000=ჵ, 10001=ჵა）
        assert_eq!(to_georgian(10000), "ჵ");
        assert_eq!(to_georgian(10001), "ჵა");
        // range 外走 decimal fallback（georgian-014：0→"0", 20000→"20000"）
        assert_eq!(to_georgian(0), "0");
        assert_eq!(to_georgian(20000), "20000");
    }

    /// R2450：hebrew 计数器（CSS Counter Styles 3 §6.1）ground-truth + spec 双对齐。
    /// 验证值取自 css-counter-styles/hebrew/css3-counter-styles-015/016/016a 真实期望输出。
    #[test]
    fn hebrew_counter_matches_wpt_ground_truth() {
        // 单位（hebrew-015）
        assert_eq!(to_hebrew(1), "א");
        assert_eq!(to_hebrew(8), "ח");
        assert_eq!(to_hebrew(9), "ט");
        // 15-19 特殊形（hebrew-016：避免神圣名 יה/יו；15=טו, 16=טז, 17=יז）
        assert_eq!(to_hebrew(10), "י");
        assert_eq!(to_hebrew(11), "יא");
        assert_eq!(to_hebrew(15), "טו");
        assert_eq!(to_hebrew(16), "טז");
        assert_eq!(to_hebrew(17), "יז");
        // 千位 + geresh（hebrew-016a：10999=י׳תתקצט）
        assert_eq!(to_hebrew(10999), "י\u{5F3}תתקצט");
        // range 外走 decimal fallback（0→"0", ≥11000→decimal）
        assert_eq!(to_hebrew(0), "0");
        assert_eq!(to_hebrew(11000), "11000");
    }

    /// R2451：arabic-indic 计数器（CSS Counter Styles 3 §6.1，numeric）ground-truth 对齐
    /// arabic-indic-101（阿拉伯-印度数字 ٠-٩ U+0660-U+0669，core Arabic block）。
    #[test]
    fn arabic_indic_counter_matches_wpt_ground_truth() {
        assert_eq!(to_arabic_indic(0), "\u{0660}"); // ٠
        assert_eq!(to_arabic_indic(1), "\u{0661}"); // ١
        assert_eq!(to_arabic_indic(9), "\u{0669}"); // ٩
        assert_eq!(to_arabic_indic(10), "\u{0661}\u{0660}"); // ١٠
        assert_eq!(to_arabic_indic(123), "\u{0661}\u{0662}\u{0663}"); // ١٢٣
    }

    /// R1701：ol start= 与 li value= 计数器语义（fixture 22 ol[start=3] type=A → C/D/J/K）。
    #[test]
    fn list_counter_respects_start_and_value_attrs() {
        let doc = parse_html("<ol start=\"3\"><li>a</li><li>b</li><li value=\"10\">c</li><li>d</li></ol>");
        assert_eq!(list_item_counter(&doc, li(&doc, 0)), 3); // C
        assert_eq!(list_item_counter(&doc, li(&doc, 1)), 4); // D
        assert_eq!(list_item_counter(&doc, li(&doc, 2)), 10); // J（value=10）
        assert_eq!(list_item_counter(&doc, li(&doc, 3)), 11); // K（从 10+1 继续）
    }

    /// R3743：负数 `start`/`value` 进入 list-item counter，供 @counter-style negative descriptor 使用。
    #[test]
    fn list_counter_allows_negative_start_and_value_attrs() {
        let doc = parse_html("<ol start=\"-2\"><li>a</li><li>b</li><li value=\"-5\">c</li><li>d</li></ol>");
        assert_eq!(list_item_counter(&doc, li(&doc, 0)), -2);
        assert_eq!(list_item_counter(&doc, li(&doc, 1)), -1);
        assert_eq!(list_item_counter(&doc, li(&doc, 2)), -5);
        assert_eq!(list_item_counter(&doc, li(&doc, 3)), -4);
    }

    /// 无 start=/value= 时等价 1-based 兄弟位置（向后兼容，R1701 前行为）。
    #[test]
    fn list_counter_default_is_one_based_position() {
        let doc = parse_html("<ol><li>a</li><li>b</li><li>c</li></ol>");
        assert_eq!(list_item_counter(&doc, li(&doc, 0)), 1);
        assert_eq!(list_item_counter(&doc, li(&doc, 1)), 2);
        assert_eq!(list_item_counter(&doc, li(&doc, 2)), 3);
    }

    /// li value= 在中间重置后续计数（value=5 后续 6/7）。
    #[test]
    fn list_counter_value_attr_resets_running_counter() {
        let doc = parse_html("<ol><li>a</li><li value=\"5\">b</li><li>c</li></ol>");
        assert_eq!(list_item_counter(&doc, li(&doc, 0)), 1);
        assert_eq!(list_item_counter(&doc, li(&doc, 1)), 5); // value=5
        assert_eq!(list_item_counter(&doc, li(&doc, 2)), 6); // 从 5+1 继续
    }

    /// R2392：构造最小 CounterStyleRule（仅 system + symbols）用于生成测试。
    fn cs_rule(
        system: zero_css_parser::ast::CounterSystem,
        symbols: &[&str],
    ) -> zero_css_parser::ast::CounterStyleRule {
        zero_css_parser::ast::CounterStyleRule {
            name: "test".to_string(),
            system,
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
            additive_symbols: Vec::new(),
            prefix: String::new(),
            suffix: ". ".to_string(),
            fallback: "decimal".to_string(),
            range: None,
            negative: ("-".to_string(), String::new()),
            pad: None,
        }
    }

    #[test]
    fn test_counter_style_cyclic() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Cyclic, &["a", "b", "c"]);
        assert_eq!(counter_style_body(&r, 1).unwrap(), "a");
        assert_eq!(counter_style_body(&r, 3).unwrap(), "c");
        assert_eq!(counter_style_body(&r, 4).unwrap(), "a"); // 循环回 a
        assert_eq!(counter_style_body(&r, 6).unwrap(), "c");
        // R2394：cyclic 表示任意整数（数学取模）；value 0 → syms[(-1)%3]=syms[2]="c"。
        assert_eq!(counter_style_body(&r, 0).unwrap(), "c");
        assert_eq!(counter_style_body(&r, -1).unwrap(), "b"); // (-2)%3=1 → "b"
    }

    #[test]
    fn test_counter_style_fixed() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Fixed(Some(5)), &["v", "w", "x"]);
        assert_eq!(counter_style_body(&r, 5).unwrap(), "v"); // first=5
        assert_eq!(counter_style_body(&r, 7).unwrap(), "x");
        assert!(counter_style_body(&r, 8).is_none(), "超出 symbols 范围走 fallback");
        assert!(counter_style_body(&r, 4).is_none(), "< first 应 None");
        // Fixed(None) 默认 first=1。
        let r2 = cs_rule(zero_css_parser::ast::CounterSystem::Fixed(None), &["a", "b"]);
        assert_eq!(counter_style_body(&r2, 1).unwrap(), "a");
        assert_eq!(counter_style_body(&r2, 2).unwrap(), "b");
    }

    #[test]
    fn test_counter_style_symbolic() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Symbolic, &["*"]);
        assert_eq!(counter_style_body(&r, 1).unwrap(), "*");
        assert_eq!(counter_style_body(&r, 2).unwrap(), "**");
        assert_eq!(counter_style_body(&r, 3).unwrap(), "***");
        // 多 symbol：idx 循环 + reps 递增。
        let r2 = cs_rule(zero_css_parser::ast::CounterSystem::Symbolic, &["*", "†"]);
        assert_eq!(counter_style_body(&r2, 1).unwrap(), "*");
        assert_eq!(counter_style_body(&r2, 2).unwrap(), "†");
        assert_eq!(counter_style_body(&r2, 3).unwrap(), "**");
        assert_eq!(counter_style_body(&r2, 4).unwrap(), "††");
    }

    #[test]
    fn test_counter_style_alphabetic() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Alphabetic, &["a", "b"]);
        assert_eq!(counter_style_body(&r, 1).unwrap(), "a");
        assert_eq!(counter_style_body(&r, 2).unwrap(), "b");
        assert_eq!(counter_style_body(&r, 3).unwrap(), "aa"); // 双射进位
        assert_eq!(counter_style_body(&r, 4).unwrap(), "ab");
        assert_eq!(counter_style_body(&r, 5).unwrap(), "ba");
        assert_eq!(counter_style_body(&r, 6).unwrap(), "bb");
    }

    #[test]
    fn test_counter_style_numeric() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Numeric, &["0", "1", "2"]);
        assert_eq!(counter_style_body(&r, 0).unwrap(), "0");
        assert_eq!(counter_style_body(&r, 1).unwrap(), "1");
        assert_eq!(counter_style_body(&r, 2).unwrap(), "2");
        assert_eq!(counter_style_body(&r, 3).unwrap(), "10"); // base-3 进位
        assert_eq!(counter_style_body(&r, 4).unwrap(), "11");
        assert_eq!(counter_style_body(&r, 6).unwrap(), "20");
    }

    #[test]
    /// R3743：`pad` 在生成 marker body 后补齐到最小宽度（对齐 descriptor-pad.html）。
    fn test_counter_style_pad_applies_to_body() {
        let mut r = cs_rule(zero_css_parser::ast::CounterSystem::Numeric, &["0", "1", "2"]);
        r.pad = Some((3, "0".to_string()));
        assert_eq!(counter_style_body(&r, 0).unwrap(), "000");
        assert_eq!(counter_style_body(&r, 1).unwrap(), "001");
        assert_eq!(counter_style_body(&r, 4).unwrap(), "011");

        let mut alphabetic = cs_rule(zero_css_parser::ast::CounterSystem::Alphabetic, &["a", "b"]);
        alphabetic.pad = Some((3, "o".to_string()));
        assert_eq!(counter_style_body(&alphabetic, 1).unwrap(), "ooa");
        assert_eq!(counter_style_body(&alphabetic, 3).unwrap(), "oaa");
    }

    #[test]
    /// R3743：负号包装和 pad 顺序对齐 descriptor-pad.html：padding 插入负号后、主体前。
    fn test_counter_style_negative_and_pad_match_wpt_order() {
        let mut r = cs_rule(zero_css_parser::ast::CounterSystem::Extends("decimal".to_string()), &[]);
        r.negative = ("(".to_string(), ")".to_string());
        r.pad = Some((3, "0".to_string()));
        assert_eq!(counter_style_body(&r, -2).unwrap(), "(2)");
        assert_eq!(counter_style_body(&r, 0).unwrap(), "000");
        assert_eq!(counter_style_body(&r, 1).unwrap(), "001");
    }

    #[test]
    /// R3743：显式 range 在 marker 格式化时生效，超出范围走 fallback。
    fn test_counter_style_range_applies_before_pad() {
        let mut r = cs_rule(
            zero_css_parser::ast::CounterSystem::Extends("upper-roman".to_string()),
            &[],
        );
        r.range = Some(vec![(i32::MIN, 5)]);
        r.pad = Some((3, "*".to_string()));
        assert_eq!(counter_style_body(&r, 4).unwrap(), "*IV");
        assert_eq!(counter_style_body(&r, 5).unwrap(), "**V");
        assert!(counter_style_body(&r, 0).is_none());
        assert!(counter_style_body(&r, 6).is_none());
    }

    #[test]
    /// R3744：`extends <custom-ident>` 使用被扩展样式的 body，但 range fallback 保留当前规则 affix。
    fn test_counter_style_custom_extends_resolves_base_body_and_own_affix() {
        let mut chapter = cs_rule(
            zero_css_parser::ast::CounterSystem::Extends("upper-roman".to_string()),
            &[],
        );
        chapter.name = "chapter".to_string();
        chapter.prefix = "Chapter ".to_string();
        chapter.range = Some(vec![(1, 5)]);

        let mut section = cs_rule(zero_css_parser::ast::CounterSystem::Extends("chapter".to_string()), &[]);
        section.name = "section".to_string();
        section.prefix = "Section ".to_string();
        section.range = Some(vec![(1, 6)]);

        let mut registry = std::collections::HashMap::new();
        registry.insert(chapter.name.clone(), chapter.clone());
        registry.insert(section.name.clone(), section.clone());

        assert_eq!(
            counter_style_marker_text(&section, 6, Some(&registry), None),
            "Section VI. "
        );
        assert_eq!(
            counter_style_marker_text(&section, 7, Some(&registry), None),
            "Section 7. "
        );
        assert_eq!(
            counter_style_marker_text(&chapter, 0, Some(&registry), None),
            "Chapter 0. "
        );
    }

    #[test]
    /// R3745：`system: extends disclosure-closed` uses the predefined directional symbol.
    fn test_counter_style_extends_disclosure_closed_uses_contextual_symbol() {
        let mut rule = cs_rule(
            zero_css_parser::ast::CounterSystem::Extends("disclosure-closed".to_string()),
            &[],
        );
        rule.suffix = ". ".to_string();

        let mut style = ComputedStyle::default();
        style.writing_mode = WritingModeValue::HorizontalTb;
        style.direction = DirectionValue::Rtl;
        assert_eq!(counter_style_marker_text(&rule, 1, None, Some(&style)), "◂ ");

        style.writing_mode = WritingModeValue::VerticalLr;
        style.direction = DirectionValue::Rtl;
        assert_eq!(counter_style_marker_text(&rule, 1, None, Some(&style)), "▴ ");
    }

    #[test]
    /// R2394：additive 系统 → 应用 defer（None，走 fallback）。parse-retain 见 parser 测试。
    fn test_counter_style_additive_deferred() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Additive, &["a"]);
        assert!(counter_style_body(&r, 1).is_none());
    }

    // ── R3835：inside marker 内容偏移 + §6.2 CJK/日/韩计数器 ──────────────

    /// §6.2 japanese-informal（ground-truth：WPT css3-counter-styles-042/043/044/045）。
    /// 1 前全省（10=十、100=百、1000=千）、无中间补零（101=百一）、0=〇、负前缀マイナス。
    #[test]
    fn test_r3835_cjk_japanese_informal() {
        assert_eq!(to_cjk_num(1, &JAPANESE_INFORMAL, false).unwrap(), "一");
        assert_eq!(to_cjk_num(9, &JAPANESE_INFORMAL, false).unwrap(), "九");
        assert_eq!(to_cjk_num(10, &JAPANESE_INFORMAL, false).unwrap(), "十");
        assert_eq!(to_cjk_num(11, &JAPANESE_INFORMAL, false).unwrap(), "十一");
        assert_eq!(to_cjk_num(100, &JAPANESE_INFORMAL, false).unwrap(), "百");
        assert_eq!(to_cjk_num(101, &JAPANESE_INFORMAL, false).unwrap(), "百一");
        assert_eq!(to_cjk_num(999, &JAPANESE_INFORMAL, false).unwrap(), "九百九十九");
        assert_eq!(to_cjk_num(1000, &JAPANESE_INFORMAL, false).unwrap(), "千");
        assert_eq!(to_cjk_num(9999, &JAPANESE_INFORMAL, false).unwrap(), "九千九百九十九");
        // 10000+ 按位逐字（chromium 语义，WPT 044）。
        assert_eq!(to_cjk_num(10000, &JAPANESE_INFORMAL, false).unwrap(), "一〇〇〇〇");
        assert_eq!(to_cjk_num(0, &JAPANESE_INFORMAL, false).unwrap(), "〇");
        assert_eq!(to_cjk_num(-11, &JAPANESE_INFORMAL, false).unwrap(), "マイナス十一");
    }

    /// §6.2 simp-chinese-formal：恒写 digit+unit（10=壹拾）、中间补零（101=壹佰零壹）、
    /// 越界 10000=一〇〇〇〇（WPT 076/078）。
    #[test]
    fn test_r3835_cjk_simp_chinese_formal() {
        assert_eq!(to_cjk_num(1, &SIMP_CHINESE_FORMAL, false).unwrap(), "壹");
        assert_eq!(to_cjk_num(10, &SIMP_CHINESE_FORMAL, false).unwrap(), "壹拾");
        assert_eq!(to_cjk_num(11, &SIMP_CHINESE_FORMAL, false).unwrap(), "壹拾壹");
        assert_eq!(to_cjk_num(101, &SIMP_CHINESE_FORMAL, false).unwrap(), "壹佰零壹");
        assert_eq!(to_cjk_num(222, &SIMP_CHINESE_FORMAL, false).unwrap(), "贰佰贰拾贰");
        assert_eq!(to_cjk_num(9999, &SIMP_CHINESE_FORMAL, false).unwrap(), "玖仟玖佰玖拾玖");
        assert_eq!(to_cjk_num(10000, &SIMP_CHINESE_FORMAL, false).unwrap(), "一〇〇〇〇");
        assert_eq!(to_cjk_num(-9, &SIMP_CHINESE_FORMAL, false).unwrap(), "负玖");
    }

    /// §6.2 korean-hanja-formal：恒写 digit+unit、无补零、range 1-9999（越界 → None 走
    /// decimal fallback，WPT 065 期望 10000="10000."）。
    #[test]
    fn test_r3835_cjk_korean_hanja_formal_range_fallback() {
        assert_eq!(to_cjk_num(9999, &KOREAN_HANJA_FORMAL, true).unwrap(), "九仟九百九拾九");
        assert_eq!(to_cjk_num(10, &KOREAN_HANJA_FORMAL, true).unwrap(), "壹拾");
        assert!(
            to_cjk_num(10000, &KOREAN_HANJA_FORMAL, true).is_none(),
            "korean range 1-9999 越界走 fallback"
        );
    }

    /// §6.1/§6.2 cyclic/alphabetic 符号循环 + 家族 suffix（WPT 201/204：1=子/甲、
    /// 13=一三?——否，cyclic 是 12/10 符号循环；katakana 11=ア+第11符号）。
    #[test]
    fn test_r3835_symbol_cycle_and_suffix() {
        assert_eq!(to_symbol_cycle(1, CJK_EARTHLY_BRANCH).unwrap(), "子");
        assert_eq!(to_symbol_cycle(12, CJK_EARTHLY_BRANCH).unwrap(), "亥");
        assert_eq!(to_symbol_cycle(13, CJK_EARTHLY_BRANCH).unwrap(), "子");
        assert_eq!(
            counter_suffix(&zero_css_parser::values::ListStyleTypeValue::JapaneseFormal),
            "、"
        );
        assert_eq!(
            counter_suffix(&zero_css_parser::values::ListStyleTypeValue::KoreanHangulFormal),
            ","
        );
        assert_eq!(
            counter_suffix(&zero_css_parser::values::ListStyleTypeValue::Decimal),
            "."
        );
    }
}
