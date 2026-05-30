//! # zero-style-system
//!
//! 自建样式系统 — 级联、继承、初始值、计算值。
//!
//! 与 DOM 集成，为 DOM 节点计算样式。
//!
//! ## 核心模块
//!
//! - [`property`] — 属性定义和 `ComputedStyle` 结构体
//! - [`cascade`] — CSS 级联算法
//! - [`inheritance`] — 属性继承
//! - [`computed`] — 计算值生成（相对单位转换）
//! - [`matcher`] — 选择器匹配
//! - [`shorthand`] — CSS 简写属性展开

#![warn(missing_docs)]

pub mod property;
pub mod cascade;
pub mod inheritance;
pub mod computed;
pub mod matcher;
pub mod shorthand;

pub use property::*;
pub use cascade::*;
pub use inheritance::*;
pub use computed::*;
pub use matcher::*;
pub use shorthand::*;

use std::collections::HashMap;
use zero_css_parser::Stylesheet;
use zero_dom::{Document, NodeId, NodeKind};

/// 样式系统，负责为文档中的元素计算样式。
///
/// 整合选择器匹配、级联、继承和计算值生成。
pub struct StyleSystem {
    /// 自定义属性存储（--variable）。
    custom_properties: HashMap<String, String>,
    /// 视口宽度（px），用于 vh/vw 计算。
    viewport_width: Option<f64>,
    /// 视口高度（px），用于 vh/vw 计算。
    viewport_height: Option<f64>,
}

impl StyleSystem {
    /// 创建新的样式系统实例。
    pub fn new() -> Self {
        Self {
            custom_properties: HashMap::new(),
            viewport_width: None,
            viewport_height: None,
        }
    }

    /// 设置视口尺寸。
    pub fn set_viewport(&mut self, width: f64, height: f64) {
        self.viewport_width = Some(width);
        self.viewport_height = Some(height);
    }

    /// 为整个文档计算样式。
    ///
    /// 遍历文档中的所有元素节点，为每个元素计算完整的计算样式。
    ///
    /// # 参数
    ///
    /// - `doc` — DOM 文档
    /// - `stylesheets` — CSS 样式表列表
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为元素 NodeId，值为对应的 ComputedStyle。
    pub fn compute_styles(
        &mut self,
        doc: &Document,
        stylesheets: &[Stylesheet],
    ) -> HashMap<NodeId, ComputedStyle> {
        let mut styles = HashMap::new();

        // 从文档根开始 DFS
        let root = doc.root();
        self.compute_styles_recursive(doc, root, stylesheets, None, &mut styles);

        styles
    }

    /// 递归计算样式。
    fn compute_styles_recursive(
        &mut self,
        doc: &Document,
        node: NodeId,
        stylesheets: &[Stylesheet],
        parent_style: Option<&ComputedStyle>,
        styles: &mut HashMap<NodeId, ComputedStyle>,
    ) {
        let node_data = match doc.get(node) {
            Some(n) => n,
            None => return,
        };

        // 判断是否为元素节点
        let is_element = matches!(&node_data.kind, NodeKind::Element(_));

        // 只为元素节点计算样式
        if is_element {
            let computed = self.compute_element_style_internal(doc, node, stylesheets, parent_style);
            styles.insert(node, computed);
        }

        // 收集子节点列表
        let children = doc.child_nodes(node);
        if children.is_empty() {
            return;
        }

        // 对于子节点：如果当前节点是元素节点且有计算样式，
        // 需要从 styles 中取出作为 parent_style。
        // 为了避免借用冲突，先克隆当前节点的样式。
        let current_style = if is_element {
            styles.get(&node).cloned()
        } else {
            None
        };

        let parent_ref = current_style.as_ref().or(parent_style);

        for child in children {
            self.compute_styles_recursive(doc, child, stylesheets, parent_ref, styles);
        }
    }

    /// 为单个元素计算样式。
    ///
    /// 完整流程：选择器匹配 → 级联 → 继承 → 计算值。
    pub fn compute_element_style(
        &mut self,
        doc: &Document,
        element: NodeId,
        stylesheets: &[Stylesheet],
        parent_style: Option<&ComputedStyle>,
    ) -> ComputedStyle {
        self.compute_element_style_internal(doc, element, stylesheets, parent_style)
    }

