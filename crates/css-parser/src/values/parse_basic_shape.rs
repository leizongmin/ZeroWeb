//! CSS basic-shape 命令式 `shape()` 函数与 `path()` / `rect()` / `xywh()`
//! 兄弟文法（css-shapes-2 §shape-function / §basic-shape-rect-notation）。
//!
//! clip-path 与 shape-outside 共享 [`ClipPathValue`] 承载；offset-path 的
//! 基本形状面（css-motion-1）后续同构消费。声明级串化保关键字
//! provenance（`from center` → `from center center`、`bottom right` →
//! `right bottom`、控制点 `from start|end|origin` 参考系按 to/by 缺省
//! 省略）；裸数字补 px；computed 绝对化（关键字→%、em→px、path 相对段
//! →绝对）归 engine 层（[`shape_axis_comp_resolved`] 供其复用）。
//!
//! <https://drafts.fxtf.org/css-shapes-2/#shape-function>
//! <https://drafts.fxtf.org/css-shapes-2/#basic-shape-rect-notation>
//! <https://drafts.fxtf.org/css-shapes-2/#path-function>
//! <https://drafts.fxtf.org/css-shapes-2/#serializing-basic-shape>

use super::types::parse_length;
use super::types::{CalcExpr, CalcOp, ClipPathValue, LengthValue};

// ── 模型类型 ────────────────────────────────────────────────────────────

/// 位置分量关键字/长度保形模型（声明级串化保关键字；computed 析 %
/// 见 [`shape_axis_comp_resolved`]）。
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeAxisComp {
    /// 长度/百分比/calc。
    Len(LengthValue),
    /// `center`（双轴通用）。
    Center,
    /// `left` / `x-start` 保形。
    Left,
    /// `right` / `x-end` 保形。
    Right,
    /// `x-start`（保词形）。
    XStart,
    /// `x-end`（保词形）。
    XEnd,
    /// `top` / `y-start` 保形。
    Top,
    /// `bottom` / `y-end` 保形。
    Bottom,
    /// `y-start`（保词形）。
    YStart,
    /// `y-end`（保词形）。
    YEnd,
    /// edge-offset 形（css-values-4 §position：`<edge> <offset>`；2 词形
    /// 缺轴补 center、4 词形双对）。right/bottom 缘折算 `100% − o` 在
    /// computed 通道完成。
    Edge(ShapeEdge, Box<ShapeAxisComp>),
}

/// edge-offset 的缘关键字（css-values-4 §position）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeEdge {
    /// `left`。
    Left,
    /// `right`。
    Right,
    /// `top`。
    Top,
    /// `bottom`。
    Bottom,
}

/// `<position>` 坐标对（x, y；声明级 2 分量保形）。
#[derive(Debug, Clone, PartialEq)]
pub struct ShapePosition {
    /// x 分量。
    pub x: ShapeAxisComp,
    /// y 分量。
    pub y: ShapeAxisComp,
}

/// 命令端点：`to <position>`（关键字保形）或 `by <length-percentage>{2}`
///（css-shapes-2 <coordinate-pair> 严格双 len-pct，关键字拒收——
/// `curve by top left` / `arc by top right` 语料钉面）。
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeEndpoint {
    /// `to <position>`（绝对形）。
    Position(ShapePosition),
    /// `by <length-percentage>{2}`（相对形）。
    Offset(LengthValue, LengthValue),
}

impl ShapeEndpoint {
    /// by（相对）形判定（控制点参考系缺省判定与 `to`/`by` 词串化共用）。
    pub fn is_by(&self) -> bool {
        matches!(self, ShapeEndpoint::Offset(..))
    }
}

/// shape() 控制点参考系（css-shapes-2 curve/smooth 控制点
/// `from start|end|origin` 后缀）。缺省参考系随命令方向：`to` 形缺省
/// `from origin`、`by` 形缺省 `from start`（串化省略缺省形；语料钉面
/// shape-function-computed/valid：to 形 origin 省略而 start 保留、
/// by 形相反）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeCoordRef {
    /// `from start`。
    Start,
    /// `from end`。
    End,
    /// `from origin`。
    Origin,
}

/// shape() 曲线/平滑控制点：`<position> [from start|end|origin]`。
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeControlPoint {
    /// 控制点位置。
    pub pos: ShapePosition,
    /// 显式参考系（None = 未写，串化一律省略）。
    pub from_ref: Option<ShapeCoordRef>,
}

/// fill-rule 关键字（shape() 前导词与 path() 首参共享文法）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    /// `nonzero`（缺省；shape()/path() 串化均省略——
    /// `shape(nonzero from …)` → `shape(from …)` 语料钉面）。
    Nonzero,
    /// `evenodd`。
    Evenodd,
}

/// arc 旋转角单位（声明级保形——`rotate 0.52rad` 保 rad 语料钉面；
/// computed 归 deg 由 engine 完成）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeAngleUnit {
    /// `deg`。
    Deg,
    /// `rad`。
    Rad,
    /// `grad`。
    Grad,
    /// `turn`。
    Turn,
}

impl ShapeAngleUnit {
    /// 单位词串化。
    pub fn as_str(self) -> &'static str {
        match self {
            ShapeAngleUnit::Deg => "deg",
            ShapeAngleUnit::Rad => "rad",
            ShapeAngleUnit::Grad => "grad",
            ShapeAngleUnit::Turn => "turn",
        }
    }
}

/// shape() 命令（css-shapes-2 §shape-function <command>）。
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeCommand {
    /// `move to|by <endpoint>`。
    Move {
        /// 端点。
        end: ShapeEndpoint,
    },
    /// `line to|by <endpoint>`。
    Line {
        /// 端点。
        end: ShapeEndpoint,
    },
    /// `hline to <x-coordinate>|by <length-percentage>`（to 可带水平
    /// 关键字；by 严格 len-pct——`hline by left` 拒收语料钉面）。
    Hline {
        /// by（相对）形。
        by: bool,
        /// x 坐标。
        x: ShapeAxisComp,
    },
    /// `vline to <y-coordinate>|by <length-percentage>`。
    Vline {
        /// by（相对）形。
        by: bool,
        /// y 坐标。
        y: ShapeAxisComp,
    },
    /// `curve to|by <endpoint> with <cp> [/ <cp>]`（`with` 必需；
    /// `using`/`via` 拒收语料钉面）。
    Curve {
        /// 端点。
        end: ShapeEndpoint,
        /// 第一控制点。
        c1: ShapeControlPoint,
        /// 第二控制点（可选）。
        c2: Option<ShapeControlPoint>,
    },
    /// `smooth to|by <endpoint> [with <cp>]`。
    Smooth {
        /// 端点。
        end: ShapeEndpoint,
        /// 控制点（可选）。
        c1: Option<ShapeControlPoint>,
    },
    /// `arc to|by <endpoint> of <radii> [<arc-sweep> || <arc-size> ||
    /// rotate <angle>]`——修饰词 `||` 组合：任意序、各至多一次；串化
    /// canonical 序 cw → large → rotate，缺省（ccw/small/rotate 0）省略，
    /// 显式 cw/large 保形（语料钉面）。
    Arc {
        /// 端点。
        end: ShapeEndpoint,
        /// 椭圆 rx。
        rx: LengthValue,
        /// 椭圆 ry（None = 未写串化单值；`of 10px 10px` 等值双写保形）。
        ry: Option<LengthValue>,
        /// `cw`（缺省 ccw，串化省略）。
        cw: bool,
        /// `large`（缺省 small，串化省略）。
        large: bool,
        /// 旋转角（弧度 + 单位保形；0 角串化省略）。
        rotate: Option<(f64, ShapeAngleUnit)>,
    },
    /// `close`。
    Close,
}

/// path() 的 SVG path data 段（命令字母保形；数字串化归一 6dp）。
#[derive(Debug, Clone, PartialEq)]
pub struct PathSeg {
    /// 命令字母（M/m/L/l/H/h/V/v/C/c/S/s/Q/q/T/t/A/a/Z/z）。
    pub cmd: char,
    /// 数字实参（A 段 7 个：rx ry x-axis-rot large-arc sweep x y；flags 0/1）。
    pub args: Vec<f64>,
}

/// path() 值：`path([<fill-rule>,] <svg-path-data>)`（css-shapes-2）。
#[derive(Debug, Clone, PartialEq)]
pub struct PathFunctionValue {
    /// fill-rule（None/Nonzero 串化省略）。
    pub fill_rule: Option<FillRule>,
    /// path data 段列表（≥1；插值/engine 消费用）。
    pub segs: Vec<PathSeg>,
    /// 声明级串化保真的原文 data（trim 后逐字；path data trivial
    /// serialization 逐字回显，段级 canonical 归 engine computed 层）。
    pub raw: String,
}

/// rect() 坐标（`<length-percentage> | auto`；css-shapes-2
/// §basic-shape-rect-notation）。
#[derive(Debug, Clone, PartialEq)]
pub enum RectCoord {
    /// `auto`。
    Auto,
    /// 长度/百分比。
    Length(LengthValue),
}

/// border-radius 文法（rect()/xywh() `round` 槽；css-backgrounds-3
/// §border-radius 1-4 值 [/ 1-4 值]；半径非负——`round -20px` 拒收
/// 语料钉面；裸非零数字无单位拒收经 parse_shape_len）。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderRadiusValue {
    /// 水平半径四元组（tl tr br bl，parse 期镜像展开）。
    pub h: [LengthValue; 4],
    /// 垂直半径四元组（`/` 形显式写才有）。
    pub v: Option<[LengthValue; 4]>,
}

/// rect() 值：`rect(top right bottom left [round <border-radius>])`——
/// 空白分隔（逗号形非法，rect-function-invalid 语料钉面）；right/bottom
/// 语义为自左上角起算的右/下缘位置，computed 折算 inset（`100% − v`）
/// 归 engine 层。
#[derive(Debug, Clone, PartialEq)]
pub struct RectFunctionValue {
    /// top。
    pub top: RectCoord,
    /// right。
    pub right: RectCoord,
    /// bottom。
    pub bottom: RectCoord,
    /// left。
    pub left: RectCoord,
    /// 圆角（可选）。
    pub round: Option<BorderRadiusValue>,
}

/// xywh() 值：`xywh(x y w h [round <border-radius>])`（css-shapes-2；
/// 语料钉面 `xywh(50px 50px 150px 100px round 20px / 20px)`——round
/// 关键字 + border-radius 文法，非 `/` 引入）。
#[derive(Debug, Clone, PartialEq)]
pub struct XywhFunctionValue {
    /// x。
    pub x: LengthValue,
    /// y。
    pub y: LengthValue,
    /// 宽。
    pub w: LengthValue,
    /// 高。
    pub h: LengthValue,
    /// 圆角（可选）。
    pub round: Option<BorderRadiusValue>,
}

/// shape() 值模型：`shape([<fill-rule>] from <position>, <command>#)`。
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeFunctionValue {
    /// fill-rule（None/Nonzero 串化省略）。
    pub fill_rule: Option<FillRule>,
    /// 起点 `from <position>`（1 词形串化展开 2 分量：`from center` →
    /// `from center center` 语料钉面）。
    pub from: ShapePosition,
    /// 命令列表（≥1——`shape(from 0px 10px)` 无命令拒收语料钉面）。
    pub commands: Vec<ShapeCommand>,
}

// ── 入口（parse_extended_border::parse_clip_path 派发） ─────────────────

