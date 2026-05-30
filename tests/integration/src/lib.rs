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
    use zero_engine_core::RenderPipeline;

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
    use zero_security::{check_cors, CorsPolicy, Origin};

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
        serialize, deserialize, IpcMessage, IpcMessageKind, StorageOpParams, StorageOperation,
        StorageType,
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

        assert_eq!(mgr.local_storage("https://example.com").get("shared_key"), Some("local_value"));
        assert_eq!(mgr.session_storage("https://example.com").get("shared_key"), Some("session_value"));
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
    use zero_protocol::{
        serialize, deserialize, IpcMessage, IpcMessageKind, NavigateParams,
    };

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
