// part69 — WebGL shader/program 管线（goal webgl-compat M3-S2，2026-10-22）：
// shader 生命周期（createShader 类型域校验 + compileShader 真编译——GLSL ES 1.0
// 前端校验 + COMPILE_STATUS/infoLog 面 + 编译失败无 GL 错误）、program link
// （attach/detach INVALID_OPERATION 域 + 双阶段齐备 + 编译前置 + bindAttribLocation
// 融合/冲突 + ATTACHED_SHADERS）、反射面（getActiveAttrib/getActiveUniform 声明序
// + 活跃剔除 + 数组 'u[0]' 命名面 + 跨阶段 uniform 合并 + getAttribLocation）、
// uniform 加载族（类型失配/采样器 1i-only/NaN INVALID_VALUE/transpose false-only/
// 容量越界/错 program/重链 stamp 失效 + getUniform 值面 + null location 静默）、
// GLSL 编译错误 infoLog 行号对位。
// 沙盒沿用 part67 `webgl_sandbox`。

#[test]
fn test_webgl_shader_compile_lifecycle() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             function err() { return gl.getError(); }\
             var vs = gl.createShader(gl.VERTEX_SHADER);\
             r.push(vs && typeof vs === 'object' ? 1 : 0);\
             r.push(gl.getShaderParameter(vs, gl.SHADER_TYPE) === gl.VERTEX_SHADER ? 1 : 0);\
             r.push(gl.createShader(999) === null && err() === gl.INVALID_ENUM ? 1 : 0);\
             var threw = 0;\
             try { gl.compileShader({}); } catch (e) { threw = e.constructor.name === 'TypeError' ? 1 : 0; }\
             r.push(threw);\
             gl.shaderSource(vs, 'attribute vec4 a_p; void main() { gl_Position = a_p; }');\
             gl.compileShader(vs);\
             r.push(gl.getShaderParameter(vs, gl.COMPILE_STATUS) === true ? 1 : 0);\
             r.push(gl.getShaderInfoLog(vs) === '' ? 1 : 0);\
             var bad = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(bad, 'void main() { gl_Position = nope; }');\
             gl.compileShader(bad);\
             var log = gl.getShaderInfoLog(bad);\
             r.push(gl.getShaderParameter(bad, gl.COMPILE_STATUS) === false ? 1 : 0);\
             r.push(log.indexOf('ERROR: 0:1:') === 0 && log.indexOf('nope') > 0 ? 1 : 0);\
             var bad2 = gl.createShader(gl.FRAGMENT_SHADER);\
             gl.shaderSource(bad2, 'attribute vec4 a_x; void main() { gl_FragColor = a_x; }');\
             gl.compileShader(bad2);\
             r.push(gl.getShaderParameter(bad2, gl.COMPILE_STATUS) === false && gl.getShaderInfoLog(bad2).indexOf('attribute is only available') >= 0 ? 1 : 0);\
             var bad3 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(bad3, 'void main() { float x = 1.0f; }');\
             gl.compileShader(bad3);\
             r.push(gl.getShaderParameter(bad3, gl.COMPILE_STATUS) === false ? 1 : 0);\
             var nom = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(nom, 'attribute vec4 a_x;');\
             gl.compileShader(nom);\
             r.push(gl.getShaderParameter(nom, gl.COMPILE_STATUS) === false && gl.getShaderInfoLog(nom).indexOf('missing main()') >= 0 ? 1 : 0);\
             var fresh = gl.createShader(gl.VERTEX_SHADER);\
             r.push(gl.getShaderParameter(fresh, gl.COMPILE_STATUS) === false ? 1 : 0);\
             gl.deleteShader(fresh);\
             r.push(gl.getShaderParameter(fresh, gl.DELETE_STATUS) === true && gl.isShader(fresh) === false ? 1 : 0);\
             r.push(err() === 0 ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "11111111111111",
        "shader 生命周期：类型域校验 + 真编译 COMPILE_STATUS/infoLog + 错误路径族 + delete 面 + 编译失败无 GL 错误"
    );
}