/// 解析 shape() 函数体（不含函数名与前导 `(`）。
pub fn parse_shape_function(rest: &str) -> Option<ClipPathValue> {
    let inner = rest.strip_suffix(')')?.trim();
    let segments = split_top_level_commas(inner)?;
    // from 段 + 至少一条命令（无命令形拒收）
    if segments.len() < 2 {
        return None;
    }
    // 首段：[<fill-rule>] from <position>（fill-rule 与 from 之间无逗号
    // ——`shape(nonzero, from …)` 拒收语料钉面）
    let from_tok = split_top_ws_depth(segments[0].trim());
    if from_tok.is_empty() {
        return None;
    }
    if !from_tok[0].eq_ignore_ascii_case("from") {
        let fill = resolve_fill_rule(from_tok[0])?;
        if from_tok.len() < 2 || !from_tok[1].eq_ignore_ascii_case("from") {
            return None;
        }
        let from = parse_shape_position(&from_tok[2..])?;
        let commands = parse_shape_commands(&segments[1..])?;
        return Some(ClipPathValue::Shape(Box::new(ShapeFunctionValue {
            fill_rule: Some(fill),
            from,
            commands,
        })));
    }
    let from = parse_shape_position(&from_tok[1..])?;
    let commands = parse_shape_commands(&segments[1..])?;
    Some(ClipPathValue::Shape(Box::new(ShapeFunctionValue {
        fill_rule: None,
        from,
        commands,
    })))
}

/// fill-rule 关键字（大小写不敏感；非关键字 None——`EvenOdd` → evenodd
/// 保形归一语料钉面）。
fn resolve_fill_rule(tok: &str) -> Option<FillRule> {
    match tok.to_ascii_lowercase().as_str() {
        "nonzero" => Some(FillRule::Nonzero),
        "evenodd" => Some(FillRule::Evenodd),
        _ => None,
    }
}

/// 命令段列表逐段解析（段内不含顶层逗号）。
fn parse_shape_commands(segments: &[&str]) -> Option<Vec<ShapeCommand>> {
    let mut commands = Vec::with_capacity(segments.len());
    for seg in segments {
        commands.push(parse_shape_command(seg)?);
    }
    Some(commands)
}

/// 解析 path() 函数体：`path([<fill-rule>,] <string>)`。
pub fn parse_path_function(rest: &str) -> Option<ClipPathValue> {
    let inner = rest.strip_suffix(')')?.trim();
    // path data 以引号承载；引号前头部只能为空或 `<fill-rule>,`
    let quote_idx = inner.find(['\'', '"'])?;
    let head = inner[..quote_idx].trim();
    let payload = &inner[quote_idx..];
    let fill_rule = if head.is_empty() {
        None
    } else {
        let word = head.strip_suffix(',')?.trim();
        Some(resolve_fill_rule(word)?)
    };
    let data = payload
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .or_else(|| payload.strip_prefix('"').and_then(|s| s.strip_suffix('"')))?;
    let segs = parse_svg_path_data(data)?;
    Some(ClipPathValue::Path(Box::new(PathFunctionValue {
        fill_rule,
        segs,
        raw: data.trim().to_string(),
    })))
}

/// 解析 rect() 函数体（空白分隔 4 值 + `round` 圆角）。
pub fn parse_rect_function(rest: &str) -> Option<ClipPathValue> {
    let inner = rest.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return None;
    }
    let (main, round) = split_round_section(inner);
    let tok = split_tokens_with_slash(main);
    if tok.len() != 4 {
        return None;
    }
    let top = parse_rect_coord(tok[0])?;
    let right = parse_rect_coord(tok[1])?;
    let bottom = parse_rect_coord(tok[2])?;
    let left = parse_rect_coord(tok[3])?;
    // `round` 关键字在场时圆角部必须非空可解析（`round` 裸关键字拒收）
    let round_val = match round {
        Some(section) => Some(parse_border_radius(section)?),
        None => None,
    };
    Some(ClipPathValue::Rect(Box::new(RectFunctionValue {
        top,
        right,
        bottom,
        left,
        round: round_val,
    })))
}

/// 解析 xywh() 函数体（空白分隔 4 值 + `round` 圆角）。
pub fn parse_xywh_function(rest: &str) -> Option<ClipPathValue> {
    let inner = rest.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return None;
    }
    let (main, round) = split_round_section(inner);
    let tok = split_top_ws_depth(main);
    if tok.len() != 4 {
        return None;
    }
    let x = parse_shape_len(tok[0])?;
    let y = parse_shape_len(tok[1])?;
    let w = parse_shape_len(tok[2])?;
    let h = parse_shape_len(tok[3])?;
    let round_val = match round {
        Some(section) => Some(parse_border_radius(section)?),
        None => None,
    };
    Some(ClipPathValue::Xywh(Box::new(XywhFunctionValue {
        x,
        y,
        w,
        h,
        round: round_val,
    })))
}

/// `round` 槽切分（主部 + 圆角部）。
fn split_round_section(inner: &str) -> (&str, Option<&str>) {
    match find_keyword_pos(inner, "round") {
        Some(idx) => (inner[..idx].trim(), Some(inner[idx + 5..].trim())),
        None => (inner, None),
    }
}

// ── shape() 命令解析 ────────────────────────────────────────────────────

/// 单条命令段解析（`move to 20px 30px` 等；段内不含顶层逗号）。
fn parse_shape_command(seg: &str) -> Option<ShapeCommand> {
    let tok = split_tokens_with_slash(seg.trim());
    if tok.is_empty() {
        return None;
    }
    let head = tok[0].to_ascii_lowercase();
    // close 无参数（`close path` 拒收语料钉面）
    if head == "close" {
        if tok.len() != 1 {
            return None;
        }
        return Some(ShapeCommand::Close);
    }
    if tok.len() < 3 {
        return None;
    }
    let to_by = tok[1].to_ascii_lowercase();
    if !matches!(to_by.as_str(), "to" | "by") {
        return None;
    }
    let rest = &tok[2..];
    match head.as_str() {
        "move" => Some(ShapeCommand::Move {
            end: parse_endpoint(rest, &to_by)?,
        }),
        "line" => Some(ShapeCommand::Line {
            end: parse_endpoint(rest, &to_by)?,
        }),
        "hline" => {
            if rest.len() != 1 {
                return None;
            }
            let by = to_by == "by";
            let x = if by {
                ShapeAxisComp::Len(parse_shape_len(rest[0])?)
            } else {
                parse_axis_single(rest[0], Axis::Horizontal)?
            };
            Some(ShapeCommand::Hline { by, x })
        }
        "vline" => {
            if rest.len() != 1 {
                return None;
            }
            let by = to_by == "by";
            let y = if by {
                ShapeAxisComp::Len(parse_shape_len(rest[0])?)
            } else {
                parse_axis_single(rest[0], Axis::Vertical)?
            };
            Some(ShapeCommand::Vline { by, y })
        }
        "curve" => {
            // <endpoint> with <cp> [/ <cp>]；with 位于端点部之后
            //（position 1/2/4 词或 by 对 2 词）
            let with_idx = rest
                .iter()
                .position(|t| t.eq_ignore_ascii_case("with"))
                .filter(|&i| matches!(i, 1 | 2 | 4))?;
            let end = parse_endpoint(&rest[..with_idx], &to_by)?;
            let after = &rest[with_idx + 1..];
            let slash_idx = after.iter().position(|t| *t == "/");
            let (cps, c2s) = match slash_idx {
                Some(i) => (&after[..i], Some(&after[i + 1..])),
                None => (after, None),
            };
            let c1 = parse_control_point(cps)?;
            // `/` 后第二控制点必须非空（`with 10px 30px /` 拒收语料钉面）
            let c2 = match c2s {
                Some(s) => Some(parse_control_point(s)?),
                None => None,
            };
            Some(ShapeCommand::Curve { end, c1, c2 })
        }
        "smooth" => {
            // <endpoint> [with <cp>]
            match rest.iter().position(|t| t.eq_ignore_ascii_case("with")) {
                Some(i) if matches!(i, 1 | 2 | 4) => {
                    let end = parse_endpoint(&rest[..i], &to_by)?;
                    let c1 = parse_control_point(&rest[i + 1..])?;
                    Some(ShapeCommand::Smooth { end, c1: Some(c1) })
                }
                Some(_) => None,
                None => {
                    let end = parse_endpoint(rest, &to_by)?;
                    Some(ShapeCommand::Smooth { end, c1: None })
                }
            }
        }
        "arc" => {
            // <endpoint> of <radii> [modifiers ||]
            let of_idx = rest
                .iter()
                .position(|t| t.eq_ignore_ascii_case("of"))
                .filter(|&i| matches!(i, 1 | 2 | 4))?;
            let end = parse_endpoint(&rest[..of_idx], &to_by)?;
            parse_arc_modifiers(end, &rest[of_idx + 1..])
        }
        _ => None,
    }
}

/// 端点解析：`to` = <position>（1/2/4 词，关键字保形）；
/// `by` = <length-percentage>{2}（严格双 len-pct）。
fn parse_endpoint(tok: &[&str], to_by: &str) -> Option<ShapeEndpoint> {
    if to_by == "by" {
        if tok.len() != 2 {
            return None;
        }
        Some(ShapeEndpoint::Offset(
            parse_shape_len(tok[0])?,
            parse_shape_len(tok[1])?,
        ))
    } else {
        Some(ShapeEndpoint::Position(parse_shape_position(tok)?))
    }
}

/// arc 半径与修饰词段解析：1-2 个 len-pct 后接 `||` 修饰词组
///（cw|ccw、large|small、rotate <angle>；任意序、各至多一次——
/// `cw ccw` / `small large` / `rotate … rotate …` 重复拒收语料钉面）。
fn parse_arc_modifiers(end: ShapeEndpoint, after: &[&str]) -> Option<ShapeCommand> {
    if after.is_empty() {
        return None;
    }
    // 半径段：至多两个 len-pct（calc 计 1 token；修饰词停机）
    let rx = parse_shape_len(after[0])?;
    let mut i = 1usize;
    let mut ry = None;
    if i < after.len() && !is_arc_modifier(after[i]) {
        ry = Some(parse_shape_len(after[i])?);
        i += 1;
    }
    // 修饰词组（|| 组合：任意序、各至多一次）
    let mut cw: Option<bool> = None;
    let mut large: Option<bool> = None;
    let mut rotate: Option<(f64, ShapeAngleUnit)> = None;
    while i < after.len() {
        let t = after[i].to_ascii_lowercase();
        match t.as_str() {
            "cw" => {
                if cw.is_some() {
                    return None;
                }
                cw = Some(true);
                i += 1;
            }
            "ccw" => {
                if cw.is_some() {
                    return None;
                }
                cw = Some(false);
                i += 1;
            }
            "large" => {
                if large.is_some() {
                    return None;
                }
                large = Some(true);
                i += 1;
            }
            "small" => {
                if large.is_some() {
                    return None;
                }
                large = Some(false);
                i += 1;
            }
            "rotate" => {
                if rotate.is_some() || i + 1 >= after.len() {
                    return None;
                }
                rotate = Some(parse_shape_angle(after[i + 1])?);
                i += 2;
            }
            _ => return None,
        }
    }
    Some(ShapeCommand::Arc {
        end,
        rx,
        ry,
        cw: cw.unwrap_or(false),
        large: large.unwrap_or(false),
        // 0 角串化省略（`rotate 0deg` → 省略语料钉面）
        rotate: rotate.filter(|(rad, _)| *rad != 0.0),
    })
}

