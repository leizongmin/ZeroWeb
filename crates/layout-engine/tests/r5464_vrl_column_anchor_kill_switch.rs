//! R5464 kill-switch 回退锚（独立进程，R3858 教训）：`ZW_VRL_COLUMN_CB_ANCHOR=0`
//! 时 apply_inner 的 definite 容器 CB 右缘平移关闭，列组回旧 restack Σ 基准——
//! restack 在 11.12 宽重解后跑（用新列宽），Σ=440：sb [420,440]、ob [20,420]、
//! sa [0,20]，溢出 CB 400 右缘 40px。主判例 r5464_vrl_column_anchor.rs 断言相反
//! 方向。

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
fn kill_switch_restores_legacy_sigma_anchor() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；pass 入口首读 env 前生效
    //（R5461/R5462/R5463 同款模式）。
    unsafe { std::env::set_var("ZW_VRL_COLUMN_CB_ANCHOR", "0") };
    let nums: String = (1..=100)
        .map(|i| format!("{:02} ", if i == 100 { 0 } else { i }))
        .collect();
    let html = format!(
        r#"<html><body><div id="cb"><p>Sentence before.</p><div id="ob">{nums}</div><p>Sentence after.</p></div></body></html>"#
    );
    let css = "html { writing-mode: vertical-rl; } \
body { font-size: 16px; line-height: 1.25; margin-left: 100px; margin-right: 100px; } \
div#cb { width: 400px; } \
div#ob { border: blue solid 3px; width: auto; writing-mode: horizontal-tb; } \
p { margin: 1em 0; }";
    let r = compute(&html, css);
    let ob = find_ortho(&r.root);
    // 旧形态：restack 以 Σ=440 为右起基准（新列宽），sb [420,440]、ob [20,420]、
    // sa [0,20]——组溢出 CB 400 右缘 40px。
    assert!(
        (ob.x - 20.0).abs() < 0.5 && (ob.x + ob.width - 420.0).abs() < 0.5,
        "kill-switch 应复现 ob Σ 锚定位 [20,420]，实得 [{},{}]",
        ob.x,
        ob.x + ob.width
    );
    let mut xs: Vec<f32> = Vec::new();
    fn find_parent<'a>(
        b: &'a zero_layout_engine::types::LayoutBox,
        target: &zero_layout_engine::types::LayoutBox,
    ) -> Option<&'a zero_layout_engine::types::LayoutBox> {
        if b.children.iter().any(|c| std::ptr::eq(c, target)) {
            return Some(b);
        }
        for c in &b.children {
            if let Some(p) = find_parent(c, target) {
                return Some(p);
            }
        }
        None
    }
    let parent = find_parent(&r.root, ob).expect("正交块父容器在树");
    for c in &parent.children {
        if (c.width - 20.0).abs() < 1.0 {
            xs.push(c.x);
        }
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(xs.len(), 2, "两单行列在树");
    assert!(
        (xs[0] - 0.0).abs() < 0.5 && (xs[1] - 420.0).abs() < 0.5,
        "kill-switch 应复现 sentence 列 Σ 锚定位 {{0, 420}}，实得 {xs:?}"
    );
}
