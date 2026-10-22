// part67 — WebGL 接口面（goal webgl-compat M2，2026-10-22）：getContext('webgl')
// 上下文对象（part08 `globalThis._zwMakeCtxWebGL`，同一 host 2d 位图作 drawing
// buffer）+ 常量/方法 WebIDL 面（idlharness 断言口径）+ getError 状态机 +
// buffer/texture 错误语义（pin rev corpus 7 案 15 subtests 验收路径）+ clear()
// 经 host 2d 位图光栅化（Rust 侧 `canvas_context_full_rgba` 逐像素回读）。
// 沙盒沿用 part05 R2795 先例（createElement/getElementById canvas + CanvasRegistry）。

/// M2 公共沙盒：body 预置 `<canvas id='cv'>`，JS 侧建 webgl 上下文并暴露
/// `__gl`（上下文）与 `__h`（host 位图 id，供 Rust 侧像素回读）。
fn webgl_sandbox() -> (
    zero_script_sandbox::V8Sandbox,
    std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>>,
) {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><canvas id='cv' width='8' height='6'></canvas></body></html>".to_string(),
    ));
    let page: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/webgl/m2.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page, &canvas_registry, None);
    sandbox
        .execute(
            "globalThis.__cv = document.getElementById('cv');\
             globalThis.__gl = __cv.getContext('webgl');\
             globalThis.__h = __gl._handle;",
        )
        .unwrap();
    (sandbox, canvas_registry)
}

/// 位图像素采样（行主序 RGBA）。
fn px(data: &[u8], w: usize, x: usize, y: usize) -> [u8; 4] {
    let i = (y * w + x) * 4;
    [data[i], data[i + 1], data[i + 2], data[i + 3]]
}

#[test]
fn test_webgl_context_creation_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = [];\
             r.push(__gl ? 1 : 0);\
             r.push(typeof __gl.getError === 'function' ? 1 : 0);\
             r.push(__gl.getError() === 0 ? 1 : 0);\
             r.push(__gl.getError() === 0 ? 1 : 0);\
             r.push(__gl.canvas === __cv ? 1 : 0);\
             r.push(__gl.drawingBufferWidth === 8 && __gl.drawingBufferHeight === 6 ? 1 : 0);\
             r.push(__cv.getContext('experimental-webgl') === __gl ? 1 : 0);\
             r.push(__cv.getContext('webgl2') === null ? 1 : 0);\
             r.push(__cv.getContext('2d') === null ? 1 : 0);\
             var c2 = document.createElement('canvas');\
             r.push(c2.getContext('2d') !== null && c2.getContext('webgl') === null ? 1 : 0);\
             r.push(__cv.getContext('webgl') === __gl ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "11111111111",
        "上下文创建/别名复用/webgl2 null/类型冲突（HTML spec canvas.getContext）"
    );
}

