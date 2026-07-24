use super::*;

/// R329：`@font-face` 自定义字体加载端到端验证。
///
/// CSS 声明 `@font-face { font-family: "R329Alias"; src: url("Ahem.ttf"); }`，
/// 基目录指向 `tests/wpt-runner/fonts/`（含真实 Ahem.ttf）。`load_font_faces_into`
/// 应解析 src、加载字体、并把**声明族名** "R329Alias" 注册为别名（与字体内部名
/// "Ahem" 不同）。`build_font_resolver` 须含 "R329Alias" → font_id。
#[test]
fn test_font_face_loads_custom_family_alias() {
    let fonts_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts");
    let ahem = fonts_dir.join("Ahem.ttf");
    if !ahem.exists() {
        // 无 Ahem.ttf 的环境（如 CI 缺字体）跳过，不计失败
        eprintln!("[R329] Ahem.ttf missing, skipping @font-face load test");
        return;
    }
    let css = r#"@font-face { font-family: "R329Alias"; src: url("Ahem.ttf"); }"#;
    let mut loader = create_font_loader();
    load_font_faces_into(&mut loader, Some(&fonts_dir), css);
    let resolver = loader.build_font_resolver();
    assert!(
        resolver.contains_key("R329Alias"),
        "@font-face declared family 'R329Alias' must resolve to a loaded font_id; \
             resolver keys: {:?}",
        resolver.keys().collect::<Vec<_>>()
    );
}

/// R329：`@font-face` 解析 + 跳过 data:/http: 源（不可本地加载）。
#[test]
fn test_font_face_resolves_local_and_skips_remote() {
    let css = r#"@font-face { font-family: "Remote"; src: url("https://example.com/x.woff"); }"#;
    let faces = extract_font_faces(css);
    assert_eq!(faces.len(), 1);
    // 远程源无法解析到本地路径
    assert!(resolve_font_src("https://example.com/x.woff", None).is_none());
    assert!(resolve_font_src("data:application/font-woff;base64,AAAA", None).is_none());
    // 相对路径需 base_dir
    assert!(resolve_font_src("rel.woff", None).is_none());
}

/// 验证 position:relative + top 偏移是否正确应用。
/// 测试：border-bottom 96px black + height 96px = 空顶 + 黑底
/// 参考：background black + height 96px + position:relative; top:96px = 空顶 + 黑底
/// 两者应在视觉上相同（black 在下半部分）。
#[test]
fn test_reftest_relative_top_offset() {
    // First, verify the test HTML renders correctly: black at bottom half
    let test_only = ReftestCase {
            id: "test/border-bottom-only".into(),
            test_html: "<html><body style=\"margin:0\"><div style=\"border-bottom: 96px solid black; height: 96px; width: 96px;\"></div></body></html>".into(),
            // Same HTML as ref - should match itself
            ref_html: "<html><body style=\"margin:0\"><div style=\"border-bottom: 96px solid black; height: 96px; width: 96px;\"></div></body></html>".into(),
            css: String::new(),
            is_match: true,
            ref_base_dir: None,
        };
    let config = ReftestConfig::default();
    let result = run_reftest(&test_only, &config);
    assert!(result.passed, "Self-comparison should always pass: {}", result.message);

    // Now verify the reference renders the same visual: black div offset down
    let case = ReftestCase {
            id: "test/relative-top".into(),
            test_html: "<html><body style=\"margin:0\"><div style=\"border-bottom: 96px solid black; height: 96px; width: 96px;\"></div></body></html>".into(),
            ref_html: "<html><body style=\"margin:0\"><div style=\"background-color: black; height: 96px; width: 96px; position: relative; top: 96px;\"></div></body></html>".into(),
            css: String::new(),
            is_match: true,
            ref_base_dir: None,
        };
    let result = run_reftest(&case, &config);
    assert!(
        result.passed,
        "position:relative + top:96px should produce same visual as border-bottom: {}",
        result.message
    );
}

#[test]
fn test_reftest_identical_pages() {
    let case = ReftestCase {
        id: "test/identical".into(),
        test_html: "<html><body><div style=\"width:100px;height:50px;background:red;\">A</div></body></html>".into(),
        ref_html: "<html><body><div style=\"width:100px;height:50px;background:red;\">A</div></body></html>".into(),
        css: String::new(),
        is_match: true,
        ref_base_dir: None,
    };
    let config = ReftestConfig::default();
    let result = run_reftest(&case, &config);
    assert!(result.passed, "Identical pages should match: {}", result.message);
}

#[test]
fn test_reftest_different_pages() {
    let case = ReftestCase {
        id: "test/different".into(),
        test_html: "<html><body><div style=\"width:100px;height:50px;background:red;\">A</div></body></html>".into(),
        ref_html: "<html><body><div style=\"width:100px;height:50px;background:blue;\">B</div></body></html>".into(),
        css: String::new(),
        is_match: true,
        ref_base_dir: None,
    };
    let config = ReftestConfig::default();
    let result = run_reftest(&case, &config);
    assert!(!result.passed, "Different pages should not match: {}", result.message);
}

#[test]
fn test_reftest_mismatch_mode() {
    let case = ReftestCase {
            id: "test/mismatch".into(),
            test_html: "<html><body style=\"margin:0\"><div style=\"width:100%;height:100%;background:red;\">Red</div></body></html>".into(),
            ref_html: "<html><body style=\"margin:0\"><div style=\"width:100%;height:100%;background:blue;\">Blue</div></body></html>".into(),
            css: String::new(),
            is_match: false,
            ref_base_dir: None,
        };
    let config = ReftestConfig::default();
    let result = run_reftest(&case, &config);
    assert!(
        result.passed,
        "Different pages should pass mismatch: {}",
        result.message
    );
}

/// R1991：`--media print` 端到端接线验证（DC-12 @media print 级联）。
///
/// 同一页面声明 `@media screen { body: blue } @media print { body: red }`：
/// `ReftestConfig.media_type = Screen` 渲染蓝底，`Print` 渲染红底。两 framebuffer
/// 应显著不同 → 证明 config.media_type 经 `pipeline.set_media_type` →
/// `StyleSystem.set_media_type` 触达级联过滤（@media print/screen 规则按渲染媒体类型生效）。
#[test]
fn test_media_type_print_applies_print_rules() {
    let html = concat!(
        "<html><head><style>",
        "@media screen { body { background-color: blue; } }",
        "@media print  { body { background-color: red;  } }",
        "</style></head>",
        "<body style=\"margin:0\"><div style=\"width:100%;height:100%;\"></div></body></html>"
    );
    let screen_cfg = ReftestConfig {
        media_type: zero_css_parser::media_query::MediaType::Screen,
        ..Default::default()
    };
    let print_cfg = ReftestConfig {
        media_type: zero_css_parser::media_query::MediaType::Print,
        ..Default::default()
    };

    let screen_fb = render_to_framebuffer_with_base(html, "", &screen_cfg, None);
    let print_fb = render_to_framebuffer_with_base(html, "", &print_cfg, None);

    // Screen(蓝) vs Print(红) 整页背景不同 → diff_pixel_count 应远大于 0。
    let (diff_pixels, _) = compare_pixels(&screen_fb, &print_fb, 5);
    assert!(
        diff_pixels > 100,
        "Screen vs Print render must differ (media_type wiring broken): diff_pixels={diff_pixels}"
    );
}

