//! CSS 级联算法。
//!
//! 实现级联优先级排序：按来源、!important、@layer、specificity、出现顺序
//! 决定每个属性的最终胜出声明。

use std::collections::HashMap;

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
#[derive(Debug, Clone)]
pub struct CascadedDeclaration {
    /// 属性名。
    pub property: String,
    /// 属性值（原始字符串）。
    pub value: String,
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

/// 判定一条声明是否为「盒模型尺寸属性的负 px 长度」（级联时应按未声明处理）。
///
/// 仅检测 px（与 `apply.rs` border-*-width 一致）；em/%/calc 等负值由下游解析处理，
/// 此处不涉及。非盒模型尺寸属性恒返回 false。
fn is_invalid_negative_length(property: &str, value: &str) -> bool {
    if !NEGATIVE_ILLEGAL_PROPS.contains(&property) {
        return false;
    }
    matches!(parse_length(value), Some(LengthValue::Px(p)) if p < 0.0)
}

/// 级联算法。
///
/// 接收一组级联声明，按属性名分组后，为每个属性选择优先级最高的声明。
///
/// # 返回值
///
/// 返回一个 HashMap，键为属性名，值为胜出的声明值。
pub fn cascade(declarations: Vec<CascadedDeclaration>) -> HashMap<String, String> {
    // 按属性名分组
    let mut by_property: HashMap<String, Vec<CascadedDeclaration>> = HashMap::new();
    for decl in declarations {
        by_property.entry(decl.property.clone()).or_default().push(decl);
    }

    let mut result = HashMap::new();

    for (property, decls) in by_property {
        // CSS 规范：非法声明（如盒模型尺寸属性的负长度）在级联时即按未声明处理，
        // 故较低优先级的合法声明可胜出。仅选最高优先级的合法声明；若该属性全部声明
        // 均非法（全为负值等），属性不进入级联结果，回退到初始值（width/height→auto、
        // max-*→none 等，由 default_impl 提供，均为 CSS 规范初始值）。
        let winner = decls
            .iter()
            .filter(|d| !is_invalid_negative_length(&property, &d.value))
            .max_by_key(|d| d.order.clone());
        if let Some(w) = winner {
            result.insert(property, w.value.clone());
        }
    }

    result
}

/// 从样式表中收集所有匹配的声明。
///
/// 返回一组带有级联信息的声明。
pub fn collect_declarations(
    declarations: &[(String, String, bool)], // (property, value, important)
    origin: Origin,
    layer_index: Option<usize>,
    specificity: (u32, u32, u32),
    base_position: usize,
) -> Vec<CascadedDeclaration> {
    declarations
        .iter()
        .enumerate()
        .map(|(i, (property, value, important))| CascadedDeclaration {
            property: property.clone(),
            value: value.clone(),
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
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 1, false),
            },
        ];

