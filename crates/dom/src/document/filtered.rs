//! Document `:filtered` 伪类判定 —— 镜像 `modal.rs` 注入通道模式。
//!
//! 本模块为 [`super::Document`] 的「filterable select 过滤 option」面
//!（whatwg/html #12050 谱系：`<input filter=idref>` 关联 select，输入值变化时
//! 文本不匹配的 option 进入过滤态）。过滤态是运行时状态：只存在于 engine shim
//! JS 的过滤算法（`_zwFilterableRecompute` 维护 per-input 关联与命中集），DOM
//! 序列化不携带 → shim 经 engine 桥线程局选择器槽逐查询注入 retained 列表
//!（[`super::Document::set_filtered_options`]，镜像 `:active` 的
//! `sync_active_state` 通道）。CSS matcher 与 DOM `querySelector` 共享本判定。
//!
//! 作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段
//!（`filtered_options`）——Rust 隐私规则：私有项对定义模块及其后代可见。

use super::Document;
use crate::node::NodeId;

impl Document {
    /// `:filtered` 的权威判定（filterable select 过滤态；whatwg/html #12050 谱系）。
    ///
    /// 本节点在注入的过滤 option 集合中 → 匹配（集合由 shim 过滤算法的真相源经
    /// 桥线程局槽逐查询重注）。未关联过滤输入、输入值为空或未注入（静态解析
    /// 路径）→ 不匹配。
    pub fn is_filtered_option(&self, node: NodeId) -> bool {
        self.filtered_options.contains(&node)
    }
}