/// 递归算 LayoutBox 树的最大 abs bottom（镜像 pipeline.rs `layout_extent_y`）。
/// 供 R2000 端到端分页测试量 layout extent。
fn r2000_max_abs_bottom(b: &zero_layout_engine::types::LayoutBox, parent_offset_y: f32) -> f32 {
    let abs_top = parent_offset_y + b.y;
    let mut max_y = abs_top + b.height;
    for child in &b.children {
        max_y = max_y.max(r2000_max_abs_bottom(child, abs_top));
    }
    max_y
}

/// R2000：Print 分页端到端验证（经完整 parse→cascade→layout→paginate 管线，真实 HTML）。
///
/// Print 模式 + `page-break-before:always` → 分页 post-process（R2000 default-on）
/// 把第二个 div（green）推到 A4 页边界（≈1122.5）→ layout extent 从 ~150 增到 ~1172。
/// 对比 pagination-off（env `ZW_PRINT_PAGINATE=0`）→ extent 显著增大（>900px）证全管线分页生效。
/// 注：framebuffer 按 viewport_height 创建（高内容裁剪），故量 **layout root extent** 非 fb.height。
#[test]
fn r2000_print_pagination_end_to_end_layout_extent_grows() {
    let html = concat!(
        "<html><body style=\"margin:0\">",
        "<div style=\"width:100px;height:100px;background:red\"></div>",
        "<div style=\"width:100px;height:50px;background:green;page-break-before:always\"></div>",
        "</body></html>"
    );
    let print_cfg = ReftestConfig {
        media_type: zero_css_parser::media_query::MediaType::Print,
        ..Default::default()
    };
    // 分页关（kill-switch）。
    unsafe {
        std::env::set_var("ZW_PRINT_PAGINATE", "0");
    }
    let (_fb_off, root_off, _) = render_to_framebuffer_with_layout_with_base(html, "", &print_cfg, None);
    let extent_off = r2000_max_abs_bottom(&root_off, 0.0);

    // 分页开（default-on：移除 env 即启用）。
    unsafe {
        std::env::remove_var("ZW_PRINT_PAGINATE");
    }
    let (_fb_on, root_on, _) = render_to_framebuffer_with_layout_with_base(html, "", &print_cfg, None);
    let extent_on = r2000_max_abs_bottom(&root_on, 0.0);

    assert!(
        extent_on > extent_off + 900.0,
        "分页须使 layout extent 显著增大（B 推到 A4 页边界 1122.5）: off={extent_off} on={extent_on}"
    );
}

#[test]
fn test_struct_check_detects_sibling_overlap() {
    // 两个**普通块流**兄弟盒经负 margin 重叠（第二个 margin-top:-50px 拉 50px，100×50=5000px²
    // 交集）须被检出。R1504：positioned（abspos/relative/fixed）/float 盒按设计重叠已被排除，
    // 故测试改用 normal-block 重叠（真 bug 模式，如 R1492 长高重叠）。
    let html = "<html><body style=\"margin:0\">\
            <div style=\"width:100px;height:100px;background:red\"></div>\
            <div style=\"width:100px;height:100px;margin-top:-50px;background:blue\"></div>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let issues = check_sibling_overlaps(&root, &labels);
    assert!(
        !issues.is_empty(),
        "overlapping normal-block siblings must be flagged, got {issues:?}"
    );
    assert!(
        issues.iter().any(|s| s.contains("overlap")),
        "issue must mention overlap: {issues:?}"
    );
}

#[test]
fn test_struct_check_passes_stacked_blocks() {
    // 两个垂直堆叠的 block 兄弟盒（正常流，无重叠）须通过。
    let html = "<html><body style=\"margin:0\">\
            <div style=\"width:100px;height:100px;background:red\"></div>\
            <div style=\"width:100px;height:100px;background:blue\"></div>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let issues = check_sibling_overlaps(&root, &labels);
    assert!(
        issues.is_empty(),
        "stacked block siblings must not flag overlap: {issues:?}"
    );
}

#[test]
fn test_collapsed_containers_detects_inline_wrapping_inline_block() {
    // R1576 inline-box-model：`<p><a><img class=inline-block h-24></a></p>` 的 `<p>` 须随
    // inline-block 子内容长高（IFC 递归 inline 元素收集嵌套 atomic inline 盒），不应塌缩为 0。
    // default-on（ZW_INLINE_BOX_RECURSE）修复后 check 须**不**检出 `<p>` 塌缩。
    let html = "<html><body style=\"margin:0\">\
            <p><a href=\"#\"><img style=\"display:inline-block;height:24px;width:24px\" src=\"x.png\"></a></p>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let issues = check_collapsed_containers(&root, &labels);
    assert!(
        !issues.iter().any(|s| s.contains("[p]")),
        "R1576: <p> wrapping inline>inline-block must grow (not collapse), still flagged: {issues:?}"
    );
}

#[test]
fn test_collapsed_containers_ignores_abspos_child() {
    // R1575：父盒 h=0 但子为 abspos（脱离流）是正确行为（position-absolute-* 测试），
    // 不应 flag。仅 in-flow 高子内容触发。
    let html = "<html><body style=\"margin:0;position:relative\">\
            <div style=\"height:0\"><div style=\"position:absolute;width:100px;height:100px\"></div></div>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let issues = check_collapsed_containers(&root, &labels);
    assert!(
        issues.is_empty(),
        "parent with only abspos child must not be flagged: {issues:?}"
    );
}

#[test]
fn test_count_boxes_by_class_exact_match() {
    // 4 个 .card + 2 个 .card-sub（须精确匹配，card-sub 不计入 card）。
    let html = "<html><body>\
            <div class=\"card\"><p class=\"card-sub\">a</p></div>\
            <div class=\"card\"><p class=\"card-sub\">b</p></div>\
            <div class=\"card\">c</div>\
            <div class=\"card\">d</div>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    assert_eq!(
        count_boxes_by_class(&root, &labels, "card"),
        4,
        "exact .card count must be 4 (not counting .card-sub)"
    );
    assert_eq!(
        count_boxes_by_class(&root, &labels, "card-sub"),
        2,
        ".card-sub count must be 2"
    );
    assert_eq!(
        count_boxes_by_class(&root, &labels, "nope"),
        0,
        "absent class must count 0"
    );
}

