//! 数学函数 specified 值常量折叠序列化（CSS Values 4 §10.11）。
//!
//! WPT css-values serialize 桶（sin-cos-tan-serialize / acos-asin-atan-atan2-serialize）
//! 断言 `el.style.transform = 'rotate(acos(1))'` 读回 `rotate(calc(0deg))`——
//! specified 值序列化须把全常量数学函数折叠为 `calc(<结果>)` 规范文本。
//! 折叠原语在此（css-parser 面）；调用方（inline style 读回/写面归一）按需接线。

use super::calc_mix::{CalcMixItem, CalcMixWeight};
use super::parse_extended::{
    calc_expr_has_angle_result, calc_expr_has_length, calc_expr_has_tree_counting, calc_expr_is_number,
};
use super::types::{
    CalcExpr, CalcOp, LengthValue, TreeCountingFn, eval_calc, eval_calc_with_default_font, is_math_function_value,
    parse_math_function, parse_time_calc,
};

/// 折叠结果的文法型别（决定 NaN/单位序列化形态）。
///
/// - [`MathFoldKind::Number`]：`<number>` 文法位（scale/z-index 等）——
///   NaN 序列化 `calc(NaN)`。
/// - [`MathFoldKind::Angle`]：`<angle>` 文法位（rotate 等）——结果弧度转度，
///   NaN 序列化 `calc(NaN * 1deg)`（Chrome/Blink 口径，WPT serialize 页期望）。
/// - [`MathFoldKind::Length`]：`<length>` 文法位（width 等尺寸槽）——物理
///   单位（in/cm/mm/q/pt/pc）parse 期已归一 px。
/// - [`MathFoldKind::Time`]：`<time>` 文法位（transition/animation duration/
///   delay 槽）——ms parse 期已归一 s。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathFoldKind {
    /// `<number>` 文法位。
    Number,
    /// `<angle>` 文法位。
    Angle,
    /// `<length>` 文法位。
    Length,
    /// `<time>` 文法位。
    Time,
}

/// CSS Values 4 §10.11：数学函数 specified 值常量折叠。
///
/// 输入值为纯数学函数（calc/min/max/clamp/裸 trig 等）时求值并产出规范文本；
/// 非数学函数、型别不符（`<number>`/`<angle>` 位含 <length> 叶、`<length>`/
/// `<time>` 位无本维度叶）——均返 `None`（调用方保留原文）。NaN/±∞ 结果合法
///（§10.10），canonical 发射（§10.6）。
///
/// <https://drafts.csswg.org/css-values-4/#calc-serialize>
pub fn fold_math_specified(value: &str, kind: MathFoldKind) -> Option<String> {
    // Time 槽须数学函数形（plain `1s` 原样保留）——共享 calc 入口拒 time 原子
    //（CalcExpr::Time 注），先验形再经 allow_time 入口重 parse。
    if matches!(kind, MathFoldKind::Time) && !is_math_function_value(value) {
        return None;
    }
    let expr = match kind {
        MathFoldKind::Time => parse_time_calc(value)?,
        _ => parse_math_function(value)?,
    };
    // <angle>/<number> 文法位不容 <length> 叶混入（§10.5 型别一致，与
    // parse_angle_or_math 同口径）。
    if calc_expr_has_length(&expr) && !matches!(kind, MathFoldKind::Length) {
        return None;
    }
    // tree-counting 叶（sibling-index()/sibling-count()）串化臂：opaque 函数形
    // canonical（CSS Values 5 §tree-counting；calc-sibling-function-parsing 语料
    // 钉面）——先于标量求值（求值无 DOM 视图恒 None）。
    if calc_expr_has_tree_counting(&expr) {
        return fold_tree_counting_calc(&expr, kind, value);
    }
    match kind {
        // <number> 位非 number 式拒（scale 槽 `calc(10px)` 等）。
        MathFoldKind::Number if !calc_expr_is_number(&expr) => return None,
        // <length> 位须含长度叶（纯 number 式属 number 位，尺寸槽不折叠——
        // `width: calc(5)` 的裁决留给写面文法门）。
        MathFoldKind::Length if dimension_suffix(&expr).is_none() => return None,
        // <time> 位须含 time 叶（同上；`animation-duration: calc(3)` 不折叠）。
        MathFoldKind::Time if !expr_has_time_leaf(&expr) => return None,
        _ => {}
    }
    match kind {
        MathFoldKind::Angle => {
            // Angle 槽单位叶 provenance 用原文串（求值 f64 无单位信息，先行绑定）。
            let has_unit = span_has_top_level_angle_unit(&value.to_ascii_lowercase());
            let value = eval_calc(&expr, None)?;
            // 求值口径全弧度 → 度的两条 provenance：①角度产出函数
            //（asin/acos/atan/atan2）；②trig 实参外角度单位叶——CalcParser
            // parse 期把 `180deg` 类平角度叶 Number 化为弧度（型别标记丢失），
            // 缺 ② 时 `rotate(calc(180deg))` 往返恒等断言会把 180deg 折成
            // `calc(3.141593deg)`（WPT serialize 页双侧同形关键）；②须屏蔽
            // trig 实参——`tan(45deg)` 实参角度叶产出 <number>，按度直读会翻倍
            // 换算（§10.5）。平裸 number 式按度直读（与 parse_angle_or_math
            // 单位口径一致）。
            let deg = if calc_expr_has_angle_result(&expr) || has_unit {
                value.to_degrees()
            } else {
                value
            };
            Some(format!("calc({})", format_dimension_inner(deg, Some("deg"))))
        }
        // Number/Length/Time 标量折叠（% /字体维度两级求值见 scalar_value）。
        _ => fold_dimension_calc(&expr),
    }
}

/// 含 tree-counting 叶的纯 calc 树 specified 串化（CSS Values 5 §tree-counting
/// 写面 canonical；calc-sibling-function-parsing 语料 20 形钉面）：
///
/// - 顶层乘法链拆因子：单 sibling 叶（opaque，函数名归一 `sibling-index( )` →
///   `sibling-index()`）殿后；静态因子单原子时保维度原子 canonical（Length 位
///   `1em`/`1%` 布局依赖不折叠——原文回显）；
/// - Angle/Time/Number 位静态积 opaque→1 掩码求值 + 规范单位（`2rad * pi` →
///   `360deg`、`100ms` → `0.1s`；角度 deg 换算 provenance 与 [`fold_math_specified`]
///   Angle 臂同口径）；
/// - 无静态因子 → calc 包裹解包（`calc(sibling-index())` → `sibling-index()`）；
/// - 加减/min/max/clamp/trig 包裹、多 opaque、除法链含 opaque → None 保留原文
///   （FIXME: 无语料钉面）。
fn fold_tree_counting_calc(expr: &CalcExpr, kind: MathFoldKind, src: &str) -> Option<String> {
    let factors = flatten_multiply_chain(expr)?;
    let mut opaque: Option<&'static str> = None;
    let mut statics: Vec<&CalcExpr> = Vec::new();
    for f in factors {
        match f {
            CalcExpr::TreeCounting(fn_) => {
                if opaque.is_some() {
                    return None;
                }
                opaque = Some(match fn_ {
                    TreeCountingFn::SiblingIndex => "sibling-index()",
                    TreeCountingFn::SiblingCount => "sibling-count()",
                });
            }
            other => statics.push(other),
        }
    }
    let opaque_str = opaque?;
    let static_str = match statics.as_slice() {
        // 单 opaque 叶：calc 包裹解包（z-index 语料 `calc(sibling-index())`
        // → `sibling-index()` 钉面）。
        [] => return Some(opaque_str.to_string()),
        // Length 位单静态原子保维度 canonical（`1em`/`1%` 原文回显；布局
        // 依赖值不折叠——求值无基准）。
        [CalcExpr::Length(lv)] if kind == MathFoldKind::Length => {
            format!("{}{}", format_number_6dp(length_atom_value(lv)?), length_suffix(lv)?)
        }
        _ => fold_static_masked(expr, kind, src)?,
    };
    Some(format!("calc({static_str} * {opaque_str})"))
}

