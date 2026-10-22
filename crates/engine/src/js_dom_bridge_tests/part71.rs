// part71 — WebGL draw 管线（goal webgl-compat M3-S4，2026-10-23）：drawArrays/
// drawElements 校验序（WebGL 1.0 §6.24 Range Checking）+ part08d GLSL 绘制求值器
// + 光栅化像素面（Rust 侧 `canvas_context_full_rgba` 回读真位图）+ readPixels
// 域语义（§6.16 只支持 RGBA/UNSIGNED_BYTE；越界区域不动、无错误）+ FS for 循环
// i++（求值器 §5.9 自增/自减）+ mat2 变换与 discard。几何与期望值与
// /tmp 冒烟同源：直角三角 a_pos (-1,-1),(1,-1),(-1,1)，u_scale=1 时
// v_color = (a_pos*0.5+0.5, 0)。

/// S4 测试沙盒：与 part67 同型 8×6 位图；JS 侧编译 VS/FS 常量、建三角/四边形
/// 几何并暴露 `__P`（程序）、`__uloc`（u_scale 位置）、`__loc`（a_pos 位置）、
/// `__vb`/`__vb2`/`__eb`。
fn webgl_draw_sandbox() -> (
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
    let page: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/webgl/s4.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page, &canvas_registry, None);
    sandbox
        .execute(
            "globalThis.__cv = document.getElementById('cv');\
             globalThis.__gl = __cv.getContext('webgl');\
             globalThis.__h = __gl._handle;\
             globalThis.__VS = 'attribute vec2 a_pos; varying vec3 v_color; uniform vec2 u_scale;\
               void main() { v_color = vec3(a_pos.x * 0.5 + 0.5, a_pos.y * 0.5 + 0.5, 0.0);\
               gl_Position = vec4(a_pos * u_scale, 0.0, 1.0); }';\
             globalThis.__FS = 'precision mediump float; varying vec3 v_color;\
               void main() { gl_FragColor = vec4(v_color, 1.0); }';\
             globalThis.__P = (function () {\
               var vs = __gl.createShader(__gl.VERTEX_SHADER);\
               __gl.shaderSource(vs, __VS); __gl.compileShader(vs);\
               var fs = __gl.createShader(__gl.FRAGMENT_SHADER);\
               __gl.shaderSource(fs, __FS); __gl.compileShader(fs);\
               var p = __gl.createProgram();\
               __gl.attachShader(p, vs); __gl.attachShader(p, fs); __gl.linkProgram(p);\
               globalThis.__link_ok = __gl.getProgramParameter(p, __gl.LINK_STATUS);\
               __gl.useProgram(p);\
               return p;})();\
             globalThis.__loc = __gl.getAttribLocation(__P, 'a_pos');\
             globalThis.__uloc = __gl.getUniformLocation(__P, 'u_scale');\
             __gl.uniform2f(__uloc, 1, 1);\
             globalThis.__vb = __gl.createBuffer();\
             __gl.bindBuffer(__gl.ARRAY_BUFFER, __vb);\
             __gl.bufferData(__gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1]), __gl.STATIC_DRAW);\
             __gl.enableVertexAttribArray(__loc);\
             __gl.vertexAttribPointer(__loc, 2, __gl.FLOAT, false, 0, 0);",
        )
        .unwrap();
    (sandbox, canvas_registry)
}

/// 回读 host 位图（Rust 侧）。
fn draw_bitmap(
    sandbox: &mut zero_script_sandbox::V8Sandbox,
    reg: &std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>,
) -> (u32, u32, Vec<u8>) {
    let id: u64 = sandbox.execute("globalThis.__h").unwrap().value.parse().unwrap();
    crate::js_dom_bridge::canvas::canvas_context_full_rgba(&reg.lock().unwrap(), id)
        .expect("host 位图存在")
}

#[test]
fn test_webgl_draw_arrays_validation_faces() {
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, _reg) = webgl_draw_sandbox();
    sandbox
        .execute(
            "var gl = __gl, r = [];\
             function err() { return gl.getError(); }\
             gl.drawArrays(gl.TRIANGLES, -1, 3);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.drawArrays(gl.TRIANGLES, 0, -3);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.drawArrays(gl.TRIANGLES, 0, 4);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.drawArrays(gl.POINTS, 0, 3);\
             r.push(err() === 0 ? 1 : 0);\
             gl.useProgram(null);\
             gl.drawArrays(gl.TRIANGLES, 0, 3);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.useProgram(__P);\
             gl.drawArrays(gl.TRIANGLES, 0, 3);\
             r.push(err() === 0 ? 1 : 0);\
             gl.enableVertexAttribArray(2);\
             gl.drawArrays(gl.TRIANGLES, 0, 3);\
             r.push(err() === 0 ? 1 : 0);\
             gl.disableVertexAttribArray(2);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "1111111",
        "drawArrays：负参 INVALID_VALUE、越界取数 INVALID_OPERATION、\
         POINTS 合法、null 程序 INVALID_OPERATION、enabled 未消费不报错（§6.24）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__link_ok").unwrap().value,
        "true",
        "S4 沙盒程序链接成功（前置）"
    );
}

