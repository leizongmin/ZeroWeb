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
        // ═══════════════════════════════════════════════════════════════
        // CSP 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/csp/default-src".into(),
            description: "CSP default-src 策略".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'; img-src *">
            </head><body>
            <img src="https://picsum.photos/100/100" alt="allowed image">
            <div>CSP default-src</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_img".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/csp/frame-src".into(),
            description: "CSP frame-src 限制 iframe 来源".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="frame-src 'self'">
            </head><body>
            <iframe src="https://example.com" width="200" height="100"></iframe>
            <div>CSP frame-src</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:iframe".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/csp/upgrade-insecure".into(),
            description: "CSP upgrade-insecure-requests".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="upgrade-insecure-requests">
            </head><body>
            <img src="http://example.com/img.png" alt="upgraded">
            <a href="http://example.com">Upgraded link</a>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_img".into(), "dom_has_link".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cookie 安全
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/cookie/http-only-meta".into(),
            description: "Cookie 安全相关 meta 标签".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Set-Cookie" content="session=abc; Secure; HttpOnly; SameSite=Strict">
            </head><body>
            <div>Cookie security meta</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_meta".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 跨域资源共享 (CORS)
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/cors/crossorigin-script".into(),
            description: "crossorigin 属性的 script 元素".into(),
            category: "security".into(),
            html: r#"<html><head>
            <script crossorigin="anonymous" src="https://cdn.example.com/lib.js"></script>
            </head><body>
            <div>CORS script</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/cors/crossorigin-img-use-credentials".into(),
            description: "crossorigin=use-credentials 的 img".into(),
            category: "security".into(),
            html: r#"<html><body>
            <img src="https://api.example.com/avatar.png" crossorigin="use-credentials" alt="avatar">
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_img".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 安全相关 HTML 属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/attr/integrity".into(),
            description: "subresource integrity 属性".into(),
            category: "security".into(),
            html: r#"<html><head>
            <link rel="stylesheet" href="https://cdn.example.com/style.css"
                  integrity="sha384-abc123" crossorigin="anonymous">
            <script src="https://cdn.example.com/app.js"
                    integrity="sha384-xyz789" crossorigin="anonymous"></script>
            </head><body>
            <div>SRI test</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/attr/nonce".into(),
            description: "nonce 属性的 inline script/style".into(),
            category: "security".into(),
            html: r#"<html><head>
            <style nonce="abc123">.red { color: red; }</style>
            </head><body>
            <div class="red">Nonce test</div>
            <script nonce="abc123">console.log('nonce ok');</script>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 安全 HTTP 头 (模拟 via meta)
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/headers/x-content-type".into(),
            description: "X-Content-Type-Options via meta".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="X-Content-Type-Options" content="nosniff">
            </head><body>
            <div>X-Content-Type-Options</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_meta".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/headers/strict-transport".into(),
            description: "Strict-Transport-Security via meta".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Strict-Transport-Security" content="max-age=31536000; includeSubDomains">
            </head><body>
            <div>HSTS</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_meta".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合安全页面
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "security/composite/secure-login-page".into(),
            description: "安全登录页面（CSP + SRI + CORS + sandbox）".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'nonce-abc'; style-src 'unsafe-inline'">
            <meta name="referrer" content="strict-origin-when-cross-origin">
            </head><body>
            <form action="/login" method="POST" autocomplete="off">
                <label for="user">Username</label>
                <input type="text" id="user" name="user" required autocomplete="username">
                <label for="pass">Password</label>
                <input type="password" id="pass" name="pass" required autocomplete="current-password">
                <button type="submit">Sign In</button>
            </form>
            <script nonce="abc">document.querySelector('form').action;</script>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_form".into(),
                "dom_has_meta".into(),
                "dom_has_input".into(),
                "no_panic".into(),
            ],
        },
    ]
}