/// 静态积串化：opaque 叶 → 1 掩码后整体求值（纯静态树，无 DOM 视图可达），
/// Angle/Time/Number 按槽位规范单位发射。
fn fold_static_masked(expr: &CalcExpr, kind: MathFoldKind, src: &str) -> Option<String> {
    let mut masked = expr.clone();
    mask_tree_counting(&mut masked, 1.0);
    let v = eval_calc(&masked, None)?;
    Some(match kind {
        // 角度 deg 换算 provenance（角度产出函数 / 原文单位叶）与
        // [`fold_math_specified`] Angle 臂同口径。
        MathFoldKind::Angle => {
            let deg = if calc_expr_has_angle_result(&masked) || span_has_top_level_angle_unit(&src.to_ascii_lowercase())
            {
                v.to_degrees()
            } else {
                v
            };
            format_dimension_inner(deg, Some("deg"))
        }
        MathFoldKind::Time => format_dimension_inner(v, Some("s")),
        _ => format_dimension_inner(v, None),
    })
}

/// 后序遍历：树内全部 tree-counting 叶替换为数值（掩码求值用）。
fn mask_tree_counting(expr: &mut CalcExpr, v: f64) {
    match expr {
        CalcExpr::TreeCounting(_) => *expr = CalcExpr::Number(v),
        CalcExpr::Number(_) | CalcExpr::Length(_) | CalcExpr::Time(_) => {}
        CalcExpr::BinaryOp(l, _, r) => {
            mask_tree_counting(l, v);
            mask_tree_counting(r, v);
        }
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter_mut().for_each(|a| mask_tree_counting(a, v)),
        CalcExpr::Clamp { min, val, max } => {
            mask_tree_counting(min, v);
            mask_tree_counting(val, v);
            mask_tree_counting(max, v);
        }
        CalcExpr::UnaryOp(_, inner) => mask_tree_counting(inner, v),
        // calc-mix() 项/权重内 opaque：串化边界外（fold_mix_dimension 先路由）。
        CalcExpr::Mix(_) => {}
        CalcExpr::BinaryMathOp(_, l, r) => {
            mask_tree_counting(l, v);
            mask_tree_counting(r, v);
        }
    }
}

/// 顶层乘法链因子展开（仅 [`CalcOp::Multiply`]；其他复合形态返 None）。
fn flatten_multiply_chain(expr: &CalcExpr) -> Option<Vec<&CalcExpr>> {
    match expr {
        CalcExpr::BinaryOp(l, CalcOp::Multiply, r) => {
            let mut out = flatten_multiply_chain(l)?;
            out.extend(flatten_multiply_chain(r)?);
            Some(out)
        }
        CalcExpr::TreeCounting(_) => Some(vec![expr]),
        // 加减/min/max/函数包裹等复合树含 opaque：串化边界外（FIXME 无钉面）。
        _ if calc_expr_has_tree_counting(expr) => None,
        leaf => Some(vec![leaf]),
    }
}

/// [`LengthValue`] calc 树可达维度原子的数值（关键字/嵌套式防御臂返 None）。
fn length_atom_value(lv: &LengthValue) -> Option<f64> {
    Some(match lv {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Cqw(v)
        | LengthValue::Cqh(v)
        | LengthValue::Cqi(v)
        | LengthValue::Cqb(v)
        | LengthValue::Cqmin(v)
        | LengthValue::Cqmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Lh(v)
        | LengthValue::Percentage(v) => *v,
        _ => return None,
    })
}

/// 裸 sibling 函数形识别（无 calc 包裹；容忍内部空白 `sibling-index( )`）——
/// z-index `<integer>` 槽语料形态。返回归一函数名。
fn sibling_bare_fn(v: &str) -> Option<&'static str> {
    let lower = v.trim().to_ascii_lowercase();
    for (name, canonical) in [
        ("sibling-index", "sibling-index()"),
        ("sibling-count", "sibling-count()"),
    ] {
        if let Some(rest) = lower.strip_prefix(name) {
            if rest.trim() == "()" {
                return Some(canonical);
            }
        }
    }
    None
}

/// 维度标量折叠入口：整体求值 NaN 且树内存在「任一实参 NaN 且实参 canonical
/// 单位分歧」clamp 节点时结构外提（§10.6——NaN × 任意数恒 NaN，外层标量上下文
/// 丢弃，clamp 实参单位维度信息保留到 used-value 期 censor；
/// `calc(1 * clamp(NaN * 2em, NaN * 4px, NaN * 8pt))` →
/// `clamp(NaN * 1em, NaN * 1px, NaN * 1px)`，calc-infinity-nan-serialize-length
/// 语料行驱动）；否则整体塌缩为 `calc(<n>)`/`calc(<n><unit>)`（min/max 含
/// NaN 亦塌缩——结果单位取树首维度叶，§10.6 毒化语义求值期已定）。
fn fold_dimension_calc(expr: &CalcExpr) -> Option<String> {
    if let CalcExpr::Mix(items) = expr {
        return fold_mix_dimension(items, expr);
    }
    if let Some(s) = fold_mix_nest_sum(expr) {
        return Some(s);
    }
    if scalar_value(expr).is_some_and(|v| v.is_nan()) {
        if let Some([min, val, max]) = find_divergent_nan_clamp(expr) {
            return Some(format!(
                "clamp({}, {}, {})",
                scalar_fold_inner(min)?,
                scalar_fold_inner(val)?,
                scalar_fold_inner(max)?
            ));
        }
    }
    // 有限百分比树不标量塌缩：eval 对 % 叶直读数值（无布局基准），min/max
    // 混 % 形会丢维度语义（`min(20px, 10%)` 误塌 `calc(20px)`——minmax-length-
    // percent-serialize 语料行）；`calc(100% + 50%)` 保留原文同口径（§10.6
    // 两级求值③）。非有限毒化形不拦——`calc(1% * NaN)` → `calc(NaN * 1%)`
    // canonical 依赖 % 叶单位 provenance（calc-infinity-nan-serialize-length
    // 语料桶）。
    if super::calc_mix::expr_has_percentage_leaf(expr) {
        let v = scalar_value(expr)?;
        if !v.is_finite() {
            return Some(format!("calc({})", format_dimension_inner(v, dimension_suffix(expr))));
        }
        return None;
    }
    Some(format!("calc({})", scalar_fold_inner(expr)?))
}

