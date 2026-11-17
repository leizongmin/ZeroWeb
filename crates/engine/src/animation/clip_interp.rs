//! wptfyi-css 基本形状族刀：clip-path 插值（css-shapes-2 §interpolating-shape /
//! css-masking-1 #the-clip-path 动画区间）。
//!
//! 插值策略（与 [`crate::animation::transform_interp`] 同型）：
//! - 同构（同变体 + 同结构）逐分量线性插值，px/px、%/% 同单位保单位；
//! - 关键字位置分量经 [`zero_css_parser::values::shape_axis_comp_resolved`]
//!   解析为 % 数值参与插值（center→50%、left/top→0%、right/bottom→100%），
//!   edge-offset 形解析为 calc 树 → 不可插值（calc 代数插值共享残面）离散；
//! - arc 旋转角归 deg 线性；cw/large、fill-rule、by/to 侧别为离散一致性
//!   门槛（失配整对离散）；
//! - path() 同段数同命令字母逐实参线性插值（A 段 flag 位钳 0/1）；
//! - rect()/xywh() 分量插值 + `auto`/圆角槽一致性门槛；
//! - 异构 / 单位失配 / calc 端点返回 `None`，调用方按 `t ≥ 0.5` 离散切换
//!   （css-values-4 §8.5.6，既有默认臂同语义零回归）。

use zero_css_parser::values::parse_basic_shape::{
    BorderRadiusValue, FillRule, PathFunctionValue, PathSeg, RectCoord, ShapeAngleUnit, ShapeAxisComp, ShapeCommand,
    ShapeControlPoint, ShapeEndpoint, ShapeFunctionValue, ShapePosition, path_to_css, shape_axis_comp_resolved,
    shape_to_css,
};
use zero_css_parser::values::{ClipPathRadius, ClipPathValue, LengthValue, parse_clip_path};

use crate::animation::lerp;

/// 插值 `clip-path` 两个计算值字符串；`None` = 不可插值（调用方离散切换）。
pub(crate) fn interpolate_clip_path(from: &str, to: &str, t: f64) -> Option<String> {
    let f = parse_clip_path(from)?;
    let v = parse_clip_path(to)?;
    match (f, v) {
        (ClipPathValue::None, ClipPathValue::None) => Some("none".to_string()),
        (
            ClipPathValue::Inset {
                top: ft,
                right: fr,
                bottom: fb,
                left: fl,
                round: frr,
            },
            ClipPathValue::Inset {
                top: tt,
                right: tr,
                bottom: tb,
                left: tl,
                round: trr,
            },
        ) => {
            let top = lerp_len(&ft, &tt, t)?;
            let right = lerp_len(&fr, &tr, t)?;
            let bottom = lerp_len(&fb, &tb, t)?;
            let left = lerp_len(&fl, &tl, t)?;
            let round = lerp_radius_opt(frr.as_ref(), trr.as_ref(), t)?;
            Some(match round {
                Some(r) => format!("inset({top} {right} {bottom} {left} round {r})"),
                None => format!("inset({top} {right} {bottom} {left})"),
            })
        }
        (
            ClipPathValue::Circle {
                radius: frr,
                position: fp,
            },
            ClipPathValue::Circle {
                radius: tr,
                position: tp,
            },
        ) => {
            let radius = lerp_radius(&frr, &tr, t)?;
            let at = lerp_pos_opt(fp.as_ref(), tp.as_ref(), t)?;
            Some(match at {
                Some(p) => format!("circle({radius} at {p})"),
                None => format!("circle({radius})"),
            })
        }
        (
            ClipPathValue::Ellipse {
                rx: frx,
                ry: fry,
                position: fp,
            },
            ClipPathValue::Ellipse {
                rx: trx,
                ry: try_,
                position: tp,
            },
        ) => {
            let rx = lerp_radius(&frx, &trx, t)?;
            let ry = lerp_radius(&fry, &try_, t)?;
            let at = lerp_pos_opt(fp.as_ref(), tp.as_ref(), t)?;
            Some(match at {
                Some(p) => format!("ellipse({rx} {ry} at {p})"),
                None => format!("ellipse({rx} {ry})"),
            })
        }
        (
            ClipPathValue::Polygon {
                fill_rule: ff,
                points: fps,
            },
            ClipPathValue::Polygon {
                fill_rule: tf,
                points: tps,
            },
        ) => {
            if ff != tf || fps.len() != tps.len() {
                return None;
            }
            let pts = fps
                .iter()
                .zip(tps.iter())
                .map(|((fx, fy), (tx, ty))| Some(format!("{} {}", lerp_len(fx, tx, t)?, lerp_len(fy, ty, t)?)))
                .collect::<Option<Vec<_>>>()?;
            let prefix = match ff {
                zero_css_parser::values::PolygonFillRule::EvenOdd => "evenodd, ",
                _ => "",
            };
            Some(format!("polygon({prefix}{})", pts.join(", ")))
        }
        (ClipPathValue::Shape(fs), ClipPathValue::Shape(ts)) => {
            let shape = lerp_shape(&fs, &ts, t)?;
            Some(shape_to_css(&shape))
        }
        (ClipPathValue::Path(fpv), ClipPathValue::Path(tpv)) => {
            let path = lerp_path(&fpv, &tpv, t)?;
            Some(path_to_css(&path))
        }
        (ClipPathValue::Rect(fr), ClipPathValue::Rect(tr)) => {
            let top = lerp_rect_coord(&fr.top, &tr.top, t)?;
            let right = lerp_rect_coord(&fr.right, &tr.right, t)?;
            let bottom = lerp_rect_coord(&fr.bottom, &tr.bottom, t)?;
            let left = lerp_rect_coord(&fr.left, &tr.left, t)?;
            let round = lerp_radius_tuple(fr.round.as_ref(), tr.round.as_ref(), t)?;
            Some(match round {
                Some(r) => format!("rect({top} {right} {bottom} {left} round {r})"),
                None => format!("rect({top} {right} {bottom} {left})"),
            })
        }
        (ClipPathValue::Xywh(fx), ClipPathValue::Xywh(tx)) => {
            let x = lerp_len(&fx.x, &tx.x, t)?;
            let y = lerp_len(&fx.y, &tx.y, t)?;
            let w = lerp_len(&fx.w, &tx.w, t)?;
            let h = lerp_len(&fx.h, &tx.h, t)?;
            let round = lerp_radius_tuple(fx.round.as_ref(), tx.round.as_ref(), t)?;
            Some(match round {
                Some(r) => format!("xywh({x} {y} {w} {h} round {r})"),
                None => format!("xywh({x} {y} {w} {h})"),
            })
        }
        _ => None,
    }
}

