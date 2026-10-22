// part68 — WebGL 查询与事件面（goal webgl-compat M3-S1，2026-10-22）：全表
// getParameter（WebGL 1.0 §5.14.6 + GLES2 Table 6.9/6.10/6.18 + WebGL §6.4
// 最小上限；pin rev webgl1.idl 常量值为准——DEPTH_WRITEMASK 0x0B72 与桌面 GL
// 表位差异等）、getContextAttributes（§5.14.2）、isEnabled/enable/disable cap
// 校验（§5.14.3）、getShaderPrecisionFormat（§5.14.5 + GLES2 §6.2 Table 6.6）、
// 状态设置族往返（depthMask/frontFace/cullFace/depthFunc/depthRange/lineWidth/
// sampleCoverage/blendColor/blendFunc/blendEquation/stencilFunc/pixelStorei）、
// WEBGL_lose_context 上下文丢失/恢复（§5.15——webglcontextlost 可取消 +
// CONTEXT_LOST_WEBGL 首查 + 丢失态方法旁路 + webglcontextrestored）。
// 沙盒沿用 part67 `webgl_sandbox`（M2 先例——body 预置 canvas + getContext('webgl')）。

#[test]
fn test_webgl_context_attributes_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             var a = gl.getContextAttributes();\
             r.push(a && typeof a === 'object' ? 1 : 0);\
             r.push(a.alpha === true && a.depth === true && a.stencil === false && a.antialias === true ? 1 : 0);\
             r.push(a.premultipliedAlpha === true && a.preserveDrawingBuffer === false ? 1 : 0);\
             r.push(a.powerPreference === 'default' && a.failIfMajorPerformanceCaveat === false ? 1 : 0);\
             r.push(a.desynchronized === false ? 1 : 0);\
             a.alpha = false; a.depth = 'mutated';\
             var b = gl.getContextAttributes();\
             r.push(b.alpha === true && b.depth === true ? 1 : 0);\
             r.push(gl.isContextLost() === false ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "1111111",
        "getContextAttributes 默认九项 + 返回副本隔离 + 未丢失"
    );
}

#[test]
fn test_webgl_parameter_table_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             function p(x) { return gl.getParameter(x); }\
             r.push(p(gl.VENDOR) === 'ZeroWeb' && p(gl.RENDERER) === 'ZeroWeb' ? 1 : 0);\
             r.push(p(gl.VERSION) === 'WebGL 1.0' && p(gl.SHADING_LANGUAGE_VERSION) === 'WebGL GLSL ES 1.0' ? 1 : 0);\
             var v = p(gl.VIEWPORT);\
             r.push(Array.isArray(v) && v[0] === 0 && v[1] === 0 && v[2] === 8 && v[3] === 6 ? 1 : 0);\
             var sb = p(gl.SCISSOR_BOX);\
             r.push(sb[0] === 0 && sb[1] === 0 && sb[2] === 8 && sb[3] === 6 ? 1 : 0);\
             r.push(p(gl.SUBPIXEL_BITS) === 4 && p(gl.RED_BITS) === 8 && p(gl.MAX_TEXTURE_SIZE) === 4096 ? 1 : 0);\
             r.push(p(gl.MAX_VERTEX_ATTRIBS) === 16 && p(gl.MAX_FRAGMENT_UNIFORM_VECTORS) === 16 ? 1 : 0);\
             r.push(p(gl.MAX_COMBINED_TEXTURE_IMAGE_UNITS) === 32 ? 1 : 0);\
             var mvd = p(gl.MAX_VIEWPORT_DIMS);\
             r.push(mvd instanceof Int32Array && mvd[0] === 16384 ? 1 : 0);\
             r.push(p(gl.DEPTH_FUNC) === gl.LESS && p(gl.DEPTH_WRITEMASK) === true ? 1 : 0);\
             r.push(p(gl.FRONT_FACE) === gl.CCW && p(gl.CULL_FACE_MODE) === gl.BACK ? 1 : 0);\
             r.push(p(gl.LINE_WIDTH) === 1 && p(gl.STENCIL_CLEAR_VALUE) === 0 ? 1 : 0);\
             r.push(p(gl.STENCIL_VALUE_MASK) === 4294967295 && p(gl.STENCIL_WRITEMASK) === 4294967295 ? 1 : 0);\
             var cc = p(gl.COLOR_CLEAR_VALUE);\
             r.push(cc instanceof Float32Array && cc[0] === 0 && cc[1] === 0 && cc[2] === 0 && cc[3] === 0 ? 1 : 0);\
             r.push(gl.getError() === 0 ? 1 : 0);\
             var fmts = p(gl.COMPRESSED_TEXTURE_FORMATS);\
             r.push(fmts instanceof Uint32Array && fmts.length === 0 ? 1 : 0);\
             gl.getExtension('WEBGL_compressed_texture_s3tc');\
             var fmts2 = p(gl.COMPRESSED_TEXTURE_FORMATS);\
             r.push(fmts2.length === 4 && fmts2[0] === 0x83F0 && fmts2[3] === 0x83F3 ? 1 : 0);\
             r.push(p(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL) === gl.BROWSER_DEFAULT_WEBGL ? 1 : 0);\
             r.push(p(gl.UNPACK_FLIP_Y_WEBGL) === false && p(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL) === false ? 1 : 0);\
             r.push(p(0x1234) === null && gl.getError() === gl.INVALID_ENUM ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "1111111111111111111",
        "getParameter 全表面：身份/位面/上限最小值/typed array 形/状态初值/压缩格式表/WebGL 专有像素参数/未知 pname INVALID_ENUM"
    );
}

