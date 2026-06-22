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
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(dead_code))]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unnecessary_get_then_check)]

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
use zero_css_parser::media_query::PrefersColorSchemeValue;
use zero_dom::{Document, NodeId, NodeKind, QuirksMode};

/// 返回 HTML 元素的 UA 默认 display 值。
///
/// 根据 HTML 规范，不同元素有不同的默认 display 类型。
/// 未列出的元素默认为 CSS 初始值 `inline`。
pub fn ua_default_display(tag: &str) -> Option<DisplayValue> {
    Some(match tag {
        // 块级元素（对齐 HTML Living Standard UA 样式表的 display:block 列表）
        "html" | "address" | "article" | "aside" | "blockquote" | "body" | "dd" | "details" | "div" | "dl" | "dt"
        | "fieldset" | "figcaption" | "figure" | "footer" | "form" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
        | "header" | "hgroup" | "hr" | "legend" | "li" | "main" | "menu" | "nav" | "ol" | "p" | "pre" | "search"
        | "section" | "summary" | "ul" => DisplayValue::Block,

        // 表格元素
        "table" => DisplayValue::Table,
        "thead" => DisplayValue::TableHeaderGroup,
        "tbody" => DisplayValue::TableRowGroup,
        "tfoot" => DisplayValue::TableFooterGroup,
        "tr" => DisplayValue::TableRow,
        "td" | "th" => DisplayValue::TableCell,
        "caption" => DisplayValue::TableCaption,
        "col" => DisplayValue::TableColumn,
        "colgroup" => DisplayValue::TableColumnGroup,

        // 内联块级元素
        "img" | "video" | "audio" | "canvas" | "iframe" | "embed" | "object" | "input" | "button" | "select"
        | "textarea" => DisplayValue::InlineBlock,

        // display:none
        "script" | "style" | "link" | "meta" | "head" | "title" | "base" | "noscript" | "template" | "dialog" => {
            DisplayValue::None
        }

        // 内联元素 — 无需覆盖（CSS 初始值即为 inline）
        _ => return None,
    })
}

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
    /// 用户颜色方案偏好（对应 `prefers-color-scheme` 媒体查询）。
    prefers_color_scheme: PrefersColorSchemeValue,
}

impl StyleSystem {
    /// 创建新的样式系统实例。
    pub fn new() -> Self {
        Self {
            custom_properties: HashMap::new(),
            viewport_width: None,
            viewport_height: None,
            prefers_color_scheme: PrefersColorSchemeValue::Light,
        }
    }

    /// 设置视口尺寸。
    pub fn set_viewport(&mut self, width: f64, height: f64) {
        self.viewport_width = Some(width);
        self.viewport_height = Some(height);
    }

    /// 设置用户颜色方案偏好。
    pub fn set_prefers_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        self.prefers_color_scheme = scheme;
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

        // 读取文档 quirks mode
        let quirks_mode = doc.quirks_mode();

        // 从文档根开始 DFS
        let root = doc.root();
        self.compute_styles_recursive(doc, root, stylesheets, None, &HashMap::new(), &mut styles, quirks_mode);

