//! url() 请求修饰符钉面（CSS Values 5 §request-url-modifiers）。
//!
//! 词法文法（known 三修饰符参量违例/同名重键、unknown (名,形态) 二元重名键、
//! 非 ident/function 词元拒、修饰符前置/无引号带修饰符拒）、canonical 序列化
//!（已知字母序重组、未知丢弃、integrity("") 空串）、写面声明裁决门。
//!
//! driving: wptfyi-css DC-2 URLS-MODIFIERS（css-values-5 urls 语料桶
//! url-request-modifiers-invalid/-serialize/-computed 页）。

use crate::tokenizer::{CrossOriginValue, Token, Tokenizer, UrlRequestModifier, canonical_url_modifier_list};
use crate::values::parse_background_image;
use crate::values::parse_extended_visual::{
    BackgroundImageValue, canonicalize_url_modifier_value, url_modifier_declaration_valid,
    value_has_url_request_modifiers,
};

/// WPT 页 `http://{{hosts[][]}}:{{ports[http][0]}}/css/support/1x1-green.png`
/// 测试 URL 的常量代换。
const U: &str = "http://example.com:8080/css/support/1x1-green.png";

// ── invalid 页 38 形（test_invalid_value('background-image', …)）───────

/// invalid 形全集：词法/写面门两通道同拒。
#[test]
fn invalid_page_matrix_rejected() {
    let cases = [
        // cross-origin 参量违例 + 同名重键 + 修饰符前置 + 裸串后随
        format!("url(\"{U}\" cross-origin())"),
        format!("url(\"{U}\" cross-origin(,))"),
        format!("url(\"{U}\" cross-origin(anonymous,))"),
        format!("url(\"{U}\" cross-origin(,anonymous))"),
        format!("url(\"{U}\" cross-origin(anonymous foobar))"),
        format!("url(\"{U}\" cross-origin(anonymous) cross-origin(anonymous))"),
        format!("url(\"{U}\" cross-origin(anonymous) cross-origin(use-credentials))"),
        format!("url(cross-origin(anonymous) \"{U}\")"),
        format!("\"{U}\" cross-origin(anonymous)"),
        // integrity 家族
        format!("url(\"{U}\" integrity())"),
        format!("url(\"{U}\" integrity(,))"),
        format!("url(\"{U}\" integrity(\"sha384-foobar\",))"),
        format!("url(\"{U}\" integrity(,\"sha384-foobar\"))"),
        format!("url(\"{U}\" integrity(\"sha384-foobar\" foobar))"),
        format!("url(\"{U}\" integrity(sha384-foobar))"),
        format!("url(\"{U}\" integrity(\"sha384-foobar\") integrity(\"sha384-foobar\"))"),
        format!("url(\"{U}\" integrity(\"sha384-foobar\") integrity(\"sha384-barbaz\"))"),
        format!("url(integrity(\"sha384-foobar\") \"{U}\")"),
        format!("\"{U}\" integrity(\"sha384-foobar\")"),
        // referrer-policy 家族
        format!("url(\"{U}\" referrer-policy())"),
        format!("url(\"{U}\" referrer-policy(,))"),
        format!("url(\"{U}\" referrer-policy(no-referrer,))"),
        format!("url(\"{U}\" referrer-policy(,no-referrer))"),
        format!("url(\"{U}\" referrer-policy(no-referrer foobar))"),
        format!("url(\"{U}\" referrer-policy(no-referrer same-origin))"),
        format!("url(\"{U}\" referrer-policy(no-referrer) referrer-policy(no-referrer))"),
        format!("url(\"{U}\" referrer-policy(no-referrer) referrer-policy(same-origin))"),
        format!("url(referrer-policy(no-referrer) \"{U}\")"),
        format!("\"{U}\" referrer-policy(no-referrer)"),
        // unknown (名,形态) 二元重名键——大小写不敏感、异参同拒
        "url(\"/css/support/1x1-green.png\" foobar(baz) foobar(baz))".to_string(),
        "url(\"/css/support/1x1-green.png\" foobar(baz) foobar(qux))".to_string(),
        "url(\"/css/support/1x1-green.png\" foobar foobar)".to_string(),
        "url(\"/css/support/1x1-green.png\" FooBar foobar)".to_string(),
        // 非 ident/function 词元
        "url(\"/css/support/1x1-green.png\" 42)".to_string(),
        "url(\"/css/support/1x1-green.png\" \"foobar\")".to_string(),
        // 无引号 url 不能接受修饰符（css-values-4 §url-modifiers）
        format!("url({U} cross-origin(anonymous))"),
        format!("url({U} integrity(\"sha384-foobar\"))"),
        format!("url({U} referrer-policy(no-referrer))"),
    ];
    assert_eq!(cases.len(), 38);
    for (idx, value) in cases.iter().enumerate() {
        assert!(!url_modifier_declaration_valid(value), "case {idx} 应拒收：{value}");
        // 值层 parse 同拒（裸串+修饰符形无 url() 包裹不属 background-image
        // 文法，本断言只对 url() 包裹的 34 形生效）。
        if value.contains("url(") {
            assert!(
                parse_background_image(value).is_none(),
                "case {idx} 值层应拒收：{value}"
            );
        }
    }
}

