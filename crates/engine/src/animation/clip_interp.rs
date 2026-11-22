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
    ShapeControlPoint, ShapeEndpoint, ShapeFunctionValue, ShapePosition, geometry_box_keyword, path_to_css,
    shape_axis_comp_resolved, shape_to_css,
};
use zero_css_parser::values::{CalcExpr, ClipPathRadius, ClipPathValue, LengthValue, parse_clip_path};

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
            // round 组按 border-radius 全模型插值（h 四元组逐位 + v 组存在性
            // 一致——rect()/xywh() 同款 [`lerp_radius_tuple`]；单半径历史见 SH-A1 刀）
            let round = lerp_radius_tuple(frr.as_ref(), trr.as_ref(), t)?;
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
        // 刀 GBX：`<basic-shape> || <geometry-box>` 组合对——同盒走内层
        // 同构插值通道、盒保形发射（css-masking-1 §the-clip-path +
        // css-shapes-2 §interpolating-shape 组合规则）；异盒/裸盒对恒异构
        // → None 离散切换。内层走原串剥盒后字符串递归（inner 串化归
        // engine computed 通道、css-parser 串化面不含 circle/inset 族，
        // 解析-串化往返不闭合，故不经 clip_path_value_to_css）。
        // default border-box 发射省略后缀（串化通道同语义），其余盒保形
        // 后缀（geometry-box 插值语料 computed oracle：`circle(20px)
        // margin-box` 同盒插值保后缀）。
        (
            ClipPathValue::WithBox {
                shape: Some(_),
                geometry: fg,
            },
            ClipPathValue::WithBox {
                shape: Some(_),
                geometry: tg,
            },
        ) if fg == tg => {
            let f_inner = strip_geometry_box_suffix(from)?;
            let t_inner = strip_geometry_box_suffix(to)?;
            let inner = interpolate_clip_path(&f_inner, &t_inner, t)?;
            if matches!(
                fg,
                zero_css_parser::values::parse_extended_visual::MaskGeometryBoxValue::BorderBox
            ) {
                Some(inner)
            } else {
                Some(format!("{inner} {}", geometry_box_keyword(&fg)))
            }
        }
        _ => None,
    }
}

