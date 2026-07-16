//! table layout / img intrinsic / float shrink 回归测试（从 engine.rs 抽出，保持 2000 行约束）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_css_parser::values::DisplayValue;
use zero_style_system::StyleSystem;

#[test]
fn test_table_styles_correct() {
    let html = r#"<html><body><table><tr><td>cell</td></tr></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);

    let root = doc.root();
    let mut stack = vec![root];
    let mut found_table = false;
    while let Some(nid) = stack.pop() {
        if let Some(style) = styles.get(&nid) {
            if let Some(n) = doc.get(nid) {
                if let zero_dom::NodeKind::Element(elem) = &n.kind {
                    if elem.local_name() == "table" {
                        found_table = true;
                        assert_eq!(style.display, DisplayValue::Table, "table should have display:table");
                    }
                }
            }
        }
        if let Some(n) = doc.get(nid) {
            stack.extend(n.children.iter().copied());
        }
    }

    assert!(found_table, "should find <table> element");
}

#[test]
fn test_table_layout_runs() {
    let html = r#"<html><body style="margin:0"><table style="width:200px"><tr><td style="width:100px;height:40px"></td><td style="width:100px;height:40px"></td></tr></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // Should not crash, and root should have non-zero size
    assert!(result.root.width > 0.0);
    assert!(result.root.height > 0.0);
}

/// `position:relative` 的 table-cell (td) 须应用 inset 偏移自身（CSS-position-3）。
///
/// table-cell 由 table.rs 定位，不经 taffy 正常流的 relative-inset 应用，故须在
/// 单元格定位处显式加 relative inset（镜像行/行组的 row_rel_dx/dy）。
/// 此前 td.relative 的 top/left 完全未应用（position-relative-table-td-{top,left} FAIL）。
#[test]
fn test_table_cell_relative_inset_applied() {
    use zero_dom::{NodeId, NodeKind};
    // 两张表：表 A 的 td 有 position:relative top:60px left:30px；表 B 的 td 正常。
    // 两表结构相同（单格 50x50），A 的 td 相对其正常单元格位置应偏移 (30, 60)。
    let html = r#"<html><body style="margin:0">
      <table id="a" style="border-collapse:collapse"><tr><td id="ra" style="position:relative;top:60px;left:30px;width:50px;height:50px"></td></tr></table>
      <table id="b" style="border-collapse:collapse"><tr><td id="rb" style="width:50px;height:50px"></td></tr></table>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // DOM 中按 id 找两个 td 的 NodeId
    let find_id = |tag_id: &str| -> Option<NodeId> {
        let mut stack = vec![doc.root()];
        while let Some(nid) = stack.pop() {
            if let Some(n) = doc.get(nid) {
                if let NodeKind::Element(e) = &n.kind {
                    if e.id.as_deref() == Some(tag_id) {
                        return Some(nid);
                    }
                }
                stack.extend(n.children.iter().copied());
            }
        }
        None
    };
    let ra = find_id("ra").expect("td#ra found");
    let rb = find_id("rb").expect("td#rb found");

    // 在 layout 树中按 node_id 找 box（相对各自 table content 的 x/y）
    fn find_box(b: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
        if b.node_id == Some(id) {
            return Some(b);
        }
        for c in &b.children {
            if let Some(found) = find_box(c, id) {
                return Some(found);
            }
        }
        None
    }
    let box_a = find_box(&result.root, ra).expect("layout box for td#ra");
    let box_b = find_box(&result.root, rb).expect("layout box for td#rb");

    // 两个 td 正常单元格位置都是各自 table content 原点 (0,0)。
    // relative td 应偏移 (left:30, top:60)；normal td 为 (0,0)。
    assert!(
        (box_a.x - 30.0).abs() < 1.0,
        "relative td 的 x 应含 left:30 inset（≈30），实际 {}",
        box_a.x
    );
    assert!(
        (box_a.y - 60.0).abs() < 1.0,
        "relative td 的 y 应含 top:60 inset（≈60），实际 {}",
        box_a.y
    );
    assert!(
        (box_b.x - 0.0).abs() < 1.0 && (box_b.y - 0.0).abs() < 1.0,
        "normal td 应在原点 (0,0)，实际 ({},{})",
        box_b.x,
        box_b.y
    );
}

