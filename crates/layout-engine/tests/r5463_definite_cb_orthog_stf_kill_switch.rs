//! R5463 kill-switch 回退锚（独立进程，R3858 教训）：`ZW_ORTHO_BLOCK_STF=0` 时
//! 11.12 pass 整体关闭（含 R5463 definite-CB 扩展），复现旧塌陷——definite-CB 域
//! 正交块按父循环首趟收缩宽断行（w=66/cw=60，3 数/行），块轴塌 ICB fill（h=784）。
//! 主判例 r5463_definite_cb_orthog_stf.rs 断言相反方向。

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

fn find_ortho(root: &zero_layout_engine::types::LayoutBox) -> &zero_layout_engine::types::LayoutBox {
    fn walk(b: &zero_layout_engine::types::LayoutBox) -> Option<&zero_layout_engine::types::LayoutBox> {
        if !b.writing_mode.is_vertical_block_flow()
            && b.node_id.is_some()
            && b.is_block_level
            && b.border_left > 2.0
            && b.border_left < 4.0
        {
            return Some(b);
        }
        for c in &b.children {
            if let Some(f) = walk(c) {
                return Some(f);
            }
        }
        None
    }
    walk(root).expect("正交块盒在树")
}

#[test]
fn kill_switch_restores_legacy_collapse() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；pass 入口首读 env 前生效
    //（R5461/R5462 同款模式）。
    unsafe { std::env::set_var("ZW_ORTHO_BLOCK_STF", "0") };
    let nums: String = (1..=100)
        .map(|i| format!("{:02} ", if i == 100 { 0 } else { i }))
        .collect();
    let html = format!(r#"<html><body><div id="cb"><div id="ob">{nums}</div></div></body></html>"#);
    let css = "html { writing-mode: vertical-rl; } \
body { font-size: 16px; line-height: 1.25; margin-left: 0px; margin-right: 0px; } \
div#cb { width: 400px; } \
div#ob { border: blue solid 3px; width: auto; writing-mode: horizontal-tb; }";
    let r = compute(&html, css);
    let ob = find_ortho(&r.root);
    // 旧塌陷形态：w=66（content 60 按 3 数断行 + border 6），块轴塌 ICB fill 784。
    assert!(
        (ob.width - 66.0).abs() < 0.5,
        "kill-switch 应复现旧塌陷 w=66，实得 {}",
        ob.width
    );
    assert!(
        (ob.height - 784.0).abs() < 0.5,
        "kill-switch 应复现旧块轴塌 fill h=784，实得 {}",
        ob.height
    );
}
