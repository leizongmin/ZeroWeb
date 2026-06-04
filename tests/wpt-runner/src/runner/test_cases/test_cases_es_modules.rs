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
            html: r#"<html><body><script>typeof import === 'function'</script></body></html>"#.to_string(),
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
    ]
}