// ── serialize 页（test_valid_value 含 expected canonical 形）──────────

/// serialize 形全集：写面门全放行 + canonical 重放等于 expected
///（expected == input 恒等形与 reorder/剥除形混排，与 WPT 页逐条对齐）。
#[test]
fn serialize_page_matrix_canonical() {
    let cases: Vec<(String, String)> = vec![
        // 无修饰符
        (format!("url(\"{U}\")"), format!("url(\"{U}\")")),
        // 三 known 单修饰符（identity 含 integrity("") 空串与 8 关键字）
        (
            format!("url(\"{U}\" cross-origin(anonymous))"),
            format!("url(\"{U}\" cross-origin(anonymous))"),
        ),
        (
            format!("url(\"{U}\" cross-origin(use-credentials))"),
            format!("url(\"{U}\" cross-origin(use-credentials))"),
        ),
        (
            format!("url(\"{U}\" integrity(\"sha384-foobar\"))"),
            format!("url(\"{U}\" integrity(\"sha384-foobar\"))"),
        ),
        (
            format!("url(\"{U}\" integrity(\"\"))"),
            format!("url(\"{U}\" integrity(\"\"))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(no-referrer))"),
            format!("url(\"{U}\" referrer-policy(no-referrer))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(no-referrer-when-downgrade))"),
            format!("url(\"{U}\" referrer-policy(no-referrer-when-downgrade))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(same-origin))"),
            format!("url(\"{U}\" referrer-policy(same-origin))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(origin))"),
            format!("url(\"{U}\" referrer-policy(origin))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(strict-origin))"),
            format!("url(\"{U}\" referrer-policy(strict-origin))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(origin-when-cross-origin))"),
            format!("url(\"{U}\" referrer-policy(origin-when-cross-origin))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(strict-origin-when-cross-origin))"),
            format!("url(\"{U}\" referrer-policy(strict-origin-when-cross-origin))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(unsafe-url))"),
            format!("url(\"{U}\" referrer-policy(unsafe-url))"),
        ),
        // 双修饰符：已序恒等 + 三族六序字母序重组
        (
            format!("url(\"{U}\" cross-origin(anonymous) integrity(\"sha384-foobar\"))"),
            format!("url(\"{U}\" cross-origin(anonymous) integrity(\"sha384-foobar\"))"),
        ),
        (
            format!("url(\"{U}\" integrity(\"sha384-foobar\") cross-origin(anonymous))"),
            format!("url(\"{U}\" cross-origin(anonymous) integrity(\"sha384-foobar\"))"),
        ),
        (
            format!("url(\"{U}\" integrity(\"sha384-foobar\") referrer-policy(no-referrer))"),
            format!("url(\"{U}\" integrity(\"sha384-foobar\") referrer-policy(no-referrer))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(no-referrer) integrity(\"sha384-foobar\"))"),
            format!("url(\"{U}\" integrity(\"sha384-foobar\") referrer-policy(no-referrer))"),
        ),
        (
            format!("url(\"{U}\" cross-origin(anonymous) referrer-policy(no-referrer))"),
            format!("url(\"{U}\" cross-origin(anonymous) referrer-policy(no-referrer))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(no-referrer) cross-origin(anonymous))"),
            format!("url(\"{U}\" cross-origin(anonymous) referrer-policy(no-referrer))"),
        ),
        // 三修饰符四种序 → 统一字母序
        (
            format!("url(\"{U}\" cross-origin(anonymous) integrity(\"sha384-foobar\") referrer-policy(no-referrer))"),
            format!("url(\"{U}\" cross-origin(anonymous) integrity(\"sha384-foobar\") referrer-policy(no-referrer))"),
        ),
        (
            format!("url(\"{U}\" integrity(\"sha384-foobar\") referrer-policy(no-referrer) cross-origin(anonymous))"),
            format!("url(\"{U}\" cross-origin(anonymous) integrity(\"sha384-foobar\") referrer-policy(no-referrer))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(no-referrer) cross-origin(anonymous) integrity(\"sha384-foobar\"))"),
            format!("url(\"{U}\" cross-origin(anonymous) integrity(\"sha384-foobar\") referrer-policy(no-referrer))"),
        ),
        (
            format!("url(\"{U}\" referrer-policy(no-referrer) integrity(\"sha384-foobar\") cross-origin(anonymous))"),
            format!("url(\"{U}\" cross-origin(anonymous) integrity(\"sha384-foobar\") referrer-policy(no-referrer))"),
        ),
        // unknown 丢弃 + known 后继仍识别
        (
            "url(\"/css/support/1x1-green.png\" foobar(baz))".to_string(),
            "url(\"/css/support/1x1-green.png\")".to_string(),
        ),
        (
            "url(\"/css/support/1x1-green.png\" foobar)".to_string(),
            "url(\"/css/support/1x1-green.png\")".to_string(),
        ),
        (
            "url(\"/css/support/1x1-green.png\" foobar(baz) cross-origin(anonymous))".to_string(),
            "url(\"/css/support/1x1-green.png\" cross-origin(anonymous))".to_string(),
        ),
        (
            "url(\"/css/support/1x1-green.png\" cross-origin(anonymous) foobar(a b c))".to_string(),
            "url(\"/css/support/1x1-green.png\" cross-origin(anonymous))".to_string(),
        ),
        (
            "url(\"/css/support/1x1-green.png\" foobar(nes(ted)) integrity(\"sha384-foobar\"))".to_string(),
            "url(\"/css/support/1x1-green.png\" integrity(\"sha384-foobar\"))".to_string(),
        ),
        (
            "url(\"/css/support/1x1-green.png\" foobar([brackets {braces}]) referrer-policy(no-referrer))".to_string(),
            "url(\"/css/support/1x1-green.png\" referrer-policy(no-referrer))".to_string(),
        ),
        // 过时名（无连字符）＝unknown 丢弃
        (
            "url(\"/css/support/1x1-green.png\" crossorigin(anonymous))".to_string(),
            "url(\"/css/support/1x1-green.png\")".to_string(),
        ),
        (
            "url(\"/css/support/1x1-green.png\" referrerpolicy(no-referrer))".to_string(),
            "url(\"/css/support/1x1-green.png\")".to_string(),
        ),
        // 异形（ident/function）同名不互斥
        (
            "url(\"/css/support/1x1-green.png\" foobar foobar(42))".to_string(),
            "url(\"/css/support/1x1-green.png\")".to_string(),
        ),
        (
            "url(\"/css/support/1x1-green.png\" foobar(42) foobar)".to_string(),
            "url(\"/css/support/1x1-green.png\")".to_string(),
        ),
    ];
    assert_eq!(cases.len(), 33);
    for (idx, (input, expected)) in cases.iter().enumerate() {
        assert!(url_modifier_declaration_valid(input), "case {idx} 应放行：{input}");
        if !input.contains(' ') {
            // 无修饰符形：不进 canonical store 径（pre-gate 判定负），
            // 序列化走 Token::Url 既有 Display（quoted 需求面旧径裁决）。
            assert!(!value_has_url_request_modifiers(input));
            continue;
        }
        assert!(
            value_has_url_request_modifiers(input),
            "case {idx} 修饰符判定异常：{input}"
        );
        let canonical = canonicalize_url_modifier_value(input);
        assert_eq!(&canonical, expected, "case {idx} canonical 漂移");
    }
}

