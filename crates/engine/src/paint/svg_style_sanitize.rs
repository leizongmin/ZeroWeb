//! R5190（css-transforms-1 #transform-attribute-specificity / SVG2 presentation
//! attribute 优先级）：SVG `style` 属性中**无效** `transform` 声明的源级剔除。
//!
//! usvg 0.47 的 style 属性处理用 `simplecss::DeclarationTokenizer` 逐声明盲替换
//! 同名 presentation attribute（parser/svgtree/parse.rs `insert_attribute`），不
//! 校验声明值合法性；替换值（如 `scale(invalid)`）在 `resolve_transform` 时经
//! svgtypes 解析失败回退 identity（converter.rs：「`Node::attribute::<Transform>`
//! will always return a valid transform」），连带丢弃同元素的 `transform` 属性。
//! 规范语义：内联 style 中无效声明在解析期即丢弃 → 回退 presentation attribute
//! （WPT svg-inline-styles-005/006/010/013 四案实证：chromium 依属性变换，
//! usvg 恒 identity）。
//!
//! 本模块在序列化文本上逐 `style="…"` 属性走查：transform 声明值用 **ZW CSS
//! transform 语法**（`zero_css_parser::parse_transform`）探测，**ZW 拒绝才剔除**
//! ——对齐 chromium 的解析期丢弃语义（chromium 对非法值同样弃声明回退属性）；
//! ZW 接受的声明原样保留。保守性：ZW 接受面 ⊇ chromium 接受面（如 `rotate(90)`
//! 无单位形式），保留即维持 usvg 既有替换行为不变；ZW 拒绝的声明（`scale(invalid)`
//! 类）usvg 替换后恒 identity 而属性本可存活，剔除后与 chromium 回退属性一致
//! （005/006/010/013 翻绿实证）。CSS-wide/恒等关键字（none/inherit 等）不剔除：
//! usvg 解析失败恒 identity 恰为 none 语义；inherit 的父子差异属既有域
//! FIXME(style-keyword)。**勿用 svgtypes 探测**：001 案 `rotate(90deg)`（CSS
//! 合法值）按 svgtypes 属性语法判定为 invalid 而剔除 → 属性 `scale(0.5)` 错误
//! 存活翻红——CSS 合法值的既有管线行为以保留为准。
//!
//! 仅覆盖 transform；fill/stroke 等属性的同类盲替换缺陷回退路径各异（paint
//! 解析失败 ≠ 属性存活），FIXME(style-grammar) 挂账。
//!
//! kill-switch：env `ZW_SVG_STYLE_SANITIZE=0`（default-on）。

use simplecss::DeclarationTokenizer;

/// CSS-wide/恒等关键字：usvg 解析失败恒 identity、与 none 同效，不剔除。
const TRANSFORM_IDENTITY_KEYWORDS: [&str; 6] = ["none", "inherit", "initial", "unset", "revert", "revert-layer"];

/// transform 声明值是否须剔除：非 CSS-wide 关键字且 ZW CSS transform 语法拒绝。
fn drops_invalid_transform(value: &str) -> bool {
    let v = value.trim();
    if TRANSFORM_IDENTITY_KEYWORDS.iter().any(|k| v.eq_ignore_ascii_case(k)) {
        return false;
    }
    zero_css_parser::values::parse_transform(v).is_none()
}

/// 重建单个 style 属性值：剔除无效 transform 声明。返回 `None` 表示无须改动。
fn sanitize_style_value(value: &str) -> Option<String> {
    let mut kept: Vec<String> = Vec::new();
    let mut dropped = false;
    for decl in DeclarationTokenizer::from(value) {
        if decl.name.eq_ignore_ascii_case("transform") && drops_invalid_transform(decl.value) {
            dropped = true;
            continue;
        }
        let mut decl_text = format!("{}:{}", decl.name, decl.value);
        if decl.important {
            decl_text.push_str("!important");
        }
        kept.push(decl_text);
    }
    if !dropped {
        return None;
    }
    Some(kept.join(";"))
}

