//! writing-mode / flex-item float / abspos static-position 回归测试（从 engine.rs 抽出，保持 2000 行约束）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 回归：`remeasure_inline_only_containers` 在子元素 inline 重测量使「高度」收缩时，
/// 无条件地把后续普通流兄弟按收缩量上移 `sibling.y += shrink_delta`。
/// 该逻辑仅适用于水平书写模式（块流方向为 y 轴）。在垂直书写模式中块流方向为 x 轴、
/// 「高度」是 inline 轴跨度，inline 轴收缩不在块轴留空隙，不应移动块兄弟；
/// 旧代码会把兄弟推到负 y（屏幕外），导致 writing-mode:vertical-rl 根页面整页空白
///（如 box-offsets-rel-pos-vrl-004）。
#[test]
fn test_vertical_rl_block_sibling_not_pushed_offscreen() {
    // 复刻 box-offsets-rel-pos-vrl-004 的 body 结构（含 4 个相对定位小块兄弟）。
    // 垂直书写模式下 taffy 把 <p> 的 inline 轴高度赋成接近 body 内容高度，
    // IFC 重测量后大幅收缩，旧代码会把后续块兄弟按收缩量推到负 y（整页空白）。
    let html = r#"<html style="writing-mode:vertical-rl"><body>
      <p><img src="p.png" width="304" height="35" /></p>
      <div style="width:100px;height:100px;padding:50px;border:50px solid orange;margin-right:8px;position:static"><img src="l.png" width="100" height="100" /></div>
      <div style="width:25px;height:25px;position:relative;writing-mode:horizontal-tb;left:75px;top:50px">TL</div>
      <div style="width:25px;height:25px;position:relative;writing-mode:horizontal-tb;left:275px;top:50px">TR</div>
      <div style="width:25px;height:25px;position:relative;writing-mode:horizontal-tb;left:125px;top:225px">BL</div>
      <div style="width:25px;height:25px;position:relative;writing-mode:horizontal-tb;left:325px;top:225px">BR</div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 任何普通流盒子都不应被推到视口之上（负 y）。
    fn any_offscreen_top(b: &LayoutBox) -> bool {
        if !b.is_absolute && !b.is_fixed && b.y < -0.5 {
            return true;
        }
        b.children.iter().any(any_offscreen_top)
    }
    assert!(
        !any_offscreen_top(&result.root),
        "vertical-rl page has in-flow boxes pushed off-screen (negative y)"
    );
}

/// 回归：CSS Flexbox §4 / Grid §4 / Tables §2.4 规定，flex/grid/table 容器的
/// 流内子元素（布局项）其 `float` 不产生浮动效果——`float` 计算为 `none`。
/// 旧代码的浮动后处理按 `child.float` 重新定位，把带 `float:right` 的 flex item
/// 误推到容器右缘（css-flexbox-test1 / css-flexbox-row）。布局容器父级的直接子元素
/// 的 `float` 应被归零，使后处理（含 paint 的 float 排斥/绘制）一致忽略它。
#[test]
fn test_flex_item_float_is_ignored() {
    // 复刻 css-flexbox-test1 结构：flex 容器 + 带 float:right 的子元素。
    // float:right 在非 flex 容器中会把元素推到右缘；在 flex 容器中应被忽略。
    let html = r#"<html><body style="margin:0">
      <div style="display:flex; width:600px; height:100px">
        <div id="a" style="width:100px; height:50px; float:right; background:orange">A</div>
        <div id="b" style="width:100px; height:50px; background:blue">B</div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 定位 flex 容器（display:flex → is_layout_container=true）的两个子元素。
    fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
        if let Some(nid) = b.node_id
            && let Some(n) = doc.get(nid)
            && let zero_dom::NodeKind::Element(elem) = &n.kind
            && elem.get_attribute("id").as_deref() == Some(id)
        {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, doc, c))
    }
    let a = find("a", &doc, &result.root).expect("flex item #a");
    let b = find("b", &doc, &result.root).expect("flex item #b");

    // float:right 被忽略：A 不应被推到容器右缘（~600）而与 B 相邻排列在左侧。
    // 若 float 仍生效，A.x 会接近容器宽度（500+）。
    assert!(
        a.x < 200.0,
        "flex item with float:right should not be floated to right edge (a.x={})",
        a.x
    );
    // B（无 float）位置不受影响。
    assert!(b.x < 300.0, "flex item B position sane (b.x={})", b.x);
}

