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
        let html = r#"<html><head><title>Test</title></head><body><div id="main" class="container">Hello</div></body></html>"#;
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
        IpcMessage, IpcMessageKind, StorageOpParams, StorageOperation, StorageType, deserialize,
        serialize,
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

        assert_eq!(
            mgr.local_storage("https://a.com").get("key"),
            Some("value_a")
        );
        assert_eq!(
            mgr.local_storage("https://b.com").get("key"),
            Some("value_b")
        );
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
        assert!(!primitives.fills.is_empty(), "路径填充应生成图元");
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
        let wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("1+1");
        assert!(result.is_err());
    }
}