/// calc-mix() 指定值折叠（CSS Values 5 tentative，serialize/computed 语料钉面）：
/// ① 任一 calc() 形权重 → 原样保留（None——serialize 页
/// `calc-mix(1 calc(150%), 3 100%)` 行，Chrome/Blink 口径：指定值期不折叠
/// calc 形权重）；② 全零塌缩（无缺失且全字面 0 → `calc(0<首项单位>)`，
/// % 项树标量求值 fail 的救面——`calc-mix(1% 0%, 3px 0%)` → `calc(0%)`）；
/// ③ 标量塌缩（全字面权重 + 值可解析 → `calc(<n>[unit])`）；
/// ④ 归一化串（布局依赖树——em 等项标量不可解析：缺失填充/Σ>100% 归一/
/// 零权重缺失项丢弃，值项复用原文保单位原貌）。
fn fold_mix_dimension(items: &[CalcMixItem], expr: &CalcExpr) -> Option<String> {
    if let Some((v, suffix)) = mix_leaf_term(items, expr) {
        return Some(format!("calc({})", format_dimension_inner(v, suffix)));
    }
    if items.iter().any(|it| matches!(it.weight, Some(CalcMixWeight::Expr(_)))) {
        return None;
    }
    fold_mix_normalized(items)
}

/// Mix 叶项化（嵌套和串收集面，[`fold_mix_dimension`] 前两臂共用）：Expr 形
/// 权重不可项化；全字面零权重塌缩 (0, 首项单位)；标量可解析塌缩 (值, 树首
/// 单位)。布局依赖树（em 等）返 None——顶层走归一化串臂、嵌套和串整体
/// 放弃项化。
fn mix_leaf_term(items: &[CalcMixItem], expr: &CalcExpr) -> Option<(f64, Option<&'static str>)> {
    if items.iter().any(|it| matches!(it.weight, Some(CalcMixWeight::Expr(_)))) {
        return None;
    }
    let all_zero = !items.is_empty()
        && items
            .iter()
            .all(|it| matches!(it.weight, Some(CalcMixWeight::Literal(p)) if p == 0.0));
    if all_zero {
        return Some((0.0, dimension_suffix(&items[0].value)));
    }
    // 纯 % 值项字面权重混合：加权和符号解析保 % 维度（% 叶 scalar_value
    // 无基准盲区补面——`calc(10px + calc-mix(10% 50%, 30% 50%))` →
    // `calc(20% + 10px)` 语料行；权重/填充/分母公式同 eval_mix，used 值
    // 期才需布局基准）。
    if let Some(ws) = filled_literal_weights(items) {
        if let Some(acc) = items.iter().zip(&ws).try_fold(0.0, |acc, (it, w)| match &it.value {
            CalcExpr::Length(LengthValue::Percentage(p)) => Some(acc + p * w),
            _ => None,
        }) {
            let denom = ws.iter().copied().sum::<f64>().max(100.0);
            return Some((acc / denom, dimension_suffix(expr)));
        }
    }
    scalar_value(expr).map(|v| (v, dimension_suffix(expr)))
}

/// 字面权重有效值表：clamp [0,100] + 缺失填充 `max(0,100-Σ)/n`；含 Expr 形
/// 权重 → None。归一化串臂与 % 值符号解析臂共用（公式同 eval_mix 求值期）。
fn filled_literal_weights(items: &[CalcMixItem]) -> Option<Vec<f64>> {
    if items.iter().any(|it| matches!(it.weight, Some(CalcMixWeight::Expr(_)))) {
        return None;
    }
    let n_missing = items.iter().filter(|it| it.weight.is_none()).count();
    let sum_explicit: f64 = items
        .iter()
        .filter_map(|it| match &it.weight {
            Some(CalcMixWeight::Literal(p)) => Some(p.clamp(0.0, 100.0)),
            _ => None,
        })
        .sum();
    let fill = if n_missing > 0 {
        (100.0 - sum_explicit).max(0.0) / n_missing as f64
    } else {
        0.0
    };
    Some(
        items
            .iter()
            .map(|it| match &it.weight {
                Some(CalcMixWeight::Literal(p)) => p.clamp(0.0, 100.0),
                _ => fill,
            })
            .collect(),
    )
}

/// calc-mix() 指定值归一化串：缺失权重填充 `max(0, 100-Σ)/n`、填充为零的
/// 缺失项丢弃、Σ>100% 全项除以 Σ（serialize 页「more than 100%」组
/// `calc-mix(10px 75%, 3em 75%, 5em, 7em)` → `calc-mix(10px 50%, 3em 50%)`）；
/// Σ<100% 且无缺失时保持原权重（同页 25% 组不归一）。可达性：仅布局依赖
/// 树（标量不可解析）——全字面权重 number 树标量恒可解析，本臂仅 Length
/// 槽可达。
fn fold_mix_normalized(items: &[CalcMixItem]) -> Option<String> {
    let ws = filled_literal_weights(items)?;
    let mut parts: Vec<(&str, f64)> = Vec::with_capacity(items.len());
    for (it, w) in items.iter().zip(&ws) {
        if it.weight.is_some() || *w > 0.0 {
            parts.push((&it.raw, *w));
        }
        // fill=0（Σ≥100%）→ 缺失项丢弃。
    }
    if parts.is_empty() {
        return None;
    }
    let total: f64 = parts.iter().map(|(_, w)| *w).sum();
    // 归一化除数按百分数口径：Σ>100% 时各项 w/(Σ/100)（75%/75% 组 → 50%），
    // Σ≤100% 保持原权重数字。
    let div = if total > 100.0 { total / 100.0 } else { 1.0 };
    let inner = parts
        .iter()
        .map(|(raw, w)| format!("{} {}%", raw, format_number_6dp(w / div)))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("calc-mix({})", inner))
}

