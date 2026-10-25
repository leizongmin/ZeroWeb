//! R5277 回归：frame-only 空 caption 在 auto 表的收缩 + 表宽下界（CSS2 §17.4.1/§17.5.2）。
//!
//! border-right-width-applies-to-015（✗3.84）：caption 只有 `border-right-width: 96px`
//! + `height: 96px`（width auto、无子盒无文本），auto 表 shrink-to-fit 时 caption 维持
//! fill 满视口、表缩到空 cell 残宽（9.6px min-content floor）→ 黑 border-right 画到视口
//! 右缘。修复双臂同轮：R5172 空 caption 收缩例外（父表 width:auto 且 frame-only）+
//! apply_table_size_constraints 的 caption 壳下界（同分类，两处口径一致）。
//!
//! 指定宽表的空 caption 维持 fill（margin-applies-to-015 chromium 实证，R5172 原判）、
//! 文本 caption 维持 fill（css-tables-3 caption 文本 intrinsic FIXME）不回归。

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

/// border-right-width-applies-to-015 形态：auto 表 + frame-only 空 caption
///（border-right 96 + height 96）。caption 与表都应收缩到 96（§17.5.2 auto 表
/// shrink-to-fit 以 caption 为内容界），黑条 96×96 落在表内容缘（旧缺陷：caption
/// fill 满视口、表缩到空 cell 残宽 9.6，黑条画到视口右缘）。
#[test]
fn r5277_frame_only_empty_caption_shrinks_in_auto_table() {
    let html = r#"<html><head><style>
#test { border-right-style: solid; border-right-width: 96px; display: table-caption; height: 96px; }
#table { display: table; }
#row { display: table-row; }
#cell { display: table-cell; }
</style></head><body>
<div id="table">
  <div id="test"></div>
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
    let cb = find_box(&result.root, caption).expect("caption 盒应存在");
    assert!(
        (cb.width - 96.0).abs() < 0.5,
        "frame-only 空 caption 在 auto 表应收缩到边框壳 96，实际 {}（旧缺陷 fill 满视口 784）",
        cb.width
    );
    let table = doc.get_element_by_id("table").expect("table 应存在");
    let tb = find_box(&result.root, table).expect("table 盒应存在");
    assert!(
        (tb.width - 96.0).abs() < 0.5,
        "auto 表应缩到 caption 壳下界 96，实际 {}（旧缺陷缩到空 cell 残宽 9.6，与 caption 96 不一致）",
        tb.width
    );
}

/// 指定宽表（width:200px）同形态对照：caption 维持 fill 200（R5172 chromium 实证）、
/// 表维持 200——壳下界臂仅 auto 表启用，不把指定宽表撑到 caption 壳。
#[test]
fn r5277_frame_only_empty_caption_keeps_fill_in_fixed_table() {
    let html = r#"<html><head><style>
#test { border-right-style: solid; border-right-width: 96px; display: table-caption; height: 96px; }
#table { display: table; width: 200px; }
#row { display: table-row; }
#cell { display: table-cell; }
</style></head><body>
<div id="table">
  <div id="test"></div>
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
    let cb = find_box(&result.root, caption).expect("caption 盒应存在");
    assert!(
        (cb.width - 200.0).abs() < 0.5,
        "指定宽表的空 caption 应维持 fill 表宽 200，实际 {}（不应收缩到壳 96）",
        cb.width
    );
    let table = doc.get_element_by_id("table").expect("table 应存在");
    let tb = find_box(&result.root, table).expect("table 盒应存在");
    assert!(
        (tb.width - 200.0).abs() < 0.5,
        "指定宽表应维持 200，实际 {}（壳下界臂不应在指定宽表启用）",
        tb.width
    );
}

/// 文本 caption（无子盒、有文本内容）在 auto 表维持 fill——frame-only 判定排除文本
/// caption（caption 文本 intrinsic 尺寸 FIXME 未实现，收缩会截文本）。
#[test]
fn r5277_text_caption_keeps_fill_in_auto_table() {
    let html = r#"<html><head><style>
#test { display: table-caption; }
#table { display: table; }
#row { display: table-row; }
#cell { display: table-cell; }
</style></head><body>
<div id="table">
  <div id="test">caption</div>
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
    let cb = find_box(&result.root, caption).expect("caption 盒应存在");
    assert!(
        (cb.width - 784.0).abs() < 0.5,
        "文本 caption 在 auto 表应维持 fill 784（文本 intrinsic FIXME，不收缩），实际 {}",
        cb.width
    );
}