/// 长度/百分比插值：同单位线性发射 chrome-canonical 数字（CSSOM shortest
/// 形——offset-path 通道 gCS verbatim 无再 parse，发射即终文）；px/% 以外
/// 单位离散。跨单位（px↔%）calc 混合（css-values-4 §8.5.6 interpolation-calc，
/// 刀 G：逐单位系数线性 `calc(P% ± Xpx)`、px 系数零折叠纯 %，复用刀 E
/// blend_rule_inset_lp 模型——css-shapes-1 §basic-shape-interpolation 坐标/
/// 半径槽同文法，刀 E GONE 定谳第 4 条 clip-path-interpolation-002 靶面）。
fn lerp_len(f: &LengthValue, v: &LengthValue, t: f64) -> Option<String> {
    match (f, v) {
        (LengthValue::Px(a), LengthValue::Px(b)) => Some(format!("{}px", fmt_num(lerp(*a, *b, t)))),
        (LengthValue::Percentage(a), LengthValue::Percentage(b)) => Some(format!("{}%", fmt_num(lerp(*a, *b, t)))),
        (LengthValue::Px(a), LengthValue::Percentage(b)) => super::blend_rule_inset_lp(a, "px", b, "%", t),
        (LengthValue::Percentage(a), LengthValue::Px(b)) => super::blend_rule_inset_lp(a, "%", b, "px", t),
        _ => None,
    }
}

/// 位置对（x, y）插值：关键字分量经 resolved 解析为数值（calc 树离散）。
fn lerp_pos(f: &(LengthValue, LengthValue), v: &(LengthValue, LengthValue), t: f64) -> Option<String> {
    let x = lerp_len(&f.0, &v.0, t)?;
    let y = lerp_len(&f.1, &v.1, t)?;
    Some(format!("{x} {y}"))
}

fn lerp_pos_opt(
    f: Option<&(LengthValue, LengthValue)>,
    v: Option<&(LengthValue, LengthValue)>,
    t: f64,
) -> Option<Option<String>> {
    match (f, v) {
        (Some(fp), Some(tp)) => Some(Some(lerp_pos(fp, tp, t)?)),
        (None, None) => Some(None),
        _ => None,
    }
}