/// 刀 GBX：`<basic-shape> || <geometry-box>` 组合串剥盒——前缀/后缀单次
/// 命中（与 parse_clip_path 双序剥除同构）；裸盒或未命中 None（调用侧
/// match 臂已保证 WithBox 形，None 仅防前缀形误入）。
fn strip_geometry_box_suffix(v: &str) -> Option<String> {
    const BOX_KEYWORDS: [&str; 7] = [
        "border-box",
        "padding-box",
        "content-box",
        "margin-box",
        "fill-box",
        "stroke-box",
        "view-box",
    ];
    let v = v.trim();
    for kw in BOX_KEYWORDS {
        if let Some(rest) = v.strip_prefix(kw) {
            let rest = rest.trim_start();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    // 后缀形：闭括号后尾随 token
    if let Some(close) = v.rfind(')') {
        let suffix = v[close + 1..].trim();
        if BOX_KEYWORDS.contains(&suffix) {
            return Some(v[..=close].trim_end().to_string());
        }
    }
    None
}

/// 长度/百分比插值：同单位线性发射 chrome-canonical 数字（CSSOM shortest
/// 形——offset-path 通道 gCS verbatim 无再 parse，发射即终文）；px/% 以外
/// 单位离散。跨单位（px↔%）calc 混合（css-values-4 §8.5.6 interpolation-calc，
/// 刀 G：逐单位系数线性 `calc(P% ± Xpx)`、px 系数零折叠纯 %，复用刀 E
/// blend_rule_inset_lp 模型——css-shapes-1 §basic-shape-interpolation 坐标/
/// 半径槽同文法，刀 E GONE 定谳第 4 条 clip-path-interpolation-002 靶面）。
pub(super) fn lerp_len(f: &LengthValue, v: &LengthValue, t: f64) -> Option<String> {
    match (f, v) {
        (LengthValue::Px(a), LengthValue::Px(b)) => Some(format!("{}px", fmt_num(lerp(*a, *b, t)))),
        (LengthValue::Percentage(a), LengthValue::Percentage(b)) => Some(format!("{}%", fmt_num(lerp(*a, *b, t)))),
        (LengthValue::Px(a), LengthValue::Percentage(b)) => super::blend_rule_inset_lp(a, "px", b, "%", t),
        (LengthValue::Percentage(a), LengthValue::Px(b)) => super::blend_rule_inset_lp(a, "%", b, "px", t),
        (LengthValue::Calc(a), LengthValue::Calc(b)) => lerp_calc_same_shape(a, b, t),
        _ => None,
    }
}

/// Calc 树同构对插值（css-values-4 §8.5.6 interpolation-calc：同结构逐系数
/// 线性）。仅收 `<percentage> <op> <length-percentage>` 同 op 同构形——
/// parse_position_pair 4 值臂缘偏移 resolve 产出形（`right/bottom o` 长度
/// 偏移 → `Calc(100% − o)`；% 偏移折纯百分比不入此臂）。off 同单位逐系数
/// 线性；px 系数符号折叠发射（负 off 翻转为 + 项）；px 系数零 → 纯 %
/// 简化（blend_rule_inset_lp 端点简化同约定）；纯 % off 对整树折单百分比
/// （`calc(100% − 30%)` ≡ `70%`，css-values-4 §calc 纯 % 代数）。px↔%
/// 跨单位 off / 异构 / 嵌套树返 None（调用方离散切换，
/// clip_pair_echo_preserve 门语义不变）。发射 chrome-canonical 文本（fmt_num
/// 6dp 去尾零，与 inset lerp 发射同约定；expected/got 两侧经 shim
/// roundNumbers 同通道比对）。
/// <https://drafts.csswg.org/css-values-4/#interpolation-calc>
fn lerp_calc_same_shape(a: &CalcExpr, b: &CalcExpr, t: f64) -> Option<String> {
    use zero_css_parser::values::{CalcExpr as CE, CalcOp};
    /// `<percentage> <op> <length-percentage>` 同构形拆解。
    fn pct_op_off(e: &CE) -> Option<(f64, CalcOp, &LengthValue)> {
        if let CE::BinaryOp(l, op, r) = e
            && matches!(op, CalcOp::Add | CalcOp::Subtract)
            && let (CE::Length(LengthValue::Percentage(p)), CE::Length(off)) = (&**l, &**r)
        {
            return Some((*p, op.clone(), off));
        }
        None
    }
    let (fa, fop, fox) = pct_op_off(a)?;
    let (ta, top, tox) = pct_op_off(b)?;
    if fop != top {
        return None;
    }
    let pct = fmt_num(lerp(fa, ta, t));
    match (fox, tox) {
        (LengthValue::Px(fv), LengthValue::Px(tv)) => {
            let signed = if fop == CalcOp::Subtract {
                -lerp(*fv, *tv, t)
            } else {
                lerp(*fv, *tv, t)
            };
            if signed.abs() < 1e-9 {
                return Some(format!("{pct}%"));
            }
            let op = if signed < 0.0 { " - " } else { " + " };
            Some(format!("calc({pct}%{op}{}px)", fmt_num(signed.abs())))
        }
        (LengthValue::Percentage(fv), LengthValue::Percentage(tv)) => {
            let off = lerp(*fv, *tv, t);
            let v = if fop == CalcOp::Subtract {
                lerp(fa, ta, t) - off
            } else {
                lerp(fa, ta, t) + off
            };
            Some(format!("{}%", fmt_num(v)))
        }
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

/// offset-path 插值：ray↔ray（同 ray-size 角度线性）、path()↔path()（段级
/// 绝对化 canonical 同构 lerp / 结构失配 None）、shape↔shape（coord-box
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
    // 刀 MOTION-2：path() 对（css-motion-1 §offset-path 插值 = css-shapes-2
    // §interpolating-shape path 规则）——双侧段级绝对化 canonical（相对→绝对、
    // 小写→大写、Z 回子路径起点语义；值面不做 S/T 平滑控制点反射——反射属
    // 渲染语义，值面仅显式实参）后同构（段数/命令字母/实参数全等）逐实参
    // 线性（A 段 flag 位钳 0/1），端点 canonical 发射（`m 50 60 h 20` at(0) =
    // `path('M 50 60 H 70')`，语料 002 钉面）。发射 offset-path 通道
    // chrome-canonical 单引号 path('...')（与 clip-path 双引号通道分层，
    // css-motion 语料钉面）；fill-rule 形属 clip-path 文法，offset-path 端点
    // 不含——不进本臂。
    //
    // 结构失配（段数/命令/实参数不等）返回 None——插值能力信号语义，调用方
    // 按通道分派离散（v2 订正，0535 电池 18 P2F 实证）：transitions 启动门
    // （css-transitions §starting-of-transitions 非 animatable 对不启动＝静态
    // 跳终值，flip-at--Infinity）、allow-discrete 注册 discrete 纯翻转、动画
    // 车道 §8.5.6 翻转原语。臂内 canonical 离散会骗过启动门（Some ⇒ 门判
    // 可插值 ⇒ 过渡启动 ⇒ t<0.5 发 from ≠ 语料期望终值）。
    if let (Some(ClipPathValue::Path(fpv)), Some(ClipPathValue::Path(tpv))) = (parse_clip_path(f), parse_clip_path(v))
        && fpv.fill_rule.is_none()
        && tpv.fill_rule.is_none()
        && let (Some(fs), Some(ts)) = (canonicalize_path_segs(&fpv.segs), canonicalize_path_segs(&tpv.segs))
        && let Some(chosen) = lerp_path_segs(&fs, &ts, t)
    {
        return Some(format!("path('{}')", fmt_path_segs(&chosen)));
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

/// 调用方离散车道（@keyframes/Web Animations 求值 `None` 回落，css-values-4
/// §8.5.6 t<0.5 → from else to）offset-path 翻转发射：双侧端点均为无 fill-rule
/// 的 path() 时绝对化 canonical 后单引号发射——纯 path() 失配对的车道端点是
/// computed 形（双引号通道），原文翻转与语料期望（声明形单引号）引号风格不符
/// （0535 电池 42 案实证）。任一侧非 path()（shape()/ray()/coord-box/none 混合
/// 对）则双侧回落声明原文 verbatim 翻转——混合对语料期望声明原文（008 页 20 案
/// 实证：`path("M10 10 h 5")` 侧被 canonical 成 `path('M 10 10 H 15')` 而期望
/// 仍为双引号原文）。失配本身不在此判：能力信号（None）由
/// [`interpolate_offset_path`] 发出，此处只承担车道翻转的发射风格。
pub(crate) fn discrete_offset_path_canonical(from: &str, to: &str, t: f64) -> String {
    let canon_side = |v: &str| {
        parse_clip_path(v.trim()).and_then(|p| match p {
            ClipPathValue::Path(pv) if pv.fill_rule.is_none() => {
                canonicalize_path_segs(&pv.segs).map(|segs| format!("path('{}')", fmt_path_segs(&segs)))
            }
            _ => None,
        })
    };
    match (canon_side(from), canon_side(to)) {
        (Some(f), Some(v)) => {
            if t < 0.5 {
                f
            } else {
                v
            }
        }
        // 混合对：双侧声明原文 verbatim 翻转。
        _ => {
            if t < 0.5 {
                from.trim().to_string()
            } else {
                to.trim().to_string()
            }
        }
    }
}

/// 命令字母实参数（与 css-parser [`parse_svg_path_data`] push_segment 文法
/// 一致；重复实参隐式段按 arity 分块逐段处理）。
fn path_seg_arity(c: char) -> Option<usize> {
    Some(match c {
        'M' | 'L' | 'T' => 2,
        'H' | 'V' => 1,
        'S' | 'Q' => 4,
        'C' => 6,
        'A' => 7,
        'Z' => 0,
        _ => return None,
    })
}

/// path data 段级绝对化 canonical（SVG 2 §path data dTKn 语义：相对命令基
/// 当前点、H/V 单轴保另一轴、Z 回子路径起点且其后相对命令基该起点、A 仅
/// 端点转换 rx/ry/rot/flags 保形；初值当前点 (0,0)——`m 10 20` 同绝对）。
/// 重复实参隐式段拆为逐命令段；全大写发射。解析器已保文法合法，本函数
/// 对非法 arity 返 None 兜底。
fn canonicalize_path_segs(segs: &[PathSeg]) -> Option<Vec<PathSeg>> {
    let mut out = Vec::with_capacity(segs.len());
    let mut cx = 0.0f64;
    let mut cy = 0.0f64;
    let mut sx = 0.0f64;
    let mut sy = 0.0f64;
    for s in segs {
        let cmd = s.cmd.to_ascii_uppercase();
        let rel = s.cmd.is_ascii_lowercase();
        let a = &s.args;
        if cmd == 'Z' {
            if !a.is_empty() {
                return None;
            }
            cx = sx;
            cy = sy;
            out.push(PathSeg {
                cmd: 'Z',
                args: Vec::new(),
            });
            continue;
        }
        let arity = path_seg_arity(cmd)?;
        if a.is_empty() || a.len() % arity != 0 {
            return None;
        }
        let ax = |cx: f64, i: usize, c: &[f64]| if rel { cx + c[i] } else { c[i] };
        let ay = |cy: f64, i: usize, c: &[f64]| if rel { cy + c[i] } else { c[i] };
        for chunk in a.chunks(arity) {
            let (mut args, x, y) = match cmd {
                'M' | 'L' => (
                    vec![ax(cx, 0, chunk), ay(cy, 1, chunk)],
                    ax(cx, 0, chunk),
                    ay(cy, 1, chunk),
                ),
                'H' => (vec![ax(cx, 0, chunk)], ax(cx, 0, chunk), cy),
                'V' => (vec![ay(cy, 0, chunk)], cx, ay(cy, 0, chunk)),
                'C' => (
                    vec![
                        ax(cx, 0, chunk),
                        ay(cy, 1, chunk),
                        ax(cx, 2, chunk),
                        ay(cy, 3, chunk),
                        ax(cx, 4, chunk),
                        ay(cy, 5, chunk),
                    ],
                    ax(cx, 4, chunk),
                    ay(cy, 5, chunk),
                ),
                'S' | 'Q' => (
                    vec![ax(cx, 0, chunk), ay(cy, 1, chunk), ax(cx, 2, chunk), ay(cy, 3, chunk)],
                    ax(cx, 2, chunk),
                    ay(cy, 3, chunk),
                ),
                'T' => (
                    vec![ax(cx, 0, chunk), ay(cy, 1, chunk)],
                    ax(cx, 0, chunk),
                    ay(cy, 1, chunk),
                ),
                'A' => (
                    vec![
                        chunk[0],
                        chunk[1],
                        chunk[2],
                        chunk[3],
                        chunk[4],
                        ax(cx, 5, chunk),
                        ay(cy, 6, chunk),
                    ],
                    ax(cx, 5, chunk),
                    ay(cy, 6, chunk),
                ),
                _ => return None,
            };
            // A 段 flag 位规范化钳 0/1（端点转换不动 flags 槽位语义）。
            if cmd == 'A' {
                args[3] = if args[3] >= 0.5 { 1.0 } else { 0.0 };
                args[4] = if args[4] >= 0.5 { 1.0 } else { 0.0 };
            }
            cx = x;
            cy = y;
            if cmd == 'M' {
                sx = x;
                sy = y;
            }
            out.push(PathSeg { cmd, args });
        }
    }
    Some(out)
}

/// 绝对化段列表同构逐实参线性（段数/命令/实参数全等门；A 段 flag 位钳
/// 0/1——css-shapes-2 §interpolating-shape path 规则）。
fn lerp_path_segs(f: &[PathSeg], v: &[PathSeg], t: f64) -> Option<Vec<PathSeg>> {
    if f.len() != v.len() {
        return None;
    }
    f.iter()
        .zip(v.iter())
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
                    if fs.cmd == 'A' && fs.args.len() == 7 && (i == 3 || i == 4) {
                        if n >= 0.5 { 1.0 } else { 0.0 }
                    } else {
                        n
                    }
                })
                .collect::<Vec<_>>();
            Some(PathSeg { cmd: fs.cmd, args })
        })
        .collect()
}

/// offset-path 通道 path data 发射（`M 50 60 H 70` 空白连接形；fmt_num
/// 6dp 去尾零——lerp 产出再 parse 幂等）。
fn fmt_path_segs(segs: &[PathSeg]) -> String {
    segs.iter()
        .map(|s| fmt_seg(s.cmd, &s.args))
        .collect::<Vec<_>>()
        .join(" ")
}

/// MOTION-2 face 标记（电池 FACE-MARKER 门判别 b0/b1 装载态；代码字面
/// ASCII 保证 strings 可提取）。
#[used]
pub(crate) static MOTION2_FACE_MARKER: &str = "MOTION2 offset-path-path-absolute-canonical-discrete-single-quote";

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_interp_edge_offset_calc_same_shape() {
        // css-masking clip-path-interpolation-001 / css-shapes
        // shape-outside-interpolation 语料主形态：4 值缘偏移端点对——x 缘
        // 百分比折数（right 5%→95%）、y 缘偏移 Calc(100% − off) 同构逐系数
        // 线性（css-values-4 §8.5.6 interpolation-calc）。此前 Calc↔Calc 落
        // None → echo 保留门回落基线（B0 伪通过池）——css-parser calc 位置
        // 词法修复（split_top_ws_depth）后语料期望串写面可解析，伪通过
        // 翻真失败，本臂兑现真插值。
        let v = interpolate_clip_path(
            "circle(25% at right 5% bottom 15px)",
            "circle(45% at right 25% bottom 35px)",
            0.25,
        )
        .expect("edge-offset calc 同构对应可插值");
        assert_eq!(v, "circle(30% at 90% calc(100% - 20px))");
        let v = interpolate_clip_path(
            "circle(25% at right 5% bottom 15px)",
            "circle(45% at right 25% bottom 35px)",
            0.5,
        )
        .expect("t=0.5 可插值");
        assert_eq!(v, "circle(35% at 85% calc(100% - 25px))");
        let v = interpolate_clip_path(
            "circle(25% at right 5% bottom 15px)",
            "circle(45% at right 25% bottom 35px)",
            0.75,
        )
        .expect("t=0.75 可插值");
        assert_eq!(v, "circle(40% at 80% calc(100% - 30px))");
    }

    #[test]
    fn test_clip_interp_calc_same_shape_edges() {
        // 端点简化：px 系数零 → 纯 %；纯 % off 对整树折单百分比；
        // 异构（op 失配/嵌套）返 None → 调用方离散。（2 词缘偏移形
        // `right 10px` 在 parse_position_pair 2 值臂属轴指派 FIXME 域
        // ——按语料计分契约拒收缘偏移语义，钉测用 4 值形。）
        let v = interpolate_clip_path(
            "circle(10% at right 10px top 20px)",
            "circle(30% at right 30px top 40px)",
            0.5,
        )
        .unwrap();
        assert_eq!(v, "circle(20% at calc(100% - 20px) 30px)");
        let a = LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
            Box::new(CalcExpr::Length(LengthValue::Percentage(100.0))),
            zero_css_parser::values::CalcOp::Subtract,
            Box::new(CalcExpr::Length(LengthValue::Percentage(30.0))),
        )));
        let b = LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
            Box::new(CalcExpr::Length(LengthValue::Percentage(100.0))),
            zero_css_parser::values::CalcOp::Subtract,
            Box::new(CalcExpr::Length(LengthValue::Percentage(10.0))),
        )));
        let (LengthValue::Calc(ae), LengthValue::Calc(be)) = (&a, &b) else {
            unreachable!("构造恒 Calc")
        };
        assert_eq!(lerp_calc_same_shape(ae, be, 0.5).as_deref(), Some("80%"));
        // op 失配异构：None。
        let add = LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
            Box::new(CalcExpr::Length(LengthValue::Percentage(100.0))),
            zero_css_parser::values::CalcOp::Add,
            Box::new(CalcExpr::Length(LengthValue::Percentage(30.0))),
        )));
        let LengthValue::Calc(add_e) = &add else {
            unreachable!("构造恒 Calc")
        };
        assert_eq!(lerp_calc_same_shape(ae, add_e, 0.5), None);
    }

    #[test]
    fn test_clip_interp_geometry_box_same_box_interp_and_cross_box_discrete() {
        // 刀 GBX（css-masking clip-path-geometry-box-interpolation 语料钉面）：
        // 同盒对走内层同构插值、非 default 盒保形后缀（oracle test 6：
        // circle(10px) margin-box ↔ circle(30px) margin-box at 0.5 →
        // `circle(20px) margin-box`）；default border-box 同盒对省略后缀
        // （computed 语义）；异盒对恒异构 → None 离散切换（oracle test 4/5
        // allow-discrete 翻转）；前缀盒形与后缀盒形同盒等价。
        let v = interpolate_clip_path("circle(10px) margin-box", "circle(30px) margin-box", 0.5)
            .expect("同盒同构对应可插值");
        assert_eq!(v, "circle(20px) margin-box");
        let v = interpolate_clip_path("circle(10px) border-box", "circle(30px) border-box", 0.25)
            .expect("border-box 同盒对应可插值");
        assert_eq!(v, "circle(15px)");
        let v = interpolate_clip_path("margin-box circle(10px)", "circle(30px) margin-box", 0.5)
            .expect("前缀/后缀同盒等价");
        assert_eq!(v, "circle(20px) margin-box");
        assert_eq!(
            interpolate_clip_path("circle(10px) margin-box", "circle(30px) padding-box", 0.5),
            None
        );
        assert_eq!(
            interpolate_clip_path("inset(1px) margin-box", "circle(30px) margin-box", 0.5),
            None
        );
        // 裸盒对（无内层 shape）恒异构离散
        assert_eq!(interpolate_clip_path("border-box", "margin-box", 0.5), None);
    }

    #[test]
    fn test_offset_path_m2_canonical_lerp() {
        // css-motion-1 §offset-path path 插值（offset-path-interpolation-003
        // 钉面）：同构 M/H 逐实参 lerp 含外推采样。
        assert_eq!(
            interpolate_offset_path("path('M 50 60 H 70')", "path('M 10 140 H 270')", 0.125).as_deref(),
            Some("path('M 45 70 H 95')")
        );
        assert_eq!(
            interpolate_offset_path("path('M 50 60 H 70')", "path('M 10 140 H 270')", -1.0).as_deref(),
            Some("path('M 90 -20 H -130')")
        );
        assert_eq!(
            interpolate_offset_path("path('M 50 60 H 70')", "path('M 10 140 H 270')", 2.0).as_deref(),
            Some("path('M -30 220 H 470')")
        );
        // 相对端点 at(0) = canonical 绝对化形（003 m/h 块钉）。
        assert_eq!(
            interpolate_offset_path("path('m 50 60 h 20')", "path('M 10 140 H 270')", 0.0).as_deref(),
            Some("path('M 50 60 H 70')")
        );
        // C 相对块外推（002 钉面）。
        assert_eq!(
            interpolate_offset_path(
                "path('m 20 10 c 12 32 32 52 100 2190')",
                "path('M 20 10 C 40 50 60 70 200 3000')",
                -1.0
            )
            .as_deref(),
            Some("path('M 20 10 C 24 34 44 54 40 1400')")
        );
        // arc：相对 a 与绝对 A 同 canonical → 恒值（002 A 块钉面）。
        assert_eq!(
            interpolate_offset_path(
                "path('m 100 400 a 10 20 30 1 0 40 50')",
                "path('M 100 400 A 10 20 30 1 0 140 450')",
                0.5
            )
            .as_deref(),
            Some("path('M 100 400 A 10 20 30 1 0 140 450')")
        );
        // 结构失配（段数不匹配）：MOTION-2 v4 失配车道拆分后本函数发 None 能力
        // 信号，001 钉面的段数不匹配 0.5 翻转由 [`discrete_offset_path_canonical`]
        // 车道承担（canonical 单引号发射见该函数钉测）。
        assert_eq!(
            interpolate_offset_path("path('M 0 0 H 1 H 2')", "path('M 0 0 H 3')", 0.3).as_deref(),
            None
        );
        assert_eq!(
            interpolate_offset_path("path('M 0 0 H 1 H 2')", "path('M 0 0 H 3')", 0.5).as_deref(),
            None
        );
        // z 回子路径起点语义 + 相对 t 平滑目标（004 t 块 at(0) canonical 钉）。
        assert_eq!(
            interpolate_offset_path(
                "path('m 10 20 l 40 50 z l 40 60 z m 60 70 l 90 60 z t 70 130')",
                "path('M 210 220 L 170 190 Z L 90 120 Z M 110 130 L 200 230 Z T 220 220')",
                0.0
            )
            .as_deref(),
            Some("path('M 10 20 L 50 70 Z L 50 80 Z M 70 90 L 160 150 Z T 140 220')")
        );
        // h/v 链 lerp（004 h/v 块中点）。
        assert_eq!(
            interpolate_offset_path(
                "path('m 10 20 h 30 v 60 h 10 v -10 l 110 60')",
                "path('M 10 20 H 40 V 80 H 50 V 70 L 120 130')",
                0.5
            )
            .as_deref(),
            Some("path('M 10 20 H 40 V 80 H 50 V 70 L 140 130')")
        );
        // 混合大小写对同 canonical → 恒值（001 钉面）。
        assert_eq!(
            interpolate_offset_path("path('m 100 200 L 120 270')", "path('m 100 200 l 20 70')", 0.25).as_deref(),
            Some("path('M 100 200 L 120 270')")
        );
    }
}
