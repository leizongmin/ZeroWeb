//! CSS 级联算法。
//!
//! 实现级联优先级排序：按来源、!important、@layer、specificity、出现顺序
//! 决定每个属性的最终胜出声明。

use std::collections::HashMap;

use crate::property::{ComputedStyle, PropertyRegistry, apply_property_value_with_quirks};
use zero_css_parser::values::{LengthValue, parse_length};

/// CSS 声明来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Origin {
    /// 用户代理（浏览器默认样式表）。
    UserAgent = 0,
    /// 用户样式表。
    User = 1,
    /// 作者样式表。
    Author = 2,
}

/// 级联优先级键。
///
/// 编码了 CSS Cascading and Inheritance Level 5 的优先级规则。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CascadeOrder {
    /// 声明来源。
    pub origin: Origin,
    /// @layer 索引（None 表示未分层）。
    pub layer_index: Option<usize>,
    /// 选择器特异性 (A, B, C)。
    pub specificity: (u32, u32, u32),
    /// 源码出现顺序（越大越后出现）。
    pub position: usize,
    /// 是否为 !important 声明。
    pub important: bool,
}

impl CascadeOrder {
    /// 创建一个新的级联顺序。
    pub fn new(
        origin: Origin,
        layer_index: Option<usize>,
        specificity: (u32, u32, u32),
        position: usize,
        important: bool,
    ) -> Self {
        Self {
            origin,
            layer_index,
            specificity,
            position,
            important,
        }
    }

    /// 计算 Ord 比较用的排序键。
    ///
    /// 返回一个元组，按照级联优先级从低到高排列：
    /// 1. normal < important
    /// 2. important 时: user-agent > user > author（反转）
    ///    normal 时: author > user > user-agent
    /// 3. unlayered > layered
    /// 4. later layer > earlier layer
    /// 5. higher specificity wins
    /// 6. later position wins
    fn sort_key(&self) -> (bool, u8, bool, usize, (u32, u32, u32), usize) {
        // 计算 origin 排序值
        // normal: author(2) > user(1) > ua(0)
        // important: ua(0) > user(1) > author(2) -- 注意反转
        let origin_priority = if self.important {
            // important 时：ua 最高
            match self.origin {
                Origin::UserAgent => 2,
                Origin::User => 1,
                Origin::Author => 0,
            }
        } else {
            // normal 时：author 最高
            match self.origin {
                Origin::Author => 2,
                Origin::User => 1,
                Origin::UserAgent => 0,
            }
        };

        // unlayered (None) > layered (Some)
        let is_unlayered = self.layer_index.is_none();
        let layer_idx = self.layer_index.unwrap_or(0);

        (
            self.important,   // 1. important > normal
            origin_priority,  // 2. 来源优先级
            is_unlayered,     // 3. unlayered > layered
            layer_idx,        // 4. later layer > earlier
            self.specificity, // 5. higher specificity
            self.position,    // 6. later position
        )
    }
}

impl PartialOrd for CascadeOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for CascadeOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

/// 样式声明（带级联信息）。
///
/// property/value 借用输入声明（计算样式热路径：每元素每属性省 2 次 String 分配，
/// 见 `cascade` 注释）。生命周期与来源声明（stylesheet AST / 展开列表）绑定。
#[derive(Debug, Clone)]
pub struct CascadedDeclaration<'a> {
    /// 属性名。
    pub property: &'a str,
    /// 属性值（原始字符串）。
    pub value: &'a str,
    /// 级联顺序。
    pub order: CascadeOrder,
}

/// 不允许负长度值的盒模型尺寸属性。
///
/// CSS 2.1 §8.3/§8.4（padding）、§10.2/§10.5（width/height）、§10.4（min-/max-）
/// 以及 §8.5.1（border-width）规定这些属性的负长度值非法，整条声明按未声明处理。
/// 与 `apply.rs` 的 border-*-width 负值拒绝同源；此处补全 width/height/min-/max-/padding，
/// 并在级联阶段处理——apply 阶段太晚（cascade 已丢弃较低优先级的合法回退声明，
/// 见 R512 numbers-units-006 机制分析）。
const NEGATIVE_ILLEGAL_PROPS: &[&str] = &[
    "width",
    "height",
    "inline-size",
    "block-size",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
];

/// 判定一条声明是否为「盒模型尺寸属性的负长度」（级联时应按未声明处理）。
///
/// 覆盖所有可解析为负值的长度单位：px（含 cm/in/mm/pt/pc/Q 等解析时转 px 的绝对单位）、
/// em/rem/ch、%。calc() 负值需另行求值，此处不涉及。非盒模型尺寸属性恒返回 false。
fn is_invalid_negative_length(property: &str, value: &str) -> bool {
    if !NEGATIVE_ILLEGAL_PROPS.contains(&property) {
        return false;
    }
    match parse_length(value) {
        Some(LengthValue::Px(p)) if p < 0.0 => true,
        Some(LengthValue::Em(e)) if e < 0.0 => true,
        Some(LengthValue::Rem(r)) if r < 0.0 => true,
        Some(LengthValue::Cap(c)) if c < 0.0 => true,
        Some(LengthValue::Rcap(c)) if c < 0.0 => true,
        Some(LengthValue::Ch(c)) if c < 0.0 => true,
        Some(LengthValue::Percentage(p)) if p < 0.0 => true,
        _ => false,
    }
}

/// 「有限关键字值」属性：值空间为固定关键词集合，`parse_X` 返回 `None` 即确定非法
/// （CSS 规范：值无法解析时整条声明作为解析错误丢弃）。
///
/// 这些属性的 `parse_X` 函数是权威整串匹配，且与 `apply.rs` 调用的是**同一个**函数
/// （均经 `zero_css_parser::values::parse_X`，定义于 `values/color.rs`）——故
/// `parse_X` 接受的值 `apply` 必能消费，`parse_X` 拒绝的 `apply` 必拒绝，零
/// false-positive。长度/颜色/简写属性值空间开放，不在此列。
const ENUM_KEYWORD_PROPS: &[&str] = &[
    "display",
    "position",
    "float",
    "clear",
    "overflow",
    "overflow-x",
    "overflow-y",
    "visibility",
    "box-sizing",
    "flex-direction",
    "flex-wrap",
    "column-count",
];