#[test]
fn test_webgl_interface_objects_webidl_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = [];\
             r.push(typeof WebGLRenderingContext === 'function' ? 1 : 0);\
             r.push(typeof WebGL2RenderingContext === 'function' ? 1 : 0);\
             r.push(Object.getPrototypeOf(WebGLBuffer) === WebGLObject ? 1 : 0);\
             r.push(Object.getPrototypeOf(WebGLTexture.prototype) === WebGLObject.prototype ? 1 : 0);\
             r.push(Object.getPrototypeOf(WebGL2RenderingContext) === Function.prototype ? 1 : 0);\
             r.push(Object.getPrototypeOf(WebGL2RenderingContext.prototype) === Object.prototype ? 1 : 0);\
             r.push(WebGLRenderingContext.NO_ERROR === 0 && WebGLRenderingContext.INVALID_ENUM === 0x0500 && WebGLRenderingContext.INVALID_OPERATION === 0x0502 ? 1 : 0);\
             var dd = Object.getOwnPropertyDescriptor(WebGLRenderingContext, 'NO_ERROR');\
             r.push(dd && !dd.writable && dd.enumerable && !dd.configurable ? 1 : 0);\
             r.push(WebGLRenderingContext.prototype.TEXTURE_2D === 0x0DE1 ? 1 : 0);\
             r.push(WebGL2RenderingContext.prototype.RGBA16F === 0x881A ? 1 : 0);\
             r.push(WebGL2RenderingContext.prototype.ARRAY_BUFFER === 0x8892 ? 1 : 0);\
             r.push(WebGLRenderingContext.prototype.texImage2D.length === 6 ? 1 : 0);\
             r.push(WebGLRenderingContext.prototype.compressedTexImage2D.length === 7 ? 1 : 0);\
             r.push(WebGLRenderingContext.prototype.getExtension.length === 1 ? 1 : 0);\
             r.push(WebGLRenderingContext.prototype.texImage2D.name === 'texImage2D' ? 1 : 0);\
             r.push(WebGL2RenderingContext.prototype.texImage3D.length === 10 ? 1 : 0);\
             function throws(f) { try { f(); return 0; } catch (e) { return e instanceof TypeError ? 1 : 0; } }\
             r.push(throws(function () { WebGLRenderingContext.prototype.getError.call({}); }));\
             r.push(throws(function () { WebGLRenderingContext.prototype.getError.call(null); }));\
             r.push(throws(function () { WebGLBuffer(); }));\
             r.push(throws(function () { new WebGLBuffer(); }));\
             r.push(throws(function () { Object.getOwnPropertyDescriptor(WebGLObject.prototype, 'label').get.call({}); }));\
             r.push(throws(function () { __gl.compressedTexImage2D(__gl.TEXTURE_2D, 0, 0x83F0, 4, 4, 0); }));\
             r.push(typeof __gl.getContextAttributes === 'function' ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "11111111111111111111111",
        "接口对象/原型链/常量双落/.length/.name/品牌 TypeError/必参 TypeError（idlharness 口径）"
    );
}

#[test]
fn test_webgl_corpus_batch_b_error_semantics() {
    // pin rev corpus 批 B（7 案 15 subtests）验收路径逐条对齐——语义与
    // tests/wpt-runner/wpt-data/webgl/*.html 断言一致。
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             function err() { return gl.getError(); }\
             var b = gl.createBuffer();\
             gl.bindBuffer(gl.ARRAY_BUFFER, b);\
             gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(1), gl.STATIC_DRAW);\
             gl.bufferSubData(gl.ARRAY_BUFFER, NaN, new Float32Array(1));\
             r.push(err() === 0 ? 1 : 0);\
             gl.compressedTexImage2D(gl.TEXTURE_2D, 0, 0x83F0, 4, 4, 0, new Uint8Array(8));\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             gl.compressedTexImage2D(gl.TEXTURE_2D, 0, 0x83F3, 4, 4, 0, new Uint8Array(8));\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             var tf = false;\
             try { gl.compressedTexImage2D(gl.TEXTURE_2D, 0, 0x83F0, 4, 4, 0); } catch (e) { tf = e instanceof TypeError; }\
             r.push(tf ? 1 : 0);\
             gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, 0x83F0, new Uint8Array(8));\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             tf = false;\
             try { gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, 0x83F0); } catch (e) { tf = e instanceof TypeError; }\
             r.push(tf ? 1 : 0);\
             var ext = gl.getExtension('WEBGL_compressed_texture_s3tc');\
             r.push(ext ? 1 : 0);\
             gl.bindTexture(gl.TEXTURE_2D, gl.createTexture());\
             gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, 0x83F0, new Uint8Array(8));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             var hit = 0;\
             var bad = { get width() { hit++; return 1; }, get height() { return 1; } };\
             tf = false;\
             try { gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, bad); } catch (e) { tf = e instanceof TypeError; }\
             r.push(tf && hit === 0 ? 1 : 0);\
             tf = false;\
             try { gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, bad); } catch (e) { tf = e instanceof TypeError; }\
             r.push(tf && hit === 0 ? 1 : 0);\
             var thr = false;\
             var sneaky = { valueOf: function () { throw new Error('should not be called'); } };\
             try {\
               gl.uniformMatrix2fv(null, sneaky, new Float32Array(4));\
               gl.uniformMatrix3fv(null, sneaky, new Float32Array(9));\
               gl.uniformMatrix4fv(null, false, new Float32Array(16));\
             } catch (e) { thr = true; }\
             r.push(!thr && err() === 0 ? 1 : 0);\
             r.push(gl.getUniformLocation(gl.createProgram(), 'u_foo') === null && err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.drawArrays(gl.TRIANGLES, 0, 3);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.getError();\
             gl.bindBuffer(0x1234, null);\
             gl.drawArrays(0x9999, 0, 1);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             r.push(err() === 0 ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "111111111111111",
        "corpus 批 B 15 subtests：NaN no-op/未使能 INVALID_ENUM/缺参 TypeError/无 base INVALID_OPERATION/非法 source TypeError（getter 不触）/null location no-throw/未链接 INVALID_OPERATION/首错 sticky"
    );
}

