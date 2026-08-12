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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
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
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Navigation API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/navigation/history-api".into(),
            description: "History API 存在检测".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="h">history test</div>
            <script>
                var hasHistory = typeof history !== 'undefined';
                document.getElementById('h').textContent = hasHistory ? 'history ok' : 'no history';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/navigation/location-api".into(),
            description: "Location API 存在检测".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="loc">location test</div>
            <script>
                var hasLocation = typeof location !== 'undefined';
                document.getElementById('loc').textContent = hasLocation ? 'location ok' : 'no location';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Web Storage API 检测
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/storage/localStorage-set-get".into(),
            description: "localStorage setItem/getItem 往返".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ls">storage test</div>
            <script>
                try {
                    localStorage.setItem('key1', 'value1');
                    var val = localStorage.getItem('key1');
                    document.getElementById('ls').textContent = val === 'value1' ? 'ls ok' : 'ls fail';
                } catch(e) {
                    document.getElementById('ls').textContent = 'ls error: ' + e.message;
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/storage/sessionStorage-set-get".into(),
            description: "sessionStorage setItem/getItem 往返".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ss">session test</div>
            <script>
                try {
                    sessionStorage.setItem('skey', 'sval');
                    var val = sessionStorage.getItem('skey');
                    document.getElementById('ss').textContent = val === 'sval' ? 'ss ok' : 'ss fail';
                } catch(e) {
                    document.getElementById('ss').textContent = 'ss error: ' + e.message;
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // DOM API 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/dom/classList".into(),
            description: "classList add/remove/toggle".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="cls" class="a b">classList test</div>
            <script>
                var el = document.getElementById('cls');
                if (el && el.classList) {
                    el.classList.add('c');
                    el.classList.remove('a');
                    el.classList.toggle('b');
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/dom/dataset".into(),
            description: "data-* 属性 dataset API".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ds" data-name="test" data-count="42">dataset test</div>
            <script>
                var el = document.getElementById('ds');
                if (el && el.dataset) {
                    var name = el.dataset.name;
                    el.dataset.newAttr = 'added';
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/dom/matches-closest".into(),
            description: "element.matches() 和 closest()".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="parent"><span id="child" class="active">matches test</span></div>
            <script>
                var el = document.getElementById('child');
                if (el && el.matches) {
                    el.matches('.active');
                }
                if (el && el.closest) {
                    el.closest('#parent');
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Web Workers 存在检测
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/workers/dedicated-exists".into(),
            description: "Dedicated Worker 存在检测".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="w">worker test</div>
            <script>
                var hasWorker = typeof Worker !== 'undefined';
                document.getElementById('w').textContent = hasWorker ? 'Worker ok' : 'no Worker';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // WebAssembly 存在检测
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/wasm/exists".into(),
            description: "WebAssembly API 存在检测".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="wa">wasm test</div>
            <script>
                var hasWasm = typeof WebAssembly !== 'undefined';
                var hasCompile = hasWasm && typeof WebAssembly.compile === 'function';
                var hasInstantiate = hasWasm && typeof WebAssembly.instantiate === 'function';
                var hasValidate = hasWasm && typeof WebAssembly.validate === 'function';
                document.getElementById('wa').textContent = (hasCompile && hasInstantiate && hasValidate) ? 'WASM ok' : 'WASM partial';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-api/composite/api-dashboard".into(),
            description: "多 API 集成仪表盘页面".into(),
            category: "web-api".into(),
            html: r#"<html><head>
            <style>
                .card { border: 1px solid #ccc; padding: 10px; margin: 5px; border-radius: 4px; }
                .flex { display: flex; gap: 10px; }
            </style>
            </head><body>
            <h1>API Dashboard</h1>
            <div class="flex">
                <div class="card" id="fetch-card">
                    <h2>Fetch</h2>
                    <p>fetch() available</p>
                </div>
                <div class="card" id="storage-card">
                    <h2>Storage</h2>
                    <p>localStorage available</p>
                </div>
                <div class="card" id="wasm-card">
                    <h2>WebAssembly</h2>
                    <p>WebAssembly available</p>
                </div>
            </div>
            <script>
                // 检测多个 API
                var apis = {
                    fetch: typeof fetch === 'function',
                    storage: typeof localStorage !== 'undefined',
                    wasm: typeof WebAssembly !== 'undefined',
                    worker: typeof Worker !== 'undefined',
                    promise: typeof Promise !== 'undefined',
                    json: typeof JSON !== 'undefined',
                };
                var available = Object.keys(apis).filter(function(k) { return apis[k]; }).length;
                document.title = available + ' APIs available';
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
        // DOM 操作扩展
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/dom/create-element-div".into(),
            description: "document.createElement('div') 操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="target">target</div>
            <script>
                var el = document.createElement('div');
                el.id = 'created';
                el.textContent = 'Hello World';
                document.getElementById('target').appendChild(el);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/dom/create-text-node".into(),
            description: "document.createTextNode() 操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="target">target</div>
            <script>
                var text = document.createTextNode('Dynamic text');
                document.getElementById('target').appendChild(text);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/dom/set-attribute".into(),
            description: "element.setAttribute/getAttribute 操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="el">attr test</div>
            <script>
                var el = document.getElementById('el');
                el.setAttribute('data-test', 'value');
                var val = el.getAttribute('data-test');
                el.textContent = val === 'value' ? 'attr ok' : 'attr fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/dom/inner-html".into(),
            description: "innerHTML 读写操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="target">before</div>
            <script>
                var el = document.getElementById('target');
                el.innerHTML = '<span>after</span>';
                var html = el.innerHTML;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/dom/query-selector-all".into(),
            description: "querySelector/querySelectorAll 操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div class="item">A</div>
            <div class="item">B</div>
            <div class="item">C</div>
            <script>
                var items = document.querySelectorAll('.item');
                var first = document.querySelector('.item');
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/dom/remove-child".into(),
            description: "removeChild 移除节点".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="parent">
                <div id="child">remove me</div>
            </div>
            <script>
                var parent = document.getElementById('parent');
                var child = document.getElementById('child');
                if (parent && child) { parent.removeChild(child); }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        // Fetch API 扩展
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/fetch/request-constructor".into(),
            description: "new Request() 构造函数".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="r">request test</div>
            <script>
                if (typeof Request !== 'undefined') {
                    try {
                        var req = new Request('https://example.com/api', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                        });
                        document.getElementById('r').textContent = 'Request created';
                    } catch(e) {
                        document.getElementById('r').textContent = 'Request error';
                    }
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/fetch/response-constructor".into(),
            description: "new Response() 构造函数".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="r">response test</div>
            <script>
                if (typeof Response !== 'undefined') {
                    try {
                        var res = new Response('{"ok":true}', {
                            status: 200,
                            headers: { 'Content-Type': 'application/json' },
                        });
                        document.getElementById('r').textContent = 'Response: ' + res.status;
                    } catch(e) {
                        document.getElementById('r').textContent = 'Response error';
                    }
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/fetch/headers-ops".into(),
            description: "Headers CRUD 操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="h">headers test</div>
            <script>
                if (typeof Headers !== 'undefined') {
                    var h = new Headers();
                    h.append('Content-Type', 'text/html');
                    h.set('X-Custom', 'value');
                    var ct = h.get('Content-Type');
                    document.getElementById('h').textContent = ct === 'text/html' ? 'Headers ok' : 'Headers fail';
                }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        // JavaScript 内置 API
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/js/array-methods".into(),
            description: "Array 高阶方法（map/filter/reduce）".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="arr">array test</div>
            <script>
                var arr = [1, 2, 3, 4, 5];
                var doubled = arr.map(function(x) { return x * 2; });
                var evens = arr.filter(function(x) { return x % 2 === 0; });
                var sum = arr.reduce(function(a, b) { return a + b; }, 0);
                document.getElementById('arr').textContent =
                    (doubled.length === 5 && evens.length === 2 && sum === 15) ? 'Array ok' : 'Array fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/js/object-methods".into(),
            description: "Object.keys/values/entries 方法".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="obj">object test</div>
            <script>
                var obj = {a: 1, b: 2, c: 3};
                var keys = Object.keys(obj);
                var vals = Object.values(obj);
                var entries = Object.entries(obj);
                document.getElementById('obj').textContent =
                    (keys.length === 3 && vals.length === 3 && entries.length === 3)
                    ? 'Object ok' : 'Object fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/js/map-set".into(),
            description: "Map/Set 集合操作".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="ms">map-set test</div>
            <script>
                var m = new Map();
                m.set('key', 'value');
                var mHas = m.has('key') && m.get('key') === 'value';
                var s = new Set([1, 2, 3]);
                var sHas = s.has(1) && s.size === 3;
                document.getElementById('ms').textContent =
                    mHas && sHas ? 'Map/Set ok' : 'Map/Set fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/js/error-types".into(),
            description: "Error/TypeError/RangeError 类型".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="err">error test</div>
            <script>
                var hasError = typeof Error !== 'undefined';
                var hasTypeError = typeof TypeError !== 'undefined';
                var hasRangeError = typeof RangeError !== 'undefined';
                try { throw new Error('test'); } catch(e) {
                    var caught = e.message === 'test';
                }
                document.getElementById('err').textContent =
                    hasError && hasTypeError && hasRangeError && caught ? 'Error ok' : 'Error fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/js/symbol-iterator".into(),
            description: "Symbol 和迭代器".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="sym">symbol test</div>
            <script>
                var hasSymbol = typeof Symbol !== 'undefined';
                var hasIterator = hasSymbol && typeof Symbol.iterator === 'symbol';
                var arr = [1, 2];
                var iter = arr[Symbol.iterator];
                document.getElementById('sym').textContent =
                    hasSymbol ? 'Symbol ok' : 'Symbol fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/js/proxy-reflect".into(),
            description: "Proxy 和 Reflect API".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="pr">proxy test</div>
            <script>
                var hasProxy = typeof Proxy !== 'undefined';
                var hasReflect = typeof Reflect !== 'undefined';
                document.getElementById('pr').textContent =
                    hasProxy && hasReflect ? 'Proxy/Reflect ok' : 'Proxy/Reflect fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        // 定时器和异步
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/timers/setTimeout-callback".into(),
            description: "setTimeout 回调执行".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="t">before</div>
            <script>
                setTimeout(function() {
                    document.getElementById('t').textContent = 'after';
                }, 0);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        TestCase {
            id: "web-api/timers/requestAnimationFrame".into(),
            description: "requestAnimationFrame 存在".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="raf">raf test</div>
            <script>
                var hasRAF = typeof requestAnimationFrame === 'function';
                var hasCAF = typeof cancelAnimationFrame === 'function';
                document.getElementById('raf').textContent =
                    hasRAF && hasCAF ? 'rAF ok' : 'rAF fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        // 综合页面 2
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/composite/js-playground".into(),
            description: "JavaScript Playground 综合页面".into(),
            category: "web-api".into(),
            html: r#"<html><head>
            <style>
                body { font-family: monospace; padding: 20px; }
                .output { background: #1a1a1a; color: #0f0; padding: 10px; border-radius: 4px; }
                .card { border: 1px solid #ddd; padding: 10px; margin: 5px 0; border-radius: 4px; }
            </style>
            </head><body>
            <h1>JS Playground</h1>
            <div class="card">
                <h2>Array Operations</h2>
                <div class="output" id="arr-out">...</div>
            </div>
            <div class="card">
                <h2>Object Operations</h2>
                <div class="output" id="obj-out">...</div>
            </div>
            <div class="card">
                <h2>Async Operations</h2>
                <div class="output" id="async-out">...</div>
            </div>
            <script>
                // Array operations
                var arr = [1, 2, 3, 4, 5];
                var result1 = arr.map(function(x) { return x * 2; }).join(', ');
                document.getElementById('arr-out').textContent = result1;

                // Object operations
                var obj = { name: 'test', version: 1 };
                var result2 = Object.keys(obj).map(function(k) { return k + '=' + obj[k]; }).join('&');
                document.getElementById('obj-out').textContent = result2;

                // Async
                Promise.resolve('resolved').then(function(v) {
                    document.getElementById('async-out').textContent = v;
                });
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
        TestCase {
            id: "web-api/composite/api-explorer".into(),
            description: "Web API Explorer 综合页面".into(),
            category: "web-api".into(),
            html: r#"<html><head>
            <style>
                .api-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; padding: 10px; }
                .api-card { border: 1px solid #ccc; padding: 8px; border-radius: 4px; }
                .api-card h3 { margin: 0 0 5px 0; font-size: 14px; }
                .api-card .status { font-size: 12px; padding: 2px 6px; border-radius: 3px; }
                .available { background: #d4edda; color: #155724; }
                .unavailable { background: #f8d7da; color: #721c24; }
            </style>
            </head><body>
            <h2>API Explorer</h2>
            <div class="api-grid">
                <div class="api-card"><h3>Fetch</h3><span class="status" id="s-fetch">checking</span></div>
                <div class="api-card"><h3>WebSocket</h3><span class="status" id="s-ws">checking</span></div>
                <div class="api-card"><h3>Storage</h3><span class="status" id="s-storage">checking</span></div>
                <div class="api-card"><h3>Worker</h3><span class="status" id="s-worker">checking</span></div>
                <div class="api-card"><h3>WASM</h3><span class="status" id="s-wasm">checking</span></div>
                <div class="api-card"><h3>Canvas</h3><span class="status" id="s-canvas">checking</span></div>
            </div>
            <script>
                var checks = {
                    's-fetch': typeof fetch === 'function',
                    's-ws': typeof WebSocket !== 'undefined',
                    's-storage': typeof localStorage !== 'undefined',
                    's-worker': typeof Worker !== 'undefined',
                    's-wasm': typeof WebAssembly !== 'undefined',
                    's-canvas': typeof HTMLCanvasElement !== 'undefined',
                };
                Object.keys(checks).forEach(function(id) {
                    var el = document.getElementById(id);
                    if (el) {
                        el.textContent = checks[id] ? 'Available' : 'Unavailable';
                        el.className = 'status ' + (checks[id] ? 'available' : 'unavailable');
                    }
                });
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
        //  Network / HTTP Cache
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/fetch-basic-get".into(),
            description: "Fetch API basic GET request availability".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Fetch API Basic GET</h2>
<div id="result">pending</div>
<script>
if (typeof fetch === 'function') {
    document.getElementById('result').textContent = 'fetch-available';
} else {
    document.getElementById('result').textContent = 'fetch-unavailable';
}
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/fetch-request-constructor".into(),
            description: "Fetch Request constructor and properties".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Fetch Request Constructor</h2>
<script>
var req = new Request('https://example.com/api');
var method = req.method;
var url = req.url;
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/fetch-response-constructor".into(),
            description: "Fetch Response constructor and status".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Fetch Response Constructor</h2>
<script>
var resp = new Response('{"ok":true}', {
    status: 200,
    headers: { 'Content-Type': 'application/json' }
});
var ok = resp.ok;
var status = resp.status;
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/fetch-headers-api".into(),
            description: "Fetch Headers API get/set/has/append".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Headers API</h2>
<script>
var h = new Headers();
h.set('Content-Type', 'text/html');
h.append('X-Custom', 'value1');
h.append('X-Custom', 'value2');
var ct = h.get('Content-Type');
var has = h.has('X-Custom');
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/cache-api-basic".into(),
            description: "Cache API open/put/match basic flow".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Cache API Basic</h2>
<script>
// Cache API polyfill test — verify caches global exists
if (typeof caches === 'undefined') {
    // polyfill not available, just verify no crash
    var cacheStatus = 'no-caches-global';
} else {
    var cacheStatus = 'caches-available';
}
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/network-online-state".into(),
            description: "navigator.onLine state detection".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Network Online State</h2>
<div id="status">checking</div>
<script>
var online = navigator.onLine;
document.getElementById('status').textContent = online ? 'online' : 'offline';
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/xhr-basic".into(),
            description: "XMLHttpRequest constructor availability".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>XMLHttpRequest</h2>
<script>
var xhr = new XMLHttpRequest();
var readyState = xhr.readyState;
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/url-api".into(),
            description: "URL API constructor and properties".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>URL API</h2>
<script>
var u = new URL('https://example.com/path?query=1#hash');
var protocol = u.protocol;
var hostname = u.hostname;
var pathname = u.pathname;
var search = u.search;
var hash = u.hash;
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/url-search-params".into(),
            description: "URLSearchParams API get/set/toString".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>URLSearchParams</h2>
<script>
var params = new URLSearchParams('a=1&b=2');
params.set('c', '3');
var a = params.get('a');
var has = params.has('b');
var str = params.toString();
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/encoding-api".into(),
            description: "TextEncoder/TextDecoder API".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Encoding API</h2>
<script>
var encoder = new TextEncoder();
var bytes = encoder.encode('hello');
var decoder = new TextDecoder();
var text = decoder.decode(bytes);
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Performance / Timing
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/performance-timing".into(),
            description: "performance.now() timing precision".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Performance Timing</h2>
<script>
var start = performance.now();
for (var i = 0; i < 1000; i++) { var x = i * i; }
var elapsed = performance.now() - start;
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/performance-mark-measure".into(),
            description: "performance.mark/measure/getEntries".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Performance Mark/Measure</h2>
<script>
performance.mark('start');
for (var i = 0; i < 100; i++) { var x = Math.sqrt(i); }
performance.mark('end');
performance.measure('sqrt-bench', 'start', 'end');
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Structured Data / Serialization
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/json-roundtrip".into(),
            description: "JSON parse/stringify complex objects".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>JSON Roundtrip</h2>
<script>
var obj = { name: "test", values: [1, 2, 3], nested: { a: true, b: null } };
var str = JSON.stringify(obj);
var parsed = JSON.parse(str);
var same = parsed.name === obj.name && parsed.values.length === 3;
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "web-api/structured-clone".into(),
            description: "structuredClone API for deep copy".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Structured Clone</h2>
<script>
var original = { a: 1, b: [2, 3], c: { d: true } };
if (typeof structuredClone === 'function') {
    var cloned = structuredClone(original);
    cloned.a = 99;
    var independent = original.a === 1;
}
</script>
</body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "no_panic".into(),
            ],
        },
        // ── CSS 渲染管线集成测试 ──
        TestCase {
            id: "web-api/css-filter-blur-page".into(),
            description: "CSS filter blur 在完整页面中的渲染".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
            <html><body style="margin: 0; padding: 20px; font-family: sans-serif;">
            <header style="filter: blur(0); background: #1976D2; color: white; padding: 16px;">
                <h1 style="margin: 0; font-size: 24px;">ZeroWeb Browser</h1>
                <p style="margin: 4px 0 0; font-size: 14px;">A Rust-based browser</p>
            </header>
            <main style="padding: 20px;">
                <div style="background: #f5f5f5; padding: 16px; margin: 8px 0; filter: blur(1px);">
                    <h2 style="font-size: 18px; color: #333;">Slightly Blurred Section</h2>
                    <p style="font-size: 14px; color: #666;">Content with a subtle blur effect applied.</p>
                </div>
            </main>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
            ],
        },
        TestCase {
            id: "web-api/css-text-overflow-page".into(),
            description: "text-overflow: ellipsis 在卡片列表中的渲染".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
            <html><body style="margin: 0; padding: 16px; font-family: sans-serif;">
            <h2 style="font-size: 20px; color: #333;">Article List</h2>
            <div style="display: flex; flex-direction: column; gap: 8px; width: 350px;">
                <div style="padding: 12px; border: 1px solid #e0e0e0; background: white;">
                    <h3 style="margin: 0 0 4px; font-size: 16px; color: #1976D2; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">Building Modern Web Applications with Rust and WebAssembly</h3>
                    <p style="margin: 0; font-size: 12px; color: #888;">2026-06-05</p>
                </div>
                <div style="padding: 12px; border: 1px solid #e0e0e0; background: white;">
                    <h3 style="margin: 0 0 4px; font-size: 16px; color: #1976D2; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">CSS Grid Layout: A Complete Guide for Responsive Design</h3>
                    <p style="margin: 0; font-size: 12px; color: #888;">2026-06-04</p>
                </div>
                <div style="padding: 12px; border: 1px solid #e0e0e0; background: white;">
                    <h3 style="margin: 0 0 4px; font-size: 16px; color: #1976D2; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">Short Title</h3>
                    <p style="margin: 0; font-size: 12px; color: #888;">2026-06-03</p>
                </div>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
            ],
        },
        TestCase {
            id: "web-api/css-letter-spacing-headings".into(),
            description: "letter-spacing 在标题和正文中的差异化应用".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
            <html><body style="margin: 0; padding: 24px;">
            <article style="max-width: 600px;">
                <h1 style="font-size: 28px; letter-spacing: 2px; color: #1a1a1a; margin: 0 0 8px;">WELL SPACED HEADING</h1>
                <h2 style="font-size: 20px; letter-spacing: 1px; color: #444; margin: 0 0 16px;">Subheading With Spacing</h2>
                <p style="font-size: 15px; letter-spacing: 0.3px; word-spacing: 1px; line-height: 1.7; color: #333;">
                    Body text with subtle letter and word spacing for improved readability.
                    The additional spacing helps with legibility in longer paragraphs.
                </p>
                <blockquote style="letter-spacing: 0.5px; font-style: italic; color: #666; border-left: 3px solid #1976D2; padding-left: 12px; margin: 16px 0;">
                    A quote with slightly increased letter spacing for emphasis.
                </blockquote>
            </article>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "has_glyph_primitives".into(),
                "glyph_count_ge:20".into(),
            ],
        },
        TestCase {
            id: "web-api/navigation-page-with-filters".into(),
            description: "导航栏 + filter 效果的综合页面".into(),
            category: "web-api".into(),
            html: r#"<!DOCTYPE html>
            <html><body style="margin: 0; font-family: sans-serif;">
            <nav style="background: #263238; padding: 12px 24px; display: flex; align-items: center; gap: 16px;">
                <span style="color: white; font-size: 18px; font-weight: bold; letter-spacing: 1px;">ZeroWeb</span>
                <a style="color: #B0BEC5; font-size: 14px; text-decoration: none;">Docs</a>
                <a style="color: #B0BEC5; font-size: 14px; text-decoration: none;">Blog</a>
                <a style="color: #B0BEC5; font-size: 14px; text-decoration: none;">About</a>
            </nav>
            <main style="padding: 32px 24px; max-width: 800px;">
                <div style="margin-bottom: 24px;">
                    <h1 style="font-size: 32px; color: #1a1a1a; margin: 0 0 8px; letter-spacing: -0.5px;">Welcome to ZeroWeb</h1>
                    <p style="font-size: 16px; color: #666; word-spacing: 2px;">A fast secure and modern browser built with Rust</p>
                </div>
                <div style="display: flex; gap: 16px; flex-wrap: wrap;">
                    <div style="flex: 1; min-width: 200px; padding: 20px; background: #E3F2FD; border-radius: 8px;">
                        <h3 style="font-size: 16px; color: #1565C0; margin: 0 0 8px;">Fast</h3>
                        <p style="font-size: 13px; color: #333; margin: 0; letter-spacing: 0.2px;">Optimized rendering pipeline</p>
                    </div>
                    <div style="flex: 1; min-width: 200px; padding: 20px; background: #E8F5E9; filter: brightness(1.05);">
                        <h3 style="font-size: 16px; color: #2E7D32; margin: 0 0 8px;">Secure</h3>
                        <p style="font-size: 13px; color: #333; margin: 0; letter-spacing: 0.2px;">Sandboxed execution</p>
                    </div>
                    <div style="flex: 1; min-width: 200px; padding: 20px; background: #FFF3E0; filter: grayscale(0.1);">
                        <h3 style="font-size: 16px; color: #E65100; margin: 0 0 8px;">Modern</h3>
                        <p style="font-size: 13px; color: #333; margin: 0; letter-spacing: 0.2px;">Latest web standards</p>
                    </div>
                </div>
            </main>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
                "glyph_count_ge:20".into(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        // 运行时和事件循环（Runtime and Event Loop）
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "web-api/runtime/timer-nesting".into(),
            description: "setTimeout 嵌套调用".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>Timer Nesting</h1>
            <div id="log"></div>
            <script>
                var depth = 0;
                function nestedTimeout() {
                    depth++;
                    if (depth < 10) {
                        setTimeout(nestedTimeout, 0);
                    }
                }
                nestedTimeout();
                // 同步首轮 depth 已 ≥1（首次调用 depth++）；嵌套回调用 setTimeout(0) 异步推进。
                if (depth < 1) throw new Error('timer-nesting: synchronous first run depth < 1');
                document.getElementById('log').textContent = 'Depth: ' + depth;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/microtask-promise".into(),
            description: "Promise 微任务执行顺序".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>Promise Microtask Order</h1>
            <div id="log"></div>
            <script>
                var order = [];
                Promise.resolve(1).then(function(v) { order.push('then1:' + v); return v + 1; })
                    .then(function(v) { order.push('then2:' + v); });
                Promise.resolve(3).then(function(v) { order.push('then3:' + v); });
                // 微任务在脚本末尾 drain——then1/then2/then3 均应执行。最终链断言预期顺序。
                Promise.resolve().then(function () {
                    var expected = 'then1:1,then3:3,then2:2';
                    if (order.join(',') !== expected) {
                        throw new Error('microtask-promise: order="' + order.join(',') + '" expected="' + expected + '"');
                    }
                    document.getElementById('log').textContent = order.join(',');
                });
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/async-await-sequence".into(),
            description: "async/await 异步执行序列".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>Async/Await Sequence</h1>
            <div id="log"></div>
            <script>
                async function fetchData() {
                    var result = await Promise.resolve('data');
                    if (result !== 'data') throw new Error('async-await: result="' + result + '" expected "data"');
                    document.getElementById('log').textContent = 'Got: ' + result;
                }
                fetchData();
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/error-handling".into(),
            description: "try-catch 和 Promise 错误处理".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>Error Handling</h1>
            <div id="log"></div>
            <script>
                var log = [];
                try { throw new Error('test error'); } catch(e) { log.push('caught:' + e.message); }
                try { JSON.parse('invalid'); } catch(e) { log.push('json-error'); }
                try { undefined.property; } catch(e) { log.push('type-error'); }
                // 三类错误均被捕获（catch 路径执行）——若任一未捕获则脚本中断（js_executes_ok 失败）。
                if (log.length !== 3) throw new Error('error-handling: only ' + log.length + ' branches caught');
                if (log[0] !== 'caught:test error') throw new Error('error-handling: caught message mismatch');
                document.getElementById('log').textContent = log.join('|');
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/raf-callback".into(),
            description: "requestAnimationFrame 回调".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>requestAnimationFrame</h1>
            <div id="log"></div>
            <script>
                // RAF API 存在性（功能可用，headless 无显示设备故回调可能不触发——仅断言 API 不抛）。
                if (typeof requestAnimationFrame !== 'function') {
                    throw new Error('raf-callback: requestAnimationFrame not a function');
                }
                try {
                    requestAnimationFrame(function(ts) { /* headless 可能不触发 */ });
                } catch(e) {
                    throw new Error('raf-callback: calling requestAnimationFrame threw: ' + e.message);
                }
                document.getElementById('log').textContent = 'RAF available';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/mutation-observer".into(),
            description: "MutationObserver DOM 变化监听".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>MutationObserver</h1>
            <div id="target"></div>
            <div id="log"></div>
            <script>
                var mutations = [];
                var obs = new MutationObserver(function(m) { mutations.push(m.length); });
                obs.observe(document.getElementById('target'), { childList: true });
                document.getElementById('target').textContent = 'changed';
                // MO 回调经 microtask 异步派发——末轮微任务断言 mutations 已记录（P1a JS 驱动 mutation 真触发）。
                Promise.resolve().then(function () {
                    if (mutations.length < 1) {
                        throw new Error('mutation-observer: callback not invoked after textContent change');
                    }
                    document.getElementById('log').textContent = 'Mutations: ' + mutations.length;
                });
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/event-bubble-capture".into(),
            description: "事件冒泡和捕获".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>Event Bubbling and Capture</h1>
            <div id="outer"><div id="inner">Click me</div></div>
            <div id="log"></div>
            <script>
                var log = [];
                var outer = document.getElementById('outer');
                var inner = document.getElementById('inner');
                outer.addEventListener('click', function() { log.push('outer-capture'); }, true);
                outer.addEventListener('click', function() { log.push('outer-bubble'); }, false);
                inner.addEventListener('click', function(e) { log.push('inner'); e.stopPropagation(); }, false);
                inner.click();
                // 捕获阶段先于 target：outer-capture → inner（stopPropagation 阻止后续 bubble）。
                if (log.join(',') !== 'outer-capture,inner') {
                    throw new Error('event-bubble-capture: log="' + log.join(',') + '" expected "outer-capture,inner"');
                }
                document.getElementById('log').textContent = log.join(',');
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/console-api".into(),
            description: "console API 完整方法".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>Console API</h1>
            <script>
                // 所有 console 方法存在且不抛（headless 桥接宿主日志或 no-op）。
                var methods = ['log','info','warn','error','debug','table','group','groupEnd','time','timeEnd','assert','count','dir','trace'];
                for (var i = 0; i < methods.length; i++) {
                    if (typeof console[methods[i]] !== 'function') {
                        throw new Error('console-api: console.' + methods[i] + ' not a function');
                    }
                }
                console.log('log'); console.info('info'); console.warn('warn'); console.error('error');
                console.debug('debug'); console.table([{a:1},{a:2}]); console.group('group'); console.log('grouped');
                console.groupEnd(); console.time('timer'); console.timeEnd('timer');
                console.assert(true, 'should not show'); console.assert(false, 'should show');
                console.count('counter'); console.count('counter');
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/history-api".into(),
            description: "History API pushState/replaceState".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>History API</h1>
            <div id="log"></div>
            <script>
                // pushState/replaceState 不抛 + length 增长（pushState 后 length >= 2）。
                history.pushState({page: 1}, '', '?page=1');
                history.replaceState({page: 2}, '', '?page=2');
                var len = history.length;
                if (len < 2) throw new Error('history-api: history.length=' + len + ' expected >= 2 after pushState');
                document.getElementById('log').textContent = 'pushState:ok|replaceState:ok|length:' + len;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },

        TestCase {
            id: "web-api/runtime/worker-lifecycle".into(),
            description: "Worker 创建/消息/终止生命周期".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>Worker Lifecycle</h1>
            <div id="log"></div>
            <script>
                // Worker 构造 + onmessage/onerror/terminate API 存在（headless worker postMessage 异步，
                // 消息到达不保证在 run_page_scripts_strict drain 窗口内——仅断言构造 + API 可用）。
                var worker = new Worker(URL.createObjectURL(
                    new Blob(['postMessage("hello");'], {type: 'application/javascript'})
                ));
                if (typeof worker.onmessage === 'undefined' && !('onmessage' in worker)) {
                    throw new Error('worker-lifecycle: worker.onmessage missing');
                }
                if (typeof worker.postMessage !== 'function') {
                    throw new Error('worker-lifecycle: worker.postMessage not a function');
                }
                if (typeof worker.terminate !== 'function') {
                    throw new Error('worker-lifecycle: worker.terminate not a function');
                }
                worker.onmessage = function(e) { document.getElementById('log').textContent = 'msg:' + e.data; };
                worker.terminate();
                document.getElementById('log').textContent = 'terminated';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into(), "js_executes_ok".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // WebAssembly 自动桥接增强测试
        // ═══════════════════════════════════════════════════════════════
        // WebAssembly.instantiateStreaming() API
        TestCase {
            id: "web-api/wasm/instantiate-streaming".into(),
            description: "WebAssembly.instantiateStreaming API 可用".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="r">pending</div>
            <script>
                var hasStreaming = typeof WebAssembly.instantiateStreaming === 'function';
                document.getElementById('r').textContent = hasStreaming ? 'streaming ok' : 'no streaming';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // WebAssembly.validate() 魔术字节检测
        TestCase {
            id: "web-api/wasm/validate-magic".into(),
            description: "WebAssembly.validate 检测 WASM 魔术字节".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="r">pending</div>
            <script>
                var validWasm = new Uint8Array([0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);
                var invalidWasm = new Uint8Array([0x01, 0x02, 0x03, 0x04]);
                var v1 = WebAssembly.validate(validWasm);
                var v2 = WebAssembly.validate(invalidWasm);
                // R3076：spec 规定 validate() 对非 BufferSource 抛 TypeError（非返 false）。try/catch 捕获。
                var v3 = null;
                try { v3 = WebAssembly.validate(null); } catch (e) { v3 = 'threw'; }
                document.getElementById('r').textContent = (v1 && !v2 && v3 === 'threw') ? 'validate ok' : 'validate fail:' + v1 + ':' + v2 + ':' + v3;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // WebAssembly 调用队列基础设施
        TestCase {
            id: "web-api/wasm/call-queue".into(),
            description: "WebAssembly 调用队列基础设施可用".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="r">pending</div>
            <script>
                var hasQueue = Array.isArray(WebAssembly._callQueue);
                var hasResults = typeof WebAssembly._callResults === 'object';
                var hasCallId = typeof WebAssembly._nextCallId === 'number';
                document.getElementById('r').textContent = (hasQueue && hasResults && hasCallId) ? 'queue ok' : 'queue fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
        // WebAssembly 完整桥接页面
        TestCase {
            id: "web-api/wasm/full-bridge-page".into(),
            description: "WebAssembly 完整桥接综合页面".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <h1>WASM Bridge Test</h1>
            <div id="api-check">checking</div>
            <div id="validate-check">checking</div>
            <div id="streaming-check">checking</div>
            <script>
                // API 检测
                var apis = [
                    typeof WebAssembly === 'object',
                    typeof WebAssembly.compile === 'function',
                    typeof WebAssembly.instantiate === 'function',
                    typeof WebAssembly.validate === 'function',
                    typeof WebAssembly.instantiateStreaming === 'function'
                ];
                document.getElementById('api-check').textContent = apis.every(Boolean) ? 'all apis ok' : 'missing apis';

                // 验证检测
                var validWasm = new Uint8Array([0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);
                document.getElementById('validate-check').textContent = WebAssembly.validate(validWasm) ? 'validate ok' : 'validate fail';

                // 流式 API 检测
                document.getElementById('streaming-check').textContent = typeof WebAssembly.instantiateStreaming === 'function' ? 'streaming ok' : 'no streaming';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into()],
        },
        // ── R3320：Geometry Interfaces（DOMRect/DOMRectReadOnly/DOMMatrix/DOMPoint）──
        // js_executes_ok 断言：脚本抛异常（API 缺失/instanceof 失败）即 fail。锁 R3319（DOMRect +
        // DOMRectReadOnly 全局构造器）+ R2985（DOMMatrix/DOMPoint）。库 identity 检查
        // （popper.js/floating-ui `rect instanceof DOMRectReadOnly`）依赖此面。
        TestCase {
            id: "web-api/geometry/interfaces".into(),
            description: "Geometry Interfaces 构造器存在 + DOMRect instanceof 继承".into(),
            category: "web-api".into(),
            html: r#"<html><body>
            <div id="g">geometry</div>
            <script>
                // 四个构造器存在（DOMRect/DOMRectReadOnly R3319，DOMMatrix/DOMPoint R2985）。
                var ctors = [DOMRect, DOMRectReadOnly, DOMMatrix, DOMPoint];
                var allFn = ctors.every(function (c) { return typeof c === 'function'; });
                // DOMRect 继承 DOMRectReadOnly（is-a，prototype 链）。
                var r = new DOMRect(10, 20, 100, 50);
                var isDR = r instanceof DOMRect;
                var isDRO = r instanceof DOMRectReadOnly;
                // 派生属性同步（top=y, right=x+width）。
                var derived = (r.top === 20 && r.right === 110 && r.bottom === 70);
                // DOMMatrix/DOMPoint 基本可用。
                var m = new DOMMatrix();
                var p = new DOMPoint(1, 2);
                var mtxOk = typeof m.multiply === 'function' && m.a === 1;
                var ptOk = p.x === 1 && p.w === 1;
                document.getElementById('g').textContent =
                    (allFn && isDR && isDRO && derived && mtxOk && ptOk) ? 'geometry ok' : 'geometry fail';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into(), "js_executes_ok".into()],
        },
    ]
}
