//! CSS Values 5 [`calc-mix()`](https://drafts.csswg.org/css-values-5/#calc-mix)
//! 值混合（tentative 草案面，WPT css-values calc-mix-serialize/computed/invalid
//! 三页语料驱动）。
//!
//! 语义钉面（草案未定稿段落以语料期望为口径）：
//! - 权重 clamp 到 [0,100]；缺失权重项均分剩余 `max(0, 100-Σ)/n`；
//!   分母 `max(Σ, 100)`——结果 `Σ(vᵢ·wᵢ)/denominator`（serialize 页
//!   「Weight percentages add up to less/more than 100%」两组行反推）。
//! - 指定值折叠仅字面权重参与；任一权重为 calc() 表达式形 → 整体原样保留
//!   （serialize 页 `calc-mix(1 calc(150%), 3 100%)` 行，Chrome/Blink 口径）。
//! - 字面权重文法 `<percentage [0,100]>`（`1 150%`/`1 -50%` 拒）；calc() 形
//!   权重不受字面范围门（`calc(150%)` 合法，求值期 clamp——invalid 页与
//!   serialize 页对同族的分工）。
//! - 项型别一致（§10.5 同族）：number / length-percentage / angle / time；
//!   angle 叶 parse 期已 Number 化（弧度），以项值原文顶层角度单位判定。

use super::parse_extended::{CalcMathType, calc_expr_math_type};
use super::types::{CalcContext, CalcExpr, LengthValue};

/// calc-mix() 项权重。
#[derive(Debug, Clone, PartialEq)]
pub enum CalcMixWeight {
    /// 字面 `<percentage [0,100]>`（parse 期范围门已过；指定值折叠参与面）。
    Literal(f64),
    /// calc() 等表达式形（含 % 叶；求值期 clamp；指定值折叠不参与）。
    Expr(CalcExpr),
}

/// calc-mix() 单项：值表达式 + 可选权重。
#[derive(Debug, Clone, PartialEq)]
pub struct CalcMixItem {
    /// 项值表达式（`<calc-sum>`）。
    pub value: CalcExpr,
    /// 项值原文（trim 后）——指定值归一化串直接复用（em 等布局依赖单位
    /// 无求值形态，原文保留单位原貌；serialize 页 `calc-mix(10px 50%, 3em 50%)` 形）。
    pub raw: Box<str>,
    /// 权重；`None` = 缺失（分配剩余权重）。
    pub weight: Option<CalcMixWeight>,
}

/// 字面百分比 token 扫描：`[+-]?<digits>[.<digits>]?%` 且 token 后必须为项界
///（空白/`,`/`)`/EOF——`1 50%, 3` 的 `50%` 后是 `,`；`1 50, 3` 无 `%` 返 None
/// 落表达式臂（number 树无 % 叶被拒）。
/// 返回 (百分比数值, 消费字节数)（含前导空白）。
pub(crate) fn literal_percentage(s: &str) -> Option<(f64, usize)> {
    let lead = s.len() - s.trim_start().len();
    let t = &s[lead..];
    let b = t.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let ds = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == fs {
            return None;
        }
    }
    if i == ds || i >= b.len() || b[i] != b'%' {
        return None;
    }
    i += 1;
    // 权重 token 后必须为项界（防 `1 50%px` 等粘连形态）。
    let after = t[i..].trim_start();
    if !after.is_empty() && !after.starts_with(',') && !after.starts_with(')') {
        return None;
    }
    t[..i - 1].parse::<f64>().ok().map(|v| (v, lead + i))
}

/// 树内是否含百分比叶（权重表达式文法门——权重必须是百分比型；
/// `1 50` 的裸 number 权重拒）。
pub(crate) fn expr_has_percentage_leaf(expr: &CalcExpr) -> bool {
    match expr {
        CalcExpr::Length(LengthValue::Percentage(_)) => true,
        CalcExpr::Number(_) | CalcExpr::Length(_) | CalcExpr::Time(_) | CalcExpr::Resolution(_) => false,
        CalcExpr::BinaryOp(a, _, b) | CalcExpr::BinaryMathOp(_, a, b) => {
            expr_has_percentage_leaf(a) || expr_has_percentage_leaf(b)
        }
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter().any(expr_has_percentage_leaf),
        CalcExpr::Clamp { min, val, max } => {
            expr_has_percentage_leaf(min) || expr_has_percentage_leaf(val) || expr_has_percentage_leaf(max)
        }
        CalcExpr::UnaryOp(_, inner) => expr_has_percentage_leaf(inner),
        CalcExpr::Mix(items) => items.iter().any(|it| {
            expr_has_percentage_leaf(&it.value)
                || matches!(&it.weight, Some(CalcMixWeight::Expr(w)) if expr_has_percentage_leaf(w))
        }),
        // tree-counting 叶无百分比（权重表达式文法门 `%` 叶判定只看维度叶）。
        CalcExpr::TreeCounting(_) => false,
        // anchor()/anchor-size() 叶产出锚几何 px（fallback % 在函数内消解，
        // 不外泄）——% 折叠门 opaque 不参与。
        CalcExpr::Anchor { .. } => false,
        // progress() 实参内 % 叶递归上报（权重表达式文法门 faithful 扫描）。
        CalcExpr::Progress { value, start, end, .. } => {
            expr_has_percentage_leaf(value) || expr_has_percentage_leaf(start) || expr_has_percentage_leaf(end)
        }
    }
}

