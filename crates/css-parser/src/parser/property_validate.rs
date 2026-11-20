//! `@property` 描述符校验门（css-properties-values-api-1 §the-syntax-descriptor /
//! §inherits-descriptor / §initial-value-descriptor / §at-property-rule）。
//!
//! 解析层此前宽容存储（R5386 注），但 CSSOM 物化与注册面需要规范有效门：无效规则
//! 整体丢弃（corpus at-property.html 'Rule not applied' / 'makes the @property rule
//! invalid' 钉面）。锚定 wptfyi-css 刀 ATP（2026-11-18）。
//! https://drafts.css-houdini.org/css-properties-values-api-1/

use crate::tokenizer::{Token, Tokenizer};

/// 已知 syntax 分型名（css-properties-values-api-1 §5 supported names 表）。
const KNOWN_TYPES: &[&str] = &[
    "color",
    "image",
    "url",
    "integer",
    "number",
    "length",
    "length-percentage",
    "percentage",
    "resolution",
    "string",
    "time",
    "angle",
    "transform-function",
    "transform-list",
    "custom-ident",
];

/// CSS 全局关键字禁止作 `<custom-ident>` syntax 分量（ci；§the-syntax-descriptor）。
const CSS_WIDE_KEYWORDS: &[&str] = &["default", "initial", "inherit", "unset", "revert", "revert-layer"];

/// 字体相对单位＝computationally dependent（§initial-value-descriptor；corpus
/// '3em' for <length> invalid 钉）。
const FONT_RELATIVE_UNITS: &[&str] = &["em", "rem", "ex", "ch", "cap", "ic", "lh", "rlh"];

fn tokens_of(text: &str) -> Vec<Token> {
    Tokenizer::new(text).collect_tokens()
}

/// syntax 声明值 → string 字面内文（外引号剥除；声明值须为单个 string token，
/// 缺引号 → None，corpus 'syntax: <color>' 缺引号钉）。CSSOM 读面/cssText 用
///（内容原文 whitespace 保形，corpus `' <color># '` 钉）。
pub fn syntax_string_content(decl_value: &str) -> Option<String> {
    let toks = tokens_of(decl_value);
    let mut content: Option<String> = None;
    for t in &toks {
        match t {
            Token::Whitespace => {}
            Token::String(s) if content.is_none() => content = Some(s.clone()),
            _ => return None,
        }
    }
    content
}

/// syntax 描述符整体校验：`syntax_string_content` 抽取 + 内容 `|` 分量文法门。
pub fn validate_syntax_declaration(decl_value: &str) -> Option<String> {
    let content = syntax_string_content(decl_value)?;
    if validate_syntax_components(&content) {
        Some(content)
    } else {
        None
    }
}

/// syntax 字符串内容文法：顶层 `|` 分段，段＝`*`（须独占）｜`<type>`(+`+`/`#` 乘子)｜
/// ident（非 CSS 全局关键字；段内禁空白/逗号）。首尾 `|` / 空段 → invalid。
pub fn validate_syntax_components(content: &str) -> bool {
    if content.trim().is_empty() {
        return false;
    }
    let is_universal = content.trim() == "*";
    let segs: Vec<&str> = content.split('|').collect();
    let mut seen = 0usize;
    for seg in &segs {
        let seg = seg.trim();
        if seg.is_empty() {
            return false; // 首尾/连续 `|`
        }
        seen += 1;
        if is_universal {
            continue;
        }
        if seg == "*" {
            return false; // universal 须独占（§the-syntax-descriptor）
        }
        if !validate_component(seg) {
            return false;
        }
    }
    !(is_universal && seen != 1)
}

/// 段内未转义空白/逗号检查（corpus `--escape-syntax` 的 `I\ dent` 转义空格 ident
/// 必须存活——`\x` 转义对的第二字符不算分隔）。
fn has_unescaped_sep(seg: &str) -> bool {
    let mut esc = false;
    for ch in seg.chars() {
        if esc {
            esc = false;
            continue;
        }
        match ch {
            '\\' => esc = true,
            c if c.is_whitespace() || c == ',' => return true,
            _ => {}
        }
    }
    false
}

fn validate_component(seg: &str) -> bool {
    if has_unescaped_sep(seg) {
        return false;
    }
    // 乘子位于尖括号外（css-properties-values-api-1 §the-syntax-descriptor 文法
    // `<syntax-component> <syntax-multiplier>?`——corpus `<color>#`/`<length>+` 钉形；
    // 乘子随分量可附 ident 字面量（`foo+`）。类型名无论带否乘子均须在支持表内
    //（`<foo>#` 未知名拒，§syntax-component 严格文法）。
    let (core, _mult) = match seg.chars().last() {
        Some(_c @ ('+' | '#')) if seg.len() > 1 => (&seg[..seg.len() - 1], true),
        _ => (seg, false),
    };
    if let Some(rest) = core.strip_prefix('<') {
        let Some(inner) = rest.strip_suffix('>') else {
            return false;
        };
        return KNOWN_TYPES.iter().any(|k| k.eq_ignore_ascii_case(inner));
    }
    !CSS_WIDE_KEYWORDS.iter().any(|k| k.eq_ignore_ascii_case(core))
}

