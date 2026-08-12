//! 焦点管理器 — 实现键盘 Tab 导航和可聚焦元素追踪。
//!
//! 支持 Tab/Shift+Tab 焦点顺序遍历、tabindex 属性解析、
//! 以及判断元素是否可聚焦。

use crate::{Document, NodeId};

/// 焦点管理器，跟踪文档中的可聚焦元素和当前焦点位置。
#[derive(Debug, Clone)]
pub struct FocusManager {
    /// 当前获得焦点的节点（如有）。
    focused: Option<NodeId>,
    /// 按焦点顺序排列的可聚焦节点列表。
    focus_order: Vec<FocusableItem>,
}

/// 可聚焦项及其焦点优先级。
#[derive(Debug, Clone)]
struct FocusableItem {
    /// 节点 ID。
    node: NodeId,
    /// tabindex 值：正数表示显式顺序，0 表示文档顺序，None 表示不可 Tab 聚焦。
    tabindex: i32,
}

/// 默认可聚焦的 HTML 元素标签（不含 tabindex 时可通过 Tab 聚焦）。
const FOCUSABLE_TAGS: &[&str] = &["a", "button", "input", "select", "textarea", "summary", "details"];

impl FocusManager {
    /// 创建新的焦点管理器。
    pub fn new() -> Self {
        Self {
            focused: None,
            focus_order: Vec::new(),
        }
    }

    /// 扫描文档，构建可聚焦元素列表。
    ///
    /// 遍历 DOM 树，找到所有带 tabindex 或属于默认可聚焦标签的元素。
    /// 按 tabindex 正值优先（升序），然后是 tabindex=0 和默认可聚焦元素（文档顺序）。
    pub fn scan(&mut self, doc: &Document) {
        self.focus_order.clear();

        // 递归遍历 DOM 树
        self.scan_node(doc, doc.root());

        // 排序：tabindex > 0 的在前（按 tabindex 升序），
        // tabindex = 0 和默认可聚焦的在后（保持文档顺序）。
        self.focus_order.sort_by(|a, b| {
            match (a.tabindex, b.tabindex) {
                (x, y) if x > 0 && y > 0 => x.cmp(&y),
                (x, _) if x > 0 => std::cmp::Ordering::Less,
                (_, y) if y > 0 => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal, // 保持原始文档顺序
            }
        });
    }

    /// 递归扫描节点及其子节点。
    fn scan_node(&mut self, doc: &Document, node_id: NodeId) {
        let Some(node) = doc.get(node_id) else {
            return;
        };

        if let crate::NodeKind::Element(elem) = &node.kind {
            let tag = elem.local_name();

            // 检查 tabindex 属性
            let tabindex_attr = elem.get_attribute("tabindex");
            let tabindex = parse_tabindex(tabindex_attr.as_deref());
            let eligible = is_sequentially_focusable(doc, node_id, elem);

            // 判断是否可聚焦：
            // - tabindex >= 0 → 可聚焦
            // - 无 tabindex 但属于默认可聚焦标签 → 可聚焦（tabindex = 0）
            // - tabindex = -1 → 仅可通过程序聚焦，不在 Tab 顺序中
            if let Some(tb) = tabindex {
                if tb >= 0 && eligible {
                    self.focus_order.push(FocusableItem {
                        node: node_id,
                        tabindex: tb,
                    });
                }
                // tabindex = -1: 不加入 Tab 顺序
            } else if FOCUSABLE_TAGS.contains(&tag) && eligible && has_natural_focus_behavior(tag, elem) {
                self.focus_order.push(FocusableItem {
                    node: node_id,
                    tabindex: 0,
                });
            }
        }

        // 递归子节点
        let children: Vec<NodeId> = node.children.clone();
        for child in children {
            self.scan_node(doc, child);
        }
    }

    /// 获取当前焦点节点。
    pub fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    /// 设置焦点到指定节点。
    ///
    /// 如果节点不在可聚焦列表中，仍然设置焦点（用于程序化聚焦 tabindex=-1 元素）。
    pub fn set_focus(&mut self, node: Option<NodeId>) {
        self.focused = node;
    }

