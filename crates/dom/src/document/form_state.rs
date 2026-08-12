//! Document 表单状态伪类判定 —— 拆自 `mod.rs`（rule 5 单文件 <2000 行，R3280）。
//!
//! 本模块为 [`super::Document`] 的表单状态面（`:disabled`/`:enabled`/`:read-write`/
//! `:read-only`/`:placeholder-shown`/`:indeterminate`/`:default` 的权威判定）。R3277-R3279
//! 为闭合 DOM 选择器与 style-system CSS 的一致性，把表单状态伪类逻辑提升为 Document 权威
//! 方法（DOM `query.rs` `element_matches_selector` 与 style-system matcher 共享之）。
//!
//! 作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段（`nodes`）与
//! `mod.rs` 的私有查询助手（`parent_element_node` 等）——Rust 隐私规则：私有项对定义模块及
//! 其后代可见，故无需任何可见性改动（行为不变重组，镜像 R3164 `shadow.rs` 拆分模式）。

use crate::node::{NodeId, NodeKind};

use super::Document;

impl Document {
    /// `:disabled` 的权威判定（HTML spec §4.10.18「禁用」概念）。
    ///
    /// 表单控件（button/input/select/textarea/option/optgroup）在以下任一情形被视为禁用：
    /// ① 自身带 `disabled` 布尔属性；② 位于带 `disabled` 属性的「禁用源」祖先元素后代——
    /// 禁用源含 `<fieldset>`（spec：首个 `<legend>` 内元素豁免）、`<select>`（其 option
    /// 随之禁用，§4.10.10）、`<optgroup disabled>`（其 option 禁用）。沿祖先链求值（须
    /// Document 上下文，故 [`crate::query`] `matches_full` 延后返 true，由本方法复评）。
    ///
    /// 供 DOM `:disabled`/`:enabled` 选择器（`element_matches_selector`）与
    /// style-system `:disabled` CSS 匹配共享，保证选择器与样式一致。
    pub fn is_effectively_disabled(&self, node: NodeId) -> bool {
        let tag = match self.nodes.get(node).and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.local_name().to_string()),
            _ => None,
        }) {
            Some(t) => t,
            None => return false,
        };
        // 仅表单控件适用 `:disabled`（fieldset 自身的禁用态由样式系统按 disabled 属性直判）。
        if !is_disableable_tag(&tag) {
            return false;
        }
        // ① 自身 disabled 属性。
        if self.get_attribute(node, "disabled").is_some() {
            return true;
        }
        // ② 祖先链找禁用源（fieldset/select/optgroup 带 disabled），记录是否位于
        // 禁用 fieldset 的首个 <legend> 内（spec 豁免）。
        let mut in_first_legend_of_disabled_fieldset = false;
        let mut current = node;
        while let Some(parent) = self.parent_element_node(current) {
            let parent_tag = self
                .nodes
                .get(parent)
                .and_then(|n| match &n.kind {
                    NodeKind::Element(e) => Some(e.local_name()),
                    _ => None,
                })
                .unwrap_or("");
            if parent_tag == "legend" {
                // 是否为最近 fieldset 祖先的首个 legend 后代（spec 豁免条件）。
                // 标记，由后续 fieldset 祖先决定是否豁免。
                if let Some(fs) = self.parent_element_node(parent) {
                    let is_fs = self
                        .nodes
                        .get(fs)
                        .and_then(|n| match &n.kind {
                            NodeKind::Element(e) => Some(e.local_name() == "fieldset"),
                            _ => None,
                        })
                        .unwrap_or(false);
                    if is_fs && self.is_first_legend_of(fs, parent) {
                        in_first_legend_of_disabled_fieldset = true;
                    }
                }
            }
            let is_disabled_source = matches!(parent_tag, "fieldset" | "select" | "optgroup")
                && self.get_attribute(parent, "disabled").is_some();
            if is_disabled_source {
                // fieldset：首个 legend 内元素豁免；select/optgroup：无豁免，后代 option 全禁用。
                return !(parent_tag == "fieldset" && in_first_legend_of_disabled_fieldset);
            }
            current = parent;
        }
        false
    }

    /// `:read-write` 权威判定（CSS Basic UI + HTML spec「mutable」）——可编辑文本控件：
    /// `<textarea>` 或文本可编辑 type 的 `<input>`，且**非禁用**（含 `<fieldset disabled>`
    /// 祖先传播，经 [`Self::is_effectively_disabled`]）且无 `readonly`。与 style-system CSS
    /// `:read-write` 同源（保证 DOM 选择器与 CSS 一致）。注：`contenteditable` 未实现。
    pub fn is_effectively_read_write(&self, node: NodeId) -> bool {
        let (tag, has_readonly, has_disabled_attr, input_type) =
            match self.nodes.get(node).and_then(|n| match &n.kind {
                NodeKind::Element(e) => Some((
                    e.local_name().to_string(),
                    e.has_attribute("readonly"),
                    e.has_attribute("disabled"),
                    e.get_attribute("type").unwrap_or_default().to_ascii_lowercase(),
                )),
                _ => None,
            }) {
                Some(t) => t,
                None => return false,
            };
        if has_readonly {
            return false;
        }
        let is_text_editable = match tag.as_str() {
            "textarea" => true,
            "input" => is_text_editable_input_type(&input_type),
            _ => false,
        };
        if !is_text_editable {
            return false;
        }
        // 禁用控件只读（含 fieldset 传播禁用）。
        if has_disabled_attr || self.is_effectively_disabled(node) {
            return false;
        }
        true
    }

    /// 元素 local_name（小写原始 tag）；非元素返 None。多处表单状态伪类求值共用。
    /// `pub(super)`：R3284 validation.rs 子模块（同 document 模块后代）复用此 helper。
    pub(super) fn element_local_name(&self, node: NodeId) -> Option<&str> {
        self.nodes.get(node).and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.local_name()),
            _ => None,
        })
    }

    /// 元素直接子文本节点是否有非空（非纯空白）内容。
    /// `pub(super)`：R3284 validation.rs 子模块复用（textarea value_missing 求值）。
    pub(super) fn element_has_text_content(&self, node: NodeId) -> bool {
        for &child in &self.child_nodes(node) {
            if let Some(n) = self.nodes.get(child)
                && let NodeKind::Text(data) = &n.kind
                && !data.content.trim().is_empty()
            {
                return true;
            }
        }
        false
    }

    /// `:placeholder-shown`（CSS UI）：input/textarea 正在显示 placeholder。
    /// = 有 `placeholder` 属性 且 当前无值：`<input>` 的 `value` 属性为空/缺省；
    /// `<textarea>` 的文本内容为空/纯空白。供 DOM `:placeholder-shown` 选择器与 CSS 同源。
    pub fn is_placeholder_shown(&self, node: NodeId) -> bool {
        let tag = match self.element_local_name(node) {
            Some(t) => t,
            None => return false,
        };
        if self.get_attribute(node, "placeholder").is_none() {
            return false;
        }
        match tag {
            "input" => self.get_attribute(node, "value").is_none_or(|v| v.is_empty()),
            "textarea" => !self.element_has_text_content(node),
            _ => false,
        }
    }

    /// `:blank`（CSS UI L4 / Selectors L4 §12）：值空或纯空白的文本输入控件。
    /// `<input>` 的 `value` 属性空/缺省；`<textarea>` 的文本内容空/纯空白。与
    /// [`Self::is_placeholder_shown`] 的空值检测同源，但**不要求** `placeholder` 属性
    /// （`:blank` 为无条件空值匹配，`:placeholder-shown` 须 placeholder 存在）。
    /// 供 DOM `:blank` 选择器与 CSS matcher 同源（R3300）。
    ///
    /// **语义范围**：当前仅文本输入控件（input/textarea）。Selectors L4 草案的 `:blank` 原始定义更宽
    /// （任何空内容元素，与 `:empty` 相近但容忍纯空白），但现实浏览器实现聚焦空输入控件
    /// （Firefox 仅 `:blank`/`:placeholder-shown` 在 input 上）。本实现取可静态判定且高频的子集。
    pub fn is_blank_element(&self, node: NodeId) -> bool {
        let tag = match self.element_local_name(node) {
            Some(t) => t,
            None => return false,
        };
        match tag {
            "input" => self.get_attribute(node, "value").is_none_or(|v| v.is_empty()),
            "textarea" => !self.element_has_text_content(node),
            _ => false,
        }
    }

    /// `:indeterminate`（HTML §4.15）静态可判定子集——
    /// - `<progress>` 无 `value` 属性（不确定进度条）；
    /// - `<input type="radio">` 其组（同 name + 同 form 宿主）内无任何 checked 成员。
    ///   checkbox 的 indeterminate 为动态 IDL 状态（无内容属性），静态不可知，不匹配。
    pub fn is_indeterminate(&self, node: NodeId) -> bool {
        let tag = match self.element_local_name(node) {
            Some(t) => t,
            None => return false,
        };
        match tag {
            "progress" => self.get_attribute(node, "value").is_none(),
            "input" => {
                let ty = self
                    .get_attribute(node, "type")
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if ty != "radio" {
                    return false;
                }
                let name = self.get_attribute(node, "name").unwrap_or_default();
                let owner = self.form_owner(node);
                let scope = owner.unwrap_or_else(|| self.root());
                !self.radio_group_has_checked(scope, &name, owner)
            }
            _ => false,
        }
    }

    /// 表单宿主：最近的 `<form>` 祖先元素。注：`form` 属性跨树关联未实现。
    /// `pub(super)`：R3284 validation.rs 子模块复用（radio required 组求值）。
    pub(super) fn form_owner(&self, node: NodeId) -> Option<NodeId> {
        let mut cur = self.parent_node(node);
        while let Some(p) = cur {
            if self.element_local_name(p) == Some("form") {
                return Some(p);
            }
            cur = self.parent_node(p);
        }
        None
    }

    /// `<input type="radio">` 且属于组（同 name + 同 form 宿主）。
    fn is_radio_in_group(&self, node: NodeId, name: &str, group_owner: Option<NodeId>) -> bool {
        if self.element_local_name(node) != Some("input") {
            return false;
        }
        let ty = self
            .get_attribute(node, "type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        ty == "radio"
            && self.get_attribute(node, "name").unwrap_or_default() == name
            && self.form_owner(node) == group_owner
    }

    /// 树序扫描子树，组内是否有 checked 成员。
    /// `pub(super)`：R3284 validation.rs 子模块复用（radio required 组 valueMissing 求值）。
    pub(super) fn radio_group_has_checked(&self, root: NodeId, name: &str, group_owner: Option<NodeId>) -> bool {
        if self.is_radio_in_group(root, name, group_owner) && self.get_attribute(root, "checked").is_some() {
            return true;
        }
        for &child in &self.child_nodes(root) {
            if self
                .nodes
                .get(child)
                .is_some_and(|n| matches!(n.kind, NodeKind::Element(_)))
                && self.radio_group_has_checked(child, name, group_owner)
            {
                return true;
            }
        }
        false
    }

    /// `:default`（HTML §4.15）：默认表单元素——
    /// `<option selected>` / `<input type=checkbox|radio>` 带 `checked` / form 内首个 submit 按钮。
    /// 供 DOM `:default` 选择器与 CSS 同源。
    pub fn is_default_form_element(&self, node: NodeId) -> bool {
        let tag = match self.element_local_name(node) {
            Some(t) => t,
            None => return false,
        };
        match tag {
            "option" => self.get_attribute(node, "selected").is_some(),
            "input" => {
                let ty = self
                    .get_attribute(node, "type")
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                match ty.as_str() {
                    "checkbox" | "radio" => self.get_attribute(node, "checked").is_some(),
                    _ => self.is_default_submit_button(node),
                }
            }
            "button" => self.is_default_submit_button(node),
            _ => false,
        }
    }

    /// submit 默认按钮判定：submit 候选（`<button>` 非 button/reset/menu，或 `<input type=submit|image>`）
    /// + 有 form 宿主 + 为该 form 内树序首个 submit 候选。
    fn is_default_submit_button(&self, node: NodeId) -> bool {
        if !self.is_submit_button_candidate(node) {
            return false;
        }
        match self.form_owner(node) {
            Some(form) => self.first_submit_button_in(form) == Some(node),
            None => false,
        }
    }

    /// submit 按钮候选（HTML §4.10.22）。
    fn is_submit_button_candidate(&self, node: NodeId) -> bool {
        let Some(tag) = self.element_local_name(node) else {
            return false;
        };
        let ty = self
            .get_attribute(node, "type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        match tag {
            "button" => !matches!(ty.as_str(), "button" | "reset" | "menu"),
            "input" => matches!(ty.as_str(), "submit" | "image"),
            _ => false,
        }
    }

    /// 树序扫描 form 子树，首个 submit 按钮候选（document order）。
    fn first_submit_button_in(&self, form: NodeId) -> Option<NodeId> {
        if self.is_submit_button_candidate(form) {
            return Some(form);
        }
        for &child in &self.child_nodes(form) {
            if self
                .nodes
                .get(child)
                .is_some_and(|n| matches!(n.kind, NodeKind::Element(_)))
                && let Some(found) = self.first_submit_button_in(child)
            {
                return Some(found);
            }
        }
        None
    }

    /// `node` 是否为 `fieldset` 的首个 `<legend>` 元素后代（HTML spec：fieldset 的首个 legend
    /// 子元素，其内控件不随 fieldset disabled 禁用）。legend 须为 fieldset 的**直接元素子**且
    /// 为首个 legend 类型元素子。
    fn is_first_legend_of(&self, fieldset: NodeId, legend: NodeId) -> bool {
        let children = self
            .nodes
            .get(fieldset)
            .map(|n| n.children.to_vec())
            .unwrap_or_default();
        for c in children {
            let is_legend = self
                .nodes
                .get(c)
                .and_then(|n| match &n.kind {
                    NodeKind::Element(e) => Some(e.local_name() == "legend"),
                    _ => None,
                })
                .unwrap_or(false);
            if is_legend {
                return c == legend;
            }
        }
        false
    }
}

