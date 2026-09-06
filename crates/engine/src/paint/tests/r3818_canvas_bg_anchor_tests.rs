//! R3818 回归：canvas 传播背景的 positioning area = 根元素（html）**padding-box**
//! （CSS2 §14.2——无论背景实际来自 html 传播还是 body fallback）。
//!
//! 旧实现 anchor = html border-box 原点（layout.x/y）；html 有 border 时相位错
//! border 值：background-root-007 test（html border 3px + body pos 0,0）tile 相位
//! 16 vs ref（html 无 border + body pos 2px）2 → 37.83% diff。修 = anchor 加 border。
//! chromium CDP 实测（2026-08-30）：007 两页 tile 网格相位一致 (2,2)；008/010 同规则。

use crate::pipeline::RenderPipeline;

/// body fallback + html border：positioning area 仍 = html padding-box（非 body 盒、
/// 非画布原点）。驱动 background-root-007/008/010（4 案翻绿）。
#[test]
fn r3818_body_fallback_bg_anchors_root_padding_box() {
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    // html{margin:16px; border:3px solid blue}（padding-box 原点 (19,19)）
    // body{background: linear-gradient(...) ; position 0,0; margin:0}
    // 若 anchor = html border-box (16,16)：gradient primitive 落 x=16；
    // 修复后 anchor = padding-box (19,19)：落 x=19。
    let html = "<html style=\"margin:16px; border:3px solid blue\">\
                <body style=\"margin:0; background-image:linear-gradient(to right, red, blue)\">\
                <p>x</p></body></html>";
    let result = pipeline.render_html(html, "");
    let grads = &result.primitives().gradients;
    assert!(!grads.is_empty(), "R3818: body fallback gradient 应传播到画布");
    // gradient positioned = anchor_x + offset（pos 默认 0%）→ left 应为 19（padding-box 原点）。
    let left = grads[0].rect.left();
    assert!(
        (left - 19.0).abs() < 1.0,
        "R3818: canvas bg anchor 应为根 padding-box 原点 x=19（border-box 16 + border 3），got {}",
        left
    );
}

/// html 传播 + html border：同一 padding-box 规则（对称面；html 自身有背景时）。
#[test]
fn r3818_html_propagation_bg_anchors_root_padding_box() {
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    // html{margin:16px; border:3px solid blue; background:linear-gradient(...)}
    let html = "<html style=\"margin:16px; border:3px solid blue; \
                background-image:linear-gradient(to right, red, blue)\">\
                <body style=\"margin:0\"><p>x</p></body></html>";
    let result = pipeline.render_html(html, "");
    let grads = &result.primitives().gradients;
    assert!(!grads.is_empty(), "R3818: html propagation gradient 应传播到画布");
    let left = grads[0].rect.left();
    assert!(
        (left - 19.0).abs() < 1.0,
        "R3818: html 传播 anchor 同为根 padding-box 原点 x=19，got {}",
        left
    );
}

/// R4083：canvas 传播渐变按根 padding-box 尺寸解析 background-size 并 repeat 平铺画布
/// （CSS Backgrounds §3.6/§14.2 + chromium oracle 实测：html 300px+margin 50px → tile 700×300
/// 锚定 (50,50)，repeat 铺满 800×600 画布；旧实现 origin=画布 → 单幅 800×600 拉伸）。
#[test]
fn r4083_canvas_propagation_gradient_tiles_by_root_padding_box() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html style=\"background-image:linear-gradient(lightblue, yellow); height:300px; margin:50px\">\
                <body style=\"margin:0\"></body></html>";
    let result = pipeline.render_html(html, "");
    let grads = &result.primitives().gradients;
    assert!(
        grads.len() >= 9,
        "R4083: 渐变 tile 应覆盖画布（≥9 个 3×3 网格），got {}",
        grads.len()
    );
    // 主 tile = 根 padding-box 尺寸 700×300 锚定 (50,50)。
    let anchor = grads
        .iter()
        .find(|g| (g.rect.origin.x - 50.0).abs() < 0.5 && (g.rect.origin.y - 50.0).abs() < 0.5)
        .expect("R4083: 应存在锚定 (50,50) 的主 tile");
    assert!(
        (anchor.rect.size.width - 700.0).abs() < 0.5 && (anchor.rect.size.height - 300.0).abs() < 0.5,
        "R4083: 主 tile 应为根 padding-box 700×300，got {:?}",
        anchor.rect.size
    );
    // 负向回退 tile（覆盖 margin 区）与正向 tile（覆盖画布尾部）存在 → repeat 铺满画布。
    let has_neg = grads.iter().any(|g| g.rect.origin.x < 0.0 || g.rect.origin.y < 0.0);
    let has_tail = grads
        .iter()
        .any(|g| g.rect.origin.x + g.rect.size.width > 750.0 || g.rect.origin.y + g.rect.size.height > 350.0);
    assert!(has_neg && has_tail, "R4083: repeat 应向两侧平铺覆盖整个画布");
}

