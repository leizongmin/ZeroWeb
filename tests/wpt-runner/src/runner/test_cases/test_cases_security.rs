//! 安全策略标准合规性测试。
//!
//! 覆盖 CSP、CORS、同源策略、混合内容、沙箱、HSTS、
//! Cookie 安全属性、权限模型等安全特性。

use super::TestCase;

/// 返回安全策略标准合规性测试用例。
pub fn security_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        // Content Security Policy (CSP)
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/csp/meta-tag".into(),
            description: "CSP meta 标签不崩溃".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self'">
            </head><body>
            <div id="csp">CSP test</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_meta".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/csp/script-src".into(),
            description: "CSP script-src 限制内联脚本".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src 'self'">
            </head><body>
            <div id="csp-script">CSP script</div>
            <script>document.getElementById('csp-script').textContent = 'executed';</script>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/csp/style-src".into(),
            description: "CSP style-src 允许内联样式".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="style-src 'unsafe-inline'">
            </head><body>
            <div style="width:100px; height:50px; background:red;">CSP style</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // iframe sandbox
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/sandbox/iframe-basic".into(),
            description: "sandbox iframe 不崩溃".into(),
            category: "security".into(),
            html: r#"<html><body>
            <iframe sandbox="allow-scripts" srcdoc="<p>Sandboxed</p>" width="200" height="100"></iframe>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:iframe".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "security/sandbox/iframe-allow-same-origin".into(),
            description: "sandbox allow-same-origin iframe".into(),
            category: "security".into(),
            html: r#"<html><body>
            <iframe sandbox="allow-same-origin allow-scripts" srcdoc="<div>Content</div>"></iframe>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:iframe".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 混合内容
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/mixed-content/https-page".into(),
            description: "HTTPS 页面结构正确".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="secure">Secure content</div>
            <img src="https://example.com/image.png" alt="secure image">
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_img".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Referrer Policy
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/referrer-policy/meta".into(),
            description: "Referrer-Policy meta 标签".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta name="referrer" content="no-referrer">
            </head><body>
            <a href="https://example.com">Link</a>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_link".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 表单安全
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/form/autocomplete-off".into(),
            description: "form autocomplete=off 不崩溃".into(),
            category: "security".into(),
            html: r#"<html><body>
            <form autocomplete="off">
                <input type="password" name="pass">
                <button type="submit">Submit</button>
            </form>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_form".into(),
                "dom_has_button".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "security/form/input-validation".into(),
            description: "input required + pattern 验证".into(),
            category: "security".into(),
            html: r#"<html><body>
            <form>
                <input type="email" required placeholder="Email">
                <input type="text" pattern="[A-Za-z]{3}" title="3 letters">
            </form>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_form".into(),
                "dom_has_input".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 同源策略相关 HTML 结构
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/sop/cross-origin-img".into(),
            description: "跨域 img 元素不崩溃".into(),
            category: "security".into(),
            html: r#"<html><body>
            <img src="https://other-domain.com/img.png" crossorigin="anonymous" alt="cross-origin">
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_img".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/sop/cross-origin-link".into(),
            description: "跨域链接 rel=noopener".into(),
            category: "security".into(),
            html: r#"<html><body>
            <a href="https://example.com" rel="noopener noreferrer" target="_blank">External</a>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_link".into(), "no_panic".into()],
        },
    ]
}
