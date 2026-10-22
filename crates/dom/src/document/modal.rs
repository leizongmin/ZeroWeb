//! Document `:modal` 伪类判定 —— 镜像 `focus.rs` 注入通道模式。
//!
//! 本模块为 [`super::Document`] 的「模态 dialog」面（Selectors L4 `:modal-pseudo`；
//! HTML 渲染规范 `dialog:modal` UA 定位的开关）。模态性是运行时状态：只存在于
//! engine shim JS 的模态栈（`showModal()`/`close()` 维护），DOM 序列化不携带 →
//! render 路径每帧 fresh-parse 的新 Document 由管线注入 retained 列表
//! （[`super::Document::set_modal_dialogs`]）。CSS matcher 与 DOM `querySelector`
//! 共享本判定，保证选择器与样式一致。
//!
//! 作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段
//! （`modal_dialogs`）——Rust 隐私规则：私有项对定义模块及其后代可见。

use super::Document;
use crate::node::NodeId;

impl Document {
    /// `:modal` 的权威判定（CSS Selectors L4 `:modal-pseudo`；HTML 渲染规范
    /// #modal-dialogs）。
    ///
    /// 本节点在管线注入的模态 dialog 集合中 → 匹配（集合由 shim `showModal()`/
    /// `close()` 的真相源经管线 retained 状态逐帧重注）。非 dialog、非模态或
    /// 未注入（静态解析路径）→ 不匹配。
    pub fn is_modal_dialog(&self, node: NodeId) -> bool {
        self.modal_dialogs.contains(&node)
    }
}