#[test]
fn test_webgl_clear_raster_full() {
    use std::sync::Mutex;
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, reg) = webgl_sandbox();
    sandbox
        .execute(
            "__gl.clearColor(1, 0, 0, 1);\
             __gl.clear(__gl.COLOR_BUFFER_BIT);\
             globalThis.__err = __gl.getError();",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__err").unwrap().value, "0");
    let id: u64 = sandbox.execute("globalThis.__h").unwrap().value.parse().unwrap();
    let (w, h, data) = crate::js_dom_bridge::canvas::canvas_context_full_rgba(&reg.lock().unwrap(), id)
        .expect("host 位图存在");
    assert_eq!((w, h), (8, 6));
    assert!(
        data.chunks(4).all(|p| p == [255, 0, 0, 255]),
        "clear(COLOR_BUFFER_BIT) 全幅红（copy 合成替换含 alpha）"
    );
}

#[test]
fn test_webgl_clear_raster_scissor_mask() {
    use std::sync::Mutex;
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, reg) = webgl_sandbox();
    // 全清蓝 → scissor 左下 2×2 清红（GL 底左原点 → 位图行 4-5）→ colorMask 全 false
    // 清绿（无变化）→ DEPTH 位清除（无变化、无错误）。
    sandbox
        .execute(
            "__gl.clearColor(0, 0, 1, 1);\
             __gl.clear(__gl.COLOR_BUFFER_BIT);\
             __gl.enable(__gl.SCISSOR_TEST);\
             __gl.scissor(0, 0, 2, 2);\
             __gl.clearColor(1, 0, 0, 1);\
             __gl.clear(__gl.COLOR_BUFFER_BIT);\
             __gl.colorMask(false, false, false, false);\
             __gl.clearColor(0, 1, 0, 1);\
             __gl.clear(__gl.COLOR_BUFFER_BIT);\
             __gl.disable(__gl.SCISSOR_TEST);\
             __gl.clear(__gl.DEPTH_BUFFER_BIT);\
             globalThis.__err = __gl.getError();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__err").unwrap().value,
        "0",
        "colorMask 全 false 与 DEPTH 位清除均无错误"
    );
    let id: u64 = sandbox.execute("globalThis.__h").unwrap().value.parse().unwrap();
    let (w, h, data) = crate::js_dom_bridge::canvas::canvas_context_full_rgba(&reg.lock().unwrap(), id)
        .expect("host 位图存在");
    assert_eq!((w, h), (8, 6));
    let blue = [0, 0, 255, 255];
    let red = [255, 0, 0, 255];
    // 行 0-3 全蓝；行 4-5 列 0-1 红（scissor 清屏），列 2-7 蓝。
    for y in 0..4usize {
        for x in 0..8usize {
            assert_eq!(px(&data, 8, x, y), blue, "row {y} col {x} 应为蓝");
        }
    }
    assert_eq!(px(&data, 8, 0, 4), red, "scissor 左下角应为红");
    assert_eq!(px(&data, 8, 1, 5), red, "scissor 覆盖区应为红");
    for x in 2..8usize {
        assert_eq!(px(&data, 8, x, 4), blue, "scissor 外列 {x} 保持蓝（colorMask 关）");
    }
}
