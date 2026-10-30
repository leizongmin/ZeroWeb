//! `color-scheme` used-scheme 合成测试（CSS Color Adjust L1 §2.3）。
//!
//! 验证 used color-scheme 由 `color-scheme` 属性列表与 `prefers-color-scheme` 媒体查询
//! 合成：`light dark` + prefers=dark → dark（全局暗模式 theming 的核心）。
//! 经 `compute_styles`（生产路径 `lib.rs` 传 `self.prefers_color_scheme`）端到端验证。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;
use zero_css_parser::media_query::PrefersColorSchemeValue;
use zero_css_parser::values::ColorValue;

use crate::property::apply::parse_color_scheme_dark;

const GREEN: ColorValue = ColorValue::Rgba(0, 128, 0, 255);
const RED: ColorValue = ColorValue::Rgba(255, 0, 0, 255);

// ── 单元：parse_color_scheme_dark 合成矩阵 ──────────────────────────────

#[test]
fn test_parse_color_scheme_synthesis_matrix() {
    // 仅 dark → dark（不受 prefers 影响）
    assert!(parse_color_scheme_dark("dark", false));
    assert!(parse_color_scheme_dark("dark", true));
    assert!(parse_color_scheme_dark("only dark", false));
    assert!(parse_color_scheme_dark("dark only", true));

    // 仅 light → light
    assert!(!parse_color_scheme_dark("light", false));
    assert!(!parse_color_scheme_dark("light", true));
    assert!(!parse_color_scheme_dark("only light", true));

    // light + dark（两种均可用）→ 由 prefers 决定（★ 合成核心）
    assert!(!parse_color_scheme_dark("light dark", false));
    assert!(parse_color_scheme_dark("light dark", true));
    assert!(!parse_color_scheme_dark("dark light", false));
    assert!(parse_color_scheme_dark("dark light", true));
    assert!(parse_color_scheme_dark("only light dark", true));

    // normal / 缺省 / 仅 custom-ident → 保守 light（不因暗 OS 整体翻转未 opt-in 页面）
    assert!(!parse_color_scheme_dark("normal", true));
    assert!(!parse_color_scheme_dark("only", true));
    assert!(!parse_color_scheme_dark("", true));
    assert!(parse_color_scheme_dark("light dark weird", true)); // 含 light+dark 仍合成
}

// ── 端到端：compute_styles + set_prefers_color_scheme ──────────────────

/// 解析 CSS，按指定 prefers 跑 compute_styles，返回 div 的计算 background-color。
fn div_background_with_prefers(css: &str, prefers: PrefersColorSchemeValue) -> ColorValue {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let stylesheets = vec![CssParser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_prefers_color_scheme(prefers);
    let styles = sys.compute_styles(&doc, &stylesheets);
    styles.get(&div).expect("div 应有计算样式").background_color.clone()
}

#[test]
/// `color-scheme: light dark` + prefers=dark → light-dark 取 dark 参数 green。
/// 这是 R2285 light-dark() 与本轮 used-scheme 合成协作的全局暗模式核心场景。
fn test_light_dark_synthesizes_to_dark_when_prefers_dark() {
    let css = "div { color-scheme: light dark; background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_with_prefers(css, PrefersColorSchemeValue::Dark),
        GREEN,
        "color-scheme:light dark + prefers-color-scheme:dark 应合成 dark，light-dark 取 green"
    );
}

#[test]
/// `color-scheme: light dark` + prefers=light → light-dark 取 light 参数 red。
fn test_light_dark_synthesizes_to_light_when_prefers_light() {
    let css = "div { color-scheme: light dark; background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_with_prefers(css, PrefersColorSchemeValue::Light),
        RED,
        "color-scheme:light dark + prefers-color-scheme:light 应合成 light，light-dark 取 red"
    );
}

#[test]
/// `color-scheme: dark`（仅 dark）→ 不受 prefers 影响，恒取 dark 参数。
/// 回归守护：仅 dark 列表不因 prefers=light 退回 light。
fn test_color_scheme_dark_only_ignores_prefers_light() {
    let css = "div { color-scheme: dark; background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_with_prefers(css, PrefersColorSchemeValue::Light),
        GREEN,
        "color-scheme:dark 应恒为 dark，不受 prefers-color-scheme:light 影响"
    );
}

#[test]
/// 未声明 color-scheme（默认 normal）+ prefers=dark → 不翻转（保守 light）。
/// 守护：未 opt-in 的页面不应在暗 OS 上整体变暗。
fn test_undeclared_color_scheme_stays_light_under_dark_prefers() {
    let css = "div { background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_with_prefers(css, PrefersColorSchemeValue::Dark),
        RED,
        "未声明 color-scheme（normal）即使 prefers=dark 也应保守取 light"
    );
}