#[test]
fn test_webgl_draw_elements_validation_faces() {
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, _reg) = webgl_draw_sandbox();
    sandbox
        .execute(
            "var gl = __gl, r = [];\
             function err() { return gl.getError(); }\
             var vb2 = gl.createBuffer();\
             gl.bindBuffer(gl.ARRAY_BUFFER, vb2);\
             gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);\
             gl.vertexAttribPointer(__loc, 2, gl.FLOAT, false, 0, 0);\
             gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             var eb = gl.createBuffer();\
             gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, eb);\
             gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 1, 2, 2, 1, 3]), gl.STATIC_DRAW);\
             gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);\
             r.push(err() === 0 ? 1 : 0);\
             gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 1);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, -2);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_INT, 0);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);\
             r.push(err() === 0 ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "111111",
        "drawElements：无 ELEMENT 缓冲 INVALID_OPERATION、offset 非倍数 INVALID_OPERATION、\
         offset 负 INVALID_VALUE、UNSIGNED_INT 需 OES_element_index_uint（未启用）INVALID_ENUM"
    );
}

#[test]
fn test_webgl_vertex_attrib_pointer_validation_faces() {
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, _reg) = webgl_draw_sandbox();
    sandbox
        .execute(
            "var gl = __gl, r = [];\
             function err() { return gl.getError(); }\
             gl.vertexAttribPointer(16, 2, gl.FLOAT, false, 0, 0);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.vertexAttribPointer(0, 5, gl.FLOAT, false, 0, 0);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.vertexAttribPointer(0, 2, gl.INT, false, 0, 0);\
             r.push(err() === gl.INVALID_ENUM ? 1 : 0);\
             gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, -4);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 300, 0);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 6, 0);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.bindBuffer(gl.ARRAY_BUFFER, null);\
             gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 4);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 8);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);\
             r.push(err() === 0 ? 1 : 0);\
             gl.vertexAttrib4f(3, 0.25, 0.5, 0.75, 1);\
             r.push(JSON.stringify(gl.getVertexAttrib(3, gl.CURRENT_VERTEX_ATTRIB)) === '[0.25,0.5,0.75,1]' ? 1 : 0);\
             gl.vertexAttrib1f(3, 0.5);\
             r.push(JSON.stringify(gl.getVertexAttrib(3, gl.CURRENT_VERTEX_ATTRIB)) === '[0.5,0,0,1]' ? 1 : 0);\
             gl.bindBuffer(gl.ARRAY_BUFFER, __vb);\
             gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "11111111111",
        "vertexAttribPointer：index/size/offset/stride 值域 INVALID_VALUE，type 域与\
         对齐/无缓冲非零 offset INVALID_OPERATION，offset 0 无缓冲合法；\
         vertexAttrib cur 值 4f/1f 补齐 (0,0,1)"
    );
}

#[test]
fn test_webgl_draw_raster_triangle_pixels() {
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, reg) = webgl_draw_sandbox();
    // 全幅三角（u_scale=1）→ 缩放 0.25 重绘。
    sandbox
        .execute(
            "__gl.uniform2f(__uloc, 1, 1);\
             __gl.drawArrays(__gl.TRIANGLES, 0, 3);\
             __gl.getError();",
        )
        .unwrap();
    let (w, h, data) = draw_bitmap(&mut sandbox, &reg);
    assert_eq!((w, h), (8, 6));
    // (1,4) 中心 (1.5,4.5)：λ=(0.5625,0.1875,0.25) → v_color=(0.1875,0.25)
    assert_eq!(
        px(&data, 8, 1, 4),
        [48, 64, 0, 255],
        "三角内部重心插值像素"
    );
    // (0,5)：λ=(0.854,0.0625,0.0833) → v_color=(0.0625,0.0833)
    assert_eq!(px(&data, 8, 0, 5), [16, 21, 0, 255], "左下角近顶点像素");
    // (7,0) 在斜边外：保持 clear 底色。
    assert_eq!(px(&data, 8, 7, 0), [0, 0, 0, 0], "斜边外未写像素");
    // 缩放 0.25：host v0(3,3.75) v1(5,3.75) v2(3,2.25)；(4,3) 中心 (4.5,3.5)
    // λ=(1/12,0.75,1/6) → a_pos=(0.5,-2/3) → v_color=(0.75,1/6)。
    sandbox
        .execute(
            "__gl.uniform2f(__uloc, 0.25, 0.25);\
             __gl.drawArrays(__gl.TRIANGLES, 0, 3);\
             globalThis.__err = __gl.getError();",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__err").unwrap().value, "0");
    let (_w, _h, data) = draw_bitmap(&mut sandbox, &reg);
    assert_eq!(
        px(&data, 8, 4, 3),
        [191, 42, 0, 255],
        "uniform 缩放后插值像素"
    );
    // retained buffer：绘制间位图 read-modify-write 保留（preserveDrawingBuffer:false
    // 允许但未要求清旧帧——FIXME(preserve) preserveDrawingBuffer 语义面）。
    assert_eq!(px(&data, 8, 1, 4), [48, 64, 0, 255], "缩放后旧帧像素保留");
}