/// HTML spec 可禁用元素 tag 集（`:enabled`/`:disabled` 适用范围；HTML §4.10.18）。
/// 注：spec「禁用」概念对 button/input/select/textarea/option/optgroup 适用；`<fieldset>`
/// 自身虽可带 disabled，但属「禁用源」而非「被禁用控件」，故 `:disabled` 选择器对 fieldset
/// 的匹配由样式系统按属性直判（此处不含 fieldset）。
pub(super) fn is_disableable_tag(tag: &str) -> bool {
    matches!(tag, "button" | "input" | "select" | "textarea" | "option" | "optgroup")
}

/// `:enabled` 用——node 是否为可禁用元素（NodeId 版）。
pub(super) fn is_disableable_tag_of_node(doc: &Document, node: NodeId) -> bool {
    doc.nodes
        .get(node)
        .and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(is_disableable_tag(e.local_name())),
            _ => None,
        })
        .unwrap_or(false)
}

/// 文本可编辑 input type（HTML spec「mutable」文本输入集；与 style-system / query.rs 同源）。
pub(super) fn is_text_editable_input_type(ty: &str) -> bool {
    matches!(
        ty,
        "" | "text"
            | "search"
            | "url"
            | "tel"
            | "email"
            | "password"
            | "date"
            | "month"
            | "week"
            | "time"
            | "datetime-local"
            | "number"
    )
}
