//! R5465 kill-switch 回退锚（独立进程，R3858 教训）：`ZW_ABSPOS_CALC_INSET=0` 时
//! viewport-CB abspos inset calc() 的百分比基准求值关闭，`left: calc(100% - 100px)`
//! 回旧 `resolve_length` 路径——CalcContext 无 `parent_length` 基准 → calc 内 % 不可
//! 解析 → 整式塌缩 0 → 盒贴视口左缘（视口 x=0，相对 x = −8）。主判例
//! r5465_abspos_calc_inset.rs 断言相反方向（相对 x=692）。

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

fn find_by_width(
    root: &zero_layout_engine::types::LayoutBox,
    w: f32,
) -> &zero_layout_engine::types::LayoutBox {
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        w: f32,
    ) -> Option<&zero_layout_engine::types::LayoutBox> {
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
fn kill_switch_restores_legacy_collapse() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；LazyLock 首读 env 前生效
    //（R5461/R5462/R5463/R5464 同款模式）。
    unsafe { std::env::set_var("ZW_ABSPOS_CALC_INSET", "0") };
    let r = compute(
        r#"<html><body><div id="a">xx</div></body></html>"#,
        "div#a { position: absolute; left: calc(100% - 100px); width: 50px; }",
    );
    let b = find_by_width(&r.root, 50.0);
    // 旧塌缩形态：calc 整式 unwrap_or(0.0) → left=0 → 相对 x = 0 − body 原点 8 = −8
    //（视口 x=0，应 700——主判例域）。
    assert!(
        (b.x + 8.0).abs() < 0.5,
        "kill-switch 应回旧塌缩形态（相对 x=−8），实得 {}",
        b.x
    );
}
