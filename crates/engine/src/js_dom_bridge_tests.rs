use super::*;

#[test]
fn test_apply_set_attr_src() {
    let html = "<html><body><img id=\"i\" src=\"old.png\"></body></html>";
    let mutations = vec![DomMutation::SetAttr {
        selector: "#i".into(),
        name: "src".into(),
        value: "new.png".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("src=\"new.png\""));
}

#[test]
fn test_apply_set_style() {
    let html = "<html><body><div id=\"d\"></div></body></html>";
    let mutations = vec![DomMutation::SetStyle {
        selector: "#d".into(),
        property: "color".into(),
        value: "red".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("color: red") || out.contains("color:red"));
}

#[test]
fn test_apply_class_name_via_set_attr() {
    let html = "<html><body><div id=\"d\"></div></body></html>";
    let mutations = vec![DomMutation::SetAttr {
        selector: "#d".into(),
        name: "class".into(),
        value: "active".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("class=\"active\""));
}

#[test]
fn test_apply_create_and_append() {
    let html = "<html><body id=\"b\"></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "p".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "p1".into(),
        },
        DomMutation::CreateTextNode {
            handle: "__n2".into(),
            text: "hello".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n1".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#p1".into(),
            child_handle: "__n2".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("<p id=\"p1\">hello</p>"));
}

#[test]
fn test_apply_create_document_fragment_flatten() {
    // createDocumentFragment + 建 2 子 + AppendFragmentChildren → 子节点 flatten 到目标，
    // fragment 自身不入树。
    let html = "<html><body><ul id=\"list\"></ul></body></html>";
    let mutations = vec![
        DomMutation::CreateDocumentFragment { handle: "__f".into() },
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "li".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "a".into(),
        },
        DomMutation::CreateElement {
            handle: "__n2".into(),
            tag: "li".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n2".into(),
            name: "id".into(),
            value: "b".into(),
        },
        // 建 fragment 子树。
        DomMutation::AppendChildByHandle {
            parent_handle: "__f".into(),
            child_handle: "__n1".into(),
        },
        DomMutation::AppendChildByHandle {
            parent_handle: "__f".into(),
            child_handle: "__n2".into(),
        },
        // flatten：fragment 子移到 #list。
        DomMutation::AppendFragmentChildren {
            parent_selector: "#list".into(),
            fragment_handle: "__f".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    let ia = out.find("<li id=\"a\">").unwrap();
    let ib = out.find("<li id=\"b\">").unwrap();
    assert!(ia < ib, "flatten 后两 li 应按序在 #list 内\n{out}");
    // fragment 应不出现（detached，未入树）。
    assert!(!out.contains("document-fragment"), "fragment 自身不应入序列化树\n{out}");
}

#[test]
fn test_apply_insert_adjacent_html_beforeend() {
    // beforeend：片段作为目标元素末子追加（多节点保持顺序）。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentHtml {
        selector: "#t".into(),
        position: "beforeend".into(),
        html: "<b>1</b><b>2</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    let i1 = out.find("<b>1</b>").unwrap();
    let i2 = out.find("<b>2</b>").unwrap();
    // 原文本 x 紧邻首个追加节点（x 不再是末子，故检查 x<b>1</b> 紧邻而非 x</div>）。
    assert!(out.contains("x<b>1</b>"), "beforeend: 原文本 x 应在追加节点之前\n{out}");
    assert!(i1 < i2, "beforeend: 片段节点保持顺序\n{out}");
}

#[test]
fn test_apply_insert_adjacent_html_afterbegin() {
    // afterbegin：片段作为目标元素首子插入，原有子节点后移、身份不变。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentHtml {
        selector: "#t".into(),
        position: "afterbegin".into(),
        html: "<b>1</b><b>2</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    let i1 = out.find("<b>1</b>").unwrap();
    let i2 = out.find("<b>2</b>").unwrap();
    let ix = out.find("x</div>").unwrap();
    assert!(i1 < i2 && i2 < ix, "afterbegin: 片段在前、原文本 x 在后\n{out}");
}

#[test]
fn test_apply_insert_adjacent_html_beforebegin() {
    // beforebegin：片段作为目标元素的前兄弟插入到父节点。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentHtml {
        selector: "#t".into(),
        position: "beforebegin".into(),
        html: "<b>1</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    let i_b = out.find("<b>1</b>").unwrap();
    let i_t = out.find("<div id=\"t\">").unwrap();
    assert!(i_b < i_t, "beforebegin: 片段应在目标元素之前\n{out}");
}

#[test]
fn test_apply_insert_adjacent_html_afterend_last_child() {
    // afterend 且目标为父节点末子：片段 append 到父节点（末位）。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentHtml {
        selector: "#t".into(),
        position: "afterend".into(),
        html: "<b>1</b><b>2</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    let i_t = out.find("<div id=\"t\">").unwrap();
    let i1 = out.find("<b>1</b>").unwrap();
    let i2 = out.find("<b>2</b>").unwrap();
    assert!(i_t < i1 && i1 < i2, "afterend(末子): 目标在前、片段在后\n{out}");
}

#[test]
fn test_apply_insert_adjacent_html_afterend_with_next_sibling() {
    // afterend 且目标有后继兄弟：片段插到后继兄弟之前。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div><i>after</i></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentHtml {
        selector: "#t".into(),
        position: "afterend".into(),
        html: "<b>1</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    let i_t = out.find("id=\"t\"").unwrap();
    let i_b = out.find("<b>1</b>").unwrap();
    let i_after = out.find("<i>after</i>").unwrap();
    assert!(
        i_t < i_b && i_b < i_after,
        "afterend(有后继): 目标 < 片段 < 后继兄弟\n{out}"
    );
}

#[test]
fn test_apply_insert_adjacent_html_text_only_fragment() {
    // 纯文本片段（无 <）：作为单 Text 节点插入。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentHtml {
        selector: "#t".into(),
        position: "beforeend".into(),
        html: "hi".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("xhi</div>"), "纯文本片段应追加为文本节点\n{out}");
}

#[test]
fn test_apply_insert_adjacent_html_invalid_position() {
    // 非法 position → apply 返错。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentHtml {
        selector: "#t".into(),
        position: "nowhere".into(),
        html: "<b>1</b>".into(),
    }];
    let err = apply_mutations_to_html(html, &mutations).unwrap_err();
    assert!(err.contains("invalid position"), "非法 position 应返错，实际：{err}");
}

#[test]
fn test_apply_insert_adjacent_html_nested_subtree() {
    // 片段含嵌套子树：深拷贝保留完整结构。
    let html = "<html><body id=\"b\"><ul id=\"list\"></ul></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentHtml {
        selector: "#list".into(),
        position: "beforeend".into(),
        html: "<li>a<span>x</span></li>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("<li>a<span>x</span></li>"), "嵌套子树应完整深拷贝\n{out}");
}

#[test]
fn test_query_outer_html_serialization() {
    // outerHTML getter：含自身 tag/属性 + 子树序列化。
    let html = "<html><body><div id=\"t\" class=\"c\">hi<span>x</span></div></body></html>";
    let outer = query_outer_html_from_html(html, "#t");
    assert!(outer.contains("<div"), "outerHTML 含自身 tag\n{outer}");
    assert!(outer.contains("id=\"t\""), "outerHTML 含属性\n{outer}");
    assert!(outer.contains("hi<span>x</span>"), "outerHTML 含子树\n{outer}");
}

#[test]
fn test_child_nodes_json_mixed() {
    // child_nodes_json：含文本/元素/注释节点，JSON 序列化（文本内容含任意字符安全）。
    let html = "<html><body><div id=\"t\">text1<span id=\"s\">x</span><!--c-->text2</div></body></html>";
    let json = child_nodes_json(html, "#t");
    assert!(json.contains("\"k\":\"T\",\"v\":\"text1\""), "首个子为 text1\n{json}");
    assert!(json.contains("\"k\":\"E\",\"s\":\"#s\""), "元素子 span → #s\n{json}");
    assert!(json.contains("\"k\":\"C\",\"v\":\"c\""), "注释子 c\n{json}");
    assert!(json.contains("\"k\":\"T\",\"v\":\"text2\""), "末子 text2\n{json}");
    // 计数：4 个顶层条目（text/span/comment/text）。
    assert_eq!(json.matches("\"k\":").count(), 4, "应含 4 个子节点条目\n{json}");
}

#[test]
fn test_child_nodes_json_escape() {
    // 文本含特殊字符（引号/反斜杠）须 JSON 转义。文本 `a"b\c` → JSON 值 `a\"b\\c`。
    let html = "<html><body><div id=\"t\">a\"b\\c</div></body></html>";
    let json = child_nodes_json(html, "#t");
    assert!(
        json.contains("a\\\"b\\\\c"),
        "文本 a\"b\\c 应 JSON 转义为 a\\\"b\\\\c\n{json}"
    );
}

#[test]
fn test_sibling_nodes_json_across_text() {
    // sibling_nodes_json：span 的前兄弟=text1、后兄弟=注释 c（含非元素节点）。
    let html = "<html><body><div id=\"t\">text1<span id=\"s\">x</span><!--c-->text2</div></body></html>";
    let json = sibling_nodes_json(html, "#s");
    assert!(
        json.contains("\"p\":{\"k\":\"T\",\"v\":\"text1\"}"),
        "前兄弟=text1\n{json}"
    );
    assert!(
        json.contains("\"n\":{\"k\":\"C\",\"v\":\"c\"}"),
        "后兄弟=comment c\n{json}"
    );
}

#[test]
fn test_parent_selector_for_nested() {
    // parent_selector_for：嵌套元素返真实元素父（修正旧 stub 恒返 body）。
    let html = "<html><body><div id=\"outer\"><div id=\"inner\">x</div></div></body></html>";
    assert_eq!(parent_selector_for(html, "#inner"), "#outer", "inner 父 = #outer");
    assert_eq!(parent_selector_for(html, "#outer"), "body", "outer 父 = body");
    // html 根无元素父 → 空串。
    assert_eq!(parent_selector_for(html, "html"), "", "html 根无元素父");
    // 不解析的选择器 → 空串。
    assert_eq!(parent_selector_for(html, "#nope"), "", "未命中 → 空串");
}

#[test]
fn test_query_tag_from_mutations() {
    // query_tag_from_mutations：从 CreateElement 记录取真实 tag（detached 元素无 selector，
    // shim _tagFromSel 恒猜 DIV）。命中 → tag；非该句柄 → 空串。
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "span".into(),
        },
        DomMutation::CreateElement {
            handle: "__n2".into(),
            tag: "tr".into(),
        },
    ];
    assert_eq!(query_tag_from_mutations(&mutations, "__n1"), "span", "命中 __n1 → span");
    assert_eq!(query_tag_from_mutations(&mutations, "__n2"), "tr", "命中 __n2 → tr");
    assert_eq!(query_tag_from_mutations(&mutations, "__nope"), "", "未记录句柄 → 空串");
}

#[test]
fn test_apply_set_outer_html_replaces_element() {
    // outerHTML setter：目标元素整体替换为解析片段，兄弟位置保留。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div><i>after</i></body></html>";
    let mutations = vec![DomMutation::SetOuterHtml {
        selector: "#t".into(),
        html: "<b>1</b><b>2</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(!out.contains("<div id=\"t\">"), "outerHTML setter 应移除原元素\n{out}");
    let i1 = out.find("<b>1</b>").unwrap();
    let i2 = out.find("<b>2</b>").unwrap();
    let i_after = out.find("<i>after</i>").unwrap();
    assert!(
        i1 < i2 && i2 < i_after,
        "替换片段应保持顺序且位于原位置（兄弟 after 之前）\n{out}"
    );
}

#[test]
fn test_apply_set_outer_html_empty_removes() {
    // outerHTML = ''：仅移除元素（spec），兄弟保留。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div><i>keep</i></body></html>";
    let mutations = vec![DomMutation::SetOuterHtml {
        selector: "#t".into(),
        html: "".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(!out.contains("<div id=\"t\">"), "空片段应移除元素\n{out}");
    assert!(out.contains("<i>keep</i>"), "兄弟应保留\n{out}");
}

#[test]
fn test_apply_set_outer_html_text_fragment() {
    // outerHTML = 纯文本：替换为单 Text 节点。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::SetOuterHtml {
        selector: "#t".into(),
        html: "just-text".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("just-text"), "纯文本片段应替换为 Text 节点\n{out}");
    assert!(!out.contains("<div id=\"t\">"), "原元素应被移除\n{out}");
}

#[test]
fn test_apply_set_outer_html_nested_fragment() {
    // outerHTML setter 片段含嵌套子树：完整深拷贝。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::SetOuterHtml {
        selector: "#t".into(),
        html: "<section><p>a<span>y</span></p></section>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(
        out.contains("<section><p>a<span>y</span></p></section>"),
        "嵌套片段应完整深拷贝替换\n{out}"
    );
    assert!(!out.contains("<div id=\"t\">"), "原元素应被移除\n{out}");
}

#[test]
fn test_apply_insert_adjacent_text_literal() {
    // insertAdjacentText：文本作**字面 Text 节点**（不解析 HTML）——含 `<` 的文本应转义
    // 为 `&lt;...&gt;` 而非解析成元素。beforeend 追加到目标。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentText {
        selector: "#t".into(),
        position: "beforeend".into(),
        text: "<b>not-an-element</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(
        out.contains("&lt;b&gt;not-an-element&lt;/b&gt;"),
        "insertAdjacentText 应转义 HTML 字符（字面文本，不解析）\n{out}"
    );
    assert!(
        !out.contains("<b>not-an-element</b>"),
        "insertAdjacentText 不应把文本解析成元素\n{out}"
    );
}

#[test]
fn test_apply_insert_adjacent_text_beforebegin_sibling() {
    // insertAdjacentText beforebegin：文本作前兄弟插入到父节点。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![DomMutation::InsertAdjacentText {
        selector: "#t".into(),
        position: "beforebegin".into(),
        text: "PRE".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    let i_pre = out.find("PRE").unwrap();
    let i_t = out.find("<div id=\"t\">").unwrap();
    assert!(i_pre < i_t, "beforebegin: 文本应在目标之前\n{out}");
}

#[test]
fn test_apply_insert_adjacent_element_beforeend_child() {
    // insertAdjacentElement beforeend：create 句柄元素作为目标末子插入。
    let html = "<html><body id=\"b\"><div id=\"t\"></div></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "span".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "moved".into(),
        },
        DomMutation::InsertAdjacentElement {
            selector: "#t".into(),
            position: "beforeend".into(),
            child_handle: "__n1".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(
        out.contains("<div id=\"t\"><span id=\"moved\"></span></div>"),
        "beforeend: 元素应作末子\n{out}"
    );
}

#[test]
fn test_apply_insert_adjacent_element_reparents() {
    // insertAdjacentElement 移动语义：__n1 先挂到 #b，再 insertAdjacentElement 到 #t beforeend
    // → 应从 #b 移除、成为 #t 末子（append_child 自动 reparent）。
    let html = "<html><body id=\"b\"><div id=\"t\"></div></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "span".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "moved".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n1".into(),
        },
        // 移动 __n1 从 #b → #t beforeend。
        DomMutation::InsertAdjacentElement {
            selector: "#t".into(),
            position: "beforeend".into(),
            child_handle: "__n1".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    // span 现在是 #t 的子（移动），#b 的直接元素子应只剩 #t（span 不再是 #b 子）。
    assert!(
        out.contains("<div id=\"t\"><span id=\"moved\"></span></div>"),
        "reparent: span 应移到 #t 内\n{out}"
    );
    // span 不应出现在 #t 之外（即不应是 #b 的直接子）。
    let i_t_close = out.find("<div id=\"t\"><span id=\"moved\"></span></div>").unwrap();
    let i_span = out.find("<span id=\"moved\"></span>").unwrap();
    assert_eq!(
        i_span,
        i_t_close + "<div id=\"t\">".len(),
        "reparent: span 应唯一出现在 #t 内\n{out}"
    );
}

#[test]
fn test_apply_insert_adjacent_element_afterend_sibling() {
    // insertAdjacentElement afterend：create 元素作目标后兄弟插入到父节点。
    let html = "<html><body id=\"b\"><div id=\"t\">x</div></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "span".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "after".into(),
        },
        DomMutation::InsertAdjacentElement {
            selector: "#t".into(),
            position: "afterend".into(),
            child_handle: "__n1".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    let i_t = out.find("<div id=\"t\">").unwrap();
    let i_span = out.find("<span id=\"after\"></span>").unwrap();
    assert!(i_t < i_span, "afterend: 元素应在目标之后\n{out}");
}

#[test]
fn test_insert_adjacent_text_and_element_e2e() {
    // 端到端：insertAdjacentText/Element JS 契约——调用入队对应 mutation。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='t'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // insertAdjacentText：入队 InsertAdjacentText（含字面 < 不解析，position/text 透传）。
    sandbox
        .execute("document.querySelector('#t').insertAdjacentText('beforeend', '<b>');")
        .unwrap();
    // insertAdjacentElement：create 元素 + 移动插入，返元素本身（非 null）。
    sandbox
        .execute(
            "globalThis.__r = document.querySelector('#t').insertAdjacentElement('afterbegin', document.createElement('span'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__r === null").unwrap().value, "false");
    // 非节点参数 → 返 null（不抛）。
    sandbox
        .execute("globalThis.__r2 = document.querySelector('#t').insertAdjacentElement('beforeend', 'not-a-node');")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__r2").unwrap().value, "null");

    let ms = mutations.lock().unwrap();
    let text_mutation = ms
        .iter()
        .any(|m| matches!(m, DomMutation::InsertAdjacentText { text, .. } if text == "<b>"));
    assert!(
        text_mutation,
        "insertAdjacentText 应入队 InsertAdjacentText（text=<b> 透传）"
    );
    let elem_mutation = ms
        .iter()
        .any(|m| matches!(m, DomMutation::InsertAdjacentElement { position, .. } if position == "afterbegin"));
    assert!(
        elem_mutation,
        "insertAdjacentElement 应入队 InsertAdjacentElement（position=afterbegin）"
    );
}

#[test]
fn test_apply_with_handles_maps_unique_selectors() {
    // 创建带 id 的元素（唯一）→ 入 map；创建无 id 的同 tag 元素 + 文档已有同 tag → 歧义 → 不入 map。
    let html = "<html><body id=\"b\"><div></div></body></html>";
    let mutations = vec![
        // __n1: <p id="unique"> —— id 唯一 → "#unique"
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "p".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "unique".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n1".into(),
        },
        // __n2: <div>（无 id）—— 文档已有 1 个 div，加这个共 2 个 → "div" 歧义 → 不入 map
        DomMutation::CreateElement {
            handle: "__n2".into(),
            tag: "div".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n2".into(),
        },
        // __n3: <span>（无 id）—— 文档无其他 span → "span" 唯一 → 入 map
        DomMutation::CreateElement {
            handle: "__n3".into(),
            tag: "span".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n3".into(),
        },
    ];
    let (out, handles) = apply_mutations_to_html_with_handles(html, &mutations).unwrap();
    // 序列化正确
    assert!(out.contains("<p id=\"unique\">"));
    assert!(out.contains("<span>"));
    // handle→selector 映射：唯一者用短选择器，歧义者回落 nth-child 结构路径。
    assert_eq!(handles.get("__n1"), Some(&"#unique".to_string()), "id 唯一 → #unique");
    assert_eq!(handles.get("__n3"), Some(&"span".to_string()), "唯一 span → tag");
    let n2_sel = handles.get("__n2").expect("歧义 div 仍入 map（结构路径）");
    assert!(
        n2_sel.contains("nth-child"),
        "无 id 的歧义 div → nth-child 结构路径，got: {n2_sel}"
    );
    // 不变量：每个 selector 在 fresh-parse(序列化 html) 上都能解析（path A 依赖此）。
    let fresh = parse_html(&out);
    for sel in handles.values() {
        assert!(
            find_by_selector(&fresh, sel).is_some(),
            "selector {sel} 须在 fresh-parse 后可解析"
        );
    }
}

#[test]
fn test_structural_path_resolves_to_correct_node() {
    // 多个无 id/class 同 tag 元素：nth-child 结构路径唯一锁定每一个，且 round-trip 正确
    // （在 mutated doc 与 fresh-parse 序列化 html 上都解析到同一节点）。
    let html = "<html><body><ul><li>1</li><li>2</li><li>3</li></ul></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n9".into(),
            tag: "li".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "ul".into(),
            child_handle: "__n9".into(),
        },
    ];
    let (out, handles) = apply_mutations_to_html_with_handles(html, &mutations).unwrap();
    let sel = handles.get("__n9").expect("__n9 入 map");
    assert!(sel.contains("nth-child"), "应回落结构路径: {sel}");
    // 在 fresh-parse(out) 上解析 → 应是第 4 个 li（"4" 无文本，但节点存在且唯一）。
    let fresh = parse_html(&out);
    let resolved = find_by_selector(&fresh, sel).expect("结构路径须可解析");
    // resolved 应是 ul 的最后一个 li（第 4 个）。
    let all_li = fresh.query_selector_all(fresh.root(), "li");
    assert_eq!(all_li.last().copied(), Some(resolved), "结构路径解析到 append 的末 li");
}

#[test]
fn test_apply_with_handles_uses_post_mutation_state() {
    // id 在 CreateElement 之后、AppendChild 之前设置（同 batch）→ 末尾算选择器时 id 已生效。
    let html = "<html><body id=\"b\"></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "div".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "late".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n1".into(),
        },
    ];
    let (_out, handles) = apply_mutations_to_html_with_handles(html, &mutations).unwrap();
    assert_eq!(handles.get("__n1"), Some(&"#late".to_string()));
}

#[test]
fn test_find_all_selectors() {
    let html = "<html><body><p class=\"x\"></p><p class=\"x\"></p></body></html>";
    let doc = parse_html(html);
    let sels = find_all_selectors(&doc, "p.x");
    assert_eq!(sels.len(), 2);
}

