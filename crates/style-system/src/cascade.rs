//! CSS 级联算法。
//!
//! 实现级联优先级排序：按来源、!important、@layer、specificity、出现顺序
//! 决定每个属性的最终胜出声明。

use std::collections::HashMap;

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
        by_property
            .entry(decl.property.clone())
            .or_default()
            .push(decl);
    }

    let mut result = HashMap::new();

    for (property, decls) in by_property {
        // 选择优先级最高的声明
        if let Some(winner) = decls.into_iter().max_by_key(|d| d.order.clone()) {
            result.insert(property, winner.value);
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
            order: CascadeOrder::new(
                origin,
                layer_index,
                specificity,
                base_position + i,
                *important,
            ),
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
}
