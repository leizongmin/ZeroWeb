//! CSS 计算值生成。
//!
//! 将级联值转换为计算值，处理相对单位到绝对单位的转换。

use std::collections::HashMap;

use zero_css_parser::values::{CalcExpr, LengthValue};

use crate::property::ComputedStyle;
use crate::property::types::{ColumnRuleWidthComputedValue, FlexBasisValue, LineHeightValue};

/// 默认根字体大小（px）。
pub const ROOT_FONT_SIZE: f64 = 16.0;

/// 将相对长度转换为绝对像素值。
///
/// 支持 em、rem、vh、vw 等相对单位的转换。
pub fn resolve_length(
    length: &LengthValue,
    font_size: f64,
    viewport_width: Option<f64>,
    viewport_height: Option<f64>,
) -> f64 {
    match length {
        LengthValue::Px(v) => *v,
        LengthValue::Em(v) => v * font_size,
        LengthValue::Rem(v) => v * ROOT_FONT_SIZE,
        LengthValue::Vh(v) => {
            let vh = viewport_height.unwrap_or(900.0);
            v * vh / 100.0
        }
        LengthValue::Vw(v) => {
            let vw = viewport_width.unwrap_or(1440.0);
            v * vw / 100.0
        }
        LengthValue::Vmin(v) => {
            let vw = viewport_width.unwrap_or(1440.0);
            let vh = viewport_height.unwrap_or(900.0);
            v * vw.min(vh) / 100.0
        }
        LengthValue::Vmax(v) => {
            let vw = viewport_width.unwrap_or(1440.0);
            let vh = viewport_height.unwrap_or(900.0);
            v * vw.max(vh) / 100.0
        }
        LengthValue::Ch(v) => {
            // 近似：1ch ≈ 0.5em（真实 1ch = '0' 字形 advance，须 font-stack C-dep；
            // Ahem/monospace 真 ch=1.0em，0.5em 对 Ahem 偏小致 pre-wrap-align 等容器宽度偏窄，
            // 但全局改 1.0em 净 −7 非-Ahem ch 案（R1338），font-aware（Ahem→1.0/余→0.5）亦
            // css-text −16 已 revert——ch 与文本度量经 font-wall 耦合，单改 ch 致容器宽变但文本
            // 度量仍错 → 包裹更发散。R1853 white-space 实测 ch=1.0em 21.3%→19.7%（−6）三证收敛。
            // ch-0.5em 对 corpus 是 settled-correct，勿再以 ch 为 lever。
            v * font_size * 0.5
        }
        // 百分比值不在此处解析，由布局引擎根据容器尺寸处理
        LengthValue::Percentage(v) => *v,
        // auto 不需要解析为 px
        LengthValue::Auto => 0.0,
        // 数学表达式：使用完整上下文求值
        LengthValue::Calc(expr) => {
            let ctx = zero_css_parser::values::CalcContext {
                font_size: Some(font_size),
                root_font_size: Some(ROOT_FONT_SIZE),
                viewport_width,
                viewport_height,
                ..Default::default()
            };
            zero_css_parser::values::eval_calc_with_context(expr, &ctx).unwrap_or(0.0)
        }
        // fit-content() 递归解析内部值
        LengthValue::FitContent(inner) => resolve_length(inner, font_size, viewport_width, viewport_height),
        // min-content/max-content 需要内容信息，此处返回 0.0
        LengthValue::MinContent | LengthValue::MaxContent => 0.0,
    }
}

/// 解析 var() 引用并**完全**（含 var() 链，传递）解析自定义属性。
///
/// 支持：嵌套 var()、回退值、自定义属性链（`--a: var(--b); --b: green` → green）。
/// **环检测**：var() 引用形成环（直接或间接自引用）→ 返回原值（该 var() 在下游无法
/// 解析为合法值 → 属性按 invalid-at-computed-value-time 处理 → 继承/初始值），解析有界
///（无指数膨胀）。driving: WPT variable-declaration-48/49（var() 环致 6GB OOM）。
pub fn resolve_var(value: &str, custom_properties: &HashMap<String, String>) -> String {
    let mut visiting = Vec::new();
    resolve_var_recursive(value, custom_properties, &mut visiting).unwrap_or_else(|| value.to_string())
}

/// 递归解析 var()。返回 None 表示该值 invalid at computed-value-time
///（环引用、或无回退的未定义引用）—— 上层 [`resolve_var`] 据此回退到原值。
fn resolve_var_recursive(
    value: &str,
    custom_properties: &HashMap<String, String>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    if !value.contains("var(") {
        return Some(value.to_string());
    }
    substitute_embedded_var(value, custom_properties, visiting)
}

/// 解析单个 `var(--name, fallback)` 引用。
///
/// - `name` 在解析栈上（环）→ 用回退（解析之），无回退 → None（invalid）。
/// - `name` 已定义 → 递归完全解析其值（name 入栈做环检测）；其值 invalid → 本引用 invalid。
/// - `name` 未定义 → 用回退，无回退 → None。
fn resolve_var_reference(
    name: &str,
    fallback: Option<&str>,
    custom_properties: &HashMap<String, String>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    if visiting.iter().any(|n| n == name) {
        return fallback.and_then(|f| resolve_var_recursive(f, custom_properties, visiting));
    }
    if let Some(raw) = custom_properties.get(name) {
        visiting.push(name.to_string());
        let resolved = resolve_var_recursive(raw, custom_properties, visiting);
        visiting.pop();
        return resolved;
    }
    fallback.and_then(|f| resolve_var_recursive(f, custom_properties, visiting))
}