#[test]
fn test_query_all_selector_list_unique() {
    // querySelectorAll 对歧义集合（无 id/class 的 option）每元素返**唯一**选择器（nth-child
    // 结构路径），而非 stable_selector（"option" 重复→全指向首个）。各 selector 互异且可解析。
    let html = "<html><body><select id='s'>\
                    <option value='a'>A</option>\
                    <option value='b' selected>B</option>\
                    <option value='c'>C</option>\
                    </select></body></html>";
    let list = query_all_selector_list(html, "#s option");
    let sels: Vec<&str> = list.split('|').collect();
    assert_eq!(sels.len(), 3, "应返回 3 个 option 的选择器");
    // 互异（唯一化后无重复）。
    let uniq: std::collections::HashSet<&str> = sels.iter().copied().collect();
    assert_eq!(uniq.len(), 3, "3 个 selector 应互异（非全 'option'）");
    // 每个 selector 在 fresh-parse 上可解析且指向不同 option。
    let doc = parse_html(html);
    let mut resolved = Vec::new();
    for s in &sels {
        let id = find_by_selector(&doc, s).expect("每个 selector 须可解析");
        resolved.push(option_value(&doc, id));
    }
    resolved.sort();
    assert_eq!(resolved, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
}

#[test]
fn test_element_matches_test_selector() {
    let html = "<html><body>\
                    <div id='outer' class='row'><div id='inner' class='cell active'>x</div></div>\
                    <span id='sib'>y</span>\
                    </body></html>";
    // 自身选择器 → find_by_selector 解析元素；test_sel 含组合器/伪类/属性运算符均支持。
    assert!(element_matches_test_selector(html, "#inner", ".cell.active"));
    assert!(element_matches_test_selector(html, "#inner", "div"));
    // 组合器：#inner 是 #outer > .cell。
    assert!(element_matches_test_selector(html, "#inner", "#outer > .cell"));
    // :has（R2669）+ 属性运算符（R2671）经 query_selector_all 全匹配集正确判定。
    assert!(element_matches_test_selector(html, "#outer", "div:has(.active)"));
    assert!(element_matches_test_selector(html, "#inner", "[class^=cell]"));
    // 不匹配。
    assert!(!element_matches_test_selector(html, "#inner", "span"));
    assert!(!element_matches_test_selector(html, "#sib", ".cell"));
    assert!(!element_matches_test_selector(html, "#inner", "#outer > span"));
    // elem_sel 不存在 → false。
    assert!(!element_matches_test_selector(html, "#nope", "div"));
}

#[test]
fn test_closest_matching_selector() {
    let html = "<html><body>\
                    <div id='outer' class='row'><section id='mid'><div id='inner' class='cell'>x</div></section></div>\
                    </body></html>";
    // 自身匹配 → 返自身唯一选择器。
    assert_eq!(closest_matching_selector(html, "#inner", ".cell"), "#inner");
    // 向上找最近祖先：#inner 的最近 .row 祖先 = #outer（跨 section）。
    assert_eq!(closest_matching_selector(html, "#inner", ".row"), "#outer");
    // 含组合器：找匹配 `section .cell` 的最近祖先（#inner 自身匹配）。
    assert_eq!(closest_matching_selector(html, "#inner", "section .cell"), "#inner");
    // 标签祖先：#inner 最近 section 祖先 = #mid。
    assert_eq!(closest_matching_selector(html, "#inner", "section"), "#mid");
    // body 也可作为 closest 目标。
    assert_eq!(closest_matching_selector(html, "#inner", "body"), "body");
    // 无匹配 → 空串。
    assert_eq!(closest_matching_selector(html, "#inner", ".nonexistent"), "");
    assert_eq!(closest_matching_selector(html, "#inner", "span"), "");
    // elem_sel 不存在 → 空串。
    assert_eq!(closest_matching_selector(html, "#nope", "div"), "");
}

#[test]
fn test_query_in_subtree_scoping() {
    // 两个容器各含 .item；container 之外也有 .item。元素子树作用域须仅返该容器后代。
    let html = "<html><body>\
                <div id='a'><span class='item'>a1</span><span class='item'>a2</span></div>\
                <div id='b'><span class='item'>b1</span></div>\
                <span class='item'>outside</span>\
                </body></html>";
    // query_match_in_subtree：#a 子树首个 .item = a1（不返 outside 或 b1）。
    let a_first = query_match_in_subtree(html, "#a", ".item");
    let doc = parse_html(html);
    let n = find_by_selector(&doc, &a_first).expect("须可解析");
    assert_eq!(doc.text_content(n), Some("a1".to_string()));
    // #b 子树首个 .item = b1。
    let b_first = query_match_in_subtree(html, "#b", ".item");
    let nb = find_by_selector(&doc, &b_first).expect("须可解析");
    assert_eq!(doc.text_content(nb), Some("b1".to_string()));
    // query_all_in_subtree：#a 子树全部 .item = a1,a2（2 个，不含 outside/b1）。
    let a_all = query_all_in_subtree(html, "#a", ".item");
    let sels: Vec<&str> = a_all.split('|').collect();
    assert_eq!(sels.len(), 2, "#a 子树应有 2 个 .item（不含外部）");
    let texts: Vec<String> = sels
        .iter()
        .map(|s| doc.text_content(find_by_selector(&doc, s).unwrap()).unwrap())
        .collect();
    assert_eq!(texts, vec!["a1".to_string(), "a2".to_string()]);
    // 子树无匹配 → 空串（#a 内无 .nonexistent）。
    assert_eq!(query_match_in_subtree(html, "#a", ".nonexistent"), "");
    assert_eq!(query_all_in_subtree(html, "#a", ".nonexistent"), "");
    // 含组合器：#a 子树内 `span.item` 仍命中（后代组合器在子树内求值）。
    assert!(!query_all_in_subtree(html, "#a", "span.item").is_empty());
    // elem_sel 不存在 → 空串。
    assert_eq!(query_match_in_subtree(html, "#nope", ".item"), "");
    assert_eq!(query_all_in_subtree(html, "#nope", ".item"), "");
}

#[test]
fn test_element_children_selectors() {
    // 元素子（跳过文本/注释），文档顺序；#b/#i 唯一 id → 唯一选择器即 "#b"/"#i"。
    let html = "<html><body>\
                <div id='p'>text<b id='b'>B</b> <i id='i'>I</i></div>\
                </body></html>";
    // 仅元素子 b/i（跳过文本与空白），文档顺序。
    assert_eq!(element_children_selectors(html, "#p"), "#b|#i");
    // 无元素子 → 空串。
    assert_eq!(element_children_selectors(html, "#b"), "");
    // elem_sel 不存在 → 空串。
    assert_eq!(element_children_selectors(html, "#nope"), "");
}

#[test]
fn test_element_sibling_selectors() {
    let html = "<html><body>\
                <div id='a'>A</div>text<div id='b'>B</div><div id='c'>C</div>\
                </body></html>";
    // #b 的前元素兄弟 = #a，后元素兄弟 = #c（跳过中间文本节点）。
    assert_eq!(element_sibling_selectors(html, "#b"), "#a|#c");
    // #a 首个 → 无前兄弟。
    assert_eq!(element_sibling_selectors(html, "#a"), "|#b");
    // #c 末个 → 无后兄弟。
    assert_eq!(element_sibling_selectors(html, "#c"), "#b|");
    // elem_sel 不存在 → 两空。
    assert_eq!(element_sibling_selectors(html, "#nope"), "|");
}

#[test]
fn test_element_contains() {
    let html = "<html><body>\
                <div id='outer'><section id='mid'><span id='inner'>x</span></section></div>\
                <div id='other'>y</div>\
                </body></html>";
    // 后代：outer 含 inner（深层）。
    assert!(element_contains(html, "#outer", "#inner"));
    // 自身：outer 含 outer。
    assert!(element_contains(html, "#outer", "#outer"));
    // 非后代：other 不在 outer 内。
    assert!(!element_contains(html, "#outer", "#other"));
    // 反向：inner 不含 outer。
    assert!(!element_contains(html, "#inner", "#outer"));
    // 容器/other 不存在 → false。
    assert!(!element_contains(html, "#nope", "#inner"));
    assert!(!element_contains(html, "#outer", "#nope"));
}

#[test]
fn test_element_attribute_names() {
    let html = "<html><body>\
                <div id='d' class='row' data-user-id='42' data-role='admin' aria-label='x'><p>t</p></div>\
                </body></html>";
    let names = element_attribute_names(html, "#d");
    let set: std::collections::HashSet<&str> = names.split('|').filter(|s| !s.is_empty()).collect();
    // 5 属性全列（id/class/data-user-id/data-role/aria-label）。
    assert_eq!(set.len(), 5, "应列全部 5 个属性");
    for expect in ["id", "class", "data-user-id", "data-role", "aria-label"] {
        assert!(set.contains(expect), "缺属性 {expect}");
    }
    // 无属性元素（<p>，经组合器定位）→ 空串。
    assert_eq!(element_attribute_names(html, "#d > p"), "");
    // 元素不存在 → 空串。
    assert_eq!(element_attribute_names(html, "#nope"), "");
}

#[test]
fn test_dataset_e2e() {
    // 端到端：注入生产 shim + register_dom_callbacks，验证 dataset JS 契约（camelCase↔kebab、
    // get/枚举/set-mutation）。set 记 mutation（apply 末尾），故验 mutation 而非同脚本回读（stale）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d' data-user-id='42' data-role='admin'>x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // get：data-user-id → dataset.userId（camelCase 键）。
    sandbox
        .execute("globalThis.__r = document.querySelector('#d').dataset.userId;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__r").unwrap().value, "42");
    // 不存在的键 → undefined。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').dataset.nope")
            .unwrap()
            .value,
        "undefined"
    );
    // 枚举：Object.keys → data-* 的 camelCase 键（userId,role）。
    sandbox
        .execute("globalThis.__k = Object.keys(document.querySelector('#d').dataset).join(',');")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__k").unwrap().value, "userId,role");
    // has：'userId' in dataset。
    assert_eq!(
        sandbox
            .execute("'userId' in document.querySelector('#d').dataset")
            .unwrap()
            .value,
        "true"
    );
    // set：dataset.newKey = 'x' → 记 SetAttr(data-new-key=x) mutation（camelCase→kebab）。
    sandbox
        .execute("document.querySelector('#d').dataset.newKey = 'x';")
        .unwrap();
    let set_val = mutations.lock().unwrap().iter().find_map(|m| match m {
        DomMutation::SetAttr { name, value, .. } if name == "data-new-key" => Some(value.clone()),
        _ => None,
    });
    assert_eq!(
        set_val.as_deref(),
        Some("x"),
        "dataset.newKey=x 应记 SetAttr data-new-key=x"
    );
}

#[test]
fn test_boolean_reflected_property_e2e() {
    // 端到端：boolean reflected property（hidden/checked/disabled/selected）——getter 属性存在性，
    // setter truthy→设存在 / falsy→真移除（修正旧 fallthrough 写空串致 falsy 仍 present 的 bug）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><input id='cb' type='checkbox' checked><input id='cb2' type='checkbox'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // getter：预置 checked → true；无 checked → false；无 hidden → false（presence，非 stale）。
    assert_eq!(
        sandbox.execute("document.querySelector('#cb').checked").unwrap().value,
        "true"
    );
    assert_eq!(
        sandbox.execute("document.querySelector('#cb2').checked").unwrap().value,
        "false"
    );
    assert_eq!(
        sandbox.execute("document.querySelector('#cb').hidden").unwrap().value,
        "false"
    );

    // setter truthy：cb2.checked = true → 记 SetAttr(checked='')（presence）。
    sandbox
        .execute("document.querySelector('#cb2').checked = true;")
        .unwrap();
    // setter falsy：cb.checked = false → 记 RemoveAttr(checked)（真移除，修正 bug）。
    sandbox
        .execute("document.querySelector('#cb').checked = false;")
        .unwrap();
    // hidden setter truthy → SetAttr(hidden='')。
    sandbox.execute("document.querySelector('#cb').hidden = true;").unwrap();

    let ms = mutations.lock().unwrap();
    let has_set = |sel: &str, name: &str| {
        ms.iter().any(|m| match m {
            DomMutation::SetAttr { selector, name: n, .. } => selector == sel && n == name,
            _ => false,
        })
    };
    let has_rem = |sel: &str, name: &str| {
        ms.iter().any(|m| match m {
            DomMutation::RemoveAttr { selector, name: n } => selector == sel && n == name,
            _ => false,
        })
    };
    assert!(has_set("#cb2", "checked"), "cb2.checked=true 应记 SetAttr(checked)");
    assert!(
        has_rem("#cb", "checked"),
        "cb.checked=false 应记 RemoveAttr(checked)（修正 bug）"
    );
    assert!(has_set("#cb", "hidden"), "cb.hidden=true 应记 SetAttr(hidden)");
}

#[test]
fn test_layout_geometry_e2e() {
    // 端到端：offsetWidth/offsetHeight/clientWidth/offsetTop/offsetLeft 从 rect 派生。
    // rect bridge 不在 register_dom_callbacks，测试注册 mock __zw_getBoundingClientRect。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id='d'>x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    // mock rect bridge：selector → 固定 rect "10,20,100,50"；handle（createElement，以 '__' 开头，
    // detached）→ 空串（无 rect，匹配真实 detached 元素无布局几何语义）。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| match args.first() {
            Some(s) if s.starts_with("__") => String::new(),
            _ => "10,20,100,50".to_string(),
        }),
    );

    // offsetWidth/Height = rect border-box w/h（精确）。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').offsetWidth")
            .unwrap()
            .value,
        "100"
    );
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').offsetHeight")
            .unwrap()
            .value,
        "50"
    );
    // clientWidth/Height ≈ offset（content-box 近似）。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').clientWidth")
            .unwrap()
            .value,
        "100"
    );
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').clientHeight")
            .unwrap()
            .value,
        "50"
    );
    // offsetTop/Left = rect y/x（viewport 相对，近似）。
    assert_eq!(
        sandbox.execute("document.querySelector('#d').offsetTop").unwrap().value,
        "20"
    );
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').offsetLeft")
            .unwrap()
            .value,
        "10"
    );
    // visibility 检查（修旧 undefined>0=false bug）。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').offsetWidth > 0")
            .unwrap()
            .value,
        "true"
    );
    // scrollWidth/scrollHeight ≈ client 尺寸（无 overflow 数据，近似）。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').scrollWidth")
            .unwrap()
            .value,
        "100"
    );
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').scrollHeight")
            .unwrap()
            .value,
        "50"
    );
    // scrollTop/scrollLeft：无滚动状态 → 恒 0。
    assert_eq!(
        sandbox.execute("document.querySelector('#d').scrollTop").unwrap().value,
        "0"
    );
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').scrollLeft")
            .unwrap()
            .value,
        "0"
    );
    // offsetParent：有 rect（已渲染）→ 非 null（body proxy 近似），匹配 `=== null` 可见性判定。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#d').offsetParent !== null")
            .unwrap()
            .value,
        "true"
    );
    // createElement 元素（无 rect）→ offsetParent = null（detached 语义）。
    assert_eq!(
        sandbox
            .execute("document.createElement('div').offsetParent === null")
            .unwrap()
            .value,
        "true"
    );
}

#[test]
fn test_request_idle_callback_e2e() {
    // requestIdleCallback 无 host __zw_setTimeout → 走 _defer fallback（微任务，execute 末尾 drain）。
    // 回调传 IdleDeadline（timeRemaining 近似 50）。cancelIdleCallback 不抛。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__ric_ran = false;\
             requestIdleCallback(function(d){ globalThis.__ric_ran = true; globalThis.__ric_tr = d.timeRemaining(); });",
        )
        .unwrap();
    // _defer 在上一 execute 末尾 microtask checkpoint drain → 回调已运行。
    assert_eq!(sandbox.execute("globalThis.__ric_ran").unwrap().value, "true");
    assert_eq!(sandbox.execute("globalThis.__ric_tr").unwrap().value, "50");
    // 返 handle 为 number；cancelIdleCallback 不抛。
    assert_eq!(
        sandbox
            .execute("typeof requestIdleCallback(function(){})")
            .unwrap()
            .value,
        "number"
    );
    assert_eq!(sandbox.execute("cancelIdleCallback(1), 'ok'").unwrap().value, "ok");
}

#[test]
fn test_clone_node_e2e() {
    // cloneNode(deep) 复用既有回调组合：create(tag) + 逐属性 set_attr_handle + (deep) set_inner_html_handle。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='src' class='row' data-x='1'><span>child</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // deep clone → 记 CreateElement + 复制源全部属性 + SetInnerHtmlOnHandle。
    sandbox
        .execute("document.querySelector('#src').cloneNode(true);")
        .unwrap();
    let ms = mutations.lock().unwrap();
    // CreateElement(tag=div)。
    let created_tag = ms.iter().find_map(|m| match m {
        DomMutation::CreateElement { tag, .. } => Some(tag.clone()),
        _ => None,
    });
    assert_eq!(created_tag.as_deref(), Some("div"), "cloneNode 应 CreateElement(div)");
    // SetAttrOnHandle 复制源全部 3 属性（id/class/data-x，含值）。
    let has_attr = |name: &str, value: &str| {
        ms.iter().any(|m| match m {
            DomMutation::SetAttrOnHandle { name: n, value: v, .. } => n == name && v == value,
            _ => false,
        })
    };
    assert!(has_attr("id", "src"), "应复制 id=src");
    assert!(has_attr("class", "row"), "应复制 class=row");
    assert!(has_attr("data-x", "1"), "应复制 data-x=1");
    // deep：SetInnerHtmlOnHandle 含源后代 <span>child</span>。
    let deep = ms.iter().any(|m| match m {
        DomMutation::SetInnerHtmlOnHandle { html, .. } => html.contains("<span>child</span>"),
        _ => false,
    });
    assert!(deep, "deep clone 应 SetInnerHtmlOnHandle 含源后代");
}

#[test]
fn test_collect_element_ids_dedup_preserve_order() {
    let html = "<html><body>\
                    <div id=\"container\"></div>\
                    <span id=\"target\"></span>\
                    <p id=\"container\"></p>\
                    <b></div>\
                    </body></html>";
    let ids = collect_element_ids(html);
    // 去重（首个 container 保留），保序，跳过无 id 元素。
    assert_eq!(ids, "container|target");
}

#[test]
fn test_collect_element_ids_empty() {
    let html = "<html><body><div></div><p class=\"x\"></p></body></html>";
    assert_eq!(collect_element_ids(html), "");
}