// ── meta color-scheme 文档 scheme 扫描（HTML §4.2.5.3，s165）────────────
//
// 根元素无 color-scheme 声明时 used-scheme 回落文档 scheme（首个树序有效 meta）；
// 根 computed color-scheme 序列化保持 normal（meta 不改根 computed 属性）；
// shadow tree 内 meta 不参与；显式 normal 声明回落文档 scheme（非 parent）。

use zero_dom::{Document, NodeId};

/// 组装带 head 内 meta color-scheme 的测试 DOM（meta 挂 html 下——扫描面为
/// 文档树序任意 light-DOM meta，head/body 位置不影响语义）。
fn meta_doc(contents: &[&str]) -> (Document, NodeId, Vec<NodeId>) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let mut metas = Vec::new();
    for content in contents {
        let meta = doc.create_element("meta");
        doc.set_attribute(meta, "name", "color-scheme");
        doc.set_attribute(meta, "content", content);
        doc.append_child(html, meta).unwrap();
        metas.push(meta);
    }
    (doc, html, metas)
}

#[test]
/// meta content=dark → 根 used-scheme dark（UA canvastext 翻白）且根 computed
/// color-scheme 保持 normal（decl 不落根——gCS 序列化断言面）。
fn test_meta_color_scheme_document_scheme_dark() {
    let (doc, html, _) = meta_doc(&["dark"]);
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);
    let root_style = styles.get(&html).expect("根应有计算样式");
    assert!(root_style.color_scheme_dark, "meta dark → 根 used-scheme dark");
    assert_eq!(root_style.color_scheme_decl, None, "meta 不落根 decl（序列化保持 normal）");
    assert_eq!(
        root_style.color,
        ColorValue::Rgba(255, 255, 255, 255),
        "UA canvastext 在 dark 文档 scheme 下翻白（CSS Color 4 系统色 used-scheme 面板）"
    );
}

#[test]
/// 无效 content（空串/逗号形）跳过继续，首个有效 meta 胜出（树序）。
fn test_meta_color_scheme_first_valid_wins() {
    let (doc, html, _) = meta_doc(&["", "light,dark", "light"]);
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);
    assert!(
        !styles.get(&html).expect("根应有计算样式").color_scheme_dark,
        "空串与 light,dark（逗号形无效）应跳过，首个有效 light 胜出"
    );
}

#[test]
/// shadow tree 内 meta 不参与文档 scheme（HTML §4.2.5.3 文档树限定）。
fn test_meta_color_scheme_shadow_tree_meta_ignored() {
    use zero_dom::ShadowRootMode;
    let (mut doc, html, _) = meta_doc(&[]);
    let host = doc.create_element("div");
    doc.append_child(html, host).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let meta = doc.create_element("meta");
    doc.set_attribute(meta, "name", "color-scheme");
    doc.set_attribute(meta, "content", "dark");
    doc.append_child(shadow, meta).unwrap();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);
    assert!(
        !styles.get(&html).expect("根应有计算样式").color_scheme_dark,
        "shadow tree 内 meta color-scheme 不应用——文档 scheme 保持 light"
    );
}

#[test]
/// 显式 `color-scheme: normal` 的后代回落文档 scheme（非 parent）——meta 改变后
/// 该后代随文档 scheme 翻转（语料 normal-descendant-change 钉死第二断言）。
fn test_explicit_normal_descendant_follows_document_scheme() {
    let (mut doc, html, _) = meta_doc(&["dark"]);
    let inner = doc.create_element("div");
    doc.set_attribute(inner, "style", "color-scheme: normal");
    doc.append_child(html, inner).unwrap();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);
    assert!(
        styles.get(&inner).expect("div 应有计算样式").color_scheme_dark,
        "显式 normal 后代应回落文档 scheme（dark）——继承语义不适用"
    );
    // meta 换 light（属性变更 → 全量重算入口重扫）→ 同一元素回落 light。
    doc.set_attribute(metas_of(&doc)[0], "content", "light");
    let styles = sys.compute_styles(&doc, &[]);
    assert!(
        !styles.get(&inner).expect("div 应有计算样式").color_scheme_dark,
        "meta 改 light 后显式 normal 后代应随文档 scheme 翻转"
    );
}

/// 取回文档树序全部 color-scheme meta（测试辅助）。
fn metas_of(doc: &Document) -> Vec<NodeId> {
    doc.get_elements_by_tag_name("meta")
}
