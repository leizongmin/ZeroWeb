//! R5169 回归：匿名 cell run 文本逐列绘制的布局侧三缺口。
//!
//! 1. **存储臂**：合成 cell（node_id 借宿主行，行样式 monospace 非 pure-Ahem）被
//!    store gate 拒存 → paint Path B 字体/推进解析与同页真实 cell（R5167 存储度量）
//!    分歧（infer-cells 绿层推进 16.9px vs 红层 16.0px）→ 放行 non-Ahem 存储，
//!    paint use_stored 重放冻结 layout 度量。
//! 2. **嵌套行盒 cell**：display:table-row 子被 build_row 当 cell 收进网格（列位
//!    正确），但其内部合成 cell 不在网格、宽度滞留 0 → IFC 按列宽重排（090 行 1
//!    h=184 vs ref 46 实证）→ position_cells 传播 cell 宽给行盒 cell 的合成 cell 子。
//! 3. **收集侧跨成员空白折叠**：span 尾随空格 + 间隔空白文本节点计双空格 → 超估算
//!    列宽误换行 → collect_items fragment 圈定域对齐 R5034 pending_space 语义。
//! 4. **boxless 合成收窄**：纯文本 run（孤儿行直系裸文本）不建空合成 cell（旧行级
//!    直绘路径正确，接管反错位——run-in-table-row-between 族），仅修 R2160 无盒
//!    inline 元素。
//! 5. `ZW_ANON_CELL_WRAP=0` kill-switch 回退由 corpus OFF 轮承担（OnceLock 进程级
//!    快照，进程内 set_var 不可靠——R3858 教训）。

/// 探针用：抽取 HTML 内联 <style> 块文本并解析（与 r5168 探针同口径）。
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
        if b.node_id == Some(id) && !b.is_anon_table_cell {
            return Some(b);
        }
        for c in b.children.iter() {
            stack.push(c);
        }
    }
    None
}

fn find_anon_cells(root: &crate::types::LayoutBox) -> Vec<&crate::types::LayoutBox> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.is_anon_table_cell {
            out.push(b);
        }
        for c in b.children.iter() {
            stack.push(c);
        }
    }
    out
}

/// 形态 A：裸 inline run（R2160 跳过、无盒）+ 显式 cell。合成 cell 应
/// ① 占列定位（x=col0）且列宽 > 0（R5168 网格预留）；
/// ② 存储 IFC（`inline_layout`，非仅 text_node_font_sizes——monospace 宿主行
///   非 pure-Ahem，旧 gate 拒存 → paint Path B 度量分歧）。
#[test]
fn r5169_bare_run_anon_cell_stores_ifc_layout() {
    let html = r#"<html><head><style>
body { font-family: monospace; font-size: 2em; }
span.row { display: table-row; }
</style></head><body>
<div>
  <span class="row" id="row1">
    <span id="s1">Row 1, </span>
    <span id="s2">Col 1</span>
    <span style="display: table-cell" id="c1">Row 1, Col 2</span>
  </span>
  <span class="row" id="row2">
    <span style="display: table-cell" id="c21">Row 22, Col 1</span>
    <span style="display: table-cell" id="c22">Row 22, Col 2</span>
    <span style="display: table-cell" id="c23">Row 22, Col 3</span>
  </span>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let anon_cells = find_anon_cells(&result.root);
    // 仅行 1 有裸 run（行 2 全显式 cell）→ 全树恰 1 个合成 cell。
    assert_eq!(anon_cells.len(), 1, "行 1 裸 run 应合成单个匿名 cell");
    let s1 = doc.get_element_by_id("s1").expect("s1 应存在");
    let anon = anon_cells
        .iter()
        .copied()
        .find(|c| c.fragment_node_ids.as_ref().is_some_and(|frags| frags.contains(&s1)))
        .expect("行 1 匿名 cell 应圈定 s1");
    assert!(
        anon.content_width > 100.0,
        "匿名 cell 应占网格列宽（fragment max-content 预留），cw={}",
        anon.content_width
    );
    assert!(
        anon.inline_layout.is_some(),
        "合成 cell 应存储 IFC（paint use_stored 冻结 layout 度量；Path B 重跑与真实 cell 分歧）"
    );
    let stored_fs: Vec<f32> = anon.text_node_font_sizes.values().copied().collect();
    assert!(
        stored_fs.iter().all(|&fs| (fs - 32.0).abs() < 0.5),
        "存储度量应为行继承 32px 字号，实际 {:?}",
        stored_fs
    );
}