        styles
    }

    /// 递归计算样式。
    #[allow(clippy::too_many_arguments)]
    fn compute_styles_recursive(
        &mut self,
        doc: &Document,
        node: NodeId,
        stylesheets: &[Stylesheet],
        parent_style: Option<&ComputedStyle>,
        parent_custom: &HashMap<String, String>,
        styles: &mut HashMap<NodeId, ComputedStyle>,
        quirks_mode: QuirksMode,
    ) {
        let node_data = match doc.get(node) {
            Some(n) => n,
            None => return,
        };

        // 判断是否为元素节点
        let is_element = matches!(&node_data.kind, NodeKind::Element(_));

        // 只为元素节点计算样式
        if is_element {
            let mut computed = self.compute_element_style_internal(
                doc,
                node,
                stylesheets,
                parent_style,
                parent_custom,
                quirks_mode,
                None,
            );
            // 计算伪元素（::before/::after）：继承自本元素的计算样式。save/restore
            // custom_properties 以防伪元素规则定义的 var 污染后续子元素继承。
            // compute_element_style_internal 在无匹配规则时早返默认（content:Normal），
            // 故无伪元素规则的元素仅多 2 次（廉价的）声明收集。
            let saved_custom = self.custom_properties.clone();
            let elem_style = computed.clone();
            let before = self.compute_element_style_internal(
                doc,
                node,
                stylesheets,
                Some(&elem_style),
                &saved_custom,
                quirks_mode,
                Some("before"),
            );
            let after = self.compute_element_style_internal(
                doc,
                node,
                stylesheets,
                Some(&elem_style),
                &saved_custom,
                quirks_mode,
                Some("after"),
            );
            self.custom_properties = saved_custom;
            if matches!(
                before.content,
                property::types::ContentComputedValue::String(_) | property::types::ContentComputedValue::Attr(_)
            ) {
                computed.before_pseudo = Some(Box::new(before));
            }
            if matches!(
                after.content,
                property::types::ContentComputedValue::String(_) | property::types::ContentComputedValue::Attr(_)
            ) {
                computed.after_pseudo = Some(Box::new(after));
            }
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

        // 子元素继承当前元素已解析的自定义属性（自定义属性是继承属性）。
        // 非元素节点不计算样式，其子节点沿用 parent_custom（隔代继承到最近元素祖先）。
        // 注意：必须在进入子树前一次性捕获，因为递归会覆写 self.custom_properties。
        let current_custom = if is_element {
            self.custom_properties.clone()
        } else {
            parent_custom.clone()
        };

        for child in children {
            self.compute_styles_recursive(
                doc,
                child,
                stylesheets,
                parent_ref,
                &current_custom,
                styles,
                quirks_mode,
            );
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
        self.compute_element_style_internal(
            doc,
            element,
            stylesheets,
            parent_style,
            &HashMap::new(),
            doc.quirks_mode(),
            None,
        )
    }

    /// 内部实现：计算单个元素的样式。
    ///
    /// `pseudo` 为 `Some(name)` 时计算指定伪元素（`::before`/`::after`）的样式：
    /// 仅收集该伪元素的声明，跳过内联样式与 UA 默认值（伪元素无 style 属性、无标签），
    /// 继承自 `parent_style`（伪元素的 originating 元素）。
    #[allow(clippy::too_many_arguments)]
    fn compute_element_style_internal(
        &mut self,
        doc: &Document,
        element: NodeId,
        stylesheets: &[Stylesheet],
        parent_style: Option<&ComputedStyle>,
        parent_custom: &HashMap<String, String>,
        quirks_mode: QuirksMode,
        pseudo: Option<&str>,
    ) -> ComputedStyle {
        // 0. 构建媒体查询上下文
        let media_ctx = match (self.viewport_width, self.viewport_height) {
            (Some(w), Some(h)) => {
                let mut ctx = zero_css_parser::media_query::MediaContext::new(w, h);
                ctx.prefers_color_scheme = self.prefers_color_scheme;
                Some(ctx)
            }
            _ => None,
        };

        // 0.5 构建容器查询上下文（简化：使用视口尺寸作为默认容器尺寸）
        let container_ctx = match (self.viewport_width, self.viewport_height) {
            (Some(w), Some(h)) => Some(matcher::ContainerContext::with_size(w, h)),
            _ => None,
        };

        // 1. 收集匹配的声明（带媒体查询和容器查询评估）
        //    pseudo=Some(name) 时收集该伪元素的声明（::before/::after 路由）。
        let matching = match pseudo {
            None => matcher::collect_matching_declarations_with_media(
                doc,
                element,
                stylesheets,
                media_ctx.as_ref(),
                container_ctx.as_ref(),
            ),
            Some(name) => matcher::collect_pseudo_declarations_with_media(
                doc,
                element,
                stylesheets,
                media_ctx.as_ref(),
                container_ctx.as_ref(),
                name,
            ),
        };

        // 伪元素无匹配规则时直接返回默认值（content: Normal），跳过整条级联/继承/计算
        // 管线——避免对无伪元素规则的元素产生 2× 额外开销。调用方据 content 判定是否合成盒。
        if pseudo.is_some() && matching.is_empty() {
            return ComputedStyle::default();
        }

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

        // 1.6. 解析内联样式（style 属性）
        // 内联样式的优先级高于任何选择器，使用 (1, 0, 0) 特异性。
        // 伪元素无 style 属性，跳过。
        if pseudo.is_none()
            && let Some(style_attr) = doc.get_attribute(element, "style")
        {
            let inline_decls = parse_inline_style(&style_attr);
            if !inline_decls.is_empty() {
                #[allow(clippy::type_complexity)]
                let input: Vec<(String, String, bool, (u32, u32, u32))> = inline_decls
                    .iter()
                    .map(|(p, v, imp)| (p.clone(), v.clone(), *imp, (1, 0, 0)))
                    .collect();
                let expanded = shorthand::expand_shorthands(&input);
                for (prop, val, imp, spec) in expanded {
                    // 内联样式没有 layer，使用 None
                    expanded_with_layer.push((prop, val, imp, spec, None));
                }
            }
        }

        // 1.7. 注入 UA 默认声明（最低优先级，可被作者样式覆盖）
        // 伪元素无标签、无 UA 默认值（::before/::after 默认 display:inline 由 ComputedStyle::default 提供）；
        // 置 tag_name=None 同时跳过下方 UA 推送与步骤 7 的 tag-based quirks。
        let tag_name = if pseudo.is_none() {
            doc.get(element).and_then(|n| {
                if let NodeKind::Element(elem) = &n.kind {
                    Some(elem.local_name().to_lowercase())
                } else {
                    None
                }
            })
        } else {
            None
        };
        #[allow(clippy::type_complexity)]
        let mut ua_decl_inputs: Vec<(String, String, bool, (u32, u32, u32), Option<usize>)> = Vec::new();
        if let Some(ref tag) = tag_name
            && let Some(display) = ua_default_display(tag)
        {
            let display_str = match display {
                DisplayValue::Block => "block",
                DisplayValue::Table => "table",
                DisplayValue::InlineTable => "inline-table",
                DisplayValue::TableRow => "table-row",
                DisplayValue::TableCell => "table-cell",
                DisplayValue::TableCaption => "table-caption",
                DisplayValue::TableColumn => "table-column",
                DisplayValue::TableColumnGroup => "table-column-group",
                DisplayValue::TableRowGroup => "table-row-group",
                DisplayValue::TableHeaderGroup => "table-header-group",
                DisplayValue::TableFooterGroup => "table-footer-group",
                DisplayValue::InlineBlock => "inline-block",
                DisplayValue::None => "none",
                _ => "inline",
            };
            ua_decl_inputs.push(("display".to_string(), display_str.to_string(), false, (0, 0, 0), None));
        }

        // UA 默认样式
        if let Some(ref tag) = tag_name {
            match tag.as_str() {
                // body margin: 8px（浏览器默认值）
                "body" => {
                    ua_decl_inputs.push(("margin".to_string(), "8px".to_string(), false, (0, 0, 0), None));
                }
                // h1-h6 默认 margin 和 font-weight/font-size
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    ua_decl_inputs.push((
                        "margin".to_string(),
                        match tag.as_str() {
                            "h1" => "0.67em 0",
                            "h2" => "0.83em 0",
                            "h3" => "1em 0",
                            "h4" => "1.33em 0",
                            "h5" => "1.67em 0",
                            "h6" => "2.33em 0",
                            _ => "1em 0",
                        }
                        .to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                    ua_decl_inputs.push(("font-weight".to_string(), "bold".to_string(), false, (0, 0, 0), None));
                }
                // p 默认 margin
                "p" => {
                    ua_decl_inputs.push(("margin".to_string(), "1em 0".to_string(), false, (0, 0, 0), None));
                }
                // ul/ol 默认 padding-left 和 margin
                "ul" | "ol" => {
                    ua_decl_inputs.push(("margin".to_string(), "1em 0".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("padding-left".to_string(), "40px".to_string(), false, (0, 0, 0), None));
                }
                _ => {}
            }
        }

        let mut ua_declarations: Vec<CascadedDeclaration> = Vec::new();
        if !ua_decl_inputs.is_empty() {
            let shorthand_inputs: Vec<(String, String, bool, (u32, u32, u32))> = ua_decl_inputs
                .iter()
                .map(|(property, value, important, specificity, _layer_index)| {
                    (property.clone(), value.clone(), *important, *specificity)
                })
                .collect();
            let expanded = shorthand::expand_shorthands(&shorthand_inputs);
            for (position, (property, value, important, specificity)) in expanded.into_iter().enumerate() {
                ua_declarations.push(CascadedDeclaration {
                    property,
                    value,
                    order: CascadeOrder::new(Origin::UserAgent, None, specificity, position, important),
                });
            }
        }

        // 2. 构建 CascadedDeclaration 列表
        let mut declarations = ua_declarations;
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

        // 4. 收集自定义属性（继承父元素 + 当前元素自身声明覆盖）
        // CSS 自定义属性是继承属性：`:root { --x }` 定义的变量需对后代可见。
        self.custom_properties = gather_custom_properties(&cascaded, parent_custom);

        // 4.5. 在级联值中解析 var() 引用
        let resolved_cascaded = resolve_var_in_cascaded(&cascaded, &self.custom_properties);

        // 5. 计算继承样式
        let style = inheritance::compute_inherited_style_with_quirks(parent_style, &resolved_cascaded, quirks_mode);

        // 6. 解析计算值（相对单位转换）
        // 提取父元素的计算 font-size，用于子元素 font-size 的 em 解析
        let parent_fs = parent_style.map(|ps| {
            // 父元素的 font_size 已经被解析为 Px
            match &ps.font_size {
                zero_css_parser::values::LengthValue::Px(v) => *v,
                _ => computed::ROOT_FONT_SIZE,
            }
        });
        let mut resolved = computed::resolve_computed_style(
            &style,
            &self.custom_properties,
            self.viewport_width,
            self.viewport_height,
            parent_fs,
        );

        // 7. Quirks mode 调整（复用步骤 1.7 已提取的 tag_name）
        if quirks_mode == QuirksMode::Quirks {
            apply_quirks_mode_adjustments(&mut resolved, parent_style, tag_name.as_deref());
        }

        resolved
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

/// 解析 HTML 元素的 inline style 属性值。
///
/// 将 `"background-color: red; width: 200px"` 格式的字符串解析为
/// `(property, value, important)` 三元组列表。
fn parse_inline_style(style_attr: &str) -> Vec<(String, String, bool)> {
    let mut declarations = Vec::new();
    // 按分号分割声明
    for decl in style_attr.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        // 按第一个冒号分割属性名和值
        if let Some(colon_pos) = decl.find(':') {
            let property = decl[..colon_pos].trim().to_lowercase();
            let mut value = decl[colon_pos + 1..].trim().to_string();
            if property.is_empty() || value.is_empty() {
                continue;
            }
            // 检查 !important
            let important = if let Some(bang_pos) = value.rfind("!important") {
                value = value[..bang_pos].trim().to_string();
                true
            } else if let Some(bang_pos) = value.rfind("! important") {
                value = value[..bang_pos].trim().to_string();
                true
            } else {
                false
            };
            declarations.push((property, value, important));
        }
    }
    declarations
}

/// 从级联值中收集自定义属性，并解析自定义属性值中的 var() 引用。
///
/// 自定义属性是继承属性：先继承父元素（`inherited`）的自定义属性，再用当前元素
/// 自身级联声明覆盖（自身优先），最后迭代解析值中的 var() 引用（可引用继承来的属性）。
fn gather_custom_properties(
    cascaded: &HashMap<String, String>,
    inherited: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut props: HashMap<String, String> = inherited.clone();
    for (k, v) in cascaded.iter().filter(|(k, _)| k.starts_with("--")) {
        props.insert(k.clone(), v.clone());
    }

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

/// 应用 quirks mode 样式调整。
///
/// 在 quirks mode 下，以下行为会改变：
/// - 百分比高度 quirks：当父元素高度为 auto 时，块级子元素的 `height: <percentage>` 视为 `auto`
/// - 表格高度 quirks：`<table>` 元素的 `height` 视为 `min-height`（height 设为 auto）
/// - inline 元素宽高 quirks：inline 元素的 `width`/`height` 在 quirks mode 下被保留
fn apply_quirks_mode_adjustments(
    style: &mut ComputedStyle,
    parent_style: Option<&ComputedStyle>,
    tag_name: Option<&str>,
) {
    use zero_css_parser::values::{DisplayValue, LengthValue};

    // 1. 百分比高度 quirks：
    // 在 quirks mode 中，如果父元素（block-level container）的高度不是明确指定的，
    // 则 block-level 子元素的 height: <percentage> 计算为 auto。
    //
    // 判断条件：父元素存在且父元素的 height 是 auto（而非明确指定值）
    if let Some(parent) = parent_style {
        let parent_height_is_auto = matches!(&parent.height, LengthValue::Auto);
        if parent_height_is_auto {
            // 如果当前元素 height 是百分比值，则回退为 auto
            if let LengthValue::Percentage(_) = &style.height {
                style.height = LengthValue::Auto;
            }
        }
    }

    // 2. 表格高度 quirks：
    // 在 quirks mode 下，<table> 元素的 height 被视为 min-height（CSS 2.1 §17.5.2）。
    // 实际高度由内容决定，但不会小于指定的 height 值。
    if let Some(tag) = tag_name
        && tag == "table"
        && !matches!(style.height, LengthValue::Auto)
    {
        style.min_height = style.height.clone();
        style.height = LengthValue::Auto;
    }

    // 3. inline 元素宽高 quirks：
    // 在 quirks mode 下，inline 元素的 width/height 被保留（CSS 2.1 规定 width/height 不适用于 inline non-replaced 元素）。
    // 当前的 layout engine 将 inline 映射为 block，所以此 quirks 实际上已经生效。
    // 这里不做额外处理——inline 元素的 width/height 自然保留。
    // 当 inline layout 正确实现后，需要在 standards mode 下将 inline 元素的 width/height 重置为 auto。
    let _ = DisplayValue::Inline; // suppress unused import warning
}

#[cfg(test)]
mod quirks_tests {
    use super::*;
    use zero_css_parser::values::LengthValue;

    /// 测试 quirks mode 下百分比高度回退为 auto
    ///
    /// 当父元素 height 为 auto 时，子元素的 height: <percentage> 应回退为 auto。
    #[test]
    fn test_quirks_mode_percentage_height_fallback() {
        let mut child_style = ComputedStyle::default();
        child_style.height = LengthValue::Percentage(50.0);

        let parent_style = ComputedStyle::default();
        // parent height is Auto by default

        apply_quirks_mode_adjustments(&mut child_style, Some(&parent_style), None);

        assert_eq!(
            child_style.height,
            LengthValue::Auto,
            "Quirks mode should convert percentage height to auto when parent height is auto"
        );
    }

    /// 测试 quirks mode 下父元素有明确高度时百分比高度不变
    #[test]
    fn test_quirks_mode_percentage_height_kept_with_explicit_parent() {
        let mut child_style = ComputedStyle::default();
        child_style.height = LengthValue::Percentage(50.0);

        let mut parent_style = ComputedStyle::default();
        parent_style.height = LengthValue::Px(200.0);

        apply_quirks_mode_adjustments(&mut child_style, Some(&parent_style), None);

        assert_eq!(
            child_style.height,
            LengthValue::Percentage(50.0),
            "Percentage height should be kept when parent has explicit height"
        );
    }

    /// 测试 quirks mode 下非百分比高度不受影响
    #[test]
    fn test_quirks_mode_px_height_unaffected() {
        let mut child_style = ComputedStyle::default();
        child_style.height = LengthValue::Px(100.0);

        let parent_style = ComputedStyle::default();

        apply_quirks_mode_adjustments(&mut child_style, Some(&parent_style), None);

        assert_eq!(
            child_style.height,
            LengthValue::Px(100.0),
            "Px height should not be affected by quirks mode"
        );
    }

    /// 测试 quirks mode 下无父元素时百分比高度不变
    #[test]
    fn test_quirks_mode_percentage_height_no_parent() {
        let mut child_style = ComputedStyle::default();
        child_style.height = LengthValue::Percentage(50.0);

        apply_quirks_mode_adjustments(&mut child_style, None, None);

        assert_eq!(
            child_style.height,
            LengthValue::Percentage(50.0),
            "Percentage height should be kept when no parent style"
        );
    }

    /// 测试 quirks mode 下 table 元素的 height 转为 min-height
    #[test]
    fn test_quirks_mode_table_height_as_min_height() {
        let mut table_style = ComputedStyle::default();
        table_style.height = LengthValue::Px(300.0);

        let mut parent_style = ComputedStyle::default();
        parent_style.height = LengthValue::Px(600.0);

        apply_quirks_mode_adjustments(&mut table_style, Some(&parent_style), Some("table"));

        assert_eq!(
            table_style.height,
            LengthValue::Auto,
            "Table height should be set to auto in quirks mode"
        );
        assert_eq!(
            table_style.min_height,
            LengthValue::Px(300.0),
            "Table height value should be moved to min-height in quirks mode"
        );
    }

    /// 测试 quirks mode 下非 table 元素的 height 不受影响
    #[test]
    fn test_quirks_mode_non_table_height_unaffected() {
        let mut div_style = ComputedStyle::default();
        div_style.height = LengthValue::Px(300.0);

        let mut parent_style = ComputedStyle::default();
        parent_style.height = LengthValue::Px(600.0);

        apply_quirks_mode_adjustments(&mut div_style, Some(&parent_style), Some("div"));

        assert_eq!(
            div_style.height,
            LengthValue::Px(300.0),
            "Non-table element height should not be affected by table quirk"
        );
    }

    /// 测试 quirks mode 下 table 元素 auto height 不受影响
    #[test]
    fn test_quirks_mode_table_auto_height_unaffected() {
        let mut table_style = ComputedStyle::default();
        // height is Auto by default

        let parent_style = ComputedStyle::default();

        apply_quirks_mode_adjustments(&mut table_style, Some(&parent_style), Some("table"));

        assert_eq!(
            table_style.height,
            LengthValue::Auto,
            "Table with auto height should remain auto"
        );
    }
}

#[cfg(test)]
mod ua_display_tests {
    use super::*;

    /// 回归测试：HTML 区块型元素必须默认 `display: block`。
    ///
    /// 历史缺陷（R253 morning-work 4× 高度根因）：`article`/`aside`/`details` 等标签缺失于
    /// `ua_default_display` 的 block 列表，回落到 CSS 初始值 `inline`。当此类「inline」元素含
    /// 块级子元素（h2/p）时，触发 R109（CSS2 §9.2.1.1）匿名块拆分，在每对块级子元素之间插入
    /// 包裹空白文本的幻影匿名块盒（继承父 node_id），把页面内容整体推开数倍高度
    ///（morning-work body 25301px ≈ chromium 5981px 的 4.2×）。
    ///
    /// 此测试钉死 HTML Living Standard UA 样式表中应为 `display:block` 的「分组/分节」元素，
    /// 防止再次遗漏导致同类幻影盒回归。
    #[test]
    fn test_html_block_level_sectioning_elements_default_to_block() {
        // R253 实证触发幻影盒的三个标签（修复前缺失）
        for tag in ["article", "aside", "details"] {
            assert_eq!(
                ua_default_display(tag),
                Some(DisplayValue::Block),
                "<{tag}> must default to display:block (was inline → R109 phantom anon blocks)"
            );
        }
        // 其余按 HTML Living Standard 应为 block 的分节/分组元素（防御性钉死）
        for tag in [
            "address",
            "blockquote",
            "dd",
            "div",
            "dl",
            "dt",
            "fieldset",
            "figcaption",
            "figure",
            "footer",
            "form",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "header",
            "hgroup",
            "hr",
            "li",
            "main",
            "menu",
            "nav",
            "ol",
            "p",
            "pre",
            "search",
            "section",
            "summary",
            "ul",
        ] {
            assert_eq!(
                ua_default_display(tag),
                Some(DisplayValue::Block),
                "<{tag}> should default to display:block per HTML UA stylesheet"
            );
        }
    }

    /// 内联元素（span/a/code 等）不得被错误标记为 block，否则破坏行内排版。
    #[test]
    fn test_inline_elements_remain_unset() {
        for tag in ["span", "a", "code", "em", "strong", "b", "i"] {
            assert_eq!(
                ua_default_display(tag),
                None,
                "<{tag}> should fall back to CSS initial inline (None), not block"
            );
        }
    }
}

#[cfg(test)]
mod tests;
