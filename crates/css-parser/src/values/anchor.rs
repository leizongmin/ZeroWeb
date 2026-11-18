//! anchor() / anchor-size() 声明值 canonical 序列化（css-anchor-1）。
//!
//! spec：`anchor() = anchor(<anchor-element>?, <anchor-side>?, <anchor-fallback>?)`
//! （[css-anchor-1 §3](https://drafts.csswg.org/css-anchor-1/#anchor)）、
//! `anchor-size() = anchor-size(<anchor-element>?, <anchor-size>?, <anchor-size-fallback>?)`
//! （[css-anchor-1 §4](https://drafts.csswg.org/css-anchor-1/#anchor-size)）。
//! `<anchor-element>` 是 `<dashed-ident>`，与 side/size 关键字在文法上可区分，上游接受
//! 两种参数序解析（WPT css-anchor-1 parse-valid 断言），canonical serialization 固定
//! 锚元素在前。本模块对**声明值字符串**做函数级参数换位：inline style 写入路径
//! （engine `js_dom_bridge` polyfill 与 `dom_bindings::css_style_declaration` native）
//! 存储前调用归一，使 `e.style` 读回 canonical 形态。自定义属性值（`--*`）按
//! spec 语义原样保留，调用方负责跳过。

/// 判定 CSS ident 续字符（函数名左边界检查——`--anchor-size(...)` 这类以 ident
/// 字符接续的名不是本函数，跳过换位）。
fn is_ident_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'
}

/// 在 `s[from..]` 找下一个 anchor()/anchor-size() 函数的实参区间。
/// 返回 `(区间内容起, 区间内容止)`（不含括号，闭区间语义配合切片）；无匹配返
/// `None`。函数名 ASCII 大小写不敏感（CSS keyword 语义），左边界非 ident 续字符。
fn find_anchor_function(s: &str, from: usize) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'a' || bytes[i] == b'A' {
            let rest = &s[i..];
            let matched_len = if rest.len() >= 11 && rest.as_bytes()[..11].eq_ignore_ascii_case(b"anchor-size") {
                11
            } else if rest.len() >= 6 && rest.as_bytes()[..6].eq_ignore_ascii_case(b"anchor") {
                6
            } else {
                0
            };
            if matched_len > 0 {
                let left_ok = i == 0 || !is_ident_continue(bytes[i - 1]);
                let after = rest[matched_len..].trim_start();
                if left_ok && after.starts_with('(') {
                    // 平衡括号找闭括号（实参内嵌套 calc()/anchor() 已覆盖；CSS 字符串
                    // 不在 anchor 实参文法内，不引入字符串感知）。
                    let paren_off = open_paren_index(rest, matched_len);
                    let mut depth = 1usize;
                    let mut j = i + paren_off + 1;
                    while j < bytes.len() && depth > 0 {
                        match bytes[j] {
                            b'(' => depth += 1,
                            b')' => depth -= 1,
                            _ => {}
                        }
                        j += 1;
                    }
                    if depth == 0 {
                        return Some((i + paren_off + 1, j - 1));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// 函数名起 `rest[name_len..]` 后首个非空白字符（即 `(`）的区内偏移。
fn open_paren_index(rest: &str, name_len: usize) -> usize {
    name_len + (rest[name_len..].len() - rest[name_len..].trim_start().len())
}

/// 括号感知的顶层逗号分块：返回块内容切片与块间逗号分隔符切片（`seps[k]` 位于
/// `chunks[k]` 与 `chunks[k+1]` 之间）。块内嵌套括号中的逗号不分块。
fn split_top_level_commas(s: &str) -> (Vec<&str>, Vec<&str>) {
    let mut chunks = Vec::new();
    let mut seps = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (idx, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                chunks.push(&s[start..idx]);
                seps.push(",");
                start = idx + 1;
            }
            _ => {}
        }
    }
    chunks.push(&s[start..]);
    (chunks, seps)
}

/// 括号感知的顶层空白分词：`calc(50% + 1px)` 计一个词。
fn split_top_level_tokens(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (idx, ch) in s.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                start = start.or(Some(idx));
            }
            ')' => {
                depth -= 1;
            }
            c if c.is_whitespace() && depth == 0 => {
                if let Some(begin) = start.take() {
                    tokens.push(&s[begin..idx]);
                }
            }
            _ => {
                start = start.or(Some(idx));
            }
        }
    }
    if let Some(begin) = start {
        tokens.push(&s[begin..]);
    }
    tokens
}

