//! 路径表示 — Canvas 2D 路径命令与 Path2D。

/// 路径命令。
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    /// 移动到指定点。
    MoveTo(f32, f32),
    /// 画线到指定点。
    LineTo(f32, f32),
    /// 二次贝塞尔曲线。
    QuadraticCurveTo(f32, f32, f32, f32),
    /// 三次贝塞尔曲线。
    BezierCurveTo(f32, f32, f32, f32, f32, f32),
    /// 圆弧（x, y, 半径, 起始角, 结束角, 逆时针）。
    Arc(f32, f32, f32, f32, f32, bool),
    /// 圆弧切线（控制点1 x, 控制点1 y, 控制点2 x, 控制点2 y, 半径）。
    ArcTo(f32, f32, f32, f32, f32),
    /// 椭圆弧（圆心 x, 圆心 y, 半径 x, 半径 y, 旋转, 起始角, 结束角）。
    Ellipse(f32, f32, f32, f32, f32, f32, f32),
    /// 圆角矩形子路径（x, y, 宽, 高, 圆角半径角对列表 [tl, tr, br, bl]——R34xx 支持
    /// DOMPoint 半径（x=水平, y=垂直）后为角对）。
    RoundRect(f32, f32, f32, f32, Vec<(f32, f32)>),
    /// 闭合路径。
    ClosePath,
}

/// 2D 路径 — 存储 Canvas 路径命令序列。
#[derive(Debug, Clone, Default)]
pub struct Path2D {
    commands: Vec<PathCommand>,
}

impl Path2D {
    /// 创建空路径。
    pub fn new() -> Self {
        Self::default()
    }