    /// 移动焦点到下一个可 Tab 聚焦的元素（Tab 键行为）。
    ///
    /// 如果当前无焦点，聚焦第一个可 Tab 元素。
    /// 如果已在最后一个元素，循环回第一个。
    pub fn focus_next(&mut self) -> Option<NodeId> {
        if self.focus_order.is_empty() {
            return None;
        }

        let next_idx = match self.focused {
            None => 0,
            Some(current) => {
                let current_idx = self.focus_order.iter().position(|item| item.node == current);
                match current_idx {
                    Some(idx) if idx + 1 < self.focus_order.len() => idx + 1,
                    Some(_) => 0, // 循环回到第一个
                    None => 0,
                }
            }
        };

        let item = &self.focus_order[next_idx];
        self.focused = Some(item.node);
        self.focused
    }

    /// 移动焦点到上一个可 Tab 聚焦的元素（Shift+Tab 键行为）。
    pub fn focus_previous(&mut self) -> Option<NodeId> {
        if self.focus_order.is_empty() {
            return None;
        }

        let prev_idx = match self.focused {
            None => self.focus_order.len() - 1,
            Some(current) => {
                let current_idx = self.focus_order.iter().position(|item| item.node == current);
                match current_idx {
                    Some(0) => self.focus_order.len() - 1,
                    Some(idx) => idx - 1,
                    None => self.focus_order.len() - 1,
                }
            }
        };

        let item = &self.focus_order[prev_idx];
        self.focused = Some(item.node);
        self.focused
    }

    /// 聚焦第一个可 Tab 元素。
    pub fn focus_first(&mut self) -> Option<NodeId> {
        if self.focus_order.is_empty() {
            return None;
        }
        let item = &self.focus_order[0];
        self.focused = Some(item.node);
        self.focused
    }

    /// 聚焦最后一个可 Tab 元素。
    pub fn focus_last(&mut self) -> Option<NodeId> {
        if self.focus_order.is_empty() {
            return None;
        }
        let item = &self.focus_order.last()?;
        self.focused = Some(item.node);
        self.focused
    }

    /// 获取可聚焦元素总数。
    pub fn focusable_count(&self) -> usize {
        self.focus_order.len()
    }

    /// 检查指定节点是否在可 Tab 聚焦列表中。
    pub fn is_tab_focusable(&self, node: NodeId) -> bool {
        self.focus_order.iter().any(|item| item.node == node)
    }

    /// 获取指定节点的 tabindex 值。
    ///
    /// 返回 `None` 表示节点不在可聚焦列表中。
    pub fn get_tabindex(&self, node: NodeId) -> Option<i32> {
        self.focus_order
            .iter()
            .find(|item| item.node == node)
            .map(|item| item.tabindex)
    }

