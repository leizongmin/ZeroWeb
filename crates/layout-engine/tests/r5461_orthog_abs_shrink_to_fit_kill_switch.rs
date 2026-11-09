//! R5461 kill-switch 回退锚（独立进程，R3858 教训）：`ZW_ABSPOS_ORTHO_STF=0` 复现
//! 旧塌陷——正交 htb abs 叶行内 extent 塌最宽词（taffy R5293 回灌臂物理高反馈成水平
//! 约束域），宽度与内容长度无关。旧行为失败证据：002-ref 形态「应 263 实得 123」。

#[test]
fn kill_switch_restores_collapsed_width() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读前生效。
    unsafe { std::env::set_var("ZW_ABSPOS_ORTHO_STF", "0") };
    let html = r#"<html><body>
<div class="vp">
<div class="abs">HTB floated left</div>
<span>VRL parent</span>
</div>
</body></html>"#;
    let css = "html { writing-mode: vertical-rl; } \
div.vp { border: orange solid 8px; font-size: 32px; line-height: 1.25; } \
div.abs { background-color: blue; color: white; right: 16px; position: absolute; writing-mode: horizontal-tb; } \
span { position: relative; top: 40px; }";
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let r = engine.compute(&doc, &styles);
    let find_abs = |root: &zero_layout_engine::types::LayoutBox| -> f32 {
        fn walk(b: &zero_layout_engine::types::LayoutBox) -> Option<f32> {
            if b.is_absolute {
                return Some(b.width);
            }
            for c in &b.children {
                if let Some(w) = walk(c) {
                    return Some(w);
                }
            }
            None
        }
        walk(root).expect("abs 盒在树")
    };
    let width = find_abs(&r.root);
    assert!(
        (width - 123.0).abs() < 0.5,
        "kill-switch=0 应复现旧塌陷 123（塌最宽词），实得 {}",
        width
    );
}

use zero_layout_engine::LayoutEngine;
