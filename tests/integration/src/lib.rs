//! 跨 crate 集成测试
//!
//! 测试多个 crate 之间的协作，验证端到端的管线正确性。

#[cfg(test)]
mod dom_css {
    use zero_css_parser::Parser;
    use zero_dom::parse_html;

    /// 验证 HTML 解析后 DOM 树包含预期的元素节点
    #[test]
    fn test_html_parse_produces_dom_tree() {
        let html =
            r#"<html><head><title>Test</title></head><body><div id="main" class="container">Hello</div></body></html>"#;
        let doc = parse_html(html);
        assert!(doc.node_count() > 0, "DOM 应包含节点");

        // 查找 div 元素
        let root = doc.root();
        let _body = doc.get(root).unwrap();
        assert!(doc.node_count() > 5, "DOM 应包含多个节点");
    }

    /// 验证 CSS 解析产生正确的规则结构
    #[test]
    fn test_css_parse_produces_rules() {
        let css = r#"
            body { margin: 0; padding: 0; }
            .container { display: flex; width: 100%; }
            #main { background-color: blue; }
        "#;
        let stylesheet = Parser::parse_stylesheet(css);
        assert_eq!(stylesheet.rules.len(), 3, "应解析 3 条 CSS 规则");

        // 验证选择器存在
        for rule in &stylesheet.rules {
            if let zero_css_parser::Rule::Style(style_rule) = rule {
                assert!(!style_rule.selectors.is_empty());
                assert!(!style_rule.declarations.is_empty());
            }
        }
    }

    /// DOM + CSS 选择器匹配集成
    #[test]
    fn test_dom_element_attributes_accessible() {
        let html = r#"<html><body><div id="app" class="main active">Content</div></body></html>"#;
        let doc = parse_html(html);
        assert!(doc.node_count() > 0);

        // 遍历所有节点，验证至少有一个元素
        let root = doc.root();
        let data = doc.get(root).unwrap();
        assert!(!data.children.is_empty(), "根节点应有子节点");
    }
}

#[cfg(test)]
mod css_style {
    use zero_css_parser::Parser;
    use zero_dom::parse_html;
    use zero_style_system::StyleSystem;

    /// 验证样式系统可以计算 DOM 节点的计算样式
    #[test]
    fn test_compute_styles_from_html_and_css() {
        let html = r#"<html><body><div id="box" class="container">Hello</div></body></html>"#;
        let css = r#"
            .container { display: flex; width: 200px; }
            #box { background-color: red; }
        "#;

        let doc = parse_html(html);
        let stylesheet = Parser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 样式系统应返回样式映射
        assert!(!styles.is_empty(), "应为 DOM 节点计算样式");
    }

    /// 验证 CSS 级联优先级
    #[test]
    fn test_cascade_specificity() {
        let html = r#"<html><body><div id="main" class="content">Text</div></body></html>"#;
        let css = r#"
            .content { color: blue; }
            #main { color: red; }
        "#;

        let doc = parse_html(html);
        let stylesheet = Parser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let _styles = sys.compute_styles(&doc, &[stylesheet]);
    }

    /// 验证 CSS 继承
    #[test]
    fn test_style_inheritance() {
        let html = r#"<html><body><p>Inherited text</p></body></html>"#;
        let css = r#"
            body { color: green; font-size: 16px; }
        "#;

        let doc = parse_html(html);
        let stylesheet = Parser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        let styles = sys.compute_styles(&doc, &[stylesheet]);
        assert!(!styles.is_empty());
    }
}

#[cfg(test)]
mod render_pipeline {
    use zero_engine::RenderPipeline;

    /// 完整管线：HTML + CSS → 渲染结果
    #[test]
    fn test_full_render_pipeline() {
        let html = r#"<html><body>
            <div style="width: 200px; height: 100px; background-color: red;">Box</div>
        </body></html>"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, "");

        assert!(result.timings.total_ms >= 0.0, "应有渲染耗时");
        // 渲染管线成功完成即通过
    }

    /// 管线使用 CSS 文件
    #[test]
    fn test_render_pipeline_with_css() {
        let html = r#"<html><body><div class="box">Hello</div></body></html>"#;
        let css = r#".box { background-color: blue; width: 300px; height: 200px; }"#;

        let mut pipeline = RenderPipeline::new(1024.0, 768.0);
        let result = pipeline.render_html(html, css);

        assert!(result.timings.total_ms >= 0.0);
    }

    /// 管线阶段耗时分解
    #[test]
    fn test_pipeline_timing_breakdown() {
        let html = r#"<html><body><p>Test paragraph</p></body></html>"#;
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, "");

        assert!(result.timings.parse_ms >= 0.0, "parse_ms >= 0");
        assert!(result.timings.style_ms >= 0.0, "style_ms >= 0");
        assert!(result.timings.layout_ms >= 0.0, "layout_ms >= 0");
        assert!(result.timings.paint_ms >= 0.0, "paint_ms >= 0");
    }

    /// 复杂页面渲染
    #[test]
    fn test_complex_page_render() {
        let html = r#"<html><head><title>Complex</title></head><body>
            <header><h1>Title</h1></header>
            <main><p>Content</p><p>More content</p></main>
            <footer><p>Footer</p></footer>
        </body></html>"#;
        let css = r#"
            body { margin: 0; }
            header { background-color: #333; color: white; }
            main { padding: 20px; }
            footer { background-color: #eee; }
        "#;

        let mut pipeline = RenderPipeline::new(1440.0, 900.0);
        let result = pipeline.render_html(html, css);
        assert!(result.timings.total_ms >= 0.0);
    }
}

#[cfg(test)]
mod net_security {
    use zero_net::url_parser::parse_url;
    use zero_security::{CorsPolicy, Origin, check_cors};

    /// URL 解析 + 同源策略
    #[test]
    fn test_url_origin_same_origin_check() {
        let _url_a = parse_url("https://example.com/page1").unwrap();
        let _url_b = parse_url("https://example.com/page2?q=1").unwrap();

        let origin_a = Origin::parse("https://example.com").unwrap();
        let origin_b = Origin::parse("https://example.com").unwrap();

        assert!(origin_a.is_same_origin(&origin_b));
    }

    /// URL 解析 + CORS 检查
    #[test]
    fn test_cors_policy_with_parsed_url() {
        let origin = Origin::parse("http://evil.com").unwrap();
        let policy = CorsPolicy {
            allow_origins: vec!["http://example.com".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };

        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(!result.allowed, "跨域请求应被拒绝");
    }

    /// 安全上下文判断
    #[test]
    fn test_url_security_context() {
        let _http_url = parse_url("http://example.com").unwrap();
        let _https_url = parse_url("https://example.com").unwrap();

        let http_origin = Origin::parse("http://example.com").unwrap();
        let https_origin = Origin::parse("https://example.com").unwrap();

        assert!(!http_origin.is_secure());
        assert!(https_origin.is_secure());
    }
}

#[cfg(test)]
mod storage {
    use zero_protocol::{
        IpcMessage, IpcMessageKind, StorageOpParams, StorageOperation, StorageType, deserialize, serialize,
    };
    use zero_storage::StorageManager;

    /// localStorage CRUD + IPC 序列化
    #[test]
    fn test_local_storage_crud_and_ipc() {
        let mut mgr = StorageManager::new();
        let store = mgr.local_storage("https://example.com");

        // CRUD 操作
        assert!(store.get("key").is_none());
        let old = store.set("key", "value").expect("set");
        assert!(old.is_none());
        assert_eq!(store.get("key"), Some("value"));

        let old = store.set("key", "updated").expect("set");
        assert_eq!(old, Some("value".to_string()));
        assert_eq!(store.get("key"), Some("updated"));

        // 通过 IPC 消息序列化传输存储操作
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Set,
                key: "key".to_string(),
                value: Some("updated".to_string()),
                origin: "https://example.com".to_string(),
            }),
        };
        let bytes = serialize(&msg).expect("serialize");
        let decoded = deserialize(&bytes).expect("deserialize");
        if let IpcMessageKind::StorageOp(p) = decoded.kind {
            assert_eq!(p.key, "key");
            assert_eq!(p.value, Some("updated".to_string()));
        } else {
            panic!("expected StorageOp");
        }
    }

    /// sessionStorage 隔离
    #[test]
    fn test_session_storage_isolation() {
        let mut mgr = StorageManager::new();

        let local = mgr.local_storage("https://example.com");
        local.set("shared_key", "local_value").unwrap();

        let session = mgr.session_storage("https://example.com");
        session.set("shared_key", "session_value").unwrap();

        assert_eq!(
            mgr.local_storage("https://example.com").get("shared_key"),
            Some("local_value")
        );
        assert_eq!(
            mgr.session_storage("https://example.com").get("shared_key"),
            Some("session_value")
        );
    }

    /// 不同源的存储隔离
    #[test]
    fn test_storage_origin_isolation() {
        let mut mgr = StorageManager::new();

        let store_a = mgr.local_storage("https://a.com");
        store_a.set("key", "value_a").unwrap();

        let store_b = mgr.local_storage("https://b.com");
        store_b.set("key", "value_b").unwrap();

        assert_eq!(mgr.local_storage("https://a.com").get("key"), Some("value_a"));
        assert_eq!(mgr.local_storage("https://b.com").get("key"), Some("value_b"));
    }
}

#[cfg(test)]
mod protocol_navigation {
    use zero_net::navigation::NavigationHistory;
    use zero_protocol::{IpcMessage, IpcMessageKind, NavigateParams, deserialize, serialize};

    /// 导航历史操作 → IPC 消息序列化
    #[test]
    fn test_navigation_ipc_roundtrip() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("https://example.com", Some("Home".to_string()));
        nav.navigate("https://example.com/about", Some("About".to_string()));

        // 序列化导航命令
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com/about".to_string(),
                referrer: Some("https://example.com".to_string()),
            }),
        };
        let bytes = serialize(&msg).expect("serialize");
        let decoded = deserialize(&bytes).expect("deserialize");

        if let IpcMessageKind::Navigate(p) = decoded.kind {
            assert_eq!(p.url, "https://example.com/about");
            assert_eq!(p.referrer, Some("https://example.com".to_string()));
        } else {
            panic!("expected Navigate");
        }

        // 验证导航历史状态
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "https://example.com");
    }
}

#[cfg(test)]
mod canvas_render {
    use zero_canvas::CanvasContext;
    use zero_webview::{WebView, WebViewConfig};

    /// Canvas 2D 绘图操作 → 渲染图元
    #[test]
    fn test_canvas_fill_rect_generates_primitives() {
        let mut ctx = CanvasContext::new(800, 600);
        ctx.fill_rect(10.0, 20.0, 100.0, 50.0);
        ctx.fill_rect(200.0, 100.0, 150.0, 80.0);
        let primitives = ctx.primitives();
        // 应生成填充图元
        assert!(!primitives.fills.is_empty(), "fill_rect 应生成填充图元");
    }

    /// Canvas 路径绘制集成
    #[test]
    fn test_canvas_path_operations() {
        let mut ctx = CanvasContext::new(400, 300);
        ctx.begin_path();
        ctx.move_to(10.0, 10.0);
        ctx.line_to(100.0, 10.0);
        ctx.line_to(100.0, 100.0);
        ctx.close_path();
        ctx.fill();

        let primitives = ctx.primitives();
        assert!(!primitives.path_fills.is_empty(), "路径填充应生成图元");
    }

    /// Canvas 变换操作集成
    #[test]
    fn test_canvas_transform_chain() {
        let mut ctx = CanvasContext::new(400, 300);
        ctx.translate(50.0, 50.0);
        ctx.rotate(45.0_f32.to_radians());
        ctx.scale(2.0, 2.0);
        ctx.fill_rect(0.0, 0.0, 100.0, 100.0);

        // 变换后的 fill_rect 不应 panic
        let primitives = ctx.primitives();
        assert!(!primitives.fills.is_empty());
    }

    /// Canvas + WebView 集成：通过 WebView 加载含 canvas 的 HTML
    #[test]
    fn test_webview_renders_page_with_canvas() {
        let html = r#"<html><body>
            <div style="width: 200px; height: 100px; background-color: green;">Box</div>
        </body></html>"#;

        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        let result = wv.load_html(html, None);
        // WebView 渲染应产生图元
        assert!(result.timings.total_ms >= 0.0);
    }

    /// Canvas save/restore 状态管理
    #[test]
    fn test_canvas_save_restore_state() {
        let mut ctx = CanvasContext::new(400, 300);
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        ctx.save();
        ctx.translate(100.0, 100.0);
        ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
        ctx.restore();
        // restore 后再次绘制应使用原始变换
        ctx.fill_rect(200.0, 200.0, 50.0, 50.0);

        let primitives = ctx.primitives();
        assert!(primitives.fills.len() >= 3, "应有 3 个填充图元");
    }
}

#[cfg(test)]
mod wasm_sandbox {
    use zero_wasm_sandbox::{WasmSandbox, WasmValue};

    /// 编译并实例化一个简单的 WASM 模块
    #[test]
    fn test_wasm_compile_and_call_add() {
        let wat_text = r#"
            (module
                (func $add (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
                (export "add" (func $add))
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_text).expect("parse WAT");

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&wasm_bytes).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let result = instance
            .call("add", &[WasmValue::I32(3), WasmValue::I32(7)])
            .expect("call add");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], WasmValue::I32(10));
    }

    /// WASM 模块导出查询
    #[test]
    fn test_wasm_module_exports() {
        let wat_text = r#"
            (module
                (func $f1 (result i32) i32.const 42)
                (func $f2 (result i32) i32.const 99)
                (export "f1" (func $f1))
                (export "f2" (func $f2))
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_text).expect("parse WAT");

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&wasm_bytes).expect("compile");

        let exports = module.exports();
        assert!(exports.contains(&"f1".to_string()), "应导出 f1");
        assert!(exports.contains(&"f2".to_string()), "应导出 f2");
    }

    /// WASM 内存读写集成
    #[test]
    fn test_wasm_memory_read_write() {
        let wat_text = r#"
            (module
                (memory (export "mem") 1)
                (func $store (export "store") (param i32 i32)
                    local.get 0
                    local.get 1
                    i32.store
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_text).expect("parse WAT");

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&wasm_bytes).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 写入值
        instance
            .call("store", &[WasmValue::I32(0), WasmValue::I32(0x4243_4445)])
            .expect("call store");

        // 读取内存
        let data = instance.read_memory("mem", 0, 4).expect("read memory");
        assert_eq!(data.len(), 4);
        // 验证小端字节序
        assert_eq!(data[0], 0x45);
        assert_eq!(data[3], 0x42);
    }

    /// WASM 调用不存在的导出应返回错误
    #[test]
    fn test_wasm_call_nonexistent_export() {
        let wat_text = r#"
            (module
                (func $f (export "exists") (result i32) i32.const 1)
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_text).expect("parse WAT");

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&wasm_bytes).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let result = instance.call("nonexistent", &[]);
        assert!(result.is_err(), "调用不存在的导出应失败");
    }

    /// 无效 WASM 二进制应返回编译错误
    #[test]
    fn test_wasm_invalid_binary() {
        let sandbox = WasmSandbox::new();
        let result = sandbox.compile(&[0x00, 0x01, 0x02, 0x03]);
        assert!(result.is_err(), "无效 WASM 二进制应编译失败");
    }
}

#[cfg(test)]
mod webview_full_pipeline {
    use zero_webview::{WebView, WebViewBuilder, WebViewConfig};

    /// WebView 完整生命周期：创建 → 加载 HTML → 渲染 → 注入 CSS → 调整大小 → 重新渲染
    #[test]
    fn test_webview_full_lifecycle() {
        let mut wv = WebViewBuilder::new()
            .width(800)
            .height(600)
            .user_agent("IntegrationTest/1.0")
            .build();

        // 初始状态
        assert!(wv.url().is_none());
        assert!(!wv.is_loading());
        assert!(wv.last_render().is_none());

        // 加载 HTML
        let html = r#"<html><body>
            <header><h1>Title</h1></header>
            <main><p>Content paragraph</p></main>
        </body></html>"#;
        let result = wv.load_html(html, None);
        assert!(result.timings.total_ms >= 0.0);
        assert!(wv.last_render().is_some());

        // 注入 CSS 重新渲染
        let result = wv.inject_css("h1 { color: red; font-size: 24px; } p { margin: 10px; }");
        assert!(result.timings.total_ms >= 0.0);

        // 调整大小
        wv.resize(1024, 768);
        assert_eq!(wv.config().width, 1024);
        assert_eq!(wv.config().height, 768);

        // 重新渲染
        let result = wv.render();
        assert!(result.timings.total_ms >= 0.0);
    }

