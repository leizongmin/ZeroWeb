//! CSSOM grid 家族声明值校验与 canonical 序列化（wptfyi-css DC-2 grid 批次 G1）。
//!
//! [`crate::dom_bindings::css_style_declaration`] 的 CSSOM 写入通道
//!（setProperty / named setter）在存储前经 [`canonicalize_grid_decl`] 分派：
//! grid 家族属性按 CSS Grid §3-§7 文法解析，非法值按 CSSOM 语义**丢弃声明**
//!（spec dom-cssstyledeclaration-setproperty：invalid value → declaration
//! ignored），合法值回写 canonical 串（等价于浏览器的 declared-value
//! serialization）。非 grid 属性不受影响（原样存储）。
//!
//! 为何不复用 style-system 解析器全部家族：`parse_grid_line` 不支持
//! `span <custom-ident>` / `span <integer> <custom-ident>` 形
//!（GridLineValue 无 span+ident 变体，style-system 侧挂账），而语料
//! grid-*-start/end 与 grid-column/row/area 合法锚依赖该形——line 文法在
//! 本模块本地实现（含 span-ident），track-list / auto-flow / areas 三面
//! 仍复用 style-system 公开解析器（形式覆盖经其单测与语料双向锚定）。
//!
//! https://drafts.csswg.org/css-grid-1/#grid-shorthand
//! https://drafts.csswg.org/css-grid-2/#typedef-grid-line

/// grid 家族属性 → canonical 串；非法值返 `None`（调用方丢弃声明）。
/// 非 grid 家族属性调用方不进本分派（原样存储）。
pub(crate) fn canonicalize_grid_decl(prop: &str, value: &str) -> Option<String> {
    let value = value.trim();
    // CSS 全局关键字（css-cascade-4 wide-keywords）任意属性合法，ASCII ci 匹配
    // canonical 小写存储；否则 `grid-template-columns: inherit` 会被文法误拒。
    if ["inherit", "initial", "unset", "revert", "revert-layer"]
        .iter()
        .any(|k| value.eq_ignore_ascii_case(k))
    {
        return Some(value.to_ascii_lowercase());
    }
    // css-variables-1 §3：值含 var() 的声明 parse 期恒有效（token stream 原文
    // 存储，computed-value 期替换后才按属性文法重验）——grid 文法校验器不识别
    // var 文法，先放行。css-typed-om 语料 'Can set grid-* to var() references'
    // 钉面（v5b 9,872P 基线，G1 门引入 -4 回退）。
    // https://drafts.csswg.org/css-variables-1/#using-variables
    if value.contains("var(") {
        return Some(value.to_string());
    }
    match prop {
        "grid-auto-flow" => canonicalize_auto_flow(value),
        "grid-template-columns" | "grid-template-rows" => {
            match zero_style_system::property::parse::parse_grid_template_track_list(value)? {
                None => Some("none".to_string()),
                Some(list) => Some(list),
            }
        }
        "grid-auto-columns" | "grid-auto-rows" => {
            zero_style_system::property::parse::parse_grid_auto_track_list(value)?
        }
        "grid-template-areas" => match zero_style_system::property::parse::parse_grid_template_areas_value(value)? {
            None => Some("none".to_string()),
            Some(areas) => Some(areas),
        },
        "grid-row-start" | "grid-row-end" | "grid-column-start" | "grid-column-end" => parse_grid_line_cssom(value),
        "grid-row" | "grid-column" => {
            let (start, end) = parse_grid_line_pair_cssom(value)?;
            Some(grid_line_pair_canonical(&start, &end))
        }
        "grid-area" => {
            let (rs, cs, re, ce) = parse_grid_area_cssom(value)?;
            Some(grid_area_canonical(&rs, &cs, &re, &ce))
        }
        _ => Some(value.to_string()),
    }
}

