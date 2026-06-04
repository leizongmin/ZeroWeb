// WASM 自动桥接集成测试
//
// 验证 JS 端 WebAssembly.instantiate() 通过 WebView 自动桥接到 wasm-sandbox：
// - WebAssembly API 可用性检测
// - instantiate() 触发桥接请求
// - call_wasm_export() 调用缓存实例的导出函数
// - base64 编解码正确性

use zero_webview::{WebView, WebViewConfig};

// ── 辅助函数 ──

fn create_webview() -> WebView {
    WebView::new(WebViewConfig::default())
}

/// 创建一个最小的 WASM 模块（add 函数：两数相加）。
///
/// 模块导出一个 `add(i32, i32) -> i32` 函数。
fn wasm_add_module() -> Vec<u8> {
    // 手工构建的 WASM 二进制：
    // (module
    //   (func (export "add") (param i32 i32) (result i32)
    //     local.get 0
    //     local.get 1
    //     i32.add)
    // )
    vec![
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
        // Type section (id=1): 1 type, func (i32, i32) -> (i32)
        0x01, 0x07, 0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F,
        // Function section (id=3): 1 function, type index 0
        0x03, 0x02, 0x01, 0x00, // Export section (id=7): 1 export, "add", func index 0
        0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // Code section (id=10): 1 function body
        0x0A, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B,
    ]
}

// ── 测试 ──

/// WASM 桥接 — JS 端 WebAssembly API 可用。
#[test]
fn test_wasm_api_available() {
    let mut wv = create_webview();
    let result = wv
        .execute_script_with_dom(
            r#"
            typeof WebAssembly === 'object' &&
            typeof WebAssembly.instantiate === 'function' &&
            typeof WebAssembly.compile === 'function' &&
            typeof WebAssembly.validate === 'function'
        "#,
        )
        .unwrap();
    assert_eq!(result, "true", "WebAssembly API 应可用");
}

/// WASM 桥接 — WebAssembly.validate() 工作。
#[test]
fn test_wasm_validate() {
    let mut wv = create_webview();
    let result = wv
        .execute_script_with_dom(
            r#"
            WebAssembly.validate(new Uint8Array([0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]))
        "#,
        )
        .unwrap();
    assert_eq!(result, "true", "有效 WASM 头应通过 validate");
}

/// WASM 桥接 — WebAssembly.compile() 返回 Promise。
#[test]
fn test_wasm_compile() {
    let mut wv = create_webview();
    let wasm_bytes = wasm_add_module();
    let js_bytes = bytes_to_js_array(&wasm_bytes);
    let result = wv
        .execute_script_with_dom(&format!(
            r#"
            var bytes = new Uint8Array([{js_bytes}]);
            var mod = WebAssembly.compile(bytes);
            typeof mod.then === 'function'
        "#
        ))
        .unwrap();
    assert_eq!(result, "true", "compile 应返回 Promise");
}

/// WASM 桥接 — instantiate() 触发桥接请求并缓存实例。
#[test]
fn test_wasm_instantiate_bridge() {
    let mut wv = create_webview();
    let wasm_bytes = wasm_add_module();
    let js_bytes = bytes_to_js_array(&wasm_bytes);

    // JS 端调用 WebAssembly.instantiate()
    let result = wv
        .execute_script_with_dom(&format!(
            r#"
            var bytes = new Uint8Array([{js_bytes}]);
            var result = WebAssembly.instantiate(bytes);
            typeof result.then === 'function'
        "#
        ))
        .unwrap();
    assert_eq!(result, "true", "instantiate 应返回 Promise");

    // 验证桥接已触发 — 检查 __wasm_results__ 是否被注入
    let check = wv
        .execute_script(
            r#"
            typeof __wasm_results__ === 'object' && Object.keys(__wasm_results__).length > 0
        "#,
        )
        .unwrap();
    assert_eq!(check, "true", "桥接应在 JS 环境中注入 WASM 结果");
}

