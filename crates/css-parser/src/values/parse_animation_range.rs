//! css-scroll-animations-1 §view-timelines-ranges：`animation-range-start` /
//! `animation-range-end` 写面文法 + canonical 指定串化 + 计算值串化助手。
//!
//! 文法（side=Start 省略 0%、side=End 省略 100%，css-scroll-animations-1
//! §animation-range-start/§animation-range-end propdef 表）：
//! `normal | [ <timeline-range-name> <length-percentage>? ]#`
//! 名必须在偏移前（`50% exit` 拒）；`normal` 独占整值（不得混列表/偏移）；
//! 单段至多一名一偏移。驱动语料：css-animations/parsing animation-range-*
//! 八页（wptfyi-css DC-2 桶 animation-range 刀）。
//!
//! canonical 规则（valid 页实证）：
//! 1. 裸数偏移补 `px`（`0` → `0px`）
//! 2. calc 形经 [`fold_math_specified`] canonical（percent-first 排序 +
//!    同族折叠保壳：`calc(10px + 10%)` → `calc(10% + 10px)`、
//!    `calc(10% - 10%)` → `calc(0%)`）
//! 3. 省略：Start 侧偏移 `0%`、End 侧偏移 `100%` 时剥偏移（`cover 0%` →
//!    `cover`；`cover 0px` 不剥）
//! 4. 非文法形 parse 返 `None`，写面整拒（invalid 页）；fold 返 `None`
//!    保留原文零漂移。

use super::parse_rule_color::split_top_level_commas;
use super::serialize_math::{fold_math_specified, MathFoldKind};

/// range 侧别（省略规则随侧别不同：Start 省略 0%、End 省略 100%）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationRangeSide {
    /// animation-range-start。
    Start,
    /// animation-range-end。
    End,
}

/// timeline range 名（css-scroll-animations-1 §timeline-range）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationRangeName {
    /// cover。
    Cover,
    /// contain。
    Contain,
    /// entry。
    Entry,
    /// exit。
    Exit,
    /// entry-crossing。
    EntryCrossing,
    /// exit-crossing。
    ExitCrossing,
}

impl AnimationRangeName {
    /// 名解析（ASCII case-insensitive）。
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cover" => Some(Self::Cover),
            "contain" => Some(Self::Contain),
            "entry" => Some(Self::Entry),
            "exit" => Some(Self::Exit),
            "entry-crossing" => Some(Self::EntryCrossing),
            "exit-crossing" => Some(Self::ExitCrossing),
            _ => None,
        }
    }

    /// canonical 名文本。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Contain => "contain",
            Self::Entry => "entry",
            Self::Exit => "exit",
            Self::EntryCrossing => "entry-crossing",
            Self::ExitCrossing => "exit-crossing",
        }
    }
}

/// 单段 range 项形态（`normal` / `名 [偏移]` / 裸偏移）。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationRangeShape {
    /// normal（独占）。
    Normal,
    /// 名（+ canonical 偏移文本，`None` = 未给偏移）。
    Name(AnimationRangeName, Option<String>),
    /// 裸偏移（canonical 文本）。
    Offset(String),
}

/// 单段 range 项。
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationRangeItem {
    /// 形态。
    pub shape: AnimationRangeShape,
}

/// 解析单侧整值（顶层逗号列表，逐段文法裁决）。
///
/// 返回 `None` = 文法 invalid（写面整拒）；`Some(items)` = 接受
///（canonical 文本由 [`animation_range_side_to_css`] 产出）。
pub fn parse_animation_range_side(v: &str, side: AnimationRangeSide) -> Option<Vec<AnimationRangeItem>> {
    let _ = side;
    let segs = split_top_level_commas(v.trim())?;
    let mut items = Vec::with_capacity(segs.len());
    let mut saw_normal = false;
    for seg in segs {
        let item = parse_single_range_item(seg)?;
        if matches!(item.shape, AnimationRangeShape::Normal) {
            saw_normal = true;
        }
        items.push(item);
    }
    // normal 独占：不得与任何其他段/偏移混排（'normal 10px'/'normal, cover' 拒）。
    if saw_normal && items.len() > 1 {
        return None;
    }
    Some(items)
}