/// 该属性是否 grid 家族（决定 setter 是否丢弃非法声明）。
pub(crate) fn is_grid_family_prop(prop: &str) -> bool {
    matches!(
        prop,
        "grid-auto-flow"
            | "grid-template-columns"
            | "grid-template-rows"
            | "grid-auto-columns"
            | "grid-auto-rows"
            | "grid-template-areas"
            | "grid-row-start"
            | "grid-row-end"
            | "grid-column-start"
            | "grid-column-end"
            | "grid-row"
            | "grid-column"
            | "grid-area"
    )
}

/// grid-auto-flow canonical 序列化（CSS Grid §3.5.3 简写序列化：row 关键字
/// 仅在 dense 同书时被省略——`row dense` → `dense`，`column dense` 全保）。
fn canonicalize_auto_flow(value: &str) -> Option<String> {
    let flow = zero_style_system::property::parse::parse_grid_auto_flow(value)?;
    Some(match flow {
        zero_style_system::property::types::GridAutoFlowValue::Row => "row".to_string(),
        zero_style_system::property::types::GridAutoFlowValue::Column => "column".to_string(),
        zero_style_system::property::types::GridAutoFlowValue::RowDense => "dense".to_string(),
        zero_style_system::property::types::GridAutoFlowValue::ColumnDense => "column dense".to_string(),
    })
}

/// CSSOM grid line 文法（`<grid-line>`，CSS Grid §6.1——本模块本地版，含
/// style-system 未覆盖的 span+ident 与 `<integer> <ident>` 组合形）：
/// `auto` / `<integer>`(≠0) / `<custom-ident>` / `<integer> <custom-ident>`
/// （任意序）/ `[ span && [ <integer> || <custom-ident> ]? ]` 组合。
/// 关键字 ASCII 大小写不敏感（canonical 小写），custom-ident 保原大小写。
/// 0 行号非法；span 后整数须为正；ident 不含 `/`、非数字、非 auto/span。
fn parse_grid_line_cssom(value: &str) -> Option<String> {
    let line = parse_grid_line_token(value)?;
    Some(grid_line_canonical(&line))
}

#[derive(Debug, Clone, PartialEq)]
enum CssomGridLine {
    Auto,
    Line(i64),
    /// `<integer> <custom-ident>`（无 span），canonical "N name"。
    LineName(i64, String),
    /// `span` / `span <integer>` / `span <custom-ident>` /
    /// `span <integer> <custom-ident>`（canonical 保 span 前缀）。
    Span(Option<i64>, Option<String>),
    Name(String),
}

fn grid_line_canonical(line: &CssomGridLine) -> String {
    match line {
        CssomGridLine::Auto => "auto".to_string(),
        CssomGridLine::Line(n) => n.to_string(),
        CssomGridLine::LineName(n, name) => format!("{n} {name}"),
        CssomGridLine::Span(count, name) => match (count, name) {
            (None, None) => "span".to_string(),
            (Some(c), None) => format!("span {c}"),
            (None, Some(n)) => format!("span {n}"),
            (Some(c), Some(n)) => format!("span {c} {n}"),
        },
        CssomGridLine::Name(n) => n.clone(),
    }
}

/// `<custom-ident>` 校验（grid-line 语境：排除 auto/span 关键字、数字开头、
/// 可解析为数值的 token 与 `/`）。
fn grid_line_custom_ident(token: &str) -> Option<String> {
    if token.eq_ignore_ascii_case("auto")
        || token.eq_ignore_ascii_case("span")
        || token.parse::<f64>().is_ok()
        || token.starts_with(|c: char| c.is_ascii_digit())
        || token.contains('/')
    {
        return None;
    }
    Some(token.to_string())
}