#[test]
fn test_webgl_state_setters_parameter_roundtrip() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             gl.depthMask(false); r.push(gl.getParameter(gl.DEPTH_WRITEMASK) === false ? 1 : 0); gl.depthMask(true);\
             gl.frontFace(gl.CW); r.push(gl.getParameter(gl.FRONT_FACE) === gl.CW ? 1 : 0);\
             gl.frontFace(0x1234); r.push(gl.getError() === gl.INVALID_ENUM && gl.getParameter(gl.FRONT_FACE) === gl.CW ? 1 : 0);\
             gl.cullFace(gl.FRONT); r.push(gl.getParameter(gl.CULL_FACE_MODE) === gl.FRONT ? 1 : 0);\
             gl.depthFunc(gl.GEQUAL); r.push(gl.getParameter(gl.DEPTH_FUNC) === gl.GEQUAL ? 1 : 0);\
             gl.depthFunc(0x1234); r.push(gl.getError() === gl.INVALID_ENUM ? 1 : 0);\
             gl.depthRange(0.25, 0.75); var dr = gl.getParameter(gl.DEPTH_RANGE);\
             r.push(dr[0] === 0.25 && dr[1] === 0.75 ? 1 : 0);\
             gl.depthRange(-1, 5); var dr2 = gl.getParameter(gl.DEPTH_RANGE);\
             r.push(dr2[0] === 0 && dr2[1] === 1 ? 1 : 0);\
             gl.lineWidth(8); r.push(gl.getParameter(gl.LINE_WIDTH) === 1 ? 1 : 0);\
             gl.lineWidth(0); r.push(gl.getError() === gl.INVALID_VALUE ? 1 : 0);\
             gl.sampleCoverage(0.5, true);\
             r.push(gl.getParameter(gl.SAMPLE_COVERAGE_VALUE) === 0.5 && gl.getParameter(gl.SAMPLE_COVERAGE_INVERT) === true ? 1 : 0);\
             gl.blendColor(0.1, 0.2, 0.3, 0.4); var bc = gl.getParameter(gl.BLEND_COLOR);\
             r.push(Math.abs(bc[2] - 0.3) < 1e-6 ? 1 : 0);\
             gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);\
             r.push(gl.getParameter(gl.BLEND_SRC_RGB) === gl.SRC_ALPHA && gl.getParameter(gl.BLEND_DST_ALPHA) === gl.ONE_MINUS_SRC_ALPHA ? 1 : 0);\
             gl.blendFunc(0x1234, gl.ZERO); r.push(gl.getError() === gl.INVALID_ENUM ? 1 : 0);\
             gl.blendEquation(gl.FUNC_SUBTRACT); r.push(gl.getParameter(gl.BLEND_EQUATION) === gl.FUNC_SUBTRACT ? 1 : 0);\
             gl.blendEquation(gl.FUNC_ADD);\
             gl.stencilFunc(gl.GEQUAL, 3, 0x0F);\
             r.push(gl.getParameter(gl.STENCIL_FUNC) === gl.GEQUAL && gl.getParameter(gl.STENCIL_REF) === 3 && gl.getParameter(gl.STENCIL_VALUE_MASK) === 15 ? 1 : 0);\
             gl.enable(gl.BLEND); r.push(gl.isEnabled(gl.BLEND) === true && gl.getParameter(gl.BLEND) === true ? 1 : 0);\
             gl.disable(gl.BLEND); r.push(gl.isEnabled(gl.BLEND) === false ? 1 : 0);\
             gl.enable(0x1234); r.push(gl.getError() === gl.INVALID_ENUM && gl.isEnabled(0x1234) === false ? 1 : 0);\
             gl.getError(); /* 排空上一断言消费的 sticky 首错 */\
             gl.pixelStorei(gl.UNPACK_ALIGNMENT, 3); r.push(gl.getError() === gl.INVALID_VALUE ? 1 : 0);\
             gl.pixelStorei(gl.UNPACK_ALIGNMENT, 8); r.push(gl.getError() === 0 ? 1 : 0);\
             var f = gl.getShaderPrecisionFormat(gl.VERTEX_SHADER, gl.HIGH_FLOAT);\
             r.push(f && f.rangeMin === 127 && f.rangeMax === 127 && f.precision === 23 ? 1 : 0);\
             var fi = gl.getShaderPrecisionFormat(gl.FRAGMENT_SHADER, gl.LOW_INT);\
             r.push(fi && fi.rangeMin === 31 && fi.rangeMax === 30 && fi.precision === 0 ? 1 : 0);\
             r.push(gl.getShaderPrecisionFormat(0x1234, gl.HIGH_FLOAT) === null && gl.getError() === gl.INVALID_ENUM ? 1 : 0);\
             r.push(gl.getShaderPrecisionFormat(gl.VERTEX_SHADER, 0x1234) === null && gl.getError() === gl.INVALID_ENUM ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "1111111111111111111111111",
        "状态设置族→getParameter 往返 + 域校验 INVALID_ENUM/INVALID_VALUE + clamp + cap 校验 + UNPACK_ALIGNMENT + 精度格式表"
    );
}