#[test]
fn test_apply_inner_html() {
    let html = "<html><body><div id=\"d\">old</div></body></html>";
    let mutations = vec![DomMutation::SetInnerHtml {
        selector: "#d".into(),
        html: "<b>new</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("<b>new</b>"));
}

#[test]
fn test_shim_not_empty() {
    assert!(generate_js_dom_shim().contains("__zw_set_attr"));
    assert!(generate_js_dom_shim().contains("addEventListener"));
}

#[test]
fn test_shim_async_resolve_callback_e2e() {
    // P1b S1（方案 A）端到端：注入**生产** DOM shim（含 __zwResolveCallback + pending 表），
    // 验证 V8Sandbox::resolve_async_callback 经 shim 的 JS 契约真实 resolve Promise。
    // 宿主回调同步返「回调 ID」，JS 建 pending Promise；Rust resolve 触发 .then。
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
    let config = SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    // 注入生产 shim（tab_js_worker.rs / js_worker.rs 同款）。
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.register_callback("__zw_start_async", Box::new(|args| format!("aid:{}", args[0])));
    sandbox
        .execute(
            "var id = __zw_start_async('99');
                 new Promise(function(resolve){ globalThis.__zw_pending[id] = resolve; })
                     .then(function(v){ globalThis.__result = v; });",
        )
        .unwrap();
    // resolve 前：Promise pending。
    let before = sandbox.execute("typeof globalThis.__result").unwrap();
    assert_eq!(before.value, "undefined");
    // Rust 异步完成 → resolve（shim 的 __zwResolveCallback 触发 + microtask drain）。
    sandbox.resolve_async_callback("aid:99", "resolved!");
    let after = sandbox.execute("globalThis.__result").unwrap();
    assert_eq!(after.value, "resolved!");
}

#[test]
fn test_shim_includes_runtime_stubs() {
    let shim = generate_js_dom_shim();
    assert!(shim.contains("globalThis.setTimeout"));
    assert!(shim.contains("globalThis.navigator"));
    assert!(shim.contains("attachEvent"));
    assert!(shim.contains("__zw_get_page_url"));
    assert!(shim.contains("globalThis.screen"));
    assert!(shim.contains("parentNode"));
    // P1b S1（方案 A）异步回调 resolve 通道 JS 侧契约。
    assert!(shim.contains("globalThis.__zwResolveCallback"));
    assert!(shim.contains("globalThis.__zw_pending"));
    // P1a select：<select>.value/selectedIndex getter + setter 经 host 回调。
    assert!(shim.contains("__zw_select_value"));
    assert!(shim.contains("__zw_select_index"));
    assert!(shim.contains("__zw_select_option"));
}

#[test]
fn test_shim_includes_modern_reftest_stubs() {
    // 现代动态 reftest 的 `requestAnimationFrame(() => …; takeScreenshot())` 模式
    // 要求这两个全局存在，否则 setup mutation 永不执行（R917 未捕获的 yield gap）。
    let shim = generate_js_dom_shim();
    assert!(shim.contains("globalThis.requestAnimationFrame"));
    assert!(shim.contains("globalThis.cancelAnimationFrame"));
    assert!(shim.contains("globalThis.takeScreenshot"));
    // `Element.append(...nodesOrStrings)` 现代 API（区别于 appendChild）。
    assert!(shim.contains("if (prop === 'append')"));
    // `getBoundingClientRect()` 方法必须返回零 DOMRect，否则调用抛 TypeError
    // 中断脚本，使其后的 mutation 丢失（120 reftest 文件用作 reflow 触发器）。
    assert!(shim.contains("if (prop === 'getBoundingClientRect')"));
    // HTML 规范 named access on window（`id="x"` → 全局 `x`，257 reftest 文件）。
    assert!(shim.contains("_installNamedAccess"));
    assert!(shim.contains("__zw_collect_ids"));
    // `createElementNS`（XHTML 命名空间 alias createElement；SVG OOS 不渲染但不中断）。
    assert!(shim.contains("createElementNS:"));
    // `getComputedStyle`：动态 reftest 常作「强制 reflow」触发器调用，缺失则抛
    // ReferenceError 中断脚本丢失后续 mutation。返空 CSSStyleDeclaration 桩不抛。
    assert!(shim.contains("globalThis.getComputedStyle"));
    assert!(shim.contains("getPropertyValue"));
}

#[test]
fn test_merge_style_property() {
    let merged = merge_style_property("color: blue", "width", "10px");
    assert!(merged.contains("color: blue"));
    assert!(merged.contains("width: 10px"));
    let replaced = merge_style_property(&merged, "color", "red");
    assert!(!replaced.contains("blue"));
    assert!(replaced.contains("color: red"));
}

#[test]
fn test_enclosing_form_selector() {
    // P1a form submit：input 在 form 内 → 返 form 的 stable selector。
    let html = "<html><body><form id='f'><input id='i'></form></body></html>";
    assert_eq!(enclosing_form_selector(html, "#i").as_deref(), Some("#f"));
    // input 无 enclosing form → None。
    let no_form = "<html><body><div><input id='i'></div></body></html>";
    assert_eq!(enclosing_form_selector(no_form, "#i"), None);
    // 嵌套：input 在 form 内的 div 内 → 仍解析到 form。
    let nested = "<html><body><form id='outer'><div><input id='deep'></div></form></body></html>";
    assert_eq!(enclosing_form_selector(nested, "#deep").as_deref(), Some("#outer"));
    // 未命中 selector → None。
    assert_eq!(enclosing_form_selector(html, "#missing"), None);
}

#[test]
fn test_is_submit_button() {
    // P1a form submit：submit-button 判定。
    assert!(is_submit_button(
        "<html><body><form><input id='b' type='submit'></form></body></html>",
        "#b",
    ));
    assert!(is_submit_button(
        "<html><body><form><input id='i' type='image'></form></body></html>",
        "#i",
    ));
    // button 默认 type=submit → 提交。
    assert!(is_submit_button(
        "<html><body><form><button id='btn'>Go</button></form></body></html>",
        "#btn",
    ));
    assert!(is_submit_button(
        "<html><body><form><button id='s' type='submit'>Go</button></form></body></html>",
        "#s",
    ));
    // 非提交：
    assert!(!is_submit_button(
        "<html><body><form><input id='t' type='text'></form></body></html>",
        "#t",
    ));
    assert!(!is_submit_button(
        "<html><body><form><button id='nb' type='button'>Go</button></form></body></html>",
        "#nb",
    ));
    assert!(!is_submit_button(
        "<html><body><form><div id='d'>x</div></form></body></html>",
        "#d",
    ));
}

#[test]
fn test_remove_attr_and_has_attribute() {
    // P1a checkbox：RemoveAttr 真正移除属性；has_attribute 判存在性。
    let html = "<html><body><input id='c' type='checkbox' checked></body></html>";
    assert!(has_attribute(html, "#c", "checked"));
    let out = apply_mutations_to_html(
        html,
        &[DomMutation::RemoveAttr {
            selector: "#c".into(),
            name: "checked".into(),
        }],
    )
    .unwrap();
    assert!(!out.contains("checked"));
    assert!(!has_attribute(&out, "#c", "checked"));
    // 无该属性 → has_attribute false。
    assert!(!has_attribute(
        "<html><body><input id='n' type='checkbox'></body></html>",
        "#n",
        "checked",
    ));
}

#[test]
fn test_is_checkbox() {
    assert!(is_checkbox(
        "<html><body><input id='c' type='checkbox'></body></html>",
        "#c",
    ));
    assert!(!is_checkbox(
        "<html><body><input id='t' type='text'></body></html>",
        "#t",
    ));
    assert!(!is_checkbox("<html><body><div id='d'></div></body></html>", "#d",));
}

#[test]
fn test_toggle_radio_html() {
    // P1a radio：toggle target → set checked + 同 name 组兄弟 unset（直接 doc 操作）。
    let html = "<html><body><form>\
            <input id='a' type='radio' name='g' checked>\
            <input id='b' type='radio' name='g'>\
            <input id='c' type='checkbox' checked>\
            </form></body></html>";
    // toggle #b → #b checked、#a unchecked（同 name 组）；#c checkbox 不受影响。
    let out = toggle_radio_html(html, "#b").unwrap();
    assert!(has_attribute(&out, "#b", "checked"));
    assert!(!has_attribute(&out, "#a", "checked"));
    assert!(has_attribute(&out, "#c", "checked"));
    // 非 radio → None。
    assert_eq!(toggle_radio_html(html, "#c"), None);
}

#[test]
fn test_select_value_read() {
    let html = "<html><body><select id='s'>\
            <option value='a'>A</option>\
            <option value='b' selected>B</option>\
            <option value='c'>C</option>\
            </select></body></html>";
    assert!(is_select(html, "#s"));
    // selected option b → "b"。
    assert_eq!(select_value_from_html(html, "#s"), "b");
    assert_eq!(select_index_from_html(html, "#s"), 1);
    // 无 selected 属性 → 默认首个 option。
    let html2 =
        "<html><body><select id='s'><option value='x'>X</option><option value='y'>Y</option></select></body></html>";
    assert_eq!(select_value_from_html(html2, "#s"), "x");
    assert_eq!(select_index_from_html(html2, "#s"), 0);
    // option 无 value 属性 → text content。
    let html3 = "<html><body><select id='s'><option>Plain</option></select></body></html>";
    assert_eq!(select_value_from_html(html3, "#s"), "Plain");
    // 无 option → 空串 / -1。
    let html4 = "<html><body><select id='s'></select></body></html>";
    assert_eq!(select_value_from_html(html4, "#s"), "");
    assert_eq!(select_index_from_html(html4, "#s"), -1);
}

#[test]
fn test_set_selected_option_html() {
    let html = "<html><body><select id='s'>\
            <option value='a' selected>A</option>\
            <option value='b'>B</option>\
            <option value='c'>C</option>\
            </select></body></html>";
    // 设 value='c' → c selected、a/b deselect。
    let out = set_selected_option_html(html, "#s", "c").unwrap();
    assert!(has_attribute(&out, "#s > option:nth-of-type(3)", "selected") || out.contains("value=\"c\" selected"));
    assert!(!has_attribute(&out, "#s > option:nth-of-type(1)", "selected"));
    // 经 value 读回 = "c"。
    assert_eq!(select_value_from_html(&out, "#s"), "c");
    // 匹配 option 无 value 属性（按 text content）。
    let html2 = "<html><body><select id='s'><option>One</option><option>Two</option></select></body></html>";
    let out2 = set_selected_option_html(html2, "#s", "Two").unwrap();
    assert_eq!(select_value_from_html(&out2, "#s"), "Two");
    // 无匹配 value → None（不改）。
    assert_eq!(set_selected_option_html(html, "#s", "zzz"), None);
    // 非 select → None。
    assert_eq!(set_selected_option_html(html, "body", "a"), None);
}

#[test]
fn test_apply_select_option_mutation() {
    let html = "<html><body><select id='s'>\
            <option value='a' selected>A</option>\
            <option value='b'>B</option>\
            </select></body></html>";
    let out = apply_mutations_to_html(
        html,
        &[DomMutation::SelectOption {
            selector: "#s".into(),
            value: "b".into(),
        }],
    )
    .unwrap();
    assert_eq!(select_value_from_html(&out, "#s"), "b");
    // SelectOption 也参与 handle→selector map（apply_dom_mutations 末尾无新 handle，map 空）。
    let (_, handles) = apply_mutations_to_html_with_handles(
        html,
        &[DomMutation::SelectOption {
            selector: "#s".into(),
            value: "b".into(),
        }],
    )
    .unwrap();
    assert!(handles.is_empty(), "SelectOption 不创建 handle");
}

#[test]
fn test_is_text_input() {
    // P1a change-on-blur：文本输入判定（textarea + input 文本类；排除 action 类型）。
    assert!(is_text_input(
        "<html><body><input id='t' type='text'></body></html>",
        "#t",
    ));
    assert!(is_text_input(
        "<html><body><input id='e' type='email'></body></html>",
        "#e",
    ));
    assert!(is_text_input(
        "<html><body><textarea id='ta'></textarea></body></html>",
        "#ta",
    ));
    // input 无 type → 默认 text。
    assert!(is_text_input("<html><body><input id='n'></body></html>", "#n"));
    // action 类型排除（change 在 click 派发）。
    assert!(!is_text_input(
        "<html><body><input id='cb' type='checkbox'></body></html>",
        "#cb",
    ));
    assert!(!is_text_input(
        "<html><body><input id='s' type='submit'></body></html>",
        "#s",
    ));
    assert!(!is_text_input("<html><body><div id='d'></div></body></html>", "#d",));
}

#[test]
fn test_next_focus_selector() {
    // P1a Tab 焦点导航：tabindex>0 升序在前（d=1, c=2），0/默认文档序在后（a, b）→ [d,c,a,b]。
    let html = "<html><body>\
            <input id='a'>\
            <button id='b'>x</button>\
            <input id='c' tabindex='2'>\
            <input id='d' tabindex='1'>\
            </body></html>";
    // 无 current → first = d（tabindex=1）。
    assert_eq!(next_focus_selector(html, None, true).as_deref(), Some("#d"));
    // current=d → c（tabindex=2）。
    assert_eq!(next_focus_selector(html, Some("#d"), true).as_deref(), Some("#c"));
    // current=c → a（文档序 tabindex=0/default）。
    assert_eq!(next_focus_selector(html, Some("#c"), true).as_deref(), Some("#a"));
    // backward：current=a → prev=c。
    assert_eq!(next_focus_selector(html, Some("#a"), false).as_deref(), Some("#c"));
    // 无 focusable → None。
    assert_eq!(
        next_focus_selector("<html><body><div>no focusable</div></body></html>", None, true),
        None
    );
}

#[test]
fn test_insert_adjacent_html_e2e() {
    // 端到端：注入生产 shim + register_dom_callbacks，验证 insertAdjacentHTML JS 契约——
    // 调用入队 InsertAdjacentHtml mutation（sel + position + html 三参数透传）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><ul id='list'><li>x</li></ul></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // beforeend：追加列表项。
    sandbox
        .execute("document.querySelector('#list').insertAdjacentHTML('beforeend', '<li>a</li>');")
        .unwrap();
    // afterbegin：首部插入。
    sandbox
        .execute("document.querySelector('#list').insertAdjacentHTML('afterbegin', '<li>0</li>');")
        .unwrap();
    // 非法 position：shim 不抛（host apply 时才错），但 mutation 仍入队（position 透传）。
    sandbox
        .execute("document.querySelector('#list').insertAdjacentHTML('nowhere', '<b/>');")
        .unwrap();

    let positions: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::InsertAdjacentHtml { position, .. } => Some(position.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        positions.len(),
        3,
        "三次 insertAdjacentHTML 均应入队 InsertAdjacentHtml mutation"
    );
    // 校验 position 透传（含非法值，host apply 时才错）。
    assert_eq!(
        positions.iter().map(String::as_str).collect::<Vec<_>>(),
        vec!["beforeend", "afterbegin", "nowhere"]
    );
}

#[test]
fn test_outer_html_e2e() {
    // 端到端：outerHTML getter 真实序列化（含自身 tag/属性/子树）+ setter 入队 SetOuterHtml。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t' class='c'>hi<span>x</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // getter：含自身 tag/属性 + 子树。
    sandbox
        .execute("globalThis.__o = document.querySelector('#t').outerHTML;")
        .unwrap();
    let outer = sandbox.execute("globalThis.__o").unwrap().value;
    assert!(outer.contains("<div"), "getter 含自身 tag\n{outer}");
    assert!(outer.contains("class=\"c\""), "getter 含属性\n{outer}");
    assert!(outer.contains("<span>x</span>"), "getter 含子树\n{outer}");

    // setter：入队 SetOuterHtml（selector + html 透传）。
    sandbox
        .execute("document.querySelector('#t').outerHTML = '<b>1</b>';")
        .unwrap();
    let set_mutation =
        mutations.lock().unwrap().iter().any(
            |m| matches!(m, DomMutation::SetOuterHtml { selector, html } if selector == "#t" && html == "<b>1</b>"),
        );
    assert!(set_mutation, "outerHTML setter 应入队 SetOuterHtml(#t, <b>1</b>)");
}

#[test]
fn test_prepend_order_e2e() {
    // prepend 多节点 + 字符串混合：参数序 == DOM 序（反序插入 afterbegin 保证）。
    // prepend(b, "str", i) on <div id=t>existing → <b></b>str<i></i>existing。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><div id='t'>existing</div></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    sandbox
        .execute(
            "var b = document.createElement('b');\
             var i = document.createElement('i');\
             document.querySelector('#t').prepend(b, 'str', i);",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    assert!(
        out.contains("<b></b>str<i></i>existing"),
        "prepend 应保持参数序（b,str,i）\n{out}"
    );
}

#[test]
fn test_before_after_order_e2e() {
    // before（前兄弟，正序 beforebegin）+ after（后兄弟，反序 afterend）。
    // 初始 <div id=t> 处于 body。before(x,y) → x,y 在 t 前；after(p,q) → p,q 在 t 后。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><div id='t'>x</div></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    sandbox
        .execute(
            "var x=document.createElement('x');var y=document.createElement('y');\
             var p=document.createElement('p');var q=document.createElement('q');\
             var t=document.querySelector('#t');\
             t.before(x,y); t.after(p,q);",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    // 期望 body 内顺序：x, y, t, p, q（before 正序在前、after 反序在后均保持参数序）。
    let ix = out.find("<x>").unwrap();
    let iy = out.find("<y>").unwrap();
    let it = out.find("<div id=\"t\">").unwrap();
    let ip = out.find("<p>").unwrap();
    let iq = out.find("<q>").unwrap();
    assert!(
        ix < iy && iy < it && it < ip && ip < iq,
        "before/after 应保持参数序 x<y<t<p<q\n{out}"
    );
}

#[test]
fn test_prepend_detached_noop_e2e() {
    // handle-only（detached）目标 prepend 无操作（无 parent/参考子，不抛）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    // detached div.prepend(...) 不抛、不入队 InsertAdjacent*。
    sandbox
        .execute("var d=document.createElement('div'); d.prepend('x'); globalThis.__ok='done';")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ok").unwrap().value, "done");
    let has_adj = mutations.lock().unwrap().iter().any(|m| {
        matches!(
            m,
            DomMutation::InsertAdjacentText { .. } | DomMutation::InsertAdjacentElement { .. }
        )
    });
    assert!(!has_adj, "detached 目标 prepend 不应入队 InsertAdjacent* mutation");
}

#[test]
fn test_replace_child_e2e() {
    // replaceChild(new, old)：在 old 位置替换为新节点，返回 old。父 [a,b] → replaceChild(newP,a) → [newP,b]。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><ul id='list'><li id='a'>A</li><li id='b'>B</li></ul></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    sandbox
        .execute(
            "var np = document.createElement('li'); np.id = 'new';\
             var list = document.querySelector('#list');\
             var old = list.replaceChild(np, document.querySelector('#a'));\
             globalThis.__ret = (old && old.id) || '';",
        )
        .unwrap();
    // spec：返回被替换的 old 节点（id=a）。
    assert_eq!(sandbox.execute("globalThis.__ret").unwrap().value, "a");
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    // new 在 a 原位置，a 被移除，b 保留。
    assert!(out.contains("<li id=\"new\">"), "replaceChild 应插入新节点\n{out}");
    assert!(!out.contains("<li id=\"a\">"), "replaceChild 应移除 old\n{out}");
    assert!(out.contains("<li id=\"b\">B</li>"), "replaceChild 应保留兄弟 b\n{out}");
    // 顺序：new 在 b 之前。
    let i_new = out.find("<li id=\"new\">").unwrap();
    let i_b = out.find("<li id=\"b\">").unwrap();
    assert!(i_new < i_b, "new 应在 b 之前（a 原位置）\n{out}");
}

#[test]
fn test_replace_with_e2e() {
    // replaceWith(x, y)：用 x,y 替换自身。body [t] → t.replaceWith(x,y) → [x,y]（t 移除）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><div id='t'>x</div></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    sandbox
        .execute(
            "var x=document.createElement('x');var y=document.createElement('y');\
             document.querySelector('#t').replaceWith(x, 'mid', y);",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    assert!(!out.contains("<div id=\"t\">"), "replaceWith 应移除自身\n{out}");
    // 顺序：x, mid(text), y 保持参数序。
    let ix = out.find("<x>").unwrap();
    let imid = out.find("mid").unwrap();
    let iy = out.find("<y>").unwrap();
    assert!(ix < imid && imid < iy, "replaceWith 应保持参数序 x<mid<y\n{out}");
}

#[test]
fn test_node_level_traversal_e2e() {
    // 节点级遍历：childNodes/firstChild/lastChild（含文本/元素/注释）、
    // nextSibling/previousSibling（跨非元素节点）。经 JS 读属性 + 断言 nodeType/nodeValue。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'>text1<span id='s'>x</span><!--c-->text2</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // childNodes：4 个子（text/span/comment/text），nodeType 正确。
    sandbox
        .execute(
            "globalThis.__cn = document.querySelector('#t').childNodes;\
             globalThis.__len = __cn.length;\
             globalThis.__types = Array.prototype.map.call(__cn, function(n){return n.nodeType;}).join(',');\
             globalThis.__t0 = __cn[0].nodeValue;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__len").unwrap().value, "4");
    // nodeType: text(3), element(1), comment(8), text(3)。
    assert_eq!(sandbox.execute("globalThis.__types").unwrap().value, "3,1,8,3");
    assert_eq!(sandbox.execute("globalThis.__t0").unwrap().value, "text1");

    // firstChild/lastChild：文本节点。
    sandbox
        .execute(
            "globalThis.__fc = document.querySelector('#t').firstChild.nodeType;\
             globalThis.__fv = document.querySelector('#t').firstChild.nodeValue;\
             globalThis.__lc = document.querySelector('#t').lastChild.nodeValue;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__fc").unwrap().value, "3");
    assert_eq!(sandbox.execute("globalThis.__fv").unwrap().value, "text1");
    assert_eq!(sandbox.execute("globalThis.__lc").unwrap().value, "text2");

    // 空元素 childNodes.length=0、firstChild=null。
    sandbox
        .execute(
            "globalThis.__e = document.querySelector('#s').childNodes.length;\
             globalThis.__ef = document.querySelector('#s').firstChild;",
        )
        .unwrap();
    // #s 含文本 "x"（1 个 text 子）。
    assert_eq!(sandbox.execute("globalThis.__e").unwrap().value, "1");

    // nextSibling/previousSibling 跨非元素节点：span 的前兄弟=text1、后兄弟=comment。
    sandbox
        .execute(
            "var s = document.querySelector('#s');\
             globalThis.__ps = s.previousSibling.nodeValue;\
             globalThis.__ns = s.nextSibling.nodeType;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ps").unwrap().value, "text1");
    assert_eq!(sandbox.execute("globalThis.__ns").unwrap().value, "8");
}

#[test]
fn test_create_document_fragment_e2e() {
    // 端到端：createDocumentFragment（nodeType 11 / nodeName）+ 建 fragment 子 + append 到 DOM
    // → flatten 子节点到目标（fragment 自身不入树）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><ul id='list'></ul></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "var f = document.createDocumentFragment();\
             var a = document.createElement('li'); a.id = 'a';\
             var b = document.createElement('li'); b.id = 'b';\
             f.appendChild(a); f.appendChild(b);\
             globalThis.__nt = f.nodeType;\
             globalThis.__nn = f.nodeName;\
             document.querySelector('#list').appendChild(f);",
        )
        .unwrap();
    // fragment nodeType 11 / nodeName '#document-fragment'。
    assert_eq!(sandbox.execute("globalThis.__nt").unwrap().value, "11");
    assert_eq!(sandbox.execute("globalThis.__nn").unwrap().value, "#document-fragment");

    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    assert!(out.contains("<li id=\"a\">"), "flatten 后 li#a 应在 #list 内\n{out}");
    assert!(out.contains("<li id=\"b\">"), "flatten 后 li#b 应在 #list 内\n{out}");
    let ia = out.find("<li id=\"a\">").unwrap();
    let ib = out.find("<li id=\"b\">").unwrap();
    assert!(ia < ib, "flatten 保持子节点顺序 a<b\n{out}");

    // 入队了 AppendFragmentChildren（sel 版）。
    let has_flatten = mutations.lock().unwrap().iter().any(
        |m| matches!(m, DomMutation::AppendFragmentChildren { parent_selector, .. } if parent_selector == "#list"),
    );
    assert!(has_flatten, "appendChild(fragment) 应入队 AppendFragmentChildren");
}

#[test]
fn test_insert_before_fragment_flatten_e2e() {
    // R2688 self-review 修复验证：insertBefore(fragment, ref) 须 flatten 子节点（spec）。
    // 旧行为：插 fragment 节点本身 → childNodes 漏子（藏在被跳过的 fragment wrapper 内）+
    //   fragment 未清空。修复后：fragment 子移到 ref 前、fragment 清空。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><ul id='list'><li id='first'>F</li></ul></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "var f = document.createDocumentFragment();\
             var a = document.createElement('li'); a.id = 'a';\
             var b = document.createElement('li'); b.id = 'b';\
             f.appendChild(a); f.appendChild(b);\
             var list = document.querySelector('#list');\
             list.insertBefore(f, list.firstChild);",
        )
        .unwrap();
    // 入队 InsertFragmentBefore（非 InsertBefore 插 fragment 节点本身）。
    let used_flatten =
        mutations.lock().unwrap().iter().any(
            |m| matches!(m, DomMutation::InsertFragmentBefore { parent_selector, .. } if parent_selector == "#list"),
        );
    assert!(
        used_flatten,
        "insertBefore(fragment, ref) 应入队 InsertFragmentBefore（flatten）"
    );

    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    // flatten 后顺序：a, b, first（fragment 子在 first 之前）。
    let ia = out.find("<li id=\"a\">").unwrap();
    let ib = out.find("<li id=\"b\">").unwrap();
    let ifirst = out.find("<li id=\"first\">").unwrap();
    assert!(ia < ib && ib < ifirst, "flatten 后 a<b<first\n{out}");
}

#[test]
fn test_fragment_flatten_all_insertion_paths_e2e() {
    // R2689：闭合 fragment flatten 同类 bug——prepend/before/after/replaceChild 接 fragment
    // 须 flatten 子节点（非插 wrapper）。经 JS→mutation→apply 序列化验最终 DOM 序。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><div id='t'>X</div></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // prepend(fragment)：fragment 子成为 #t 首子（在 X 前）。
    sandbox
        .execute(
            "var f1=document.createDocumentFragment();\
             var a=document.createElement('a'); var b=document.createElement('b');\
             f1.appendChild(a); f1.appendChild(b);\
             document.querySelector('#t').prepend(f1);",
        )
        .unwrap();
    let ms1 = mutations.lock().unwrap().clone();
    let (out1, _) = apply_mutations_to_html_with_handles(&initial, &ms1).unwrap();
    // #t 内：a, b, X（fragment 子在前）。
    let o1a = out1.find("<a>").unwrap();
    let o1b = out1.find("<b>").unwrap();
    let o1x = out1.find("X</div>").unwrap();
    assert!(o1a < o1b && o1b < o1x, "prepend(fragment): a<b<X\n{out1}");

    // before(fragment)：fragment 子作 #t 前兄弟。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var f2=document.createDocumentFragment();\
             var c=document.createElement('c');\
             f2.appendChild(c);\
             document.querySelector('#t').before(f2);",
        )
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let (out2, _) = apply_mutations_to_html_with_handles(&out1, &ms2).unwrap();
    let o2c = out2.find("<c>").unwrap();
    let o2t = out2.find("<div id=\"t\">").unwrap();
    assert!(o2c < o2t, "before(fragment): c 在 #t 前\n{out2}");

    // after(fragment)：fragment 子作 #t 后兄弟。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var f3=document.createDocumentFragment();\
             var d=document.createElement('d');\
             f3.appendChild(d);\
             document.querySelector('#t').after(f3);",
        )
        .unwrap();
    let ms3 = mutations.lock().unwrap().clone();
    let (out3, _) = apply_mutations_to_html_with_handles(&out2, &ms3).unwrap();
    let o3t = out3.find("<div id=\"t\">").unwrap();
    let o3d = out3.find("<d>").unwrap();
    assert!(o3t < o3d, "after(fragment): d 在 #t 后\n{out3}");

    // replaceChild(fragment, old)：fragment 子替换 #t（old=#t）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var f4=document.createDocumentFragment();\
             var e=document.createElement('e');\
             f4.appendChild(e);\
             var body=document.querySelector('body');\
             body.replaceChild(f4, document.querySelector('#t'));",
        )
        .unwrap();
    let ms4 = mutations.lock().unwrap().clone();
    let (out4, _) = apply_mutations_to_html_with_handles(&out3, &ms4).unwrap();
    assert!(out4.contains("<e>"), "replaceChild(fragment): e 替换 #t\n{out4}");
    assert!(
        !out4.contains("<div id=\"t\">"),
        "replaceChild(fragment): #t 应被移除\n{out4}"
    );
}

#[test]
fn test_parent_node_nested_e2e() {
    // R2690：parentNode/parentElement 嵌套正确性（旧 stub 恒返 body）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='outer'><div id='inner'>x</div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // inner.parentNode.id === 'outer'（旧 stub 错返 body → id ''）。
    sandbox
        .execute("globalThis.__p = document.querySelector('#inner').parentNode.id;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__p").unwrap().value, "outer");
    // inner.parentElement 同。
    sandbox
        .execute("globalThis.__pe = document.querySelector('#inner').parentElement.id;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__pe").unwrap().value, "outer");
    // outer.parentNode.tagName === 'BODY'。
    sandbox
        .execute("globalThis.__op = document.querySelector('#outer').parentNode.tagName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__op").unwrap().value, "BODY");
    // html 根 parentNode === null。
    sandbox
        .execute("globalThis.__hp = document.querySelector('html').parentNode === null;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__hp").unwrap().value, "true");
}

#[test]
fn test_tag_name_real_not_div_heuristic() {
    // R2691：tagName/nodeName 真实化（旧 _tagFromSel 对 #id 选择器恒猜 DIV）。
    // sel-based：id-bearing 非 DIV 元素返真实 tag；handle-based：detached createElement 返真实 tag。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><span id=\"s\">x</span><input id=\"i\"></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // sel-based：#s 是 <span>（旧 stub 错返 DIV），#i 是 <input>。
    sandbox
        .execute("globalThis.__s = document.querySelector('#s').tagName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__s").unwrap().value, "SPAN");
    sandbox
        .execute("globalThis.__i = document.querySelector('#i').tagName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__i").unwrap().value, "INPUT");
    // nodeName 同 tagName（元素节点）。
    sandbox
        .execute("globalThis.__sn = document.querySelector('#s').nodeName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__sn").unwrap().value, "SPAN");
    // 大小写：tagName 在 HTML 命名空间须大写（createElement('svg')→'SVG'）。
    sandbox
        .execute("globalThis.__tr = document.createElement('tr').tagName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__tr").unwrap().value, "TR");
}

#[test]
fn test_event_bubbling_to_ancestor() {
    // R2692：事件冒泡。旧 dispatchEvent/__zw_dispatch_event 仅派发 target 自身 listener，
    // 不冒泡到祖先——事件委托（document/body 上注册的 listener 捕获子元素事件）失效。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"p\"><span id=\"c\">x</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 祖先 #p 与 document(html key) 各注册 click listener。
    sandbox
        .execute(
            "document.querySelector('#p').addEventListener('click', function(e){ globalThis.__p = e.currentTarget.id; });",
        )
        .unwrap();
    sandbox
        .execute("document.addEventListener('click', function(){ globalThis.__doc = true; });")
        .unwrap();
    // 在子 #c 上派发 click → 应冒泡到 #p 和 document（html）。
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__p").unwrap().value,
        "p",
        "#p listener 应经冒泡触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__doc").unwrap().value,
        "true",
        "document listener 应经冒泡触发（事件委托）"
    );

    // currentTarget 在 target 阶段 = target 自身。
    sandbox
        .execute(
            "document.querySelector('#c').addEventListener('click', function(e){ globalThis.__ct = e.currentTarget.id; });",
        )
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ct").unwrap().value,
        "c",
        "target 阶段 currentTarget = target"
    );
}

#[test]
fn test_event_bubbling_stop_and_nonbubble() {
    // R2692 续：stopPropagation 中断冒泡；bubbles:false 的事件不冒泡。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"a\"><div id=\"b\"><i id=\"c\">x</i></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // #b stopPropagation → #a 不应触发。注册顺序：先 #a 后 #b（冒泡 #c→#b→#a）。
    sandbox
        .execute("document.querySelector('#a').addEventListener('click', function(){ globalThis.__a = true; });")
        .unwrap();
    sandbox
        .execute("document.querySelector('#b').addEventListener('click', function(e){ globalThis.__b = true; e.stopPropagation(); });")
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__b === true").unwrap().value,
        "true",
        "#b 应触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__a === true").unwrap().value,
        "false",
        "#a 不应触发（stopPropagation 中断冒泡）"
    );

    // bubbles:false 的事件不冒泡：dispatchEvent 自定义非冒泡事件到 #c，#b 不触发。
    sandbox
        .execute("document.querySelector('#b').addEventListener('foo', function(){ globalThis.__foo = true; });")
        .unwrap();
    sandbox
        .execute("document.querySelector('#c').dispatchEvent(new Event('foo', { bubbles: false }));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__foo === true").unwrap().value,
        "false",
        "bubbles:false 事件不应冒泡到 #b"
    );
}

#[test]
fn test_event_capture_phase() {
    // R2693：capture 阶段。祖先 capture listener（addEventListener 第三参 true）在 root→target
    // 捕获期触发，先于 target（AT_TARGET）与 bubble。旧实现祖先 capture listener 永不触发。
    // 同时验证 legacy 布尔第三参 `addEventListener(t, fn, true)` 注册 capture（_optCapture 修复）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"a\"><div id=\"b\"><i id=\"c\">x</i></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // #a capture（legacy 布尔第三参 true）→ #c target（非 capture）→ #b bubble，记录派发顺序。
    sandbox
        .execute(
            "document.querySelector('#a').addEventListener('click', function(e){ globalThis.__order = (globalThis.__order||'') + 'capA:' + e.currentTarget.id + ';'; }, true);",
        )
        .unwrap();
    sandbox
        .execute(
            "document.querySelector('#c').addEventListener('click', function(e){ globalThis.__order += 'tgt:' + e.currentTarget.id + ';'; });",
        )
        .unwrap();
    sandbox
        .execute(
            "document.querySelector('#b').addEventListener('click', function(e){ globalThis.__order += 'bubB:' + e.currentTarget.id + ';'; });",
        )
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    // 捕获期 #a（root 方向）先于 target #c，先于冒泡期 #b。
    assert_eq!(
        sandbox.execute("globalThis.__order").unwrap().value,
        "capA:a;tgt:c;bubB:b;",
        "capture(#a) → target(#c) → bubble(#b) 顺序"
    );
}

#[test]
fn test_event_capture_stop_propagation() {
    // R2693 续：capture 期 stopPropagation → target 与 bubble 阶段不触发。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"a\"><div id=\"b\"><i id=\"c\">x</i></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // #a capture stopPropagation；#c target；#b bubble。
    sandbox
        .execute("document.querySelector('#a').addEventListener('click', function(e){ globalThis.__cap = true; e.stopPropagation(); }, { capture: true });")
        .unwrap();
    sandbox
        .execute("document.querySelector('#c').addEventListener('click', function(){ globalThis.__tgt = true; });")
        .unwrap();
    sandbox
        .execute("document.querySelector('#b').addEventListener('click', function(){ globalThis.__bub = true; });")
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cap === true").unwrap().value,
        "true",
        "#a capture 应触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__tgt === true").unwrap().value,
        "false",
        "capture stopPropagation 后 target 不应触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__bub === true").unwrap().value,
        "false",
        "capture stopPropagation 后 bubble 不应触发"
    );
}