/// arc 修饰词判定（半径段消费停机条件）。
fn is_arc_modifier(tok: &str) -> bool {
    matches!(
        tok.to_ascii_lowercase().as_str(),
        "cw" | "ccw" | "large" | "small" | "rotate"
    )
}

/// 角度解析：`<number><unit>`（deg/rad/grad/turn，大小写不敏感）→
///（弧度，单位保形）。
fn parse_shape_angle(tok: &str) -> Option<(f64, ShapeAngleUnit)> {
    let idx = tok.rfind(|c: char| c.is_ascii_alphabetic())?;
    let unit_start = tok[..=idx]
        .rfind(|c: char| !c.is_ascii_alphabetic())
        .map_or(0, |j| j + 1);
    let value: f64 = tok[..unit_start].parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    let unit_word = tok[unit_start..].to_ascii_lowercase();
    let radians = match unit_word.as_str() {
        "deg" => value * std::f64::consts::PI / 180.0,
        "rad" => value,
        "grad" => value * std::f64::consts::PI / 200.0,
        "turn" => value * 2.0 * std::f64::consts::PI,
        _ => return None,
    };
    let unit = match unit_word.as_str() {
        "deg" => ShapeAngleUnit::Deg,
        "rad" => ShapeAngleUnit::Rad,
        "grad" => ShapeAngleUnit::Grad,
        _ => ShapeAngleUnit::Turn,
    };
    Some((radians, unit))
}

/// 控制点：`<position> [from start|end|origin]`（position 部 1/2/4 词）。
/// `from` 后缀仅控制点可带——端点携带形（`curve by 20px 30px from
/// start`）拒收语料钉面（端点解析不经此路径）。
fn parse_control_point(tok: &[&str]) -> Option<ShapeControlPoint> {
    if tok.is_empty() {
        return None;
    }
    // `from <ref>` 后缀（from 位于 position 部之后：1/2/4）
    let (pos_tok, from_ref) = match tok.iter().position(|t| t.eq_ignore_ascii_case("from")) {
        Some(i) if matches!(i, 1 | 2 | 4) && tok.len() == i + 2 => {
            let r = match tok[i + 1].to_ascii_lowercase().as_str() {
                "start" => ShapeCoordRef::Start,
                "end" => ShapeCoordRef::End,
                "origin" => ShapeCoordRef::Origin,
                _ => return None,
            };
            (&tok[..i], Some(r))
        }
        Some(_) => return None,
        None => (tok, None),
    };
    let pos = parse_shape_position(pos_tok)?;
    Some(ShapeControlPoint { pos, from_ref })
}

/// 轴限定（hline/vline 关键字正交校验）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Horizontal,
    Vertical,
}

/// 单坐标（hline/vline to）：轴限定关键字（保形）或 len-pct。
/// 轴外关键字拒收（`hline to top` / `vline to x-start` 语料钉面）；
/// 双词拒收（`hline to top 20px` / `vline to bottom left` 语料钉面——
/// 调用方限制单 token）。
fn parse_axis_single(tok: &str, axis: Axis) -> Option<ShapeAxisComp> {
    let k = tok.to_ascii_lowercase();
    let kw = match (axis, k.as_str()) {
        (Axis::Horizontal, "left") => ShapeAxisComp::Left,
        (Axis::Horizontal, "right") => ShapeAxisComp::Right,
        (Axis::Horizontal, "center") => ShapeAxisComp::Center,
        (Axis::Horizontal, "x-start") => ShapeAxisComp::XStart,
        (Axis::Horizontal, "x-end") => ShapeAxisComp::XEnd,
        (Axis::Vertical, "top") => ShapeAxisComp::Top,
        (Axis::Vertical, "bottom") => ShapeAxisComp::Bottom,
        (Axis::Vertical, "center") => ShapeAxisComp::Center,
        (Axis::Vertical, "y-start") => ShapeAxisComp::YStart,
        (Axis::Vertical, "y-end") => ShapeAxisComp::YEnd,
        // 非本轴关键字：交由 len 解析（必败——关键字非长度）
        _ => ShapeAxisComp::Len(parse_shape_len(tok)?),
    };
    Some(kw)
}

/// `<position>` token 序列（1/2/4 词）→ 坐标对（关键字保形）：
/// 1 词 = 关键字展 (kw, center)（`from center` → `center center`）或
/// len 双轴同值（`from 0Px` → `0px 0px`）；2 词 = 轴归类（轴关键字落
/// 本轴、center/len 填空位；物理缘关键字 + len = edge-offset 形缺轴补
/// center）；4 词 = edge-offset 双对；3 词拒收（css-values-4 §position）。
fn parse_shape_position(tok: &[&str]) -> Option<ShapePosition> {
    match tok.len() {
        // 1 词：关键字展 (kw, center)；非关键字 token 落 len 双轴同值
        //（`from 0Px` → `0px 0px` 语料钉面）
        1 => match classify_free_keyword(tok[0]) {
            Some(KwClass::H(c)) => Some(ShapePosition {
                x: c,
                y: ShapeAxisComp::Center,
            }),
            Some(KwClass::V(c)) => Some(ShapePosition {
                x: ShapeAxisComp::Center,
                y: c,
            }),
            Some(KwClass::Free(c)) => Some(ShapePosition { x: c.clone(), y: c }),
            _ => {
                let v = parse_shape_len(tok[0])?;
                Some(ShapePosition {
                    x: ShapeAxisComp::Len(v.clone()),
                    y: ShapeAxisComp::Len(v),
                })
            }
        },
        2 => build_position_2(classify_position_token(tok[0])?, classify_position_token(tok[1])?),
        3 => None,
        4 => {
            // <edge> <offset> <edge> <offset>：首词定轴序（同轴双对拒）
            let edge0 = classify_edge(tok[0])?;
            let edge2 = classify_edge(tok[2])?;
            if edge_axis(edge0) == edge_axis(edge2) {
                return None;
            }
            let pair = |edge: ShapeEdge, offset: &str| {
                Some(ShapeAxisComp::Edge(
                    edge,
                    Box::new(ShapeAxisComp::Len(parse_shape_len(offset)?)),
                ))
            };
            let (x, y) = if edge_axis(edge0) == Axis::Horizontal {
                (pair(edge0, tok[1])?, pair(edge2, tok[3])?)
            } else {
                (pair(edge2, tok[3])?, pair(edge0, tok[1])?)
            };
            Some(ShapePosition { x, y })
        }
        _ => None,
    }
}

/// 2 词位置合成（轴归类 + edge-offset 形）。
fn build_position_2(c0: KwClass, c1: KwClass) -> Option<ShapePosition> {
    use ShapeAxisComp as C;
    match (c0, c1) {
        (KwClass::H(h), KwClass::V(v)) | (KwClass::V(v), KwClass::H(h)) => Some(ShapePosition { x: h, y: v }),
        (KwClass::H(h), KwClass::Free(f)) | (KwClass::Free(f), KwClass::H(h)) => Some(ShapePosition { x: h, y: f }),
        (KwClass::V(v), KwClass::Free(f)) | (KwClass::Free(f), KwClass::V(v)) => Some(ShapePosition { x: f, y: v }),
        (KwClass::Free(f0), KwClass::Free(f1)) => Some(ShapePosition { x: f0, y: f1 }),
        (KwClass::Len(a), KwClass::Len(b)) => Some(ShapePosition {
            x: C::Len(a),
            y: C::Len(b),
        }),
        (KwClass::Free(f), KwClass::Len(l)) => Some(ShapePosition { x: f, y: C::Len(l) }),
        (KwClass::Len(l), KwClass::Free(f)) => Some(ShapePosition { x: C::Len(l), y: f }),
        // edge-offset 2 词形（css-values-4 §position）：物理缘关键字 + len
        // → 本轴 Edge、缺轴 center；逻辑关键字（x-start 等）不带 offset
        (KwClass::H(C::Left), KwClass::Len(l)) => Some(ShapePosition {
            x: C::Edge(ShapeEdge::Left, Box::new(C::Len(l))),
            y: C::Center,
        }),
        (KwClass::H(C::Right), KwClass::Len(l)) => Some(ShapePosition {
            x: C::Edge(ShapeEdge::Right, Box::new(C::Len(l))),
            y: C::Center,
        }),
        (KwClass::V(C::Top), KwClass::Len(l)) => Some(ShapePosition {
            x: C::Center,
            y: C::Edge(ShapeEdge::Top, Box::new(C::Len(l))),
        }),
        (KwClass::V(C::Bottom), KwClass::Len(l)) => Some(ShapePosition {
            x: C::Center,
            y: C::Edge(ShapeEdge::Bottom, Box::new(C::Len(l))),
        }),
        // len 后随轴关键字（`10px left`）/逻辑缘 + len 等残形拒收
        _ => None,
    }
}

/// 位置 token 关键字归类（轴专属 / 双轴 / 长度）。
enum KwClass {
    /// 水平轴关键字。
    H(ShapeAxisComp),
    /// 垂直轴关键字。
    V(ShapeAxisComp),
    /// 双轴通用关键字（center）。
    Free(ShapeAxisComp),
    /// 长度/百分比/calc。
    Len(LengthValue),
}

/// 单 token 归类（2 词形的轴归属依据）。
fn classify_position_token(tok: &str) -> Option<KwClass> {
    let k = tok.to_ascii_lowercase();
    match k.as_str() {
        "left" => Some(KwClass::H(ShapeAxisComp::Left)),
        "right" => Some(KwClass::H(ShapeAxisComp::Right)),
        "x-start" => Some(KwClass::H(ShapeAxisComp::XStart)),
        "x-end" => Some(KwClass::H(ShapeAxisComp::XEnd)),
        "top" => Some(KwClass::V(ShapeAxisComp::Top)),
        "bottom" => Some(KwClass::V(ShapeAxisComp::Bottom)),
        "y-start" => Some(KwClass::V(ShapeAxisComp::YStart)),
        "y-end" => Some(KwClass::V(ShapeAxisComp::YEnd)),
        "center" => Some(KwClass::Free(ShapeAxisComp::Center)),
        _ => parse_shape_len(tok).map(KwClass::Len),
    }
}

/// 1 词形关键字归类（轴关键字展本轴 + center）。
fn classify_free_keyword(tok: &str) -> Option<KwClass> {
    let k = tok.to_ascii_lowercase();
    match k.as_str() {
        "left" => Some(KwClass::H(ShapeAxisComp::Left)),
        "right" => Some(KwClass::H(ShapeAxisComp::Right)),
        "x-start" => Some(KwClass::H(ShapeAxisComp::XStart)),
        "x-end" => Some(KwClass::H(ShapeAxisComp::XEnd)),
        "top" => Some(KwClass::V(ShapeAxisComp::Top)),
        "bottom" => Some(KwClass::V(ShapeAxisComp::Bottom)),
        "y-start" => Some(KwClass::V(ShapeAxisComp::YStart)),
        "y-end" => Some(KwClass::V(ShapeAxisComp::YEnd)),
        "center" => Some(KwClass::Free(ShapeAxisComp::Center)),
        _ => None,
    }
}

/// edge 关键字归类（物理四缘；center 作 edge 拒收——语料钉面）。
fn classify_edge(tok: &str) -> Option<ShapeEdge> {
    match tok.to_ascii_lowercase().as_str() {
        "left" => Some(ShapeEdge::Left),
        "right" => Some(ShapeEdge::Right),
        "top" => Some(ShapeEdge::Top),
        "bottom" => Some(ShapeEdge::Bottom),
        _ => None,
    }
}