/// 回归：vertical-rl 容器内 abspos 子元素 height:auto 应 shrink-to-fit 到内容
/// inline 跨度（CSS §10.3.7 + writing-modes §7.1），而非填满 CB cross-axis。
///
/// 复刻 abs-pos-non-replaced-vrl-006 结构。taffy 把 abspos auto height 当
/// cross-axis stretch（给 320=CB 高），fix_vertical_mode_abs_pos 应收缩到
/// 内容（单 80px 字形的 inline 跨度）。
#[test]
fn test_abspos_vertical_rl_height_auto_shrink_to_fit() {
    let html = r#"<html style="writing-mode:vertical-rl"><body>
      <div id="cb" style="background:red; direction:ltr; font:80px/1 monospace; height:320px; width:320px; position:relative">12<span id="s" style="position:absolute; top:auto; bottom:auto; height:auto; color:green">X</span></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
        if let Some(nid) = b.node_id
            && let Some(n) = doc.get(nid)
            && let zero_dom::NodeKind::Element(elem) = &n.kind
            && elem.get_attribute("id").as_deref() == Some(id)
        {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, doc, c))
    }
    let s = find("s", &doc, &result.root).expect("span #s");
    assert!(s.is_absolute, "span should be absolute");
    // 旧 bug：height=320（填满 CB cross-axis）。修复后应 shrink 到内容（80px 字形）。
    assert!(
        s.height < 200.0,
        "abspos height:auto in vertical-rl should shrink-to-fit to content, not fill CB (got h={})",
        s.height
    );
    assert!(
        s.height >= 79.0 && s.height <= 81.0,
        "expected content height ~80px (single 80px glyph), got h={}",
        s.height
    );
}

