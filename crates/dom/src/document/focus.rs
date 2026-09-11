//! Document `:focus`/`:focus-within` 伪类判定 —— 镜像 `target.rs`（R3283）拆分模式。
//!
//! 本模块为 [`super::Document`] 的「焦点」面（Selectors L4 §14 `:focus-within-pseudo` /
//! §13 `:focus`）。焦点是运行时状态：engine JS 桥在 `element.focus()`/`element.blur()` 时经
//! [`super::Document::set_focus_element`] 注入（与 `document.activeElement` 的 engine 线程
//! 局部追踪同源镜像），CSS matcher 与 DOM `querySelector` 复评共享本判定，保证选择器与样式
//! 一致。解析（re-parse/导航）不携带焦点 → 新 Document 初始 `None`（同 URL fragment 的
//! `:target` 语义：读取注入的运行时状态，而非解析期属性）。
//!
//! 作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段（`focus_element`）
//! 与私有方法（`parent_node`）——Rust 隐私规则：私有项对定义模块及其后代可见。

use super::Document;
use crate::node::NodeId;

impl Document {
    /// `:focus` 的权威判定（CSS Selectors L4 §13；HTML §6.6.2 焦点元素）。
    ///
    /// 当前文档焦点元素（engine JS 桥 `.focus()` 注入）即本节点 → 匹配。无焦点或焦点在
    /// 别处 → 不匹配。`<input autofocus>` 静态声明不在此判定（焦点注入仍须 JS/用户交互；
    /// WPT 驱动面均为 `.focus()` 脚本）。
    pub fn is_focus_element(&self, node: NodeId) -> bool {
        self.focus_element == Some(node)
    }

    /// `:focus-within` 的权威判定（CSS Selectors L4 §14：自身或后代获得焦点的元素）。
    ///
    /// 等价于「本节点是焦点元素的祖先或自身」：从焦点元素沿父链上溯，命中本节点即匹配。
    /// 焦点在文档外（`None`）或无关子树 → 不匹配。上溯深度 = 焦点元素树深，O(depth)。
    pub fn has_focus_within(&self, node: NodeId) -> bool {
        let Some(mut cur) = self.focus_element else {
            return false;
        };
        loop {
            if cur == node {
                return true;
            }
            match self.parent_node(cur) {
                Some(p) => cur = p,
                None => return false,
            }
        }
    }
}
