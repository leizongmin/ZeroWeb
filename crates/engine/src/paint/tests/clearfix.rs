//! wrapper::after clearfix（content:"";display:block;clear:both）+ 嵌套 float 行——
//! baidu 热榜叠字根因端到端判别。两个缺口：① 容器内 **float** 子的 clear 误置
//! `clearance_active` → R1319 sibling-shift 把后续兄弟的合法 clearance 当「泄漏」
//! 撤销（伪元素被拉回 float 容器底，wrapper 高度塌回）；② R1392 嵌套浮动底边在
//! content-rel 模式多减 content_y_offset（带 padding-top 容器 clearance 不足）。
//! CSS2 §9.5.1（float 自身 clear 由 float 摆位解决）/ §9.5.2（流内 clearance）/
//! §10.6.3（auto 高度只计 in-flow 子）/ §14.2 clearfix 惯用法。

const CSS: &str = r#"
body { margin: 0 }
.w { position: relative; margin: 93px auto 0; padding-top: 21px; width: 760px }
.w::after { content: ""; display: block; clear: both }
.t { height: 24px; margin-bottom: 18px }
.ul { list-style: none; padding: 0; margin: 0 }
.li { float: left; width: 369px; height: 36px }
.li.odd { clear: both; margin-right: 20px }
"#;

const HTML: &str = r#"<html><head><style></style></head>
<body>
<div class="w">
  <div class="t">TITLE</div>
  <ul class="ul">
    <li class="li odd">row1</li><li class="li even">row2</li>
    <li class="li odd">row3</li><li class="li even">row4</li>
  </ul>
</div>
<div class="after">AFTERCONTENT</div>
</body></html>"#;

fn find_by_class<'a>(
    b: &'a zero_layout_engine::LayoutBox,
    doc: &zero_dom::Document,
    cls: &str,
) -> Option<&'a zero_layout_engine::LayoutBox> {
    if let Some(nid) = b.node_id
        && doc
            .get_attribute(nid, "class")
            .is_some_and(|v| v.split_whitespace().any(|c| c == cls))
    {
        return Some(b);
    }
    for c in &b.children {
        if let Some(f) = find_by_class(c, doc, cls) {
            return Some(f);
        }
    }
    None
}

fn build_layout(doc: &mut zero_dom::Document) -> zero_layout_engine::LayoutResult {
    let sheet = zero_css_parser::Parser::parse_stylesheet(CSS);
    let stylesheets = vec![sheet];
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let mut styles = sys.compute_styles(doc, &stylesheets);
    // 全管线同款：伪元素注入（pipeline render_html 步骤 3.5）后再布局。
    crate::pipeline::inject_pseudo_text_nodes(doc, &mut styles, &stylesheets);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    engine.compute(doc, &styles)
}

/// wrapper::after clearance 须落在嵌套 float 底边（114），wrapper 高度包含之，
/// 后续兄弟不与 float 行叠压。
#[test]
fn wrapper_after_clearfix_contains_nested_floats() {
    let mut doc = zero_dom::parse_html(HTML);
    let result = build_layout(&mut doc);

    let w = find_by_class(&result.root, &doc, "w").expect("wrapper 布局盒");
    // wrapper = padding-top 21 + title 24 + collapsed mb 18 + float 行 72（clearance 底 114）
    // ≥130 留 2× 裕度；塌缩形（回归）为 63/84。
    assert!(
        w.height >= 130.0,
        "clearfix 应使 wrapper 高度包含 float 行（≥130），实际 {}",
        w.height
    );

    // ::after 伪元素盒（zw-pseudo）应落在 float 底边 114（非塌缩位 42 / 虚减位 93）
    let pseudo = w
        .children
        .iter()
        .find(|c| {
            c.node_id.is_some_and(|nid| {
                doc.get(nid)
                    .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "zw-pseudo"))
            })
        })
        .expect("wrapper::after 应产出 zw-pseudo 盒");
    assert!(
        (pseudo.y - 114.0).abs() < 1.0,
        "::after clearance 应落 float 底边 114，实际 y={}",
        pseudo.y
    );

    // 后续兄弟 div.after 不与 float 行叠压：其 y 须在 wrapper 底之下
    let after = find_by_class(&result.root, &doc, "after").expect("after 布局盒");
    let wrapper_bottom = w.y + w.height;
    assert!(
        after.y >= wrapper_bottom - 0.5,
        "后续兄弟应在 wrapper 底 {} 之下，实际 y={}",
        wrapper_bottom,
        after.y
    );
}
