//! Document 语言与方向性伪类判定 —— 拆自 `mod.rs`（rule 5 单文件 <2000 行，R3281）。
//!
//! 本模块为 [`super::Document`] 的语言/方向面（`:lang()`/`:dir()`/`:scope` 的权威判定）。
//! R3281 为闭合 DOM 选择器与 style-system CSS 的一致性，把这三类伪类的逻辑从 style-system
//! matcher（私有 `lang_range_matches`/`element_is_rtl`/`auto_is_rtl`/`auto_first_strong`/
//! `is_strong_rtl`/`is_strong_ltr`）提升为 Document 权威方法（DOM `query.rs`
//! `element_matches_selector` 与 style-system matcher 共享之，删去 style-system 重复 helper）。
//!
//! 作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段（`nodes`）与
//! `mod.rs` 的私有查询助手（`parent_element_node` 等）——Rust 隐私规则：私有项对定义模块及
//! 其后代可见，故无需任何可见性改动（行为不变重组，镜像 R3280 `form_state.rs`、R3164 `shadow.rs`
//! 拆分模式）。

use crate::node::{NodeId, NodeKind};

use super::Document;

impl Document {
    /// `:scope` 的权威判定（CSS Selectors L4 §8）。
    ///
    /// 文档样式表中等价 `:root`（匹配文档根元素 `<html>`）。本引擎 DOM 选择器路径镜像 style-system
    /// 语义：`:scope` 命中文档根元素。注：querySelector 的「调用元素即 scope」语义（如
    /// `el.querySelectorAll(":scope > div")`）需把 scope NodeId 贯穿 matches 链，为 follow-up。
    ///
    /// 供 DOM `:scope` 选择器（`element_matches_selector`）与 style-system `:scope` CSS 匹配共享。
    pub fn is_scope_element(&self, node: NodeId) -> bool {
        // 文档根元素 = 无元素父的元素（`<html>`），与 `compute_element_position` 的 is_root 同义。
        self.nodes
            .get(node)
            .is_some_and(|n| matches!(n.kind, NodeKind::Element(_)))
            && self.parent_element_node(node).is_none()
    }

    /// `:lang(ranges)` 的权威匹配（CSS Selectors L4 §14）。
    ///
    /// 元素语言 = 自身或最近祖先的 `xml:lang`/`lang` 属性（向上查找首个）；元素语言匹配
    /// **任一**语言范围即命中（逗号列表 OR）。范围大小写不敏感；`*` 为 BCP 47 通配符。
    /// 须祖先链求值（故 [`crate::query`] `matches_full` 延后返 true，由本方法复评）。
    ///
    /// 供 DOM `:lang()` 选择器（`element_matches_selector`）与 style-system `:lang()` CSS 匹配共享，
    /// 保证选择器与样式一致。
    pub fn matches_lang(&self, node: NodeId, ranges: &[String]) -> bool {
        let mut current = Some(node);
        while let Some(n) = current {
            // xml:lang 优先于 lang（XML 规范），HTML 仅 lang。
            if let Some(lang) = self
                .get_attribute(n, "xml:lang")
                .or_else(|| self.get_attribute(n, "lang"))
            {
                if lang.is_empty() {
                    return false;
                }
                return ranges.iter().any(|r| lang_range_matches(r, &lang));
            }
            current = self.parent_node(n);
        }
        false
    }

    /// `:dir(ltr|rtl)` 的权威匹配（CSS Selectors L4 §14）。
    ///
    /// 元素方向性 = 最近 `dir` 属性解析（ltr/rtl/auto），沿祖先继承，终极默认 LTR
    /// （HTML §3.2.6）；`auto` 按子树首个强方向字符。`dir` 参数归一化为小写，非 ltr/rtl
    /// 不匹配。须祖先链 + 子树扫描（故 [`crate::query`] `matches_full` 延后返 true，由本方法复评）。
    ///
    /// 供 DOM `:dir()` 选择器（`element_matches_selector`）与 style-system `:dir()` CSS 匹配共享。
    pub fn matches_dir(&self, node: NodeId, dir: &str) -> bool {
        let is_rtl = element_is_rtl(self, node);
        match dir {
            "rtl" => is_rtl,
            "ltr" => !is_rtl,
            // 未知方向值（含空）不匹配任何元素
            _ => false,
        }
    }
}

