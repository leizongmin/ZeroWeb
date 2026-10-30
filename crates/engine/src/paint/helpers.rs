//! 绘制辅助 — 共享长度工具与杂项入口。原单文件 3815 行（2000 行准则 2.4×）按职责
//! 拆为 `transform.rs`（transform/translate）、`clip.rs`（裁剪/opacity）、
//! `gradient.rs`（渐变转换）、`border_shape.rs`（corner-shape/border-shape 形状
//! 几何）（T59，2026-10-17；纯移动零行为变化）。本文件保留跨族共享的长度/杂项
//! 工具，并按原 `paint::helpers::X` 路径 re-export 四族全部公开项——调用方零改动。

use zero_css_parser::values::{CornerShapeKind, LengthValue, eval_calc};
use zero_layout_engine::LayoutBox;
use zero_style_system::{BorderCollapseValue, BorderStyleValue, ComputedStyle, TextTransformValue};

pub use super::border_shape::*;
pub use super::clip::*;
pub use super::gradient::*;
pub use super::transform::*;

/// 本函数保留为 paint 模块的稳定入口（`&TextTransformValue` 入参），委托到该方法。
pub fn apply_text_transform(text: &str, transform: &TextTransformValue) -> String {
    transform.apply(text)
}

/// R4248（CSS Borders 4 §corner-shaping）：构造 corner-shape 形状的边界多边形
///（border-box 绝对坐标，顺时针 TL→TR→BR→BL）。四角均 round（初始态）返回 None。
///
/// 形状语义（css-borders-4 §3.7：superellipse(K) 的单位曲线 x^(2K) + y^(2K) = 1，
/// round ≡ K=1、bevel ≡ K=0、scoop ≡ K=−1、notch ≡ K=−∞、square ≡ K=+∞；
/// §3.9.4.2 渲染算法按 K 分 regime）：
/// - 凸侧 K ≥ 0：曲线心 = 内角点 C，指数 n = 2K（K=1 圆、K=0 直弦 ≡ bevel）。
/// - 凹侧 K ≤ −1：曲线心 = 盒角点 P，n = 2|K|（K=−1 即 scoop 凹圆弧）。
/// - square = 直角（无切削，经盒角点 P）；notch = 经内角点 C 的阶梯。
///
/// R5183：形角半径规范解析（`BorderRadiusSpec::from_style_with_box` 口径：百分比
/// 相对 border-box、min(w,h)/2 钳制、§5.5 相邻重叠收缩）——此前形角三消费端用裸
/// `length_to_f32`，Percentage 返 0 整盒退直角（同页 plain 弧带出圆弧的自相矛盾
/// 渲染）。
pub(crate) fn shaped_radius_spec(style: &ComputedStyle, box_w: f32, box_h: f32) -> [f32; 4] {
    let spec = BorderRadiusSpec::from_style_with_box(style, box_w, box_h);
    [spec.top_left, spec.top_right, spec.bottom_right, spec.bottom_left]
}

/// R4248：元素形角多边形便捷入口——半径按 [`shaped_radius_spec`] 规范解析（R5183
/// 起 % 相对 border-box），顶点为绝对页面坐标；全 round（corner-shape 初始态）返
/// 回 None。
pub fn shaped_corner_polygon(
    style: &ComputedStyle,
    box_w: f32,
    box_h: f32,
    abs_x: f32,
    abs_y: f32,
) -> Option<Vec<(f32, f32)>> {
    shaped_corner_polygon_in_box(
        style,
        abs_x,
        abs_y,
        box_w,
        box_h,
        shaped_radius_spec(style, box_w, box_h),
    )
}

/// R4248：指定盒与各角半径的形角多边形（corner-shape 内圈/外圈复用）。
/// `radii` 为四角半径（绝对 px，调用方按用途缩放，如边框内圈 = r - border 宽）。
pub fn shaped_corner_polygon_in_box(
    style: &ComputedStyle,
    x0: f32,
    y0: f32,
    box_w: f32,
    box_h: f32,
    radii4: [f32; 4],
) -> Option<Vec<(f32, f32)>> {
    if style.corner_shape.is_all_round() {
        return None;
    }
    Some(corner_polygon_with_kinds(
        [
            style.corner_shape.corner(0),
            style.corner_shape.corner(1),
            style.corner_shape.corner(2),
            style.corner_shape.corner(3),
        ],
        x0,
        y0,
        box_w,
        box_h,
        radii4,
        // FIXME(R5341)：形角（superellipse/scoop）曲线的椭圆双轴缩放未实现——
        // shaped 路径维持单轴 r（ry=rx），与 R4248 起行为一致。
        radii4,
    ))
}

