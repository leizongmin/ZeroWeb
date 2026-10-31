//! p125r1(TV-1) 常驻回归：ShadowRoot 直接子元素的位置伪类取值必须与修复前
//! （merge-base bd040de1c 逐节点两遍循环）逐值一致。
//!
//! 机理：`parent_element_node` 对 ShadowRoot 直接子越过非元素父取 host，修复前
//! 两遍循环对 host 子列表跑满（`sib == node` 永不命中即不 break），得
//! `{child_index = child_count = host 元素子总数, type_index = type_count = host
//! 内同 tag 数}`；p125 索引路径 miss 后若落 default 全零，`:first-child`/
//! `:only-child`/`:nth-child(1)` 相对修复前 T→F 翻转。
//!
//! 规范语义（ShadowRoot 子按 shadow 实际元素子计位，CSS Scoping / Selectors L4）
//! 修复前后均未实现（FIXME：规范正确化另立任务）；本文件钉住的是「与修复前一致」
//! 这一冻结判据，fixture 期望值由 merge-base 算法逐字段推导。

use zero_dom::{Document, ShadowRootMode};

/// fixture 1：host 只有 1 个 light 元素子（span#l1），shadow root 有 [div#s1, span#s2]。
/// merge-base 推导：s1(div) → {1,1,0,0}；s2(span，与 l1 同 tag) → {1,1,1,1}。
#[test]
fn p125_shadow_direct_children_positions_match_merge_base() {
    let mut doc = Document::new();
    let root = doc.root();
    let host = doc.create_element("div");
    doc.append_child(root, host).unwrap();
    let light_span = doc.create_element("span");
    doc.append_child(host, light_span).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let s1 = doc.create_element("div");
    doc.append_child(shadow, s1).unwrap();
    let s2 = doc.create_element("span");
    doc.append_child(shadow, s2).unwrap();

    let qs = |sel: &str| doc.query_selector_shadow(shadow, sel).is_some();

    // s1：{ci=1, cc=1, ti=0, tc=0}——位置族全真；type 族除 `:last-of-type` 外全假
    // （matcher 判 `ti == tc`，{0,0} 退化相等 → T，base 怪癖按冻结判据逐值复刻，
    // 见 query.rs `PseudoClass::LastOfType`）。
    assert!(qs("div:first-child"), "s1 :first-child 修复前=T");
    assert!(qs("div:only-child"), "s1 :only-child 修复前=T");
    assert!(qs("div:nth-child(1)"), "s1 :nth-child(1) 修复前=T");
    assert!(qs("div:nth-child(odd)"), "s1 :nth-child(odd) 修复前=T");
    assert!(qs("div:last-child"), "s1 :last-child 修复前=T");
    assert!(qs("div:nth-last-child(1)"), "s1 :nth-last-child(1) 修复前=T");
    assert!(!qs("div:first-of-type"), "s1 :first-of-type 修复前=F（ti=0≠1）");
    assert!(
        qs("div:last-of-type"),
        "s1 :last-of-type 修复前=T（ti==tc 的 {{0,0}} 退化相等）"
    );
    assert!(!qs("div:only-of-type"), "s1 :only-of-type 修复前=F（tc=0≠1）");
    assert!(!qs("div:nth-of-type(1)"), "s1 :nth-of-type(1) 修复前=F");
    assert!(!qs("div:nth-child(2)"), "s1 :nth-child(2) 修复前=F");

    // s2：{ci=1, cc=1, ti=1, tc=1}——与 l1 同 tag 使 type 族同真。
    assert!(qs("span:first-child"), "s2 :first-child 修复前=T");
    assert!(qs("span:only-child"), "s2 :only-child 修复前=T");
    assert!(qs("span:nth-child(1)"), "s2 :nth-child(1) 修复前=T");
    assert!(qs("span:last-child"), "s2 :last-child 修复前=T");
    assert!(qs("span:nth-last-child(1)"), "s2 :nth-last-child(1) 修复前=T");
    assert!(qs("span:first-of-type"), "s2 :first-of-type 修复前=T");
    assert!(qs("span:only-of-type"), "s2 :only-of-type 修复前=T");
    assert!(qs("span:nth-of-type(1)"), "s2 :nth-of-type(1) 修复前=T");

    // matches() 单调用（无查询守卫）与查询路径同值（两路同源）。
    assert!(doc.matches(s2, "span:only-child"), "matches 单调用与查询路径同值");
    assert!(doc.matches(s1, "div:first-child"), "matches 单调用与查询路径同值");
    assert!(!doc.matches(s1, "div:first-of-type"), "matches 单调用与查询路径同值");

    // light DOM 对照：正常元素父路径保持规范语义。
    assert!(doc.matches(light_span, ":only-child"), "light 子 :only-child=T");
}

/// fixture 2：host 有 2 个 light 元素子（div#ld1, span#ld2），shadow root 有 1 个
/// a#sa。merge-base 推导：sa → {ci=2, cc=2, ti=0, tc=0}（'a' 在 host 子内计数 0）。
#[test]
fn p125_shadow_child_inherits_host_totals_like_merge_base() {
    let mut doc = Document::new();
    let root = doc.root();
    let host = doc.create_element("div");
    doc.append_child(root, host).unwrap();
    let ld1 = doc.create_element("div");
    doc.append_child(host, ld1).unwrap();
    let ld2 = doc.create_element("span");
    doc.append_child(host, ld2).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let sa = doc.create_element("a");
    doc.append_child(shadow, sa).unwrap();

    let qs = |sel: &str| doc.query_selector_shadow(shadow, sel).is_some();

    assert!(qs("a:nth-child(2)"), "sa :nth-child(2) 修复前=T（ci=cc=2）");
    assert!(qs("a:last-child"), "sa :last-child 修复前=T");
    assert!(!qs("a:first-child"), "sa :first-child 修复前=F");
    assert!(!qs("a:only-child"), "sa :only-child 修复前=F");
    assert!(!qs("a:nth-child(1)"), "sa :nth-child(1) 修复前=F");
    assert!(!qs("a:first-of-type"), "sa :first-of-type 修复前=F（ti=tc=0）");
    assert!(!qs("a:only-of-type"), "sa :only-of-type 修复前=F");
    assert!(qs("a:nth-child(even)"), "sa :nth-child(even) 修复前=T");

    // light DOM 对照保持规范语义。
    assert!(!doc.matches(ld1, ":only-child"), "ld1 :only-child=F");
    assert!(doc.matches(ld2, ":last-child"), "ld2 :last-child=T");
}