#[test]
fn test_count_lines_for_class() {
    // title = 1 line；tagline = 2 lines（<br> 强制断行）；验证 content_height / line_height 行数估算。
    let html = "<html><body style=\"margin:0\">\
            <h1 class=\"title\">ZeroBrowser</h1>\
            <p class=\"tagline\">line one<br>line two</p>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    assert_eq!(
        count_lines_for_class(&root, &labels, "title"),
        Some(1),
        "title (single word) must be 1 line"
    );
    assert_eq!(
        count_lines_for_class(&root, &labels, "tagline"),
        Some(2),
        "tagline (<br>) must be 2 lines"
    );
}

#[test]
fn test_text_concatenation_passes_grid_container() {
    // welcome 谱系：`.cards`（grid）含 2 个 `.card` block 子元素，各自持文本。正常布局下
    // 容器自身 text_node 映射为空（文本由各 card 各自的 IFC 渲染）→ 不应 flag 串联。
    let html = "<html><body style=\"margin:0\">\
            <div class=\"cards\" style=\"display:grid;grid-template-columns:1fr 1fr\">\
              <div class=\"card\">Card one text</div>\
              <div class=\"card\">Card two text</div>\
            </div>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let (has_direct_text, non_ws_text_nodes) = collect_concat_dom_info(html);
    let issues = check_text_concatenation(&root, &labels, &has_direct_text, &non_ws_text_nodes);
    assert!(
        !issues
            .iter()
            .any(|s| s.contains("text concatenation") && s.contains("[div.cards]")),
        "correct grid container must not be flagged for concatenation: {issues:?}"
    );
}

#[test]
fn test_text_concatenation_flags_absorbed_children() {
    // 模拟 R109 inline-ownership 退化：把一个 card 子盒 text_node_line_heights 的条目注入
    // `.cards` 容器自身映射（= 父容器 IFC 吸收了子元素文本）。容器无直接文本、有 ≥2 block 子、
    // 自身 text_node 映射含非空白文本节点 → 必被 flag（sibling 文本串联）。
    let html = "<html><body style=\"margin:0\">\
            <div class=\"cards\" style=\"display:grid;grid-template-columns:1fr 1fr\">\
              <div class=\"card\">Card one text</div>\
              <div class=\"card\">Card two text</div>\
            </div>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let (has_direct_text, non_ws_text_nodes) = collect_concat_dom_info(html);
    // 基线：正常渲染不 flag。
    assert!(
        check_text_concatenation(&root, &labels, &has_direct_text, &non_ws_text_nodes)
            .iter()
            .all(|s| !(s.contains("text concatenation") && s.contains("[div.cards]"))),
        "baseline must not flag the cards container"
    );
    // 递归找一个含**非空白**文本节点的 text_node_line_heights（card 子树内的真实文本），
    // 注入 cards 容器模拟吸收。须过滤空白节点（store_font_sizes_from_ifc 也存空白片段，
    // 空白节点不在 non_ws_text_nodes，注入它们不会触发检查——与检查的过滤一致）。
    fn first_non_ws_text_entries(
        b: &zero_layout_engine::types::LayoutBox,
        non_ws: &std::collections::HashSet<zero_dom::NodeId>,
    ) -> Option<Vec<(zero_dom::NodeId, f32)>> {
        let matched: Vec<_> = b
            .text_node_line_heights
            .iter()
            .filter(|(k, _)| non_ws.contains(k))
            .map(|(k, v)| (*k, *v))
            .collect();
        if !matched.is_empty() {
            return Some(matched);
        }
        b.children.iter().find_map(|c| first_non_ws_text_entries(c, non_ws))
    }
    let mut root2 = root.clone();
    fn find_mut<'a>(
        b: &'a mut zero_layout_engine::types::LayoutBox,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        needle: &str,
    ) -> Option<&'a mut zero_layout_engine::types::LayoutBox> {
        if let Some(id) = b.node_id
            && let Some(label) = labels.get(&id)
            && label.split('.').any(|c| c == needle)
        {
            return Some(b);
        }
        for child in &mut b.children {
            if let Some(found) = find_mut(child, labels, needle) {
                return Some(found);
            }
        }
        None
    }
    let entries = first_non_ws_text_entries(&root, &non_ws_text_nodes)
        .expect("some box has non-whitespace text_node_line_heights entries");
    let cards = find_mut(&mut root2, &labels, "cards").expect("cards container found");
    for (id, h) in entries {
        cards.text_node_line_heights.insert(id, h);
    }
    let issues = check_text_concatenation(&root2, &labels, &has_direct_text, &non_ws_text_nodes);
    assert!(
        issues
            .iter()
            .any(|s| s.contains("text concatenation") && s.contains("[div.cards]")),
        "absorbed child text into cards container must be flagged: {issues:?}"
    );
}

#[test]
fn test_text_concatenation_ignores_container_with_direct_text() {
    // 条件 3 排除：容器有直接文本子节点 + block 子元素（合法 block-in-inline / 自带文本）。
    // 容器自身 IFC 含其直接文本是合法的，不应 flag。
    let html = "<html><body style=\"margin:0\">\
            <div class=\"wrap\">Intro direct text\
              <div>block one</div>\
              <div>block two</div>\
            </div>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let (has_direct_text, non_ws_text_nodes) = collect_concat_dom_info(html);
    assert!(
        has_direct_text
            .iter()
            .any(|id| labels.get(id).is_some_and(|l| l.contains("wrap"))),
        ".wrap has direct text and must be in has_direct_text set"
    );
    let issues = check_text_concatenation(&root, &labels, &has_direct_text, &non_ws_text_nodes);
    assert!(
        !issues
            .iter()
            .any(|s| s.contains("text concatenation") && s.contains("[div.wrap]")),
        "container with legitimate direct text must not be flagged: {issues:?}"
    );
}

#[test]
fn test_text_concatenation_skips_table_internal() {
    // R1652：table-internal 容器（tr/td/th/tbody/…）合法拥有自身/子 cell 文本（td IFC 处理
    // cell 内容，tr 的 text_node_line_heights 含子 cell 文本属正常 table 布局）。本检查针对
    // flex/grid/block 的 R109 inline-ownership 串联，table 不属此列 → 即使满足三条件也不应 flag。
    // legacy-html fixture 19-testpage-minimal 的 `<tr>` 误报即此（LAYOUT_DUMP 表格几何正确）。
    let html = "<html><body style=\"margin:0\">\
            <table>\
              <tr><td>cell A text</td><td>cell B text</td></tr>\
            </table>\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let (has_direct_text, non_ws_text_nodes) = collect_concat_dom_info(html);
    let issues = check_text_concatenation(&root, &labels, &has_direct_text, &non_ws_text_nodes);
    assert!(
        !issues
            .iter()
            .any(|s| s.contains("text concatenation") && (s.contains("[tr]") || s.contains("[td]"))),
        "table-internal containers must not be flagged (legitimate cell text ownership): {issues:?}"
    );
}