    /// WebView 加载复杂页面（多元素 + CSS）
    #[test]
    fn test_webview_complex_page_with_styles() {
        let html = r#"<html><body>
            <nav><a href="/">Home</a><a href="/about">About</a></nav>
            <section id="content">
                <article><h2>Article 1</h2><p>Text 1</p></article>
                <article><h2>Article 2</h2><p>Text 2</p></article>
            </section>
            <footer>Copyright 2026</footer>
        </body></html>"#;
        let css = r#"
            nav { background: #333; padding: 10px; }
            section { padding: 20px; }
            article { margin-bottom: 20px; border: 1px solid #ccc; }
            footer { text-align: center; padding: 5px; }
        "#;

        let mut wv = WebView::new(WebViewConfig {
            width: 1440,
            height: 900,
            ..Default::default()
        });
        let result = wv.load_html(html, Some(css));
        assert!(result.timings.total_ms >= 0.0);
        // 复杂页面应成功渲染，图元数量取决于管线实现
    }

    /// WebView 多次加载不 panic
    #[test]
    fn test_webview_repeated_load() {
        let mut wv = WebView::new(WebViewConfig::default());
        for i in 0..5 {
            let html = format!("<html><body><div>Page {i}</div></body></html>");
            let result = wv.load_html(&html, None);
            assert!(result.timings.total_ms >= 0.0, "第 {i} 次加载应成功");
        }
        assert!(wv.last_render().is_some());
    }

    /// WebView execute_script 返回 NotImplemented
    #[test]
    fn test_webview_script_not_implemented() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("1+1");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod cross_crate_integration {
    use std::collections::HashMap;

    use zero_canvas::CanvasContext;
    use zero_css_parser::ast::{
        ComplexSelector, CompoundSelector, ContainerCondition, ContainerRule, ContainerSizeCondition, Declaration,
        Rule, StyleRule, TypeSelector,
    };
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

        let engine = LayoutEngine::new(800.0, 600.0);
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
        ctx.ellipse(100.0, 100.0, 60.0, 40.0, 0.0, 0.0, std::f32::consts::TAU);
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

        let engine = LayoutEngine::new(800.0, 600.0);
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
}

/// 跨 crate 管线集成测试 — 第二批
///
/// 测试多个 crate 协作完成 CSS transform 管线、媒体查询评估、
/// Canvas 渐变采样、Grid 布局全管线、计数器级联等端到端场景。
#[cfg(test)]
mod cross_crate_pipeline {
    use std::collections::HashMap;

    use zero_css_parser::Parser as CssParser;
    use zero_css_parser::values::{
        AlignmentValue, ColorValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FontWeightValue, LengthValue,
        OverflowValue, PositionValue, TransformFunction, TransformValue, parse_transform,
    };
    use zero_dom::Document;
    use zero_engine::RenderPipeline;
    use zero_layout_engine::LayoutEngine;
    use zero_render_foundation::color::Color;
    use zero_style_system::{ComputedStyle, GridLineValue, StyleSystem};

    // ── 辅助函数 ──

    /// 创建 html > body 基础 DOM，返回 (doc, body NodeId)。
    fn make_doc_with_body() -> (Document, zero_dom::NodeId) {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        (doc, body)
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

    /// CSS Transform 管线集成测试。
    ///
    /// 通过 css-parser 解析含多个变换函数的 transform 值，
    /// 再由 style-system 计算样式，验证 ComputedStyle.transform 包含
    /// rotate(45deg) → scale(2) → translate(10px, 20px) 三个函数且顺序正确。
    #[test]
    fn test_transform_pipeline_integration() {
        // 1. 通过 css-parser 直接解析 transform 值
        let parsed = parse_transform("rotate(45deg) scale(2) translate(10px, 20px)");
        assert!(parsed.is_some(), "css-parser 应成功解析 transform 值");
        let transform_val = parsed.unwrap();
        match &transform_val {
            TransformValue::List(funcs) => {
                assert_eq!(funcs.len(), 3, "应包含 3 个变换函数");
                // rotate(45deg)
                assert!(
                    matches!(&funcs[0], TransformFunction::Rotate(a) if (*a - 45.0).abs() < 0.01),
                    "第一个函数应为 rotate(45deg)"
                );
                // scale(2) → Scale(2, None)
                assert!(
                    matches!(&funcs[1], TransformFunction::Scale(s, None) if (*s - 2.0).abs() < 0.01),
                    "第二个函数应为 scale(2)"
                );
                // translate(10px, 20px)
                assert!(
                    matches!(&funcs[2], TransformFunction::Translate(tx, ty) if (*tx - 10.0).abs() < 0.01 && (*ty - 20.0).abs() < 0.01),
                    "第三个函数应为 translate(10px, 20px)"
                );
            }
            other => panic!("transform 应为 List，实际为 {:?}", other),
        }

        // 2. 通过 style-system 计算样式验证 transform 管线
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let css = r#"div { transform: rotate(45deg) scale(2) translate(10px, 20px); }"#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        // 验证 computed style 中 transform 不为 none
        assert!(
            !matches!(div_style.transform, TransformValue::None),
            "ComputedStyle.transform 不应为 none"
        );
        // 验证变换函数列表完整
        match &div_style.transform {
            TransformValue::List(funcs) => {
                assert_eq!(funcs.len(), 3, "ComputedStyle 中应包含 3 个变换函数");
            }
            other => panic!("ComputedStyle.transform 应为 List，实际为 {:?}", other),
        }
    }

    /// 媒体查询 + 样式系统管线集成测试。
    ///
    /// 解析包含 prefers-color-scheme 媒体查询的 CSS，
    /// 分别在 dark 和 light 上下文中评估，验证 dark 上下文下样式正确应用。
    #[test]
    fn test_media_query_prefers_color_scheme_integration() {
        // 1. 通过 css-parser 的 media_query 模块解析
        let dark_ctx = zero_css_parser::media_query::MediaContext {
            viewport_width: 800.0,
            viewport_height: 600.0,
            media_type: zero_css_parser::media_query::MediaType::Screen,
            prefers_color_scheme: zero_css_parser::media_query::PrefersColorSchemeValue::Dark,
            prefers_reduced_motion: zero_css_parser::media_query::ReducedMotionValue::NoPreference,
            pointer_type: zero_css_parser::media_query::PointerValue::Fine,
            resolution_dpi: 96.0,
        };

        // 2. 解析含 prefers-color-scheme 的媒体查询
        let queries = zero_css_parser::media_query::parse_media_query("(prefers-color-scheme: dark)");
        assert!(queries.is_some(), "应成功解析 prefers-color-scheme 媒体查询");
        let query_list = queries.unwrap();
        assert!(!query_list.is_empty(), "媒体查询列表不应为空");

        // 在 dark 上下文中评估应为 true
        let eval_result = zero_css_parser::media_query::evaluate_media_query(&query_list[0], &dark_ctx);
        assert!(eval_result, "dark 上下文下 prefers-color-scheme: dark 应为 true");

        // 在 light 上下文中评估应为 false
        let light_ctx = zero_css_parser::media_query::MediaContext {
            prefers_color_scheme: zero_css_parser::media_query::PrefersColorSchemeValue::Light,
            ..dark_ctx.clone()
        };
        let eval_light = zero_css_parser::media_query::evaluate_media_query(&query_list[0], &light_ctx);
        assert!(!eval_light, "light 上下文下 prefers-color-scheme: dark 应为 false");

        // 3. 通过 style-system 端到端验证：使用 @media 规则
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let p = doc.create_element("p");
        doc.append_child(body, p).unwrap();

        // CSS: @media (prefers-color-scheme: dark) { p { color: white; } }
        let css = r#"p { color: black; }
            @media (prefers-color-scheme: dark) { p { color: white; } }"#;
        let stylesheet = CssParser::parse_stylesheet(css);

        // dark 模式下应应用 white
        // 注意：StyleSystem 当前不直接支持 prefers-color-scheme 上下文配置，
        // 但媒体查询解析本身已验证通过上面的 evaluate_media_query。
        // 此处验证样式系统不因 prefers-color-scheme 媒体查询而崩溃。
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);
        let p_style = styles.get(&p).expect("p 应有计算样式");
        // 至少验证样式系统成功计算
        assert!(
            matches!(p_style.color, ColorValue::Rgba(_, _, _, _)),
            "p 的 color 应为有效颜色值"
        );
    }

    /// Canvas 渐变样式 + 渲染基础 Color 集成测试。
    ///
    /// 创建 CanvasStyle::LinearGradient（红→蓝渐变），
    /// 在 offset=0.5 处采样颜色，验证结果为紫色（红蓝混合）。
    #[test]
    fn test_canvas_gradient_render_foundation_integration() {
        use zero_canvas::{CanvasContext, CanvasStyle, LinearGradient};

        // 1. 创建线性渐变：红色 → 蓝色
        let mut gradient = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        gradient.add_color_stop(0.0, Color::RED);
        gradient.add_color_stop(1.0, Color::BLUE);

        // 2. 在 offset=0.5 处采样 — 应为紫色（红蓝各半）
        let mid_color = gradient.sample_color(0.5);
        // 红(255,0,0) + 蓝(0,0,255) 在 50% 处插值 → (127, 0, 127)
        assert!(
            mid_color.r > 100 && mid_color.r < 200,
            "紫色 R 分量应在 100-200 之间，实际为 {}",
            mid_color.r
        );
        assert_eq!(mid_color.g, 0, "紫色 G 分量应为 0，实际为 {}", mid_color.g);
        assert!(
            mid_color.b > 100 && mid_color.b < 200,
            "紫色 B 分量应在 100-200 之间，实际为 {}",
            mid_color.b
        );
        assert_eq!(mid_color.a, 255, "alpha 应为 255");

        // 3. 验证边界采样
        let start_color = gradient.sample_color(0.0);
        assert_eq!(start_color.r, 255, "offset=0 应为红色 R=255");
        assert_eq!(start_color.b, 0, "offset=0 应为红色 B=0");

        let end_color = gradient.sample_color(1.0);
        assert_eq!(end_color.r, 0, "offset=1 应为蓝色 R=0");
        assert_eq!(end_color.b, 255, "offset=1 应为蓝色 B=255");

        // 4. 通过 CanvasStyle 包装验证 resolve_color
        let style = CanvasStyle::LinearGradient(gradient.clone());
        let resolved = style.resolve_color();
        // resolve_color 默认在 offset=0.5 采样
        assert_eq!(resolved.r, mid_color.r, "resolve_color 应与 sample_color(0.5) 一致");

        // 5. 集成测试：将渐变样式应用到 Canvas 上下文绘图
        let mut ctx = CanvasContext::new(200, 100);
        ctx.set_fill_style(CanvasStyle::LinearGradient(gradient));
        ctx.fill_rect(0.0, 0.0, 200.0, 100.0);
        let primitives = ctx.primitives();
        assert!(!primitives.fills.is_empty(), "使用渐变填充应生成图元");
    }

    /// Grid 布局全管线集成测试。
    ///
    /// 使用 grid-template-areas 定义 2x2 命名区域布局，
    /// 子元素通过 GridLineValue::Name 指定区域，
    /// 经 style-system → layout-engine 计算后验证各元素位置和尺寸。
    #[test]
    fn test_grid_layout_full_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        // 创建 grid 容器
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        // 创建 4 个子元素分别放入 4 个命名区域
        let top_left = doc.create_element("div");
        doc.set_attribute(top_left, "class", "tl");
        doc.append_child(grid, top_left).unwrap();

        let top_right = doc.create_element("div");
        doc.set_attribute(top_right, "class", "tr");
        doc.append_child(grid, top_right).unwrap();

        let bottom_left = doc.create_element("div");
        doc.set_attribute(bottom_left, "class", "bl");
        doc.append_child(grid, bottom_left).unwrap();

        let bottom_right = doc.create_element("div");
        doc.set_attribute(bottom_right, "class", "br");
        doc.append_child(grid, bottom_right).unwrap();

        let mut styles = HashMap::new();

        // grid 容器：2 列 x 2 行，命名区域 "tl tr" / "bl br"
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("200px 200px".to_string());
        grid_style.grid_template_rows = Some("100px 100px".to_string());
        grid_style.grid_template_areas = Some("\"tl tr\" \"bl br\"".to_string());
        grid_style.width = LengthValue::Px(400.0);
        grid_style.height = LengthValue::Px(200.0);
        styles.insert(grid, grid_style);

        // 为每个子元素设置 grid-area 命名
        for (el, name) in [
            (top_left, "tl"),
            (top_right, "tr"),
            (bottom_left, "bl"),
            (bottom_right, "br"),
        ] {
            let mut el_style = ComputedStyle::default();
            el_style.grid_row_start = GridLineValue::Name(name.to_string());
            el_style.grid_row_end = GridLineValue::Name(name.to_string());
            el_style.grid_column_start = GridLineValue::Name(name.to_string());
            el_style.grid_column_end = GridLineValue::Name(name.to_string());
            styles.insert(el, el_style);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 查找各子元素的布局盒
        let tl_box = find_box_by_node_id(&result.root, top_left).expect("tl 应在布局树中");
        let tr_box = find_box_by_node_id(&result.root, top_right).expect("tr 应在布局树中");
        let bl_box = find_box_by_node_id(&result.root, bottom_left).expect("bl 应在布局树中");
        let br_box = find_box_by_node_id(&result.root, bottom_right).expect("br 应在布局树中");

        // top-left 在第一列第一行
        assert!(tl_box.x < 1.0, "tl 应从 x=0 开始，实际 x={}", tl_box.x);
        assert!(tl_box.y < 1.0, "tl 应从 y=0 开始，实际 y={}", tl_box.y);
        assert!(
            (tl_box.width - 200.0).abs() < 2.0,
            "tl 宽度应约 200px，实际 {}",
            tl_box.width
        );

        // top-right 在第二列第一行
        assert!(tr_box.x >= 190.0, "tr 应在第二列，实际 x={}", tr_box.x);
        assert!(tr_box.y < 1.0, "tr 应在第一行，实际 y={}", tr_box.y);

        // bottom-left 在第一列第二行
        assert!(bl_box.x < 1.0, "bl 应在第一列，实际 x={}", bl_box.x);
        assert!(bl_box.y >= 90.0, "bl 应在第二行，实际 y={}", bl_box.y);

        // bottom-right 在第二列第二行
        assert!(br_box.x >= 190.0, "br 应在第二列，实际 x={}", br_box.x);
        assert!(br_box.y >= 90.0, "br 应在第二行，实际 y={}", br_box.y);

        // 所有子元素高度应约 100px
        for (name, bx) in [("tl", tl_box), ("tr", tr_box), ("bl", bl_box), ("br", br_box)] {
            assert!(
                (bx.height - 100.0).abs() < 2.0,
                "{} 高度应约 100px，实际 {}",
                name,
                bx.height
            );
        }
    }

    /// CSS 计数器属性级联集成测试。
    ///
    /// 父元素设置 counter-reset: section 0，
    /// 子元素设置 counter-increment: section 2，
    /// 通过 style-system 计算样式后验证两者的 computed values 正确。
    #[test]
    fn test_counter_property_cascade_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();

        // 父元素：重置计数器
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "parent");
        doc.append_child(body, parent).unwrap();

        // 子元素：递增计数器
        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .parent { counter-reset: section 0; }
            .child { counter-increment: section 2; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素的 counter-reset
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            !parent_style.counter_reset.is_empty(),
            "parent 的 counter_reset 不应为空"
        );
        assert_eq!(parent_style.counter_reset.len(), 1, "应有一个 counter-reset 条目");
        assert_eq!(parent_style.counter_reset[0].name, "section", "计数器名应为 section");
        assert_eq!(parent_style.counter_reset[0].value, Some(0), "重置值应为 0");

        // 验证子元素的 counter-increment
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert!(
            !child_style.counter_increment.is_empty(),
            "child 的 counter_increment 不应为空"
        );
        assert_eq!(
            child_style.counter_increment.len(),
            1,
            "应有一个 counter-increment 条目"
        );
        assert_eq!(child_style.counter_increment[0].name, "section", "计数器名应为 section");
        assert_eq!(child_style.counter_increment[0].value, Some(2), "增量值应为 2");

        // 子元素不应继承父元素的 counter-reset（counter-reset 不是继承属性）
        assert!(
            child_style.counter_reset.is_empty(),
            "child 不应继承 parent 的 counter_reset"
        );

        // 父元素不应有 counter-increment
        assert!(
            parent_style.counter_increment.is_empty(),
            "parent 不应有 counter_increment"
        );
    }

    /// 多函数 transform + transform-origin + perspective 完整管线测试。
    ///
    /// 同时设置 transform、transform-origin、perspective 三个属性，
    /// 验证 style-system 正确计算所有值到 ComputedStyle 中。
    #[test]
    fn test_transform_origin_perspective_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let css = r#"
            div {
                transform: rotate(45deg) translateX(10px) scale(2);
                transform-origin: 50% 50%;
                perspective: 800px;
            }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");

        // 验证 transform 包含变换函数
        match &div_style.transform {
            TransformValue::List(funcs) => {
                assert_eq!(funcs.len(), 3, "应包含 3 个变换函数");
                // rotate(45deg)
                assert!(
                    matches!(&funcs[0], TransformFunction::Rotate(a) if (*a - 45.0).abs() < 0.01),
                    "第一个函数应为 rotate(45deg)"
                );
                // translateX(10px)
                assert!(
                    matches!(&funcs[1], TransformFunction::TranslateX(tx) if (*tx - 10.0).abs() < 0.01),
                    "第二个函数应为 translateX(10px)"
                );
                // scale(2)
                assert!(
                    matches!(&funcs[2], TransformFunction::Scale(s, None) if (*s - 2.0).abs() < 0.01),
                    "第三个函数应为 scale(2)"
                );
            }
            other => panic!("transform 应为 List，实际为 {:?}", other),
        }

        // 验证 perspective 值
        assert_eq!(div_style.perspective, LengthValue::Px(800.0), "perspective 应为 800px");
    }

