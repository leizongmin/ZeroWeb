//! CSS 字符编码检测（CSS Syntax §6.2 charset determination）。
//!
//! ZW 用 html5ever 按 UTF-8 解析 CSS，但上游 WPT corpus 含大量 `.xht`/`.css` 文件用
//! 非 UTF-8 编码（ISO-8859-1/5/6/7/8/11、koi8-r、UTF-16BE 等）。若按 UTF-8 强解，
//! 非 ASCII 字节（如 ISO-8859-1 的 `é` = 0xE9）变 U+FFFD，致选择器 `.tést` 不匹配
//! HTML class → 渲染失败（WPT at-charset-071~077 / character-encoding-031~037,041）。
//!
//! 按 CSS Syntax §6.2 优先级确定编码（高→低）：
//! 1. BOM（UTF-8 / UTF-16LE / UTF-16BE）
//! 2. `@charset "xxx";` 规则（须位于文档开头，紧随可能的 BOM）
//! 3. HTTP Content-Type header 的 charset 参数（file:// 下由 `.headers` sidecar 提供）
//! 4. UTF-8（默认，无效字节替换为 U+FFFD）
//!
//! WPT at-charset-* 用 (2)；character-encoding-* 用 (3)。

use encoding_rs::Encoding;

/// 按 CSS Syntax §6.2 优先级解码 CSS 字节流为字符串。
///
/// `content_type` 为 HTTP `Content-Type` header 值（如 `text/css; charset=iso-8859-1`），
/// 可能为 `None`（无 header）。
pub fn decode_css_bytes(body: &[u8], content_type: Option<&str>) -> String {
    // 1. BOM
    if let Some(enc) = bom_encoding(body) {
        return decode_with(enc, body);
    }
    // 2. @charset
    if let Some(enc) = sniff_at_charset(body) {
        return decode_with(enc, body);
    }
    // 3. Content-Type charset
    if let Some(ct) = content_type
        && let Some(enc) = charset_from_content_type(ct)
    {
        return decode_with(enc, body);
    }
    // 4. UTF-8 默认
    decode_with(encoding_rs::UTF_8, body)
}

/// 检测 BOM 并返回对应编码。
fn bom_encoding(body: &[u8]) -> Option<&'static Encoding> {
    if body.starts_with(&[0xEF, 0xBB, 0xBF]) {
        Some(encoding_rs::UTF_8)
    } else if body.starts_with(&[0xFE, 0xFF]) {
        Some(encoding_rs::UTF_16BE)
    } else if body.starts_with(&[0xFF, 0xFE]) {
        Some(encoding_rs::UTF_16LE)
    } else {
        None
    }
}

/// 嗅探 CSS 开头的 `@charset "xxx";` 规则（须在最开头，CSS Syntax §6.2）。
///
/// `@charset` 规则本身是 ASCII，可在任意 ASCII 兼容编码的字节中直接读取。
fn sniff_at_charset(body: &[u8]) -> Option<&'static Encoding> {
    // 只看前 1024 字节（@charset 须紧跟可能的 BOM，实际在最开头）。
    let head = &body[..body.len().min(1024)];
    let rest = head.strip_prefix(b"@charset")?;
    // 跳过 @charset 与引号间的空白
    let mut rest = rest;
    while rest.first().is_some_and(|b| b.is_ascii_whitespace()) {
        rest = &rest[1..];
    }
    // 第一个非空白字符必须是引号
    let quote = *rest.first()?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    rest = &rest[1..];
    let end = rest.iter().position(|b| *b == quote)?;
    let label = std::str::from_utf8(&rest[..end]).ok()?;
    Encoding::for_label(label.as_bytes())
}

/// 从 `Content-Type` header 解析 `charset=xxx` 参数为编码。
///
/// 按 RFC 7231 把 header 切成 `;` 分隔的参数段，只匹配**参数名恰好为 `charset`** 的段——
/// 不用子串匹配（旧实现 `find("charset=")` 会误命中 `xcharset=`/`supercharset=` 等非标准
/// 参数名，错误提取其值作为编码，偏离 CSS Syntax §6.2 语义）。
fn charset_from_content_type(content_type: &str) -> Option<&'static Encoding> {
    // 第一段是媒体类型（如 `text/css`），后续段为参数。逐段检查参数名。
    for param in content_type.split(';').skip(1) {
        // 无 `=` 的畸形参数段跳过（不应让一个坏参数终止整个解析）。
        let Some((name, value)) = param.split_once('=') else {
            continue;
        };
        // 参数名去首尾空白后大小写不敏感比较（RFC 7231 token 大小写不敏感）。
        if name.trim().eq_ignore_ascii_case("charset") {
            // label 到下一个 `;` 或空白结束（参数段内已无 `;`，这里主要防尾随空白/引号）。
            let label: String = value.trim().chars().take_while(|c| !c.is_ascii_whitespace()).collect();
            let label = label.trim_matches(|c| c == '"' || c == '\'');
            if let Some(enc) = Encoding::for_label(label.as_bytes()) {
                return Some(enc);
            }
        }
    }
    None
}

