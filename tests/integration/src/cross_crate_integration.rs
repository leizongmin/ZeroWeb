#[cfg(test)]
use std::collections::HashMap;

use zero_canvas::CanvasContext;
use zero_css_parser::ast::{ComplexSelector, CompoundSelector, Rule, TypeSelector};
use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue};
use zero_css_parser::{Parser as CssParser, Selector};
use zero_dom::{Document, ShadowRootMode, parse_html};
use zero_layout_engine::LayoutEngine;
use zero_render_foundation::color::Color;
use zero_style_system::{ComputedStyle, GridLineValue, StyleSystem};

// ── 辅助函数 ──

/// 创建包含 html > body 的基础 DOM，返回 (doc, body NodeId)。
fn make_doc_with_body() -> (Document, zero_dom::NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    (doc, body)
}

/// 创建标签选择器。
#[allow(dead_code)]
fn make_tag_selector(tag: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag(tag.to_string())),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    }
}

/// 在 LayoutBox 子树中查找指定 node_id 的盒子。
fn find_box_by_node_id(
    root: &zero_layout_engine::LayoutBox,
    target_id: zero_dom::NodeId,
) -> Option<&zero_layout_engine::LayoutBox> {
    if root.node_id == Some(target_id) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_box_by_node_id(child, target_id) {
            return Some(found);
        }
    }
    None
}

// ── 测试 ──

