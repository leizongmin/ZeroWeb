//! CSS `if()` 替换函数（css-values-5 §if-notation，k111 刀）。
//!
//! 四件：调用形定位与结构解析（S1，tokenizer-lite——字符串字面/嵌套括号
//! 感知）、style() 条件求值（S2——自订属性 token 级等值＋bare truthiness
//! ＋关系符 `< > <= >=` 数值比较＋`not/and/or` 布尔代数）、支路选取与
//! 代入（S3——已取支路走 var/attr/嵌套 if 共站代入；random 由站调用方
//! 整文本预代入，序号跨条件/支路同源序空间）、cycle 六规则（S4——R1 达
//! 被求值声明的环 → 整声明 IACVT；R2 局部环 → 条件 false；R3 invalid
//! 操作数 → 比较 false；R4 first-match-wins 未取支路不求值）。
//!
//! S5 常规属性门在站调用方（lib.rs 4.76 站）：if() 只在自订属性声明
//! 代入，常规属性保持裸文本（下游 IACVT＝现行为，FIXME 随全值域刀放开）。
//! 关系符数值域仅纯 `<number>`（长度/百分比 FIXME）；`initial` 操作数、
//! `supports()/media()` 条件不支持 → 条件 false（FIXME）。
//!
//! <https://drafts.csswg.org/css-values-5/#if-notation>

use std::collections::HashMap;

use zero_css_parser::values::{
    RandomEvalContext, has_attr_word, substitute_attr_values_checked, substitute_random_values,
};

use crate::computed::{OperandRead, VarTextErr, resolve_custom_operand, substitute_var_text_attributed};
use crate::decl_value_has_random;

/// 嵌套 if() 求值深度上限（互引用嵌套链 fail-closed——未取支路不求值前
/// 提下深度只来自条件/支路文本内嵌 if()）。
const MAX_EVAL_DEPTH: usize = 8;

/// 操作数代入不动点上限（var→attr→random→嵌套 if 迭代；残替换函数
/// fail-closed）。
const MAX_OPERAND_PASSES: usize = 4;

/// 站求值结果：`Unchanged`＝无可代入调用（含解析失败保原文——下游 IACVT
/// ＝现行为）；`Substituted`＝全部调用已支路化；`Iacvt`＝整声明 invalid
/// at computed-value time（cycle 达被求值声明／无命中支路／支路代入
/// invalid）。
#[derive(Debug)]
pub(crate) enum IfOutcome {
    Unchanged,
    Substituted(String),
    Iacvt,
}

/// 站求值上下文（lib.rs 4.76 站构造——元素视图）。
pub(crate) struct IfEvalCtx<'a> {
    /// 元素 gather 自订属性表（裸文本；被求值声明自身在表中存 if() 原文，
    /// cycle 由解析栈预置 `under_eval` 归因）。
    pub customs: &'a HashMap<String, String>,
    /// 元素属性读取器（attr() 条件操作数）。
    pub attr_reader: &'a dyn Fn(&str) -> Option<String>,
    /// random 求值语境（§9.4 element seed——条件内 random 直调求值）。
    pub random_ctx: &'a RandomEvalContext,
}