/// 半径插值：Length 同单位线性；任一端为关键字（closest-side/farthest-side 等）
/// 即整对不可插值（css-shapes-1 §interpolation：关键字半径不参与插值，含
/// 关键字↔关键字对——Chrome/WPT 对该组期望恒为离散端点形，二十一轮 A/B
/// 18 REG 定谳修正，此前同关键字误判恒等可插值）。
fn lerp_radius(f: &ClipPathRadius, v: &ClipPathRadius, t: f64) -> Option<String> {
    match (f, v) {
        (ClipPathRadius::Length(fl), ClipPathRadius::Length(tl)) => lerp_len(fl, tl, t),
        _ => None,
    }
}

fn lerp_radius_opt(f: Option<&ClipPathRadius>, v: Option<&ClipPathRadius>, t: f64) -> Option<Option<String>> {
    match (f, v) {
        (Some(fr), Some(tr)) => Some(Some(lerp_radius(fr, tr, t)?)),
        (None, None) => Some(None),
        _ => None,
    }
}

/// rect()/xywh() `round <border-radius>` 槽：水平四元组逐位 + 垂直组
/// 存在性一致；发射 4 值全形（parse 镜像展开幂等）。
fn lerp_radius_tuple(f: Option<&BorderRadiusValue>, v: Option<&BorderRadiusValue>, t: f64) -> Option<Option<String>> {
    let (Some(fr), Some(tr)) = (f, v) else {
        return match (f, v) {
            (None, None) => Some(None),
            _ => None,
        };
    };
    let h =
        fr.h.iter()
            .zip(tr.h.iter())
            .map(|(a, b)| lerp_len(a, b, t))
            .collect::<Option<Vec<_>>>()?;
    let vpart = match (&fr.v, &tr.v) {
        (Some(fv), Some(tv)) => {
            let vs = fv
                .iter()
                .zip(tv.iter())
                .map(|(a, b)| lerp_len(a, b, t))
                .collect::<Option<Vec<_>>>()?;
            format!(" / {}", vs.join(" "))
        }
        (None, None) => String::new(),
        _ => return None,
    };
    Some(Some(format!("{}{vpart}", h.join(" "))))
}

/// rect() 坐标：auto↔auto 保持；长度↔长度同单位线性；失配离散。
fn lerp_rect_coord(f: &RectCoord, v: &RectCoord, t: f64) -> Option<String> {
    match (f, v) {
        (RectCoord::Auto, RectCoord::Auto) => Some("auto".to_string()),
        (RectCoord::Length(fl), RectCoord::Length(tl)) => lerp_len(fl, tl, t),
        _ => None,
    }
}

/// shape() 同构逐命令插值（css-shapes-2 §interpolating-shape：命令序列同构
/// 逐命令分量线性；fill-rule / by-to 侧别 / arc cw-large 一致性门槛）。
fn lerp_shape(f: &ShapeFunctionValue, v: &ShapeFunctionValue, t: f64) -> Option<ShapeFunctionValue> {
    if !fill_rule_eq(f.fill_rule, v.fill_rule) || f.commands.len() != v.commands.len() {
        return None;
    }
    let from = lerp_shape_position(&f.from, &v.from, t)?;
    let commands = f
        .commands
        .iter()
        .zip(v.commands.iter())
        .map(|(fc, tc)| lerp_shape_command(fc, tc, t))
        .collect::<Option<Vec<_>>>()?;
    Some(ShapeFunctionValue {
        fill_rule: f.fill_rule,
        from,
        commands,
    })
}

fn fill_rule_eq(f: Option<FillRule>, v: Option<FillRule>) -> bool {
    matches!(
        (f, v),
        (None | Some(FillRule::Nonzero), None | Some(FillRule::Nonzero))
            | (Some(FillRule::Evenodd), Some(FillRule::Evenodd))
    )
}

