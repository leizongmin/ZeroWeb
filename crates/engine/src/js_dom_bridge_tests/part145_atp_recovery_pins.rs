// 刀 ATP 恢复道钉测（2026-11-19）：① R2 note_stylesheets 接线——CSS 源
// `@property` 规则经 stylesheet 喂入注册表（known/syntax_of/computed_value
// 三消费面实证）；② 同名重声明后者胜（insert-or-replace，CSS 源 re-registration
// 语义）；③ 无效规则不注册（解析门丢弃→AST 无规则→注册表空）；④ string 值
// canonical 双引号重串化（R1 strict 门 StringKind 臂，`'foo'` → `"foo"`）；
// ⑤ universal 无 initial 可注册（declared None → None 兜底）。
#[test]
fn test_atp_note_stylesheets_registry_pins() {
    use crate::js_dom_bridge::registered_properties as rp;
    use zero_css_parser::Parser;

    // ① 基本 CSS 源注册（typed `<color>` + initial 兜底）。
    let sheet =
        Parser::parse_stylesheet("@property --atp-pin { syntax: \"<color>\"; inherits: false; initial-value: red; }");
    rp::note_stylesheets(&[sheet]);
    assert!(rp::registration_known("--atp-pin"), "css-source rule must register");
    assert_eq!(rp::syntax_of("--atp-pin").as_deref(), Some("<color>"));
    assert_eq!(
        rp::computed_value("--atp-pin", None, 16.0).as_deref(),
        Some("rgb(255, 0, 0)"),
        "typed initial surfaces as gCS fallback in computed form (strict gate canonicalization)"
    );

    // ② 重声明后者胜（两 sheet 先后喂入）。
    let sheet2 =
        Parser::parse_stylesheet("@property --atp-pin { syntax: \"<color>\"; inherits: false; initial-value: blue; }");
    rp::note_stylesheets(&[sheet2]);
    assert_eq!(
        rp::computed_value("--atp-pin", None, 16.0).as_deref(),
        Some("rgb(0, 0, 255)"),
        "re-declaration must replace (last wins)"
    );

    // ③ 无效规则（syntax 缺引号）已被解析门丢弃 → 不注册。
    let bad = Parser::parse_stylesheet("@property --atp-bad { syntax: <color>; inherits: false; initial-value: red; }");
    rp::note_stylesheets(&[bad]);
    assert!(!rp::registration_known("--atp-bad"), "invalid rule must not register");

    // ④ string 值 canonical 双引号重串化（单引号输入）。
    let str_rule = Parser::parse_stylesheet(
        "@property --atp-str { syntax: \"<string>\"; inherits: false; initial-value: 'foo'; }",
    );
    rp::note_stylesheets(&[str_rule]);
    assert_eq!(
        rp::computed_value("--atp-str", None, 16.0).as_deref(),
        Some("\"foo\""),
        "string initial must re-serialize double-quoted"
    );

    // ⑤ universal 形无 initial 合法注册（兜底 None）。
    let uni = Parser::parse_stylesheet("@property --atp-u { syntax: \"*\"; inherits: true; }");
    rp::note_stylesheets(&[uni]);
    assert!(rp::registration_known("--atp-u"));
    assert_eq!(
        rp::computed_value("--atp-u", None, 16.0),
        Some(String::new()),
        "guaranteed-invalid serializes empty (gCS unset registered property semantics)"
    );

    // ⑥ 规则通道换代重建（fix C）：后一代 stylesheet 不含前代规则 → 前代注册
    // 消失（@property 离场语义；JS 通道 REGISTRY 分表不受影响）。
    let gen2 = Parser::parse_stylesheet("@property --atp-u2 { syntax: \"*\"; inherits: true; }");
    rp::note_stylesheets(&[gen2]);
    assert!(rp::registration_known("--atp-u2"), "next generation registers");
    assert!(
        !rp::registration_known("--atp-u"),
        "rule absent from next generation must deregister (per-generation rebuild)"
    );
}