#[test]
fn test_webgl_program_link_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             function err() { return gl.getError(); }\
             var vs = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(vs, 'attribute vec4 a_p; uniform mat4 u_m; varying vec4 v_c; void main() { v_c = a_p; gl_Position = u_m * a_p; }');\
             gl.compileShader(vs);\
             var fs = gl.createShader(gl.FRAGMENT_SHADER);\
             gl.shaderSource(fs, 'precision mediump float; uniform vec4 u_color; varying vec4 v_c; void main() { gl_FragColor = v_c * u_color; }');\
             gl.compileShader(fs);\
             var prog = gl.createProgram();\
             r.push(gl.getProgramParameter(prog, gl.LINK_STATUS) === false ? 1 : 0);\
             gl.attachShader(prog, vs); gl.attachShader(prog, fs);\
             r.push(gl.getProgramParameter(prog, gl.ATTACHED_SHADERS) === 2 ? 1 : 0);\
             gl.attachShader(prog, vs);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.getError();\
             r.push(gl.getProgramParameter(prog, gl.ATTACHED_SHADERS) === 2 ? 1 : 0);\
             gl.linkProgram(prog);\
             r.push(gl.getProgramParameter(prog, gl.LINK_STATUS) === true ? 1 : 0);\
             r.push(gl.getProgramInfoLog(prog) === '' ? 1 : 0);\
             r.push(err() === 0 ? 1 : 0);\
             var p2 = gl.createProgram();\
             gl.linkProgram(p2);\
             r.push(gl.getProgramParameter(p2, gl.LINK_STATUS) === false && gl.getProgramInfoLog(p2).indexOf('no vertex shader') === 0 ? 1 : 0);\
             r.push(err() === 0 ? 1 : 0);\
             var p3 = gl.createProgram();\
             var vsRaw = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(vsRaw, 'void main() {}');\
             gl.attachShader(p3, vsRaw);\
             var fs3 = gl.createShader(gl.FRAGMENT_SHADER);\
             gl.shaderSource(fs3, 'void main() {}');\
             gl.compileShader(fs3);\
             gl.attachShader(p3, fs3);\
             gl.linkProgram(p3);\
             r.push(gl.getProgramParameter(p3, gl.LINK_STATUS) === false && gl.getProgramInfoLog(p3).indexOf('vertex shader has not been compiled') >= 0 ? 1 : 0);\
             var p4 = gl.createProgram();\
             var shX = gl.createShader(gl.VERTEX_SHADER);\
             gl.detachShader(p4, shX);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.getError();\
             gl.detachShader(prog, fs);\
             r.push(gl.getProgramParameter(prog, gl.ATTACHED_SHADERS) === 1 ? 1 : 0);\
             gl.attachShader(prog, fs);\
             gl.linkProgram(prog);\
             r.push(gl.getProgramParameter(prog, gl.LINK_STATUS) === true ? 1 : 0);\
             gl.bindAttribLocation(prog, 16, 'a_p');\
             r.push(err() === gl.INVALID_VALUE ? 1 : 0);\
             gl.getError();\
             gl.bindAttribLocation(prog, 0, 'gl_Position');\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.getError();\
             gl.bindAttribLocation(prog, 3, 'a_p');\
             gl.linkProgram(prog);\
             r.push(gl.getProgramParameter(prog, gl.LINK_STATUS) === true && gl.getAttribLocation(prog, 'a_p') === 3 ? 1 : 0);\
             var vs2 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(vs2, 'attribute vec4 a_p; attribute vec3 a_n; void main() { gl_Position = a_p + vec4(a_n, 0.0); }');\
             gl.compileShader(vs2);\
             var p5 = gl.createProgram();\
             gl.attachShader(p5, vs2); gl.attachShader(p5, fs);\
             gl.bindAttribLocation(p5, 1, 'a_p');\
             gl.bindAttribLocation(p5, 1, 'a_n');\
             gl.linkProgram(p5);\
             r.push(gl.getProgramParameter(p5, gl.LINK_STATUS) === false && gl.getProgramInfoLog(p5).indexOf('same index') >= 0 ? 1 : 0);\
             var p6 = gl.createProgram();\
             gl.attachShader(p6, vs); gl.attachShader(p6, vs2); gl.attachShader(p6, fs);\
             gl.linkProgram(p6);\
             r.push(gl.getProgramParameter(p6, gl.LINK_STATUS) === false ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "111111111111111111",
        "program link：attach 域校验 + 双阶段/编译前置 + 链接失败无 GL 错误 + bindAttribLocation 域/融合/冲突 + 重链"
    );
}

