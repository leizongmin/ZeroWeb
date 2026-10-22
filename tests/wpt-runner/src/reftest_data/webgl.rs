//! webgl 本地自建 inline reftest（goal webgl-compat M2，2026-10-22 用户裁决②
//! 标尺①）——getContext('webgl') clear() 颜色清屏光栅化（经既有 host 2d 位图，
//! GL 底左原点换算 + scissor 裁剪）。缺陷态（上下文缺席 / clear 不落位图 /
//! scissor 换算错位）与 ref 像素不等。
//!
//! spec：https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
//! （clear / scissor——清屏仅写 scissor 矩形，GL 窗口坐标原点在左下）。
//! 文件孪生（make reftest-upstream 域）：
//! tests/wpt-runner/local-reftests/webgl/clear-{solid,scissor}-zw-00{1,2}*.html。

use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── clear 全幅清色 ──
    // 主臂：canvas 100×50 getContext('webgl') + clearColor 绿 + clear(COLOR_BUFFER_BIT)
    // → drawing buffer 全绿，经 data-zw-canvas-ctx 桥接出页面图元。
    // ref 臂：同尺寸绿块 div（display:block 对齐 canvas 几何）。
    InlineReftestDef {
        id: "webgl/clear-solid-zw-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><canvas id=\"cv\" width=\"100\" height=\"50\" style=\"display:block\"></canvas><script>var gl = document.getElementById('cv').getContext('webgl'); gl.clearColor(0, 0.502, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT);</script></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:rgb(0,128,0)\"></div></body></html>",
        is_match: true,
    },
    // ── clear + scissor 裁剪清屏 ──
    // 主臂：全清蓝后 enable(SCISSOR_TEST) + scissor(0,0,50,25)（GL 底左原点——
    // 左下 50×25）清红 → 位图上半全蓝、左下红、右下蓝。
    // ref 臂：上条 100×25 蓝 + 下条 flex 左红右蓝（几何逐格对应）。
    InlineReftestDef {
        id: "webgl/clear-scissor-zw-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><canvas id=\"cv\" width=\"100\" height=\"50\" style=\"display:block\"></canvas><script>var gl = document.getElementById('cv').getContext('webgl'); gl.clearColor(0, 0, 1, 1); gl.clear(gl.COLOR_BUFFER_BIT); gl.enable(gl.SCISSOR_TEST); gl.scissor(0, 0, 50, 25); gl.clearColor(1, 0, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT);</script></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:25px;background:rgb(0,0,255)\"></div><div style=\"display:flex;width:100px;height:25px\"><div style=\"width:50px;height:25px;background:rgb(255,0,0)\"></div><div style=\"width:50px;height:25px;background:rgb(0,0,255)\"></div></div></body></html>",
        is_match: true,
    },
    // ── drawElements 全幅四边形实心蓝（M3-S4，2026-10-23）──
    // 主臂：GLSL program（VS a_pos 直通 clip，FS uniform 纯色）+ drawElements
    // UNSIGNED_SHORT 两三角形覆盖全 canvas → draw 管线端到端（索引取数/图元
    // 装配/重心光栅/putImageData 呈现）任一环缺席即与 ref 不等。
    // ref 臂：同尺寸纯蓝 div。文件孪生：
    // tests/wpt-runner/local-reftests/webgl/draw-quad-solid-zw-003.html。
    InlineReftestDef {
        id: "webgl/draw-quad-solid-zw-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><canvas id=\"cv\" width=\"100\" height=\"50\" style=\"display:block\"></canvas><script>var gl = document.getElementById('cv').getContext('webgl'); var vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(vs, 'attribute vec2 a_pos; void main() { gl_Position = vec4(a_pos, 0.0, 1.0); }'); gl.compileShader(vs); var fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(fs, 'precision mediump float; uniform vec4 u_color; void main() { gl_FragColor = u_color; }'); gl.compileShader(fs); var p = gl.createProgram(); gl.attachShader(p, vs); gl.attachShader(p, fs); gl.linkProgram(p); gl.useProgram(p); var vb = gl.createBuffer(); gl.bindBuffer(gl.ARRAY_BUFFER, vb); gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW); var loc = gl.getAttribLocation(p, 'a_pos'); gl.enableVertexAttribArray(loc); gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0); gl.uniform4f(gl.getUniformLocation(p, 'u_color'), 0, 0, 1, 1); var eb = gl.createBuffer(); gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, eb); gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 1, 2, 2, 1, 3]), gl.STATIC_DRAW); gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);</script></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:rgb(0,0,255)\"></div></body></html>",
        is_match: true,
    },
    // ── 全屏三角 drawArrays + scissor 左下红（M3-S4，2026-10-23）──
    // 主臂：超大三角（-1,-1 / 3,-1 / -1,3）覆盖整 clip 方域实心蓝，再
    // enable(SCISSOR_TEST) + scissor(0,0,50,25)（GL 底左原点）重绘红 →
    // 左下 50×25 红、其余蓝。覆盖 viewport/scissor host rect 换算与绘制间
    // retained buffer 读改写。ref 臂：上条 100×25 蓝 + 下条 flex 左红右蓝
    // （几何逐格对应）。文件孪生：
    // tests/wpt-runner/local-reftests/webgl/draw-scissor-zw-004.html。
    InlineReftestDef {
        id: "webgl/draw-scissor-zw-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><canvas id=\"cv\" width=\"100\" height=\"50\" style=\"display:block\"></canvas><script>var gl = document.getElementById('cv').getContext('webgl'); var vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(vs, 'attribute vec2 a_pos; void main() { gl_Position = vec4(a_pos, 0.0, 1.0); }'); gl.compileShader(vs); var fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(fs, 'precision mediump float; uniform vec4 u_color; void main() { gl_FragColor = u_color; }'); gl.compileShader(fs); var p = gl.createProgram(); gl.attachShader(p, vs); gl.attachShader(p, fs); gl.linkProgram(p); gl.useProgram(p); var vb = gl.createBuffer(); gl.bindBuffer(gl.ARRAY_BUFFER, vb); gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW); var loc = gl.getAttribLocation(p, 'a_pos'); gl.enableVertexAttribArray(loc); gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0); gl.uniform4f(gl.getUniformLocation(p, 'u_color'), 0, 0, 1, 1); gl.drawArrays(gl.TRIANGLES, 0, 3); gl.enable(gl.SCISSOR_TEST); gl.scissor(0, 0, 50, 25); gl.uniform4f(gl.getUniformLocation(p, 'u_color'), 1, 0, 0, 1); gl.drawArrays(gl.TRIANGLES, 0, 3);</script></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:25px;background:rgb(0,0,255)\"></div><div style=\"display:flex;width:100px;height:25px\"><div style=\"width:50px;height:25px;background:rgb(255,0,0)\"></div><div style=\"width:50px;height:25px;background:rgb(0,0,255)\"></div></div></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}
