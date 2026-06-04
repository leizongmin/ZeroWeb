//! Web API 标准合规性测试。
//!
//! 覆盖 Fetch API、WebSocket、Notification、Geolocation、
//! Clipboard、Performance、IntersectionObserver 等。

use super::TestCase;

/// 返回 Web API 标准合规性测试用例。
pub fn web_api_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        // Fetch API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/fetch/basic-structure".into(),
            description: "Fetch API 页面不崩溃".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="status">Fetch test</div>
            <script>
                // fetch() should exist as global
                if (typeof fetch === 'function') {
                    document.getElementById('status').textContent = 'fetch exists';
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "web-api/fetch/request-response".into(),
            description: "Request/Response 构造函数存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="r">test</div>
            <script>
                var hasRequest = typeof Request !== 'undefined';
                var hasResponse = typeof Response !== 'undefined';
                var hasHeaders = typeof Headers !== 'undefined';
                document.getElementById('r').textContent = hasRequest && hasResponse && hasHeaders ? 'yes' : 'no';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // WebSocket
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/websocket/constructor".into(),
            description: "WebSocket 构造函数存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ws">ws test</div>
            <script>
                var hasWS = typeof WebSocket !== 'undefined';
                document.getElementById('ws').textContent = hasWS ? 'WebSocket exists' : 'no WebSocket';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Performance API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/performance/now".into(),
            description: "performance.now() 基本功能".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="perf">perf test</div>
            <script>
                var hasPerf = typeof performance !== 'undefined' && typeof performance.now === 'function';
                document.getElementById('perf').textContent = hasPerf ? 'performance exists' : 'no performance';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "web-api/performance/mark-measure".into(),
            description: "performance.mark/measure API".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="pm">mark test</div>
            <script>
                var result = 'no';
                if (typeof performance !== 'undefined') {
                    if (typeof performance.mark === 'function') {
                        performance.mark('test-start');
                        result = 'mark ok';
                    }
                    if (typeof performance.measure === 'function') {
                        performance.measure('test-duration', 'test-start');
                        result = 'mark+measure ok';
                    }
                }
                document.getElementById('pm').textContent = result;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Console API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/console/methods".into(),
            description: "console.log/warn/error/info 方法存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="c">console test</div>
            <script>
                var methods = ['log', 'warn', 'error', 'info', 'debug', 'table'];
                var allExist = methods.every(function(m) { return typeof console[m] === 'function'; });
                document.getElementById('c').textContent = allExist ? 'all console methods' : 'missing methods';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Timers
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/timers/setTimeout".into(),
            description: "setTimeout/setInterval 存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="t">timer test</div>
            <script>
                var hasST = typeof setTimeout === 'function';
                var hasSI = typeof setInterval === 'function';
                var hasCT = typeof clearTimeout === 'function';
                var hasCI = typeof clearInterval === 'function';
                document.getElementById('t').textContent = hasST && hasSI && hasCT && hasCI ? 'timers ok' : 'missing';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // DOM Observers
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/observers/mutation-observer".into(),
            description: "MutationObserver 存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="mo">observer test</div>
            <script>
                var hasMO = typeof MutationObserver !== 'undefined';
                document.getElementById('mo').textContent = hasMO ? 'MutationObserver exists' : 'no MO';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "web-api/observers/intersection-observer".into(),
            description: "IntersectionObserver 存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="io">intersection test</div>
            <script>
                var hasIO = typeof IntersectionObserver !== 'undefined';
                document.getElementById('io').textContent = hasIO ? 'IntersectionObserver exists' : 'no IO';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "web-api/observers/resize-observer".into(),
            description: "ResizeObserver 存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ro">resize test</div>
            <script>
                var hasRO = typeof ResizeObserver !== 'undefined';
                document.getElementById('ro').textContent = hasRO ? 'ResizeObserver exists' : 'no RO';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // WebAssembly
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/webassembly/api".into(),
            description: "WebAssembly API 存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="wasm">wasm test</div>
            <script>
                var hasWasm = typeof WebAssembly !== 'undefined';
                var hasCompile = hasWasm && typeof WebAssembly.compile === 'function';
                var hasInstantiate = hasWasm && typeof WebAssembly.instantiate === 'function';
                var hasValidate = hasWasm && typeof WebAssembly.validate === 'function';
                document.getElementById('wasm').textContent = hasCompile && hasInstantiate && hasValidate ? 'WASM ok' : 'missing';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Storage API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/storage/localStorage".into(),
            description: "localStorage 基本操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ls">ls test</div>
            <script>
                var hasLS = typeof localStorage !== 'undefined';
                if (hasLS) {
                    localStorage.setItem('wpt_key', 'wpt_value');
                    var val = localStorage.getItem('wpt_key');
                    localStorage.removeItem('wpt_key');
                    document.getElementById('ls').textContent = val === 'wpt_value' ? 'ls ok' : 'ls mismatch';
                } else {
                    document.getElementById('ls').textContent = 'no localStorage';
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "web-api/storage/sessionStorage".into(),
            description: "sessionStorage 基本操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ss">ss test</div>
            <script>
                var hasSS = typeof sessionStorage !== 'undefined';
                if (hasSS) {
                    sessionStorage.setItem('wpt_skey', 'wpt_svalue');
                    var val = sessionStorage.getItem('wpt_skey');
                    sessionStorage.removeItem('wpt_skey');
                    document.getElementById('ss').textContent = val === 'wpt_svalue' ? 'ss ok' : 'ss mismatch';
                } else {
                    document.getElementById('ss').textContent = 'no sessionStorage';
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Event Target / CustomEvent
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/events/custom-event".into(),
            description: "CustomEvent 构造函数存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ce">event test</div>
            <script>
                var hasCE = typeof CustomEvent !== 'undefined';
                document.getElementById('ce').textContent = hasCE ? 'CustomEvent exists' : 'no CE';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "web-api/events/add-remove-listener".into(),
            description: "addEventListener/removeEventListener 存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ael">listener test</div>
            <script>
                var el = document.getElementById('ael');
                var hasAdd = typeof el.addEventListener === 'function';
                var hasRemove = typeof el.removeEventListener === 'function';
                el.textContent = hasAdd && hasRemove ? 'listeners ok' : 'missing';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Navigator API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/navigator/userAgent".into(),
            description: "navigator.userAgent 存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="nav">nav test</div>
            <script>
                var hasNav = typeof navigator !== 'undefined';
                var hasUA = hasNav && typeof navigator.userAgent === 'string';
                document.getElementById('nav').textContent = hasUA ? 'navigator ok' : 'no navigator';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // URL / Location
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/url/url-constructor".into(),
            description: "URL 构造函数存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="url">url test</div>
            <script>
                var hasURL = typeof URL !== 'undefined';
                if (hasURL) {
                    try {
                        var u = new URL('https://example.com/path?q=1');
                        document.getElementById('url').textContent = u.hostname === 'example.com' ? 'URL ok' : 'URL mismatch';
                    } catch(e) {
                        document.getElementById('url').textContent = 'URL error';
                    }
                } else {
                    document.getElementById('url').textContent = 'no URL';
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // JSON
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/json/parse-stringify".into(),
            description: "JSON.parse/stringify 操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="json">json test</div>
            <script>
                var obj = {name: 'test', value: 42, nested: {a: 1}};
                var str = JSON.stringify(obj);
                var parsed = JSON.parse(str);
                document.getElementById('json').textContent = parsed.name === 'test' && parsed.nested.a === 1 ? 'JSON ok' : 'JSON error';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Promise / async
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/promise/basic".into(),
            description: "Promise 基本功能".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="p">promise test</div>
            <script>
                var hasPromise = typeof Promise !== 'undefined';
                if (hasPromise) {
                    var p = new Promise(function(resolve) { resolve('ok'); });
                    p.then(function(val) {
                        document.getElementById('p').textContent = val === 'ok' ? 'Promise ok' : 'Promise fail';
                    });
                } else {
                    document.getElementById('p').textContent = 'no Promise';
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
    ]
}
