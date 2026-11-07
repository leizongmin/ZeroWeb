//! R5457 kill-switch 回退锚（独立测试进程——`ZW_ORTHO_FLOAT_RL_ANCHOR=0` 时 RL 容器
//! 孤立正交 float 不做块流起始重锚：taffy 交换帧左起排原始几何维持，float:left
//! content 系 x≈0（002 test 臂旧物理 x=784 右出视口被裁的几何源）；进程内 set_var
//! 与并行单测互踩，R3858 教训——本文件仅此一测，独占进程环境）。
//!
//! 谱系与几何同 r5457_orthog_float_rl_anchor.rs 主判例（002 形态；R5456 测量修复
//! 独立 knob，几何仍有限）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

#[test]
fn kill_switch_restores_taffy_left_flow_position() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读前生效。
    unsafe { std::env::set_var("ZW_ORTHO_FLOAT_RL_ANCHOR", "0") };

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
    let (vp, float_box) = {
        let body = result
            .root
            .children
            .iter()
            .find(|c| c.node_id.is_some())
            .expect("body 盒（node_id 在树）");
        let vp = body
            .children
            .iter()
            .find(|c| c.node_id.is_some())
            .expect(".vp 盒（node_id 在树）");
        let float_box = vp
            .children
            .iter()
            .find(|c| c.node_id.is_some())
            .expect(".vp float 子盒（node_id 在树）");
        (vp, float_box)
    };
    assert!(
        float_box.x.abs() <= 0.5,
        "kill-switch=0 恢复旧行为：float:left 维持 taffy 交换帧左起排（content 系 \
         x≈0，右缘重锚关闭；#33 slice-2 002 出视口被裁几何源复现），实得 x={} \
         content_width={}",
        float_box.x,
        vp.content_width
    );
}