#[test]
fn test_event_listener_once() {
    // R2694：`once` 选项。`{once:true}` 注册的 listener 派发一次后自动移除（再次派发不触发）。
    // 旧实现完全忽略 once → listener 重复触发。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><button id=\"b\">x</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "document.querySelector('#b').addEventListener('click', function(){ globalThis.__n = (globalThis.__n|0) + 1; }, { once: true });",
        )
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#b', 'click', null);").unwrap();
    sandbox.execute("__zw_dispatch_event('#b', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__n").unwrap().value,
        "1",
        "once listener 应仅触发一次（第二次派发不触发）"
    );
}

#[test]
fn test_remove_event_listener_capture_aware() {
    // R2694：capture-aware removeEventListener。spec：useCapture 须匹配才移除——
    // `addEventListener(t, fn, true)`（capture）仅 `removeEventListener(t, fn, true)` 能移除；
    // `removeEventListener(t, fn)`（capture=false）不应动 capture 注册。旧实现按 fn 误删。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"p\"><i id=\"c\">x</i></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // #p 上注册 capture listener（fn），随后 removeEventListener 不带 capture → 不应移除。
    sandbox
        .execute(
            "globalThis.__fn = function(){ globalThis.__cap = (globalThis.__cap|0) + 1; };\n\
             document.querySelector('#p').addEventListener('click', globalThis.__fn, true);\n\
             document.querySelector('#p').removeEventListener('click', globalThis.__fn);\n\
             __zw_dispatch_event('#c', 'click', null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cap").unwrap().value,
        "1",
        "removeEventListener(fn) 不带 capture 不应移除 capture 注册（仍触发）"
    );
    // 现在 removeEventListener 带 capture=true → 应移除，再次派发不触发。
    sandbox
        .execute(
            "document.querySelector('#p').removeEventListener('click', globalThis.__fn, true);\n\
             __zw_dispatch_event('#c', 'click', null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cap").unwrap().value,
        "1",
        "removeEventListener(fn, true) 应移除 capture 注册（再次派发不触发）"
    );
}

#[test]
fn test_style_proxy_methods() {
    // R2695：style 代理 API。getPropertyValue 读初始 style；setProperty 经 SetStyle 应用；
    // per-property get/set 保留；cssText get 读原始串、set 经 SetAttr 整体替换。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\" style=\"color: red\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // getPropertyValue 读初始 style 快照（'color: red' → 'red'）。
    sandbox
        .execute("globalThis.__gv = document.querySelector('#d').style.getPropertyValue('color');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__gv").unwrap().value,
        "red",
        "getPropertyValue 读初始 color"
    );
    // per-property get 保留（'color' → 'red'）。
    sandbox
        .execute("globalThis.__pg = document.querySelector('#d').style.color;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__pg").unwrap().value,
        "red",
        "per-property style.color 保留"
    );
    // cssText get 读原始串。
    sandbox
        .execute("globalThis.__ct = document.querySelector('#d').style.cssText;")
        .unwrap();
    assert!(
        sandbox.execute("globalThis.__ct").unwrap().value.contains("color: red"),
        "cssText getter 读原始 style 串"
    );

    // setProperty（dashed 名）+ per-property set → 应用后验证序列化。
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.style.setProperty('background-color', 'blue');\n\
             d.style.fontSize = '10px';",
        )
        .unwrap();
    let ms1 = mutations.lock().unwrap().clone();
    let out1 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms1).unwrap();
    assert!(out1.contains("background-color: blue"), "setProperty 应用\n{out1}");
    assert!(
        out1.contains("font-size: 10px"),
        "per-property style.fontSize 须归一为 kebab-case 应用\n{out1}"
    );

    // cssText set → 整体替换（原 color: red 应消失）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute("document.querySelector('#d').style.cssText = 'margin: 0; padding: 5px';")
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    assert!(out2.contains("margin: 0"), "cssText setter 应用 margin\n{out2}");
    assert!(out2.contains("padding: 5px"), "cssText setter 应用 padding\n{out2}");
    assert!(
        !out2.contains("color: red"),
        "cssText setter 应整体替换（原 color 消失）\n{out2}"
    );
}

#[test]
fn test_style_remove_property() {
    // R2695：removeProperty 真移除 style 声明（SetStyle 空值仍 push 'prop: '，不移除）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\" style=\"color: red; font-size: 10px\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute("document.querySelector('#d').style.removeProperty('color');")
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        !out.contains("color"),
        "removeProperty('color') 应真移除 color 声明\n{out}"
    );
    assert!(
        out.contains("font-size: 10px"),
        "removeProperty 不应影响其他属性\n{out}"
    );
}

#[test]
fn test_style_camel_to_kebab() {
    // R2696：per-property camelCase style 须归一为 kebab-case 存 style 属性（CSS parser 不认
    // camelCase → 渲染静默失效）。覆盖 backgroundColor / WebkitTransform（vendor 前缀）/ cssFloat
    // （→float）/ per-property camelCase 读 kebab 属性。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\" style=\"font-size: 10px\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // per-property camelCase 读 kebab 属性（font-size → fontSize 读出 '10px'）。
    sandbox
        .execute("globalThis.__fs = document.querySelector('#d').style.fontSize;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fs").unwrap().value,
        "10px",
        "camelCase 读 kebab 属性"
    );

    // camelCase set → kebab 存储（不残留 camelCase）。
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.style.backgroundColor = 'red';\n\
             d.style.WebkitTransform = 'scale(2)';\n\
             d.style.cssFloat = 'left';",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        out.contains("background-color: red"),
        "backgroundColor → background-color\n{out}"
    );
    assert!(
        !out.contains("backgroundColor"),
        "不应残留 camelCase backgroundColor\n{out}"
    );
    assert!(
        out.contains("-webkit-transform: scale(2)"),
        "WebkitTransform → -webkit-transform\n{out}"
    );
    assert!(out.contains("float: left"), "cssFloat → float\n{out}");
    assert!(!out.contains("cssFloat"), "不应残留 cssFloat\n{out}");
}

#[test]
fn test_classlist_consecutive_ops() {
    // R2697：classList 连续操作不丢类。旧实现每次读 stale snapshot + SetAttr 整体替换，
    // 同脚本 add('a');add('b');add('c') 仅保留末个（base c）。客户端缓存累积全量后修复。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mk = || -> (V8Sandbox, Arc<Mutex<Vec<DomMutation>>>, Arc<Mutex<String>>) {
        let mut sb = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sb.execute(generate_js_dom_shim()).unwrap();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
        let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
            "<html><body><div id=\"d\" class=\"base\"></div></body></html>".to_string(),
        ));
        let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
        register_dom_callbacks(&mut sb, &mutations, &dom_html, &page_url);
        (sb, mutations, dom_html)
    };

    // ① 连续 add 三类 → apply 后 class 含 base/a/b/c 全部（旧实现仅 'base c'）。
    let (mut sandbox, mutations, dom_html) = mk();
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.classList.add('a');\n\
             d.classList.add('b');\n\
             d.classList.add('c');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    let class_val: String = out
        .split("class=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or("")
        .to_string();
    for cls in ["base", "a", "b", "c"] {
        assert!(
            class_val.split_whitespace().any(|t| t == cls),
            "class 应含 {cls}（got '{class_val}'）\n{out}"
        );
    }

    // ② className set + classList add 协作（className 写缓存、classList 读缓存累加）。
    let (mut sandbox, mutations, dom_html) = mk();
    sandbox
        .execute(
            "var e = document.querySelector('#d');\n\
             e.className = 'x';\n\
             e.classList.add('y');",
        )
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    assert!(
        out2.contains("class=\"x y\""),
        "className=x 后 classList.add(y) → 'x y'\n{out2}"
    );

    // ③ toggle 首次加（true）/ contains 反映 / 二次移除（false），双 toggle 后 on 消失。
    let (mut sandbox, mutations, dom_html) = mk();
    sandbox
        .execute(
            "globalThis.__t1 = document.querySelector('#d').classList.toggle('on');\n\
             globalThis.__has = document.querySelector('#d').classList.contains('on');\n\
             globalThis.__t2 = document.querySelector('#d').classList.toggle('on');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__t1").unwrap().value,
        "true",
        "toggle 首次加返 true"
    );
    assert_eq!(
        sandbox.execute("globalThis.__has").unwrap().value,
        "true",
        "toggle 后 contains(on) 反映缓存"
    );
    assert_eq!(
        sandbox.execute("globalThis.__t2").unwrap().value,
        "false",
        "toggle 二次移除返 false"
    );
    let ms3 = mutations.lock().unwrap().clone();
    let out3 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms3).unwrap();
    let class_val3: String = out3
        .split("class=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or("")
        .to_string();
    assert!(
        !class_val3.split_whitespace().any(|t| t == "on"),
        "双 toggle 后 on 应移除（got '{class_val3}'）\n{out3}"
    );
}

#[test]
fn test_remove_attribute_truly_removes() {
    // R2698：removeAttribute 真移除。旧 set-empty 残留 `checked=""`（present）→ el.checked 仍 true。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><input id=\"i\" checked></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute("document.querySelector('#i').removeAttribute('checked');")
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        !out.contains("checked"),
        "removeAttribute('checked') 应真移除（不残留 checked=\"\"）\n{out}"
    );
}

#[test]
fn test_attribute_query_api() {
    // R2698：hasAttribute/hasAttributes/getAttributeNames/toggleAttribute。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><input id=\"i\" type=\"text\" disabled></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // hasAttribute（present/absent）。
    sandbox
        .execute(
            "globalThis.__hd = document.querySelector('#i').hasAttribute('disabled');\n\
             globalThis.__hid = document.querySelector('#i').hasAttribute('id');\n\
             globalThis.__no = document.querySelector('#i').hasAttribute('checked');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hd").unwrap().value,
        "true",
        "hasAttribute(disabled)"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hid").unwrap().value,
        "true",
        "hasAttribute(id)"
    );
    assert_eq!(
        sandbox.execute("globalThis.__no").unwrap().value,
        "false",
        "hasAttribute(checked) absent"
    );

    // hasAttributes + getAttributeNames。
    sandbox
        .execute(
            "globalThis.__hs = document.querySelector('#i').hasAttributes();\n\
             globalThis.__names = document.querySelector('#i').getAttributeNames().join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hs").unwrap().value,
        "true",
        "hasAttributes"
    );
    assert_eq!(
        sandbox.execute("globalThis.__names").unwrap().value,
        "id,type,disabled",
        "getAttributeNames 顺序"
    );
}

#[test]
fn test_toggle_attribute() {
    // R2701：toggleAttribute 经 server-side mutation（apply 时决策），连续 toggle 正确复合。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id=\"d\"></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 单次 toggle 加 → 返 true；force=false 移除（即便刚加，server-side 不受 stale 影响）。
    sandbox
        .execute(
            "globalThis.__r1 = document.querySelector('#d').toggleAttribute('hidden');\n\
             document.querySelector('#d').toggleAttribute('hidden', false);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "true",
        "toggle 加返 true"
    );
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    // toggle(hidden) 加 → ToggleAttribute(want=true)；toggle(hidden,false) → want=false。
    // apply 顺序：先加 hidden，再移除 → net 无 hidden。
    assert!(!out.contains("hidden"), "force=false 应移除（net 无 hidden）\n{out}");

    // 连续双 toggle（无 force）：朴素实现都读 stale 都加 → 残留；server-side 决策正确复合 → net 移除。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "document.querySelector('#d').toggleAttribute('x');\n\
             document.querySelector('#d').toggleAttribute('x');",
        )
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    // 两次 toggle(x)：apply 时第一次无 x→加，第二次有 x→移除 → net 无 x（朴素实现都加会残留 x）。
    assert!(
        !out2.contains("x"),
        "连续双 toggle(x) server-side 决策 → net 移除（无 x）\n{out2}"
    );

    // force=true 强加（即便存在也保留）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute("document.querySelector('#d').toggleAttribute('aria-label', true);")
        .unwrap();
    let ms3 = mutations.lock().unwrap().clone();
    let out3 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms3).unwrap();
    assert!(out3.contains("aria-label"), "force=true 强加 aria-label\n{out3}");
}

#[test]
fn test_get_computed_style_display_position_visibility_opacity() {
    // R2704：getComputedStyle 计算值（首批 display/position/visibility/opacity）。旧全属性返 '' →
    // visibility/hidden 分支全断。现经 __zw_get_computed_style 返真实计算值（UA builtin + <style>）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <div id=\"d\"></div>\
         <span id=\"s\" style=\"display:none\"></span>\
         <style>#d { position: relative; opacity: 0.5 }</style>\
         <p id=\"hid\" style=\"visibility:hidden\"></p>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // div：UA display=block；<style> 设 position=relative、opacity=0.5。
    sandbox
        .execute(
            "globalThis.__dd = getComputedStyle(document.querySelector('#d')).display;\n\
             globalThis.__dp = getComputedStyle(document.querySelector('#d')).position;\n\
             globalThis.__do = getComputedStyle(document.querySelector('#d')).opacity;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dd").unwrap().value,
        "block",
        "div UA display=block"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dp").unwrap().value,
        "relative",
        "<style> position=relative"
    );
    assert_eq!(
        sandbox.execute("globalThis.__do").unwrap().value,
        "0.5",
        "<style> opacity=0.5"
    );
    // span inline display:none；getPropertyValue(kebab) 路径。
    sandbox
        .execute("globalThis.__sd = getComputedStyle(document.querySelector('#s')).getPropertyValue('display');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__sd").unwrap().value,
        "none",
        "inline style display:none（getPropertyValue kebab 路径）"
    );
    // p inline visibility:hidden。
    sandbox
        .execute("globalThis.__pv = getComputedStyle(document.querySelector('#hid')).visibility;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__pv").unwrap().value,
        "hidden",
        "inline visibility:hidden"
    );
}

#[test]
fn test_get_computed_style_colors() {
    // R2705：getComputedStyle 颜色族（color/background-color/border-*-color）。compute_styles 保留
    // 颜色未解析（Named/CurrentColor），经 paint 层 resolve_color_current 解析为 rgb/rgba 串。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <div id=\"a\" style=\"color: red; background-color: rgb(0, 128, 0)\"></div>\
         <div id=\"b\" style=\"border: 1px solid blue\"></div>\
         <div id=\"c\" style=\"color: transparent\"></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // color: red（named → rgb）+ background-color: rgb(0,128,0)。
    sandbox
        .execute(
            "globalThis.__col = getComputedStyle(document.querySelector('#a')).color;\n\
             globalThis.__bg = getComputedStyle(document.querySelector('#a')).backgroundColor;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__col").unwrap().value,
        "rgb(255, 0, 0)",
        "color: red → rgb(255,0,0)"
    );
    assert_eq!(
        sandbox.execute("globalThis.__bg").unwrap().value,
        "rgb(0, 128, 0)",
        "background-color: rgb(0,128,0)"
    );
    // border: 1px solid blue → border-color (4 边) = rgb(0,0,255)。
    sandbox
        .execute(
            "globalThis.__bt = getComputedStyle(document.querySelector('#b')).getPropertyValue('border-top-color');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__bt").unwrap().value,
        "rgb(0, 0, 255)",
        "border shorthand 的 blue → border-top-color rgb(0,0,255)"
    );
    // color: transparent → rgba(0,0,0,0)。
    sandbox
        .execute("globalThis.__tc = getComputedStyle(document.querySelector('#c')).color;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__tc").unwrap().value,
        "rgba(0, 0, 0, 0)",
        "color: transparent → rgba(0,0,0,0)"
    );
}

#[test]
fn test_computed_style_cache_reuse_composition() {
    // R2706：getComputedStyle per-snapshot 缓存 = compute_document_styles（一次）+
    // lookup_computed_property（多次）。验证「build (doc, styles) once → query N 属性」与无缓存
    // computed_style_property 逐次等价——锁缓存命中路径返回值正确（缓存复用不改变结果）。
    let html = "<html><body>\
        <div id=\"d\" style=\"color: red; display: none; opacity: 0.25\"></div>\
        <style>#d { position: relative }</style>\
        </body></html>";
    let (doc, styles) = compute_document_styles(html);
    // 同一 (doc, styles) 连续查 4 个属性（缓存命中场景）。
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "color"), "rgb(255, 0, 0)");
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "display"), "none");
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "opacity"), "0.25");
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "position"), "relative");
    // 与无缓存参考实现逐属性等价。
    assert_eq!(computed_style_property(html, "#d", "color"), "rgb(255, 0, 0)");
    assert_eq!(computed_style_property(html, "#d", "display"), "none");
    assert_eq!(computed_style_property(html, "#d", "position"), "relative");
    // 未命中选择器 → ''；margin-top R2707 起已覆盖（长度族）→ div 默认 0px。
    assert_eq!(lookup_computed_property(&doc, &styles, "#missing", "color"), "");
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "margin-top"), "0px");
}

#[test]
fn test_get_computed_style_cache_invalidation() {
    // R2706：getComputedStyle per-snapshot 缓存失效（核心正确性风险）。同一会话内首查填缓存后
    // 改 dom_html snapshot，再查须反映新 html（不返 stale 缓存值）。缓存 keyed on html。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id=\"d\"></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 首查：div UA display=block（填缓存）。
    sandbox
        .execute("globalThis.__v1 = getComputedStyle(document.querySelector('#d')).display;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__v1").unwrap().value, "block");

    // 改 snapshot：注入 <style>#d{display:none}</style>。缓存 keyed on html → 失效 → 重算。
    *dom_html.lock().unwrap() = "<html><body><div id=\"d\"></div>\
        <style>#d { display: none }</style></body></html>"
        .to_string();
    sandbox
        .execute("globalThis.__v2 = getComputedStyle(document.querySelector('#d')).display;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__v2").unwrap().value,
        "none",
        "html snapshot 变 → 缓存失效重算，返新 display=none（非 stale 的 block）"
    );
}

#[test]
fn test_get_computed_style_lengths() {
    // R2707：getComputedStyle 长度族（width/height/min-max/margin/padding/border-width/
    // border-radius/outline-width/font-size/gap/letter-spacing/text-indent 等）。compute_styles
    // 已把相对单位解析为 Px，故 px 指定值精确；百分比/auto 保留（无 layout 不解析为 used 值）。
    // border-width 在 style:none 时返 "0px"（used=0）；outline-width 则保留 computed medium→3px（R2754）；max-*:none → "none"。
    let html = "<html><body>\
        <div id=\"box\" style=\"\
            width: 100px; height: 50%; \
            margin-top: 10px; margin-right: 20px; margin-bottom: 10px; margin-left: 20px; \
            padding: 5px; \
            border-top-width: 3px; border-top-style: solid; \
            border-top-left-radius: 8px; \
            outline-width: 2px; outline-style: solid; \
            max-width: 500px; min-width: auto; \
            font-size: 20px; \
            gap: 12px; letter-spacing: 0.1em; \
        \"></div>\
        <div id=\"plain\"></div>\
        </body></html>";

    // px 指定 → 精确（Chrome 一致）。
    assert_eq!(computed_style_property(html, "#box", "width"), "100px");
    assert_eq!(computed_style_property(html, "#box", "margin-top"), "10px");
    assert_eq!(computed_style_property(html, "#box", "margin-right"), "20px");
    assert_eq!(computed_style_property(html, "#box", "margin-bottom"), "10px");
    assert_eq!(computed_style_property(html, "#box", "margin-left"), "20px");
    assert_eq!(computed_style_property(html, "#box", "padding-top"), "5px");
    assert_eq!(computed_style_property(html, "#box", "padding-left"), "5px");
    // 百分比 → 保留（计算值，无 layout 不解析 used 值）。
    assert_eq!(computed_style_property(html, "#box", "height"), "50%");
    // em → 解析为 px（letter-spacing 0.1em @ font-size 20px = 2px）。
    assert_eq!(computed_style_property(html, "#box", "letter-spacing"), "2px");
    assert_eq!(computed_style_property(html, "#box", "font-size"), "20px");
    assert_eq!(computed_style_property(html, "#box", "gap"), "12px");
    // border-width：style=solid → 真宽；border-radius px。
    assert_eq!(computed_style_property(html, "#box", "border-top-width"), "3px");
    assert_eq!(computed_style_property(html, "#box", "border-top-left-radius"), "8px");
    // outline-width：style=solid → 真宽。
    assert_eq!(computed_style_property(html, "#box", "outline-width"), "2px");
    // max-width 指定 → px；min-width auto → "auto"。
    assert_eq!(computed_style_property(html, "#box", "max-width"), "500px");
    assert_eq!(computed_style_property(html, "#box", "min-width"), "auto");

    // 默认 div（无 border）：border-width 返 "0px"（border-style:none → used=0，对齐 Chromium）。
    assert_eq!(computed_style_property(html, "#plain", "border-top-width"), "0px");
    // R2754：outline-width 不套 border 的 none→0 规则——outline-style:none 时 outline-width 仍保留
    // computed 值（medium→3px），Chromium getComputedStyle 返 "3px"（与 border-width 行为不同）。
    assert_eq!(computed_style_property(html, "#plain", "outline-width"), "3px");
    // 默认 max-width/max-height:none → "none"；默认 margin:0 → "0px"；默认 width:auto → "auto"。
    assert_eq!(computed_style_property(html, "#plain", "max-width"), "none");
    assert_eq!(computed_style_property(html, "#plain", "max-height"), "none");
    assert_eq!(computed_style_property(html, "#plain", "margin-top"), "0px");
    assert_eq!(computed_style_property(html, "#plain", "width"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "font-size"), "16px");
}

#[test]
fn test_get_computed_style_keywords() {
    // R2708：getComputedStyle 关键字/枚举族（float/clear/box-sizing/overflow/text-align/
    // white-space/font-weight/font-style/line-height/z-index/cursor/text-transform/text-overflow/
    // direction/border-collapse/table-layout/caption-side/border-*-style/outline-style）。
    let html = "<html><body>\
        <div id=\"k\" style=\"\
            float: left; clear: both; box-sizing: border-box; \
            overflow: hidden; text-align: center; white-space: pre-wrap; \
            font-weight: bold; font-style: italic; line-height: 1.5; \
            z-index: 10; cursor: pointer; text-transform: uppercase; \
            text-overflow: ellipsis; direction: rtl; \
            border: 2px dashed red; outline: 3px dotted blue; \
        \"></div>\
        <table id=\"t\" style=\"border-collapse: collapse; table-layout: fixed;\
            \"><caption id=\"cap\"></caption><tr><td></td></tr></table>\
        <div id=\"plain\"></div>\
        </body></html>";

    // 显式设置的关键字直映。
    assert_eq!(computed_style_property(html, "#k", "float"), "left");
    assert_eq!(computed_style_property(html, "#k", "clear"), "both");
    assert_eq!(computed_style_property(html, "#k", "box-sizing"), "border-box");
    assert_eq!(computed_style_property(html, "#k", "overflow-x"), "hidden");
    assert_eq!(computed_style_property(html, "#k", "overflow-y"), "hidden");
    assert_eq!(computed_style_property(html, "#k", "text-align"), "center");
    assert_eq!(computed_style_property(html, "#k", "white-space"), "pre-wrap");
    // font-weight bold→700（对齐 Chromium 绝对值）、font-style italic、line-height number。
    assert_eq!(computed_style_property(html, "#k", "font-weight"), "700");
    assert_eq!(computed_style_property(html, "#k", "font-style"), "italic");
    assert_eq!(computed_style_property(html, "#k", "line-height"), "1.5");
    assert_eq!(computed_style_property(html, "#k", "z-index"), "10");
    assert_eq!(computed_style_property(html, "#k", "cursor"), "pointer");
    assert_eq!(computed_style_property(html, "#k", "text-transform"), "uppercase");
    assert_eq!(computed_style_property(html, "#k", "text-overflow"), "ellipsis");
    assert_eq!(computed_style_property(html, "#k", "direction"), "rtl");
    // border/outline shorthand → longhand style。
    assert_eq!(computed_style_property(html, "#k", "border-top-style"), "dashed");
    assert_eq!(computed_style_property(html, "#k", "outline-style"), "dotted");
    // 表格属性。
    assert_eq!(computed_style_property(html, "#t", "border-collapse"), "collapse");
    assert_eq!(computed_style_property(html, "#t", "table-layout"), "fixed");

    // 默认值（initial 关键字）——验证关键字族 fallback 正确。
    assert_eq!(computed_style_property(html, "#plain", "float"), "none");
    assert_eq!(computed_style_property(html, "#plain", "box-sizing"), "content-box");
    assert_eq!(computed_style_property(html, "#plain", "overflow-x"), "visible");
    assert_eq!(computed_style_property(html, "#plain", "text-align"), "start");
    assert_eq!(computed_style_property(html, "#plain", "white-space"), "normal");
    assert_eq!(computed_style_property(html, "#plain", "font-weight"), "400");
    assert_eq!(computed_style_property(html, "#plain", "font-style"), "normal");
    assert_eq!(computed_style_property(html, "#plain", "line-height"), "normal");
    assert_eq!(computed_style_property(html, "#plain", "z-index"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "cursor"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "text-transform"), "none");
    assert_eq!(computed_style_property(html, "#plain", "text-overflow"), "clip");
    assert_eq!(computed_style_property(html, "#plain", "direction"), "ltr");
    assert_eq!(computed_style_property(html, "#plain", "border-top-style"), "none");
    assert_eq!(computed_style_property(html, "#plain", "outline-style"), "none");
}