/// 回归：vertical-rl + direction:rtl 下 abspos 静态位置沿 inline 轴镜像（R334）。
///
/// 复刻 abs-pos-non-replaced-vrl-002(ltr)/vrl-012(rtl) 结构。CSS §10.3.7 + writing-modes §7.1：
/// all-three-auto（top/bottom 即映射后的 left/right 均 auto）下，ltr 把 inline-start 边置
/// 静态位置、rtl 把 inline-end 边置静态位置，两者最终盒位沿 inline 轴镜像：
///   rtl_top + ltr_top + height == CB_inline_extent
/// 旧实现 rtl 与 ltr 渲染完全相同（direction 被忽略）。
#[test]
fn test_abspos_vertical_rl_direction_rtl_mirrors_inline_position() {
    fn span_y(direction: &str) -> (f32, f32) {
        let html = format!(
            r#"<html style="writing-mode:vertical-rl"><body>
      <div id="cb" style="background:red; direction:{dir}; font:80px/1 monospace; height:320px; width:320px; position:relative">1 2 34<span id="s" style="position:absolute; top:auto; bottom:auto; height:auto; color:green">X</span></div>
    </body></html>"#,
            dir = direction
        );
        let doc = zero_dom::parse_html(&html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let zero_dom::NodeKind::Element(elem) = &n.kind
                && elem.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let cb = find("cb", &doc, &result.root).expect("cb");
        let s = find("s", &doc, &result.root).expect("span #s");
        // span 相对 CB 的 inline 轴（视觉 y）位置与高度
        (s.y - cb.y, s.height)
    }
    let (ltr_y, h) = span_y("ltr");
    let (rtl_y, _) = span_y("rtl");
    // CB inline extent（视觉高度）= 320px（CB height，无 padding/border）。
    let cb_inline_extent = 320.0;
    // ltr 与 rtl 必须不同（direction 必须生效）。
    assert_ne!(
        ltr_y, rtl_y,
        "direction:rtl must change abspos static position (ltr_y={ltr_y}, rtl_y={rtl_y})"
    );
    // 镜像不变式：rtl_top + ltr_top + height == CB inline extent。
    assert!(
        (ltr_y + rtl_y + h - cb_inline_extent).abs() < 0.5,
        "rtl must mirror ltr along inline axis: ltr_y({ltr_y}) + rtl_y({rtl_y}) + h({h}) should == {cb_inline_extent}"
    );
}

/// 回归：CSS §10.3.3 — 根元素 margin-left/right 均为 auto 且边框盒宽度小于视口时，
/// 应水平居中。taffy 对嵌套 block 正确处理 auto margin 居中，但对根节点不应用
///（根无父级提供居中上下文）。验证 compute() 的根居中补丁（html-display-table-ref 用例）。
#[test]
fn test_root_block_margin_auto_centers() {
    // 根 html 显式窄宽度 + margin:auto → 应居中于 800px 视口（同 html-display-table-ref）。
    let html = r#"<html style="width:280px;margin:auto"><body style="margin:0">
      <div id="t" style="width:280px;height:300px;background:yellow"></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // html 根边框盒宽度 = 280，居中偏移 = (800-280)/2 = 260。
    let html_box = &result.root;
    assert!(
        (html_box.x - 260.0).abs() < 1.0,
        "root block with width<viewport + margin:auto should be centered (x≈260, got {})",
        html_box.x
    );
}

/// 回归：CSS §17.5.2 — display:table 容器 width:auto 收缩到内容后，margin:auto
/// 应居中。验证 shrink_table_to_block_content 的居中补丁（html-display-table 用例）。
#[test]
fn test_display_table_margin_auto_centers() {
    // html display:table 内含窄内容（200+80 inline-block），margin:auto → 收缩并居中。
    let html = r#"<html style="display:table;margin:auto;border:10px solid green"><body style="margin:0">
      <div style="width:200px;height:50px;display:inline-block"></div><div style="width:80px;height:50px;display:inline-block"></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let html_box = &result.root;
    // 内容 ≈ 280 + 20 border = 300；居中偏移 = (800-300)/2 = 250。
    assert!(
        html_box.x > 200.0,
        "display:table with margin:auto should be centered (x>200, got {}) — shrink-to-fit centering missing",
        html_box.x
    );
    assert!(
        html_box.width < 400.0,
        "display:table should shrink to content (width<400, got {}) — shrink missing",
        html_box.width
    );
}

/// R1108 α-4b-1 回归：vertical-rl 表的行沿 x 右到左、cell 沿 y 顶到底（转置）。
/// 验证 position_cells_vertical 的轴语义（RFC vertical-mode-table-rl-transpose §2）。
#[test]
fn test_r1108_vertical_rl_table_rows_right_to_left() {
    let html = r#"<html><body style="margin:0">
      <table style="writing-mode:vertical-rl; border-spacing:0; font:20px/1 Ahem">
        <tr><td>A</td><td>B</td><td>C</td></tr>
        <tr><td>D</td><td>E</td><td>F</td></tr>
      </table>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 收集所有 td 的绝对位置（x,y）。
    let mut tds: Vec<(String, f32, f32)> = Vec::new();
    fn collect_tds(box_node: &LayoutBox, doc: &Document, abs_x: f32, abs_y: f32, out: &mut Vec<(String, f32, f32)>) {
        for child in &box_node.children {
            let x = abs_x + child.x;
            let y = abs_y + child.y;
            let tag = child.node_id.and_then(|nid| doc.get(nid)).and_then(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => Some(e.local_name().to_string()),
                _ => None,
            });
            if tag.as_deref() == Some("td") {
                let text = child
                    .node_id
                    .and_then(|nid| doc.get(nid))
                    .and_then(|n| {
                        n.first_child().and_then(|_| {
                            // 取 cell 的文本内容首字符
                            let c = n.first_child()?;
                            let cn = doc.get(c)?;
                            match &cn.kind {
                                zero_dom::NodeKind::Text(t) => {
                                    Some(t.content.chars().next().unwrap_or('?').to_string())
                                }
                                _ => None,
                            }
                        })
                    })
                    .unwrap_or_else(|| "?".to_string());
                out.push((text, x, y));
            }
            let cx = x + child.content_x;
            let cy = y + child.content_y;
            collect_tds(child, doc, cx, cy, out);
        }
    }
    collect_tds(&result.root, &doc, 0.0, 0.0, &mut tds);

    // 按 text 排序定位 A..F。
    let pos = |label: &str| -> (f32, f32) {
        tds.iter()
            .find(|(t, _, _)| t == label)
            .map(|(_, x, y)| (*x, *y))
            .unwrap_or((-1.0, -1.0))
    };
    let (ax, ay) = pos("A");
    let (dx, dy) = pos("D");
    let (bx, by) = pos("B");
    // vertical-rl：同一行（A,B,C）的 cell 沿 y 递增（顶到底）。
    assert!(
        bx > ay.abs() || by >= ay,
        "cell B should be below A in same column (Ay={} By={})",
        ay,
        by
    );
    assert!(
        by > ay,
        "vertical-rl: cells in a row stack along y top-to-bottom (Ay={} < By={})",
        ay,
        by
    );
    // vertical-rl：1st row (A) 在最右，2nd row (D) 在其左 → A.x > D.x。
    assert!(
        ax > dx,
        "vertical-rl: 1st row (A) should be rightmost, A.x={} > D.x={} (rows right-to-left)",
        ax,
        dx
    );
    // 同一列 A,D 沿 x 不同（A 右，D 左），y 相近（行起始）。
    assert!(
        (ay - dy).abs() < 50.0 || ay.abs() < 5.0,
        "rows share inline start (Ay={} Dy={})",
        ay,
        dy
    );
}