/// R5180（css-backgrounds-3 §5.5 弧带）：plain border-radius（corner-shape 初始态，
/// 全 round）的圆角矩形多边形——与 [`CornerShapeKind::Round`] 臂同一 8 段圆弧采样，
/// 供边框弧带（外圈+内圈 even-odd 环）复用 shaped 采样体。
pub fn rounded_corner_polygon_in_box(
    x0: f32,
    y0: f32,
    box_w: f32,
    box_h: f32,
    radii4: [f32; 4],
    ry4: [f32; 4],
) -> Vec<(f32, f32)> {
    corner_polygon_with_kinds([CornerShapeKind::Round; 4], x0, y0, box_w, box_h, radii4, ry4)
}

/// R5339（css-filters-2 #BackdropFilter）：backdrop-filter 滤区多边形——shaped 形角
/// 优先；corner-shape 初始态（全 round）回退 plain 圆角多边形（plain border-radius
/// 元素滤区同样按圆角轮廓裁剪，backdrop-filter-clip-rect-2 实证方角）；无半径返
/// None（矩形滤区，不裁）。
pub fn backdrop_filter_clip_polygon(
    style: &ComputedStyle,
    x0: f32,
    y0: f32,
    box_w: f32,
    box_h: f32,
) -> Option<Vec<(f32, f32)>> {
    let radii4 = shaped_radius_spec(style, box_w, box_h);
    shaped_corner_polygon_in_box(style, x0, y0, box_w, box_h, radii4).or_else(|| {
        if radii4.iter().any(|r| *r > 0.0) {
            Some(rounded_corner_polygon_in_box(x0, y0, box_w, box_h, radii4, radii4))
        } else {
            None
        }
    })
}

/// 弧带 even-odd 顶点串联（keyhole 切点）。各多边形先在闭合边（末点→首点）中点
/// 插入切点顶点再串联：外圈末点与内圈首点为同一切点对，两条桥边几何重合互为
/// 反向，even-odd 扫描交点逐对抵消，仅填环带。朴素串联（外末→内首直连）两条桥
/// 边以不同 x 交叉扫描线，在带内撕出背景色隙（R5180 corner-shape-square 案 385px
/// 判例；R5182 border-shape fill 用户 polygon 同根因）。R5180 限定 shaped 多边形
///（首点 = TL start、末点 = BL end 同在左缘竖边）；R5182 起切点 = 闭合边真中点，
/// 任意顶点序（用户 polygon）适用——首末同 x 时中点 x 不变，既有调用方行为不变。
pub fn concat_ring_vertices(mut outer: Vec<(f32, f32)>, inner: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let outer_cut = cut_polygon_at_closing_edge(&outer);
    let inner_cut = cut_polygon_at_closing_edge(&inner);
    outer.clear();
    outer.extend(outer_cut);
    outer.extend(inner_cut);
    outer
}

/// keyhole 切点——在多边形闭合边（末点→首点）中点插入切点顶点（首尾各一）。
/// R5180 供 shaped/round 多边形（首末同在左缘竖边）使用；R5182 通用化为任意
/// 顶点序（用户 polygon）：首末同 x 时切点与旧左缘中点逐位一致。供环带串联方
///（[`concat_ring_vertices`]）自行预切。
pub fn cut_polygon_at_closing_edge(pts: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let n = pts.len();
    let (fx, fy) = pts[0];
    let (lx, ly) = pts[n - 1];
    // 切点 = 闭合边中点；首末点同 x（shaped 构造约定）时即左缘竖边中点。
    let mid = ((fx + lx) / 2.0, (fy + ly) / 2.0);
    let mut out = Vec::with_capacity(n + 2);
    out.push(mid);
    out.extend_from_slice(pts);
    out.push(mid);
    out
}

