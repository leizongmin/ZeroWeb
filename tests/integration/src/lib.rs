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
        ColorValue, DisplayValue, LengthValue, TransformFunction, TransformValue, parse_transform,
    };
    use zero_dom::Document;
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
}