#[test]
fn test_replaced_collapse_flags_zero_size_img() {
    // DC-13 line 327：塌缩的 img（width/height<2）须被检出（logo 不可见退化）。
    // 用一个 0×0 的 <img>（无 src + 无尺寸属性 → 塌缩）+ 一个有尺寸的 img 对比。
    let html = "<html><body style=\"margin:0\">\
            <img class=\"collapsed\" style=\"width:0;height:0\" src=\"x.png\">\
            <img class=\"visible\" style=\"width:40px;height:30px\" src=\"y.png\">\
            </body></html>";
    let config = ReftestConfig::default();
    let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
    let labels = collect_dom_labels(html);
    let issues = check_replaced_collapse(&root, &labels);
    // 仅 .collapsed 被检出；.visible（40×30）不报。
    assert!(
        issues
            .iter()
            .any(|s| s.contains("[img.collapsed]") && s.contains("collapsed replaced")),
        "zero-size img must be flagged: {issues:?}"
    );
    assert!(
        !issues.iter().any(|s| s.contains("[img.visible]")),
        "sized img must not be flagged: {issues:?}"
    );
}

#[test]
fn test_reftest_config_default() {
    let config = ReftestConfig::default();
    assert_eq!(config.viewport_width, 800);
    assert_eq!(config.viewport_height, 600);
    assert!((config.max_diff_ratio - 0.01).abs() < f64::EPSILON);
    assert_eq!(config.max_channel_diff, 5);
}

#[test]
fn test_reftest_fuzzy_threshold() {
    let case = ReftestCase {
        id: "test/fuzzy".into(),
        test_html:
            "<html><body><div style=\"background:rgb(100,100,100);width:50px;height:50px;\">A</div></body></html>"
                .into(),
        ref_html:
            "<html><body><div style=\"background:rgb(102,102,102);width:50px;height:50px;\">A</div></body></html>"
                .into(),
        css: String::new(),
        is_match: true,
        ref_base_dir: None,
    };
    let config = ReftestConfig {
        max_diff_ratio: 0.1,
        max_channel_diff: 10,
        ..Default::default()
    };
    let result = run_reftest(&case, &config);
    assert!(
        result.passed,
        "Small color diff should match with fuzzy threshold: {}",
        result.message
    );
}

#[test]
fn test_extract_stylesheet_hrefs() {
    let html = r#"
            <html><head>
                <link rel="stylesheet" href="/fonts/ahem.css">
                <link rel='alternate stylesheet' href='theme.css'>
                <link rel="help" href="spec.html">
            </head></html>
        "#;
    let hrefs = extract_stylesheet_hrefs(html);
    assert_eq!(hrefs, vec!["/fonts/ahem.css".to_string(), "theme.css".to_string()]);
}