/// R978：table-internal 容器（display:table-row-group）内**裸文本**须经 compute_final IFC 渲染。
/// CSS Tables §3.1 要求裸文本生成匿名 cell；ZW 未实现匿名 cell 生成（text node 非 LayoutBox
/// child），故作为 partial fix：让 compute_final 例外允许 table-internal 行/行组（含直接 text child）
/// 跑 IFC，使裸文本至少按容器 font/size 渲染（不再 orphan 渲染为 16px 默认）。
/// 旧实现 engine.rs:1007 is_block_level 不含 TableRowGroup → compute_final 早返 → 裸文本 orphan。
/// 驱动：css-tables/table-row-group-color-inheritance-001 oracle 8.99%→0.79%（200px green Ahem X）。
#[test]
fn test_table_row_group_bare_text_runs_ifc() {
    use zero_dom::{NodeId, NodeKind};
    let html = r#"<html><body style="margin:0">
      <div style="display:table"><div id="rg" style="display:table-row-group; font-size:50px; color:green">X</div></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let find_id = |tag_id: &str| -> Option<NodeId> {
        let mut stack = vec![doc.root()];
        while let Some(nid) = stack.pop() {
            if let Some(n) = doc.get(nid) {
                if let NodeKind::Element(e) = &n.kind {
                    if e.id.as_deref() == Some(tag_id) {
                        return Some(nid);
                    }
                }
                stack.extend(n.children.iter().copied());
            }
        }
        None
    };
    let rg = find_id("rg").expect("div#rg found");
    // 裸文本「X」的 NodeId（rg 的直接 text child）
    let text_id = doc
        .child_nodes(rg)
        .into_iter()
        .find(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))))
        .expect("bare text node child of row-group");
    fn find_box(b: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
        if b.node_id == Some(id) {
            return Some(b);
        }
        for c in &b.children {
            if let Some(found) = find_box(c, id) {
                return Some(found);
            }
        }
        None
    }
    let rg_box = find_box(&result.root, rg).expect("layout box for div#rg");
    // 裸文本「X」须触发 IFC：text_node_font_sizes 被填充（旧实现 compute_final 早返 → 空 map → orphan 16px）。
    // inline_layout 仅 pure-Ahem 存（text_node_font_sizes 是 font-无关的 IFC-ran 信号）。
    assert!(
        rg_box.text_node_font_sizes.contains_key(&text_id),
        "table-row-group with bare text should register text in text_node_font_sizes (R978 IFC fix); got empty"
    );
    // 且 font-size 应为容器继承值（50px），非 16px 默认（orphan 的症状）。
    let stored_fs = rg_box.text_node_font_sizes.get(&text_id).copied().unwrap_or(0.0);
    assert!(
        (stored_fs - 50.0).abs() < 2.0,
        "bare text font-size should be ~50 (inherited from row-group), got {}; orphan bug = 16",
        stored_fs
    );
}

/// `<img>` 无 width/height 属性时应使用解码固有尺寸（DC-11 替换元素固有尺寸）。
#[test]
fn test_img_intrinsic_size_from_decoded() {
    let html = r#"<html><body style="margin:0"><img src="logo.jpg"></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);

    // 模拟解码后的固有尺寸
    let img_id = doc
        .get_elements_by_tag_name("img")
        .into_iter()
        .next()
        .expect("img element exists");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (120.0, 90.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());

    // 在布局树中找到 img 盒，断言其尺寸 ≈ 解码固有尺寸
    let mut found = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(img_id) {
            found = Some((b.width, b.height));
            break;
        }
        stack.extend(b.children.iter());
    }
    let (w, h) = found.expect("img box found in layout tree");
    assert!((w - 120.0).abs() < 1.0, "img width should use intrinsic 120, got {w}");
    assert!((h - 90.0).abs() < 1.0, "img height should use intrinsic 90, got {h}");
}

/// 回归：CSS §10.3/§10.6 替换元素——`<img>` 仅设 CSS width（height auto）时，
/// height 应按固有宽高比从 width 推导，而非用固有绝对高度。
/// 旧 bug：正方形 SVG（intrinsic 441×441）+ width:80px 渲染成 80×441（巨高），
/// 致真实页面 logo（仅设 width 或 height）严重变形（wintertc logo）。
#[test]
fn test_img_width_set_height_auto_preserves_aspect() {
    let html = r#"<html><body style="margin:0"><img src="logo.svg" style="width:80px"></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (441.0, 441.0)); // 正方形固有尺寸
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());
    let mut found = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(img_id) {
            found = Some((b.width, b.height));
            break;
        }
        stack.extend(b.children.iter());
    }
    let (w, h) = found.expect("img box found");
    assert!((w - 80.0).abs() < 1.0, "img width should be 80 (CSS), got {w}");
    assert!(
        (h - 80.0).abs() < 1.5,
        "img height should be aspect-preserved ~80 (square @ width 80), got {h}"
    );
}

/// 对称：仅设 CSS height（width auto）时，width 按固有比例推导。
#[test]
fn test_img_height_set_width_auto_preserves_aspect() {
    let html = r#"<html><body style="margin:0"><img src="logo.svg" style="height:48px"></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (200.0, 100.0)); // 2:1 宽图
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());
    let mut found = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(img_id) {
            found = Some((b.width, b.height));
            break;
        }
        stack.extend(b.children.iter());
    }
    let (w, h) = found.expect("img box found");
    assert!((h - 48.0).abs() < 1.0, "img height should be 48 (CSS), got {h}");
    assert!(
        (w - 96.0).abs() < 1.5,
        "img width should be aspect-preserved ~96 (2:1 @ height 48), got {w}"
    );
}