fn lerp_shape_position(f: &ShapePosition, v: &ShapePosition, t: f64) -> Option<ShapePosition> {
    let (fx, fy) = (shape_axis_comp_resolved(&f.x), shape_axis_comp_resolved(&f.y));
    let (tx, ty) = (shape_axis_comp_resolved(&v.x), shape_axis_comp_resolved(&v.y));
    let x = lerp_len(&fx, &tx, t)?;
    let y = lerp_len(&fy, &ty, t)?;
    // 再 parse 回 LengthValue（emit 中间形 "8px"/"50%"——`from` 声明级
    // 串化保关键字形，此处插值输出数值形即可，引擎层串化保真）。
    let px = parse_len_back(&x)?;
    let py = parse_len_back(&y)?;
    Some(ShapePosition {
        x: ShapeAxisComp::Len(px),
        y: ShapeAxisComp::Len(py),
    })
}

/// 插值中间形（"8px"/"7%"）回读 LengthValue（仅 px/% 两族）。
fn parse_len_back(s: &str) -> Option<LengthValue> {
    if let Some(v) = s.strip_suffix('%') {
        return v.parse::<f64>().ok().map(LengthValue::Percentage);
    }
    s.strip_suffix("px")
        .and_then(|v| v.parse::<f64>().ok())
        .map(LengthValue::Px)
}

fn lerp_endpoint(f: &ShapeEndpoint, v: &ShapeEndpoint, t: f64) -> Option<ShapeEndpoint> {
    match (f, v) {
        (ShapeEndpoint::Position(fp), ShapeEndpoint::Position(tp)) => {
            Some(ShapeEndpoint::Position(lerp_shape_position(fp, tp, t)?))
        }
        (ShapeEndpoint::Offset(fx, fy), ShapeEndpoint::Offset(tx, ty)) => {
            let x = parse_len_back(&lerp_len(fx, tx, t)?)?;
            let y = parse_len_back(&lerp_len(fy, ty, t)?)?;
            Some(ShapeEndpoint::Offset(x, y))
        }
        _ => None,
    }
}

fn lerp_control_point(f: &ShapeControlPoint, v: &ShapeControlPoint, t: f64) -> Option<ShapeControlPoint> {
    if f.from_ref != v.from_ref {
        return None;
    }
    Some(ShapeControlPoint {
        pos: lerp_shape_position(&f.pos, &v.pos, t)?,
        from_ref: f.from_ref,
    })
}

fn lerp_shape_command(f: &ShapeCommand, v: &ShapeCommand, t: f64) -> Option<ShapeCommand> {
    match (f, v) {
        (ShapeCommand::Move { end: fe }, ShapeCommand::Move { end: te }) => Some(ShapeCommand::Move {
            end: lerp_endpoint(fe, te, t)?,
        }),
        (ShapeCommand::Line { end: fe }, ShapeCommand::Line { end: te }) => Some(ShapeCommand::Line {
            end: lerp_endpoint(fe, te, t)?,
        }),
        (ShapeCommand::Hline { by: fb, x: fx }, ShapeCommand::Hline { by: tb, x: tx }) => {
            if fb != tb {
                return None;
            }
            let fxr = shape_axis_comp_resolved(fx);
            let txr = shape_axis_comp_resolved(tx);
            Some(ShapeCommand::Hline {
                by: *fb,
                x: ShapeAxisComp::Len(parse_len_back(&lerp_len(&fxr, &txr, t)?)?),
            })
        }
        (ShapeCommand::Vline { by: fb, y: fy }, ShapeCommand::Vline { by: tb, y: ty }) => {
            if fb != tb {
                return None;
            }
            let fyr = shape_axis_comp_resolved(fy);
            let tyr = shape_axis_comp_resolved(ty);
            Some(ShapeCommand::Vline {
                by: *fb,
                y: ShapeAxisComp::Len(parse_len_back(&lerp_len(&fyr, &tyr, t)?)?),
            })
        }
        (
            ShapeCommand::Curve {
                end: fe,
                c1: f1,
                c2: f2,
            },
            ShapeCommand::Curve {
                end: te,
                c1: t1,
                c2: t2,
            },
        ) => Some(ShapeCommand::Curve {
            end: lerp_endpoint(fe, te, t)?,
            c1: lerp_control_point(f1, t1, t)?,
            c2: match (f2, t2) {
                (Some(fcp), Some(tcp)) => Some(lerp_control_point(fcp, tcp, t)?),
                (None, None) => None,
                _ => return None,
            },
        }),
        (ShapeCommand::Smooth { end: fe, c1: f1 }, ShapeCommand::Smooth { end: te, c1: t1 }) => {
            Some(ShapeCommand::Smooth {
                end: lerp_endpoint(fe, te, t)?,
                c1: match (f1, t1) {
                    (Some(fcp), Some(tcp)) => Some(lerp_control_point(fcp, tcp, t)?),
                    (None, None) => None,
                    _ => return None,
                },
            })
        }
        (
            ShapeCommand::Arc {
                end: fe,
                rx: frx,
                ry: fry,
                cw: fcw,
                large: flarge,
                rotate: frot,
            },
            ShapeCommand::Arc {
                end: te,
                rx: trx,
                ry: try_,
                cw: tcw,
                large: tlarge,
                rotate: trot,
            },
        ) => {
            if fcw != tcw || flarge != tlarge {
                return None;
            }
            let rx = lerp_len(frx, trx, t)?;
            let ry = match (fry, try_) {
                (Some(fr), Some(tr)) => Some(parse_len_back(&lerp_len(fr, tr, t)?)?),
                (None, None) => None,
                _ => return None,
            };
            let rotate = match (frot, trot) {
                (Some((fa, _fu)), Some((ta, _tu))) => {
                    // 存储恒为弧度（unit 词仅声明级保形，见 parse_shape_angle
                    // ——归 deg 线性，发射 deg 形）。
                    let fd = fa.to_degrees();
                    let td = ta.to_degrees();
                    Some((lerp(fd, td, t).to_radians(), ShapeAngleUnit::Deg))
                }
                (None, None) => None,
                _ => return None,
            };
            Some(ShapeCommand::Arc {
                end: lerp_endpoint(fe, te, t)?,
                rx: parse_len_back(&rx)?,
                ry,
                cw: *fcw,
                large: *flarge,
                rotate,
            })
        }
        (ShapeCommand::Close, ShapeCommand::Close) => Some(ShapeCommand::Close),
        _ => None,
    }
}

