//! 基础选择器解析与匹配。
//!
//! 支持的简单选择器格式：
//! - 标签名：`"div"`, `"span"`
//! - ID 选择器：`"#myid"`
//! - 类选择器：`".myclass"`
//! - 属性选择器：`"[attr]"`, `"[attr=value]"`, `"[attr~=value]"`

use crate::node::ElementData;

/// 简单选择器（仅支持单层选择器，不支持组合器）。
#[derive(Debug, Clone)]
pub struct SimpleSelector {
    /// 标签名匹配（大小写不敏感）。
    pub tag: Option<String>,
    /// ID 匹配。
    pub id: Option<String>,
    /// 类名匹配列表（支持多个类选择器，如 `.a.b`）。
    pub classes: Vec<String>,
    /// 属性匹配。
    pub attribute: Option<AttributeSelector>,
}

/// 属性选择器。
#[derive(Debug, Clone)]
pub struct AttributeSelector {
    /// 属性名。
    pub name: String,
    /// 属性值匹配模式。
    pub matcher: AttributeMatcher,
}

/// 属性值匹配模式。
#[derive(Debug, Clone)]
pub enum AttributeMatcher {
    /// 仅存在：`[attr]`
    Exists,
    /// 精确匹配：`[attr=value]`
    Exact(String),
    /// 空格分隔列表包含：`[attr~=value]`
    Includes(String),
}