/// 替换值中嵌入的所有 var() 调用。任一 var() invalid → 整值 invalid（None）。
fn substitute_embedded_var(
    value: &str,
    custom_properties: &HashMap<String, String>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    let mut result = String::new();
    let mut idx = 0;
    while let Some(rel) = value[idx..].find("var(") {
        let start = idx + rel;
        result.push_str(&value[idx..start]);
        let inner_start = start + 4;
        let end = find_matching_paren(value, inner_start)?;
        let inner = &value[inner_start..end];
        let (name, fallback) = split_var_name_fallback(inner);
        let replacement = resolve_var_reference(name, fallback, custom_properties, visiting)?;
        result.push_str(&replacement);
        idx = end + 1;
    }
    result.push_str(&value[idx..]);
    Some(result)
}

/// 将 var() 内部内容拆为 `(name, Option<fallback>)`（按顶层逗号）。
fn split_var_name_fallback(inner: &str) -> (&str, Option<&str>) {
    match find_top_level_comma(inner) {
        Some(pos) => (inner[..pos].trim(), Some(inner[pos + 1..].trim())),
        None => (inner.trim(), None),
    }
}

/// 在字符串中查找匹配的右括号。
fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 1;
    let mut i = start;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    if depth == 0 { Some(i - 1) } else { None }
}

/// 查找字符串中顶层的逗号（不在括号内的）。
fn find_top_level_comma(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// 解析值中嵌入的所有 `env()` 调用（CSS Environment Values）。
///
/// 桌面浏览器无 display cutout / notch，`safe-area-inset-*` 解析为 `0px`；其他环境变量
/// （`titlebar-area-*`、`viewport-*` 等）未定义 → 用 fallback，无 fallback → 保留字面量
/// `env(name)`（下游解析失败 → 属性取 initial/inherited，同 var() invalid-at-computed-value-time）。
///
/// 返回 `Some(new)` 当发生替换，`None` 当值不含 `env(` 或替换后与原值相同（便于调用方判断
/// 是否需要更新级联值，镜像 [`resolve_var`] 的语义）。
pub fn resolve_env(value: &str) -> Option<String> {
    if !value.contains("env(") {
        return None;
    }
    let substituted = substitute_env(value);
    if substituted != value { Some(substituted) } else { None }
}

/// 替换值中所有 `env()` 调用为环境值或 fallback。
fn substitute_env(value: &str) -> String {
    let mut result = String::new();
    let mut idx = 0;
    while let Some(rel) = value[idx..].find("env(") {
        let start = idx + rel;
        result.push_str(&value[idx..start]);
        let inner_start = start + 4;
        let Some(end) = find_matching_paren(value, inner_start) else {
            // 未匹配的 `(`：保留剩余原文，终止扫描（防 OOM 死循环，同 substitute_embedded_var）。
            result.push_str(&value[start..]);
            return result;
        };
        let inner = &value[inner_start..end];
        let (name, fallback) = split_var_name_fallback(inner);
        result.push_str(&resolve_env_reference(name, fallback));
        idx = end + 1;
    }
    result.push_str(&value[idx..]);
    result
}

/// 解析单个 `env(name, fallback)`：已定义 → 环境值；未定义 → fallback（递归解析其中嵌套 env）；
/// 无 fallback → 保留字面量（下游 invalid）。
fn resolve_env_reference(name: &str, fallback: Option<&str>) -> String {
    if let Some(v) = env_value(name) {
        return v.to_string();
    }
    match fallback {
        // 递归解析 fallback 中的嵌套 env()（如 env(a, env(b, 1px))）。
        Some(f) => substitute_env(f),
        // 未定义且无 fallback：保留字面量 env(name)，下游解析失败 → initial/inherited（同 var()）。
        None => format!("env({})", name),
    }
}

/// 桌面环境下 `env()` 的固定值表。
///
/// CSS Environment Values §2：`safe-area-inset-{top,right,bottom,left}` 在桌面浏览器
/// （无显示缺口）解析为 `0px`。env 名大小写不敏感（CSS 关键字约定，对齐 chromium 行为）。
fn env_value(name: &str) -> Option<&'static str> {
    match name.trim().to_ascii_lowercase().as_str() {
        "safe-area-inset-top" | "safe-area-inset-right" | "safe-area-inset-bottom" | "safe-area-inset-left" => {
            Some("0px")
        }
        _ => None,
    }
}

/// 先解析 `env()`，再解析 `var()`（CSS Environment Values + CSS Variables）。
///
/// env() 须先于 var()：env() 可出现在 var() 的 fallback 中（如 `var(--x, env(safe-area-inset-top))`），
/// 先替换所有 env() 再解 var() 可正确处理嵌套。供级联值 broad 路径与 length 计算路径共用。
pub fn resolve_env_and_var(value: &str, custom_properties: &HashMap<String, String>) -> String {
    let after_env = resolve_env(value).unwrap_or_else(|| value.to_string());
    resolve_var(&after_env, custom_properties)
}