/// 按 per-corner 形状构建形角多边形（shaped 入口与 plain round 入口共用采样体）。
/// R5341：椭圆角——每角携带 (rx, ry)，弧采样 y 分量按 ry 缩放；ry=rx 时逐位等
/// 旧圆弧（kill-switch `ZW_ELLIPTIC_RADIUS=0` 落在此前解析层）。
fn corner_polygon_with_kinds(
    kinds: [CornerShapeKind; 4],
    x0: f32,
    y0: f32,
    box_w: f32,
    box_h: f32,
    radii4: [f32; 4],
    ry4: [f32; 4],
) -> Vec<(f32, f32)> {
    let radii = BorderRadiusSpec {
        top_left: radii4[0],
        top_right: radii4[1],
        bottom_right: radii4[2],
        bottom_left: radii4[3],
        ry: ry4,
    };
    let x1 = x0 + box_w;
    let y1 = y0 + box_h;
    // (kind, r, ry, start=前一边缘端点, end=后一边缘端点, C=圆弧心, P=盒角点, 弧起角, 弧终角)
    // 段方向沿顺时针多边形；弧角按极角（相对 C）自 start 到 end。
    let corners: [(
        CornerShapeKind,
        f32,
        f32,
        (f32, f32),
        (f32, f32),
        (f32, f32),
        (f32, f32),
        f64,
        f64,
    ); 4] = [
        // TL：左缘端 (x0,y0+ry) → 顶缘端 (x0+rx,y0)；C=(x0+rx,y0+ry)；弧 180°→270°
        (
            kinds[0],
            radii.top_left,
            radii.ry[0],
            (x0, y0 + radii.ry[0]),
            (x0 + radii.top_left, y0),
            (x0 + radii.top_left, y0 + radii.ry[0]),
            (x0, y0),
            std::f64::consts::PI,
            3.0 * std::f64::consts::FRAC_PI_2,
        ),
        // TR：顶缘端 (x1-rx,y0) → 右缘端 (x1,y0+ry)；C=(x1-rx,y0+ry)；弧 270°→360°
        (
            kinds[1],
            radii.top_right,
            radii.ry[1],
            (x1 - radii.top_right, y0),
            (x1, y0 + radii.ry[1]),
            (x1 - radii.top_right, y0 + radii.ry[1]),
            (x1, y0),
            3.0 * std::f64::consts::FRAC_PI_2,
            2.0 * std::f64::consts::PI,
        ),
        // BR：右缘端 (x1,y1-ry) → 底缘端 (x1-rx,y1)；C=(x1-rx,y1-ry)；弧 0°→90°
        (
            kinds[2],
            radii.bottom_right,
            radii.ry[2],
            (x1, y1 - radii.ry[2]),
            (x1 - radii.bottom_right, y1),
            (x1 - radii.bottom_right, y1 - radii.ry[2]),
            (x1, y1),
            0.0,
            std::f64::consts::FRAC_PI_2,
        ),
        // BL：底缘端 (x0+rx,y1) → 左缘端 (x0,y1-ry)；C=(x0+rx,y1-ry)；弧 90°→180°
        (
            kinds[3],
            radii.bottom_left,
            radii.ry[3],
            (x0 + radii.bottom_left, y1),
            (x0, y1 - radii.ry[3]),
            (x0 + radii.bottom_left, y1 - radii.ry[3]),
            (x0, y1),
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::PI,
        ),
    ];

    // 每角 8 段基线：R5179 实测 16 段对 corpus 残差（square/notch-mixed 等径向系统
    // 偏差）零改善——弦高 AA 非残差来源，加密只增顶点成本，保持 8。
    // R5254：大半径按 r/8 自适应加密（上限 64）——8 段弦的矢高 r(1−cos(π/16)) 在
    // 切线端横向放大（r=边长极端案外圈弦距真弧 ~8px/扫描行，弧带 even-odd 环
    // （R5180）与背景精确椭圆光栅共享外轮廓时漏出底色 ~300px，round-2 族
    // background-clip-content-box-with-border-radius-003 实证）；r≤64 维持 8 段
    // 既有行为不变。
    let n_samples_for = |r: f32| -> usize { 8usize.max((r / 8.0).ceil() as usize).min(64) };
    let mut pts: Vec<(f32, f32)> = Vec::new();
    for (kind, r, ry, start, end, c, corner_pt, a0, a1) in corners {
        let n_samples = n_samples_for(r.max(ry));
        pts.push(start);
        match kind {
            CornerShapeKind::Round => {
                // round ≡ superellipse(1)：指数 n = 2K = 2（圆）。R5341：椭圆弧——
                // y 分量按 ry 缩放（ry=r 时逐位等圆弧）。
                for k in 1..n_samples {
                    let t = k as f64 / n_samples as f64;
                    let ang = a0 + (a1 - a0) * t;
                    // 圆弧参数化：以 C 为基准，按角的象限符号。
                    let (sx, sy) = ((ang.cos()).signum(), (ang.sin()).signum());
                    let (ca, sa) = (ang.cos().abs(), ang.sin().abs());
                    let px = c.0 as f64 + sx * r as f64 * ca;
                    let py = c.1 as f64 + sy * ry as f64 * sa;
                    pts.push((px as f32, py as f32));
                }
            }
            CornerShapeKind::Superellipse(k_arg) if k_arg > 0.0 => {
                // 凸侧（css-borders-4 §3.7/§3.9.4.2）：指数 n = 2K，曲线心 = C；
                // |dx/r|^n + |dy/r|^n = 1（K=1 退化为圆即 round，K→大逼近 square）。
                // R5341：y 分量按 ry 缩放（椭圆 superellipse；ry=r 逐位等旧）。
                let n = 2.0 * k_arg;
                for k in 1..n_samples {
                    let t = k as f64 / n_samples as f64;
                    let ang = a0 + (a1 - a0) * t;
                    let (sx, sy) = ((ang.cos()).signum(), (ang.sin()).signum());
                    let (ca, sa) = (ang.cos().abs(), ang.sin().abs());
                    let px = c.0 as f64 + sx * r as f64 * ca.powf(2.0 / n);
                    let py = c.1 as f64 + sy * ry as f64 * sa.powf(2.0 / n);
                    pts.push((px as f32, py as f32));
                }
            }
            CornerShapeKind::Scoop | CornerShapeKind::Superellipse(_) => {
                // superellipse(0) ≡ bevel：直弦（start→end），无中间点（k=0 直通）。
                // 凹侧（css-borders-4 §3.9.4.2）：曲线心 = 盒角点 P。
                // K ≤ −1（scoop ≡ K=−1，step 37）：n = 2|K|，
                //   point = P + T^(1/n)·(end−P) + (1−T)^(1/n)·(start−P)
                //   —— K=−1 时 n=2 即以 P 为圆心、过 start/end 的凹圆弧；
                //   K→−∞ 塌缩为经 C 的阶梯（≡ notch，解析层已映射关键字）。
                // −1 < K < 0（step 38，切线交点族）：
                //   point = P + (1−(1−T)^½)·(end−P) + (1−T^½)·(start−P)。
                let k_arg = match kind {
                    CornerShapeKind::Scoop => -1.0, // scoop ≡ superellipse(-1)
                    CornerShapeKind::Superellipse(k) => k,
                    _ => unreachable!("arm 仅 Scoop | Superellipse 进入"),
                };
                if k_arg != 0.0 {
                    // R5179：深凹 K（n = 2|K| ≥ 64）曲线与 notch 阶梯的最大偏差
                    // （T=½ 处）= r·√2·(½ − 2^(−1/n)) ≤ r·0.0153 ≈ 0.38px @ r=25
                    // ——低于 8 段圆弧弦高的既有近似容差（r(1−cos(π/16)) ≈ 0.049r），
                    // 直接走 notch 阶梯。亚像素贴边的规范曲线经 path_fill 无 AA 光栅
                    // 化会放大成整像素腿位溢出（superellipse(-100) 案形 204px 差→0），
                    // 且 WPT ref 即精确阶梯。
                    if k_arg <= -32.0 {
                        pts.push(c);
                    } else {
                        let deep_n = if k_arg <= -1.0 {
                            Some(-2.0 * k_arg)
                        } else {
                            None // −1 < K < 0：切线交点族
                        };
                        for k in 1..n_samples {
                            let t = k as f64 / n_samples as f64;
                            let (wx, wy) = match deep_n {
                                Some(n) => (t.powf(1.0 / n), (1.0 - t).powf(1.0 / n)),
                                None => (1.0 - (1.0 - t).sqrt(), 1.0 - t.sqrt()),
                            };
                            pts.push((
                                (corner_pt.0 as f64
                                    + wx * (end.0 - corner_pt.0) as f64
                                    + wy * (start.0 - corner_pt.0) as f64) as f32,
                                (corner_pt.1 as f64
                                    + wx * (end.1 - corner_pt.1) as f64
                                    + wy * (start.1 - corner_pt.1) as f64) as f32,
                            ));
                        }
                    }
                }
            }
            CornerShapeKind::Notch => {
                pts.push(c); // 经内角点的阶梯
            }
            CornerShapeKind::Square | CornerShapeKind::Bevel => {}
        }
        // square 直角：段终点换到盒角点（边界经 P）；其余形状段终点 = end。
        pts.push(if matches!(kind, CornerShapeKind::Square) {
            corner_pt
        } else {
            end
        });
    }
    pts
}