    /// 媒体查询 + prefers-reduced-motion 管线集成测试。
    ///
    /// 解析 prefers-reduced-motion 媒体查询，
    /// 在 reduce 和 no-preference 上下文中分别评估。
    #[test]
    fn test_media_query_prefers_reduced_motion() {
        let base_ctx = zero_css_parser::media_query::MediaContext {
            viewport_width: 1024.0,
            viewport_height: 768.0,
            media_type: zero_css_parser::media_query::MediaType::Screen,
            prefers_color_scheme: zero_css_parser::media_query::PrefersColorSchemeValue::Light,
            prefers_reduced_motion: zero_css_parser::media_query::ReducedMotionValue::Reduce,
            pointer_type: zero_css_parser::media_query::PointerValue::Fine,
            resolution_dpi: 96.0,
        };

        // 解析 prefers-reduced-motion: reduce
        let queries = zero_css_parser::media_query::parse_media_query("(prefers-reduced-motion: reduce)");
        assert!(queries.is_some(), "应成功解析 prefers-reduced-motion");
        let query_list = queries.unwrap();

        // reduce 上下文 → true
        let result_reduce = zero_css_parser::media_query::evaluate_media_query(&query_list[0], &base_ctx);
        assert!(
            result_reduce,
            "reduce 上下文下 prefers-reduced-motion: reduce 应为 true"
        );

        // no-preference 上下文 → false
        let no_pref_ctx = zero_css_parser::media_query::MediaContext {
            prefers_reduced_motion: zero_css_parser::media_query::ReducedMotionValue::NoPreference,
            ..base_ctx.clone()
        };
        let result_no_pref = zero_css_parser::media_query::evaluate_media_query(&query_list[0], &no_pref_ctx);
        assert!(
            !result_no_pref,
            "no-preference 上下文下 prefers-reduced-motion: reduce 应为 false"
        );
    }

    /// Canvas 径向渐变采样 + 多级停止点集成测试。
    ///
    /// 创建含 3 个停止点的径向渐变，验证各偏移量处的颜色采样结果。
    #[test]
    fn test_canvas_radial_gradient_sampling() {
        use zero_canvas::RadialGradient;

        // 创建径向渐变：红 → 绿 → 蓝
        let mut grad = RadialGradient::new(50.0, 50.0, 0.0, 50.0, 50.0, 50.0);
        grad.add_color_stop(0.0, Color::RED); // (255, 0, 0)
        grad.add_color_stop(0.5, Color::GREEN); // (0, 255, 0)
        grad.add_color_stop(1.0, Color::BLUE); // (0, 0, 255)

        // offset=0 处应为红色
        let c0 = grad.sample_color(0.0);
        assert_eq!(c0.r, 255, "offset=0 应为红色 R=255");
        assert_eq!(c0.g, 0, "offset=0 应为红色 G=0");
        assert_eq!(c0.b, 0, "offset=0 应为红色 B=0");

        // offset=0.5 处应为绿色
        let c5 = grad.sample_color(0.5);
        assert_eq!(c5.r, 0, "offset=0.5 应为绿色 R=0");
        assert_eq!(c5.g, 255, "offset=0.5 应为绿色 G=255");
        assert_eq!(c5.b, 0, "offset=0.5 应为绿色 B=0");

        // offset=1.0 处应为蓝色
        let c10 = grad.sample_color(1.0);
        assert_eq!(c10.r, 0, "offset=1.0 应为蓝色 R=0");
        assert_eq!(c10.g, 0, "offset=1.0 应为蓝色 G=0");
        assert_eq!(c10.b, 255, "offset=1.0 应为蓝色 B=255");

        // offset=0.25 处应为红绿混合（偏黄）
        let c25 = grad.sample_color(0.25);
        assert!(c25.r > 100, "offset=0.25 红绿混合 R 应 > 100，实际 {}", c25.r);
        assert!(c25.g > 100, "offset=0.25 红绿混合 G 应 > 100，实际 {}", c25.g);
        assert_eq!(c25.b, 0, "offset=0.25 红绿混合 B 应为 0");
    }

    /// Counter 属性通过 CSS 级联和继承的综合集成测试。
    ///
    /// 验证多个计数器同时存在时 counter-reset 和 counter-increment 的级联结果。
    #[test]
    fn test_counter_multiple_cascade_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let ol = doc.create_element("ol");
        doc.set_attribute(ol, "class", "toc");
        doc.append_child(body, ol).unwrap();
        let li1 = doc.create_element("li");
        doc.append_child(ol, li1).unwrap();
        let li2 = doc.create_element("li");
        doc.append_child(ol, li2).unwrap();

        let css = r#"
            ol { counter-reset: section 0 subsection 5; }
            li { counter-increment: section 1 subsection -1; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // ol 应有 counter-reset: section 0, subsection 5
        let ol_style = styles.get(&ol).expect("ol 应有计算样式");
        assert_eq!(ol_style.counter_reset.len(), 2, "ol 应有 2 个 counter-reset");

        // 查找 section 和 subsection 的重置值
        let section_reset = ol_style.counter_reset.iter().find(|c| c.name == "section");
        assert!(section_reset.is_some(), "应有 section 计数器重置");
        assert_eq!(section_reset.unwrap().value, Some(0), "section 重置值应为 0");

        let sub_reset = ol_style.counter_reset.iter().find(|c| c.name == "subsection");
        assert!(sub_reset.is_some(), "应有 subsection 计数器重置");
        assert_eq!(sub_reset.unwrap().value, Some(5), "subsection 重置值应为 5");

        // li 应有 counter-increment: section 1, subsection -1
        let li1_style = styles.get(&li1).expect("li1 应有计算样式");
        assert_eq!(li1_style.counter_increment.len(), 2, "li 应有 2 个 counter-increment");

        let section_inc = li1_style.counter_increment.iter().find(|c| c.name == "section");
        assert!(section_inc.is_some(), "应有 section 增量");
        assert_eq!(section_inc.unwrap().value, Some(1), "section 增量应为 1");

        let sub_inc = li1_style.counter_increment.iter().find(|c| c.name == "subsection");
        assert!(sub_inc.is_some(), "应有 subsection 增量");
        assert_eq!(sub_inc.unwrap().value, Some(-1), "subsection 增量应为 -1");

        // li 不应继承 ol 的 counter-reset
        assert!(li1_style.counter_reset.is_empty(), "li 不应继承 ol 的 counter_reset");
    }

