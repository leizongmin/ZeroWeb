//! Document 约束校验伪类判定 —— 拆自 `mod.rs`（rule 5 单文件 <2000 行，R3284）。
//!
//! 本模块为 [`super::Document`] 的约束校验面（`:valid`/`:invalid`/`:in-range`/`:out-of-range`
//! 的权威判定）。R3284 为闭合 DOM 选择器与 style-system CSS 的一致性，把这四个 CSS 解析器
//! **已识别**但双引擎（DOM `query.rs` + style-system matcher）均走 `_ => false`/`_ => None`
//! 的约束校验伪类补全为静态可判定子集（HTML §4.10.20 Constraint validation + CSS Selectors L4）。
//!
//! 语义范围（诚实标注）：headless 静态 DOM 无 JS 运行时值变更，本实现按 **内容属性**（value/
//! min/max/required/type）静态求值，覆盖最常见的两类约束——`valueMissing`（required + 空值）
//! 与 `rangeUnderflow`/`rangeOverflow`（数值/日期类 min/max）。`patternMismatch`（需 regex 引擎，
//! dom crate 无 regex 依赖）与 `typeMismatch`（URL/email 格式）不在静态范围，不参与判定（
//! 即不因此判 invalid——permissive valid，与 engine shim `ValidityState` R2825 哲学一致）。
//!
//! 作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段（`nodes`）与
//! `mod.rs` 的私有查询助手——Rust 隐私规则：私有项对定义模块及其后代可见，故无需任何可见性
//! 改动（镜像 R3280 `form_state.rs`、R3281 `lang_dir.rs`、R3283 `target.rs` 拆分模式）。

use crate::node::NodeId;

use super::Document;

impl Document {
    /// `:invalid` 的权威判定（HTML spec `selector-invalid` / §4.10.20）。
    ///
    /// spec 三类匹配（https://html.spec.whatwg.org/multipage/semantics-other.html#selector-invalid）：
    /// ① 候选校验元素自身不满足约束；② **form 元素**是 ≥1 个无效候选的 form owner；
    /// ③ **fieldset 元素**拥有 ≥1 个无效候选**后代**（R153：WPT Element-closest
    /// `test11.closest(':invalid')` 期望 fieldset#test2——其内 input#test9 required 空 value）。
    /// 候选/静态范围同旧：`valueMissing`（required + 空值）+ `rangeUnderflow`/`rangeOverflow`
    ///（range-applicable type）。patternMismatch/typeMismatch 不在静态范围。
    ///
    /// 供 DOM `:invalid` 选择器（`element_matches_selector`）与 style-system `:invalid` CSS 匹配共享。
    pub fn is_invalid_element(&self, node: NodeId) -> bool {
        if self.is_validation_candidate(node) {
            return self.value_missing(node) || self.range_underflow(node) || self.range_overflow(node);
        }
        // ② form：form owner 关系（祖先链最近 form）。③ fieldset：任意后代。
        let tag = self.element_local_name(node);
        if tag == Some("form") {
            return self.has_invalid_candidate_in(node, false);
        }
        if tag == Some("fieldset") {
            return self.has_invalid_candidate_in(node, true);
        }
        false
    }