/// 解析单段（不含顶层逗号）。
fn parse_single_range_item(seg: &str) -> Option<AnimationRangeItem> {
    let toks = split_ws_outside_parens(seg.trim());
    if toks.is_empty() {
        return None;
    }
    let lower: Vec<String> = toks.iter().map(|t| t.to_ascii_lowercase()).collect();
    match lower.len() {
        1 => {
            let one = lower[0].as_str();
            if one == "normal" {
                return Some(AnimationRangeItem { shape: AnimationRangeShape::Normal });
            }
            if let Some(name) = AnimationRangeName::parse(one) {
                return Some(AnimationRangeItem { shape: AnimationRangeShape::Name(name, None) });
            }
            let offset = canonical_offset(one)?;
            Some(AnimationRangeItem { shape: AnimationRangeShape::Offset(offset) })
        }
        2 => {
            // 名必须在偏移前（'50% exit' 拒——首 token 非名即整段拒）。
            let name = AnimationRangeName::parse(&lower[0])?;
            let offset = canonical_offset(&lower[1])?;
            Some(AnimationRangeItem { shape: AnimationRangeShape::Name(name, Some(offset)) })
        }
        _ => None,
    }
}

/// 偏移 canonical 文法裁决 + 串化：裸数补 `px`；calc 形走
/// [`fold_math_specified`]（Length 槽：percent-first 排序 + 同族折叠保壳）。
fn canonical_offset(tok: &str) -> Option<String> {
    if tok.starts_with("calc(") {
        if let Some(folded) = fold_math_specified(tok, MathFoldKind::Length) {
            return Some(folded);
        }
        // sign 树形乘积：fold_math_specified 拒 sign 树（font-size-adjust 臂
        // 同挂账注记）——本槽 narrow 局部重排兜底：顶层单 `*` 积、恰一裸
        // % 叶时 % 移首（animation-range-parsing valid 页
        // `calc(sign(100em - 1px) * 1%)` → `calc(1% * sign(100em - 1px))`
        // 钉面）。其余形返原文保底（接受面不丢）。
        // FIXME: css-values 通用 percent-first 乘积排序挂账（css-parser
        // calc canonical 域，折叠机制统一后收编本兜底）。
        return Some(reorder_percent_product_front(tok).unwrap_or_else(|| tok.to_string()));
    }
    let (num, unit) = split_number_unit(tok)?;
    let value: f64 = num.parse().ok()?;
    let unit = if unit.is_empty() { "px" } else { unit };
    if !is_length_percentage_unit(unit) {
        return None;
    }
    Some(format!("{}{}", fmt_num(value), unit))
}

/// 顶层单 `*` 乘积、恰一裸 % 叶的 percent 移首（窄形——不递归不多项）。
/// `calc(` 包裹与尾 `)` 剥离；实参 span 无嵌套 `(` 时才适用。
fn reorder_percent_product_front(v: &str) -> Option<String> {
    let inner = v.strip_prefix("calc(")?.strip_suffix(')')?;
    if inner.contains('(') {
        // 嵌套函数/括号形不适用（sign 树含内层括号——需按平衡深度切顶层积）。
        let depth = inner.chars().fold(0i32, |d, c| d + if c == '(' { 1 } else if c == ')' { -1 } else { 0 });
        if depth != 0 {
            return None;
        }
    }
    // 顶层（深度 0）单 `*` 切分。
    let mut depth = 0i32;
    let mut star = None;
    for (i, c) in inner.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '*' if depth == 0 => {
                if star.is_some() {
                    return None; // 多项积不适用
                }
                star = Some(i);
            }
            _ => {}
        }
    }
    let star = star?;
    let left = inner[..star].trim();
    let right = inner[star + 1..].trim();
    let is_bare_percent = |s: &str| -> bool {
        matches!(split_number_unit(s), Some((_, "%"))) && !s.starts_with("calc(")
    };
    if is_bare_percent(right) && !is_bare_percent(left) {
        Some(format!("calc({right} * {left})"))
    } else if is_bare_percent(left) && !is_bare_percent(right) {
        Some(format!("calc({left} * {right})"))
    } else {
        None
    }
}

