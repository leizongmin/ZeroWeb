// WASM 自动桥接单元测试
//
// 验证 WebView 的 WebAssembly 桥接方法：
// - process_wasm_bridge 探测和处理
// - base64 编解码工具函数
// - execute_wasm 直接执行
// - call_wasm_export 缓存实例调用
// - WASM 实例缓存管理

use super::super::*;

/// 创建最小的 WASM 模块（add 函数：两数相加）。
fn wasm_add_module() -> Vec<u8> {
    vec![
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
        0x01, 0x07, 0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F, // type section
        0x03, 0x02, 0x01, 0x00, // function section
        0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // export section
        0x0A, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B, // code section
    ]
}

/// 创建一个空 _start 函数的 WASM 模块（验证桥接自动执行不崩溃）。
///
/// 使用简单的方法：将 add 函数重命名为 _start，确保桥接自动调用它不会 panic。
fn wasm_start_module() -> Vec<u8> {
    // (module
    //   (func (export "_start") (param i32 i32) (result i32)
    //     local.get 0
    //     local.get 1
    //     i32.add)
    // )
    // 与 wasm_add_module 相同，但导出名改为 "_start"
    vec![
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
        // Type section: 1 type, func (i32, i32) -> (i32)
        0x01, 0x07, 0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F, // Function section: 1 function, type index 0
        0x03, 0x02, 0x01, 0x00, // Export section: 1 export, "_start", func index 0
        0x07, 0x0A, 0x01, 0x06, 0x5F, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x00,
        // Code section: 1 function body
        0x0A, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B,
    ]
}

// ── base64 编解码 ──

#[test]
fn test_base64_decode_empty() {
    let result = crate::webview::base64_decode("").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_base64_decode_hello() {
    // "hello" → base64: "aGVsbG8="
    let result = crate::webview::base64_decode("aGVsbG8=").unwrap();
    assert_eq!(result, b"hello");
}

#[test]
fn test_base64_decode_wasm_magic() {
    // WASM magic bytes: 0x00 0x61 0x73 0x6D
    let b64 = crate::webview::base64_encode(&[0x00, 0x61, 0x73, 0x6D]);
    let decoded = crate::webview::base64_decode(&b64).unwrap();
    assert_eq!(decoded, vec![0x00, 0x61, 0x73, 0x6D]);
}

#[test]
fn test_base64_roundtrip() {
    let data = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
    let encoded = crate::webview::base64_encode(&data);
    let decoded = crate::webview::base64_decode(&encoded).unwrap();
    assert_eq!(data, decoded);
}

#[test]
fn test_base64_roundtrip_long() {
    let wasm = wasm_add_module();
    let encoded = crate::webview::base64_encode(&wasm);
    let decoded = crate::webview::base64_decode(&encoded).unwrap();
    assert_eq!(wasm, decoded);
}

// ── execute_wasm 直接执行 ──

#[test]
fn test_execute_wasm_add() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_add_module();
    let result = wv
        .execute_wasm(
            &wasm,
            "add",
            &[
                zero_wasm_sandbox::WasmValue::I32(3),
                zero_wasm_sandbox::WasmValue::I32(7),
            ],
        )
        .unwrap();
    assert_eq!(result, "i32(10)");
}

#[test]
fn test_execute_wasm_zero_args() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_add_module();
    // 调用 add(0, 0)
    let result = wv
        .execute_wasm(
            &wasm,
            "add",
            &[
                zero_wasm_sandbox::WasmValue::I32(0),
                zero_wasm_sandbox::WasmValue::I32(0),
            ],
        )
        .unwrap();
    assert_eq!(result, "i32(0)");
}

#[test]
fn test_execute_wasm_invalid_bytes() {
    let wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_wasm(&[0x00, 0x01, 0x02], "add", &[]);
    assert!(result.is_err(), "无效 WASM 字节应返回错误");
}

#[test]
fn test_execute_wasm_missing_function() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_add_module();
    let result = wv.execute_wasm(&wasm, "nonexistent", &[]);
    assert!(result.is_err(), "不存在的函数应返回错误");
}

// ── call_wasm_export 缓存实例调用 ──

#[test]
fn test_call_wasm_export_missing_instance() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.call_wasm_export(
        99999,
        "add",
        &[
            zero_wasm_sandbox::WasmValue::I32(1),
            zero_wasm_sandbox::WasmValue::I32(2),
        ],
    );
    assert!(result.is_err(), "不存在的实例 ID 应返回错误");
}