/// R4083：根元素 scrollbar-gutter:stable both-edges 时，positioning area 收缩为
/// gutter 内侧的 padding box（css-overflow-4 §5.2 + oracle：渐变跨 x=[15,785]=770px），
/// painting area 同步排除 gutter 条带（条带为滚动容器 UI 领土，oracle 实测为白）。
#[test]
fn r4083_canvas_propagation_gutter_shrinks_positioning_area() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html style=\"height:37px; scrollbar-gutter:stable both-edges; background-image:linear-gradient(to right, green, blue)\"><body style=\"margin:0\"></body></html>";
    let result = pipeline.render_html(html, "");
    let grads = &result.primitives().gradients;
    assert!(
        grads.len() >= 2,
        "R4083: gutter 场景应存在 tile 发射，got {}",
        grads.len()
    );
    // 主 tile 宽 770（=800 − 2×15 gutter），锚定 x=15。
    let main = grads
        .iter()
        .find(|g| (g.rect.origin.x - 15.0).abs() < 0.5)
        .expect("R4083: 应存在锚定 x=15 的主 tile");
    assert!(
        (main.rect.size.width - 770.0).abs() < 0.5,
        "R4083: positioning area 应收缩为 770px（800−2×15），got {}",
        main.rect.size.width
    );
}

/// R4086（css-sizing-4 §7 + quirks spec body-fills-html）：quirks 模式下 body 自动高度视为
/// definite，子元素 `height:stretch` / `-webkit-fill-available` 解析为视口高（= ref 100vh）。
/// 驱动 stretch-quirk-001（chromium oracle diff 0.00%）。
#[test]
fn r4086_stretch_resolves_against_quirks_body_height() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    // 无 doctype → quirks 模式；body 高自动 → body-fills-html-which-fills-viewport（600）。
    let html = "<html><body style=\"margin:0\"><div style=\"height:stretch; width:40px\">x</div>\
                <div style=\"height:-webkit-fill-available; width:40px\"></div></body></html>";
    let result = pipeline.render_html(html, "");
    let root = &result.layout.root;
    let body = &root.children[root.children.len() - 1];
    let stretched: Vec<_> = body
        .children
        .iter()
        .filter(|c| (c.height - 600.0).abs() < 0.5)
        .collect();
    assert!(
        stretched.len() >= 2,
        "R4086: quirks 下 stretch/-webkit-fill-available 子元素高度应解析为视口高 600，got {:?}",
        body.children.iter().map(|c| c.height).collect::<Vec<_>>()
    );
}

/// R4086（css-sizing-4 §7）：确定 CB 下 `height:stretch` 解析为 CB 高（百分比式同链）。
#[test]
fn r4086_stretch_resolves_against_definite_cb() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    // standards 模式（有 doctype）+ 定高父容器：stretch 子 = 父内容高 300。
    let html = "<!DOCTYPE html><html><body style=\"margin:0\">\
                <div style=\"height:300px\"><div style=\"height:stretch; width:40px\">x</div></div>\
                </body></html>";
    let result = pipeline.render_html(html, "");
    let root = &result.layout.root;
    let outer = &root.children[root.children.len() - 1];
    let inner = &outer.children[outer.children.len() - 1];
    assert!(
        (inner.height - 300.0).abs() < 0.5,
        "R4086: 确定 CB 下 stretch 高应为 300，got {}",
        inner.height
    );
}