#[test]
fn test_webgl_context_lost_restored_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             var lostEvents = []; var restoredEvents = 0;\
             var onLost = function (ev) {\
               lostEvents.push(ev);\
               r.push('evt=' + ev.constructor.name + ':' + ev.cancelable + ':' + typeof ev.statusMessage);\
               ev.preventDefault();\
             };\
             __cv.addEventListener('webglcontextlost', onLost);\
             __cv.addEventListener('webglcontextrestored', function () { restoredEvents++; });\
             var lc = gl.getExtension('WEBGL_lose_context');\
             r.push(lc && typeof lc.loseContext === 'function' && typeof lc.restoreContext === 'function' ? 1 : 0);\
             r.push(gl.getExtension('WEBGL_lose_context') === lc ? 1 : 0);\
             lc.loseContext();\
             r.push(gl.isContextLost() === true ? 1 : 0);\
             r.push(lostEvents.length === 1 ? 1 : 0);\
             r.push(gl.getError() === gl.CONTEXT_LOST_WEBGL ? 1 : 0);\
             r.push(gl.getError() === 0 ? 1 : 0);\
             r.push(gl.getContextAttributes() === null ? 1 : 0);\
             gl.clearColor(1, 0, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT);\
             r.push(gl.getError() === 0 ? 1 : 0);\
             lc.loseContext();\
             r.push(lostEvents.length === 1 ? 1 : 0);\
             lc.restoreContext();\
             r.push(gl.isContextLost() === true ? 1 : 0);\
             globalThis.__r2 = r.join('|') + '#' + restoredEvents;",
        )
        .unwrap();
    let r2 = sandbox.execute("globalThis.__r2").unwrap().value;
    // preventDefault 已抑制 restore：restoreContext no-op、restored 事件 0 次。
    // 注：listener 内的 push 发生在 loseContext() 调用中，先于其后的 isContextLost push。
    assert_eq!(
        r2,
        "1|1|evt=WebGLContextEvent:true:string|1|1|1|1|1|1|1|1#0",
        "丢失事件（可取消 WebGLContextEvent）+ CONTEXT_LOST_WEBGL 首查 + 丢失态旁路 + 幂等 lose + preventDefault 抑制恢复"
    );
    // 恢复路径需新上下文（§5.15.3：被 preventDefault 抑制后 restoreContext
    // 对该上下文永久失效，已由上段 isContextLost===true 断言）；此处用沙盒
    // 预置的第二 canvas（part04 getElementById 代理路径）走完整 lost→restored 链。
    sandbox
        .execute(
            "var cvB = document.getElementById('cv2');\
             var glB = cvB.getContext('webgl');\
             var lcB = glB.getExtension('WEBGL_lose_context');\
             var restoredB = 0; var lostB = 0;\
             cvB.addEventListener('webglcontextlost', function () { lostB++; });\
             cvB.addEventListener('webglcontextrestored', function () { restoredB++; });\
             lcB.loseContext();\
             lcB.restoreContext();\
             globalThis.__r3 = glB.isContextLost() === false && restoredB === 1 && lostB === 1 && glB.getError() === 0 && gl.isContextLost() === true ? 1 : 0;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3").unwrap().value,
        "1",
        "未被取消的丢失经 restoreContext 恢复 + webglcontextrestored 事件 + 抑制态上下文保持丢失"
    );
}