/// 单个实参首块 canonical 换位：恰两词且**末词是 dashed-ident、首词不是**（锚元素
/// 在后）→ 换为锚元素在前；其余形态原样保留（已 canonical / 单词 / 非法均不动）。
fn canonicalize_arg_chunk(chunk: &str) -> String {
    let tokens = split_top_level_tokens(chunk);
    if tokens.len() == 2 && !tokens[0].starts_with("--") && tokens[1].starts_with("--") {
        return format!("{} {}", tokens[1], tokens[0]);
    }
    chunk.to_string()
}

/// anchor()/anchor-size() 单调用实参 canonical 归一（CalcParser anchor 叶
/// parse 期消费）：顶层逗号分块 ≤2、`anchor()` 空参拒（`anchor-size()` 隐式
/// 空参合法）；首块参数序换位（锚元素 dashed-ident 前置）；fallback 块走
/// [`canonicalize_fallback_chunk`]。非法形返 `None`（调用方 parse 拒）。
pub fn canonicalize_anchor_call_args(seg: &str, is_size: bool) -> Option<String> {
    let (chunks, seps) = split_top_level_commas(seg);
    if chunks.len() > 2 {
        return None;
    }
    let head = chunks[0].trim();
    if head.is_empty() && !is_size {
        // anchor() 空参无语料收（anchor-size-parse-valid implicit 段仅
        // anchor-size 隐式形）——parse 拒。
        return None;
    }
    let mut rebuilt = canonicalize_arg_chunk(head);
    if let Some(fb) = chunks.get(1) {
        rebuilt.push_str(seps[0]);
        rebuilt.push(' ');
        rebuilt.push_str(&canonicalize_fallback_chunk(fb));
    }
    Some(rebuilt)
}

/// fallback 块 canonical（css-anchor-1 §3/§4 fallback `<length-percentage>`
/// 位）：无单位数字补 px（type 归一——`anchor(--foo left, 0)` →
/// `anchor(--foo left, 0px)`，anchor-parse-valid 语料 canonical 钉面）；
/// % /维度/calc/嵌套调用保形（嵌套 anchor 序换位递归）。
fn canonicalize_fallback_chunk(chunk: &str) -> String {
    let c = chunk.trim();
    if !c.is_empty() && c.parse::<f64>().is_ok() {
        format!("{}px", c)
    } else {
        canonicalize_anchor_functions(c)
    }
}

/// Canonicalize anchor()/anchor-size() argument order in a declared CSS value
/// （css-anchor-1 parse-valid canonical serialization——WPT
/// `css/css-anchor-position/anchor-size-parse-valid.html` 断言形态：
/// `anchor-size(width --foo)` → `anchor-size(--foo width)`，锚元素 dashed-ident
/// 固定在前）。comma 后 fallback 块与嵌套函数递归处理；函数名、非函数文本与
/// 空白结构原样保留；已 canonical 输入幂等零改动。**自定义属性值调用方须跳过
/// 本函数**（spec `--*` 值原样保留语义）。
pub fn canonicalize_anchor_functions(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut copied = 0usize;
    let mut cursor = 0usize;
    while let Some((start, end)) = find_anchor_function(value, cursor) {
        let content = &value[start..end];
        let (chunks, seps) = split_top_level_commas(content);
        // 首块：参数序换位（换位输出以单空格重排）；其余块：fallback canonical
        //（无单位数字补 px / 嵌套调用递归）——发射保留逗号后原空白结构。
        let mut rebuilt = canonicalize_arg_chunk(chunks[0]);
        for (k, chunk) in chunks.iter().enumerate().skip(1) {
            rebuilt.push_str(seps[k - 1]);
            let lead_ws_len = chunk.len() - chunk.trim_start().len();
            rebuilt.push_str(&chunk[..lead_ws_len]);
            rebuilt.push_str(&canonicalize_fallback_chunk(chunk));
        }
        // `value[copied..start]` 含函数名、名后空白与左括号，原样保留。
        out.push_str(&value[copied..start]);
        out.push_str(&rebuilt);
        out.push(')');
        copied = end + 1;
        cursor = end + 1;
    }
    out.push_str(&value[copied..]);
    out
}