/// `inherits` 描述符：仅 `true`/`false`（ci）；其余（none/0/1/引号串/calc）→ invalid
///（corpus test_descriptor('inherits', …, null) 钉面）。
pub fn validate_inherits_declaration(decl_value: &str) -> Option<bool> {
    let v = decl_value.trim();
    if v.eq_ignore_ascii_case("true") {
        Some(true)
    } else if v.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

/// `initial-value` 存在性与分型校验门：universal syntax 可缺省（§the-syntax-descriptor；
/// 有值时宽容受理）；非 universal 须存在且按 syntax 联合文法可解析 + computational
/// independence（禁 var() 与字体相对单位）。corpus 'Rule not applied' 钉面。
pub fn validate_initial_gate(syntax_content: &str, initial: Option<&str>) -> bool {
    let universal = syntax_content.trim() == "*";
    match initial {
        None => universal,
        Some(text) => {
            // var() 引用禁（§initial-value-descriptor computationally independent
            //——universal 形同禁，corpus `Attribute 'initial-value' makes the
            // @property rule invalid for [var(--x)]` 钉）。
            if text.to_ascii_lowercase().contains("var(") {
                return false;
            }
            if universal {
                return true;
            }
            let toks = tokens_of(text);
            // 顶层逗号门（非 `#` 语法拒列表值）——只数函数括号**外**的逗号：
            // `rgb(1, 2, 3)`/`calc(10%, 10px)` 括号内逗号是函数参数
            //（corpus 'Rule applied [<color>, rgb(1, 2, 3), false]' 钉），顶层
            // `1px, 2px` 列表形仍拒（§initial-value-descriptor）。
            if count_top_level_commas(&toks) > 0 && !is_comma_list_syntax(syntax_content) {
                return false;
            }
            syntax_content.split('|').any(|seg| {
                let seg = seg.trim();
                if seg == "*" {
                    return true;
                }
                validate_initial_for_component(seg, &toks, text)
            })
        }
    }
}

fn is_comma_list_syntax(syntax: &str) -> bool {
    syntax.split('|').any(|seg| {
        let seg = seg.trim();
        seg.len() > 1 && seg.ends_with('#') && seg.as_bytes()[seg.len() - 2] == b'>'
    })
}

/// 函数括号外顶层逗号计数（`rgb(1, 2, 3)` 函数内逗号不计——函数参数属值内部
/// 文法；`Function`/`LParen` 增深、`RParen` 减深，深度 0 层的 `Comma` 才计数）。
fn count_top_level_commas(toks: &[Token]) -> usize {
    let mut depth = 0i32;
    let mut n = 0usize;
    for t in toks {
        match t {
            Token::Function(_) | Token::LParen => depth += 1,
            Token::RParen => depth -= 1,
            Token::Comma if depth <= 0 => n += 1,
            _ => {}
        }
    }
    n
}

fn is_font_relative_dim(t: &Token) -> bool {
    matches!(t, Token::Dimension(_, u)
        if FONT_RELATIVE_UNITS.iter().any(|k| k.eq_ignore_ascii_case(u)))
}

fn dim_unit(t: &Token) -> Option<&str> {
    match t {
        Token::Dimension(_, u) => Some(u.as_str()),
        _ => None,
    }
}

fn unit_in(t: &Token, units: &[&str]) -> bool {
    dim_unit(t).map_or(false, |u| units.iter().any(|k| k.eq_ignore_ascii_case(u)))
}

fn balanced_func_shape(toks: &[Token]) -> bool {
    let mut depth = 0i32;
    let mut saw_fn = false;
    for t in toks {
        match t {
            Token::Function(_) => {
                depth += 1;
                saw_fn = true;
            }
            Token::LParen => depth += 1,
            Token::RParen => depth -= 1,
            _ => {}
        }
    }
    saw_fn && depth == 0
}

/// 函数形括号平衡（`calc(10px)` 的 RParen 属闭合偶对——all-token 臂按整函数
/// 许可，修复前 RParen 落单拒整值＝`<length>` initial `calc(10px)` 拒形根因，
/// zz-atp-probe6 zzb2 锚定；值面可解析性仍由 engine 注册门 strict 分型裁决）。
fn balanced_function_form(toks: &[Token]) -> bool {
    let mut depth = 0i32;
    for t in toks {
        match t {
            Token::Function(_) | Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn validate_initial_for_component(seg: &str, toks: &[Token], text: &str) -> bool {
    // 乘子位于尖括号外（同 validate_component 文法注记）。
    let (core, mult_ch) = match seg.chars().last() {
        Some(c @ ('+' | '#')) if seg.len() > 1 => (&seg[..seg.len() - 1], c),
        _ => (seg, ' '),
    };
    let (name, mult) = if let Some(rest) = core.strip_prefix('<') {
        let Some(inner) = rest.strip_suffix('>') else {
            return false;
        };
        (inner, mult_ch)
    } else {
        // ident 分量：initial 须为单个 ident token 且与分量字面量（解码后）全等
        //（css-properties-values-api-1 §initial-value-descriptor——corpus
        // `gandalf`/`grey` 拒钉；解码归一化经 tokenizer 往返）。
        // https://drafts.css-houdini.org/css-properties-values-api-1/
        return match (tokens_of(seg).as_slice(), toks) {
            ([Token::Ident(want)], [Token::Ident(got)]) => want == got,
            _ => false,
        };
    };
    let non_ws: Vec<&Token> = toks.iter().filter(|t| !matches!(t, Token::Whitespace)).collect();
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "string" => {
            // `+`＝空白分隔串列表；`#`＝逗号分隔串列表；单值＝单串。
            let strings = non_ws.iter().filter(|t| matches!(t, Token::String(_))).count();
            let others = non_ws
                .iter()
                .filter(|t| !matches!(t, Token::String(_)) && !matches!(t, Token::Comma))
                .count();
            others == 0
                && strings > 0
                && (mult != '#' || non_ws.iter().any(|t| matches!(t, Token::Comma)) || strings == 1)
                && (mult != '+' || strings >= 1)
        }
        "color" => {
            non_ws.iter().any(|t| matches!(t, Token::Hash(_)))
                || matches!(non_ws.first(), Some(Token::Ident(s))
                if crate::values::named_colors::parse_named_color(s).is_some())
                || balanced_func_shape(&non_ws.iter().map(|t| (*t).clone()).collect::<Vec<_>>())
        }
        "url" | "image" => {
            matches!(non_ws.first(), Some(Token::Url(_)))
                || (lower == "image" && balanced_func_shape(&non_ws.iter().map(|t| (*t).clone()).collect::<Vec<_>>()))
        }
        "length" => {
            (non_ws
                .iter()
                .all(|t| matches!(t, Token::Dimension(_, _)) || matches!(t, Token::Number(0.0, _)))
                || balanced_function_form(toks))
                && non_ws.iter().all(|t| !is_font_relative_dim(t))
                && non_ws.iter().all(|t| !matches!(t, Token::Percentage(_)))
                && !non_ws.is_empty()
        }
        "percentage" => matches!(non_ws[..], [Token::Percentage(_)]) || balanced_function_form(toks),
        "length-percentage" => !non_ws.iter().any(|t| is_font_relative_dim(t)) && !non_ws.is_empty(),
        "angle" => {
            (non_ws
                .iter()
                .all(|t| unit_in(t, &["deg", "grad", "rad", "turn"]) || matches!(t, Token::Number(0.0, _)))
                || balanced_function_form(toks))
                && !non_ws.is_empty()
        }
        "time" => {
            (non_ws.iter().all(|t| unit_in(t, &["s", "ms"])) || balanced_function_form(toks)) && !non_ws.is_empty()
        }
        "resolution" => {
            (non_ws.iter().all(|t| unit_in(t, &["dpi", "dppx", "dpcm"])) || balanced_function_form(toks))
                && !non_ws.is_empty()
        }
        "integer" => matches!(non_ws[..], [Token::Number(_, true)]) || balanced_function_form(toks),
        "number" => matches!(non_ws[..], [Token::Number(_, _)]) || balanced_function_form(toks),
        "transform-function" => {
            balanced_func_shape(&non_ws.iter().map(|t| (*t).clone()).collect::<Vec<_>>())
            // 字体相对单位 computationally dependent（§initial-value-descriptor
            //——corpus `translateX(1em)`/`translateY(1lh)` 拒钉；token 扫描含函数
            // 参数内层）。
            && !toks.iter().any(is_font_relative_dim)
        }
        "transform-list" => {
            text.split_whitespace().count() >= 1
                && toks.iter().any(|t| matches!(t, Token::Function(_)))
                && !toks.iter().any(is_font_relative_dim)
        }
        "custom-ident" => matches!(non_ws[..], [Token::Ident(_)]),
        _ => true, // 未知段名（ident 分量已过 syntax 门）宽容受理
    }
}