#[test]
fn test_get_computed_style_composite() {
    // R2710：getComputedStyle 复合/列表族（font-family/flex-*/justify-content/align-*
    // /writing-mode/object-fit/isolation/mix-blend-mode/pointer-events/user-select/list-style-*）。
    let html = "<html><body>\
        <div id=\"c\" style=\"\
            font-family: 'Helvetica Neue', Arial, sans-serif; \
            flex-direction: column; flex-wrap: wrap; \
            justify-content: space-between; align-items: center; align-self: flex-end; \
            writing-mode: vertical-rl; object-fit: cover; isolation: isolate; \
            mix-blend-mode: multiply; pointer-events: none; user-select: all; \
        \"></div>\
        <ul id=\"l\" style=\"list-style-type: lower-alpha; list-style-position: inside;\
            \"><li></li></ul>\
        <div id=\"plain\"></div>\
        </body></html>";

    // font-family：逗号分隔，带空格的族名加引号，简单 ident（Arial/sans-serif）不引号。
    assert_eq!(
        computed_style_property(html, "#c", "font-family"),
        "\"Helvetica Neue\", Arial, sans-serif"
    );
    // flex / alignment / writing-mode / object-fit / 隔离·混合·交互。
    assert_eq!(computed_style_property(html, "#c", "flex-direction"), "column");
    assert_eq!(computed_style_property(html, "#c", "flex-wrap"), "wrap");
    assert_eq!(computed_style_property(html, "#c", "justify-content"), "space-between");
    assert_eq!(computed_style_property(html, "#c", "align-items"), "center");
    assert_eq!(computed_style_property(html, "#c", "align-self"), "flex-end");
    assert_eq!(computed_style_property(html, "#c", "writing-mode"), "vertical-rl");
    assert_eq!(computed_style_property(html, "#c", "object-fit"), "cover");
    assert_eq!(computed_style_property(html, "#c", "isolation"), "isolate");
    assert_eq!(computed_style_property(html, "#c", "mix-blend-mode"), "multiply");
    assert_eq!(computed_style_property(html, "#c", "pointer-events"), "none");
    assert_eq!(computed_style_property(html, "#c", "user-select"), "all");
    // list-style。
    assert_eq!(computed_style_property(html, "#l", "list-style-type"), "lower-alpha");
    assert_eq!(computed_style_property(html, "#l", "list-style-position"), "inside");

    // 默认值（ZeroWeb initial：justify-content=flex-start、align-items=stretch、align-self=auto；
    // 注：Chromium Box Align 3 initial 为 normal，ZeroWeb default 取 flex-start/stretch，diverge）。
    assert_eq!(computed_style_property(html, "#plain", "flex-direction"), "row");
    assert_eq!(computed_style_property(html, "#plain", "flex-wrap"), "nowrap");
    assert_eq!(computed_style_property(html, "#plain", "justify-content"), "flex-start");
    assert_eq!(computed_style_property(html, "#plain", "align-items"), "stretch");
    assert_eq!(computed_style_property(html, "#plain", "align-self"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "writing-mode"), "horizontal-tb");
    assert_eq!(computed_style_property(html, "#plain", "object-fit"), "fill");
    assert_eq!(computed_style_property(html, "#plain", "isolation"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "mix-blend-mode"), "normal");
    assert_eq!(computed_style_property(html, "#plain", "pointer-events"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "user-select"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "list-style-type"), "disc");
    assert_eq!(
        computed_style_property(html, "#plain", "list-style-position"),
        "outside"
    );
}

#[test]
fn test_get_computed_style_numeric_special() {
    // R2711：getComputedStyle 数值/special 族（flex-grow/flex-shrink/order/flex-basis/aspect-ratio）。
    let html = "<html><body>\
        <div id=\"n\" style=\"\
            flex-grow: 2.5; flex-shrink: 0; order: 3; \
            flex-basis: 120px; aspect-ratio: 16 / 9; \
        \"></div>\
        <div id=\"plain\"></div>\
        </body></html>";

    // 显式数值/special。
    assert_eq!(computed_style_property(html, "#n", "flex-grow"), "2.5");
    assert_eq!(computed_style_property(html, "#n", "flex-shrink"), "0");
    assert_eq!(computed_style_property(html, "#n", "order"), "3");
    assert_eq!(computed_style_property(html, "#n", "flex-basis"), "120px");
    // aspect-ratio: ZeroWeb 只存合并比值 → 数值（Chrome 返 "16 / 9"，diverge，已记 known-limitation）。
    assert_eq!(computed_style_property(html, "#n", "aspect-ratio"), "1.778");

    // 默认值（ZeroWeb initial：flex-grow=0、flex-shrink=1、order=0、flex-basis=auto、aspect-ratio=auto）。
    assert_eq!(computed_style_property(html, "#plain", "flex-grow"), "0");
    assert_eq!(computed_style_property(html, "#plain", "flex-shrink"), "1");
    assert_eq!(computed_style_property(html, "#plain", "order"), "0");
    assert_eq!(computed_style_property(html, "#plain", "flex-basis"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "aspect-ratio"), "auto");
}

#[test]
fn test_get_computed_style_transform() {
    // R2715：getComputedStyle transform 序列化（CSS Transforms L1/L2 计算值 = 函数列表）。
    // Chromium 返 resolved matrix（diverge）；ZeroWeb 返 parsed 函数列表（spec-correct + Firefox 一致）。
    let html = "<html><body>\
        <div id=\"t\" style=\"transform: translate(10px, 20px) rotate(45deg) scale(2);\"></div>\
        <div id=\"pct\" style=\"transform: translateX(50%);\"></div>\
        <div id=\"none\"></div>\
        </body></html>";
    // 组合：translate + rotate + scale（空格分隔函数列表）。
    assert_eq!(
        computed_style_property(html, "#t", "transform"),
        "translate(10px, 20px) rotate(45deg) scale(2)"
    );
    // 百分比 translate 保留（border-box 相对，须 layout 故保 %）。
    assert_eq!(computed_style_property(html, "#pct", "transform"), "translateX(50%)");
    // 默认 none。
    assert_eq!(computed_style_property(html, "#none", "transform"), "none");
}

#[test]
fn test_get_computed_style_transform_origin() {
    // R2716：getComputedStyle transform-origin 序列化（2 LengthValue，空格连接）。
    // Chromium 返 used 值（border-box 中心绝对 px，diverge）；ZeroWeb 返计算值（spec-correct + Firefox 一致）。
    let html = "<html><body>\
        <div id=\"px\" style=\"transform-origin: 10px 20px;\"></div>\
        <div id=\"pct\" style=\"transform-origin: 25% 75%;\"></div>\
        <div id=\"center\" style=\"transform-origin: center;\"></div>\
        <div id=\"single\" style=\"transform-origin: 0px;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // 显式 px（computed == used，与 real browser 一致）。
    assert_eq!(computed_style_property(html, "#px", "transform-origin"), "10px 20px");
    // 显式百分比保留为计算值（Chromium 返 used px，diverge）。
    assert_eq!(computed_style_property(html, "#pct", "transform-origin"), "25% 75%");
    // 关键字 center 计算值 = 50% 50%（apply 未解析关键字降级为默认，恰等于 center 计算值，行为正确）。
    assert_eq!(computed_style_property(html, "#center", "transform-origin"), "50% 50%");
    // 单值：x 指定，y 默认 50%。
    assert_eq!(computed_style_property(html, "#single", "transform-origin"), "0px 50%");
    // 默认值 50% 50%。
    assert_eq!(computed_style_property(html, "#def", "transform-origin"), "50% 50%");
}

#[test]
fn test_get_computed_style_contain() {
    // R2717：getComputedStyle contain 序列化（CSS Containment L1/L2 计算值）。
    // Strict/Content 保留 shorthand 不展开（与 Chromium 一致）；组合值按 spec 语法序 size/layout/paint/style。
    let html = "<html><body>\
        <div id=\"none\" style=\"contain: none;\"></div>\
        <div id=\"strict\" style=\"contain: strict;\"></div>\
        <div id=\"content\" style=\"contain: content;\"></div>\
        <div id=\"single\" style=\"contain: layout;\"></div>\
        <div id=\"combo\" style=\"contain: layout paint;\"></div>\
        <div id=\"size-style\" style=\"contain: size style;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // none（默认）。
    assert_eq!(computed_style_property(html, "#none", "contain"), "none");
    // shorthand 保留。
    assert_eq!(computed_style_property(html, "#strict", "contain"), "strict");
    assert_eq!(computed_style_property(html, "#content", "contain"), "content");
    // 单关键字。
    assert_eq!(computed_style_property(html, "#single", "contain"), "layout");
    // 组合：位掩码解码，spec 语法序（layout paint）。
    assert_eq!(computed_style_property(html, "#combo", "contain"), "layout paint");
    // 组合：size + style（非连续位）按语法序 size 在前。
    assert_eq!(computed_style_property(html, "#size-style", "contain"), "size style");
    // 默认 none。
    assert_eq!(computed_style_property(html, "#def", "contain"), "none");
}

#[test]
fn test_get_computed_style_filter() {
    // R2718：getComputedStyle filter 序列化（CSS Filter Effects 函数列表，空格分隔）。
    let html = "<html><body>\
        <div id=\"none\" style=\"filter: none;\"></div>\
        <div id=\"blur\" style=\"filter: blur(5px);\"></div>\
        <div id=\"combo\" style=\"filter: brightness(1.5) contrast(0.8);\"></div>\
        <div id=\"hue\" style=\"filter: hue-rotate(90deg);\"></div>\
        <div id=\"shadow\" style=\"filter: drop-shadow(2px 4px 6px red);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // none（显式与默认均为空 Vec）。
    assert_eq!(computed_style_property(html, "#none", "filter"), "none");
    // 单函数：blur 长度为 px。
    assert_eq!(computed_style_property(html, "#blur", "filter"), "blur(5px)");
    // 多函数组合：空格分隔，数值函数无单位。
    assert_eq!(
        computed_style_property(html, "#combo", "filter"),
        "brightness(1.5) contrast(0.8)"
    );
    // hue-rotate 角度为 deg。
    assert_eq!(computed_style_property(html, "#hue", "filter"), "hue-rotate(90deg)");
    // drop-shadow：3 长度 px + 颜色解析为 rgb()。
    assert_eq!(
        computed_style_property(html, "#shadow", "filter"),
        "drop-shadow(2px 4px 6px rgb(255, 0, 0))"
    );
    // 默认 none。
    assert_eq!(computed_style_property(html, "#def", "filter"), "none");
}

#[test]
fn test_get_computed_style_transform_family() {
    // R2719：getComputedStyle 3D transform 簇（transform-style / backface-visibility / perspective /
    // perspective-origin，完成 R2715/R2716 启动的 transform 簇）。
    let html = "<html><body>\
        <div id=\"ts-3d\" style=\"transform-style: preserve-3d;\"></div>\
        <div id=\"bv-hidden\" style=\"backface-visibility: hidden;\"></div>\
        <div id=\"persp\" style=\"perspective: 800px;\"></div>\
        <div id=\"po\" style=\"perspective-origin: 25% 75%;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // transform-style：默认 flat，显式 preserve-3d。
    assert_eq!(
        computed_style_property(html, "#ts-3d", "transform-style"),
        "preserve-3d"
    );
    assert_eq!(computed_style_property(html, "#def", "transform-style"), "flat");
    // backface-visibility：默认 visible，显式 hidden。
    assert_eq!(
        computed_style_property(html, "#bv-hidden", "backface-visibility"),
        "hidden"
    );
    assert_eq!(computed_style_property(html, "#def", "backface-visibility"), "visible");
    // perspective：默认 none（Px(0.0)），显式 px。
    assert_eq!(computed_style_property(html, "#persp", "perspective"), "800px");
    assert_eq!(computed_style_property(html, "#def", "perspective"), "none");
    // perspective-origin：默认 50% 50%，显式百分比保留。
    assert_eq!(computed_style_property(html, "#po", "perspective-origin"), "25% 75%");
    assert_eq!(computed_style_property(html, "#def", "perspective-origin"), "50% 50%");
}

#[test]
fn test_get_computed_style_will_change() {
    // R2720：getComputedStyle will-change 序列化（CSS Will Change 列表，perf hint 常查）。
    let html = "<html><body>\
        <div id=\"auto\" style=\"will-change: auto;\"></div>\
        <div id=\"scroll\" style=\"will-change: scroll-position;\"></div>\
        <div id=\"contents\" style=\"will-change: contents;\"></div>\
        <div id=\"custom\" style=\"will-change: transform;\"></div>\
        <div id=\"multi\" style=\"will-change: transform opacity;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // auto（显式与默认均为空 Vec）。
    assert_eq!(computed_style_property(html, "#auto", "will-change"), "auto");
    assert_eq!(computed_style_property(html, "#def", "will-change"), "auto");
    // 关键字标识符。
    assert_eq!(
        computed_style_property(html, "#scroll", "will-change"),
        "scroll-position"
    );
    assert_eq!(computed_style_property(html, "#contents", "will-change"), "contents");
    // 自定义属性名原样。
    assert_eq!(computed_style_property(html, "#custom", "will-change"), "transform");
    // 多属性组合：空格分隔。
    assert_eq!(
        computed_style_property(html, "#multi", "will-change"),
        "transform opacity"
    );
}

#[test]
fn test_get_computed_style_clip_path() {
    // R2721：getComputedStyle clip-path 序列化（CSS Masking basic-shape 函数）。
    let html = "<html><body>\
        <div id=\"none\" style=\"clip-path: none;\"></div>\
        <div id=\"inset1\" style=\"clip-path: inset(10%);\"></div>\
        <div id=\"inset2\" style=\"clip-path: inset(10% 20%);\"></div>\
        <div id=\"inset-round\" style=\"clip-path: inset(5px round 10px);\"></div>\
        <div id=\"circle\" style=\"clip-path: circle(50px at 25% 75%);\"></div>\
        <div id=\"circle-def\" style=\"clip-path: circle();\"></div>\
        <div id=\"polygon\" style=\"clip-path: polygon(0% 0%, 100% 0%, 50% 100%);\"></div>\
        <div id=\"polygon-ee\" style=\"clip-path: polygon(evenodd, 0% 0%, 100% 0%, 50% 100%);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // none。
    assert_eq!(computed_style_property(html, "#none", "clip-path"), "none");
    assert_eq!(computed_style_property(html, "#def", "clip-path"), "none");
    // inset 单值折叠（解析展开 4 值全等 → 重新折叠为 1 值）。
    assert_eq!(computed_style_property(html, "#inset1", "clip-path"), "inset(10%)");
    // inset 双值（top==bottom, left==right）。
    assert_eq!(computed_style_property(html, "#inset2", "clip-path"), "inset(10% 20%)");
    // inset + round（圆角半径）。
    assert_eq!(
        computed_style_property(html, "#inset-round", "clip-path"),
        "inset(5px round 10px)"
    );
    // circle 半径 + at 位置。
    assert_eq!(
        computed_style_property(html, "#circle", "clip-path"),
        "circle(50px at 25% 75%)"
    );
    // circle() 空（默认 closest-side，无位置）。
    assert_eq!(
        computed_style_property(html, "#circle-def", "clip-path"),
        "circle(closest-side)"
    );
    // polygon 默认 nonzero 省略填充规则，顶点逗号分隔。
    assert_eq!(
        computed_style_property(html, "#polygon", "clip-path"),
        "polygon(0% 0%, 100% 0%, 50% 100%)"
    );
    // polygon evenodd 输出填充规则。
    assert_eq!(
        computed_style_property(html, "#polygon-ee", "clip-path"),
        "polygon(evenodd, 0% 0%, 100% 0%, 50% 100%)"
    );
}

#[test]
fn test_get_computed_style_content() {
    // R2722：getComputedStyle content 序列化（CSS Generated Content，::before/::after 生成内容）。
    let html = "<html><body>\
        <div id=\"none\" style=\"content: none;\"></div>\
        <div id=\"str\" style=\"content: 'hello';\"></div>\
        <div id=\"counter\" style=\"content: counter(c);\"></div>\
        <div id=\"counter-style\" style=\"content: counter(c, upper-roman);\"></div>\
        <div id=\"counters\" style=\"content: counters(n, '.');\"></div>\
        <div id=\"url\" style=\"content: url(x.png);\"></div>\
        <div id=\"list\" style=\"content: 'Chapter ' counter(c);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // none / normal（默认）。
    assert_eq!(computed_style_property(html, "#none", "content"), "none");
    assert_eq!(computed_style_property(html, "#def", "content"), "normal");
    // 字符串：双引号包裹。
    assert_eq!(computed_style_property(html, "#str", "content"), "\"hello\"");
    // counter(name) / counter(name, style)。
    assert_eq!(computed_style_property(html, "#counter", "content"), "counter(c)");
    assert_eq!(
        computed_style_property(html, "#counter-style", "content"),
        "counter(c, upper-roman)"
    );
    // counters(name, "sep")：分隔符引号串化。
    assert_eq!(
        computed_style_property(html, "#counters", "content"),
        "counters(n, \".\")"
    );
    // url(...)。
    assert_eq!(computed_style_property(html, "#url", "content"), "url(x.png)");
    // 多 component value 列表：空格连接。
    assert_eq!(
        computed_style_property(html, "#list", "content"),
        "\"Chapter \" counter(c)"
    );
}

#[test]
fn test_get_computed_style_font_weight_bolder_lighter() {
    // R2723：getComputedStyle bolder/lighter 按父链 resolved 绝对值解析（CSS Fonts 3 §5.2，
    // 对齐 Chromium；ZeroWeb 保关键字供 paint 二值 want_bold，仅 gCS 路径解析）。
    let html = "<html><body>\
        <b id=\"bolder-normal\" style=\"font-weight: bolder\"></b>\
        <div style=\"font-weight: bold\"><b id=\"bolder-bold\" style=\"font-weight: bolder\"></b></div>\
        <div style=\"font-weight: bold\"><span id=\"lighter-bold\" style=\"font-weight: lighter\"></span></div>\
        <span id=\"lighter-normal\" style=\"font-weight: lighter\"></span>\
        <div id=\"explicit\" style=\"font-weight: 500\"></div>\
        </body></html>";
    // bolder on normal(400) parent → 700。
    assert_eq!(computed_style_property(html, "#bolder-normal", "font-weight"), "700");
    // bolder on bold(700) parent → 900。
    assert_eq!(computed_style_property(html, "#bolder-bold", "font-weight"), "900");
    // lighter on bold(700) parent → 400。
    assert_eq!(computed_style_property(html, "#lighter-bold", "font-weight"), "400");
    // lighter on normal(400) parent → 100。
    assert_eq!(computed_style_property(html, "#lighter-normal", "font-weight"), "100");
    // 非 bolder/lighter 不受影响（显式数值原样）。
    assert_eq!(computed_style_property(html, "#explicit", "font-weight"), "500");
}

#[test]
fn test_get_computed_style_background_position() {
    // R2724：getComputedStyle background-position 序列化（CSS Backgrounds <bg-position># 多层）。
    // Chromium 解析关键字为百分比（WPT background-computed.html），单关键字按轴展开（缺省轴 center 50%）。
    let html = "<html><body>\
        <div id=\"center\" style=\"background-position: center;\"></div>\
        <div id=\"lt\" style=\"background-position: left top;\"></div>\
        <div id=\"rb\" style=\"background-position: right bottom;\"></div>\
        <div id=\"px\" style=\"background-position: 10px 20px;\"></div>\
        <div id=\"pct\" style=\"background-position: 25% 75%;\"></div>\
        <div id=\"multi\" style=\"background-position: center, left top;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // 默认 0% 0%（TwoValue(Percent 0, Percent 0)）。
    assert_eq!(computed_style_property(html, "#def", "background-position"), "0% 0%");
    // 单关键字 center → 两轴展开 50% 50%。
    assert_eq!(
        computed_style_property(html, "#center", "background-position"),
        "50% 50%"
    );
    // TwoValue 关键字 → 解析为 %。
    assert_eq!(computed_style_property(html, "#lt", "background-position"), "0% 0%");
    assert_eq!(computed_style_property(html, "#rb", "background-position"), "100% 100%");
    // TwoValue 长度 → px。
    assert_eq!(computed_style_property(html, "#px", "background-position"), "10px 20px");
    // TwoValue 百分比 → %。
    assert_eq!(computed_style_property(html, "#pct", "background-position"), "25% 75%");
    // 多背景层：逗号分隔。
    assert_eq!(
        computed_style_property(html, "#multi", "background-position"),
        "50% 50%, 0% 0%"
    );
}

#[test]
fn test_get_computed_style_background_size_repeat() {
    // R2725：getComputedStyle background-size + background-repeat 序列化（CSS Backgrounds 多层）。
    let html = "<html><body>\
        <div id=\"size-cover\" style=\"background-size: cover;\"></div>\
        <div id=\"size-px\" style=\"background-size: 100px;\"></div>\
        <div id=\"size-multi\" style=\"background-size: 50%, auto;\"></div>\
        <div id=\"repeat-x\" style=\"background-repeat: repeat-x;\"></div>\
        <div id=\"repeat-multi\" style=\"background-repeat: no-repeat, space;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // background-size 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "background-size"), "auto");
    assert_eq!(computed_style_property(html, "#size-cover", "background-size"), "cover");
    assert_eq!(computed_style_property(html, "#size-px", "background-size"), "100px");
    // 多层逗号分隔。
    assert_eq!(
        computed_style_property(html, "#size-multi", "background-size"),
        "50%, auto"
    );
    // background-repeat 默认 repeat。
    assert_eq!(computed_style_property(html, "#def", "background-repeat"), "repeat");
    assert_eq!(
        computed_style_property(html, "#repeat-x", "background-repeat"),
        "repeat-x"
    );
    // 多层逗号分隔。
    assert_eq!(
        computed_style_property(html, "#repeat-multi", "background-repeat"),
        "no-repeat, space"
    );
}

#[test]
fn test_get_computed_style_background_attachment_clip_origin() {
    // R2726：getComputedStyle background-attachment/clip/origin 序列化（单值 box-model 枚举）。
    let html = "<html><body>\
        <div id=\"att-fixed\" style=\"background-attachment: fixed;\"></div>\
        <div id=\"att-local\" style=\"background-attachment: local;\"></div>\
        <div id=\"clip-pad\" style=\"background-clip: padding-box;\"></div>\
        <div id=\"clip-content\" style=\"background-clip: content-box;\"></div>\
        <div id=\"clip-text\" style=\"background-clip: text;\"></div>\
        <div id=\"origin-border\" style=\"background-origin: border-box;\"></div>\
        <div id=\"origin-content\" style=\"background-origin: content-box;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // background-attachment 默认 scroll。
    assert_eq!(computed_style_property(html, "#def", "background-attachment"), "scroll");
    assert_eq!(
        computed_style_property(html, "#att-fixed", "background-attachment"),
        "fixed"
    );
    assert_eq!(
        computed_style_property(html, "#att-local", "background-attachment"),
        "local"
    );
    // background-clip 默认 border-box。
    assert_eq!(computed_style_property(html, "#def", "background-clip"), "border-box");
    assert_eq!(
        computed_style_property(html, "#clip-pad", "background-clip"),
        "padding-box"
    );
    assert_eq!(
        computed_style_property(html, "#clip-content", "background-clip"),
        "content-box"
    );
    assert_eq!(computed_style_property(html, "#clip-text", "background-clip"), "text");
    // background-origin 默认 padding-box（注意：与 clip 的 border-box 默认不同）。
    assert_eq!(
        computed_style_property(html, "#def", "background-origin"),
        "padding-box"
    );
    assert_eq!(
        computed_style_property(html, "#origin-border", "background-origin"),
        "border-box"
    );
    assert_eq!(
        computed_style_property(html, "#origin-content", "background-origin"),
        "content-box"
    );
}

#[test]
fn test_get_computed_style_alignment_cluster() {
    // R2727：getComputedStyle align-content/justify-items/justify-self 序列化（Box Alignment 簇补齐）。
    let html = "<html><body>\
        <div id=\"ac-center\" style=\"align-content: center;\"></div>\
        <div id=\"ac-between\" style=\"align-content: space-between;\"></div>\
        <div id=\"ji-start\" style=\"justify-items: start;\"></div>\
        <div id=\"ji-right\" style=\"justify-items: right;\"></div>\
        <div id=\"js-end\" style=\"justify-self: end;\"></div>\
        <div id=\"js-stretch\" style=\"justify-self: stretch;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // align-content 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "align-content"), "normal");
    assert_eq!(computed_style_property(html, "#ac-center", "align-content"), "center");
    assert_eq!(
        computed_style_property(html, "#ac-between", "align-content"),
        "space-between"
    );
    // justify-items 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "justify-items"), "normal");
    assert_eq!(computed_style_property(html, "#ji-start", "justify-items"), "start");
    assert_eq!(computed_style_property(html, "#ji-right", "justify-items"), "right");
    // justify-self 默认 auto（注意：与 justify-items 的 normal 默认不同）。
    assert_eq!(computed_style_property(html, "#def", "justify-self"), "auto");
    assert_eq!(computed_style_property(html, "#js-end", "justify-self"), "end");
    assert_eq!(computed_style_property(html, "#js-stretch", "justify-self"), "stretch");
}