/// edge 的轴归属。
fn edge_axis(edge: ShapeEdge) -> Axis {
    match edge {
        ShapeEdge::Left | ShapeEdge::Right => Axis::Horizontal,
        ShapeEdge::Top | ShapeEdge::Bottom => Axis::Vertical,
    }
}

/// len-pct 坐标（负值合法；裸数字 0 → px；calc → LengthValue::Calc）。
fn parse_shape_len(tok: &str) -> Option<LengthValue> {
    let v = parse_length(tok)
        .or_else(|| super::types::parse_math_function(tok).map(|expr| LengthValue::Calc(Box::new(expr))))?;
    // 形状坐标只收长度族：排除 parse_length 的非长度关键字产物
    if is_nonnegative_keyword(&v) {
        return None;
    }
    Some(v)
}

/// parse_length 可能产出的非长度关键字（min-content 等）对形状坐标非法。
fn is_nonnegative_keyword(v: &LengthValue) -> bool {
    matches!(
        v,
        LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
    )
}

/// LengthValue → calc 叶（长度/百分比直载；Calc 展开）。
fn calc_leaf(lv: &LengthValue) -> CalcExpr {
    match lv {
        LengthValue::Calc(expr) => (**expr).clone(),
        other => CalcExpr::Length(other.clone()),
    }
}

/// rect() 坐标：`auto | <length-percentage>`。
fn parse_rect_coord(tok: &str) -> Option<RectCoord> {
    if tok.trim().eq_ignore_ascii_case("auto") {
        return Some(RectCoord::Auto);
    }
    parse_shape_len(tok).map(RectCoord::Length)
}

/// border-radius 文法：1-4 len [/ 1-4 len]（负半径拒收——`round -20px`
/// 语料钉面；`round / 1px` 空水平组拒收；5 值拒收）。
fn parse_border_radius(section: &str) -> Option<BorderRadiusValue> {
    let tok = split_tokens_with_slash(section.trim());
    let slash_idx = tok.iter().position(|t| *t == "/");
    let (h_tok, v_tok) = match slash_idx {
        Some(i) => (&tok[..i], Some(&tok[i + 1..])),
        None => (&tok[..], None),
    };
    let h = expand_radius(parse_radius_values(h_tok)?);
    let v = match v_tok {
        Some(vt) => Some(expand_radius(parse_radius_values(vt)?)),
        None => None,
    };
    Some(BorderRadiusValue { h, v })
}

/// 1-4 半径值解析（len-pct；负值拒）。
fn parse_radius_values(tok: &[&str]) -> Option<Vec<LengthValue>> {
    if tok.is_empty() || tok.len() > 4 {
        return None;
    }
    let mut out = Vec::with_capacity(tok.len());
    for t in tok {
        let v = parse_shape_len(t)?;
        if is_definitely_negative(&v) {
            return None;
        }
        out.push(v);
    }
    Some(out)
}

/// 1-4 值镜像展开为 4 元组（css-backgrounds-3 §border-radius）：
/// 1→全同、2→[a b a b]、3→[a b c b]、4→原样。
fn expand_radius(vals: Vec<LengthValue>) -> [LengthValue; 4] {
    match vals.len() {
        1 => {
            let v = vals.into_iter().next().unwrap_or_else(|| unreachable!());
            [v.clone(), v.clone(), v.clone(), v]
        }
        2 => {
            let mut it = vals.into_iter();
            let a = it.next().unwrap_or_else(|| unreachable!());
            let b = it.next().unwrap_or_else(|| unreachable!());
            [a.clone(), b.clone(), a, b]
        }
        3 => {
            let mut it = vals.into_iter();
            let a = it.next().unwrap_or_else(|| unreachable!());
            let b = it.next().unwrap_or_else(|| unreachable!());
            let c = it.next().unwrap_or_else(|| unreachable!());
            [a, b.clone(), c, b]
        }
        _ => vals.try_into().unwrap_or_else(|_| unreachable!()),
    }
}

/// 长度显式负值判定（数值符号；calc 形不可静态判定放行）。
fn is_definitely_negative(v: &LengthValue) -> bool {
    match v {
        LengthValue::Px(n)
        | LengthValue::Percentage(n)
        | LengthValue::Em(n)
        | LengthValue::Ex(n)
        | LengthValue::Rem(n)
        | LengthValue::Ch(n)
        | LengthValue::Vh(n)
        | LengthValue::Vw(n)
        | LengthValue::Vmin(n)
        | LengthValue::Vmax(n) => *n < 0.0,
        _ => false,
    }
}

// ── SVG path data（path()） ─────────────────────────────────────────────

/// SVG path data 扫描：命令字母 + 数字实参表（段实参数按命令文法整除
/// 校验；逗号仅允许夹在同命令两数字间——`M0 0, L1 1` 拒收语料钉面）。
fn parse_svg_path_data(data: &str) -> Option<Vec<PathSeg>> {
    let mut segs: Vec<PathSeg> = Vec::new();
    let mut cur: Option<(char, Vec<f64>)> = None;
    let it = data.trim().as_bytes();
    let mut i = 0usize;
    let mut last_was_number = false;
    while i < it.len() {
        let b = it[i];
        if b.is_ascii_alphabetic() {
            let c = b as char;
            arity_in(c)?;
            // 段收口：实参数须整除该命令文法（Z 段空实参合法）
            if let Some((pc, args)) = cur.take() {
                push_segment(&mut segs, pc, args)?;
            }
            cur = Some((c, Vec::new()));
            last_was_number = false;
            i += 1;
        } else if b == b',' {
            // 逗号仅合法于同命令两数字之间
            if !last_was_number {
                return None;
            }
            let mut j = i + 1;
            while j < it.len() && matches!(it[j], b' ' | b'\t' | b'\r' | b'\n') {
                j += 1;
            }
            if j >= it.len() || !matches!(it[j], b'0'..=b'9' | b'-' | b'+' | b'.') {
                return None;
            }
            last_was_number = false;
            i += 1;
        } else if matches!(b, b' ' | b'\t' | b'\r' | b'\n') {
            i += 1;
        } else {
            // 数字（含符号/小数/指数）
            let start = i;
            if matches!(b, b'-' | b'+' | b'.') {
                i += 1;
            }
            let mut seen_digit = false;
            let mut seen_exp = false;
            while i < it.len() {
                let c = it[i];
                if c.is_ascii_digit() {
                    seen_digit = true;
                    i += 1;
                } else if c == b'.' && !seen_exp {
                    i += 1;
                } else if (c == b'e' || c == b'E') && seen_digit && !seen_exp {
                    seen_exp = true;
                    i += 1;
                    if i < it.len() && matches!(it[i], b'-' | b'+') {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            if !seen_digit {
                return None;
            }
            let n: f64 = std::str::from_utf8(&it[start..i]).ok()?.parse().ok()?;
            cur.as_mut()?.1.push(n); // 数字先于命令字母拒收
            last_was_number = true;
        }
    }
    // 末段收口
    if let Some((pc, args)) = cur.take() {
        push_segment(&mut segs, pc, args)?;
    }
    if segs.is_empty() {
        return None;
    }
    Some(segs)
}

/// 段收口：实参数按命令文法整除校验（Z 段空实参合法）。
fn push_segment(segs: &mut Vec<PathSeg>, cmd: char, args: Vec<f64>) -> Option<()> {
    let n = arity_in(cmd)?;
    if n == 0 {
        if !args.is_empty() {
            return None;
        }
    } else if args.is_empty() || !args.len().is_multiple_of(n) {
        return None;
    }
    segs.push(PathSeg { cmd, args });
    Some(())
}

/// 命令字母实参数（[`parse_svg_path_data`] 内嵌 arity 的提升复用形）。
fn arity_in(c: char) -> Option<usize> {
    Some(match c {
        'M' | 'm' | 'L' | 'l' | 'T' | 't' => 2,
        'H' | 'h' | 'V' | 'v' => 1,
        'S' | 's' | 'Q' | 'q' => 4,
        'C' | 'c' => 6,
        'A' | 'a' => 7,
        'Z' | 'z' => 0,
        _ => return None,
    })
}

// ── 文法辅助 ────────────────────────────────────────────────────────────

/// 括号感知顶层逗号切分（calc()/min() 内逗号不下沉）。
fn split_top_level_commas(s: &str) -> Option<Vec<&str>> {
    let v = split_top_level_commas_always(s);
    if v.iter().any(|p| p.trim().is_empty()) {
        // 悬挂逗号 / 空段拒收
        return None;
    }
    Some(v)
}

/// 逗号切分（不校验空段）。
fn split_top_level_commas_always(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// 括号感知空白切分（calc(-4.5% - 6.5px) 内空白不下沉）。
pub(crate) fn split_top_ws_depth(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start: Option<usize> = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => {
                depth += 1;
                start.get_or_insert(i);
            }
            b')' => {
                depth = depth.saturating_sub(1);
            }
            b' ' | b'\t' | b'\r' | b'\n' if depth == 0 => {
                if let Some(st) = start.take() {
                    out.push(&s[st..i]);
                }
            }
            _ => {
                start.get_or_insert(i);
            }
        }
    }
    if let Some(st) = start {
        out.push(&s[st..]);
    }
    out
}

/// token 化 + 无括号 token 内 `/` 拆分（`from start/12px` 紧贴斜杠形
/// 语料合法钉面；calc() 等括号 token 已成单 token 不受影响）。
fn split_tokens_with_slash(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    for tok in split_top_ws_depth(s) {
        if tok.contains('/') && !tok.contains('(') {
            let mut rest = tok;
            while let Some(idx) = rest.find('/') {
                let head = &rest[..idx];
                out.extend(split_top_ws_depth(head));
                out.push("/");
                rest = &rest[idx + 1..];
            }
            out.extend(split_top_ws_depth(rest));
        } else {
            out.push(tok);
        }
    }
    out
}

/// 关键字位置扫描（词边界对齐；`round` 于 rect/xywh 参数区）。
fn find_keyword_pos(s: &str, keyword: &str) -> Option<usize> {
    let pat = format!(" {keyword}");
    if let Some(i) = s.to_ascii_lowercase().find(&pat) {
        return Some(i + 1);
    }
    if s.to_ascii_lowercase().starts_with(keyword) {
        return Some(0);
    }
    None
}

// ── computed 折算辅助（Stage B engine 复用） ─────────────────────────────

/// 位置分量 computed 折算：关键字→百分比（left/x-start/top/y-start→0%、
/// center→50%、right/x-end/bottom/y-end→100%）、edge-offset right/bottom
/// → `calc(100% − o)`（与 style-system parse_origin_xy 14 轮扩展同型）、
/// 长度保形。
pub fn shape_axis_comp_resolved(c: &ShapeAxisComp) -> LengthValue {
    match c {
        ShapeAxisComp::Len(v) => v.clone(),
        ShapeAxisComp::Center => LengthValue::Percentage(50.0),
        ShapeAxisComp::Left | ShapeAxisComp::Top | ShapeAxisComp::XStart | ShapeAxisComp::YStart => {
            LengthValue::Percentage(0.0)
        }
        ShapeAxisComp::Right | ShapeAxisComp::Bottom | ShapeAxisComp::XEnd | ShapeAxisComp::YEnd => {
            LengthValue::Percentage(100.0)
        }
        ShapeAxisComp::Edge(edge, off) => {
            let o = shape_axis_comp_resolved(off);
            match edge {
                ShapeEdge::Left | ShapeEdge::Top => o,
                ShapeEdge::Right | ShapeEdge::Bottom => LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
                    Box::new(CalcExpr::Length(LengthValue::Percentage(100.0))),
                    CalcOp::Subtract,
                    Box::new(calc_leaf(&o)),
                ))),
            }
        }
    }
}