/// R1108 α-4b-1 回归：vertical-lr 表的行沿 x 左到右（首行最左）。
#[test]
fn test_r1108_vertical_lr_table_rows_left_to_right() {
    let html = r#"<html><body style="margin:0">
      <table style="writing-mode:vertical-lr; border-spacing:0; font:20px/1 Ahem">
        <tr><td>A</td><td>B</td></tr>
        <tr><td>D</td><td>E</td></tr>
      </table>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut tds: Vec<(String, f32, f32)> = Vec::new();
    fn collect_tds(box_node: &LayoutBox, doc: &Document, abs_x: f32, abs_y: f32, out: &mut Vec<(String, f32, f32)>) {
        for child in &box_node.children {
            let x = abs_x + child.x;
            let y = abs_y + child.y;
            let tag = child.node_id.and_then(|nid| doc.get(nid)).and_then(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => Some(e.local_name().to_string()),
                _ => None,
            });
            if tag.as_deref() == Some("td") {
                let text = child
                    .node_id
                    .and_then(|nid| doc.get(nid))
                    .and_then(|n| {
                        let c = n.first_child()?;
                        let cn = doc.get(c)?;
                        match &cn.kind {
                            zero_dom::NodeKind::Text(t) => Some(t.content.chars().next().unwrap_or('?').to_string()),
                            _ => None,
                        }
                    })
                    .unwrap_or_else(|| "?".to_string());
                out.push((text, x, y));
            }
            let cx = x + child.content_x;
            let cy = y + child.content_y;
            collect_tds(child, doc, cx, cy, out);
        }
    }
    collect_tds(&result.root, &doc, 0.0, 0.0, &mut tds);
    let pos = |l: &str| {
        tds.iter()
            .find(|(t, _, _)| t == l)
            .map(|(_, x, y)| (*x, *y))
            .unwrap_or((-1.0, -1.0))
    };
    let (ax, _) = pos("A");
    let (dx, _) = pos("D");
    // vertical-lr：1st row (A) 在最左 → A.x < D.x。
    assert!(
        ax < dx,
        "vertical-lr: 1st row (A) should be leftmost, A.x={} < D.x={} (rows left-to-right)",
        ax,
        dx
    );
}