#[cfg(test)]
mod tests {
    use super::canonicalize_anchor_functions;

    #[test]
    fn anchor_size_flip_order_canonicalizes() {
        assert_eq!(
            canonicalize_anchor_functions("anchor-size(width --foo)"),
            "anchor-size(--foo width)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor-size(height --foo)"),
            "anchor-size(--foo height)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor-size(self-block --foo)"),
            "anchor-size(--foo self-block)"
        );
    }

    #[test]
    fn canonical_order_untouched() {
        assert_eq!(
            canonicalize_anchor_functions("anchor-size(--foo width)"),
            "anchor-size(--foo width)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor-size(--foo)"),
            "anchor-size(--foo)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor-size(width)"),
            "anchor-size(width)"
        );
        assert_eq!(canonicalize_anchor_functions("anchor-size()"), "anchor-size()");
        assert_eq!(canonicalize_anchor_functions("anchor-size(10px)"), "anchor-size(10px)");
    }

    #[test]
    fn fallback_after_comma_preserved_and_recursed() {
        assert_eq!(
            canonicalize_anchor_functions("anchor-size(width --foo, 1px)"),
            "anchor-size(--foo width, 1px)"
        );
        assert_eq!(
            canonicalize_anchor_functions(
                "anchor-size(width --foo, anchor-size(block --bar, anchor-size(--baz inline)))"
            ),
            "anchor-size(--foo width, anchor-size(--bar block, anchor-size(--baz inline)))"
        );
    }

    #[test]
    fn anchor_function_side_flip() {
        assert_eq!(canonicalize_anchor_functions("anchor(50% --foo)"), "anchor(--foo 50%)");
        assert_eq!(
            canonicalize_anchor_functions("anchor(left --foo)"),
            "anchor(--foo left)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor(calc(50%) --foo)"),
            "anchor(--foo calc(50%))"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor(--foo left)"),
            "anchor(--foo left)"
        );
    }

    #[test]
    fn inside_calc_tree() {
        assert_eq!(
            canonicalize_anchor_functions("calc((anchor-size(width --foo) + anchor-size(height --bar)) / 2)"),
            "calc((anchor-size(--foo width) + anchor-size(--bar height)) / 2)"
        );
    }

    #[test]
    fn non_function_text_untouched() {
        assert_eq!(
            canonicalize_anchor_functions("var(--width) 10px solid red"),
            "var(--width) 10px solid red"
        );
        // 左边界：`--anchor-size(` 是以 ident 续字符接续的名，不换位
        assert_eq!(
            canonicalize_anchor_functions("--anchor-size(width --foo)"),
            "--anchor-size(width --foo)"
        );
    }

    #[test]
    fn case_insensitive_function_name_preserves_spelling() {
        assert_eq!(
            canonicalize_anchor_functions("ANCHOR-SIZE(width --foo)"),
            "ANCHOR-SIZE(--foo width)"
        );
    }

    #[test]
    fn idempotent_on_canonical_input() {
        let canonical = "anchor-size(--foo width, anchor-size(--bar block))";
        assert_eq!(canonicalize_anchor_functions(canonical), canonical);
    }

    #[test]
    fn fallback_unitless_number_gets_px() {
        // css-anchor-1 §3/§4 fallback <length-percentage> type 归一
        //（anchor-parse-valid / anchor-size-parse-valid 语料 canonical 钉面：
        // `anchor(--foo left, 0)` → `anchor(--foo left, 0px)`）。% /维度/calc
        // 保形；幂等。
        assert_eq!(
            canonicalize_anchor_functions("anchor(--foo left, 0)"),
            "anchor(--foo left, 0px)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor-size(--foo width, 0)"),
            "anchor-size(--foo width, 0px)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor(--foo left, 1px)"),
            "anchor(--foo left, 1px)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor(--foo left, 50%)"),
            "anchor(--foo left, 50%)"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor(--foo left, calc(0.5 * (1px + 1em)))"),
            "anchor(--foo left, calc(0.5 * (1px + 1em)))"
        );
        assert_eq!(
            canonicalize_anchor_functions("anchor(--foo left, 0px)"),
            "anchor(--foo left, 0px)"
        );
    }
}