/// 嵌套 calc-mix 树的指定值 canonical 和串（CSS Values 4 §10.9 指定值
/// canonical 化；calc-mix serialize 语料「外层 calc 嵌套」桶驱动）：顶层非
/// Mix 但加法树含 Mix 叶——各叶项化（标量叶直读、Mix 叶按 [`mix_leaf_term`]
/// 塌缩取维度），同单位常量合并、% 项前置其余保序、单项落既有塌缩串
///（`calc(10px + calc-mix(1% 0%, 3px 0%))` → `calc(0% + 10px)`）。
/// min/max/clamp/数学函数复合节点、布局依赖叶、number 位 Mix 叶不可项化
/// → None 保持原文（sibling-index() 形挂账同理）。
fn fold_mix_nest_sum(expr: &CalcExpr) -> Option<String> {
    if !super::calc_mix::expr_contains_mix(expr) {
        return None;
    }
    let mut terms = Vec::new();
    collect_sum_terms(expr, 1.0, &mut terms)?;
    let mut merged: Vec<(f64, Option<&'static str>)> = Vec::new();
    for (v, s) in terms {
        if let Some(m) = merged.iter_mut().find(|(_, ms)| *ms == s) {
            m.0 += v;
        } else {
            merged.push((v, s));
        }
    }
    // % 项前置（语料面 px+% 形 canonical 序）；FIXME(canonical): 完整单位序
    //（number < % < 各 canonical 单位序）待全值域钉面。
    merged.sort_by_key(|(_, s)| if *s == Some("%") { 0 } else { 1 });
    if let [(v, s)] = &merged[..] {
        return Some(format!("calc({})", format_dimension_inner(*v, *s)));
    }
    let inner = merged
        .iter()
        .map(|(v, s)| format_dimension_inner(*v, *s))
        .collect::<Vec<_>>()
        .join(" + ");
    Some(format!("calc({})", inner))
}

/// 加法树叶收集（符号沿 Subtract 右臂传播）；非有限值毒化返 None（NaN
/// 塌缩臂接手）。乘除实参位、min/max/clamp/数学函数节点不展开——叶级
/// 常量项化仅覆盖纯和差树。
fn collect_sum_terms(expr: &CalcExpr, sign: f64, terms: &mut Vec<(f64, Option<&'static str>)>) -> Option<()> {
    match expr {
        CalcExpr::BinaryOp(l, op, r) => match op {
            CalcOp::Add => {
                collect_sum_terms(l, sign, terms)?;
                collect_sum_terms(r, sign, terms)
            }
            CalcOp::Subtract => {
                collect_sum_terms(l, sign, terms)?;
                collect_sum_terms(r, -sign, terms)
            }
            _ => None,
        },
        CalcExpr::Mix(items) => {
            let (v, suffix) = mix_leaf_term(items, expr)?;
            if !v.is_finite() {
                return None;
            }
            // number 位 Mix 叶在维度和串中不可项化（number+length 型别混入，
            // 语料无驱动形；None 落回既有标量塌缩臂零漂移）。
            suffix?;
            terms.push((sign * v, suffix));
            Some(())
        }
        // 裸 % 叶：无 ctx 不可标量求值（scalar_value 两级 fallback 均滤有限
        // 值）——系数直读，% 即 canonical 后缀（`calc(10% + 0%)` 合并面）。
        CalcExpr::Length(LengthValue::Percentage(p)) => {
            terms.push((sign * *p, Some("%")));
            Some(())
        }
        _ => {
            let v = scalar_value(expr)?;
            if !v.is_finite() {
                return None;
            }
            let suffix = dimension_suffix(expr)?;
            terms.push((sign * v, Some(suffix)));
            Some(())
        }
    }
}

/// 树内首个「任一实参 NaN 且实参 canonical 单位分歧」clamp 节点（左序深先）。
fn find_divergent_nan_clamp(expr: &CalcExpr) -> Option<[&CalcExpr; 3]> {
    match expr {
        CalcExpr::Clamp { min, val, max } => {
            let args = [min.as_ref(), val.as_ref(), max.as_ref()];
            let any_nan = args.iter().any(|a| scalar_value(a).is_some_and(|v| v.is_nan()));
            let suffixes = args.map(dimension_suffix);
            let divergent = suffixes[0] != suffixes[1] || suffixes[1] != suffixes[2];
            if any_nan && divergent {
                return Some(args);
            }
            args.into_iter().find_map(find_divergent_nan_clamp)
        }
        CalcExpr::Number(_) | CalcExpr::Length(_) | CalcExpr::Time(_) => None,
        CalcExpr::BinaryOp(a, _, b) | CalcExpr::BinaryMathOp(_, a, b) => {
            find_divergent_nan_clamp(a).or_else(|| find_divergent_nan_clamp(b))
        }
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter().find_map(find_divergent_nan_clamp),
        CalcExpr::UnaryOp(_, inner) => find_divergent_nan_clamp(inner),
        CalcExpr::Mix(items) => items
            .iter()
            .filter_map(|it| {
                find_divergent_nan_clamp(&it.value).or_else(|| match &it.weight {
                    Some(CalcMixWeight::Expr(w)) => find_divergent_nan_clamp(w),
                    _ => None,
                })
            })
            .next(),
        // tree-counting 叶非复合节点（无 clamp 实参位）。
        CalcExpr::TreeCounting(_) => None,
    }
}

/// 两级求值（§10.6 非有限值与参考维度无关；有限维度依赖值不折叠）：
/// ①无 ctx 求值（px/s/纯 number 树可达）；②失败→默认字体基准重试
///（em/rem 树，NaN/∞ 与字体无关；有限 em 值依赖实际字体，不折叠）；
/// ③仍失败→parent_length=0 参考核证（% 树；有限 % 式布局依赖，不折叠——
/// `calc(100% + 50%)` 保留原文，Chrome 同口径）。
fn scalar_value(expr: &CalcExpr) -> Option<f64> {
    match eval_calc(expr, None) {
        Some(v) => Some(v),
        None => match eval_calc_with_default_font(expr) {
            Some(v) if !v.is_finite() => Some(v),
            None => eval_calc(expr, Some(0.0)).filter(|v| !v.is_finite()),
            _ => None,
        },
    }
}

/// 折叠内文（无 calc 包裹）：有限 → `<n>`/`<n><unit>`；NaN/±∞ →
/// `NaN|±infinity * 1<unit>`（§10.6 canonical，`calc(NaN * 1px)` 语料形态）；
/// 无维度叶裸形态（number 位 `calc(NaN)`）。
fn format_dimension_inner(value: f64, suffix: Option<&str>) -> String {
    match (value.is_finite(), suffix) {
        (true, Some(s)) => format!("{}{}", format_number_6dp(value), s),
        (true, None) | (false, None) => format_number_6dp(value),
        (false, Some(s)) => format!("{} * 1{}", format_number_6dp(value), s),
    }
}

/// 标量折叠内文：求值（[`scalar_value`] 两级）+ 树首维度叶 canonical 单位。
fn scalar_fold_inner(expr: &CalcExpr) -> Option<String> {
    Some(format_dimension_inner(scalar_value(expr)?, dimension_suffix(expr)))
}

/// 树首维度叶 canonical 单位后缀（左序第一个 Length/Time 叶——min/max NaN
/// 塌缩与非有限发射的单位 provenance；物理单位 parse 期已归一 px、ms 归一 s，
/// 后缀即 canonical 形）。无维度叶 → None（number 位）。
fn dimension_suffix(expr: &CalcExpr) -> Option<&'static str> {
    match expr {
        CalcExpr::Number(_) => None,
        CalcExpr::Length(lv) => length_suffix(lv),
        CalcExpr::Time(_) => Some("s"),
        CalcExpr::BinaryOp(a, _, b) | CalcExpr::BinaryMathOp(_, a, b) => {
            dimension_suffix(a).or_else(|| dimension_suffix(b))
        }
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter().find_map(dimension_suffix),
        CalcExpr::Clamp { min, val, max } => dimension_suffix(min)
            .or_else(|| dimension_suffix(val))
            .or_else(|| dimension_suffix(max)),
        CalcExpr::UnaryOp(_, inner) => dimension_suffix(inner),
        // calc-mix()：树首维度叶 = 首项维度叶（全零塌缩与 NaN 塌缩的单位
        // provenance——`calc-mix(1% 0%, 3px 0%)` → `calc(0%)` 首项 % 语料钉面）。
        CalcExpr::Mix(items) => items.iter().find_map(|it| dimension_suffix(&it.value)),
        // tree-counting 叶无维度单位（DOM 序号 <integer>）——or_else/find_map
        // 链继续向其余叶溯源（`1% * sibling-index()` 单位取 % 叶）。
        CalcExpr::TreeCounting(_) => None,
    }
}

/// [`LengthValue`] 维度原子 → canonical 后缀（calc 树内仅维度原子可达；
/// 关键字/嵌套式防御臂返 None）。
fn length_suffix(lv: &LengthValue) -> Option<&'static str> {
    Some(match lv {
        LengthValue::Px(_) => "px",
        LengthValue::Em(_) => "em",
        LengthValue::Ex(_) => "ex",
        LengthValue::Rex(_) => "rex",
        LengthValue::Cap(_) => "cap",
        LengthValue::Rcap(_) => "rcap",
        LengthValue::Rem(_) => "rem",
        LengthValue::Vh(_) => "vh",
        LengthValue::Vw(_) => "vw",
        LengthValue::Vmin(_) => "vmin",
        LengthValue::Vmax(_) => "vmax",
        LengthValue::Cqw(_) => "cqw",
        LengthValue::Cqh(_) => "cqh",
        LengthValue::Cqi(_) => "cqi",
        LengthValue::Cqb(_) => "cqb",
        LengthValue::Cqmin(_) => "cqmin",
        LengthValue::Cqmax(_) => "cqmax",
        LengthValue::Ch(_) => "ch",
        LengthValue::Rch(_) => "rch",
        LengthValue::Ic(_) => "ic",
        LengthValue::Ric(_) => "ric",
        LengthValue::Lh(_) => "lh",
        LengthValue::Percentage(_) => "%",
        _ => return None,
    })
}