    /// 清除焦点状态。
    pub fn blur(&mut self) {
        self.focused = None;
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析 tabindex 属性值。
///
/// - `None` → 无 tabindex 属性
/// - `"3"` → `Some(3)`
/// - `"-1"` → `Some(-1)`
/// - `""` 或无效值 → `Some(0)`（浏览器行为：空 tabindex 等同于 0）
fn parse_tabindex(value: Option<&str>) -> Option<i32> {
    match value {
        None => None,
        Some("") => Some(0),
        Some(s) => s.trim().parse::<i32>().ok().or(Some(0)),
    }
}

/// 检查元素是否被禁用（disabled 属性）。
fn is_disabled(elem: &crate::ElementData) -> bool {
    elem.get_attribute("disabled").is_some()
}

fn is_sequentially_focusable(doc: &Document, node: NodeId, elem: &crate::ElementData) -> bool {
    if is_disabled(elem)
        || (elem.local_name().eq_ignore_ascii_case("input")
            && elem
                .get_attribute("type")
                .is_some_and(|value| value.eq_ignore_ascii_case("hidden")))
    {
        return false;
    }
    let mut current = Some(node);
    while let Some(id) = current {
        let Some(node) = doc.get(id) else {
            return false;
        };
        if let crate::NodeKind::Element(element) = &node.kind
            && (element.get_attribute("hidden").is_some() || element.get_attribute("inert").is_some())
        {
            return false;
        }
        current = doc.parent_node(id);
    }
    true
}

fn has_natural_focus_behavior(tag: &str, elem: &crate::ElementData) -> bool {
    !tag.eq_ignore_ascii_case("a") || elem.get_attribute("href").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_html;

    #[test]
    fn test_parse_tabindex() {
        assert_eq!(parse_tabindex(None), None);
        assert_eq!(parse_tabindex(Some("")), Some(0));
        assert_eq!(parse_tabindex(Some("3")), Some(3));
        assert_eq!(parse_tabindex(Some("-1")), Some(-1));
        assert_eq!(parse_tabindex(Some("0")), Some(0));
        assert_eq!(parse_tabindex(Some("abc")), Some(0));
        assert_eq!(parse_tabindex(Some("  5  ")), Some(5));
    }

    #[test]
    fn test_empty_document() {
        let doc = parse_html("<html><body></body></html>");
        let mut fm = FocusManager::new();
        fm.scan(&doc);
        assert_eq!(fm.focusable_count(), 0);
        assert_eq!(fm.focus_next(), None);
        assert_eq!(fm.focus_previous(), None);
    }

    #[test]
    fn test_natural_focus_order() {
        let doc = parse_html(
            r#"<html><body>
            <a href="/home">Home</a>
            <button>Click</button>
            <input type="text" />
        </body></html>"#,
        );
        let mut fm = FocusManager::new();
        fm.scan(&doc);
        assert_eq!(fm.focusable_count(), 3);

        // Tab 循环
        let first = fm.focus_next();
        assert!(first.is_some());
        let second = fm.focus_next();
        assert!(second.is_some());
        let third = fm.focus_next();
        assert!(third.is_some());
        // 循环回第一个
        let wrap = fm.focus_next();
        assert_eq!(wrap, first);
    }

    #[test]
    fn test_tabindex_ordering() {
        let doc = parse_html(
            r#"<html><body>
            <div tabindex="3">Third</div>
            <div tabindex="1">First</div>
            <button>After tabindex</button>
            <div tabindex="2">Second</div>
        </body></html>"#,
        );
        let mut fm = FocusManager::new();
        fm.scan(&doc);

        // tabindex=1 → tabindex=2 → tabindex=3 → button (natural order)
        assert_eq!(fm.focusable_count(), 4);

        let first = fm.focus_next();
        let first_idx = fm.get_tabindex(first.unwrap());
        assert_eq!(first_idx, Some(1));

        let second = fm.focus_next();
        let second_idx = fm.get_tabindex(second.unwrap());
        assert_eq!(second_idx, Some(2));

        let third = fm.focus_next();
        let third_idx = fm.get_tabindex(third.unwrap());
        assert_eq!(third_idx, Some(3));

        let fourth = fm.focus_next();
        let fourth_idx = fm.get_tabindex(fourth.unwrap());
        assert_eq!(fourth_idx, Some(0)); // button 自然顺序
    }

    #[test]
    fn test_tabindex_negative_excluded() {
        let doc = parse_html(
            r#"<html><body>
            <button>First</button>
            <div tabindex="-1">Not tabbable</div>
            <button>Second</button>
        </body></html>"#,
        );
        let mut fm = FocusManager::new();
        fm.scan(&doc);
        assert_eq!(fm.focusable_count(), 2);
    }

    #[test]
    fn test_focus_previous() {
        let doc = parse_html(
            r#"<html><body>
            <a href="/">A</a>
            <button>B</button>
            <input type="text" />
        </body></html>"#,
        );
        let mut fm = FocusManager::new();
        fm.scan(&doc);

        let third = fm.focus_last();
        let _second = fm.focus_previous();
        let _first = fm.focus_previous();
        // 循环回最后一个
        let wrap = fm.focus_previous();
        assert_eq!(wrap, third);
    }

    #[test]
    fn test_focus_first_last() {
        let doc = parse_html(
            r#"<html><body>
            <a href="/">First</a>
            <button>Middle</button>
            <input type="text" />
        </body></html>"#,
        );
        let mut fm = FocusManager::new();
        fm.scan(&doc);

        let first = fm.focus_first();
        let last = fm.focus_last();
        assert_ne!(first, last);
    }

    #[test]
    fn test_blur() {
        let doc = parse_html(r#"<html><body><button>B</button></body></html>"#);
        let mut fm = FocusManager::new();
        fm.scan(&doc);
        fm.focus_first();
        assert!(fm.focused().is_some());
        fm.blur();
        assert!(fm.focused().is_none());
    }

    #[test]
    fn test_set_focus_programmatic() {
        let doc = parse_html(r#"<html><body><div id="x">X</div></body></html>"#);
        let mut fm = FocusManager::new();
        fm.scan(&doc);
        // div without tabindex is not focusable via Tab
        assert_eq!(fm.focusable_count(), 0);
        // 但可以程序化聚焦
        let div = doc.query_selector(doc.root(), "#x").unwrap();
        fm.set_focus(Some(div));
        assert_eq!(fm.focused(), Some(div));
    }

    #[test]
    fn test_disabled_excluded() {
        let doc = parse_html(
            r#"<html><body>
            <button>Enabled</button>
            <button disabled>Disabled</button>
            <input type="text" disabled />
            <input type="text" />
        </body></html>"#,
        );
        let mut fm = FocusManager::new();
        fm.scan(&doc);
        // 只有 enabled button 和 enabled input
        assert_eq!(fm.focusable_count(), 2);
    }

    #[test]
    fn sequential_focus_skips_non_focusable_controls() {
        let doc = parse_html(
            r#"<html><body>
              <button id="natural">Natural</button>
              <input id="positive-two" tabindex="2">
              <input id="positive-one" tabindex="1">
              <button id="disabled-positive" disabled tabindex="3">Disabled</button>
              <input id="hidden-input" type="hidden">
              <div hidden><button id="hidden-descendant">Hidden</button></div>
              <div inert><button id="inert-descendant">Inert</button></div>
              <a id="no-href">No href</a>
              <a id="link" href="/next">Link</a>
            </body></html>"#,
        );
        let mut focus = FocusManager::new();
        focus.scan(&doc);

        let ids: Vec<String> = (0..4)
            .map(|_| {
                let node = focus.focus_next().expect("focus target");
                doc.get_attribute(node, "id").expect("id")
            })
            .collect();
        assert_eq!(ids, ["positive-one", "positive-two", "natural", "link"]);
        assert_eq!(focus.focusable_count(), 4);
    }

    #[test]
    fn test_select_textarea_focusable() {
        let doc = parse_html(
            r#"<html><body>
            <select><option>A</option></select>
            <textarea></textarea>
        </body></html>"#,
        );
        let mut fm = FocusManager::new();
        fm.scan(&doc);
        assert_eq!(fm.focusable_count(), 2);
    }

    #[test]
    fn test_default_impl() {
        let fm = FocusManager::default();
        assert_eq!(fm.focused(), None);
        assert_eq!(fm.focusable_count(), 0);
    }

    #[test]
    fn test_is_tab_focusable() {
        let doc = parse_html(
            r#"<html><body>
            <a href="/">Link</a>
            <div>Not focusable</div>
        </body></html>"#,
        );
        let mut fm = FocusManager::new();
        fm.scan(&doc);

        let link = doc.query_selector(doc.root(), "a").unwrap();
        let div = doc.query_selector(doc.root(), "div").unwrap();
        assert!(fm.is_tab_focusable(link));
        assert!(!fm.is_tab_focusable(div));
    }
}