// ── canonical 串化（声明级；computed 关键字折算入口 shape_to_css_computed） ──

/// 值含物理绝对长度单位（pt/pc/cm/mm/in/Q）判定（数字后紧随单位字母段、
/// 大小写不敏感）。共享解析器将这些单位 parse 期折 px（types.rs 单位表
/// `"pt"` 臂同族），声明级 canonical 串化无法保原单位形——声明值折叠站
/// 据此跳过 canonical 回落原文通道（CSSOM specified 保形，Chrome 同）。
pub fn has_physical_length_unit(v: &str) -> bool {
    let b = v.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n {
        if b[i].is_ascii_digit() {
            let mut j = i + 1;
            while j < n && (b[j].is_ascii_alphanumeric() || b[j] == b'.') {
                j += 1;
            }
            let mut k = j;
            while k > i && b[k - 1].is_ascii_alphabetic() {
                k -= 1;
            }
            if matches!(
                v[k..j].to_ascii_lowercase().as_str(),
                "pt" | "pc" | "cm" | "mm" | "in" | "q"
            ) {
                return true;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    false
}

// ── shape-outside 声明值 specified canonical 串化（wptfyi-css 刀 Q）──────

/// shape-outside 声明值 specified canonical 串化（css-shapes-1
/// §shape-outside-property 串形 + cssom-1 §serializing CSS values 位置
/// 规则；wptfyi-css 刀 Q：shape-outside-circle/ellipse-002 位置补全
/// 29F×2、-004 位置组 inline 143F×2、inset-003 slash 折叠 52F 靶面）。
///
/// 纯文本级 token 重排/补全，不落 AST——共享解析器 parse 期折 px
///（[`super::types::parse_length`] 物理单位臂），AST 往返无法保原单位形
///（oracle specified 级保形 `circle(at 50cm)` → `circle(at 50cm center)`），
/// 与 clip-path canonical 臂的物理单位回落原文不同（该属性 oracle
/// specified 不保单位，本属性保）。写面 gate（apply 臂 parse_clip_path）
/// 已裁 invalid，本串化只见合法形；无法识别的合法形返 None 保留原文
/// 零漂移（3 token 短 offset 形、长度先行序等无语料驱动不接入）。
///
/// chrome 语料钉面规则（circle-002/004 全形清单）：
/// - 单 token 位置补全：len/pct → `V center`、水平缘 → `V center`、
///   垂直缘 → `center V`、center → `center center`；
/// - 双关键字垂直序（`top left`）→ 水平先行（`left top`）；
/// - 4 token edge-offset 对（`top 50% left 50cm`）→ 水平对先行
///   （`left 50cm top 50%`）；
/// - inset round 组：垂直组按 css-backgrounds-3 §border-radius 展开规则
///   逐角等值于水平组 → 省略 `/ <vertical>`（`round 10cm / 10cm` →
///   `round 10cm`；非等值组保形）。
///
/// 前置 shape-box 关键字（`border-box circle(at 50%)`）保形；替换函数
///（var()/attr()）与 CSS-wide 关键字 None passthrough（写面放行语义同型）。
/// https://drafts.csswg.org/css-shapes/#shape-outside-property
/// https://drafts.csswg.org/cssom-1/#serializing-css-values
pub fn canonicalize_shape_outside_specified(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    let lower = v.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    ) {
        return None;
    }
    if lower.contains("var(") || lower.contains("attr(") {
        return None;
    }
    // 前置 shape-box 关键字剥离（`margin-box circle(at 50%)` 形；keyword
    // 保形回拼，语料 oracle 前缀保形）。
    let mut prefix = "";
    let mut core = lower.as_str();
    for box_kw in ["content-box", "padding-box", "border-box", "margin-box"] {
        if let Some(rest) = core.strip_prefix(box_kw) {
            let trimmed = rest.trim_start();
            if trimmed.is_empty() {
                return None;
            }
            prefix = box_kw;
            core = trimmed;
            break;
        }
    }
    // 单函数形裁形：`name(inner)`（inner 至串尾闭括号——trim 后值尾即函数
    // 闭括号；calc 嵌套括号由 token 化处理）。
    let open = core.find('(')?;
    let name = &core[..open];
    let inner = core.get(open + 1..)?.strip_suffix(')')?;
    let body = match name {
        "circle" | "ellipse" => canonicalize_circle_ellipse_inner(name, inner)?,
        "inset" => canonicalize_inset_inner(inner)?,
        _ => return None,
    };
    Some(if prefix.is_empty() {
        body
    } else {
        format!("{prefix} {body}")
    })
}

/// circle()/ellipse() 位置子句 canonical：radius 段保形 + `at` 位置段
/// token 重排（半径段无语料驱动面不重排；无 `at` 子句 passthrough——
/// `circle(50px)` 位置省略形 oracle 串形未钉，不接入）。
fn canonicalize_circle_ellipse_inner(name: &str, inner: &str) -> Option<String> {
    let toks = split_top_ws_depth(inner);
    let at_idx = toks.iter().position(|t| t.eq_ignore_ascii_case("at"))?;
    let radius = toks[..at_idx].join(" ");
    let position = canonicalize_at_position(&toks[at_idx + 1..])?;
    let head = if radius.is_empty() {
        String::new()
    } else {
        format!("{radius} ")
    };
    Some(format!("{name}({head}at {position})"))
}

/// at 位置段 canonical（见 [`canonicalize_shape_outside_specified`] 规则）：
/// 单 token 补全 / 双关键字垂直序翻转 / 4 token 水平对先行；其余形 None。
fn canonicalize_at_position(toks: &[&str]) -> Option<String> {
    let kw = |t: &str| t.to_ascii_lowercase();
    match toks.len() {
        1 => {
            let t = kw(toks[0]);
            match t.as_str() {
                "left" | "right" => Some(format!("{t} center")),
                "top" | "bottom" => Some(format!("center {t}")),
                "center" => Some("center center".to_string()),
                _ if parse_shape_len(toks[0]).is_some() => Some(format!("{} center", toks[0])),
                _ => None,
            }
        }
        2 => {
            // 仅垂直-水平关键字序翻转（`top left` → `left top`）；长度
            // 混合形（`top 50%`/`50px top`）oracle 未钉 passthrough。
            let a = kw(toks[0]);
            let b = kw(toks[1]);
            let is_h = |s: &str| matches!(s, "left" | "right");
            let is_v = |s: &str| matches!(s, "top" | "bottom");
            // center 混合形（探针余 4F）：center 归属缺失轴、canonical x 先行
            // ——`top center` → `center top`、`center left` → `left center`
            // （css-values-3 <position> 2-keyword 序）。
            let swap = (is_v(&a) && (is_h(&b) || b == "center")) || (a == "center" && is_h(&b));
            if swap { Some(format!("{b} {a}")) } else { None }
        }
        4 => {
            // edge-offset 对序：首对垂直 → 水平对先行。token 文法裁形
            //（缘关键字 + len 交替；len 用共享解析器裁——非长度形弃）。
            let e0 = kw(toks[0]);
            let e2 = kw(toks[2]);
            let is_h = |s: &str| matches!(s, "left" | "right");
            let is_v = |s: &str| matches!(s, "top" | "bottom");
            if is_v(&e0) && is_h(&e2) && parse_shape_len(toks[1]).is_some() && parse_shape_len(toks[3]).is_some() {
                Some(format!("{} {} {} {}", toks[2], toks[3], toks[0], toks[1]))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// inset() round 组 canonical：垂直组展开逐角等值于水平组（css-backgrounds-3
/// §border-radius 1-4 值展开镜像规则）→ 省略 `/ <vertical>`；无 round 组或
/// 非等值形 passthrough。insets 段保形（物理单位 oracle specified 保形）。
/// 等值判定经共享解析器 parse 比对（`10px / 1em` 文本异而解析同值形亦折——
/// 单位折算单点 parse_shape_len）。
fn canonicalize_inset_inner(inner: &str) -> Option<String> {
    let round_idx = find_keyword_pos(inner, "round")?;
    let round = inner[round_idx + "round".len()..].trim();
    if round.is_empty() {
        return None;
    }
    let slash_toks = split_tokens_with_slash(round);
    let slash_pos = slash_toks.iter().position(|t| *t == "/")?;
    let h_toks = &slash_toks[..slash_pos];
    let v_toks = &slash_toks[slash_pos + 1..];
    if h_toks.is_empty() || v_toks.is_empty() || h_toks.len() > 4 || v_toks.len() > 4 {
        return None;
    }
    // 文法裁形（共享 border-radius 解析器单点；负半径已在其内拒）——解析
    // 结果仅作等值判定，回拼保原 token 文本（单位保形）。
    let h_parsed = parse_radius_values(h_toks)?;
    let v_parsed = parse_radius_values(v_toks)?;
    if expand_radius(h_parsed) != expand_radius(v_parsed) {
        return None;
    }
    Some(format!(
        "inset({} round {})",
        inner[..round_idx].trim_end(),
        h_toks.join(" ")
    ))
}

/// clip-path 声明值 specified canonical（wptfyi-css 刀 S：css-masking
/// parsing/clip-path-valid 语料 14F 靶面）。规则（页源逐行钉面）：
/// - `<geometry-box>` 前缀/后缀全剥除（语料 `border-box circle(...)` /
///   `circle(...) border-box` → `circle(...)`；裸 box 关键字恒等）；
/// - 裸 `0` 长度补 px（inset 参数/半径、xywh/rect 参数与 round 半径、
///   polygon 坐标；`auto`/`%`/calc 形保形）；
/// - circle 单 `closest-side` 半径省略（缺省值）→ `circle()`；ellipse
///   半径全 `closest-side` 省略；混合半径（`1px closest-side`）保形；
/// - polygon `nonzero`（缺省 fill-rule）省略、`evenodd` 保形；全零 round
///   半径组省略、非全零组保形（组内 `0` 补 px）；
/// - path() `nonzero` 省略（`evenodd` 保形）+ 路径数据 token 规范：
///   逗号→空白、命令字母间单空格连接、小写 `z` → `Z`（其余字母大小写
///   保形）、数值 token 原文保形；
/// - xywh/rect 全零 round 组省略（`round 0`）；非全零组保形补 px；
/// - at 位置段重排复用 [`canonicalize_at_position`]；
/// - url()/ray()/shape()、identity 形与不可识别形 None passthrough（回落
///   既有 AST 串化通道零漂移）。
///
/// https://drafts.fxtf.org/css-masking-1/#the-clip-path
/// https://drafts.csswg.org/css-shapes-2/#serializing-basic-shape
pub fn canonicalize_clip_path_specified(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    let core = canonicalize_strip_box_keywords(v)?;
    let open = core.find('(')?;
    let name = core[..open].trim().to_ascii_lowercase();
    let close = core.rfind(')')?;
    if close < open {
        return None;
    }
    let inner = core.get(open + 1..close)?;
    let suffix = core.get(close + 1..)?.trim();
    // 后缀仅允许 geometry-box（已剥除语义）——其余尾随 token 非法 passthrough。
    if !suffix.is_empty() && !is_shape_box_keyword(suffix) {
        return None;
    }
    let body = match name.as_str() {
        "circle" => canonicalize_clip_circle_ellipse("circle", inner, true),
        "ellipse" => canonicalize_clip_circle_ellipse("ellipse", inner, true),
        "inset" => canonicalize_clip_inset(inner),
        "polygon" => canonicalize_clip_polygon(inner),
        "path" => canonicalize_clip_path_fn(inner),
        "xywh" | "rect" => canonicalize_clip_xywh_rect(&name, inner),
        _ => None,
    }?;
    if body == v {
        return None;
    }
    Some(body)
}

/// 物理单位 guard 形（含 pt/pc/cm/mm/in/Q——AST 通道 parse 期折 px 丢单位，
/// clip-path 臂对这类值回落原文通道）的 shape() 声明值文本级缺省补刀：
/// 仅做三丢省——arc `small` 修饰词（缺省弧向）、零值 `rotate <angle>` 对
/// （缺省 0 角，同 parse_arc_modifiers `*rad != 0.0` 过滤）、控制点缺省
/// 参考系（by 形 `from start` / to 形 `from origin`，与 AST 串化
/// push_control_point 缺省省略同规；css-shapes-2 §shape-function 修饰词
/// `||` 组缺省省略）。调用方先以 `parse_clip_path` 门裁形（拒收形不进
/// 手术）；未发生任何丢弃返回 None（passthrough 原文，单位/token 大小写
/// 保形）。发生丢弃的命令段按单空白重建（canonical 串化形）；from 段与
/// 未变更段原文保形。
pub fn canonicalize_shape_fn_text_physical(v: &str) -> Option<String> {
    let inner = v.trim().strip_prefix("shape(")?.strip_suffix(')')?;
    let segments = split_top_level_commas_always(inner);
    // from 段 + ≥1 命令段（与 parse_shape_function 入口同构）
    if segments.len() < 2 {
        return None;
    }
    let mut changed = false;
    let mut out = String::new();
    for (i, seg) in segments.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let seg_text = seg.trim();
        let toks = split_top_ws_depth(seg_text);
        if toks.is_empty() {
            return None;
        }
        match canonicalize_shape_command_text(&toks) {
            Some(r) => {
                changed = true;
                out.push_str(&r);
            }
            None => out.push_str(seg_text),
        }
    }
    if !changed {
        return None;
    }
    Some(format!("shape({out})"))
}

/// 单命令段文本级缺省丢弃（token 视角；无丢弃 None）。arc 走 `of` 后
/// 修饰词区，curve/smooth 走 `with` 后控制点 run；move/line/hline/vline/
/// close 无缺省修饰 passthrough。
fn canonicalize_shape_command_text(toks: &[&str]) -> Option<String> {
    if toks.len() < 3 {
        return None;
    }
    let head = toks[0].to_ascii_lowercase();
    if head == "close" {
        return None;
    }
    let is_by = toks[1].eq_ignore_ascii_case("by");
    let mut drops: Vec<(usize, usize)> = Vec::new();
    // arc：`of` 后修饰词区丢 `small` 与零值 `rotate` 对（修饰词区起点 =
    // of 后首个修饰词 token；parse 门保证 of 后为 1-2 个 len-pct 半径段）
    if head == "arc" {
        if let Some(of_idx) = toks.iter().position(|t| t.eq_ignore_ascii_case("of")) {
            if let Some(mz) = toks[of_idx + 1..]
                .iter()
                .position(|t| is_arc_modifier(t))
                .map(|j| of_idx + 1 + j)
            {
                let mut i = mz;
                while i < toks.len() {
                    let t = toks[i].to_ascii_lowercase();
                    if t == "small" {
                        drops.push((i, i + 1));
                        i += 1;
                    } else if t == "rotate"
                        && i + 1 < toks.len()
                        && parse_shape_angle(toks[i + 1]).is_some_and(|(rad, _)| rad == 0.0)
                    {
                        drops.push((i, i + 2));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }
    // curve/smooth：`with` 后控制点 run 尾部缺省参考系丢省（run 边界 =
    // `/` 分隔的第二控制点；`from end` 两形均非缺省保形）
    if let Some(with_idx) = toks.iter().position(|t| t.eq_ignore_ascii_case("with")) {
        let default_ref = if is_by { "start" } else { "origin" };
        let mut i = with_idx + 1;
        while i < toks.len() {
            let end = toks[i..].iter().position(|t| *t == "/").map_or(toks.len(), |j| i + j);
            if end >= i + 2
                && toks[end - 2].eq_ignore_ascii_case("from")
                && toks[end - 1].eq_ignore_ascii_case(default_ref)
            {
                drops.push((end - 2, end));
            }
            i = end + 1;
        }
    }
    if drops.is_empty() {
        return None;
    }
    // 区间升序不相交（arc 无 with、curve/smooth 无 of），按待丢区间拼接
    let mut out = String::new();
    let mut k = 0usize;
    let push = |out: &mut String, t: &str| {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(t);
    };
    for (s, e) in &drops {
        for t in &toks[k..*s] {
            push(&mut out, t);
        }
        k = *e;
    }
    for t in &toks[k..] {
        push(&mut out, t);
    }
    Some(out)
}

/// 裸 box 关键字判定（大小写不敏感）。
fn is_shape_box_keyword(tok: &str) -> bool {
    matches!(
        tok.to_ascii_lowercase().as_str(),
        "content-box" | "padding-box" | "border-box" | "margin-box" | "fill-box" | "stroke-box" | "view-box"
    )
}

/// 剥除前后缀 geometry-box（clip-path 语料 oracle 两序全剥）。裸 box 值或
/// 双 box（前+后缀）None passthrough（前者 identity、后者非法）。
fn canonicalize_strip_box_keywords(v: &str) -> Option<&str> {
    let mut core = v;
    for kw in [
        "content-box",
        "padding-box",
        "border-box",
        "margin-box",
        "fill-box",
        "stroke-box",
        "view-box",
    ] {
        if let Some(rest) = core.strip_prefix(kw) {
            if rest.trim().is_empty() {
                return None; // 裸 box 关键字：identity passthrough
            }
            core = rest.trim_start();
            break;
        }
    }
    // 后缀 box 剥除由调用方经 suffix 检查处理（返回 core 含后缀，函数体
    // 重建时不含）；此处仅裁形前缀单次命中。
    Some(core)
}

/// 裸 `0` 数值 token → `0px`（其余 token 保形引回）。裸非零数值不动
///（非法长度形由 invalid 门管，canonical 层 passthrough）。
fn norm_zero_tok(tok: &str) -> String {
    if tok.parse::<f64>().map(|n| n == 0.0).unwrap_or(false) {
        "0px".to_string()
    } else {
        tok.to_string()
    }
}

/// clip-path circle()/ellipse() 段 canonical：缺省半径省略 + at 位置重排。
fn canonicalize_clip_circle_ellipse(name: &str, inner: &str, _drop_default: bool) -> Option<String> {
    let inner = inner.trim();
    let toks = split_top_ws_depth(inner);
    let at_idx = toks.iter().position(|t| t.eq_ignore_ascii_case("at"));
    let (radius_toks, pos_toks) = match at_idx {
        // 裸 `at`（无位置参数）拒收——`circle(at)` invalid 语料钉面（此前
        // 静默修整为 `circle()`）
        Some(i) if i + 1 < toks.len() => (&toks[..i], Some(&toks[i + 1..])),
        Some(_) => return None,
        None => (&toks[..], None),
    };
    // 半径数裁形：circle 0|1、ellipse 0|2（`ellipse(closest-side)` 单半径
    // 误套 circle 缺省省略修整为 `ellipse()` invalid 语料钉面；超数形同拒）
    let max_radii = if name == "ellipse" { 2 } else { 1 };
    if radius_toks.len() > max_radii || (name == "ellipse" && radius_toks.len() == 1) {
        return None;
    }
    // 缺省半径：全部 token 均为 closest-side（1-2 个）→ 省略。
    let drop = !radius_toks.is_empty()
        && radius_toks.len() <= 2
        && radius_toks.iter().all(|t| t.eq_ignore_ascii_case("closest-side"));
    let radius = if drop { String::new() } else { radius_toks.join(" ") };
    let position = match pos_toks {
        Some(p) if !p.is_empty() => {
            let canonical = canonicalize_at_position(p);
            Some(match canonical {
                Some(c) => c,
                None => p.join(" "),
            })
        }
        _ => None,
    };
    let mut out = String::from(name);
    out.push('(');
    if !radius.is_empty() {
        out.push_str(&radius);
        if position.is_some() {
            out.push(' ');
        }
    }
    if let Some(pos) = position {
        out.push_str("at ");
        out.push_str(&pos);
    }
    out.push(')');
    Some(out)
}

/// clip-path inset() 段 canonical：参数与 round 半径裸 `0` 补 px（含 slash
/// 组；非零组保形）。
fn canonicalize_clip_inset(inner: &str) -> Option<String> {
    let inner = inner.trim();
    let round_idx = find_keyword_pos(inner, "round");
    let (args, radii) = match round_idx {
        Some(i) => (&inner[..i], Some(inner[i + "round".len()..].trim())),
        None => (inner, None),
    };
    let args_toks = split_top_ws_depth(args);
    if args_toks.is_empty() || args_toks.len() > 4 {
        return None;
    }
    let mut out = String::from("inset(");
    out.push_str(&args_toks.iter().map(|t| norm_zero_tok(t)).collect::<Vec<_>>().join(" "));
    if let Some(r) = radii {
        // 裸 `round`（无半径）拒收——`inset(0px round)` invalid 语料钉面
        //（此前静默修整为 `inset(0px)`；AST 臂 parse_clip_round_list 空
        // 串本就 None，此为文本通道单侧缺口）
        if r.is_empty() {
            return None;
        }
        let r_toks = split_tokens_with_slash(r);
        out.push_str(" round ");
        out.push_str(
            &r_toks
                .iter()
                .map(|t| if *t == "/" { "/".to_string() } else { norm_zero_tok(t) })
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    out.push(')');
    Some(out)
}

/// clip-path polygon() 段 canonical：nonzero 省略、全零 round 组省略、
/// 坐标裸 `0` 补 px。段数 ≥1（首段可携 fill-rule/round + 坐标；后续段
/// 逐段 2 token 坐标裁形——非 2 token 形 None passthrough）。
fn canonicalize_clip_polygon(inner: &str) -> Option<String> {
    let segs = split_top_level_commas_always(inner.trim());
    let mut out_segs: Vec<String> = Vec::with_capacity(segs.len());
    for (idx, seg) in segs.iter().enumerate() {
        let toks = split_top_ws_depth(seg.trim());
        if toks.is_empty() {
            return None;
        }
        if idx == 0 {
            let mut fill_kw: Option<&str> = None;
            let mut rest = &toks[..];
            let f = rest[0].to_ascii_lowercase();
            if f == "nonzero" || f == "evenodd" {
                fill_kw = if f == "evenodd" { Some(rest[0]) } else { None };
                rest = &rest[1..];
            }
            if rest.is_empty() {
                // 段仅 fill-rule：evenodd 保留独立段，nonzero 省略（整值仅
                // nonzero 无坐标形非法 passthrough）。
                if let Some(fk) = fill_kw {
                    out_segs.push(fk.to_string());
                } else if segs.len() == 1 {
                    return None;
                }
                continue;
            }
            // round 组：关键字 + 1-4 半径 token（calc 形由 token 化保整段）。
            if rest[0].eq_ignore_ascii_case("round") {
                let radii = &rest[1..];
                if radii.is_empty() || radii.len() > 4 {
                    return None;
                }
                let norm: Vec<String> = radii.iter().map(|t| norm_zero_tok(t)).collect();
                if norm.iter().all(|t| t == "0px") {
                    // 全零 round 组省略；段内仅 round（±fill）按 fill 处置。
                    if let Some(fk) = fill_kw {
                        out_segs.push(fk.to_string());
                    } else if segs.len() == 1 {
                        return None;
                    }
                    continue;
                }
                let mut seg_out = String::new();
                if let Some(fk) = fill_kw {
                    seg_out.push_str(fk);
                    seg_out.push(' ');
                }
                seg_out.push_str("round ");
                seg_out.push_str(&norm.join(" "));
                out_segs.push(seg_out);
                continue;
            }
            // fill + 坐标同段形。
            if rest.len() != 2 {
                return None;
            }
            let mut seg_out = String::new();
            if let Some(fk) = fill_kw {
                seg_out.push_str(fk);
                seg_out.push(' ');
            }
            seg_out.push_str(&norm_zero_tok(rest[0]));
            seg_out.push(' ');
            seg_out.push_str(&norm_zero_tok(rest[1]));
            out_segs.push(seg_out);
            continue;
        }
        // 后续段：2 token 坐标（裸 0 补 px）。
        if toks.len() != 2 {
            return None;
        }
        out_segs.push(format!("{} {}", norm_zero_tok(toks[0]), norm_zero_tok(toks[1])));
    }
    if out_segs.iter().any(|s| s.is_empty()) {
        return None;
    }
    Some(format!("polygon({})", out_segs.join(", ")))
}

/// clip-path path() 段 canonical：nonzero 省略（evenodd 保形）+ 路径数据
/// token 规范（见 [`canonicalize_path_data`]）。
fn canonicalize_clip_path_fn(inner: &str) -> Option<String> {
    // 不能用逗号切分——路径数据串内含逗号；以引号前缀判定单段/双段，
    // 双段在首个逗号处分割（fill 段在 data 前，首逗号即分隔符）。
    let trimmed = inner.trim();
    let first = trimmed.chars().next()?;
    let (fill_seg, data_seg) = if first == '"' || first == '\'' {
        (None, trimmed)
    } else {
        let i = trimmed.find(',')?;
        (Some(trimmed[..i].trim()), trimmed[i + 1..].trim())
    };
    let (fill, data_seg) = match fill_seg {
        None => (None, data_seg),
        Some(f) => {
            let fl = f.to_ascii_lowercase();
            if fl != "nonzero" && fl != "evenodd" {
                return None;
            }
            let fill = if fl == "evenodd" { Some(f) } else { None };
            (fill, data_seg)
        }
    };
    let q = data_seg;
    let quote = q.chars().next()?;
    if (quote != '"' && quote != '\'') || q.len() < 2 || !q.ends_with(quote) {
        return None;
    }
    let data = canonicalize_path_data(&q[1..q.len() - 1])?;
    let mut out = String::from("path(");
    if let Some(f) = fill {
        out.push_str(f);
        out.push_str(", ");
    }
    out.push(quote);
    out.push_str(&data);
    out.push(quote);
    out.push(')');
    Some(out)
}

/// SVG 路径数据 token 规范：命令字母（大小写保形，`z`→`Z`）+ 数值 token
///（原文保形）单空格连接；逗号作分隔符吞除；其余字符（引号/未知）None。
fn canonicalize_path_data(data: &str) -> Option<String> {
    let b = data.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_ascii_alphabetic() {
            out.push((if c == 'z' { 'Z' } else { c }).to_string());
            i += 1;
        } else if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' {
            let start = i;
            if c == '-' || c == '+' {
                i += 1;
            }
            let mut saw_digit = false;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
                saw_digit = true;
            }
            if i < b.len() && b[i] == b'.' {
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                    saw_digit = true;
                }
            }
            if !saw_digit {
                return None;
            }
            // 指数段（e/E + 可选符号 + 数字）并入同一数值 token。
            if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
                let mut j = i + 1;
                if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
                    j += 1;
                }
                let e0 = j;
                while j < b.len() && b[j].is_ascii_digit() {
                    j += 1;
                }
                if j > e0 {
                    i = j;
                }
            }
            out.push(data[start..i].to_string());
        } else if c.is_whitespace() || c == ',' {
            i += 1;
        } else {
            return None;
        }
    }
    if out.is_empty() {
        return None;
    }
    Some(out.join(" "))
}

/// clip-path xywh()/rect() 段 canonical：参数裸 `0` 补 px（rect `auto`
/// 保形）；round 半径全零省略、非全零保形补 px。
fn canonicalize_clip_xywh_rect(name: &str, inner: &str) -> Option<String> {
    let inner = inner.trim();
    let round_idx = find_keyword_pos(inner, "round");
    let (args, radii) = match round_idx {
        Some(i) => (&inner[..i], Some(inner[i + "round".len()..].trim())),
        None => (inner, None),
    };
    let args_toks = split_top_ws_depth(args);
    if args_toks.is_empty() || args_toks.len() > 4 {
        return None;
    }
    let mut out = format!("{name}(");
    out.push_str(&args_toks.iter().map(|t| norm_zero_tok(t)).collect::<Vec<_>>().join(" "));
    if let Some(r) = radii {
        if !r.is_empty() {
            let r_toks = split_top_ws_depth(r);
            if r_toks.is_empty() || r_toks.len() > 4 {
                return None;
            }
            let norm: Vec<String> = r_toks.iter().map(|t| norm_zero_tok(t)).collect();
            let all_zero = norm.iter().all(|t| t == "0px");
            if !all_zero {
                out.push_str(" round ");
                out.push_str(&norm.join(" "));
            }
        }
    }
    out.push(')');
    Some(out)
}

/// ClipPathValue 声明级 canonical 串化分派（含本模块四兄弟）。
pub fn clip_path_value_to_css(v: &ClipPathValue) -> Option<String> {
    match v {
        ClipPathValue::None => Some("none".to_string()),
        ClipPathValue::Shape(s) => Some(shape_to_css(s)),
        ClipPathValue::Path(p) => Some(path_to_css(p)),
        ClipPathValue::Rect(r) => Some(rect_to_css(r)),
        ClipPathValue::Xywh(x) => Some(xywh_to_css(x)),
        _ => None, // inset/circle/ellipse/polygon 串化归 engine 既有通道
    }
}

/// shape() 声明级 canonical 串化：fill-rule（evenodd 保形、nonzero/缺省
/// 省略）、from 位置 2 分量、命令逐段、arc 修饰 canonical 序 cw → large
/// → rotate（缺省省略）、控制点参考系按 to/by 缺省省略（css-shapes-2
/// §serializing-basic-shape；语料钉面 shape-function-computed/valid）。
pub fn shape_to_css(s: &ShapeFunctionValue) -> String {
    shape_to_css_mode(s, false)
}

/// shape() computed 级串化：位置分量经 [`shape_axis_comp_resolved`]
/// 关键字折百分比（`from center`→`50% 50%`、`hline to left`→`0%`、
/// edge right/bottom→`calc(100% − o)`）、rotate 归 deg（Chrome computed
/// 串形；声明级保 rad 语料钉面见 [`shape_to_css`]）。
pub fn shape_to_css_computed(s: &ShapeFunctionValue) -> String {
    shape_to_css_mode(s, true)
}

fn shape_to_css_mode(s: &ShapeFunctionValue, computed: bool) -> String {
    let mut out = String::from("shape(");
    if s.fill_rule == Some(FillRule::Evenodd) {
        out.push_str("evenodd ");
    }
    out.push_str("from");
    push_position(&mut out, &s.from, computed);
    for c in &s.commands {
        out.push_str(", ");
        push_command(&mut out, c, computed);
    }
    out.push(')');
    out
}

fn push_command(out: &mut String, c: &ShapeCommand, computed: bool) {
    match c {
        ShapeCommand::Move { end } => push_endpoint_cmd(out, "move", end, computed),
        ShapeCommand::Line { end } => push_endpoint_cmd(out, "line", end, computed),
        ShapeCommand::Hline { by, x } => {
            out.push_str(if *by { "hline by " } else { "hline to " });
            push_comp(out, x, computed);
        }
        ShapeCommand::Vline { by, y } => {
            out.push_str(if *by { "vline by " } else { "vline to " });
            push_comp(out, y, computed);
        }
        ShapeCommand::Curve { end, c1, c2 } => {
            push_endpoint_cmd(out, "curve", end, computed);
            out.push_str(" with");
            push_control_point(out, c1, end.is_by(), computed);
            if let Some(p) = c2 {
                out.push_str(" /");
                push_control_point(out, p, end.is_by(), computed);
            }
        }
        ShapeCommand::Smooth { end, c1 } => {
            push_endpoint_cmd(out, "smooth", end, computed);
            if let Some(p) = c1 {
                out.push_str(" with");
                push_control_point(out, p, end.is_by(), computed);
            }
        }
        ShapeCommand::Arc {
            end,
            rx,
            ry,
            cw,
            large,
            rotate,
        } => {
            push_endpoint_cmd(out, "arc", end, computed);
            out.push_str(" of ");
            out.push_str(&len_to_css(rx));
            if let Some(r) = ry {
                out.push(' ');
                out.push_str(&len_to_css(r));
            }
            if *cw {
                out.push_str(" cw");
            }
            if *large {
                out.push_str(" large");
            }
            if let Some((radians, unit)) = rotate {
                // 声明级单位保形（`rotate 0.52rad` 声明保 rad 语料钉面）；
                // computed 级归 deg（Chrome computed 串形，rad→180deg 钉面）
                let (value, suffix) = if computed {
                    (radians * 180.0 / std::f64::consts::PI, "deg")
                } else {
                    let value = match unit {
                        ShapeAngleUnit::Deg => radians * 180.0 / std::f64::consts::PI,
                        ShapeAngleUnit::Rad => *radians,
                        ShapeAngleUnit::Grad => radians * 200.0 / std::f64::consts::PI,
                        ShapeAngleUnit::Turn => radians / (2.0 * std::f64::consts::PI),
                    };
                    (value, unit.as_str())
                };
                out.push_str(" rotate ");
                out.push_str(&format_number_6dp(value));
                out.push_str(suffix);
            }
        }
        ShapeCommand::Close => out.push_str("close"),
    }
}

fn push_endpoint_cmd(out: &mut String, name: &str, end: &ShapeEndpoint, computed: bool) {
    push_endpoint_head(out, name, end);
    match end {
        ShapeEndpoint::Position(p) => push_position(out, p, computed),
        ShapeEndpoint::Offset(x, y) => {
            out.push(' ');
            out.push_str(&len_to_css(x));
            out.push(' ');
            out.push_str(&len_to_css(y));
        }
    }
}

fn push_endpoint_head(out: &mut String, name: &str, end: &ShapeEndpoint) {
    out.push_str(name);
    out.push_str(if end.is_by() { " by" } else { " to" });
}

/// 位置串化（2 分量 x-then-y；声明级关键字保形、computed 级折百分比）。
fn push_position(out: &mut String, p: &ShapePosition, computed: bool) {
    out.push(' ');
    push_comp(out, &p.x, computed);
    out.push(' ');
    push_comp(out, &p.y, computed);
}

fn push_comp(out: &mut String, c: &ShapeAxisComp, computed: bool) {
    if computed {
        // 关键字→百分比（含 edge right/bottom calc 折算）由
        // shape_axis_comp_resolved 单点承担（14 轮 offset-position 同型）。
        out.push_str(&len_to_css(&shape_axis_comp_resolved(c)));
        return;
    }
    match c {
        ShapeAxisComp::Len(v) => out.push_str(&len_to_css(v)),
        ShapeAxisComp::Center => out.push_str("center"),
        ShapeAxisComp::Left => out.push_str("left"),
        ShapeAxisComp::Right => out.push_str("right"),
        ShapeAxisComp::XStart => out.push_str("x-start"),
        ShapeAxisComp::XEnd => out.push_str("x-end"),
        ShapeAxisComp::Top => out.push_str("top"),
        ShapeAxisComp::Bottom => out.push_str("bottom"),
        ShapeAxisComp::YStart => out.push_str("y-start"),
        ShapeAxisComp::YEnd => out.push_str("y-end"),
        ShapeAxisComp::Edge(edge, off) => {
            out.push_str(match edge {
                ShapeEdge::Left => "left",
                ShapeEdge::Right => "right",
                ShapeEdge::Top => "top",
                ShapeEdge::Bottom => "bottom",
            });
            out.push(' ');
            push_comp(out, off, computed);
        }
    }
}

/// 控制点串化：位置 + 显式非缺省参考系（to 形缺省 origin、by 形缺省
/// start——缺省省略）。
fn push_control_point(out: &mut String, c: &ShapeControlPoint, by: bool, computed: bool) {
    push_position(out, &c.pos, computed);
    let default_ref = if by {
        ShapeCoordRef::Start
    } else {
        ShapeCoordRef::Origin
    };
    if let Some(r) = c.from_ref {
        if r != default_ref {
            out.push_str(" from ");
            out.push_str(match r {
                ShapeCoordRef::Start => "start",
                ShapeCoordRef::End => "end",
                ShapeCoordRef::Origin => "origin",
            });
        }
    }
}

/// path() 声明级串化：双引号承载、data 逐字回显（trivial serialization）、
/// fill-rule（nonzero/缺省省略）。段级 canonical（绝对化+空格归一）归
/// engine computed 层。
pub fn path_to_css(p: &PathFunctionValue) -> String {
    match p.fill_rule {
        Some(FillRule::Evenodd) => format!("path(evenodd, \"{}\")", p.raw),
        _ => format!("path(\"{}\")", p.raw),
    }
}

/// rect() 声明级串化：`rect(T R B L[ round <radius>])`（空白分隔语料钉面）。
pub fn rect_to_css(r: &RectFunctionValue) -> String {
    let c = |x: &RectCoord| match x {
        RectCoord::Auto => "auto".to_string(),
        RectCoord::Length(l) => len_to_css(l),
    };
    let mut out = format!("rect({} {} {} {}", c(&r.top), c(&r.right), c(&r.bottom), c(&r.left));
    if let Some(rad) = &r.round {
        out.push_str(" round ");
        out.push_str(&border_radius_to_css(rad));
    }
    out.push(')');
    out
}

/// xywh() 声明级串化：`xywh(X Y W H[ round <radius>])`。
pub fn xywh_to_css(x: &XywhFunctionValue) -> String {
    let mut out = format!(
        "xywh({} {} {} {}",
        len_to_css(&x.x),
        len_to_css(&x.y),
        len_to_css(&x.w),
        len_to_css(&x.h)
    );
    if let Some(rad) = &x.round {
        out.push_str(" round ");
        out.push_str(&border_radius_to_css(rad));
    }
    out.push(')');
    out
}

/// border-radius 串化：四元组最小轮转形折叠（1/2/3/4 值）+ `/` 垂直组。
fn border_radius_to_css(r: &BorderRadiusValue) -> String {
    let mut out = push_radius_group(&r.h);
    if let Some(v) = &r.v {
        out.push_str(" / ");
        out.push_str(&push_radius_group(v));
    }
    out
}

/// 半径组最小形折叠串化（css-backgrounds-3 §border-radius）。
fn push_radius_group(vals: &[LengthValue; 4]) -> String {
    let eq = |a: &LengthValue, b: &LengthValue| a == b;
    let show: Vec<&LengthValue> = if eq(&vals[0], &vals[1]) && eq(&vals[1], &vals[2]) && eq(&vals[2], &vals[3]) {
        vec![&vals[0]]
    } else if eq(&vals[0], &vals[2]) && eq(&vals[1], &vals[3]) {
        vec![&vals[0], &vals[1]]
    } else if eq(&vals[1], &vals[3]) {
        vec![&vals[0], &vals[1], &vals[2]]
    } else {
        vals.iter().collect()
    };
    show.iter().map(|v| len_to_css(v)).collect::<Vec<_>>().join(" ")
}

/// 长度声明级串化（单位保形；6dp；Calc 单层 BinaryOp canonical）。
/// rule-inset 档共享（css-gaps-1 §inset 槽指定值 canonical 串化）。
pub fn len_to_css(lv: &LengthValue) -> String {
    match lv {
        LengthValue::Px(v) => format!("{}px", format_number_6dp(*v)),
        LengthValue::Percentage(v) => format!("{}%", format_number_6dp(*v)),
        LengthValue::Em(v) => format!("{}em", format_number_6dp(*v)),
        LengthValue::Ex(v) => format!("{}ex", format_number_6dp(*v)),
        LengthValue::Rex(v) => format!("{}rex", format_number_6dp(*v)),
        LengthValue::Cap(v) => format!("{}cap", format_number_6dp(*v)),
        LengthValue::Rcap(v) => format!("{}rcap", format_number_6dp(*v)),
        LengthValue::Rem(v) => format!("{}rem", format_number_6dp(*v)),
        LengthValue::Ch(v) => format!("{}ch", format_number_6dp(*v)),
        LengthValue::Rch(v) => format!("{}rch", format_number_6dp(*v)),
        LengthValue::Ic(v) => format!("{}ic", format_number_6dp(*v)),
        LengthValue::Ric(v) => format!("{}ric", format_number_6dp(*v)),
        LengthValue::Lh(v) => format!("{}lh", format_number_6dp(*v)),
        LengthValue::Rlh(v) => format!("{}rlh", format_number_6dp(*v)),
        LengthValue::Vh(v) => format!("{}vh", format_number_6dp(*v)),
        LengthValue::Vw(v) => format!("{}vw", format_number_6dp(*v)),
        LengthValue::Vmin(v) => format!("{}vmin", format_number_6dp(*v)),
        LengthValue::Vmax(v) => format!("{}vmax", format_number_6dp(*v)),
        LengthValue::Vb(v) => format!("{}vb", format_number_6dp(*v)),
        LengthValue::Vi(v) => format!("{}vi", format_number_6dp(*v)),
        LengthValue::Svh(v) => format!("{}svh", format_number_6dp(*v)),
        LengthValue::Svw(v) => format!("{}svw", format_number_6dp(*v)),
        LengthValue::Svmin(v) => format!("{}svmin", format_number_6dp(*v)),
        LengthValue::Svmax(v) => format!("{}svmax", format_number_6dp(*v)),
        LengthValue::Svb(v) => format!("{}svb", format_number_6dp(*v)),
        LengthValue::Svi(v) => format!("{}svi", format_number_6dp(*v)),
        LengthValue::Lvh(v) => format!("{}lvh", format_number_6dp(*v)),
        LengthValue::Lvw(v) => format!("{}lvw", format_number_6dp(*v)),
        LengthValue::Lvmin(v) => format!("{}lvmin", format_number_6dp(*v)),
        LengthValue::Lvmax(v) => format!("{}lvmax", format_number_6dp(*v)),
        LengthValue::Lvb(v) => format!("{}lvb", format_number_6dp(*v)),
        LengthValue::Lvi(v) => format!("{}lvi", format_number_6dp(*v)),
        LengthValue::Dvh(v) => format!("{}dvh", format_number_6dp(*v)),
        LengthValue::Dvw(v) => format!("{}dvw", format_number_6dp(*v)),
        LengthValue::Dvmin(v) => format!("{}dvmin", format_number_6dp(*v)),
        LengthValue::Dvmax(v) => format!("{}dvmax", format_number_6dp(*v)),
        LengthValue::Dvb(v) => format!("{}dvb", format_number_6dp(*v)),
        LengthValue::Dvi(v) => format!("{}dvi", format_number_6dp(*v)),
        LengthValue::Cqw(v) => format!("{}cqw", format_number_6dp(*v)),
        LengthValue::Cqh(v) => format!("{}cqh", format_number_6dp(*v)),
        LengthValue::Cqi(v) => format!("{}cqi", format_number_6dp(*v)),
        LengthValue::Cqb(v) => format!("{}cqb", format_number_6dp(*v)),
        LengthValue::Cqmin(v) => format!("{}cqmin", format_number_6dp(*v)),
        LengthValue::Cqmax(v) => format!("{}cqmax", format_number_6dp(*v)),
        LengthValue::Calc(expr) => calc_to_css(expr),
        // 形状坐标解析门内不可达（Auto/min-content 等已被 parse_shape_len 拒）
        _ => String::new(),
    }
}

/// 形状坐标 calc canonical 串化（单层 BinaryOp / 叶；深形回落折叠原文由
/// engine computed 通道处理——此处保 parse 门内实形）。
fn calc_to_css(expr: &CalcExpr) -> String {
    match expr {
        CalcExpr::Number(v) => format_number_6dp(*v),
        CalcExpr::Length(l) => len_to_css(l),
        CalcExpr::BinaryOp(l, op, r) => {
            // 单层形 `calc(A op B)`；规范空格。
            // `0 − v`（parse_math_function 一元负号产物）折 `-v`：
            // `calc(0px - 4.5%)` → `calc(-4.5%)`；左操作数嵌套负号同样
            // 展开——`calc((0px - 4.5%) - 6.5px)` → `calc(-4.5% - 6.5px)`
            //（语料钉面）
            let l_str = match &**l {
                CalcExpr::BinaryOp(ll, CalcOp::Subtract, lr) if calc_leaf_is_zero(ll) => {
                    format!("-{}", calc_to_css(lr))
                }
                _ => calc_to_css(l),
            };
            if *op == CalcOp::Subtract && calc_leaf_is_zero(l) {
                return format!("calc(-{})", calc_to_css(r));
            }
            format!(
                "calc({} {} {})",
                l_str,
                match op {
                    CalcOp::Add => "+",
                    CalcOp::Subtract => "-",
                    CalcOp::Multiply => "*",
                    CalcOp::Divide => "/",
                },
                calc_to_css(r)
            )
        }
        _ => String::new(),
    }
}

/// calc 叶零值判定（一元负号折叠用；Number(0)/零长度原子）。
fn calc_leaf_is_zero(e: &CalcExpr) -> bool {
    match e {
        CalcExpr::Number(v) => *v == 0.0,
        CalcExpr::Length(l) => matches!(l, LengthValue::Px(n) | LengthValue::Percentage(n) if *n == 0.0),
        _ => false,
    }
}

/// 6 位小数舍入数字串化（css-values-4 §7.1；serialize_math 同语义）。
fn format_number_6dp(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value > 0.0 {
            "infinity".into()
        } else {
            "-infinity".into()
        };
    }
    let mut s = format!("{value:.6}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".into();
    }
    s
}

#[cfg(test)]
mod tests;
