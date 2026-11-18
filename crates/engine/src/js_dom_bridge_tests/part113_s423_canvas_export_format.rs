// js_dom_bridge 测试切片 113。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s423：canvas 导出编码格式面——toBlob/toDataURL type 参数接通（spec
/// #dom-canvas-todataurl/#dom-canvas-toblob：type parse as MIME type 按 essence
/// 比较；非受支持编码格式回落 image/png，PNG fallback）。引擎支持集 =
/// image/png + image/jpeg（host 编码臂 png crate + jpeg-encoder）。
///
/// 复现面：toBlob.jpeg.html 1F——`toBlob(cb, 'image/jpeg')` 旧臂忽略 type 恒产
/// `image/png` Blob，`assert_equals(data.type, "image/jpeg")` 失败。spec 链接
/// https://html.spec.whatwg.org/multipage/canvas.html#dom-canvas-toblob 。
///
/// 断言面：①proxy trap 路径 toBlob('image/jpeg') → Blob.type 'image/jpeg' +
/// JPEG SOI 魔数（FF D8 FF）非空；②大小写不敏感 essence（'IMAGE/JPEG'）同
/// jpeg；③非受支持格式（'image/webp'）→ PNG fallback（type 'image/png' + PNG
/// 签名）；④standalone（part05 _zwMakeCanvas 经 createElement）toDataURL
/// ('image/jpeg') → `data:image/jpeg;base64,` 前缀；⑤toDataURL 无效类型 →
/// `data:image/png;base64,` 前缀；⑥PNG 默认面回归（无参调用保持
/// data:image/png 前缀）。
#[test]
fn test_s423_canvas_export_format() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // ①proxy trap 路径（createElement('canvas') → part04 get-trap）toBlob('image/jpeg')：
    // Blob.type 'image/jpeg' + SOI 魔数 FF,D8,FF（microtask 每 execute 末 drain）。
    sandbox
        .execute(
            "var c = document.createElement('canvas'); c.width=8; c.height=8;\
             var cx = c.getContext('2d'); cx.fillStyle='red'; cx.fillRect(0,0,8,8);\
             c.toBlob(function(b){ globalThis.__jb = b; }, 'image/jpeg');",
        )
        .unwrap();
    sandbox.execute("globalThis.__noop1 = 1;").unwrap(); // drain toBlob microtask
    assert_eq!(
        sandbox.execute("globalThis.__jb.type").unwrap().value,
        "image/jpeg",
        "toBlob('image/jpeg') 须产 type 'image/jpeg' Blob（旧臂忽略 type 恒 png）"
    );
    sandbox
        .execute(
            "globalThis.__soi = null;\
             globalThis.__jb.arrayBuffer().then(function(buf){\
               var u = new Uint8Array(buf);\
               globalThis.__soi = u[0] + ',' + u[1] + ',' + u[2] + ',' + u.length;\
             });",
        )
        .unwrap();
    sandbox.execute("globalThis.__noop2 = 1;").unwrap(); // drain arrayBuffer Promise
    assert_eq!(
        sandbox
            .execute(
                "String(globalThis.__soi.slice(0, 11)) + '|' + String(globalThis.__soi.length > 11)"
            )
            .unwrap()
            .value,
        "255,216,255|true",
        "JPEG Blob 须为真 JPEG 字节（SOI FF D8 FF）且非空"
    );

    // ②essence 大小写不敏感（MIME parse 小写化）。
    sandbox
        .execute("c.toBlob(function(b){ globalThis.__jb2 = b; }, 'IMAGE/JPEG');")
        .unwrap();
    sandbox.execute("globalThis.__noop3 = 1;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__jb2.type").unwrap().value,
        "image/jpeg",
        "'IMAGE/JPEG' essence 小写化后须命中 jpeg 白名单"
    );

    // ③非受支持格式 → spec PNG fallback（type 'image/png' + PNG 签名）。
    sandbox
        .execute("c.toBlob(function(b){ globalThis.__wb = b; }, 'image/webp');")
        .unwrap();
    sandbox.execute("globalThis.__noop4 = 1;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__wb.type").unwrap().value,
        "image/png",
        "'image/webp' 非受支持 → PNG fallback（spec 步骤：type 不支持用 image/png）"
    );
    sandbox
        .execute(
            "globalThis.__psig = null;\
             globalThis.__wb.arrayBuffer().then(function(buf){\
               var u = new Uint8Array(buf);\
               globalThis.__psig = u[0] + ',' + u[1] + ',' + u[2] + ',' + u[3];\
             });",
        )
        .unwrap();
    sandbox.execute("globalThis.__noop5 = 1;").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__psig)").unwrap().value,
        "137,80,78,71",
        "PNG fallback Blob 须为真 PNG 字节（签名 89 50 4E 47）"
    );

    // ④standalone（part05 _zwMakeCanvas 同款 createElement 面）toDataURL('image/jpeg')。
    assert_eq!(
        sandbox
            .execute(
                "var d = document.createElement('canvas'); d.width=6; d.height=6;\
                 d.toDataURL('image/jpeg').slice(0, 22)"
            )
            .unwrap()
            .value,
        "data:image/jpeg;base64",
        "toDataURL('image/jpeg') 须返 jpeg data URL 前缀"
    );

    // ⑤toDataURL 无效类型 → PNG 前缀（fallback）。
    assert_eq!(
        sandbox
            .execute("d.toDataURL('image/bogus').slice(0, 15)").unwrap().value,
        "data:image/png;",
        "toDataURL 非受支持类型须回落 image/png 前缀"
    );

    // ⑥PNG 默认面回归（无参调用保持原前缀——R2797 面不变）。
    assert_eq!(
        sandbox.execute("d.toDataURL().slice(0, 15)").unwrap().value,
        "data:image/png;",
        "无参 toDataURL 保持 image/png 前缀（R2797 面回归）"
    );
}
