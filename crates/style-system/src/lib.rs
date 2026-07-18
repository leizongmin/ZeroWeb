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
        "html" | "address" | "article" | "aside" | "blockquote" | "body" | "center" | "dd" | "details" | "dir"
        | "div" | "dl" | "dt" | "fieldset" | "figcaption" | "figure" | "footer" | "form" | "h1" | "h2" | "h3"
        | "h4" | "h5" | "h6" | "header" | "hgroup" | "hr" | "legend" | "li" | "listing" | "main" | "menu" | "nav"
        | "ol" | "p" | "plaintext" | "pre" | "search" | "section" | "summary" | "ul" | "xmp" => DisplayValue::Block,

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
        "script" | "style" | "link" | "meta" | "head" | "title" | "base" | "bgsound" | "noframes" | "noembed"
        | "noscript" | "template" | "dialog" => DisplayValue::None,

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
        // 1.5a. CSS2 Appendix D 表现提示（img 的 width/height 属性 → 作者样式）。
        // Author origin + specificity (0,0,0) + 最早位置（cascade 按 origin/layer/specificity/
        // position 排序，(0,0,0) 低于任意真实选择器 ≥(0,0,1) 与 inline (1,0,0)，高于 UA 默认）。
        if pseudo.is_none() {
            let hints = collect_presentational_hints(doc, element);
            if !hints.is_empty() {
                #[allow(clippy::type_complexity)]
                let input: Vec<(String, String, bool, (u32, u32, u32))> = hints
                    .iter()
                    .map(|(p, v)| (p.clone(), v.clone(), false, (0, 0, 0)))
                    .collect();
                for (prop, val, imp, spec) in shorthand::expand_shorthands(&input) {
                    expanded_with_layer.push((prop, val, imp, spec, None));
                }
            }
        }
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
                // h1-h6 默认 margin、font-size 和 font-weight（HTML 渲染规范 UA 样式表）
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    ua_decl_inputs.push((
                        "font-size".to_string(),
                        match tag.as_str() {
                            "h1" => "2em",
                            "h2" => "1.5em",
                            "h3" => "1.17em",
                            "h4" => "1em",
                            "h5" => "0.83em",
                            "h6" => "0.67em",
                            _ => "1em",
                        }
                        .to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
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
                // pre/xmp/listing/plaintext：HTML 渲染规范 UA 样式表 white-space:pre（保留空白/换行）。
                // R1658：ZW default_impl white_space 默认 Normal，故 <pre> 此前折叠空白/换行（真 bug）。
                // 仅 white-space:pre（monospace 字体属 font-wall 高方差，单独 A/B 切片）。
                "pre" | "xmp" | "listing" | "plaintext" => {
                    ua_decl_inputs.push(("white-space".to_string(), "pre".to_string(), false, (0, 0, 0), None));
                }
                // ul/ol 默认 padding-left 和 margin
                "ul" | "ol" => {
                    ua_decl_inputs.push(("margin".to_string(), "1em 0".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("padding-left".to_string(), "40px".to_string(), false, (0, 0, 0), None));
                }
                "b" | "strong" => {
                    ua_decl_inputs.push(("font-weight".to_string(), "bold".to_string(), false, (0, 0, 0), None));
                }
                "th" => {
                    ua_decl_inputs.push(("font-weight".to_string(), "bold".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("text-align".to_string(), "center".to_string(), false, (0, 0, 0), None));
                }
                "i" | "em" => {
                    ua_decl_inputs.push(("font-style".to_string(), "italic".to_string(), false, (0, 0, 0), None));
                }
                "a" => {
                    let link_color = html_body_link_color(doc).unwrap_or_else(|| "#0000ee".to_string());
                    ua_decl_inputs.push(("color".to_string(), link_color, false, (0, 0, 0), None));
                    ua_decl_inputs.push((
                        "text-decoration".to_string(),
                        "underline".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                }
                // hr 默认渲染为水平线（HTML 渲染规范 / Chromium UA）：
                // display:block（已在 ua_default_display）+ 0.5em 上下 margin + inset 1px 边框。
                // 元素无内容（height:auto≈0），1px inset 上下边框合成 ~2px 阴影线。
                // SIZE/NOSHADE/WIDTH/ALIGN 由 presentational hints 覆盖（见 collect_presentational_hints）。
                // 注：pixel-diff 趋势上可能因上游 line-metric glyph 位置偏移而短期上升，但 DC-13
                // legacy 验收口径是「结构不崩」（goal line 318），hr 可见 = 结构正确性提升。
                "hr" => {
                    ua_decl_inputs.push(("margin".to_string(), "0.5em 0".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("border-style".to_string(), "inset".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("border-width".to_string(), "1px".to_string(), false, (0, 0, 0), None));
                }
                // R1396：表单控件默认外观（Chromium UA html.css）。旧实现把 button/input 渲成
                // 纯文本（无 bg/border/padding），legacy 表单页不可读。此处补 ButtonFace 近似灰
                // bg + 边框 + padding 使控件可见（产品验收口径「结构不崩、核心语义可见」）。
                // 精确像素匹配系统色 ButtonFace/Canvas 须系统色支持（后续），当前用近似 hex。
                "button" => {
                    ua_decl_inputs.push((
                        "background-color".to_string(),
                        "#d4d4d4".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                    ua_decl_inputs.push((
                        "border".to_string(),
                        "1px solid #767676".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                    ua_decl_inputs.push(("padding".to_string(), "1px 6px".to_string(), false, (0, 0, 0), None));
                }
                "input" | "select" | "textarea" => {
                    ua_decl_inputs.push((
                        "background-color".to_string(),
                        "white".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                    ua_decl_inputs.push((
                        "border".to_string(),
                        "1px solid #767676".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                    ua_decl_inputs.push(("padding".to_string(), "2px".to_string(), false, (0, 0, 0), None));

                    // R1659：`<input>` 固有尺寸（form-control intrinsic sizing）。
                    // `<input>` 是 void inline-block（无子节点），无固有尺寸时 ZW 把 auto 宽度当
                    // 全容器宽（fixture 37 实测 784×6 = body 内容宽 × padding/border），致包裹它的
                    // `<label>` 被迫换行 → 同父 label 盒大面积重叠（DC-13 struct FAIL）+ submit
                    // input 与 `<button>` 重叠。Chromium 把表单控件建模为有固有尺寸的替换类元素
                    //（text 默认 size=20 字符宽、checkbox/radio ~13px 方框、submit/reset 按 value
                    // 文本宽）。此处按 HTML 渲染规范补 UA width/height（同 h1/p/hr/button UA 谱系，
                    // 最低优先级 specificity(0,0,0)，可被作者/内联样式覆盖）。select/textarea 已按
                    // 内容（option/文本子节点）正确测宽，不加 width 以免覆盖内容尺寸。
                    if tag == "input" {
                        let itype = doc.get_attribute(element, "type").unwrap_or_default().to_lowercase();
                        // 字符宽度近似因子（默认字体平均字符宽，≈7px）。
                        const CHAR_PX: f32 = 7.0;
                        match itype.as_str() {
                            // checkbox/radio：固定方框（Chromium ~13px）。
                            "checkbox" | "radio" | "color" => {
                                ua_decl_inputs.push(("width".to_string(), "13px".to_string(), false, (0, 0, 0), None));
                                ua_decl_inputs.push(("height".to_string(), "13px".to_string(), false, (0, 0, 0), None));
                            }
                            // submit/reset/button/image：按 value 文本字符数估宽（value 即按钮标签）。
                            // image 有 src 走替换元素路径，给个最小宽兜底。
                            "submit" | "reset" | "button" | "image" => {
                                let value = doc.get_attribute(element, "value").unwrap_or_default();
                                let chars = value.chars().count().max(1) as f32;
                                // +14 ≈ 按钮左右 padding/border chrome（1px 6px padding + 2px border）。
                                let w = chars * CHAR_PX + 14.0;
                                ua_decl_inputs.push((
                                    "width".to_string(),
                                    format!("{:.0}px", w),
                                    false,
                                    (0, 0, 0),
                                    None,
                                ));
                            }
                            // 文本类（默认无 type 当 text）：按 size 属性（默认 20 字符）估宽 + 行高。
                            // height 给文本字段合理内容高（content-box，+padding/border 共 ~21px 总高）。
                            _ => {
                                let size = doc
                                    .get_attribute(element, "size")
                                    .and_then(|s| s.trim().parse::<f32>().ok())
                                    .filter(|&n| n >= 1.0)
                                    .unwrap_or(20.0);
                                // +8 ≈ 输入框左右 padding/border chrome（2px padding + 1px border ×2）。
                                let w = size * CHAR_PX + 8.0;
                                ua_decl_inputs.push((
                                    "width".to_string(),
                                    format!("{:.0}px", w),
                                    false,
                                    (0, 0, 0),
                                    None,
                                ));
                                ua_decl_inputs.push(("height".to_string(), "15px".to_string(), false, (0, 0, 0), None));
                            }
                        }
                    }
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

        // 8. CSS 2.1 §9.7：float 不为 none 时的计算 display 调整。
        //    (a) table-internal（table-row-group / header-group / footer-group / row /
        //        column / column-group / cell）→ block：元素脱离表格结构成为浮动块。
        //    (b) inline-level（inline / inline-block / inline-table / inline-flex /
        //        inline-grid）→ 对应 block-level（block / table / flex / grid），即「块化」
        //        （§9.7 row 2-3：浮动元素必须是 block-level）。修复 float-applies-to-012
        //        （inline-block + float:right 应块化为 block 后右侧浮动）等。
        if !matches!(resolved.float, zero_css_parser::values::FloatValue::None) {
            use zero_css_parser::values::DisplayValue as Dv;
            match resolved.display {
                Dv::TableRowGroup
                | Dv::TableHeaderGroup
                | Dv::TableFooterGroup
                | Dv::TableRow
                | Dv::TableColumn
                | Dv::TableColumnGroup
                | Dv::TableCell => resolved.display = Dv::Block,
                Dv::Inline | Dv::InlineBlock => resolved.display = Dv::Block,
                Dv::InlineTable => resolved.display = Dv::Table,
                Dv::InlineFlex => resolved.display = Dv::Flex,
                Dv::InlineGrid => resolved.display = Dv::Grid,
                _ => {}
            }
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
/// CSS2 Appendix D 表现提示：把 HTML 表现属性映射为作者样式声明（property, value）。
/// specificity 由调用方设为 (0,0,0)。
fn collect_presentational_hints(doc: &Document, element: NodeId) -> Vec<(String, String)> {
    let Some(n) = doc.get(element) else {
        return Vec::new();
    };
    let NodeKind::Element(elem) = &n.kind else {
        return Vec::new();
    };
    let tag = elem.local_name().to_ascii_lowercase();
    let mut hints = Vec::new();

    if let Some(text) = elem_attr(elem, "text") {
        hints.push(("color".to_string(), normalize_html_color(&text)));
    }

    match tag.as_str() {
        "img" => {
            if let Some(bg) = elem_attr(elem, "bgcolor") {
                hints.push(("background-color".to_string(), normalize_html_color(&bg)));
            }
            for attr in ["width", "height"] {
                if let Some(v) = elem_attr(elem, attr) {
                    if let Some(val) = html_length_attr(&v) {
                        hints.push((attr.to_string(), val));
                    }
                }
            }
            if let Some(align) = elem_attr(elem, "align") {
                if let Some(va) = html_align_to_vertical_align(&align) {
                    hints.push(("vertical-align".to_string(), va));
                }
            }
        }
        "table" => {
            if let Some(bg) = elem_attr(elem, "bgcolor") {
                hints.push(("background-color".to_string(), normalize_html_color(&bg)));
            }
            let w = table_border_width_attr(elem);
            if w > 0 {
                hints.push(("border".to_string(), format!("{w}px solid")));
            }
            if elem_attr(elem, "cellpadding").is_some() {
                hints.push(("border-collapse".to_string(), "separate".to_string()));
            }
            if let Some(cs) = elem_attr(elem, "cellspacing").and_then(|v| parse_html_px(&v)) {
                hints.push(("border-spacing".to_string(), format!("{cs}px")));
            }
            for attr in ["width", "height"] {
                if let Some(v) = elem_attr(elem, attr).and_then(|v| html_length_attr(&v)) {
                    hints.push((attr.to_string(), v));
                }
            }
        }
        "td" | "th" => {
            if let Some(bg) = parent_tr_bgcolor(doc, element) {
                hints.push(("background-color".to_string(), bg));
            }
            if let Some(bg) = elem_attr(elem, "bgcolor") {
                hints.push(("background-color".to_string(), normalize_html_color(&bg)));
            }
            if let Some(table_id) = ancestor_table(doc, element) {
                if table_border_width_attr_from_doc(doc, table_id) > 0 {
                    hints.push(("border".to_string(), "1px solid".to_string()));
                }
                if let Some(cp) = doc
                    .get_attribute(table_id, "cellpadding")
                    .and_then(|v| parse_html_px(&v))
                {
                    hints.push(("padding".to_string(), format!("{cp}px")));
                }
            }
            if let Some(align) = elem_attr(elem, "align") {
                if let Some(ta) = html_align_to_text_align(&align) {
                    hints.push(("text-align".to_string(), ta));
                }
            }
            for attr in ["width", "height"] {
                if let Some(v) = elem_attr(elem, attr).and_then(|v| html_length_attr(&v)) {
                    hints.push((attr.to_string(), v));
                }
            }
        }
        "tr" => {
            if let Some(table_id) = ancestor_table(doc, element) {
                if table_border_width_attr_from_doc(doc, table_id) > 0 {
                    hints.push(("border".to_string(), "1px solid".to_string()));
                }
            }
        }
        // HTML 3.2/4 <font> 元素的展示属性（COLOR/SIZE/FACE）。color/font-size/font-family
        // 均为继承属性，设置在 <font> 上会传播到其文本子节点。SIZE 用 em 倍数以正确随父
        // font-size 缩放（HTML 七级字号非线性刻度，HTML5 §10.4）。
        "font" => {
            if let Some(color) = elem_attr(elem, "color") {
                hints.push(("color".to_string(), normalize_html_color(&color)));
            }
            if let Some(face) = elem_attr(elem, "face") {
                let fam = font_family_from_face(&face);
                if !fam.is_empty() {
                    hints.push(("font-family".to_string(), fam));
                }
            }
            // 注：`<font size>` 暂未映射——1.5em 等刻度实测使 testpage-020 单元格增高
            //（行高叠加），净负向。待 cell-height spacing gap 先解再启用（见 master.md R808）。
        }
        // HTML <center>：等价 text-align:center（继承到块子元素的内联内容）。
        "center" => {
            hints.push(("text-align".to_string(), "center".to_string()));
        }
        // hr presentational hints（HTML 4 §13.2）：SIZE/NOSHADE/WIDTH/ALIGN。
        // chromium 实测：<hr size="3" noshade> = 3px 实心满宽带。模型：noshade→solid，
        // size=N→border-top-width:N（实心或 inset 的 N px 线，覆盖 UA 默认 1px 四边）。
        "hr" => {
            let noshade = elem_attr(elem, "noshade").is_some();
            let size = elem_attr(elem, "size").and_then(|v| parse_html_px(&v));
            if noshade {
                hints.push(("border-style".to_string(), "solid".to_string()));
            }
            if let Some(s) = size {
                if s >= 1.0 {
                    hints.push(("border-width".to_string(), format!("{s}px 0 0 0")));
                }
            }
            if let Some(v) = elem_attr(elem, "width").and_then(|v| html_length_attr(&v)) {
                hints.push(("width".to_string(), v));
            }
            if let Some(align) = elem_attr(elem, "align") {
                match align.to_ascii_lowercase().as_str() {
                    "left" => hints.push(("margin-right".to_string(), "auto".to_string())),
                    "right" => hints.push(("margin-left".to_string(), "auto".to_string())),
                    "center" => {
                        hints.push(("margin-left".to_string(), "auto".to_string()));
                        hints.push(("margin-right".to_string(), "auto".to_string()));
                    }
                    _ => {}
                }
            }
        }
        _ => {
            if let Some(bg) = elem_attr(elem, "bgcolor") {
                hints.push(("background-color".to_string(), normalize_html_color(&bg)));
            }
        }
    }

    hints
}

fn elem_attr(elem: &zero_dom::ElementData, name: &str) -> Option<String> {
    elem.get_attribute(name)
}

fn normalize_html_color(value: &str) -> String {
    value.trim().to_string()
}

/// 将 `<font face>` 属性值转为 CSS font-family 值：逗号分隔的字体名，含非标识符字符
///（空格等）的名字加引号。
fn font_family_from_face(face: &str) -> String {
    face.split(',')
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .map(|f| {
            let needs_quote = f.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'));
            if needs_quote {
                // 去掉既有引号后重新加，避免双引号
                let inner = f.trim_matches(|c| c == '"' || c == '\'');
                format!("\"{inner}\"")
            } else {
                f.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// 将 `<font size>` 属性（HTML 七级字号 1-7，或相对 ±N）转为 em 倍数。
/// 绝对值映射自 HTML5 §10.4 的非线性刻度（基准 size 3 = 1.0em）；相对值从基准 3 解析。
#[allow(dead_code)]
fn html_font_size_to_em(size: &str) -> Option<f32> {
    let v = size.trim();
    let (sign, n_str) = if let Some(rest) = v.strip_prefix('+') {
        (1i32, rest.trim())
    } else if let Some(rest) = v.strip_prefix('-') {
        (-1i32, rest.trim())
    } else {
        (0, v)
    };
    let n: i32 = n_str.parse().ok()?;
    // 相对值 ±N 解析为绝对级（从基准 3），并钳到 1..=7。
    let abs = if sign != 0 {
        (3 + sign * n).clamp(1, 7)
    } else if (1..=7).contains(&n) {
        n
    } else {
        return None;
    };
    let em = match abs {
        1 => 0.63,
        2 => 0.82,
        3 => 1.0,
        4 => 1.13,
        5 => 1.5,
        6 => 2.0,
        _ => 3.0, // 7
    };
    Some(em)
}

fn parse_html_px(value: &str) -> Option<f32> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    if let Ok(n) = v.parse::<f32>() {
        return Some(n);
    }
    v.strip_suffix("px").and_then(|n| n.parse().ok())
}

fn html_length_attr(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    let is_bare_number = v.chars().all(|c| c.is_ascii_digit() || c == '.');
    Some(if is_bare_number {
        format!("{v}px")
    } else {
        v.to_string()
    })
}

fn table_border_width_attr(elem: &zero_dom::ElementData) -> u32 {
    match elem_attr(elem, "border") {
        None => 0,
        Some(v) if v.trim().is_empty() => 1,
        Some(v) => v.trim().parse::<u32>().unwrap_or(1),
    }
}

fn table_border_width_attr_from_doc(doc: &Document, table_id: NodeId) -> u32 {
    doc.get(table_id)
        .and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(table_border_width_attr(e)),
            _ => None,
        })
        .unwrap_or(0)
}

fn parent_tr_bgcolor(doc: &Document, element: NodeId) -> Option<String> {
    let parent = doc.parent_node(element)?;
    let n = doc.get(parent)?;
    let NodeKind::Element(tr) = &n.kind else {
        return None;
    };
    if !tr.local_name().eq_ignore_ascii_case("tr") {
        return None;
    }
    elem_attr(tr, "bgcolor").map(|bg| normalize_html_color(&bg))
}

fn ancestor_table(doc: &Document, mut node: NodeId) -> Option<NodeId> {
    loop {
        if let Some(n) = doc.get(node)
            && let NodeKind::Element(e) = &n.kind
            && e.local_name().eq_ignore_ascii_case("table")
        {
            return Some(node);
        }
        node = doc.parent_node(node)?;
    }
}

fn html_align_to_text_align(align: &str) -> Option<String> {
    match align.trim().to_ascii_lowercase().as_str() {
        "left" => Some("left".to_string()),
        "center" | "middle" => Some("center".to_string()),
        "right" => Some("right".to_string()),
        _ => None,
    }
}

fn html_align_to_vertical_align(align: &str) -> Option<String> {
    match align.trim().to_ascii_lowercase().as_str() {
        "top" => Some("top".to_string()),
        "middle" | "center" => Some("middle".to_string()),
        "bottom" => Some("bottom".to_string()),
        _ => None,
    }
}

fn html_body_link_color(doc: &Document) -> Option<String> {
    doc.get_elements_by_tag_name("body")
        .into_iter()
        .next()
        .and_then(|body| doc.get_attribute(body, "link"))
        .map(|c| normalize_html_color(&c))
}

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
            // R1651：<center> HTML4 块级（等价 <div align=center>）；先前缺 → inline 致 4px 盒
            // 与块子元素 overlap（legacy-html fixture 17-center struct-check FAIL 抓到）。
            "center",
            "dd",
            "dir",
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
            "listing",
            "main",
            "menu",
            "nav",
            "ol",
            "p",
            "plaintext",
            "pre",
            "search",
            "section",
            "summary",
            "ul",
            "xmp",
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

    /// 隐藏元素（script/style/noframes/noscript 等）必须 display:none。
    /// `<noframes>` 内容在 frame-capable UA（含 chromium oracle，所有现代浏览器）中按
    /// HTML 渲染规范隐藏；`<noscript>` 在脚本启用时同理隐藏。R1657：legacy-html fixture
    /// 38-noframes 实测 ZW 误渲染 noframes 回退文本（5 行）vs chromium 隐藏（2 段）。
    #[test]
    fn test_hidden_elements_default_to_none() {
        for tag in [
            "script", "style", "link", "meta", "head", "title", "base", "bgsound", "noframes", "noembed", "noscript",
            "template", "dialog",
        ] {
            assert_eq!(
                ua_default_display(tag),
                Some(DisplayValue::None),
                "<{tag}> should default to display:none (hidden content) per HTML UA stylesheet"
            );
        }
    }
}

#[cfg(test)]
mod presentational_hint_tests {
    use super::*;
    use zero_dom::parse_html;

    #[test]
    fn body_bgcolor_maps_to_background_color() {
        let doc = parse_html("<body bgcolor=\"#FFFFCC\"><p>x</p></body>");
        let body = doc.get_elements_by_tag_name("body")[0];
        let hints = collect_presentational_hints(&doc, body);
        assert!(
            hints.iter().any(|(p, v)| p == "background-color" && v == "#FFFFCC"),
            "hints: {hints:?}"
        );
    }

    #[test]
    fn table_border_and_cell_padding_map_to_css() {
        let doc = parse_html("<table border=\"1\" cellpadding=\"6\"><tr><td>Layer</td></tr></table>");
        let table = doc.get_elements_by_tag_name("table")[0];
        let td = doc.get_elements_by_tag_name("td")[0];
        let table_hints = collect_presentational_hints(&doc, table);
        assert!(
            table_hints.iter().any(|(p, v)| p == "border" && v.contains("1px")),
            "table hints: {table_hints:?}"
        );
        let td_hints = collect_presentational_hints(&doc, td);
        assert!(
            td_hints.iter().any(|(p, v)| p == "padding" && v == "6px"),
            "td hints: {td_hints:?}"
        );
        assert!(
            td_hints.iter().any(|(p, _)| p == "border"),
            "td should inherit table border hint"
        );
    }

    #[test]
    fn anchor_ua_uses_body_link_color() {
        let doc = parse_html("<body LINK=\"#0000EE\"><a href=\"#\">x</a></body>");
        let mut system = StyleSystem::new();
        let styles = system.compute_styles(&doc, &[]);
        let a_id = doc.get_elements_by_tag_name("a")[0];
        let style = styles.get(&a_id).expect("anchor styled");
        assert!(
            matches!(&style.color, zero_css_parser::values::ColorValue::Rgba(0, 0, 238, _)),
            "link color {:?}",
            style.color
        );
    }

    #[test]
    fn tr_bgcolor_applies_to_cells_not_row() {
        let doc = parse_html("<table><tr bgcolor=\"#CCCCCC\"><th>Layer</th><td>x</td></tr></table>");
        let tr = doc.get_elements_by_tag_name("tr")[0];
        let th = doc.get_elements_by_tag_name("th")[0];
        let tr_hints = collect_presentational_hints(&doc, tr);
        assert!(
            !tr_hints.iter().any(|(p, _)| p == "background-color"),
            "tr should not get row-wide bgcolor: {tr_hints:?}"
        );
        let th_hints = collect_presentational_hints(&doc, th);
        assert!(
            th_hints.iter().any(|(p, v)| p == "background-color" && v == "#CCCCCC"),
            "th hints: {th_hints:?}"
        );
    }

    #[test]
    fn bold_tag_gets_font_weight_from_ua() {
        let doc = parse_html("<p><b>bold</b></p>");
        let mut system = StyleSystem::new();
        let styles = system.compute_styles(&doc, &[]);
        let b_id = doc.get_elements_by_tag_name("b")[0];
        let style = styles.get(&b_id).expect("b styled");
        assert!(matches!(
            style.font_weight,
            zero_css_parser::values::FontWeightValue::Bold
        ));
    }

    #[test]
    fn heading_gets_ua_font_size_and_weight() {
        let doc = parse_html("<body><h1>Title</h1><h2>Section</h2></body>");
        let mut system = StyleSystem::new();
        let styles = system.compute_styles(&doc, &[]);
        let h1_id = doc.get_elements_by_tag_name("h1")[0];
        let h2_id = doc.get_elements_by_tag_name("h2")[0];
        let h1 = styles.get(&h1_id).expect("h1 styled");
        let h2 = styles.get(&h2_id).expect("h2 styled");
        assert_eq!(h1.font_size, zero_css_parser::values::LengthValue::Px(32.0));
        assert_eq!(h2.font_size, zero_css_parser::values::LengthValue::Px(24.0));
        assert!(matches!(h1.font_weight, zero_css_parser::values::FontWeightValue::Bold));
    }

    /// `<pre>`/`<xmp>`/`<listing>`/`<plaintext>` 必须从 UA 样式表继承 `white-space: pre`
    /// （HTML 渲染规范）。R1658：ZW default_impl white_space 默认 Normal，pre-family 此前折叠
    /// 空白/换行（真 bug）；css-text 全 1644 oracle A/B net-0（零回归）+ legacy fixture 26/35
    /// 小幅改善（whitespace 保真）。monospace 字体独立 A/B 切片（font-wall 高方差）。
    #[test]
    fn pre_family_gets_white_space_pre_from_ua() {
        let doc = parse_html("<body><pre>p</pre><xmp>x</xmp><listing>l</listing><plaintext>t</plaintext></body>");
        let mut system = StyleSystem::new();
        let styles = system.compute_styles(&doc, &[]);
        for tag in ["pre", "xmp", "listing", "plaintext"] {
            let id = doc.get_elements_by_tag_name(tag)[0];
            let style = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
            assert!(
                matches!(style.white_space, WhiteSpaceValue::Pre),
                "<{tag}> should inherit white-space:pre from UA stylesheet, got {:?}",
                style.white_space
            );
        }
    }

    /// R1659：`<input>` 是 void inline-block（无子节点），缺固有尺寸时 ZW 把 auto 宽度当全容器宽
    ///（fixture 37 实测 784×6）致 `<label>` 换行重叠。本测试钉死 UA 按类型注入的固有 width/height：
    /// 文本类按 `size` 属性（默认 20）估宽 + 15px 内容高；checkbox/radio/color 固定 13px 方框；
    /// submit/reset/button 按 `value` 字符数估宽。select/textarea 仍 width:auto（按内容测宽）。
    #[test]
    fn input_gets_intrinsic_sizing_from_ua_by_type() {
        use zero_css_parser::values::LengthValue;
        let doc = parse_html(
            "<body>\
             <input type=\"text\" value=\"Alice\">\
             <input type=\"password\">\
             <input size=\"10\">\
             <input type=\"checkbox\">\
             <input type=\"radio\">\
             <input type=\"submit\" value=\"Send\">\
             <input type=\"reset\" value=\"Clear\">\
             <select><option>x</option></select>\
             <textarea>t</textarea>\
             </body>",
        );
        let mut system = StyleSystem::new();
        let styles = system.compute_styles(&doc, &[]);
        let inputs = doc.get_elements_by_tag_name("input");
        // 文本类（type=text/password）：默认 size=20 → 20*7+8 = 148px，height 15px。
        for &i in &inputs[0..2] {
            let s = styles.get(&i).expect("text input styled");
            assert!(
                matches!(s.width, LengthValue::Px(w) if (140.0..=160.0).contains(&w)),
                "text input default-size width ~148px, got {:?}",
                s.width
            );
            assert!(
                matches!(s.height, LengthValue::Px(15.0)),
                "text input content height 15px, got {:?}",
                s.height
            );
        }
        // 显式 size=10（无 type → 文本类）→ 10*7+8 = 78px（窄于默认 20）。
        let sized = styles.get(&inputs[2]).expect("size=10 input styled");
        assert!(
            matches!(sized.width, LengthValue::Px(w) if (74.0..=84.0).contains(&w)),
            "size=10 input width ~78px, got {:?}",
            sized.width
        );
        // checkbox / radio：固定 13px 方框。
        for &i in &inputs[3..5] {
            let s = styles.get(&i).expect("check input styled");
            assert!(
                matches!(s.width, LengthValue::Px(13.0)),
                "checkbox/radio width 13px, got {:?}",
                s.width
            );
            assert!(
                matches!(s.height, LengthValue::Px(13.0)),
                "checkbox/radio height 13px, got {:?}",
                s.height
            );
        }
        // submit value="Send"（4 字符）→ 4*7+14 = 42px；reset value="Clear"（5）→ 49px。
        let submit = styles.get(&inputs[5]).expect("submit styled");
        assert!(
            matches!(submit.width, LengthValue::Px(w) if (38.0..=48.0).contains(&w)),
            "submit value=Send width ~42px, got {:?}",
            submit.width
        );
        let reset = styles.get(&inputs[6]).expect("reset styled");
        assert!(
            matches!(reset.width, LengthValue::Px(w) if (45.0..=55.0).contains(&w)),
            "reset value=Clear width ~49px, got {:?}",
            reset.width
        );
        // select / textarea：仍 width:auto（按内容测宽，UA 不注入固有 width）。
        for tag in ["select", "textarea"] {
            let id = doc.get_elements_by_tag_name(tag)[0];
            let s = styles.get(&id).unwrap_or_else(|| panic!("{tag} styled"));
            assert!(
                matches!(s.width, LengthValue::Auto),
                "<{tag}> must stay width:auto (content-sized), got {:?}",
                s.width
            );
        }
    }

    #[test]
    fn font_element_presentational_hints() {
        let doc = parse_html("<font color=\"#990000\" face=\"Arial, Times New Roman\" size=\"5\">txt</font>");
        let font = doc.get_elements_by_tag_name("font")[0];
        let hints = collect_presentational_hints(&doc, font);
        assert!(
            hints.iter().any(|(p, v)| p == "color" && v == "#990000"),
            "font color: {hints:?}"
        );
        assert!(
            hints
                .iter()
                .any(|(p, v)| p == "font-family" && v.contains("Arial") && v.contains("\"Times New Roman\"")),
            "font face (quoted multi-word): {hints:?}"
        );
        // SIZE 暂未启用（见 html_font_size_to_em 注释 + master.md R808）。
        assert!(!hints.iter().any(|(p, _)| p == "font-size"), "size disabled: {hints:?}");
    }

    #[test]
    fn center_element_text_align_hint() {
        let doc = parse_html("<center><p>x</p></center>");
        let center = doc.get_elements_by_tag_name("center")[0];
        let hints = collect_presentational_hints(&doc, center);
        assert!(
            hints.iter().any(|(p, v)| p == "text-align" && v == "center"),
            "center text-align hint: {hints:?}"
        );
    }

    #[test]
    fn font_size_mapping_matches_html5_scale() {
        // 七级绝对字号（HTML5 §10.4 非线性刻度，基准 3 = 1.0em）
        assert_eq!(html_font_size_to_em("1"), Some(0.63));
        assert_eq!(html_font_size_to_em("2"), Some(0.82));
        assert_eq!(html_font_size_to_em("3"), Some(1.0));
        assert_eq!(html_font_size_to_em("4"), Some(1.13));
        assert_eq!(html_font_size_to_em("5"), Some(1.5));
        assert_eq!(html_font_size_to_em("6"), Some(2.0));
        assert_eq!(html_font_size_to_em("7"), Some(3.0));
        // 相对值从基准 3 解析
        assert_eq!(html_font_size_to_em("+2"), Some(1.5)); // 3+2=5
        assert_eq!(html_font_size_to_em("-1"), Some(0.82)); // 3-1=2
        assert_eq!(html_font_size_to_em("9"), None); // 超范围
    }
}

#[cfg(test)]
mod tests;