/// 单 `<grid-line>` 解析（CSS Grid §6.1）。
fn parse_grid_line_token(value: &str) -> Option<CssomGridLine> {
    let tokens: Vec<&str> = value.trim().split_ascii_whitespace().collect();
    match tokens.as_slice() {
        [] => None,
        [t] => {
            if t.eq_ignore_ascii_case("auto") {
                return Some(CssomGridLine::Auto);
            }
            if t.eq_ignore_ascii_case("span") {
                return Some(CssomGridLine::Span(None, None));
            }
            if let Ok(n) = t.parse::<i64>() {
                return (n != 0).then_some(CssomGridLine::Line(n));
            }
            grid_line_custom_ident(t).map(CssomGridLine::Name)
        }
        [first, rest @ ..] => {
            let span = first.eq_ignore_ascii_case("span");
            if !span && first.eq_ignore_ascii_case("auto") {
                return None;
            }
            let rest: &[&str] = if span { rest } else { tokens.as_slice() };
            match rest {
                [t] if span => {
                    if let Ok(n) = t.parse::<i64>() {
                        return (n > 0).then_some(CssomGridLine::Span(Some(n), None));
                    }
                    grid_line_custom_ident(t).map(|n| CssomGridLine::Span(None, Some(n)))
                }
                // `[ <integer> || <custom-ident> ]`（任意序，canonical N name）。
                [a, b] => {
                    let int_a = a.parse::<i64>().ok().filter(|&n| n != 0);
                    let int_b = b.parse::<i64>().ok().filter(|&n| n != 0);
                    let (n, name) = match (int_a, int_b) {
                        (Some(n), None) => (n, grid_line_custom_ident(b)?),
                        (None, Some(n)) => (n, grid_line_custom_ident(a)?),
                        _ => return None,
                    };
                    if span {
                        (n > 0).then_some(CssomGridLine::Span(Some(n), Some(name)))
                    } else {
                        Some(CssomGridLine::LineName(n, name))
                    }
                }
                _ => None,
            }
        }
    }
}

/// 顶层 `/` 切分（grid-line token 内不含 `/`，直接 split 即顶层）。
fn split_slash_components(value: &str) -> impl Iterator<Item = &str> {
    value.split('/').map(str::trim)
}

/// grid-column / grid-row 简写：`(start, end)`；无 `/` 时 end=auto。
/// canonical 序列化最小化（computed_style.rs Chromium oracle 同口径）：
/// start==end → 单值；end==auto 且 start 非 Name → 单值 start；否则 `start / end`。
fn parse_grid_line_pair_cssom(value: &str) -> Option<(CssomGridLine, CssomGridLine)> {
    let mut components = split_slash_components(value);
    let start = parse_grid_line_token(components.next()?)?;
    let end = match components.next() {
        Some(rest) => {
            if components.next().is_some() {
                return None;
            }
            parse_grid_line_token(rest)?
        }
        None => CssomGridLine::Auto,
    };
    Some((start, end))
}

fn grid_line_pair_canonical(start: &CssomGridLine, end: &CssomGridLine) -> String {
    let s = grid_line_canonical(start);
    let e = grid_line_canonical(end);
    if s == e || (matches!(end, CssomGridLine::Auto) && !matches!(start, CssomGridLine::Name(_))) {
        s
    } else {
        format!("{s} / {e}")
    }
}

/// grid-area 简写：1-4 个 `/` 分量，序 rs / cs / re / ce。缺省分量按
/// CSS Grid §7.4 填充：缺 end 以对应 start 的 custom-ident 填（其余
/// auto）；缺 column-start 以 row-start 的 custom-ident 填。
fn parse_grid_area_cssom(value: &str) -> Option<(CssomGridLine, CssomGridLine, CssomGridLine, CssomGridLine)> {
    let parts: Vec<&str> = split_slash_components(value).collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    let mut lines: Vec<CssomGridLine> = Vec::with_capacity(4);
    for part in &parts {
        lines.push(parse_grid_line_token(part)?);
    }
    while lines.len() < 4 {
        // 填充源：cs←rs、re←rs、ce←cs。
        let fill_from = match lines.len() {
            1 | 2 => 0,
            3 => 1,
            _ => break,
        };
        lines.push(match &lines[fill_from] {
            CssomGridLine::Name(n) => CssomGridLine::Name(n.clone()),
            _ => CssomGridLine::Auto,
        });
    }
    Some((lines[0].clone(), lines[1].clone(), lines[2].clone(), lines[3].clone()))
}

