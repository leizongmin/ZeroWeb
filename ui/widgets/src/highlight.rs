//! 简化的语法高亮——将源码拆分成带颜色的 token 片段（P3-6-2 从 gallery 提升到 ui-sdk）。
///
/// 设计：纯字符串扫描，不做 AST 解析。覆盖：
/// - YAML：`#` 注释、`"..."` 字符串、`key:` 关键字、数字
/// - Rust：`//` 注释、`"..."` 字符串、关键字、数字
///
/// 已知简化：字符串字面量内部的转义不严格处理；多行场景按行独立扫描。
pub fn highlight_yaml(src: &str) -> Vec<(&str, &str)> {
    let mut result = Vec::new();
    let mut remaining = src;
    while !remaining.is_empty() {
        let comment_pos = find_yaml_comment(remaining);
        let string_pos = remaining.find('"');
        let anchor = earliest(comment_pos, string_pos);

        match anchor {
            Some((pos, kind)) => {
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

fn emit_yaml_plain<'a>(out: &mut Vec<(&'a str, &'a str)>, segment: &'a str) {
    if segment.is_empty() {
        return;
    }
    if let Some(col) = segment.find(':')
        && {
            let after = &segment[col + 1..];
            after.is_empty() || after.starts_with(' ') || after.starts_with('\n')
        }
    {
        let key = &segment[..col + 1];
        out.push((key, "keyword"));
        emit_yaml_plain(out, &segment[col + 1..]);
        return;
    }
    if let Some(len) = leading_number(segment) {
        out.push((&segment[..len], "number"));
        emit_yaml_plain(out, &segment[len..]);
        return;
    }
    let mut next = next_yaml_break(segment);
    if next == 0 {
        next = 1;
    }
    out.push((&segment[..next], "default"));
    if next < segment.len() {
        emit_yaml_plain(out, &segment[next..]);
    }
}

fn next_yaml_break(s: &str) -> usize {
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if *b == b':' || b.is_ascii_digit() {
            return i;
        }
    }
    s.len()
}

fn find_yaml_comment(s: &str) -> Option<usize> {
    s.find('#').filter(|&pos| {
        let before = &s[..pos];
        let is_line_start = before.ends_with('\n') || before.is_empty();
        let is_space_prefixed = before.ends_with(' ') || before.ends_with('\t');
        is_line_start || is_space_prefixed
    })
}

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

fn emit_rust_plain<'a>(out: &mut Vec<(&'a str, &'a str)>, segment: &'a str, keywords: &[&'a str]) {
    if segment.is_empty() {
        return;
    }
    for kw in keywords {
        if let Some(after) = segment.strip_prefix(kw)
            && (after.is_empty() || (!after.as_bytes()[0].is_ascii_alphanumeric() && after.as_bytes()[0] != b'_'))
        {
            out.push((kw, "keyword"));
            emit_rust_plain(out, after, keywords);
            return;
        }
    }
    if let Some(len) = leading_number(segment) {
        out.push((&segment[..len], "number"));
        emit_rust_plain(out, &segment[len..], keywords);
        return;
    }
    let mut next = next_rust_break(segment, keywords);
    if next == 0 {
        next = 1;
    }
    out.push((&segment[..next], "default"));
    if next < segment.len() {
        emit_rust_plain(out, &segment[next..], keywords);
    }
}

fn next_rust_break(s: &str, keywords: &[&str]) -> usize {
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if b.is_ascii_digit() {
            return i;
        }
        for kw in keywords {
            if s[i..].starts_with(kw) {
                let before_ok = i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
                if before_ok {
                    return i;
                }
            }
        }
    }
    s.len()
}

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
/// 选色原则：在 light/dark 两种背景下都可读。
pub fn token_color(token_type: &str) -> (f32, f32, f32) {
    match token_type {
        "keyword" => (0.35, 0.55, 0.9),
        "string" => (0.2, 0.7, 0.3),
        "comment" => (0.55, 0.55, 0.55),
        "number" => (0.9, 0.55, 0.15),
        _ => (0.2, 0.2, 0.25),
    }
}
