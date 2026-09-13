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

#[cfg(test)]
mod tests {
    use super::absolutize_css_urls;
    use zero_engine::extract_font_faces;

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
