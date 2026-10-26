//! 合并外链样式表前，保留其中 URL 的声明位置语义。

use zero_css_parser::{Token, Tokenizer};

pub(crate) fn absolutize_css_urls(css: &str, stylesheet_url: &str) -> String {
    let Ok(base) = url::Url::parse(stylesheet_url) else {
        return css.to_string();
    };
    // Tokenizer 的偏移相对去 BOM 后的输入；输出保留原始文本及 BOM。
    let bom_len = css.len() - css.strip_prefix('\u{feff}').unwrap_or(css).len();
    let mut tokenizer = Tokenizer::new(css);
    let mut out = String::with_capacity(css.len());
    let mut copied = 0;
    while let Some(spanned) = tokenizer.next() {
        let Token::Url(value) = spanned.token else {
            continue;
        };
        // https://drafts.csswg.org/css-values-4/#relative-urls
        // 空 URL、局部 fragment 和绝对 URL 保持原样；字符串/注释不产生 Url token。
        if value.is_empty() || value.starts_with('#') || url::Url::parse(&value).is_ok() {
            continue;
        }
        if let Ok(resolved) = base.join(&value) {
            out.push_str(&css[copied..bom_len + spanned.offset]);
            out.push_str(&Token::Url(resolved.to_string()).to_string());
            copied = bom_len + tokenizer.position();
        }
    }
    out.push_str(&css[copied..]);
    out
}

/// 剥离样式表文本中的顶层 `@import` 语句（css-syntax「consume an at-rule」：
/// prelude 消费到 `;` 或 `{}` 块为止）。
///
/// 导入的子表由调用方单独抓取后拼接——`@import` 语句文本若留在合并文本里，
/// 无分号形态（上游 import.css 即 `@import url("background.css")` 无 `;`）会把
/// **后续拼接表的首条规则**吞进 at-rule 块：css-syntax 的 at-rule 消费遇裸 `{`
/// 即吞块，单文件内无害（语句后本就无规则），合并后致命（WPT
/// execution-timing 106-import 面：`#test{position:fixed}` 恒缺级联）。
/// 嵌套（块内）/注释/字符串里的 `@import` 不动（Tokenizer 不在注释与字符串里
/// 产 AtKeyword；嵌套 `@import` 本就非法，留给解析器）。
pub(crate) fn strip_import_statements(css: &str) -> String {
    // Tokenizer 的偏移相对去 BOM 后的输入；输出保留原始文本及 BOM（同上）。
    let bom_len = css.len() - css.strip_prefix('\u{feff}').unwrap_or(css).len();
    let mut tokenizer = Tokenizer::new(css);
    let mut out = String::with_capacity(css.len());
    let mut copied = 0;
    let mut depth: usize = 0; // 顶层花括号深度（只剥顶层 @import）
    // Some(start) = 正在切除的语句起点（BOM 后偏移）；in_block = 切除中已进入
    // at-rule 块（无分号形态——块整体属于该语句，吞到配对 `}` 为止）。
    let mut cutting: Option<usize> = None;
    let mut in_block = false;
    while let Some(spanned) = tokenizer.next() {
        let end = bom_len + tokenizer.position();
        match spanned.token {
            Token::AtKeyword(name) if depth == 0 && name.eq_ignore_ascii_case("import") => {
                if cutting.is_none() {
                    cutting = Some(bom_len + spanned.offset);
                    in_block = false;
                }
            }
            Token::LBrace => {
                depth += 1;
                if cutting.is_some() {
                    in_block = true;
                }
            }
            Token::RBrace => {
                depth = depth.saturating_sub(1);
                if cutting.is_some() && in_block && depth == 0 {
                    out.push_str(&css[copied..cutting.take().unwrap()]);
                    copied = end;
                }
            }
            Token::Semicolon if cutting.is_some() && !in_block && depth == 0 => {
                out.push_str(&css[copied..cutting.take().unwrap()]);
                copied = end;
            }
            _ => {}
        }
    }
    if let Some(start) = cutting {
        // 语句无 `;`/块终结（至 EOF）——吞到文本尾。
        out.push_str(&css[copied..start]);
        copied = css.len();
    }
    out.push_str(&css[copied..]);
    out
}

#[cfg(test)]
mod tests {
    use super::{absolutize_css_urls, strip_import_statements};
    use zero_engine::extract_font_faces;

    #[test]
    fn semicolonless_import_only_sheet_strips_clean() {
        // 上游 import.css 原文（无 `;`，EOF 终结）——剥离后为空，不影响后续
        // 拼接表（106-import 面：@import 文本残留会吞掉下一张表的首条规则）。
        assert_eq!(strip_import_statements("@import url(\"background.css\")"), "");
    }

    #[test]
    fn semicolon_import_keeps_rest_of_sheet() {
        assert_eq!(
            strip_import_statements("@import \"a.css\";\np{color:red}"),
            "\np{color:red}"
        );
    }

    #[test]
    fn semicolonless_import_swallows_following_block_as_at_rule() {
        // css-syntax at-rule 消费遇裸 `{` 吞块——单文件内后续块本就不是规则，
        // 剥离语义与解析器一致（整段移除）。
        assert_eq!(strip_import_statements("@import url(a.css)\n#test{color:red}"), "");
    }

    #[test]
    fn nested_quoted_and_case_insensitive_forms() {
        // 块内 @import 非法、留给解析器；字符串/注释里的 "@import" 是普通文本；
        // 大小写不敏感；多语句逐条剥离。
        assert_eq!(
            strip_import_statements(
                "@media print {@import url(x)} p{content:\"@import url(y)\"}/* @import */ @IMPORT \"a.css\";@import url(b.css);p{color:red}"
            ),
            "@media print {@import url(x)} p{content:\"@import url(y)\"}/* @import */ p{color:red}"
        );
    }

    #[test]
    fn quoted_url_retains_closing_delimiter_and_following_rule() {
        let css = r#"@font-face{font-family:Test;src:url("fonts/a.ttf")} p{color:red}"#;
        let rewritten = absolutize_css_urls(css, "https://example.test/assets/main.css");
        assert_eq!(
            rewritten,
            "@font-face{font-family:Test;src:url(https://example.test/assets/fonts/a.ttf)} p{color:red}"
        );
        assert_eq!(
            extract_font_faces(&rewritten)[0].1,
            ["https://example.test/assets/fonts/a.ttf"]
        );
    }

    #[test]
    fn only_url_tokens_are_rewritten_with_bom_and_css_escapes() {
        let css = "\u{feff}/* url(fake.png) */ p{content:'url(fake.png)';background:U\\52 L(\"字\\20 图.png\")}";
        assert_eq!(
            absolutize_css_urls(css, "https://example.test/css/main.css"),
            "\u{feff}/* url(fake.png) */ p{content:'url(fake.png)';background:url(https://example.test/css/%E5%AD%97%20%E5%9B%BE.png)}"
        );
    }

    #[test]
    fn absolute_empty_local_fragment_and_bad_urls_stay_unchanged() {
        let css = r#"a{a:url("");b:url(#local);c:url("data:image/png;base64,AA==");d:url(https://cdn.test/a);e:url("bad" trailing)}"#;
        assert_eq!(absolutize_css_urls(css, "https://example.test/css/main.css"), css);
        assert_eq!(
            absolutize_css_urls("a{background:url(x.png)}", "invalid"),
            "a{background:url(x.png)}"
        );
    }
}