    /// 内部实现：计算单个元素的样式。
    fn compute_element_style_internal(
        &mut self,
        doc: &Document,
        element: NodeId,
        stylesheets: &[Stylesheet],
        parent_style: Option<&ComputedStyle>,
    ) -> ComputedStyle {
        // 0. 构建媒体查询上下文
        let media_ctx = match (self.viewport_width, self.viewport_height) {
            (Some(w), Some(h)) => Some(zero_css_parser::media_query::MediaContext::new(w, h)),
            _ => None,
        };

        // 1. 收集匹配的声明（带媒体查询评估）
        let matching = matcher::collect_matching_declarations_with_media(
            doc,
            element,
            stylesheets,
            media_ctx.as_ref(),
        );

        // 1.5. 展开简写属性
        let expanded = shorthand::expand_shorthands(&matching);

        // 2. 构建 CascadedDeclaration 列表
        let mut declarations = Vec::new();
        for (position, (property, value, important, specificity)) in expanded.iter().enumerate() {
            declarations.push(CascadedDeclaration {
                property: property.clone(),
                value: value.clone(),
                order: CascadeOrder::new(
                    Origin::Author,
                    None,
                    *specificity,
                    position,
                    *important,
                ),
            });
        }

        // 3. 运行级联算法
        let cascaded = cascade::cascade(declarations);

        // 4. 收集自定义属性
        self.custom_properties = gather_custom_properties(&cascaded);

        // 5. 计算继承样式
        let style = inheritance::compute_inherited_style(parent_style, &cascaded);

        // 6. 解析计算值（相对单位转换）
        computed::resolve_computed_style(
            &style,
            &self.custom_properties,
            self.viewport_width,
            self.viewport_height,
        )
    }
}