/// 树内是否含 progress 节点**之外**的百分比叶（% 布局依赖折叠门消费——
/// progress 实参内 % 在函数内比值消解、与基准无关，不构成外泄维度：
/// `progress(1%, 0%, 100%)` 可标量折叠，`calc(100% + 50%)` 保留原文）。
pub(crate) fn expr_has_percentage_leaf_outside_progress(expr: &CalcExpr) -> bool {
    match expr {
        CalcExpr::Length(LengthValue::Percentage(_)) => true,
        CalcExpr::Number(_) | CalcExpr::Length(_) | CalcExpr::Time(_) | CalcExpr::Resolution(_) => false,
        CalcExpr::BinaryOp(a, _, b) | CalcExpr::BinaryMathOp(_, a, b) => {
            expr_has_percentage_leaf_outside_progress(a) || expr_has_percentage_leaf_outside_progress(b)
        }
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter().any(expr_has_percentage_leaf_outside_progress),
        CalcExpr::Clamp { min, val, max } => {
            expr_has_percentage_leaf_outside_progress(min)
                || expr_has_percentage_leaf_outside_progress(val)
                || expr_has_percentage_leaf_outside_progress(max)
        }
        CalcExpr::UnaryOp(_, inner) => expr_has_percentage_leaf_outside_progress(inner),
        CalcExpr::Mix(items) => items.iter().any(|it| {
            expr_has_percentage_leaf_outside_progress(&it.value)
                || matches!(&it.weight, Some(CalcMixWeight::Expr(w)) if expr_has_percentage_leaf_outside_progress(w))
        }),
        CalcExpr::TreeCounting(_) => false,
        // anchor()/anchor-size() 叶 % 依赖函数内消解（fallback 槽轴基），
        // 外泄维度折叠门 opaque。
        CalcExpr::Anchor { .. } => false,
        // progress() 节点屏蔽（实参 % 函数内消解——递归不入）。
        CalcExpr::Progress { .. } => false,
    }
}

/// 树内是否含 calc-mix 节点（嵌套和串折叠路由——顶层非 Mix 但树内嵌
/// calc-mix() 时 serialize_math 走 canonical 和串臂）。
pub(crate) fn expr_contains_mix(expr: &CalcExpr) -> bool {
    match expr {
        CalcExpr::Mix(_) => true,
        CalcExpr::Number(_) | CalcExpr::Length(_) | CalcExpr::Time(_) | CalcExpr::Resolution(_) => false,
        CalcExpr::BinaryOp(a, _, b) | CalcExpr::BinaryMathOp(_, a, b) => expr_contains_mix(a) || expr_contains_mix(b),
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter().any(expr_contains_mix),
        CalcExpr::Clamp { min, val, max } => expr_contains_mix(min) || expr_contains_mix(val) || expr_contains_mix(max),
        CalcExpr::UnaryOp(_, inner) => expr_contains_mix(inner),
        // tree-counting 叶非 Mix 节点。
        CalcExpr::TreeCounting(_) => false,
        // anchor()/anchor-size() 叶非 Mix 节点。
        CalcExpr::Anchor { .. } => false,
        // progress() 实参内嵌 calc-mix 递归上报（嵌套和串折叠路由）。
        CalcExpr::Progress { value, start, end, .. } => {
            expr_contains_mix(value) || expr_contains_mix(start) || expr_contains_mix(end)
        }
    }
}

