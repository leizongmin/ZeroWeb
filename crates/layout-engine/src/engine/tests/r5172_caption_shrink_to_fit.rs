//! R5172 回归：table-caption 宽度 shrink-to-fit（CSS2 §17.4.1）。
//!
//! converter 把 TableCaption 映射为 taffy Block → caption 盒 fill 满包含块宽；
//! `width: auto` 的 caption 应 shrink-to-fit（padding-top/right-applies-to-015：
//! 200px 内容的 caption 被拉满视口，border 线错位 1.22%/1.67%）。无子盒的空
//! caption 维持 fill（margin-applies-to-015 chromium 实证，收缩致 border 缺边）。
//!
//! kill-switch `ZW_CAPTION_SHRINK_TO_FIT=0` 回退由 corpus OFF 轮承担（LazyLock
//! 进程级快照，进程内 set_var 不可靠——R3858 教训）。

fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn find_box(root: &crate::types::LayoutBox, id: zero_dom::NodeId) -> Option<&crate::types::LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        for c in b.children.iter() {
            stack.push(c);
        }
    }
    None
}

/// padding-right-applies-to-015 形态：caption（padding-right:50px 与 border-right:10px
/// 加内容 div width:200px）——caption border-box 宽应 = 210+50+10 = 270（shrink-to-fit，
/// 旧实现 fill 满视口 800，orange border 落到视口右缘）。
#[test]
fn r5172_caption_shrinks_to_content_with_padding_right() {
    let html = r#"<html><head><style>
#test { border-right: 10px solid orange; display: table-caption; padding-right: 50px; }
#test div { border-right: 10px solid blue; width: 200px; height: 200px; }
#table { display: table; }
#row { display: table-row; }
#cell { display: table-cell; }
</style></head><body>
<div id="table">
  <div id="test"><div></div></div>
  <div id="row"><div id="cell"></div></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let caption = doc.get_element_by_id("test").expect("caption 应存在");
    let b = find_box(&result.root, caption).expect("caption 盒应存在");
    assert!(
        (b.width - 270.0).abs() < 0.5,
        "caption 应 shrink-to-fit 到内容 div border-box 210+padding 50+border 10=270，实际 {}（旧缺陷 fill 满视口）",
        b.width
    );
}

/// padding-top-applies-to-015 形态：caption（padding-top:50px + border-top:10px +
/// 内容 div width:200px，水平无 frame）——caption 宽应 = 200（旧缺陷 fill 满视口，
/// blue border-top 线拉满 800 宽 1.22%）。
#[test]
fn r5172_caption_shrinks_to_content_width_only() {
    let html = r#"<html><head><style>
#test { border-top: 10px solid blue; display: table-caption; padding-top: 50px; }
#test div { border-top: 10px solid orange; width: 200px; }
#table { display: table; }
#row { display: table-row; }
#cell { display: table-cell; }
</style></head><body>
<div id="table">
  <div id="test"><div></div></div>
  <div id="row"><div id="cell"></div></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let caption = doc.get_element_by_id("test").expect("caption 应存在");
    let b = find_box(&result.root, caption).expect("caption 盒应存在");
    assert!(
        (b.width - 200.0).abs() < 0.5,
        "caption 应 shrink-to-fit 到内容 200（水平无 frame），实际 {}",
        b.width
    );
}

/// margin-applies-to-015 形态对照：无子盒的空 caption 维持 fill（chromium 实证：
/// 空 caption 宽 = 表可用宽，不收缩；收缩会致 border 缺边翻红）。表 width:320px
/// 无 border/padding → caption 宽 = 320。
#[test]
fn r5172_empty_caption_keeps_fill() {
    let html = r#"<html><head><style>
#wrapper { border: 10px solid blue; position: absolute; }
#test { border: 10px solid orange; display: table-caption; height: 200px; margin: 50px; }
#table { display: table; width: 320px; }
#row { display: table-row; }
#cell { display: table-cell; }
</style></head><body>
<div id="wrapper">
<div id="table">
  <div id="test"></div>
  <div id="row"><div id="cell"></div></div>
</div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let caption = doc.get_element_by_id("test").expect("caption 应存在");
    let b = find_box(&result.root, caption).expect("caption 盒应存在");
    assert!(
        (b.width - 220.0).abs() < 0.5,
        "无子盒空 caption 应维持 fill（表 content 320 − 水平 margin 100 = 220，chromium ref 形态），实际 {}（不应收缩）",
        b.width
    );
}
