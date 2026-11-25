// wptfyi-css k131 刀 常驻钉测（2026-11-26）：shadow defs——gCS 判读通道 shadow
// scopes 全集（外链条目 + 文档 shadow 树内联 `<style>`）＋shadow 表携文档级
// `@mixin` 聚合 parse（css-mixins-1 × css-scoping——zwsh 寻址面级联缺位修复）。
// 钉面四向：
//   D1 document defs → shadow 内 @apply（mixin-shadow-dom host1 形——gCS 静态
//      通道 zwsh 选择器查值 green）；
//   D2 shadow 内 apply 块先于定义块（host2 形——聚合先于展开两轮式）；
//   D3 adopted 汇总 defs → shadow 内 @apply（host3 形——adopted 尾接聚合；
//      线程局部 override 通道）；
//   D4 scope 定义不回灌 document 聚合（host4 隔离红线——document @apply 落旧
//      路径，绿保位）。
// https://drafts.csswg.org/css-mixins-1/#using-mixins
// https://drafts.csswg.org/css-scoping/#declarations-in-shadow-trees

/// 判据页形蒸馏（mixin-shadow-dom host1/host2/host4 原文缩样）。
const K131_HOST1_HTML: &str = "<html><head><style>@mixin --outside() { @result { color: rgb(0, 128, 0); } }</style></head>\
<body><div id=\"host\"><template shadowrootmode=\"open\"><style>#e1 { @apply --outside; }</style><div id=\"e1\">x</div></template></div></body></html>";

const K131_HOST2_HTML: &str = "<html><body><div id=\"host\"><template shadowrootmode=\"open\">\
<style>#e2 { @apply --m1; }</style><style>@mixin --m1() { @result { color: rgb(0, 128, 0); } }</style>\
<div id=\"e2\">x</div></template></div></body></html>";

const K131_HOST4_HTML: &str = "<html><body><div id=\"host\"><template shadowrootmode=\"open\">\
<style>@mixin --in-shadow() { @result { color: rgb(255, 0, 0); } }</style></template></div>\
<style>#e4 { color: rgb(0, 128, 0); @apply --in-shadow; }</style><div id=\"e4\">x</div></body></html>";

/// D1：document defs 对 shadow 内 `@apply` 可见（zwsh5:#host|#e1 gCS green）。
#[test]
fn test_kc5r131_shadow_apply_document_def_gcs() {
    let (doc, styles) = crate::js_dom_bridge::compute_document_styles(K131_HOST1_HTML);
    let color = crate::js_dom_bridge::lookup_computed_property(&doc, &styles, "zwsh5:#host|#e1", "color");
    assert_eq!(color, "rgb(0, 128, 0)", "D1 document defs visible in shadow apply, got {color:?}");
}

/// D2：shadow 内定义块在 apply 块之后——聚合先于展开，apply 跨表命中。
#[test]
fn test_kc5r131_shadow_apply_inside_def_after_apply_gcs() {
    let (doc, styles) = crate::js_dom_bridge::compute_document_styles(K131_HOST2_HTML);
    let color = crate::js_dom_bridge::lookup_computed_property(&doc, &styles, "zwsh5:#host|#e2", "color");
    assert_eq!(color, "rgb(0, 128, 0)", "D2 inside def visible to earlier apply block, got {color:?}");
}

/// D3：adopted 汇总 defs（shadowRoot.adoptedStyleSheets 槽文本）对 shadow 内
/// `@apply` 可见——adopted 尾接 document_mixin_defs 聚合（host3 形；reftest
/// harness 线程局部 override 通道装填，case 尾清槽）。
#[test]
fn test_kc5r131_shadow_apply_adopted_def_gcs() {
    let html = "<html><body><div id=\"host\"><template shadowrootmode=\"open\">\
<style>#e3 { @apply --adopted-only; }</style><div id=\"e3\">x</div></template></div></body></html>";
    crate::js_dom_bridge::set_active_adopted_css_render_override(Some(
        "@mixin --adopted-only() { @result { color: rgb(0, 128, 0); } }",
    ));
    let (doc, styles) = crate::js_dom_bridge::compute_document_styles(html);
    let color = crate::js_dom_bridge::lookup_computed_property(&doc, &styles, "zwsh5:#host|#e3", "color");
    crate::js_dom_bridge::set_active_adopted_css_render_override(None);
    assert_eq!(color, "rgb(0, 128, 0)", "D3 adopted defs visible in shadow apply, got {color:?}");
}

/// D4 红线：scope 定义不回灌 document 聚合——document `#e4` `@apply --in-shadow`
/// 落旧路径（@apply 条目留存，color: rgb(0, 128, 0) 保位）。
#[test]
fn test_kc5r131_shadow_def_not_leaking_to_document() {
    let (doc, styles) = crate::js_dom_bridge::compute_document_styles(K131_HOST4_HTML);
    let color = crate::js_dom_bridge::lookup_computed_property(&doc, &styles, "#e4", "color");
    assert_eq!(color, "rgb(0, 128, 0)", "D4 scope def isolated from document, got {color:?}");
    // 阴性对照：shadow 内元素无可见 @apply 污染（隔离面自证）。
    let inner = crate::js_dom_bridge::lookup_computed_property(&doc, &styles, "zwsh5:#host|div", "color");
    assert_ne!(inner, "rgb(255, 0, 0)", "D4 scope def must not style via leak path, got {inner:?}");
}