#[test]
fn test_execute_wasm_negative_args() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_add_module();
    let result = wv
        .execute_wasm(
            &wasm,
            "add",
            &[
                zero_wasm_sandbox::WasmValue::I32(-5),
                zero_wasm_sandbox::WasmValue::I32(3),
            ],
        )
        .unwrap();
    assert_eq!(result, "i32(-2)");
}

#[test]
fn test_execute_wasm_large_args() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_add_module();
    let result = wv
        .execute_wasm(
            &wasm,
            "add",
            &[
                zero_wasm_sandbox::WasmValue::I32(i32::MAX),
                zero_wasm_sandbox::WasmValue::I32(0),
            ],
        )
        .unwrap();
    assert_eq!(result, format!("i32({})", i32::MAX));
}

// ── _start 自动执行 ──

#[test]
fn test_execute_wasm_start_module() {
    // 验证含 _start 导出的 WASM 模块可以通过 execute_wasm 正常执行
    let wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_start_module();
    // _start 接受两个 i32 参数（与 add 相同的签名）
    let result = wv
        .execute_wasm(
            &wasm,
            "_start",
            &[
                zero_wasm_sandbox::WasmValue::I32(10),
                zero_wasm_sandbox::WasmValue::I32(20),
            ],
        )
        .unwrap();
    assert_eq!(result, "i32(30)", "_start 函数应正确执行加法");
}

#[test]
fn test_wasm_bridge_start_auto_execution() {
    // 验证桥接自动执行 _start 不崩溃
    let mut wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_start_module();
    let js_bytes: String = wasm.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",");

    // 通过桥接实例化（桥接应自动执行 _start）
    let result = wv
        .execute_script_with_dom(&format!(
            r#"
        var bytes = new Uint8Array([{js_bytes}]);
        var result = WebAssembly.instantiate(bytes);
        typeof result.then === 'function'
        "#
        ))
        .unwrap();
    assert_eq!(result, "true", "instantiate 含 _start 模块应成功");

    // 验证实例已缓存且 __wasm_results__ 存在
    let check = wv.execute_script("Object.keys(__wasm_results__).length > 0").unwrap();
    assert_eq!(check, "true", "_start 自动执行后实例应已缓存");
}

// ── WASM 桥接集成（通过 execute_script_with_dom）──

#[test]
fn test_wasm_bridge_no_wasm() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 不使用 WASM 时应正常工作
    let result = wv.execute_script_with_dom("1 + 1").unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_wasm_bridge_instantiate() {
    let mut wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_add_module();
    let js_bytes: String = wasm.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",");

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

    // 验证实例已缓存
    let check = wv
        .execute_script("typeof __wasm_results__ === 'object' && Object.keys(__wasm_results__).length > 0")
        .unwrap();
    assert_eq!(check, "true", "WASM 实例应被注入到 JS 环境");
}

#[test]
fn test_wasm_bridge_validate() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv
        .execute_script_with_dom(
            r#"
            var validWasm = new Uint8Array([0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);
            var invalid = new Uint8Array([0xFF, 0xFF, 0xFF, 0xFF]);
            WebAssembly.validate(validWasm) && !WebAssembly.validate(invalid)
        "#,
        )
        .unwrap();
    assert_eq!(result, "true", "validate 应检测 WASM 魔术字节");
}

#[test]
fn test_wasm_bridge_instantiate_streaming() {
    let mut wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_add_module();
    let js_bytes: String = wasm.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",");

    let result = wv
        .execute_script_with_dom(&format!(
            r#"
        var bytes = new Uint8Array([{js_bytes}]);
        var result = WebAssembly.instantiateStreaming(bytes);
        typeof result.then === 'function'
        "#
        ))
        .unwrap();
    assert_eq!(result, "true", "instantiateStreaming 应返回 Promise");
}

#[test]
fn test_wasm_bridge_compile() {
    let mut wv = WebView::new(WebViewConfig::default());
    let wasm = wasm_add_module();
    let js_bytes: String = wasm.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",");

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

#[test]
fn test_wasm_call_queue_infrastructure() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv
        .execute_script_with_dom(
            r#"
            Array.isArray(WebAssembly._callQueue) &&
            typeof WebAssembly._callResults === 'object' &&
            typeof WebAssembly._nextCallId === 'number'
        "#,
        )
        .unwrap();
    assert_eq!(result, "true", "调用队列基础设施应可用");
}
