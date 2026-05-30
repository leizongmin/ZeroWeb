//! # zero-dom
//!
//! DOM 树实现 — 完整的 DOM 节点类型、树操作和增量更新。
//!
//! 基于 html5ever 解析 HTML 并构建 DOM 树，支持完整的 DOM Level 2+ 核心 API。

#![warn(missing_docs)]

/// DOM 节点 ID 类型
pub type NodeId = u64;

/// DOM 文档
pub struct Document {
    // TODO: 在 M2 里程碑中实现
}

/// DOM 节点
pub enum Node {
    // TODO: 在 M2 里程碑中实现
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        // M2 里程碑将在此处实现 DOM 树测试
    }
}