/// 090 形态：display:table-row 子被当 cell 收进网格，其内部 run 的合成 cell 须按
/// 网格解析的 cell 宽排版（单行存储；0 宽 → 逐词换行撑高宿主行）。收窄后 boxless
/// 合成要求 run 含 inline 元素（纯文本 run 不合成——见 wrap_boxless_runs_in_row），
/// 故嵌套行内容用 inline span 表达。
#[test]
fn r5169_nested_row_cell_child_gets_column_width() {
    let html = r#"<html><head><style>
body { font-family: monospace; font-size: 2em; }
span.row { display: table-row; }
</style></head><body>
<div>
  <span class="row" id="row1">
    <span id="s1">Row 1, </span>
    <span id="s2">Col 1</span>
    <span style="display: table-cell" id="c1">Row 1, Col 2</span>
    <span class="row" id="nested"><span id="ns">Row 1, Col 3</span></span>
  </span>
  <span class="row" id="row2">
    <span style="display: table-cell" id="c21">Row 22, Col 1</span>
    <span style="display: table-cell" id="c22">Row 22, Col 2</span>
    <span style="display: table-cell" id="c23">Row 22, Col 3</span>
  </span>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let nested_node = doc.get_element_by_id("nested").expect("嵌套行应存在");
    let nested_box = find_box(&result.root, nested_node).expect("嵌套行盒应存在");
    let inner: Vec<&crate::types::LayoutBox> = nested_box.children.iter().filter(|c| c.is_anon_table_cell).collect();
    assert_eq!(inner.len(), 1, "嵌套行内 inline run 应合成单个匿名 cell");
    let cell = inner[0];
    assert!(
        cell.content_width > 100.0,
        "嵌套行盒 cell 的合成 cell 应继承网格列宽，cw={}",
        cell.content_width
    );
    let lines = cell.inline_layout.as_ref().expect("合成 cell 应存储 IFC").len();
    assert_eq!(lines, 1, "Row 1, Col 3 按列宽应单行存储（0 宽逐词换行为旧缺陷）");
}

/// 088 行 2 形态：span 尾随空格 + span 间空白文本节点的 run——收集侧跨成员折叠为
/// **单个**空格（CSS Text §4.1，与 R5034 估算侧 pending_space 对齐）。旧实现逐成员
/// 拼接计双空格 → 内容宽超列宽（估算按单空格）→ 误换行两行。
#[test]
fn r5169_fragment_collection_collapses_ws_between_members() {
    let html = r#"<html><head><style>
body { font-family: monospace; font-size: 2em; }
span.row { display: table-row; }
</style></head><body>
<div>
  <span class="row" id="row1">
    <span id="sa">Row 1, </span>
    <span id="sb">Col 1</span>
    <span style="display: table-cell" id="c1">Row 1, Col 2</span>
    <span id="sc">Row 1, Col 3</span>
  </span>
  <span class="row" id="row2">
    <span style="display: table-cell" id="c21">Row 22, Col 1</span>
    <span id="s1">Row </span>
    <span id="s2">22, </span>
    <span id="s3">Col </span>
    <span id="s4">2</span>
    <span style="display: table-cell" id="c22">Row 22, Col 3</span>
  </span>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let s1 = doc.get_element_by_id("s1").expect("s1 应存在");
    let anon = find_anon_cells(&result.root)
        .into_iter()
        .find(|c| c.fragment_node_ids.as_ref().is_some_and(|frags| frags.contains(&s1)))
        .expect("裸 span run 应合成匿名 cell");
    let lines = anon.inline_layout.as_ref().expect("应存储 IFC");
    assert_eq!(lines.len(), 1, "单空格内容应按估算列宽单行存放（双空格超宽误换行）");
    // 尾随空格计入推进（fragment.text 剥离）——相邻片段 x 间隙应为**单个**空格宽
    // （monospace 32px = 19.2；双空格缺陷为 38.4）。
    let frags = &lines[0].fragments;
    assert_eq!(frags.len(), 4, "Row/22,/Col/2 四片段");
    // 断言口径：跨成员单空格的片段间隙须等于**单 span 同文本**的字内空格间隙
    //（测试环境空格推进走 estimate 兜底，与字内推进不同源，不能硬编码像素值——
    // 对照组即语义：跨 run 折叠 = 单个空格）。旧实现双空格 → 间隙 ≈ 2×对照。
    let ctrl_html = html.replace(
        r#"<span id="s1">Row </span>
    <span id="s2">22, </span>
    <span id="s3">Col </span>
    <span id="s4">2</span>"#,
        r#"<span id="sall">Row 22, Col 2</span>"#,
    );
    let ctrl_doc = zero_dom::parse_html(&ctrl_html);
    let mut ctrl_sys = zero_style_system::StyleSystem::new();
    ctrl_sys.set_viewport(800.0, 600.0);
    let ctrl_styles = ctrl_sys.compute_styles(&ctrl_doc, &sheets(&ctrl_html));
    let mut ctrl_engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let ctrl_result = ctrl_engine.compute(&ctrl_doc, &ctrl_styles);
    let sall = ctrl_doc.get_element_by_id("sall").expect("sall 应存在");
    let ctrl_cell = find_anon_cells(&ctrl_result.root)
        .into_iter()
        .find(|c| c.fragment_node_ids.as_ref().is_some_and(|frags| frags.contains(&sall)))
        .expect("对照 run 应合成匿名 cell");
    let ctrl_lines = ctrl_cell.inline_layout.as_ref().expect("对照应存储 IFC");
    assert_eq!(ctrl_lines.len(), 1, "对照单行");
    let ctrl_gaps: Vec<f32> = ctrl_lines[0]
        .fragments
        .windows(2)
        .map(|p| p[1].x - (p[0].x + p[0].width))
        .collect();
    assert_eq!(ctrl_gaps.len(), 3, "对照 3 个字内空格间隙");
    let gaps: Vec<f32> = frags.windows(2).map(|p| p[1].x - (p[0].x + p[0].width)).collect();
    for (i, (g, cg)) in gaps.iter().zip(ctrl_gaps.iter()).enumerate() {
        assert!(
            (g - cg).abs() < 1.0,
            "跨成员间隙 {g} 应等于字内单空格间隙 {cg}（第 {} 对；双空格 ≈ 2×对照为旧缺陷）",
            i + 1
        );
    }
}