/// 对序列化 SVG 源剔除 style 属性中的无效 transform 声明（无效声明回退
/// presentation attribute，css-transforms-1 #transform-attribute-specificity）。
pub(crate) fn sanitize_svg_style_attrs(source: &str) -> String {
    if std::env::var("ZW_SVG_STYLE_SANITIZE").as_deref() == Ok("0") {
        return source.to_string();
    }
    if !source.contains("style") {
        return source.to_string();
    }
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut last = 0usize; // 已复制水位（切片复制，不逐字节，保 UTF-8 完整）
    let mut i = 0usize;
    while i < bytes.len() {
        if source.is_char_boundary(i) && source[i..].starts_with("style") {
            // 属性名边界：前一字符不得构成更长名字（`data-style=`、`font-style=`）。
            let prev = bytes[i - 1];
            let prev_ok =
                i == 0 || !prev.is_ascii_alphanumeric() && prev != b'-' && prev != b'_' && prev != b':' && prev != b'.';
            let mut j = i + "style".len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            // `<style>` 元素（后随 `>`/属性名）不是 style **属性**。
            if prev_ok && j < bytes.len() && bytes[j] == b'=' {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                    let quote = bytes[j] as char;
                    let Some(close_rel) = source[j + 1..].find(quote) else {
                        break;
                    };
                    let value_start = j + 1;
                    let value_end = j + 1 + close_rel;
                    if let Some(sanitized) = sanitize_style_value(&source[value_start..value_end]) {
                        out.push_str(&source[last..i]);
                        out.push_str(&source[i..value_start]);
                        out.push_str(&sanitized);
                        i = value_end;
                        last = value_end;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out.push_str(&source[last..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drops_invalid_transform_keeps_presentation_attribute() {
        // svg-inline-styles-013 缩样：无效内联声明剔除，transform 属性存活。
        let source = r#"<rect y="-60" width="100" height="100" fill="url(#grad)" style="transform: scale(invalid)" transform="rotate(90,20,20)"/>"#;
        let out = sanitize_svg_style_attrs(source);
        assert!(out.contains(r#"transform="rotate(90,20,20)""#));
        assert!(out.contains(r#"style="""#));
        assert!(!out.contains("scale(invalid)"));
    }

    #[test]
    fn test_valid_transform_declaration_kept() {
        let source = r#"<rect style="transform: translate(10) rotate(20)" transform="rotate(90)"/>"#;
        let out = sanitize_svg_style_attrs(source);
        assert_eq!(out, source);
    }

    #[test]
    fn test_transform_list_value_is_valid() {
        let source = r#"<g style="transform: translate(5,5) scale(2) skewX(30)"/>"#;
        assert_eq!(sanitize_svg_style_attrs(source), source);
    }

    #[test]
    fn test_identity_keywords_kept() {
        let source = r#"<rect style="transform: none" transform="rotate(90)"/>"#;
        assert_eq!(sanitize_svg_style_attrs(source), source);
    }

    #[test]
    fn test_css_unit_values_kept() {
        // 001 案教训：CSS 合法单位值（deg/px）保留——既有管线按 CSS 语义消费，
        // 勿按 SVG 属性语法误剔（曾致 001/002/012 翻红）。
        let deg = r#"<rect style="transform: rotate(90deg)" transform="scale(0.5)"/>"#;
        assert_eq!(sanitize_svg_style_attrs(deg), deg);
        let px = r#"<rect style="transform: translateY(-100px)"/>"#;
        assert_eq!(sanitize_svg_style_attrs(px), px);
    }

    #[test]
    fn test_only_invalid_declaration_dropped_among_multiple() {
        let source = r#"<rect style="fill:red;transform:scale(invalid);stroke:blue"/>"#;
        let out = sanitize_svg_style_attrs(source);
        assert!(out.contains("fill:red"));
        assert!(out.contains("stroke:blue"));
        assert!(!out.contains("scale(invalid)"));
    }

    #[test]
    fn test_single_quoted_and_no_style_source_unchanged() {
        let single = r#"<rect style='transform: scale(invalid)' transform="rotate(90)"/>"#;
        let out = sanitize_svg_style_attrs(single);
        // 引号风格保留（重建仅替换引号内值）。
        assert!(out.contains(r#"style=''"#));
        let none = r#"<rect fill="red" transform="rotate(90)"/>"#;
        assert_eq!(sanitize_svg_style_attrs(none), none);
    }

    #[test]
    fn test_style_element_content_untouched() {
        let source =
            r#"<svg><style>rect{transform:scale(invalid)}</style><rect style="transform: scale(invalid)"/></svg>"#;
        let out = sanitize_svg_style_attrs(source);
        assert!(out.contains("<style>rect{transform:scale(invalid)}</style>"));
        assert!(out.contains(r#"style="""#));
    }

    #[test]
    fn test_prefixed_attribute_names_not_matched() {
        let source = r#"<rect data-style="transform: scale(invalid)" font-style="normal"/>"#;
        assert_eq!(sanitize_svg_style_attrs(source), source);
    }

    #[test]
    fn test_important_flag_preserved_on_kept_decls() {
        let source = r#"<rect style="fill:red !important;transform:scale(invalid)"/>"#;
        let out = sanitize_svg_style_attrs(source);
        assert!(out.contains("fill:red!important"));
        assert!(!out.contains("scale(invalid)"));
    }
}
