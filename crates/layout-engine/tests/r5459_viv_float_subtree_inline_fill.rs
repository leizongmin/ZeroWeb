//! R5459 主判例：viv 域 float 子树容器行内轴 fill——CSS Writing Modes §7.1 块级盒
//! auto 行内尺寸 = fill 包含块行内尺寸，float 子不阻断（chromium oracle：002 test 臂
//! .vp h=497 = body content height 精确 fill）。taffy 交换帧对此类容器不 stretch
//!（float 子不入同 BFC → float contribution sizing，塌 h=56/ch=40），
//! `fill_viv_float_subtree_inline_axis` 在 Final Inline Layout Pass 前补 fill
//!（ZW_VIV_INLINE_FILL default-on），使垂直 IFC `max_depth = container_width`
//! 直接见正确值（002 可见残差主项＝逐词断列 2 列）。
//!
//! 同源 kill-switch 回退锚见 r5459_viv_float_subtree_inline_fill_kill_switch.rs
//!（独立进程，R3858 教训）。谱系：R5457（float 块流起始右缘锚定，本判例同时钉
//! fill 后 float 贴缘性质保持）、R4446（正交域 fill 臂，显式排除 float 子树）。
//! 塌陷机制＝taffy same-BFC 排除 float（block.rs is_in_same_bfc）→ float contribution
//! sizing；与 R4428 viv 打包排除无关（ZW_VIV_SIZING=0 不恢复 fill，R5459 勘误）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

const HTML: &str = r#"<html><body>
<div class="vp">VRL parent
<div class="f">HTB floated left</div>
</div>
</body></html>"#;

const CSS: &str = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.f { background-color: blue; color: white; float: left; writing-mode: horizontal-tb; \
height: 40px; width: 200px; margin: 0; }";

/// 同构对照（无 float 子）：.vp 已由 taffy 交换帧 stretch 正确 fill，本臂 gate
///（viv_subtree_has_float）不得触碰——回归哨戒误伤非 float 子树。
const HTML_NO_FLOAT: &str = r#"<html><body>
<div class="vp">VRL parent
</div>
</body></html>"#;

fn compute(html: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(CSS);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

#[test]
fn viv_float_subtree_container_fills_inline_axis() {
    let result = compute(HTML);
    // body：html content 系 (8,8)，800 视口 − 16 margin = 784（only-grow：body 自身
    // 子树含 float 但 784 ≥ html content_height，fill 臂不收缩既有 ≥ fill 域）。
    let (body, vp, float_box) = {
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
        (body, vp, float_box)
    };
    assert!(
        (body.height - 784.0).abs() <= 0.5,
        "body 既有 784 不得被 only-grow 收缩，实得 h={}",
        body.height
    );
    // .vp 行内轴 fill：h = body content_height 784（margin 0），ch = fill − 16 border。
    assert!(
        (vp.height - 784.0).abs() <= 0.5,
        ".vp 行内轴 fill body content height 784（塌陷态 h=56），实得 h={}",
        vp.height
    );
    assert!(
        (vp.content_height - 768.0).abs() <= 0.5,
        ".vp content_height = fill − frame 768（垂直 IFC max_depth 源），实得 ch={}",
        vp.content_height
    );
    // R5457 性质保持：行内轴 fill ≠ float 流几何——float 贴容器 content 右缘
    //（content_width=0 系 x = −200，右缘 x+w = 0 = content_width）。
    assert!(
        (float_box.x + float_box.width - vp.content_width).abs() <= 0.5,
        "float 右缘贴 .vp content 右缘（R5457 锚定不被 fill 改写），实得 x={} w={} \
         content_width={}",
        float_box.x,
        float_box.width,
        vp.content_width
    );
}

#[test]
fn non_float_subtree_untouched_by_fill_gate() {
    let result = compute(HTML_NO_FLOAT);
    let vp = {
        let body = result
            .root
            .children
            .iter()
            .find(|c| c.node_id.is_some())
            .expect("body 盒");
        body.children.iter().find(|c| c.node_id.is_some()).expect(".vp 盒")
    };
    // 非 float 子树：taffy 交换帧 stretch 已 fill 784，本臂不触碰（同值断言＝哨戒）。
    assert!(
        (vp.height - 784.0).abs() <= 0.5,
        "无 float 子树 .vp 维持 taffy fill 784（gate 不误伤），实得 h={}",
        vp.height
    );
}