/// calc-mix() 项族型别一致性门（§10.5 同族）：全项同文法族，任一无效 →
/// [`CalcMathType::Invalid`]。angle 叶 parse 期已 Number 化（弧度），以项值
/// 原文顶层角度单位判定（serialize_math `span_has_top_level_angle_unit` 同款
/// provenance——`1deg 50%, 3turn 50%` 在 number/length 槽均须拒）。
pub(crate) fn mix_math_type(items: &[CalcMixItem], value_has_angle_unit: &[bool]) -> CalcMathType {
    let mut first: Option<CalcMathType> = None;
    for (it, has_angle) in items.iter().zip(value_has_angle_unit) {
        let mut t = calc_expr_math_type(&it.value);
        if *has_angle {
            // 顶层角度单位叶 → <angle> 族（树内已 Number 化，原文补 provenance）。
            t = if t == CalcMathType::Number || t == CalcMathType::Angle {
                CalcMathType::Angle
            } else {
                // 非纯 number/angle 树混角度单位属病态形（parse 期不可达，防御）。
                CalcMathType::Invalid
            };
        }
        if t == CalcMathType::Invalid {
            return CalcMathType::Invalid;
        }
        match first {
            None => first = Some(t),
            Some(f) if f != t => return CalcMathType::Invalid,
            _ => {}
        }
    }
    first.unwrap_or(CalcMathType::Invalid)
}

/// 权重表达式求值：百分比叶直读数值（`50%` → 50.0，无 parent 基准换算——
/// 权重是归一化系数非长度），算术递归。非百分比族树返 None。
pub(crate) fn eval_weight_expr(expr: &CalcExpr) -> Option<f64> {
    match expr {
        CalcExpr::Length(LengthValue::Percentage(p)) => Some(*p),
        // 负百分比权重经 parse_factor 负号族保持重写为 `0px - <pct>`——0px
        // 常量叶按系数直读（parse 文法门 expr_has_percentage_leaf 已拒非 % 族
        // 权重树，权重树内 Px 叶只能来自该重写 artifact）。
        CalcExpr::Length(LengthValue::Px(v)) => Some(*v),
        CalcExpr::Number(n) => Some(*n),
        CalcExpr::BinaryOp(a, op, b) => {
            let x = eval_weight_expr(a)?;
            let y = eval_weight_expr(b)?;
            match op {
                super::types::CalcOp::Add => Some(x + y),
                super::types::CalcOp::Subtract => Some(x - y),
                super::types::CalcOp::Multiply => Some(x * y),
                super::types::CalcOp::Divide => Some(x / y),
            }
        }
        _ => None,
    }
}

/// calc-mix() 求值（[`CalcContext`] 上下文；项值求值与既有 calc 语义同链——
/// % 项按 parent_length 解析、em 按字号）。NaN 任一环节自然传播（IEEE）。
pub(crate) fn eval_mix(items: &[CalcMixItem], ctx: &CalcContext) -> Option<f64> {
    let mut sum_explicit = 0.0;
    let mut weights = Vec::with_capacity(items.len());
    let mut missing = 0usize;
    for it in items {
        match &it.weight {
            Some(CalcMixWeight::Literal(p)) => {
                let p = p.clamp(0.0, 100.0);
                weights.push(p);
                sum_explicit += p;
            }
            Some(CalcMixWeight::Expr(e)) => {
                let p = eval_weight_expr(e)?;
                let p = p.clamp(0.0, 100.0); // NaN clamp 仍 NaN → 下游自然传播。
                weights.push(p);
                sum_explicit += p;
            }
            None => {
                missing += 1;
                weights.push(0.0); // 占位，fill 后回填。
            }
        }
    }
    // 缺失权重均分剩余（Σ>100% 时剩余为负 → 0——serialize 页
    // `calc-mix(1 75%, 3 75%, 5, 7)` 行）。
    let fill = if missing > 0 {
        (100.0 - sum_explicit).max(0.0) / missing as f64
    } else {
        0.0
    };
    let mut total = 0.0;
    let mut acc = 0.0;
    for (it, w) in items.iter().zip(weights.iter_mut()) {
        if it.weight.is_none() {
            *w = fill;
        }
        total += *w;
        let v = match super::types::eval_calc_with_context(&it.value, ctx) {
            Some(v) => v,
            // 符号零（公式 0·v = 0）：权重为 0 的不可求值项贡献 0——DOM 依赖
            // 树（sibling-index() 等求值桥挂账期）不阻塞整体解析（serialize 页
            // `calc-mix(1 * sibling-index() 0%, 3 100%)` → `calc(3)` 语料行
            // 注记「0% so calc-mix() can still be resolved」）。可求值项照常
            // 相乘（NaN·0 = NaN 毒化不丢，IEEE 传播链保持）。
            None if *w == 0.0 => 0.0,
            None => return None,
        };
        acc += v * *w;
    }
    // 分母 max(Σ, 100)：Σ<100% 稀释不归一（`calc-mix(1 25%, 3 25%)` = 1），
    // Σ>100% 归一（`calc-mix(1 75%, 3 75%)` = 2）。Σ=0 全零 → /100 = 0。
    Some(acc / total.max(100.0))
}
