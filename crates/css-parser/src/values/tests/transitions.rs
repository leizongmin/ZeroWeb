//! transition 族声明值 canonical 串化钉测（刀 U）。

use super::super::serialize_transitions::{
    canonicalize_timing_function_specified as canon_t, canonicalize_transition_property_specified as canon_p,
};

#[test]
fn test_transition_timing_function_canonical_steps() {
    // step-* 关键字展开与缺省步进位省略（valid 语料钉面）
    assert_eq!(canon_t("step-start").as_deref(), Some("steps(1, start)"));
    assert_eq!(canon_t("step-end").as_deref(), Some("steps(1)"));
    assert_eq!(canon_t("steps(2, end)").as_deref(), Some("steps(2)"));
    assert_eq!(canon_t("steps(2, jump-end)").as_deref(), Some("steps(2)"));
    // 大小写不敏感（keyword/函数名 canonical 小写）
    assert_eq!(canon_t("STEP-START").as_deref(), Some("steps(1, start)"));
    assert_eq!(canon_t("Steps(2, END)").as_deref(), Some("steps(2)"));
    // 非缺省步进位 identity passthrough
    assert_eq!(canon_t("steps(4)"), None);
    assert_eq!(canon_t("steps(4, start)"), None);
    assert_eq!(canon_t("steps(2, jump-start)"), None);
    assert_eq!(canon_t("steps(2, jump-both)"), None);
    assert_eq!(canon_t("steps(2, jump-none)"), None);
    assert_eq!(canon_t("steps(sibling-index(), jump-none)"), None);
    // ease 系/linear()/cubic-bezier() identity
    assert_eq!(canon_t("ease"), None);
    assert_eq!(canon_t("cubic-bezier(0, -2, 1, 3)"), None);
    assert_eq!(canon_t("linear(0 0%, 0.5 50%, 1 100%)"), None);
    // 列表：mixed per-component 处置 + 全 identity None
    assert_eq!(canon_t("ease, step-start").as_deref(), Some("ease, steps(1, start)"));
    assert_eq!(canon_t("linear, ease, linear"), None);
    // CSS-wide keyword / var() None
    assert_eq!(canon_t("initial"), None);
    assert_eq!(canon_t("var(--x)"), None);
}

#[test]
fn test_transition_property_canonical_keywords() {
    // 关键字归一小写，custom ident 原文保形（valid 语料钉面）
    assert_eq!(
        canon_p("ALL, INVALID, SYNTAX, SRC").as_deref(),
        Some("all, INVALID, SYNTAX, SRC")
    );
    assert_eq!(canon_p("NONE").as_deref(), Some("none"));
    // 全 identity None
    assert_eq!(canon_p("none"), None);
    assert_eq!(canon_p("all"), None);
    assert_eq!(canon_p("one"), None);
    assert_eq!(canon_p("one-two-three"), None);
    assert_eq!(canon_p("one, two, three"), None);
    assert_eq!(canon_p("width, all"), None);
    // CSS-wide keyword None（列表混入形归写面 gate，不在此处置）
    assert_eq!(canon_p("inherit"), None);
    assert_eq!(canon_p("var(--p)"), None);
}
