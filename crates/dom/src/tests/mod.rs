//! DOM crate 综合测试套件。
//!
//! 覆盖：节点类型、树操作、属性操作、HTML 解析、查询、序列化、MutationObserver。

use crate::*;

/// Helper: get the body node from a parsed HTML document.
pub(super) fn body_of(doc: &Document) -> NodeId {
    let html = doc.first_child(doc.root()).unwrap();
    doc.last_child(html).unwrap()
}

mod tests_1;
mod tests_2;
mod tests_3;
mod tests_4;
mod tests_5;