impl Default for StyleSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 从级联值中收集自定义属性。
fn gather_custom_properties(cascaded: &HashMap<String, String>) -> HashMap<String, String> {
    cascaded
        .iter()
        .filter(|(k, _)| k.starts_with("--"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::ast::{
        ComplexSelector, CompoundSelector, Declaration, Rule, Selector, StyleRule,
        SubclassSelector, TypeSelector,
    };
    use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue, OverflowValue};
    use zero_dom::{Document, NodeId};

    /// 创建测试 DOM：html > body > div#main > p.text
    fn make_test_dom() -> (Document, NodeId, NodeId, NodeId, NodeId) {
        let mut doc = Document::new();
        let root = doc.root();

        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();

        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();

        let div = doc.create_element("div");
        doc.set_attribute(div, "id", "main");
        doc.append_child(body, div).unwrap();

        let p = doc.create_element("p");
        doc.set_attribute(p, "class", "text");
        doc.append_child(div, p).unwrap();

        (doc, html, body, div, p)
    }

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

    #[test]
    fn test_style_system_new() {
        let sys = StyleSystem::new();
        assert!(sys.custom_properties.is_empty());
    }

    #[test]
    fn test_compute_styles_empty() {
        let (doc, _html, _body, _div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        let stylesheets = vec![];
        let styles = sys.compute_styles(&doc, &stylesheets);
        // 应该有 html, body, div, p 四个元素的样式
        assert!(styles.len() >= 4);
    }

    #[test]
    fn test_compute_styles_with_rules() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_compute_styles_inheritance() {
        let (doc, _html, _body, _div, p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let p_style = styles.get(&p).expect("p 应该有样式");
        // p 应该继承 div 的 color
        assert_eq!(p_style.color, ColorValue::Rgba(0, 0, 255, 255));
    }

    #[test]
    fn test_compute_element_style() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "display".to_string(),
                    value: "flex".to_string(),
                    important: false,
                }],
            })],
        }];

        let style = sys.compute_element_style(&doc, div, &stylesheets, None);
        assert_eq!(style.display, DisplayValue::Flex);
    }

    #[test]
    fn test_compute_styles_with_class_selector() {
        let (doc, _html, _body, _div, p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let class_sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Class("text".to_string())],
                    },
                    None,
                )],
            },
        };

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![class_sel],
                declarations: vec![Declaration {
                    property: "font-size".to_string(),
                    value: "20px".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let p_style = styles.get(&p).expect("p 应该有样式");
        assert_eq!(p_style.font_size, LengthValue::Px(20.0));
    }

    #[test]
    fn test_compute_styles_specificity() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let tag_sel = make_tag_selector("div");
        let id_sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Id("main".to_string())],
                    },
                    None,
                )],
            },
        };

        // tag 选择器设置 color: red
        // id 选择器设置 color: blue
        // id 选择器特异性更高，应该胜出
        let stylesheets = vec![Stylesheet {
            rules: vec![
                Rule::Style(StyleRule {
                    selectors: vec![tag_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![id_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
    }

    #[test]
    fn test_set_viewport() {
        let mut sys = StyleSystem::new();
        sys.set_viewport(1920.0, 1080.0);
        assert_eq!(sys.viewport_width, Some(1920.0));
        assert_eq!(sys.viewport_height, Some(1080.0));
    }

    #[test]
    fn test_default_style_system() {
        let sys = StyleSystem::default();
        assert!(sys.custom_properties.is_empty());
    }

    // ── 简写属性端到端测试 ──

    #[test]
    fn test_shorthand_margin_in_style_computation() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "margin".to_string(),
                    value: "10px 20px".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.margin_top, LengthValue::Px(10.0));
        assert_eq!(div_style.margin_right, LengthValue::Px(20.0));
        assert_eq!(div_style.margin_bottom, LengthValue::Px(10.0));
        assert_eq!(div_style.margin_left, LengthValue::Px(20.0));
    }

    #[test]
    fn test_shorthand_padding_in_style_computation() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "padding".to_string(),
                    value: "5px".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.padding_top, LengthValue::Px(5.0));
        assert_eq!(div_style.padding_right, LengthValue::Px(5.0));
        assert_eq!(div_style.padding_bottom, LengthValue::Px(5.0));
        assert_eq!(div_style.padding_left, LengthValue::Px(5.0));
    }

    #[test]
    fn test_shorthand_border_in_style_computation() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "border".to_string(),
                    value: "1px solid red".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.border_top_width, LengthValue::Px(1.0));
        assert_eq!(div_style.border_right_width, LengthValue::Px(1.0));
        assert_eq!(div_style.border_bottom_width, LengthValue::Px(1.0));
        assert_eq!(div_style.border_left_width, LengthValue::Px(1.0));
        assert_eq!(div_style.border_top_style, property::BorderStyleValue::Solid);
        assert_eq!(div_style.border_top_color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_shorthand_overflow_in_style_computation() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "overflow".to_string(),
                    value: "hidden".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.overflow_x, OverflowValue::Hidden);
        assert_eq!(div_style.overflow_y, OverflowValue::Hidden);
    }

    #[test]
    fn test_shorthand_border_radius_in_style_computation() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "border-radius".to_string(),
                    value: "5px 10px".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.border_top_left_radius, LengthValue::Px(5.0));
        assert_eq!(div_style.border_top_right_radius, LengthValue::Px(10.0));
        assert_eq!(div_style.border_bottom_right_radius, LengthValue::Px(5.0));
        assert_eq!(div_style.border_bottom_left_radius, LengthValue::Px(10.0));
    }

    #[test]
    fn test_shorthand_margin_with_longhand_override() {
        // margin 简写设置后，后面的 longhand 应该覆盖
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "margin".to_string(),
                        value: "10px".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "margin-top".to_string(),
                        value: "20px".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // margin-top 被 longhand 覆盖为 20px
        assert_eq!(div_style.margin_top, LengthValue::Px(20.0));
        // 其他边保持 10px
        assert_eq!(div_style.margin_right, LengthValue::Px(10.0));
    }

    // ── 媒体查询端到端测试 ──

    #[test]
    fn test_media_query_applies_when_condition_matches() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(1024.0, 768.0); // 宽屏

        // @media (min-width: 600px) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::At(zero_css_parser::ast::AtRule {
                name: "media".to_string(),
                prelude: "(min-width: 600px)".to_string(),
                body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })]),
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_media_query_skips_when_condition_fails() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(400.0, 300.0); // 窄屏

        // @media (min-width: 600px) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::At(zero_css_parser::ast::AtRule {
                name: "media".to_string(),
                prelude: "(min-width: 600px)".to_string(),
                body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })]),
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 条件不满足，color 保持默认黑色
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    fn test_media_query_with_regular_rules() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);

        // 正常规则 + @media 规则
        let stylesheets = vec![Stylesheet {
            rules: vec![
                // 基础样式
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
                // 响应式样式
                Rule::At(zero_css_parser::ast::AtRule {
                    name: "media".to_string(),
                    prelude: "(min-width: 600px)".to_string(),
                    body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "margin-top".to_string(),
                            value: "20px".to_string(),
                            important: false,
                        }],
                    })]),
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 基础样式应用
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
        // @media 条件满足，响应式样式也应用
        assert_eq!(div_style.margin_top, LengthValue::Px(20.0));
    }

    #[test]
    fn test_media_query_no_viewport_skips() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        // 不设置视口

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::At(zero_css_parser::ast::AtRule {
                name: "media".to_string(),
                prelude: "(min-width: 600px)".to_string(),
                body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })]),
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 没有视口信息，@media 不应用
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }
}
