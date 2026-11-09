//! R5456 kill-switch 回退锚（独立测试进程——`ZW_ORTHO_HTB_MEASURE=0` 时正交 htb 叶
//! 恢复旧 vertical 测量路径：taffy float MaxContent 约束 INFINITY → vertical-rl 列
//! 右缘锚定 measured_width=inf → 物理高 inf → 后处理 inf−inf=NaN 复现；进程内
//! set_var 与并行单测互踩，R3858 教训——本文件仅此一测，独占进程环境）。
//!
//! 谱系与几何同 r5456_orthog_htb_float_measure.rs 主判例（002 形态）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

#[test]
fn kill_switch_restores_nan_path() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读前生效。
    unsafe { std::env::set_var("ZW_ORTHO_HTB_MEASURE", "0") };

    let html = r#"<html><body>
<div class="vp">VRL parent
<div class="f">HTB floated left</div>
</div>
</body></html>"#;
    let css = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.f { background-color: blue; color: white; float: left; writing-mode: horizontal-tb; margin: 0; }";
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result: LayoutResult = engine.compute(&doc, &styles);

    // 跳过 node_id=None 匿名盒（arm=0 时 html 匿名包裹在树，固定下标越界）。
    let float_box = result
        .root
        .children
        .iter()
        .find(|c| c.node_id.is_some())
        .and_then(|body| body.children.iter().find(|c| c.node_id.is_some()))
        .and_then(|vp| vp.children.iter().find(|c| c.node_id.is_some()))
        .expect("html > body > .vp > float 层级在树");
    assert!(
        float_box.height.is_nan(),
        "kill-switch=0 恢复旧行为：正交 float 物理高 inf−inf=NaN（#33 债复现），实得 h={}",
        float_box.height
    );
}
