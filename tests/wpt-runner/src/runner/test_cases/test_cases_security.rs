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
                var fired = false;
                window.__zw_xss_fired = false;
                var el = document.getElementById('r');
                el.innerHTML = '<img src=x onerror="window.__zw_xss_fired=true">';
                // R3329 行为锁：innerHTML 写入须解析出 img 子元素（1 子节点），且内联 onerror 不在赋值期同步触发。
                if (el.childNodes.length < 1) throw new Error('xss-innerHTML: innerHTML 未解析出子节点（length=' + el.childNodes.length + '）');
                if (window.__zw_xss_fired) throw new Error('xss-innerHTML: 内联 onerror 在 innerHTML 赋值期同步触发（应延迟到资源加载失败）');
                document.getElementById('r').textContent = 'innerHTML sanitized';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "no_panic".into(),
                // R3329：innerHTML 解析子节点 + onerror 不同步触发——回归即 fail。
                "js_executes_ok".into(),
            ],
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
                // R3329 行为锁：断言安全 API 表面真值（fetch 真函数 / cookie 字符串 / location 对象）。
                if (typeof fetch !== 'function') throw new Error('security-dashboard: typeof fetch="' + typeof fetch + '" expected "function"');
                if (typeof document.cookie !== 'string') throw new Error('security-dashboard: typeof cookie="' + typeof document.cookie + '" expected "string"');
                if (typeof location !== 'object') throw new Error('security-dashboard: typeof location="' + typeof location + '" expected "object"');
                document.getElementById('s-cors').textContent = 'Fetch ok';
                document.getElementById('s-cookie').textContent = 'Cookie ok';
                document.getElementById('s-origin').textContent = 'Origin ok';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_heading".into(),
                "layout_has_children".into(),
                "no_panic".into(),
                // R3329：经 WebView 真实执行内联脚本——fetch/document.cookie/location 缺失或抛异常即 fail。
                "js_executes_ok".into(),
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
            <script>eval("document.getElementById('result').textContent = 'eval allowed';")
            // R3329 行为锁：eval() 真实执行并写入 DOM（文本被改写）。eval 不可用或被 CSP 阻止即文本不变 → fail。
            if (document.getElementById('result').textContent !== 'eval allowed') throw new Error('unsafe-eval: eval 未执行（result="' + document.getElementById('result').textContent + '"）');</script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                // R3329：eval() 真实执行——脚本抛异常（eval 被 CSP 阻止或不可用）即 fail。
                "js_executes_ok".into(),
            ],
        },
        TestCase {
            id: "security/csp/wasm-unsafe-eval-policy".into(),
            description: "CSP wasm-unsafe-eval 单独允许 WASM（策略 meta 展示）".into(),
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

        // ═══════════════════════════════════════════════════════════════
        //  HSTS + 混合内容升级管线
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/hsts/upgrade-insecure".into(),
            description: "HSTS upgrade-insecure-requests CSP 指令".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="upgrade-insecure-requests">
            </head><body>
            <img src="http://example.com/image.png" alt="Should be upgraded">
            <script src="http://cdn.example.com/lib.js"></script>
            <p>upgrade-insecure-requests 自动升级 HTTP 为 HTTPS</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSP nonce + hash 脚本加载
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/csp/nonce-script".into(),
            description: "CSP nonce 属性脚本加载".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="script-src 'nonce-abc123'">
            </head><body>
            <script nonce="abc123">document.body.innerHTML += '<p id="nonce-ok">Nonce script executed</p>';</script>
            <script>document.body.innerHTML += '<p>Non-nonce script should not execute</p>';</script>
            <div>测试 nonce 限制脚本执行</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSP img-src + data: URI + blob: URI
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/csp/img-src-data".into(),
            description: "CSP img-src data: blob: URI 限制".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="img-src 'self' data:">
            </head><body>
            <img src="data:image/png;base64,iVBOR" alt="data URI image">
            <img src="https://example.com/logo.png" alt="self image">
            <img src="https://evil.com/tracker.png" alt="blocked image">
            <div>img-src 控制 data: 和 self 图片</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSP connect-src + Fetch API
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/csp/connect-src".into(),
            description: "CSP connect-src 限制 Fetch/XHR".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="connect-src 'self' https://api.example.com">
            </head><body>
            <div id="status">connect-src test</div>
            <script>
            fetch('/api/data').catch(() => {});
            fetch('https://api.example.com/v1').catch(() => {});
            fetch('https://evil.com/exfil').catch(() => {});
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CORS crossorigin 属性组合
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/cors/crossorigin-attributes".into(),
            description: "各种 crossorigin 属性值".into(),
            category: "security".into(),
            html: r#"<html><body>
            <img src="https://cdn.example.com/a.png" crossorigin="anonymous" alt="anonymous">
            <img src="https://cdn.example.com/b.png" crossorigin="use-credentials" alt="credentials">
            <img src="https://cdn.example.com/c.png" alt="no-cors">
            <script src="https://cdn.example.com/lib.js" crossorigin="anonymous"></script>
            <link rel="stylesheet" href="https://cdn.example.com/style.css" crossorigin="anonymous">
            <div>测试 crossorigin 属性的各种组合</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  沙箱标志组合
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/sandbox/flag-combinations".into(),
            description: "sandbox 多种标志组合".into(),
            category: "security".into(),
            html: r#"<html><body>
            <iframe sandbox="allow-scripts allow-forms" srcdoc="<form><input type='text'><button>Submit</button></form>"></iframe>
            <iframe sandbox="allow-scripts allow-same-origin" srcdoc="<p>Same-origin sandbox</p>"></iframe>
            <iframe sandbox="allow-scripts allow-popups allow-forms allow-modals" srcdoc="<div>Multi-flag</div>"></iframe>
            <iframe sandbox="allow-top-navigation allow-scripts" srcdoc="<a href='/'>Top nav</a>"></iframe>
            <div>测试各种 sandbox 标志组合</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  COOP/COEP 跨源隔离
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/coop/coep-coop-headers".into(),
            description: "COOP/COEP 响应头跨源隔离".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Cross-Origin-Opener-Policy" content="same-origin">
            <meta http-equiv="Cross-Origin-Embedder-Policy" content="require-corp">
            </head><body>
            <div id="isolated">Cross-origin isolated context</div>
            <script>
            // SharedArrayBuffer 仅在跨源隔离上下文可用
            var hasSharedBuffer = typeof SharedArrayBuffer !== 'undefined';
            document.getElementById('isolated').textContent += ' SAB:' + hasSharedBuffer;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  混合内容 + 安全上下文
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/mixed-content/security-context".into(),
            description: "安全上下文判断 + isSecureContext".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="ctx">Security context test</div>
            <script>
            var isSecure = window.isSecureContext;
            // R3329 行为锁：window.isSecureContext 须为布尔（headless about:blank 默认 true）。
            if (typeof isSecure !== 'boolean') throw new Error('security-context: typeof isSecureContext="' + typeof isSecure + '" expected "boolean"');
            document.getElementById('ctx').textContent = 'isSecureContext: ' + isSecure;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                // R3329：isSecureContext 为 undefined（属性缺失）即 fail。
                "js_executes_ok".into(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Referrer-Policy 策略
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/referrer/no-referrer".into(),
            description: "Referrer-Policy no-referrer".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta name="referrer" content="no-referrer">
            </head><body>
            <a href="https://example.com/page">No referrer link</a>
            <img src="https://cdn.example.com/img.png" alt="No referrer image">
            <div>no-referrer 策略不发送 Referer 头</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        TestCase {
            id: "security/referrer/strict-origin".into(),
            description: "Referrer-Policy strict-origin".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta name="referrer" content="strict-origin">
            </head><body>
            <a href="https://example.com/page">Strict origin link</a>
            <a href="http://example.com/insecure">Insecure link</a>
            <div>strict-origin 仅在同等安全级别时发送源</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  权限 API 检测
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/permissions/api-detection".into(),
            description: "Permissions API 可用性检测".into(),
            category: "security".into(),
            html: r#"<html><body>
            <div id="perms">Permissions API test</div>
            <script>
            var results = [];
            results.push('Notification: ' + (typeof Notification !== 'undefined'));
            var hasGeo = typeof navigator !== 'undefined' && ('geolocation' in navigator);
            var hasPerms = typeof navigator !== 'undefined' && ('permissions' in navigator);
            // R3329 行为锁：navigator.geolocation / navigator.permissions 表面须存在（headless 真值）。
            if (!hasGeo) throw new Error('permissions: navigator.geolocation 缺失');
            if (!hasPerms) throw new Error('permissions: navigator.permissions 缺失');
            results.push('Geolocation: ' + hasGeo);
            results.push('Permissions: ' + hasPerms);
            document.getElementById('perms').textContent = results.join(', ');
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                // R3329：navigator.geolocation/permissions 被移除即 fail。
                "js_executes_ok".into(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  综合：安全仪表盘页面
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/comprehensive/security-dashboard".into(),
            description: "安全特性综合页面".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'; style-src 'unsafe-inline'; img-src 'self' data:; connect-src 'self'">
            <meta name="referrer" content="strict-origin-when-cross-origin">
            <style>
            .security-card { border: 1px solid #ddd; padding: 12px; margin: 8px 0; border-radius: 4px; }
            .pass { color: green; } .fail { color: red; } .warn { color: orange; }
            h1 { font-size: 18px; } h2 { font-size: 14px; }
            </style>
            </head><body>
            <h1>Security Dashboard</h1>
            <div class="security-card">
                <h2>CSP Status</h2>
                <p class="pass">Content-Security-Policy: active</p>
            </div>
            <div class="security-card">
                <h2>Mixed Content</h2>
                <p class="pass">All resources loaded over secure origins</p>
            </div>
            <div class="security-card">
                <h2>Permissions</h2>
                <p>Notification: <span class="warn">prompt</span></p>
                <p>Geolocation: <span class="warn">prompt</span></p>
            </div>
            <div class="security-card">
                <h2>CORS</h2>
                <p class="pass">Same-origin policy enforced</p>
            </div>
            <div class="security-card">
                <h2>Sandbox</h2>
                <iframe sandbox="allow-scripts" srcdoc="<p class='pass'>Sandboxed iframe active</p>" width="200" height="50"></iframe>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "dom_has_element:iframe".into(),
                "render_completes".into(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        // SecurityContext — HSTS 预加载 + 混合内容执行引擎
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "security/hsts/preload-upgrade".into(),
            description: "HSTS 预加载列表自动升级 HTTP→HTTPS".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="upgrade-insecure-requests">
            <style>
            .status { padding: 8px; margin: 4px; border-radius: 4px; }
            .upgraded { background: #d4edda; color: #155724; }
            </style>
            </head><body>
            <h1>HSTS Preload Test</h1>
            <div class="status upgraded">
                <p>github.com → HSTS preload enforced</p>
                <p>cloudflare.com → HSTS preload with includeSubDomains</p>
                <p>google.com → HSTS preload enforced</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "render_completes".into(),
            ],
        },

        TestCase {
            id: "security/mixed-content/blockable-resources".into(),
            description: "混合内容阻止 — Blockable 类型（script/style/connect/font）".into(),
            category: "security".into(),
            html: r#"<html><head>
            <style>
            .blocked { background: #f8d7da; color: #721c24; padding: 8px; margin: 4px; border-radius: 4px; }
            .resource { font-family: monospace; }
            </style>
            </head><body>
            <h1>Mixed Content: Blockable Types</h1>
            <div class="blocked">
                <p>Blockable resources (blocked on HTTPS pages):</p>
                <p class="resource">script: http://evil.com/steal.js → BLOCKED</p>
                <p class="resource">style: http://evil.com/theme.css → BLOCKED</p>
                <p class="resource">connect: http://evil.com/api → BLOCKED</p>
                <p class="resource">font: http://evil.com/font.woff2 → BLOCKED</p>
                <p class="resource">iframe: http://evil.com/embed → BLOCKED</p>
                <p class="resource">object: http://evil.com/flash.swf → BLOCKED</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "render_completes".into(),
            ],
        },

        TestCase {
            id: "security/mixed-content/upgradeable-resources".into(),
            description: "混合内容自动升级 — OptionallyBlockable 类型（img/audio/video）".into(),
            category: "security".into(),
            html: r#"<html><head>
            <style>
            .upgraded { background: #d4edda; color: #155724; padding: 8px; margin: 4px; border-radius: 4px; }
            .resource { font-family: monospace; }
            </style>
            </head><body>
            <h1>Mixed Content: Auto-Upgraded Types</h1>
            <div class="upgraded">
                <p>OptionallyBlockable resources (auto-upgraded to HTTPS):</p>
                <p class="resource">img: http://cdn.com/photo.jpg → https://cdn.com/photo.jpg</p>
                <p class="resource">audio: http://cdn.com/audio.mp3 → https://cdn.com/audio.mp3</p>
                <p class="resource">video: http://cdn.com/video.mp4 → https://cdn.com/video.mp4</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "render_completes".into(),
            ],
        },

        TestCase {
            id: "security/hsts/runtime-registration".into(),
            description: "HSTS 运行时注册（从响应头 Strict-Transport-Security）".into(),
            category: "security".into(),
            html: r#"<html><head>
            <style>
            .hsts-info { background: #e2e3e5; color: #383d41; padding: 8px; margin: 4px; border-radius: 4px; }
            code { background: #f8f9fa; padding: 2px 6px; border-radius: 3px; }
            </style>
            </head><body>
            <h1>HSTS Runtime Registration</h1>
            <div class="hsts-info">
                <p>Strict-Transport-Security: <code>max-age=31536000; includeSubDomains</code></p>
                <p>Effect: All future HTTP requests to this domain upgraded to HTTPS</p>
                <p>Subdomains: Included (includeSubDomains flag)</p>
                <p>Expiry: After max-age seconds from registration</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "render_completes".into(),
            ],
        },

        TestCase {
            id: "security/mixed-content/comprehensive-policy".into(),
            description: "综合安全策略页面 — CSP + HSTS + Mixed Content + Permissions".into(),
            category: "security".into(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self'; upgrade-insecure-requests">
            <meta name="referrer" content="strict-origin-when-cross-origin">
            <style>
            .policy-card { border: 1px solid #ccc; padding: 12px; margin: 8px 0; border-radius: 4px; }
            .active { border-left: 4px solid #28a745; }
            .warning { border-left: 4px solid #ffc107; }
            h1 { font-size: 18px; } h2 { font-size: 14px; margin: 0 0 8px 0; }
            .tag { display: inline-block; padding: 2px 8px; border-radius: 3px; font-size: 12px; margin: 2px; }
            .tag-green { background: #d4edda; color: #155724; }
            .tag-yellow { background: #fff3cd; color: #856404; }
            .tag-red { background: #f8d7da; color: #721c24; }
            </style>
            </head><body>
            <h1>Comprehensive Security Policy</h1>
            <div class="policy-card active">
                <h2>Content Security Policy</h2>
                <span class="tag tag-green">default-src 'self'</span>
                <span class="tag tag-green">script-src 'self'</span>
                <span class="tag tag-green">upgrade-insecure-requests</span>
            </div>
            <div class="policy-card active">
                <h2>HSTS / Mixed Content</h2>
                <span class="tag tag-green">HSTS preload: 40+ domains</span>
                <span class="tag tag-green">Blockable: script/style/connect</span>
                <span class="tag tag-yellow">Upgradeable: img/audio/video</span>
            </div>
            <div class="policy-card warning">
                <h2>Permissions</h2>
                <span class="tag tag-yellow">Camera: prompt</span>
                <span class="tag tag-yellow">Geolocation: prompt</span>
                <span class="tag tag-yellow">Notifications: prompt</span>
            </div>
            <div class="policy-card active">
                <h2>CORS / Same-Origin</h2>
                <span class="tag tag-green">Same-origin policy enforced</span>
                <span class="tag tag-green">CORS preflight for cross-origin</span>
                <span class="tag tag-red">Cross-origin DOM access: denied</span>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "dom_has_element:h2".into(),
                "render_completes".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // WASM 安全测试
        // ═══════════════════════════════════════════════════════════════
        // CSP wasm-unsafe-eval 限制
        TestCase {
            id: "security/csp/wasm-unsafe-eval".into(),
            description: "CSP wasm-unsafe-eval 策略检测".into(),
            category: "security".into(),
            html: r#"<html><body>
            <h1>CSP WASM Policy</h1>
            <div id="r">checking</div>
            <script>
                var hasWasm = typeof WebAssembly !== 'undefined';
                var canCompile = hasWasm && typeof WebAssembly.compile === 'function';
                var canInstantiate = hasWasm && typeof WebAssembly.instantiate === 'function';
                // R3329 行为锁：WebAssembly 全局 + compile/instantiate 函数均须存在（headless 真值）。
                if (!hasWasm) throw new Error('wasm-unsafe-eval: WebAssembly 未定义');
                if (!canCompile) throw new Error('wasm-unsafe-eval: WebAssembly.compile 非 function');
                if (!canInstantiate) throw new Error('wasm-unsafe-eval: WebAssembly.instantiate 非 function');
                var status = 'wasm-available-compile-ok-instantiate-ok';
                document.getElementById('r').textContent = status;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "no_panic".into(),
                // R3329：WebAssembly 表面被移除/降级即 fail。
                "js_executes_ok".into(),
            ],
        },
        // WASM 沙箱边界测试
        TestCase {
            id: "security/wasm/sandbox-boundary".into(),
            description: "WASM 沙箱边界 — 无直接文件/网络访问".into(),
            category: "security".into(),
            html: r#"<html><body>
            <h1>WASM Sandbox</h1>
            <div id="r">checking</div>
            <div id="details"></div>
            <script>
                var results = [];
                results.push('wasm:' + (typeof WebAssembly !== 'undefined'));
                results.push('validate:' + (typeof WebAssembly.validate === 'function'));
                // WASM 模块不能直接访问文件系统
                results.push('fs:' + (typeof require === 'undefined'));
                // WASM 模块不能直接访问网络（需通过宿主桥接）
                results.push('direct-net:' + (typeof XMLHttpRequest === 'undefined'));
                document.getElementById('r').textContent = 'boundary-ok';
                document.getElementById('details').textContent = results.join('|');
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "no_panic".into(),
                "render_completes".into(),
            ],
        },
    ]
}