#[test]
fn test_webgl_draw_elements_quad_pixels() {
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, reg) = webgl_draw_sandbox();
    sandbox
        .execute(
            "var gl = __gl;\
             var vb2 = gl.createBuffer();\
             gl.bindBuffer(gl.ARRAY_BUFFER, vb2);\
             gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);\
             gl.vertexAttribPointer(__loc, 2, gl.FLOAT, false, 0, 0);\
             var eb = gl.createBuffer();\
             gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, eb);\
             gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 1, 2, 2, 1, 3]), gl.STATIC_DRAW);\
             gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);\
             globalThis.__err = gl.getError();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__err").unwrap().value,
        "0",
        "四边形 drawElements 无错误"
    );
    let (_w, _h, data) = draw_bitmap(&mut sandbox, &reg);
    // 全幅四边形共享对角 (2,2,1,3)；(4,1) 中心 ndc=(0.125,0.5) → v_color=(0.5625,0.75)
    assert_eq!(
        px(&data, 8, 4, 1),
        [143, 191, 0, 255],
        "四边形跨两个三角形索引取数插值"
    );
}

#[test]
fn test_webgl_read_pixels_domain_faces() {
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, _reg) = webgl_draw_sandbox();
    sandbox
        .execute(
            "var gl = __gl, r = [];\
             function err() { return gl.getError(); }\
             gl.drawArrays(gl.TRIANGLES, 0, 3);\
             var rp = new Uint8Array(4 * 2 * 2);\
             gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, rp);\
             r.push(err() === 0 ? 1 : 0);\
             r.push(rp[12] === 48 && rp[13] === 64 && rp[15] === 255 ? 1 : 0);\
             r.push(rp[0] === 16 && rp[1] === 21 ? 1 : 0);\
             var rp2 = new Uint8Array(16);\
             gl.readPixels(-1, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, rp2);\
             r.push(err() === 0 ? 1 : 0);\
             r.push(rp2[0] === 0 && rp2[4] === 16 ? 1 : 0);\
             gl.readPixels(0, 0, 2, 2, gl.RGB, gl.UNSIGNED_BYTE, rp);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.FLOAT, rp);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, null);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(4));\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.readPixels(0, 0, -2, 2, gl.RGBA, gl.UNSIGNED_BYTE, rp);\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "1111111111",
        "readPixels：RGBA/UNSIGNED_BYTE 之外 INVALID_OPERATION，null/param 负 INVALID_VALUE，\
         容量不足 INVALID_OPERATION；行翻转（GL 底行=位图顶行）；负 x 越界区域不动无错误（§6.16）"
    );
}