    /// node 子树内是否存在无效候选校验元素。`descendant_any`：form 只认 form owner 为本
    /// form 的候选（祖先链上行到本 form 之前无其他 form 嵌套），fieldset 认任意后代
    ///（spec 措辞即 descendant，不要求 owner 关系）。
    fn has_invalid_candidate_in(&self, root: NodeId, descendant_any: bool) -> bool {
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            for &c in &self.child_nodes(n) {
                if self.element_local_name(c).is_none() {
                    continue;
                }
                if self.is_validation_candidate(c)
                    && (self.value_missing(c) || self.range_underflow(c) || self.range_overflow(c))
                {
                    // form 变体：候选的 form owner 须是 root 本身（嵌套 form 的候选不算）。
                    if descendant_any || self.form_owner(c) == Some(root) {
                        return true;
                    }
                }
                stack.push(c);
            }
        }
        false
    }

    /// `:valid` 的权威判定（CSS Selectors L4 + HTML §4.10.20）。
    ///
    /// 候选校验元素且无静态约束失败。`:valid` 的范围与 `:invalid` 的**候选元素**判定对称——
    /// 非候选元素不匹配（HTML：button/output 等 barred 元素、form/fieldset 的 :invalid 祖先
    /// 形态在 :valid 无对应形态——spec `selector-valid` 仅「candidates that satisfy」）。
    pub fn is_valid_element(&self, node: NodeId) -> bool {
        if !self.is_validation_candidate(node) {
            return false;
        }
        !(self.value_missing(node) || self.range_underflow(node) || self.range_overflow(node))
    }

    /// `:in-range` 的权威判定（CSS Selectors L4 + HTML §4.10.20「Suffering from being out of range」）。
    ///
    /// range-applicable type（number/range/date/time/datetime-local/month/week）的 input，**有 value**
    /// 且 value 落在 [min, max]（min/max 为声明边界，缺省侧不约束）。无 value（空）不匹配
    /// （既不在范围也不在范围外）。
    pub fn is_in_range_element(&self, node: NodeId) -> bool {
        // 有 value 且 min/max 至少一侧声明；range_order 返 Some(ord) 仅当 value 可解析。
        // in-range = ord 不是 Less（越下界）也不是 Greater（越上界），即 Equal（含落在区间内）。
        matches!(self.range_order(node).1, Some(std::cmp::Ordering::Equal))
    }

    /// `:out-of-range` 的权威判定（`:in-range` 的补集语义）。
    pub fn is_out_of_range_element(&self, node: NodeId) -> bool {
        matches!(
            self.range_order(node).1,
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Greater)
        )
    }

    // ── 内部判定助手 ─────────────────────────────────────────────────────

    /// 元素是否为「约束校验候选」（HTML §4.10.20「barred/skipped」之外的 submittable）。
    ///
    /// 静态子集：input/select/textarea，非 `disabled`（复用 `is_effectively_disabled` 含祖先传播），
    /// 非 `readonly`（input 的 readonly 属性 bar 其约束）。注：button/output/fieldset/form/option
    /// 均 barred 或非 submittable，不在候选——既不 :valid 也不 :invalid。
    fn is_validation_candidate(&self, node: NodeId) -> bool {
        let Some(tag) = self.element_local_name(node) else {
            return false;
        };
        if !matches!(tag, "input" | "select" | "textarea") {
            return false;
        }
        if self.is_effectively_disabled(node) {
            return false;
        }
        // input readonly 属性 bar 其约束校验（spec「barred from constraint validation」）。
        if tag == "input" && self.get_attribute(node, "readonly").is_some() {
            return false;
        }
        true
    }

    /// `valueMissing`：候选元素带 `required` 但无值（HTML §4.10.20.3）。
    fn value_missing(&self, node: NodeId) -> bool {
        let Some(tag) = self.element_local_name(node) else {
            return false;
        };
        if self.get_attribute(node, "required").is_none() {
            return false;
        }
        match tag {
            "input" => {
                let ty = self
                    .get_attribute(node, "type")
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                // checkbox/radio：required 时「checked 状态缺失」即 valueMissing。
                if ty == "checkbox" {
                    return self.get_attribute(node, "checked").is_none();
                }
                if ty == "radio" {
                    let name = self.get_attribute(node, "name").unwrap_or_default();
                    let owner = self.form_owner(node);
                    let scope = owner.unwrap_or_else(|| self.root());
                    return !self.radio_group_has_checked(scope, &name, owner);
                }
                // 文本类：value 属性空/缺失。
                self.get_attribute(node, "value").is_none_or(|v| v.is_empty())
            }
            "textarea" => !self.element_has_text_content(node),
            "select" => {
                // select required：无 selected option（无 selected 属性的 option）即 valueMissing。
                // 简化：检查是否有任何 option 带 selected 属性。
                !self.has_selected_option(node)
            }
            _ => false,
        }
    }

    /// `<select>` 子树是否有带 `selected` 属性的 `<option>`。
    fn has_selected_option(&self, root: NodeId) -> bool {
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            let is_sel_opt = self.element_local_name(n).is_some_and(|t| t == "option")
                && self.get_attribute(n, "selected").is_some();
            if is_sel_opt {
                return true;
            }
            for &c in &self.child_nodes(n) {
                stack.push(c);
            }
        }
        false
    }

    /// 计算 range-applicable input 的 value 相对 [min,max] 的顺序。
    ///
    /// 返回 `(value_opt, order_opt)`：value_opt=解析出的可比较值（仅 range type 才非 None），
    /// order_opt=`Some(cmp)` 为 value 与约束边界的顺序——仅当 value 可解析**且** min/max 至少一侧
    /// 声明时才非 None：`Less`（<min，越下界）=positive→out-of-range，`Greater`（>max，越上界）
    /// =positive→out-of-range，否则（无越界，含仅有单侧边界且未越）`Equal`。
    fn range_order(&self, node: NodeId) -> (Option<RangeVal>, Option<std::cmp::Ordering>) {
        let Some(tag) = self.element_local_name(node) else {
            return (None, None);
        };
        if tag != "input" {
            return (None, None);
        }
        let ty = self
            .get_attribute(node, "type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let parse = match range_value_type(&ty) {
            Some(p) => p,
            None => return (None, None),
        };
        let Some(value) = self.get_attribute(node, "value").filter(|v| !v.is_empty()) else {
            return (None, None);
        };
        let Some(val) = parse(&value) else {
            // value 存在但不可解析（如 number input 含非数字）→ 既不 :in-range 也不 :out-of-range。
            return (None, None);
        };
        let min = self
            .get_attribute(node, "min")
            .filter(|v| !v.is_empty())
            .and_then(|m| parse(&m));
        let max = self
            .get_attribute(node, "max")
            .filter(|v| !v.is_empty())
            .and_then(|m| parse(&m));
        if min.is_none() && max.is_none() {
            // 无声明边界 → 非约束（spec：in-range/out-of-range 仅适用有 min 或 max 的 range type）。
            return (None, None);
        }
        let ord = match (min.as_ref(), max.as_ref()) {
            (Some(lo), _) if val.cmp(lo) == std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            (_, Some(hi)) if val.cmp(hi) == std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        };
        (Some(val), Some(ord))
    }

    /// `rangeUnderflow`：range-applicable input 的 value < min。
    fn range_underflow(&self, node: NodeId) -> bool {
        matches!(self.range_order(node).1, Some(std::cmp::Ordering::Less))
    }

    /// `rangeOverflow`：range-applicable input 的 value > max。
    fn range_overflow(&self, node: NodeId) -> bool {
        matches!(self.range_order(node).1, Some(std::cmp::Ordering::Greater))
    }
}

