//! R5460 主判例：viv 域混合内容容器（inline 内容 + 脱流子）块轴 extent 补偿——
//! css-writing-modes-3 §7.3 auto block-size = fit-content，块轴 extent =
//! max(自身 IFC Σ 列宽, Σ 流内子 margin-box extent)。taffy 交换帧对含 float 子树
//!（R4428 viv float 排除域）或 abs/fixed 子（交换帧 absolute sizing 域）的 viv 容器
//! 不给自身列宽贡献（塌 frame-only w=16），`viv_mixed_content_block_extent` 在
//! Final Inline Layout Pass 前补 extent（ZW_VIV_MIXED_EXTENT default-on）。
//!
//! chromium oracle（float-lft-orthog-htb-in-vrl-002/vlr-002 探针，800×600）：
//! .vp extent = 自身文本列 40 + border 16 = 56（含 float 不阻断，B 变体；float-only
//! 无行盒 extent=0 保持 16，C 变体）。驱动案 solo：vrl-002 3.25%→1.39%、vlr-002
//! 2.99%→1.24%（R5459 fill 只修行内轴后的最大残差项）。
//!
//! 同源 kill-switch 回退锚见 r5460_viv_mixed_content_block_extent_kill_switch.rs
//!（独立进程，R3858 教训）。谱系：R5459 fill（行内轴，本臂前置依赖——列断深度 =
//! content_height 需 fill 后可信）、R4445/R4469（5.5 纯 IFC 域同测量模型）、
//! R5457 float 右缘锚定（apply step 12.7 按最终 content_width 重锚，本臂改写自动
//! 跟随）。块流起始 = 右缘（vertical-rl）width grow 左缘左移保持块流位。

use zero_layout_engine::{LayoutEngine, LayoutResult};

const CSS: &str = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.f { background-color: blue; color: white; float: left; writing-mode: horizontal-tb; \
height: 40px; width: 200px; margin: 0; }";

fn compute_with(css: &str, html: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

fn compute(html: &str) -> LayoutResult {
    compute_with(CSS, html)
}

/// 从树中取盒：root→body→.vp。返回 (body.x, body.width, .vp)。
fn boxes(result: &LayoutResult) -> (f32, f32, &zero_layout_engine::types::LayoutBox) {
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
    (body.x, body.width, vp)
}

#[test]
fn mixed_content_container_grows_block_extent() {
    let html = r#"<html><body>
<div class="vp">VRL parent
<div class="f">HTB floated left</div>
</div>
</body></html>"#;
    let result = compute(html);
    let (_bx, bw, vp) = boxes(&result);
    // 块轴 extent = 自身文本列 40 + border 16 = 56（塌陷态 w=16 frame-only）。
    assert!(
        (vp.width - 56.0).abs() <= 0.5,
        ".vp 块轴 extent 补偿 56（塌陷态 16），实得 w={}",
        vp.width
    );
    assert!(
        (vp.content_width - 40.0).abs() <= 0.5,
        ".vp content_width = Σ 列宽 40（32px × line-height 1.25），实得 cw={}",
        vp.content_width
    );
    // 块流起始 = 右缘（vertical-rl）：grow 后右缘仍锚 body content 右缘
    //（子坐标 = 父 content-box 系：vp.x + vp.width == body.width；塌陷态两者同 16）。
    assert!(
        (vp.x + vp.width - bw).abs() <= 0.5,
        ".vp 右缘锚 body content 右缘（块流位保持），实得 x={} w={} body w={}",
        vp.x,
        vp.width,
        bw
    );
    // R5457 性质保持：float 右缘贴容器 content 右缘（本臂 content_width 0→40 后
    // apply step 12.7 重锚自动跟随：x = content_width − mr − w）。
    let float_box = vp
        .children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect(".vp float 子盒");
    assert!(
        (float_box.x + float_box.width - vp.content_width).abs() <= 0.5,
        "float 右缘贴 .vp content 右缘（R5457 锚定随 content_width 跟随），实得 x={} w={} \
         content_width={}",
        float_box.x,
        float_box.width,
        vp.content_width
    );
}

#[test]
fn pure_text_and_float_only_containers_untouched() {
    // V1 纯文本（无 float/abs 子）：taffy 语义已对（56），本臂不触碰（避免 R4445
    // 记录的 measure-vs-taffy 分歧翻红域）。
    let result = compute(r#"<html><body><div class="vp">VRL parent</div></body></html>"#);
    let (_bx, _bw, vp) = boxes(&result);
    assert!(
        (vp.width - 56.0).abs() <= 0.5,
        "纯文本 .vp 维持 taffy 56（gate 不误伤），实得 w={}",
        vp.width
    );
    assert!(vp.x.abs() <= 0.5, "纯文本 .vp 不位移（no-op），实得 x={}", vp.x);

    // V3 float-only（无行盒）：chromium C 变体 extent=0 保持 16，不补偿。
    let result = compute(r#"<html><body><div class="vp"><div class="f">f</div></div></body></html>"#);
    let (_bx, _bw, vp) = boxes(&result);
    assert!(
        (vp.width - 16.0).abs() <= 0.5,
        "float-only .vp 维持 16（无行盒 extent=0），实得 w={}",
        vp.width
    );
}

#[test]
fn abs_child_container_compensated_same_as_float() {
    // ref 臂形态（abs 子 + inline span）：abs 域 taffy 交换帧同样不给自身列宽贡献，
    // reftest 双臂自洽要求与本臂 float 域同步补偿（no-op 或 grow 至 56）。
    let html = r#"<html><body>
<div class="vp"><div style="position: absolute; top: 40px; right: 16px; background: red;">abs</div>
<span>VRL parent</span></div>
</body></html>"#;
    let result = compute(html);
    let (_bx, bw, vp) = boxes(&result);
    assert!(
        (vp.width - 56.0).abs() <= 0.5,
        "abs 子 .vp 块轴 extent 56（与 float 域同规则），实得 w={}",
        vp.width
    );
    assert!(
        (vp.x + vp.width - bw).abs() <= 0.5,
        ".vp 右缘锚 body content 右缘，实得 x={} w={} body w={}",
        vp.x,
        vp.width,
        bw
    );
}

#[test]
fn explicit_width_container_not_overridden() {
    // 显式物理宽（style.width 落 Px）：物理属性不换轴存储，本臂不得越权改写。
    let html = r#"<html><body>
<div class="vp" style="width: 30px;">VRL parent
<div class="f">HTB floated left</div>
</div>
</body></html>"#;
    let result = compute(html);
    let (_bx, _bw, vp) = boxes(&result);
    // 显式 width:30px 为 content-box 宽，border-box = 30 + 16 border = 46：
    // 本臂 gate（width auto 限定）不越权改写，taffy 显式宽产物保持。
    assert!(
        (vp.width - 46.0).abs() <= 0.5,
        "显式 width:30px 容器不被 extent 补偿越权（border-box 46），实得 w={}",
        vp.width
    );
}
