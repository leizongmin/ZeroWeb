//! Document 纯净树快照 —— S1 文档级指纹缓存（engine `pipeline::frame_fingerprint`）
//! 的缓存值语义支撑。
//!
//! [`super::Document`] 不可 `derive(Clone)`：`observers`/`event_listeners` 持
//! `Box<dyn Fn>` 回调，不可复制。本模块提供[`super::Document::pristine_snapshot`]：
//! 深拷树存储与结构侧表（NodeId 保持有效），脚本回调态与导航态复位为解析初态
//! ——快照等价于「对同一 HTML 重新 `parse_html` 的文档」（树内容与 NodeId 逐位
//! 一致，脚本效应不携带）。
//!
//! 作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段
//! ——Rust 隐私规则：私有项对定义模块及其后代可见（`modal.rs` 同模式）。

use super::Document;

impl Document {
    /// 创建文档的**纯净树快照**：深拷节点存储与树结构侧表，脚本运行时状态与
    /// 导航态复位为解析初态。
    ///
    /// - 深拷面（快照刻的精确状态，NodeId 全部保持有效）：`nodes`（slotmap 全表，
    ///   `NodeData: Clone` 逐节点深拷）、`root`/`id_map`/`template_contents_map`/
    ///   `shadow_roots`/`slot_assignments`（NodeId 侧表随树一致）、
    ///   `encoding_label`/`scripting_enabled`（解析期产物）、`focus_element`/
    ///   `modal_dialogs`（运行时注入态，快照刻原值保留——与产生该帧的样式计算
    ///   状态一致）。
    /// - 复位面：`observers`/`pending_mutations`/`event_listeners`（脚本注册的
    ///   回调态——`parse_html` 产物恒空，回调不可复制，复位使快照脱离注册生命
    ///   周期）与 `url`/`referrer`（Parse 步注入的导航态；缓存键不含 URL，跨
    ///   导航复用时由消费方重放当前值——S1 命中臂重放 `set_url`/`set_referrer`）。
    ///
    /// 契约：`snapshot == 对同一 HTML 重新 parse_html 的文档`（树内容与 NodeId
    /// 逐位一致，脚本效应不携带）。S1 指纹缓存以此保证「回填快照不被后续脚本
    /// 原位突变写穿、命中还原 = 改前 fresh parse 语义」。
    pub fn pristine_snapshot(&self) -> Document {
        Document {
            nodes: self.nodes.clone(),
            root: self.root,
            id_map: self.id_map.clone(),
            url: None,
            referrer: None,
            encoding_label: self.encoding_label.clone(),
            scripting_enabled: self.scripting_enabled,
            observers: Vec::new(),
            pending_mutations: Vec::new(),
            event_listeners: Default::default(),
            shadow_roots: self.shadow_roots.clone(),
            template_contents_map: self.template_contents_map.clone(),
            slot_assignments: self.slot_assignments.clone(),
            focus_element: self.focus_element,
            modal_dialogs: self.modal_dialogs.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_html;

    /// 快照契约：树与 NodeId 保持、导航态复位、与活文档脱钩（快照后突变不进入
    /// 快照——S1 指纹缓存免疫脚本突变的核心性质）。
    #[test]
    fn snapshot_preserves_tree_and_node_ids_resets_navigation_state() {
        let mut doc = parse_html("<html><body><div id=\"a\"><span>t</span></div></body></html>");
        // 导航态由引擎注入（Parse 步 `set_url`/`set_referrer` 同款，R3169/R3176）。
        doc.set_url(Some("https://old.example/x".to_string()));
        doc.set_referrer(Some("https://ref.example/".to_string()));
        let count = doc.node_count();
        let orig_a = doc.get_element_by_id("a").expect("orig #a");

        let snap = doc.pristine_snapshot();
        assert_eq!(snap.node_count(), count, "节点表逐节点深拷");
        assert_eq!(snap.get_element_by_id("a"), Some(orig_a), "NodeId 保持有效");
        assert_eq!(snap.url(), None::<&str>, "导航态复位：url 由消费方重放");
        assert_eq!(snap.referrer(), None::<&str>, "导航态复位：referrer 由消费方重放");

        // 脱钩：快照后活文档突变不进入快照。
        let probe = doc.create_element("div");
        let a = doc.get_element_by_id("a").expect("#a");
        doc.append_child(a, probe).expect("append");
        assert_eq!(snap.node_count(), count, "快照节点数不受快照后突变影响");
        assert_eq!(
            snap.get(a).map(|n| n.children.len()),
            doc.get(a).map(|n| n.children.len() - 1),
            "快照树结构停在快照刻"
        );
    }
}
