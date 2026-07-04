/// 简化的语法高亮——将源码拆分成带颜色的 token 片段。
///
/// 设计：纯字符串扫描，不做 AST 解析（RFC §3.1）。覆盖：
/// - YAML：`#` 注释、`"..."` 字符串、`key:` 关键字、数字
/// - Rust：`//` 注释、`"..."` 字符串、关键字、数字
///
/// 已知简化：字符串字面量内部的转义不严格处理；多行场景按行独立扫描。
pub fn highlight_yaml(src: &str) -> Vec<(&str, &str)> {
    let mut result = Vec::new();
    let mut remaining = src;
    while !remaining.is_empty() {
        // 找下一个锚点：注释起点（合法位置上的 #）或字符串起点（"）。
        let comment_pos = find_yaml_comment(remaining);
        let string_pos = remaining.find('"');
        let anchor = earliest(comment_pos, string_pos);

        match anchor {
            Some((pos, kind)) => {
                // 锚点之前的片段按 key:/number/default 切分
                if pos > 0 {
                    emit_yaml_plain(&mut result, &remaining[..pos]);
                }
                let rest = &remaining[pos..];
                match kind {
                    AnchorKind::Comment => {
                        let end = rest.find('\n').unwrap_or(rest.len());
                        result.push((&rest[..end], "comment"));
                        remaining = &rest[end..];
                    }
                    AnchorKind::String => {
                        if let Some(close) = rest[1..].find('"') {
                            let s = &rest[..close + 2];
                            result.push((s, "string"));
                            remaining = &rest[close + 2..];
                        } else {
                            result.push((rest, "default"));
                            break;
                        }
                    }
                }
            }
            None => {
                emit_yaml_plain(&mut result, remaining);
                break;
            }
        }
    }
    result
}

#[derive(Clone, Copy)]
enum AnchorKind {
    Comment,
    String,
}

/// 取最早出现的锚点；同时出现时取位置小的，位置相同优先 comment。
fn earliest(comment: Option<usize>, string: Option<usize>) -> Option<(usize, AnchorKind)> {
    match (comment, string) {
        (Some(c), Some(s)) => {
            if c <= s {
                Some((c, AnchorKind::Comment))
            } else {
                Some((s, AnchorKind::String))
            }
        }
        (Some(c), None) => Some((c, AnchorKind::Comment)),
        (None, Some(s)) => Some((s, AnchorKind::String)),
        (None, None) => None,
    }
}

/// YAML 普通段：识别 `key:` 与数字。
fn emit_yaml_plain<'a>(out: &mut Vec<(&'a str, &'a str)>, segment: &'a str) {
    if segment.is_empty() {
        return;
    }
    // 找首个 ':' 作为 key 切分点（要求 ':' 之后是空格或行尾）
    if let Some(col) = segment.find(':') {
        let after = &segment[col + 1..];
        if after.is_empty() || after.starts_with(' ') || after.starts_with('\n') {
            let key = &segment[..col + 1];
            out.push((key, "keyword"));
            emit_yaml_plain(out, after);
            return;
        }
    }
    // 数字：扫到下个非数字非小数点字符
    if let Some(len) = leading_number(segment) {
        out.push((&segment[..len], "number"));
        emit_yaml_plain(out, &segment[len..]);
        return;
    }
    // 默认：到下个 ':' 或数字起点为止
    let mut next = next_yaml_break(segment);
    if next == 0 {
        next = 1;
    }
    out.push((&segment[..next], "default"));
    if next < segment.len() {
        emit_yaml_plain(out, &segment[next..]);
    }
}

/// 找下一个可能开始新 token 的位置（':' 或数字字面量起点）。
fn next_yaml_break(s: &str) -> usize {
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if *b == b':' || b.is_ascii_digit() {
            return i;
        }
    }
    s.len()
}

/// 找到当前行里「合法的」 # 注释起点：行首或前面是空白的 #。
fn find_yaml_comment(s: &str) -> Option<usize> {
    s.find('#').filter(|&pos| {
        let before = &s[..pos];
        let is_line_start = before.ends_with('\n') || before.is_empty();
        let is_space_prefixed = before.ends_with(' ') || before.ends_with('\t');
        is_line_start || is_space_prefixed
    })
}

/// Rust 高亮：关键字 → Keyword, "..." → String, // → Comment, 数字 → Number。
pub fn highlight_rust(src: &str) -> Vec<(&str, &str)> {
    static KEYWORDS: &[&str] = &[
        "fn", "let", "mut", "pub", "struct", "impl", "use", "mod", "const", "static", "enum", "trait", "for", "match",
        "if", "else", "while", "return", "true", "false", "Some", "None", "Box", "Ok", "Err", "self", "&", "->", "=>",
        "|",
    ];
    let mut result = Vec::new();
    let mut remaining = src;
    while !remaining.is_empty() {
        let comment_pos = remaining.find("//");
        let string_pos = remaining.find('"');
        let anchor = earliest(comment_pos, string_pos);

        match anchor {
            Some((pos, kind)) => {
                if pos > 0 {
                    emit_rust_plain(&mut result, &remaining[..pos], KEYWORDS);
                }
                let rest = &remaining[pos..];
                match kind {
                    AnchorKind::Comment => {
                        let end = rest.find('\n').unwrap_or(rest.len());
                        result.push((&rest[..end], "comment"));
                        remaining = &rest[end..];
                    }
                    AnchorKind::String => {
                        if let Some(close) = rest[1..].find('"') {
                            let s = &rest[..close + 2];
                            result.push((s, "string"));
                            remaining = &rest[close + 2..];
                        } else {
                            result.push((rest, "default"));
                            break;
                        }
                    }
                }
            }
            None => {
                emit_rust_plain(&mut result, remaining, KEYWORDS);
                break;
            }
        }
    }
    result
}