impl SimpleSelector {
    /// 检查元素是否匹配此选择器。
    pub fn matches(&self, elem: &ElementData) -> bool {
        // 标签名匹配
        if let Some(tag) = &self.tag
            && !elem.local_name().eq_ignore_ascii_case(tag)
        {
            return false;
        }

        // ID 匹配
        if let Some(id) = &self.id
            && elem.id.as_deref() != Some(id.as_str())
        {
            return false;
        }

        // 类名匹配（所有指定的类名都必须存在）
        for class in &self.classes {
            if !elem.class_list.iter().any(|c| c == class) {
                return false;
            }
        }

        // 属性匹配
        if let Some(attr_sel) = &self.attribute {
            match &attr_sel.matcher {
                AttributeMatcher::Exists => {
                    if !elem.has_attribute(&attr_sel.name) {
                        return false;
                    }
                }
                AttributeMatcher::Exact(value) => {
                    if elem.get_attribute(&attr_sel.name).as_deref() != Some(value.as_str()) {
                        return false;
                    }
                }
                AttributeMatcher::Includes(value) => {
                    let attr_val = match elem.get_attribute(&attr_sel.name) {
                        Some(v) => v,
                        None => return false,
                    };
                    if !attr_val.split_whitespace().any(|v| v == value) {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// 选择器组合器（连接两个简单选择器）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// 后代选择器（空格）。
    Descendant,
    /// 子选择器（`>`）。
    Child,
}

/// 由简单选择器与组合器构成的选择器链（如 `div > span.foo`）。
#[derive(Debug, Clone)]
pub struct SelectorChain {
    /// 从左到右的简单选择器序列。
    pub parts: Vec<SimpleSelector>,
    /// `combinators[i]` 连接 `parts[i]` 与 `parts[i + 1]`。
    pub combinators: Vec<Combinator>,
}

/// 解析含后代/子选择器的选择器链；单段时退化为简单选择器。
pub fn parse_selector_chain(selector: &str) -> Option<SelectorChain> {
    let trimmed = selector.trim();
    if trimmed.is_empty() {
        return None;
    }
    let segments: Vec<&str> = trimmed.split('>').map(str::trim).collect();
    let mut parts = Vec::new();
    let mut combinators = Vec::new();

    for (seg_idx, segment) in segments.iter().enumerate() {
        let subs: Vec<&str> = segment.split_whitespace().filter(|s| !s.is_empty()).collect();
        if subs.is_empty() {
            return None;
        }
        for (sub_idx, sub) in subs.iter().enumerate() {
            parts.push(parse_simple_selector(sub)?);
            let is_last_in_segment = sub_idx + 1 == subs.len();
            let is_last_segment = seg_idx + 1 == segments.len();
            if !is_last_in_segment {
                combinators.push(Combinator::Descendant);
            } else if !is_last_segment {
                combinators.push(Combinator::Child);
            }
        }
    }

    if parts.len() == 1 {
        combinators.clear();
    }

    Some(SelectorChain { parts, combinators })
}
///
/// 支持格式：
/// - `"div"` — 标签名
/// - `"#myid"` — ID
/// - `".myclass"` — 类名
/// - `"[attr]"` — 属性存在
/// - `"[attr=value]"` — 属性精确匹配
/// - `"[attr~=value]"` — 属性空格分隔匹配
/// - `"div#id.class[attr=val]"` — 组合
pub fn parse_simple_selector(selector: &str) -> Option<SimpleSelector> {
    let s = selector.trim();
    if s.is_empty() {
        return None;
    }

    let mut result = SimpleSelector {
        tag: None,
        id: None,
        classes: Vec::new(),
        attribute: None,
    };

    let mut rest = s;

    // 解析标签名（开头的连续非特殊字符）
    if let Some(pos) = rest.find(['#', '.', '[']) {
        if pos > 0 {
            result.tag = Some(rest[..pos].to_string());
        }
        rest = &rest[pos..];
    } else {
        result.tag = Some(rest.to_string());
        return Some(result);
    }

    // 解析后续的选择器部分
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('#') {
            // ID 选择器
            let end = r.find(['.', '[']).unwrap_or(r.len());
            if end == 0 {
                return None; // 空的 ID 选择器
            }
            result.id = Some(r[..end].to_string());
            rest = &r[end..];
        } else if let Some(r) = rest.strip_prefix('.') {
            // 类选择器
            let end = r.find(['#', '.', '[']).unwrap_or(r.len());
            if end == 0 {
                return None; // 空的类选择器
            }
            result.classes.push(r[..end].to_string());
            rest = &r[end..];
        } else {
            let r = rest.strip_prefix('[')?;
            // 属性选择器
            let end_bracket = r.find(']')?;
            let attr_content = &r[..end_bracket];

            let attr_sel = if let Some(eq_pos) = attr_content.find("~=") {
                AttributeSelector {
                    name: attr_content[..eq_pos].trim().to_string(),
                    matcher: AttributeMatcher::Includes(attr_content[eq_pos + 2..].trim().to_string()),
                }
            } else if let Some(eq_pos) = attr_content.find('=') {
                AttributeSelector {
                    name: attr_content[..eq_pos].trim().to_string(),
                    matcher: AttributeMatcher::Exact(attr_content[eq_pos + 1..].trim().to_string()),
                }
            } else {
                AttributeSelector {
                    name: attr_content.trim().to_string(),
                    matcher: AttributeMatcher::Exists,
                }
            };

            result.attribute = Some(attr_sel);
            rest = &r[end_bracket + 1..];
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_selector() {
        let sel = parse_simple_selector("div").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("div"));
        assert!(sel.id.is_none());
        assert!(sel.classes.is_empty());
    }

    #[test]
    fn test_parse_id_selector() {
        let sel = parse_simple_selector("#myid").unwrap();
        assert!(sel.tag.is_none());
        assert_eq!(sel.id.as_deref(), Some("myid"));
    }

    #[test]
    fn test_parse_class_selector() {
        let sel = parse_simple_selector(".myclass").unwrap();
        assert!(sel.tag.is_none());
        assert_eq!(sel.classes, vec!["myclass"]);
    }

    #[test]
    fn test_parse_attribute_selector_exists() {
        let sel = parse_simple_selector("[data-test]").unwrap();
        assert!(sel.attribute.is_some());
        let attr = sel.attribute.unwrap();
        assert_eq!(attr.name, "data-test");
        assert!(matches!(attr.matcher, AttributeMatcher::Exists));
    }

    #[test]
    fn test_parse_attribute_selector_exact() {
        let sel = parse_simple_selector("[type=text]").unwrap();
        let attr = sel.attribute.unwrap();
        assert_eq!(attr.name, "type");
        assert!(matches!(attr.matcher, AttributeMatcher::Exact(v) if v == "text"));
    }

    #[test]
    fn test_parse_combined_selector() {
        let sel = parse_simple_selector("div#myid.myclass").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("div"));
        assert_eq!(sel.id.as_deref(), Some("myid"));
        assert_eq!(sel.classes, vec!["myclass"]);
    }

    #[test]
    fn test_parse_multiple_class_selector() {
        let sel = parse_simple_selector(".foo.bar").unwrap();
        assert!(sel.tag.is_none());
        assert_eq!(sel.classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_parse_tag_with_multiple_classes() {
        let sel = parse_simple_selector("div.a.b").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("div"));
        assert_eq!(sel.classes, vec!["a", "b"]);
    }

    #[test]
    fn test_parse_empty_selector() {
        assert!(parse_simple_selector("").is_none());
        assert!(parse_simple_selector("  ").is_none());
    }

    #[test]
    fn test_parse_selector_chain_descendant() {
        let chain = parse_selector_chain("div .child").unwrap();
        assert_eq!(chain.parts.len(), 2);
        assert_eq!(chain.combinators, vec![Combinator::Descendant]);
    }

    #[test]
    fn test_parse_selector_chain_child() {
        let chain = parse_selector_chain("div > span").unwrap();
        assert_eq!(chain.parts.len(), 2);
        assert_eq!(chain.combinators, vec![Combinator::Child]);
    }
}