/// 用指定编码解码字节，剥离 UTF-8 BOM（若有）。
fn decode_with(enc: &'static Encoding, body: &[u8]) -> String {
    let body = body.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(body);
    let (cow, _used, _had_errors) = enc.decode(body);
    cow.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_utf8_default_no_charset() {
        // 无 BOM / @charset / Content-Type → UTF-8 默认
        let css = b".test { color: green; }";
        assert_eq!(decode_css_bytes(css, None), ".test { color: green; }");
    }

    #[test]
    fn decode_utf8_invalid_bytes_replaced() {
        // 无效 UTF-8 字节 → U+FFFD（不 panic）
        let css = b"`\xE9\xE9`";
        let s = decode_css_bytes(css, None);
        assert!(s.contains('\u{FFFD}'));
    }

    #[test]
    fn decode_at_charset_iso_8859_1() {
        // WPT at-charset-071：`@charset "iso-8859-1";` + 0xE9（é）应解码为 U+00E9
        let mut css = Vec::from(&b"@charset \"iso-8859-1\";\n.t"[..]);
        css.push(0xE9); // ISO-8859-1 的 é
        css.extend_from_slice(b"st { color: green; }");
        let s = decode_css_bytes(&css, Some("text/css"));
        assert!(s.starts_with("@charset \"iso-8859-1\";"));
        assert!(s.contains(".tést"), "0xE9 应解码为 é，got: {s:?}");
    }

    #[test]
    fn decode_content_type_charset_iso_8859_1() {
        // WPT character-encoding-031：Content-Type charset（无 @charset）
        let mut css = Vec::from(&b".t"[..]);
        css.push(0xE9); // é
        css.extend_from_slice(b"st { color: green; }");
        let s = decode_css_bytes(&css, Some("text/css; charset=iso-8859-1"));
        assert!(s.contains(".tést"), "Content-Type charset=iso-8859-1 应解码 0xE9 为 é");
    }

    #[test]
    fn decode_content_type_charset_utf16be() {
        // WPT character-encoding-041：UTF-16BE
        let css_utf16be: Vec<u8> = ".test { }".encode_utf16().flat_map(|u| u.to_be_bytes()).collect();
        let s = decode_css_bytes(&css_utf16be, Some("text/css; charset=utf-16be"));
        assert_eq!(s, ".test { }");
    }

    #[test]
    fn decode_content_type_charset_koi8_r() {
        // WPT character-encoding-037：koi8-r（西里尔）
        let s = decode_css_bytes(&[0xE1], Some("text/css; charset=koi8-r"));
        // koi8-r 0xE1 = А（U+0410）
        assert_eq!(s, "А");
    }

    #[test]
    fn decode_bom_utf8_wins_over_charset_param() {
        // BOM 优先于 Content-Type charset
        let mut css = Vec::from(&[0xEF, 0xBB, 0xBF][..]);
        css.extend_from_slice(b".test { }");
        let s = decode_css_bytes(&css, Some("text/css; charset=iso-8859-1"));
        assert!(s.starts_with(".test")); // UTF-8 BOM 剥离后正确，无 U+FEFF
    }

    #[test]
    fn content_type_charset_extraction() {
        // encoding_rs 把 "iso-8859-1" label 映射到 WINDOWS_1252（HTML 规范兼容，
        // ISO-8859-1 与 Windows-1252 在 0x80-0x9F 区间外的字节一致）。
        assert_eq!(
            charset_from_content_type("text/css; charset=iso-8859-1"),
            Encoding::for_label(b"iso-8859-1")
        );
        assert_eq!(
            charset_from_content_type("text/css; charset=\"utf-8\""),
            Some(encoding_rs::UTF_8)
        );
        assert_eq!(charset_from_content_type("text/css"), None);
    }

    // ── R3370：charset 参数边界匹配（防子串误命中）──

    #[test]
    /// R3370：非标准参数名 `xcharset=`/`supercharset=` 含子串 `charset=`，
    /// 不得被误当作 charset 参数提取其值（旧实现 `find("charset=")` 会误命中）。
    fn content_type_charset_param_name_must_be_exact_r3370() {
        // xcharset=iso-8859-1 不是 charset 参数 → 应返 None（无合法 charset）
        assert_eq!(
            charset_from_content_type("text/css; xcharset=iso-8859-1"),
            None,
            "xcharset= 不得误命中为 charset 参数"
        );
        assert_eq!(
            charset_from_content_type("text/css; supercharset=utf-16be"),
            None,
            "supercharset= 不得误命中为 charset 参数"
        );
        // 真正的 charset 参数仍正确提取
        assert_eq!(
            charset_from_content_type("text/css; charset=utf-8"),
            Some(encoding_rs::UTF_8)
        );
    }

    #[test]
    /// R3370：charset 参数名大小写不敏感（RFC 7231 token 大小写不敏感），
    /// 且其前可有多余空白。
    fn content_type_charset_param_case_insensitive_r3370() {
        assert_eq!(
            charset_from_content_type("text/css;  CHARSET=utf-8"),
            Some(encoding_rs::UTF_8),
            "CHARSET= 应大小写不敏感匹配 charset"
        );
        assert_eq!(
            charset_from_content_type("text/css; CharSet=\"utf-8\""),
            Some(encoding_rs::UTF_8)
        );
    }

    #[test]
    /// R3370：畸形无 `=` 参数段不应终止解析——charset 在其后仍应被找到。
    fn content_type_malformed_param_does_not_abort_r3370() {
        assert_eq!(
            charset_from_content_type("text/css; bogusparam; charset=utf-8"),
            Some(encoding_rs::UTF_8),
            "无 `=` 的畸形参数段应跳过，不影响后续 charset 解析"
        );
    }
}
