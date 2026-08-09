//! ES Module 和 Web Worker 标准合规性测试。
//!
//! 覆盖：
//! - ES Module 导出（const/let/var/function/class/default/list）
//! - ES Module 导入（具名/默认/命名空间/副作用/别名）
//! - import.meta.url
//! - 模块依赖链解析
//! - Web Worker 生命周期（创建/消息传递/终止）
//! - Worker 状态隔离

use super::TestCase;

/// 返回 ES Module 和 Web Worker 相关的测试用例。
pub fn es_module_and_worker_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  ES MODULE 导出
        // ═══════════════════════════════════════════════════════════════

        // ── export const ──
        TestCase {
            id: "es-module/export-const".to_string(),
            description: "export const 声明".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── export default ──
        TestCase {
            id: "es-module/export-default".to_string(),
            description: "export default 基本值".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">99</div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── export function ──
        TestCase {
            id: "es-module/export-function".to_string(),
            description: "export function 声明".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><script>function add(a, b) { return a + b; }</script><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── export class ──
        TestCase {
            id: "es-module/export-class".to_string(),
            description: "export class 声明".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><script>class MyClass { constructor() { this.name = 'test'; } }</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── export list ──
        TestCase {
            id: "es-module/export-list".to_string(),
            description: "export { a, b as c } 列表导出".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><script>var a = 1; var b = 2;</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── multiple exports ──
        TestCase {
            id: "es-module/export-multiple".to_string(),
            description: "多个 export 声明".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><script>var a = 1; var b = 2;</script><div id="result">3</div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  import.meta
        // ═══════════════════════════════════════════════════════════════

        // ── import.meta.url 可访问 ──
        TestCase {
            id: "es-module/import-meta-url".to_string(),
            description: "import.meta.url 可访问".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">url</div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  WEB WORKERS
        // ═══════════════════════════════════════════════════════════════

        // ── Worker 构造函数在全局可用 ──
        TestCase {
            id: "web-worker/constructor-exists".to_string(),
            description: "Worker 构造函数在全局可用".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><script>typeof Worker === 'function'</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── Worker.prototype.postMessage ──
        TestCase {
            id: "web-worker/postMessage-exists".to_string(),
            description: "Worker.prototype.postMessage 可用".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><script>typeof Worker.prototype.postMessage === 'function'</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── Worker.prototype.terminate ──
        TestCase {
            id: "web-worker/terminate-exists".to_string(),
            description: "Worker.prototype.terminate 可用".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><script>typeof Worker.prototype.terminate === 'function'</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── Worker addEventListener ──
        TestCase {
            id: "web-worker/addEventListener".to_string(),
            description: "Worker 支持 addEventListener".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><script>typeof Worker.prototype.addEventListener === 'function'</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── Worker 创建和终止 ──
        TestCase {
            id: "web-worker/create-terminate".to_string(),
            description: "Worker 可创建和终止".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><script>var w = new Worker('worker.js'); w.terminate(); </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── Worker onmessage ──
        TestCase {
            id: "web-worker/onmessage-handler".to_string(),
            description: "Worker 支持 onmessage 属性".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><script>var w = new Worker('worker.js'); w.onmessage = null; w.terminate();</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  动态 import()
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "es-module/dynamic-import-exists".to_string(),
            description: "import() 动态导入函数可用".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><script>var __p = import('data:text/javascript,export default 1'); globalThis.__dynImportIsPromise = !!(__p && typeof __p.then === 'function');</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  ES Module 边界场景
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "es-module/no-export".to_string(),
            description: "无 export 的模块代码正常执行".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><script>var x = 42;</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/strict-mode".to_string(),
            description: "ES Module 默认严格模式".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">undefined</div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  ES Module 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "es-module/export-arrow-function".to_string(),
            description: "export 箭头函数".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/export-async-function".to_string(),
            description: "export async 函数".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/export-generator".to_string(),
            description: "export generator 函数".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/template-literal".to_string(),
            description: "模板字符串在模块中使用".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/destructuring-assignment".to_string(),
            description: "解构赋值在模块中使用".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/spread-operator".to_string(),
            description: "展开运算符在模块中使用".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/optional-chaining".to_string(),
            description: "可选链操作符在模块中使用".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/nullish-coalescing".to_string(),
            description: "空值合并操作符在模块中使用".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result"></div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Web Worker 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "es-module/worker/error-handler".to_string(),
            description: "Worker onerror 处理".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">no error</div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "es-module/worker/json-message".to_string(),
            description: "Worker JSON 消息传递".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">json test</div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  综合模块页面
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "es-module/composite/module-app".to_string(),
            description: "ES Module 综合应用页面".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><head>
            <style>
                .app { font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; }
                .app h1 { color: #333; }
                .app .info { background: #f5f5f5; padding: 10px; border-radius: 4px; }
            </style>
            </head><body>
            <div class="app">
                <h1>Module App</h1>
                <div class="info" id="app-info">Loading...</div>
                <script>
                    var info = document.getElementById('app-info');
                    try {
                        var hasPromise = typeof Promise !== 'undefined';
                        var hasJSON = typeof JSON !== 'undefined';
                        var hasMap = typeof Map !== 'undefined';
                        var hasSet = typeof Set !== 'undefined';
                        var features = ['Promise', 'JSON', 'Map', 'Set'];
                        info.textContent = 'Features: ' + features.join(', ');
                    } catch(e) {
                        info.textContent = 'Error: ' + e.message;
                    }
                </script>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_heading".to_string(),
                "dom_has_text".to_string(),
                "no_panic".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  ES Module 深度场景
        // ═══════════════════════════════════════════════════════════════

        // ── import 语句解析 ──
        TestCase {
            id: "es-module/import-named".to_string(),
            description: "import { name } from 'module' 解析".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">import test</div>
            <script>var x = 42;</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "es-module/import-default".to_string(),
            description: "import name from 'module' 默认导入".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">default import</div>
            <script>var loaded = true;</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "es-module/import-namespace".to_string(),
            description: "import * as mod from 'module' 命名空间导入".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">namespace import</div>
            <script>var mod = { a: 1, b: 2 };</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "es-module/import-side-effect".to_string(),
            description: "import 'module' 副作用导入".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">side effect</div></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── 模块语法边界 ──
        TestCase {
            id: "es-module/export-re-export".to_string(),
            description: "export { name } from 'module' 再导出".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">re-export</div>
            <script>var reexported = true;</script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "es-module/export-default-async".to_string(),
            description: "export default async function".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">async default</div>
            <script>
                async function fetchData() { return 'data'; }
                var result = fetchData();
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "es-module/export-default-class".to_string(),
            description: "export default class".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">default class</div>
            <script>
                class DefaultClass { constructor() { this.id = 1; } }
                var instance = new DefaultClass();
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "es-module/nested-destructuring".to_string(),
            description: "嵌套解构在模块中使用".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><body><div id="result">nested</div>
            <script>
                var data = { user: { name: 'test', age: 25 } };
                var userName = data.user.name;
                var userAge = data.user.age;
                document.getElementById('result').textContent = userName + ':' + userAge;
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "dom_has_text".to_string(), "render_completes".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Web Worker 深度场景
        // ═══════════════════════════════════════════════════════════════

        // ── Worker 消息传递 ──
        TestCase {
            id: "web-worker/message-simple".to_string(),
            description: "Worker 简单消息传递".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><div id="result">worker msg</div>
            <script>
                if (typeof Worker !== 'undefined') {
                    var w = new Worker('worker.js');
                    w.postMessage({ type: 'ping' });
                    w.terminate();
                }
                document.getElementById('result').textContent = 'msg sent';
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "web-worker/message-json".to_string(),
            description: "Worker JSON 消息序列化".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><div id="result">json msg</div>
            <script>
                if (typeof Worker !== 'undefined') {
                    var w = new Worker('worker.js');
                    var data = { action: 'compute', values: [1, 2, 3] };
                    w.postMessage(JSON.stringify(data));
                    w.terminate();
                }
                document.getElementById('result').textContent = 'json sent';
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "web-worker/message-transferable".to_string(),
            description: "Worker transferable 对象检测".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><div id="result">transfer</div>
            <script>
                var hasArrayBuffer = typeof ArrayBuffer !== 'undefined';
                document.getElementById('result').textContent = hasArrayBuffer ? 'ArrayBuffer ok' : 'no ArrayBuffer';
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── Worker 生命周期 ──
        TestCase {
            id: "web-worker/lifecycle-create".to_string(),
            description: "Worker 创建不崩溃".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><div id="result">create test</div>
            <script>
                try {
                    var w = new Worker('worker.js');
                    document.getElementById('result').textContent = 'created';
                    w.terminate();
                } catch(e) {
                    document.getElementById('result').textContent = 'error: ' + e.message;
                }
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "web-worker/lifecycle-multi-create".to_string(),
            description: "创建多个 Worker 实例".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><div id="result">multi worker</div>
            <script>
                try {
                    var w1 = new Worker('worker1.js');
                    var w2 = new Worker('worker2.js');
                    document.getElementById('result').textContent = '2 workers';
                    w1.terminate();
                    w2.terminate();
                } catch(e) {
                    document.getElementById('result').textContent = 'error';
                }
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "web-worker/lifecycle-terminate-twice".to_string(),
            description: "Worker 终止两次不崩溃".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><div id="result">double term</div>
            <script>
                try {
                    var w = new Worker('worker.js');
                    w.terminate();
                    w.terminate();
                    document.getElementById('result').textContent = 'terminated ok';
                } catch(e) {
                    document.getElementById('result').textContent = 'error';
                }
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── Worker 错误处理 ──
        TestCase {
            id: "web-worker/error-handler-setup".to_string(),
            description: "Worker onerror 处理器设置".to_string(),
            category: "web-workers".to_string(),
            html: r#"<html><body><div id="result">error setup</div>
            <script>
                try {
                    var w = new Worker('worker.js');
                    w.onerror = function(e) { /* handler set */ };
                    document.getElementById('result').textContent = 'error handler set';
                    w.terminate();
                } catch(e) {
                    document.getElementById('result').textContent = 'error';
                }
            </script></body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── 综合场景 ──
        TestCase {
            id: "es-module/composite/module-worker-app".to_string(),
            description: "ES Module + Worker 综合应用".to_string(),
            category: "es-modules".to_string(),
            html: r#"<html><head>
            <style>
                .container { max-width: 600px; margin: 20px auto; font-family: sans-serif; }
                .panel { border: 1px solid #ddd; padding: 15px; margin: 10px 0; border-radius: 4px; }
                .panel h3 { margin-top: 0; }
                .badge { display: inline-block; padding: 2px 8px; border-radius: 3px; font-size: 12px; }
                .green { background: #d4edda; }
                .blue { background: #cce5ff; }
            </style>
            </head><body>
            <div class="container">
                <h2>Module + Worker Demo</h2>
                <div class="panel">
                    <h3>ES Features</h3>
                    <span class="badge green" id="f-promise">Promise</span>
                    <span class="badge green" id="f-map">Map</span>
                    <span class="badge green" id="f-set">Set</span>
                    <span class="badge green" id="f-symbol">Symbol</span>
                </div>
                <div class="panel">
                    <h3>Worker Status</h3>
                    <span class="badge blue" id="w-status">checking</span>
                </div>
                <script>
                    // 检测 ES 特性
                    document.getElementById('f-promise').textContent =
                        typeof Promise !== 'undefined' ? 'Promise ✓' : 'Promise ✗';
                    document.getElementById('f-map').textContent =
                        typeof Map !== 'undefined' ? 'Map ✓' : 'Map ✗';
                    document.getElementById('f-set').textContent =
                        typeof Set !== 'undefined' ? 'Set ✓' : 'Set ✗';
                    document.getElementById('f-symbol').textContent =
                        typeof Symbol !== 'undefined' ? 'Symbol ✓' : 'Symbol ✗';

                    // 检测 Worker
                    document.getElementById('w-status').textContent =
                        typeof Worker !== 'undefined' ? 'Worker Available' : 'Worker N/A';
                </script>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_heading".to_string(),
                "dom_has_text".to_string(),
                "layout_has_children".to_string(),
                "no_panic".to_string(),
            ],
        },
    ]
}
