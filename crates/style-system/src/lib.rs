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
            let computed =
                self.compute_element_style_internal(doc, node, stylesheets, parent_style);
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
        for (position, (property, value, important, specificity, layer_index)) in expanded_with_layer.iter().enumerate() {
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
        assert_eq!(
            div_style.border_top_style,
            property::BorderStyleValue::Solid
        );
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
                condition: zero_css_parser::ast::SupportsCondition::Property(
                    "display".to_string(),
                    "grid".to_string(),
                ),
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
                    zero_css_parser::ast::SupportsCondition::Property(
                        "display".to_string(),
                        "grid".to_string(),
                    ),
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
                    zero_css_parser::ast::SupportsCondition::Property(
                        "display".to_string(),
                        "flex".to_string(),
                    ),
                    zero_css_parser::ast::SupportsCondition::Property(
                        "color".to_string(),
                        "blue".to_string(),
                    ),
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
                    zero_css_parser::ast::SupportsCondition::Property(
                        "display".to_string(),
                        "unknown".to_string(),
                    ),
                    zero_css_parser::ast::SupportsCondition::Property(
                        "display".to_string(),
                        "flex".to_string(),
                    ),
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

    // ── Grid 属性端到端测试 ──

    #[test]
    fn test_grid_template_columns_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                }, Declaration {
                    property: "grid-template-columns".to_string(),
                    value: "100px 1fr auto".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.display, DisplayValue::Grid);
        assert_eq!(
            div_style.grid_template_columns,
            Some("100px 1fr auto".to_string())
        );
    }

    #[test]
    fn test_grid_template_rows_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                }, Declaration {
                    property: "grid-template-rows".to_string(),
                    value: "50px 1fr".to_string(),
                    important: false,
                }],
            })],
        }];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");
        assert_eq!(div_style.display, DisplayValue::Grid);
        assert_eq!(
            div_style.grid_template_rows,
            Some("50px 1fr".to_string())
        );
    }

    #[test]
    fn test_grid_auto_flow_end_to_end() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let mut sys = StyleSystem::new();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                }, Declaration {
                    property: "grid-auto-flow".to_string(),
                    value: "column dense".to_string(),
                    important: false,
                }],
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
                                        subclass_selectors: vec![SubclassSelector::Id(
                                            "main".to_string(),
                                        )],
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
                declarations: vec![
                    Declaration {
                        property: "grid-column-start".to_string(),
                        value: "-1".to_string(),
                        important: false,
                    },
                ],
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
        assert!(!matches!(div_style.transform, zero_css_parser::values::TransformValue::None));
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
}