/// 数+单位拆分（`-3.14%` → `("-3.14", "%")`；`0` → `("0", "")`）。
/// 指数仅在后随 [sign]数字 时消费（`1e3px` 数=`1e3`；`1em` 数=`1` 单位=`em`
/// ——'e' 贪吃会吞单位首字母，须状态机扫描）。非数前缀返 `None`。
fn split_number_unit(tok: &str) -> Option<(&str, &str)> {
    let b = tok.as_bytes();
    let mut i = 0usize;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    // 无整数亦无小数位：非数形（ident/裸符号）。
    if i == int_start {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        if j < b.len() && b[j].is_ascii_digit() {
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            i = j;
        }
    }
    let (num, unit) = tok.split_at(i);
    if num.parse::<f64>().is_err() {
        return None;
    }
    if !unit.is_empty() && !(unit.ends_with('%') || unit.bytes().all(|c| c.is_ascii_alphabetic())) {
        return None;
    }
    Some((num, unit))
}

/// `<length-percentage>` 单位族（css-values-4 length/percentage 全集）。
fn is_length_percentage_unit(unit: &str) -> bool {
    matches!(
        unit,
        "%" | "px" | "em" | "rem" | "ex" | "ch" | "cm" | "mm" | "in" | "pt" | "pc" | "q"
            | "vw" | "vh" | "vmin" | "vmax"
    )
}

/// canonical 数文本（整值无小数点；小数走 f64 最短 roundtrip Display）。
pub(crate) fn fmt_num(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

/// 括号感知空白切分（calc() 内空白不切）。
fn split_ws_outside_parens(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = None;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                if start.is_none() {
                    start = Some(i);
                }
            }
            ')' => depth -= 1,
            c if c.is_ascii_whitespace() && depth == 0 => {
                if let Some(st) = start.take() {
                    out.push(&s[st..i]);
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
    }
    if let Some(st) = start {
        out.push(&s[st..]);
    }
    out
}

/// 单侧 canonical 指定串化：逐段 canonical + 侧别省略 + `", "` 连接。
/// 空 items 输出空串（调用方不产空声明）。
pub fn animation_range_side_to_css(items: &[AnimationRangeItem], side: AnimationRangeSide) -> String {
    let parts: Vec<String> = items.iter().map(|it| item_to_css(&it.shape, side)).collect();
    parts.join(", ")
}

/// 单项 canonical 文本（含侧别省略）。
fn item_to_css(shape: &AnimationRangeShape, side: AnimationRangeSide) -> String {
    match shape {
        AnimationRangeShape::Normal => "normal".to_string(),
        AnimationRangeShape::Name(name, None) => name.as_str().to_string(),
        AnimationRangeShape::Name(name, Some(offset)) => {
            if range_offset_elides(offset, side) {
                name.as_str().to_string()
            } else {
                format!("{} {}", name.as_str(), offset)
            }
        }
        AnimationRangeShape::Offset(offset) => offset.clone(),
    }
}

/// 侧别省略判定：Start 侧 `0%`、End 侧 `100%` 剥偏移（css-scroll-animations-1
/// propdef 缺省偏移——Start 0%、End 100%；px 偏移不剥）。
pub fn range_offset_elides(offset: &str, side: AnimationRangeSide) -> bool {
    match side {
        AnimationRangeSide::Start => offset == "0%",
        AnimationRangeSide::End => offset == "100%",
    }
}

/// 写面 fold 入口（[`super::serialize_math::fold_decl_specified`] 分派臂）：
/// parse 成功 → canonical 文本；失败 → `None` 保留原文零漂移。
pub fn fold_animation_range_specified(v: &str, side: AnimationRangeSide) -> Option<String> {
    let items = parse_animation_range_side(v, side)?;
    Some(animation_range_side_to_css(&items, side))
}