/// 四角圆角半径集合。
#[derive(Debug, Clone, Copy)]
pub struct BorderRadiusSpec {
    /// 左上角半径。
    pub top_left: f32,
    /// 右上角半径。
    pub top_right: f32,
    /// 右下角半径。
    pub bottom_right: f32,
    /// 左下角半径。
    pub bottom_left: f32,
    /// 四角垂直半径 [tl, tr, br, bl]（css-backgrounds-3 §5.5 椭圆角；R5341）。
    /// 垂直百分比按 border-box 高解析；单值声明的垂直半径 = 水平声明值重解析（§5.1
    /// 两轴各自相对对应边）。kill-switch `ZW_ELLIPTIC_RADIUS=0` 时随水平值（旧
    /// 单轴渲染，逐位等 R5341 前）。
    pub ry: [f32; 4],
}

impl BorderRadiusSpec {
    /// 水平半径数组 [tl, tr, br, bl]。
    pub fn rx4(&self) -> [f32; 4] {
        [self.top_left, self.top_right, self.bottom_right, self.bottom_left]
    }

    /// 从 ComputedStyle 提取圆角半径（无盒尺寸上下文——垂直半径随水平原始值）。
    pub fn from_style(style: &ComputedStyle) -> Self {
        let rx = [
            length_to_f32(&style.border_top_left_radius),
            length_to_f32(&style.border_top_right_radius),
            length_to_f32(&style.border_bottom_right_radius),
            length_to_f32(&style.border_bottom_left_radius),
        ];
        Self {
            top_left: rx[0],
            top_right: rx[1],
            bottom_right: rx[2],
            bottom_left: rx[3],
            ry: rx,
        }
    }

