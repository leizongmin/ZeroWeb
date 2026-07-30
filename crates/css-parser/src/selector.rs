//! CSS 选择器匹配。
//!
//! 提供选择器与 DOM 元素的匹配功能。此模块定义选择器的运行时匹配逻辑，
//! 选择器解析由 `parser` 模块处理。

use crate::ast::*;

// 选择器匹配功能将在 style-system crate 中实现，
// 因为它需要访问 DOM 节点。此模块提供辅助工具。

/// 计算选择器的 specificity（权重）。
///
/// Specificity 使用 (A, B, C) 三元组：
/// - A = ID 选择器数量
/// - B = 类选择器、属性选择器、伪类选择器数量
/// - C = 类型选择器、伪元素选择器数量
pub fn specificity(selector: &Selector) -> (u32, u32, u32) {
    let mut a = 0u32; // ID
    let mut b = 0u32; // class, attribute, pseudo-class
    let mut c = 0u32; // type, pseudo-element

    for (compound, _) in &selector.complex.parts {
        if let Some(ts) = &compound.type_selector {
            match ts {
                TypeSelector::Tag(_) => c += 1,
                TypeSelector::Universal => {}
            }
        }

        for sub in &compound.subclass_selectors {
            match sub {
                SubclassSelector::Id(_) => a += 1,
                SubclassSelector::Class(_) => b += 1,
                SubclassSelector::Attribute(_) => b += 1,
                // `&` 嵌套选择器：编译后已替换为父级化合物（特异性自然正确）；
                // 未编译残余（不应出现）贡献 0。
                SubclassSelector::Nesting => {}
                SubclassSelector::PseudoClass(pc) => {
                    match pc {
                        PseudoClassSelector::Is(sels)
                        | PseudoClassSelector::Not(sels)
                        | PseudoClassSelector::Has(sels) => {
                            // :is()/:not()/:has() 取参数列表中最大的 specificity
                            let max_spec = max_specificity(sels);
                            a += max_spec.0;
                            b += max_spec.1;
                            c += max_spec.2;
                        }
                        PseudoClassSelector::Where(_) => {
                            // :where() specificity 为 0，不增加
                        }
                        // :nth-child(an+b of S) / :nth-last-child(an+b of S)（Selectors L4 §16）：
                        // specificity = 伪类基 (0,1,0) + of 列表最大 specificity
                        //（如 `:nth-child(even of .foo,#bar,target)` = (1,1,0) = (0,1,0)+(1,0,0)）。
                        PseudoClassSelector::NthChildOf(_, sels) | PseudoClassSelector::NthLastChildOf(_, sels) => {
                            b += 1;
                            let max_spec = max_specificity(sels);
                            a += max_spec.0;
                            b += max_spec.1;
                            c += max_spec.2;
                        }
                        _ => b += 1,
                    }
                }
                SubclassSelector::PseudoElement(_) => c += 1,
            }
        }
    }

    (a, b, c)
}

/// 选择器列表的最大 specificity（用于 :is()/:not()/:has() 与 :nth-child(an+b of S)）。
fn max_specificity(sels: &[Selector]) -> (u32, u32, u32) {
    sels.iter()
        .map(specificity)
        .max_by(|(a1, b1, c1), (a2, b2, c2)| a1.cmp(a2).then(b1.cmp(b2)).then(c1.cmp(c2)))
        .unwrap_or((0, 0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tag_selector(tag: &str) -> Selector {
        Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag(tag.to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                )],
            },
        }
    }

    fn make_id_selector(id: &str) -> Selector {
        Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Id(id.to_string())],
                    },
                    None,
                )],
            },
        }
    }

    fn make_class_selector(cls: &str) -> Selector {
        Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Class(cls.to_string())],
                    },
                    None,
                )],
            },
        }
    }

    #[test]
    fn test_specificity_tag() {
        let sel = make_tag_selector("div");
        assert_eq!(specificity(&sel), (0, 0, 1));
    }

    #[test]
    fn test_specificity_id() {
        let sel = make_id_selector("main");
        assert_eq!(specificity(&sel), (1, 0, 0));
    }

    #[test]
    fn test_specificity_class() {
        let sel = make_class_selector("container");
        assert_eq!(specificity(&sel), (0, 1, 0));
    }

    #[test]
    fn test_specificity_universal() {
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Universal),
                        subclass_selectors: vec![],
                    },
                    None,
                )],
            },
        };
        assert_eq!(specificity(&sel), (0, 0, 0));
    }

    #[test]
    fn test_specificity_complex() {
        // div#main.container
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![
                            SubclassSelector::Id("main".to_string()),
                            SubclassSelector::Class("container".to_string()),
                        ],
                    },
                    None,
                )],
            },
        };
        assert_eq!(specificity(&sel), (1, 1, 1));
    }

    #[test]
    fn test_specificity_where_is_zero() {
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Where(vec![
                            make_tag_selector("div"),
                        ]))],
                    },
                    None,
                )],
            },
        };
        assert_eq!(specificity(&sel), (0, 0, 0));
    }

    #[test]
    fn test_specificity_nth_child_of_selector_list() {
        // :nth-child(even of .foo, #bar, target) = (1,1,0) = 伪类基 (0,1,0) + max(S)
        // 其中 max(S) = max((0,1,0), (1,0,0), (0,0,1)) = (1,0,0)。Selectors L4 §16。
        let of_list = vec![
            make_class_selector("foo"),
            make_id_selector("bar"),
            make_tag_selector("target"),
        ];
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthChildOf(
                            NthPattern { a: 2, b: 0 },
                            of_list,
                        ))],
                    },
                    None,
                )],
            },
        };
        assert_eq!(specificity(&sel), (1, 1, 0));
    }

    #[test]
    fn test_specificity_nth_child_of_no_list() {
        // of 列表为空（不应发生，parser 总带 of 列表）→ 仅伪类基 (0,1,0)。
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthChildOf(
                            NthPattern { a: 2, b: 0 },
                            vec![],
                        ))],
                    },
                    None,
                )],
            },
        };
        assert_eq!(specificity(&sel), (0, 1, 0));
    }
}