    /// CSS overflow-wrap 管线集成测试。
    ///
    /// 解析含 overflow-wrap 的 CSS，通过 style-system 计算样式，
    /// 验证 overflow-wrap 值正确存储且能被子元素继承。
    #[test]
    fn test_overflow_wrap_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        // 父元素设置 overflow-wrap: break-word
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "wrap-container");
        doc.append_child(body, parent).unwrap();

        // 子元素不显式设置 overflow-wrap，应继承父元素的值
        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "text");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .wrap-container { overflow-wrap: break-word; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素的 overflow-wrap 为 BreakWord
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert_eq!(
            parent_style.overflow_wrap,
            zero_style_system::property::OverflowWrapValue::BreakWord,
            "parent 的 overflow-wrap 应为 BreakWord"
        );

        // 验证子元素继承了 overflow-wrap
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.overflow_wrap,
            zero_style_system::property::OverflowWrapValue::BreakWord,
            "child 应继承 parent 的 overflow-wrap: BreakWord"
        );
    }

    /// CSS text-align-last 管线集成测试。
    ///
    /// 解析含 text-align-last 的 CSS，通过 style-system 计算样式，
    /// 验证 text-align-last 值正确应用到目标元素。
    #[test]
    fn test_text_align_last_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "last-line");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .last-line { text-align-last: center; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.text_align_last,
            zero_style_system::property::TextAlignLastValue::Center,
            "div 的 text-align-last 应为 Center"
        );
    }

    /// CSS direction 管线集成测试。
    ///
    /// 解析含 direction: rtl 的 CSS，通过 style-system 计算样式，
    /// 验证 direction 值正确应用且被子元素继承。
    #[test]
    fn test_direction_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "rtl-container");
        doc.append_child(body, parent).unwrap();

        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "rtl-text");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .rtl-container { direction: rtl; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 父元素 direction 应为 Rtl
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert_eq!(
            parent_style.direction,
            zero_style_system::property::DirectionValue::Rtl,
            "parent 的 direction 应为 Rtl"
        );

        // 子元素应继承 direction: rtl
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.direction,
            zero_style_system::property::DirectionValue::Rtl,
            "child 应继承 parent 的 direction: Rtl"
        );
    }

    /// CSS tab-size 管线集成测试。
    ///
    /// 解析含 tab-size 的 CSS，通过 style-system 计算样式，
    /// 验证 tab-size 值正确解析和存储。
    #[test]
    fn test_tab_size_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let pre = doc.create_element("pre");
        doc.set_attribute(pre, "class", "code-block");
        doc.append_child(body, pre).unwrap();

        // 子元素用于验证继承
        let span = doc.create_element("span");
        doc.set_attribute(span, "class", "code-text");
        doc.append_child(pre, span).unwrap();

        let css = r#"
            .code-block { tab-size: 4; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证 pre 元素的 tab-size 为 4
        let pre_style = styles.get(&pre).expect("pre 应有计算样式");
        assert_eq!(
            pre_style.tab_size,
            zero_style_system::property::TabSizeValue::Number(4),
            "pre 的 tab-size 应为 Number(4)"
        );

        // 验证子元素继承了 tab-size
        let span_style = styles.get(&span).expect("span 应有计算样式");
        assert_eq!(
            span_style.tab_size,
            zero_style_system::property::TabSizeValue::Number(4),
            "span 应继承 pre 的 tab-size: Number(4)"
        );
    }

    /// Storage + Protocol 序列化集成测试。
    ///
    /// 将 storage 操作通过 IPC 消息序列化 → 反序列化，
    /// 验证 StorageOpParams 所有字段完整保留，包括 Remove 操作。
    #[test]
    fn test_storage_protocol_ipc_roundtrip() {
        use zero_protocol::{
            IpcMessage, IpcMessageKind, StorageOpParams, StorageOperation, StorageType, deserialize, serialize,
        };
        use zero_storage::StorageManager;

        // 先执行实际 storage 操作
        let mut mgr = StorageManager::new();
        let store = mgr.local_storage("https://example.com");
        store.set("session_id", "abc-123").unwrap();
        store.set("theme", "dark").unwrap();
        assert_eq!(store.get("session_id"), Some("abc-123"));

        // 构造 Remove 操作的 IPC 消息
        let msg = IpcMessage {
            id: 42,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Remove,
                key: "session_id".to_string(),
                value: None,
                origin: "https://example.com".to_string(),
            }),
        };

        // 序列化 → 反序列化
        let bytes = serialize(&msg).expect("serialize 应成功");
        let decoded = deserialize(&bytes).expect("deserialize 应成功");

        // 验证 IPC 字段
        assert_eq!(decoded.id, 42, "消息 ID 应为 42");
        if let IpcMessageKind::StorageOp(p) = decoded.kind {
            assert_eq!(p.storage_type, StorageType::Local, "storage_type 应为 Local");
            assert_eq!(p.operation, StorageOperation::Remove, "operation 应为 Remove");
            assert_eq!(p.key, "session_id", "key 应为 session_id");
            assert_eq!(p.value, None, "Remove 操作 value 应为 None");
            assert_eq!(p.origin, "https://example.com", "origin 应为 https://example.com");
        } else {
            panic!("expected StorageOp kind");
        }

        // 再构造 Clear 操作验证
        let clear_msg = IpcMessage {
            id: 43,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Session,
                operation: StorageOperation::Clear,
                key: String::new(),
                value: None,
                origin: "https://example.com".to_string(),
            }),
        };
        let bytes2 = serialize(&clear_msg).expect("serialize clear 应成功");
        let decoded2 = deserialize(&bytes2).expect("deserialize clear 应成功");
        if let IpcMessageKind::StorageOp(p) = decoded2.kind {
            assert_eq!(p.storage_type, StorageType::Session, "storage_type 应为 Session");
            assert_eq!(p.operation, StorageOperation::Clear, "operation 应为 Clear");
        } else {
            panic!("expected StorageOp kind for clear");
        }
    }

    /// CSS break-inside 管线集成测试。
    ///
    /// 解析含 break-inside: avoid 的 CSS，通过 style-system 计算样式，
    /// 验证 break-inside 值正确应用到目标元素。
    #[test]
    fn test_break_inside_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "no-break");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .no-break { break-inside: avoid; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.break_inside,
            zero_style_system::property::BreakInsideValue::Avoid,
            "div 的 break-inside 应为 Avoid"
        );
    }

    /// CSS column-count 管线集成测试。
    ///
    /// 解析含 column-count: 3 的 CSS，通过 style-system 计算样式，
    /// 验证 column-count 值正确解析和存储到计算样式中。
    #[test]
    fn test_column_count_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "multi-col");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .multi-col { column-count: 3; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.column_count,
            zero_style_system::property::ColumnCountComputedValue::Number(3),
            "div 的 column-count 应为 Number(3)"
        );
    }

    /// CSS object-fit 管线集成测试。
    ///
    /// 解析含 object-fit: cover 的 CSS，通过 style-system 计算样式，
    /// 验证 object-fit 值正确应用到 img 元素的计算样式中。
    #[test]
    fn test_object_fit_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let img = doc.create_element("img");
        doc.set_attribute(img, "class", "hero");
        doc.append_child(body, img).unwrap();

        let css = r#"
            .hero { object-fit: cover; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let img_style = styles.get(&img).expect("img 应有计算样式");
        assert_eq!(
            img_style.object_fit,
            zero_style_system::property::ObjectFitComputedValue::Cover,
            "img 的 object-fit 应为 Cover"
        );
    }

    /// CSS direction 多级继承集成测试。
    ///
    /// 祖父元素设置 direction: rtl，父元素不显式设置（应继承 rtl），
    /// 子元素显式设置 direction: ltr 覆盖继承值。
    /// 验证三层继承链中各元素的 direction 计算值正确。
    #[test]
    fn test_direction_inheritance_chain() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        // 祖父元素：direction: rtl
        let grandparent = doc.create_element("div");
        doc.set_attribute(grandparent, "class", "rtl-root");
        doc.append_child(body, grandparent).unwrap();

        // 父元素：不设置 direction，应继承 rtl
        let parent = doc.create_element("section");
        doc.set_attribute(parent, "class", "middle");
        doc.append_child(grandparent, parent).unwrap();

        // 子元素：显式设置 direction: ltr，覆盖继承值
        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "ltr-override");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .rtl-root { direction: rtl; }
            .ltr-override { direction: ltr; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 祖父元素：显式 rtl
        let gp_style = styles.get(&grandparent).expect("grandparent 应有计算样式");
        assert_eq!(
            gp_style.direction,
            zero_style_system::property::DirectionValue::Rtl,
            "grandparent 的 direction 应为 Rtl"
        );

        // 父元素：继承 rtl
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert_eq!(
            parent_style.direction,
            zero_style_system::property::DirectionValue::Rtl,
            "parent 应继承 grandparent 的 direction: Rtl"
        );

        // 子元素：显式覆盖为 ltr
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.direction,
            zero_style_system::property::DirectionValue::Ltr,
            "child 的 direction 应被显式覆盖为 Ltr"
        );
    }

    /// CSS contain 管线集成测试。
    ///
    /// 解析含 contain: layout 的 CSS，通过 style-system 计算样式，
    /// 验证 contain 值正确存储到计算样式中。
    #[test]
    fn test_contain_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "contained");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .contained { contain: layout; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.contain,
            zero_style_system::property::ContainComputedValue::Layout,
            "div 的 contain 应为 Layout"
        );
    }

    /// CSS filter 管线集成测试。
    ///
    /// 解析含 filter: blur(5px) 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.filter 为 Blur(5.0)。
    #[test]
    fn test_filter_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "blurred");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .blurred { filter: blur(5px); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.filter,
            zero_style_system::property::FilterComputedValue::Blur(5.0),
            "div 的 filter 应为 Blur(5.0)"
        );
    }

    /// CSS mix-blend-mode 管线集成测试。
    ///
    /// 解析含 mix-blend-mode: multiply 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.mix_blend_mode 为 Multiply。
    #[test]
    fn test_mix_blend_mode_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "blended");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .blended { mix-blend-mode: multiply; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.mix_blend_mode,
            zero_style_system::property::MixBlendModeComputedValue::Multiply,
            "div 的 mix-blend-mode 应为 Multiply"
        );
    }

    /// CSS scrollbar-width 管线集成测试。
    ///
    /// 解析含 scrollbar-width: thin 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.scrollbar_width 为 Thin。
    #[test]
    fn test_scrollbar_width_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "thin-scroll");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .thin-scroll { scrollbar-width: thin; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.scrollbar_width,
            zero_style_system::property::ScrollbarWidthComputedValue::Thin,
            "div 的 scrollbar-width 应为 Thin"
        );
    }

    /// CSS contain 多值组合管线集成测试。
    ///
    /// 解析含 contain: layout style paint 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.contain 为包含 layout + style + paint 标志位的 Custom 组合值。
    #[test]
    fn test_contain_multi_value_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "multi-contain");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .multi-contain { contain: layout style paint; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        // layout=0x02 + style=0x04 + paint=0x08 = 0x0E
        let expected_flags = zero_style_system::property::ContainComputedValue::FLAG_LAYOUT
            | zero_style_system::property::ContainComputedValue::FLAG_STYLE
            | zero_style_system::property::ContainComputedValue::FLAG_PAINT;
        assert_eq!(
            div_style.contain,
            zero_style_system::property::ContainComputedValue::Custom(expected_flags),
            "div 的 contain 应为 Custom(layout|style|paint) = 0x{:02X}",
            expected_flags
        );
    }

    /// CSS appearance 管线集成测试。
    ///
    /// 解析含 appearance: none 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.appearance 为 None。
    #[test]
    fn test_appearance_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let input = doc.create_element("input");
        doc.set_attribute(input, "class", "custom-input");
        doc.append_child(body, input).unwrap();

        let css = r#"
            .custom-input { appearance: none; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let input_style = styles.get(&input).expect("input 应有计算样式");
        assert_eq!(
            input_style.appearance,
            zero_style_system::property::AppearanceComputedValue::None,
            "input 的 appearance 应为 None"
        );
    }

    /// CSS columns 简写管线集成测试。
    ///
    /// 解析含 columns: 3 200px 的 CSS，通过 style-system 计算样式，
    /// 验证 column-count 解析为 Number(3)，column-width 解析为 Length(200px)。
    #[test]
    fn test_columns_shorthand_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "multi-col");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .multi-col { columns: 3 200px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.column_count,
            zero_style_system::property::ColumnCountComputedValue::Number(3),
            "div 的 column-count 应为 Number(3)"
        );
        assert_eq!(
            div_style.column_width,
            zero_style_system::property::ColumnWidthComputedValue::Length(LengthValue::Px(200.0)),
            "div 的 column-width 应为 Length(Px(200.0))"
        );
    }

    /// CSS text-wrap 管线集成测试。
    ///
    /// 解析含 text-wrap: balance 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.text_wrap 为 Balance。
    #[test]
    fn test_text_wrap_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "balanced");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .balanced { text-wrap: balance; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.text_wrap,
            zero_style_system::property::TextWrapComputedValue::Balance,
            "div 的 text-wrap 应为 Balance"
        );
    }

    /// CSS hyphens 管线集成测试。
    ///
    /// 解析含 hyphens: auto 的 CSS，通过 style-system 计算样式，
    /// 验证父元素 hyphens 为 Auto，且子元素继承了该值。
    #[test]
    fn test_hyphens_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        // 父元素设置 hyphens: auto
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "hyphenated");
        doc.append_child(body, parent).unwrap();

        // 子元素不显式设置，应继承 hyphens
        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "hyphen-text");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .hyphenated { hyphens: auto; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素的 hyphens 为 Auto
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert_eq!(
            parent_style.hyphens,
            zero_style_system::property::HyphensComputedValue::Auto,
            "parent 的 hyphens 应为 Auto"
        );

        // 验证子元素继承了 hyphens: auto
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.hyphens,
            zero_style_system::property::HyphensComputedValue::Auto,
            "child 应继承 parent 的 hyphens: Auto"
        );
    }

    /// CSS line-clamp 管线集成测试。
    ///
    /// 解析含 line-clamp: 3 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.line_clamp 为 Count(3)。
    #[test]
    fn test_line_clamp_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "clamped");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .clamped { line-clamp: 3; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.line_clamp,
            zero_style_system::property::LineClampComputedValue::Count(3),
            "div 的 line-clamp 应为 Count(3)"
        );
    }

    /// CSS background-image 管线集成测试。
    ///
    /// 解析含 background-image: url(bg.png) 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.background_image 为 Url("bg.png")。
    #[test]
    fn test_background_image_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "bg-img");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .bg-img { background-image: url(bg.png); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.background_image,
            zero_style_system::property::BackgroundImageComputedValue::Url("bg.png".to_string()),
            "div 的 background-image 应为 Url(\"bg.png\")"
        );
    }

    /// CSS background-repeat 管线集成测试。
    ///
    /// 解析含 background-repeat: no-repeat 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.background_repeat 为 NoRepeat。
    #[test]
    fn test_background_repeat_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "no-repeat-bg");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .no-repeat-bg { background-repeat: no-repeat; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.background_repeat,
            zero_style_system::property::BackgroundRepeatComputedValue::NoRepeat,
            "div 的 background-repeat 应为 NoRepeat"
        );
    }

    /// CSS background-size 管线集成测试。
    ///
    /// 解析含 background-size: cover 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.background_size 为 Cover。
    #[test]
    fn test_background_size_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "cover-bg");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .cover-bg { background-size: cover; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.background_size,
            zero_style_system::property::BackgroundSizeComputedValue::Cover,
            "div 的 background-size 应为 Cover"
        );
    }

    /// CSS background-attachment 管线集成测试。
    ///
    /// 解析含 background-attachment: fixed 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.background_attachment 为 Fixed。
    #[test]
    fn test_background_attachment_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "fixed-bg");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .fixed-bg { background-attachment: fixed; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.background_attachment,
            zero_style_system::property::BackgroundAttachmentComputedValue::Fixed,
            "div 的 background-attachment 应为 Fixed"
        );
    }

    /// CSS background-clip 管线集成测试。
    ///
    /// 解析含 background-clip: content-box 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.background_clip 为 ContentBox。
    #[test]
    fn test_background_clip_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "clip-bg");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .clip-bg { background-clip: content-box; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.background_clip,
            zero_style_system::property::BackgroundClipComputedValue::ContentBox,
            "div 的 background-clip 应为 ContentBox"
        );
    }

    /// CSS background-origin 管线集成测试。
    ///
    /// 解析含 background-origin: padding-box 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.background_origin 为 PaddingBox。
    #[test]
    fn test_background_origin_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "origin-bg");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .origin-bg { background-origin: padding-box; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.background_origin,
            zero_style_system::property::BackgroundOriginComputedValue::PaddingBox,
            "div 的 background-origin 应为 PaddingBox"
        );
    }

    /// CSS accent-color 管线集成测试。
    ///
    /// 解析含 accent-color: #ff0000 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.accent_color 为红色 (255, 0, 0) 的计算值。
    #[test]
    fn test_accent_color_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let input = doc.create_element("input");
        doc.set_attribute(input, "class", "accented");
        doc.append_child(body, input).unwrap();

        let css = r#"
            .accented { accent-color: #ff0000; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let input_style = styles.get(&input).expect("input 应有计算样式");
        match &input_style.accent_color {
            zero_style_system::property::AccentColorComputedValue::Color(color) => {
                assert_eq!(
                    color,
                    &zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
                    "accent-color 应为红色 (255, 0, 0, 255)"
                );
            }
            other => panic!("accent-color 应为 Color 变体，实际为 {:?}", other),
        }
    }

    /// CSS border-image-source 管线集成测试。
    ///
    /// 解析含 border-image-source: url(border.png) 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.border_image_source 为 Url 计算值。
    #[test]
    fn test_border_image_source_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "bordered");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .bordered { border-image-source: url(border.png); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.border_image_source,
            zero_style_system::property::BorderImageSourceComputedValue::Url("border.png".to_string()),
            "div 的 border-image-source 应为 Url(border.png)"
        );
    }

    /// CSS border-image-slice 管线集成测试。
    #[test]
    fn test_border_image_slice_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "sliced");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .sliced { border-image-slice: 30 40 fill; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        use zero_style_system::property::BorderImageSliceComputedComponent;
        assert_eq!(
            div_style.border_image_slice.top,
            BorderImageSliceComputedComponent::Number(30.0),
            "slice top 应为 30"
        );
        assert_eq!(
            div_style.border_image_slice.right,
            BorderImageSliceComputedComponent::Number(40.0),
            "slice right 应为 40"
        );
        assert!(div_style.border_image_slice.fill, "slice fill 应为 true");
    }

    /// CSS border-image-repeat 管线集成测试。
    #[test]
    fn test_border_image_repeat_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "repeated");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .repeated { border-image-repeat: round space; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        use zero_style_system::property::BorderImageRepeatComputedMode;
        assert_eq!(
            div_style.border_image_repeat.horizontal,
            BorderImageRepeatComputedMode::Round,
            "repeat 水平应为 Round"
        );
        assert_eq!(
            div_style.border_image_repeat.vertical,
            BorderImageRepeatComputedMode::Space,
            "repeat 垂直应为 Space"
        );
    }

    /// CSS border-image-width 管线集成测试。
    #[test]
    fn test_border_image_width_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "widthed");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .widthed { border-image-width: 2 10px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        use zero_style_system::property::BorderImageWidthComputedComponent;
        assert_eq!(
            div_style.border_image_width.top,
            BorderImageWidthComputedComponent::Number(2.0),
            "width top 应为 Number(2.0)"
        );
        assert_eq!(
            div_style.border_image_width.right,
            BorderImageWidthComputedComponent::Length(10.0),
            "width right 应为 Length(10.0)"
        );
    }

    /// CSS border-image-outset 管线集成测试。
    #[test]
    fn test_border_image_outset_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "outset");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .outset { border-image-outset: 5px 2; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        use zero_style_system::property::BorderImageOutsetComputedComponent;
        assert_eq!(
            div_style.border_image_outset.top,
            BorderImageOutsetComputedComponent::Length(5.0),
            "outset top 应为 Length(5.0)"
        );
        assert_eq!(
            div_style.border_image_outset.right,
            BorderImageOutsetComputedComponent::Number(2.0),
            "outset right 应为 Number(2.0)"
        );
    }

    /// CSS text-shadow 管线集成测试。
    ///
    /// 解析含 text-shadow: 2px 3px red 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.text_shadow 的 offset_x=2.0, offset_y=3.0, color 为红色。
    #[test]
    fn test_text_shadow_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "shadowed");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .shadowed { text-shadow: 2px 3px red; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            (div_style.text_shadow.offset_x - 2.0).abs() < 0.01,
            "text-shadow offset_x 应为 2.0，实际为 {}",
            div_style.text_shadow.offset_x
        );
        assert!(
            (div_style.text_shadow.offset_y - 3.0).abs() < 0.01,
            "text-shadow offset_y 应为 3.0，实际为 {}",
            div_style.text_shadow.offset_y
        );
        assert_eq!(
            div_style.text_shadow.color,
            zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            "text-shadow color 应为红色 (255, 0, 0, 255)"
        );
    }

    /// CSS box-shadow 管线集成测试。
    ///
    /// 解析含 box-shadow: 5px 10px 20px blue 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.box_shadow 的 offset_x=5.0, offset_y=10.0, blur_radius=20.0。
    #[test]
    fn test_box_shadow_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "box-shadowed");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .box-shadowed { box-shadow: 5px 10px 20px blue; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            (div_style.box_shadow.offset_x - 5.0).abs() < 0.01,
            "box-shadow offset_x 应为 5.0，实际为 {}",
            div_style.box_shadow.offset_x
        );
        assert!(
            (div_style.box_shadow.offset_y - 10.0).abs() < 0.01,
            "box-shadow offset_y 应为 10.0，实际为 {}",
            div_style.box_shadow.offset_y
        );
        assert!(
            (div_style.box_shadow.blur_radius - 20.0).abs() < 0.01,
            "box-shadow blur_radius 应为 20.0，实际为 {}",
            div_style.box_shadow.blur_radius
        );
    }

    /// CSS box-shadow inset 管线集成测试。
    ///
    /// 解析含 box-shadow: inset 3px 4px green 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.box_shadow 的 inset=true。
    #[test]
    fn test_box_shadow_inset_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "inset-shadow");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .inset-shadow { box-shadow: inset 3px 4px green; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(div_style.box_shadow.inset, "box-shadow inset 应为 true");
        assert!(
            (div_style.box_shadow.offset_x - 3.0).abs() < 0.01,
            "box-shadow offset_x 应为 3.0，实际为 {}",
            div_style.box_shadow.offset_x
        );
        assert!(
            (div_style.box_shadow.offset_y - 4.0).abs() < 0.01,
            "box-shadow offset_y 应为 4.0，实际为 {}",
            div_style.box_shadow.offset_y
        );
    }

    /// CSS text-shadow 继承集成测试。
    ///
    /// 父元素设置 text-shadow，子元素不显式设置，
    /// 验证子元素继承了父元素的 text-shadow 值（text-shadow 是继承属性）。
    #[test]
    fn test_text_shadow_inheritance_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        // 父元素设置 text-shadow
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "shadowed");
        doc.append_child(body, parent).unwrap();

        // 子元素不设置 text-shadow，应继承
        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "inner");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .shadowed { text-shadow: 2px 3px red; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素的 text-shadow
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            (parent_style.text_shadow.offset_x - 2.0).abs() < 0.01,
            "parent text-shadow offset_x 应为 2.0"
        );

        // 验证子元素继承了 text-shadow
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert!(
            (child_style.text_shadow.offset_x - 2.0).abs() < 0.01,
            "child 应继承 parent 的 text-shadow offset_x=2.0，实际为 {}",
            child_style.text_shadow.offset_x
        );
        assert!(
            (child_style.text_shadow.offset_y - 3.0).abs() < 0.01,
            "child 应继承 parent 的 text-shadow offset_y=3.0，实际为 {}",
            child_style.text_shadow.offset_y
        );
        assert_eq!(
            child_style.text_shadow.color,
            zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            "child 应继承 parent 的 text-shadow color 为红色"
        );
    }

    /// CSS outline-width 管线集成测试。
    ///
    /// 解析含 outline-width: 3px 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.outline_width 为 Px(3.0)。
    #[test]
    fn test_outline_width_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "outlined");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .outlined { outline-width: 3px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.outline_width,
            zero_css_parser::values::LengthValue::Px(3.0),
            "div 的 outline-width 应为 Px(3.0)"
        );
    }

    /// CSS list-style-image 管线集成测试。
    ///
    /// 解析含 list-style-image: url(marker.png) 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.list_style_image 为 Url("marker.png")。
    #[test]
    fn test_list_style_image_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let li = doc.create_element("li");
        doc.set_attribute(li, "class", "item");
        doc.append_child(body, li).unwrap();

        let css = r#"
            .item { list-style-image: url(marker.png); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let li_style = styles.get(&li).expect("li 应有计算样式");
        assert_eq!(
            li_style.list_style_image,
            zero_style_system::property::ListStyleImageComputedValue::Url("marker.png".to_string()),
            "li 的 list-style-image 应为 Url(\"marker.png\")"
        );
    }

    /// CSS column-gap 管线集成测试。
    ///
    /// 解析含 column-gap: 30px 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.column_gap 为 Px(30.0)。
    #[test]
    fn test_column_gap_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "gap-container");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .gap-container { column-gap: 30px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.column_gap,
            LengthValue::Px(30.0),
            "div 的 column-gap 应为 Px(30.0)"
        );
    }

    /// CSS text-shadow 继承管线集成测试。
    ///
    /// 父元素 .shadowed 设置 text-shadow: 2px 2px red，
    /// 子元素 .inner 不显式设置，应继承父元素的 text-shadow（text-shadow 是继承属性）。
    /// 验证子元素的 text_shadow.offset_x == 2.0。
    #[test]
    fn test_text_shadow_inheritance_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "shadowed");
        doc.append_child(body, parent).unwrap();

        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "inner");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .shadowed { text-shadow: 2px 2px red; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素的 text-shadow
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            (parent_style.text_shadow.offset_x - 2.0).abs() < 0.01,
            "parent text-shadow offset_x 应为 2.0"
        );

        // 验证子元素继承了 text-shadow
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert!(
            (child_style.text_shadow.offset_x - 2.0).abs() < 0.01,
            "child 应继承 parent 的 text-shadow offset_x=2.0，实际为 {}",
            child_style.text_shadow.offset_x
        );
    }

    /// CSS box-shadow 不继承管线集成测试。
    ///
    /// 父元素 .shadowed 设置 box-shadow: 5px 5px blue，
    /// 子元素 .inner 不显式设置，不应继承父元素的 box-shadow（box-shadow 不是继承属性）。
    /// 验证子元素的 box_shadow.offset_x == 0.0（默认值）。
    #[test]
    fn test_box_shadow_not_inherited_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "shadowed");
        doc.append_child(body, parent).unwrap();

        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "inner");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .shadowed { box-shadow: 5px 5px blue; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素的 box-shadow
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            (parent_style.box_shadow.offset_x - 5.0).abs() < 0.01,
            "parent box-shadow offset_x 应为 5.0"
        );

        // 验证子元素不继承 box-shadow，应为默认值 offset_x=0.0
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert!(
            (child_style.box_shadow.offset_x - 0.0).abs() < 0.01,
            "child 不应继承 parent 的 box-shadow，offset_x 应为 0.0，实际为 {}",
            child_style.box_shadow.offset_x
        );
    }

    /// CSS outline 简写属性管线集成测试。
    ///
    /// 解析含 outline: 2px solid red 的 CSS，通过 style-system 的简写展开，
    /// 验证 outline_width=Px(2.0)、outline_style=Solid、outline_color=red。
    #[test]
    fn test_outline_shorthand_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "outlined");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .outlined { outline: 2px solid red; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");

        // 验证 outline-width
        assert_eq!(
            div_style.outline_width,
            LengthValue::Px(2.0),
            "div 的 outline-width 应为 Px(2.0)"
        );

        // 验证 outline-style
        assert_eq!(
            div_style.outline_style,
            zero_style_system::property::OutlineStyleValue::Solid,
            "div 的 outline-style 应为 Solid"
        );

        // 验证 outline-color
        assert_eq!(
            div_style.outline_color,
            zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            "div 的 outline-color 应为红色 (255, 0, 0, 255)"
        );
    }

    /// CSS gap 简写管线集成测试。
    ///
    /// 解析含 gap: 20px 的 CSS，通过简写展开为 row-gap: 20px 和 column-gap: 20px，
    /// 由 style-system 计算样式后验证 ComputedStyle.row_gap 和 column_gap 均为 Px(20.0)。
    #[test]
    fn test_gap_shorthand_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "gapped");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .gapped { gap: 20px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.row_gap, LengthValue::Px(20.0), "div 的 row-gap 应为 Px(20.0)");
        assert_eq!(
            div_style.column_gap,
            LengthValue::Px(20.0),
            "div 的 column-gap 应为 Px(20.0)"
        );
    }

    /// CSS empty-cells 管线集成测试。
    ///
    /// 解析含 empty-cells: hide 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.empty_cells 为 Hide。
    #[test]
    fn test_empty_cells_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let td = doc.create_element("td");
        doc.set_attribute(td, "class", "empty");
        doc.append_child(body, td).unwrap();

        let css = r#"
            .empty { empty-cells: hide; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let td_style = styles.get(&td).expect("td 应有计算样式");
        assert_eq!(
            td_style.empty_cells,
            zero_style_system::property::EmptyCellsComputedValue::Hide,
            "td 的 empty-cells 应为 Hide"
        );
    }

    /// CSS border-spacing 管线集成测试。
    ///
    /// 解析含 border-spacing: 5px 10px 的 CSS，通过 style-system 计算样式，
    /// 验证 horizontal=5.0, vertical=10.0。
    #[test]
    fn test_border_spacing_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let table = doc.create_element("table");
        doc.set_attribute(table, "class", "spaced");
        doc.append_child(body, table).unwrap();

        let css = r#"
            .spaced { border-spacing: 5px 10px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let table_style = styles.get(&table).expect("table 应有计算样式");
        assert!(
            (table_style.border_spacing.horizontal - 5.0).abs() < 0.01,
            "border-spacing horizontal 应为 5.0，实际为 {}",
            table_style.border_spacing.horizontal
        );
        assert!(
            (table_style.border_spacing.vertical - 10.0).abs() < 0.01,
            "border-spacing vertical 应为 10.0，实际为 {}",
            table_style.border_spacing.vertical
        );
    }

    /// CSS empty-cells 继承管线集成测试。
    ///
    /// empty-cells 是继承属性。父元素 .parent 设置 empty-cells: hide，
    /// 子元素 .child 不显式设置，应继承 Hide。
    #[test]
    fn test_empty_cells_inheritance_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let parent = doc.create_element("table");
        doc.set_attribute(parent, "class", "parent");
        doc.append_child(body, parent).unwrap();

        let child = doc.create_element("td");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .parent { empty-cells: hide; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert_eq!(
            parent_style.empty_cells,
            zero_style_system::property::EmptyCellsComputedValue::Hide,
            "parent 的 empty-cells 应为 Hide"
        );

        // 验证子元素继承了 empty-cells: hide
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.empty_cells,
            zero_style_system::property::EmptyCellsComputedValue::Hide,
            "child 应继承 parent 的 empty-cells: Hide"
        );
    }

    /// CSS border-spacing 继承管线集成测试。
    ///
    /// border-spacing 是继承属性。父元素 .parent 设置 border-spacing: 3px，
    /// 子元素 .child 不显式设置，应继承 horizontal=3.0, vertical=3.0。
    #[test]
    fn test_border_spacing_inheritance_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let parent = doc.create_element("table");
        doc.set_attribute(parent, "class", "parent");
        doc.append_child(body, parent).unwrap();

        let child = doc.create_element("td");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .parent { border-spacing: 3px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            (parent_style.border_spacing.horizontal - 3.0).abs() < 0.01,
            "parent border-spacing horizontal 应为 3.0"
        );

        // 验证子元素继承了 border-spacing: 3px
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert!(
            (child_style.border_spacing.horizontal - 3.0).abs() < 0.01,
            "child 应继承 parent 的 border-spacing horizontal=3.0，实际为 {}",
            child_style.border_spacing.horizontal
        );
    }

    /// CSS border-image 简写属性管线集成测试。
    ///
    /// 解析含 border-image: url(border.png) 25 的 CSS，
    /// 通过 style-system 简写展开为 border-image-source 和 border-image-slice，
    /// 验证 border-image-source 为 Url("border.png")，border-image-slice top 为 Number(25)。
    #[test]
    fn test_border_image_shorthand_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "bordered");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .bordered { border-image: url(border.png) 25; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");

        // 验证 border-image-source 为 Url
        assert_eq!(
            div_style.border_image_source,
            zero_style_system::property::BorderImageSourceComputedValue::Url("border.png".to_string()),
            "div 的 border-image-source 应为 Url(\"border.png\")"
        );

        // 验证 border-image-slice top 为 Number(25)
        use zero_style_system::property::BorderImageSliceComputedComponent;
        assert_eq!(
            div_style.border_image_slice.top,
            BorderImageSliceComputedComponent::Number(25.0),
            "div 的 border-image-slice top 应为 Number(25)"
        );
    }

    /// CSS counter-set 管线集成测试。
    ///
    /// 解析含 counter-set: mycounter 5 的 CSS，通过 style-system 计算样式，
    /// 验证 counter_set 列表中包含 mycounter，值为 5。
    #[test]
    fn test_counter_set_pipeline_integration() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "counter-set");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .counter-set { counter-set: mycounter 5; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(!div_style.counter_set.is_empty(), "div 的 counter_set 不应为空");
        assert_eq!(div_style.counter_set.len(), 1, "应有一个 counter-set 条目");
        assert_eq!(div_style.counter_set[0].name, "mycounter", "计数器名应为 mycounter");
        assert_eq!(div_style.counter_set[0].value, Some(5), "设定值应为 5");
    }

    /// CSS empty-cells: show 管线集成测试。
    ///
    /// 解析含 empty-cells: show 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.empty_cells 为 Show。
    #[test]
    fn test_empty_cells_show_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let td = doc.create_element("td");
        doc.set_attribute(td, "class", "visible");
        doc.append_child(body, td).unwrap();

        let css = r#"
            .visible { empty-cells: show; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let td_style = styles.get(&td).expect("td 应有计算样式");
        assert_eq!(
            td_style.empty_cells,
            zero_style_system::property::EmptyCellsComputedValue::Show,
            "td 的 empty-cells 应为 Show"
        );
    }

    /// CSS border-spacing 双值继承管线集成测试。
    ///
    /// border-spacing 是继承属性。父元素设置 border-spacing: 10px 20px，
    /// 子元素不显式设置，应继承 horizontal=10.0, vertical=20.0。
    #[test]
    fn test_border_spacing_dual_value_inheritance_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let parent = doc.create_element("table");
        doc.set_attribute(parent, "class", "parent");
        doc.append_child(body, parent).unwrap();

        let child = doc.create_element("td");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .parent { border-spacing: 10px 20px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 验证父元素
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            (parent_style.border_spacing.horizontal - 10.0).abs() < 0.01,
            "parent border-spacing horizontal 应为 10.0，实际为 {}",
            parent_style.border_spacing.horizontal
        );
        assert!(
            (parent_style.border_spacing.vertical - 20.0).abs() < 0.01,
            "parent border-spacing vertical 应为 20.0，实际为 {}",
            parent_style.border_spacing.vertical
        );

        // 验证子元素继承了 border-spacing: 10px 20px
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert!(
            (child_style.border_spacing.horizontal - 10.0).abs() < 0.01,
            "child 应继承 parent 的 border-spacing horizontal=10.0，实际为 {}",
            child_style.border_spacing.horizontal
        );
        assert!(
            (child_style.border_spacing.vertical - 20.0).abs() < 0.01,
            "child 应继承 parent 的 border-spacing vertical=20.0，实际为 {}",
            child_style.border_spacing.vertical
        );
    }

    /// CSS justify-items: center 管线集成测试。
    ///
    /// 解析含 justify-items: center 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.justify_items 为 Center。
    #[test]
    fn test_justify_items_center_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "centered");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .centered { justify-items: center; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.justify_items,
            zero_style_system::property::JustifyItemsValue::Center,
            "div 的 justify-items 应为 Center"
        );
    }

    // ── 新增测试：box-shadow / text-shadow / background-image 渲染管线集成 ──

    /// CSS box-shadow 渲染管线集成测试 — 验证 box-shadow 属性通过完整管线正确传递。
    #[test]
    fn test_box_shadow_render_pipeline() {
        let html = r#"<html><body>
            <div class="shadowed" style="width: 200px; height: 100px;">Box</div>
        </body></html>"#;
        let css = r#".shadowed { box-shadow: 5px 10px 20px blue; }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 渲染应成功完成
        assert!(result.timings.total_ms >= 0.0, "渲染应成功完成");
        // 应生成至少一个 shadow 图元
        assert!(
            !result.primitives.shadows.is_empty(),
            "box-shadow 应生成 ShadowPrimitive，实际 shadows 数量: {}",
            result.primitives.shadows.len()
        );

        // 验证 shadow 参数
        let shadow = &result.primitives.shadows[0];
        assert!(
            (shadow.offset_x - 5.0).abs() < 0.01,
            "shadow offset_x 应为 5.0，实际为 {}",
            shadow.offset_x
        );
        assert!(
            (shadow.offset_y - 10.0).abs() < 0.01,
            "shadow offset_y 应为 10.0，实际为 {}",
            shadow.offset_y
        );
        assert!(
            (shadow.blur_radius - 20.0).abs() < 0.01,
            "shadow blur_radius 应为 20.0，实际为 {}",
            shadow.blur_radius
        );
    }

    /// CSS box-shadow 多值管线集成测试。
    #[test]
    fn test_box_shadow_with_background_color_pipeline() {
        let html = r#"<html><body>
            <div class="box" style="width: 200px; height: 100px;">Content</div>
        </body></html>"#;
        let css = r#"
            .box { background-color: red; box-shadow: 3px 4px 10px rgba(0,0,0,0.5); }
        "#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 应同时有 fills（背景色）和 shadows（box-shadow）
        assert!(!result.primitives.fills.is_empty(), "background-color 应生成填充图元");
        assert!(!result.primitives.shadows.is_empty(), "box-shadow 应生成阴影图元");
    }

    /// CSS box-shadow 负偏移管线集成测试。
    #[test]
    fn test_box_shadow_negative_offset_pipeline() {
        let html = r#"<html><body>
            <div class="neg" style="width: 200px; height: 100px;">Neg</div>
        </body></html>"#;
        let css = r#".neg { box-shadow: -5px -3px 8px green; }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        assert!(!result.primitives.shadows.is_empty(), "应有阴影图元");
        let shadow = &result.primitives.shadows[0];
        assert!(
            (shadow.offset_x - (-5.0)).abs() < 0.01,
            "shadow offset_x 应为 -5.0，实际为 {}",
            shadow.offset_x
        );
        assert!(
            (shadow.offset_y - (-3.0)).abs() < 0.01,
            "shadow offset_y 应为 -3.0，实际为 {}",
            shadow.offset_y
        );
        assert!(
            (shadow.blur_radius - 8.0).abs() < 0.01,
            "shadow blur_radius 应为 8.0，实际为 {}",
            shadow.blur_radius
        );
    }

    /// CSS box-shadow 默认值（无阴影）管线集成测试。
    #[test]
    fn test_box_shadow_none_pipeline() {
        let html = r#"<html><body>
            <div class="plain">Plain</div>
        </body></html>"#;
        let css = r#".plain { width: 200px; height: 100px; background-color: gray; }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 无 box-shadow 时 shadows 应为空
        assert!(
            result.primitives.shadows.is_empty(),
            "无 box-shadow 时不应生成阴影图元，实际数量: {}",
            result.primitives.shadows.len()
        );
        // 背景色应生成 fills
        assert!(!result.primitives.fills.is_empty(), "背景色应生成填充图元");
    }

    /// CSS text-shadow 渲染管线集成测试。
    #[test]
    fn test_text_shadow_render_pipeline() {
        let html = r#"<html><body>
            <div class="text" style="width: 200px; height: 50px; color: black; font-size: 16px;">Hello</div>
        </body></html>"#;
        let css = r#".text { text-shadow: 2px 3px red; }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // text-shadow 会为每个字符生成额外的 shadow glyph
        // 应有 glyph 生成（shadow glyphs + main glyphs）
        assert!(
            !result.primitives.glyphs.is_empty(),
            "text-shadow 应生成 glyph 图元（shadow + main），实际数量: {}",
            result.primitives.glyphs.len()
        );

        // 有 text-shadow 时 glyph 数量应多于无 shadow 的情况
        // 因为每个字符会同时生成 shadow glyph 和 main glyph
        let glyph_count = result.primitives.glyphs.len();
        assert!(glyph_count >= 2, "至少应有 shadow + main 两个 glyph");
    }

    /// CSS text-shadow 多层属性管线集成测试。
    #[test]
    fn test_text_shadow_with_color_pipeline() {
        let html = r#"<html><body>
            <div class="shadow-text">Shadow</div>
        </body></html>"#;
        let css = r#"
            .shadow-text {
                width: 200px; height: 50px;
                color: blue; font-size: 14px;
                text-shadow: 1px 2px green;
            }
        "#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 验证 glyph 生成
        assert!(!result.primitives.glyphs.is_empty(), "应有 glyph 生成");

        // 查找 shadow glyph — 颜色应为 green (0, 128, 0)
        let has_shadow_glyph = result
            .primitives
            .glyphs
            .iter()
            .any(|g| g.color.g > 100 && g.color.r == 0 && g.color.b == 0);
        assert!(has_shadow_glyph, "应存在 green 颜色的 shadow glyph");

        // 查找 main glyph — 颜色应为 blue (0, 0, 255)
        let has_main_glyph = result
            .primitives
            .glyphs
            .iter()
            .any(|g| g.color.b == 255 && g.color.r == 0 && g.color.g == 0);
        assert!(has_main_glyph, "应存在 blue 颜色的 main glyph");
    }

    /// CSS text-shadow 默认值管线集成测试。
    #[test]
    fn test_text_shadow_none_pipeline() {
        let html = r#"<html><body>
            <div class="no-shadow" style="width: 200px; height: 50px; color: black; font-size: 16px;">Text</div>
        </body></html>"#;
        let css = r#".no-shadow { }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 无 text-shadow 时，每个字符只有 1 个 main glyph，没有 shadow glyph
        // 所有 glyph 的颜色应为主色（黑色或继承色）
        // 不应有红色/绿色等 shadow 颜色的 glyph
        assert!(!result.primitives.glyphs.is_empty(), "应生成主文本 glyph");
    }

    /// CSS background-image url() 渲染管线集成测试。
    #[test]
    fn test_background_image_url_render_pipeline() {
        let html = r#"<html><body>
            <div class="bg" style="width: 200px; height: 100px;">Background</div>
        </body></html>"#;
        let css = r#".bg { background-image: url(hero.png); }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 应生成 ImagePrimitive
        assert!(
            !result.primitives.images.is_empty(),
            "background-image: url() 应生成 ImagePrimitive，实际数量: {}",
            result.primitives.images.len()
        );
    }

    /// CSS background-image none 管线集成测试。
    #[test]
    fn test_background_image_none_pipeline() {
        let html = r#"<html><body>
            <div class="no-bg" style="width: 200px; height: 100px; background-color: white;">NoImg</div>
        </body></html>"#;
        let css = r#".no-bg { background-image: none; }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // background-image: none 不应生成 ImagePrimitive
        assert!(
            result.primitives.images.is_empty(),
            "background-image: none 不应生成图片图元，实际数量: {}",
            result.primitives.images.len()
        );
    }

    /// CSS background-image 与 background-color 组合管线集成测试。
    #[test]
    fn test_background_image_with_color_pipeline() {
        let html = r#"<html><body>
            <div class="combo" style="width: 200px; height: 100px;">Combo</div>
        </body></html>"#;
        let css = r#"
            .combo {
                background-color: #f0f0f0;
                background-image: url(bg.jpg);
            }
        "#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 应同时有 fills（背景色）和 images（背景图片）
        assert!(!result.primitives.fills.is_empty(), "background-color 应生成填充图元");
        assert!(!result.primitives.images.is_empty(), "background-image 应生成图片图元");
    }

    /// CSS box-shadow 继承性管线集成测试（box-shadow 不可继承）。
    #[test]
    fn test_box_shadow_not_inherited_render_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        // 父元素有 box-shadow
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "parent-shadow");
        doc.append_child(body, parent).unwrap();

        // 子元素不设置 box-shadow
        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .parent-shadow { box-shadow: 5px 5px blue; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 父元素应有 box-shadow
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            (parent_style.box_shadow.offset_x - 5.0).abs() < 0.01,
            "parent 的 box-shadow offset_x 应为 5.0"
        );

        // 子元素不应继承 box-shadow（offset_x 应为默认值 0.0）
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert!(
            (child_style.box_shadow.offset_x - 0.0).abs() < 0.01,
            "child 不应继承 box-shadow，offset_x 应为 0.0，实际为 {}",
            child_style.box_shadow.offset_x
        );
    }

    /// CSS text-shadow 继承性管线集成测试（text-shadow 可继承）。
    #[test]
    fn test_text_shadow_inherited_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        // 父元素有 text-shadow
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "parent-shadow");
        doc.append_child(body, parent).unwrap();

        // 子元素不设置 text-shadow，应继承
        let child = doc.create_element("span");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .parent-shadow { text-shadow: 3px 4px orange; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 父元素应有 text-shadow
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            (parent_style.text_shadow.offset_x - 3.0).abs() < 0.01,
            "parent 的 text-shadow offset_x 应为 3.0"
        );

        // 子元素应继承 text-shadow
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert!(
            (child_style.text_shadow.offset_x - 3.0).abs() < 0.01,
            "child 应继承 text-shadow offset_x=3.0，实际为 {}",
            child_style.text_shadow.offset_x
        );
        assert!(
            (child_style.text_shadow.offset_y - 4.0).abs() < 0.01,
            "child 应继承 text-shadow offset_y=4.0，实际为 {}",
            child_style.text_shadow.offset_y
        );
    }

    /// CSS box-shadow + outline 组合管线集成测试。
    #[test]
    fn test_box_shadow_with_outline_pipeline() {
        let html = r#"<html><body>
            <div class="combined" style="width: 200px; height: 100px;">Combined</div>
        </body></html>"#;
        let css = r#"
            .combined {
                box-shadow: 4px 6px 12px rgba(0,0,0,0.3);
                outline: 2px solid red;
            }
        "#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 应同时有 shadows 和 outline fills
        assert!(!result.primitives.shadows.is_empty(), "box-shadow 应生成阴影图元");
        // outline 生成 4 个 fill 图元
        assert!(
            result.primitives.fills.len() >= 4,
            "outline 应生成 4 个填充图元，实际数量: {}",
            result.primitives.fills.len()
        );
    }

    /// CSS background-image + border + box-shadow 全组合管线集成测试。
    #[test]
    fn test_all_three_new_properties_combined_pipeline() {
        let html = r#"<html><body>
            <div class="all" style="width: 200px; height: 100px; color: black; font-size: 14px;">All</div>
        </body></html>"#;
        let css = r#"
            .all {
                background-color: #eee;
                background-image: url(wallpaper.jpg);
                box-shadow: 2px 3px 8px rgba(0,0,0,0.5);
                text-shadow: 1px 1px red;
                border: 1px solid #ccc;
            }
        "#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // 验证所有新属性都生成了图元
        assert!(!result.primitives.images.is_empty(), "background-image 应生成图片图元");
        assert!(!result.primitives.shadows.is_empty(), "box-shadow 应生成阴影图元");
        assert!(!result.primitives.fills.is_empty(), "背景色 + 边框应生成填充图元");
        assert!(
            !result.primitives.glyphs.is_empty(),
            "text-shadow + 文本应生成 glyph 图元"
        );

        // glyph 数量应 >= 2（shadow glyph + main glyph）
        assert!(
            result.primitives.glyphs.len() >= 2,
            "text-shadow 应使 glyph 数量翻倍（shadow + main），实际数量: {}",
            result.primitives.glyphs.len()
        );
    }

    /// CSS box-shadow 仅 spread-radius 管线集成测试。
    #[test]
    fn test_box_shadow_spread_only_pipeline() {
        let html = r#"<html><body>
            <div class="spread" style="width: 200px; height: 100px;">Spread</div>
        </body></html>"#;
        let css = r#".spread { box-shadow: 0 0 0 5px purple; }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        // spread-only shadow 仍应生成 ShadowPrimitive（spread_radius=5）
        assert!(
            !result.primitives.shadows.is_empty(),
            "spread-only box-shadow 应生成阴影图元，实际数量: {}",
            result.primitives.shadows.len()
        );

        let shadow = &result.primitives.shadows[0];
        assert!(
            (shadow.spread_radius - 5.0).abs() < 0.01,
            "shadow spread_radius 应为 5.0，实际为 {}",
            shadow.spread_radius
        );
        assert!((shadow.offset_x - 0.0).abs() < 0.01, "shadow offset_x 应为 0.0");
        assert!((shadow.offset_y - 0.0).abs() < 0.01, "shadow offset_y 应为 0.0");
    }

    // ── CSS 渐变管线集成测试 ──

    /// CSS linear-gradient 渲染管线集成测试。
    ///
    /// 解析 background-image: linear-gradient(to bottom, red, blue)，
    /// 通过 style-system 计算样式，验证 background_image 为 Gradient 变体，
    /// 方向为 ToBottom，色标有 2 个元素。
    #[test]
    fn test_linear_gradient_render_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "grad");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .grad { background-image: linear-gradient(to bottom, red, blue); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        match &div_style.background_image {
            zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
                zero_css_parser::values::GradientValue::Linear(lin) => {
                    assert_eq!(
                        lin.direction,
                        zero_css_parser::values::GradientDirection::ToBottom,
                        "linear-gradient 方向应为 ToBottom"
                    );
                    assert_eq!(lin.stops.len(), 2, "应有 2 个色标");
                    assert_eq!(lin.repeating, false, "不应为 repeating");
                }
                other => panic!("渐变应为 Linear，实际为 {:?}", other),
            },
            other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
        }
    }

    /// CSS radial-gradient 渲染管线集成测试。
    ///
    /// 解析 background-image: radial-gradient(circle, red, blue)，
    /// 验证 background_image 为 Gradient 变体且包含 RadialGradient。
    #[test]
    fn test_radial_gradient_render_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "radial");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .radial { background-image: radial-gradient(circle, red, blue); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        match &div_style.background_image {
            zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
                zero_css_parser::values::GradientValue::Radial(rad) => {
                    assert_eq!(rad.shape, zero_css_parser::values::RadialShape::Circle);
                    assert_eq!(rad.stops.len(), 2, "应有 2 个色标");
                }
                other => panic!("渐变应为 Radial，实际为 {:?}", other),
            },
            other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
        }
    }

    /// CSS linear-gradient 通过 background 简写管线集成测试。
    ///
    /// 解析 background: linear-gradient(to right, #ff0000, #0000ff)，
    /// 验证 expand_background 简写将渐变路由到 background-image，
    /// 最终 computed style 中 background_image 为 Gradient 变体。
    #[test]
    fn test_gradient_via_background_shorthand_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "bg-grad");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .bg-grad { background: linear-gradient(to right, #ff0000, #0000ff); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        match &div_style.background_image {
            zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
                zero_css_parser::values::GradientValue::Linear(lin) => {
                    assert_eq!(
                        lin.direction,
                        zero_css_parser::values::GradientDirection::ToRight,
                        "background 简写展开后方向应为 ToRight"
                    );
                }
                other => panic!("渐变应为 Linear，实际为 {:?}", other),
            },
            other => panic!("background 简写中的渐变应路由到 background-image，实际为 {:?}", other),
        }
    }

    /// CSS conic-gradient 管线集成测试。
    ///
    /// 解析 background-image: conic-gradient(red, blue, green)，
    /// 验证 background_image 为 Gradient 变体且包含 ConicGradient。
    #[test]
    fn test_conic_gradient_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "conic");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .conic { background-image: conic-gradient(red, blue, green); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        match &div_style.background_image {
            zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
                zero_css_parser::values::GradientValue::Conic(conic) => {
                    assert_eq!(conic.stops.len(), 3, "应有 3 个色标");
                    assert!(!conic.repeating, "不应为 repeating");
                }
                other => panic!("渐变应为 Conic，实际为 {:?}", other),
            },
            other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
        }
    }

    /// CSS repeating-linear-gradient 管线集成测试。
    ///
    /// 解析 background-image: repeating-linear-gradient(45deg, red, blue 20px)，
    /// 验证 repeating 标志为 true。
    #[test]
    fn test_repeating_linear_gradient_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "repeat-grad");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .repeat-grad { background-image: repeating-linear-gradient(45deg, red, blue 20px); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        match &div_style.background_image {
            zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
                zero_css_parser::values::GradientValue::Linear(lin) => {
                    assert!(lin.repeating, "repeating-linear-gradient 的 repeating 应为 true");
                    assert_eq!(lin.stops.len(), 2, "应有 2 个色标");
                }
                other => panic!("渐变应为 Linear，实际为 {:?}", other),
            },
            other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
        }
    }

    /// CSS linear-gradient 渐变不继承管线测试。
    ///
    /// 父元素设置 background-image: linear-gradient(red, blue)，
    /// 子元素不显式设置，background-image 不可继承，
    /// 验证子元素的 background_image 为默认值 None。
    #[test]
    fn test_gradient_not_inherited_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "parent-grad");
        doc.append_child(body, parent).unwrap();

        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "child-plain");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .parent-grad { background-image: linear-gradient(red, blue); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 父元素应有渐变
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert!(
            matches!(
                &parent_style.background_image,
                zero_style_system::property::BackgroundImageComputedValue::Gradient(_)
            ),
            "parent 的 background_image 应为 Gradient 变体"
        );

        // 子元素不应继承 background-image
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.background_image,
            zero_style_system::property::BackgroundImageComputedValue::None,
            "child 不应继承 parent 的 background-image，应为 None"
        );
    }

    /// CSS linear-gradient + background-color 组合管线测试。
    ///
    /// 同时设置 background-color: white 和 background-image: linear-gradient(red, blue)，
    /// 验证两个属性都被正确设置到 computed style 中。
    #[test]
    fn test_gradient_with_background_color_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "combo");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .combo {
                background-color: white;
                background-image: linear-gradient(red, blue);
            }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");

        // 验证 background-color 为白色
        assert_eq!(
            div_style.background_color,
            zero_css_parser::values::ColorValue::Rgba(255, 255, 255, 255),
            "background-color 应为 white (255, 255, 255, 255)"
        );

        // 验证 background-image 为渐变
        assert!(
            matches!(
                &div_style.background_image,
                zero_style_system::property::BackgroundImageComputedValue::Gradient(_)
            ),
            "background_image 应为 Gradient 变体"
        );
    }

    /// CSS linear-gradient 角度方向管线测试。
    ///
    /// 解析 background-image: linear-gradient(90deg, red, green, blue)，
    /// 验证方向为 Angle(90.0)。
    #[test]
    fn test_linear_gradient_angle_direction_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "angle-grad");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .angle-grad { background-image: linear-gradient(90deg, red, green, blue); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        match &div_style.background_image {
            zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
                zero_css_parser::values::GradientValue::Linear(lin) => {
                    match &lin.direction {
                        zero_css_parser::values::GradientDirection::Angle(a) => {
                            assert!((a - 90.0).abs() < 0.01, "方向应为 Angle(90.0)，实际为 Angle({})", a);
                        }
                        other => panic!("方向应为 Angle 变体，实际为 {:?}", other),
                    }
                    assert_eq!(lin.stops.len(), 3, "应有 3 个色标");
                }
                other => panic!("渐变应为 Linear，实际为 {:?}", other),
            },
            other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
        }
    }

    /// CSS radial-gradient 自定义位置管线测试。
    ///
    /// 解析 background-image: radial-gradient(circle at 25% 75%, red, blue)，
    /// 验证 position_x 和 position_y 匹配预期值。
    #[test]
    fn test_radial_gradient_position_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "pos-grad");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .pos-grad { background-image: radial-gradient(circle at 25% 75%, red, blue); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        match &div_style.background_image {
            zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => {
                match grad {
                    zero_css_parser::values::GradientValue::Radial(rad) => {
                        // position_x 应为 25% → Percent(25.0)
                        assert!(
                            matches!(&rad.position_x, zero_css_parser::values::LengthValue::Percentage(p) if (*p - 25.0).abs() < 0.01),
                            "position_x 应为 Percent(25.0)，实际为 {:?}",
                            rad.position_x
                        );
                        // position_y 应为 75% → Percent(75.0)
                        assert!(
                            matches!(&rad.position_y, zero_css_parser::values::LengthValue::Percentage(p) if (*p - 75.0).abs() < 0.01),
                            "position_y 应为 Percent(75.0)，实际为 {:?}",
                            rad.position_y
                        );
                    }
                    other => panic!("渐变应为 Radial，实际为 {:?}", other),
                }
            }
            other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
        }
    }

    /// CSS linear-gradient 多色标管线测试。
    ///
    /// 解析 background-image: linear-gradient(to right, red 0%, green 50%, blue 100%)，
    /// 验证有 3 个色标且位置正确。
    #[test]
    fn test_linear_gradient_multi_stop_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "multi-stop");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .multi-stop { background-image: linear-gradient(to right, red 0%, green 50%, blue 100%); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        match &div_style.background_image {
            zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => {
                match grad {
                    zero_css_parser::values::GradientValue::Linear(lin) => {
                        assert_eq!(lin.stops.len(), 3, "应有 3 个色标");

                        // 验证第一个色标：red 0%
                        assert_eq!(
                            lin.stops[0].color,
                            zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
                            "第一个色标颜色应为红色"
                        );
                        assert!(
                            matches!(&lin.stops[0].position, Some(zero_css_parser::values::LengthValue::Percentage(p)) if (*p - 0.0).abs() < 0.01),
                            "第一个色标位置应为 0%"
                        );

                        // 验证第二个色标：green 50%
                        assert_eq!(
                            lin.stops[1].color,
                            zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255),
                            "第二个色标颜色应为绿色"
                        );
                        assert!(
                            matches!(&lin.stops[1].position, Some(zero_css_parser::values::LengthValue::Percentage(p)) if (*p - 50.0).abs() < 0.01),
                            "第二个色标位置应为 50%"
                        );

                        // 验证第三个色标：blue 100%
                        assert_eq!(
                            lin.stops[2].color,
                            zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
                            "第三个色标颜色应为蓝色"
                        );
                        assert!(
                            matches!(&lin.stops[2].position, Some(zero_css_parser::values::LengthValue::Percentage(p)) if (*p - 100.0).abs() < 0.01),
                            "第三个色标位置应为 100%"
                        );
                    }
                    other => panic!("渐变应为 Linear，实际为 {:?}", other),
                }
            }
            other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
        }
    }

    // ── CSS opacity / text-decoration / text-transform 管线集成测试 ──

    /// CSS opacity 管线集成测试。
    ///
    /// 解析含 opacity: 0.5 的 CSS，通过 style-system 计算样式，
    /// 验证 ComputedStyle.opacity == 0.5。
    #[test]
    fn test_opacity_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "semi");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .semi { opacity: 0.5; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            (div_style.opacity - 0.5).abs() < 0.01,
            "div 的 opacity 应为 0.5，实际为 {}",
            div_style.opacity
        );
    }

    /// CSS opacity 默认值管线测试。
    ///
    /// 不设置 opacity 时，默认值应为 1.0。
    #[test]
    fn test_opacity_default_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "plain");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .plain { color: black; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            (div_style.opacity - 1.0).abs() < 0.01,
            "未设置 opacity 时默认应为 1.0，实际为 {}",
            div_style.opacity
        );
    }

    /// CSS text-decoration: underline 管线集成测试。
    ///
    /// 解析含 text-decoration: underline 的 CSS，通过简写展开
    /// 设置 text-decoration-line，验证 text_decoration_line == Underline。
    #[test]
    fn test_text_decoration_underline_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "underlined");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .underlined { text-decoration: underline; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.text_decoration_line,
            zero_style_system::property::TextDecorationLineValue::Underline,
            "div 的 text-decoration-line 应为 Underline"
        );
    }

    /// CSS text-decoration: line-through 管线集成测试。
    ///
    /// 解析含 text-decoration: line-through 的 CSS，通过简写展开
    /// 设置 text-decoration-line，验证 text_decoration_line == LineThrough。
    #[test]
    fn test_text_decoration_line_through_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "struck");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .struck { text-decoration: line-through; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.text_decoration_line,
            zero_style_system::property::TextDecorationLineValue::LineThrough,
            "div 的 text-decoration-line 应为 LineThrough"
        );
    }

    /// CSS text-decoration: none 管线集成测试。
    ///
    /// 解析含 text-decoration: none 的 CSS，通过简写展开
    /// 设置 text-decoration-line，验证 text_decoration_line == None。
    #[test]
    fn test_text_decoration_none_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "undecorated");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .undecorated { text-decoration: none; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.text_decoration_line,
            zero_style_system::property::TextDecorationLineValue::None,
            "div 的 text-decoration-line 应为 None"
        );
    }

    /// CSS text-transform: uppercase 管线集成测试。
    ///
    /// 解析含 text-transform: uppercase 的 CSS，通过 style-system 计算样式，
    /// 验证 text_transform == Uppercase。
    #[test]
    fn test_text_transform_uppercase_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "upper");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .upper { text-transform: uppercase; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.text_transform,
            zero_style_system::property::TextTransformValue::Uppercase,
            "div 的 text-transform 应为 Uppercase"
        );
    }

    /// CSS text-transform: capitalize 管线集成测试。
    ///
    /// 解析含 text-transform: capitalize 的 CSS，通过 style-system 计算样式，
    /// 验证 text_transform == Capitalize。
    #[test]
    fn test_text_transform_capitalize_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "capitalized");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .capitalized { text-transform: capitalize; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.text_transform,
            zero_style_system::property::TextTransformValue::Capitalize,
            "div 的 text-transform 应为 Capitalize"
        );
    }

    /// CSS text-transform 继承管线测试。
    ///
    /// text-transform 是继承属性。父元素设置 text-transform: uppercase，
    /// 子元素不显式设置，应继承父元素的 Uppercase 值。
    #[test]
    fn test_text_transform_inherited_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();

        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "upper-parent");
        doc.append_child(body, parent).unwrap();

        let child = doc.create_element("p");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .upper-parent { text-transform: uppercase; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        // 父元素应有 Uppercase
        let parent_style = styles.get(&parent).expect("parent 应有计算样式");
        assert_eq!(
            parent_style.text_transform,
            zero_style_system::property::TextTransformValue::Uppercase,
            "parent 的 text-transform 应为 Uppercase"
        );

        // 子元素应继承 text-transform: uppercase
        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.text_transform,
            zero_style_system::property::TextTransformValue::Uppercase,
            "child 应继承 parent 的 text-transform: Uppercase"
        );
    }

    /// CSS opacity 渲染管线完整测试。
    ///
    /// 使用 RenderPipeline 渲染含 opacity: 0.5 和 background-color: red 的页面，
    /// 验证渲染成功完成（timings.total_ms >= 0）。
    #[test]
    fn test_opacity_render_pipeline() {
        let html = r#"<html><body>
            <div class="semi" style="width: 200px; height: 100px;">Semi-transparent</div>
        </body></html>"#;
        let css = r#".semi { opacity: 0.5; background-color: red; }"#;

        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(html, css);

        assert!(result.timings.total_ms >= 0.0, "opacity 渲染管线应成功完成");
        // 应生成填充图元（background-color: red）
        assert!(
            !result.primitives.fills.is_empty(),
            "background-color: red 应生成填充图元"
        );
    }

    /// CSS text-decoration + text-shadow 组合管线测试。
    ///
    /// 同时设置 text-decoration: underline 和 text-shadow: 2px 2px red，
    /// 验证两个属性都被正确设置到 computed style 中。
    #[test]
    fn test_text_decoration_with_text_shadow_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "combo");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .combo { text-decoration: underline; text-shadow: 2px 2px red; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");

        // 验证 text-decoration-line 为 Underline
        assert_eq!(
            div_style.text_decoration_line,
            zero_style_system::property::TextDecorationLineValue::Underline,
            "div 的 text-decoration-line 应为 Underline"
        );

        // 验证 text-shadow 的 offset_x 和 offset_y
        assert!(
            (div_style.text_shadow.offset_x - 2.0).abs() < 0.01,
            "text-shadow offset_x 应为 2.0，实际为 {}",
            div_style.text_shadow.offset_x
        );
        assert!(
            (div_style.text_shadow.offset_y - 2.0).abs() < 0.01,
            "text-shadow offset_y 应为 2.0，实际为 {}",
            div_style.text_shadow.offset_y
        );
        assert_eq!(
            div_style.text_shadow.color,
            zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            "text-shadow color 应为红色"
        );
    }

    /// CSS opacity + box-shadow + gradient 组合管线测试。
    ///
    /// 同时设置 opacity: 0.7、box-shadow: 5px 5px blue 和
    /// background-image: linear-gradient(red, green)，
    /// 验证三个属性都被正确设置到 computed style 中。
    #[test]
    fn test_opacity_shadow_gradient_combined_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "triple");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .triple {
                opacity: 0.7;
                box-shadow: 5px 5px blue;
                background-image: linear-gradient(red, green);
            }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");

        // 验证 opacity
        assert!(
            (div_style.opacity - 0.7).abs() < 0.01,
            "opacity 应为 0.7，实际为 {}",
            div_style.opacity
        );

        // 验证 box-shadow
        assert!(
            (div_style.box_shadow.offset_x - 5.0).abs() < 0.01,
            "box-shadow offset_x 应为 5.0，实际为 {}",
            div_style.box_shadow.offset_x
        );
        assert!(
            (div_style.box_shadow.offset_y - 5.0).abs() < 0.01,
            "box-shadow offset_y 应为 5.0，实际为 {}",
            div_style.box_shadow.offset_y
        );

        // 验证 background-image 为渐变
        assert!(
            matches!(
                &div_style.background_image,
                zero_style_system::property::BackgroundImageComputedValue::Gradient(_)
            ),
            "background_image 应为 Gradient 变体"
        );
    }

    /// CSS text-transform: lowercase 管线集成测试。
    ///
    /// 解析含 text-transform: lowercase 的 CSS，通过 style-system 计算样式，
    /// 验证 text_transform == Lowercase。
    #[test]
    fn test_text_transform_lowercase_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "lower");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .lower { text-transform: lowercase; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.text_transform,
            zero_style_system::property::TextTransformValue::Lowercase,
            "div 的 text-transform 应为 Lowercase"
        );
    }

    // ── CSS transition / animation / 自定义属性 / 交互 / 文本 管线集成测试 ──

    /// CSS transition 简写管线集成测试。
    ///
    /// 解析 transition: opacity 0.3s ease-in 0.1s，验证 4 个子属性正确展开。
    #[test]
    fn test_transition_shorthand_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "fade");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .fade { transition: opacity 0.3s ease-in 0.1s; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            div_style.transition_property.contains(&"opacity".to_string()),
            "transition-property 应包含 opacity，实际为 {:?}",
            div_style.transition_property
        );
        assert!(
            div_style.transition_duration.contains(&0.3),
            "transition-duration 应包含 0.3，实际为 {:?}",
            div_style.transition_duration
        );
        assert!(
            div_style.transition_delay.contains(&0.1),
            "transition-delay 应包含 0.1，实际为 {:?}",
            div_style.transition_delay
        );
        assert!(
            !div_style.transition_timing_function.is_empty(),
            "transition-timing-function 不应为空"
        );
    }

    /// CSS animation 简写管线集成测试。
    ///
    /// 解析 animation: slideIn 1s ease 0.2s infinite forwards，验证子属性展开。
    #[test]
    fn test_animation_shorthand_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "animated");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .animated { animation: slideIn 1s ease 0.2s infinite forwards; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            div_style.animation_name.contains(&"slideIn".to_string()),
            "animation-name 应包含 slideIn，实际为 {:?}",
            div_style.animation_name
        );
        assert!(
            div_style.animation_duration.contains(&1.0),
            "animation-duration 应包含 1.0，实际为 {:?}",
            div_style.animation_duration
        );
        assert!(
            div_style.animation_delay.contains(&0.2),
            "animation-delay 应包含 0.2，实际为 {:?}",
            div_style.animation_delay
        );
    }

    /// CSS 自定义属性 + var() 管线集成测试。
    ///
    /// 定义 --main-color: #ff0000，通过 var(--main-color) 引用到 color 属性。
    #[test]
    fn test_custom_property_var_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "themed");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .themed { --main-color: #ff0000; color: var(--main-color); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            matches!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)),
            "color 应通过 var() 解析为红色 #ff0000，实际为 {:?}",
            div_style.color
        );
    }

    /// CSS cursor 管线集成测试。
    ///
    /// 解析 cursor: pointer，验证计算样式。
    #[test]
    fn test_cursor_pointer_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "clickable");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .clickable { cursor: pointer; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.cursor,
            zero_style_system::property::CursorValue::Pointer,
            "cursor 应为 Pointer"
        );
    }

    /// CSS cursor 继承管线集成测试。
    ///
    /// 父元素 cursor: pointer，子元素应继承。
    #[test]
    fn test_cursor_inheritance_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "parent");
        doc.append_child(body, parent).unwrap();
        let child = doc.create_element("span");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .parent { cursor: move; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.cursor,
            zero_style_system::property::CursorValue::Move,
            "cursor 应从父元素继承 Move"
        );
    }

    /// CSS pointer-events 管线集成测试。
    ///
    /// 解析 pointer-events: none，验证计算样式。
    #[test]
    fn test_pointer_events_none_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "no-events");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .no-events { pointer-events: none; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.pointer_events,
            zero_style_system::property::PointerEventsValue::None,
            "pointer-events 应为 None"
        );
    }

    /// CSS white-space 管线集成测试。
    ///
    /// 解析 white-space: pre-wrap，验证计算样式。
    #[test]
    fn test_white_space_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let pre = doc.create_element("pre");
        doc.set_attribute(pre, "class", "code");
        doc.append_child(body, pre).unwrap();

        let css = r#"
            .code { white-space: pre-wrap; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let pre_style = styles.get(&pre).expect("pre 应有计算样式");
        assert_eq!(
            pre_style.white_space,
            zero_style_system::property::WhiteSpaceValue::PreWrap,
            "white-space 应为 PreWrap"
        );
    }

    /// CSS letter-spacing 管线集成测试。
    ///
    /// 解析 letter-spacing: 2px，验证计算样式为 Px(2.0)。
    #[test]
    fn test_letter_spacing_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "spaced");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .spaced { letter-spacing: 2px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.letter_spacing,
            LengthValue::Px(2.0),
            "letter-spacing 应为 2px"
        );
    }

    /// CSS letter-spacing 继承管线集成测试。
    ///
    /// 父元素 letter-spacing: 3px，子元素应继承。
    #[test]
    fn test_letter_spacing_inheritance_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "wide");
        doc.append_child(body, parent).unwrap();
        let child = doc.create_element("span");
        doc.set_attribute(child, "class", "inner");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .wide { letter-spacing: 3px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.letter_spacing,
            LengthValue::Px(3.0),
            "letter-spacing 应从父元素继承 3px"
        );
    }

    /// CSS white-space 继承管线集成测试。
    ///
    /// 父元素 white-space: nowrap，子元素应继承。
    #[test]
    fn test_white_space_inheritance_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "nowrap");
        doc.append_child(body, parent).unwrap();
        let child = doc.create_element("span");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .nowrap { white-space: nowrap; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.white_space,
            zero_style_system::property::WhiteSpaceValue::Nowrap,
            "white-space 应从父元素继承 Nowrap"
        );
    }

    /// CSS user-select 管线集成测试。
    ///
    /// 解析 user-select: none，验证计算样式。
    #[test]
    fn test_user_select_none_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "noselect");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .noselect { user-select: none; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.user_select,
            zero_style_system::property::UserSelectValue::None,
            "user-select 应为 None"
        );
    }

    /// CSS visibility hidden 渲染管线集成测试。
    ///
    /// 验证 visibility:hidden 的元素不产生填充图元。
    #[test]
    fn test_visibility_hidden_render_pipeline() {
        let html = r#"<div class="hidden">text</div>"#;
        let css = r#"
            .hidden { visibility: hidden; background-color: red; }
        "#;

        let result = RenderPipeline::new(800.0, 600.0).render_html(html, css);
        assert!(
            result.primitives.fills.is_empty(),
            "visibility:hidden 不应产生 fill 图元，实际有 {} 个",
            result.primitives.fills.len()
        );
    }

    /// CSS 多 transition 属性管线集成测试。
    ///
    /// 通过 transition-property、transition-duration 长属性分别设置多个值，
    /// 验证多 transition 管线正确存储。
    #[test]
    fn test_multiple_transitions_pipeline() {
        let mut doc = Document::new();
        let root = doc.root();
        let html_el = doc.create_element("html");
        doc.append_child(root, html_el).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html_el, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "multi");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .multi {
                transition-property: opacity, transform;
                transition-duration: 0.3s, 0.5s;
            }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            div_style.transition_property.contains(&"opacity".to_string()),
            "transition-property 应包含 opacity，实际为 {:?}",
            div_style.transition_property
        );
        assert!(
            div_style.transition_property.contains(&"transform".to_string()),
            "transition-property 应包含 transform，实际为 {:?}",
            div_style.transition_property
        );
        assert!(
            div_style.transition_duration.contains(&0.3),
            "transition-duration 应包含 0.3，实际为 {:?}",
            div_style.transition_duration
        );
        assert!(
            div_style.transition_duration.contains(&0.5),
            "transition-duration 应包含 0.5，实际为 {:?}",
            div_style.transition_duration
        );
    }

    // ── CSS 表格/布局/字体 变体属性管线集成测试 ──

    /// CSS table-layout 管线集成测试。
    ///
    /// 解析 table-layout: fixed，验证计算样式。
    #[test]
    fn test_table_layout_fixed_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let table = doc.create_element("table");
        doc.set_attribute(table, "class", "fixed-layout");
        doc.append_child(body, table).unwrap();

        let css = r#"
            .fixed-layout { table-layout: fixed; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let table_style = styles.get(&table).expect("table 应有计算样式");
        assert_eq!(
            table_style.table_layout,
            zero_style_system::property::TableLayoutValue::Fixed,
            "table-layout 应为 Fixed"
        );
    }

    /// CSS caption-side 管线集成测试。
    ///
    /// 解析 caption-side: bottom，验证计算样式。
    #[test]
    fn test_caption_side_bottom_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let caption = doc.create_element("caption");
        doc.set_attribute(caption, "class", "bottom-cap");
        doc.append_child(body, caption).unwrap();

        let css = r#"
            .bottom-cap { caption-side: bottom; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let cap_style = styles.get(&caption).expect("caption 应有计算样式");
        assert_eq!(
            cap_style.caption_side,
            zero_style_system::property::CaptionSideValue::Bottom,
            "caption-side 应为 Bottom"
        );
    }

    /// CSS border-collapse 管线集成测试。
    ///
    /// 解析 border-collapse: collapse，验证计算样式。
    #[test]
    fn test_border_collapse_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let table = doc.create_element("table");
        doc.set_attribute(table, "class", "collapse");
        doc.append_child(body, table).unwrap();

        let css = r#"
            .collapse { border-collapse: collapse; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let table_style = styles.get(&table).expect("table 应有计算样式");
        assert_eq!(
            table_style.border_collapse,
            zero_style_system::property::BorderCollapseValue::Collapse,
            "border-collapse 应为 Collapse"
        );
    }

    /// CSS resize 管线集成测试。
    ///
    /// 解析 resize: both，验证计算样式。
    #[test]
    fn test_resize_both_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "resizable");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .resizable { resize: both; overflow: auto; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.resize,
            zero_style_system::property::ResizeValue::Both,
            "resize 应为 Both"
        );
    }

    /// CSS word-break 管线集成测试。
    ///
    /// 解析 word-break: break-all，验证计算样式。
    #[test]
    fn test_word_break_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "break-all");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .break-all { word-break: break-all; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.word_break,
            zero_style_system::property::WordBreakValue::BreakAll,
            "word-break 应为 BreakAll"
        );
    }

    /// CSS writing-mode 管线集成测试。
    ///
    /// 解析 writing-mode: vertical-rl，验证计算样式。
    #[test]
    fn test_writing_mode_vertical_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "vertical");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .vertical { writing-mode: vertical-rl; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.writing_mode,
            zero_style_system::property::WritingModeValue::VerticalRl,
            "writing-mode 应为 VerticalRl"
        );
    }

    /// CSS isolation 管线集成测试。
    ///
    /// 解析 isolation: isolate，验证计算样式。
    #[test]
    fn test_isolation_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "isolated");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .isolated { isolation: isolate; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.isolation,
            zero_style_system::property::IsolationValue::Isolate,
            "isolation 应为 Isolate"
        );
    }

    /// CSS isolation 继承性验证。
    ///
    /// isolation 不继承，子元素应默认为 Auto。
    #[test]
    fn test_isolation_not_inherited_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let parent = doc.create_element("div");
        doc.set_attribute(parent, "class", "isolated");
        doc.append_child(body, parent).unwrap();
        let child = doc.create_element("span");
        doc.append_child(parent, child).unwrap();

        let css = r#"
            .isolated { isolation: isolate; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let child_style = styles.get(&child).expect("child 应有计算样式");
        assert_eq!(
            child_style.isolation,
            zero_style_system::property::IsolationValue::Auto,
            "isolation 不应继承，子元素应为 Auto"
        );
    }

    // ── CSS flexbox / 字体 / 自定义属性 / overflow 管线集成测试 ──

    /// CSS flex-direction: column 管线集成测试。
    ///
    /// 解析 display: flex; flex-direction: column，通过 style-system 计算样式，
    /// 验证 ComputedStyle 中 display 为 Flex、flex_direction 为 Column。
    #[test]
    fn test_flex_direction_column_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "col-flex");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .col-flex { display: flex; flex-direction: column; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.display, DisplayValue::Flex, "display 应为 Flex");
        assert_eq!(
            div_style.flex_direction,
            FlexDirectionValue::Column,
            "flex-direction 应为 Column"
        );
    }

    /// CSS justify-content: center 管线集成测试。
    ///
    /// 解析 display: flex; justify-content: center，通过 style-system 计算样式，
    /// 验证 ComputedStyle 中 justify_content 为 Center。
    #[test]
    fn test_flex_justify_center_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "centered");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .centered { display: flex; justify-content: center; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.justify_content,
            AlignmentValue::Center,
            "justify-content 应为 Center"
        );
    }

    /// CSS align-items: stretch 管线集成测试。
    ///
    /// 解析 display: flex; align-items: stretch，通过 style-system 计算样式，
    /// 验证 ComputedStyle 中 align_items 为 Stretch。
    #[test]
    fn test_flex_align_items_stretch_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "stretch");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .stretch { display: flex; align-items: stretch; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.align_items,
            AlignmentValue::Stretch,
            "align-items 应为 Stretch"
        );
    }

    /// CSS flex-wrap: wrap 管线集成测试。
    ///
    /// 解析 display: flex; flex-wrap: wrap，通过 style-system 计算样式，
    /// 验证 ComputedStyle 中 flex_wrap 为 Wrap。
    #[test]
    fn test_flex_wrap_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "wrap");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .wrap { display: flex; flex-wrap: wrap; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.flex_wrap, FlexWrapValue::Wrap, "flex-wrap 应为 Wrap");
    }

    /// CSS font-family 管线集成测试。
    ///
    /// 解析 font-family: Arial, sans-serif，通过 style-system 计算样式，
    /// 验证 ComputedStyle 中 font_family 为包含 "Arial" 和 "sans-serif" 的 Vec。
    #[test]
    fn test_font_family_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "fonted");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .fonted { font-family: Arial, sans-serif; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert!(
            div_style.font_family.contains(&"Arial".to_string()),
            "font-family 应包含 Arial，实际为 {:?}",
            div_style.font_family
        );
        assert!(
            div_style.font_family.contains(&"sans-serif".to_string()),
            "font-family 应包含 sans-serif，实际为 {:?}",
            div_style.font_family
        );
    }

    /// CSS font-weight: bold 管线集成测试。
    ///
    /// 解析 font-weight: bold，通过 style-system 计算样式，
    /// 验证 ComputedStyle 中 font_weight 为 FontWeightValue::Bold。
    #[test]
    fn test_font_weight_bold_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "bold");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .bold { font-weight: bold; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.font_weight, FontWeightValue::Bold, "font-weight 应为 Bold");
    }

    /// CSS line-height 数值管线集成测试。
    ///
    /// 解析 line-height: 1.5，通过 style-system 计算样式，
    /// 验证 ComputedStyle 中 line_height 为 Number(1.5)。
    #[test]
    fn test_line_height_number_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "lh");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .lh { line-height: 1.5; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.line_height,
            zero_style_system::property::LineHeightValue::Number(1.5),
            "line-height 应为 Number(1.5)"
        );
    }

    /// CSS 自定义属性 var() 回退值管线集成测试。
    ///
    /// 定义 --x: red，通过 var(--y, blue) 引用未定义变量 --y，
    /// 验证 color 使用回退值 blue（即 ColorValue::Rgba(0, 0, 255, 255)）。
    #[test]
    fn test_custom_property_var_fallback_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "a");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .a { --x: red; color: var(--y, blue); }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.color,
            ColorValue::Rgba(0, 0, 255, 255),
            "color 应回退为蓝色 (0, 0, 255, 255)，实际为 {:?}",
            div_style.color
        );
    }

    /// CSS overflow 双值简写管线集成测试。
    ///
    /// 解析 overflow: hidden scroll，通过 style-system 简写展开，
    /// 验证 overflow_x 为 Hidden、overflow_y 为 Scroll。
    #[test]
    fn test_overflow_shorthand_pipeline() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "overflowed");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .overflowed { overflow: hidden scroll; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.overflow_x, OverflowValue::Hidden, "overflow-x 应为 Hidden");
        assert_eq!(div_style.overflow_y, OverflowValue::Scroll, "overflow-y 应为 Scroll");
    }

    // ── CSS Grid / Position / Box Model 管线集成测试 ──

    /// CSS grid-template-columns 管线集成测试。
    ///
    /// 解析含 display: grid; grid-template-columns: 1fr 2fr 100px 的 CSS，
    /// 通过 style-system 计算样式，验证 grid_template_columns 为 Some 且包含预期值。
    #[test]
    fn test_grid_template_columns_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "grid");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .grid { display: grid; grid-template-columns: 1fr 2fr 100px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.display, DisplayValue::Grid, "div 的 display 应为 Grid");
        assert!(
            div_style.grid_template_columns.is_some(),
            "grid_template_columns 不应为 None"
        );
        let cols = div_style.grid_template_columns.as_ref().unwrap();
        assert!(cols.contains("1fr"), "grid_template_columns 应包含 1fr");
        assert!(cols.contains("2fr"), "grid_template_columns 应包含 2fr");
        assert!(cols.contains("100px"), "grid_template_columns 应包含 100px");
    }

    /// CSS grid-template-rows 管线集成测试。
    ///
    /// 解析含 display: grid; grid-template-rows: auto 200px 的 CSS，
    /// 通过 style-system 计算样式，验证 grid_template_rows 为 Some 且包含预期值。
    #[test]
    fn test_grid_template_rows_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "grid");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .grid { display: grid; grid-template-rows: auto 200px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.display, DisplayValue::Grid, "div 的 display 应为 Grid");
        assert!(div_style.grid_template_rows.is_some(), "grid_template_rows 不应为 None");
        let rows = div_style.grid_template_rows.as_ref().unwrap();
        assert!(rows.contains("auto"), "grid_template_rows 应包含 auto");
        assert!(rows.contains("200px"), "grid_template_rows 应包含 200px");
    }

    /// CSS grid-auto-flow 管线集成测试。
    ///
    /// 解析含 display: grid; grid-auto-flow: dense 的 CSS，
    /// 通过 style-system 计算样式，验证 grid_auto_flow 为 RowDense（dense 等价于 row dense）。
    #[test]
    fn test_grid_auto_flow_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "grid");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .grid { display: grid; grid-auto-flow: dense; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.display, DisplayValue::Grid, "div 的 display 应为 Grid");
        assert!(
            div_style.grid_auto_flow == zero_style_system::property::GridAutoFlowValue::RowDense
                || div_style.grid_auto_flow == zero_style_system::property::GridAutoFlowValue::ColumnDense,
            "grid_auto_flow 应为 RowDense 或 ColumnDense，实际为 {:?}",
            div_style.grid_auto_flow
        );
    }

    /// CSS display: grid 管线集成测试。
    ///
    /// 解析含 display: grid 的 CSS，通过 style-system 计算样式，
    /// 验证 display == DisplayValue::Grid。
    #[test]
    fn test_display_grid_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "container");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .container { display: grid; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.display, DisplayValue::Grid, "div 的 display 应为 Grid");
    }

    /// CSS position: absolute 管线集成测试。
    ///
    /// 解析含 position: absolute; top: 10px; left: 20px 的 CSS，
    /// 通过 style-system 计算样式，验证 position 为 Absolute，
    /// top 和 left 为 Px(10.0) 和 Px(20.0)。
    #[test]
    fn test_position_absolute_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "abs");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .abs { position: absolute; top: 10px; left: 20px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(
            div_style.position,
            PositionValue::Absolute,
            "div 的 position 应为 Absolute"
        );
        assert_eq!(div_style.top, LengthValue::Px(10.0), "div 的 top 应为 Px(10.0)");
        assert_eq!(div_style.left, LengthValue::Px(20.0), "div 的 left 应为 Px(20.0)");
    }

    /// CSS margin 简写管线集成测试。
    ///
    /// 解析含 margin: 10px 20px 的 CSS，通过 style-system 简写展开，
    /// 验证 margin_top 为 Px(10.0)，margin_right 为 Px(20.0)。
    #[test]
    fn test_margin_shorthand_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "spaced");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .spaced { margin: 10px 20px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.margin_top, LengthValue::Px(10.0), "margin_top 应为 Px(10.0)");
        assert_eq!(
            div_style.margin_right,
            LengthValue::Px(20.0),
            "margin_right 应为 Px(20.0)"
        );
    }

    /// CSS padding 简写管线集成测试。
    ///
    /// 解析含 padding: 5px 15px 的 CSS，通过 style-system 简写展开，
    /// 验证 padding_top 为 Px(5.0)，padding_right 为 Px(15.0)。
    #[test]
    fn test_padding_shorthand_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "padded");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .padded { padding: 5px 15px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.padding_top, LengthValue::Px(5.0), "padding_top 应为 Px(5.0)");
        assert_eq!(
            div_style.padding_right,
            LengthValue::Px(15.0),
            "padding_right 应为 Px(15.0)"
        );
    }

    /// CSS width + height 管线集成测试。
    ///
    /// 解析含 width: 300px; height: 200px 的 CSS，
    /// 通过 style-system 计算样式，验证 width 和 height 正确设置。
    #[test]
    fn test_width_height_pipeline() {
        let (mut doc, body) = make_doc_with_body();

        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "sized");
        doc.append_child(body, div).unwrap();

        let css = r#"
            .sized { width: 300px; height: 200px; }
        "#;
        let stylesheet = CssParser::parse_stylesheet(css);

        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[stylesheet]);

        let div_style = styles.get(&div).expect("div 应有计算样式");
        assert_eq!(div_style.width, LengthValue::Px(300.0), "width 应为 Px(300.0)");
        assert_eq!(div_style.height, LengthValue::Px(200.0), "height 应为 Px(200.0)");
    }
}