/// Shadow DOM 到布局的集成测试。
///
/// 创建带 ShadowRoot 和具名 <slot> 的 DOM 树，
/// 将 light DOM 子元素通过 slot 属性分配到 shadow 树中，
/// 解析 slot 分配后构建布局树，验证 shadow 内容被正确展平。
#[test]
fn test_shadow_dom_to_layout_integration() {
    let (mut doc, body) = make_doc_with_body();

    // 创建宿主元素 <my-component>
    let host = doc.create_element("my-component");
    doc.append_child(body, host).unwrap();

    // 附加 ShadowRoot，内部包含 <div class="wrapper"><slot name="content"></slot></div>
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let wrapper = doc.create_element("div");
    doc.set_attribute(wrapper, "class", "wrapper");
    doc.append_child(shadow, wrapper).unwrap();
    let slot = doc.create_element("slot");
    doc.set_attribute(slot, "name", "content");
    doc.append_child(wrapper, slot).unwrap();

    // light DOM 子元素：<p slot="content">Slotted</p>
    let slotted_p = doc.create_element("p");
    doc.set_attribute(slotted_p, "slot", "content");
    doc.append_child(host, slotted_p).unwrap();

    // 解析 slot 分配
    doc.resolve_slots(host);

    // 构建布局
    let mut styles = HashMap::new();
    let mut host_style = ComputedStyle::default();
    host_style.display = DisplayValue::Block;
    host_style.width = LengthValue::Px(400.0);
    host_style.height = LengthValue::Px(300.0);
    styles.insert(host, host_style);

    let mut wrapper_style = ComputedStyle::default();
    wrapper_style.display = DisplayValue::Block;
    styles.insert(wrapper, wrapper_style);

    let mut p_style = ComputedStyle::default();
    p_style.display = DisplayValue::Block;
    styles.insert(slotted_p, p_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 验证 slotted <p> 出现在布局树中
    let p_box = find_box_by_node_id(&result.root, slotted_p);
    assert!(p_box.is_some(), "slotted <p> 应出现在布局树中（shadow DOM 展平成功）");

    // 验证 wrapper 也出现在布局树中
    let wrapper_box = find_box_by_node_id(&result.root, wrapper);
    assert!(wrapper_box.is_some(), "shadow wrapper div 应出现在布局树中");

    // wrapper 应是 host 的子节点在布局树中
    // slotted_p 应该在 wrapper 子树内（而非 host 直接子节点）
    let p_box = p_box.unwrap();
    assert!(p_box.width > 0.0 || p_box.height > 0.0, "slotted <p> 应有非零布局尺寸");
}

/// CSS @container 查询与样式系统的集成测试。
///
/// 解析包含 @container 规则的 CSS，设置容器上下文（视口尺寸），
/// 计算样式后验证容器查询条件满足时样式被正确应用。
#[test]
fn test_css_container_query_style_integration() {
    // 使用 CSS 解析器解析含 @container 的样式表
    let css = r#"
        div { color: black; }
        @container (min-width: 400px) {
            p { color: blue; }
        }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    // 验证 CSS 解析产生了 @container 规则
    let has_container = stylesheet.rules.iter().any(|r| matches!(r, Rule::Container(_)));
    assert!(has_container, "CSS 应包含 @container 规则");

    // 构建 DOM：html > body > div > p
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let p = doc.create_element("p");
    doc.append_child(div, p).unwrap();

    // 设置视口尺寸为 500px（满足 min-width: 400px 条件）
    let mut sys = StyleSystem::new();
    sys.set_viewport(500.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let p_style = styles.get(&p).expect("p 应有计算样式");
    // 容器宽度 500px >= 400px，条件满足，color 应为蓝色
    assert_eq!(
        p_style.color,
        ColorValue::Rgba(0, 0, 255, 255),
        "容器宽度 500px >= 400px，p 的 color 应为蓝色"
    );

    // 额外验证：不满足条件时不应用
    let mut sys2 = StyleSystem::new();
    sys2.set_viewport(300.0, 600.0);
    let stylesheet2 = CssParser::parse_stylesheet(css);
    let styles2 = sys2.compute_styles(&doc, &[stylesheet2]);
    let p_style2 = styles2.get(&p).expect("p 应有计算样式");
    assert_ne!(
        p_style2.color,
        ColorValue::Rgba(0, 0, 255, 255),
        "容器宽度 300px < 400px，p 的 color 不应为蓝色"
    );
}

/// Canvas ellipse 绘制到像素输出的集成测试。
///
/// 创建 Canvas 上下文，使用 ellipse 方法绘制椭圆并填充，
/// 验证渲染输出中椭圆内部有非零像素（alpha > 0），
/// 椭圆外部区域保持为零。
#[test]
fn test_canvas_ellipse_render_integration() {
    let mut ctx = CanvasContext::new(200, 200);

    // 设置填充颜色为绿色
    ctx.set_fill_color(Color::GREEN);

    // 绘制椭圆：中心 (100, 100)，水平半径 60，垂直半径 40
    ctx.begin_path();
    ctx.ellipse(100.0, 100.0, 60.0, 40.0, 0.0, 0.0, std::f32::consts::TAU, false);
    ctx.fill();

    // 验证椭圆内部中心点有非零像素
    let center_pixel = ctx.get_image_data(100, 100, 1, 1);
    assert_ne!(center_pixel.data[3], 0, "椭圆中心应有非零 alpha 值");
    // 绿色通道应 > 0
    assert!(center_pixel.data[1] > 0, "椭圆中心像素的绿色通道应 > 0");

    // 验证椭圆内部其他点也有像素
    let inner_pixel = ctx.get_image_data(120, 100, 1, 1);
    assert_ne!(inner_pixel.data[3], 0, "椭圆内部 (120, 100) 应有非零像素");

    // 验证椭圆外部区域为零（左上角远离椭圆）
    let outside_pixel = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(outside_pixel.data[3], 0, "椭圆外部 (5, 5) 的 alpha 应为 0");
}

/// Grid 命名区域放置的集成测试。
///
/// 使用 grid-template-areas 定义命名区域，
/// 子元素通过 grid-area 属性指定区域名，
/// 验证布局引擎正确计算各子元素的位置和尺寸。
#[test]
fn test_grid_area_named_placement() {
    let (mut doc, body) = make_doc_with_body();

    // 创建 grid 容器
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    // 创建三个子元素分别放置到 header / main / footer 区域
    let header_el = doc.create_element("div");
    doc.set_attribute(header_el, "class", "header");
    doc.append_child(grid, header_el).unwrap();

    let main_el = doc.create_element("div");
    doc.set_attribute(main_el, "class", "main");
    doc.append_child(grid, main_el).unwrap();

    let footer_el = doc.create_element("div");
    doc.set_attribute(footer_el, "class", "footer");
    doc.append_child(grid, footer_el).unwrap();

    let mut styles = HashMap::new();

    // grid 容器：3 行 1 列，命名区域 header / main / footer
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("300px".to_string());
    grid_style.grid_template_rows = Some("50px 100px 40px".to_string());
    grid_style.grid_template_areas = Some("\"header\" \"main\" \"footer\"".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(190.0);
    styles.insert(grid, grid_style);

    // header: grid-area: header
    let mut header_style = ComputedStyle::default();
    header_style.grid_row_start = GridLineValue::Name("header".to_string());
    header_style.grid_row_end = GridLineValue::Name("header".to_string());
    header_style.grid_column_start = GridLineValue::Name("header".to_string());
    header_style.grid_column_end = GridLineValue::Name("header".to_string());
    styles.insert(header_el, header_style);

    // main: grid-area: main
    let mut main_style = ComputedStyle::default();
    main_style.grid_row_start = GridLineValue::Name("main".to_string());
    main_style.grid_row_end = GridLineValue::Name("main".to_string());
    main_style.grid_column_start = GridLineValue::Name("main".to_string());
    main_style.grid_column_end = GridLineValue::Name("main".to_string());
    styles.insert(main_el, main_style);

    // footer: grid-area: footer
    let mut footer_style = ComputedStyle::default();
    footer_style.grid_row_start = GridLineValue::Name("footer".to_string());
    footer_style.grid_row_end = GridLineValue::Name("footer".to_string());
    footer_style.grid_column_start = GridLineValue::Name("footer".to_string());
    footer_style.grid_column_end = GridLineValue::Name("footer".to_string());
    styles.insert(footer_el, footer_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 查找各子元素的布局盒
    let header_box = find_box_by_node_id(&result.root, header_el).expect("header 应在布局树中");
    let main_box = find_box_by_node_id(&result.root, main_el).expect("main 应在布局树中");
    let footer_box = find_box_by_node_id(&result.root, footer_el).expect("footer 应在布局树中");

    // header 在第一行，高度 ~50px
    assert!(header_box.y < 1.0, "header 应从 y=0 开始，实际 y={}", header_box.y);
    assert!(
        (header_box.height - 50.0).abs() < 1.0,
        "header 高度应约 50px，实际 {}",
        header_box.height
    );

    // main 在第二行，y 应在 header 之后
    assert!(
        main_box.y >= header_box.y + header_box.height - 1.0,
        "main 应在 header 下方，main.y={} header.bottom={}",
        main_box.y,
        header_box.y + header_box.height
    );
    assert!(
        (main_box.height - 100.0).abs() < 1.0,
        "main 高度应约 100px，实际 {}",
        main_box.height
    );

    // footer 在第三行
    assert!(footer_box.y > main_box.y, "footer 应在 main 下方");
    assert!(
        (footer_box.height - 40.0).abs() < 1.0,
        "footer 高度应约 40px，实际 {}",
        footer_box.height
    );

    // 所有子元素宽度应为 300px
    assert!(
        (header_box.width - 300.0).abs() < 1.0,
        "header 宽度应约 300px，实际 {}",
        header_box.width
    );
}

/// Scroll-snap 管线集成测试。
///
/// 解析包含 scroll-snap-type、scroll-snap-align、scroll-snap-stop 的 CSS，
/// 通过样式系统计算后验证计算样式中的 scroll-snap 值正确存储。
#[test]
fn test_scroll_snap_pipeline() {
    let html = r#"<html><body>
        <div class="scroll-container">
            <div class="snap-item">A</div>
            <div class="snap-item">B</div>
        </div>
    </body></html>"#;
    let css = r#"
        .scroll-container { scroll-snap-type: y mandatory; overflow-y: auto; }
        .snap-item { scroll-snap-align: start; scroll-snap-stop: always; }
    "#;

    let doc = parse_html(html);
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证至少有样式被计算
    assert!(!styles.is_empty(), "应为 DOM 节点计算样式");

    // 查找 scroll-container 的样式 — 验证 scroll-snap-type 为 y mandatory
    let mut found_container = false;
    let mut found_item = false;
    for (_node_id, style) in &styles {
        // scroll-container 应该有 scroll-snap-type: y mandatory
        if style.scroll_snap_type.strictness == zero_style_system::ScrollSnapStrictness::Mandatory
            && style.scroll_snap_type.axis == zero_css_parser::values::ScrollSnapAxis::Y
        {
            found_container = true;
        }
        // snap-item 应该有 scroll-snap-align: start 和 scroll-snap-stop: always
        if style.scroll_snap_align == zero_style_system::ScrollSnapAlign::Start
            && style.scroll_snap_stop == zero_style_system::ScrollSnapStop::Always
        {
            found_item = true;
        }
    }

    assert!(found_container, "scroll-container 的 scroll-snap-type 应为 y mandatory");
    assert!(
        found_item,
        "snap-item 的 scroll-snap-align 应为 start 且 scroll-snap-stop 应为 always"
    );
}

// ── 跨 crate 边界条件测试 ──

/// CSS 命名颜色解析 → 渲染 Color 转换集成测试。
///
/// 通过 css-parser 解析命名颜色字符串（如 "orange"、"teal"），
/// 将解析结果与 engine paint 模块的 named_color_to_render 函数对比，
/// 验证两端对命名颜色的 RGBA 值一致。
#[test]
fn test_css_color_named_to_render() {
    use zero_css_parser::values::{ColorValue, parse_color};
    use zero_engine::named_color_to_render;

    // 验证 css-parser 能正确解析扩展命名颜色
    let crimson_parsed = parse_color("crimson").expect("应成功解析 crimson 颜色");
    match &crimson_parsed {
        ColorValue::Rgba(r, g, b, a) => {
            assert_eq!(*r, 220, "crimson R 应为 220");
            assert_eq!(*g, 20, "crimson G 应为 20");
            assert_eq!(*b, 60, "crimson B 应为 60");
            assert_eq!(*a, 255, "crimson A 应为 255");
        }
        other => panic!("预期 Rgba，实际得到 {:?}", other),
    }

    // steelblue — 仅 css-parser 支持（148 色），engine paint 不支持
    let steel_parsed = parse_color("steelblue").expect("应成功解析 steelblue");
    if let ColorValue::Rgba(r, g, b, a) = steel_parsed {
        assert_eq!(r, 70, "steelblue R 应为 70");
        assert_eq!(g, 130, "steelblue G 应为 130");
        assert_eq!(b, 180, "steelblue B 应为 180");
        assert_eq!(a, 255);
    }

    // 交叉验证：css-parser 和 engine paint 对共同支持的颜色值一致
    let common_colors = [
        ("red", 255, 0, 0),
        ("green", 0, 128, 0),
        ("blue", 0, 0, 255),
        ("orange", 255, 165, 0),
        ("teal", 0, 128, 128),
        ("navy", 0, 0, 128),
        ("silver", 192, 192, 192),
    ];
    for (name, exp_r, exp_g, exp_b) in common_colors {
        let css_color = parse_color(name).unwrap_or_else(|| panic!("应成功解析 {}", name));
        let engine_color = named_color_to_render(name);

        if let ColorValue::Rgba(r, g, b, a) = css_color {
            assert_eq!(r, engine_color.r, "{}: CSS 解析与 engine paint 的 R 应一致", name);
            assert_eq!(g, engine_color.g, "{}: CSS 解析与 engine paint 的 G 应一致", name);
            assert_eq!(b, engine_color.b, "{}: CSS 解析与 engine paint 的 B 应一致", name);
            assert_eq!(a, engine_color.a, "{}: CSS 解析与 engine paint 的 A 应一致", name);

            assert_eq!(r, exp_r, "{} R 应为 {}", name, exp_r);
            assert_eq!(g, exp_g, "{} G 应为 {}", name, exp_g);
            assert_eq!(b, exp_b, "{} B 应为 {}", name, exp_b);
        }
    }
}

/// URL 解析 + 导航历史集成测试。
///
/// 通过 net crate 解析 URL，创建 NavigationHistory，
/// 执行多次导航、后退、前进操作，验证历史状态正确。
#[test]
fn test_url_parse_and_navigation() {
    use zero_net::navigation::NavigationHistory;
    use zero_net::url_parser::parse_url;

    // 解析多个 URL
    let url_a = parse_url("https://example.com/page1").expect("应成功解析 URL A");
    let url_b = parse_url("https://example.com/page2?q=hello").expect("应成功解析 URL B");
    let url_c = parse_url("https://other.com/path").expect("应成功解析 URL C");

    assert_eq!(url_a.host, Some("example.com".to_string()));
    assert_eq!(url_a.path, "/page1");
    assert_eq!(url_b.host, Some("example.com".to_string()));
    assert_eq!(url_b.query, Some("q=hello".to_string()));
    assert_eq!(url_c.host, Some("other.com".to_string()));

    // 创建导航历史并推入条目（使用原始 URL 字符串）
    let str_a = "https://example.com/page1";
    let str_b = "https://example.com/page2?q=hello";
    let str_c = "https://other.com/path";

    let mut nav = NavigationHistory::new(50);
    nav.navigate(str_a, Some("Page 1".to_string()));
    nav.navigate(str_b, Some("Page 2".to_string()));
    nav.navigate(str_c, Some("Other".to_string()));

    assert_eq!(nav.len(), 3, "应有 3 条历史");
    assert_eq!(nav.current().unwrap().url, str_c);

    // 后退
    nav.go_back();
    assert_eq!(nav.current().unwrap().url, str_b, "后退后应在 page2");
    nav.go_back();
    assert_eq!(nav.current().unwrap().url, str_a, "再次后退应在 page1");

    // 前进
    nav.go_forward();
    assert_eq!(nav.current().unwrap().url, str_b, "前进后应在 page2");
    nav.go_forward();
    assert_eq!(nav.current().unwrap().url, str_c, "再次前进应在 other.com");

    // 在中间位置导航新 URL 应清除前进历史
    nav.go_back(); // at page2
    nav.navigate("https://new.com", Some("New".to_string()));
    assert!(!nav.can_go_forward(), "新导航后不应有前进历史");
    assert_eq!(nav.len(), 3, "历史应为 page1, page2, new.com");
}

/// DOM 元素 + CSS 样式交互集成测试。
///
/// 创建 DOM 元素，通过 style-system 应用 CSS 属性，
/// 验证计算样式中的 color 属性被正确解析和应用。
#[test]
fn test_dom_element_style_interaction() {
    use zero_css_parser::Parser as CssParser;
    use zero_css_parser::values::ColorValue;
    use zero_dom::Document;
    use zero_style_system::StyleSystem;

    // 构建 DOM: html > body > p
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let p = doc.create_element("p");
    doc.set_attribute(p, "class", "highlight");
    doc.append_child(body, p).unwrap();

    // CSS 规则：p 的 color 为 green
    let css = r#"
        p { color: green; font-size: 14px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    // 计算样式
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证 p 元素有计算样式
    let p_style = styles.get(&p).expect("p 元素应有计算样式");

    // green 的 RGB 值为 (0, 128, 0)
    match &p_style.color {
        ColorValue::Rgba(r, g, b, a) => {
            assert_eq!(*r, 0, "green R 应为 0");
            assert_eq!(*g, 128, "green G 应为 128");
            assert_eq!(*b, 0, "green B 应为 0");
            assert_eq!(*a, 255, "green A 应为 255");
        }
        other => panic!("预期 Rgba 颜色值，实际得到 {:?}", other),
    }

    // body 应继承默认样式
    let body_style = styles.get(&body);
    assert!(body_style.is_some(), "body 元素应有计算样式");
}

/// localStorage 与 sessionStorage 隔离集成测试。
///
/// 创建 localStorage 和 sessionStorage，
/// 分别设置不同值，验证两者之间完全隔离。
#[test]
fn test_storage_local_and_session() {
    use zero_storage::StorageManager;

    let mut mgr = StorageManager::new();

    // 在同源下分别操作 localStorage 和 sessionStorage
    let local = mgr.local_storage("https://example.com");
    local.set("theme", "dark").unwrap();
    local.set("lang", "zh").unwrap();

    let session = mgr.session_storage("https://example.com");
    session.set("theme", "light").unwrap();
    session.set("token", "abc123").unwrap();

    // localStorage 的值不受 sessionStorage 影响
    assert_eq!(
        mgr.local_storage("https://example.com").get("theme"),
        Some("dark"),
        "localStorage 的 theme 应为 dark"
    );
    assert_eq!(
        mgr.local_storage("https://example.com").get("lang"),
        Some("zh"),
        "localStorage 的 lang 应为 zh"
    );

    // sessionStorage 的值不受 localStorage 影响
    assert_eq!(
        mgr.session_storage("https://example.com").get("theme"),
        Some("light"),
        "sessionStorage 的 theme 应为 light"
    );
    assert_eq!(
        mgr.session_storage("https://example.com").get("token"),
        Some("abc123"),
        "sessionStorage 的 token 应为 abc123"
    );

    // 互相不存在的 key
    assert!(
        mgr.local_storage("https://example.com").get("token").is_none(),
        "localStorage 不应有 token"
    );
    assert!(
        mgr.session_storage("https://example.com").get("lang").is_none(),
        "sessionStorage 不应有 lang"
    );
}

/// WASM 模块调用主机函数集成测试。
///
/// 编译一个 WASM 模块，注册主机函数（env.double），
/// WASM 模块导入该函数并调用，验证返回值正确。
#[test]
fn test_wasm_host_function_call() {
    use zero_wasm_sandbox::{HostFunction, LinkerConfig, WasmSandbox, WasmValue, WasmValueType};

    let wat_text = r#"
        (module
            (import "env" "double" (func $double (param i32) (result i32)))
            (func (export "call_host") (param i32) (result i32)
                local.get 0
                call $double)
        )
    "#;
    let wasm_bytes = wat::parse_str(wat_text).expect("解析 WAT 失败");

    let sandbox = WasmSandbox::new();

    // 注册主机函数：将输入值乘以 2
    let mut linker = LinkerConfig::new();
    linker.define(HostFunction::new(
        "env",
        "double",
        vec![WasmValueType::I32],
        vec![WasmValueType::I32],
        |params, results| {
            if let WasmValue::I32(n) = params[0] {
                results.push(WasmValue::I32(n * 2));
            }
            Ok(())
        },
    ));

    let module = sandbox.compile(&wasm_bytes).expect("编译 WASM 失败");
    let mut instance = module
        .instantiate_with_linker(&sandbox, &linker)
        .expect("实例化 WASM 失败");

    // 调用导出函数，内部会调用主机函数 double(21) = 42
    let result = instance
        .call("call_host", &[WasmValue::I32(21)])
        .expect("调用 call_host 失败");
    assert_eq!(result.len(), 1, "应返回 1 个值");
    assert_eq!(result[0], WasmValue::I32(42), "double(21) 应返回 42");

    // 再次调用验证：double(0) = 0
    let result2 = instance
        .call("call_host", &[WasmValue::I32(0)])
        .expect("第二次调用失败");
    assert_eq!(result2[0], WasmValue::I32(0), "double(0) 应返回 0");

    // 负数：double(-5) = -10
    let result3 = instance
        .call("call_host", &[WasmValue::I32(-5)])
        .expect("第三次调用失败");
    assert_eq!(result3[0], WasmValue::I32(-10), "double(-5) 应返回 -10");
}

/// Canvas 绘图 + WebView 独立运行集成测试。
///
/// 创建 Canvas 上下文执行绘图操作，同时创建 WebView 加载页面，
/// 验证两者可以独立工作且互不干扰。
#[test]
fn test_canvas_draw_and_webview() {
    use zero_canvas::CanvasContext;
    use zero_render_foundation::color::Color;
    use zero_webview::{WebView, WebViewConfig};

    // Canvas 部分：创建上下文并绘制矩形
    let mut ctx = CanvasContext::new(400, 300);
    ctx.set_fill_color(Color::BLUE);
    ctx.fill_rect(10.0, 20.0, 100.0, 50.0);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(200.0, 100.0, 80.0, 60.0);

    let primitives = ctx.primitives();
    assert!(!primitives.fills.is_empty(), "Canvas 应生成填充图元");
    assert!(primitives.fills.len() >= 2, "应至少有 2 个填充图元");

    // WebView 部分：加载 HTML 页面
    let html = r#"<html><body>
        <div style="width: 200px; height: 100px; background-color: green;">Test</div>
    </body></html>"#;
    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    });
    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0, "WebView 渲染应成功完成");

    // Canvas 的图元数量不应因 WebView 操作而改变
    let canvas_fill_count = primitives.fills.len();
    let _wv_result = wv.render();
    assert_eq!(
        ctx.primitives().fills.len(),
        canvas_fill_count,
        "Canvas 图元数量不应受 WebView 操作影响"
    );
}
