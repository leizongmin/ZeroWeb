//! HTML drag source 判定——`draggable` enumerated 属性状态与默认可拖元素。
//!
//! 纯 DOM 逻辑，不依赖脚本引擎 feature：命中测试快照构建（[`crate::hit_test`]，core 面，
//! dnd-compat M3 拖拽源仲裁）与 native 反射（`dom_bindings`，v8 门控）共用同一语义源，
//! 防双实现分叉。

use zero_dom::{Document, NodeId, NodeKind};

/// auto 状态 default-draggable 判定（spec HTML `draggable`：缺省/invalid → auto 状态）。
///
/// auto 状态下元素的拖拽性由 UA 默认行为决定——spec/Chrome：`img`/`audio`/`video` 默认可拖拽，
/// `a`（带 `href`）默认可拖拽，余默认不可拖拽。headless 简化：覆盖这组明确 default-draggable 元素。
/// https://html.spec.whatwg.org/multipage/interaction.html#the-draggable-attribute
pub(crate) fn default_draggable(doc: &Document, id: NodeId) -> bool {
    let Some(node) = doc.get(id) else {
        return false;
    };
    let NodeKind::Element(e) = &node.kind else {
        return false;
    };
    match e.tag_name().as_bytes() {
        b"IMG" | b"AUDIO" | b"VIDEO" => true,
        // `a` 仅当带 href 时 default-draggable（spec：a with an href content attribute）。
        b"A" => doc.get_attribute(id, "href").is_some(),
        _ => false,
    }
}

/// 元素 `draggable` IDL 状态（spec HTML `draggable`，enumerated content 反射 boolean）：
/// 关键字 true/false ASCII case-insensitive（`draggable="TRUE"` → true 状态）；
/// 缺省/非法 → auto 状态 → [`default_draggable`]。
/// https://html.spec.whatwg.org/multipage/interaction.html#the-draggable-attribute
pub(crate) fn element_draggable(doc: &Document, id: NodeId) -> bool {
    match doc.get_attribute(id, "draggable").as_deref() {
        Some(v) if v.eq_ignore_ascii_case("true") => true,
        Some(v) if v.eq_ignore_ascii_case("false") => false,
        // 缺省 / 非法 / 其它 → auto 状态：元素 default-draggable（img/a[href]/audio/video）。
        _ => default_draggable(doc, id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draggable_enumerated_attribute_states() {
        // 显式 true/false 关键字（ASCII case-insensitive）。
        let doc = zero_dom::parse_html(
            r#"<body><div id="t" draggable="TRUE"></div><div id="f" draggable="false"></div></body>"#,
        );
        let t = doc.get_element_by_id("t").expect("t");
        let f = doc.get_element_by_id("f").expect("f");
        assert!(
            element_draggable(&doc, t),
            "draggable=TRUE → true（ASCII case-insensitive）"
        );
        assert!(!element_draggable(&doc, f), "draggable=false → false");
    }

    #[test]
    fn draggable_auto_state_resolves_defaults() {
        // 缺省 → auto 状态：img/a[href]/audio/video 默认可拖，div/无 href 的 a 不可拖。
        let doc = zero_dom::parse_html(
            r#"<body><img id="i"><audio id="au"></audio><video id="v"></video><a id="l" href="/x">l</a><a id="n">n</a><div id="d"></div><div id="bad" draggable="maybe"></div></body>"#,
        );
        for (id, want) in [
            ("i", true),
            ("au", true),
            ("v", true),
            ("l", true),
            ("n", false),
            ("d", false),
        ] {
            let el = doc.get_element_by_id(id).expect(id);
            assert_eq!(element_draggable(&doc, el), want, "auto 状态默认可拖判定：#{id}");
        }
        // 非法关键字 → auto 状态（同缺省）。
        let bad = doc.get_element_by_id("bad").expect("bad");
        assert!(!element_draggable(&doc, bad), "draggable=maybe → auto → div 默认不可拖");
        // img 显式 draggable=false 压过默认可拖。
        let doc2 = zero_dom::parse_html(r#"<body><img id="i2" draggable="false"></body>"#);
        let i2 = doc2.get_element_by_id("i2").expect("i2");
        assert!(!element_draggable(&doc2, i2), "显式 false 压过 img 默认可拖");
    }
}