/// 值文本是否含 `if(` 调用形（ASCII ci；ident 边界感知——`notify(` 非
/// if 调用）。站前 any() 预滤门；字符串字面内形似体由站内字符串感知
/// 解析天然跳过。
pub(crate) fn has_if_call(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    for k in 0..bytes.len().saturating_sub(2) {
        if &bytes[k..k + 3] == b"if(" {
            // ident 边界：`notify(` 的 if( 前是 ident 字符 → 非独立函数名。
            if k > 0 && is_ident_byte(bytes[k - 1]) {
                continue;
            }
            return true;
        }
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

/// 条件求值结果。`CycleUnder`＝求值命中被求值声明（R1 → 整声明 IACVT）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cond {
    True,
    False,
    CycleUnder,
}

/// 操作数代入错误：`UnderEval`＝代入命中被求值声明（支路/操作数文本 →
/// R1 整声明 IACVT）；`Invalid`＝未定义无回退/局部环/残替换函数
///（支路 → 整声明 IACVT；条件操作数 → 条件 false）。
enum OperandErr {
    UnderEval,
    Invalid,
}

/// 解析 if() 实参部为条件/支路子句集。段以顶层 `;` 分隔；段内首个顶层
/// `:` 分隔条件与支路；条件为 `else`（ASCII ci）→ else 支路。形不全
///（缺 `:`、空条件、多重 else）→ None（调用保原文）。
struct IfClauses {
    cond_branches: Vec<(String, String)>,
    else_branch: Option<String>,
}

/// 定位 `rest` 中首个可解析 if() 调用：返回（函数名起点，实参部内容，
/// 实参部闭括号后一字节索引）。跳过字符串字面量；括号深度跨 `()[]{}`。
fn locate_if_call(rest: &str) -> Option<(usize, &str, usize)> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i + 3 <= bytes.len() {
        // 字符串字面跳越（引号体不识函数名）。
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            let mut j = i + 1;
            while j < bytes.len() {
                if bytes[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if bytes[j] == q {
                    break;
                }
                j += 1;
            }
            i = j + 1;
            continue;
        }
        if bytes[i..].len() >= 3
            && bytes[i..i + 3].eq_ignore_ascii_case(b"if(")
            && (i == 0 || !is_ident_byte(bytes[i - 1]))
        {
            // 深度扫描匹配闭括号（()[]{}`` 计数＋字符串跳越）。
            let mut depth = 1usize;
            let mut j = i + 3;
            while j < bytes.len() {
                match bytes[j] {
                    b'"' | b'\'' => {
                        let q = bytes[j];
                        let mut k = j + 1;
                        while k < bytes.len() {
                            if bytes[k] == b'\\' {
                                k += 2;
                                continue;
                            }
                            if bytes[k] == q {
                                break;
                            }
                            k += 1;
                        }
                        j = k + 1;
                        continue;
                    }
                    b'(' | b'[' | b'{' => depth += 1,
                    b')' | b']' | b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            let args = &rest[i + 3..j];
                            let close_end = j + 1;
                            // 调用名前缀完整性：`if(` 前不得是函数 token 的一部分
                            //（已查 ident 边界），后随不得是 `=` 类 token 序列。
                            return Some((i, args, close_end));
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            // 未闭合 → 按 EOF 恢复：跳过该 `if(` 继续（同 attr 站口径）。
            i += 3;
            continue;
        }
        i += 1;
    }
    None
}

/// 解析实参部。段切分复用 css-parser 顶层分号单源（字符串字面/嵌套
/// 感知）；段内顶层 `:` 扫描（find_top_level_char）。
fn parse_if_args(args: &str) -> Option<IfClauses> {
    let mut cond_branches: Vec<(String, String)> = Vec::new();
    let mut else_branch: Option<String> = None;
    for seg in zero_css_parser::parser::split_top_level_semicolons(args) {
        let seg_trim = seg.trim();
        if seg_trim.is_empty() {
            continue;
        }
        let colon = find_top_level_char(seg, b':')?;
        let cond = seg[..colon].trim();
        let branch = seg[colon + 1..].trim();
        if cond.is_empty() {
            return None;
        }
        if cond.eq_ignore_ascii_case("else") {
            // else 必须唯一且殿后（多重 else → 形不全保原文）。
            if else_branch.is_some() {
                return None;
            }
            else_branch = Some(branch.to_string());
        } else {
            cond_branches.push((cond.to_string(), branch.to_string()));
        }
    }
    if cond_branches.is_empty() && else_branch.is_none() {
        return None;
    }
    Some(IfClauses {
        cond_branches,
        else_branch,
    })
}

/// 顶层单字符扫描（深度计数 ()[]{}＋字符串跳越）。
fn find_top_level_char(s: &str, target: u8) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i];
                let mut j = i + 1;
                while j < bytes.len() {
                    if bytes[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if bytes[j] == q {
                        break;
                    }
                    j += 1;
                }
                i = j + 1;
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            c if c == target && depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// 条件求值入口（style() 实参整体）。
fn eval_condition(text: &str, prop: &str, ctx: &IfEvalCtx, depth: usize) -> Cond {
    eval_style_query(text.trim(), prop, ctx, depth)
}

/// 布尔层：括号剥离 → not → or → and → style-test。
fn eval_style_query(text: &str, prop: &str, ctx: &IfEvalCtx, depth: usize) -> Cond {
    let t = text.trim();
    if t.is_empty() {
        return Cond::False;
    }
    // 整体括号包裹 → 剥一层递归。
    if t.starts_with('(') {
        let bytes = t.as_bytes();
        let mut depth_c = 0usize;
        let mut close = None;
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'"' | b'\'' => {
                    let q = bytes[i];
                    let mut j = i + 1;
                    while j < bytes.len() {
                        if bytes[j] == b'\\' {
                            j += 2;
                            continue;
                        }
                        if bytes[j] == q {
                            break;
                        }
                        j += 1;
                    }
                    i = j + 1;
                    continue;
                }
                b'(' => depth_c += 1,
                b')' => {
                    depth_c -= 1;
                    if depth_c == 0 {
                        close = Some(i);
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if let Some(close) = close
            && close == t.len() - 1
        {
            return eval_style_query(&t[1..close], prop, ctx, depth);
        }
    }
    // not（ident 边界——`nothing` 非 not 前缀）。
    if let Some(rest) = strip_boolean_word(t, "not") {
        let inner = eval_style_query(rest, prop, ctx, depth);
        // 环归因穿透 not：R1 整声明失效不随布尔取反消解。
        return match inner {
            Cond::True => Cond::False,
            Cond::False => Cond::True,
            Cond::CycleUnder => Cond::CycleUnder,
        };
    }
    // or（最低优先）。环归因支配布尔结果（R1）：任一操作数环达被求值
    // 声明 → 整条件 CycleUnder（全操作数求值不短路——若短路于 True，
    // 环操作数的失效语义被吞，if-cycle #10 判例）。
    if let Some(parts) = split_boolean_word(t, "or") {
        let mut saw_cycle = false;
        let mut saw_true = false;
        for p in parts {
            match eval_style_query(p, prop, ctx, depth) {
                Cond::True => saw_true = true,
                Cond::CycleUnder => saw_cycle = true,
                Cond::False => {}
            }
        }
        return if saw_cycle {
            Cond::CycleUnder
        } else if saw_true {
            Cond::True
        } else {
            Cond::False
        };
    }
    // and。环归因同 or 支配（全操作数求值；False 只在无环时短路终局）。
    if let Some(parts) = split_boolean_word(t, "and") {
        let mut saw_cycle = false;
        let mut saw_false = false;
        for p in parts {
            match eval_style_query(p, prop, ctx, depth) {
                Cond::False => saw_false = true,
                Cond::CycleUnder => saw_cycle = true,
                Cond::True => {}
            }
        }
        return if saw_cycle {
            Cond::CycleUnder
        } else if saw_false {
            Cond::False
        } else {
            Cond::True
        };
    }
    eval_style_test(t, prop, ctx, depth)
}

/// 布尔连接词剥离（顶层 ident 边界感知；返回首词后的余部）。
fn strip_boolean_word<'a>(t: &'a str, word: &str) -> Option<&'a str> {
    let lower = t.to_ascii_lowercase();
    if !lower.starts_with(word) {
        return None;
    }
    let wb = word.len();
    let rest = t[wb..].trim_start();
    let rb = rest.as_bytes();
    if rb.is_empty() || is_ident_byte(rb[0]) {
        return None;
    }
    Some(rest)
}

/// 顶层连接词切分（or/and；ident 边界＋字符串跳越——复用顶层扫描逐字
/// 判定）。
fn split_boolean_word<'a>(t: &'a str, word: &str) -> Option<Vec<&'a str>> {
    let lower = t.to_ascii_lowercase();
    let wb = word.as_bytes();
    let bytes = lower.as_bytes();
    let mut hits: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + wb.len() <= bytes.len() {
        if &bytes[i..i + wb.len()] == wb {
            let prev_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
            let next_ok = i + wb.len() >= bytes.len() || !is_ident_byte(bytes[i + wb.len()]);
            if prev_ok && next_ok && top_level_span(t, i, i + wb.len()) {
                hits.push(i);
            }
            i += wb.len();
            continue;
        }
        i += 1;
    }
    if hits.is_empty() {
        return None;
    }
    let mut parts = Vec::with_capacity(hits.len() + 1);
    let mut start = 0;
    for h in hits {
        parts.push(t[start..h].trim());
        start = h + wb.len();
    }
    parts.push(t[start..].trim());
    Some(parts)
}