/// 判定一条声明是否为「有限关键字值属性的非法值」（级联时应按未声明处理）。
///
/// 与 `is_invalid_negative_length` 同理：在 apply 阶段拒绝太晚——cascade 已丢弃较低
/// 优先级的合法回退声明。例 `display: flex inline-flex`（无效双值）应回退到 UA 默认
/// `block`（保留已级联值），而非重置为初值 `inline`（CSS error-handling：非法声明
/// 忽略）。驱动案 css-flexbox/flexbox_display（19.99% diff）。无合法回退时属性不进入
/// 级联结果→初值/继承，与旧行为一致。
fn is_invalid_enum_value(property: &str, value: &str) -> bool {
    use zero_css_parser::values::{
        parse_box_sizing, parse_clear, parse_column_count, parse_display, parse_flex_direction, parse_flex_wrap,
        parse_float, parse_overflow, parse_position, parse_visibility,
    };
    if !ENUM_KEYWORD_PROPS.contains(&property) {
        return false;
    }
    let v = value.trim();
    match property {
        "display" => parse_display(v).is_none(),
        "position" => parse_position(v).is_none(),
        "float" => parse_float(v).is_none(),
        "clear" => parse_clear(v).is_none(),
        "overflow" | "overflow-x" | "overflow-y" => parse_overflow(v).is_none(),
        "visibility" => parse_visibility(v).is_none(),
        "box-sizing" => parse_box_sizing(v).is_none(),
        "flex-direction" => parse_flex_direction(v).is_none(),
        "flex-wrap" => parse_flex_wrap(v).is_none(),
        // R2066：column-count 须为 auto 或正整数（CSS Multicol §3）。
        // 非法值（-1 / 2.1 等）须在级联时按未声明处理——否则后到的非法声明因 order 更高
        // 胜出，apply 阶段 parse_column_count 返 None 不赋值，column_count 回退初值 Auto，
        // 覆盖前到的合法值（multicol-count-negative/non-integer：column-count:4 后跟 -1/2.1）。
        "column-count" => parse_column_count(v).is_none(),
        _ => false,
    }
}

/// 将遗留属性名别名规范化为标准名。
///
/// CSS 规范中部分属性有遗留别名，指向同一属性（同一 computed 字段、同一级联槽位）。
/// 必须在级联分组前规范化，否则别名名与标准名会成为**两个不同的级联键**——
/// 别名声明的值会被后续「该属性未出现在级联表」的继承逻辑用父值/初始值覆盖。
///
/// 当前映射（CSS Text 3 §5.3）：
/// - `word-wrap` → `overflow-wrap`（遗留别名，二者完全等价）
fn canonical_property_name(property: &str) -> &str {
    match property {
        "word-wrap" => "overflow-wrap",
        // CSS Logical Properties（sizing 族）canonical 化：logical 尺寸属性须规范化为
        // 水平书写模式的物理等价，使 logical 与 physical 声明在级联中按 specificity/order
        // 竞争（CSS Logical Props：computed value 由 logical+physical 共同声明的级联决定）。
        // 否则二者各占独立 canonical 槽位 → 都被应用 → specificity 丢失，终值由 apply 顺序/
        // HashMap 迭代序决定（错误）。垂直书写模式的轴交换由 converter swap_writing_mode_axes
        // 在 apply 之后处理（width↔height 互换），不受此处 canonical 化影响。
        // driving: css-logical/cascading-001。
        "inline-size" => "width",
        "block-size" => "height",
        "min-inline-size" => "min-width",
        "min-block-size" => "min-height",
        "max-inline-size" => "max-width",
        "max-block-size" => "max-height",
        // R2919：vendor 前缀 longhand 别名 canonical 化（legacy static web 兼容，DC-13 Tier 1）。
        // 2010s 老式静态页面大量用 `-webkit-` 前缀属性；parser 容忍 vendor 前缀（不丢弃声明），
        // 但未规范化前这些声明在 apply 阶段无匹配槽位 → 静默 no-op。此处把**值语法与标准
        // 完全一致、且标准 longhand 已全实现（apply+registry+inherit）的 1:1 别名**规范化为
        // 标准名，使其经 cascade 正常应用。仅显式列出经验证安全的别名——不通用 strip 前缀，
        // 因部分 `-webkit-` 属性值语法/语义与标准不同（如 `-webkit-background-clip` 历史接受
        // 无 `-box` 后缀的 `padding`/`border`，标准要求 `padding-box`），通用映射会致值解析
        // 失败。每个别名与标准名同槽位竞争（同优先级后者胜），与真实浏览器行为一致。
        "-webkit-user-select" => "user-select",
        "-webkit-appearance" => "appearance",
        "-webkit-box-shadow" => "box-shadow",
        "-webkit-background-size" => "background-size",
        // R2920：3D Transforms 簇 vendor 前缀 longhand 别名（值语法与标准完全一致；transform/
        // origin/perspective/backface/transform-style 均为 longhand，无 shorthand 展开路径，故
        // cascade canonical 点即可；不依赖外部资源故无 broken-image 风险）。早期移动端优化页
        // 常仅用 `-webkit-transform` 无 unprefixed 回退，此处使其生效。
        "-webkit-transform" => "transform",
        "-webkit-transform-origin" => "transform-origin",
        "-webkit-transform-style" => "transform-style",
        "-webkit-backface-visibility" => "backface-visibility",
        "-webkit-perspective" => "perspective",
        "-webkit-perspective-origin" => "perspective-origin",
        // R2921：`-webkit-line-clamp` → `line-clamp`（web-compat 遗留别名，值语法与标准
        // 完全一致——apply_advanced 已按别名处理，但未 canonical 化前与 `line-clamp` 各占
        // 独立槽位 → 同规则内两个声明双写 style.line_clamp，终值由 HashMap 迭代序（进程
        // 随机种子）决定 → line-clamp-019 结果进程级硬币翻转（64px/128px）。canonical 化后
        // 同槽位按 CascadeOrder（position）竞争，后声明胜，与真实浏览器一致。
        // driving: css-overflow/line-clamp/line-clamp-019。
        "-webkit-line-clamp" => "line-clamp",
        _ => property,
    }
}