/// path() 同段数同命令字母逐实参线性插值（css-shapes-2 §interpolating-shape
/// path 段；A 段 flag 位插值后钳 0/1 整数）。
fn lerp_path(f: &PathFunctionValue, v: &PathFunctionValue, t: f64) -> Option<PathFunctionValue> {
    if f.segs.len() != v.segs.len() || !fill_rule_eq(f.fill_rule, v.fill_rule) {
        return None;
    }
    let mut out = String::new();
    let segs = f
        .segs
        .iter()
        .zip(v.segs.iter())
        .map(|(fs, ts)| {
            if fs.cmd != ts.cmd || fs.args.len() != ts.args.len() {
                return None;
            }
            let args = fs
                .args
                .iter()
                .zip(ts.args.iter())
                .enumerate()
                .map(|(i, (a, b))| {
                    let n = lerp(*a, *b, t);
                    // A 段 idx 3/4 为 flag 位（rx ry rot large sweep x y）——
                    // 插值后钳整数 0/1。
                    if matches!(fs.cmd, 'A' | 'a') && fs.args.len() == 7 && (i == 3 || i == 4) {
                        if n >= 0.5 { 1.0 } else { 0.0 }
                    } else {
                        n
                    }
                })
                .collect::<Vec<_>>();
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&fmt_seg(fs.cmd, &args));
            Some(PathSeg { cmd: fs.cmd, args })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(PathFunctionValue {
        fill_rule: f.fill_rule,
        segs,
        raw: out.trim().to_string(),
    })
}

/// 段文本发射（`M 8 5` 形；数字 6dp 去尾零——path data 再 parse 幂等）。
fn fmt_seg(cmd: char, args: &[f64]) -> String {
    let mut s = format!("{cmd}");
    for (i, a) in args.iter().enumerate() {
        // A 段 idx 3/4 为 flag 位。
        let n = if matches!(cmd, 'A' | 'a') && (i == 3 || i == 4) {
            if *a >= 0.5 { 1.0 } else { 0.0 }
        } else {
            *a
        };
        s.push(' ');
        s.push_str(&fmt_num(n));
    }
    s
}

fn fmt_num(n: f64) -> String {
    let s = format!("{n:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

// ── offset-path（css-motion-1 §offset-path：ray() + 基本形状 + coord-box 后缀；
// 十三轮残面「ray()/基本形状文法族」插值臂。gCS offset-path verbatim 通道——
// 发射即终文，须 chrome-canonical 形） ─────────────────────────────────────

/// ray() 尺寸关键字（css-motion-1 §ray-size）。
const RAY_SIZES: [&str; 5] = [
    "closest-side",
    "closest-corner",
    "farthest-side",
    "farthest-corner",
    "sides",
];

/// coord-box 关键词（css-box-4 §coord-box；offset-path `|| <coord-box>` 后缀）。
const COORD_BOXES: [&str; 5] = ["content-box", "padding-box", "border-box", "margin-box", "view-box"];

/// offset-path 插值：ray↔ray（同 ray-size 角度线性）、shape↔shape（coord-box
/// 后缀完全一致才插形状——后缀差异 = 离散，css-shapes-2 §interpolating-shape
/// 组合规则）、纯 coord-box 恒值对；异构返回 None（调用方离散切换）。
pub(crate) fn interpolate_offset_path(from: &str, to: &str, t: f64) -> Option<String> {
    let f = from.trim();
    let v = to.trim();
    if f.eq_ignore_ascii_case("none") && v.eq_ignore_ascii_case("none") {
        return Some("none".to_string());
    }
    // ray() 对：同 ray-size 角度线性（at 子句存在性须一致——位置插值后置）。
    if let (Some((fdeg, fsize, fat)), Some((tdeg, tsize, tat))) = (parse_ray(f), parse_ray(v)) {
        if fsize == tsize && fat == tat {
            let deg = lerp(fdeg, tdeg, t);
            let size = fsize.map(|s| format!(" {s}")).unwrap_or_default();
            return Some(format!("ray({}deg{})", fmt_num(deg), size));
        }
        return None;
    }
    // 纯 coord-box 恒值对（css-motion-1 offset-path 单 coord-box 形）。
    let (fc, fbox) = split_trailing_coord_box(f);
    let (tc, tbox) = split_trailing_coord_box(v);
    if fc.is_empty() && tc.is_empty() {
        return (fbox == tbox).then(|| f.to_string());
    }
    // shape 对：后缀一致才走形状机制（后缀差异 = 不可插值离散）。
    if fbox == tbox {
        let shape = interpolate_clip_path(fc, tc, t)?;
        return Some(match fbox {
            Some(b) => format!("{shape} {b}"),
            None => shape,
        });
    }
    None
}

/// `ray(<angle> [<ray-size>] [at <position>])` 声明形解析（canonical 序；
/// 乱序 `&&` 形 supports 通道残面挂账——语料端点全 canonical 序）。
fn parse_ray(v: &str) -> Option<(f64, Option<&'static str>, bool)> {
    let inner = strip_fn_prefix(v, "ray")?;
    let at_split = inner.find(" at ").map(|i| (&inner[..i], &inner[i + 1..]));
    let (head, has_at) = match at_split {
        Some((h, tail)) => {
            // at 子句须以 at 引导且非空（位置文法 supports 通道 shim JS 校验域）。
            if !tail.trim().starts_with("at") || tail.trim().len() <= 2 {
                return None;
            }
            (h, true)
        }
        None => (inner, false),
    };
    let mut toks = head.split_whitespace();
    let deg = parse_ray_angle(toks.next()?)?;
    let size = match toks.next() {
        None => None,
        Some(s) => {
            let kw = RAY_SIZES.iter().find(|k| s.eq_ignore_ascii_case(k))?;
            Some(*kw)
        }
    };
    if toks.next().is_some() {
        return None;
    }
    Some((deg, size, has_at))
}

/// 剥 `<name>(` 前缀与 `)` 尾（大小写不敏感；返回括号体内文）。
fn strip_fn_prefix<'a>(v: &'a str, name: &str) -> Option<&'a str> {
    let body = v.strip_prefix(name)?;
    let rest = body.strip_prefix('(')?;
    rest.strip_suffix(')')
}

/// ray 角度实参（deg 直读；rad/grad/turn 经 parse_angle_to_radians 归 deg）。
fn parse_ray_angle(tok: &str) -> Option<f64> {
    if let Some(v) = tok.strip_suffix("deg") {
        return v.trim().parse::<f64>().ok();
    }
    zero_css_parser::values::parse_angle_to_radians(tok).map(|r| r.to_degrees())
}

/// 尾部 coord-box 后缀分离（`shape(...) border-box` 形；返回 (形状核心, 后缀)）。
fn split_trailing_coord_box(v: &str) -> (&str, Option<&'static str>) {
    for b in COORD_BOXES {
        if let Some(core) = v.strip_suffix(b) {
            let core = core.trim_end();
            return (core, Some(b));
        }
    }
    (v, None)
}
