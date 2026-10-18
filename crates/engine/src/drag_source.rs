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

/// UA 拖拽手势屏障：文本控件（text 类 `input`/`textarea`）、`select` 与
/// `contenteditable` true 状态元素上，按压-拖动手势被文本选择/控件交互消费，
/// 不得作为元素拖拽起点——自身不可起拖（即使显式 `draggable=true`），且**阻断向
/// 可拖祖先上溯**（corpus `interactiveelements/*_inside_draggable_element`：可拖父级
/// 不得因屏障内元素的按压-move 起拖）。
///
/// spec 依据：processing model 只规定「沿祖先链取首个 IDL `draggable` true 元素」，
/// 编辑区/文本控件的例外属 UA 手势语义（Chromium 行为口径，`Node::CanStartDrag`
/// 编辑/禁用链划界同构）；语料验收 `html/editing/dnd/interactiveelements/`。
/// input 文本态集合 = spec Text/Search/Tel/URL/Email/Password + Number（缺省/未识别
/// type → Text 状态）；`contenteditable` 取 true 关键字或空串（true 状态）。
/// FIXME(dnd-compat)：继承可编辑区内的非编辑子元素（编辑宿主后代）未屏障——corpus
/// 未覆盖，Chromium 语义按编辑样式链判定，待语料证据再收。
/// https://html.spec.whatwg.org/multipage/dnd.html#drag-and-drop-processing-model
pub(crate) fn drag_gesture_barrier(doc: &Document, id: NodeId) -> bool {
    let Some(node) = doc.get(id) else {
        return false;
    };
    let NodeKind::Element(e) = &node.kind else {
        return false;
    };
    match e.tag_name().as_bytes() {
        b"TEXTAREA" | b"SELECT" => true,
        b"INPUT" => {
            let itype = doc.get_attribute(id, "type").unwrap_or_default();
            // 仅非文本入口不设屏障（corpus：input button/reset/submit/image 默认可拖）。
            matches!(
                itype.to_ascii_lowercase().as_str(),
                "" | "text" | "search" | "tel" | "url" | "email" | "password" | "number"
            )
        }
        _ => doc
            .get_attribute(id, "contenteditable")
            .is_some_and(|v| v.is_empty() || v.eq_ignore_ascii_case("true")),
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

    /// 手势屏障矩阵：文本控件（text 类 input/textarea/select/contenteditable）设屏障；
    /// 按钮类 input 与普通元素不设。显式 `draggable=true` 不豁免（corpus 口径）。
    #[test]
    fn drag_gesture_barrier_matrix() {
        let doc = zero_dom::parse_html(
            r#"<body>
                <input id="t" type="text" draggable="true">
                <input id="t2">
                <input id="pw" type="PASSWORD">
                <input id="num" type="number">
                <textarea id="ta" draggable="true"></textarea>
                <select id="sel" draggable="true"></select>
                <div id="ce" contenteditable="true" draggable="true"></div>
                <div id="ce-empty" contenteditable="" draggable="true"></div>
                <button id="btn" draggable="true">b</button>
                <input id="btn2" type="button" draggable="true">
                <input id="img" type="image" draggable="true">
                <input id="chk" type="checkbox">
                <div id="plain"></div>
                <div id="ce-false" contenteditable="false"></div>
            </body>"#,
        );
        for (id, want) in [
            ("t", true),
            ("t2", true), // type 缺省 → Text 状态
            ("pw", true), // ASCII case-insensitive
            ("num", true),
            ("ta", true),
            ("sel", true),
            ("ce", true),
            ("ce-empty", true), // contenteditable="" = true 状态
            ("btn", false),
            ("btn2", false),
            ("img", false),
            ("chk", false),
            ("plain", false),
            ("ce-false", false), // false 状态非编辑宿主
        ] {
            let el = doc.get_element_by_id(id).expect(id);
            assert_eq!(drag_gesture_barrier(&doc, el), want, "屏障判定：#{id}");
        }
    }
}
