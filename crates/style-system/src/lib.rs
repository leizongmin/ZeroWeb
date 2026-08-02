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
use zero_css_parser::media_query::{MediaType, PrefersColorSchemeValue};
use zero_dom::{Document, NodeId, NodeKind, QuirksMode};

/// 返回 HTML 元素的 UA 默认 display 值。
///
/// 根据 HTML 规范，不同元素有不同的默认 display 类型。
/// 未列出的元素默认为 CSS 初始值 `inline`。
pub fn ua_default_display(tag: &str) -> Option<DisplayValue> {
    // R1679：option/optgroup 抑制（select 渲染切片，≡ R1675 datalist/source/track 谱系）。
    // ZW 无 select popup shadow tree，option/optgroup 默认 inline 会把所有 option 文本串联显示
    // 在 select 按钮内（chromium 只显示 selected option 标签，其余进 popup）。display:none 使
    // option 不生成盒，并让 has_direct_paintable_text 对 select 返回 false（paint_text 跳过），
    // 由 paint_select_value（controls.rs）绘 selected option 文本。select 失去内容测宽后由
    // select 固有宽（见 collect UA 宽）兜底，避免 R1659 width 回归。
    // kill-switch `ZW_SELECT_SUPPRESS_OPTIONS=0` 关闭（default-on）。
    if std::env::var("ZW_SELECT_SUPPRESS_OPTIONS").as_deref() != Ok("0")
        && (tag.eq_ignore_ascii_case("option") || tag.eq_ignore_ascii_case("optgroup"))
    {
        return Some(DisplayValue::None);
    }
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
        | "textarea" | "keygen" | "progress" | "meter" => DisplayValue::InlineBlock,

        // display:none
        // R1669：+ `area`（image map 区域，HTML 渲染规范 area{display:none}——仅定义可点击区不渲染盒）
        //   + `frame`（frameset 子元素，是 nested browsing context 非普通 CSS 盒；ZW 未实现 frameset
        //   帧模式网格渲染，display:none 避免把 frame 渲成 6×24.6 断盒，见 legacy-html fixture 46）。
        // R1675：+ `datalist`（自动补全建议容器，HTML 渲染规范 datalist{display:none}——其 option 仅作
        //   input 建议不渲染；ZW 误把 option 文本当 inline 渲染）+ `source`/`track`（media 子元素，
        //   分别提供 src / 文本轨道，自身无盒——ZW 误渲成 6×24.6 断盒致 video collapsed-container）。
        // R1676：+ `rp`（ruby 括号 fallback，ruby-capable UA 应 rp{display:none}——"("/")" 仅在不支持
        //   ruby 的 UA 显示；ZW 误把括号当 inline 渲染，见 legacy-html fixture 48）。
        "script" | "style" | "link" | "meta" | "head" | "title" | "base" | "basefont" | "bgsound" | "noframes"
        | "noembed" | "param" | "noscript" | "template" | "dialog" | "area" | "frame" | "datalist" | "source"
        | "track" | "rp" => DisplayValue::None,
        // R1688（ruby Slice 1 探针）：rt/rtc → display:none（kill rt 垂直堆叠盒 + rt 文本出父 IFC）。
        // ruby base 文本经 collect_text_excluding 收集进父 IFC（owner=ruby），R1022 overlay 据此绘
        // segment annotation。env `ZW_RUBY_RT_NONE=0` 关闭（A/B 探针，default-on 试）。
        "rt" | "rtc" if std::env::var("ZW_RUBY_RT_NONE").as_deref() != Ok("0") => DisplayValue::None,

        // 内联元素 — 无需覆盖（CSS 初始值即为 inline）
        _ => return None,
    })
}

/// R2246：HTML rendering §quotes（`<q>` 元素自动引号）— 英语默认引号对。
///
/// chromium/HTML 规范对未声明 `quotes` 的 `<q>`：depth 0 用双弯引号 “ ”（U+201C/U+201D），
/// depth 1+ 用单弯引号 ‘ ’（U+2018/U+2019）。非英语 locale 的 per-lang 默认表未实现
///（WPT css-content 多为英语，此默认覆盖主流案）。
fn q_default_quote_pair(depth: usize) -> (String, String) {
    if depth == 0 {
        ("\u{201C}".to_string(), "\u{201D}".to_string())
    } else {
        ("\u{2018}".to_string(), "\u{2019}".to_string())
    }
}

/// R2246：解析 `<q>` 元素在给定 `depth` 的开/闭引号字符串。
///
/// `depth` = 该 `<q>` 的 `<q>` 祖先数（CSS `open-quote` 深度；`<q>`-only 嵌套 ≡ 祖先数）。
/// `quotes` 属性优先：`Pairs` 取 `pairs[min(depth, len-1)]`（空 Pairs 回落默认）；
/// `Auto`/无声明 → [`q_default_quote_pair`]；`None` → 无引号（返回 `None`，`<q>` 不注入）。
/// driving: WPT css-content quotes-*（`<q>` 自动引号）。
pub fn resolve_q_quotes(quotes: &QuotesComputedValue, depth: usize) -> Option<(String, String)> {
    match quotes {
        QuotesComputedValue::None => None,
        QuotesComputedValue::Auto => Some(q_default_quote_pair(depth)),
        QuotesComputedValue::Pairs(pairs) => {
            if pairs.is_empty() {
                Some(q_default_quote_pair(depth))
            } else {
                let idx = depth.min(pairs.len() - 1);
                Some((pairs[idx].0.clone(), pairs[idx].1.clone()))
            }
        }
    }
}