    /// 从 ComputedStyle 提取圆角半径，按 border-box 尺寸解析百分比与含百分比 calc。
    ///
    /// `box_w`/`box_h` 为元素 border-box 尺寸（layout 后已知）。CSS Backgrounds §5.1：
    /// border-radius 百分比相对于 border-box 对应轴（水平按宽、垂直按高）。百分比与
    /// 含百分比的 calc()/min()/max()/clamp() 在 computed 阶段无法解析（需容器尺寸），
    /// 此处按 border-box 解析。不含百分比的长度（含 calc）已在 computed 阶段解析为
    /// Px，经此路径原值返回（byte-identical 于 [`Self::from_style`]）。
    ///
    /// R5341（css-backgrounds-3 §5.5 椭圆角）：垂直半径取 `border_radius_y` 槽位按
    /// `box_h` 解析；槽位缺省（纯单值声明）时垂直 = 水平**声明值**重按 `box_h` 解析
    /// ——`border-radius: 50%` 于非方盒为椭圆角（rx=50%宽、ry=50%高），非胶囊。
    /// kill-switch `ZW_ELLIPTIC_RADIUS=0` 回退 ry=rx（旧单轴渲染）。
    ///
    /// 解析后按 css-backgrounds-3 §5.5 对相邻圆角做重叠收缩（见
    /// [`Self::shrink_overlapping_corners`]）。
    ///
    /// driving：R2314 border-radius 百分比（`border-radius: 50%` 圆形）与含百分比 calc；
    /// R5341 counter-hebrew-nested 之外的 border-shape-outline-with-border-double-ellipse
    /// （corpus ref 侧 40×100 `border-radius: 50%` 误渲染胶囊）。
    pub fn from_style_with_box(style: &ComputedStyle, box_w: f32, box_h: f32) -> Self {
        let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
        let rx_declared = [
            &style.border_top_left_radius,
            &style.border_top_right_radius,
            &style.border_bottom_right_radius,
            &style.border_bottom_left_radius,
        ];
        let mut spec = Self {
            top_left: resolve_radius_length(rx_declared[0], box_w, font_size_px),
            top_right: resolve_radius_length(rx_declared[1], box_w, font_size_px),
            bottom_right: resolve_radius_length(rx_declared[2], box_w, font_size_px),
            bottom_left: resolve_radius_length(rx_declared[3], box_w, font_size_px),
            ry: [0.0; 4],
        };
        match &style.border_radius_y {
            // 槽位在：各角垂直声明值（显式双值或单值哨兵已存声明副本，apply.rs）按高解析。
            Some(slot) => {
                for (i, v) in slot.iter().enumerate() {
                    spec.ry[i] = resolve_radius_length(v, box_h, font_size_px);
                }
            }
            // 槽位缺省：全部单值声明——垂直 = 水平声明值重按高解析（§5.1）。
            None => {
                for (i, v) in rx_declared.iter().enumerate() {
                    spec.ry[i] = resolve_radius_length(v, box_h, font_size_px);
                }
            }
        }
        if std::env::var("ZW_ELLIPTIC_RADIUS").as_deref() == Ok("0") {
            spec.ry = spec.rx4();
        }
        spec.shrink_overlapping_corners(box_w, box_h);
        spec
    }

    /// css-backgrounds-3 §5.5（corner overlap）：相邻两角半径之和超过所在边长时，
    /// 四角半径按 f = min(Li/Si) 等比收缩（Li = 边长：上/下取宽、左/右取高；
    /// Si = 该边两角半径之和；Si=0 的边不参与）。R5341 椭圆角：Si 逐轴取值——
    /// 上/下边用水平半径和、左/右边用垂直半径和，单一 f 同缩两轴。
    ///
    /// 圆角光栅化以「角部曲线不重叠（半径 ≤ 半边长）」为前提——pill 习语
    /// `border-radius: 999px` 依赖本收缩：未收缩半径会使每个像素都落入角部
    /// 判定分支且距圆心远超半径，背景整体丢失（welcome 页 v0.1.0 徽章根因）。
    /// 不重叠（f ≥ 1 或全零）时保持原值。ry=rx（kill-switch/单值方盒）时 8 项
    /// 与旧 4 项逐项重复，f 逐位一致。
    fn shrink_overlapping_corners(&mut self, box_w: f32, box_h: f32) {
        let sides = [
            (box_w, self.top_left + self.top_right),
            (box_h, self.ry[1] + self.ry[2]),
            (box_w, self.bottom_right + self.bottom_left),
            (box_h, self.ry[3] + self.ry[0]),
        ];
        let mut scale = f32::INFINITY;
        for (length, sum) in sides {
            if sum > 0.0 {
                scale = scale.min(length / sum);
            }
        }
        if scale.is_finite() && scale < 1.0 {
            self.top_left *= scale;
            self.top_right *= scale;
            self.bottom_right *= scale;
            self.bottom_left *= scale;
            for r in &mut self.ry {
                *r *= scale;
            }
        }
    }

    /// 所有圆角都为零。
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
            && self.ry.iter().all(|&r| r == 0.0)
    }
}