/// 树内是否含 [`CalcExpr::Time`] 叶（<time> 位折叠门）。
fn expr_has_time_leaf(expr: &CalcExpr) -> bool {
    match expr {
        CalcExpr::Time(_) => true,
        CalcExpr::Number(_) | CalcExpr::Length(_) => false,
        CalcExpr::BinaryOp(a, _, b) | CalcExpr::BinaryMathOp(_, a, b) => expr_has_time_leaf(a) || expr_has_time_leaf(b),
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter().any(expr_has_time_leaf),
        CalcExpr::Clamp { min, val, max } => {
            expr_has_time_leaf(min) || expr_has_time_leaf(val) || expr_has_time_leaf(max)
        }
        CalcExpr::UnaryOp(_, inner) => expr_has_time_leaf(inner),
        CalcExpr::Mix(items) => items.iter().any(|it| {
            expr_has_time_leaf(&it.value) || matches!(&it.weight, Some(CalcMixWeight::Expr(w)) if expr_has_time_leaf(w))
        }),
        // tree-counting 叶产出 <integer>（非 <time>）。
        CalcExpr::TreeCounting(_) => false,
    }
}

/// 声明值 specified 折叠入口（按属性分派；None = 保留原文）。
///
/// 消费方：inline style 写面（native CSD canonicalize 链）与读出视图
///（mutation log replay `style_from_mutations_lw`）。白名单仅本轮有 WPT
/// 驱动案的属性：transform（rotate/scale 族 wrapper 折叠）、rotate/scale/
/// opacity（裸数学函数）、width + transition/animation duration/delay
///（calc-infinity-nan-serialize-length/time 语料桶）。其余属性原样保留。
pub fn fold_decl_specified(prop: &str, value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    match prop {
        "transform" => fold_transform_math_specified(v),
        "rotate" => fold_math_specified(v, MathFoldKind::Angle),
        "scale" | "opacity" => fold_math_specified(v, MathFoldKind::Number),
        // 尺寸/时长槽数学形 canonical 折叠（g12 刀2：CSD 层 camelCase→kebab
        // 归一后到达，白名单 kebab 形）。text-indent 为 <length-percentage> 槽
        //（calc-mix 语料 serialize 页驱动 + calc(20% + 10px) 接受面回归钉）。
        "width" | "text-indent" => fold_math_specified(v, MathFoldKind::Length),
        "transition-duration" | "transition-delay" | "animation-duration" | "animation-delay" => {
            fold_math_specified(v, MathFoldKind::Time)
        }
        // left 槽仅 sibling 树串化（`sibling-index( )` 空白归一；非 sibling
        // calc 形不折叠——inline store 原文行为零漂移，% 树布局依赖保留）。
        "left" => {
            let expr = parse_math_function(v)?;
            if calc_expr_has_tree_counting(&expr) {
                fold_tree_counting_calc(&expr, MathFoldKind::Length, v)
            } else {
                None
            }
        }
        // z-index：裸 sibling 函数形归一 + calc 包裹 sibling 树（单 opaque 叶
        // 解包 `calc(sibling-index())` → `sibling-index()` 语料钉面）；非
        // sibling 形不折叠（parse_z_index 通道原文往返）。
        "z-index" => {
            if let Some(canonical) = sibling_bare_fn(v) {
                return Some(canonical.to_string());
            }
            let expr = parse_math_function(v)?;
            if calc_expr_has_tree_counting(&expr) {
                fold_tree_counting_calc(&expr, MathFoldKind::Number, v)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// transform 列表 wrapper 折叠：rotate/rotatex/rotatey/rotatez/rotate3d 的角度
/// 实参、scale/scalex/scaley/scalez/scale3d 的 number 实参为数学函数时以
/// [`fold_math_specified`] 折叠替换，其余函数与原文大小写保留；无折叠发生返
/// `None`。skew 族未接线（FIXME：无驱动案，WPT serialize 页仅 rotate/scale 族）。
pub fn fold_transform_math_specified(value: &str) -> Option<String> {
    const ANGLE_FNS: [&str; 5] = ["rotate(", "rotatex(", "rotatey(", "rotatez(", "rotate3d("];
    // scale 族 number 槽（hypot-pow-sqrt-serialize `scale(pow(1,1))`、
    // signs-abs-serialize `scale(abs(1))` 案驱动）。
    const SCALE_FNS: [&str; 5] = ["scale(", "scalex(", "scaley(", "scalez(", "scale3d("];
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // 函数名 + '('；名前不得为 ident 字符（排除嵌套 ident 段误命中；
        // `scalex(`/`scaley(`/`scalez(`/`scale3d(` 起始不同，不命中 `scale(`）。
        let hit = ANGLE_FNS
            .iter()
            .map(|name| (*name, MathFoldKind::Angle))
            .chain(SCALE_FNS.iter().map(|name| (*name, MathFoldKind::Number)))
            .find(|(name, _)| {
                lower[i..].starts_with(name)
                    && (i == 0
                        || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_'))
            });
        let Some((name, kind)) = hit else {
            i += 1;
            continue;
        };
        let open = i + name.len();
        let mut depth = 1usize;
        let mut j = open;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            // 括号不平衡——非法面整体保留原文。
            return None;
        }
        // rotate3d(x y z angle)/scale3d(x, y, z) 多实参形逐 token 折叠替换
        //（rotate3d 仅末 token 为角度；scale3d 三 number token 各自折叠）；
        // 其余单参形整体折叠。
        let span = &lower[open..j - 1];
        let folded = if name == "rotate3d(" {
            let trimmed = span.trim_end();
            let tok_rel = trimmed.rfind([' ', ',']).map(|p| p + 1).unwrap_or(0);
            let tok = trimmed[tok_rel..].trim_start();
            let tok_start = open + tok_rel + (trimmed[tok_rel..].len() - tok.len());
            fold_math_specified(tok.trim(), MathFoldKind::Angle).map(|f| (tok_start, j - 1, f))
        } else if name == "scale3d(" {
            fold_scale3d_tokens(&lower, open, j - 1, &mut replacements);
            i = j;
            continue;
        } else {
            fold_math_specified(span.trim(), kind).map(|f| (open, j - 1, f))
        };
        if let Some(f) = folded {
            replacements.push(f);
        }
        i = j;
    }
    if replacements.is_empty() {
        return None;
    }
    // 仅替换折叠跨度，原文其余部分（含大小写）逐字节保留。
    let mut out = String::with_capacity(value.len());
    let mut last = 0;
    for (start, end, text) in replacements {
        out.push_str(&value[last..start]);
        out.push_str(&text);
        last = end;
    }
    out.push_str(&value[last..]);
    Some(out)
}

/// scale3d(sx, sy, sz) 逗号分隔三 number 实参逐 token 折叠（常量数学函数各以
/// [`fold_math_specified`]（Number）替换，大小写保留）。切分括号感知——实参
/// 内嵌 math 函数（`pow(1,1)`）的逗号不分裂 token。任意折叠未发生不产条目。
fn fold_scale3d_tokens(lower: &str, open: usize, close: usize, replacements: &mut Vec<(usize, usize, String)>) {
    let span = &lower[open..close];
    let mut tok_start = open;
    let mut depth = 0usize;
    for (idx, ch) in span.bytes().enumerate() {
        match ch {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => {
                fold_scale3d_token(lower, tok_start, open + idx, replacements);
                tok_start = open + idx + 1;
            }
            _ => {}
        }
    }
    fold_scale3d_token(lower, tok_start, close, replacements);
}

/// 折叠单个 token 跨度 [start, end)（Number 槽）。
fn fold_scale3d_token(lower: &str, start: usize, end: usize, replacements: &mut Vec<(usize, usize, String)>) {
    let tok = lower[start..end].trim();
    if let Some(f) = fold_math_specified(tok, MathFoldKind::Number) {
        let s = start + (lower[start..end].len() - lower[start..end].trim_start().len());
        replacements.push((s, end, f));
    }
}

/// CSSOM serialize-a-number 口径：最多 6 位小数、去尾零、`-0` 归 `0`；
/// NaN/Infinity 字面（Blink `calc(sin(infinity))` → `calc(NaN)` 驱动）。
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
    let mut s = format!("{:.6}", value);
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

/// 检测 asin/acos/atan（单参反三角）实参跨度内是否出现角度单位叶。
///
/// CSS Values 4 §10.5：asin/acos/atan 实参仅 `<number>`（结果才是 `<angle>`）。
/// CalcParser 把角度叶 parse 期 Number 化（`45deg` → 弧度 Number），型别信息
/// 丢失，须在字符串面拒绝——WPT acos-asin-atan-atan2-invalid
/// `rotate(atan(45deg ))` 驱动。`atan2(`/sin/cos/tan 实参容角度，不受此限。
pub(crate) fn inverse_trig_arg_has_angle_unit(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // 函数名 + '('；名前不得为 ident 字符（排除 `xasin(` 类嵌套 ident 段；
        // `atan2(` 因 '(' 不紧随 'atan' 天然不命中）。
        let hit = ["asin(", "acos(", "atan("].iter().find(|name| {
            lower[i..].starts_with(*name)
                && (i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_'))
        });
        let Some(name) = hit else {
            i += 1;
            continue;
        };
        // 平衡括号取实参跨度；只查**直系顶层叶**（嵌套括号组屏蔽）——
        // `asin(sin(100grad))` 直系实参为函数 `sin(...)`，内层 sin 容角度（§10.5
        // trig 实参 <angle>|<number>，结果 <number> 喂 asin 合法），整段扫会误拒
        // （WPT acos-asin-atan-atan2-computed used 面 6 案驱动）。内层反三角由
        // 下方线性扫描继续深入逐个 gate——`acos(asin(1deg) + 2)` 内层直系叶
        // 照样命中拒绝。
        let open = i + name.len();
        let mut depth = 1usize;
        let mut j = open;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            // 括号不平衡——上层 parse 整体拒绝，此处不误判放行。
            return false;
        }
        if direct_span_has_angle_unit(&lower[open..j - 1]) {
            return true;
        }
        // 不跳过整段：从函数名后继续扫，嵌套反三角 occurrence 各自 gate。
        i += name.len();
    }
    false
}

/// 直系顶层叶角度单位检测：屏蔽嵌套括号组后的平铺段逐段查
/// [`span_has_angle_unit`]——`asin(1deg + sin(1rad))` 平铺段 '1deg + sin' 命中；
/// `asin(sin(1deg))` 平铺段仅 'sin'（组内内容跳过，由外层线性扫描对内层
/// occurrence 重复 gate）。
fn direct_span_has_angle_unit(span: &str) -> bool {
    let bytes = span.as_bytes();
    let mut seg_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            // 平铺段为组前文（组内内容由外层线性扫描对内层 occurrence gate）。
            if span_has_angle_unit(&span[seg_start..i]) {
                return true;
            }
            let mut depth = 1usize;
            i += 1;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            seg_start = i;
        } else {
            i += 1;
        }
    }
    span_has_angle_unit(&span[seg_start..])
}