/// R325：CSS §10 替换元素——`<img>` 同时显式设置 width 与 height 时，两者都必须生效，
/// 不得用固有宽高比强制（否则 taffy 会把 height 拉到 width 比例，忽略显式 height）。
/// 旧实现 `<img style="width:200px;height:50px">` 渲染成 200×200（height 被忽略）。
#[test]
fn test_img_both_width_height_set_no_aspect_enforcement() {
    let html = r#"<html><body style="margin:0"><img src="red.png" style="width:200px;height:50px"></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (100.0, 100.0)); // 正方形 intrinsic（ratio 1:1）
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());
    let mut found = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(img_id) {
            found = Some((b.width, b.height));
            break;
        }
        stack.extend(b.children.iter());
    }
    let (w, h) = found.expect("img box found");
    assert!((w - 200.0).abs() < 1.0, "img width should be 200 (CSS), got {w}");
    assert!(
        (h - 50.0).abs() < 1.0,
        "img height should be 50 (CSS, not aspect-forced 200), got {h}"
    );
}

/// width:auto 的浮动元素，块级子元素全 0 宽（如 visibility:collapse 的 flex item
/// 主尺寸归零，或空内容块）时，应 shrink-to-fit 收缩到 padding+border，
/// 而非撑满容器全宽。旧实现 `content_max_w > 0.0` 条件在此跳过收缩（R300/R301）。
#[test]
fn test_float_with_zero_width_block_child_shrinks() {
    use zero_css_parser::values::FloatValue;
    let html = r#"<html><body style="margin:0"><div style="float:left"><div style="width:0px;height:10px"></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找到 float div（float != None，含 width:0 块级子元素）
    let mut float_w = None;
    fn walk(b: &LayoutBox, out: &mut Option<f32>) {
        if out.is_some() {
            return;
        }
        if b.float != FloatValue::None && b.children.iter().any(|c| c.is_block_level) {
            *out = Some(b.width);
            return;
        }
        for c in &b.children {
            walk(c, out);
        }
    }
    walk(&result.root, &mut float_w);
    let w = float_w.expect("should find a float with block-level children");
    assert!(
        w < 50.0,
        "width:auto float with 0-width block child should shrink (<<800), got {w} (old bug left it full-width)"
    );
}

/// R1570 table-sizing 切片：max-height / max-width 不应压缩 table 的固有内容
/// （chromium 行为，css-tables §computing-the-table-height——行/列不因 max 而收缩）。
/// min-max-size-table-content-box v4（max-height:50px + 75px div）期望 table 内容
/// 高度保持 75（td 内容）而非被压到 max-height。显式 width/height 的 max 约束已在
/// 行/列分布中处理，此处仅守「内容 > max 时不收缩」。
#[test]
fn test_table_maxsize_does_not_shrink_intrinsic_content() {
    use zero_css_parser::values::LengthValue;
    use zero_dom::{NodeId, NodeKind};
    // table 内含 75px 高 div；max-height:50 不应把 table 高度压到 50。
    let html = r#"<html><body style="margin:0">
      <table id="t" style="border-spacing:0"><tr>
        <td style="padding:0"><div style="width:75px;height:75px"></div></td>
      </tr></table>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let mut styles = sys.compute_styles(&doc, &[]);
    // 找到 table 元素 NodeId，注入 max-height:50（复现 min-max-size-table-content-box v4）
    let mut table_id: Option<NodeId> = None;
    let mut stack = vec![doc.root()];
    while let Some(nid) = stack.pop() {
        if let Some(n) = doc.get(nid) {
            if let NodeKind::Element(e) = &n.kind {
                if e.local_name().eq_ignore_ascii_case("table") && table_id.is_none() {
                    table_id = Some(nid);
                }
            }
            stack.extend(n.children.iter().copied());
        }
    }
    let table_id = table_id.expect("table element found");
    if let Some(s) = styles.get_mut(&table_id) {
        s.max_height = LengthValue::Px(50.0);
    }
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find_box(b: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
        if b.node_id == Some(id) {
            return Some(b);
        }
        for c in &b.children {
            if let Some(found) = find_box(c, id) {
                return Some(found);
            }
        }
        None
    }
    let table_box = find_box(&result.root, table_id).expect("table box found");
    // div 内容 75px + td padding 0 = 行高 75；max-height:50 不应把 table 内容压到 50。
    assert!(
        table_box.height >= 75.0,
        "table content (75px div) must not be shrunk by max-height:50, got {}",
        table_box.height
    );
}
