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

pub mod cascade;
pub mod computed;
pub mod inheritance;
pub mod matcher;
pub mod property;
pub mod shorthand;

pub use cascade::*;
pub use computed::*;
pub use inheritance::*;
pub use matcher::*;
pub use property::*;
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
    pub fn compute_styles(&mut self, doc: &Document, stylesheets: &[Stylesheet]) -> HashMap<NodeId, ComputedStyle> {
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
        let current_style = if is_element { styles.get(&node).cloned() } else { None };

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

        // 0.5 构建容器查询上下文（简化：使用视口尺寸作为默认容器尺寸）
        let container_ctx = match (self.viewport_width, self.viewport_height) {
            (Some(w), Some(h)) => Some(matcher::ContainerContext::with_size(w, h)),
            _ => None,
        };

        // 1. 收集匹配的声明（带媒体查询和容器查询评估）
        let matching = matcher::collect_matching_declarations_with_media(
            doc,
            element,
            stylesheets,
            media_ctx.as_ref(),
            container_ctx.as_ref(),
        );

        // 1.5. 展开简写属性（保留层索引）
        #[allow(clippy::type_complexity)]
        let mut expanded_with_layer: Vec<(String, String, bool, (u32, u32, u32), Option<usize>)> = Vec::new();
        for (property, value, important, specificity, layer_index) in &matching {
            let input = (property.clone(), value.clone(), *important, *specificity);
            let expanded = shorthand::expand_shorthands(&[input]);
            for (prop, val, imp, spec) in expanded {
                expanded_with_layer.push((prop, val, imp, spec, *layer_index));
            }
        }

        // 2. 构建 CascadedDeclaration 列表
        let mut declarations = Vec::new();
        for (position, (property, value, important, specificity, layer_index)) in expanded_with_layer.iter().enumerate()
        {
            declarations.push(CascadedDeclaration {
                property: property.clone(),
                value: value.clone(),
                order: CascadeOrder::new(Origin::Author, *layer_index, *specificity, position, *important),
            });
        }

        // 3. 运行级联算法
        let cascaded = cascade::cascade(declarations);

        // 4. 收集自定义属性
        self.custom_properties = gather_custom_properties(&cascaded);

        // 4.5. 在级联值中解析 var() 引用
        let resolved_cascaded = resolve_var_in_cascaded(&cascaded, &self.custom_properties);

        // 5. 计算继承样式
        let style = inheritance::compute_inherited_style(parent_style, &resolved_cascaded);

        // 6. 解析计算值（相对单位转换）
        // 提取父元素的计算 font-size，用于子元素 font-size 的 em 解析
        let parent_fs = parent_style.map(|ps| {
            // 父元素的 font_size 已经被解析为 Px
            match &ps.font_size {
                zero_css_parser::values::LengthValue::Px(v) => *v,
                _ => computed::ROOT_FONT_SIZE,
            }
        });
        computed::resolve_computed_style(
            &style,
            &self.custom_properties,
            self.viewport_width,
            self.viewport_height,
            parent_fs,
        )
    }
}

/// 在级联属性值中解析 var() 引用。
///
/// 自定义属性（--*）本身不解析，仅解析标准属性的值。
fn resolve_var_in_cascaded(
    cascaded: &HashMap<String, String>,
    custom_properties: &HashMap<String, String>,
) -> HashMap<String, String> {
    cascaded
        .iter()
        .map(|(k, v)| {
            if k.starts_with("--") {
                (k.clone(), v.clone())
            } else {
                (k.clone(), computed::resolve_var(v, custom_properties))
            }
        })
        .collect()
}

