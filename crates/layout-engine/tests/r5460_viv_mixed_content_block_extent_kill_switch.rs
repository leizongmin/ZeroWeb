//! R5460 kill-switch 回退锚（独立测试进程——`ZW_VIV_MIXED_EXTENT=0` 时 viv 域混合
//! 内容容器不做块轴 extent 补偿：塌陷态几何维持 w=16 frame-only＝002 左橙条
//! 776..784 错位几何源；进程内 set_var 与并行单测互踩，R3858 教训——本文件仅此一测，
//! 独占进程环境）。
//!
//! 谱系与几何同 r5460_viv_mixed_content_block_extent.rs 主判例（002 形态）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

#[test]
fn kill_switch_restores_collapsed_block_extent() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读前生效。
    unsafe { std::env::set_var("ZW_VIV_MIXED_EXTENT", "0") };

    let html = r#"<html><body>
<div class="vp">VRL parent
<div class="f">HTB floated left</div>
</div>
</body></html>"#;
    let css = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.f { background-color: blue; color: white; float: left; writing-mode: horizontal-tb; \
height: 40px; width: 200px; margin: 0; }";
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result: LayoutResult = engine.compute(&doc, &styles);

    let vp = {
        let body = result
            .root
            .children
            .iter()
            .find(|c| c.node_id.is_some())
            .expect("body 盒（node_id 在树）");
        body.children
            .iter()
            .find(|c| c.node_id.is_some())
            .expect(".vp 盒（node_id 在树）")
    };
    // 塌陷态：taffy 交换帧 frame-only，w = 16 纯边框，content_width = 0
    //（＝002 左橙条 776..784 错位几何源；R5459 fill 只修行内轴 h=784/ch=768 不变）。
    assert!(
        (vp.width - 16.0).abs() <= 0.5,
        "kill-switch=0 恢复塌陷态 w=16（frame-only，左橙条错位几何源），实得 w={}",
        vp.width
    );
    assert!(
        vp.content_width.abs() <= 0.5,
        "kill-switch=0 content_width=0，实得 cw={}",
        vp.content_width
    );
}
