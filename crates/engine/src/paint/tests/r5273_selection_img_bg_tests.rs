//! R5273（css-pseudo-4 §highlight-painting + Bugzilla 1449010）：**作者声明
//! `::selection` 背景的图片替换元素**在选区覆盖下以 0.8 alpha 叠加该底色——
//! driving: selection-image-001（mismatch vs noref，chromium 实证 TEST/REF
//! 差异只在图片：作者底覆写图片内容）。noref 侧（无作者规则）字节不动；
//! kill-switch `ZW_SELECTION_IMG_BG` 独立进程案见
//! crates/engine/tests/r5273_selection_img_bg_kill_switch.rs。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// 选区 tint 的签名 alpha（0.8 = chromium 红案逐通道自洽值）。
const TINT_ALPHA: u8 = 204;

fn img_page(style: &str) -> String {
    format!(
        r#"<html><head><style>{style}</style></head><body style="margin:0"><p style="margin:0">A <img src="blue.png" style="width:15px;height:15px"> B</p></body></html>"#
    )
}

/// selection-image-001 形：selectNode(html) 整 contents 选区 + 作者
/// `img::selection { background: green }` → 图片绘制区叠绿 tint（a=204）。
#[test]
fn img_author_selection_bg_tints_image() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("html".to_string(), 0, 100)]);
    let result = pipeline.render_html(&img_page("img::selection { background: green; }"), "");
    let prims = result.primitives();
    let tints: Vec<_> = prims
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(0, 128, 0, TINT_ALPHA))
        .collect();
    assert!(
        !tints.is_empty(),
        "作者 img::selection 绿底应叠 0.8 alpha tint，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        tints
            .iter()
            .any(|f| f.rect.size.width == 15.0 && f.rect.size.height == 15.0),
        "tint 应覆盖图片绘制区 15×15，实际 rects {:?}",
        tints.iter().map(|f| f.rect).collect::<Vec<_>>()
    );
}

/// selection-image-001-noref 形：整 contents 选区但**无作者规则**（UA 默认域）
/// → 图片无 tint，noref 侧字节不动（R5272 高亮 fill 照常）。
#[test]
fn img_without_author_rule_untinted() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("html".to_string(), 0, 100)]);
    let result = pipeline.render_html(&img_page(""), "");
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color.a == TINT_ALPHA),
        "无作者 ::selection 规则不应叠 tint，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 祖先链**部分**选区（首个有记录祖先非整 contents）→ 不叠 tint
/// （部分选区覆盖图片维持挂账）。
#[test]
fn img_partial_ancestor_selection_untinted() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("html".to_string(), 0, 3)]);
    let result = pipeline.render_html(&img_page("img::selection { background: green; }"), "");
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color.a == TINT_ALPHA),
        "部分选区不应叠 tint，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 无选区页（product 形）+ 作者规则 → 零扫描零 tint。
#[test]
fn img_no_selection_untinted() {
    let result = RenderPipeline::new(400.0, 300.0).render_html(&img_page("img::selection { background: green; }"), "");
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color.a == TINT_ALPHA),
        "无选区不应叠 tint，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}