/// Rust 普通段：识别关键字与数字。
fn emit_rust_plain<'a>(out: &mut Vec<(&'a str, &'a str)>, segment: &'a str, keywords: &[&'a str]) {
    if segment.is_empty() {
        return;
    }
    // 关键字：要求其后是非字母数字下划线
    for kw in keywords {
        if let Some(after) = segment.strip_prefix(kw)
            && (after.is_empty() || (!after.as_bytes()[0].is_ascii_alphanumeric() && after.as_bytes()[0] != b'_'))
        {
            out.push((kw, "keyword"));
            emit_rust_plain(out, after, keywords);
            return;
        }
    }
    // 数字
    if let Some(len) = leading_number(segment) {
        out.push((&segment[..len], "number"));
        emit_rust_plain(out, &segment[len..], keywords);
        return;
    }
    // 默认：扫到下一个可能开始关键字/数字的位置
    let mut next = next_rust_break(segment, keywords);
    if next == 0 {
        // segment 起点本身被识别为关键字起点，但前面关键字扫描没匹配上
        // （例如 `&` 后面紧跟字母，`&` 不算 keyword）。强制前进 1 字符避免死递归。
        next = 1;
    }
    out.push((&segment[..next], "default"));
    if next < segment.len() {
        emit_rust_plain(out, &segment[next..], keywords);
    }
}

/// 找下一个关键字起点或数字起点。
fn next_rust_break(s: &str, keywords: &[&str]) -> usize {
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if b.is_ascii_digit() {
            return i;
        }
        for kw in keywords {
            if s[i..].starts_with(kw) {
                // 需要确认不是标识符中段
                let before_ok = i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
                if before_ok {
                    return i;
                }
            }
        }
    }
    s.len()
}

/// 扫描起头的数字字面量长度（含可选小数点）。不支持科学计数/后缀以保持简化。
fn leading_number(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_digit() {
        return None;
    }
    let mut end = 1;
    while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
        end += 1;
    }
    Some(end)
}

/// Token 颜色映射（线性 sRGB 三元组）。
///
/// 选色原则：在 light/dark 两种背景下都可读。default 由调用方与背景混色，这里给中性偏亮。
pub fn token_color(token_type: &str) -> (f32, f32, f32) {
    match token_type {
        "keyword" => (0.35, 0.55, 0.9),
        "string" => (0.2, 0.7, 0.3),
        "comment" => (0.55, 0.55, 0.55),
        "number" => (0.9, 0.55, 0.15),
        _ => (0.2, 0.2, 0.25),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_highlights_comment_string_and_key() {
        let src = "label: \"hi\" # comment";
        let toks = highlight_yaml(src);
        let kinds: Vec<_> = toks.iter().map(|(_, k)| *k).collect();
        assert!(kinds.contains(&"keyword"), "missing key: {kinds:?}");
        assert!(kinds.contains(&"string"), "missing string: {kinds:?}");
        assert!(kinds.contains(&"comment"), "missing comment: {kinds:?}");
    }

    #[test]
    fn yaml_does_not_treat_hash_in_string_as_comment() {
        let src = "value: \"a#b\"";
        let toks = highlight_yaml(src);
        assert!(
            !toks.iter().any(|(_, k)| *k == "comment"),
            "误把字符串内的 # 当注释: {toks:?}"
        );
    }

    #[test]
    fn yaml_highlights_number() {
        let src = "count: 3";
        let toks = highlight_yaml(src);
        assert!(toks.iter().any(|(_, k)| *k == "number"), "missing number: {toks:?}");
    }

    #[test]
    fn rust_highlights_keyword_string_comment_number() {
        let src = "let x = 1; // hi\nlet s = \"ok\";";
        let toks = highlight_rust(src);
        let kinds: Vec<_> = toks.iter().map(|(_, k)| *k).collect();
        assert!(kinds.contains(&"keyword"), "missing keyword: {kinds:?}");
        assert!(kinds.contains(&"string"), "missing string: {kinds:?}");
        assert!(kinds.contains(&"comment"), "missing comment: {kinds:?}");
        assert!(kinds.contains(&"number"), "missing number: {kinds:?}");
    }

    /// 回归：所有页的真实 source_rust/source_dsl 不应导致栈溢出或死递归。
    #[test]
    fn all_pages_source_highlight_without_overflow() {
        use crate::gallery::pages::ALL_PAGES;
        for page in ALL_PAGES {
            let _ = highlight_yaml(page.source_dsl);
            let _ = highlight_rust(page.source_rust);
        }
    }

    /// 回归：button 页真实 source_dsl 不应导致栈溢出。
    #[test]
    fn yaml_handles_real_button_source_without_overflow() {
        let src = r#"Button:
  id: my_button
  props:
    label: "Click me"
    action: "button.clicked"
    enabled: true"#;
        let _ = highlight_yaml(src);
    }
}
