/// 简化的语法高亮——将源码拆分成带颜色的 token 片段。
/// YAML 高亮：key: → Keyword, "..." → String, # → Comment
pub fn highlight_yaml(src: &str) -> Vec<(&str, &str)> {
    let mut result = Vec::new();
    let mut remaining = src;
    while !remaining.is_empty() {
        if let Some(pos) = remaining.find('#') {
            if pos > 0 {
                result.push((&remaining[..pos], "default"));
            }
            let end = remaining[pos..].find('\n').map(|e| pos + e).unwrap_or(remaining.len());
            result.push((&remaining[pos..end], "comment"));
            remaining = &remaining[end..];
        } else if let Some(start) = remaining.find('"') {
            if start > 0 {
                let before = &remaining[..start];
                // check for key:
                if let Some(col) = before.rfind(':') {
                    let key = &before[..col + 1];
                    let rest = &before[col + 1..];
                    result.push((key, "keyword"));
                    if !rest.is_empty() {
                        result.push((rest, "default"));
                    }
                } else {
                    result.push((before, "default"));
                }
            }
            let rest = &remaining[start..];
            if let Some(end) = rest[1..].find('"') {
                let s = &rest[..end + 2];
                result.push((s, "string"));
                remaining = &rest[end + 2..];
            } else {
                result.push((rest, "default"));
                break;
            }
        } else if let Some(col) = remaining.find(':') {
            let key_end = col + 1;
            result.push((&remaining[..key_end], "keyword"));
            let after = &remaining[key_end..];
            if let Some(stripped) = after.strip_prefix(' ') {
                result.push((&after[..1], "default"));
                remaining = stripped;
            } else {
                remaining = after;
            }
        } else {
            result.push((remaining, "default"));
            break;
        }
    }
    result
}

/// Rust 高亮：关键字 → Keyword, "..." → String, // → Comment
pub fn highlight_rust(src: &str) -> Vec<(&str, &str)> {
    let keywords = [
        "fn", "let", "mut", "pub", "struct", "impl", "use", "mod", "const", "static", "enum", "trait", "for", "match",
        "if", "else", "while", "return", "true", "false", "Some", "None", "Box", "Ok", "Err", "self", "&", "->", "=>",
        "|",
    ];
    let mut result = Vec::new();
    let mut remaining = src;
    while !remaining.is_empty() {
        if remaining.starts_with("//") {
            let end = remaining.find('\n').unwrap_or(remaining.len());
            result.push((&remaining[..end], "comment"));
            remaining = &remaining[end..];
            continue;
        }
        if let Some(start) = remaining.find('"') {
            if start > 0 {
                result.push((&remaining[..start], "default"));
            }
            let rest = &remaining[start..];
            if let Some(end) = rest[1..].find('"') {
                let s = &rest[..end + 2];
                result.push((s, "string"));
                remaining = &rest[end + 2..];
            } else {
                result.push((rest, "default"));
                break;
            }
            continue;
        }
        let mut found_keyword = false;
        for kw in &keywords {
            if remaining.starts_with(kw) {
                let after = &remaining[kw.len()..];
                if after.is_empty() || !after.as_bytes()[0].is_ascii_alphanumeric() && after.as_bytes()[0] != b'_' {
                    result.push((kw, "keyword"));
                    remaining = after;
                    found_keyword = true;
                    break;
                }
            }
        }
        if found_keyword {
            continue;
        }
        let next_special = remaining.find(['"', '/', '\n']).unwrap_or(remaining.len());
        if next_special > 0 {
            result.push((&remaining[..next_special], "default"));
            remaining = &remaining[next_special..];
        } else {
            let ch = &remaining[..1];
            result.push((ch, "default"));
            remaining = &remaining[1..];
        }
    }
    result
}

/// Token 颜色映射
pub fn token_color(token_type: &str) -> (f32, f32, f32) {
    match token_type {
        "keyword" => (0.2, 0.4, 0.8),
        "string" => (0.1, 0.6, 0.1),
        "comment" => (0.5, 0.5, 0.5),
        _ => (0.1, 0.1, 0.1),
    }
}