/// 跨度内是否出现 `数字 + 角度单位` token：单位前须紧邻数字/点，后须非
/// ident 字符——`45deg`/`3.5grad`/`1turn` 命中；`gradius`/`45radix` 不误报。
fn span_has_angle_unit(span: &str) -> bool {
    const UNITS: [&str; 4] = ["deg", "grad", "rad", "turn"];
    let bytes = span.as_bytes();
    for unit in UNITS {
        let mut from = 0;
        while let Some(pos) = span[from..].find(unit) {
            let start = from + pos;
            let end = start + unit.len();
            let prev_is_num = start > 0 && (bytes[start - 1].is_ascii_digit() || bytes[start - 1] == b'.');
            let next_is_ident =
                end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-' || bytes[end] == b'_');
            if prev_is_num && !next_is_ident {
                return true;
            }
            from = end;
        }
    }
    false
}

/// 跨度内 trig 实参**外**是否出现角度单位叶。
///
/// CSS Values 4 §10.5：sin/cos/tan 实参容 `<angle>`（产出 `<number>`）、
/// asin/acos/atan 实参仅 `<number>`、atan2 同型双实参——角度叶出现在 trig
/// 实参内不代表结果是 `<angle>`（`tan(45deg)` 产出 number，按度直读即错）。
/// 因此顶层判定须屏蔽 trig 调用实参跨度；非 trig 括号组照常扫入
///（`calc((45deg + 1))` 组内角度叶仍命中）。与 [`inverse_trig_arg_has_angle_unit`]
/// 同构：函数名前平铺段先行判定，命中即返回。
pub(crate) fn span_has_top_level_angle_unit(span: &str) -> bool {
    const TRIG: [&str; 7] = ["sin(", "cos(", "tan(", "asin(", "acos(", "atan(", "atan2("];
    let lower = span.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut seg_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        // 函数名 + '('；名前不得为 ident 字符（`xsin(` 不命中；`asin(`/`atan2(`
        // 起始字符不同，不命中 `sin(`/`tan(` 检测）。
        let hit = TRIG.iter().find(|name| {
            lower[i..].starts_with(*name)
                && (i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_'))
        });
        let Some(name) = hit else {
            i += 1;
            continue;
        };
        if span_has_angle_unit(&lower[seg_start..i]) {
            return true;
        }
        // 平衡括号取实参跨度整体屏蔽；不平衡时上层 parse 拒绝，此处不误判。
        let open = i + name.len();
        let mut depth = 1usize;
        let mut j = open + 1;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            return false;
        }
        seg_start = j;
        i = j;
    }
    span_has_angle_unit(&lower[seg_start..])
}