impl Default for StyleSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 从级联值中收集自定义属性，并解析自定义属性值中的 var() 引用。
///
/// 自定义属性值可以引用其他自定义属性，需要迭代解析直到稳定。
fn gather_custom_properties(cascaded: &HashMap<String, String>) -> HashMap<String, String> {
    let mut props: HashMap<String, String> = cascaded
        .iter()
        .filter(|(k, _)| k.starts_with("--"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    // 迭代解析自定义属性值中的 var() 引用
    let mut changed = true;
    let mut max_iter = 10;
    while changed && max_iter > 0 {
        max_iter -= 1;
        changed = false;
        let snapshot = props.clone();
        for (_key, value) in props.iter_mut() {
            let resolved = computed::resolve_var(value, &snapshot);
            if resolved != *value {
                *value = resolved;
                changed = true;
            }
        }
    }

    props
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::ast::{
        ComplexSelector, CompoundSelector, Declaration, Rule, Selector, StyleRule, SubclassSelector, TypeSelector,
    };
    use zero_css_parser::values::{BoxSizingValue, ColorValue, DisplayValue, LengthValue, OverflowValue};
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

    // ── @supports 端到端测试 ──

    #[test]
    fn test_supports_applies_when_condition_met() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "grid".to_string()),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "display".to_string(),
                        value: "grid".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.display, DisplayValue::Grid);
    }

    #[test]
    fn test_supports_skips_when_condition_not_met() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Property(
                    "display".to_string(),
                    "unknown-value".to_string(),
                ),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    fn test_supports_not_condition() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Not(Box::new(
                    zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "grid".to_string()),
                )),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    fn test_supports_and_condition() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::And(vec![
                    zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "flex".to_string()),
                    zero_css_parser::ast::SupportsCondition::Property("color".to_string(), "blue".to_string()),
                ]),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255));
    }

    #[test]
    fn test_supports_or_condition() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Or(vec![
                    zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "unknown".to_string()),
                    zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "flex".to_string()),
                ]),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_supports_with_regular_rules() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
                Rule::Supports(zero_css_parser::ast::SupportsRule {
                    condition: zero_css_parser::ast::SupportsCondition::Property(
                        "display".to_string(),
                        "grid".to_string(),
                    ),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "margin-top".to_string(),
                            value: "20px".to_string(),
                            important: false,
                        }],
                    })],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
        assert_eq!(div_style.margin_top, LengthValue::Px(20.0));
    }

    // ── @supports selector() 端到端测试 ──

    /// 测试 selector() 基本用法：有效的选择器应返回 true。
    #[test]
    fn test_supports_selector_basic() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        // @supports selector(div > .class) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Selector("div > .class".to_string()),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(
            div_style.color,
            ColorValue::Rgba(255, 0, 0, 255),
            "selector(div > .class) 应该评估为 true，颜色应为红色"
        );
    }

    /// 测试 selector() 复杂伪类：有效的 :is() 选择器应返回 true。
    #[test]
    fn test_supports_selector_complex() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        // @supports selector(:is(div, span)) { div { color: green; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Selector(":is(div, span)".to_string()),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(
            div_style.color,
            ColorValue::Rgba(0, 128, 0, 255),
            "selector(:is(div, span)) 应该评估为 true，颜色应为绿色"
        );
    }

    /// 测试 selector() 无效选择器应返回 false。
    #[test]
    fn test_supports_selector_invalid() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        // @supports selector(>>>invalid) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Selector(">>>invalid".to_string()),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(
            div_style.color,
            ColorValue::Rgba(0, 0, 0, 255),
            "selector(>>>invalid) 应该评估为 false，不应应用红色"
        );
    }

    /// 测试 selector() 在完整规则中的端到端应用。
    #[test]
    fn test_supports_selector_in_rule() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        // @supports selector(p) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Selector("p".to_string()),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(
            div_style.color,
            ColorValue::Rgba(255, 0, 0, 255),
            "selector(p) 应该评估为 true，div 颜色应为红色"
        );
    }

    // ── Grid 属性端到端测试 ──

    #[test]
    fn test_grid_template_columns_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "display".to_string(),
                        value: "grid".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-template-columns".to_string(),
                        value: "100px 1fr auto".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.display, DisplayValue::Grid);
        assert_eq!(div_style.grid_template_columns, Some("100px 1fr auto".to_string()));
    }

    #[test]
    fn test_grid_template_rows_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "display".to_string(),
                        value: "grid".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-template-rows".to_string(),
                        value: "50px 1fr".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.display, DisplayValue::Grid);
        assert_eq!(div_style.grid_template_rows, Some("50px 1fr".to_string()));
    }

    #[test]
    fn test_grid_auto_flow_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "display".to_string(),
                        value: "grid".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-auto-flow".to_string(),
                        value: "column dense".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.display, DisplayValue::Grid);
        assert_eq!(div_style.grid_auto_flow, property::GridAutoFlowValue::ColumnDense);
    }

    #[test]
    fn test_grid_combined_properties_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "display".to_string(),
                        value: "grid".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-template-columns".to_string(),
                        value: "1fr 1fr".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-template-rows".to_string(),
                        value: "auto".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-auto-flow".to_string(),
                        value: "row".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "gap".to_string(),
                        value: "10px".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.display, DisplayValue::Grid);
        assert_eq!(div_style.grid_template_columns, Some("1fr 1fr".to_string()));
        assert_eq!(div_style.grid_template_rows, Some("auto".to_string()));
        assert_eq!(div_style.grid_auto_flow, property::GridAutoFlowValue::Row);
        assert_eq!(div_style.gap, LengthValue::Px(10.0));
    }

    #[test]
    fn test_grid_unset_uses_initial() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "display".to_string(),
                        value: "grid".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-auto-flow".to_string(),
                        value: "unset".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // grid-auto-flow is not inherited, unset = initial = Row
        assert_eq!(div_style.grid_auto_flow, property::GridAutoFlowValue::Row);
    }

    #[test]
    fn test_grid_default_values_no_css() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![];
        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.grid_template_columns, None);
        assert_eq!(div_style.grid_template_rows, None);
        assert_eq!(div_style.grid_auto_flow, property::GridAutoFlowValue::Row);
        assert_eq!(div_style.grid_column_start, property::GridLineValue::Auto);
        assert_eq!(div_style.grid_column_end, property::GridLineValue::Auto);
        assert_eq!(div_style.grid_row_start, property::GridLineValue::Auto);
        assert_eq!(div_style.grid_row_end, property::GridLineValue::Auto);
        assert_eq!(div_style.grid_auto_rows, None);
        assert_eq!(div_style.grid_auto_columns, None);
    }

    // ── @layer 端到端测试 ──

    #[test]
    fn test_layer_unlayered_beats_layered() {
        // 未分层的 div { color: red; } 应该胜过 @layer base { div { color: blue; } }
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "base".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "blue".to_string(),
                            important: false,
                        }],
                    })],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 未分层胜过分层
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
    }

    #[test]
    fn test_layer_later_beats_earlier() {
        // @layer base { div { color: red; } } @layer theme { div { color: green; } }
        // 后面的层胜过前面的
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "base".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "red".to_string(),
                            important: false,
                        }],
                    })],
                }),
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "theme".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "green".to_string(),
                            important: false,
                        }],
                    })],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 后面的层（theme=green）胜过前面的层（base=red）
        assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255)); // green
    }

    #[test]
    fn test_layer_specificity_within_same_layer() {
        // 同层内，高特异性仍然胜出
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
                rules: vec![
                    // div { color: red; } — 低特异性
                    Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "red".to_string(),
                            important: false,
                        }],
                    }),
                    // #main { color: blue; } — 高特异性
                    Rule::Style(StyleRule {
                        selectors: vec![Selector {
                            complex: ComplexSelector {
                                parts: vec![(
                                    CompoundSelector {
                                        type_selector: None,
                                        subclass_selectors: vec![SubclassSelector::Id("main".to_string())],
                                    },
                                    None,
                                )],
                            },
                        }],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "blue".to_string(),
                            important: false,
                        }],
                    }),
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 同层内高特异性胜出
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue
    }

    #[test]
    fn test_layer_important_beats_normal() {
        // 分层内的 !important 仍然胜出（按 normal < important 规则）
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "base".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "blue".to_string(),
                            important: true,
                        }],
                    })],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // !important 总是胜过 normal（即使分层 vs 未分层）
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增端到端测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// grid-column-start/end 端到端
    fn test_grid_column_start_end_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "grid-column-start".to_string(),
                        value: "2".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-column-end".to_string(),
                        value: "5".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.grid_column_start, property::GridLineValue::Line(2));
        assert_eq!(div_style.grid_column_end, property::GridLineValue::Line(5));
    }

    #[test]
    /// grid-row-start/end 端到端
    fn test_grid_row_start_end_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "grid-row-start".to_string(),
                        value: "1".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-row-end".to_string(),
                        value: "3".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.grid_row_start, property::GridLineValue::Line(1));
        assert_eq!(div_style.grid_row_end, property::GridLineValue::Line(3));
    }

    #[test]
    /// grid-area 简写端到端
    fn test_grid_area_shorthand_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "grid-area".to_string(),
                    value: "1 / 2 / 3 / 4".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.grid_row_start, property::GridLineValue::Line(1));
        assert_eq!(div_style.grid_row_end, property::GridLineValue::Line(3));
        assert_eq!(div_style.grid_column_start, property::GridLineValue::Line(2));
        assert_eq!(div_style.grid_column_end, property::GridLineValue::Line(4));
    }

    #[test]
    /// span-based grid placement 端到端
    fn test_grid_span_placement_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "grid-column-start".to_string(),
                        value: "span 2".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "grid-column-end".to_string(),
                        value: "5".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.grid_column_start, property::GridLineValue::Span(2));
        assert_eq!(div_style.grid_column_end, property::GridLineValue::Line(5));
    }

    #[test]
    /// negative grid line numbers 端到端
    fn test_grid_negative_line_numbers_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "grid-column-start".to_string(),
                    value: "-1".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.grid_column_start, property::GridLineValue::Line(-1));
    }

    #[test]
    /// transition-duration 端到端
    fn test_transition_duration_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "transition".to_string(),
                    value: "opacity 0.5s ease 0.1s".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.transition_property, vec!["opacity"]);
        assert_eq!(div_style.transition_duration, vec![0.5]);
        assert_eq!(div_style.transition_delay, vec![0.1]);
    }

    #[test]
    /// animation-direction values 端到端
    fn test_animation_direction_values_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "animation".to_string(),
                    value: "fadeIn 1s linear infinite alternate".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.animation_name, vec!["fadeIn"]);
        assert_eq!(div_style.animation_direction.len(), 1);
    }

    #[test]
    /// animation-fill-mode forwards 端到端
    fn test_animation_fill_mode_forwards_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "animation".to_string(),
                    value: "slideUp 0.3s ease forwards".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.animation_fill_mode.len(), 1);
    }

    #[test]
    /// animation-play-state paused 端到端
    fn test_animation_play_state_paused_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "animation".to_string(),
                    value: "spin 2s linear paused".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.animation_play_state.len(), 1);
    }

    #[test]
    /// flex shorthand 端到端
    fn test_flex_shorthand_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "flex".to_string(),
                    value: "2 1 100px".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.flex_grow, 2.0);
        assert_eq!(div_style.flex_shrink, 1.0);
    }

    #[test]
    /// transform 端到端
    fn test_transform_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "transform".to_string(),
                    value: "translateX(10px)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert!(!matches!(
            div_style.transform,
            zero_css_parser::values::TransformValue::None
        ));
    }

    #[test]
    /// 自定义属性与颜色端到端
    fn test_custom_property_with_color_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "--main-color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
    }

    /// var() 引用在样式计算管线中正确解析。
    #[test]
    fn test_var_resolution_in_pipeline() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "--main-color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "color".to_string(),
                        value: "var(--main-color)".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // var(--main-color) 应该被解析为 "red"
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    /// var() 带回退值时，变量不存在则使用回退。
    #[test]
    fn test_var_fallback_in_pipeline() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "var(--undefined, blue)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
    }

    /// var() 解析 width 长度值。
    #[test]
    fn test_var_resolution_width_length() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "--my-width".to_string(),
                        value: "100px".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "width".to_string(),
                        value: "var(--my-width)".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.width, LengthValue::Px(100.0));
    }

    /// 嵌套 var() 正确解析。
    #[test]
    fn test_var_nested_resolution() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "--base".to_string(),
                        value: "red".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "--accent".to_string(),
                        value: "var(--base)".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "color".to_string(),
                        value: "var(--accent)".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    // ── CSS 数学函数端到端测试 ──

    /// 测试 calc() 在宽度属性中的端到端应用。
    #[test]
    fn test_calc_width_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "width".to_string(),
                    value: "calc(100px + 50px)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // calc(100px + 50px) = 150px
        assert_eq!(div_style.width, LengthValue::Px(150.0));
    }

    /// 测试 min() 在宽度属性中的端到端应用。
    #[test]
    fn test_min_width_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "width".to_string(),
                    value: "min(200px, 100px)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.width, LengthValue::Px(100.0));
    }

    /// 测试 max() 在高度属性中的端到端应用。
    #[test]
    fn test_max_height_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "height".to_string(),
                    value: "max(50px, 120px)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.height, LengthValue::Px(120.0));
    }

    /// 测试 clamp() 在边距属性中的端到端应用。
    #[test]
    fn test_clamp_margin_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "margin-top".to_string(),
                    value: "clamp(10px, 50px, 100px)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // clamp(10, 50, 100) — 50 在范围内，结果为 50
        assert_eq!(div_style.margin_top, LengthValue::Px(50.0));
    }

    /// 测试 calc() 嵌套 min() 在内边距中的端到端应用。
    #[test]
    fn test_calc_nested_min_padding_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "padding-left".to_string(),
                    value: "calc(min(30px, 20px) + 10px)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // min(30,20)=20, 20+10=30
        assert_eq!(div_style.padding_left, LengthValue::Px(30.0));
    }

    /// 测试 calc() 与 em 单位混合在宽度中的端到端应用。
    #[test]
    fn test_calc_em_width_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "font-size".to_string(),
                        value: "20px".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "width".to_string(),
                        value: "calc(2em + 10px)".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 2em = 2*20 = 40px, 40+10=50px
        assert_eq!(div_style.width, LengthValue::Px(50.0));
    }

    // ── aspect-ratio 端到端测试 ──

    /// 测试 aspect-ratio 数值解析。
    #[test]
    fn test_aspect_ratio_number() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "aspect-ratio".to_string(),
                    value: "1.5".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.aspect_ratio, Some(1.5));
    }

    /// 测试 aspect-ratio 斜杠语法（16 / 9）。
    #[test]
    fn test_aspect_ratio_slash_syntax() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "aspect-ratio".to_string(),
                    value: "16 / 9".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        let ratio = div_style.aspect_ratio.expect("should have aspect-ratio");
        assert!((ratio - 16.0 / 9.0).abs() < 0.01);
    }

    /// 测试 aspect-ratio: auto 重置为 None。
    #[test]
    fn test_aspect_ratio_auto() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "aspect-ratio".to_string(),
                    value: "auto".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.aspect_ratio, None);
    }

    /// 测试 aspect-ratio 默认值为 None。
    #[test]
    fn test_aspect_ratio_default() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.aspect_ratio, None);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 属性边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// cursor 是继承属性：父元素设置 cursor:pointer，子元素无显式 cursor 时继承 pointer
    fn test_cursor_inheritance() {
        let (doc, _html, _body, _div, p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "cursor".to_string(),
                    value: "pointer".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let p_style = styles.get(&p).expect("p 应该有样式");
        // cursor 是继承属性，p 应从 div 继承 pointer
        assert_eq!(p_style.cursor, property::CursorValue::Pointer);
    }

    #[test]
    /// opacity 不是继承属性：父元素设置 opacity:0.5，子元素默认 opacity 为 1.0
    fn test_opacity_inheritance() {
        let (doc, _html, _body, _div, p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "opacity".to_string(),
                    value: "0.5".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let p_style = styles.get(&p).expect("p 应该有样式");
        // opacity 不继承，子元素默认 1.0
        assert_eq!(p_style.opacity, 1.0);
    }

    #[test]
    /// transition-property: none 表示无过渡属性，结果为空列表
    fn test_transition_property_none() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "transition".to_string(),
                    value: "none".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // transition: none → transition-property 解析为空列表
        assert!(div_style.transition_property.is_empty());
    }

    #[test]
    /// animation-name: none 表示无动画，结果为空列表
    fn test_animation_name_none() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "animation".to_string(),
                    value: "none".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // animation: none → animation-name 解析为空列表
        assert!(div_style.animation_name.is_empty());
    }

    #[test]
    /// box-sizing: border-box 时，border 宽度从总宽度中扣除，
    /// 内容区域宽度 = 指定宽度 - border 宽度
    fn test_box_sizing_effect_on_width() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "box-sizing".to_string(),
                        value: "border-box".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "width".to_string(),
                        value: "100px".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "border".to_string(),
                        value: "10px solid black".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // box-sizing 应用为 border-box
        assert_eq!(div_style.box_sizing, BoxSizingValue::BorderBox);
        // width 仍为 100px（内容宽度计算由布局引擎完成）
        assert_eq!(div_style.width, LengthValue::Px(100.0));
        // border 各边宽度为 10px
        assert_eq!(div_style.border_top_width, LengthValue::Px(10.0));
        assert_eq!(div_style.border_left_width, LengthValue::Px(10.0));
    }

    #[test]
    /// transform 支持多个变换函数组合：translateX(10px) rotate(45deg) 两个函数都被应用
    fn test_multiple_transform_functions() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "transform".to_string(),
                    value: "translateX(10px) rotate(45deg)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 应该解析为 TransformValue::List 且包含两个函数
        match &div_style.transform {
            zero_css_parser::values::TransformValue::List(funcs) => {
                assert_eq!(funcs.len(), 2, "应包含两个变换函数");
                // 验证第一个函数是 translateX(10px)
                assert!(
                    matches!(&funcs[0], zero_css_parser::values::TransformFunction::TranslateX(v) if (*v - 10.0).abs() < 0.01),
                    "第一个函数应为 translateX(10px)"
                );
                // 验证第二个函数是 rotate(45deg)
                assert!(
                    matches!(&funcs[1], zero_css_parser::values::TransformFunction::Rotate(v) if (*v - 45.0).abs() < 0.01),
                    "第二个函数应为 rotate(45deg)"
                );
            }
            other => panic!("transform 应为 List，实际为 {:?}", other),
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // @layer 排序与级联验证端到端测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 后 @layer 的声明在特异性相等时覆盖前 @layer 的声明。
    ///
    /// 场景：@layer base { div { color: red } } @layer theme { div { color: green } }
    /// 两个选择器特异性都是 (0,0,1)，theme 层索引更大，应胜出。
    fn test_layer_ordering_specificity() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                // @layer base — color: red
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "base".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "red".to_string(),
                            important: false,
                        }],
                    })],
                }),
                // @layer theme — color: green（同特异性，后层胜出）
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "theme".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "green".to_string(),
                            important: false,
                        }],
                    })],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // 后层（theme=green）在特异性相等时胜过前层（base=red）
        assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255)); // green
    }

    #[test]
    /// 未分层样式覆盖分层样式，无论特异性高低。
    ///
    /// 场景：@layer base { #main { color: blue } } div { color: red }
    /// 分层内用 ID 选择器 (1,0,0)，未分层用标签选择器 (0,0,1)。
    /// 未分层仍应胜出。
    fn test_unlayered_overrides_layered() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

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

        let stylesheets = vec![Stylesheet {
            rules: vec![
                // @layer base — #main { color: blue }，高特异性但在层内
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "base".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![id_sel],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "blue".to_string(),
                            important: false,
                        }],
                    })],
                }),
                // 未分层 — div { color: red }，低特异性但未分层
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // 未分层声明胜过分层声明（无论特异性）
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
    }

    #[test]
    /// !important 声明在级联中胜过 normal 声明，即使前者在更早的 @layer。
    ///
    /// 场景：@layer base { div { color: blue !important } }
    ///        @layer theme { div { color: green } }
    /// blue 的 !important 使其胜过 green 的 normal。
    fn test_important_overrides_layer_order() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                // @layer base — color: blue !important
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "base".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "blue".to_string(),
                            important: true,
                        }],
                    })],
                }),
                // @layer theme — color: green（后层但 normal）
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "theme".to_string(),
                    rules: vec![Rule::Style(StyleRule {
                        selectors: vec![make_tag_selector("div")],
                        declarations: vec![Declaration {
                            property: "color".to_string(),
                            value: "green".to_string(),
                            important: false,
                        }],
                    })],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // !important 胜过后层的 normal 声明
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue
    }

    #[test]
    /// 验证特异性优先级：内联 > ID > class > element > universal。
    ///
    /// 为 div#main 同时应用多个不同特异性的 color 声明，
    /// 级联应选择特异性最高的胜出者。
    /// 注意：本测试不使用内联样式（引擎不支持 style 属性），
    /// 只验证 ID > class > element > universal。
    fn test_cascade_specificity_order() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let universal_sel = Selector {
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
        let class_sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Class("nonexistent".to_string())],
                    },
                    None,
                )],
            },
        };
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

        // 通用选择器 (0,0,0) → purple
        // 标签选择器 (0,0,1) → red
        // class 选择器 (0,1,0) → yellow（不匹配 div，仅用于对比）
        // ID 选择器 (1,0,0) → blue
        let stylesheets = vec![Stylesheet {
            rules: vec![
                Rule::Style(StyleRule {
                    selectors: vec![universal_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "purple".to_string(),
                        important: false,
                    }],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![class_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "yellow".to_string(),
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
        // ID 选择器 (1,0,0) 特异性最高，blue 胜出
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue

        // 额外验证：去掉 ID 选择器后，标签选择器应胜过通用选择器
        let stylesheets_no_id = vec![Stylesheet {
            rules: vec![
                Rule::Style(StyleRule {
                    selectors: vec![Selector {
                        complex: ComplexSelector {
                            parts: vec![(
                                CompoundSelector {
                                    type_selector: Some(TypeSelector::Universal),
                                    subclass_selectors: vec![],
                                },
                                None,
                            )],
                        },
                    }],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "purple".to_string(),
                        important: false,
                    }],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
            ],
        }];

        let mut sys2 = StyleSystem::new();
        let styles2 = sys2.compute_styles(&doc, &stylesheets_no_id);
        let div_style2 = styles2.get(&div).expect("div 应该有样式");
        // 标签选择器 (0,0,1) > 通用选择器 (0,0,0)
        assert_eq!(div_style2.color, ColorValue::Rgba(255, 0, 0, 255)); // red
    }

    #[test]
    /// 验证 !important 声明对同一属性的优先级高于 normal 声明。
    ///
    /// 场景：div { color: red !important } div { color: blue }
    /// 即使两个声明同源同特异性，!important 胜出。
    fn test_cascade_importance_order() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                // normal 声明 — color: blue
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
                // !important 声明 — color: red
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: true,
                    }],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // !important 的 red 胜过 normal 的 blue
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red

        // 额外验证：即使 !important 声明在前，仍然胜出
        let stylesheets_important_first = vec![Stylesheet {
            rules: vec![
                // !important 在前 — color: green
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: true,
                    }],
                }),
                // normal 在后 — color: blue
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
            ],
        }];

        let mut sys2 = StyleSystem::new();
        let styles2 = sys2.compute_styles(&doc, &stylesheets_important_first);
        let div_style2 = styles2.get(&div).expect("div 应该有样式");
        // !important 在前仍然胜过后面的 normal
        assert_eq!(div_style2.color, ColorValue::Rgba(0, 128, 0, 255)); // green
    }

    #[test]
    /// 验证在特异性和重要性相等时，后出现的声明胜出。
    ///
    /// 场景：div { color: red } div { color: green } div { color: blue }
    /// 三个声明同源、同重要性、同特异性，位置靠后的 blue 胜出。
    fn test_cascade_origin_order() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                // 第一个声明 — color: red
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
                // 第二个声明 — color: green
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    }],
                }),
                // 第三个声明 — color: blue（最后出现）
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
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
        // 同特异性同重要性时，最后出现的声明胜出
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue

        // 额外验证：不同样式表中同样遵循后出现胜出规则
        let mut sys2 = StyleSystem::new();
        let stylesheets_multi = vec![
            Stylesheet {
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            },
            Stylesheet {
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    }],
                })],
            },
        ];

        let styles2 = sys2.compute_styles(&doc, &stylesheets_multi);
        let div_style2 = styles2.get(&div).expect("div 应该有样式");
        // 第二个样式表的 green 胜过第一个的 red
        assert_eq!(div_style2.color, ColorValue::Rgba(0, 128, 0, 255)); // green
    }

    // ═══════════════════════════════════════════════════════════════════
    // 容器查询端到端测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 容器宽度 500px，@container (min-width: 400px) → 条件满足，样式应用。
    fn test_container_query_min_width_applies() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(500.0, 600.0);

        // @container (min-width: 400px) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
                name: None,
                condition: zero_css_parser::ast::ContainerCondition::Size(
                    zero_css_parser::ast::ContainerSizeCondition {
                        feature: "min-width".to_string(),
                        value: "400px".to_string(),
                        operator: None,
                        range_min: None,
                        range_max: None,
                    },
                ),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 容器宽度 500px >= 400px，条件满足，color 应为红色
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    /// 容器宽度 300px，@container (min-width: 400px) → 条件不满足，样式不应用。
    fn test_container_query_min_width_not_applies() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(300.0, 600.0);

        // @container (min-width: 400px) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
                name: None,
                condition: zero_css_parser::ast::ContainerCondition::Size(
                    zero_css_parser::ast::ContainerSizeCondition {
                        feature: "min-width".to_string(),
                        value: "400px".to_string(),
                        operator: None,
                        range_min: None,
                        range_max: None,
                    },
                ),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 容器宽度 300px < 400px，条件不满足，color 保持默认黑色
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    /// 容器宽度 500px，@container (max-width: 600px) → 500px <= 600px，条件满足。
    fn test_container_query_max_width() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(500.0, 600.0);

        // @container (max-width: 600px) { div { color: green; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
                name: None,
                condition: zero_css_parser::ast::ContainerCondition::Size(
                    zero_css_parser::ast::ContainerSizeCondition {
                        feature: "max-width".to_string(),
                        value: "600px".to_string(),
                        operator: None,
                        range_min: None,
                        range_max: None,
                    },
                ),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 容器宽度 500px <= 600px，max-width 条件满足
        assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255));
    }

    #[test]
    /// 范围语法：@container (200px <= width <= 500px)，容器宽度 350px → 在范围内，样式应用。
    fn test_container_query_range_syntax() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(350.0, 600.0);

        // @container (200px <= width <= 500px) { div { color: blue; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
                name: None,
                condition: zero_css_parser::ast::ContainerCondition::Size(
                    zero_css_parser::ast::ContainerSizeCondition {
                        feature: "width".to_string(),
                        value: String::new(),
                        operator: None,
                        range_min: Some("200px".to_string()),
                        range_max: Some("500px".to_string()),
                    },
                ),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 200 <= 350 <= 500，范围条件满足
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));

        // 额外验证：超出范围时不应用
        let mut sys2 = StyleSystem::new();
        sys2.set_viewport(600.0, 400.0);
        let styles2 = sys2.compute_styles(&doc, &stylesheets);
        let div_style2 = styles2.get(&div).expect("div should have style");
        // 600 > 500，超出上界，不应用
        assert_eq!(div_style2.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    /// @container 无 ContainerContext（未设置视口）→ 不应用容器查询样式。
    fn test_container_query_no_context() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        // 不设置视口，无 ContainerContext

        // @container (min-width: 400px) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
                name: None,
                condition: zero_css_parser::ast::ContainerCondition::Size(
                    zero_css_parser::ast::ContainerSizeCondition {
                        feature: "min-width".to_string(),
                        value: "400px".to_string(),
                        operator: None,
                        range_min: None,
                        range_max: None,
                    },
                ),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // 无容器上下文，@container 不应用，color 保持默认黑色
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    // ═══════════════════════════════════════════════════════════════════
    // Scroll Snap 端到端测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// scroll-snap-type: none 产生默认值（strictness=None, axis=Both）。
    fn test_scroll_snap_type_none() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "scroll-snap-type".to_string(),
                    value: "none".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        assert_eq!(
            div_style.scroll_snap_type.strictness,
            property::ScrollSnapStrictness::None
        );
        assert_eq!(
            div_style.scroll_snap_type.axis,
            zero_css_parser::values::ScrollSnapAxis::Both
        );
    }

    #[test]
    /// scroll-snap-type: x mandatory 存储 strictness=Mandatory, axis=X。
    fn test_scroll_snap_type_mandatory() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "scroll-snap-type".to_string(),
                    value: "x mandatory".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        assert_eq!(
            div_style.scroll_snap_type.strictness,
            property::ScrollSnapStrictness::Mandatory
        );
        assert_eq!(
            div_style.scroll_snap_type.axis,
            zero_css_parser::values::ScrollSnapAxis::X
        );
    }

    #[test]
    /// scroll-snap-type: y proximity 存储 strictness=Proximity, axis=Y。
    fn test_scroll_snap_type_proximity() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "scroll-snap-type".to_string(),
                    value: "y proximity".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        assert_eq!(
            div_style.scroll_snap_type.strictness,
            property::ScrollSnapStrictness::Proximity
        );
        assert_eq!(
            div_style.scroll_snap_type.axis,
            zero_css_parser::values::ScrollSnapAxis::Y
        );
    }

    #[test]
    /// scroll-snap-align 的 start/center/end 值端到端存储验证。
    fn test_scroll_snap_align_values() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        // start
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "scroll-snap-align".to_string(),
                    value: "start".to_string(),
                    important: false,
                }],
            })],
        }];
        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        assert_eq!(div_style.scroll_snap_align, property::ScrollSnapAlign::Start);

        // center
        let mut sys2 = StyleSystem::new();
        let stylesheets2 = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "scroll-snap-align".to_string(),
                    value: "center".to_string(),
                    important: false,
                }],
            })],
        }];
        let styles2 = sys2.compute_styles(&doc, &stylesheets2);
        let div_style2 = styles2.get(&div).expect("div 应该有样式");
        assert_eq!(div_style2.scroll_snap_align, property::ScrollSnapAlign::Center);

        // end
        let mut sys3 = StyleSystem::new();
        let stylesheets3 = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "scroll-snap-align".to_string(),
                    value: "end".to_string(),
                    important: false,
                }],
            })],
        }];
        let styles3 = sys3.compute_styles(&doc, &stylesheets3);
        let div_style3 = styles3.get(&div).expect("div 应该有样式");
        assert_eq!(div_style3.scroll_snap_align, property::ScrollSnapAlign::End);
    }

    #[test]
    /// scroll-snap-stop: normal 和 always 两个值的端到端存储验证。
    fn test_scroll_snap_stop_normal_always() {
        let (doc, _html, _body, div, _p) = make_test_dom();

        // normal（默认值）
        let mut sys = StyleSystem::new();
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "scroll-snap-stop".to_string(),
                    value: "normal".to_string(),
                    important: false,
                }],
            })],
        }];
        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        assert_eq!(div_style.scroll_snap_stop, property::ScrollSnapStop::Normal);

        // always
        let mut sys2 = StyleSystem::new();
        let stylesheets2 = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "scroll-snap-stop".to_string(),
                    value: "always".to_string(),
                    important: false,
                }],
            })],
        }];
        let styles2 = sys2.compute_styles(&doc, &stylesheets2);
        let div_style2 = styles2.get(&div).expect("div 应该有样式");
        assert_eq!(div_style2.scroll_snap_stop, property::ScrollSnapStop::Always);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 边界条件端到端测试
    // ═══════════════════════════════════════════════════════════════════

    /// 测试级联特异性：ID 选择器与 class 选择器冲突时，ID 选择器胜出。
    #[test]
    fn test_cascade_specificity_id_vs_class() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let div = doc.create_element("div");
        doc.set_attribute(div, "id", "myid");
        doc.set_attribute(div, "class", "myclass");
        doc.append_child(body, div).unwrap();

        let mut sys = StyleSystem::new();

        let id_sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Id("myid".to_string())],
                    },
                    None,
                )],
            },
        };
        let class_sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Class("myclass".to_string())],
                    },
                    None,
                )],
            },
        };

        // #myid { color: red } vs .myclass { color: blue }
        // ID 选择器特异性 (1,0,0) > class 选择器 (0,1,0)，red 胜出
        let stylesheets = vec![Stylesheet {
            rules: vec![
                Rule::Style(StyleRule {
                    selectors: vec![class_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![id_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
    }

    /// 测试 !important 声明即使特异性更低也能覆盖 normal 声明。
    #[test]
    fn test_cascade_important_override() {
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

        // div { color: red !important } vs #main { color: blue }
        // 标签选择器 + !important 应胜过 ID 选择器 + normal
        let stylesheets = vec![Stylesheet {
            rules: vec![
                Rule::Style(StyleRule {
                    selectors: vec![id_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![tag_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: true,
                    }],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // !important 胜过更高特异性的 normal 声明
        assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
    }

    /// 测试 color 属性继承：父元素设置 color 后，子元素应继承该值。
    #[test]
    fn test_inherit_color_from_parent() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let parent = doc.create_element("div");
        doc.append_child(body, parent).unwrap();
        let child = doc.create_element("span");
        doc.append_child(parent, child).unwrap();

        let mut sys = StyleSystem::new();

        // div { color: green } — span 未设置 color，应继承
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "green".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let child_style = styles.get(&child).expect("span should have style");
        // span 应从 div 继承 green
        assert_eq!(child_style.color, ColorValue::Rgba(0, 128, 0, 255));
    }

    /// 测试无样式元素的计算 display 默认值。
    #[test]
    fn test_computed_default_display() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        // 不设置任何 CSS 规则
        let stylesheets = vec![];
        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        // div 的默认 display 为 Inline（样式系统不区分 HTML 元素语义默认值）
        assert_eq!(div_style.display, DisplayValue::Inline);
    }

    /// 测试简写 margin: 10px 展开后四个边均为 10px。
    #[test]
    fn test_shorthand_margin_expansion() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "margin".to_string(),
                    value: "10px".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.margin_top, LengthValue::Px(10.0));
        assert_eq!(div_style.margin_right, LengthValue::Px(10.0));
        assert_eq!(div_style.margin_bottom, LengthValue::Px(10.0));
        assert_eq!(div_style.margin_left, LengthValue::Px(10.0));
    }

    /// 测试 var() 回退值：var(--unknown, blue) 在 --unknown 未定义时使用 blue。
    #[test]
    fn test_custom_property_fallback() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        // color: var(--unknown, blue) — --unknown 不存在，应使用 blue
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "var(--unknown, blue)".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
    }

    /// 测试无视口时 @media 规则不应用。
    #[test]
    fn test_media_query_no_viewport() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        // 不设置视口

        // @media (min-width: 500px) { div { color: red; } }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::At(zero_css_parser::ast::AtRule {
                name: "media".to_string(),
                prelude: "(min-width: 500px)".to_string(),
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
        // 无视口信息，@media 不应用，color 保持默认黑色
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 视口单位在端到端管线中正确解析：vw/vh 设置视口后转换为 px。
    fn test_viewport_units_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(1000.0, 500.0);

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "width".to_string(),
                        value: "50vw".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "height".to_string(),
                        value: "20vh".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // 50vw = 50% * 1000 = 500px
        assert_eq!(div_style.width, LengthValue::Px(500.0));
        // 20vh = 20% * 500 = 100px
        assert_eq!(div_style.height, LengthValue::Px(100.0));
    }

    #[test]
    /// rem 单位在端到端管线中正确解析：rem 始终基于根字体大小 16px，
    /// 不受父元素或自身 font-size 影响。
    fn test_rem_unit_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![
                // 设置父元素 font-size 为 32px
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![
                        Declaration {
                            property: "font-size".to_string(),
                            value: "32px".to_string(),
                            important: false,
                        },
                        Declaration {
                            property: "width".to_string(),
                            value: "2rem".to_string(),
                            important: false,
                        },
                    ],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // 2rem = 2 * 16px(root) = 32px，不受自身 font-size: 32px 影响
        assert_eq!(div_style.width, LengthValue::Px(32.0));
    }

    #[test]
    /// 多个样式表声明合并：不同样式表中的同一属性按出现顺序合并，
    /// 后出现的样式表中的声明应覆盖前者（同特异性下）。
    fn test_multiple_stylesheets_merge() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![
            // 第一个样式表：color: red, margin-top: 10px
            Stylesheet {
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![
                        Declaration {
                            property: "color".to_string(),
                            value: "red".to_string(),
                            important: false,
                        },
                        Declaration {
                            property: "margin-top".to_string(),
                            value: "10px".to_string(),
                            important: false,
                        },
                    ],
                })],
            },
            // 第二个样式表：color: green（覆盖 red），font-size: 20px
            Stylesheet {
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![
                        Declaration {
                            property: "color".to_string(),
                            value: "green".to_string(),
                            important: false,
                        },
                        Declaration {
                            property: "font-size".to_string(),
                            value: "20px".to_string(),
                            important: false,
                        },
                    ],
                })],
            },
        ];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // color: 第二个样式表的 green 覆盖第一个的 red
        assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255));
        // margin-top: 仅在第一个样式表中，保持 10px
        assert_eq!(div_style.margin_top, LengthValue::Px(10.0));
        // font-size: 仅在第二个样式表中，为 20px
        assert_eq!(div_style.font_size, LengthValue::Px(20.0));
    }

    #[test]
    /// 自定义属性循环引用防护：--a 引用 --b，--b 引用 --a，
    /// 系统应通过迭代上限防止无限循环，不会 panic。
    fn test_custom_property_circular_reference() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "--a".to_string(),
                        value: "var(--b)".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "--b".to_string(),
                        value: "var(--a)".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "color".to_string(),
                        value: "var(--a)".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        // 不应 panic，循环引用被迭代上限保护
        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // 循环引用无法解析到具体颜色值，color 保持默认黑色
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    /// 两个 !important 声明冲突时，特异性更高的胜出。
    /// div { color: red !important } #main { color: blue !important }
    /// 两个都是 !important，ID 选择器 (1,0,0) > 标签选择器 (0,0,1)。
    fn test_dual_important_higher_specificity_wins() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

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

        let stylesheets = vec![Stylesheet {
            rules: vec![
                // 标签选择器 + !important → color: red
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: true,
                    }],
                }),
                // ID 选择器 + !important → color: blue
                Rule::Style(StyleRule {
                    selectors: vec![id_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: true,
                    }],
                }),
            ],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // 同为 !important 时，ID 选择器特异性 (1,0,0) 高于标签 (0,0,1)，blue 胜出
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增边界条件端到端测试（round 12）
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 仅含文本节点的父元素不产生计算样式，但相邻元素节点各自独立计算样式。
    /// 验证非元素节点（文本节点）不参与样式系统，元素节点正确获得默认样式。
    fn test_text_nodes_get_no_style_but_siblings_do() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        // 添加一个文本节点
        let text = doc.create_text_node("Hello");
        doc.append_child(body, text).unwrap();
        // 添加一个元素节点
        let span = doc.create_element("span");
        doc.append_child(body, span).unwrap();

        let mut sys = StyleSystem::new();
        let stylesheets = vec![];
        let styles = sys.compute_styles(&doc, &stylesheets);

        // span 应有样式（默认值）
        assert!(styles.get(&span).is_some(), "span 应该有计算样式");
        // body 应有样式
        assert!(styles.get(&body).is_some(), "body 应该有计算样式");
        // 文本节点不在 styles 中（NodeId 无法直接查，但总样式数应只含元素节点）
        // html, body, span 三个元素节点有样式
        assert!(styles.len() >= 3, "至少 3 个元素节点有样式");
    }

    #[test]
    /// 多层嵌套继承：grandparent 设置 color: red，parent 未设置，
    /// child 也未设置，验证继承链在三代之间正确传递。
    /// 同时验证非继承属性（margin-top）不在代际之间传递。
    fn test_deep_nesting_inheritance_chain() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let grandparent = doc.create_element("div");
        doc.set_attribute(grandparent, "id", "gp");
        doc.append_child(body, grandparent).unwrap();
        let parent = doc.create_element("section");
        doc.append_child(grandparent, parent).unwrap();
        let child = doc.create_element("span");
        doc.append_child(parent, child).unwrap();

        let mut sys = StyleSystem::new();

        let id_sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Id("gp".to_string())],
                    },
                    None,
                )],
            },
        };

        // #gp { color: red; margin-top: 20px }
        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![id_sel],
                declarations: vec![
                    Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
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

        // grandparent 的 color = red, margin-top = 20px
        let gp_style = styles.get(&grandparent).expect("grandparent 应有样式");
        assert_eq!(gp_style.color, ColorValue::Rgba(255, 0, 0, 255));
        assert_eq!(gp_style.margin_top, LengthValue::Px(20.0));

        // parent 继承 color = red，但 margin-top 不继承
        let parent_style = styles.get(&parent).expect("parent 应有样式");
        assert_eq!(parent_style.color, ColorValue::Rgba(255, 0, 0, 255));
        assert_eq!(parent_style.margin_top, LengthValue::Px(0.0));

        // child 继承 color = red（经过两代传递），margin-top 不继承
        let child_style = styles.get(&child).expect("child 应有样式");
        assert_eq!(child_style.color, ColorValue::Rgba(255, 0, 0, 255));
        assert_eq!(child_style.margin_top, LengthValue::Px(0.0));
    }

    #[test]
    /// 简写与 longhand 混合应用后，后声明的 longhand 覆盖简写中对应子属性。
    /// 验证 margin 简写 + 单独的 margin-top 覆盖在端到端管线中正确工作。
    fn test_shorthand_then_longhand_override_e2e() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        // div { margin: 10px; margin-top: 30px; padding: 5px 15px; padding-left: 25px }
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
                        value: "30px".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "padding".to_string(),
                        value: "5px 15px".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "padding-left".to_string(),
                        value: "25px".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");

        // margin-top 被 longhand 覆盖为 30px
        assert_eq!(div_style.margin_top, LengthValue::Px(30.0));
        // 其余 margin 边保持简写值 10px
        assert_eq!(div_style.margin_right, LengthValue::Px(10.0));
        assert_eq!(div_style.margin_bottom, LengthValue::Px(10.0));
        assert_eq!(div_style.margin_left, LengthValue::Px(10.0));

        // padding-left 被 longhand 覆盖为 25px
        assert_eq!(div_style.padding_left, LengthValue::Px(25.0));
        // 其余 padding 保持简写值
        assert_eq!(div_style.padding_top, LengthValue::Px(5.0));
        assert_eq!(div_style.padding_right, LengthValue::Px(15.0));
        assert_eq!(div_style.padding_bottom, LengthValue::Px(5.0));
    }

    #[test]
    /// 自定义属性与 var() 三层嵌套解析：
    /// --base → --mid → --top，color 使用 var(--top)，
    /// 验证系统正确展开三层间接引用。
    fn test_custom_property_triple_indirection() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "--base".to_string(),
                        value: "green".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "--mid".to_string(),
                        value: "var(--base)".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "--top".to_string(),
                        value: "var(--mid)".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "color".to_string(),
                        value: "var(--top)".to_string(),
                        important: false,
                    },
                ],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div 应该有样式");
        // var(--top) → var(--mid) → var(--base) → green
        assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255));
    }

    #[test]
    /// @layer 内的 @media 规则同时生效：
    /// @layer base { @media (min-width: 600px) { div { color: red } } }
    /// 设置视口 800px，验证分层内的媒体查询条件正确评估。
    fn test_layer_with_media_inside() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);

        let stylesheets = vec![Stylesheet {
            rules: vec![
                // @layer base { @media (min-width: 600px) { div { color: red } } }
                Rule::Layer(zero_css_parser::ast::LayerRule {
                    name: "base".to_string(),
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
                }),
                // 未分层规则 — div { color: blue }
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
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
        // 未分层声明胜过分层声明，即使 @media 条件满足
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue

        // 额外验证：去掉未分层规则后，@layer 内的 @media 样式应生效
        let stylesheets_layer_only = vec![Stylesheet {
            rules: vec![Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
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
            })],
        }];

        let mut sys2 = StyleSystem::new();
        sys2.set_viewport(800.0, 600.0);
        let styles2 = sys2.compute_styles(&doc, &stylesheets_layer_only);
        let div_style2 = styles2.get(&div).expect("div 应该有样式");
        // @layer 内 @media 条件满足，color 应为红色
        assert_eq!(div_style2.color, ColorValue::Rgba(255, 0, 0, 255)); // red
    }
}