    /// 移动到指定点。
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::MoveTo(x, y));
    }

    /// 画线到指定点。
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::LineTo(x, y));
    }

    /// 闭合路径。
    pub fn close_path(&mut self) {
        self.commands.push(PathCommand::ClosePath);
    }

    /// 添加圆弧。
    pub fn arc(&mut self, x: f32, y: f32, radius: f32, start: f32, end: f32, anticlockwise: bool) {
        self.commands
            .push(PathCommand::Arc(x, y, radius, start, end, anticlockwise));
    }

    /// 添加圆弧切线（arcTo）。
    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        self.commands.push(PathCommand::ArcTo(x1, y1, x2, y2, radius));
    }

    /// 添加二次贝塞尔曲线。
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::QuadraticCurveTo(cpx, cpy, x, y));
    }

    /// 添加三次贝塞尔曲线。
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.commands
            .push(PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y));
    }

    /// 添加椭圆弧。
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start_angle: f32,
        end_angle: f32,
    ) {
        self.commands.push(PathCommand::Ellipse(
            cx,
            cy,
            radius_x,
            radius_y,
            rotation,
            start_angle,
            end_angle,
        ));
    }

    /// 添加矩形子路径（四个 line_to + close）。
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.commands.push(PathCommand::MoveTo(x, y));
        self.commands.push(PathCommand::LineTo(x + w, y));
        self.commands.push(PathCommand::LineTo(x + w, y + h));
        self.commands.push(PathCommand::LineTo(x, y + h));
        self.commands.push(PathCommand::ClosePath);
    }

    /// 添加圆角矩形子路径。
    pub fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, radii: Vec<(f32, f32)>) {
        // R34xx：roundRect 开始新子路径（spec：不连当前点——否则 flatten 的 current 连接
        // 段把当前点拉进填充，2d.path.roundrect.* 的圆角外像素被误填）。
        self.commands.push(PathCommand::MoveTo(x, y));
        self.commands.push(PathCommand::RoundRect(x, y, w, h, radii));
        // R56i（M8/DC-8）：spec roundRect 是**闭合子路径**——显式 ClosePath 标记
        //（stroke 的环绕 join / cap 语义按此判定；flatten 的 ClosePath arm 有
        // 距离守卫，末角终点 == 子路径起点时不 push 额外段，fill 无影响）。
        self.commands.push(PathCommand::ClosePath);
    }

    /// 返回路径命令数量。
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// 返回路径命令列表。
    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    /// 返回路径命令列表的可变引用。
    pub fn commands_mut(&mut self) -> &mut Vec<PathCommand> {
        &mut self.commands
    }

    /// 路径是否为空。
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// 清空路径。
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// 将另一个 Path2D 的所有路径命令追加到当前路径。
    pub fn add_path(&mut self, other: &Path2D) {
        self.commands.extend_from_slice(&other.commands);
    }

    /// 从 SVG path data 字符串解析（HTML Canvas `new Path2D(d)`，
    /// https://html.spec.whatwg.org/multipage/canvas.html#dom-path2d + SVG 2 §9.3 path data）。
    ///
    /// R3307：补全 Path2D svgString 构造形式（R3306 createPath lenient 建空路径）。支持全部 SVG 路径命令：
    /// `M/L/H/V/C/S/Q/T/A/Z`（大小写 = 绝对/相对），含隐式重复（命令后多组参数）+ 参数分隔符（空白/逗号，
    /// flag 例外）。`A`（椭圆弧）经端点→中心参数转换（SVG 实现说明 F.6.5）映射到本 Path2D 的 `ellipse`。
    ///
    /// **诚实范围**：① 浮点解析容忍非法数字（NaN/溢出）→ 该命令跳过；② 极少数病态输入（rx/ry=0、
    /// 端点重合）按 SVG 规范退化为 line_to（spec 同语义）；③ 数值精度受 f32 限制。非法/截断命令静默跳过
    /// （real browser spec 亦尽力解析非法 path data 不抛）。返 `true` 表示至少解析出一组有效命令。
    pub fn from_svg(&mut self, d: &str) -> bool {
        let mut p = SvgPathParser::new(d);
        let mut cur_x = 0.0f32;
        let mut cur_y = 0.0f32;
        let mut sub_start_x = 0.0f32;
        let mut sub_start_y = 0.0f32;
        // 上一贝塞尔控制点（S/T 平滑反射用），None 表无前置曲线。
        let mut prev_cubic_cp: Option<(f32, f32)> = None;
        let mut prev_quad_cp: Option<(f32, f32)> = None;
        let mut any = false;

        while let Some((cmd, relative)) = p.next_command() {
            match cmd {
                b'M' | b'L' => {
                    // M/L：隐式重复 = 后续按 L 处理（M 首点 move_to，重复点 line_to）。
                    let mut first = true;
                    while let Some((x, y)) = p.parse_point(cur_x, cur_y, relative) {
                        if cmd == b'M' && first {
                            self.move_to(x, y);
                            sub_start_x = x;
                            sub_start_y = y;
                            first = false;
                        } else {
                            self.line_to(x, y);
                        }
                        cur_x = x;
                        cur_y = y;
                        any = true;
                    }
                    prev_cubic_cp = None;
                    prev_quad_cp = None;
                }
                b'H' => {
                    while let Some(v) = p.parse_number() {
                        let x = if relative { cur_x + v } else { v };
                        self.line_to(x, cur_y);
                        cur_x = x;
                        any = true;
                    }
                    prev_cubic_cp = None;
                    prev_quad_cp = None;
                }
                b'V' => {
                    while let Some(v) = p.parse_number() {
                        let y = if relative { cur_y + v } else { v };
                        self.line_to(cur_x, y);
                        cur_y = y;
                        any = true;
                    }
                    prev_cubic_cp = None;
                    prev_quad_cp = None;
                }
                b'C' => {
                    while let (Some(c1), Some(c2), Some(e)) = (
                        p.parse_point(cur_x, cur_y, relative),
                        p.parse_point(cur_x, cur_y, relative),
                        p.parse_point(cur_x, cur_y, relative),
                    ) {
                        self.bezier_curve_to(c1.0, c1.1, c2.0, c2.1, e.0, e.1);
                        prev_cubic_cp = Some(c2);
                        cur_x = e.0;
                        cur_y = e.1;
                        any = true;
                    }
                    prev_quad_cp = None;
                }
                b'S' => {
                    while let (Some(c2), Some(e)) = (
                        p.parse_point(cur_x, cur_y, relative),
                        p.parse_point(cur_x, cur_y, relative),
                    ) {
                        // 反射前三次控制点作 c1（无前置 → c1 = 当前点）。
                        let c1 = match prev_cubic_cp {
                            Some((px, py)) => (2.0 * cur_x - px, 2.0 * cur_y - py),
                            None => (cur_x, cur_y),
                        };
                        self.bezier_curve_to(c1.0, c1.1, c2.0, c2.1, e.0, e.1);
                        prev_cubic_cp = Some(c2);
                        cur_x = e.0;
                        cur_y = e.1;
                        any = true;
                    }
                    prev_quad_cp = None;
                }
                b'Q' => {
                    while let (Some(c), Some(e)) = (
                        p.parse_point(cur_x, cur_y, relative),
                        p.parse_point(cur_x, cur_y, relative),
                    ) {
                        self.quadratic_curve_to(c.0, c.1, e.0, e.1);
                        prev_quad_cp = Some(c);
                        cur_x = e.0;
                        cur_y = e.1;
                        any = true;
                    }
                    prev_cubic_cp = None;
                }
                b'T' => {
                    while let Some(e) = p.parse_point(cur_x, cur_y, relative) {
                        let c = match prev_quad_cp {
                            Some((px, py)) => (2.0 * cur_x - px, 2.0 * cur_y - py),
                            None => (cur_x, cur_y),
                        };
                        self.quadratic_curve_to(c.0, c.1, e.0, e.1);
                        prev_quad_cp = Some(c);
                        cur_x = e.0;
                        cur_y = e.1;
                        any = true;
                    }
                    prev_cubic_cp = None;
                }
                b'A' => {
                    while let Some(arc) = p.parse_arc_args(relative, cur_x, cur_y) {
                        if let Some((cx, cy, rx, ry, rot, start, end)) = arc_to_center(
                            cur_x, cur_y, arc.rx, arc.ry, arc.rot, arc.large, arc.sweep, arc.x, arc.y,
                        ) {
                            self.ellipse(cx, cy, rx, ry, rot, start, end);
                        } else {
                            // 退化（rx/ry=0 或端点重合）→ line_to（SVG spec 同语义）。
                            self.line_to(arc.x, arc.y);
                        }
                        cur_x = arc.x;
                        cur_y = arc.y;
                        any = true;
                    }
                    prev_cubic_cp = None;
                    prev_quad_cp = None;
                }
                b'Z' => {
                    self.close_path();
                    cur_x = sub_start_x;
                    cur_y = sub_start_y;
                    any = true;
                    // Z 无参数；跳过任何尾随数字（非法但 lenient）。
                    prev_cubic_cp = None;
                    prev_quad_cp = None;
                }
                _ => {
                    // 未知命令：跳过（lenient）。
                }
            }
        }
        any
    }

    /// 判断点是否在路径内部（使用射线法，奇偶填充规则）。
    /// 将路径扁平化为多边形顶点后，用射线法判断点是否在多边形内。
    /// 支持多个子路径。
    pub fn is_point_in_path(&self, x: f32, y: f32) -> bool {
        let vertices = self.flatten_to_vertices();
        if vertices.is_empty() {
            return false;
        }
        let points: Vec<(f32, f32)> = vertices.chunks_exact(2).map(|c| (c[0], c[1])).collect();
        point_in_polygon(x, y, &points)
    }

    /// 将路径命令扁平化为顶点列表（x, y 交替）。
    pub fn flatten_to_vertices(&self) -> Vec<f32> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut subpath_start_x = 0.0f32;
        let mut subpath_start_y = 0.0f32;
        // R34xx（arcTo）：当前子路径存在性（无子路径 → arcTo 先 moveTo 首控制点）。
        let mut subpath_open = false;
        const ARC_SEGMENTS: usize = 16;

        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(mx, my) => {
                    subpath_start_x = *mx;
                    subpath_start_y = *my;
                    current_x = *mx;
                    current_y = *my;
                    subpath_open = true;
                }
                PathCommand::LineTo(lx, ly) => {
                    vertices.push(current_x);
                    vertices.push(current_y);
                    vertices.push(*lx);
                    vertices.push(*ly);
                    current_x = *lx;
                    current_y = *ly;
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, qx, qy) => {
                    let (cpx, cpy, qx, qy) = (*cpx, *cpy, *qx, *qy);
                    // R56h：二次转三次 + 自适应细分（同 raster.rs flatten）。
                    let (sx0, sy0) = (current_x, current_y);
                    let c1x = sx0 + (cpx - sx0) * 2.0 / 3.0;
                    let c1y = sy0 + (cpy - sy0) * 2.0 / 3.0;
                    let c2x = qx + (cpx - qx) * 2.0 / 3.0;
                    let c2y = qy + (cpy - qy) * 2.0 / 3.0;
                    flatten_bezier_adaptive(&mut vertices, sx0, sy0, c1x, c1y, c2x, c2y, qx, qy, 0.1, 0);
                    current_x = qx;
                    current_y = qy;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, bx, by) => {
                    let (cp1x, cp1y, cp2x, cp2y, bx, by) = (*cp1x, *cp1y, *cp2x, *cp2y, *bx, *by);
                    // R56h：自适应细分（同 raster.rs flatten）。
                    flatten_bezier_adaptive(
                        &mut vertices,
                        current_x,
                        current_y,
                        cp1x,
                        cp1y,
                        cp2x,
                        cp2y,
                        bx,
                        by,
                        0.1,
                        0,
                    );
                    current_x = bx;
                    current_y = by;
                }
                PathCommand::Arc(cx, cy, radius, start_angle, end_angle, anticlockwise) => {
                    let (cx, cy, radius, start_angle, end_angle) = (*cx, *cy, *radius, *start_angle, *end_angle);
                    // R34xx：anticlockwise 方向（canvas y 向下：角度递增 = 顺时针，递减 = 逆时针）。
                    let dir = if *anticlockwise { -1.0 } else { 1.0 };
                    // R56（M8/DC-8）：角度归一化对齐 spec dom-context-2d-arc——
                    // |span| ≥ 2π → 整圆；否则按方向取**同向** mod 2π 弧（顺时针
                    // span ∈ [0,2π)、逆时针 ∈ (−2π,0]）。旧 `raw % TAU` 对顺时针
                    // 负差得负 span → 弧走向反向（2d.path.arc.angle.5 扇形翻侧）。
                    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-arc
                    let tau = std::f32::consts::TAU;
                    let raw_span = end_angle - start_angle;
                    let span = if !*anticlockwise {
                        if raw_span >= tau {
                            tau
                        } else {
                            ((raw_span % tau) + tau) % tau
                        }
                    } else if raw_span <= -tau {
                        -tau
                    } else {
                        // R34xx：mod==0 且 raw≠0 → 整圆（arc(0, 2π, true) 逆时针
                        // 全圆——2d.line.join.round 的圆盘填充；Chromium 语义）。
                        let m = -(((-raw_span) % tau + tau) % tau);
                        if m == 0.0 && raw_span != 0.0 { -tau } else { m }
                    };
                    // R56：span 归一化已含方向（顺时针 ∈ [0,τ] / 逆时针 ∈ [−τ,0]），
                    // 不再乘 dir（旧 span 为同号绝对值需 dir 定向——双重取反会翻弧）。
                    let _ = dir;
                    let angle_span = span;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let mut px = cx + radius * start_angle.cos();
                    let mut py = cy + radius * start_angle.sin();
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let nx = cx + radius * angle.cos();
                        let ny = cy + radius * angle.sin();
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::ArcTo(x1, y1, x2, y2, radius) => {
                    let had_subpath = subpath_open;
                    subpath_open = true;
                    let (nx, ny) = arc_to_tangent_segments(
                        &mut vertices,
                        current_x,
                        current_y,
                        *x1,
                        *y1,
                        *x2,
                        *y2,
                        *radius,
                        had_subpath,
                    );
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::Ellipse(cx, cy, rx, ry, rotation, start_angle, end_angle) => {
                    let (cx, cy, rx, ry, rotation, start_angle, end_angle) =
                        (*cx, *cy, *rx, *ry, *rotation, *start_angle, *end_angle);
                    let cos_r = rotation.cos();
                    let sin_r = rotation.sin();
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let compute_point = |angle: f32| -> (f32, f32) {
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let px = rx * cos_a;
                        let py = ry * sin_a;
                        (cx + px * cos_r - py * sin_r, cy + px * sin_r + py * cos_r)
                    };
                    let (mut px, mut py) = compute_point(start_angle);
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let (nx, ny) = compute_point(angle);
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::RoundRect(rx, ry, w, h, _radii) => {
                    // R34xx：角对列表（退化矩形保留——几何 flatten 在 raster.rs）。
                    let (rx, ry, w, h) = (*rx, *ry, *w, *h);
                    // 简化：退化为矩形的 4 条边
                    let corners = [(rx, ry), (rx + w, ry), (rx + w, ry + h), (rx, ry + h)];
                    // R34xx：自包含子路径（不连当前点）。
                    vertices.push(corners[0].0);
                    vertices.push(corners[0].1);
                    for i in 0..3 {
                        vertices.push(corners[i].0);
                        vertices.push(corners[i].1);
                        vertices.push(corners[i + 1].0);
                        vertices.push(corners[i + 1].1);
                    }
                    vertices.push(corners[3].0);
                    vertices.push(corners[3].1);
                    vertices.push(corners[0].0);
                    vertices.push(corners[0].1);
                    current_x = corners[0].0;
                    current_y = corners[0].1;
                }
                PathCommand::ClosePath => {
                    if (current_x - subpath_start_x).abs() > f32::EPSILON
                        || (current_y - subpath_start_y).abs() > f32::EPSILON
                    {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(subpath_start_x);
                        vertices.push(subpath_start_y);
                    }
                    current_x = subpath_start_x;
                    current_y = subpath_start_y;
                    subpath_open = false;
                }
            }
        }
        vertices
    }
}