#[test]
#[ignore]
fn debug_clear_applies_to_009_blue_bbox() {
    fn blue_bbox(fb: &FrameBuffer) -> Option<(u32, u32, u32, u32)> {
        let mut min_x = fb.width;
        let mut min_y = fb.height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..fb.height {
            for x in 0..fb.width {
                let idx = ((y * fb.width + x) * 4) as usize;
                let px = &fb.data[idx..idx + 4];
                let is_blue = px[0] < 32 && px[1] < 32 && px[2] > 200 && px[3] > 200;
                if is_blue {
                    found = true;
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        found.then_some((min_x, min_y, max_x, max_y))
    }

    let wpt_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data");
    let case_path = wpt_root.join("css/CSS2/floats-clear/clear-applies-to-009.xht");
    let ref_path = wpt_root.join("css/CSS2/floats-clear/clear-applies-to-009-ref.xht");
    let test_html = std::fs::read_to_string(&case_path).expect("read test html");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let base_dir = case_path.parent().expect("base dir");
    let config = ReftestConfig::default();

    let test_fb = render_to_framebuffer_with_base(&test_html, "", &config, Some(base_dir));
    let ref_fb = render_to_framebuffer_with_base(&ref_html, "", &config, Some(base_dir));

    println!("test blue bbox: {:?}", blue_bbox(&test_fb));
    println!("ref  blue bbox: {:?}", blue_bbox(&ref_fb));
}

#[test]
#[ignore]
fn debug_clear_applies_to_009_layout_snapshot() {
    let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-applies-to-009.xht");
    let html = std::fs::read_to_string(&case_path).expect("read test html");
    let linked_css = load_linked_stylesheets(&html, case_path.parent());
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_skip_indicators(true);
    let font_loader = create_font_loader();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    let rendered = pipeline.render_html(&html, &linked_css);
    println!("{}", rendered.layout.snapshot());
    for i in 0..8 {
        if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
            println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
    for fill in &rendered.primitives.fills {
        if fill.color.r < 32 && fill.color.g < 32 && fill.color.b > 200 && fill.color.a > 200 {
            println!(
                "blue fill rect=({:.2},{:.2},{:.2},{:.2})",
                fill.rect.origin.x, fill.rect.origin.y, fill.rect.size.width, fill.rect.size.height
            );
        }
    }
    for rr in &rendered.primitives.rounded_rects {
        if rr.color.r < 32 && rr.color.g < 32 && rr.color.b > 200 && rr.color.a > 200 {
            println!(
                "blue rr rect=({:.2},{:.2},{:.2},{:.2})",
                rr.rect.origin.x, rr.rect.origin.y, rr.rect.size.width, rr.rect.size.height
            );
        }
    }

    let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-applies-to-009-ref.xht");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
    let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
    ref_pipeline.set_skip_indicators(true);
    let ref_font_loader = create_font_loader();
    ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
    let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
    println!("--- ref ---");
    println!("{}", ref_rendered.layout.snapshot());
    for i in 0..8 {
        if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
            println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
}

#[test]
#[ignore]
fn debug_clear_applies_to_001_layout_snapshot() {
    let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-applies-to-001.xht");
    let html = std::fs::read_to_string(&case_path).expect("read test html");
    let linked_css = load_linked_stylesheets(&html, case_path.parent());
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_skip_indicators(true);
    let font_loader = create_font_loader();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    let rendered = pipeline.render_html(&html, &linked_css);
    println!("{}", rendered.layout.snapshot());
    for i in 0..12 {
        if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
            println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }

    let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-applies-to-001-ref.xht");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
    let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
    ref_pipeline.set_skip_indicators(true);
    let ref_font_loader = create_font_loader();
    ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
    let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
    println!("--- ref ---");
    println!("{}", ref_rendered.layout.snapshot());
    for i in 0..12 {
        if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
            println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
}

#[test]
#[ignore]
fn debug_clear_clearance_calculation_001_layout_snapshot() {
    let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-001.xht");
    let html = std::fs::read_to_string(&case_path).expect("read test html");
    let linked_css = load_linked_stylesheets(&html, case_path.parent());
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_skip_indicators(true);
    let font_loader = create_font_loader();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    let rendered = pipeline.render_html(&html, &linked_css);
    println!("{}", rendered.layout.snapshot());
    for i in 0..12 {
        if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
            println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }

    let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-001-ref.xht");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
    let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
    ref_pipeline.set_skip_indicators(true);
    let ref_font_loader = create_font_loader();
    ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
    let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
    println!("--- ref ---");
    println!("{}", ref_rendered.layout.snapshot());
    for i in 0..12 {
        if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
            println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
}

#[test]
#[ignore]
fn debug_clear_clearance_calculation_003_layout_snapshot() {
    let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-003.xht");
    let html = std::fs::read_to_string(&case_path).expect("read test html");
    let linked_css = load_linked_stylesheets(&html, case_path.parent());
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_skip_indicators(true);
    let font_loader = create_font_loader();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    let rendered = pipeline.render_html(&html, &linked_css);
    println!("{}", rendered.layout.snapshot());
    for i in 0..14 {
        if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
            println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }

    let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-003-ref.xht");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
    let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
    ref_pipeline.set_skip_indicators(true);
    let ref_font_loader = create_font_loader();
    ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
    let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
    println!("--- ref ---");
    println!("{}", ref_rendered.layout.snapshot());
    for i in 0..14 {
        if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
            println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
}

#[test]
#[ignore]
fn debug_clear_clearance_calculation_004_layout_snapshot() {
    let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-004.xht");
    let html = std::fs::read_to_string(&case_path).expect("read test html");
    let linked_css = load_linked_stylesheets(&html, case_path.parent());
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_skip_indicators(true);
    let font_loader = create_font_loader();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    let rendered = pipeline.render_html(&html, &linked_css);
    println!("{}", rendered.layout.snapshot());
    for i in 0..14 {
        if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
            println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }

    let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-004-ref.xht");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
    let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
    ref_pipeline.set_skip_indicators(true);
    let ref_font_loader = create_font_loader();
    ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
    let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
    println!("--- ref ---");
    println!("{}", ref_rendered.layout.snapshot());
    for i in 0..14 {
        if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
            println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
}

#[test]
#[ignore]
fn debug_clear_clearance_calculation_005_layout_snapshot() {
    let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-005.xht");
    let html = std::fs::read_to_string(&case_path).expect("read test html");
    let linked_css = load_linked_stylesheets(&html, case_path.parent());
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_skip_indicators(true);
    let font_loader = create_font_loader();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    let rendered = pipeline.render_html(&html, &linked_css);
    println!("{}", rendered.layout.snapshot());
    for i in 0..16 {
        if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
            println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
    for (i, fill) in rendered.primitives.fills.iter().enumerate().take(16) {
        println!(
            "fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
            fill.rect.origin.x,
            fill.rect.origin.y,
            fill.rect.size.width,
            fill.rect.size.height,
            fill.color.r,
            fill.color.g,
            fill.color.b,
            fill.color.a
        );
    }

    let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-005-ref.xht");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
    let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
    ref_pipeline.set_skip_indicators(true);
    let ref_font_loader = create_font_loader();
    ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
    let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
    println!("--- ref ---");
    println!("{}", ref_rendered.layout.snapshot());
    for i in 0..16 {
        if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
            println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
}

#[test]
#[ignore]
fn debug_clear_003_layout_snapshot() {
    let case_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/floats-clear/clear-003.xht");
    let html = std::fs::read_to_string(&case_path).expect("read test html");
    let linked_css = load_linked_stylesheets(&html, case_path.parent());
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_skip_indicators(true);
    let font_loader = create_font_loader();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    let rendered = pipeline.render_html(&html, &linked_css);
    println!("{}", rendered.layout.snapshot());
    for i in 0..12 {
        if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
            println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
    for (i, fill) in rendered.primitives.fills.iter().enumerate().take(12) {
        println!(
            "fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
            fill.rect.origin.x,
            fill.rect.origin.y,
            fill.rect.size.width,
            fill.rect.size.height,
            fill.color.r,
            fill.color.g,
            fill.color.b,
            fill.color.a
        );
    }

    let ref_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/floats-clear/clear-003-ref.xht");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
    let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
    ref_pipeline.set_skip_indicators(true);
    let ref_font_loader = create_font_loader();
    ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
    let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
    println!("--- ref ---");
    println!("{}", ref_rendered.layout.snapshot());
    for i in 0..12 {
        if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
            println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
    for (i, fill) in ref_rendered.primitives.fills.iter().enumerate().take(12) {
        println!(
            "ref fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
            fill.rect.origin.x,
            fill.rect.origin.y,
            fill.rect.size.width,
            fill.rect.size.height,
            fill.color.r,
            fill.color.g,
            fill.color.b,
            fill.color.a
        );
    }
}

#[test]
#[ignore]
fn debug_clear_float_003_layout_snapshot() {
    let case_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/floats-clear/clear-float-003.xht");
    let html = std::fs::read_to_string(&case_path).expect("read test html");
    let linked_css = load_linked_stylesheets(&html, case_path.parent());
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_skip_indicators(true);
    let font_loader = create_font_loader();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    let rendered = pipeline.render_html(&html, &linked_css);
    println!("{}", rendered.layout.snapshot());
    for i in 0..12 {
        if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
            println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
    for (i, fill) in rendered.primitives.fills.iter().enumerate().take(12) {
        println!(
            "fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
            fill.rect.origin.x,
            fill.rect.origin.y,
            fill.rect.size.width,
            fill.rect.size.height,
            fill.color.r,
            fill.color.g,
            fill.color.b,
            fill.color.a
        );
    }

    let ref_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/floats-clear/clear-float-003-ref.xht");
    let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
    let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
    let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
    ref_pipeline.set_skip_indicators(true);
    let ref_font_loader = create_font_loader();
    ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
    let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
    println!("--- ref ---");
    println!("{}", ref_rendered.layout.snapshot());
    for i in 0..12 {
        if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
            println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
        }
    }
    for (i, fill) in ref_rendered.primitives.fills.iter().enumerate().take(12) {
        println!(
            "ref fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
            fill.rect.origin.x,
            fill.rect.origin.y,
            fill.rect.size.width,
            fill.rect.size.height,
            fill.color.r,
            fill.color.g,
            fill.color.b,
            fill.color.a
        );
    }
}

// ── 分类容差测试 ──

#[test]
fn test_category_from_path_layout() {
    assert_eq!(
        ReftestCategory::from_path("css/CSS2/box-001.html"),
        ReftestCategory::Layout
    );
    assert_eq!(
        ReftestCategory::from_path("css/css-flexbox/001.html"),
        ReftestCategory::Layout
    );
}

#[test]
fn test_category_from_path_text() {
    assert_eq!(
        ReftestCategory::from_path("css/css-text/001.html"),
        ReftestCategory::Text
    );
    assert_eq!(
        ReftestCategory::from_path("css/css-fonts/001.html"),
        ReftestCategory::Text
    );
}

#[test]
fn test_category_defaults() {
    assert_eq!(ReftestCategory::Layout.default_max_diff_ratio(), 0.01);
    assert_eq!(ReftestCategory::Text.default_max_diff_ratio(), 0.05);
    assert_eq!(ReftestCategory::Layout.default_max_channel_diff(), 5);
    assert_eq!(ReftestCategory::Text.default_max_channel_diff(), 15);
}

#[test]
fn test_config_for_category() {
    let config = ReftestConfig::for_category(ReftestCategory::Text);
    assert!((config.max_diff_ratio - 0.05).abs() < f64::EPSILON);
    assert_eq!(config.max_channel_diff, 15);
}

/// DC-14 锁定严格容差不变量：Layout 0.1%/2、Text 0.5%/5、Unknown 0.1%/2。
/// 严格容差是默认松容差（Layout 1%/5、Text 5%/15）的 1/10（R280 量化），
/// 是唯一可信达标指标（goal DC-14 line 162-163/315-316，不可放宽）。
#[test]
fn test_strict_tolerance_dc14_locked() {
    // Layout: 默认 1% / 5 → 严格 0.1% / 2
    assert!((ReftestCategory::Layout.strict_max_diff_ratio() - 0.001).abs() < f64::EPSILON);
    assert_eq!(ReftestCategory::Layout.strict_max_channel_diff(), 2);
    assert!((ReftestCategory::Layout.default_max_diff_ratio() - 0.01).abs() < f64::EPSILON);
    assert_eq!(ReftestCategory::Layout.default_max_channel_diff(), 5);

    // Text: 默认 5% / 15 → 严格 0.5% / 5
    assert!((ReftestCategory::Text.strict_max_diff_ratio() - 0.005).abs() < f64::EPSILON);
    assert_eq!(ReftestCategory::Text.strict_max_channel_diff(), 5);
    assert!((ReftestCategory::Text.default_max_diff_ratio() - 0.05).abs() < f64::EPSILON);
    assert_eq!(ReftestCategory::Text.default_max_channel_diff(), 15);

    // Unknown: 默认 2% / 8 → 严格 0.1% / 2（未知分类按最严格处理）
    assert!((ReftestCategory::Unknown.strict_max_diff_ratio() - 0.001).abs() < f64::EPSILON);
    assert_eq!(ReftestCategory::Unknown.strict_max_channel_diff(), 2);

    // 严格恒为默认的 1/10（10× 松 → 严格，R280 量化）
    for cat in [ReftestCategory::Layout, ReftestCategory::Text] {
        let ratio_factor = cat.default_max_diff_ratio() / cat.strict_max_diff_ratio();
        assert!(
            (ratio_factor - 10.0).abs() < 1e-9,
            "strict ratio should be 1/10 of default"
        );
    }
}

#[test]
fn test_fuzzy_override() {
    let mut config = ReftestConfig::for_category(ReftestCategory::Layout);
    let fuzzy = FuzzyMeta {
        max_diff: Some(20),
        total_pixels: Some(500),
    };
    config.with_fuzzy_override(&fuzzy);
    assert_eq!(config.max_channel_diff, 20);
    // total_pixels=500, viewport=800x600=480000, ratio=500/480000≈0.001
    assert!(config.max_diff_ratio < 0.01);
}

// --- CSS 布局 reftest 用例 ---

/// 辅助函数：使用默认配置运行 match reftest。
fn assert_match(id: &str, test_html: &str, ref_html: &str) {
    let case = ReftestCase {
        id: id.into(),
        test_html: test_html.into(),
        ref_html: ref_html.into(),
        css: String::new(),
        is_match: true,
        ref_base_dir: None,
    };
    let config = ReftestConfig {
        viewport_width: 200,
        viewport_height: 200,
        ..Default::default()
    };
    let result = run_reftest(&case, &config);
    assert!(result.passed, "{}: {}", id, result.message);
}

/// 辅助函数：使用默认配置运行 mismatch reftest。
fn assert_mismatch(id: &str, test_html: &str, ref_html: &str) {
    let case = ReftestCase {
        id: id.into(),
        test_html: test_html.into(),
        ref_html: ref_html.into(),
        css: String::new(),
        is_match: false,
        ref_base_dir: None,
    };
    let config = ReftestConfig {
        viewport_width: 200,
        viewport_height: 200,
        ..Default::default()
    };
    let result = run_reftest(&case, &config);
    assert!(result.passed, "{}: {}", id, result.message);
}

// ── Block 布局 ──

#[test]
fn reftest_block_width_height() {
    assert_match(
        "block/width-height",
        "<div style=\"width:100px;height:80px;background:red;\"></div>",
        "<div style=\"width:100px;height:80px;background:red;\"></div>",
    );
}

#[test]
fn reftest_block_margin_collapsing() {
    assert_match(
        "block/margin-no-effect-on-bg",
        "<div style=\"width:100px;height:50px;background:blue;margin:10px;\"></div>",
        "<div style=\"width:100px;height:50px;background:blue;margin:10px;\"></div>",
    );
}

#[test]
fn reftest_block_different_margin() {
    assert_mismatch(
        "block/different-margin",
        "<div style=\"width:80px;height:40px;background:green;margin:0;\"></div>",
        "<div style=\"width:80px;height:40px;background:green;margin:20px;\"></div>",
    );
}

#[test]
fn reftest_block_stacking() {
    assert_mismatch(
        "block/stacking-vs-single",
        "<div style=\"width:100px;height:40px;background:red;\"></div><div style=\"width:100px;height:40px;background:blue;\"></div>",
        "<div style=\"width:100px;height:80px;background:red;\"></div>",
    );
}

// ── 盒模型 ──

#[test]
fn reftest_padding_expands_box() {
    assert_mismatch(
        "box-model/padding-expands",
        "<div style=\"width:80px;height:40px;background:red;padding:10px;\"></div>",
        "<div style=\"width:80px;height:40px;background:red;padding:0;\"></div>",
    );
}

#[test]
fn reftest_border_visible() {
    assert_mismatch(
        "box-model/border-visible",
        "<div style=\"width:80px;height:40px;background:yellow;border:2px solid black;\"></div>",
        "<div style=\"width:80px;height:40px;background:yellow;border:none;\"></div>",
    );
}

// ── Flexbox ──

#[test]
fn reftest_flex_direction_row() {
    assert_match(
        "flex/row-identical",
        "<div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
        "<div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
    );
}

#[test]
fn reftest_flex_vs_block() {
    assert_mismatch(
        "flex/row-vs-block",
        "<div style=\"display:flex;width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
        "<div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
    );
}

// ── 定位 ──

#[test]
fn reftest_absolute_position() {
    assert_mismatch(
        "position/absolute-shift",
        "<div style=\"position:relative;width:200px;height:100px;\"><div style=\"position:absolute;top:20px;left:20px;width:50px;height:50px;background:green;\"></div></div>",
        "<div style=\"position:relative;width:200px;height:100px;\"><div style=\"position:absolute;top:0;left:0;width:50px;height:50px;background:green;\"></div></div>",
    );
}

// ── 背景颜色 ──

#[test]
fn reftest_named_vs_hex_color() {
    assert_match(
        "color/named-vs-hex",
        "<div style=\"width:100px;height:50px;background:red;\"></div>",
        "<div style=\"width:100px;height:50px;background:#FF0000;\"></div>",
    );
}

#[test]
fn reftest_rgb_vs_hex() {
    assert_match(
        "color/rgb-vs-hex",
        "<div style=\"width:100px;height:50px;background:rgb(0,128,255);\"></div>",
        "<div style=\"width:100px;height:50px;background:#0080FF;\"></div>",
    );
}

#[test]
fn reftest_different_colors() {
    assert_mismatch(
        "color/different",
        "<div style=\"width:100px;height:50px;background:red;\"></div>",
        "<div style=\"width:100px;height:50px;background:green;\"></div>",
    );
}

// ── 尺寸 ──

#[test]
fn reftest_different_sizes() {
    assert_mismatch(
        "size/different",
        "<div style=\"width:100px;height:50px;background:blue;\"></div>",
        "<div style=\"width:50px;height:100px;background:blue;\"></div>",
    );
}

#[test]
fn reftest_display_none() {
    assert_mismatch(
        "display/none-vs-visible",
        "<div style=\"width:100px;height:50px;background:red;\"></div>",
        "<div style=\"width:100px;height:50px;background:red;display:none;\"></div>",
    );
}

// ── 嵌套结构 ──

#[test]
fn reftest_nested_same_bg() {
    assert_match(
        "nested/same-structure",
        "<div style=\"width:100px;height:80px;background:red;\"><div style=\"width:50px;height:40px;background:blue;\"></div></div>",
        "<div style=\"width:100px;height:80px;background:red;\"><div style=\"width:50px;height:40px;background:blue;\"></div></div>",
    );
}

#[test]
fn reftest_sibling_order() {
    assert_mismatch(
        "nested/sibling-order",
        "<div style=\"width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
        "<div style=\"width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:red;\"></div></div>",
    );
}

// DC-13 产品静态 smoke：渲染 morning.work 中文文章 fixture（含外链 CSS + 图片）。
// 通过 base_dir 测试 <link> 外链 CSS 与 <img> 子资源加载路径。
#[test]
#[ignore]
fn dump_morning_work_png() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/browser/assets/morning-work");
    let html = std::fs::read_to_string(base.join("article.html")).expect("read article.html");
    let config = ReftestConfig::default();
    let fb = render_to_framebuffer_with_base(&html, "", &config, Some(&base));
    let out = std::path::Path::new("/tmp/mw-zeroweb-cpu.png");
    save_fb_as_png(&fb, out);
    // 报告关键区域像素：文章正文区应有 CJK 文本（深色像素），代码块应有背景
    let px = fb.data;
    let w = fb.width as usize;
    let at = |x: usize, y: usize| -> (u8, u8, u8, u8) {
        let i = (y * w + x) * 4;
        (px[i], px[i + 1], px[i + 2], px[i + 3])
    };
    println!("morning.work samples (CJK 文本应深色，代码块应有灰背景):");
    for &(x, y) in &[(60, 80), (100, 150), (100, 250), (100, 400)] {
        println!("  ({},{}) = {:?}", x, y, at(x, y));
    }
    // 统计非背景像素（页面 bg #f9f7f4 ≈ (249,247,244)）
    let mut non_bg = 0usize;
    for i in (0..px.len()).step_by(4) {
        let c = (px[i], px[i + 1], px[i + 2]);
        if !(c.0 > 245 && c.1 > 243 && c.2 > 240) {
            non_bg += 1;
        }
    }
    println!("non-background pixels: {} (of {})", non_bg, px.len() / 4);
}

/// R878：根元素 `display:none` 时背景不传播到画布（CSS §9.2.4/§14.2）。
///
/// `html{display:none;background:green}` + `body{background:red}` + `<p>FAIL</p>`
/// ——根元素不生成盒、整个文档树不参与渲染，故根元素与 body 背景均不传播到 canvas，
/// canvas 保持默认白（与 chromium 实测一致，root-box-003）。
#[test]
fn test_root_element_display_none_no_canvas_background() {
    let html = r#"<html><head><style>
   html { display: none; background: green; color: red; }
   body { background: red; color: yellow; }
  </style></head>
  <body>
   <p>FAIL</p>
  </body></html>"#;
    let cfg = ReftestConfig::default();
    let fb = render_to_framebuffer(html, "", &cfg);
    let px = &fb.data;
    // 全画布应为白（无背景传播，无文档树渲染）
    for chunk in px.chunks(4) {
        if chunk.len() == 4 {
            assert_eq!(
                [chunk[0], chunk[1], chunk[2]],
                [255, 255, 255],
                "canvas must stay white"
            );
        }
    }
}

/// R878 回归守卫：根元素背景正常传播到画布不受 display:none 修复影响。
///
/// `html{background:green}`（无 display:none）→ 全画布绿（canvas 传播仍工作）。
#[test]
fn test_root_element_background_still_propagates_to_canvas() {
    let html = r#"<html><head><style>
   html { background: green; }
  </style></head>
  <body></body></html>"#;
    let cfg = ReftestConfig::default();
    let fb = render_to_framebuffer(html, "", &cfg);
    let px = &fb.data;
    // 全画布应为绿（根背景传播）
    for chunk in px.chunks(4) {
        if chunk.len() == 4 {
            assert_eq!([chunk[0], chunk[1], chunk[2]], [0, 128, 0], "canvas must be green");
        }
    }
}

/// R879：`background:transparent`（解析为 `background-image:none` → `vec![None]`）
/// 不应阻止 body 背景传播到画布。CSS §14.2：html 背景透明时 body 背景传播到 canvas。
///
/// `body{background:green}` + `html{background:transparent}` → 全画布绿
///（background-root-005 等 cluster，R721 定位、R879 修 `vec![None]` 误判）。
#[test]
fn test_background_image_none_does_not_block_body_canvas_propagation() {
    let html = r#"<html><head><style>
   body { background: green; color: white; }
   html { background: transparent; color: yellow; }
  </style></head>
  <body>
   <p>body green should fill canvas</p>
  </body></html>"#;
    let cfg = ReftestConfig::default();
    let fb = render_to_framebuffer(html, "", &cfg);
    let px = &fb.data;
    // 绝大多数像素应为绿（body green 传播到画布）；允许少量文本反走样像素
    let mut green = 0usize;
    let mut total = 0usize;
    for chunk in px.chunks(4) {
        if chunk.len() == 4 {
            total += 1;
            if chunk[1] >= 100 && chunk[0] < 80 && chunk[2] < 80 {
                green += 1;
            }
        }
    }
    assert!(
        green * 100 > total * 90,
        "canvas must be predominantly green: {green}/{total}"
    );
}

/// R880：无 positioned 祖先的 abspos 元素以初始包含块（视口）为 CB，其
/// `left/top`（Px 或百分比）与百分比 `width/height` 相对**视口**解析，不受
/// CB 链上祖先 border/padding 影响（CSS §10.1/§10.3/§10.6）。
///
/// `body{border+padding:1em}` + `div{position:absolute;top:0;left:0;width:100%;
/// height:100%;background:green}` → div 覆盖整个视口（绿），body 的红不外露。
/// 旧实现把 div 放在父 content origin（=border-box+border+padding=48px）而非视口
/// (0,0)，致左上 L 形红区（abspos-containing-block-010，9.35% diff）。
#[test]
fn test_abspos_viewport_cb_ignores_ancestor_border_padding() {
    let html = r#"<!DOCTYPE html><html><head><style>
   body { margin: 1em; border: 1em solid red; padding: 1em; background: red; }
   div { position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: green; }
  </style></head>
  <body>
   <p>FAIL</p>
   <div>x</div>
  </body></html>"#;
    let cfg = ReftestConfig::default();
    let fb = render_to_framebuffer(html, "", &cfg);
    let px = &fb.data;
    // div 覆盖整个视口：几乎全绿，无任何红（body 红被完全覆盖）
    let mut red = 0usize;
    for chunk in px.chunks(4) {
        if chunk.len() == 4 && chunk[0] >= 150 && chunk[1] < 80 && chunk[2] < 80 {
            red += 1;
        }
    }
    assert_eq!(
        red, 0,
        "no red (body bg) must be visible; abspos div must cover viewport"
    );
}
/// R881：`float:left` 容器（width:auto）应 shrink-to-fit 包裹其 inline-level
/// replaced 子元素（img），CSS §10.3.5。旧 float shrink 只考虑 block-level 子元素，
/// 致 `div{float:left}` 仅含 `<img>` 时撑满全宽，img 无法覆盖 div 背景（max-width-110，
/// 200×200 img 受 max-width:100px 约束为 100×100，但 div 784px 满宽→红 68400px 外露）。
#[test]
fn test_float_shrink_to_fit_includes_inline_replaced_child() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/normal-flow");
    let img = base.join("support/green200x200.png");
    if !img.exists() {
        eprintln!("[R881] green200x200.png missing, skipping");
        return;
    }
    let html = r#"<!DOCTYPE html><html><head><style>
  div { background-color: red; float: left; }
  img { height: auto; max-width: 100px; vertical-align: bottom; width: auto; }
  </style></head>
  <body>
  <p>x</p>
  <div><img src="support/green200x200.png" alt="x" /></div>
  </body></html>"#;
    let cfg = ReftestConfig::default();
    let fb = render_to_framebuffer_with_base(html, "", &cfg, Some(&base));
    let px = &fb.data;
    // div shrink 包裹 img（100×100）→ green img 完全覆盖 div，无 red 外露
    let mut red = 0usize;
    for chunk in px.chunks(4) {
        if chunk.len() == 4 && chunk[0] >= 150 && chunk[1] < 80 && chunk[2] < 80 {
            red += 1;
        }
    }
    assert_eq!(red, 0, "float div must shrink-to-fit img; no red bg must be visible");
}

/// R988 端到端回归门禁：background-root-101/102 类（onload setTimeout + className
/// mutation）经 harness 应用 JS mutation 后必须渲染绿色 canvas。覆盖 V8-init、
/// setTimeout-onload、className-mutation 捕获、dom serializer 保留 `<style>` CDATA
/// （R917 续）、head+body 相邻兄弟选择器、§14.2 canvas 背景传播全链。
/// 任一环节回归 → body 不绿。
#[test]
fn test_r988_background_root_render_after_mutation() {
    let script = r#"<script type="text/javascript">
    function test() {
      document.getElementsByTagName('$ROOT')[0].className = 'after';
      document.getElementsByTagName('p')[0].className = 'after';
      document.documentElement.className = "";
    }
  </script>"#;
    // 102：body.class mutation（无兄弟选择器）。
    let html_102 = format!(
        r#"<html class="reftest-wait"><head><style><![CDATA[
    body.before {{ background: red; }} body.after {{ background: green; }}
  ]]></style>{script}</head>
 <body class="before" onload="setTimeout(test, 5)"><p class="before">x</p></body></html>"#,
        script = script.replace("$ROOT", "body")
    );
    // 101：head+body 相邻兄弟选择器（JS 改 head.class）。
    let html_101 = format!(
        r#"<html class="reftest-wait"><head class="before"><style><![CDATA[
    head.before + body {{ background: red; }} head.after + body {{ background: green; }}
  ]]></style>{script}</head>
 <body onload="setTimeout(test, 5)"><p class="before">x</p></body></html>"#,
        script = script.replace("$ROOT", "head")
    );

    let cfg = ReftestConfig::default();
    let green_pct = |html: &str| -> usize {
        let fb = render_to_framebuffer(html, "", &cfg);
        let (w, h) = (fb.width as usize, fb.height as usize);
        let (mut green, mut total) = (0usize, 0usize);
        for y in (h / 2..h).step_by(4) {
            for x in (0..w).step_by(4) {
                let i = (y * w + x) * 4;
                if i + 2 < fb.data.len() {
                    total += 1;
                    if fb.data[i + 1] > 80 && fb.data[i] < 100 && fb.data[i + 2] < 100 {
                        green += 1;
                    }
                }
            }
        }
        green * 100 / total.max(1)
    };
    let pct102 = green_pct(&html_102);
    let pct101 = green_pct(&html_101);
    assert!(
        pct102 > 50,
        "102 body.class mutation must paint green canvas (got {pct102}%) — harness-JS or canvas-propagation regression"
    );
    assert!(
        pct101 > 50,
        "101 head+body sibling selector must paint green canvas after head.class mutation (got {pct101}%) — sibling-selector or serializer regression"
    );
}