/// WASM 桥接 — call_wasm_export() 调用缓存的导出函数。
#[test]
fn test_wasm_call_export() {
    let mut wv = create_webview();
    let wasm_bytes = wasm_add_module();
    let js_bytes = bytes_to_js_array(&wasm_bytes);

    // 先通过 JS instantiate 触发桥接
    let _ = wv.execute_script_with_dom(&format!(
        r#"
        var bytes = new Uint8Array([{js_bytes}]);
        WebAssembly.instantiate(bytes);
        "#
    ));

    // 获取 JS 端的 instance_id
    let id_result = wv
        .execute_script(
            r#"
            Object.keys(__wasm_results__)[0]
        "#,
        )
        .unwrap();

    let instance_id: u64 = id_result.parse().expect("instance ID should be a number");

    // 通过 Rust API 调用导出函数
    let result = wv
        .call_wasm_export(
            instance_id,
            "add",
            &[
                zero_wasm_sandbox::WasmValue::I32(3),
                zero_wasm_sandbox::WasmValue::I32(7),
            ],
        )
        .unwrap();
    assert_eq!(result, "i32(10)", "3 + 7 应等于 10");
}

/// WASM 桥接 — 多次调用同一实例。
#[test]
fn test_wasm_multiple_calls() {
    let mut wv = create_webview();
    let wasm_bytes = wasm_add_module();
    let js_bytes = bytes_to_js_array(&wasm_bytes);

    let _ = wv.execute_script_with_dom(&format!(
        r#"
        var bytes = new Uint8Array([{js_bytes}]);
        WebAssembly.instantiate(bytes);
        "#
    ));

    let id_result = wv.execute_script("Object.keys(__wasm_results__)[0]").unwrap();
    let instance_id: u64 = id_result.parse().unwrap();

    // 多次调用
    let r1 = wv
        .call_wasm_export(
            instance_id,
            "add",
            &[
                zero_wasm_sandbox::WasmValue::I32(1),
                zero_wasm_sandbox::WasmValue::I32(2),
            ],
        )
        .unwrap();
    let r2 = wv
        .call_wasm_export(
            instance_id,
            "add",
            &[
                zero_wasm_sandbox::WasmValue::I32(100),
                zero_wasm_sandbox::WasmValue::I32(200),
            ],
        )
        .unwrap();
    let r3 = wv
        .call_wasm_export(
            instance_id,
            "add",
            &[
                zero_wasm_sandbox::WasmValue::I32(0),
                zero_wasm_sandbox::WasmValue::I32(0),
            ],
        )
        .unwrap();

    assert_eq!(r1, "i32(3)");
    assert_eq!(r2, "i32(300)");
    assert_eq!(r3, "i32(0)");
}

/// WASM 桥接 — 无 WASM 使用时不影响正常脚本执行。
#[test]
fn test_wasm_no_bridge_when_not_used() {
    let mut wv = create_webview();
    let result = wv.execute_script_with_dom("42 + 8").unwrap();
    assert_eq!(result, "50", "不使用 WASM 时脚本应正常执行");
}

/// WASM 桥接 — WebAssembly._pendingBridge 被消费后清空。
#[test]
fn test_wasm_pending_bridge_cleared() {
    let mut wv = create_webview();
    let wasm_bytes = wasm_add_module();
    let js_bytes = bytes_to_js_array(&wasm_bytes);

    let _ = wv.execute_script_with_dom(&format!(
        r#"
        var bytes = new Uint8Array([{js_bytes}]);
        WebAssembly.instantiate(bytes);
        "#
    ));

    // _pendingBridge 应被消费
    let check = wv.execute_script("WebAssembly._pendingBridge === null").unwrap();
    assert_eq!(check, "true", "_pendingBridge 应在桥接后被清空");
}

// ── 辅助函数 ──

/// 将字节切片转为 JS 数组字面量字符串，如 "0,97,115,109"。
fn bytes_to_js_array(bytes: &[u8]) -> String {
    bytes.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",")
}