/// 顶层跨度判定（span 不穿任何括号/字符串——用于 or/and 词切分）。
fn top_level_span(s: &str, from: usize, to: usize) -> bool {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i];
                let mut j = i + 1;
                while j < bytes.len() {
                    if bytes[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if bytes[j] == q {
                        break;
                    }
                    j += 1;
                }
                i = j + 1;
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {
                if depth == 0 && i >= from && i < to {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// style-test 分发：字面 true/false → 关系符 → 冒号等值 → bare truthiness
/// → 不支持（False，FIXME：initial 操作数/supports()/media()/标准属性
/// 查询）。
fn eval_style_test(t: &str, prop: &str, ctx: &IfEvalCtx, depth: usize) -> Cond {
    if t.eq_ignore_ascii_case("true") {
        return Cond::True;
    }
    if t.eq_ignore_ascii_case("false") {
        return Cond::False;
    }
    // 条件函数原子：style() 剥包裹递归内层查询；supports()/media() 本轮
    // 不支持 → false（FIXME：supports()/media() 条件域）。
    let lower = t.to_ascii_lowercase();
    for (prefix, supported) in [("style(", true), ("supports(", false), ("media(", false)] {
        if lower.starts_with(prefix) && wraps_whole(t, prefix.len()) {
            if !supported {
                return Cond::False;
            }
            return eval_style_query(&t[prefix.len()..t.len() - 1], prop, ctx, depth);
        }
    }
    // 冒号等值形（顶层冒号）。
    if let Some(colon) = find_top_level_char(t, b':') {
        let left = t[..colon].trim();
        let right = t[colon + 1..].trim();
        if !left.starts_with("--") {
            // 标准属性查询/其他 unsupported → false。
            return Cond::False;
        }
        let left_val = match read_custom(left, prop, ctx) {
            OperandRead::Resolved(v) => v,
            OperandRead::Invalid => return Cond::False,
            OperandRead::CycleOn => return Cond::CycleUnder,
        };
        let right_val = match sub_operand(right, prop, ctx, depth) {
            Ok(v) => v,
            Err(OperandErr::UnderEval) => return Cond::CycleUnder,
            Err(OperandErr::Invalid) => return Cond::False,
        };
        return if text_eq(&left_val, &right_val) {
            Cond::True
        } else {
            Cond::False
        };
    }
    // 关系符（顶层首现，两字长优先）。
    if let Some((lop, rop, op)) = split_relational(t) {
        let l = resolve_numeric_operand(lop, prop, ctx, depth);
        let r = resolve_numeric_operand(rop, prop, ctx, depth);
        let (lv, rv) = match (l, r) {
            (Ok(a), Ok(b)) => (a, b),
            (Err(NumErr::UnderEval), _) | (_, Err(NumErr::UnderEval)) => return Cond::CycleUnder,
            (Err(NumErr::Invalid), _) | (_, Err(NumErr::Invalid)) => return Cond::False,
        };
        let cmp = match op {
            b"LE" => lv <= rv,
            b"GE" => lv >= rv,
            b"LT" => lv < rv,
            b"GT" => lv > rv,
            _ => (lv - rv).abs() < f64::EPSILON,
        };
        return if cmp { Cond::True } else { Cond::False };
    }
    // bare truthiness（自订属性名）：Resolved→true；Invalid（未定义/
    // 局部环，R2/R3）→false；CycleOn（环归因达被求值声明，R1）→CycleUnder。
    if t.starts_with("--") && is_bare_custom_name(t) {
        return match read_custom(t, prop, ctx) {
            OperandRead::Resolved(_) => Cond::True,
            OperandRead::Invalid => Cond::False,
            OperandRead::CycleOn => Cond::CycleUnder,
        };
    }
    Cond::False
}

/// bare 自订属性名判定（`--` + 非空 ident 尾，无空白/运算符）。
fn is_bare_custom_name(t: &str) -> bool {
    let b = t.as_bytes();
    b.len() > 2 && b[1] == b'-' && b[2..].iter().all(|c| is_ident_byte(*c))
}

/// 判定 `t` 中 `open` 位（闭前缀后）开括号是否包裹整段（匹配闭括号在
/// 末位——style()/supports()/media() 原子剥离判据）。
fn wraps_whole(t: &str, open: usize) -> bool {
    let bytes = t.as_bytes();
    if open == 0 || open > bytes.len() || bytes[open - 1] != b'(' {
        return false;
    }
    let mut depth = 0usize;
    let mut i = open - 1;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i];
                let mut j = i + 1;
                while j < bytes.len() {
                    if bytes[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if bytes[j] == q {
                        break;
                    }
                    j += 1;
                }
                i = j + 1;
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return i == bytes.len() - 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// 关系符切分（顶层首现；两字长 `<=`/`>=` 优先于单字）。返回
///（左操作数，右操作数，算子标记）。
fn split_relational(t: &str) -> Option<(&str, &str, &'static [u8; 2])> {
    let bytes = t.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i];
                let mut j = i + 1;
                while j < bytes.len() {
                    if bytes[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if bytes[j] == q {
                        break;
                    }
                    j += 1;
                }
                i = j + 1;
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b'<' | b'>' | b'=' if depth == 0 => {
                let (op, adv): (&'static [u8; 2], usize) = match bytes[i] {
                    b'<' if i + 1 < bytes.len() && bytes[i + 1] == b'=' => (b"LE", 2),
                    b'>' if i + 1 < bytes.len() && bytes[i + 1] == b'=' => (b"GE", 2),
                    b'<' => (b"LT", 1),
                    b'>' => (b"GT", 1),
                    _ => (b"EQ", 1),
                };
                return Some((&t[..i], &t[i + adv..], op));
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// 关系操作数数值解析：代入后纯 `<number>` 文本（`f64` 解析；单位尾/
/// 非数 → Invalid，FIXME 长度/百分比域）。
enum NumErr {
    UnderEval,
    Invalid,
}

fn resolve_numeric_operand(text: &str, prop: &str, ctx: &IfEvalCtx, depth: usize) -> Result<f64, NumErr> {
    // bare 自订属性名操作数（语料形 `--condition-val >= attr(...)`) → 直读
    // 计算值再解析（文本代入对裸名是恒等——须走归因读取通道）。
    let bare = text.trim();
    let sub = if bare.starts_with("--") && is_bare_custom_name(bare) {
        match read_custom(bare, prop, ctx) {
            OperandRead::Resolved(v) => v,
            OperandRead::Invalid => return Err(NumErr::Invalid),
            OperandRead::CycleOn => return Err(NumErr::UnderEval),
        }
    } else {
        match sub_operand(text, prop, ctx, depth) {
            Ok(v) => v,
            Err(OperandErr::UnderEval) => return Err(NumErr::UnderEval),
            Err(OperandErr::Invalid) => return Err(NumErr::Invalid),
        }
    };
    let t = sub.trim();
    // 数值形边界：整段须是可选符号＋数字（含小数/指数）——拒绝 `3px`/
    // `foo3` 混合形（单位域 FIXME）。
    let ok = !t.is_empty() && t.parse::<f64>().map(|v| v.is_finite()).unwrap_or(false);
    match ok {
        true => Ok(t.parse::<f64>().unwrap_or(f64::NAN)),
        false => Err(NumErr::Invalid),
    }
}

/// 操作数/支路文本代入不动点：嵌套 if → attr → var → random（var 环
/// 归因锚被求值声明）。attr 先于 var：属性在场时调用即被吃掉、回退
/// 参数永不求值（css-values-5 §attr-notation；if-cycle #29 判例——
/// var 通道先跑会在回退 `var(--y)` 自环处盲代 → 假 Invalid）。attr
/// None（属性缺席/含 var 延迟形）不早退：var 代入后下一轮重试，循环
/// 末残检 fail-closed。
fn sub_operand(text: &str, prop: &str, ctx: &IfEvalCtx, depth: usize) -> Result<String, OperandErr> {
    let mut cur = text.to_string();
    for _ in 0..MAX_OPERAND_PASSES {
        let mut changed = false;
        if has_if_call(&cur) {
            match substitute_if_calls(&cur, prop, ctx, depth + 1) {
                IfOutcome::Iacvt | IfOutcome::Unchanged => return Err(OperandErr::Invalid),
                IfOutcome::Substituted(s) => {
                    cur = s;
                    changed = true;
                }
            }
        }
        if has_attr_word(&cur) {
            match substitute_attr_values_checked(&cur, ctx.attr_reader) {
                // IACVT / 文法 invalid / 延迟残 attr → 不早退，var 代入
                // 后重试（残检兜底）。
                None => {}
                Some((std::borrow::Cow::Owned(o), _tainted)) => {
                    cur = o;
                    changed = true;
                }
                Some((std::borrow::Cow::Borrowed(_), _)) => {}
            }
        }
        if cur.to_ascii_lowercase().contains("var(") {
            match substitute_var_text_attributed(&cur, prop, ctx.customs) {
                Ok(s) => {
                    if s != cur {
                        cur = s;
                        changed = true;
                    }
                }
                Err(VarTextErr::UnderEval) => return Err(OperandErr::UnderEval),
                Err(VarTextErr::Invalid) => return Err(OperandErr::Invalid),
            }
        }
        if decl_value_has_random(&cur) {
            // fail-closed 保原文——下游数值解析/文本比较自然落 False。
            if let Some(std::borrow::Cow::Owned(o)) = substitute_random_values(&cur, prop, ctx.random_ctx) {
                cur = o;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let lower = cur.to_ascii_lowercase();
    if lower.contains("var(") || has_attr_word(&cur) || has_if_call(&cur) {
        return Err(OperandErr::Invalid);
    }
    Ok(cur)
}

/// 自订属性读取（条件左操作数/truthiness）：环归因锚被求值声明
///（computed::resolve_custom_operand——R1/R2/R3 归因通道）。
fn read_custom(name: &str, prop: &str, ctx: &IfEvalCtx) -> OperandRead {
    resolve_custom_operand(name, prop, ctx.customs)
}

/// token 级文本等值：空白折叠归一后字节等值（自订属性值大小写敏感）。
fn text_eq(a: &str, b: &str) -> bool {
    fn norm(s: &str) -> String {
        s.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
    }
    norm(a) == norm(b)
}

/// if() 替换站主入口：值内全部可解析 if() 调用支路化。
pub(crate) fn substitute_if_calls(value: &str, prop: &str, ctx: &IfEvalCtx, depth: usize) -> IfOutcome {
    if depth > MAX_EVAL_DEPTH {
        return IfOutcome::Iacvt;
    }
    let mut result = String::new();
    let mut rest = value;
    let mut any = false;
    loop {
        match locate_if_call(rest) {
            None => {
                result.push_str(rest);
                break;
            }
            Some((fn_start, args, close_end)) => {
                result.push_str(&rest[..fn_start]);
                match parse_if_args(args) {
                    None => {
                        // 形不全 → 调用保原文（下游 IACVT＝现行为），扫描越闭括号续。
                        result.push_str(&rest[fn_start..close_end]);
                        rest = &rest[close_end..];
                    }
                    Some(clauses) => {
                        let mut taken: Option<&str> = None;
                        let mut cycle = false;
                        for (cond_text, branch) in &clauses.cond_branches {
                            match eval_condition(cond_text, prop, ctx, depth) {
                                Cond::True => {
                                    taken = Some(branch.as_str());
                                    break;
                                }
                                Cond::False => {}
                                Cond::CycleUnder => {
                                    cycle = true;
                                    break;
                                }
                            }
                        }
                        if cycle {
                            return IfOutcome::Iacvt;
                        }
                        let branch = taken.or(clauses.else_branch.as_deref());
                        match branch {
                            // 无命中支路且无 else → IACVT（css-values-5 §if-notation）。
                            None => return IfOutcome::Iacvt,
                            Some(b) => match sub_operand(b, prop, ctx, depth) {
                                Err(_) => return IfOutcome::Iacvt,
                                Ok(sub) => {
                                    result.push_str(sub.trim());
                                    any = true;
                                }
                            },
                        }
                        rest = &rest[close_end..];
                    }
                }
            }
        }
    }
    if any {
        IfOutcome::Substituted(result)
    } else {
        IfOutcome::Unchanged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx<'a>(customs: &'a HashMap<String, String>) -> IfEvalCtx<'a> {
        IfEvalCtx {
            customs,
            attr_reader: &|_: &str| None,
            random_ctx: &RandomEvalContext {
                element_seed: None,
                pct_typed: false,
                font_base_px: None,
            },
        }
    }

    fn run(value: &str, prop: &str, customs: &HashMap<String, String>) -> String {
        let c = ctx(customs);
        match substitute_if_calls(value, prop, &c, 0) {
            IfOutcome::Unchanged => value.to_string(),
            IfOutcome::Substituted(s) => s,
            IfOutcome::Iacvt => String::new(),
        }
    }

    fn m(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn equality_true_and_else_branches() {
        let c = m(&[("--x", "3")]);
        assert_eq!(run("if(style(--x: 3): value; else: other)", "--prop", &c), "value");
        assert_eq!(
            run("if(style(--x: 3): value; else: other)", "--prop", &m(&[("--x", "0")])),
            "other"
        );
    }

    #[test]
    fn truthiness_bare_custom_name() {
        let c = m(&[("--x", "3")]);
        assert_eq!(run("if(style(--x): tv; else: fv)", "--prop", &c), "tv");
        assert_eq!(run("if(style(--x): tv; else: fv)", "--prop", &HashMap::new()), "fv");
    }

    #[test]
    fn relational_numeric() {
        assert_eq!(run("if(style(2 < 3): a; else: b)", "--prop", &HashMap::new()), "a");
        assert_eq!(
            run("if(style(--n >= 3): a; else: b)", "--prop", &m(&[("--n", "3")])),
            "a"
        );
        assert_eq!(
            run("if(style(--n >= 3): a; else: b)", "--prop", &m(&[("--n", "2")])),
            "b"
        );
        assert_eq!(
            run("if(style(--n < 3): a; else: b)", "--prop", &m(&[("--n", "text")])),
            "b"
        );
    }

    #[test]
    fn boolean_algebra_not_or_and() {
        let c = m(&[("--x", "3"), ("--y", "var(--y)")]);
        assert_eq!(run("if(style(not (--x)): tv; else: fv)", "--prop", &c), "fv");
        assert_eq!(run("if(style(not (--x: var(--y))): tv; else: fv)", "--prop", &c), "tv");
        assert_eq!(run("if(style((--x) or (--y)): tv; else: fv)", "--prop", &c), "tv");
        assert_eq!(run("if(style((--x) and (--y)): tv; else: fv)", "--prop", &c), "fv");
    }

    #[test]
    fn r1_cycle_through_under_eval_is_iacvt() {
        // 条件 truthiness 直接自指 → R1 整声明 IACVT（''）。
        assert_eq!(run("if(style(--prop): tv; else: value)", "--prop", &HashMap::new()), "");
        // 条件左操作数 var 链达被求值声明 → R1。
        let c = m(&[("--x", "var(--prop)")]);
        assert_eq!(run("if(style(--x): tv; else: value)", "--prop", &c), "");
        // 已取支路 var(--prop) 环 → R1。
        let c2 = m(&[("--x", "3")]);
        assert_eq!(run("if(style(--x: 3): var(--prop); else: value)", "--prop", &c2), "");
    }

    #[test]
    fn r2_local_cycle_condition_false_only() {
        // --y 自环（不达被求值声明）→ truthiness false → else 支路照常代入。
        let c = m(&[("--y", "var(--y)")]);
        assert_eq!(
            run("if(style(--y): tv; else: false_value)", "--prop", &c),
            "false_value"
        );
    }

    #[test]
    fn r4_unused_branch_not_evaluated() {
        // 未取支路含 var(--prop) 环——不求值，声明照常代入。
        let c = m(&[("--x", "3")]);
        assert_eq!(
            run("if(style(--x: 0): var(--prop); else: value)", "--prop", &c),
            "value"
        );
    }

    #[test]
    fn no_match_no_else_is_iacvt() {
        let c = m(&[("--x", "0")]);
        assert_eq!(run("if(style(--x: 3): tv)", "--prop", &c), "");
    }

    #[test]
    fn equality_right_side_var_substitutes() {
        let c = m(&[("--x", "3")]);
        assert_eq!(run("if(style(--x: var(--x)): tv; else: fv)", "--prop", &c), "tv");
        // unset vs unset 恒 false（R3）。
        assert_eq!(
            run("if(style(--ne: var(--ne)): tv; else: fv)", "--prop", &HashMap::new()),
            "fv"
        );
    }

    #[test]
    fn unparseable_and_string_literal_kept() {
        // 结构形不全（段内无顶层 `:`）→ 调用保原文；`if(style(--x 3): tv)`
        // 则是可解析假条件（无 else → IACVT），见 no_match_no_else。
        assert_eq!(run("if(style(--x 3))", "--prop", &HashMap::new()), "if(style(--x 3))");
        // 字符串字面内 if( 不识别。
        assert_eq!(
            run("content \"if(style(--x: 3): tv)\"", "--prop", &HashMap::new()),
            "content \"if(style(--x: 3): tv)\""
        );
    }

    #[test]
    fn empty_branch_and_two_calls_splice() {
        let c = m(&[("--x", "0")]);
        assert_eq!(run("if(style(--x: 3): ; else: fb)", "--prop", &c), "fb");
        // 值内两个 if() 调用依序支路化。
        assert_eq!(
            run(
                "if(style(--x: 0): a; else: b) if(style(--x: 0): c; else: d)",
                "--prop",
                &c
            ),
            "a c"
        );
    }

    #[test]
    fn attr_operand_via_reader() {
        // 语料形（attr-security-if #73）：等值左操作数是自订属性名（读表），
        // 右操作数 attr() typed 调用（读元素属性）。
        let customs = m(&[("--condition-val", "3")]);
        let c = IfEvalCtx {
            customs: &customs,
            attr_reader: &|name: &str| if name == "data-n" { Some("3".into()) } else { None },
            random_ctx: &RandomEvalContext {
                element_seed: None,
                pct_typed: false,
                font_base_px: None,
            },
        };
        let out = match substitute_if_calls(
            "if(style(--condition-val: attr(data-n type(*))): tv; else: fv)",
            "--prop",
            &c,
            0,
        ) {
            IfOutcome::Substituted(s) => s,
            other => panic!("{other:?}"),
        };
        assert_eq!(out, "tv");
    }

    #[test]
    fn has_if_call_boundaries() {
        assert!(has_if_call("if(style(--x): 1;)"));
        assert!(has_if_call("  IF(style(--x): 1;)"));
        assert!(!has_if_call("notify(x)"));
        assert!(!has_if_call("plain value"));
    }

    #[test]
    fn cycle_dominates_or_fold() {
        // 语料 if-cycle #10 判例：`(--prop) or (--y)`、--y:3。环操作数
        // （--prop 环达被求值声明）支配布尔真——or 不短路于 True，整条
        // 件环达 → 整声明 IACVT（非 true_value）。
        let c = m(&[("--y", "3")]);
        assert_eq!(
            run(
                "if(style((--prop) or (--y)): true_value; else: false_value)",
                "--prop",
                &c
            ),
            ""
        );
        // 对照：无环操作数（未定义 --zz → Invalid → False）时 or 真值
        // 正常放行 → true_value——证明 IACVT 只来自环归因。
        assert_eq!(
            run(
                "if(style((--zz) or (--y)): true_value; else: false_value)",
                "--prop",
                &c
            ),
            "true_value"
        );
    }

    #[test]
    fn cycle_dominates_and_fold() {
        // 语料 if-cycle #11 判例：and 形同 or——环归因支配，False 不在
        // 有环时终局。
        let c = m(&[("--y", "3")]);
        assert_eq!(
            run(
                "if(style((--prop) and (--y)): true_value; else: false_value)",
                "--prop",
                &c
            ),
            ""
        );
    }

    #[test]
    fn attr_present_beats_var_fallback_cycle() {
        // 语料 if-cycle #29 判例：attr() 先评估——属性在场即吃掉调用，
        // 回退 var(--y)（自环）永不求值 → 等值真 → true_value。attr 晚
        // 于 var 通道时 var 先盲代回退自环 → 假 Invalid → false_value。
        let customs = m(&[("--x", "\"30px\""), ("--y", "var(--y)")]);
        let c = IfEvalCtx {
            customs: &customs,
            attr_reader: &|name: &str| if name == "data-foo" { Some("30px".into()) } else { None },
            random_ctx: &RandomEvalContext {
                element_seed: None,
                pct_typed: false,
                font_base_px: None,
            },
        };
        let out = match substitute_if_calls(
            "if(style(--x: attr(data-foo, var(--y))): true_value; else: false_value)",
            "--prop",
            &c,
            0,
        ) {
            IfOutcome::Substituted(s) => s,
            other => panic!("{other:?}"),
        };
        assert_eq!(out, "true_value");
        // 对照：属性缺席时回退被取——var(--y) 自环 → Invalid → 条件
        // false → false_value。
        let c_miss = IfEvalCtx {
            customs: &customs,
            attr_reader: &|_: &str| None,
            random_ctx: &RandomEvalContext {
                element_seed: None,
                pct_typed: false,
                font_base_px: None,
            },
        };
        let out_miss = match substitute_if_calls(
            "if(style(--x: attr(data-foo, var(--y))): true_value; else: false_value)",
            "--prop",
            &c_miss,
            0,
        ) {
            IfOutcome::Substituted(s) => s,
            other => panic!("{other:?}"),
        };
        assert_eq!(out_miss, "false_value");
    }
}