/// 跨度内是否出现产出 `<number>` 的三角函数调用（sin/cos/tan）。
///
/// `<angle>` 槽型别门用：式子含 sin/cos/tan 而无 `<angle>` 产出 provenance
///（反三角/atan2 或 [`span_has_top_level_angle_unit`]）时按度直读违反槽型别
///——`rotate(tan(45deg ))` invalid 案驱动（WPT sin-cos-tan-invalid）。
pub(crate) fn span_has_number_result_trig(span: &str) -> bool {
    const NUMBER_TRIG: [&str; 3] = ["sin(", "cos(", "tan("];
    let lower = span.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    for name in NUMBER_TRIG {
        let mut from = 0;
        while let Some(pos) = lower[from..].find(name) {
            let start = from + pos;
            let prev_ok = start == 0
                || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'-' || bytes[start - 1] == b'_');
            if prev_ok {
                return true;
            }
            from = start + name.len();
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_fold_sin_cos_tan_serialize_table() {
        // WPT sin-cos-tan-serialize.html 期望表（specified 层，type: number）。
        use MathFoldKind::Number;
        assert_eq!(fold_math_specified("cos(0)", Number).as_deref(), Some("calc(1)"));
        assert_eq!(fold_math_specified("sin(0)", Number).as_deref(), Some("calc(0)"));
        assert_eq!(fold_math_specified("tan(0)", Number).as_deref(), Some("calc(0)"));
        assert_eq!(fold_math_specified("calc(cos(0))", Number).as_deref(), Some("calc(1)"));
        assert_eq!(
            fold_math_specified("calc(sin(0) + 0.5)", Number).as_deref(),
            Some("calc(0.5)")
        );
        assert_eq!(
            fold_math_specified("calc(sin(0) + cos(0) + tan(0))", Number).as_deref(),
            Some("calc(1)")
        );
        assert_eq!(fold_math_specified("sin(30deg)", Number).as_deref(), Some("calc(0.5)"));
        assert_eq!(fold_math_specified("cos(60deg)", Number).as_deref(), Some("calc(0.5)"));
        assert_eq!(fold_math_specified("tan(45deg)", Number).as_deref(), Some("calc(1)"));
        assert_eq!(fold_math_specified("cos(180deg)", Number).as_deref(), Some("calc(-1)"));
        assert_eq!(fold_math_specified("sin(270deg)", Number).as_deref(), Some("calc(-1)"));
        assert_eq!(fold_math_specified("sin(-180deg)", Number).as_deref(), Some("calc(0)"));
        assert_eq!(
            fold_math_specified("calc(sin(infinity))", Number).as_deref(),
            Some("calc(NaN)")
        );
        assert_eq!(
            fold_math_specified("calc(cos(NaN))", Number).as_deref(),
            Some("calc(NaN)")
        );
    }

    #[test]
    fn angle_fold_acos_serialize_table() {
        // WPT acos-asin-atan-atan2-serialize.html 期望表（transform: rotate(...)）。
        use MathFoldKind::Angle;
        assert_eq!(fold_math_specified("acos(1)", Angle).as_deref(), Some("calc(0deg)"));
        assert_eq!(fold_math_specified("acos(-1)", Angle).as_deref(), Some("calc(180deg)"));
        assert_eq!(fold_math_specified("acos(0)", Angle).as_deref(), Some("calc(90deg)"));
        assert_eq!(fold_math_specified("acos(0.5)", Angle).as_deref(), Some("calc(60deg)"));
        assert_eq!(fold_math_specified("asin(0.5)", Angle).as_deref(), Some("calc(30deg)"));
        assert_eq!(fold_math_specified("asin(1)", Angle).as_deref(), Some("calc(90deg)"));
        assert_eq!(fold_math_specified("asin(-1)", Angle).as_deref(), Some("calc(-90deg)"));
        assert_eq!(fold_math_specified("atan(1)", Angle).as_deref(), Some("calc(45deg)"));
        assert_eq!(fold_math_specified("atan(0)", Angle).as_deref(), Some("calc(0deg)"));
        assert_eq!(
            fold_math_specified("atan(infinity)", Angle).as_deref(),
            Some("calc(90deg)")
        );
        assert_eq!(
            fold_math_specified("atan2(37.320508075, 10)", Angle).as_deref(),
            Some("calc(75deg)")
        );
        assert_eq!(
            fold_math_specified("atan2(infinity, infinity)", Angle).as_deref(),
            Some("calc(45deg)")
        );
        assert_eq!(
            fold_math_specified("atan2(-infinity, -infinity)", Angle).as_deref(),
            Some("calc(-135deg)")
        );
        // NaN 域外（§10.10 合法）：`calc(NaN * 1deg)`。
        assert_eq!(
            fold_math_specified("acos(1.5)", Angle).as_deref(),
            Some("calc(NaN * 1deg)")
        );
        assert_eq!(
            fold_math_specified("asin(-1.5)", Angle).as_deref(),
            Some("calc(NaN * 1deg)")
        );
        // pi 常量组合。
        assert_eq!(
            fold_math_specified("asin(pi - pi + 1)", Angle).as_deref(),
            Some("calc(90deg)")
        );
    }

    #[test]
    fn fold_rejects_non_math_and_type_mismatch() {
        use MathFoldKind::{Angle, Number};
        // 非数学函数原样保留（None）。
        assert_eq!(fold_math_specified("45deg", Number), None);
        assert_eq!(fold_math_specified("rotate(45deg)", Angle), None);
        assert_eq!(fold_math_specified("none", Number), None);
        // <length> 叶混入：型别不符拒折。
        assert_eq!(fold_math_specified("calc(1px + 2px)", Number), None);
        assert_eq!(fold_math_specified("asin(90px)", Angle), None);
        // <number> 位不容角度产出式（结果为 <angle>）。
        assert_eq!(fold_math_specified("acos(1)", Number), None);
        // 角度位 ±∞ canonical 发射（§10.10 无穷合法非 invalid——
        // calc-infinity-nan-serialize-angle ±∞ 行驱动，旧拒折口径已翻案）。
        assert_eq!(
            fold_math_specified("calc(infinity)", Angle).as_deref(),
            Some("calc(infinity * 1deg)")
        );
    }

    #[test]
    fn inverse_trig_angle_unit_gate_scopes() {
        assert!(inverse_trig_arg_has_angle_unit("atan(45deg )"));
        assert!(inverse_trig_arg_has_angle_unit("asin(30deg + 1rad)"));
        assert!(inverse_trig_arg_has_angle_unit("calc(acos(45deg))"));
        assert!(inverse_trig_arg_has_angle_unit("ACOS(1turn)"));
        // sin/cos/tan 与 atan2 实参容角度——不命中。
        assert!(!inverse_trig_arg_has_angle_unit("sin(30deg)"));
        assert!(!inverse_trig_arg_has_angle_unit("atan2(30deg, 1rad)"));
        assert!(!inverse_trig_arg_has_angle_unit("tan(1turn)"));
        // ident 边界不误报：gradius/45radix/嵌套 ident 段。
        assert!(!inverse_trig_arg_has_angle_unit("asin(gradius)"));
        assert!(!inverse_trig_arg_has_angle_unit("asin(45radix)"));
        // atan2( 不命中 atan( 名扫描。
        assert!(!inverse_trig_arg_has_angle_unit("atan2(1dag, 2deg)"));
        // 括号不平衡不误判。
        assert!(!inverse_trig_arg_has_angle_unit("asin(45deg"));
        // 嵌套非反三角内部角度不命中直系叶（WPT computed used 面 valid 形）。
        assert!(!inverse_trig_arg_has_angle_unit("asin(sin(100grad))"));
        assert!(!inverse_trig_arg_has_angle_unit("atan(tan(30deg + 0.261799rad))"));
        assert!(!inverse_trig_arg_has_angle_unit("asin(sin(0.25turn))"));
        // 嵌套反三角的直系叶照样命中（线性扫描深入逐个 gate）。
        assert!(inverse_trig_arg_has_angle_unit("acos(asin(1deg) + 2)"));
        assert!(inverse_trig_arg_has_angle_unit("asin(atan(45deg))"));
    }

    #[test]
    fn span_top_level_angle_unit_masks_trig_args() {
        // 顶层角度叶命中。
        assert!(span_has_top_level_angle_unit("calc(180deg)"));
        assert!(span_has_top_level_angle_unit("calc(45deg + sin(30deg))"));
        assert!(span_has_top_level_angle_unit("calc((45deg + 1))"));
        assert!(span_has_top_level_angle_unit("90grad"));
        assert!(span_has_top_level_angle_unit("CALC(180DEG)"));
        // trig 实参内角度叶屏蔽（§10.5 trig 实参容角度、产出 <number>）。
        assert!(!span_has_top_level_angle_unit("tan(45deg)"));
        assert!(!span_has_top_level_angle_unit("TAN(45DEG)"));
        assert!(!span_has_top_level_angle_unit("sin(cos(45deg))"));
        assert!(!span_has_top_level_angle_unit("calc(sin(30deg) + tan(45deg))"));
        assert!(!span_has_top_level_angle_unit("atan2(30deg, 1rad)"));
        // ident 边界：xsin( 非 trig 调用，内部照扫。
        assert!(span_has_top_level_angle_unit("xsin(45deg)"));
        // 括号不平衡不误判。
        assert!(!span_has_top_level_angle_unit("tan(45deg"));
    }

    #[test]
    fn span_number_result_trig_detection() {
        assert!(span_has_number_result_trig("tan(45deg )"));
        assert!(span_has_number_result_trig("calc(sin(0))"));
        assert!(span_has_number_result_trig("cos(0deg)"));
        // asin/acos/atan/atan2 产出 <angle>，非 number-result trig；
        // "asin(" 不命中 "sin(" 子串（ident 边界）。
        assert!(!span_has_number_result_trig("asin(0.5)"));
        assert!(!span_has_number_result_trig("acos(1)"));
        assert!(!span_has_number_result_trig("atan(1)"));
        assert!(!span_has_number_result_trig("atan2(1, 1)"));
        assert!(!span_has_number_result_trig("xsin(0)"));
    }

    #[test]
    fn format_number_6dp_rounding() {
        assert_eq!(format_number_6dp(1.0), "1");
        assert_eq!(format_number_6dp(0.5), "0.5");
        assert_eq!(format_number_6dp(0.49999999999999994), "0.5");
        assert_eq!(format_number_6dp(100.0), "100");
        assert_eq!(format_number_6dp(-0.0), "0");
        assert_eq!(format_number_6dp(-1.0), "-1");
        assert_eq!(format_number_6dp(f64::NAN), "NaN");
        assert_eq!(format_number_6dp(f64::INFINITY), "infinity");
        assert_eq!(format_number_6dp(f64::NEG_INFINITY), "-infinity");
    }

    #[test]
    fn fold_decl_dispatch_by_property() {
        // WPT serialize 页驱动形态：transform: rotate(...)、scale: cos(0)。
        assert_eq!(
            fold_decl_specified("transform", "rotate(acos(1))").as_deref(),
            Some("rotate(calc(0deg))")
        );
        // 平角度叶往返恒等（WPT serialize 页对期望值双侧同形断言关键——
        // parse 期 Number 化丢单位标记，含单位叶须按弧度→度还原）。
        assert_eq!(
            fold_decl_specified("transform", "rotate(calc(180deg))").as_deref(),
            Some("rotate(calc(180deg))")
        );
        assert_eq!(
            fold_decl_specified("transform", "rotate(calc(90deg))").as_deref(),
            Some("rotate(calc(90deg))")
        );
        assert_eq!(
            fold_decl_specified("rotate", "calc(180deg)").as_deref(),
            Some("calc(180deg)")
        );
        assert_eq!(fold_decl_specified("scale", "cos(0)").as_deref(), Some("calc(1)"));
        assert_eq!(
            fold_decl_specified("rotate", "acos(-1)").as_deref(),
            Some("calc(180deg)")
        );
        // opacity 裸数学函数折叠（hypot-pow-sqrt-serialize `calc(pow(1,1) - 0.5)`
        // → `calc(0.5)` 驱动）；普通值/非数学原样。
        assert_eq!(
            fold_decl_specified("opacity", "calc(pow(1,1) - 0.5)").as_deref(),
            Some("calc(0.5)")
        );
        assert_eq!(fold_decl_specified("opacity", "abs(1)").as_deref(), Some("calc(1)"));
        assert_eq!(fold_decl_specified("opacity", "0.5"), None);
        assert_eq!(fold_decl_specified("opacity", "calc(1px + 2px)"), None);
        // 白名单外属性 / 非数学值 / 空值保留原文；width 已接线（g12 刀2
        // calc-infinity-nan-serialize-length 桶），纯 px 式折叠 canonical 形。
        assert_eq!(
            fold_decl_specified("width", "calc(1px + 2px)").as_deref(),
            Some("calc(3px)")
        );
        assert_eq!(fold_decl_specified("transform", "rotate(45deg) scale(2)"), None);
        assert_eq!(fold_decl_specified("scale", "2"), None);
        assert_eq!(fold_decl_specified("scale", ""), None);
    }

    #[test]
    fn fold_transform_wrapper_replacements() {
        // 混合列表：仅折叠 rotate 族跨度，其余原样（大小写保留）。
        assert_eq!(
            fold_transform_math_specified("rotate(acos(1)) scale(2)"),
            Some("rotate(calc(0deg)) scale(2)".into())
        );
        assert_eq!(
            fold_transform_math_specified("Rotate(ASIN(0.5))"),
            Some("Rotate(calc(30deg))".into())
        );
        assert_eq!(
            fold_transform_math_specified("rotate3d(1, 0, 0, acos(1))"),
            Some("rotate3d(1, 0, 0, calc(0deg))".into())
        );
        assert_eq!(
            fold_transform_math_specified("rotate(acos(1)) rotate(acos(0))"),
            Some("rotate(calc(0deg)) rotate(calc(90deg))".into())
        );
        // 无折叠点 / 括号不平衡 → None（保留原文）。
        assert_eq!(fold_transform_math_specified("scale(2) translateX(10px)"), None);
        assert_eq!(fold_transform_math_specified("rotate(acos(1)"), None);
    }
}