/// 元素是否为 RTL 方向。沿祖先找首个有效的 `dir` 属性：
/// `rtl`→true、`ltr`→false、`auto`→按子树文本首个强方向字符、无效值→继续向上；缺省→false（LTR）。
fn element_is_rtl(doc: &Document, node: NodeId) -> bool {
    let mut current = Some(node);
    while let Some(n) = current {
        if let Some(val) = doc.get_attribute(n, "dir") {
            match val.to_ascii_lowercase().as_str() {
                "rtl" => return true,
                "ltr" => return false,
                "auto" => return auto_is_rtl(doc, n),
                _ => {} // 无效值，继续向上查找
            }
        }
        current = doc.parent_node(n);
    }
    false
}

/// `dir="auto"` 方向性：子树文本（前序）首个强方向字符——RTL 脚本（希伯来/阿拉伯等）→ RTL，
/// LTR 脚本（拉丁/希腊/西里尔字母）→ LTR；无强字符 → LTR（默认）。
/// 注：简化实现，未跳过带自身 `dir` 的后代隔离节点（静态罕见边角）。
fn auto_is_rtl(doc: &Document, root: NodeId) -> bool {
    let mut found_rtl = false;
    auto_first_strong(doc, root, &mut found_rtl);
    found_rtl
}

/// 前序遍历子树，定位首个强方向字符；命中即设 `found_rtl` 并提前返回。
fn auto_first_strong(doc: &Document, node_id: NodeId, found_rtl: &mut bool) {
    if *found_rtl {
        return;
    }
    let Some(node) = doc.get(node_id) else {
        return;
    };
    match &node.kind {
        NodeKind::Text(data) => {
            for ch in data.content.chars() {
                if is_strong_rtl(ch) {
                    *found_rtl = true;
                    return;
                }
                if is_strong_ltr(ch) {
                    return; // LTR 强字符定方向
                }
            }
        }
        NodeKind::Element(_) => {
            for &child in &doc.child_nodes(node_id) {
                auto_first_strong(doc, child, found_rtl);
                if *found_rtl {
                    return;
                }
            }
        }
        _ => {}
    }
}

/// BCP 47 语言范围匹配（大小写不敏感）。
///
/// 裸 `*` 匹配任意非空语言。否则按子标签（`-` 分隔）位置匹配：范围的每个子标签须与语言对应
/// 子标签相等，`*` 匹配任意单个子标签；范围子标签数 ≤ 语言子标签数（前缀语义，故
/// `:lang(en)` 匹配 `en-US`，而 `:lang(en-US)` 不匹配 `en`）。
fn lang_range_matches(range: &str, lang: &str) -> bool {
    let range = range.to_ascii_lowercase();
    let lang = lang.to_ascii_lowercase();
    if range == "*" {
        return !lang.is_empty();
    }
    let r_tags: Vec<&str> = range.split('-').collect();
    let l_tags: Vec<&str> = lang.split('-').collect();
    if r_tags.len() > l_tags.len() {
        return false;
    }
    r_tags.iter().enumerate().all(|(i, r)| *r == "*" || *r == l_tags[i])
}

/// 强 RTL 字符：希伯来/阿拉伯/叙利亚/塔安那/恩科等 RTL 脚本区（含 presentation forms）。
fn is_strong_rtl(c: char) -> bool {
    matches!(c as u32,
        0x0590..=0x05FF   // Hebrew
        | 0x0600..=0x06FF // Arabic
        | 0x0700..=0x074F // Syriac
        | 0x0780..=0x07BF // Thaana
        | 0x07C0..=0x07FF // NKo
        | 0x0800..=0x083F // Samaritan
        | 0x0840..=0x085F // Mandaic
        | 0xFB1D..=0xFB4F // Hebrew presentation forms
        | 0xFB50..=0xFDFF // Arabic presentation forms-A
        | 0xFE70..=0xFEFF // Arabic presentation forms-B
    )
}

/// 强 LTR 字符：拉丁/希腊/西里尔字母（CJK 等中性字符不算强 LTR，继续扫描）。
fn is_strong_ltr(c: char) -> bool {
    let u = c as u32;
    (0x0041..=0x024F).contains(&u) // Latin + Latin Extended
        || (0x0370..=0x03FF).contains(&u) // Greek
        || (0x0400..=0x04FF).contains(&u) // Cyrillic
}