/// R5180（css-backgrounds-3 §5.5 弧带）：plain round 弧带适用判定——非 collapse、
/// 四边 Solid 同色等宽且非零宽，成立时返回等宽宽度 b。border.rs 边框弧带与
/// border-area sliver 环（effects.rs）共用口径，保证两路径逐像素同形。
/// kill-switch `ZW_ROUND_BORDER_BAND=0` 整体恢复条带（旧行为）。
pub fn round_band_uniform_border(style: &ComputedStyle, box_node: &LayoutBox) -> Option<f32> {
    if std::env::var("ZW_ROUND_BORDER_BAND").as_deref() == Ok("0") {
        return None;
    }
    if !style.corner_shape.is_all_round() {
        return None;
    }
    if matches!(style.border_collapse, BorderCollapseValue::Collapse) {
        return None;
    }
    let b = [
        box_node.border_top,
        box_node.border_right,
        box_node.border_bottom,
        box_node.border_left,
    ];
    if !(b[0] > 0.0 && b[1] == b[0] && b[2] == b[0] && b[3] == b[0]) {
        return None;
    }
    let solid = |s: &BorderStyleValue| matches!(s, BorderStyleValue::Solid);
    if !(solid(&style.border_top_style)
        && solid(&style.border_right_style)
        && solid(&style.border_bottom_style)
        && solid(&style.border_left_style))
    {
        return None;
    }
    let uniform_color = style.border_top_color == style.border_right_color
        && style.border_right_color == style.border_bottom_color
        && style.border_bottom_color == style.border_left_color;
    if !uniform_color {
        return None;
    }
    Some(b[0])
}

/// 解析 border-radius 单角长度值为像素半径。
///
/// - `Px`：已解析的绝对值（em/rem/vw 与不含百分比的 calc 已在 computed 阶段解析为 Px），
///   原值返回。
/// - `Percentage`：CSS Backgrounds §5.1，相对于 border-box 宽度（水平半径语义）。
/// - `Calc`：含百分比的 calc/min/max/clamp（无百分比的已在 computed 解析为 Px），
///   以 `box_w` 为百分比基准（parent_length）求值；不可解（如需 font 上下文）→ 回退 0.0。
///
/// R5203：不按半短边钳制——§5.5 角重叠收缩（[`BorderRadiusSpec::shrink_overlapping_corners`]）
/// 才是规范口径，均匀半径下二者等价（f·r = 半边长），非均匀单角大半径（100% 单角
/// = 四分之一圆）此前被过度钳制致与 px 半径不同形（background-clip-*-with-
/// border-radius-002 %/px 两侧不对称实证）。
fn resolve_radius_length(v: &LengthValue, box_w: f32, font_size_px: f64) -> f32 {
    match v {
        LengthValue::Px(p) => *p as f32,
        LengthValue::Percentage(pct) => ((pct / 100.0) * box_w as f64) as f32,
        LengthValue::Calc(expr) => eval_calc(expr, Some(box_w as f64)).unwrap_or(0.0) as f32,
        // computed 阶段未解析的其余真实长度单位：按当前 font-size 在 paint 边界解析。
        _ => zero_style_system::computed::resolve_length(v, font_size_px, None, None) as f32,
    }
}

/// 将 LengthValue 转换为 f32（仅支持 Px）。
pub fn length_to_f32(v: &LengthValue) -> f32 {
    match v {
        LengthValue::Px(p) => *p as f32,
        _ => 0.0,
    }
}

/// 解析 clip-path basic-shape `<length-percentage>` 为像素（CSS Masking §inset/circle/ellipse/polygon）。
///
/// 通用解析器：Px 原值；Em/Rem 按 font-size 解析；Percentage 相对调用方传入的 `box_dim`。
/// clip-path 的长度值位于 `ClipPathValue::*` enum 内，computed 阶段的 `resolve_length_field`
/// 不触及，故在 paint 补解析。调用方按形状语义传 `box_dim`：
/// - inset：top/bottom→height、left/right→width（R2365）
/// - circle 半径：sqrt(w²+h²)/√2；ellipse rx→width、ry→height（R2366）
/// - polygon 顶点 / circle·ellipse 圆心 position：x→width、y→height（R2366）
///
/// vw/vh/ch 等视口/字体度量单位近似为 0（clip-path 极罕用，缺视口上下文）。
pub fn resolve_inset_length(v: &LengthValue, box_dim: f32, font_size_px: f32) -> f32 {
    match v {
        LengthValue::Px(p) => *p as f32,
        LengthValue::Em(e) => (*e as f32) * font_size_px,
        LengthValue::Rem(e) => (*e as f32) * 16.0,
        LengthValue::Percentage(p) => (*p as f32 / 100.0) * box_dim,
        _ => 0.0,
    }
}

/// 简单的字符串哈希函数（用于从 URL 字符串生成 ImageKey）。
pub fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