/// 级联算法。
///
/// 接收一组级联声明，按属性名分组后，为每个属性选择优先级最高的声明。
///
/// # 返回值
///
/// 返回一个 HashMap，键为属性名，值为胜出的声明值。
// https://drafts.csswg.org/css-cascade-4/#cascading
pub fn cascade<'a>(declarations: Vec<CascadedDeclaration<'a>>, quirks: bool) -> HashMap<String, String> {
    // 按属性名分组（遗留别名先规范化为标准名——见 canonical_property_name）。
    // by_property 键借用声明自身的 property（&'a str，不克隆）——热路径每属性省 1 次
    // String 分配；分组内声明也是借用（构造侧已省克隆，见 collect_declarations）。
    let mut by_property: HashMap<&'a str, Vec<CascadedDeclaration<'a>>> = HashMap::new();
    for decl in declarations {
        // `all` 简写（CSS Cascading 4 §3.1 / CSS All 1）：值必须是 CSS-wide 关键字
        // （initial/inherit/unset/revert/revert-layer）。展开为对所有已知 longhand 属性的
        // 虚拟声明（**同一 order**），让既有 per-property max-by-order 级联自然解析
        // longhand-vs-`all` 优先级——同规则内 longhand 后于 `all` 声明则胜，先于则被 `all` 覆盖；
        // 高特异性规则的 `all` 胜过低特异性规则的 longhand。排除 `direction`/`unicode-bidi`
        // （CSS All 1 §3：`all` 不重置这两者）；自定义属性 `--*` 不在 known_properties，天然不受影响。
        // 未实现前 `all` 在此处被 apply 无识别当非法丢，从未生效。driving: css-cascade all-prop-*。
        // kill-switch `ZW_ALL_SHORTHAND=0`（default-on）关闭则退化为「`all` 不展开 = 旧无效果」。
        if decl.property.eq_ignore_ascii_case("all")
            && is_css_wide_keyword(decl.value)
            && std::env::var("ZW_ALL_SHORTHAND").as_deref() != Ok("0")
        {
            let order = decl.order.clone();
            for prop in PropertyRegistry::known_properties() {
                if matches!(*prop, "direction" | "unicode-bidi") {
                    continue;
                }
                by_property.entry(*prop).or_default().push(CascadedDeclaration {
                    property: prop,
                    value: decl.value,
                    order: order.clone(),
                });
            }
            continue;
        }
        let canonical = canonical_property_name(decl.property);
        by_property.entry(canonical).or_default().push(decl);
    }

    let mut result = HashMap::new();
    // kill-switch `ZW_REVERT_LAYER=0`（default-on）：进程运行中 env 不变，读一次。
    let revert_layer_active = std::env::var("ZW_REVERT_LAYER").as_deref() != Ok("0");

    for (property, decls) in by_property {
        // CSS 规范：非法声明（盒模型尺寸属性的负长度，或有限关键字值属性的非法值）
        // 在级联时即按未声明处理，故较低优先级的合法声明可胜出。仅选最高优先级的合法
        // 声明；若该属性全部声明均非法（全为负值/全为无法解析的关键字值等），属性不
        // 进入级联结果，回退到初始值（width/height→auto、max-*→none、display→inline 等，
        // 由 default_impl 提供，均为 CSS 规范初始值）。
        //
        // 惰性探测（性能）：合法性探测结果（bool）只取决于值能否解析，与 dummy 状态无关
        //（apply.rs 唯一读字段的 light-dark 取参不影响「能否解析」），故探测顺序不影响
        // 结果。Fast path：一遍线性扫描 + 探测，取「最高 order 的合法声明」（max-by-order，
        // 与旧「全量过滤 + 降序取首个」等价），**不排序、不分配**——常见页面每个属性仅
        // 1 个候选，或胜出者即最高优先级（只探测到胜出者为止，N 次 apply-on-dummy 全量
        // 解析 → 常见 1 次）。
        //
        // R2388：`revert-layer`（CSS Cascade 5 §6.1）须回退到「更低优先级层」的值——当
        // 某属性最高优先级合法声明是 revert-layer 时，跳过该声明所属 tier（同
        // origin+important+layer 的声明组），取下一更低优先级 tier 的胜出声明。revert-layer
        // 罕见 → 仅在检测到合法 revert-layer 声明时走 slow path（排序 + tier 回退），
        // 全部 tier 均 revert-layer → 保留最高声明值（即 revert-layer 关键字）交
        // inheritance.rs 按 ≈unset 解析（不破坏 R2386「lone revert-layer 通过 cascade」）。
        // kill-switch `ZW_REVERT_LAYER=0` 关闭时 revert-layer 是普通 CSS-wide 值，永不走
        // slow path。
        let mut dummy = ComputedStyle::default();
        let mut best: Option<&CascadedDeclaration> = None;
        let mut saw_revert_layer = false;
        for d in &decls {
            // CSS-wide 关键字（inherit/initial/unset/revert/revert-layer）对任何属性都合法，
            // 由 inheritance/compute pass 解析——须短路，否则会被 is_invalid_enum_value
            // 误判为非法（如 `display: initial` parse_display 返 None → 被丢，display 不重置）。
            let valid = is_css_wide_keyword(d.value)
                || (!is_invalid_negative_length(property, d.value)
                    && !is_invalid_enum_value(property, d.value)
                    && is_cascade_value_valid(property, d.value, quirks, &mut dummy));
            if !valid {
                continue;
            }
            if revert_layer_active && is_revert_layer_value(d.value) {
                saw_revert_layer = true;
                continue;
            }
            if best.is_none_or(|b| d.order > b.order) {
                best = Some(d);
            }
        }
        if !saw_revert_layer {
            if let Some(b) = best {
                result.insert(property.to_string(), b.value.to_string());
            }
            continue;
        }

        // Slow path（含合法 revert-layer）：降序探测 + tier 回退（语义见 effective_cascade_value 注释）。
        let mut sorted: Vec<&CascadedDeclaration> = decls.iter().collect();
        sorted.sort_by(|a, b| b.order.cmp(&a.order));
        let mut first_valid: Option<&CascadedDeclaration> = None;
        let mut winner: Option<&CascadedDeclaration> = None;
        let mut i = 0;
        while i < sorted.len() {
            let d = sorted[i];
            let valid = is_css_wide_keyword(d.value)
                || (!is_invalid_negative_length(property, d.value)
                    && !is_invalid_enum_value(property, d.value)
                    && is_cascade_value_valid(property, d.value, quirks, &mut dummy));
            if !valid {
                i += 1;
                continue;
            }
            if first_valid.is_none() {
                first_valid = Some(d);
            }
            if revert_layer_active && is_revert_layer_value(d.value) {
                // 跳过整个 tier（同 tier 的较低优先级声明亦属「本层」须一并移除，不再探测）。
                let tier = cascade_tier_key(&d.order);
                while i < sorted.len() && cascade_tier_key(&sorted[i].order) == tier {
                    i += 1;
                }
                continue;
            }
            winner = Some(d);
            break;
        }
        if let Some(w) = winner.or(first_valid) {
            result.insert(property.to_string(), w.value.to_string());
        }
    }

    result
}

/// 值是否为 `revert-layer` 关键字（R2388 tier 回退触发条件）。
fn is_revert_layer_value(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case("revert-layer")
}

/// 级联 tier 标识（origin + important + layer）——三者相同即同 tier。
fn cascade_tier_key(order: &CascadeOrder) -> (Origin, bool, Option<usize>) {
    (order.origin, order.important, order.layer_index)
}

/// 级联合法性探测：声明值能否被 apply 解析（drop-if-invalid 语义）。
///
/// 用 apply-on-dummy（调真实 `apply_property_value_with_quirks`）而非 `is_property_supported`，
/// 因后者用简化 parser（parse_length 不含 calc）会误丢合法值——apply-on-dummy 与生产 apply
/// 完全一致的解析路径（含 calc/min/max/clamp、quirks 宽容、shorthand 展开）。
///
/// **例外**（须保留，不丢）：
/// - 自定义属性 `--foo`：由 gather_custom_properties 处理（var() 解析源），apply 不识别。
/// - 含 `var(` 的值：var() 在 cascade 之后才解析（resolve_var_in_cascaded），此时未知合法
///   性，留给解析 + apply 阶段处理。
/// - CSS-wide 关键字（inherit/initial/unset/revert）：合法但由 inheritance/compute pass 处理
///   （非 apply 直接解析），apply 返 false（driving：max-width-104/max-height-104/height-inherit-001
///   `max-width: inherit` 等曾被误丢）。
///
/// driving：keywords-000（`background: "red"` string 值 apply 拒绝 → 丢，下个合法 green 胜出）。
/// 未知属性 apply 亦返 false → 丢（CSS：未知属性忽略；ZW 原本 apply 也忽略，渲染不变）。
fn is_cascade_value_valid(property: &str, value: &str, quirks: bool, dummy: &mut ComputedStyle) -> bool {
    if property.starts_with("--") || value.contains("var(") || is_css_wide_keyword(value) {
        return true;
    }
    // 有效性检查（apply-on-dummy）：color-scheme 合法性不依赖 prefers，传 light=false。
    apply_property_value_with_quirks(dummy, property, value, quirks, false)
}