        let result = cascade(decls);
        assert_eq!(result.get("color"), Some(&"blue".to_string()));
    }

    #[test]
    fn test_cascade_skips_invalid_negative_with_valid_fallback() {
        // CSS §10.5：height 负值非法。height:1in; height:-1px 中 -1px 优先级更高但非法，
        // 应回退到合法的 1in（numbers-units-006 场景）。
        let decls = vec![
            CascadedDeclaration {
                property: "height".to_string(),
                value: "1in".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
            CascadedDeclaration {
                property: "height".to_string(),
                value: "-1px".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 2, false),
            },
        ];

        let result = cascade(decls);
        assert_eq!(result.get("height"), Some(&"1in".to_string()));
    }

    #[test]
    fn test_cascade_sole_negative_rejected_to_initial() {
        // 仅有负值声明时全为非法 → 属性不进入级联结果（回退到初始值 width→auto）。
        let decls = vec![CascadedDeclaration {
            property: "width".to_string(),
            value: "-5px".to_string(),
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];

        let result = cascade(decls);
        assert!(result.get("width").is_none());
    }

    #[test]
    fn test_cascade_negative_check_only_box_model() {
        // 非盒模型尺寸属性（如 margin-top 允许负值）不受影响。
        let decls = vec![CascadedDeclaration {
            property: "margin-top".to_string(),
            value: "-5px".to_string(),
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];

        let result = cascade(decls);
        assert_eq!(result.get("margin-top"), Some(&"-5px".to_string()));
    }

    #[test]
    fn test_cascade_multiple_properties() {
        let decls = vec![
            CascadedDeclaration {
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "display".to_string(),
                value: "flex".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];

        let result = cascade(decls);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
        assert_eq!(result.get("display"), Some(&"flex".to_string()));
    }

    #[test]
    fn test_cascade_important_wins_over_specificity() {
        let decls = vec![
            CascadedDeclaration {
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, true),
            },
            CascadedDeclaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 1, false),
            },
        ];

        let result = cascade(decls);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
    }

    #[test]
    fn test_cascade_empty() {
        let result = cascade(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_declarations() {
        let decls = vec![
            ("color".to_string(), "red".to_string(), false),
            ("display".to_string(), "block".to_string(), true),
        ];

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
                property: "color".to_string(),
                value: "blue".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false),
            },
            CascadedDeclaration {
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 1, false),
            },
        ];
        let result = cascade(decls);
        assert_eq!(result.get("color"), Some(&"red".to_string())); // ID wins
    }

    #[test]
    /// 同优先级下位置靠后的声明胜出
    fn test_cascade_same_specificity_later_wins() {
        let decls = vec![
            CascadedDeclaration {
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color".to_string(),
                value: "green".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls);
        assert_eq!(result.get("color"), Some(&"green".to_string()));
    }

    #[test]
    /// collect_declarations 带 layer_index
    fn test_collect_declarations_with_layer() {
        let decls = vec![("color".to_string(), "red".to_string(), false)];
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
                property: "margin".to_string(),
                value: "10px".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "margin".to_string(),
                value: "20px".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
        ];
        let result = cascade(decls);
        assert_eq!(result.get("margin"), Some(&"20px".to_string()));
    }

    #[test]
    /// !important 覆盖正常声明（即使特异性更低）
    fn test_important_overrides_normal_declaration() {
        let decls = vec![
            CascadedDeclaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 0, false),
            },
            CascadedDeclaration {
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, true),
            },
        ];
        let result = cascade(decls);
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
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "display".to_string(),
                value: "flex".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 1, false),
            },
            CascadedDeclaration {
                property: "opacity".to_string(),
                value: "0.5".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 2, false),
            },
        ];
        let result = cascade(decls);
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
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color".to_string(),
                value: "green".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 1, false),
            },
            CascadedDeclaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (1, 0, 0), 2, false),
            },
        ];
        let result = cascade(decls);
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
        let decls = vec![
            ("a".to_string(), "1".to_string(), false),
            ("b".to_string(), "2".to_string(), true),
            ("c".to_string(), "3".to_string(), false),
        ];
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
        let result = cascade(vec![]);
        assert!(result.is_empty(), "空声明应返回空 map");
    }

    #[test]
    /// cascade 函数单一声明返回正确值
    fn test_cascade_single_declaration() {
        let decls = vec![CascadedDeclaration {
            property: "color".to_string(),
            value: "red".to_string(),
            order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
        }];
        let result = cascade(decls);
        assert_eq!(result.get("color"), Some(&"red".to_string()));
    }

    #[test]
    /// cascade 多属性各自保留最高优先级值（补充验证）
    fn test_cascade_multiple_properties_comprehensive() {
        let decls = vec![
            CascadedDeclaration {
                property: "color".to_string(),
                value: "red".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
            CascadedDeclaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 1, 0), 0, false),
            },
            CascadedDeclaration {
                property: "display".to_string(),
                value: "block".to_string(),
                order: CascadeOrder::new(Origin::Author, None, (0, 0, 1), 0, false),
            },
        ];
        let result = cascade(decls);
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