/// 使用射线法（ray casting）判断点是否在多边形内部。
pub fn point_in_polygon(px: f32, py: f32, points: &[(f32, f32)]) -> bool {
    let n = points.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = points[i];
        let (xj, yj) = points[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

// ── SVG path data 解析器（R3307：Path2D::from_svg）──────────────────────────

/// SVG 弧参数（端点形式）。
struct SvgArcArgs {
    rx: f32,
    ry: f32,
    rot: f32,
    large: bool,
    sweep: bool,
    x: f32,
    y: f32,
}

/// SVG path data 词法解析器。逐命令消费输入，提供 number/point/arc 参数解析。
struct SvgPathParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> SvgPathParser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            bytes: s.as_bytes(),
            pos: 0,
        }
    }

    /// 跳过空白与逗号（SVG path data 参数分隔符）。
    fn skip_sep(&mut self) {
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == b',' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// 返回下一命令字母（绝对/相对标志）。命令字母后的隐式重复参数由调用方循环 parse_* 消费。
    fn next_command(&mut self) -> Option<(u8, bool)> {
        loop {
            self.skip_sep();
            if self.pos >= self.bytes.len() {
                return None;
            }
            let b = self.bytes[self.pos];
            // SVG path 命令字母集合。
            if matches!(
                b,
                b'M' | b'm'
                    | b'L'
                    | b'l'
                    | b'H'
                    | b'h'
                    | b'V'
                    | b'v'
                    | b'C'
                    | b'c'
                    | b'S'
                    | b's'
                    | b'Q'
                    | b'q'
                    | b'T'
                    | b't'
                    | b'A'
                    | b'a'
                    | b'Z'
                    | b'z'
            ) {
                self.pos += 1;
                return Some((b.to_ascii_uppercase(), b.is_ascii_lowercase()));
            }
            // 非命令字母（含数字/符号 → 调用方应已消费）：跳过避免死循环。
            self.pos += 1;
        }
    }

    /// 解析一个浮点数。容忍前导符号 + 科学计数法。失败返 None。
    fn parse_number(&mut self) -> Option<f32> {
        self.skip_sep();
        let start = self.pos;
        // 简易浮点扫描：[+-]? digits? . digits? ([eE][+-]? digits)?；至少一位数字。
        let mut seen_digit = false;
        if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-') {
            self.pos += 1;
        }
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            seen_digit = true;
            self.pos += 1;
        }
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                seen_digit = true;
                self.pos += 1;
            }
        }
        // 科学计数法
        if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'e' || self.bytes[self.pos] == b'E') {
            let save = self.pos;
            self.pos += 1;
            if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-') {
                self.pos += 1;
            }
            let mut exp_digit = false;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                exp_digit = true;
                self.pos += 1;
            }
            if !exp_digit {
                self.pos = save; // 回退：e 后无数字，非科学计数法
            }
        }
        if !seen_digit {
            self.pos = start;
            return None;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos]).ok()?;
        // R3254-C13：f64 解析 + clamp 到 f32——f32 溢出（如 `1e40`）在浏览器（double）是合法数，
        // 直接 parse::<f32> 溢出返回 inf 且 pos 已消费 → 整条命令静默丢失。
        let v = s.parse::<f64>().ok()?;
        if v.is_finite() {
            Some(v.clamp(f64::from(f32::MIN), f64::from(f32::MAX)) as f32)
        } else {
            None
        }
    }

    /// 解析一个点 (x, y)。相对坐标按 (cur_x, cur_y) 偏移（cur 由调用方传入，用于相对偏移）。
    fn parse_point(&mut self, cx: f32, cy: f32, relative: bool) -> Option<(f32, f32)> {
        let x = self.parse_number()?;
        let y = self.parse_number()?;
        if relative { Some((cx + x, cy + y)) } else { Some((x, y)) }
    }

    /// 解析弧参数（rx ry rot large-arc-flag sweep-flag x y）。flag 为单字符（0/1，无分隔符要求）。
    /// (x,y) 相对按 cur 偏移。返回 None 表参数不足/非法。
    fn parse_arc_args(&mut self, relative: bool, cur_x: f32, cur_y: f32) -> Option<SvgArcArgs> {
        let rx = self.parse_number()?;
        let ry = self.parse_number()?;
        let rot = self.parse_number()?;
        let rot = rot.to_radians();
        let large = self.parse_flag()?;
        let sweep = self.parse_flag()?;
        let x = self.parse_number()?;
        let y = self.parse_number()?;
        let (x, y) = if relative { (cur_x + x, cur_y + y) } else { (x, y) };
        Some(SvgArcArgs {
            rx,
            ry,
            rot,
            large,
            sweep,
            x,
            y,
        })
    }

    /// 解析弧 flag（单字符 0/1，可无分隔符）。SVG 规范 flag 不容忍多字符/分隔符歧义，本实现 lenient：
    /// 跳过空白/逗号后取下一字符。
    fn parse_flag(&mut self) -> Option<bool> {
        self.skip_sep();
        if self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'0' || b == b'1' {
                self.pos += 1;
                return Some(b == b'1');
            }
        }
        None
    }
}