/// R1108 α-4b-1 回归：horizontal-tb 表行为不变（WM gate 零回归）。
#[test]
fn test_r1108_horizontal_tb_table_unchanged_by_wm_gate() {
    let html = r#"<html><body style="margin:0">
      <table style="border-spacing:0; font:20px/1 Ahem">
        <tr><td>A</td><td>B</td></tr>
        <tr><td>D</td><td>E</td></tr>
      </table>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut tds: Vec<(String, f32, f32)> = Vec::new();
    fn collect_tds(box_node: &LayoutBox, doc: &Document, abs_x: f32, abs_y: f32, out: &mut Vec<(String, f32, f32)>) {
        for child in &box_node.children {
            let x = abs_x + child.x;
            let y = abs_y + child.y;
            let tag = child.node_id.and_then(|nid| doc.get(nid)).and_then(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => Some(e.local_name().to_string()),
                _ => None,
            });
            if tag.as_deref() == Some("td") {
                let text = child
                    .node_id
                    .and_then(|nid| doc.get(nid))
                    .and_then(|n| {
                        let c = n.first_child()?;
                        let cn = doc.get(c)?;
                        match &cn.kind {
                            zero_dom::NodeKind::Text(t) => Some(t.content.chars().next().unwrap_or('?').to_string()),
                            _ => None,
                        }
                    })
                    .unwrap_or_else(|| "?".to_string());
                out.push((text, x, y));
            }
            let cx = x + child.content_x;
            let cy = y + child.content_y;
            collect_tds(child, doc, cx, cy, out);
        }
    }
    collect_tds(&result.root, &doc, 0.0, 0.0, &mut tds);
    let pos = |l: &str| {
        tds.iter()
            .find(|(t, _, _)| t == l)
            .map(|(_, x, y)| (*x, *y))
            .unwrap_or((-1.0, -1.0))
    };
    let (_, ay) = pos("A");
    let (_, dy) = pos("D");
    let (ax, _) = pos("A");
    let (bx, _) = pos("B");
    // horizontal-tb：行沿 y（A 上，D 下），cell 沿 x（A 左，B 右）。
    assert!(ay < dy, "horizontal-tb: rows stack along y (Ay={} < Dy={})", ay, dy);
    assert!(ax < bx, "horizontal-tb: cells stack along x (Ax={} < Bx={})", ax, bx);
}

/// R1112 α-4b-4 回归：vertical-rl 表 height 属性被应用（row_extras 沿 y 分配到 cell）。
/// height:7em=140px，3 cell 沿 y，base ~60px → 各 cell 应 +~27px 使表高 ≈140px。
#[test]
fn test_r1112_vertical_table_height_distributed_to_cells() {
    let html = r#"<html><body style="margin:0">
      <table style="writing-mode:vertical-rl; border-spacing:0; height:7em; font:20px/1 Ahem">
        <tr><td>A</td><td>B</td><td>C</td></tr>
      </table>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_table<'a>(b: &'a LayoutBox, doc: &Document) -> Option<&'a LayoutBox> {
        let tag = b.node_id.and_then(|nid| doc.get(nid)).and_then(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => Some(e.local_name().to_string()),
            _ => None,
        });
        if tag.as_deref() == Some("table") {
            return Some(b);
        }
        for c in &b.children {
            if let Some(t) = find_table(c, doc) {
                return Some(t);
            }
        }
        None
    }
    let t = find_table(&result.root, &doc).expect("table");
    // height:7em × 20px = 140px。content_height 应接近 140（旧实现 60，未分配）。
    assert!(
        t.content_height > 120.0,
        "vertical table height:7em should distribute to ~140px content_height, got {} (old: 60, no row_extras)",
        t.content_height
    );
}

/// R1117 α-4b-4 回归：vertical 表 caption-side 沿逻辑 block 轴（x）定位。
/// caption-side:top = block-start（vrl 右 / vlr 左），caption-side:bottom = block-end（vrl 左 / vlr 右）。
/// 本测覆盖「caption-at-right」两案：vrl+top（block-start=右）+ vlr+bottom（block-end=右），
/// caption 应在 cell 右侧（caption.x > cell.x），旧实现 caption 与 cell 重叠同 x。
fn run_r1117(wm: &str, side: &str) -> (f32, f32, f32) {
    let html = format!(
        r#"<html><body style="margin:0">
      <table style="writing-mode:{}; border-spacing:0; caption-side:{}; font:20px/1 Ahem">
        <caption>CA</caption>
        <tr><td>T</td></tr>
      </table>
    </body></html>"#,
        wm, side
    );
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find<'a>(b: &'a LayoutBox, doc: &Document, want: &str) -> Option<&'a LayoutBox> {
        let tag = b.node_id.and_then(|nid| doc.get(nid)).and_then(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => Some(e.local_name().to_string()),
            _ => None,
        });
        if tag.as_deref() == Some(want) {
            return Some(b);
        }
        for c in &b.children {
            if let Some(t) = find(c, doc, want) {
                return Some(t);
            }
        }
        None
    }
    let cap = find(&result.root, &doc, "caption").expect("caption");
    let td = find(&result.root, &doc, "td").expect("td");
    let table = find(&result.root, &doc, "table").expect("table");
    (cap.x, td.x, table.content_width)
}

