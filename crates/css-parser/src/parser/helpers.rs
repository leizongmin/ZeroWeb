//! CSS 解析器自由辅助函数（从 `mod.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 这些函数不依赖 `Parser` 的内部状态（非 `&self` 方法），按主题分组：
//! 容器查询条件解析、`@counter-style` 描述符解析、`@font-face` src 提取、`@page` 尺寸/外边距解析。
//! 原 `mod.rs` 内的调用方经 `use super::helpers::*;` 拉入；`pub(super)` 等价于原「parser 模块私有」语义。

use crate::ast::*;

/// 解析容器条件文本。
///
/// 支持格式如 `min-width: 400px`、`width > 300px`、`max-width: 800px`。
pub(super) fn parse_container_condition(text: &str) -> Option<ContainerCondition> {
    let text = text.trim();

    // 检查 size() 或 inline-size() 包装
    if let Some(inner) = text.strip_prefix("size(").and_then(|s| s.strip_suffix(')')) {
        return Some(ContainerCondition::Size(parse_size_condition(inner.trim())?));
    }
    if let Some(inner) = text.strip_prefix("inline-size(").and_then(|s| s.strip_suffix(')')) {
        return Some(ContainerCondition::InlineSize(parse_size_condition(inner.trim())?));
    }

    // 默认为 Size 条件（裸条件如 `min-width: 400px`）
    Some(ContainerCondition::Size(parse_size_condition(text)?))
}

