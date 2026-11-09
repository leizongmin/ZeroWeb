//! R5465 主判例：viewport-CB abspos inset `calc(<percentage> ± length)` 按 ICB 求值。
//!
//! 病理（sizing-orthog-htb-in-vrl-007 ref 臂残余 4.61%，R5465 归因）：abspos inset
//! 为含百分比 calc 时，`resolve_abspos_real_length` 走 `resolve_length` 旧路径——其
//! CalcContext 无 `parent_length` 百分比基准 → calc 内 % 不可解析 → 整式塌缩
//! `unwrap_or(0.0)`：`left: calc(100% - 136px - 400px - 136px)` 落 x=0（应 128）；
//! `max-width: calc(50% + 10px)` 同病塌缩 0 触发错误钳制。计算值阶段转换器对含 %
//! calc 保留信号（style-system converter `Calc if calc_contains_percentage` 臂），
//! 布局侧按 CB 对应轴消费（CSS2 §10.1：无 positioned ancestor → CB = ICB 视口；
//! css-values-4：calc 内 % 在 used-value 时按包含块解析）。
//!
//! 修复落点：`resolve_abspos_calc_inset`（水平 inset 基准 = CB 宽、垂直 = CB 高）
//! 替换 11.5 pass `adjust_absolute_pct_to_viewport` 的 left/right/top/bottom/max_width
//! 解析点及 R5376/R5377 方程 helper 的 `right` 解析。margin 解析点不动（% margin
//! 相对 CB 的既有口径维持——calc-% margin 与 % margin 同域另账 FIXME）。
//! kill-switch `ZW_ABSPOS_CALC_INSET`，回退锚见
//! r5465_abspos_calc_inset_kill_switch.rs（独立进程，R3858 教训）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

fn compute(html: &str, css: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 按 border-box 宽找目标 abs 盒（各判例仅一个定宽 abs 子）。
fn find_by_width(root: &zero_layout_engine::types::LayoutBox, w: f32) -> &zero_layout_engine::types::LayoutBox {
    fn walk(b: &zero_layout_engine::types::LayoutBox, w: f32) -> Option<&zero_layout_engine::types::LayoutBox> {
        if b.node_id.is_some() && (b.width - w).abs() < 0.5 {
            return Some(b);
        }
        for c in &b.children {
            if let Some(f) = walk(c, w) {
                return Some(f);
            }
        }
        None
    }
    walk(root, w).expect("目标 abs 盒在树")
}

#[test]
fn calc_left_resolves_against_viewport() {
    // left: calc(100% - 100px) → 视口 x = 800 - 100 = 700；body content 原点 x=8
    //（UA body margin 8px）→ 相对 x = 692。旧版本失败证据：「应 700 实得 0」。
    let r = compute(
        r#"<html><body><div id="a">xx</div></body></html>"#,
        "div#a { position: absolute; left: calc(100% - 100px); width: 50px; }",
    );
    let b = find_by_width(&r.root, 50.0);
    assert!(
        (b.x - 692.0).abs() < 0.5,
        "left:calc(100% - 100px) 应落相对 x=692（视口 700 − body 原点 8），实得 {}",
        b.x
    );
}

#[test]
fn calc_left_table_ref_form_at_128() {
    // 驱动案 ref 臂精确形态：table { position:absolute;
    // left: calc(100% - 136px - 400px - 136px); width: 672px } → 视口 x = 128，
    // 相对 x = 120。旧版本失败证据：「应 128 实得 0」（表贴视口左缘）。
    let r = compute(
        r#"<html><body><table id="t"><tr><td>xx</td></tr></table></body></html>"#,
        "table#t { position: absolute; left: calc(100% - 136px - 400px - 136px); width: 672px; border-spacing: 0; }",
    );
    let b = find_by_width(&r.root, 672.0);
    assert!(
        (b.x - 120.0).abs() < 0.5,
        "ref 臂形态 table 应落相对 x=120（视口 128 − body 原点 8），实得 {}",
        b.x
    );
}

#[test]
fn calc_top_uses_height_basis() {
    // 垂直 inset 基准 = CB 高：top: calc(50% + 10px) → 视口 y = 300 + 10 = 310，
    // 相对 y = 302。旧版本失败证据：「应 310 实得 0」（水平/垂直同病）。
    let r = compute(
        r#"<html><body><div id="a">xx</div></body></html>"#,
        "div#a { position: absolute; top: calc(50% + 10px); width: 50px; }",
    );
    let b = find_by_width(&r.root, 50.0);
    assert!(
        (b.y - 302.0).abs() < 0.5,
        "top:calc(50% + 10px) 应落相对 y=302（视口 310 − body 原点 8），实得 {}",
        b.y
    );
}

#[test]
fn calc_right_with_auto_left_right_anchors() {
    // left:auto + right:calc(100% - 200px)（= 600）→ 视口右锚：x = 800 − 600 − 0
    //（mr）− 50（宽）= 150，相对 x = 142（§10.3.18 rule 2 反解）。
    let r = compute(
        r#"<html><body><div id="a">xx</div></body></html>"#,
        "div#a { position: absolute; right: calc(100% - 200px); width: 50px; }",
    );
    let b = find_by_width(&r.root, 50.0);
    assert!(
        (b.x - 142.0).abs() < 0.5,
        "right:calc 应右锚至相对 x=142（视口 150 − body 原点 8），实得 {}",
        b.x
    );
}

#[test]
fn calc_max_width_clamps_stretch() {
    // width:auto + left/right:0 全 stretch（800）+ max-width: calc(50% + 10px)
    // → 钳 410（旧版本 mw 塌缩 0 → 盒宽 0：错误钳制域，同根因另一症状面）。
    let r = compute(
        r#"<html><body><div id="a">xx</div></body></html>"#,
        "div#a { position: absolute; left: 0; right: 0; max-width: calc(50% + 10px); }",
    );
    let b = find_by_width(&r.root, 410.0);
    assert!(
        (b.width - 410.0).abs() < 0.5,
        "stretch 宽应被 max-width:calc(50% + 10px) 钳至 410，实得 {}",
        b.width
    );
}

#[test]
fn px_inset_untouched() {
    // 纯 Px inset 不入 calc 臂（既有口径回归守卫）：left:128px → 相对 x = 120。
    let r = compute(
        r#"<html><body><div id="a">xx</div></body></html>"#,
        "div#a { position: absolute; left: 128px; width: 50px; }",
    );
    let b = find_by_width(&r.root, 50.0);
    assert!((b.x - 120.0).abs() < 0.5, "Px left 既有口径应维持 x=120，实得 {}", b.x);
}