#[test]
fn test_get_computed_style_text_break_cluster() {
    // R2728：getComputedStyle word-break/overflow-wrap/hyphens/line-break 序列化（CSS Text 换行/断词簇）。
    let html = "<html><body>\
        <div id=\"wb-all\" style=\"word-break: break-all;\"></div>\
        <div id=\"wb-keep\" style=\"word-break: keep-all;\"></div>\
        <div id=\"ow-word\" style=\"overflow-wrap: break-word;\"></div>\
        <div id=\"ow-any\" style=\"overflow-wrap: anywhere;\"></div>\
        <div id=\"hyph-auto\" style=\"hyphens: auto;\"></div>\
        <div id=\"hyph-manual\" style=\"hyphens: manual;\"></div>\
        <div id=\"lb-strict\" style=\"line-break: strict;\"></div>\
        <div id=\"lb-anywhere\" style=\"line-break: anywhere;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // word-break 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "word-break"), "normal");
    assert_eq!(computed_style_property(html, "#wb-all", "word-break"), "break-all");
    assert_eq!(computed_style_property(html, "#wb-keep", "word-break"), "keep-all");
    // overflow-wrap 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "overflow-wrap"), "normal");
    assert_eq!(computed_style_property(html, "#ow-word", "overflow-wrap"), "break-word");
    assert_eq!(computed_style_property(html, "#ow-any", "overflow-wrap"), "anywhere");
    // hyphens：ZeroWeb 默认 none（diverge：CSS 规范/Chromium 初值 manual）。
    assert_eq!(computed_style_property(html, "#def", "hyphens"), "none");
    assert_eq!(computed_style_property(html, "#hyph-auto", "hyphens"), "auto");
    assert_eq!(computed_style_property(html, "#hyph-manual", "hyphens"), "manual");
    // line-break 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "line-break"), "auto");
    assert_eq!(computed_style_property(html, "#lb-strict", "line-break"), "strict");
    assert_eq!(computed_style_property(html, "#lb-anywhere", "line-break"), "anywhere");
}

#[test]
fn test_get_computed_style_va_bidi_empty() {
    // R2729：getComputedStyle vertical-align/unicode-bidi/empty-cells 序列化（单值关键字枚举）。
    let html = "<html><body>\
        <div id=\"va-middle\" style=\"vertical-align: middle;\"></div>\
        <div id=\"va-text-top\" style=\"vertical-align: text-top;\"></div>\
        <div id=\"va-sub\" style=\"vertical-align: sub;\"></div>\
        <div id=\"ub-isolate\" style=\"unicode-bidi: isolate;\"></div>\
        <div id=\"ub-plaintext\" style=\"unicode-bidi: plaintext;\"></div>\
        <div id=\"ec-hide\" style=\"empty-cells: hide;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // vertical-align 默认 baseline。
    assert_eq!(computed_style_property(html, "#def", "vertical-align"), "baseline");
    assert_eq!(computed_style_property(html, "#va-middle", "vertical-align"), "middle");
    assert_eq!(
        computed_style_property(html, "#va-text-top", "vertical-align"),
        "text-top"
    );
    assert_eq!(computed_style_property(html, "#va-sub", "vertical-align"), "sub");
    // unicode-bidi 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "unicode-bidi"), "normal");
    assert_eq!(computed_style_property(html, "#ub-isolate", "unicode-bidi"), "isolate");
    assert_eq!(
        computed_style_property(html, "#ub-plaintext", "unicode-bidi"),
        "plaintext"
    );
    // empty-cells 默认 show。
    assert_eq!(computed_style_property(html, "#def", "empty-cells"), "show");
    assert_eq!(computed_style_property(html, "#ec-hide", "empty-cells"), "hide");
}

#[test]
fn test_get_computed_style_caret_accent_color() {
    // R2730：getComputedStyle caret-color + accent-color 序列化（CSS UI 颜色 auto | <color>）。
    let html = "<html><body>\
        <div id=\"cc-red\" style=\"caret-color: red;\"></div>\
        <div id=\"cc-cc\" style=\"color: blue; caret-color: currentcolor;\"></div>\
        <div id=\"ac-green\" style=\"accent-color: #00ff00;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // caret-color 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "caret-color"), "auto");
    assert_eq!(
        computed_style_property(html, "#cc-red", "caret-color"),
        "rgb(255, 0, 0)"
    );
    // currentcolor 解析为元素自身 color（blue → rgb(0,0,255)）。
    assert_eq!(computed_style_property(html, "#cc-cc", "caret-color"), "rgb(0, 0, 255)");
    // accent-color 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "accent-color"), "auto");
    assert_eq!(
        computed_style_property(html, "#ac-green", "accent-color"),
        "rgb(0, 255, 0)"
    );
}

#[test]
fn test_get_computed_style_misc_ui() {
    // R2731：getComputedStyle text-wrap/text-align-last/resize/appearance 序列化（misc 单值关键字枚举）。
    let html = "<html><body>\
        <div id=\"tw-balance\" style=\"text-wrap: balance;\"></div>\
        <div id=\"tw-pretty\" style=\"text-wrap: pretty;\"></div>\
        <div id=\"tal-justify\" style=\"text-align-last: justify;\"></div>\
        <div id=\"tal-right\" style=\"text-align-last: right;\"></div>\
        <div id=\"rz-both\" style=\"resize: both;\"></div>\
        <div id=\"rz-horizontal\" style=\"resize: horizontal;\"></div>\
        <div id=\"ap-none\" style=\"appearance: none;\"></div>\
        <div id=\"ap-textfield\" style=\"appearance: textfield;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // text-wrap 默认 wrap。
    assert_eq!(computed_style_property(html, "#def", "text-wrap"), "wrap");
    assert_eq!(computed_style_property(html, "#tw-balance", "text-wrap"), "balance");
    assert_eq!(computed_style_property(html, "#tw-pretty", "text-wrap"), "pretty");
    // text-align-last 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "text-align-last"), "auto");
    assert_eq!(
        computed_style_property(html, "#tal-justify", "text-align-last"),
        "justify"
    );
    assert_eq!(computed_style_property(html, "#tal-right", "text-align-last"), "right");
    // resize 默认 none。
    assert_eq!(computed_style_property(html, "#def", "resize"), "none");
    assert_eq!(computed_style_property(html, "#rz-both", "resize"), "both");
    assert_eq!(computed_style_property(html, "#rz-horizontal", "resize"), "horizontal");
    // appearance 默认 auto；CamelCase→kebab（textfield 不变，slider-horizontal 会变）。
    assert_eq!(computed_style_property(html, "#def", "appearance"), "auto");
    assert_eq!(computed_style_property(html, "#ap-none", "appearance"), "none");
    assert_eq!(
        computed_style_property(html, "#ap-textfield", "appearance"),
        "textfield"
    );
}

#[test]
fn test_get_computed_style_container_ui() {
    // R2732：getComputedStyle box-decoration-break/scrollbar-*/touch-action 序列化（容器交互/UI 枚举）。
    let html = "<html><body>\
        <div id=\"bdb-clone\" style=\"box-decoration-break: clone;\"></div>\
        <div id=\"sw-thin\" style=\"scrollbar-width: thin;\"></div>\
        <div id=\"sg-stable\" style=\"scrollbar-gutter: stable;\"></div>\
        <div id=\"sg-both\" style=\"scrollbar-gutter: stable both-edges;\"></div>\
        <div id=\"ta-panx\" style=\"touch-action: pan-x;\"></div>\
        <div id=\"ta-manip\" style=\"touch-action: manipulation;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // box-decoration-break 默认 slice。
    assert_eq!(computed_style_property(html, "#def", "box-decoration-break"), "slice");
    assert_eq!(
        computed_style_property(html, "#bdb-clone", "box-decoration-break"),
        "clone"
    );
    // scrollbar-width 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "scrollbar-width"), "auto");
    assert_eq!(computed_style_property(html, "#sw-thin", "scrollbar-width"), "thin");
    // scrollbar-gutter 默认 auto；stable / stable both-edges。
    assert_eq!(computed_style_property(html, "#def", "scrollbar-gutter"), "auto");
    assert_eq!(
        computed_style_property(html, "#sg-stable", "scrollbar-gutter"),
        "stable"
    );
    assert_eq!(
        computed_style_property(html, "#sg-both", "scrollbar-gutter"),
        "stable both-edges"
    );
    // touch-action 默认 auto；pan-x / manipulation。
    assert_eq!(computed_style_property(html, "#def", "touch-action"), "auto");
    assert_eq!(computed_style_property(html, "#ta-panx", "touch-action"), "pan-x");
    assert_eq!(
        computed_style_property(html, "#ta-manip", "touch-action"),
        "manipulation"
    );
}

#[test]
fn test_get_computed_style_outline_break() {
    // R2733：getComputedStyle outline-offset + break-* 序列化（补齐 outline 簇 + Fragmentation 簇）。
    let html = "<html><body>\
        <div id=\"oo-px\" style=\"outline-offset: 4px;\"></div>\
        <div id=\"oo-neg\" style=\"outline-offset: -2px;\"></div>\
        <div id=\"bb-avoid\" style=\"break-before: avoid;\"></div>\
        <div id=\"bb-column\" style=\"break-before: column;\"></div>\
        <div id=\"ba-avoid-page\" style=\"break-after: avoid-page;\"></div>\
        <div id=\"bi-avoid\" style=\"break-inside: avoid;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // outline-offset 默认 0px。
    assert_eq!(computed_style_property(html, "#def", "outline-offset"), "0px");
    assert_eq!(computed_style_property(html, "#oo-px", "outline-offset"), "4px");
    assert_eq!(computed_style_property(html, "#oo-neg", "outline-offset"), "-2px");
    // break-before 默认 auto；avoid / column。
    assert_eq!(computed_style_property(html, "#def", "break-before"), "auto");
    assert_eq!(computed_style_property(html, "#bb-avoid", "break-before"), "avoid");
    assert_eq!(computed_style_property(html, "#bb-column", "break-before"), "column");
    // break-after 默认 auto；avoid-page（CamelCase→kebab）。
    assert_eq!(computed_style_property(html, "#def", "break-after"), "auto");
    assert_eq!(
        computed_style_property(html, "#ba-avoid-page", "break-after"),
        "avoid-page"
    );
    // break-inside 默认 auto；avoid。
    assert_eq!(computed_style_property(html, "#def", "break-inside"), "auto");
    assert_eq!(computed_style_property(html, "#bi-avoid", "break-inside"), "avoid");
}

#[test]
fn test_get_computed_style_grid_container() {
    // R2734：getComputedStyle grid-auto-flow + container-type/name + tab-size 序列化。
    let html = "<html><body>\
        <div id=\"gaf-col\" style=\"grid-auto-flow: column;\"></div>\
        <div id=\"gaf-dense\" style=\"grid-auto-flow: dense;\"></div>\
        <div id=\"ct-size\" style=\"container-type: size;\"></div>\
        <div id=\"ct-inline\" style=\"container-type: inline-size;\"></div>\
        <div id=\"cn-named\" style=\"container-name: sidebar;\"></div>\
        <div id=\"ts-px\" style=\"tab-size: 24px;\"></div>\
        <div id=\"ts-num\" style=\"tab-size: 4;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // grid-auto-flow 默认 row；column / dense（ZeroWeb 解析 dense→RowDense 多词）。
    assert_eq!(computed_style_property(html, "#def", "grid-auto-flow"), "row");
    assert_eq!(computed_style_property(html, "#gaf-col", "grid-auto-flow"), "column");
    assert_eq!(
        computed_style_property(html, "#gaf-dense", "grid-auto-flow"),
        "row dense"
    );
    // container-type 默认 normal；size / inline-size。
    assert_eq!(computed_style_property(html, "#def", "container-type"), "normal");
    assert_eq!(computed_style_property(html, "#ct-size", "container-type"), "size");
    assert_eq!(
        computed_style_property(html, "#ct-inline", "container-type"),
        "inline-size"
    );
    // container-name 默认 none；显式字符串。
    assert_eq!(computed_style_property(html, "#def", "container-name"), "none");
    assert_eq!(computed_style_property(html, "#cn-named", "container-name"), "sidebar");
    // tab-size 默认 8（CSS 规范初值）；px / number。
    assert_eq!(computed_style_property(html, "#def", "tab-size"), "8");
    assert_eq!(computed_style_property(html, "#ts-px", "tab-size"), "24px");
    assert_eq!(computed_style_property(html, "#ts-num", "tab-size"), "4");
}

#[test]
fn test_get_computed_style_table_list_font() {
    // R2735：getComputedStyle border-spacing + list-style-image + font-size-adjust 序列化。
    let html = "<html><body>\
        <div id=\"bs-eq\" style=\"border-spacing: 5px;\"></div>\
        <div id=\"bs-diff\" style=\"border-spacing: 3px 8px;\"></div>\
        <div id=\"lsi-url\" style=\"list-style-image: url(star.png);\"></div>\
        <div id=\"fsa-num\" style=\"font-size-adjust: 0.5;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // border-spacing 默认 0px；等值单值 / 不等两值。
    assert_eq!(computed_style_property(html, "#def", "border-spacing"), "0px");
    assert_eq!(computed_style_property(html, "#bs-eq", "border-spacing"), "5px");
    assert_eq!(computed_style_property(html, "#bs-diff", "border-spacing"), "3px 8px");
    // list-style-image 默认 none；url() 引号形式。
    assert_eq!(computed_style_property(html, "#def", "list-style-image"), "none");
    assert_eq!(
        computed_style_property(html, "#lsi-url", "list-style-image"),
        "url(\"star.png\")"
    );
    // font-size-adjust 默认 none；number。
    assert_eq!(computed_style_property(html, "#def", "font-size-adjust"), "none");
    assert_eq!(computed_style_property(html, "#fsa-num", "font-size-adjust"), "0.5");
}

#[test]
fn test_get_computed_style_border_img_obj_pos_quotes() {
    // R2736：getComputedStyle border-image-source + object-position + quotes 序列化。
    let html = "<html><body>\
        <div id=\"bis-url\" style=\"border-image-source: url(border.png);\"></div>\
        <div id=\"op-kw\" style=\"object-position: top left;\"></div>\
        <div id=\"op-px\" style=\"object-position: 10px 20px;\"></div>\
        <div id=\"q-none\" style=\"quotes: none;\"></div>\
        <div id='q-pairs' style='quotes: \"\u{00ab}\" \"\u{00bb}\" \"\u{2039}\" \"\u{203a}\";'></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // border-image-source 默认 none；url() 引号形式（同 list-style-image）。
    assert_eq!(computed_style_property(html, "#def", "border-image-source"), "none");
    assert_eq!(
        computed_style_property(html, "#bis-url", "border-image-source"),
        "url(\"border.png\")"
    );
    // object-position 默认 Center→50% 50%；关键字两值 / 长度两值（复用 background-position 序列化）。
    assert_eq!(computed_style_property(html, "#def", "object-position"), "50% 50%");
    assert_eq!(computed_style_property(html, "#op-kw", "object-position"), "0% 0%");
    assert_eq!(computed_style_property(html, "#op-px", "object-position"), "10px 20px");
    // quotes 初值 auto；none；pairs→空格分隔双引号串。
    assert_eq!(computed_style_property(html, "#def", "quotes"), "auto");
    assert_eq!(computed_style_property(html, "#q-none", "quotes"), "none");
    assert_eq!(
        computed_style_property(html, "#q-pairs", "quotes"),
        "\"\u{00ab}\" \"\u{00bb}\" \"\u{2039}\" \"\u{203a}\""
    );
}

#[test]
fn test_get_computed_style_multicol_fontvar_img() {
    // R2737：getComputedStyle CSS Multi-column 簇（rule-width/style/color + count/width/fill/span）
    // + font-variant-numeric + image-rendering 序列化。column-gap 已由 R2707 长度族覆盖。
    let html = "<html><body>\
        <div id=\"cr\" style=\"column-rule: 2px dashed red;\"></div>\
        <div id=\"crt\" style=\"column-rule: thick solid blue;\"></div>\
        <div id=\"cc\" style=\"column-count: 3;\"></div>\
        <div id=\"cw\" style=\"column-width: 100px;\"></div>\
        <div id=\"cf\" style=\"column-fill: auto;\"></div>\
        <div id=\"cs\" style=\"column-span: all;\"></div>\
        <div id=\"fvn\" style=\"font-variant-numeric: tabular-nums;\"></div>\
        <div id=\"ir\" style=\"image-rendering: pixelated;\"></div>\
        <div id=\"def\" style=\"color: red;\"></div>\
        </body></html>";
    // column-rule-width：长度 2px；thick→5px（UA used px）；默认 style=none 仍 medium→3px（R2755 oracle：
    // column-rule-width 的 computed 值独立于 style，不套 border-width 的 none/hidden→0px 规则，纠正 R2737 误判）。
    assert_eq!(computed_style_property(html, "#cr", "column-rule-width"), "2px");
    assert_eq!(computed_style_property(html, "#crt", "column-rule-width"), "5px");
    assert_eq!(computed_style_property(html, "#def", "column-rule-width"), "3px");
    // column-rule-style：dashed/solid；默认 none。
    assert_eq!(computed_style_property(html, "#cr", "column-rule-style"), "dashed");
    assert_eq!(computed_style_property(html, "#crt", "column-rule-style"), "solid");
    assert_eq!(computed_style_property(html, "#def", "column-rule-style"), "none");
    // column-rule-color：显式 red/blue → rgb；默认 currentcolor → 元素 color（#def color:red）。
    assert_eq!(
        computed_style_property(html, "#cr", "column-rule-color"),
        "rgb(255, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#crt", "column-rule-color"),
        "rgb(0, 0, 255)"
    );
    assert_eq!(
        computed_style_property(html, "#def", "column-rule-color"),
        "rgb(255, 0, 0)"
    );
    // column-count：Number(3)→"3"；默认 auto。
    assert_eq!(computed_style_property(html, "#cc", "column-count"), "3");
    assert_eq!(computed_style_property(html, "#def", "column-count"), "auto");
    // column-width：100px；默认 auto。
    assert_eq!(computed_style_property(html, "#cw", "column-width"), "100px");
    assert_eq!(computed_style_property(html, "#def", "column-width"), "auto");
    // column-fill：auto；初值 balance。
    assert_eq!(computed_style_property(html, "#cf", "column-fill"), "auto");
    assert_eq!(computed_style_property(html, "#def", "column-fill"), "balance");
    // column-span：all；初值 none。
    assert_eq!(computed_style_property(html, "#cs", "column-span"), "all");
    assert_eq!(computed_style_property(html, "#def", "column-span"), "none");
    // font-variant-numeric：tabular-nums；初值 normal。
    assert_eq!(
        computed_style_property(html, "#fvn", "font-variant-numeric"),
        "tabular-nums"
    );
    assert_eq!(computed_style_property(html, "#def", "font-variant-numeric"), "normal");
    // image-rendering：pixelated；初值 auto。
    assert_eq!(computed_style_property(html, "#ir", "image-rendering"), "pixelated");
    assert_eq!(computed_style_property(html, "#def", "image-rendering"), "auto");
}

#[test]
fn test_get_computed_style_shorthands_r2755() {
    // R2755：getComputedStyle 残余简写序列化（columns / column-rule / list-style / text-decoration）。
    // 每项期望串经本地 Chromium 150 oracle 提取（--dump-dom 写 DOM 法），TDD red→green 对齐确切串。
    let html = "<html><body>\
        <div id=\"def\" style=\"color: red;\"></div>\
        <div id=\"cw\" style=\"column-width: 100px;\"></div>\
        <div id=\"cn\" style=\"column-count: 3;\"></div>\
        <div id=\"cb\" style=\"columns: 200px 4;\"></div>\
        <div id=\"cb2\" style=\"columns: 5;\"></div>\
        <div id=\"cr\" style=\"column-rule: thick double rgb(255, 0, 0);\"></div>\
        <div id=\"crp\" style=\"column-rule: 2px solid;\"></div>\
        <div id=\"crh\" style=\"column-rule-style: hidden;\"></div>\
        <div id=\"ls\" style=\"list-style: square inside;\"></div>\
        <div id=\"lsp\" style=\"list-style: lower-alpha outside;\"></div>\
        <div id=\"lsn\" style=\"list-style: none;\"></div>\
        <div id=\"td\" style=\"text-decoration: underline overline;\"></div>\
        <div id=\"tdl\" style=\"text-decoration: line-through;\"></div>\
        <div id=\"tdp\" style=\"text-decoration: underline dotted rgb(255, 0, 0);\"></div>\
        <div id=\"tdc\" style=\"text-decoration: underline; text-decoration-color: rgb(170, 187, 204);\"></div>\
        </body></html>";
    // columns 简写：auto 省略；全 auto→"auto"；width-only→"W"；count-only→"N"；both→"W N"。
    assert_eq!(computed_style_property(html, "#def", "columns"), "auto");
    assert_eq!(computed_style_property(html, "#cw", "columns"), "100px");
    assert_eq!(computed_style_property(html, "#cn", "columns"), "3");
    assert_eq!(computed_style_property(html, "#cb", "columns"), "200px 4");
    assert_eq!(computed_style_property(html, "#cb2", "columns"), "5");
    // column-rule 简写：style=none 省略（hidden 保留）；width 恒显；color 恒显。
    // #def 默认（style none）→"3px rgb(255, 0, 0)"（color=currentcolor→元素 red）。
    assert_eq!(
        computed_style_property(html, "#def", "column-rule"),
        "3px rgb(255, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#cr", "column-rule"),
        "5px double rgb(255, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#crp", "column-rule"),
        "2px solid rgb(0, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#crh", "column-rule"),
        "3px hidden rgb(0, 0, 0)"
    );
    // list-style 简写：恒 3 段 "position image type"。
    assert_eq!(computed_style_property(html, "#def", "list-style"), "outside none disc");
    assert_eq!(computed_style_property(html, "#ls", "list-style"), "inside none square");
    assert_eq!(
        computed_style_property(html, "#lsp", "list-style"),
        "outside none lower-alpha"
    );
    assert_eq!(computed_style_property(html, "#lsn", "list-style"), "outside none none");
    // text-decoration 简写：line=none→"none"；否则 line/thickness(!auto)/style(!solid)/color(!currentcolor)。
    assert_eq!(computed_style_property(html, "#def", "text-decoration"), "none");
    assert_eq!(
        computed_style_property(html, "#td", "text-decoration"),
        "underline overline"
    );
    assert_eq!(computed_style_property(html, "#tdl", "text-decoration"), "line-through");
    assert_eq!(
        computed_style_property(html, "#tdp", "text-decoration"),
        "underline dotted rgb(255, 0, 0)"
    );
    // #tdc：line + 显式 color（非 currentcolor）；style solid / thickness auto 省略。
    assert_eq!(
        computed_style_property(html, "#tdc", "text-decoration"),
        "underline rgb(170, 187, 204)"
    );
}

#[test]
fn test_get_computed_style_transition_animation_shorthand_r2756() {
    // R2756：getComputedStyle transition / animation 简写（CSSOM 列表 zip 重组）。
    // 每项期望串经本地 Chromium 150 oracle 提取（--dump-dom 写 DOM 法），TDD red→green 对齐。
    let html = "<html><body>\
        <div id=\"def\"></div>\
        <div id=\"tn\" style=\"transition: none;\"></div>\
        <div id=\"t1\" style=\"transition: margin 2s;\"></div>\
        <div id=\"t2\" style=\"transition: margin 2s ease-in 1s;\"></div>\
        <div id=\"t5\" style=\"transition: 2s;\"></div>\
        <div id=\"tm\" style=\"transition: margin 2s ease-in 1s, padding 0.5s;\"></div>\
        <div id=\"an\" style=\"animation: none;\"></div>\
        <div id=\"a1\" style=\"animation: bounce 2s;\"></div>\
        <div id=\"a2\" style=\"animation: bounce 2s linear infinite alternate;\"></div>\
        <div id=\"ad\" style=\"animation: 2s;\"></div>\
        <div id=\"ap\" style=\"animation: bounce paused;\"></div>\
        <div id=\"anm\" style=\"animation: bounce 2s ease-in 1s, spin 1s linear 2;\"></div>\
        </body></html>";
    // transition 简写：默认（空列表）→"all"；none→"none"；省初值（property=all 仅在其余全初值时显）。
    assert_eq!(computed_style_property(html, "#def", "transition"), "all");
    assert_eq!(computed_style_property(html, "#tn", "transition"), "none");
    assert_eq!(computed_style_property(html, "#t1", "transition"), "margin 2s");
    assert_eq!(
        computed_style_property(html, "#t2", "transition"),
        "margin 2s ease-in 1s"
    );
    // #t5：property=all（初值）省略，仅 duration 显。
    assert_eq!(computed_style_property(html, "#t5", "transition"), "2s");
    // 多条目逗号连接，逐索引 zip。
    assert_eq!(
        computed_style_property(html, "#tm", "transition"),
        "margin 2s ease-in 1s, padding 0.5s"
    );
    // animation 简写：默认（空列表）→"none"；none→"none"；顺序 dur/tf/delay/iter/dir/fill/play/name 省初值。
    assert_eq!(computed_style_property(html, "#def", "animation"), "none");
    assert_eq!(computed_style_property(html, "#an", "animation"), "none");
    assert_eq!(computed_style_property(html, "#a1", "animation"), "2s bounce");
    assert_eq!(
        computed_style_property(html, "#a2", "animation"),
        "2s linear infinite alternate bounce"
    );
    // #ad：name=none（初值）省略，仅 duration 显。
    assert_eq!(computed_style_property(html, "#ad", "animation"), "2s");
    // #ap：play-state=paused 显（running 初值省），duration 0s 省。
    assert_eq!(computed_style_property(html, "#ap", "animation"), "paused bounce");
    // 多条目逗号连接，逐索引 zip。
    assert_eq!(
        computed_style_property(html, "#anm", "animation"),
        "2s ease-in 1s bounce, 1s linear 2 spin"
    );
}