#[test]
fn test_webgl_program_reflection_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             var vs = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(vs, 'attribute vec3 a_pos; attribute vec2 a_uv; uniform mat4 u_mvp; uniform float u_common; uniform float u_dead; varying vec2 v_uv; void main() { v_uv = a_uv; gl_Position = u_mvp * vec4(a_pos, 1.0) + u_common; }');\
             gl.compileShader(vs);\
             var fs = gl.createShader(gl.FRAGMENT_SHADER);\
             gl.shaderSource(fs, 'precision mediump float; uniform vec4 u_color; uniform vec3 u_lights[4]; uniform float u_common; varying vec2 v_uv; void main() { gl_FragColor = u_color + u_common + vec4(v_uv, 0.0, 1.0) + vec4(u_lights[0], 1.0); }');\
             gl.compileShader(fs);\
             var prog = gl.createProgram();\
             gl.attachShader(prog, vs); gl.attachShader(prog, fs);\
             gl.linkProgram(prog);\
             r.push(gl.getProgramParameter(prog, gl.LINK_STATUS) === true ? 1 : 0);\
             r.push(gl.getProgramParameter(prog, gl.ACTIVE_UNIFORMS) === 4 ? 1 : 0);\
             r.push(gl.getProgramParameter(prog, gl.ACTIVE_ATTRIBUTES) === 2 ? 1 : 0);\
             var au = gl.getActiveUniform(prog, 0);\
             r.push(au.name === 'u_mvp' && au.type === gl.FLOAT_MAT4 && au.size === 1 ? 1 : 0);\
             var au1 = gl.getActiveUniform(prog, 1);\
             r.push(au1.name === 'u_common' && au1.type === gl.FLOAT ? 1 : 0);\
             var au2 = gl.getActiveUniform(prog, 2);\
             r.push(au2.name === 'u_color' && au2.type === gl.FLOAT_VEC4 ? 1 : 0);\
             var au3 = gl.getActiveUniform(prog, 3);\
             r.push(au3.name === 'u_lights[0]' && au3.type === gl.FLOAT_VEC3 && au3.size === 4 ? 1 : 0);\
             r.push(gl.getActiveUniform(prog, 4) === null && gl.getError() === gl.INVALID_VALUE ? 1 : 0);\
             gl.getError();\
             var aa = gl.getActiveAttrib(prog, 0);\
             r.push(aa.name === 'a_pos' && aa.type === gl.FLOAT_VEC3 ? 1 : 0);\
             var aa1 = gl.getActiveAttrib(prog, 1);\
             r.push(aa1.name === 'a_uv' && aa1.type === gl.FLOAT_VEC2 ? 1 : 0);\
             r.push(gl.getAttribLocation(prog, 'a_pos') === 0 && gl.getAttribLocation(prog, 'a_uv') === 1 ? 1 : 0);\
             r.push(gl.getAttribLocation(prog, 'a_none') === -1 && gl.getError() === 0 ? 1 : 0);\
             var p2 = gl.createProgram();\
             r.push(gl.getAttribLocation(p2, 'x') === -1 && gl.getError() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.getError();\
             r.push(gl.getActiveAttrib(p2, 0) === null && gl.getError() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.getError();\
             r.push(gl.getUniformLocation(p2, 'x') === null && gl.getError() === gl.INVALID_OPERATION ? 1 : 0);\
             gl.getError();\
             r.push(gl.getActiveUniform(prog, 0) instanceof WebGLActiveInfo ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "1111111111111111",
        "反射面：ACTIVE_* 计数 + 声明序/活跃剔除 + 数组 'u[0]' 命名/size + 跨阶段合并 + getAttribLocation + 未链接 INVALID_OPERATION + ActiveInfo 原型"
    );
}

#[test]
fn test_webgl_uniform_op_face() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             function err() { return gl.getError(); }\
             var vs = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(vs, 'attribute vec4 a_p; uniform mat4 u_m; uniform vec4 u_v; uniform float u_f; uniform int u_i; uniform bool u_b; uniform vec3 u_arr[2]; void main() { gl_Position = u_m * a_p + u_v + vec4(u_f) + vec4(u_arr[0], 0.0) + vec4(u_i) + vec4(u_b); }');\
             gl.compileShader(vs);\
             var fs = gl.createShader(gl.FRAGMENT_SHADER);\
             gl.shaderSource(fs, 'precision mediump float; uniform sampler2D u_s; uniform vec4 u_c; void main() { gl_FragColor = texture2D(u_s, vec2(0.5, 0.5)) + u_c; }');\
             gl.compileShader(fs);\
             var prog = gl.createProgram();\
             gl.attachShader(prog, vs); gl.attachShader(prog, fs);\
             gl.linkProgram(prog);\
             gl.useProgram(prog);\
             var locM = gl.getUniformLocation(prog, 'u_m');\
             var locV = gl.getUniformLocation(prog, 'u_v');\
             var locF = gl.getUniformLocation(prog, 'u_f');\
             var locI = gl.getUniformLocation(prog, 'u_i');\
             var locB = gl.getUniformLocation(prog, 'u_b');\
             var locA0 = gl.getUniformLocation(prog, 'u_arr');\
             var locA0b = gl.getUniformLocation(prog, 'u_arr[0]');\
             var locA1 = gl.getUniformLocation(prog, 'u_arr[1]');\
             r.push(locM && locM instanceof WebGLUniformLocation ? 1 : 0);\
             r.push(gl.getUniformLocation(prog, 'u_m') === locM ? 1 : 0);\
             gl.uniform3fv(locA0, [4, 5, 6]);\
             var viaAlias = gl.getUniform(prog, locA0b);\
             r.push(viaAlias[0] === 4 && viaAlias[2] === 6 ? 1 : 0);\
             r.push(locA1 !== locA0 && locA1 !== null ? 1 : 0);\
             r.push(gl.getUniformLocation(prog, 'u_none') === null && err() === 0 ? 1 : 0);\
             gl.getError();\
             r.push(gl.getUniformLocation(prog, 'u_arr[2]') === null ? 1 : 0);\
             gl.uniformMatrix4fv(locM, false, [1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1]);\
             r.push(err() === 0 ? 1 : 0);\
             var m = gl.getUniform(prog, locM);\
             r.push(m instanceof Float32Array && m.length === 16 && m[0] === 1 && m[15] === 1 ? 1 : 0);\
             gl.uniform4f(locV, 0.25, 0.5, 0.75, 1.0);\
             var v4 = gl.getUniform(prog, locV);\
             r.push(v4 instanceof Float32Array && v4[1] === 0.5 && v4[3] === 1 ? 1 : 0);\
             gl.uniform1f(locF, 2.5);\
             r.push(gl.getUniform(prog, locF) === 2.5 ? 1 : 0);\
             gl.uniform1i(locI, 7);\
             r.push(gl.getUniform(prog, locI) === 7 ? 1 : 0);\
             gl.uniform1i(locB, 1);\
             r.push(gl.getUniform(prog, locB) === true ? 1 : 0);\
             gl.uniform1i(locB, 0);\
             r.push(gl.getUniform(prog, locB) === false ? 1 : 0);\
             gl.uniform3fv(locA1, [9, 8, 7]);\
             var a1v = gl.getUniform(prog, locA1);\
             r.push(a1v instanceof Float32Array && a1v[0] === 9 && a1v[2] === 7 ? 1 : 0);\
             var a0v = gl.getUniform(prog, locA0);\
             r.push(a0v[0] === 4 && a0v[1] === 5 && a0v[2] === 6 ? 1 : 0);\
             gl.getError();\
             gl.uniform1f(locV, 1.0); r.push(err() === gl.INVALID_OPERATION ? 1 : 0); gl.getError();\
             gl.uniform2f(locF, 1, 2); r.push(err() === gl.INVALID_OPERATION ? 1 : 0); gl.getError();\
             gl.uniformMatrix3fv(locM, false, new Float32Array(9)); r.push(err() === gl.INVALID_OPERATION ? 1 : 0); gl.getError();\
             var locS = gl.getUniformLocation(prog, 'u_s');\
             gl.uniform1f(locS, 0.0); r.push(err() === gl.INVALID_OPERATION ? 1 : 0); gl.getError();\
             gl.uniform1i(locS, 0); r.push(err() === 0 && gl.getUniform(prog, locS) === 0 ? 1 : 0);\
             gl.uniform1f(locF, NaN); r.push(err() === gl.INVALID_VALUE ? 1 : 0); gl.getError();\
             gl.uniformMatrix4fv(locM, true, new Float32Array(16)); r.push(err() === gl.INVALID_VALUE ? 1 : 0); gl.getError();\
             gl.uniform3fv(locA0, [1,2,3, 4,5,6, 7,8,9]); r.push(err() === gl.INVALID_OPERATION ? 1 : 0); gl.getError();\
             var prog2 = gl.createProgram();\
             gl.useProgram(prog2);\
             gl.uniform1f(locF, 1.0); r.push(err() === gl.INVALID_OPERATION ? 1 : 0); gl.getError();\
             gl.useProgram(prog);\
             gl.uniform1f(locF, 4.5);\
             r.push(gl.getUniform(prog, locF) === 4.5 ? 1 : 0);\
             gl.linkProgram(prog);\
             gl.uniform1f(locF, 9.5);\
             r.push(err() === gl.INVALID_OPERATION ? 1 : 0); gl.getError();\
             var locF2 = gl.getUniformLocation(prog, 'u_f');\
             r.push(locF2 !== locF ? 1 : 0);\
             gl.uniform1f(locF2, 9.5);\
             r.push(gl.getUniform(prog, locF2) === 9.5 ? 1 : 0);\
             gl.uniform4fv(null, [1,2,3,4]); gl.uniformMatrix2fv(null, false, new Float32Array(4));\
             r.push(err() === 0 ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "11111111111111111111111111111",
        "uniform 加载族：location 对象同一性/数组寻址 + getUniform 值/初值 + 类型失配/采样器 1i-only/NaN/transpose/容量/错 program/重链失效 + null 静默"
    );
}

