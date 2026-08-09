//! Canvas 2D API 标准合规性测试。
//!
//! 覆盖 Canvas 基本操作、路径绘制、文本渲染、变换、合成、像素操作、
//! 渐变填充、阴影效果、图像绘制。

use super::TestCase;

/// 返回 Canvas 2D API 标准合规性测试用例。
pub fn canvas_compliance_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  Canvas 基本结构
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "canvas/basic-element".to_string(),
            description: "Canvas element with 2D context".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c1" width="200" height="100"></canvas>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_element:canvas".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "canvas/multiple-canvas".to_string(),
            description: "Multiple canvas elements on one page".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c1" width="100" height="100"></canvas>
<canvas id="c2" width="100" height="100"></canvas>
<canvas id="c3" width="100" height="100"></canvas>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Canvas 与 CSS 布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "canvas/css-sizing".to_string(),
            description: "Canvas element sized by CSS".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c1" style="width:300px; height:200px;"></canvas>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "layout_has_children".to_string()],
        },
        TestCase {
            id: "canvas/flex-layout".to_string(),
            description: "Canvas elements in flex container".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div style="display:flex; gap:10px;">
<canvas width="100" height="100"></canvas>
<canvas width="100" height="100"></canvas>
</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "layout_has_children".to_string()],
        },
        TestCase {
            id: "canvas/grid-layout".to_string(),
            description: "Canvas elements in grid container".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div style="display:grid; grid-template-columns:1fr 1fr; gap:20px;">
<canvas width="200" height="150"></canvas>
<canvas width="200" height="150"></canvas>
</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "layout_has_children".to_string()],
        },
        TestCase {
            id: "canvas/with-border".to_string(),
            description: "Canvas with CSS border".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas width="200" height="100" style="border:2px solid #333;"></canvas>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "layout_has_children".to_string()],
        },
        TestCase {
            id: "canvas/with-margin-centering".to_string(),
            description: "Canvas centered with margin auto".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas width="200" height="100" style="display:block; margin:20px auto;"></canvas>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "layout_has_children".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Canvas 内嵌脚本（JS API 可用性验证）
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "canvas/script-2d-context".to_string(),
            description: "Canvas 2D context available via script".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var canvas = document.getElementById('c');
var ctx = canvas.getContext('2d');
ctx.fillStyle = '#ff0000';
ctx.fillRect(10, 10, 50, 50);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_element:canvas".to_string()],
        },
        TestCase {
            id: "canvas/script-fill-rect".to_string(),
            description: "Canvas fillRect draws rectangle".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.fillStyle = 'blue';
ctx.fillRect(0, 0, 100, 50);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-stroke-rect".to_string(),
            description: "Canvas strokeRect draws outline".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.strokeStyle = 'green';
ctx.lineWidth = 2;
ctx.strokeRect(10, 10, 80, 40);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-path-ops".to_string(),
            description: "Canvas path operations (moveTo, lineTo, closePath)".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="200"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.beginPath();
ctx.moveTo(10, 10);
ctx.lineTo(100, 10);
ctx.lineTo(100, 100);
ctx.closePath();
ctx.fillStyle = 'orange';
ctx.fill();
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-text-drawing".to_string(),
            description: "Canvas fillText draws text".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="300" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.font = '24px sans-serif';
ctx.fillStyle = 'black';
ctx.fillText('Hello Canvas!', 10, 50);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-transform".to_string(),
            description: "Canvas transform operations (translate, rotate, scale)".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="200"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.save();
ctx.translate(100, 100);
ctx.rotate(Math.PI / 4);
ctx.fillStyle = 'red';
ctx.fillRect(-25, -25, 50, 50);
ctx.restore();
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-save-restore".to_string(),
            description: "Canvas save/restore state management".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.fillStyle = 'red';
ctx.save();
ctx.fillStyle = 'blue';
ctx.fillRect(0, 0, 50, 50);
ctx.restore();
ctx.fillRect(60, 0, 50, 50);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-gradient".to_string(),
            description: "Canvas linear and radial gradient".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
var grad = ctx.createLinearGradient(0, 0, 200, 0);
grad.addColorStop(0, 'red');
grad.addColorStop(0.5, 'yellow');
grad.addColorStop(1, 'green');
ctx.fillStyle = grad;
ctx.fillRect(0, 0, 200, 100);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-clip".to_string(),
            description: "Canvas clip path constrains drawing".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.beginPath();
ctx.arc(100, 50, 40, 0, Math.PI * 2);
ctx.clip();
ctx.fillStyle = 'purple';
ctx.fillRect(0, 0, 200, 100);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-global-alpha".to_string(),
            description: "Canvas globalAlpha transparency".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.fillStyle = 'red';
ctx.fillRect(0, 0, 100, 100);
ctx.globalAlpha = 0.5;
ctx.fillStyle = 'blue';
ctx.fillRect(50, 0, 100, 100);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-composite-ops".to_string(),
            description: "Canvas globalCompositeOperation".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.fillStyle = 'red';
ctx.fillRect(10, 10, 80, 80);
ctx.globalCompositeOperation = 'destination-over';
ctx.fillStyle = 'blue';
ctx.fillRect(50, 10, 80, 80);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Canvas 与页面内容组合
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "canvas/canvas-with-text".to_string(),
            description: "Canvas alongside text content".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Chart Title</h2>
<canvas width="400" height="200" style="border:1px solid #ccc;"></canvas>
<p>Description below canvas.</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_heading".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        TestCase {
            id: "canvas/canvas-with-form".to_string(),
            description: "Canvas with form controls".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<form>
<label>Width: <input type="number" value="200"></label>
<label>Height: <input type="number" value="100"></label>
<button type="button">Draw</button>
</form>
<canvas width="200" height="100"></canvas>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_form".to_string()],
        },
        TestCase {
            id: "canvas/canvas-in-table".to_string(),
            description: "Canvas inside table cell".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<table border="1">
<tr><td>Label</td><td><canvas width="200" height="50"></canvas></td></tr>
<tr><td>Value</td><td><canvas width="200" height="50"></canvas></td></tr>
</table>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_table".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Canvas 响应式布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "canvas/responsive-percent".to_string(),
            description: "Canvas with percentage width".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div style="width:80%; margin:auto;">
<canvas style="width:100%; height:200px;"></canvas>
</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "layout_has_children".to_string()],
        },
        TestCase {
            id: "canvas/absolute-positioned".to_string(),
            description: "Canvas with absolute positioning".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div style="position:relative; width:400px; height:300px;">
<canvas width="200" height="200" style="position:absolute; top:50px; left:100px;"></canvas>
</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "layout_has_children".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Canvas 脚本高级 API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "canvas/script-arc-ellipse".to_string(),
            description: "Canvas arc and ellipse drawing".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="200"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.beginPath();
ctx.arc(50, 50, 30, 0, Math.PI * 2);
ctx.fillStyle = 'teal';
ctx.fill();
ctx.beginPath();
ctx.ellipse(150, 100, 40, 20, 0, 0, Math.PI * 2);
ctx.fillStyle = 'olive';
ctx.fill();
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-bezier-curves".to_string(),
            description: "Canvas quadratic and bezier curves".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="300" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.beginPath();
ctx.moveTo(10, 80);
ctx.quadraticCurveTo(50, 10, 150, 80);
ctx.strokeStyle = 'red';
ctx.stroke();
ctx.beginPath();
ctx.moveTo(160, 80);
ctx.bezierCurveTo(180, 10, 250, 10, 290, 80);
ctx.strokeStyle = 'blue';
ctx.stroke();
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-shadow".to_string(),
            description: "Canvas shadow effects".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.shadowColor = 'rgba(0,0,0,0.5)';
ctx.shadowBlur = 10;
ctx.shadowOffsetX = 5;
ctx.shadowOffsetY = 5;
ctx.fillStyle = 'coral';
ctx.fillRect(20, 20, 80, 60);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-imagedata".to_string(),
            description: "Canvas getImageData/putImageData pixel operations".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="100" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
var imageData = ctx.createImageData(100, 100);
for (var i = 0; i < imageData.data.length; i += 4) {
    imageData.data[i] = 255;
    imageData.data[i+1] = 128;
    imageData.data[i+2] = 0;
    imageData.data[i+3] = 255;
}
ctx.putImageData(imageData, 0, 0);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-line-dash".to_string(),
            description: "Canvas setLineDash stroke pattern".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="200" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.setLineDash([5, 3]);
ctx.strokeStyle = 'red';
ctx.lineWidth = 2;
ctx.strokeRect(10, 10, 80, 40);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "canvas/script-text-measure".to_string(),
            description: "Canvas measureText and text alignment".to_string(),
            category: "canvas".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<canvas id="c" width="300" height="100"></canvas>
<script>
var ctx = document.getElementById('c').getContext('2d');
ctx.font = '20px monospace';
ctx.textAlign = 'center';
ctx.textBaseline = 'middle';
var metrics = ctx.measureText('Hello');
ctx.fillText('Hello', 150, 50);
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Canvas 绘图操作
        // ═══════════════════════════════════════════════════════════════

        // ── Canvas 变换操作 ──
        TestCase {
            id: "canvas/transform-ops".to_string(),
            description: "Canvas 变换（translate/rotate/scale/save/restore）".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="300" height="200"></canvas>
            <script>
            var c = document.getElementById('c');
            var ctx = c.getContext('2d');
            ctx.save();
            ctx.translate(50, 50);
            ctx.fillStyle = 'red';
            ctx.fillRect(0, 0, 40, 40);
            ctx.rotate(Math.PI / 4);
            ctx.fillStyle = 'blue';
            ctx.fillRect(0, 0, 40, 40);
            ctx.restore();
            ctx.save();
            ctx.scale(2, 0.5);
            ctx.fillStyle = 'green';
            ctx.fillRect(100, 100, 30, 60);
            ctx.restore();
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        // ── Canvas 渐变和图案 ──
        TestCase {
            id: "canvas/gradient-pattern".to_string(),
            description: "Canvas 渐变填充（linear/radial）".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="400" height="200"></canvas>
            <script>
            var c = document.getElementById('c');
            var ctx = c.getContext('2d');
            var lg = ctx.createLinearGradient(0, 0, 200, 0);
            lg.addColorStop(0, 'red');
            lg.addColorStop(0.5, 'yellow');
            lg.addColorStop(1, 'green');
            ctx.fillStyle = lg;
            ctx.fillRect(0, 0, 200, 100);
            var rg = ctx.createRadialGradient(300, 100, 10, 300, 100, 80);
            rg.addColorStop(0, 'white');
            rg.addColorStop(1, 'blue');
            ctx.fillStyle = rg;
            ctx.fillRect(200, 0, 200, 200);
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        // ── Canvas 图案（R3085：createPattern + fillStyle/strokeStyle 接图案 + 平铺光栅化）──
        TestCase {
            id: "canvas/pattern-fill".to_string(),
            description: "Canvas 图案平铺（createPattern repeat/no-repeat + fill/stroke）".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="200" height="100"></canvas>
            <script>
            var c = document.getElementById('c');
            var ctx = c.getContext('2d');
            // 图案源：8×8 红蓝棋盘 ImageData
            var imgd = ctx.createImageData(8, 8);
            for (var y = 0; y < 8; y++) {
              for (var x = 0; x < 8; x++) {
                var i = (y * 8 + x) * 4;
                var red = ((x + y) % 2 === 0);
                imgd.data[i] = red ? 255 : 0;
                imgd.data[i + 2] = red ? 0 : 255;
                imgd.data[i + 3] = 255;
              }
            }
            // repeat 平铺填充整画布
            var pat = ctx.createPattern(imgd, 'repeat');
            ctx.fillStyle = pat;
            ctx.fillRect(0, 0, 200, 100);
            // no-repeat 单次铺贴描边
            var pat2 = ctx.createPattern(imgd, 'no-repeat');
            ctx.strokeStyle = pat2;
            ctx.strokeRect(10, 10, 50, 50);
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        // ── Canvas 路径绘制 ──
        TestCase {
            id: "canvas/path-drawing".to_string(),
            description: "Canvas 路径绘制（arc/bezier/quadratic）".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="300" height="300"></canvas>
            <script>
            var c = document.getElementById('c');
            var ctx = c.getContext('2d');
            ctx.beginPath();
            ctx.arc(60, 60, 40, 0, Math.PI * 2);
            ctx.strokeStyle = 'orange';
            ctx.lineWidth = 3;
            ctx.stroke();
            ctx.beginPath();
            ctx.moveTo(150, 20);
            ctx.bezierCurveTo(200, 80, 250, 20, 280, 100);
            ctx.strokeStyle = 'purple';
            ctx.stroke();
            ctx.beginPath();
            ctx.moveTo(20, 200);
            ctx.quadraticCurveTo(150, 150, 280, 250);
            ctx.strokeStyle = 'teal';
            ctx.stroke();
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        // ── Canvas 像素操作 ──
        TestCase {
            id: "canvas/pixel-ops".to_string(),
            description: "Canvas 像素操作（createImageData/putImageData）".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="100" height="100"></canvas>
            <script>
            var c = document.getElementById('c');
            var ctx = c.getContext('2d');
            var img = ctx.createImageData(100, 100);
            for (var i = 0; i < img.data.length; i += 4) {
                img.data[i] = Math.random() * 255;
                img.data[i+1] = Math.random() * 255;
                img.data[i+2] = Math.random() * 255;
                img.data[i+3] = 255;
            }
            ctx.putImageData(img, 0, 0);
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        // ── Canvas 合成模式 ──
        TestCase {
            id: "canvas/composite-modes".to_string(),
            description: "Canvas globalCompositeOperation 多种合成模式".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="200" height="200"></canvas>
            <script>
            var c = document.getElementById('c');
            var ctx = c.getContext('2d');
            ctx.fillStyle = 'blue';
            ctx.fillRect(20, 20, 80, 80);
            ctx.globalCompositeOperation = 'multiply';
            ctx.fillStyle = 'red';
            ctx.fillRect(60, 60, 80, 80);
            ctx.globalCompositeOperation = 'source-over';
            ctx.globalAlpha = 0.5;
            ctx.fillStyle = 'green';
            ctx.fillRect(100, 100, 80, 80);
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Canvas 高级操作
        // ═══════════════════════════════════════════════════════════════

        // ── Canvas 裁剪路径 ──
        TestCase {
            id: "canvas/clip-path".to_string(),
            description: "Canvas clip() 裁剪路径".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="200" height="200"></canvas>
            <script>
            var ctx = document.getElementById('c').getContext('2d');
            ctx.beginPath();
            ctx.arc(100, 100, 80, 0, Math.PI * 2);
            ctx.clip();
            ctx.fillStyle = 'orange';
            ctx.fillRect(0, 0, 200, 200);
            ctx.fillStyle = 'blue';
            ctx.font = '24px sans-serif';
            ctx.fillText('Clipped!', 50, 110);
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── Canvas 阴影效果 ──
        TestCase {
            id: "canvas/shadow-effects".to_string(),
            description: "Canvas shadowBlur/shadowColor 效果".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="300" height="200"></canvas>
            <script>
            var ctx = document.getElementById('c').getContext('2d');
            ctx.shadowColor = 'rgba(0, 0, 0, 0.5)';
            ctx.shadowBlur = 10;
            ctx.shadowOffsetX = 5;
            ctx.shadowOffsetY = 5;
            ctx.fillStyle = 'coral';
            ctx.fillRect(20, 20, 100, 60);
            ctx.shadowColor = 'rgba(0, 0, 255, 0.3)';
            ctx.shadowBlur = 15;
            ctx.shadowOffsetX = -3;
            ctx.shadowOffsetY = -3;
            ctx.fillStyle = 'gold';
            ctx.fillRect(160, 40, 100, 60);
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── Canvas 线型（lineDash/cap/join） ──
        TestCase {
            id: "canvas/line-styles".to_string(),
            description: "Canvas lineDash/lineCap/lineJoin 样式".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="300" height="200"></canvas>
            <script>
            var ctx = document.getElementById('c').getContext('2d');
            ctx.setLineDash([10, 5]);
            ctx.lineWidth = 3;
            ctx.strokeStyle = 'red';
            ctx.lineCap = 'round';
            ctx.beginPath(); ctx.moveTo(20, 30); ctx.lineTo(280, 30); ctx.stroke();
            ctx.setLineDash([2, 4]);
            ctx.lineCap = 'butt';
            ctx.strokeStyle = 'green';
            ctx.beginPath(); ctx.moveTo(20, 80); ctx.lineTo(280, 80); ctx.stroke();
            ctx.setLineDash([]);
            ctx.lineWidth = 10;
            ctx.lineJoin = 'round';
            ctx.strokeStyle = 'blue';
            ctx.beginPath(); ctx.moveTo(50, 120); ctx.lineTo(150, 180); ctx.lineTo(250, 120); ctx.stroke();
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── Canvas 文本度量 ──
        TestCase {
            id: "canvas/text-metrics".to_string(),
            description: "Canvas measureText 文本度量".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="400" height="100"></canvas>
            <script>
            var ctx = document.getElementById('c').getContext('2d');
            ctx.font = '20px serif';
            var m = ctx.measureText('Hello World');
            ctx.fillStyle = 'black';
            ctx.fillText('Hello World', 10, 30);
            ctx.strokeStyle = 'red';
            ctx.strokeRect(10, 30 - m.actualBoundingBoxAscent, m.width, m.actualBoundingBoxAscent + m.actualBoundingBoxDescent);
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── Canvas globalAlpha 和状态保存 ──
        TestCase {
            id: "canvas/state-management".to_string(),
            description: "Canvas save/restore + globalAlpha 状态管理".to_string(),
            category: "canvas".to_string(),
            html: r#"<html><body>
            <canvas id="c" width="200" height="200"></canvas>
            <script>
            var ctx = document.getElementById('c').getContext('2d');
            ctx.fillStyle = 'red';
            ctx.globalAlpha = 1.0;
            ctx.fillRect(10, 10, 80, 80);
            ctx.save();
            ctx.globalAlpha = 0.5;
            ctx.fillStyle = 'green';
            ctx.fillRect(50, 50, 80, 80);
            ctx.save();
            ctx.globalAlpha = 0.3;
            ctx.fillStyle = 'blue';
            ctx.fillRect(90, 90, 80, 80);
            ctx.restore();
            ctx.fillStyle = 'yellow';
            ctx.fillRect(110, 30, 80, 40);
            ctx.restore();
            ctx.fillStyle = 'purple';
            ctx.fillRect(30, 110, 80, 40);
            </script>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
    ]
}
