use crate::node::NodeKind;

use super::Document;

impl Document {
    /// 获取文档中的元素节点总数。
    #[inline]
    pub fn element_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|node| matches!(&node.kind, NodeKind::Element(_)))
            .count()
    }
}