/// 将计算样式中的相对长度解析为绝对值。
///
/// 返回一个新的 ComputedStyle，其中所有相对长度都被转换为 px。
///
/// # 参数
///
/// - `parent_font_size` — 父元素的计算 font-size（px），用于 font-size 属性本身的 em 解析。
///   为 None 时（根元素）使用 ROOT_FONT_SIZE。
pub fn resolve_computed_style(
    style: &ComputedStyle,
    _custom_properties: &HashMap<String, String>,
    viewport_width: Option<f64>,
    viewport_height: Option<f64>,
    parent_font_size: Option<f64>,
) -> ComputedStyle {
    // font-size 属性本身：em/百分比 都相对于父元素的 font-size。
    // 注意：font-size 的百分比语义特殊（= 父 font-size 的百分比，CSS §10.1），
    // 与 width/height 等的百分比（容器尺寸）不同——故不沿用 resolve_length 的
    // Percentage 分支（那里返回原始数值交由布局引擎按容器解析），在此就地解析。
    let font_size_context = parent_font_size.unwrap_or(ROOT_FONT_SIZE);
    let font_size_px = match &style.font_size {
        LengthValue::Percentage(v) => v / 100.0 * font_size_context,
        other => resolve_length(other, font_size_context, viewport_width, viewport_height),
    };

    let mut resolved = style.clone();

    // 将 font-size 解析为绝对值
    resolved.font_size = LengthValue::Px(font_size_px);

    // 解析 line-height 中的长度值（em/rem 等需要相对于元素自身的 font-size）
    // CSS 规范：line-height 的 em 单位相对于元素自身的 font-size
    if let LineHeightValue::Length(ref len) = resolved.line_height {
        match len {
            LengthValue::Em(v) => {
                resolved.line_height = LineHeightValue::Length(LengthValue::Px(v * font_size_px));
            }
            LengthValue::Rem(v) => {
                let rem_px = viewport_width.map(|_| 16.0).unwrap_or(16.0);
                resolved.line_height = LineHeightValue::Length(LengthValue::Px(v * rem_px));
            }
            _ => {}
        }
    }

    // 解析所有长度属性（使用元素自身的 font-size）
    resolve_length_field(&mut resolved.width, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.height, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.min_width, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.min_height, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.max_width, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.max_height, font_size_px, viewport_width, viewport_height);

    resolve_length_field(&mut resolved.margin_top, font_size_px, viewport_width, viewport_height);
    resolve_length_field(
        &mut resolved.margin_right,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.margin_bottom,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(&mut resolved.margin_left, font_size_px, viewport_width, viewport_height);

    resolve_length_field(&mut resolved.padding_top, font_size_px, viewport_width, viewport_height);
    resolve_length_field(
        &mut resolved.padding_right,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.padding_bottom,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.padding_left,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    resolve_length_field(
        &mut resolved.border_top_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_right_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_bottom_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_left_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    // flex-basis 的 em/rem/ch 需解析为 Px（与 width/height 等同一 chokepoint）。
    // FlexBasisValue 包装 LengthValue，此前未入 resolve 列表 → `flex:0 0 4em` 的
    // flex-basis em 在 converter 当裸数字（4em→4px 而非 64），flex base size 错误。
    if let FlexBasisValue::Length(ref mut lv) = resolved.flex_basis {
        resolve_length_field(lv, font_size_px, viewport_width, viewport_height);
    }

    // csswg #2768/#11494（覆盖 CSS2.1 §8.5.3 旧表述）：computed border-width **始终**
    // 为 specified 值（none/hidden 都不归零），仅 used（布局/绘制）为 0。这样
    // `border-width:inherit` 才能继承（border-width-011 body 无 border-style→none，
    // p inherit 2em；border-width-012 body hidden）。converter border_lp / paint
    // border.rs / table_borders resolve_collapsed_borders 各自独立按 none|hidden=0 used，
    // 故移除 computed zeroing 对渲染无副作用，只让 computed 值保留供 inherit。
    // （R769e 仅保留 hidden；R769f 扩展到 none —— 完整 csswg #2768/#11494。）

    resolve_length_field(
        &mut resolved.border_top_left_radius,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_top_right_radius,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_bottom_right_radius,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.border_bottom_left_radius,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    resolve_length_field(&mut resolved.top, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.right, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.bottom, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.left, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.gap, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.row_gap, font_size_px, viewport_width, viewport_height);
    resolve_length_field(&mut resolved.column_gap, font_size_px, viewport_width, viewport_height);
    // R907：column-rule-width 是 Medium/Thin/Thick/Length 枚举（非裸 LengthValue），
    // 不在上方 resolve_length_field 列表内。其 Length 内部值（如 1em）须解析为 Px，
    // 否则 paint（painter/text.rs::paint_column_rules rule_w match）仅匹配 Length(Px)，
    // em 案落入 `_ => 1.0` → column-rule-width:1em 渲染为 1px（应按 element font-size）。
    if let ColumnRuleWidthComputedValue::Length(lv) = &resolved.column_rule_width.clone() {
        let mut lv = lv.clone();
        resolve_length_field(&mut lv, font_size_px, viewport_width, viewport_height);
        resolved.column_rule_width = ColumnRuleWidthComputedValue::Length(lv);
    }
    resolve_length_field(
        &mut resolved.letter_spacing,
        font_size_px,
        viewport_width,
        viewport_height,
    );
    resolve_length_field(
        &mut resolved.word_spacing,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    // outline-width 的 em/rem/ch 须解析为 Px（R907 同模式：column-rule-width em 缺
    // resolve 致 paint 仅匹配 Px）。否则 paint_outline（painter/border.rs:554）经
    // length_to_f32（helpers.rs:583，Px-only）把 em 丢为 0.0 → outline 消失。
    // outline-offset 0 corpus 用量故不入列（code-guidelines 不做零价值）。
    resolve_length_field(
        &mut resolved.outline_width,
        font_size_px,
        viewport_width,
        viewport_height,
    );

    resolved
}

/// 将单个长度字段解析为绝对 px。
///
/// 百分比值和 auto 保持不变，由布局引擎处理。
fn resolve_length_field(
    field: &mut LengthValue,
    font_size: f64,
    viewport_width: Option<f64>,
    viewport_height: Option<f64>,
) {
    match field {
        LengthValue::Px(_) => { /* 已经是绝对值 */ }
        LengthValue::Percentage(_) | LengthValue::Auto => { /* 由布局引擎处理 */ }
        // min-content/max-content 关键字需内容信息，此处无法解析；
        // 保留信号到布局引擎，由两趟固有宽度测量（layout-engine）解析。
        // converter 把 width/height 的 MaxContent 映射为塌缩（length(0)），
        // 保持与旧「解析为 Px(0)」行为中性，避免 taffy 把 width:auto 容器拉伸填充。
        LengthValue::MinContent | LengthValue::MaxContent => {}
        // 包含百分比的 calc 表达式保留，由布局引擎处理
        LengthValue::Calc(expr) if calc_contains_percentage(expr) => {}
        _ => {
            let px = resolve_length(field, font_size, viewport_width, viewport_height);
            *field = LengthValue::Px(px);
        }
    }
}

/// 检查 calc 表达式是否包含百分比值。
///
/// 包含百分比的 calc 表达式无法在计算值阶段完全解析，
/// 因为百分比需要相对于包含块的尺寸，这在布局阶段才可知。
fn calc_contains_percentage(expr: &CalcExpr) -> bool {
    match expr {
        CalcExpr::Number(_) => false,
        CalcExpr::Length(lv) => matches!(lv, LengthValue::Percentage(_)),
        CalcExpr::BinaryOp(left, _, right) => calc_contains_percentage(left) || calc_contains_percentage(right),
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter().any(calc_contains_percentage),
        CalcExpr::Clamp { min, val, max } => {
            calc_contains_percentage(min) || calc_contains_percentage(val) || calc_contains_percentage(max)
        }
        CalcExpr::UnaryOp(_, inner) => calc_contains_percentage(inner),
        CalcExpr::BinaryMathOp(_, a, b) => calc_contains_percentage(a) || calc_contains_percentage(b),
    }
}

/// 解析可能包含 var() / env() 的属性值。
///
/// 先解析 env() 再解析 var() 引用（见 [`resolve_env_and_var`]），再尝试解析为长度值。
pub fn compute_value(value: &str, custom_properties: &HashMap<String, String>, _font_size: f64) -> Option<String> {
    let resolved = resolve_env_and_var(value, custom_properties);
    if resolved != value { Some(resolved) } else { None }
}

/// 从自定义属性中收集所有 -- 开头的属性。
pub fn collect_custom_properties(cascaded: &HashMap<String, String>) -> HashMap<String, String> {
    cascaded
        .iter()
        .filter(|(k, _)| k.starts_with("--"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::property::ComputedStyle;
    use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue, PositionValue};

    #[test]
    fn test_resolve_px() {
        let px = LengthValue::Px(10.0);
        assert_eq!(resolve_length(&px, 16.0, None, None), 10.0);
    }

    #[test]
    fn test_resolve_em() {
        let em = LengthValue::Em(2.0);
        assert_eq!(resolve_length(&em, 16.0, None, None), 32.0);
    }

    #[test]
    fn test_resolve_rem() {
        let rem = LengthValue::Rem(1.5);
        assert_eq!(resolve_length(&rem, 20.0, None, None), 24.0); // rem 用 root font size
    }

    #[test]
    fn test_resolve_vh() {
        let vh = LengthValue::Vh(50.0);
        let result = resolve_length(&vh, 16.0, Some(1440.0), Some(900.0));
        assert_eq!(result, 450.0);
    }

    #[test]
    fn test_resolve_vw() {
        let vw = LengthValue::Vw(25.0);
        let result = resolve_length(&vw, 16.0, Some(1440.0), Some(900.0));
        assert_eq!(result, 360.0);
    }

    #[test]
    fn test_resolve_vmin() {
        let vmin = LengthValue::Vmin(10.0);
        let result = resolve_length(&vmin, 16.0, Some(1440.0), Some(900.0));
        assert_eq!(result, 90.0); // min(1440, 900) * 0.1
    }

    #[test]
    fn test_resolve_vmax() {
        let vmax = LengthValue::Vmax(10.0);
        let result = resolve_length(&vmax, 16.0, Some(1440.0), Some(900.0));
        assert_eq!(result, 144.0); // max(1440, 900) * 0.1
    }

    #[test]
    fn test_resolve_ch() {
        let ch = LengthValue::Ch(2.0);
        let result = resolve_length(&ch, 16.0, None, None);
        assert_eq!(result, 16.0); // 2 * 16 * 0.5
    }

    #[test]
    fn test_resolve_outline_width_em() {
        // R907 同模式：outline-width em 须在 compute 时解析为 Px，否则 paint_outline
        // 经 length_to_f32（Px-only）把 em 丢为 0.0 → outline 消失。
        // outline-width:5em @ font-size 32 → Px(160)。
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(32.0);
        style.outline_width = LengthValue::Em(5.0);
        let resolved = resolve_computed_style(&style, &HashMap::new(), None, None, Some(32.0));
        assert_eq!(resolved.outline_width, LengthValue::Px(160.0));
    }

    #[test]
    fn test_outline_width_initial_is_medium_3px() {
        // CSS UI §outline-width 初始值 = medium(3px)，与 border-width 初始 medium=3px 一致。
        // outline-style 初始 none 抑制绘制，故默认元素无可见 outline，但计算值须为 3px。
        // 修复前 ZW 初始 = 0px（偏离 spec + 与 border-width 不一致）。
        let default = ComputedStyle::default();
        assert_eq!(
            default.outline_width,
            LengthValue::Px(3.0),
            "outline-width 初始应为 medium=3px"
        );
    }

    #[test]
    fn test_resolve_var_simple() {
        let mut custom = HashMap::new();
        custom.insert("--main-color".to_string(), "red".to_string());

        let result = resolve_var("var(--main-color)", &custom);
        assert_eq!(result, "red");
    }

    #[test]
    fn test_resolve_var_with_fallback() {
        let custom = HashMap::new();
        let result = resolve_var("var(--undefined, blue)", &custom);
        assert_eq!(result, "blue");
    }

    #[test]
    fn test_resolve_var_undefined_no_fallback() {
        let custom = HashMap::new();
        let result = resolve_var("var(--undefined)", &custom);
        assert_eq!(result, "var(--undefined)"); // 返回原值
    }

    #[test]
    fn test_resolve_var_embedded() {
        let mut custom = HashMap::new();
        custom.insert("--size".to_string(), "10px".to_string());

        let result = resolve_var("margin: var(--size);", &custom);
        assert_eq!(result, "margin: 10px;");
    }

    #[test]
    fn test_resolve_env_safe_area_inset() {
        // 桌面环境：safe-area-inset-* 解析为 0px（无显示缺口）。
        assert_eq!(resolve_env("env(safe-area-inset-top)"), Some("0px".to_string()));
        assert_eq!(resolve_env("env(safe-area-inset-left)"), Some("0px".to_string()));
    }

    #[test]
    fn test_resolve_env_defined_ignores_fallback() {
        // 已定义 env：忽略 fallback（与 var() 定义优先一致）。
        assert_eq!(resolve_env("env(safe-area-inset-top, 10px)"), Some("0px".to_string()));
    }

    #[test]
    fn test_resolve_env_undefined_uses_fallback() {
        // 未定义 env：用 fallback（移动端专属环境变量桌面未定义）。
        assert_eq!(resolve_env("env(titlebar-area-x, 20px)"), Some("20px".to_string()));
    }

    #[test]
    fn test_resolve_env_undefined_no_fallback_keeps_literal() {
        // 未定义且无 fallback：保留字面量（下游解析失败 → initial/inherited，同 var() invalid）。
        assert_eq!(resolve_env("env(titlebar-area-x)"), None);
        // None → 调用方保留原值 "env(titlebar-area-x)"，下游 length 解析失败 → initial。
    }

    #[test]
    fn test_resolve_env_embedded_in_value() {
        // env() 嵌入更大值（padding-shorthand component）。
        assert_eq!(
            resolve_env("padding: env(safe-area-inset-top) 5px;"),
            Some("padding: 0px 5px;".to_string())
        );
    }

    #[test]
    fn test_resolve_env_case_insensitive_name() {
        // env 名大小写不敏感（CSS 关键字约定，对齐 chromium）。
        assert_eq!(resolve_env("env(SAFE-AREA-INSET-TOP)"), Some("0px".to_string()));
    }

    #[test]
    fn test_resolve_env_nested_in_fallback() {
        // 嵌套 env()：未定义外层 → 递归解 fallback 中的内层 env。
        assert_eq!(
            resolve_env("env(undefined-x, env(safe-area-inset-left))"),
            Some("0px".to_string())
        );
    }

    #[test]
    fn test_resolve_env_then_var_ordering() {
        // env() 先于 var()：env 出现在 var fallback 中须先替换。
        let mut custom = HashMap::new();
        custom.insert("--pad".to_string(), "8px".to_string());
        // var 定义 → 用 --pad，env fallback 不触发。
        assert_eq!(
            resolve_env_and_var("var(--pad, env(safe-area-inset-top))", &custom),
            "8px"
        );
        // var 未定义 → 用 fallback，其中 env() 替换为 0px。
        assert_eq!(
            resolve_env_and_var("var(--missing, env(safe-area-inset-top))", &custom),
            "0px"
        );
    }

    #[test]
    fn test_resolve_env_no_env_returns_none() {
        assert_eq!(resolve_env("10px"), None);
        assert_eq!(resolve_env("red"), None);
    }

    #[test]
    fn test_resolve_computed_style() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.margin_top = LengthValue::Em(2.0);
        style.padding_left = LengthValue::Rem(1.0);

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, None);

        assert_eq!(resolved.margin_top, LengthValue::Px(40.0)); // 2em * 20px
        assert_eq!(resolved.padding_left, LengthValue::Px(16.0)); // 1rem * 16px
    }

    #[test]
    fn test_compute_value_with_var() {
        let mut custom = HashMap::new();
        custom.insert("--width".to_string(), "100px".to_string());

        let result = compute_value("var(--width)", &custom, 16.0);
        assert_eq!(result, Some("100px".to_string()));
    }

    #[test]
    fn test_compute_value_no_var() {
        let custom = HashMap::new();
        let result = compute_value("100px", &custom, 16.0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_collect_custom_properties() {
        let mut cascaded = HashMap::new();
        cascaded.insert("--color".to_string(), "red".to_string());
        cascaded.insert("--size".to_string(), "10px".to_string());
        cascaded.insert("display".to_string(), "flex".to_string());

        let custom = collect_custom_properties(&cascaded);
        assert_eq!(custom.len(), 2);
        assert_eq!(custom.get("--color"), Some(&"red".to_string()));
        assert_eq!(custom.get("--size"), Some(&"10px".to_string()));
    }

    #[test]
    fn test_resolve_var_chained() {
        // var() 链应**完全**（传递）解析，而非只展开一层。
        let mut custom = HashMap::new();
        custom.insert("--a".to_string(), "var(--b)".to_string());
        custom.insert("--b".to_string(), "resolved".to_string());

        let result = resolve_var("var(--a)", &custom);
        assert_eq!(result, "resolved");
    }

    #[test]
    fn test_resolve_var_cycle_returns_original() {
        // var() 环（间接自引用）→ invalid at computed-value-time → 返回原值（含 var()，
        // 下游按非法处理 → 继承/初始）。解析必须**有界**（旧实现指数膨胀致 OOM，
        // driving: WPT variable-declaration-48/49）。
        let mut custom = HashMap::new();
        custom.insert("--a".to_string(), "red var(--b)".to_string());
        custom.insert("--b".to_string(), "var(--c)".to_string());
        custom.insert("--c".to_string(), "var(--d)".to_string());
        custom.insert("--d".to_string(), "var(--e)".to_string());
        custom.insert("--e".to_string(), "var(--a)".to_string());
        custom.insert("--f".to_string(), "var(--e)".to_string());

        let result = resolve_var("var(--f)", &custom);
        // 环 → 返回原值（不应膨胀、不应挂起）
        assert_eq!(result, "var(--f)");
        assert!(result.len() < 100, "环引用解析结果不应膨胀: len={}", result.len());
    }

    #[test]
    fn test_resolve_var_self_cycle_returns_original() {
        // 直接自引用 `--a: var(--a)` → 环 → 原值。
        let mut custom = HashMap::new();
        custom.insert("--a".to_string(), "var(--a)".to_string());
        let result = resolve_var("var(--a)", &custom);
        assert_eq!(result, "var(--a)");
    }

    // ── Percentage 和 Auto 测试 ──

    #[test]
    fn test_resolve_length_percentage() {
        let pct = LengthValue::Percentage(50.0);
        assert_eq!(resolve_length(&pct, 16.0, None, None), 50.0);
    }

    #[test]
    fn test_resolve_length_auto() {
        let auto = LengthValue::Auto;
        assert_eq!(resolve_length(&auto, 16.0, None, None), 0.0);
    }

    #[test]
    fn test_resolve_length_field_preserves_percentage() {
        let mut field = LengthValue::Percentage(50.0);
        resolve_length_field(&mut field, 16.0, None, None);
        assert_eq!(field, LengthValue::Percentage(50.0));
    }

    #[test]
    fn test_resolve_length_field_preserves_auto() {
        let mut field = LengthValue::Auto;
        resolve_length_field(&mut field, 16.0, None, None);
        assert_eq!(field, LengthValue::Auto);
    }

    #[test]
    fn test_resolve_length_field_converts_em() {
        let mut field = LengthValue::Em(2.0);
        resolve_length_field(&mut field, 16.0, None, None);
        assert_eq!(field, LengthValue::Px(32.0));
    }

    #[test]
    fn test_resolve_computed_style_preserves_auto_width() {
        let style = ComputedStyle::default();
        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, None);
        assert_eq!(resolved.width, LengthValue::Auto);
        assert_eq!(resolved.height, LengthValue::Auto);
    }

    #[test]
    fn test_resolve_computed_style_preserves_percentage() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Percentage(50.0);
        style.margin_top = LengthValue::Percentage(10.0);

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, None);
        assert_eq!(resolved.width, LengthValue::Percentage(50.0));
        assert_eq!(resolved.margin_top, LengthValue::Percentage(10.0));
    }

    #[test]
    fn test_resolve_computed_style_flex_basis_em_to_px() {
        // flex-basis 的 em 应解析为 px（4em * 16px = 64px）。
        // 此前 flex_basis 未入 resolve 列表 → `flex:0 0 4em` 在 converter 当裸数字 4em→4px。
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(16.0);
        style.flex_basis = FlexBasisValue::Length(LengthValue::Em(4.0));

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, Some(16.0));
        assert_eq!(resolved.flex_basis, FlexBasisValue::Length(LengthValue::Px(64.0)));
    }

    #[test]
    fn test_resolve_computed_style_column_rule_width_em_to_px() {
        // R907：column-rule-width 的 em 应按 element font-size 解析为 px。
        // 旧实现 Length(Em) 不 resolve → paint paint_column_rules rule_w 仅匹配 Length(Px)，
        // em 落入 `_ => 1.0` → column-rule-width:1em 渲染为 1px（应 20px @ font-size 20）。
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.column_rule_width = ColumnRuleWidthComputedValue::Length(LengthValue::Em(1.0));

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, Some(20.0));
        assert_eq!(
            resolved.column_rule_width,
            ColumnRuleWidthComputedValue::Length(LengthValue::Px(20.0))
        );
        // 关键字 Medium 不受影响。
        let mut style_kw = ComputedStyle::default();
        style_kw.column_rule_width = ColumnRuleWidthComputedValue::Medium;
        let resolved_kw = resolve_computed_style(&style_kw, &HashMap::new(), None, None, None);
        assert_eq!(resolved_kw.column_rule_width, ColumnRuleWidthComputedValue::Medium);
    }

    #[test]
    fn test_resolve_computed_style_flex_basis_preserves_non_length() {
        // flex-basis 的 auto/content 不应被长度解析改写。
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(16.0);
        style.flex_basis = FlexBasisValue::Auto;
        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, Some(16.0));
        assert_eq!(resolved.flex_basis, FlexBasisValue::Auto);
    }

    #[test]
    fn test_explicit_zero_px_not_auto() {
        // 确保 0px 不被误认为 auto
        let mut field = LengthValue::Px(0.0);
        resolve_length_field(&mut field, 16.0, None, None);
        assert_eq!(field, LengthValue::Px(0.0));
        assert_ne!(field, LengthValue::Auto);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增计算值解析测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// em 转 px：2em * 20px font-size = 40px
    fn test_em_to_px_conversion() {
        let em = LengthValue::Em(2.0);
        assert_eq!(resolve_length(&em, 20.0, None, None), 40.0);
    }

    #[test]
    /// em 转 px：1.5em * 32px font-size = 48px
    fn test_em_to_px_different_font_size() {
        let em = LengthValue::Em(1.5);
        assert_eq!(resolve_length(&em, 32.0, None, None), 48.0);
    }

    #[test]
    /// rem 转 px：2rem * 16px (root) = 32px（忽略 font_size 参数）
    fn test_rem_to_px_ignores_font_size() {
        let rem = LengthValue::Rem(2.0);
        // rem 使用 ROOT_FONT_SIZE (16.0)，不使用传入的 font_size
        assert_eq!(resolve_length(&rem, 32.0, None, None), 32.0);
    }

    #[test]
    /// 百分比值不在此处解析，原样返回
    fn test_percentage_not_resolved() {
        let pct = LengthValue::Percentage(50.0);
        assert_eq!(resolve_length(&pct, 16.0, None, None), 50.0);
    }

    #[test]
    /// resolve_computed_style 将 em 转换为 px（使用 font-size）
    fn test_resolve_computed_style_em_with_font_context() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(24.0);
        style.margin_top = LengthValue::Em(2.0); // 2 * 24 = 48

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, None);
        assert_eq!(resolved.margin_top, LengthValue::Px(48.0));
    }

    #[test]
    /// resolve_computed_style 使用 viewport 尺寸
    fn test_resolve_computed_style_with_viewport() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Vw(50.0);
        style.height = LengthValue::Vh(25.0);

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, Some(1000.0), Some(800.0), None);
        assert_eq!(resolved.width, LengthValue::Px(500.0)); // 50vw of 1000
        assert_eq!(resolved.height, LengthValue::Px(200.0)); // 25vh of 800
    }

    #[test]
    /// vmin 和 vmax 计算
    fn test_resolve_vmin_vmax() {
        let vmin = LengthValue::Vmin(10.0);
        let vmax = LengthValue::Vmax(10.0);
        // viewport: 1440 x 900, min=900, max=1440
        assert_eq!(resolve_length(&vmin, 16.0, Some(1440.0), Some(900.0)), 90.0);
        assert_eq!(resolve_length(&vmax, 16.0, Some(1440.0), Some(900.0)), 144.0);
    }

    #[test]
    /// ch 单位近似为 0.5em
    fn test_resolve_ch_approximation() {
        let ch = LengthValue::Ch(4.0);
        // 4ch * 20 * 0.5 = 40
        assert_eq!(resolve_length(&ch, 20.0, None, None), 40.0);
    }

    #[test]
    /// resolve_var 嵌套 var() 带回退值
    fn test_resolve_var_nested_with_fallback() {
        let mut custom = HashMap::new();
        custom.insert("--primary".to_string(), "blue".to_string());

        // var(--undefined, var(--primary)) — fallback 是 var(--primary)
        let result = resolve_var("var(--undefined, var(--primary))", &custom);
        assert_eq!(result, "blue");
    }

    #[test]
    /// resolve_var 嵌入多个 var() 调用
    fn test_resolve_var_multiple_embedded() {
        let mut custom = HashMap::new();
        custom.insert("--a".to_string(), "10px".to_string());
        custom.insert("--b".to_string(), "20px".to_string());

        let result = resolve_var("margin: var(--a) var(--b);", &custom);
        assert_eq!(result, "margin: 10px 20px;");
    }

    #[test]
    /// compute_value 返回 None 当值不含 var()
    fn test_compute_value_plain_value() {
        let custom = HashMap::new();
        assert_eq!(compute_value("100px", &custom, 16.0), None);
    }

    #[test]
    /// font-size 的 em 应该使用父元素的 font-size，而不是自身的
    fn test_font_size_em_uses_parent() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Em(1.5); // 1.5em

        let custom = HashMap::new();
        // 父元素 font-size 为 20px，1.5em 应该 = 30px
        let resolved = resolve_computed_style(&style, &custom, None, None, Some(20.0));
        assert_eq!(resolved.font_size, LengthValue::Px(30.0));
    }

    #[test]
    /// rem 应始终使用 16px 根字体大小，不受父元素影响
    fn test_font_size_rem_uses_root() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Rem(2.0); // 2rem

        let custom = HashMap::new();
        // 即使父元素 font-size 为 32px，rem 仍使用 root 16px
        let resolved = resolve_computed_style(&style, &custom, None, None, Some(32.0));
        assert_eq!(resolved.font_size, LengthValue::Px(32.0)); // 2 * 16 = 32
    }

    #[test]
    /// font-size 的百分比应解析为父元素 font-size 的百分比（CSS §10.1），
    /// 而非返回原始数值（区别于 width/height 等的容器相对百分比）。
    /// 回归守卫：R308 修复 `font-size: 500%` 曾被错误解析为 500px 的 bug。
    fn test_font_size_percentage_uses_parent() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Percentage(500.0); // 500%

        let custom = HashMap::new();
        // 父元素 font-size 为 16px，500% 应 = 80px（旧 bug 返回 500.0）
        let resolved = resolve_computed_style(&style, &custom, None, None, Some(16.0));
        assert_eq!(resolved.font_size, LengthValue::Px(80.0));

        // 父元素 font-size 为 20px，150% 应 = 30px
        style.font_size = LengthValue::Percentage(150.0);
        let resolved = resolve_computed_style(&style, &custom, None, None, Some(20.0));
        assert_eq!(resolved.font_size, LengthValue::Px(30.0));

        // 根元素（parent_font_size=None）使用 ROOT_FONT_SIZE(16)，100% 应 = 16px
        style.font_size = LengthValue::Percentage(100.0);
        let resolved = resolve_computed_style(&style, &custom, None, None, None);
        assert_eq!(resolved.font_size, LengthValue::Px(16.0));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增计算值边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// ComputedStyle::default 所有字段初始值正确性
    fn test_default_computed_style_all_fields() {
        let style = ComputedStyle::default();
        assert_eq!(style.display, DisplayValue::Inline);
        assert_eq!(style.position, PositionValue::Static);
        assert_eq!(style.width, LengthValue::Auto);
        assert_eq!(style.height, LengthValue::Auto);
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(style.background_color, ColorValue::Transparent);
        assert_eq!(style.opacity, 1.0);
        assert_eq!(style.font_size, LengthValue::Px(16.0));
        assert_eq!(style.margin_top, LengthValue::Px(0.0));
        assert_eq!(style.padding_top, LengthValue::Px(0.0));
        // border-width 初始值 = medium（CSS §8.5.1，ZeroWeb 取 3px）；实际无布局边框，
        // 因为 border-style 初始 = none，converter 在 style=none 时把 width 抑制为 0。
        assert_eq!(style.border_top_width, LengthValue::Px(3.0));
    }

    #[test]
    /// resolve_length: Px 原样返回
    fn test_resolve_length_px_passthrough() {
        let px = LengthValue::Px(42.0);
        assert_eq!(resolve_length(&px, 16.0, None, None), 42.0);
    }

    #[test]
    /// resolve_length: em 使用当前 font-size
    fn test_resolve_length_em_context() {
        let em = LengthValue::Em(3.0);
        assert_eq!(resolve_length(&em, 10.0, None, None), 30.0);
    }

    #[test]
    /// resolve_length: vmin 使用 min(vw, vh)
    fn test_resolve_length_vmin_calculation() {
        let vmin = LengthValue::Vmin(25.0);
        // viewport 800x600, min=600, 25vmin = 150
        assert_eq!(resolve_length(&vmin, 16.0, Some(800.0), Some(600.0)), 150.0);
    }

    #[test]
    /// resolve_length: vmax 使用 max(vw, vh)
    fn test_resolve_length_vmax_calculation() {
        let vmax = LengthValue::Vmax(25.0);
        // viewport 800x600, max=800, 25vmax = 200
        assert_eq!(resolve_length(&vmax, 16.0, Some(800.0), Some(600.0)), 200.0);
    }

    #[test]
    /// resolve_var 带嵌套回退值
    fn test_resolve_var_nested_fallback() {
        let mut custom = HashMap::new();
        custom.insert("--primary".to_string(), "#ff0".to_string());

        let result = resolve_var("var(--missing, var(--primary))", &custom);
        assert_eq!(result, "#ff0");
    }

    #[test]
    /// resolve_var 对不存在的变量无回退时返回原值
    fn test_resolve_var_undefined_no_fallback_returns_original() {
        let custom = HashMap::new();
        let result = resolve_var("var(--nonexistent)", &custom);
        assert_eq!(result, "var(--nonexistent)");
    }

    #[test]
    /// resolve_var 空自定义属性表
    fn test_resolve_var_empty_custom_properties() {
        let custom = HashMap::new();
        let result = resolve_var("color: var(--x, blue);", &custom);
        assert_eq!(result, "color: blue;");
    }

    #[test]
    /// compute_value 对不含 var() 的值返回 None
    fn test_compute_value_plain_returns_none() {
        let custom = HashMap::new();
        assert_eq!(compute_value("100px", &custom, 16.0), None);
        assert_eq!(compute_value("red", &custom, 16.0), None);
    }

    #[test]
    /// collect_custom_properties 过滤非自定义属性
    fn test_collect_custom_properties_filters_standard() {
        let mut cascaded = HashMap::new();
        cascaded.insert("--a".to_string(), "1".to_string());
        cascaded.insert("--b".to_string(), "2".to_string());
        cascaded.insert("color".to_string(), "red".to_string());
        cascaded.insert("display".to_string(), "flex".to_string());

        let custom = collect_custom_properties(&cascaded);
        assert_eq!(custom.len(), 2);
        assert_eq!(custom.get("--a"), Some(&"1".to_string()));
        assert_eq!(custom.get("--b"), Some(&"2".to_string()));
        assert!(!custom.contains_key("color"));
    }

    #[test]
    /// resolve_computed_style 带 parent_font_size 上下文
    fn test_resolve_computed_style_parent_font_context() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Em(2.0);
        style.margin_top = LengthValue::Em(1.0);

        let custom = HashMap::new();
        // parent font-size = 20px, so 2em font-size = 40px
        // then margin-top 1em uses element's own font-size (40px) = 40px
        let resolved = resolve_computed_style(&style, &custom, None, None, Some(20.0));
        assert_eq!(resolved.font_size, LengthValue::Px(40.0));
        assert_eq!(resolved.margin_top, LengthValue::Px(40.0));
    }

    #[test]
    /// resolve_computed_style 百分比值保持不变
    fn test_resolve_computed_style_preserves_percentages() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Percentage(75.0);
        style.height = LengthValue::Percentage(50.0);
        style.padding_top = LengthValue::Percentage(10.0);

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, None);
        assert_eq!(resolved.width, LengthValue::Percentage(75.0));
        assert_eq!(resolved.height, LengthValue::Percentage(50.0));
        assert_eq!(resolved.padding_top, LengthValue::Percentage(10.0));
    }

    #[test]
    /// resolve_computed_style Auto 值保持不变
    fn test_resolve_computed_style_preserves_auto() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Auto;
        style.height = LengthValue::Auto;

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, None);
        assert_eq!(resolved.width, LengthValue::Auto);
        assert_eq!(resolved.height, LengthValue::Auto);
    }

    #[test]
    /// resolve_computed_style 默认视口值（None 时使用默认值）
    fn test_resolve_computed_style_default_viewport() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Vw(10.0);

        let custom = HashMap::new();
        let resolved = resolve_computed_style(&style, &custom, None, None, None);
        // 默认视口宽度 1440, 10vw = 144
        assert_eq!(resolved.width, LengthValue::Px(144.0));
    }

    #[test]
    /// resolve_var 同一字符串中多个不同 var() 替换
    fn test_resolve_var_multiple_different_vars() {
        let mut custom = HashMap::new();
        custom.insert("--x".to_string(), "10px".to_string());
        custom.insert("--y".to_string(), "20px".to_string());

        let result = resolve_var("calc(var(--x) + var(--y))", &custom);
        assert_eq!(result, "calc(10px + 20px)");
    }

    #[test]
    /// find_matching_paren 正确匹配嵌套括号
    fn test_find_matching_paren_nested() {
        let s = "var(--x, calc(100px + 50px))";
        let start = 4; // position of '(' after 'var'
        let end = find_matching_paren(s, start);
        assert_eq!(end, Some(s.len() - 1));
    }
}