/// grid-area canonical（rs / cs / re / ce 序，尾部最小化）：ce==cs|auto 可
/// 弃，re==rs|auto 可弃，cs==auto|rs 可弃（语料锚 `1 / 2` → `1 / 2`、
/// `a / b / c / d` 全保、`auto` → `auto`）。
fn grid_area_canonical(rs: &CssomGridLine, cs: &CssomGridLine, re: &CssomGridLine, ce: &CssomGridLine) -> String {
    let end_droppable = |end: &CssomGridLine, start: &CssomGridLine| end == start || matches!(end, CssomGridLine::Auto);
    let keep = if !end_droppable(ce, cs) {
        4
    } else if !end_droppable(re, rs) {
        3
    } else if matches!(cs, CssomGridLine::Auto) || cs == rs {
        1
    } else {
        2
    };
    match keep {
        1 => grid_line_canonical(rs),
        2 => format!("{} / {}", grid_line_canonical(rs), grid_line_canonical(cs)),
        n => [rs, cs, re, ce][..n]
            .iter()
            .map(|l| grid_line_canonical(l))
            .collect::<Vec<_>>()
            .join(" / "),
    }
}

/// `in` 检查用 supported property 名单（CSSOM supported property names 的
/// 引擎支持面）。grid 家族全量 + 核心 box/typography/color 常用簇——语料
/// `prop in el.style` 动态检查按此应答；未列属性（冷门新特性）暂 `false`
/// （与缺 query 回调时的全 false 相比已是收窄缺口，扩面按语料驱动追加）。
pub(crate) const SUPPORTED_STYLE_PROPS: &[&str] = &[
    // grid 家族（本批次校验+序列化面）
    "grid-auto-flow",
    "grid-template-columns",
    "grid-template-rows",
    "grid-auto-columns",
    "grid-auto-rows",
    "grid-template-areas",
    "grid-template",
    "grid-row-start",
    "grid-row-end",
    "grid-column-start",
    "grid-column-end",
    "grid-row",
    "grid-column",
    "grid-area",
    // 核心簇（语料 in-style 动态检查实测面：grid-layout-properties 等页）
    "display",
    "position",
    "color",
    "opacity",
    "visibility",
    "overflow",
    "z-index",
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "margin",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "padding",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "border",
    "border-width",
    "border-style",
    "border-color",
    "background",
    "background-color",
    "background-image",
    "font",
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "line-height",
    "letter-spacing",
    "text-align",
    "text-decoration",
    "text-transform",
    "white-space",
    "vertical-align",
    "flex",
    "flex-direction",
    "flex-wrap",
    "flex-grow",
    "flex-shrink",
    "flex-basis",
    "justify-content",
    "align-items",
    "align-self",
    "align-content",
    "gap",
    "row-gap",
    "column-gap",
    "top",
    "right",
    "bottom",
    "left",
    "inset",
    "float",
    "clear",
    "content",
    "cursor",
    "transform",
    "transition",
    "animation",
    "box-sizing",
];

#[cfg(test)]
mod tests {
    use super::*;

    // css-variables-1 §3：值含 var() 的声明 parse 期恒有效（G1 门曾误拒，
    // css-typed-om 'Can set grid-* to var() references' 钉面 -4 回退）。
    // https://drafts.csswg.org/css-variables-1/#using-variables
    #[test]
    fn test_grid_decl_var_reference_accepted_at_parse_time() {
        for prop in [
            "grid-auto-flow",
            "grid-template-columns",
            "grid-template-rows",
            "grid-template-areas",
            "grid-row-start",
            "grid-area",
        ] {
            assert_eq!(
                canonicalize_grid_decl(prop, "var(--A)").as_deref(),
                Some("var(--A)"),
                "var() reference must be accepted for {prop}"
            );
        }
        // CSS-wide keyword 臂不回退；非 var 非法值仍拒（声明忽略语义保持）。
        assert_eq!(
            canonicalize_grid_decl("grid-auto-flow", "INHERIT").as_deref(),
            Some("inherit")
        );
        assert_eq!(canonicalize_grid_decl("grid-auto-flow", "nonsense"), None);
    }
}