/// 解析尺寸条件。
///
/// 支持格式如 `min-width: 400px`、`width > 300px`、`200px <= width <= 500px`。
fn parse_size_condition(text: &str) -> Option<ContainerSizeCondition> {
    let text = text.trim();

    // 尝试范围语法：`200px <= width <= 500px`
    // 找到 "<=" ... "<=" 模式
    let mut first_le = None;
    let mut second_le = None;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'=' {
            if first_le.is_none() {
                first_le = Some(i);
            } else {
                second_le = Some(i);
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    if let (Some(pos1), Some(pos2)) = (first_le, second_le) {
        let min_val = text[..pos1].trim().to_string();
        let feature = text[pos1 + 2..pos2].trim().to_string();
        let max_val = text[pos2 + 2..].trim().to_string();
        if !min_val.is_empty() && !feature.is_empty() && !max_val.is_empty() {
            return Some(ContainerSizeCondition {
                feature,
                value: String::new(),
                operator: None,
                range_min: Some(min_val),
                range_max: Some(max_val),
            });
        }
    }

    // 尝试冒号分隔格式：`min-width: 400px`
    if let Some(colon_pos) = text.find(':') {
        let feature = text[..colon_pos].trim().to_string();
        let value = text[colon_pos + 1..].trim().to_string();
        if feature.is_empty() || value.is_empty() {
            return None;
        }
        return Some(ContainerSizeCondition {
            feature,
            value,
            operator: None,
            range_min: None,
            range_max: None,
        });
    }

    // 尝试比较运算符格式：`width > 300px`、`width >= 300px`、`width < 300px`、`width <= 300px`
    for op in [">=", "<=", ">", "<"] {
        if let Some(op_pos) = text.find(op) {
            let feature = text[..op_pos].trim().to_string();
            let value = text[op_pos + op.len()..].trim().to_string();
            if feature.is_empty() || value.is_empty() {
                return None;
            }
            return Some(ContainerSizeCondition {
                feature,
                value,
                operator: Some(op.to_string()),
                range_min: None,
                range_max: None,
            });
        }
    }

    None
}

/// 去掉 CSS 字符串值两端的引号（单引号或双引号）。
pub(super) fn strip_css_quotes(s: &str) -> String {
    let s = s.trim();
    let bytes = s.as_bytes();
    if bytes.len() >= 2 && (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
        || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// 解析 `@counter-style` 的 `system` 描述符为类型化算法（CSS Counter Styles 3 §3.1.4）。
/// driving: R2392。`None` = 非法 system（at-rule 无效）。
pub(super) fn parse_counter_system(value: Option<&str>) -> Option<CounterSystem> {
    let v = value.unwrap_or("symbolic").trim(); // 缺省 symbolic
    let lower = v.to_ascii_lowercase();
    let mut parts = lower.split_whitespace();
    let head = parts.next()?;
    let system = match head {
        "cyclic" => CounterSystem::Cyclic,
        "fixed" => {
            // `fixed <integer>?`：首符号值（缺省 1）。
            let first = parts.next().and_then(|s| s.parse::<i32>().ok());
            CounterSystem::Fixed(first)
        }
        "symbolic" => CounterSystem::Symbolic,
        "alphabetic" => CounterSystem::Alphabetic,
        "numeric" => CounterSystem::Numeric,
        "additive" => CounterSystem::Additive,
        "extends" => {
            // `extends <counter-style-name>`：继承名（原始大小写，取未 lower 的下一段）。
            let ext = v.split_whitespace().nth(1)?.to_string();
            if ext.is_empty() {
                return None;
            }
            CounterSystem::Extends(ext)
        }
        _ => return None,
    };
    Some(system)
}

/// 切分 `@counter-style` 的 `symbols` 描述符值为独立符号列表（CSS Counter Styles 3 §3.1.5）。
/// 符号可为带引号串（`"a"` / `'◆'`）或裸标识/字形（`◆`），按空白分隔；逐个去引号。
/// driving: R2392。
pub(super) fn split_counter_symbols(value: &str) -> Vec<String> {
    value.split_whitespace().map(strip_css_quotes).collect()
}

/// 解析 `additive-symbols` 描述符（CSS Counter Styles 3 §3.1.8）。
///
/// 格式：逗号分隔的 `<integer> && <symbol>` 对，如 `6 \2685, 5 \2684, ...` 或
/// `3 "a", 2 "b"`。每对中整数与符号（引号串/裸字形）顺序可互换。结果按 weight 降序排序
/// （贪心分解算法所需）。任一对缺整数或符号 → 该对跳过；全无效返回空 Vec（上层据空判非法）。
/// driving: R2394 slice 2。
pub(super) fn parse_counter_additive_symbols(value: &str) -> Vec<(i32, String)> {
    let mut pairs: Vec<(i32, String)> = Vec::new();
    for part in value.split(',') {
        let tokens: Vec<&str> = part.split_whitespace().collect();
        if tokens.len() < 2 {
            continue;
        }
        // 整数与符号二元组：整数可能在首或次位置。
        let (weight, symbol) = match (tokens[0].parse::<i32>().ok(), tokens[1].parse::<i32>().ok()) {
            (Some(w), None) => (w, strip_css_quotes(tokens[1])),
            (None, Some(w)) => (w, strip_css_quotes(tokens[0])),
            _ => continue, // 两端都非整数 / 都为整数 → 非法对，跳过。
        };
        pairs.push((weight, symbol));
    }
    // 降序排序（贪心分解从最大 weight 起）。稳定排序保留同 weight 声明顺序。
    pairs.sort_by_key(|b| std::cmp::Reverse(b.0));
    pairs
}

/// 解析 `range` 描述符（CSS Counter Styles 3 §3.1.2）。
///
/// 格式：逗号分隔的 `[lower upper]` 对，每对两值，`infinite` → i32::{MIN,MAX}。
/// 如 `1 5`、`1 5, 10 20`、`infinite -1`。仅当所有对解析成功时返回 Some；任一畸形返回 None
/// （缺省 range 由系统默认决定，slice 2 不应用）。`auto` 关键字返回 None（走系统默认）。
/// driving: R2394 slice 2。
pub(super) fn parse_counter_range(value: &str) -> Option<Vec<(i32, i32)>> {
    let lower = value.to_ascii_lowercase();
    if lower.split_whitespace().eq(["auto"]) {
        return None;
    }
    let mut ranges = Vec::new();
    for part in lower.split(',') {
        let mut iter = part.split_whitespace();
        // lower 为 infinite → -∞（i32::MIN）；upper 为 infinite → +∞（i32::MAX）。
        let lo = parse_range_bound(iter.next()?, false)?;
        let hi = parse_range_bound(iter.next()?, true)?;
        ranges.push((lo, hi));
    }
    if ranges.is_empty() { None } else { Some(ranges) }
}

/// 解析单个 range 边界：`infinite` → 极值（lower→MIN，upper→MAX），否则十进制整数。
fn parse_range_bound(tok: &str, is_upper: bool) -> Option<i32> {
    match tok {
        "infinite" => Some(if is_upper { i32::MAX } else { i32::MIN }),
        _ => tok.parse::<i32>().ok(),
    }
}

/// 从 `src` 描述符值中提取所有 `url(...)` 内的 URL（按出现顺序，去引号）。
///
/// 支持 `url("X.woff")`、`url(X.woff)`、`url('X.woff')`，忽略 `format(...)` 等其他部分。
pub(super) fn extract_urls_from_src(src: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for source in split_top_level_src_items(src) {
        let source = source.trim();
        if source.len() < 4 || !source[..4].eq_ignore_ascii_case("url(") {
            continue;
        }
        let Some(close) = find_matching_paren(source, 3) else {
            continue;
        };
        let inner = source[4..close].trim();
        let tail = source[close + 1..].trim();
        if !src_url_tail_is_valid(tail) {
            continue;
        }
        if let Some(url) = crate::values::parse_extended_visual::parse_css_url_payload(inner) {
            urls.push(url);
        }
    }
    urls
}

fn split_top_level_src_items(src: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut start = 0usize;
    for (idx, ch) in src.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            ',' if depth == 0 => {
                parts.push(&src[start..idx]);
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&src[start..]);
    parts
}

fn find_matching_paren(s: &str, open_idx: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (idx, ch) in s.char_indices().skip_while(|(idx, _)| *idx < open_idx) {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
                if depth < 0 {
                    return None;
                }
            }
            _ => {}
        }
    }
    None
}

fn src_url_tail_is_valid(mut tail: &str) -> bool {
    while !tail.is_empty() {
        let trimmed = tail.trim_start();
        if trimmed.is_empty() {
            return true;
        }
        let Some(open) = trimmed.find('(') else {
            return false;
        };
        let name = trimmed[..open].trim();
        if name.is_empty() || name.chars().any(char::is_whitespace) {
            return false;
        }
        let Some(close) = find_matching_paren(trimmed, open) else {
            return false;
        };
        tail = &trimmed[close + 1..];
    }
    true
}

/// 解析 @page `size` 描述符为像素 `(width, height)`（@96dpi）。
///
/// 支持：
/// - 命名尺寸：`a3` / `a4` / `a5` / `b5` / `letter` / `legal` / `ledger`（portrait 朝向）
/// - 朝向修饰：`<name> portrait` / `<name> landscape`（或单独 `portrait` / `landscape`，默认 A4）
/// - 显式长度：`<length>`（正方页）或 `<length> <length>`（宽 高）
///
/// 其他值（`auto` / 未知关键字 / 相对单位）→ `None`（调用方回退默认 A4）。
pub fn resolve_page_size_px(size: &str) -> Option<(f32, f32)> {
    use crate::values::{LengthValue, parse_length};

    /// @96dpi 命名页尺寸 `(width, height)`，portrait 朝向（w ≤ h）。
    fn named(name: &str) -> Option<(f32, f32)> {
        const PX_PER_MM: f32 = 96.0 / 25.4;
        const PX_PER_IN: f32 = 96.0;
        match name {
            "a5" => Some((148.0 * PX_PER_MM, 210.0 * PX_PER_MM)),
            "a4" => Some((210.0 * PX_PER_MM, 297.0 * PX_PER_MM)),
            "a3" => Some((297.0 * PX_PER_MM, 420.0 * PX_PER_MM)),
            "b5" => Some((176.0 * PX_PER_MM, 250.0 * PX_PER_MM)),
            "letter" => Some((8.5 * PX_PER_IN, 11.0 * PX_PER_IN)),
            "legal" => Some((8.5 * PX_PER_IN, 14.0 * PX_PER_IN)),
            "ledger" => Some((11.0 * PX_PER_IN, 17.0 * PX_PER_IN)),
            _ => None,
        }
    }

    let lower = size.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();
    match parts.as_slice() {
        [one] => {
            if let Some(b) = named(one) {
                return Some(b);
            }
            if *one == "portrait" {
                return named("a4");
            }
            if *one == "landscape" {
                return named("a4").map(|(w, h)| (h, w));
            }
            // 单长度 → 正方页
            match parse_length(one) {
                Some(LengthValue::Px(p)) => Some((p as f32, p as f32)),
                _ => None,
            }
        }
        [a, b] => {
            let base = named(a).or_else(|| named(b));
            let orient_is_landscape = *a == "landscape" || *b == "landscape";
            if let Some((w, h)) = base {
                // named 返回 portrait（w ≤ h）；landscape 交换两轴。
                return Some(if orient_is_landscape { (h, w) } else { (w, h) });
            }
            // `<length> <length>`（宽 高）
            match (parse_length(a), parse_length(b)) {
                (Some(LengthValue::Px(w)), Some(LengthValue::Px(h))) => Some((w as f32, h as f32)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// 解析 @page `margin` 描述符为像素 `(top, right, bottom, left)`。
///
/// 同 CSS `margin` 1-4 值简写：1 值四边同；2 值 `(top bottom, right left)`；
/// 3 值 `(top, right left, bottom)`；4 值 `(top, right, bottom, left)`。仅绝对长度
/// （px/in/cm/mm/pt/pc），相对单位 / 未知 / 空串 → `None`。
pub fn resolve_page_margin_px(margin: &str) -> Option<(f32, f32, f32, f32)> {
    use crate::values::{LengthValue, parse_length};
    let to_px = |s: &str| match parse_length(s) {
        Some(LengthValue::Px(p)) => Some(p as f32),
        _ => None,
    };
    let parts: Vec<&str> = margin.split_whitespace().collect();
    match parts.as_slice() {
        [a] => to_px(a).map(|v| (v, v, v, v)),
        [a, b] => Some((to_px(a)?, to_px(b)?, to_px(a)?, to_px(b)?)),
        [a, b, c] => Some((to_px(a)?, to_px(b)?, to_px(c)?, to_px(b)?)),
        [a, b, c, d] => Some((to_px(a)?, to_px(b)?, to_px(c)?, to_px(d)?)),
        _ => None,
    }
}
