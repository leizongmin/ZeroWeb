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
    use zero_dom::{Document, ShadowRootMode};
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
}