/// 是否为 CSS-wide 关键字（合法，但非 apply 直接解析，由 inheritance/compute 处理）。
fn is_css_wide_keyword(value: &str) -> bool {
    let v = value.trim();
    v.eq_ignore_ascii_case("inherit")
        || v.eq_ignore_ascii_case("initial")
        || v.eq_ignore_ascii_case("unset")
        || v.eq_ignore_ascii_case("revert")
        || v.eq_ignore_ascii_case("revert-layer")
}

/// 从样式表中收集所有匹配的声明。
///
/// 返回一组带有级联信息的声明。
pub fn collect_declarations<'a>(
    declarations: &[(&'a str, &'a str, bool)], // (property, value, important)
    origin: Origin,
    layer_index: Option<usize>,
    specificity: (u32, u32, u32),
    base_position: usize,
) -> Vec<CascadedDeclaration<'a>> {
    declarations
        .iter()
        .enumerate()
        .map(|(i, (property, value, important))| CascadedDeclaration {
            property,
            value,
            order: CascadeOrder::new(origin, layer_index, specificity, base_position + i, *important),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascade_origin_priority_normal() {
        // normal: author > user > user-agent
        let ua = CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, false);
        let user = CascadeOrder::new(Origin::User, None, (0, 0, 0), 0, false);
        let author = CascadeOrder::new(Origin::Author, None, (0, 0, 0), 0, false);

        assert!(author > user);
        assert!(user > ua);
        assert!(author > ua);
    }

    #[test]
    fn test_canonical_property_name_word_wrap_alias() {
        // CSS Text 3 §5.3：word-wrap 是 overflow-wrap 的遗留别名，须规范化为同一属性。
        assert_eq!(canonical_property_name("word-wrap"), "overflow-wrap");
        // 非别名属性原样返回。
        assert_eq!(canonical_property_name("overflow-wrap"), "overflow-wrap");
        assert_eq!(canonical_property_name("color"), "color");
        assert_eq!(canonical_property_name("font-size"), "font-size");
    }

    #[test]
    fn test_cascade_normalizes_word_wrap_to_overflow_wrap() {
        // word-wrap 声明经 cascade() 后须以标准名 overflow-wrap 出现在结果中，
        // 而非作为独立键 word-wrap（否则会被继承逻辑当「未声明」用父值覆盖——见 canonical_property_name 注释）。
        let decls = vec![CascadedDeclaration {
            property: "word-wrap",
            value: "break-word",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        // 标准名存在，别名名不存在。
        assert_eq!(result.get("overflow-wrap"), Some(&"break-word".to_string()));
        assert!(!result.contains_key("word-wrap"));
    }

    #[test]
    fn test_cascade_word_wrap_and_overflow_wrap_share_slot() {
        // word-wrap 与 overflow-wrap 是同一属性：同优先级下后声明者胜，而非两个独立键各取其值。
        let decls = vec![
            CascadedDeclaration {
                property: "overflow-wrap",
                value: "normal",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "word-wrap",
                value: "break-word",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        // 同一槽位：后声明的 word-wrap:break-word 胜出。
        assert_eq!(result.get("overflow-wrap"), Some(&"break-word".to_string()));
        assert!(!result.contains_key("word-wrap"));
    }

    /// R2499：CSS Logical Properties sizing 族 canonical 化——logical 尺寸属性须规范化为
    /// 其水平书写模式的物理等价，使 logical 与 physical 在级联中按 specificity 竞争。
    /// driving: css-logical/cascading-001（physical+logical 同元素须 cascade 共解）。
    #[test]
    fn test_canonical_logical_size_aliases() {
        // inline-size↔width、block-size↔height 及 min/max 变体。
        assert_eq!(canonical_property_name("inline-size"), "width");
        assert_eq!(canonical_property_name("block-size"), "height");
        assert_eq!(canonical_property_name("min-inline-size"), "min-width");
        assert_eq!(canonical_property_name("min-block-size"), "min-height");
        assert_eq!(canonical_property_name("max-inline-size"), "max-width");
        assert_eq!(canonical_property_name("max-block-size"), "max-height");
        // 物理名原样返回（已 canonical）。
        assert_eq!(canonical_property_name("width"), "width");
        assert_eq!(canonical_property_name("height"), "height");
    }

    /// R2919：vendor 前缀 longhand 别名 canonical 化（legacy static web 兼容，DC-13 Tier 1）。
    /// `-webkit-` 前缀属性值语法与标准完全一致的 1:1 longhand 别名规范化为标准名，使其经
    /// cascade 正常应用（否则 apply 无匹配槽位静默 no-op）。driving：2010s 老式静态页面用
    /// `-webkit-user-select`/`-webkit-appearance`/`-webkit-box-shadow`/`-webkit-background-size`。
    #[test]
    fn test_canonical_webkit_vendor_aliases() {
        // 4 个安全 1:1 longhand 别名 → 标准名。
        assert_eq!(canonical_property_name("-webkit-user-select"), "user-select");
        assert_eq!(canonical_property_name("-webkit-appearance"), "appearance");
        assert_eq!(canonical_property_name("-webkit-box-shadow"), "box-shadow");
        assert_eq!(canonical_property_name("-webkit-background-size"), "background-size");
        // R2920：3D Transforms 簇（值语法与标准一致，均为 longhand）。
        assert_eq!(canonical_property_name("-webkit-transform"), "transform");
        assert_eq!(canonical_property_name("-webkit-transform-origin"), "transform-origin");
        assert_eq!(canonical_property_name("-webkit-transform-style"), "transform-style");
        assert_eq!(
            canonical_property_name("-webkit-backface-visibility"),
            "backface-visibility"
        );
        assert_eq!(canonical_property_name("-webkit-perspective"), "perspective");
        assert_eq!(
            canonical_property_name("-webkit-perspective-origin"),
            "perspective-origin"
        );
        // 标准名原样返回（已 canonical）。
        assert_eq!(canonical_property_name("user-select"), "user-select");
        assert_eq!(canonical_property_name("box-shadow"), "box-shadow");
        // 非安全别名（值语法/语义与标准不同）不规范化——保持 no-op，避免值解析失败。
        assert_eq!(
            canonical_property_name("-webkit-background-clip"),
            "-webkit-background-clip"
        );
        assert_eq!(
            canonical_property_name("-webkit-text-fill-color"),
            "-webkit-text-fill-color"
        );
        // 未列入的任意 vendor 前缀属性原样返回。
        assert_eq!(canonical_property_name("-webkit-foo"), "-webkit-foo");
    }

    /// R2919：`-webkit-` 别名经 cascade 后须以标准名出现在结果中（而非作为独立键被继承逻辑
    /// 当「未声明」覆盖——见 canonical_property_name 注释）。
    #[test]
    fn test_cascade_normalizes_webkit_alias_to_standard() {
        let decls = vec![CascadedDeclaration {
            property: "-webkit-user-select",
            value: "none",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        // 标准名存在，别名名不存在。
        assert_eq!(result.get("user-select"), Some(&"none".to_string()));
        assert!(!result.contains_key("-webkit-user-select"));
    }

    /// R2919：`-webkit-` 别名与标准名共享同一 canonical 槽位——同优先级下后声明者胜，
    /// 而非两个独立键各取其值（与 word-wrap/overflow-wrap 同语义）。
    #[test]
    fn test_cascade_webkit_alias_shares_slot_with_standard() {
        let decls = vec![
            CascadedDeclaration {
                property: "box-shadow",
                value: "none",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "-webkit-box-shadow",
                value: "1px 1px red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        // 同一槽位：后声明的 -webkit-box-shadow 胜出。
        assert_eq!(result.get("box-shadow"), Some(&"1px 1px red".to_string()));
        assert!(!result.contains_key("-webkit-box-shadow"));
    }

    /// R2499：logical 与 physical 尺寸属性共享同一 canonical 槽位 → 按 specificity 竞争，
    /// 而非各占独立槽位双双应用（致 specificity 丢失、终值由 apply/HashMap 序决定）。
    /// driving: css-logical/cascading-001（class inline-size:100px vs ID width:10px → ID 须胜）。
    #[test]
    fn test_cascade_logical_and_physical_size_share_slot() {
        // class（specificity 0,1,0）inline-size:100px vs ID（1,0,0）width:10px → ID 胜。
        let decls = vec![
            CascadedDeclaration {
                property: "inline-size",
                value: "100px",
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false),
            },
            CascadedDeclaration {
                property: "width",
                value: "10px",
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 1, false),
            },
        ];
        let result = cascade(decls, false);
        // canonical "width" 槽位 = ID 的 10px；无残留 "inline-size" 键。
        assert_eq!(
            result.get("width"),
            Some(&"10px".to_string()),
            "ID width:10px must win over class inline-size:100px (logical+physical compete by specificity)"
        );
        assert!(!result.contains_key("inline-size"));

        // 反向：class width:10px vs ID inline-size:100px → ID inline-size 胜（canonical width=100px）。
        let decls_rev = vec![
            CascadedDeclaration {
                property: "width",
                value: "10px",
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false),
            },
            CascadedDeclaration {
                property: "inline-size",
                value: "100px",
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 1, false),
            },
        ];
        let result_rev = cascade(decls_rev, false);
        assert_eq!(
            result_rev.get("width"),
            Some(&"100px".to_string()),
            "ID inline-size:100px must win over class width:10px"
        );
        assert!(!result_rev.contains_key("inline-size"));
    }

    /// R2386：`all` 简写展开为 CSS-wide 关键字——`all: initial` 把 color 重置为 initial。
    #[test]
    fn test_all_shorthand_initial_resets_color() {
        let decls = vec![CascadedDeclaration {
            property: "all",
            value: "initial",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"initial".to_string()));
        assert_eq!(result.get("display"), Some(&"initial".to_string()));
    }

    /// R2386：`all` 接受全部 CSS-wide 关键字（inherit/unset/revert/revert-layer）。
    #[test]
    fn test_all_shorthand_accepts_all_wide_keywords() {
        for kw in ["inherit", "unset", "revert", "revert-layer", "INITIAL"] {
            let decls = vec![CascadedDeclaration {
                property: "all",
                value: kw,
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            }];
            let result = cascade(decls, false);
            assert_eq!(
                result.get("color").map(String::as_str),
                Some(kw),
                "all: {kw} 应展开为 color:{kw}"
            );
        }
    }

    /// R2386：`all` 不重置 `direction` / `unicode-bidi`（CSS All 1 §3 排除项）。
    #[test]
    fn test_all_shorthand_excludes_direction_and_unicode_bidi() {
        let decls = vec![CascadedDeclaration {
            property: "all",
            value: "initial",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        assert!(
            !result.contains_key("direction"),
            "all 不应重置 direction（CSS All 1 §3 排除）"
        );
        assert!(
            !result.contains_key("unicode-bidi"),
            "all 不应重置 unicode-bidi（CSS All 1 §3 排除）"
        );
    }

    /// R2386：同规则内 longhand 后于 `all` 声明则胜出（source order 更大）。
    #[test]
    fn test_all_shorthand_longhand_after_all_wins() {
        // `all: initial; color: red;` — color(i=1) order > all(i=0) order → color:red 胜。
        let decls = vec![
            CascadedDeclaration {
                property: "all",
                value: "initial",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
        // 其余属性仍被 all 重置。
        assert_eq!(result.get("display"), Some(&"initial".to_string()));
    }

    /// R2386：同规则内 longhand 先于 `all` 声明则被 `all` 覆盖。
    #[test]
    fn test_all_shorthand_longhand_before_all_loses() {
        // `color: red; all: initial;` — all(i=1) order > color(i=0) order → color:initial 胜。
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "all",
                value: "initial",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(
            result.get("color"),
            Some(&"initial".to_string()),
            "all 在 color 之后声明应覆盖 color"
        );
    }

    /// R2386：`all` 非 CSS-wide 关键字值（如 `all: red`）按规范忽略，不展开、不影响任何属性。
    #[test]
    fn test_all_shorthand_non_keyword_value_ignored() {
        let decls = vec![CascadedDeclaration {
            property: "all",
            value: "red",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        // 非 keyword 的 all 被丢，无任何属性被设置。
        assert!(result.get("color").is_none());
        assert!(result.get("display").is_none());
    }

    /// R2386：端到端——父 color:red，子 `all: initial` 经 inheritance 解析为初始黑。
    #[test]
    fn test_all_shorthand_end_to_end_initial_resets_inherited_color() {
        use crate::inheritance::compute_inherited_style;
        use crate::property::ComputedStyle;
        use zero_css_parser::values::ColorValue;

        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(255, 0, 0, 255); // red

        let mut cascaded = std::collections::HashMap::new();
        cascaded.insert("color".to_string(), "initial".to_string());
        cascaded.insert("display".to_string(), "initial".to_string());

        let child = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(child.color, ColorValue::Rgba(0, 0, 0, 255)); // initial = black
        assert_eq!(child.display, zero_css_parser::values::DisplayValue::Inline);
    }

    /// R2386：`revert-layer` 作为 longhand 值不再被 cascade 丢弃（latent fix：is_css_wide_keyword 补 revert-layer）。
    #[test]
    fn test_revert_layer_longhand_not_dropped_at_cascade() {
        let decls = vec![CascadedDeclaration {
            property: "color",
            value: "revert-layer",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        assert_eq!(
            result.get("color"),
            Some(&"revert-layer".to_string()),
            "revert-layer longhand 应通过 cascade（此前 is_css_wide_keyword 漏列被丢）"
        );
    }

    /// R2388：`revert-layer` 回退到更低优先级层的值——两个 @layer，高层 revert-layer
    /// 应回退到低层的 concrete 值（driving: revert-layer-001 green 而非 red）。
    #[test]
    fn test_revert_layer_falls_back_to_lower_layer() {
        // layer 0（低优先级）：green；layer 1（高优先级）：revert-layer → 应取 layer 0 的 green。
        let decls = vec![
            CascadedDeclaration {
                property: "background-color",
                value: "green",
                order: CascadeOrder::new(Origin::Author, Some(0), (1, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "background-color",
                value: "revert-layer",
                order: CascadeOrder::new(Origin::Author, Some(1), (1, 0, 0), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(
            result.get("background-color"),
            Some(&"green".to_string()),
            "revert-layer 应回退到更低优先级层的 green"
        );
    }

    /// R2388：高层 tier 为 concrete 值时不回退（revert-layer 仅当胜出声明本身是 revert-layer 才触发）。
    #[test]
    fn test_revert_layer_not_triggered_when_higher_layer_concrete() {
        let decls = vec![
            CascadedDeclaration {
                property: "background-color",
                value: "green",
                order: CascadeOrder::new(Origin::Author, Some(0), (1, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "background-color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, Some(1), (1, 0, 0), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("background-color"), Some(&"red".to_string()));
    }

    /// R2388：unlayered 声明优先级高于 layered——unlayered revert-layer 回退到 layered 值。
    #[test]
    fn test_revert_layer_unlayered_falls_to_layered() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "blue",
                order: CascadeOrder::new(Origin::Author, Some(0), (1, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "revert-layer",
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(
            result.get("color"),
            Some(&"blue".to_string()),
            "unlayered revert-layer 应回退到 layered 的 blue"
        );
    }

    /// R2388：revert-layer 无更低层可回退 → 保留 revert-layer 关键字交 inheritance 解析
    ///（≈unset；与无更低层时回退到上一 origin≈initial 等价，且不破坏 R2386 lone-revert-layer 语义）。
    #[test]
    fn test_revert_layer_no_lower_layer_kept_for_inheritance() {
        let decls = vec![CascadedDeclaration {
            property: "background-color",
            value: "revert-layer",
            order: CascadeOrder::new(Origin::Author, Some(0), (1, 0, 0), 0, false),
        }];
        let result = cascade(decls, false);
        assert_eq!(
            result.get("background-color"),
            Some(&"revert-layer".to_string()),
            "无更低层时保留 revert-layer 关键字交 inheritance"
        );
    }

    /// R2388：revert-layer 链式回退（高层 revert-layer → 中层 revert-layer → 低层 concrete）。
    #[test]
    fn test_revert_layer_chained() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "green",
                order: CascadeOrder::new(Origin::Author, Some(0), (1, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "revert-layer",
                order: CascadeOrder::new(Origin::Author, Some(1), (1, 0, 0), 1, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "revert-layer",
                order: CascadeOrder::new(Origin::Author, Some(2), (1, 0, 0), 2, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(
            result.get("color"),
            Some(&"green".to_string()),
            "链式 revert-layer 应一路回退到最低层的 green"
        );
    }

    /// R2126：apply-on-dummy 合法性探测——非法值声明按未声明处理，较低优先级合法声明胜出。
    #[test]
    fn test_cascade_drops_invalid_value_lower_priority_wins() {
        // keywords-001 模式：`color: "red"`（string 值 apply 拒绝）应被丢，
        // 早先合法的 green 胜出（同特异性，后声明无效不影响早合法）。
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "green",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "\"red\"",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"green".to_string()));
    }

    /// R2126：CSS-wide 关键字（inherit/initial/unset/revert）合法，须保留不丢。
    #[test]
    fn test_cascade_keeps_css_wide_keywords() {
        for kw in ["inherit", "initial", "unset", "revert", "INHERIT"] {
            let decls = vec![CascadedDeclaration {
                property: "max-width",
                value: kw,
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            }];
            let result = cascade(decls, false);
            assert_eq!(result.get("max-width"), Some(&kw.to_string()), "kw {kw} must be kept");
        }
    }

    /// R2126：var() 与自定义属性须保留（var() 后续解析；--foo 由 gather_custom_properties 处理）。
    #[test]
    fn test_cascade_keeps_var_and_custom_property() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "var(--c)",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "--c",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"var(--c)".to_string()));
        assert_eq!(result.get("--c"), Some(&"red".to_string()));
    }

    #[test]
    fn test_cascade_origin_priority_important() {
        // important: user-agent > user > author (反转)
        let ua = CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, true);
        let user = CascadeOrder::new(Origin::User, None, (0, 0, 0), 0, true);
        let author = CascadeOrder::new(Origin::Author, None, (0, 0, 0), 0, true);

        assert!(ua > user);
        assert!(user > author);
        assert!(ua > author);
    }

    #[test]
    fn test_cascade_important_beats_normal() {
        let normal = CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, false);
        let important = CascadeOrder::new(Origin::Author, None, (0, 0, 0), 0, true);

        assert!(important > normal);
    }

    #[test]
    fn test_cascade_specificity() {
        let low = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false);
        let high = CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, false);

        assert!(high > low);
    }

    #[test]
    fn test_cascade_position() {
        let early = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false);
        let late = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false);

        assert!(late > early);
    }

    #[test]
    fn test_cascade_unlayered_beats_layered() {
        let layered = CascadeOrder::new(Origin::Author, Some(0), (0, 0, 1), 0, false);
        let unlayered = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false);

        assert!(unlayered > layered);
    }

    #[test]
    fn test_cascade_later_layer_beats_earlier() {
        let early = CascadeOrder::new(Origin::Author, Some(0), (0, 0, 1), 0, false);
        let late = CascadeOrder::new(Origin::Author, Some(1), (0, 0, 1), 0, false);

        assert!(late > early);
    }

    #[test]
    fn test_cascade_basic() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "blue",
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 1, false),
            },
        ];

        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"blue".to_string()));
    }

    #[test]
    fn test_cascade_skips_invalid_negative_with_valid_fallback() {
        // CSS §10.5：height 负值非法。height:1in; height:-1px 中 -1px 优先级更高但非法，
        // 应回退到合法的 1in（numbers-units-006 场景）。
        let decls = vec![
            CascadedDeclaration {
                property: "height",
                value: "1in",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
            CascadedDeclaration {
                property: "height",
                value: "-1px",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 2, false),
            },
        ];

        let result = cascade(decls, false);
        assert_eq!(result.get("height"), Some(&"1in".to_string()));
    }

    #[test]
    fn test_cascade_sole_negative_rejected_to_initial() {
        // 仅有负值声明时全为非法 → 属性不进入级联结果（回退到初始值 width→auto）。
        let decls = vec![CascadedDeclaration {
            property: "width",
            value: "-5px",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];

        let result = cascade(decls, false);
        assert!(result.get("width").is_none());
    }

    #[test]
    fn test_cascade_negative_em_percent_rejected() {
        // em/%/ch 负长度同样非法（height-089 / max-width-089 / max-height-067 等）。
        let decls = vec![
            CascadedDeclaration {
                property: "max-width",
                value: "-10%",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "padding-top",
                value: "-2em",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];

        let result = cascade(decls, false);
        assert!(result.get("max-width").is_none());
        assert!(result.get("padding-top").is_none());
    }

    #[test]
    fn test_cascade_negative_check_only_box_model() {
        // 非盒模型尺寸属性（如 margin-top 允许负值）不受影响。
        let decls = vec![CascadedDeclaration {
            property: "margin-top",
            value: "-5px",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];

        let result = cascade(decls, false);
        assert_eq!(result.get("margin-top"), Some(&"-5px".to_string()));
    }

    #[test]
    fn test_cascade_skips_invalid_enum_with_valid_fallback() {
        // CSS error-handling：display 无效值（如 `flex inline-flex` 双值、`bogus`、
        // `flex extra junk` 尾部垃圾）整条声明丢弃，较低优先级合法声明胜出。
        // 驱动案 css-flexbox/flexbox_display：UA `display:block` (0,0,0) 应胜过作者
        // `display: flex inline-flex` (0,0,1) 的非法声明（旧实现重置为初值 inline）。
        let decls = vec![
            CascadedDeclaration {
                property: "display",
                value: "block",
                order: CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "display",
                value: "flex inline-flex",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("display"), Some(&"block".to_string()));
    }

    #[test]
    fn test_cascade_sole_invalid_enum_rejected_to_initial() {
        // 仅有非法 enum 声明 → 属性不进入级联结果（display 回退初值 inline）。
        let decls = vec![CascadedDeclaration {
            property: "display",
            value: "bogus",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        assert!(result.get("display").is_none());
    }

    #[test]
    fn test_cascade_invalid_enum_variants_and_other_props() {
        // 有效值+尾部垃圾（`flex extra junk`）整条声明丢弃 → 回退 UA block。
        let decls = vec![
            CascadedDeclaration {
                property: "display",
                value: "block",
                order: CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "display",
                value: "flex extra junk",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        assert_eq!(cascade(decls, false).get("display"), Some(&"block".to_string()));

        // 其它 enum 属性同理：position 无效值回退 UA static。
        let decls = vec![
            CascadedDeclaration {
                property: "position",
                value: "static",
                order: CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "position",
                value: "definitely-not-a-position",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        assert_eq!(cascade(decls, false).get("position"), Some(&"static".to_string()));

        // 有效 enum 值不被误拒（display:flex、float:left、overflow:hidden 均保留）。
        let decls = vec![
            CascadedDeclaration {
                property: "display",
                value: "flex",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "float",
                value: "left",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
            CascadedDeclaration {
                property: "overflow-x",
                value: "hidden",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 2, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("display"), Some(&"flex".to_string()));
        assert_eq!(result.get("float"), Some(&"left".to_string()));
        assert_eq!(result.get("overflow-x"), Some(&"hidden".to_string()));
    }

    #[test]
    fn test_cascade_multiple_properties() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "display",
                value: "flex",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];

        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
        assert_eq!(result.get("display"), Some(&"flex".to_string()));
    }

    #[test]
    fn test_cascade_important_wins_over_specificity() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, true),
            },
            CascadedDeclaration {
                property: "color",
                value: "blue",
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 1, false),
            },
        ];

        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
    }

    #[test]
    fn test_cascade_empty() {
        let result = cascade(vec![], false);
        assert!(result.is_empty());
    }

    /// R2921：`-webkit-line-clamp` 与 `line-clamp` 是同一属性的别名（值语法一致）。
    /// 未 canonical 化前二者各占独立槽位 → 同规则双声明都被应用、终值由 HashMap
    /// 迭代序（进程随机种子）决定 → line-clamp-019 结果进程级硬币翻转。
    /// canonical 化后同槽位按 CascadeOrder（position）竞争，**后声明者胜**。
    #[test]
    fn test_cascade_webkit_line_clamp_canonicalized_to_line_clamp() {
        // 同规则同 specificity：`line-clamp: 2` 先声明、`-webkit-line-clamp: 4` 后声明。
        let decls = vec![("line-clamp", "2", false), ("-webkit-line-clamp", "4", false)];
        let order = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false);
        let cascaded = collect_declarations(&decls, Origin::Author, None, (0, 1, 0), 0);
        assert_eq!(cascaded.len(), 2);
        assert_eq!(cascaded[0].order, order);
        let result = cascade(cascaded, false);
        // 归一化为单槽位（后声明胜），而非两个 key 并存。
        assert_eq!(result.len(), 1, "别名须归一化为单一槽位：{result:?}");
        assert_eq!(result.get("line-clamp").map(String::as_str), Some("4"));
        assert!(result.get("-webkit-line-clamp").is_none());
    }

    #[test]
    fn test_collect_declarations() {
        let decls = vec![("color", "red", false), ("display", "block", true)];

        let cascaded = collect_declarations(&decls, Origin::Author, None, (0, 1, 0), 0);
        assert_eq!(cascaded.len(), 2);
        assert_eq!(cascaded[0].property, "color");
        assert!(!cascaded[0].order.important);
        assert!(cascaded[1].order.important);
    }

    #[test]
    fn test_cascade_author_important_beats_ua_normal() {
        let ua_normal = CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, false);
        let author_important = CascadeOrder::new(Origin::Author, None, (0, 0, 0), 0, true);
        // important 胜过 normal（无论来源）
        assert!(author_important > ua_normal);
    }

    #[test]
    fn test_cascade_ua_important_beats_author_important() {
        let author_important = CascadeOrder::new(Origin::Author, None, (0, 0, 0), 0, true);
        let ua_important = CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, true);
        // important 时 UA > Author
        assert!(ua_important > author_important);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增级联特异性测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// ID 选择器特异性 (1,0,0) 胜过类选择器 (0,1,0)
    fn test_cascade_id_beats_class() {
        let class_sel = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false);
        let id_sel = CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, false);
        assert!(id_sel > class_sel);
    }

    #[test]
    /// 多个类选择器 (0,2,0) 胜过单个类选择器 (0,1,0)
    fn test_cascade_multiple_classes_beats_single() {
        let single = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false);
        let double = CascadeOrder::new(Origin::Author, None, (0, 2, 0), 0, false);
        assert!(double > single);
    }

    #[test]
    /// !important 胜过所有选择器特异性
    fn test_cascade_important_overrides_specificity() {
        let high_spec = CascadeOrder::new(Origin::Author, None, (1, 0, 0), 10, false);
        let low_spec_important = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, true);
        assert!(low_spec_important > high_spec);
    }

    #[test]
    /// @layer 最后层胜过前面层（同优先级）
    fn test_cascade_layer_ordering() {
        let layer_0 = CascadeOrder::new(Origin::Author, Some(0), (0, 0, 1), 0, false);
        let layer_1 = CascadeOrder::new(Origin::Author, Some(1), (0, 0, 1), 0, false);
        let layer_2 = CascadeOrder::new(Origin::Author, Some(2), (0, 0, 1), 0, false);
        assert!(layer_2 > layer_1);
        assert!(layer_1 > layer_0);
        assert!(layer_2 > layer_0);
    }

    #[test]
    /// 属性选择器特异性 (0,1,0) 等于类选择器
    fn test_cascade_attribute_vs_class_specificity() {
        // 属性选择器 [type=text] 的特异性为 (0,1,0)，与 .text 相同
        // 同特异性时，后面的声明胜出
        let attr_sel = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false);
        let class_sel = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 1, false);
        assert!(class_sel > attr_sel); // later position wins
    }

    #[test]
    /// cascade 函数测试：ID 选择器胜过类选择器
    fn test_cascade_function_id_vs_class() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "blue",
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"red".to_string())); // ID wins
    }

    #[test]
    /// 同优先级下位置靠后的声明胜出
    fn test_cascade_same_specificity_later_wins() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "green",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"green".to_string()));
    }

    #[test]
    /// collect_declarations 带 layer_index
    fn test_collect_declarations_with_layer() {
        let decls = vec![("color", "red", false)];
        let cascaded = collect_declarations(&decls, Origin::Author, Some(2), (0, 1, 0), 10);
        assert_eq!(cascaded.len(), 1);
        assert_eq!(cascaded[0].order.layer_index, Some(2));
        assert_eq!(cascaded[0].order.position, 10);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增级联边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 同优先级下后声明覆盖前声明
    fn test_same_specificity_later_decl_wins() {
        let decls = vec![
            CascadedDeclaration {
                property: "margin-top",
                value: "10px",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "margin-top",
                value: "20px",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("margin-top"), Some(&"20px".to_string()));
    }

    #[test]
    /// !important 覆盖正常声明（即使特异性更低）
    fn test_important_overrides_normal_declaration() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "blue",
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, true),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
    }

    #[test]
    /// specificity: ID > class > type
    fn test_specificity_id_gt_class_gt_type() {
        let type_sel = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false);
        let class_sel = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false);
        let id_sel = CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, false);
        assert!(id_sel > class_sel);
        assert!(class_sel > type_sel);
        assert!(id_sel > type_sel);
    }

    #[test]
    /// 同优先级同特异性时，位置靠后的声明胜出
    fn test_later_position_wins_at_same_specificity() {
        let early = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 5, false);
        let late = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 10, false);
        assert!(late > early);
    }

    #[test]
    /// 作者来源 important 仍然低于 UA 来源 important
    fn test_author_important_below_ua_important() {
        let ua_imp = CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, true);
        let author_imp = CascadeOrder::new(Origin::Author, None, (1, 0, 0), 10, true);
        assert!(ua_imp > author_imp);
    }

    #[test]
    /// 用户来源 important 在作者来源 important 和 UA 来源 important 之间
    fn test_user_important_between_ua_and_author() {
        let ua_imp = CascadeOrder::new(Origin::UserAgent, None, (0, 0, 0), 0, true);
        let user_imp = CascadeOrder::new(Origin::User, None, (0, 0, 0), 0, true);
        let author_imp = CascadeOrder::new(Origin::Author, None, (0, 0, 0), 0, true);
        assert!(ua_imp > user_imp);
        assert!(user_imp > author_imp);
    }

    #[test]
    /// 未分层声明胜过分层声明
    fn test_unlayered_always_beats_layered() {
        let layered = CascadeOrder::new(Origin::Author, Some(5), (1, 0, 0), 10, false);
        let unlayered = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false);
        assert!(unlayered > layered);
    }

    #[test]
    /// 三个不同属性在级联中各自独立
    fn test_cascade_three_distinct_properties() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "display",
                value: "flex",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
            CascadedDeclaration {
                property: "opacity",
                value: "0.5",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 2, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
        assert_eq!(result.get("display"), Some(&"flex".to_string()));
        assert_eq!(result.get("opacity"), Some(&"0.5".to_string()));
    }

    #[test]
    /// cascade 对同一属性的多个声明只保留一个胜者
    fn test_cascade_single_winner_per_property() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "green",
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 1, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "blue",
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 2, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("color"), Some(&"blue".to_string()));
    }

    #[test]
    /// CascadeOrder 相等时比较结果
    fn test_cascade_order_equality() {
        let a = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 5, false);
        let b = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 5, false);
        assert_eq!(a, b);
        assert!(a >= b);
        assert!(a <= b);
    }

    #[test]
    /// collect_declarations 递增位置
    fn test_collect_declarations_position_increment() {
        let decls = vec![("a", "1", false), ("b", "2", true), ("c", "3", false)];
        let cascaded = collect_declarations(&decls, Origin::Author, None, (0, 0, 1), 100);
        assert_eq!(cascaded[0].order.position, 100);
        assert_eq!(cascaded[1].order.position, 101);
        assert_eq!(cascaded[2].order.position, 102);
    }

    #[test]
    /// 分层内 important 胜过未分层 normal
    fn test_layered_important_beats_unlayered_normal() {
        let layered_imp = CascadeOrder::new(Origin::Author, Some(0), (0, 0, 1), 0, true);
        let unlayered_normal = CascadeOrder::new(Origin::Author, None, (1, 0, 0), 10, false);
        assert!(layered_imp > unlayered_normal);
    }

    #[test]
    /// 用户来源 normal 胜过 UA 来源 normal
    fn test_user_normal_beats_ua_normal() {
        let ua = CascadeOrder::new(Origin::UserAgent, None, (1, 0, 0), 0, false);
        let user = CascadeOrder::new(Origin::User, None, (0, 0, 1), 0, false);
        assert!(user > ua);
    }

    // ── 新增边界测试 ──

    #[test]
    /// cascade 函数空声明返回空 map
    fn test_cascade_empty_declarations() {
        let result = cascade(vec![], false);
        assert!(result.is_empty(), "空声明应返回空 map");
    }

    #[test]
    /// cascade 函数单一声明返回正确值
    fn test_cascade_single_declaration() {
        let decls = vec![CascadedDeclaration {
            property: "color",
            value: "red",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
    }

    #[test]
    /// R2066：column-count 非法值（负数 / 非整数）须在级联时按未声明处理——
    /// 后到的非法声明不得胜出覆盖前到的合法值（CSS Multicol §3 + CSS error-handling）。
    fn test_cascade_column_count_invalid_filtered() {
        // column-count: 4; column-count: -1;  → 4 胜出（-1 非法被过滤）
        let decls = vec![
            CascadedDeclaration {
                property: "column-count",
                value: "4",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "column-count",
                value: "-1",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(
            result.get("column-count"),
            Some(&"4".to_string()),
            "非法 column-count:-1 应被过滤，保留合法 column-count:4"
        );

        // column-count: 4; column-count: 2.1;  → 4 胜出（2.1 非整数非法）
        let decls = vec![
            CascadedDeclaration {
                property: "column-count",
                value: "4",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "column-count",
                value: "2.1",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(
            result.get("column-count"),
            Some(&"4".to_string()),
            "非整数 column-count:2.1 应被过滤，保留合法 column-count:4"
        );

        // 全非法 → 属性不进级联结果（回退初值）
        let decls = vec![CascadedDeclaration {
            property: "column-count",
            value: "-5",
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls, false);
        assert!(
            !result.contains_key("column-count"),
            "全非法 column-count 应不进级联结果（回退初值 Auto）"
        );
    }

    #[test]
    /// cascade 多属性各自保留最高优先级值（补充验证）
    fn test_cascade_multiple_properties_comprehensive() {
        let decls = vec![
            CascadedDeclaration {
                property: "color",
                value: "red",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color",
                value: "blue",
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false),
            },
            CascadedDeclaration {
                property: "display",
                value: "block",
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
        ];
        let result = cascade(decls, false);
        assert_eq!(result.get("color"), Some(&"blue".to_string()), "高特异性胜出");
        assert_eq!(result.get("display"), Some(&"block".to_string()));
    }

    #[test]
    /// 同层级下后出现的声明胜出（位置递增）
    fn test_cascade_position_ordering() {
        let first = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false);
        let second = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false);
        assert!(second > first, "后出现的声明应胜出");
    }

    #[test]
    /// 高特异性胜过低特异性
    fn test_cascade_specificity_ordering() {
        let id_sel = CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, false);
        let class_sel = CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false);
        let type_sel = CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false);
        assert!(id_sel > class_sel, "ID 选择器应胜过类选择器");
        assert!(class_sel > type_sel, "类选择器应胜过类型选择器");
    }
}