#[test]
fn test_webgl_fs_loop_increment_discard_mat2() {
    use zero_script_sandbox::Sandbox;
    let (mut sandbox, reg) = webgl_draw_sandbox();
    // mat2 变换 + discard：小三角 a_pos ∈ {-0.25,0.25}，u_m=2I → clip [-0.5,0.5]
    // → host bbox x[2,6] y[1.5,4.5]。首帧 u_color.r=0 全部 discard；次帧品红。
    sandbox
        .execute(
            "var gl = __gl;\
             var vb2 = gl.createBuffer();\
             gl.bindBuffer(gl.ARRAY_BUFFER, vb2);\
             gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-0.25, -0.25, 0.25, -0.25, -0.25, 0.25]), gl.STATIC_DRAW);\
             var vs2 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(vs2, 'attribute vec2 a_pos; uniform mat2 u_m;\
               void main() { gl_Position = vec4(u_m * a_pos, 0.0, 1.0); }');\
             gl.compileShader(vs2);\
             var fs2 = gl.createShader(gl.FRAGMENT_SHADER);\
             gl.shaderSource(fs2, 'precision mediump float; uniform vec4 u_color;\
               void main() { if (u_color.r < 0.5) { discard; } gl_FragColor = u_color; }');\
             gl.compileShader(fs2);\
             var p2 = gl.createProgram();\
             gl.attachShader(p2, vs2); gl.attachShader(p2, fs2); gl.linkProgram(p2);\
             gl.useProgram(p2);\
             globalThis.__p2_ok = gl.getProgramParameter(p2, gl.LINK_STATUS);\
             var loc2 = gl.getAttribLocation(p2, 'a_pos');\
             gl.enableVertexAttribArray(loc2);\
             gl.vertexAttribPointer(loc2, 2, gl.FLOAT, false, 0, 0);\
             gl.uniformMatrix2fv(gl.getUniformLocation(p2, 'u_m'), false, [2, 0, 0, 2]);\
             globalThis.__cloc = gl.getUniformLocation(p2, 'u_color');\
             gl.uniform4f(__cloc, 0, 0, 1, 0.5);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__p2_ok").unwrap().value,
        "true",
        "mat2 程序链接成功"
    );
    // 绘制前捕获初始位图（宿主画布初始内容非全零，不预设具体值）。
    let (_w, _h, before) = draw_bitmap(&mut sandbox, &reg);
    sandbox
        .execute(
            "__gl.drawArrays(__gl.TRIANGLES, 0, 3);\
             globalThis.__err1 = __gl.getError();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__err1").unwrap().value,
        "0",
        "discard 分支绘制无错误"
    );
    let (_w, _h, after_discard) = draw_bitmap(&mut sandbox, &reg);
    assert_eq!(
        after_discard, before,
        "全 discard 帧与绘制前逐字节一致（逐片段 discard 无写入）"
    );
    sandbox
        .execute(
            "__gl.uniform4f(__cloc, 1, 0, 1, 1);\
             __gl.drawArrays(__gl.TRIANGLES, 0, 3);\
             globalThis.__err2 = __gl.getError();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__err2").unwrap().value,
        "0",
        "品红绘制无错误"
    );
    let (_w, _h, data) = draw_bitmap(&mut sandbox, &reg);
    // bbox 内像素中心 x∈{2..5} y∈{2..4}。(3,3) 覆盖；u_color 原样输出品红。
    assert_eq!(px(&data, 8, 3, 3), [255, 0, 255, 255], "mat2 缩放后品红像素");
    assert_eq!(px(&data, 8, 2, 4), [255, 0, 255, 255], "bbox 左上角点在边上仍覆盖");
    // bbox 外像素保持绘制前内容（不预设宿主初始值）。
    assert_eq!(
        px(&data, 8, 6, 1),
        px(&before, 8, 6, 1),
        "bbox 外未写像素"
    );
    // FS for 循环 i++（§5.9 求值器自增）+ swizzle 写：切回全幅三角几何，
    // 循环累加 s=0.75 → c.a = 0.375 → 96。
    sandbox
        .execute(
            "var gl = __gl;\
             var fs3 = gl.createShader(gl.FRAGMENT_SHADER);\
             gl.shaderSource(fs3, 'precision mediump float; varying vec3 v_color;\
               void main() { float s = 0.0; for (int i = 0; i < 3; i++) { s += 0.25; }\
                 vec4 c = vec4(v_color, 1.0); c.a = s * 0.5; gl_FragColor = c; }');\
             gl.compileShader(fs3);\
             var vs3 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(vs3, __VS);\
             gl.compileShader(vs3);\
             var p3 = gl.createProgram();\
             gl.attachShader(p3, vs3); gl.attachShader(p3, fs3); gl.linkProgram(p3);\
             gl.useProgram(p3);\
             globalThis.__p3_ok = gl.getProgramParameter(p3, gl.LINK_STATUS);\
             gl.uniform2f(gl.getUniformLocation(p3, 'u_scale'), 1, 1);\
             gl.bindBuffer(gl.ARRAY_BUFFER, __vb);\
             gl.vertexAttribPointer(__loc, 2, gl.FLOAT, false, 0, 0);\
             gl.drawArrays(gl.TRIANGLES, 0, 3);\
             globalThis.__err3 = gl.getError();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__p3_ok").unwrap().value,
        "true",
        "for 循环程序链接成功"
    );
    assert_eq!(
        sandbox.execute("globalThis.__err3").unwrap().value,
        "0",
        "for 循环 i++ FS 绘制无错误"
    );
    let (_w, _h, data) = draw_bitmap(&mut sandbox, &reg);
    assert_eq!(
        px(&data, 8, 1, 4),
        [48, 64, 0, 96],
        "循环累加 s=0.75 → alpha 0.375 → 96"
    );
}