/// 值层 [`parse_background_image`] 带修饰符拆分 + computed/绘制同径 url。
#[test]
fn value_layer_split_roundtrip() {
    let v = parse_background_image("url(\"/a.png\" integrity(\"s\"))").expect("应解析");
    match v {
        BackgroundImageValue::UrlWithModifiers(url, mods) => {
            assert_eq!(url, "/a.png");
            assert_eq!(mods.len(), 1);
            assert_eq!(mods[0], UrlRequestModifier::Integrity("s".to_string()));
        }
        other => panic!("应落 UrlWithModifiers 臂，得 {other:?}"),
    }
    // 无修饰符形落既有 Url 臂（零漂移）。
    assert_eq!(
        parse_background_image("url(\"/a.png\")"),
        Some(BackgroundImageValue::Url("/a.png".to_string()))
    );
    // 未知修饰符参量含右括号串。
    let v = parse_background_image("url(\"/a.png\" foobar(\"x)\"))").expect("应解析");
    assert!(matches!(v, BackgroundImageValue::UrlWithModifiers(..)));
}

/// [`canonical_url_modifier_list`] 单元面：排序、未知剥除、escape。
#[test]
fn canonical_list_unit() {
    let mods = vec![
        UrlRequestModifier::Unknown,
        UrlRequestModifier::ReferrerPolicy("unsafe-url".to_string()),
        UrlRequestModifier::Integrity("a\"b".to_string()),
        UrlRequestModifier::CrossOrigin(CrossOriginValue::UseCredentials),
    ];
    assert_eq!(
        canonical_url_modifier_list(&mods),
        " cross-origin(use-credentials) integrity(\"a\\\"b\") referrer-policy(unsafe-url)"
    );
    assert_eq!(canonical_url_modifier_list(&[]), "");
    assert_eq!(canonical_url_modifier_list(&[UrlRequestModifier::Unknown]), "");
}

/// 词元级：UrlWithModifiers 变体与 Token::Url 区分（11 个既有消费位双臂
/// 匹配的词元事实）。
#[test]
fn token_level_variants() {
    let toks: Vec<Token> = Tokenizer::new("url(\"/a.png\")").map(|s| s.token).collect();
    assert!(matches!(toks[0], Token::Url(_)));
    let toks: Vec<Token> = Tokenizer::new("url(\"/a.png\" foobar)").map(|s| s.token).collect();
    assert!(matches!(toks[0], Token::UrlWithModifiers(..)));
    // 修饰符违例落 Error 词元（bad-url remnants 语义）。
    let toks: Vec<Token> = Tokenizer::new("url(\"/a.png\" foobar foobar)")
        .map(|s| s.token)
        .collect();
    assert!(matches!(toks[0], Token::Error(_)));
}