/// SVG 弧端点→中心参数转换（SVG 2 实现说明 F.6.5）。返 (cx, cy, rx, ry, rot, start_angle, end_angle)。
/// None 表退化（rx/ry=0 或端点重合）—— 调用方应退化 line_to。
#[allow(clippy::too_many_arguments)]
fn arc_to_center(
    x1: f32,
    y1: f32,
    rx: f32,
    ry: f32,
    phi: f32,
    large: bool,
    sweep: bool,
    x2: f32,
    y2: f32,
) -> Option<(f32, f32, f32, f32, f32, f32, f32)> {
    // 退化：端点重合或半径为 0。
    // R3254-C13：端点**精确相等**才判退化（SVG 2 F.6.6）——此前 1e-6 容差把「接近但不
    // 相等」的端点也退化为 line_to（应画正常弧）；精确相等时 spec 为整段省略（调用方
    // line_to 的差异仅圆头线帽场景多一个点，可接受）。
    if (x1 == x2 && y1 == y2) || rx.abs() < 1e-6 || ry.abs() < 1e-6 {
        return None;
    }
    let rx = rx.abs();
    let ry = ry.abs();
    let cos_p = phi.cos();
    let sin_p = phi.sin();
    // 步骤1：将端点变换到 x'-y'（旋转消除）。
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cos_p * dx + sin_p * dy;
    let y1p = -sin_p * dx + cos_p * dy;
    // 步骤2：修正半径（保证方程有解）。
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    let (rx, ry) = if lambda > 1.0 {
        let s = lambda.sqrt();
        (rx * s, ry * s)
    } else {
        (rx, ry)
    };
    // 步骤3：计算中心 c'。
    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;
    let denom = rx2 * y1p2 + ry2 * x1p2;
    let num = (rx2 * ry2 - denom).max(0.0);
    let factor = if denom > 1e-12 { (num / denom).sqrt() } else { 0.0 };
    let sign = if large == sweep { -1.0 } else { 1.0 };
    let cxp = sign * factor * (rx * y1p) / ry;
    let cyp = sign * factor * -(ry * x1p) / rx;
    // 步骤4：变换回原坐标系得中心。
    let cx = cos_p * cxp - sin_p * cyp + (x1 + x2) / 2.0;
    let cy = sin_p * cxp + cos_p * cyp + (y1 + y2) / 2.0;
    // 步骤5：计算起止角（相对中心、椭圆参数角）。
    let angle = |vx: f32, vy: f32| vy.atan2(vx);
    let mut start = angle((x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut delta = angle((-x1p - cxp) / rx, (-y1p - cyp) / ry) - start;
    // sweep 方向调整到 [0, 2π) 范围语义。
    if !sweep && delta > 0.0 {
        delta -= std::f32::consts::TAU;
    } else if sweep && delta < 0.0 {
        delta += std::f32::consts::TAU;
    }
    let mut end = start + delta;
    // 规范化 start 到 [-π, π]，便于光栅化。
    while start > std::f32::consts::PI {
        start -= std::f32::consts::TAU;
        end -= std::f32::consts::TAU;
    }
    while start < -std::f32::consts::PI {
        start += std::f32::consts::TAU;
        end += std::f32::consts::TAU;
    }
    Some((cx, cy, rx, ry, phi, start, end))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn arc_to_tangent_segments(
    vertices: &mut Vec<f32>,
    current_x: f32,
    current_y: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    radius: f32,
    has_subpath: bool,
) -> (f32, f32) {
    // moveTo 首控制点（2d.path.arcTo.ensuresubpath.*）；否则 P0→P1→P2
    // 的切线弧（半径 r）：切点 T1 = P1 − û1·t、T2 = P1 + û2·t
    //（t = r/tan(φ/2)）；共线/半径过大 → lineTo(P1)。
    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-arcto
    if !has_subpath {
        // 无子路径：moveTo 首控制点——**不画线段**（"nothing is
        // drawn up to it"，2d.path.arcTo.ensuresubpath.1）。
        return (x1, y1);
    }
    let (x0, y0) = (current_x, current_y);
    let (v1x, v1y) = (x1 - x0, y1 - y0);
    let (v2x, v2y) = (x2 - x1, y2 - y1);
    let l1 = (v1x * v1x + v1y * v1y).sqrt();
    let l2 = (v2x * v2x + v2y * v2y).sqrt();
    if l1 < f32::EPSILON || l2 < f32::EPSILON || radius <= 0.0 {
        vertices.push(x0);
        vertices.push(y0);
        vertices.push(x1);
        vertices.push(y1);
        return (x1, y1);
    }
    let a1 = v1y.atan2(v1x);
    let a2 = v2y.atan2(v2x);
    let mut phi = a2 - a1;
    while phi > std::f32::consts::PI {
        phi -= std::f32::consts::TAU;
    }
    while phi < -std::f32::consts::PI {
        phi += std::f32::consts::TAU;
    }
    if phi.abs() < 1e-4 {
        // 共线 → lineTo(P1)。
        vertices.push(x0);
        vertices.push(y0);
        vertices.push(x1);
        vertices.push(y1);
        return (x1, y1);
    }
    let t = radius / (phi.abs() / 2.0).tan();
    if t > l1 || t > l2 {
        // 半径过大（切点超出线段）→ lineTo(P1)（spec）。
        vertices.push(x0);
        vertices.push(y0);
        vertices.push(x1);
        vertices.push(y1);
        return (x1, y1);
    }
    let (u1x, u1y) = (v1x / l1, v1y / l1);
    let (u2x, u2y) = (v2x / l2, v2y / l2);
    let (t1x, t1y) = (x1 - u1x * t, y1 - u1y * t);
    let (t2x, t2y) = (x1 + u2x * t, y1 + u2y * t);
    // R56f：圆心 = T1 + n·r，n 为 T1 处切线 u1 的法线（符号取弧凹侧）。
    // 两个候选 ±n 取与 T2 距离恰为 r 的那个（规范几何恒等——切点
    // 到圆心距离恒 r；旧「u2−u1 平分」方向不垂直 u1，圆心算错致弧
    // 张角偏差，2d.path.arcTo.transformation/scale 实证）。
    let (nx, ny) = (-u1y, u1x); // rot90(u1)
    let d_plus = ((t2x - (t1x + nx * radius)).powi(2) + (t2y - (t1y + ny * radius)).powi(2)).sqrt();
    let d_minus = ((t2x - (t1x - nx * radius)).powi(2) + (t2y - (t1y - ny * radius)).powi(2)).sqrt();
    let (cpx, cpy) = if (d_plus - radius).abs() <= (d_minus - radius).abs() {
        (t1x + nx * radius, t1y + ny * radius)
    } else {
        (t1x - nx * radius, t1y - ny * radius)
    };
    let da1 = (t1y - cpy).atan2(t1x - cpx);
    let da2 = (t2y - cpy).atan2(t2x - cpx);
    let mut span = da2 - da1;
    while span > std::f32::consts::PI {
        span -= std::f32::consts::TAU;
    }
    while span < -std::f32::consts::PI {
        span += std::f32::consts::TAU;
    }
    // 弧转向须与 phi 一致（T1→T2）。
    if (span > 0.0) != (phi > 0.0) {
        span = if phi > 0.0 {
            span + std::f32::consts::TAU
        } else {
            span - std::f32::consts::TAU
        };
    }
    // 线段 P0→T1 + 弧 T1→T2。
    vertices.push(x0);
    vertices.push(y0);
    vertices.push(t1x);
    vertices.push(t1y);
    let steps = 16usize.max((span.abs() / 0.15) as usize);
    let step = span / steps as f32;
    let (mut px, mut py) = (t1x, t1y);
    for i in 0..steps {
        let ang = da1 + step * (i + 1) as f32;
        let (ax, ay) = (cpx + radius * ang.cos(), cpy + radius * ang.sin());
        vertices.push(px);
        vertices.push(py);
        vertices.push(ax);
        vertices.push(ay);
        px = ax;
        py = ay;
    }
    (t2x, t2y)
}

/// 三次贝塞尔自适应细分（设备空间平直度 ε——控制点到弦线距离之和 ≤ ε 时以弦近似）。
/// 旧固定 8 段对跨越大画布的曲线过粗（描边带漏外缘、scale 后弦长数百 px——
/// 2d.path.bezierCurveTo.shape/scaled）；de Casteljau 中点分裂递归。
/// 二次贝塞尔经标准转换（CP1 = P0 + 2/3(CP−P0)，CP2 = P1 + 2/3(CP−P1)）后同用本函数。
/// https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-beziercurveto
#[allow(clippy::too_many_arguments)]
pub(crate) fn flatten_bezier_adaptive(
    vertices: &mut Vec<f32>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    epsilon: f32,
    depth: u32,
) {
    let dx = x3 - x0;
    let dy = y3 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    let flat = if len > f32::EPSILON {
        // 控制点到弦线（P0→P3）距离之和（AGG 式平直度判据）。
        ((dx * (y0 - y1) - dy * (x0 - x1)).abs() + (dx * (y0 - y2) - dy * (x0 - x2)).abs()) / len
    } else {
        0.0
    };
    if flat <= epsilon || depth >= 24 {
        vertices.push(x0);
        vertices.push(y0);
        vertices.push(x3);
        vertices.push(y3);
        return;
    }
    // de Casteljau 中点分裂。
    let (ax, ay) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    let (bx, by) = ((x1 + x2) * 0.5, (y1 + y2) * 0.5);
    let (cx, cy) = ((x2 + x3) * 0.5, (y2 + y3) * 0.5);
    let (dx1, dy1) = ((ax + bx) * 0.5, (ay + by) * 0.5);
    let (ex, ey) = ((bx + cx) * 0.5, (by + cy) * 0.5);
    let (fx, fy) = ((dx1 + ex) * 0.5, (dy1 + ey) * 0.5);
    // 左半段控制点 = (P0, A, D, F)——第二控制点须为 A（中点），非原 P1
    // （笔误致左半段几何失真——2d.path.isPointInPath.bezier 的 (40,2) 被
    // y=-30 的假段包进多边形）。
    flatten_bezier_adaptive(vertices, x0, y0, ax, ay, dx1, dy1, fx, fy, epsilon, depth + 1);
    // 右半段控制点 = (F, E, C, P3)——第三控制点须为 C（中点），非原 P2。
    flatten_bezier_adaptive(vertices, fx, fy, ex, ey, cx, cy, x3, y3, epsilon, depth + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_new() {
        let p = Path2D::new();
        assert!(p.is_empty());
        assert!(p.commands().is_empty());
    }

    #[test]
    fn test_path_move_to() {
        let mut p = Path2D::new();
        p.move_to(10.0, 20.0);
        assert_eq!(p.commands().len(), 1);
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
    }

    #[test]
    fn test_path_line_to() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(100.0, 50.0);
        assert_eq!(p.commands().len(), 2);
        assert!(matches!(p.commands()[1], PathCommand::LineTo(100.0, 50.0)));
    }

    #[test]
    fn test_path_close() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(10.0, 0.0);
        p.close_path();
        assert!(matches!(p.commands().last(), Some(PathCommand::ClosePath)));
    }

    #[test]
    fn test_path_arc() {
        let mut p = Path2D::new();
        p.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI, false);
        assert_eq!(p.commands().len(), 1);
        assert!(matches!(p.commands()[0], PathCommand::Arc(50.0, 50.0, 25.0, 0.0, _, _)));
    }

    #[test]
    fn test_path_rect() {
        let mut p = Path2D::new();
        p.rect(10.0, 20.0, 100.0, 50.0);
        // rect = MoveTo + 3x LineTo + ClosePath
        assert_eq!(p.commands().len(), 5);
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
        assert!(matches!(p.commands()[4], PathCommand::ClosePath));
    }

    #[test]
    fn test_path_clear() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(10.0, 10.0);
        assert!(!p.is_empty());
        p.clear();
        assert!(p.is_empty());
    }

    #[test]
    fn test_path_quadratic_curve_to() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.commands_mut()
            .push(PathCommand::QuadraticCurveTo(10.0, 20.0, 30.0, 40.0));
        assert_eq!(p.commands().len(), 2);
        assert!(matches!(
            p.commands()[1],
            PathCommand::QuadraticCurveTo(10.0, 20.0, 30.0, 40.0)
        ));
    }

    #[test]
    fn test_path_bezier_curve_to() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.commands_mut()
            .push(PathCommand::BezierCurveTo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0));
        assert_eq!(p.commands().len(), 2);
        assert!(matches!(
            p.commands()[1],
            PathCommand::BezierCurveTo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0)
        ));
    }

    #[test]
    fn test_path_commands_mut_modification() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.commands_mut().push(PathCommand::LineTo(5.0, 5.0));
        p.commands_mut().remove(0);
        assert_eq!(p.commands().len(), 1);
    }

    #[test]
    fn test_path_default_trait() {
        let p = Path2D::default();
        assert!(p.is_empty());
    }

    #[test]
    fn test_path_clone_equality() {
        let mut p = Path2D::new();
        p.move_to(1.0, 2.0);
        p.line_to(3.0, 4.0);
        let cloned = p.clone();
        assert_eq!(p.commands().len(), cloned.commands().len());
    }

    #[test]
    fn test_path_multiple_rects() {
        let mut p = Path2D::new();
        p.rect(0.0, 0.0, 10.0, 10.0);
        p.rect(20.0, 20.0, 10.0, 10.0);
        assert_eq!(p.commands().len(), 10); // 2 × 5
    }

    // ── from_svg 测试（R3307：Path2D svgString 解析）──

    #[test]
    fn test_from_svg_moveto_lineto() {
        let mut p = Path2D::new();
        assert!(p.from_svg("M10 20 L30 40"));
        assert_eq!(p.commands().len(), 2);
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
        assert!(matches!(p.commands()[1], PathCommand::LineTo(30.0, 40.0)));
    }

    #[test]
    fn test_from_svg_implicit_repeat() {
        // M 后多组坐标：首点 move_to，余点 implicit line_to（spec）。
        let mut p = Path2D::new();
        assert!(p.from_svg("M0 0 10 10 20 20"));
        assert_eq!(p.commands().len(), 3);
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(0.0, 0.0)));
        assert!(matches!(p.commands()[1], PathCommand::LineTo(10.0, 10.0)));
        assert!(matches!(p.commands()[2], PathCommand::LineTo(20.0, 20.0)));
    }

    #[test]
    fn test_from_svg_relative() {
        // 相对坐标 m/l（小写）= 当前点 + 偏移。
        let mut p = Path2D::new();
        assert!(p.from_svg("M10 10 l5 5"));
        assert_eq!(p.commands().len(), 2);
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 10.0)));
        assert!(matches!(p.commands()[1], PathCommand::LineTo(15.0, 15.0))); // 10+5
    }

    #[test]
    fn test_from_svg_hv() {
        let mut p = Path2D::new();
        assert!(p.from_svg("M0 0 H50 V30"));
        assert_eq!(p.commands().len(), 3);
        assert!(matches!(p.commands()[1], PathCommand::LineTo(50.0, 0.0)));
        assert!(matches!(p.commands()[2], PathCommand::LineTo(50.0, 30.0)));
    }

    #[test]
    fn test_from_svg_bezier_quadratic() {
        let mut p = Path2D::new();
        assert!(p.from_svg("M0 0 C10 10 20 20 30 30"));
        assert!(matches!(
            p.commands()[1],
            PathCommand::BezierCurveTo(10.0, 10.0, 20.0, 20.0, 30.0, 30.0)
        ));

        let mut q = Path2D::new();
        assert!(q.from_svg("M0 0 Q10 10 20 20"));
        assert!(matches!(
            q.commands()[1],
            PathCommand::QuadraticCurveTo(10.0, 10.0, 20.0, 20.0)
        ));
    }

    #[test]
    fn test_from_svg_smooth_bezier() {
        // S 无前置曲线 → c1 = 当前点；有前置 → c1 = 反射。
        let mut p = Path2D::new();
        assert!(p.from_svg("M0 0 C10 10 20 20 30 30 S50 50 60 60"));
        // 第二段 S：c1 = 反射 (30,30) 前控制点 (20,20) → (2*30-20, 2*30-20) = (40,40)
        assert!(matches!(
            p.commands().last(),
            Some(PathCommand::BezierCurveTo(40.0, 40.0, 50.0, 50.0, 60.0, 60.0))
        ));
    }

    #[test]
    fn test_from_svg_closepath() {
        let mut p = Path2D::new();
        assert!(p.from_svg("M0 0 L10 10 Z"));
        assert!(matches!(p.commands().last(), Some(PathCommand::ClosePath)));
    }

    #[test]
    fn test_from_svg_arc() {
        // A rx ry rot large sweep x y：非退化弧（端点不重合、半径>0）→ 经端点→中心转换产 Ellipse 命令。
        let mut p = Path2D::new();
        assert!(p.from_svg("M10 10 A5 5 0 0 0 20 10"));
        assert_eq!(p.commands().len(), 2); // MoveTo + Ellipse
        assert!(
            matches!(p.commands()[1], PathCommand::Ellipse(_, _, 5.0, 5.0, _, _, _)),
            "非退化弧应产 Ellipse 命令（半径 abs=5），实际: {:?}",
            p.commands()[1]
        );
    }

    #[test]
    fn test_from_svg_arc_degenerate() {
        // 端点重合 → 退化 line_to（SVG spec 同语义）。
        let mut p = Path2D::new();
        assert!(p.from_svg("M10 10 A5 5 0 0 0 10 10"));
        assert!(matches!(p.commands().last(), Some(PathCommand::LineTo(10.0, 10.0))));
    }

    #[test]
    fn test_from_svg_arc_zero_radius_degenerate() {
        // rx=0 → 退化 line_to（SVG spec：零半径弧等价于直线到端点）。
        let mut p = Path2D::new();
        assert!(p.from_svg("M10 10 A0 0 0 0 0 30 40"));
        assert!(matches!(p.commands().last(), Some(PathCommand::LineTo(30.0, 40.0))));
    }

    #[test]
    fn test_from_svg_comma_separators() {
        // 逗号分隔（"M10,20"）+ 负号充当分隔符（"L30-40" = x=30, y=-40，SVG path data 允许符号紧贴）。
        let mut p = Path2D::new();
        assert!(p.from_svg("M10,20 L30-40"));
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)), "逗号分隔");
        assert!(
            matches!(p.commands()[1], PathCommand::LineTo(30.0, -40.0)),
            "负号分隔（符号紧贴下一数）: {:?}",
            p.commands()[1]
        );
    }

    #[test]
    fn test_from_svg_invalid_returns_false() {
        // 空串 / 无命令字母 → false（无有效命令解析出）。
        let mut p = Path2D::new();
        assert!(!p.from_svg(""));
        assert!(!p.from_svg("   "));
        assert_eq!(p.commands().len(), 0);
    }

    #[test]
    fn test_from_svg_truncated_lenient() {
        // 截断命令（C 不足 6 参）静默跳过，不抛。
        let mut p = Path2D::new();
        let _ = p.from_svg("M0 0 C10 10"); // 缺参，lenient 跳过 C
        assert!(p.commands().len() >= 1); // 至少 MoveTo 解析出
    }
}