/// 判断 `href` 是否应原样使用（已有 scheme、片段锚点等），不参与相对 base 解析。
fn is_non_relative_href(href: &str) -> bool {
    if href.starts_with('#') {
        return true;
    }
    if href.starts_with("data:")
        || href.starts_with("http://")
        || href.starts_with("https://")
        || href.starts_with("zero://")
        || href.starts_with("file://")
        || href.starts_with("ftp://")
    {
        return true;
    }
    if let Some(colon) = href.find(':') {
        let scheme = &href[..colon];
        if !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || matches!(c, '+' | '-' | '.'))
            && scheme.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        {
            return true;
        }
    }
    false
}

/// 将相对 URL 解析为绝对 URL（解析失败时原样返回 `href`）。
pub fn resolve_document_url(base_url: &str, href: &str) -> String {
    let href = href.trim();
    if href.is_empty() || is_non_relative_href(href) {
        return href.to_string();
    }
    match url::Url::parse(base_url).and_then(|base| base.join(href)) {
        Ok(abs) => abs.to_string(),
        Err(_) => href.to_string(),
    }
}

/// 图像子资源 lookup key：始终对**解析后的绝对 URL** 哈希，与 webview 抓取路径一致。
pub fn image_resource_key(src: &str, document_url: Option<&str>) -> u64 {
    // R4996：src 属性值首尾可含空白（HTML 源内长 base64 折行的尾随换行等）。URL 解析
    // 按规范剥离首尾 C0 控制符与空格（https://url.spec.whatwg.org/#concept-basic-url-parser），
    // 抓取路径（`extract_img_srcs` trim 后）与 key 哈希（此处）须同串——否则 ImageCache
    // / img_intrinsic_sizes 双查皆 miss，图像整体不渲染（flex-aspect-ratio-img-row-016）。
    let src = src.trim();
    let resolved = document_url
        .map(|base| resolve_document_url(base, src))
        .unwrap_or_else(|| src.to_string());
    simple_hash(&resolved)
}
/// R4296（CSS2 §8.3/§8.4 + filter-effects-2 #BackdropFilterProperty）：inline 非原子盒的
/// 垂直 border-box **缺失外延量** `(top, bottom)`（R4297 改口径：缺失量 = CSS 厚度 − 盒
/// 已含厚度，非全额 CSS 厚度）。
///
/// CSS2 §10.8.1：inline 的垂直 margin/padding/border 不入行盒高，但**绘制**发生于 inline
/// box 之外、可上溢/下溢覆盖邻接行盒。box-level 背景（paint_background）与 backdrop-filter
/// 区域（apply_backdrop_filter）的矩形须补足缺失厚度——R1442 已对**多行** inline 的
/// fragment bg/border 做 bleed，此处为**单行** box-level 统一口径。厚度取 Px 值（Em/% 等
/// 非 Px 与 R1442 px_of 同口径记 0）。非 inline 盒 → (0, 0)。
/// driving: backdrop-filter-inline-positioning（50px padding 的 inline span，backdrop
/// 区域应 ~120px 高而非 ~20px 行盒）。
///
/// **R4297 缺失量口径**：taffy 对 display:inline 归零垂直 padding（converter
/// `inline_vpadding_zero`）但**保留 border**（box h = 行盒 + border，bleed-001 实证
/// h=55=40+15）；R2197 空 inline 全几何重写盒与 R4297 fragment 行位同步盒则把各自厚度
/// 写进了 box 字段。全额外延会把已含部分双计（bleed-001：h55 + 全额 40 = 95 vs chromium
/// 80）。故按 box 字段差值取 `max(0, style − box)`——未同步 taffy 盒 padding_top=0 得
/// 全额 padding、border 差 0；已同步盒差值为 0。
///
/// **default-off**（调用方经 `Painter.inline_bleed_enabled` 构造期单次读取 env
/// `ZW_INLINE_BLEED=1` 门控，热路径零 env 查询——R3858 同款教训）。R4297 层②
/// fragment 行位同步落地后行位已锚定，转由 corpus A/B 定 default。
pub fn inline_box_vertical_bleed(style: &ComputedStyle, box_node: &LayoutBox) -> (f32, f32) {
    if !matches!(style.display, zero_css_parser::values::DisplayValue::Inline) {
        return (0.0, 0.0);
    }
    use zero_style_system::property::types::BorderStyleValue;
    let side = |w: &LengthValue, s: &BorderStyleValue| match s {
        BorderStyleValue::None | BorderStyleValue::Hidden => 0.0,
        _ => length_to_f32(w),
    };
    let missing = |style_v: f32, box_v: f32| (style_v - box_v).max(0.0);
    let top = missing(length_to_f32(&style.padding_top), box_node.padding_top)
        + missing(
            side(&style.border_top_width, &style.border_top_style),
            box_node.border_top,
        );
    let bottom = missing(length_to_f32(&style.padding_bottom), box_node.padding_bottom)
        + missing(
            side(&style.border_bottom_width, &style.border_bottom_style),
            box_node.border_bottom,
        );
    (top, bottom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::{LengthValue, TransformValue};

    // ── apply_text_transform ────────────────────────────────────────────

    #[test]
    fn test_text_transform_none() {
        assert_eq!(
            apply_text_transform("hello World", &TextTransformValue::None),
            "hello World"
        );
    }

    #[test]
    fn test_text_transform_uppercase() {
        assert_eq!(apply_text_transform("hello", &TextTransformValue::Uppercase), "HELLO");
    }

    #[test]
    fn test_text_transform_lowercase() {
        assert_eq!(apply_text_transform("HELLO", &TextTransformValue::Lowercase), "hello");
    }

    #[test]
    fn test_text_transform_capitalize() {
        assert_eq!(
            apply_text_transform("hello world", &TextTransformValue::Capitalize),
            "Hello World"
        );
    }

    #[test]
    fn test_text_transform_capitalize_with_spaces() {
        assert_eq!(
            apply_text_transform("  multiple  spaces", &TextTransformValue::Capitalize),
            "  Multiple  Spaces"
        );
    }

    #[test]
    fn test_text_transform_capitalize_empty() {
        assert_eq!(apply_text_transform("", &TextTransformValue::Capitalize), "");
    }

    #[test]
    fn test_text_transform_capitalize_numbers() {
        assert_eq!(
            apply_text_transform("abc123def", &TextTransformValue::Capitalize),
            "Abc123def"
        );
    }

    // ── R2327：text-transform full-width / full-size-kana（CSS Text 3 §3.1）──

    #[test]
    fn test_r2327_text_transform_full_width() {
        // CSS Text 3 §3.1：ASCII 可打印 U+0021–U+007E → 全角 U+FF01–U+FF5E（+0xFEE0）；
        // U+0020 → 表意空格 U+3000（R4381，spec + fullwidth-009 assert）；非 ASCII 不变。
        // driving: css-text text-transform-fullwidth-001/009。
        assert_eq!(
            apply_text_transform("Hello!", &TextTransformValue::FullWidth),
            "\u{FF28}\u{FF45}\u{FF4C}\u{FF4C}\u{FF4F}\u{FF01}", // Ｈｅｌｌｏ！
            "ASCII letters + punct -> fullwidth"
        );
        // 数字与符号（空格转表意空格）
        assert_eq!(
            apply_text_transform("A1 #", &TextTransformValue::FullWidth),
            "\u{FF21}\u{FF11}\u{3000}\u{FF03}", // Ａ１　＃（空格 → U+3000，# → U+FF03）
            "digits/symbols -> fullwidth, space -> ideographic space"
        );
        // 空格（U+0020）→ 表意空格（U+3000），两侧字母转全角
        let r = apply_text_transform("A B", &TextTransformValue::FullWidth);
        assert_eq!(
            r, "\u{FF21}\u{3000}\u{FF22}",
            "space U+0020 -> ideographic U+3000, letters fullwidth'd"
        );
        // 非 ASCII（中文）不变
        assert_eq!(apply_text_transform("中文", &TextTransformValue::FullWidth), "中文");
    }

    #[test]
    fn test_r2327_text_transform_full_size_kana() {
        // CSS Text 3 §3.1：小書き仮名 → 普通仮名。driving: css-text text-transform-full-size-kana-005。
        assert_eq!(
            apply_text_transform("ぁぃぅぇぉっゃゅょゎ", &TextTransformValue::FullSizeKana),
            "あいうえおつやゆよわ",
            "hiragana small -> regular"
        );
        assert_eq!(
            apply_text_transform("ァィゥェォッャュョォ", &TextTransformValue::FullSizeKana),
            "アイウエオツヤユヨオ",
            "katakana small -> regular (ォ->オ)"
        );
        // ヵ ヶ → カ ケ
        assert_eq!(apply_text_transform("ヶ", &TextTransformValue::FullSizeKana), "ケ");
        // 普通仮名不变
        assert_eq!(apply_text_transform("あい", &TextTransformValue::FullSizeKana), "あい");
    }

    // ── length_to_f32 ───────────────────────────────────────────────────

    #[test]
    fn test_length_to_f32_px() {
        assert_eq!(length_to_f32(&LengthValue::Px(42.0)), 42.0);
    }

    #[test]
    fn test_length_to_f32_non_px() {
        assert_eq!(length_to_f32(&LengthValue::Percentage(50.0)), 0.0);
    }

    // ── simple_hash ─────────────────────────────────────────────────────

    #[test]
    fn test_simple_hash_deterministic() {
        let h1 = simple_hash("https://example.com/image.png");
        let h2 = simple_hash("https://example.com/image.png");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_simple_hash_different_inputs() {
        let h1 = simple_hash("abc");
        let h2 = simple_hash("def");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_simple_hash_empty() {
        let h = simple_hash("");
        assert_eq!(h, 5381); // initial value with no bytes processed
    }
}
