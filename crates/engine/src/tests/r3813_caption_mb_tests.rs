//! R3813：table-caption 的 margin-bottom 计入表内总高（CSS Tables §17.1.1）。
//!
//! margin-applies-to-015：abspos wrapper（shrink-wrap）内 table-caption margin 50 —
//! caption margin-bottom 未计入表高 → wrapper 高 290（= 10 border + 50 mt + 220 + 10
//! border），chromium 340（再 + 50 mb）。orange 内容两侧本已对齐，纯表高 accounting
//! 缺口。
#[test]
fn r3813_caption_margin_bottom_in_table_height() {
    let mut pipeline = crate::pipeline::RenderPipeline::new(400.0, 600.0);
    let html = r#"<html><head><style>
    #wrapper { position: absolute; border: 10px solid blue; }
    #test { display: table-caption; height: 200px; margin: 50px; }
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
    let _ = pipeline.render_html(html, "");
    let layout = pipeline.layout().expect("layout");
    // wrapper（abspos，唯一顶层 div）高度须含 caption margin-bottom：
    // 10 border + 50 mt + 200 content + 50 mb + 10 border = 320（本探针 caption 无边框）。
    // wrapper 是 absolute 定位：root(anonymous) → body → wrapper。取最后一个有高度的
    // BOX（绝对定位 wrapper 不一定挂在最后一层）。
    fn find_abs_wrapper(b: &zero_layout_engine::types::LayoutBox) -> Option<&zero_layout_engine::types::LayoutBox> {
        for c in &b.children {
            if c.is_absolute {
                return Some(c);
            }
            if let Some(found) = find_abs_wrapper(c) {
                return Some(found);
            }
        }
        None
    }
    let w = find_abs_wrapper(&layout.root).expect("abspos wrapper found");
    assert!(
        (w.height - 320.0).abs() < 2.0,
        "wrapper 高应含 caption margin-bottom（340），实际 {}",
        w.height
    );
}