/// R2246：统计节点的 `<q>` 祖先数（沿 DOM parent 链向上，含嵌套）。
///
/// 用于 `<q>` 自动引号的 depth（CSS open-quote 深度 = 当前打开的 `<q>` 祖先数）。
fn count_q_ancestors(doc: &Document, mut node: NodeId) -> usize {
    let mut count = 0usize;
    while let Some(parent) = doc.parent_node(node) {
        if let Some(pn) = doc.get(parent)
            && let NodeKind::Element(e) = &pn.kind
            && e.local_name().eq_ignore_ascii_case("q")
        {
            count += 1;
        }
        node = parent;
    }
    count
}

/// R1679：返回 `<option>` 的标签文本（HTML §4.10.10）。
///
/// `label` 属性非空时优先（trim 后），否则回落到 option 的 text content。
fn option_label(doc: &Document, id: NodeId) -> String {
    if let Some(l) = doc.get_attribute(id, "label") {
        let trimmed = l.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    doc.text_content(id).unwrap_or_default().trim().to_string()
}

/// R1679：计算 `<select>` 的固有宽度（px）= 最宽 option 标签宽 + 下拉 chrome。
///
/// 遍历 select 直接 option 子 + optgroup 内 option（HTML 允许两种嵌套）。字符宽近似
/// `CHAR_PX`（≡ R1659 input sizing，默认字体平均字符宽）。chrome = 下拉箭头 + padding/border
///（chromium ≈ 20px 箭头 + 4px ≈ 24px）。无 option 时回落 chrome 最小宽。
fn select_intrinsic_width(doc: &Document, select: NodeId) -> f32 {
    const CHAR_PX: f32 = 7.0;
    const CHROME: f32 = 24.0;
    let mut max_chars: usize = 0;
    for child in doc.child_nodes(select) {
        let Some(node) = doc.get(child) else { continue };
        let NodeKind::Element(e) = &node.kind else { continue };
        let name = e.local_name();
        if name.eq_ignore_ascii_case("option") {
            max_chars = max_chars.max(option_label(doc, child).chars().count());
        } else if name.eq_ignore_ascii_case("optgroup") {
            for gc in doc.child_nodes(child) {
                if let Some(gn) = doc.get(gc)
                    && let NodeKind::Element(ge) = &gn.kind
                    && ge.local_name().eq_ignore_ascii_case("option")
                {
                    max_chars = max_chars.max(option_label(doc, gc).chars().count());
                }
            }
        }
    }
    (max_chars as f32 * CHAR_PX + CHROME).max(CHROME)
}

/// 样式系统，负责为文档中的元素计算样式。
///
/// 整合选择器匹配、级联、继承和计算值生成。
pub struct StyleSystem {
    /// 自定义属性存储（--variable）。
    custom_properties: HashMap<String, String>,
    /// `@property` 注册的自定义属性（名称 → 注册信息）。由 `compute_styles` 预扫描
    /// 样式表填充，在 `gather_custom_properties` 中为未显式声明的注册属性提供
    /// `initial-value` 兜底默认值（并按 `inherits` 控制继承）。
    registered_properties: HashMap<String, RegisteredProperty>,
    /// 视口宽度（px），用于 vh/vw 计算。
    viewport_width: Option<f64>,
    /// 视口高度（px），用于 vh/vw 计算。
    viewport_height: Option<f64>,
    /// 用户颜色方案偏好（对应 `prefers-color-scheme` 媒体查询）。
    prefers_color_scheme: PrefersColorSchemeValue,
    /// 渲染媒体类型（对应 `@media screen/print/all`）。默认 Screen；
    /// 设为 Print 时 `@media print` 规则生效、`@media screen` 规则失效（CSS §7）。
    media_type: MediaType,
}

impl StyleSystem {
    /// 创建新的样式系统实例。
    pub fn new() -> Self {
        Self {
            custom_properties: HashMap::new(),
            registered_properties: HashMap::new(),
            viewport_width: None,
            viewport_height: None,
            prefers_color_scheme: PrefersColorSchemeValue::Light,
            media_type: MediaType::Screen,
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

    /// 设置渲染媒体类型（`MediaType::Print` 用于 `@media print` 渲染，如打印预览）。
    pub fn set_media_type(&mut self, media_type: MediaType) {
        self.media_type = media_type;
    }

    /// 读取渲染媒体类型（R1999：供 layout 层判断是否触发 Print 分页 post-process）。
    pub fn media_type(&self) -> MediaType {
        self.media_type
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

        // 预扫描所有样式表的 `@property` 规则，注册自定义属性（syntax/inherits/initial-value）。
        // 注册信息在 `gather_custom_properties` 中为未显式声明的注册属性提供 initial-value
        // 兜底默认值（CSS Properties and Values API）。
        self.registered_properties = collect_registered_properties(stylesheets);

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
            // R2246：HTML rendering §quotes — `<q>` 自动引号。depth = `<q>` 祖先数（CSS
            // open-quote 深度）；解析引号对（`quotes` 属性优先，Auto/无声明→英语默认，None→无）。
            // 仅在**无显式 ::before/::after content**（content:Normal）时注入开/闭引号，经既有
            // before_pseudo/after_pseudo + pipeline 文本节点注入渲染。driving: WPT css-content quotes-*。
            let is_q = matches!(&node_data.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("q"));
            let q_quotes = if is_q {
                resolve_q_quotes(&computed.quotes, count_q_ancestors(doc, node))
            } else {
                None
            };
            if matches!(
                before.content,
                property::types::ContentComputedValue::String(_)
                    | property::types::ContentComputedValue::Attr(_)
                    | property::types::ContentComputedValue::List(_)
            ) {
                computed.before_pseudo = Some(Box::new(before));
            } else if is_q
                && matches!(before.content, property::types::ContentComputedValue::Normal)
                && let Some((open, _)) = &q_quotes
            {
                let mut b = before.clone();
                b.content = property::types::ContentComputedValue::String(open.clone());
                computed.before_pseudo = Some(Box::new(b));
            }
            if matches!(
                after.content,
                property::types::ContentComputedValue::String(_)
                    | property::types::ContentComputedValue::Attr(_)
                    | property::types::ContentComputedValue::List(_)
            ) {
                computed.after_pseudo = Some(Box::new(after));
            } else if is_q
                && matches!(after.content, property::types::ContentComputedValue::Normal)
                && let Some((_, close)) = &q_quotes
            {
                let mut a = after.clone();
                a.content = property::types::ContentComputedValue::String(close.clone());
                computed.after_pseudo = Some(Box::new(a));
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
                ctx.media_type = self.media_type;
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
                // R1690：HTML 渲染规范 UA 块级元素的默认 margin（chromium UA 样式表）。
                // ZW 此前仅给 display:block 无 margin → blockquote/dd/figure 内容无缩进，
                // 与 chromium 发散。blockquote/figure margin 1em 40px（上下 1em + 左右 40px），
                // dl margin 1em 0，dd margin-left 40px（margin-inline-start，垂直 0）。
                "blockquote" | "figure" => {
                    ua_decl_inputs.push(("margin".to_string(), "1em 40px".to_string(), false, (0, 0, 0), None));
                }
                "dl" => {
                    ua_decl_inputs.push(("margin".to_string(), "1em 0".to_string(), false, (0, 0, 0), None));
                }
                "dd" => {
                    ua_decl_inputs.push(("margin".to_string(), "0 0 0 40px".to_string(), false, (0, 0, 0), None));
                }
                // pre/xmp/listing/plaintext：HTML 渲染规范 UA 样式表 white-space:pre（保留空白/换行）。
                // R1658：ZW default_impl white_space 默认 Normal，故 <pre> 此前折叠空白/换行（真 bug）。
                // 仅 white-space:pre（monospace 字体属 font-wall 高方差，单独 A/B 切片）。
                "pre" | "xmp" | "listing" | "plaintext" => {
                    ua_decl_inputs.push(("white-space".to_string(), "pre".to_string(), false, (0, 0, 0), None));
                }
                // ul/ol 默认 padding-left、margin 和 list-style-type（HTML 渲染规范 UA 样式表）。
                // R1699：list-style-type 继承、CSS initial=Disc，故 <ul> 隐式得 Disc（正确），
                // 但 <ol> 也继承 Disc → 渲染 disc 圆点而非 decimal 数字（BUG：ol 显示圆点而非序号）。
                // 显式标 ul=disc / ol=decimal 同时修正 ol 默认 + ul 嵌套在 ol 内不误继承 decimal
                //（chromium UA `ul,menu,dir{list-style-type:disc}` / `ol{list-style-type:decimal}`）。
                "ul" | "ol" => {
                    ua_decl_inputs.push(("margin".to_string(), "1em 0".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("padding-left".to_string(), "40px".to_string(), false, (0, 0, 0), None));
                    let lst = if tag == "ul" { "disc" } else { "decimal" };
                    ua_decl_inputs.push(("list-style-type".to_string(), lst.to_string(), false, (0, 0, 0), None));
                }
                "b" | "strong" => {
                    ua_decl_inputs.push(("font-weight".to_string(), "bold".to_string(), false, (0, 0, 0), None));
                }
                "th" => {
                    ua_decl_inputs.push(("font-weight".to_string(), "bold".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("text-align".to_string(), "center".to_string(), false, (0, 0, 0), None));
                }
                // R1698：HTML 渲染规范 UA `caption { text-align: center }`（chromium UA）。
                // caption 默认水平居中（caption-side 上下定位由 R1653 top_caption_extent 独立处理）。
                // ZW 此前无 → caption 文本左对齐，与 chromium 发散。specificity 0,0,0 可被作者覆盖。
                "caption" => {
                    ua_decl_inputs.push(("text-align".to_string(), "center".to_string(), false, (0, 0, 0), None));
                }
                "i" | "em" => {
                    ua_decl_inputs.push(("font-style".to_string(), "italic".to_string(), false, (0, 0, 0), None));
                }
                // R1691：HTML 渲染规范 UA font-style:italic 短语元素（≡ i/em，chromium UA）。
                // address（block+italic，block 已在 display 列表）+ cite/var/dfn（inline italic）。
                "address" | "cite" | "var" | "dfn" => {
                    ua_decl_inputs.push(("font-style".to_string(), "italic".to_string(), false, (0, 0, 0), None));
                }
                // R1691/R1697：HTML 渲染规范 UA text-decoration（chromium UA）。
                // u|ins → underline；s/del/strike → line-through（deprecated strike ≡ s）。
                // ins 是 del 的对称对（编辑标记：插入/删除），chromium UA 把 ins 与 u 同组 underline。
                "u" | "ins" => {
                    ua_decl_inputs.push((
                        "text-decoration".to_string(),
                        "underline".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                }
                "s" | "del" | "strike" => {
                    ua_decl_inputs.push((
                        "text-decoration".to_string(),
                        "line-through".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                }
                // R1692：HTML 渲染规范 UA font-size/vertical-align（chromium UA `smaller` ≈0.83em，
                // ZW 无 smaller 关键字 → 用 0.83em 显式）。ZW vertical-align:sub/super 已支持
                //（parse + layout 基线偏移 inline/mod.rs:1189）。specificity 0,0,0 可被作者覆盖。
                "small" => {
                    ua_decl_inputs.push(("font-size".to_string(), "0.83em".to_string(), false, (0, 0, 0), None));
                }
                // R1714：HTML 渲染规范 UA `big { font-size: larger }`（≡ small 的对称对，chromium UA
                // `larger` ≈1.2em；ZW 无 larger 关键字 → 用 1.2em 显式，与 small 0.83em 对称）。
                // `<big>` 已废弃（HTML5）但 legacy 页仍用；ZW 此前无 → 渲成普通 inline（无放大）。
                // specificity 0,0,0 可被作者覆盖。corpus 0（WPT 罕用），legacy fixture 34 可见。
                "big" => {
                    ua_decl_inputs.push(("font-size".to_string(), "1.2em".to_string(), false, (0, 0, 0), None));
                }
                "sub" => {
                    ua_decl_inputs.push(("font-size".to_string(), "0.83em".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("vertical-align".to_string(), "sub".to_string(), false, (0, 0, 0), None));
                }
                "sup" => {
                    ua_decl_inputs.push(("font-size".to_string(), "0.83em".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push((
                        "vertical-align".to_string(),
                        "super".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                }
                // R1693：HTML 渲染规范 UA font-family:monospace（chromium UA
                // `code,kbd,samp,tt { font-family: monospace }`）。ZW 解析 monospace →
                // DejaVu Sans Mono（font/loader.rs:258）。pre 已 R1658 white-space:pre；
                // pre 的 monospace 字体同此（pre-family monospace A/B 见 R1658 forward，本 slice
                // 只 code/kbd/samp/tt，pre 另切片避 font-wall 耦合）。specificity 0,0,0 可被作者覆盖。
                "code" | "kbd" | "samp" | "tt" => {
                    ua_decl_inputs.push((
                        "font-family".to_string(),
                        "monospace".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                }
                // mark：HTML 渲染规范 UA `mark { background-color: yellow; color: black }`
                //（高亮文本）。ZW 默认无 → <mark> 渲成普通 inline（无高亮）。R1685：补 UA 默认值
                //（≡ pre white-space / h1 bold 谱系，specificity 0,0,0 可被作者样式覆盖）。
                "mark" => {
                    ua_decl_inputs.push((
                        "background-color".to_string(),
                        "#ffff00".to_string(),
                        false,
                        (0, 0, 0),
                        None,
                    ));
                    ua_decl_inputs.push(("color".to_string(), "black".to_string(), false, (0, 0, 0), None));
                }
                // summary：为 disclosure 标记（▶/▼，R1686 paint_summary_marker）让出左侧空间。
                // chromium 标记占首行行首 ~0.4em + gap，text 让位 ≈1.2em。padding-left 给标记绘区，
                // 否则标记压字（无 padding 时 paint_summary_marker 跳过）。specificity 0,0,0 可被作者覆盖。
                "summary" => {
                    ua_decl_inputs.push(("padding-left".to_string(), "1.2em".to_string(), false, (0, 0, 0), None));
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
                    } else if tag == "select" && std::env::var("ZW_SELECT_SUPPRESS_OPTIONS").as_deref() != Ok("0") {
                        // R1679：select 固有宽 = 最宽 option 标签宽 + 下拉 chrome（form-control
                        // intrinsic sizing，≡ R1659 input）。option/optgroup 经 ua_default_display
                        // 抑制为 display:none 后，select 失去 inline 内容测宽（R1659 width 回归——
                        // inline-block width:auto 会塌缩/拉满），此处补 UA width 兜底（最低优先级
                        // specificity(0,0,0)，可被作者/内联样式覆盖）。
                        let w = select_intrinsic_width(doc, element);
                        ua_decl_inputs.push(("width".to_string(), format!("{w:.0}px"), false, (0, 0, 0), None));
                    } else if tag == "textarea" && std::env::var("ZW_TEXTAREA_INTRINSIC_SIZE").as_deref() != Ok("0") {
                        // R1681：textarea 固有尺寸 = cols×char宽 + rows×行高（form-control intrinsic
                        // sizing，≡ R1659 input / R1679 select 谱系）。textarea 现 width:auto 按 text
                        // content 测宽（短内容塌缩/长内容拉满），chromium 按 cols（默认 20）/rows
                        //（默认 2）属性定尺寸。最低优先级 specificity(0,0,0)，可被作者/内联样式覆盖。
                        const CHAR_PX: f32 = 7.0; // ≡ R1659 input/select 平均字符宽
                        const ROW_PX: f32 = 19.0; // ≈ 默认 16px font × 1.2 line-height（行高）
                        const WIDTH_CHROME: f32 = 8.0; // ≡ R1659 input：左右 padding(4)+border(2)+边距
                        let cols = doc
                            .get_attribute(element, "cols")
                            .and_then(|s| s.trim().parse::<f32>().ok())
                            .filter(|&n| n >= 1.0)
                            .unwrap_or(20.0);
                        let rows = doc
                            .get_attribute(element, "rows")
                            .and_then(|s| s.trim().parse::<f32>().ok())
                            .filter(|&n| n >= 1.0)
                            .unwrap_or(2.0);
                        let w = cols * CHAR_PX + WIDTH_CHROME;
                        let h = rows * ROW_PX;
                        ua_decl_inputs.push(("width".to_string(), format!("{w:.0}px"), false, (0, 0, 0), None));
                        ua_decl_inputs.push(("height".to_string(), format!("{h:.0}px"), false, (0, 0, 0), None));
                    }
                }
                // R1669：`<keygen>` deprecated void 表单控件（≡ R1659 `<input>` / R1396 form-control 谱系）。
                // keygen 是 void inline-block（无子节点），无固有尺寸时 ZW 把 auto 宽当 6px sliver 致
                // 包裹它的 `<p>` 塌缩（legacy-html fixture 45 struct FAIL: collapsed container h=0 < h=25）。
                // Chromium 把 keygen 建模为 menulist 替换控件（已废弃，Chrome 57+/Firefox 69+ 移除）。
                // 此处补 bg/border/padding（控件可见，与 input/select 同 R1396 外观）+ UA width/height
                //（menulist 近似 90×24，最低优先级 specificity(0,0,0)，可被作者/内联样式覆盖）。
                "keygen" => {
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
                    ua_decl_inputs.push(("width".to_string(), "90px".to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("height".to_string(), "24px".to_string(), false, (0, 0, 0), None));
                }
                // R1670：`<progress>`/`<meter>` inline-block 替换控件固有尺寸（sizing 半，≡ R1659 input
                // 谱系；value-bar/gauge 绘制 = paint 半，forward）。progress/meter 是 replaced inline-block，
                // 无固有尺寸时 ZW 按默认 inline 渲成 fallback 文本宽的薄盒（fixture 45：progress 61.6×18 /
                // meter 22.4×18，应 chromium progress 10em×1em=160×16 / meter 5em=80×16，chrome-127 oracle
                // 实测 progress x[8,167]=160 value-fill 60%=96px；meter x[8,87]=80 value-fill 30%=24px）。
                // 此处补 track 外观（border + 灰 bg，近似 chromium track）+ UA width/height（最低优先级
                // specificity(0,0,0)，可被作者样式覆盖）。
                //
                // ★ fallback-content forward（本轮不修，架构缺口）：chromium 把 progress/meter 当 replaced
                // 元素，**渲染时不显示 fallback 子节点**（"60% done" 等仅在不支持时显示）。ZW 无「replaced
                // 元素抑制子节点 layout」机制——`<select>` 的 `<option>` 同样当 inline 文本渲染（latent gap，
                // 非 progress/meter 独有）；`is_replaced` 仅影响 sizing 不抑制子节点。真修须 tree.rs 加
                // replaced-element 子节点抑制（跨 select/object/embed/applet/progress/meter），多 session。
                // 故本轮 fallback 文本仍会渲染在 track 盒内（与 select 现状一致，非新引入 bug 类）。
                "progress" | "meter" => {
                    ua_decl_inputs.push((
                        "background-color".to_string(),
                        "#efefef".to_string(),
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
                    let (w, h) = if tag == "progress" {
                        ("160px", "16px")
                    } else {
                        ("80px", "16px")
                    };
                    ua_decl_inputs.push(("width".to_string(), w.to_string(), false, (0, 0, 0), None));
                    ua_decl_inputs.push(("height".to_string(), h.to_string(), false, (0, 0, 0), None));
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

        // 3. 运行级联算法（apply-on-dummy 合法性探测：非法值声明按未声明处理，
        //    较低优先级合法声明可胜出；driving：keywords-000 `background:"red"`）
        let cascaded = cascade::cascade(declarations, quirks_mode == QuirksMode::Quirks);

        // 4. 收集自定义属性（继承父元素 + 当前元素自身声明覆盖）
        // CSS 自定义属性是继承属性：`:root { --x }` 定义的变量需对后代可见。
        self.custom_properties = gather_custom_properties(&cascaded, parent_custom, &self.registered_properties);

        // 4.5. 在级联值中解析 var() 引用
        let resolved_cascaded = resolve_var_in_cascaded(&cascaded, &self.custom_properties);

        // 5. 计算继承样式（prefers-color-scheme 参与 color-scheme used-scheme 合成）
        let prefers_dark = matches!(self.prefers_color_scheme, PrefersColorSchemeValue::Dark);
        let style = inheritance::compute_inherited_style_with_quirks(
            parent_style,
            &resolved_cascaded,
            quirks_mode,
            prefers_dark,
        );

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

/// 在级联属性值中解析 env() / var() 引用。
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
                (k.clone(), computed::resolve_env_and_var(v, custom_properties))
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
            // R1710：HTML4 §13.7.3 `<img border=N>` → border:Npx solid（currentColor；
            // <a> 内经继承自动取链接色）。隔离实测 border 单独 net −381px（fixture 24
            // 0.79%→0.71%，border ring 匹配 chromium）；hspace/vspace 因 font-wall 文本宽
            // + inline 垂直 margin 发散（+168/+541px），defer。
            let bw = table_border_width_attr(elem);
            if bw > 0 {
                hints.push(("border".to_string(), format!("{bw}px solid")));
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
            // R1716：HTML4 §11.2.4 `<table align>` 表现提示（chromium UA）。
            // center → margin-left/right:auto（display:table 块居中，非浮动）；
            // left/right → float:left/right（HTML4 表格浮动，后续块内容环绕）。
            if let Some(align) = elem_attr(elem, "align") {
                match align.trim().to_ascii_lowercase().as_str() {
                    "center" => {
                        hints.push(("margin-left".to_string(), "auto".to_string()));
                        hints.push(("margin-right".to_string(), "auto".to_string()));
                    }
                    "left" => hints.push(("float".to_string(), "left".to_string())),
                    "right" => hints.push(("float".to_string(), "right".to_string())),
                    _ => {}
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
            // R1715：HTML4 `valign` 属性 → vertical-align（CSS2 App D）。td/th 自身
            // valign 优先；否则继承父 `<tr valign>`（chromium 行级 valign 传播到单元格）。
            let valign = elem_attr(elem, "valign").or_else(|| parent_tr_valign(doc, element));
            if let Some(va) = valign.and_then(|v| html_valign_to_vertical_align(&v)) {
                hints.push(("vertical-align".to_string(), va));
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
        // R1700：ol/ul/li 的 HTML4 `type` 属性 → list-style-type（CSS2 App D 表现提示）。
        // ol type: 1/a/A/i/I；ul/li type: disc/circle/square（li 也可取 ol 的 1/a/A/i/I）。
        // 仅 type 属性；start=/value= 计数器语义是独立切片（须 counter-reset 支持）。
        "ol" | "ul" | "li" => {
            if let Some(t) = elem_attr(elem, "type").and_then(|v| html_list_type_attr(&v)) {
                hints.push(("list-style-type".to_string(), t.to_string()));
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

/// R1715：取父 `<tr valign>` 属性（行级 valign 传播到单元格，≡ parent_tr_bgcolor 模式）。
fn parent_tr_valign(doc: &Document, element: NodeId) -> Option<String> {
    let parent = doc.parent_node(element)?;
    let n = doc.get(parent)?;
    let NodeKind::Element(tr) = &n.kind else {
        return None;
    };
    if !tr.local_name().eq_ignore_ascii_case("tr") {
        return None;
    }
    elem_attr(tr, "valign")
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

/// R1715：HTML4 `valign` 属性值 → CSS vertical-align（CSS2 App D 表现提示）。
/// chromium UA `<td/th/tr valign>` 传播到单元格：top/middle/bottom/baseline +
/// 同义词 center(=middle)/texttop(=text-top)/textbottom(=text-bottom)。
/// 非法值返回 None（忽略，回落到 td/th 默认）。注意与 `html_align_to_vertical_align`
/// 区别：后者映射 `<img align>`（align 取 top/middle/bottom 表垂直），本函数映射
/// `valign` 专用属性（语义独立，多 baseline/text-top/text-bottom 值）。
fn html_valign_to_vertical_align(valign: &str) -> Option<String> {
    match valign.trim().to_ascii_lowercase().as_str() {
        "top" => Some("top".to_string()),
        "middle" | "center" => Some("middle".to_string()),
        "bottom" => Some("bottom".to_string()),
        "baseline" => Some("baseline".to_string()),
        "texttop" => Some("text-top".to_string()),
        "textbottom" => Some("text-bottom".to_string()),
        _ => None,
    }
}

/// R1700：HTML4 `<ol/ul/li type>` 属性值 → CSS list-style-type 关键字（CSS2 App D）。
/// ol type: 1/a/A/i/I → decimal/lower-alpha/upper-alpha/lower-roman/upper-roman；
/// ul/li type: disc/circle/square（li 也可取 ol 的序数类型）。非法值返回 None（忽略）。
/// 注意：type 大小写敏感（`a` vs `A` 是不同 list-style-type），故不 to_lowercase。
fn html_list_type_attr(attr: &str) -> Option<&'static str> {
    match attr.trim() {
        "1" => Some("decimal"),
        "a" => Some("lower-alpha"),
        "A" => Some("upper-alpha"),
        "i" => Some("lower-roman"),
        "I" => Some("upper-roman"),
        "disc" => Some("disc"),
        "circle" => Some("circle"),
        "square" => Some("square"),
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

/// 从级联值中收集自定义属性（继承父元素 + 当前元素自身声明覆盖）。
///
/// 自定义属性是继承属性：先继承父元素（`inherited`），再用当前元素自身级联声明覆盖
///（自身优先）。值中的 var() 引用**不在收集期预解析**——[`computed::resolve_var`] 在
/// 使用点（常规属性解析时）做完全传递解析 + 环检测，使用元素自身的 custom_properties
/// 上下文（更正确：子元素可覆盖被引用的变量）。旧实现的收集期迭代预解析在 var() 环引用
/// 下指数膨胀 → 6GB OOM（driving: WPT variable-declaration-48/49），已移除。
fn gather_custom_properties(
    cascaded: &HashMap<String, String>,
    inherited: &HashMap<String, String>,
    registered: &HashMap<String, RegisteredProperty>,
) -> HashMap<String, String> {
    let mut props: HashMap<String, String> = HashMap::new();
    // 1. 继承父元素的自定义属性。`inherits: false` 的注册属性不继承——在子元素重置为
    //    初值（由步骤 3 兜底），符合 CSS Properties and Values API 语义。
    for (k, v) in inherited {
        if let Some(reg) = registered.get(k)
            && !reg.inherits
        {
            continue;
        }
        props.insert(k.clone(), v.clone());
    }
    // 2. 当前元素自身的级联声明覆盖（自身优先）。
    for (k, v) in cascaded.iter().filter(|(k, _)| k.starts_with("--")) {
        props.insert(k.clone(), v.clone());
    }
    // 3. 注册属性的 initial-value 作为兜底默认（未显式声明/继承时）。这使得
    //    `@property --x { initial-value: green; }` 后，未声明 `--x` 处的 `var(--x)`
    //    解析为 green（而非 invalid at computed-value-time）。
    for (name, reg) in registered {
        if let Some(iv) = &reg.initial_value {
            props.entry(name.clone()).or_insert_with(|| iv.clone());
        }
    }
    props
}

/// `@property` 注册的自定义属性信息（仅消费 var() 解析所需的最小语义）。
#[derive(Debug, Clone)]
struct RegisteredProperty {
    /// 是否继承。`true` 时像普通自定义属性一样继承；`false` 时每个元素从初值起算。
    inherits: bool,
    /// `initial-value` 描述符原始值；`None` = 缺省（仅 `syntax: "*"` 时合法，此时无兜底）。
    initial_value: Option<String>,
}

/// 扫描样式表中的 `@property` 规则，构建「自定义属性名 → 注册信息」映射。
///
/// 后注册的同名属性覆盖先注册者（CSS Properties and Values API：后者胜出）。仅顶层
/// `@property` 规则有效（不递归进入 @media 等条件块——@property 是全局注册，无作用域）。
fn collect_registered_properties(stylesheets: &[Stylesheet]) -> HashMap<String, RegisteredProperty> {
    use zero_css_parser::ast::Rule;
    let mut map: HashMap<String, RegisteredProperty> = HashMap::new();
    for ss in stylesheets {
        for rule in &ss.rules {
            if let Rule::Property(pr) = rule {
                map.insert(
                    pr.name.clone(),
                    RegisteredProperty {
                        inherits: pr.inherits,
                        initial_value: pr.initial_value.clone(),
                    },
                );
            }
        }
    }
    map
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

    // 1. （已移除，R2016）原「百分比高度 quirks」规则**反向**：它在 quirks mode 把
    //    height:<percentage>（auto 父）compute-to-auto，但那是 **standards** 行为（CSS §10.5），
    //    layout 的 `apply_indefinite_percent_height_to_auto` 已为 standards 实现。quirks mode 的
    //    正确行为是百分比按 ICB（viewport）解析（「百分比高度生效」legacy 行为），由 layout 的
    //    quirks 分支（R2016）处理。故本 style-system 规则删除——保留百分比，交 layout 按模式裁决。
    let _ = parent_style; // 保留签名稳定（其他 quirks 规则未来可能用 parent_style）。

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
    /// R2016：quirks mode 下百分比高度**保留**（不再 compute-to-auto）——layout 的
    /// `apply_indefinite_percent_height_to_auto` quirks 分支按 ICB（viewport）解析。
    /// （原规则反向：把 standards 的 compute-to-auto 误安到 quirks gate 上，已移除。）
    #[test]
    fn test_quirks_mode_percentage_height_fallback() {
        let mut child_style = ComputedStyle::default();
        child_style.height = LengthValue::Percentage(50.0);

        let parent_style = ComputedStyle::default();
        // parent height is Auto by default

        apply_quirks_mode_adjustments(&mut child_style, Some(&parent_style), None);

        assert_eq!(
            child_style.height,
            LengthValue::Percentage(50.0),
            "Quirks mode must KEEP percentage height (resolved against viewport in layout), not convert to auto"
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
            "script", "style", "link", "meta", "head", "title", "base", "basefont", "bgsound", "noframes", "noembed",
            "param", "noscript", "template", "dialog",
            // R1669：area（image map 区域，HTML 渲染规范 area{display:none}）+ frame（frameset 子，
            // nested browsing context 非普通 CSS 盒）。legacy-html fixture 44/46 LAYOUT_DUMP 抓到
            // 两者误渲染（area 6×24.6 盒、frame 6×24.6 断盒 @负 y）。
            "area", "frame",
            // R1675：datalist（自动补全建议容器）+ source/track（media 子元素，无盒）。legacy-html
            // fixture 47 LAYOUT_DUMP + pixel 采样抓到误渲染（datalist option 文本当 inline 渲染；
            // source/track 渲成 6×24.6 断盒致 video collapsed-container + sibling overlap）。
            "datalist", "source", "track",
            // R1676：rp（ruby 括号 fallback，ruby-capable UA display:none）。legacy-html fixture 48
            // pixel 采样抓到 ZW 误渲 "(" ")"（chrome 隐藏）。
            "rp",
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
mod presentational_hint_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod q_quotes_tests {
    use super::*;

    /// R2246：resolve_q_quotes 引号对解析——None 无引号；Auto 英语默认（depth 0 “ ”，
    /// depth 1+ ‘ ’）；Pairs 取对应 depth（clamp 到末对）。
    #[test]
    fn test_resolve_q_quotes() {
        use property::types::QuotesComputedValue;
        // None → 无引号。
        assert!(resolve_q_quotes(&QuotesComputedValue::None, 0).is_none(), "None 无引号");
        // Auto depth 0 → “ ”，depth 1 → ‘ ’。
        let (o, c) = resolve_q_quotes(&QuotesComputedValue::Auto, 0).unwrap();
        assert_eq!(o, "\u{201C}", "Auto depth 0 开引号 “");
        assert_eq!(c, "\u{201D}", "Auto depth 0 闭引号 ”");
        let (o, c) = resolve_q_quotes(&QuotesComputedValue::Auto, 1).unwrap();
        assert_eq!(o, "\u{2018}", "Auto depth 1 开引号 ‘");
        assert_eq!(c, "\u{2019}", "Auto depth 1 闭引号 ’");
        // Pairs → 取对应 depth，clamp 到末对。
        let pairs = QuotesComputedValue::Pairs(vec![
            ("«".to_string(), "»".to_string()),
            ("‹".to_string(), "›".to_string()),
        ]);
        let (o, c) = resolve_q_quotes(&pairs, 0).unwrap();
        assert_eq!((o.as_str(), c.as_str()), ("«", "»"), "Pairs depth 0");
        let (o, c) = resolve_q_quotes(&pairs, 1).unwrap();
        assert_eq!((o.as_str(), c.as_str()), ("‹", "›"), "Pairs depth 1");
        // depth 超过 Pairs 长度 → clamp 到末对。
        let (o, c) = resolve_q_quotes(&pairs, 5).unwrap();
        assert_eq!((o.as_str(), c.as_str()), ("‹", "›"), "Pairs depth 5 clamp");
        // 空 Pairs → 回落英语默认。
        let (o, _) = resolve_q_quotes(&QuotesComputedValue::Pairs(vec![]), 0).unwrap();
        assert_eq!(o, "\u{201C}", "空 Pairs 回落默认 depth 0");
    }

    /// 辅助：在 doc 中按标签名查找首个元素 NodeId。
    fn find_first_tag(doc: &Document, root: NodeId, tag: &str) -> Option<NodeId> {
        if let Some(n) = doc.get(root)
            && let NodeKind::Element(e) = &n.kind
            && e.local_name().eq_ignore_ascii_case(tag)
        {
            return Some(root);
        }
        for child in doc.child_nodes(root) {
            if let Some(found) = find_first_tag(doc, child, tag) {
                return Some(found);
            }
        }
        None
    }

    /// R2246：`<q>text</q>` 经 compute_styles 注入 before_pseudo=“ / after_pseudo=”
    ///（英语默认，经既有 before_pseudo + pipeline 文本节点渲染）。
    #[test]
    fn test_q_element_auto_quotes_default() {
        let doc = zero_dom::parse_html("<html><body><q>hello</q></body></html>");
        let body = doc.root();
        let q = find_first_tag(&doc, body, "q").expect("应找到 <q>");
        let mut sys = StyleSystem::new();
        let styles = sys.compute_styles(&doc, &[]);
        let q_style = styles.get(&q).expect("<q> 应有计算样式");
        let before = q_style.before_pseudo.as_ref().expect("<q> 应注入 before_pseudo");
        assert!(
            matches!(&before.content, property::types::ContentComputedValue::String(s) if s == "\u{201C}"),
            "<q> before_pseudo content 须为 “，got {:?}",
            before.content
        );
        let after = q_style.after_pseudo.as_ref().expect("<q> 应注入 after_pseudo");
        assert!(
            matches!(&after.content, property::types::ContentComputedValue::String(s) if s == "\u{201D}"),
            "<q> after_pseudo content 须为 ”，got {:?}",
            after.content
        );
    }

    /// R2246：`quotes` 属性覆盖默认对（作者声明 `quotes: « »`）。
    #[test]
    fn test_q_element_quotes_property_override() {
        let doc =
            zero_dom::parse_html("<html><head><style>q{quotes:\"«\" \"»\"}</style></head><body><q>x</q></body></html>");
        let body = doc.root();
        let q = find_first_tag(&doc, body, "q").expect("应找到 <q>");
        // 解析样式表
        let css = "q{quotes:\"«\" \"»\"}";
        let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = StyleSystem::new();
        let styles = sys.compute_styles(&doc, &[stylesheet]);
        let q_style = styles.get(&q).expect("<q> 应有计算样式");
        let before = q_style.before_pseudo.as_ref().expect("<q> 应注入 before_pseudo");
        assert!(
            matches!(&before.content, property::types::ContentComputedValue::String(s) if s == "«"),
            "quotes:« » 覆盖 → before 须为 «，got {:?}",
            before.content
        );
    }

    /// R2246：嵌套 `<q><q>` — 内层 depth=1 用单引号对（英语默认 depth 1 = ‘ ’）。
    #[test]
    fn test_q_element_nested_depth() {
        let doc = zero_dom::parse_html("<html><body><q>outer <q>inner</q></q></body></html>");
        let body = doc.root();
        // 找到内层 <q>（DFS 最后一个 q = inner）。
        fn find_last_tag(doc: &Document, root: NodeId, tag: &str, found: &mut Option<NodeId>) {
            if let Some(n) = doc.get(root)
                && let NodeKind::Element(e) = &n.kind
                && e.local_name().eq_ignore_ascii_case(tag)
            {
                *found = Some(root);
            }
            for child in doc.child_nodes(root) {
                find_last_tag(doc, child, tag, found);
            }
        }
        let mut inner = None;
        find_last_tag(&doc, body, "q", &mut inner);
        let inner = inner.expect("应找到内层 <q>");
        let mut sys = StyleSystem::new();
        let styles = sys.compute_styles(&doc, &[]);
        let q_style = styles.get(&inner).expect("内层 <q> 应有计算样式");
        let before = q_style.before_pseudo.as_ref().expect("内层 <q> 应注入 before_pseudo");
        assert!(
            matches!(&before.content, property::types::ContentComputedValue::String(s) if s == "\u{2018}"),
            "内层 <q> depth=1 → before 须为 ‘，got {:?}",
            before.content
        );
    }
}