#[test]
fn test_webgl_glsl_error_info_lines() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = webgl_sandbox().0;
    sandbox
        .execute(
            "var r = []; var gl = __gl;\
             var sh = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(sh, 'attribute vec4 a;\\nvoid main() {\\n  float ok = 1.0;\\n  gl_Position = missing;\\n}');\
             gl.compileShader(sh);\
             var log = gl.getShaderInfoLog(sh);\
             r.push(log.indexOf('ERROR: 0:4:') >= 0 ? 1 : 0);\
             r.push(log.indexOf('undeclared identifier') >= 0 ? 1 : 0);\
             var sh2 = gl.createShader(gl.FRAGMENT_SHADER);\
             gl.shaderSource(sh2, 'precision mediump float;\\n\\nvoid main() {\\n  if (1.0 {\\n}');\
             gl.compileShader(sh2);\
             r.push(gl.getShaderInfoLog(sh2).indexOf('ERROR: 0:4:') >= 0 ? 1 : 0);\
             var sh3 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(sh3, 'void main() { break; }');\
             gl.compileShader(sh3);\
             r.push(gl.getShaderInfoLog(sh3).indexOf('outside of loop') >= 0 ? 1 : 0);\
             var sh4 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(sh4, 'void f() { return 1.0; } void main() { f(); }');\
             gl.compileShader(sh4);\
             r.push(gl.getShaderInfoLog(sh4).indexOf('void function cannot return') >= 0 ? 1 : 0);\
             var sh5 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(sh5, 'uniform vec4 gl_Mine; void main() {}');\
             gl.compileShader(sh5);\
             r.push(gl.getShaderInfoLog(sh5).indexOf('gl_') >= 0 ? 1 : 0);\
             var sh6 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(sh6, 'uniform float u_d; uniform float u_d; void main() { gl_Position = vec4(u_d); }');\
             gl.compileShader(sh6);\
             r.push(gl.getShaderInfoLog(sh6).indexOf('duplicate declaration') >= 0 ? 1 : 0);\
             var sh7 = gl.createShader(gl.VERTEX_SHADER);\
             gl.shaderSource(sh7, 'void main() { float x = ; }');\
             gl.compileShader(sh7);\
             r.push(gl.getShaderInfoLog(sh7).indexOf('expected') >= 0 ? 1 : 0);\
             r.push(gl.getError() === 0 ? 1 : 0);\
             globalThis.__r = r.join('');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "111111111",
        "GLSL 编译错误 infoLog：行号对位（ERROR: 0:N:）+ 未声明/break 域/void 返回值/gl_ 保留/重复声明/语法错误信息 + 编译失败无 GL 错误"
    );
}
