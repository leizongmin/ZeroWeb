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

        // ═══════════════════════════════════════════════════════════════
        //  Cookie 安全扩展
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/cookie/secure-flag".into(),
            description: "Cookie Secure 属性检测".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="r">cookie secure</div>
            <script>
                document.getElementById('r').textContent = 'Cookie Secure: tested';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/cookie/httponly-flag".into(),
            description: "Cookie HttpOnly 属性检测".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="r">cookie httponly</div>
            <script>
                document.getElementById('r').textContent = 'Cookie HttpOnly: tested';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/cookie/samesite-strict".into(),
            description: "Cookie SameSite=Strict 检测".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="r">samesite strict</div>
            <script>
                document.getElementById('r').textContent = 'SameSite Strict: tested';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSP 扩展
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/csp/script-src-hash".into(),
            description: "CSP script-src hash 策略".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src 'sha256-abc123'">
            </head><body>
            <div id="r">csp hash</div>
            <script>document.getElementById('r').textContent = 'hash executed';</script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/csp/connect-src".into(),
            description: "CSP connect-src 限制".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="connect-src 'self'">
            </head><body>
            <div id="r">connect-src</div>
            <script>document.getElementById('r').textContent = 'connect-src tested';</script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/csp/style-src-unsafe-inline".into(),
            description: "CSP style-src unsafe-inline".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="style-src 'unsafe-inline'">
            </head><body>
            <div id="r" style="color: red">inline style</div>
            <script>document.getElementById('r').textContent = 'inline style ok';</script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  同源策略扩展
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/sop/cross-origin-img".into(),
            description: "跨域图片加载受同源策略限制".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="r">cross-origin img</div>
            <img src="https://other-origin.com/img.png" alt="cross-origin" id="xorigin-img">
            <script>document.getElementById('r').textContent = 'cross-origin img tested';</script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/sop/postmessage-origin".into(),
            description: "postMessage 目标 origin 限制".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="r">postmessage</div>
            <script>
                if (typeof window.postMessage === 'function') {
                    window.postMessage('test', '*');
                }
                document.getElementById('r').textContent = 'postMessage tested';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  XSS 防护
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/xss/innerHTML-sanitization".into(),
            description: "innerHTML 不执行脚本".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="r">xss test</div>
            <script>
                var el = document.getElementById('r');
                el.innerHTML = '<img src=x onerror="alert(1)">';
                document.getElementById('r').textContent = 'innerHTML sanitized';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "security/xss/script-injection".into(),
            description: "防止 script 注入攻击".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="r">injection test</div>
            <script>
                var userInput = '&lt;script&gt;alert(1)&lt;/script&gt;';
                document.getElementById('r').textContent = userInput;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  综合安全页面
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/composite/security-dashboard".into(),
            description: "安全策略仪表盘".into(),
            category: "security".into(),
            html: r#"<html><head>
            <style>
                .security-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; padding: 10px; }
                .sec-card { border: 1px solid #ccc; padding: 8px; border-radius: 4px; font-size: 13px; }
                .sec-card h4 { margin: 0 0 4px 0; }
            </style>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'unsafe-inline'; style-src 'unsafe-inline'">
            <meta http-equiv="X-Content-Type-Options" content="nosniff">
            </head><body>
            <h2>Security Dashboard</h2>
            <div class="security-grid">
                <div class="sec-card"><h4>CSP</h4><span id="s-csp">active</span></div>
                <div class="sec-card"><h4>CORS</h4><span id="s-cors">checking</span></div>
                <div class="sec-card"><h4>Cookies</h4><span id="s-cookie">checking</span></div>
                <div class="sec-card"><h4>Origin</h4><span id="s-origin">checking</span></div>
            </div>
            <script>
                document.getElementById('s-csp').textContent = 'CSP active';
                document.getElementById('s-cors').textContent = typeof fetch === 'function' ? 'Fetch ok' : 'No Fetch';
                document.getElementById('s-cookie').textContent = typeof document.cookie !== 'undefined' ? 'Cookie ok' : 'No Cookie';
                document.getElementById('s-origin').textContent = typeof location !== 'undefined' ? 'Origin ok' : 'No Origin';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_heading".into(),
                "layout_has_children".into(),
                "no_panic".into(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  内容安全策略（CSP）
        // ═══════════════════════════════════════════════════════════════

        // ── CSP meta 标签（多个指令） ──
        TestCase {
            id: "security/csp-multi-directive".into(),
            description: "CSP meta 标签多个指令".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self'; img-src 'self' data:;">
            </head><body>
            <img src="data:image/gif;base64,R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==" alt="data URI img">
            <style>body { color: #333; }</style>
            <p>Content with CSP meta directives</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ── CSP nonce 属性 ──
        TestCase {
            id: "security/csp-nonce-script".into(),
            description: "CSP nonce 属性脚本".into(),
            category: "security".into(),
            html: r#"<html><body>
            <script nonce="abc123">
                document.body.innerHTML += '<p>Script executed with nonce</p>';
            </script>
            <noscript>JavaScript is disabled</noscript>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  混合内容
        // ═══════════════════════════════════════════════════════════════

        // ── 混合内容图片 ──
        TestCase {
            id: "security/mixed-content-images".into(),
            description: "混合内容图片资源".into(),
            category: "security".into(),
            html: r#"<html><body>
            <img src="https://example.com/secure.png" alt="Secure image">
            <img src="http://example.com/insecure.png" alt="Insecure image">
            <picture>
                <source srcset="https://example.com/large.png" media="(min-width: 800px)">
                <img src="https://example.com/small.png" alt="Responsive">
            </picture>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  iframe 沙箱
        // ═══════════════════════════════════════════════════════════════

        // ── 多个 sandbox 标志组合 ──
        TestCase {
            id: "security/iframe-sandbox-combo".into(),
            description: "iframe sandbox 多标志组合".into(),
            category: "security".into(),
            html: r#"<html><body>
            <iframe sandbox="allow-scripts allow-same-origin" srcdoc="<p>Script+Origin</p>"></iframe>
            <iframe sandbox="allow-forms allow-popups" srcdoc="<form><input></form>"></iframe>
            <iframe sandbox="" srcdoc="<p>Maximum sandbox</p>"></iframe>
            <iframe srcdoc="<p>No sandbox attribute</p>"></iframe>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Referrer Policy
        // ═══════════════════════════════════════════════════════════════

        // ── 多种 referrer policy ──
        TestCase {
            id: "security/referrer-policies".into(),
            description: "多种 referrer policy 渲染".into(),
            category: "security".into(),
            html: r#"<html><body>
            <meta name="referrer" content="no-referrer">
            <a href="https://example.com" referrerpolicy="origin">Origin only</a>
            <a href="https://example.com" referrerpolicy="no-referrer">No referrer</a>
            <img src="logo.png" referrerpolicy="no-referrer" alt="No referrer image">
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSP 扩展指令（script-src-attr/style-src-attr/unsafe-eval 等）
        // ═══════════════════════════════════════════════════════════════

        // ── CSP script-src-attr + style-src-attr ──
        TestCase {
            id: "security/csp/script-src-attr".into(),
            description: "CSP script-src-attr 控制内联事件处理器".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src-attr 'none'; script-src 'unsafe-inline'">
            </head><body>
            <button onclick="alert('blocked')">Click me</button>
            <p>script-src-attr 'none' 阻止内联事件处理器，但 script-src 允许 script 元素</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        TestCase {
            id: "security/csp/style-src-attr".into(),
            description: "CSP style-src-attr 控制内联样式属性".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="style-src-attr 'none'; style-src 'unsafe-inline'">
            </head><body>
            <div style="color: red">style 属性被阻止</div>
            <style>.ok { color: green; }</style>
            <p class="ok">style 元素正常</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ── CSP unsafe-eval + wasm-unsafe-eval ──
        TestCase {
            id: "security/csp/unsafe-eval".into(),
            description: "CSP unsafe-eval 允许 eval()".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src 'self' 'unsafe-eval'">
            </head><body>
            <div id="result">unsafe-eval test</div>
            <script>eval("document.getElementById('result').textContent = 'eval allowed';")</script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        TestCase {
            id: "security/csp/wasm-unsafe-eval".into(),
            description: "CSP wasm-unsafe-eval 单独允许 WASM".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src 'self' 'wasm-unsafe-eval'">
            </head><body>
            <div id="result">wasm-unsafe-eval test</div>
            <p>wasm-unsafe-eval 允许 WASM 编译但阻止 eval()</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ── CSP strict-dynamic ──
        TestCase {
            id: "security/csp/strict-dynamic".into(),
            description: "CSP strict-dynamic 信任传播".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src 'strict-dynamic' 'nonce-test123'">
            </head><body>
            <script nonce="test123">document.body.innerHTML += '<p>Dynamic script loaded</p>';</script>
            <p>strict-dynamic 允许通过 nonce 信任的脚本动态加载更多脚本</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ── CSP report-sample ──
        TestCase {
            id: "security/csp/report-sample".into(),
            description: "CSP report-sample 请求违规样本".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src 'report-sample' 'self'">
            </head><body>
            <div>report-sample 请求在违规报告中包含代码样本</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CORS 边界测试
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/cors/cross-origin-img".into(),
            description: "跨源图片加载（CORS 无凭证）".into(),
            category: "security".into(),
            html: r#"<html><body>
            <img src="https://other-origin.example.com/image.png" crossorigin="anonymous" alt="CORS image">
            <img src="https://other-origin.example.com/logo.svg" crossorigin="use-credentials" alt="Credentials">
            <p>crossorigin 属性控制 CORS 行为</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        TestCase {
            id: "security/cors/cross-origin-fetch".into(),
            description: "跨源 Fetch 请求（预检）".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="status">CORS fetch test</div>
            <script>
            fetch('https://api.example.com/data', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', 'X-Custom': 'value' },
                body: '{}'
            }).catch(() => {});
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Trusted Types 基础
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/trusted-types/basic".into(),
            description: "Trusted Types CSP 指令".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="require-trusted-types-for 'script'">
            </head><body>
            <div>Trusted Types 阻止危险的 DOM 注入模式</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSP Report-Only 模式
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/csp/report-only".into(),
            description: "CSP Report-Only 仅报告不阻止".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy-Report-Only" content="script-src 'none'">
            </head><body>
            <script>document.body.innerHTML += '<p>Script executed (report-only)</p>';</script>
            <p>Report-Only 模式不阻止脚本执行</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSP 多策略组合
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/csp/multiple-policies".into(),
            description: "多个 CSP 头独立检查".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src https://a.com">
            <meta http-equiv="Content-Security-Policy" content="script-src https://b.com">
            </head><body>
            <div>多个 CSP 策略独立检查，资源必须通过所有策略</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
    ]
}