#[test]
fn test_r1117_vrl_caption_top_on_right_of_cells() {
    // caption-side:top + vertical-rl → block-start = 右 → caption 在 cell 右侧。
    let (cap_x, td_x, _tw) = run_r1117("vertical-rl", "top");
    assert!(
        cap_x > td_x,
        "vrl caption-side:top: caption.x={} should be > td.x={} (caption at block-start=right)",
        cap_x,
        td_x
    );
}

#[test]
fn test_r1117_vlr_caption_bottom_on_right_of_cells() {
    // caption-side:bottom + vertical-lr → block-end = 右 → caption 在 cell 右侧。
    let (cap_x, td_x, _tw) = run_r1117("vertical-lr", "bottom");
    assert!(
        cap_x > td_x,
        "vlr caption-side:bottom: caption.x={} should be > td.x={} (caption at block-end=right)",
        cap_x,
        td_x
    );
}

#[test]
fn test_r1117_vrl_caption_bottom_kept_on_left() {
    // caption-side:bottom + vertical-rl → block-end = 左 → caption 留在 cell 左侧（x=0），
    // 不被 caption-at-right 分支移动。
    let (cap_x, td_x, _tw) = run_r1117("vertical-rl", "bottom");
    assert!(
        cap_x <= td_x,
        "vrl caption-side:bottom: caption.x={} should be <= td.x={} (caption stays at block-end=left)",
        cap_x,
        td_x
    );
}

/// R1122 回归：vertical-rl 表 caption 的 vrl 列 x 用 block_extent(content_width)，非
/// container_width(=content_height，inline 深度)。旧实现在 caption content_height 暴涨(784)
/// 时把单列推到 col.y=734 → paint off-screen → caption 文本完全不绘制（caption-side-vrl
/// NO green）。修后单列 col.y=0 → 文本在 content_x 正确绘制。
/// 本测验：caption 的 stored IFC 首 fragment x ≈ 0（相对 content box），非 ~734。
#[test]
fn test_r1122_vrl_caption_column_x_uses_block_extent() {
    let html = r#"<html><body style="margin:0">
      <table style="writing-mode:vertical-rl; border-spacing:0; caption-side:top; font:50px/1 Ahem">
        <caption>CA</caption>
        <tr><td>T</td><td>D</td></tr>
      </table>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find<'a>(b: &'a LayoutBox, doc: &Document, want: &str) -> Option<&'a LayoutBox> {
        let tag = b.node_id.and_then(|nid| doc.get(nid)).and_then(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => Some(e.local_name().to_string()),
            _ => None,
        });
        if tag.as_deref() == Some(want) {
            return Some(b);
        }
        for c in &b.children {
            if let Some(t) = find(c, doc, want) {
                return Some(t);
            }
        }
        None
    }
    let cap = find(&result.root, &doc, "caption").expect("caption");
    eprintln!(
        "R1122 caption: w={:.1} h={:.1} cw={:.1} ch={:.1} il={}",
        cap.width,
        cap.height,
        cap.content_width,
        cap.content_height,
        cap.inline_layout.as_ref().map(|l| l.len()).unwrap_or(0)
    );
    if let Some(lines) = &cap.inline_layout {
        for (i, l) in lines.iter().enumerate().take(3) {
            eprintln!("R1122 line[{}] y={:.1} frags={}", i, l.y, l.fragments.len());
            for f in &l.fragments {
                eprintln!("R1122   frag x={:.1}", f.x);
            }
        }
    }
    // 断言：首 fragment x < content_width（在 content box 内），旧 bug 会是 ~container_width(784)-col。
    if let Some(lines) = &cap.inline_layout
        && let Some(first) = lines.first()
        && let Some(frag) = first.fragments.first()
    {
        assert!(
            frag.x < cap.content_width + 1.0,
            "R1122: vrl caption fragment x={} should be within content box (< cw={}), old bug put it off-screen at ~container_width-col",
            frag.x,
            cap.content_width
        );
    }
}