#[test]
fn test_get_computed_style_background_shorthand_r2757() {
    // R2757：getComputedStyle background 简写（CSSOM 完整规范形重组，无省略）。
    // 每项期望串经本地 Chromium 150 oracle 提取（--dump-dom 写 DOM 法），TDD red→green 对齐。
    // 注：避开 url() 图层（ZW 存相对 URL，oracle 解析绝对 URL，属 pre-existing longhand 差异）。
    // 注：attachment/box 经 **longhand** 设置——ZW 的 background 简写 parser 对含 rgb()/var() 的值
    // bail-out（整体作 color，丢 attachment），且 box token 故意 drop（R2479/R2481），故用 longhand
    // 隔离测试**序列化**正确性（本切片范围），不依赖简写 parser。
    let html = "<html><body>\
        <div id=\"def\"></div>\
        <div id=\"c\" style=\"background: rgb(255, 0, 0);\"></div>\
        <div id=\"fi\" style=\"background-color: rgb(0, 128, 0); background-attachment: fixed;\"></div>\
        <div id=\"oc\" style=\"background-origin: content-box; background-clip: padding-box;\"></div>\
        </body></html>";
    // background 简写恒完整规范形："<color> <image> <repeat> <attachment> <position> / <size> <origin> <clip>"。
    // 默认：transparent none repeat scroll 0% 0% / auto padding-box border-box。
    assert_eq!(
        computed_style_property(html, "#def", "background"),
        "rgba(0, 0, 0, 0) none repeat scroll 0% 0% / auto padding-box border-box"
    );
    // 纯色（简写声明）：color 改变，其余默认。
    assert_eq!(
        computed_style_property(html, "#c", "background"),
        "rgb(255, 0, 0) none repeat scroll 0% 0% / auto padding-box border-box"
    );
    // attachment=fixed（经 longhand 设置，测序列化）。
    assert_eq!(
        computed_style_property(html, "#fi", "background"),
        "rgb(0, 128, 0) none repeat fixed 0% 0% / auto padding-box border-box"
    );
    // origin/clip（origin 在前 clip 在后，即使相等也双显；经 longhand 设置，测序列化）。
    assert_eq!(
        computed_style_property(html, "#oc", "background"),
        "rgba(0, 0, 0, 0) none repeat scroll 0% 0% / auto content-box padding-box"
    );
}

#[test]
fn test_get_computed_style_place_shorthands_r2758() {
    // R2758：getComputedStyle place-content/items/self 简写（align+justify CSSOM 2 值最小化）。
    // 每项期望串经本地 Chromium 150 oracle 提取（--dump-dom 写 DOM 法），TDD red→green 对齐。
    // 注：place-content/items 默认值受 ZW layout-coupled 默认（justify-content FlexStart / align-items
    // Stretch vs Chromium normal）影响 diverge——测**显式设置**的值（含单值同值），序列化本身正确。
    let html = "<html><body>\
        <div id=\"pc1\" style=\"place-content: center;\"></div>\
        <div id=\"pc2\" style=\"place-content: center start;\"></div>\
        <div id=\"pc3\" style=\"place-content: space-between;\"></div>\
        <div id=\"pi1\" style=\"place-items: center;\"></div>\
        <div id=\"pi2\" style=\"place-items: center start;\"></div>\
        <div id=\"ps0\" style=\"color: red;\"></div>\
        <div id=\"ps1\" style=\"place-self: center;\"></div>\
        <div id=\"ps2\" style=\"place-self: start end;\"></div>\
        </body></html>";
    // place-content：align==justify→单值，否则 "align justify"。
    assert_eq!(computed_style_property(html, "#pc1", "place-content"), "center");
    assert_eq!(computed_style_property(html, "#pc2", "place-content"), "center start");
    assert_eq!(computed_style_property(html, "#pc3", "place-content"), "space-between");
    // place-items：同 2 值最小化。
    assert_eq!(computed_style_property(html, "#pi1", "place-items"), "center");
    assert_eq!(computed_style_property(html, "#pi2", "place-items"), "center start");
    // place-self：默认 align-self/justify-self 均 auto→"auto"（默认匹配 Chromium）。
    assert_eq!(computed_style_property(html, "#ps0", "place-self"), "auto");
    assert_eq!(computed_style_property(html, "#ps1", "place-self"), "center");
    assert_eq!(computed_style_property(html, "#ps2", "place-self"), "start end");
}

#[test]
fn test_get_computed_style_grid_lines_r2759() {
    // R2759：getComputedStyle grid 线定位 longhand（grid-column/row-start/end）+ 简写
    // （grid-column/row/area，CSSOM 最小化）。每项期望串经本地 Chromium 150 oracle 提取，TDD 对齐。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"cs\" style=\"grid-column-start: 2;\"></div>\
        <div id=\"cname\" style=\"grid-column-start: main;\"></div>\
        <div id=\"gc1\" style=\"grid-column: 2;\"></div>\
        <div id=\"gc2\" style=\"grid-column: 2 / 4;\"></div>\
        <div id=\"gc3\" style=\"grid-column: 1 / span 2;\"></div>\
        <div id=\"gc4\" style=\"grid-column: span 2;\"></div>\
        <div id=\"gr\" style=\"grid-row: span 3 / 5;\"></div>\
        <div id=\"ga1\" style=\"grid-area: 1 / 1 / 3 / 3;\"></div>\
        <div id=\"ga3\" style=\"grid-area: 2 / 3;\"></div>\
        </body></html>";
    // longhand：Auto→auto / Line(n)→n / Span(n)→span n / Name(s)→s。
    assert_eq!(computed_style_property(html, "#d", "grid-column-start"), "auto");
    assert_eq!(computed_style_property(html, "#cs", "grid-column-start"), "2");
    assert_eq!(computed_style_property(html, "#cname", "grid-column-start"), "main");
    assert_eq!(computed_style_property(html, "#gc4", "grid-column-start"), "span 2");
    // grid-column 简写：start==end→单值；end==auto 且 start 非 Name→单值；Name 保留 "name / auto"。
    assert_eq!(computed_style_property(html, "#d", "grid-column"), "auto");
    assert_eq!(computed_style_property(html, "#gc1", "grid-column"), "2");
    assert_eq!(computed_style_property(html, "#gc2", "grid-column"), "2 / 4");
    assert_eq!(computed_style_property(html, "#gc3", "grid-column"), "1 / span 2");
    assert_eq!(computed_style_property(html, "#gc4", "grid-column"), "span 2");
    assert_eq!(computed_style_property(html, "#cname", "grid-column"), "main / auto");
    // grid-row 简写：同 grid-column 规则。
    assert_eq!(computed_style_property(html, "#gr", "grid-row"), "span 3 / 5");
    // grid-area 简写：4 槽 trailing-drop 最小化。注：单值 `grid-area: header`（CSS 应四值同设）
    // 受 ZW expand_grid_area 仅设 row-start 的 pre-existing parser diverge 限——此处测 ZW 正确解析的
    // 4 值 / 2 值形式（序列化本身正确，单值 diverge 另记）。
    assert_eq!(computed_style_property(html, "#d", "grid-area"), "auto");
    assert_eq!(computed_style_property(html, "#ga1", "grid-area"), "1 / 1 / 3 / 3");
    assert_eq!(computed_style_property(html, "#ga3", "grid-area"), "2 / 3");
    // #gc1（cs=2，re/ce=auto，cs 非 Name）→grid-area drop ce/re→"auto / 2"。
    assert_eq!(computed_style_property(html, "#gc1", "grid-area"), "auto / 2");
    // #cname（cs=Name main，阻止 ce 省）→grid-area 全 4 槽 "auto / main / auto / auto"。
    assert_eq!(
        computed_style_property(html, "#cname", "grid-area"),
        "auto / main / auto / auto"
    );
    // #gr（rs=span3, re=5, ce=auto）→drop ce（cs=auto 非 Name），re=5 留→"span 3 / auto / 5"。
    assert_eq!(computed_style_property(html, "#gr", "grid-area"), "span 3 / auto / 5");
}

#[test]
fn test_get_computed_style_inset_shorthand_r2760() {
    // R2760：getComputedStyle inset 简写（top/right/bottom/left CSSOM 4 值最小化）。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。ZW 解析 inset 简写
    // （parse_rect_values），序列化复用 box_4_to_css（同 margin/padding/border-radius）。
    let html = "<html><body>\
        <div id=\"i1\" style=\"inset: 10px;\"></div>\
        <div id=\"i2\" style=\"inset: 10px 20px;\"></div>\
        <div id=\"i3\" style=\"inset: 10px 20px 30px;\"></div>\
        <div id=\"i4\" style=\"inset: 10px 20px 30px 40px;\"></div>\
        <div id=\"mix\" style=\"inset: 5px 5px 5px 5px;\"></div>\
        </body></html>";
    // inset 简写 = top/right/bottom/left 的 CSSOM 4 值最小化。
    assert_eq!(computed_style_property(html, "#i1", "inset"), "10px");
    assert_eq!(computed_style_property(html, "#i2", "inset"), "10px 20px");
    assert_eq!(computed_style_property(html, "#i3", "inset"), "10px 20px 30px");
    assert_eq!(computed_style_property(html, "#i4", "inset"), "10px 20px 30px 40px");
    // 全等→单值。
    assert_eq!(computed_style_property(html, "#mix", "inset"), "5px");
    // 经 longhand 设置非等值（验证简写重组，非仅依赖 shorthand 声明）。
    let html2 = "<html><body>\
        <div id=\"lh\" style=\"top: 1px; right: 2px; bottom: 3px; left: 4px;\"></div>\
        </body></html>";
    assert_eq!(computed_style_property(html2, "#lh", "inset"), "1px 2px 3px 4px");
}

#[test]
fn test_get_computed_style_border_radius_shorthand() {
    // R2738：getComputedStyle border-radius 简写（CSSOM 4 值最小化）。4 角 longhand 早覆（R2707）。
    let html = "<html><body>\
        <div id=\"br1\" style=\"border-radius: 5px;\"></div>\
        <div id=\"br2\" style=\"border-radius: 5px 10px;\"></div>\
        <div id=\"br3\" style=\"border-radius: 5px 10px 15px;\"></div>\
        <div id=\"br4\" style=\"border-radius: 5px 10px 15px 20px;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // 全等→1 值；TL==BR&&TR==BL→2 值；TR==BL→3 值；否则 4 值（CSSOM 同 margin 语法）。
    assert_eq!(computed_style_property(html, "#br1", "border-radius"), "5px");
    assert_eq!(computed_style_property(html, "#br2", "border-radius"), "5px 10px");
    assert_eq!(computed_style_property(html, "#br3", "border-radius"), "5px 10px 15px");
    assert_eq!(
        computed_style_property(html, "#br4", "border-radius"),
        "5px 10px 15px 20px"
    );
    // 默认 4 角均 0px → 最小化 "0px"（对齐 Chromium）。
    assert_eq!(computed_style_property(html, "#def", "border-radius"), "0px");
}

#[test]
fn test_get_computed_style_box_text_shadow() {
    // R2739：getComputedStyle box-shadow + text-shadow 序列化。
    // Chromium/WPT 格式：color 在前（currentcolor 经元素 color 解析）+ 全长度（box 4 长+inset / text 3 长），
    // 多阴影逗号分隔，空→none。格式锚定 WPT box-shadow-interpolation/composition 的 expect 串。
    let html = "<html><body>\
        <div id=\"bs\" style=\"box-shadow: 5px 5px;\"></div>\
        <div id=\"bsi\" style=\"box-shadow: inset 0 0 10px red;\"></div>\
        <div id=\"bss\" style=\"box-shadow: 1px 2px 3px 4px blue;\"></div>\
        <div id=\"bsm\" style=\"box-shadow: 1px 1px red, 2px 2px blue;\"></div>\
        <div id=\"cc\" style=\"color: green; box-shadow: 5px 5px;\"></div>\
        <div id=\"ts\" style=\"text-shadow: 2px 4px;\"></div>\
        <div id=\"tsc\" style=\"text-shadow: 0 0 10px red;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // box-shadow：color 在前（无 color→currentcolor 默认元素 color=black）+ ox oy blur spread 全含；inset 在末。
    assert_eq!(
        computed_style_property(html, "#bs", "box-shadow"),
        "rgb(0, 0, 0) 5px 5px 0px 0px"
    );
    assert_eq!(
        computed_style_property(html, "#bsi", "box-shadow"),
        "rgb(255, 0, 0) 0px 0px 10px 0px inset"
    );
    assert_eq!(
        computed_style_property(html, "#bss", "box-shadow"),
        "rgb(0, 0, 255) 1px 2px 3px 4px"
    );
    // 多阴影逗号分隔。
    assert_eq!(
        computed_style_property(html, "#bsm", "box-shadow"),
        "rgb(255, 0, 0) 1px 1px 0px 0px, rgb(0, 0, 255) 2px 2px 0px 0px"
    );
    // currentcolor 解析为元素 color（green→rgb(0,128,0)）。
    assert_eq!(
        computed_style_property(html, "#cc", "box-shadow"),
        "rgb(0, 128, 0) 5px 5px 0px 0px"
    );
    // text-shadow：color 在前 + ox oy blur 3 长（无 spread/inset）。
    assert_eq!(
        computed_style_property(html, "#ts", "text-shadow"),
        "rgb(0, 0, 0) 2px 4px 0px"
    );
    assert_eq!(
        computed_style_property(html, "#tsc", "text-shadow"),
        "rgb(255, 0, 0) 0px 0px 10px"
    );
    // 默认空列表→none。
    assert_eq!(computed_style_property(html, "#def", "box-shadow"), "none");
    assert_eq!(computed_style_property(html, "#def", "text-shadow"), "none");
}

#[test]
fn test_get_computed_style_grid_tracks() {
    // R2740：getComputedStyle grid 轨道簇序列化（Option<String> 存原始 specified 值）。
    let html = "<html><body>\
        <div id=\"gtc\" style=\"grid-template-columns: 1fr 1fr 1fr;\"></div>\
        <div id=\"gtc2\" style=\"grid-template-columns: 100px minmax(200px, 1fr);\"></div>\
        <div id=\"gtr\" style=\"grid-template-rows: 50px 50px;\"></div>\
        <div id=\"gac\" style=\"grid-auto-columns: 200px;\"></div>\
        <div id=\"gar\" style=\"grid-auto-rows: 100px;\"></div>\
        <div id='gta' style='grid-template-areas: \"a b\" \"c d\";'></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // grid-template-columns/rows：Some→原样 specified 串；None→none（CSS 初值）。
    assert_eq!(
        computed_style_property(html, "#gtc", "grid-template-columns"),
        "1fr 1fr 1fr"
    );
    assert_eq!(
        computed_style_property(html, "#gtc2", "grid-template-columns"),
        "100px minmax(200px, 1fr)"
    );
    assert_eq!(computed_style_property(html, "#gtr", "grid-template-rows"), "50px 50px");
    // grid-auto-columns/rows：Some→原样；None→auto（CSS Grid §6.4 初值，非 none）。
    assert_eq!(computed_style_property(html, "#gac", "grid-auto-columns"), "200px");
    assert_eq!(computed_style_property(html, "#gar", "grid-auto-rows"), "100px");
    // grid-template-areas：Some→原样 specified 串（含引号）。
    assert_eq!(
        computed_style_property(html, "#gta", "grid-template-areas"),
        "\"a b\" \"c d\""
    );
    // 默认：grid-template-* → none；grid-auto-* → auto。
    assert_eq!(computed_style_property(html, "#def", "grid-template-columns"), "none");
    assert_eq!(computed_style_property(html, "#def", "grid-auto-columns"), "auto");
    assert_eq!(computed_style_property(html, "#def", "grid-auto-rows"), "auto");
    assert_eq!(computed_style_property(html, "#def", "grid-template-areas"), "none");
}

#[test]
fn test_get_computed_style_containment() {
    // R2741：getComputedStyle containment 簇（content-visibility + contain-intrinsic-width/height）。
    let html = "<html><body>\
        <div id=\"cvh\" style=\"content-visibility: hidden;\"></div>\
        <div id=\"cva\" style=\"content-visibility: auto;\"></div>\
        <div id=\"ciw\" style=\"contain-intrinsic-width: 100px;\"></div>\
        <div id=\"cih\" style=\"contain-intrinsic-height: 50px;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // content-visibility：visible/hidden/auto（CSS Containment 2，初值 visible）。
    assert_eq!(computed_style_property(html, "#cvh", "content-visibility"), "hidden");
    assert_eq!(computed_style_property(html, "#cva", "content-visibility"), "auto");
    assert_eq!(computed_style_property(html, "#def", "content-visibility"), "visible");
    // contain-intrinsic-width/height：None→none（初值）；Some→px。
    assert_eq!(
        computed_style_property(html, "#ciw", "contain-intrinsic-width"),
        "100px"
    );
    assert_eq!(
        computed_style_property(html, "#cih", "contain-intrinsic-height"),
        "50px"
    );
    assert_eq!(computed_style_property(html, "#def", "contain-intrinsic-width"), "none");
    assert_eq!(
        computed_style_property(html, "#def", "contain-intrinsic-height"),
        "none"
    );
}

#[test]
fn test_get_computed_style_counter_actions() {
    // R2742：getComputedStyle counter-increment / counter-reset 序列化。
    let html = "<html><body>\
        <div id=\"ci\" style=\"counter-increment: h1;\"></div>\
        <div id=\"ci2\" style=\"counter-increment: c 2;\"></div>\
        <div id=\"cr\" style=\"counter-reset: sec;\"></div>\
        <div id=\"crm\" style=\"counter-reset: a 5 b 3;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // counter-increment：空格分隔 name integer；value 省略→默认 1。
    assert_eq!(computed_style_property(html, "#ci", "counter-increment"), "h1 1");
    assert_eq!(computed_style_property(html, "#ci2", "counter-increment"), "c 2");
    // counter-reset：value 省略→默认 0；多计数器空格连接。
    assert_eq!(computed_style_property(html, "#cr", "counter-reset"), "sec 0");
    assert_eq!(computed_style_property(html, "#crm", "counter-reset"), "a 5 b 3");
    // 默认空→none。
    assert_eq!(computed_style_property(html, "#def", "counter-increment"), "none");
    assert_eq!(computed_style_property(html, "#def", "counter-reset"), "none");
}

#[test]
fn test_get_computed_style_transition_animation() {
    // R2743：getComputedStyle transition/animation 簇（10 属性，timing-function defer 到后续轮）。
    let html = "<html><body>\
        <div id=\"tp\" style=\"transition-property: margin, padding;\"></div>\
        <div id=\"tps\" style=\"transition-property: opacity;\"></div>\
        <div id=\"td\" style=\"transition-duration: 0.3s, 0.5s;\"></div>\
        <div id=\"tde\" style=\"transition-delay: 0.1s;\"></div>\
        <div id=\"an\" style=\"animation-name: fade, slide;\"></div>\
        <div id=\"ad\" style=\"animation-duration: 2s;\"></div>\
        <div id=\"adel\" style=\"animation-delay: 1s;\"></div>\
        <div id=\"aic\" style=\"animation-iteration-count: infinite;\"></div>\
        <div id=\"aicn\" style=\"animation-iteration-count: 2.5;\"></div>\
        <div id=\"adi\" style=\"animation-direction: alternate;\"></div>\
        <div id=\"afm\" style=\"animation-fill-mode: forwards;\"></div>\
        <div id=\"aps\" style=\"animation-play-state: paused;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // transition-property：逗号分隔；单值；默认 all。
    assert_eq!(
        computed_style_property(html, "#tp", "transition-property"),
        "margin, padding"
    );
    assert_eq!(computed_style_property(html, "#tps", "transition-property"), "opacity");
    assert_eq!(computed_style_property(html, "#def", "transition-property"), "all");
    // transition-duration/delay：Ns；默认 0s。
    assert_eq!(
        computed_style_property(html, "#td", "transition-duration"),
        "0.3s, 0.5s"
    );
    assert_eq!(computed_style_property(html, "#tde", "transition-delay"), "0.1s");
    assert_eq!(computed_style_property(html, "#def", "transition-duration"), "0s");
    // animation-name：逗号分隔；默认 none。
    assert_eq!(computed_style_property(html, "#an", "animation-name"), "fade, slide");
    assert_eq!(computed_style_property(html, "#def", "animation-name"), "none");
    // animation-duration/delay：Ns；默认 0s。
    assert_eq!(computed_style_property(html, "#ad", "animation-duration"), "2s");
    assert_eq!(computed_style_property(html, "#adel", "animation-delay"), "1s");
    // animation-iteration-count：infinite / 数值；默认 1。
    assert_eq!(
        computed_style_property(html, "#aic", "animation-iteration-count"),
        "infinite"
    );
    assert_eq!(
        computed_style_property(html, "#aicn", "animation-iteration-count"),
        "2.5"
    );
    assert_eq!(computed_style_property(html, "#def", "animation-iteration-count"), "1");
    // animation-direction/fill-mode/play-state：关键字；默认 normal/none/running。
    assert_eq!(
        computed_style_property(html, "#adi", "animation-direction"),
        "alternate"
    );
    assert_eq!(computed_style_property(html, "#afm", "animation-fill-mode"), "forwards");
    assert_eq!(computed_style_property(html, "#aps", "animation-play-state"), "paused");
    assert_eq!(computed_style_property(html, "#def", "animation-direction"), "normal");
    assert_eq!(computed_style_property(html, "#def", "animation-fill-mode"), "none");
    assert_eq!(computed_style_property(html, "#def", "animation-play-state"), "running");
}

#[test]
fn test_get_computed_style_timing_function() {
    // R2744：getComputedStyle transition/animation-timing-function。
    // 关键字保 keyword 不展开；cubic-bezier 4 数；steps(n) 默认 End 省略（spec-aligned，待 Chromium A/B 核实）。
    let html = "<html><body>\
        <div id=\"ease\" style=\"transition-timing-function: ease;\"></div>\
        <div id=\"lin\" style=\"transition-timing-function: linear;\"></div>\
        <div id=\"eio\" style=\"transition-timing-function: ease-in-out;\"></div>\
        <div id=\"cb\" style=\"transition-timing-function: cubic-bezier(0.25, 0.1, 0.25, 1);\"></div>\
        <div id=\"ss\" style=\"transition-timing-function: step-start;\"></div>\
        <div id=\"st\" style=\"transition-timing-function: steps(4);\"></div>\
        <div id=\"sts\" style=\"transition-timing-function: steps(4, start);\"></div>\
        <div id=\"multi\" style=\"transition-timing-function: ease-in, ease-out;\"></div>\
        <div id=\"atf\" style=\"animation-timing-function: linear;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // 关键字（保原样）。
    assert_eq!(
        computed_style_property(html, "#ease", "transition-timing-function"),
        "ease"
    );
    assert_eq!(
        computed_style_property(html, "#lin", "transition-timing-function"),
        "linear"
    );
    assert_eq!(
        computed_style_property(html, "#eio", "transition-timing-function"),
        "ease-in-out"
    );
    // cubic-bezier：4 数逗号分隔（整数 1 无小数点）。
    assert_eq!(
        computed_style_property(html, "#cb", "transition-timing-function"),
        "cubic-bezier(0.25, 0.1, 0.25, 1)"
    );
    // step-start；steps(4) 默认 End 省略；steps(4, start) 含位置。
    assert_eq!(
        computed_style_property(html, "#ss", "transition-timing-function"),
        "step-start"
    );
    assert_eq!(
        computed_style_property(html, "#st", "transition-timing-function"),
        "steps(4)"
    );
    assert_eq!(
        computed_style_property(html, "#sts", "transition-timing-function"),
        "steps(4, start)"
    );
    // 多值逗号分隔。
    assert_eq!(
        computed_style_property(html, "#multi", "transition-timing-function"),
        "ease-in, ease-out"
    );
    // animation-timing-function 同结构。
    assert_eq!(
        computed_style_property(html, "#atf", "animation-timing-function"),
        "linear"
    );
    // 默认空→ease。
    assert_eq!(
        computed_style_property(html, "#def", "transition-timing-function"),
        "ease"
    );
    assert_eq!(
        computed_style_property(html, "#def", "animation-timing-function"),
        "ease"
    );
}

#[test]
fn test_get_computed_style_overflow_shorthand() {
    // R2745：getComputedStyle overflow 简写（overflow-x/y longhand 早覆）。
    let html = "<html><body>\
        <div id=\"eq\" style=\"overflow: hidden;\"></div>\
        <div id=\"ne\" style=\"overflow: hidden scroll;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // x==y→单值；x!=y→"x y"（CSS Overflow 3）；默认 visible。
    assert_eq!(computed_style_property(html, "#eq", "overflow"), "hidden");
    assert_eq!(computed_style_property(html, "#ne", "overflow"), "hidden scroll");
    assert_eq!(computed_style_property(html, "#def", "overflow"), "visible");
}