/// 可比较的范围值（数值或归一化时间戳）。
type RangeVal = i128;

/// range-applicable input type 的值解析器：返回把属性串解析为可比较 `RangeVal` 的闭包。
///
/// - 数值类（number/range）：解析为整数 ×1000（保留 3 位小数精度，避开 f64 比较）。
/// - 日期/时间类：解析为「归一化可比序数」（ISO 字符串逐字节比较已对齐字典序=时序，故直接转
///   `i128` 的 ASCII 字节哈希近似——但为确定性，这里用字符级字典序比较的代理：因 RangeVal 为
///   整数，日期类转成其归一化 YYYYMMDD... 整数）。
fn range_value_type(ty: &str) -> Option<fn(&str) -> Option<RangeVal>> {
    match ty {
        "number" | "range" => Some(parse_decimal),
        "date" | "datetime-local" | "month" | "week" | "time" => Some(parse_temporal),
        _ => None,
    }
}

/// 解析十进制数（含负号、小数点）为 `i128 ×1000`（3 位小数精度）。
fn parse_decimal(s: &str) -> Option<RangeVal> {
    let s = s.trim();
    let (neg, rest) = if let Some(r) = s.strip_prefix('-') {
        (true, r)
    } else if let Some(r) = s.strip_prefix('+') {
        (false, r)
    } else {
        (false, s)
    };
    let mut parts = rest.split('.');
    let int_part = parts.next()?;
    let frac_part = parts.next().unwrap_or("");
    if parts.next().is_some() || int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    if !int_part.is_empty() && !int_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !frac_part.is_empty() && !frac_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let int_val: i128 = int_part.parse().ok()?;
    // 小数部分取前 3 位（不足补 0，超出截断——3 位精度足够 range 比较）。
    let mut frac3: [u8; 3] = [b'0'; 3];
    for (i, b) in frac_part.bytes().take(3).enumerate() {
        frac3[i] = b;
    }
    let frac_val: i128 = (frac3[0] as i128 - b'0' as i128) * 100
        + (frac3[1] as i128 - b'0' as i128) * 10
        + (frac3[2] as i128 - b'0' as i128);
    let mut total = int_val * 1000 + frac_val;
    if neg {
        total = -total;
    }
    Some(total)
}

/// 解析时间日期类（date/month/week/time/datetime-local）为可比较 `i128`。
///
/// 这些类型的合法值均为「定宽前导补零」的 ISO 格式（如 `2026-08-12`、`13:45`、`2026-W32`），
/// 字典序与时间序一致——故把字符串字节作为基 128 大整数（每个 ASCII 字节 ≤127，作为「数字」
/// 拼接）。范围比较 = 字典序 = 时间序。非法格式（解析失败）返 None。
fn parse_temporal(s: &str) -> Option<RangeVal> {
    let s = s.trim();
    // 粗校验：非空 + 仅含 ASCII 可见字符（字母/数字/-:T W）。
    if s.is_empty()
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b':' | b' ' | b'+'))
    {
        return None;
    }
    // 基 128 大整数：每字节作为一个「数字」，base 128 保证乘法不溢出 i128（最多 ~18 字节仍安全，
    // date/datetime-local 串长 ≤ 16）。
    let mut acc: i128 = 0;
    for b in s.bytes() {
        acc = acc.checked_mul(128)?.checked_add((b as i128).min(127))?;
    }
    Some(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_parsing_basic() {
        assert_eq!(parse_decimal("5"), Some(5000));
        assert_eq!(parse_decimal("5.5"), Some(5500));
        assert_eq!(parse_decimal("-5.5"), Some(-5500));
        assert_eq!(parse_decimal("0.001"), Some(1));
        assert_eq!(parse_decimal("abc"), None);
    }

    #[test]
    fn temporal_parsing_order() {
        // 字典序 = 时间序：2026-01-01 < 2026-12-31
        let a = parse_temporal("2026-01-01").unwrap();
        let b = parse_temporal("2026-12-31").unwrap();
        assert!(a < b);
    }

    #[test]
    fn range_type_recognition() {
        assert!(range_value_type("number").is_some());
        assert!(range_value_type("date").is_some());
        assert!(range_value_type("text").is_none());
        assert!(range_value_type("").is_none());
    }
}