#[test]
fn test_get_computed_style_scroll_mask() {
    // R2746：getComputedStyle scroll-margin-*/scroll-padding-*（Scroll Snap 边距）+ mask-mode。
    let html = "<html><body>\
        <div id=\"sm\" style=\"scroll-margin-top: 10px; scroll-margin-right: 20px; scroll-margin-bottom: 30px; scroll-margin-left: 40px;\"></div>\
        <div id=\"sp\" style=\"scroll-padding-top: 5px; scroll-padding-left: 35px;\"></div>\
        <div id=\"mm\" style=\"mask-mode: alpha;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // scroll-margin：longhand 各 f32→px（CSS Scroll Snap 2，scroll-margin 简写未实现故逐 longhand 测）；默认 0px。
    assert_eq!(computed_style_property(html, "#sm", "scroll-margin-top"), "10px");
    assert_eq!(computed_style_property(html, "#sm", "scroll-margin-right"), "20px");
    assert_eq!(computed_style_property(html, "#sm", "scroll-margin-bottom"), "30px");
    assert_eq!(computed_style_property(html, "#sm", "scroll-margin-left"), "40px");
    assert_eq!(computed_style_property(html, "#def", "scroll-margin-top"), "0px");
    // scroll-padding：ScrollPadding Auto/Length；默认 auto。
    assert_eq!(computed_style_property(html, "#sp", "scroll-padding-top"), "5px");
    assert_eq!(computed_style_property(html, "#sp", "scroll-padding-left"), "35px");
    assert_eq!(computed_style_property(html, "#def", "scroll-padding-top"), "auto");
    // mask-mode：alpha/luminance/match-source（初值 match-source）。
    assert_eq!(computed_style_property(html, "#mm", "mask-mode"), "alpha");
    assert_eq!(computed_style_property(html, "#def", "mask-mode"), "match-source");
}

#[test]
fn test_get_computed_style_background_mask_image() {
    // R2747：getComputedStyle background-image + mask-image（None/Url 逐层；gradient defer→''）。
    let html = "<html><body>\
        <div id=\"url\" style=\"background-image: url(bg.png);\"></div>\
        <div id=\"none\" style=\"background-image: none;\"></div>\
        <div id=\"multi\" style=\"background-image: url(a.png), url(b.png);\"></div>\
        <div id=\"grad\" style=\"background-image: radial-gradient(circle, red, blue);\"></div>\
        <div id=\"mask\" style=\"mask-image: url(m.png);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // url → url("u")（同 list-style-image）；none；多层逗号分隔。
    assert_eq!(
        computed_style_property(html, "#url", "background-image"),
        "url(\"bg.png\")"
    );
    assert_eq!(computed_style_property(html, "#none", "background-image"), "none");
    assert_eq!(
        computed_style_property(html, "#multi", "background-image"),
        "url(\"a.png\"), url(\"b.png\")"
    );
    // radial-gradient(circle, ...) 层 → 序列化（R2750 radial 已实现；见 test_get_computed_style_radial_conic_gradient 全覆盖）。
    assert_eq!(
        computed_style_property(html, "#grad", "background-image"),
        "radial-gradient(circle, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // mask-image 同结构。
    assert_eq!(computed_style_property(html, "#mask", "mask-image"), "url(\"m.png\")");
    // 默认 → none。
    assert_eq!(computed_style_property(html, "#def", "background-image"), "none");
    assert_eq!(computed_style_property(html, "#def", "mask-image"), "none");
}

#[test]
fn test_get_computed_style_margin_padding_shorthand() {
    // R2748：getComputedStyle margin + padding 简写（CSSOM 4 值最小化，复用 box_4_to_css）。
    let html = "<html><body>\
        <div id=\"m1\" style=\"margin: 5px;\"></div>\
        <div id=\"m2\" style=\"margin: 5px 10px;\"></div>\
        <div id=\"m3\" style=\"margin: 5px 10px 15px;\"></div>\
        <div id=\"m4\" style=\"margin: 5px 10px 15px 20px;\"></div>\
        <div id=\"ma\" style=\"margin: auto;\"></div>\
        <div id=\"p1\" style=\"padding: 8px;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // margin：全等→1 值；top==bottom&&right==left→2 值；right==left→3 值；否则 4 值。
    assert_eq!(computed_style_property(html, "#m1", "margin"), "5px");
    assert_eq!(computed_style_property(html, "#m2", "margin"), "5px 10px");
    assert_eq!(computed_style_property(html, "#m3", "margin"), "5px 10px 15px");
    assert_eq!(computed_style_property(html, "#m4", "margin"), "5px 10px 15px 20px");
    // margin: auto → auto（LengthValue::Auto 经 length_to_css）。
    assert_eq!(computed_style_property(html, "#ma", "margin"), "auto");
    // padding 同结构；默认 margin/padding 均 0px → "0px"。
    assert_eq!(computed_style_property(html, "#p1", "padding"), "8px");
    assert_eq!(computed_style_property(html, "#def", "margin"), "0px");
    assert_eq!(computed_style_property(html, "#def", "padding"), "0px");
}

#[test]
fn test_get_computed_style_linear_gradient() {
    // R2749：getComputedStyle background-image linear-gradient 层序列化（radial/conic 仍 defer）。
    let html = "<html><body>\
        <div id=\"d\" style=\"background-image: linear-gradient(to right, red, blue);\"></div>\
        <div id=\"defdir\" style=\"background-image: linear-gradient(red, blue);\"></div>\
        <div id=\"ang\" style=\"background-image: linear-gradient(45deg, red, blue);\"></div>\
        <div id=\"pos\" style=\"background-image: linear-gradient(to right, red 0%, blue 100%);\"></div>\
        <div id=\"rep\" style=\"background-image: repeating-linear-gradient(red, blue);\"></div>\
        </body></html>";
    // to right + 色标解析为 rgb。
    assert_eq!(
        computed_style_property(html, "#d", "background-image"),
        "linear-gradient(to right, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // 默认方向 to bottom 省略。
    assert_eq!(
        computed_style_property(html, "#defdir", "background-image"),
        "linear-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // 角度 → Xdeg。
    assert_eq!(
        computed_style_property(html, "#ang", "background-image"),
        "linear-gradient(45deg, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // 色标位置。
    assert_eq!(
        computed_style_property(html, "#pos", "background-image"),
        "linear-gradient(to right, rgb(255, 0, 0) 0%, rgb(0, 0, 255) 100%)"
    );
    // repeating- 前缀。
    assert_eq!(
        computed_style_property(html, "#rep", "background-image"),
        "repeating-linear-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
}

#[test]
fn test_get_computed_style_radial_conic_gradient() {
    // R2750：getComputedStyle radial-gradient（WPT oracle 锚定）+ conic-gradient（spec-aligned）。
    let html = "<html><body>\
        <div id=\"def\" style=\"background-image: radial-gradient(red, blue);\"></div>\
        <div id=\"ctr\" style=\"background-image: radial-gradient(at center, red, blue);\"></div>\
        <div id=\"pos\" style=\"background-image: radial-gradient(at 10px 10px, red, blue);\"></div>\
        <div id=\"cir\" style=\"background-image: radial-gradient(circle, red, blue);\"></div>\
        <div id=\"fs\" style=\"background-image: radial-gradient(farthest-side, red, blue);\"></div>\
        <div id=\"cp\" style=\"background-image: radial-gradient(circle at 25% 40%, red, blue);\"></div>\
        <div id=\"cl\" style=\"background-image: radial-gradient(circle 50px, red, blue);\"></div>\
        <div id=\"cdef\" style=\"background-image: conic-gradient(red, blue);\"></div>\
        <div id=\"cfrom\" style=\"background-image: conic-gradient(from 90deg, red, blue);\"></div>\
        <div id=\"cf0\" style=\"background-image: conic-gradient(from 0deg, red, blue);\"></div>\
        <div id=\"cat\" style=\"background-image: conic-gradient(at 25% 75%, red, blue);\"></div>\
        </body></html>";
    // 默认 ellipse farthest-corner at center 全省略。
    assert_eq!(
        computed_style_property(html, "#def", "background-image"),
        "radial-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // position-首位 `at center`（默认 position）→ 省略（R2751 parser fix 支持 position-首位 config）。
    assert_eq!(
        computed_style_property(html, "#ctr", "background-image"),
        "radial-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // position-首位 非默认 → at X Y。
    assert_eq!(
        computed_style_property(html, "#pos", "background-image"),
        "radial-gradient(at 10px 10px, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // circle（默认 size）保留。
    assert_eq!(
        computed_style_property(html, "#cir", "background-image"),
        "radial-gradient(circle, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // 非默认 size 关键字 farthest-side 保留（ellipse 默认形状省略）。
    assert_eq!(
        computed_style_property(html, "#fs", "background-image"),
        "radial-gradient(farthest-side, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // circle + 非默认 position。
    assert_eq!(
        computed_style_property(html, "#cp", "background-image"),
        "radial-gradient(circle at 25% 40%, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // circle + 显式半径 → 半径（circle 省略）。
    assert_eq!(
        computed_style_property(html, "#cl", "background-image"),
        "radial-gradient(50px, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // conic 默认 from 0deg at center 全省略。
    assert_eq!(
        computed_style_property(html, "#cdef", "background-image"),
        "conic-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // conic from <angle>。
    assert_eq!(
        computed_style_property(html, "#cfrom", "background-image"),
        "conic-gradient(from 90deg, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // conic from 0deg（默认）→ 省略（WPT oracle 锚定）。
    assert_eq!(
        computed_style_property(html, "#cf0", "background-image"),
        "conic-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // conic 非默认 position → at X Y（WPT oracle 锚定）。
    assert_eq!(
        computed_style_property(html, "#cat", "background-image"),
        "conic-gradient(at 25% 75%, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
}

#[test]
fn test_get_computed_style_border_image_source_gradient() {
    // R2753：border-image-source gradient 支持（旧仅 None/Url，gradient→none divergence；oracle 锚定）。
    // currentcolor 经元素 color 解析（#g 设 color:blue → rgb(0,0,255)，匹配 oracle）。
    let html = "<html><body>\
        <div id=\"g\" style=\"color: blue; border-image-source: linear-gradient(-45deg, red, currentcolor);\"></div>\
        <div id=\"r\" style=\"color: blue; border-image-source: radial-gradient(10px at 20px 30px, currentcolor, lime);\"></div>\
        <div id=\"u\" style=\"border-image-source: url(b.png);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // linear-gradient（含 -45deg 方向 + currentcolor→元素 color）。
    assert_eq!(
        computed_style_property(html, "#g", "border-image-source"),
        "linear-gradient(-45deg, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // radial-gradient（10px 半径 + at 20px 30px + currentcolor/lime）。
    assert_eq!(
        computed_style_property(html, "#r", "border-image-source"),
        "radial-gradient(10px at 20px 30px, rgb(0, 0, 255), rgb(0, 255, 0))"
    );
    // url 仍正常；默认 none。
    assert_eq!(
        computed_style_property(html, "#u", "border-image-source"),
        "url(\"b.png\")"
    );
    assert_eq!(computed_style_property(html, "#def", "border-image-source"), "none");
}

#[test]
fn test_raf_frame_driven_on_path() {
    // R2713a：帧驱动 rAF（__ZW_RAF_FRAME_DRIVEN=true）。requestAnimationFrame 注册回调延后到
    // host render 后的 __zw_raf_tick；tick 前不 fire，tick 后按注册序 fire 并传 ts、清空队列。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // host 在 execute 前注入 env flag（模拟 worker init 读 ZW_RAF_FRAME_DRIVEN=1）。
    sandbox.execute("globalThis.__ZW_RAF_FRAME_DRIVEN = true;").unwrap();
    sandbox
        .execute(
            "globalThis.__count = 0; globalThis.__ts = 'none';\
         requestAnimationFrame(function(t){ globalThis.__count++; globalThis.__ts = String(t); });\
         requestAnimationFrame(function(){ globalThis.__count++; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__count)").unwrap().value,
        "0",
        "帧驱动：tick 前回调不应 fire"
    );
    // host render 后调 __zw_raf_tick(16.7) → 按注册序 fire 两个、传 ts、清空队列。
    sandbox.execute("globalThis.__zw_raf_tick(16.7);").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__count)").unwrap().value,
        "2",
        "tick 后按注册序 fire 两个回调"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ts").unwrap().value,
        "16.7",
        "回调收到 ts 参数"
    );
}

#[test]
fn test_raf_frame_driven_cancel() {
    // R2713a：cancelAnimationFrame（ON 路径）移除待 fire 回调；tick 后不 fire。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute("globalThis.__ZW_RAF_FRAME_DRIVEN = true;").unwrap();
    sandbox
        .execute(
            "globalThis.__fired = 'no';\
         var id = requestAnimationFrame(function(){ globalThis.__fired = 'yes'; });\
         cancelAnimationFrame(id);",
        )
        .unwrap();
    sandbox.execute("globalThis.__zw_raf_tick(0);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fired").unwrap().value,
        "no",
        "cancelAnimationFrame 后回调不 fire"
    );
}

#[test]
fn test_raf_sync_stub_off_path() {
    // R2713a：OFF 路径（env unset = 默认）保留同步 stub——rAF 立即同步 fire（reftest 兼容），
    // __zw_raf_tick 为 no-op。零默认行为变更的回归守护。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // 不设 __ZW_RAF_FRAME_DRIVEN（默认 false）。
    sandbox
        .execute(
            "globalThis.__fired = 'no';\
         requestAnimationFrame(function(){ globalThis.__fired = 'yes'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fired").unwrap().value,
        "yes",
        "OFF 路径：rAF 立即同步 fire（reftest 兼容，零默认行为变更）"
    );
    // __zw_raf_tick OFF 路径 no-op（不应抛错、不重复 fire）。
    sandbox.execute("globalThis.__zw_raf_tick(0);").unwrap();
    assert_eq!(sandbox.execute("globalThis.__fired").unwrap().value, "yes");
}

#[test]
fn test_element_attributes_nodelist() {
    // R2699：el.attributes（NamedNodeMap 只读快照）——length/item/getNamedItem/数值索引/迭代。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\" class=\"c\" title=\"t\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // length + 数值索引 + item。
    sandbox
        .execute(
            "globalThis.__len = document.querySelector('#d').attributes.length;\n\
             globalThis.__i0 = document.querySelector('#d').attributes[0].name;\n\
             globalThis.__item1 = document.querySelector('#d').attributes.item(1).name;\n\
             globalThis.__item_oob = document.querySelector('#d').attributes.item(9);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__len").unwrap().value,
        "3",
        "attributes.length"
    );
    assert_eq!(
        sandbox.execute("globalThis.__i0").unwrap().value,
        "id",
        "attributes[0].name"
    );
    assert_eq!(
        sandbox.execute("globalThis.__item1").unwrap().value,
        "class",
        "attributes.item(1).name"
    );
    assert_eq!(
        sandbox.execute("globalThis.__item_oob === null").unwrap().value,
        "true",
        "out-of-range item → null"
    );

    // getNamedItem（命中 + value + 未命中 null）。
    sandbox
        .execute(
            "globalThis.__gn = document.querySelector('#d').attributes.getNamedItem('title').value;\n\
             globalThis.__gnn = document.querySelector('#d').attributes.getNamedItem('nope');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__gn").unwrap().value,
        "t",
        "getNamedItem('title').value"
    );
    assert_eq!(
        sandbox.execute("globalThis.__gnn === null").unwrap().value,
        "true",
        "getNamedItem 未命中 → null"
    );

    // 迭代（Symbol.iterator）→ 属性名顺序。
    sandbox
        .execute(
            "globalThis.__iter = Array.prototype.map.call(document.querySelector('#d').attributes, function(a){ return a.name; }).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__iter").unwrap().value,
        "id,class,title",
        "attributes 迭代顺序"
    );
}

#[test]
fn test_set_remove_attr_syncs_cache() {
    // R2700：setAttribute/removeAttribute 须同步 class/value 客户端缓存，否则后续 classList/.value
    // 读 stale 缓存丢值（setAttribute('class','a');classList.add('b') 旧丢 'a'）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mk = |html: &str| -> (V8Sandbox, Arc<Mutex<Vec<DomMutation>>>, Arc<Mutex<String>>) {
        let mut sb = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sb.execute(generate_js_dom_shim()).unwrap();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
        let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
        let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
        register_dom_callbacks(&mut sb, &mutations, &dom_html, &page_url);
        (sb, mutations, dom_html)
    };

    // ① setAttribute('class','a') + classList.add('b') → 'a b'（旧 'base b' 丢 a）。
    let (mut sandbox, mutations, dom_html) = mk("<html><body><div id=\"d\" class=\"base\"></div></body></html>");
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.setAttribute('class', 'a');\n\
             d.classList.add('b');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        out.contains("class=\"a b\""),
        "setAttribute+classList 协作 → 'a b'\n{out}"
    );

    // ② setAttribute('value','x') + .value 读 → 'x'（旧 stale 读 'old'）。
    let (mut sandbox, _mutations, _dom_html) = mk("<html><body><input id=\"i\" value=\"old\"></body></html>");
    sandbox
        .execute(
            "document.querySelector('#i').setAttribute('value', 'x');\n\
             globalThis.__v = document.querySelector('#i').value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__v").unwrap().value,
        "x",
        "setAttribute('value','x') 后 .value 读见 'x'"
    );

    // ③ classList.add('a'); removeAttribute('class'); classList.add('b') → 'b'
    //    （removeAttribute 清缓存，否则 add('b') 读 stale 'base a' → 'base a b'）。
    let (mut sandbox, mutations, dom_html) = mk("<html><body><div id=\"d\" class=\"base\"></div></body></html>");
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.classList.add('a');\n\
             d.removeAttribute('class');\n\
             d.classList.add('b');",
        )
        .unwrap();
    let ms3 = mutations.lock().unwrap().clone();
    let out3 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms3).unwrap();
    assert!(
        out3.contains("class=\"b\""),
        "removeAttribute('class') 清缓存后 add('b') → 'b'\n{out3}"
    );
}

#[test]
fn test_get_computed_style_gap_shorthand() {
    // R2754：gap 简写 = row-gap/column-gap 双轴（CSS Box Alignment 3）。旧实现仅读 legacy
    // gap 字段（= row-gap），致 `gap: 5px 10px` 丢 column-gap 返 "5px"。改用 longhand 字段
    // 做 2 值最小化（row==col→单值，否则 "row col"）。Chromium oracle：单值→"5px"，双值→"5px 10px"。
    let html = "<html><body>\
        <div id=\"g1\" style=\"gap: 5px;\"></div>\
        <div id=\"g2\" style=\"gap: 5px 10px;\"></div>\
        </body></html>";
    assert_eq!(computed_style_property(html, "#g1", "gap"), "5px");
    assert_eq!(computed_style_property(html, "#g1", "row-gap"), "5px");
    assert_eq!(computed_style_property(html, "#g1", "column-gap"), "5px");
    assert_eq!(computed_style_property(html, "#g2", "gap"), "5px 10px");
    assert_eq!(computed_style_property(html, "#g2", "row-gap"), "5px");
    assert_eq!(computed_style_property(html, "#g2", "column-gap"), "10px");
}

#[test]
fn test_get_computed_style_text_decoration_longhands() {
    // R2754：text-decoration 4 longhand 早有 storage，补 getComputedStyle 序列化。
    // line（多 flag 规范序 underline overline line-through，空→none）/ style / color（currentcolor
    // 解析）/ thickness（auto/from-font/length）。Chromium oracle 锚定。
    let html = "<html><body>\
        <div id=\"td\" style=\"text-decoration: underline dotted red 2px;\"></div>\
        <div id=\"td2\" style=\"text-decoration: line-through overline;\"></div>\
        <div id=\"plain\"></div>\
        </body></html>";
    assert_eq!(
        computed_style_property(html, "#td", "text-decoration-line"),
        "underline"
    );
    assert_eq!(computed_style_property(html, "#td", "text-decoration-style"), "dotted");
    assert_eq!(
        computed_style_property(html, "#td", "text-decoration-color"),
        "rgb(255, 0, 0)"
    );
    assert_eq!(computed_style_property(html, "#td", "text-decoration-thickness"), "2px");
    // 多值组合按规范序重组（输入 line-through overline → overline line-through）。
    assert_eq!(
        computed_style_property(html, "#td2", "text-decoration-line"),
        "overline line-through"
    );
    // 默认值：line=none / style=solid / thickness=auto。
    assert_eq!(computed_style_property(html, "#plain", "text-decoration-line"), "none");
    assert_eq!(
        computed_style_property(html, "#plain", "text-decoration-style"),
        "solid"
    );
    assert_eq!(
        computed_style_property(html, "#plain", "text-decoration-thickness"),
        "auto"
    );
}

#[test]
fn test_get_computed_style_flex_shorthand() {
    // R2754：flex 简写 = "<grow> <shrink> <basis>"（恒 3 段）。关键：spec §7.1.1 省略 basis 时
    // flex-basis=0%（百分比），故 `flex: 1`→"1 1 0%"（Chromium oracle；旧 ZW basis="0"→"0px" diverge）。
    // none→"0 0 auto" / auto→"1 1 auto" / 显式 basis 原样。
    let html = "<html><body>\
        <div id=\"fl\" style=\"flex: 2 1 50px;\"></div>\
        <div id=\"flone\" style=\"flex: 1;\"></div>\
        <div id=\"fln\" style=\"flex: none;\"></div>\
        <div id=\"fla\" style=\"flex: auto;\"></div>\
        <div id=\"plain\"></div>\
        </body></html>";
    assert_eq!(computed_style_property(html, "#fl", "flex"), "2 1 50px");
    assert_eq!(computed_style_property(html, "#flone", "flex"), "1 1 0%");
    assert_eq!(computed_style_property(html, "#flone", "flex-basis"), "0%");
    assert_eq!(computed_style_property(html, "#fln", "flex"), "0 0 auto");
    assert_eq!(computed_style_property(html, "#fla", "flex"), "1 1 auto");
    assert_eq!(computed_style_property(html, "#plain", "flex"), "0 1 auto");
}

#[test]
fn test_get_computed_style_flex_flow_shorthand() {
    // R2754：flex-flow = "<direction> <wrap>"（恒 2 段）。Chromium oracle：column wrap→"column wrap"，
    // 单值 wrap→"row wrap"（direction 缺省 row），default→"row nowrap"。
    let html = "<html><body>\
        <div id=\"ff\" style=\"flex-flow: column wrap;\"></div>\
        <div id=\"ffw\" style=\"flex-flow: wrap;\"></div>\
        <div id=\"plain\"></div>\
        </body></html>";
    assert_eq!(computed_style_property(html, "#ff", "flex-flow"), "column wrap");
    assert_eq!(computed_style_property(html, "#ffw", "flex-flow"), "row wrap");
    assert_eq!(computed_style_property(html, "#plain", "flex-flow"), "row nowrap");
}

#[test]
fn test_get_computed_style_outline_and_border_shorthands() {
    // R2754：outline = "<color> <style> <width>"（注意与 border 的 width-style-color 顺序相反！），
    // 恒 3 段含 none。border/per-side = "<width> <style> <color>"，全边 border 仅 4 边全等时返单边值
    // 否则 ''。outline-width 不套 border 的 none→0 规则（保留 computed medium→3px）。
    // Chromium oracle 锚定全部断言。
    let html = "<html><body>\
        <div id=\"o\" style=\"outline: 2px solid red;\"></div>\
        <div id=\"olt\" style=\"outline: thick solid #0f0;\"></div>\
        <div id=\"b\" style=\"border: 3px dashed blue;\"></div>\
        <div id=\"bt\" style=\"border-top: 3px dashed blue;\"></div>\
        <div id=\"bdiff\" style=\"border-top: 1px solid; border-bottom: 2px solid;\"></div>\
        <div id=\"plain\"></div>\
        </body></html>";
    // outline 简写（color style width 顺序）。
    assert_eq!(
        computed_style_property(html, "#o", "outline"),
        "rgb(255, 0, 0) solid 2px"
    );
    assert_eq!(
        computed_style_property(html, "#olt", "outline"),
        "rgb(0, 255, 0) solid 5px"
    );
    // outline 默认：style=none 仍保留 width medium→3px（与 border-width 不同）。
    assert_eq!(
        computed_style_property(html, "#plain", "outline"),
        "rgb(0, 0, 0) none 3px"
    );
    assert_eq!(computed_style_property(html, "#plain", "outline-width"), "3px");
    // border 简写：4 边全等 → "width style color"；不一致 → ''。
    assert_eq!(
        computed_style_property(html, "#b", "border"),
        "3px dashed rgb(0, 0, 255)"
    );
    assert_eq!(
        computed_style_property(html, "#b", "border-top"),
        "3px dashed rgb(0, 0, 255)"
    );
    assert_eq!(
        computed_style_property(html, "#bt", "border-top"),
        "3px dashed rgb(0, 0, 255)"
    );
    assert_eq!(
        computed_style_property(html, "#bdiff", "border-top"),
        "1px solid rgb(0, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#bdiff", "border-bottom"),
        "2px solid rgb(0, 0, 0)"
    );
    assert_eq!(computed_style_property(html, "#bdiff", "border"), "");
    assert_eq!(
        computed_style_property(html, "#plain", "border"),
        "0px none rgb(0, 0, 0)"
    );
}
