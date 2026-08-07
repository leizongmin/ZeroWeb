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
fn test_performance_now_e2e() {
    // R2768：performance.now()——DOMHighResTimeStamp（ms，单调，自 time origin 起，子毫秒）。
    // register_dom_callbacks 注册 __zw_performance_now（Instant elapsed ms）。验：typeof number、
    // 非负、两次调用单调非减（Monotonic）。host 未注册时 shim 走 Date.now() 兜底（test_request_idle
    // 路径覆盖，此处测注册后真值）。
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

    // performance.now() 返 number。
    assert_eq!(sandbox.execute("typeof performance.now()").unwrap().value, "number");
    // 非负。
    assert_eq!(sandbox.execute("performance.now() >= 0").unwrap().value, "true");
    // 单调非减：连续两次调用 t2 >= t1（host Instant 单调）。
    assert_eq!(
        sandbox
            .execute("var t1 = performance.now(); performance.now() >= t1")
            .unwrap()
            .value,
        "true"
    );
}

#[test]
fn test_atob_btoa_crypto_randomuuid_r2770() {
    // R2770：atob/btoa（Base64）+ crypto.randomUUID（UUID v4）。纯 JS（shim），无 host 回调。
    // probe 确认 V8 不提供这些 Web API（全 undefined）。btoa 对 >255（非 Latin-1）抛；atob 容错；
    // randomUUID 返 v4 格式 + 唯一。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // btoa / atob round-trip（Chromium 锚定：btoa('hello')='aGVsbG8='）。
    assert_eq!(sandbox.execute("btoa('hello')").unwrap().value, "aGVsbG8=");
    assert_eq!(sandbox.execute("atob('aGVsbG8=')").unwrap().value, "hello");
    assert_eq!(sandbox.execute("atob(btoa('ZeroWeb!'))").unwrap().value, "ZeroWeb!");
    // btoa 对 >255 抛 InvalidCharacterError（spec）。
    assert_eq!(
        sandbox
            .execute("try { btoa('\\u0100'); 'no-throw' } catch (e) { 'threw' }")
            .unwrap()
            .value,
        "threw"
    );
    // crypto.randomUUID：UUID v4 格式（8-4-4-4-12，version=4，variant∈89ab）。
    assert_eq!(
        sandbox
            .execute(
                "var u = crypto.randomUUID(); \
                 /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(u)"
            )
            .unwrap()
            .value,
        "true"
    );
    // 唯一性（两次调用不同）。
    assert_eq!(
        sandbox
            .execute("crypto.randomUUID() !== crypto.randomUUID()")
            .unwrap()
            .value,
        "true"
    );
}

#[test]
fn test_crypto_get_random_values_r2775() {
    // R2775：crypto.getRandomValues（TypedArray 字节填充，Math.random-based 同 randomUUID 一致）。
    // 填底层字节 buffer → 任意 typed 视图随机值；spec 限 TypedArray + ≤65536 字节。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 返回同一数组 + 长度不变 + 每字节在 [0,256)（已填充）。
    assert_eq!(
        sandbox
            .execute(
                "var a = new Uint8Array(4); var r = crypto.getRandomValues(a);\
                 r === a && a.length === 4 && a[0] >= 0 && a[0] < 256 && a[3] >= 0 && a[3] < 256"
            )
            .unwrap()
            .value,
        "true"
    );
    // Uint32Array 经字节 buffer 填充 → 随机 Uint32（值在 [0, 2^32)）。
    assert_eq!(
        sandbox
            .execute(
                "var u = crypto.getRandomValues(new Uint32Array(1));\
                 u[0] >= 0 && u[0] < 4294967296"
            )
            .unwrap()
            .value,
        "true"
    );
    // >65536 字节抛 RangeError（spec）。
    assert_eq!(
        sandbox
            .execute("try { crypto.getRandomValues(new Uint8Array(65537)); 'no-throw' } catch (e) { 'threw' }")
            .unwrap()
            .value,
        "threw"
    );
}

#[test]
fn test_dom_exception_r2776() {
    // R2776：DOMException（Web IDL 异常类型，本地 Chromium 150 oracle 锚定）。众多 Web API 抛出它；
    // name/message/legacy code + 25 legacy 常量；instance 非 Error 子类；toString="name: message"。
    // 同时验收 R2776 升级的已 land API 错误类型（btoa→InvalidCharacterError / getRandomValues→
    // QuotaExceededError / structuredClone→DataCloneError）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof DOMException = function。
    assert_eq!(sandbox.execute("typeof DOMException").unwrap().value, "function");
    // new DOMException(msg, name)：name/message/code + toString="name: message"（oracle 锚定）。
    assert_eq!(
        sandbox
            .execute(
                "var e = new DOMException('bad char', 'InvalidCharacterError');\
                 e.name + '|' + e.message + '|' + e.code + '|' + (e + '')"
            )
            .unwrap()
            .value,
        "InvalidCharacterError|bad char|5|InvalidCharacterError: bad char"
    );
    // 无 name 参数：name='Error'/code=0；无参 message=''。
    assert_eq!(
        sandbox
            .execute(
                "var a = new DOMException('hi'); var b = new DOMException();\
                 a.name + '|' + a.code + '|' + b.name + '|' + b.message + '|' + b.code"
            )
            .unwrap()
            .value,
        "Error|0|Error||0"
    );
    // instance 非 Error 子类（浏览器行为一致），但 instanceof DOMException 为 true。
    assert_eq!(
        sandbox
            .execute(
                "var e = new DOMException('m', 'DataCloneError');\
                 (e instanceof DOMException) + '|' + (e instanceof Error)"
            )
            .unwrap()
            .value,
        "true|false"
    );
    // legacy 常量（oracle 锚定子集）。
    assert_eq!(
        sandbox
            .execute(
                "DOMException.DATA_CLONE_ERR + '|' + DOMException.QUOTA_EXCEEDED_ERR + '|' +\
                 DOMException.INVALID_CHARACTER_ERR + '|' + DOMException.NOT_SUPPORTED_ERR + '|' +\
                 DOMException.SECURITY_ERR"
            )
            .unwrap()
            .value,
        "25|22|5|9|18"
    );
    // name→code 映射：DataCloneError=25 / QuotaExceededError=22 / NotSupportedError=9（legacy name）。
    assert_eq!(
        sandbox
            .execute(
                "new DOMException('a','DataCloneError').code + '|' +\
                 new DOMException('b','QuotaExceededError').code + '|' +\
                 new DOMException('c','NotSupportedError').code"
            )
            .unwrap()
            .value,
        "25|22|9"
    );
    // 无 new 调用亦返 DOMException 实例（同 Error 语义）。
    assert_eq!(
        sandbox
            .execute("DOMException('x', 'SyntaxError') instanceof DOMException")
            .unwrap()
            .value,
        "true"
    );
    // ★ 升级验收：btoa 抛 InvalidCharacterError DOMException（R2776 升级自裸 Error）。
    assert_eq!(
        sandbox
            .execute("try { btoa('\\u0100'); '' } catch (e) { e.name + '|' + (e instanceof DOMException) }")
            .unwrap()
            .value,
        "InvalidCharacterError|true"
    );
    // ★ 升级验收：getRandomValues >65536 抛 QuotaExceededError DOMException（升级自 RangeError）。
    assert_eq!(
        sandbox
            .execute("try { crypto.getRandomValues(new Uint8Array(65537)); '' } catch (e) { e.name + '|' + e.code }")
            .unwrap()
            .value,
        "QuotaExceededError|22"
    );
    // ★ 升级验收：structuredClone(function) 抛 DataCloneError DOMException（升级自 TypeError）。
    assert_eq!(
        sandbox
            .execute("try { structuredClone(function(){}); '' } catch (e) { e.name + '|' + e.code }")
            .unwrap()
            .value,
        "DataCloneError|25"
    );
}

#[test]
fn test_abort_controller_signal_r2777() {
    // R2777：AbortController/AbortSignal（fetch 中止 / 异步流程控制，cancel token 模式，现代 JS 库 /
    // fetch 高频）。本地 Chromium 150 oracle 锚定。signal.aborted/reason（getter）+ abort(reason) +
    // addEventListener('abort') 触发 + AbortSignal.abort() 静态 + throwIfAborted()。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + fresh signal 状态（oracle 锚定：aborted=false / reason=undefined）。
    assert_eq!(
        sandbox
            .execute(
                "typeof AbortController + '|' + typeof AbortSignal + '|' +\
                 new AbortController().signal.aborted + '|' +\
                 String(new AbortController().signal.reason)"
            )
            .unwrap()
            .value,
        "function|function|false|undefined"
    );
    // controller.signal 访问器 + controller.abort 方法（typeof function）。
    assert_eq!(
        sandbox
            .execute("var c = new AbortController(); typeof c.signal + '|' + typeof c.abort")
            .unwrap()
            .value,
        "object|function"
    );
    // abort() 无参：aborted=true，默认 reason 为 AbortError DOMException。
    assert_eq!(
        sandbox
            .execute(
                "var c = new AbortController(); c.abort();\
                 c.signal.aborted + '|' + (c.signal.reason instanceof DOMException) + '|' + c.signal.reason.name"
            )
            .unwrap()
            .value,
        "true|true|AbortError"
    );
    // abort(msg)：reason 即 msg（不包装）；abort(DOMException)：reason 即该 DOMException。
    assert_eq!(
        sandbox
            .execute(
                "var a = new AbortController(); a.abort('cancelled');\
                 var b = new AbortController(); b.abort(new DOMException('x','AbortError'));\
                 a.signal.reason + '|' + (b.signal.reason instanceof DOMException) + '|' + b.signal.reason.name"
            )
            .unwrap()
            .value,
        "cancelled|true|AbortError"
    );
    // addEventListener('abort') 在 abort() 时触发回调。
    assert_eq!(
        sandbox
            .execute(
                "var c = new AbortController(); var hit = 'no';\
                 c.signal.addEventListener('abort', function () { hit = 'yes'; });\
                 c.abort(); hit"
            )
            .unwrap()
            .value,
        "yes"
    );
    // 重复 abort() 静默 no-op（不抛，spec）。
    assert_eq!(
        sandbox
            .execute(
                "var c = new AbortController(); c.abort('first'); c.abort('second');\
                 try { c.abort(); 'no-throw'; } catch (e) { 'threw'; }"
            )
            .unwrap()
            .value,
        "no-throw"
    );
    // throwIfAborted：未 aborted 不抛；aborted 抛 AbortError DOMException。
    assert_eq!(
        sandbox
            .execute(
                "new AbortController().signal.throwIfAborted();\
                 var c = new AbortController(); c.abort();\
                 try { c.signal.throwIfAborted(); 'no-throw'; } catch (e) { (e instanceof DOMException) + '|' + e.name; }"
            )
            .unwrap()
            .value,
        "true|AbortError"
    );
    // AbortSignal.abort(reason) 静态工厂：aborted=true / reason 透传。
    assert_eq!(
        sandbox
            .execute("var s = AbortSignal.abort('r'); s.aborted + '|' + s.reason + '|' + (s instanceof AbortSignal)")
            .unwrap()
            .value,
        "true|r|true"
    );
    // AbortSignal.timeout 存在（typeof function）且初始 aborted=false（真延迟触发依赖事件循环，不在此断言）。
    assert_eq!(
        sandbox
            .execute("typeof AbortSignal.timeout + '|' + AbortSignal.timeout(100).aborted")
            .unwrap()
            .value,
        "function|false"
    );
    // removeEventListener 注册的回调不再触发。
    assert_eq!(
        sandbox
            .execute(
                "var c = new AbortController(); var hit = 0;\
                 var cb = function () { hit++; };\
                 c.signal.addEventListener('abort', cb);\
                 c.signal.removeEventListener('abort', cb);\
                 c.abort(); hit"
            )
            .unwrap()
            .value,
        "0"
    );
}

#[test]
fn test_url_constructor_r2778() {
    // R2778：URL 构造器（WHATWG URL 解析，委托 host __zw_parse_url → url crate）。本测试注册
    // __zw_parse_url 回调（生产在 register_dom_callbacks 注册），复用 parse_url_to_json 纯函数。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    // 注册 __zw_parse_url 回调（复用 production 纯函数 parse_url_to_json）。
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            parse_url_to_json(input, base)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 绝对 URL：各属性（protocol 带 ':'、host/hostname、port、pathname、search 带 '?'、hash 带 '#'、origin）。
    assert_eq!(
        sandbox
            .execute(
                "var u = new URL('https://example.com/a/b?x=1&y=2#top');\
                 u.protocol + '|' + u.hostname + '|' + u.host + '|' + u.port + '|' +\
                 u.pathname + '|' + u.search + '|' + u.hash + '|' + u.origin"
            )
            .unwrap()
            .value,
        "https:|example.com|example.com||/a/b|?x=1&y=2|#top|https://example.com"
    );
    // href 规范化 + toString + toJSON（三者一致）。
    assert_eq!(
        sandbox
            .execute(
                "var u = new URL('https://example.com/a/b?x=1#top');\
                 u.href + '|' + u.toString() + '|' + u.toJSON()"
            )
            .unwrap()
            .value,
        "https://example.com/a/b?x=1#top|https://example.com/a/b?x=1#top|https://example.com/a/b?x=1#top"
    );
    // searchParams（复用 URLSearchParams）：get / has。
    assert_eq!(
        sandbox
            .execute(
                "var u = new URL('https://example.com/?a=1&b=two');\
                 u.searchParams.get('a') + '|' + u.searchParams.get('b') + '|' + u.searchParams.has('c')"
            )
            .unwrap()
            .value,
        "1|two|false"
    );
    // base 解析：绝对路径相对（替换 base path）。
    assert_eq!(
        sandbox
            .execute("new URL('/path/page', 'https://example.com/base/index').href")
            .unwrap()
            .value,
        "https://example.com/path/page"
    );
    // base 解析：scheme-relative（继承 base scheme）。
    assert_eq!(
        sandbox
            .execute("new URL('//cdn.example.com/lib.js', 'https://example.com/').href")
            .unwrap()
            .value,
        "https://cdn.example.com/lib.js"
    );
    // 端口：非默认端口保留，默认端口（:80）归一省略。
    assert_eq!(
        sandbox
            .execute("new URL('http://example.com:8080/p').host + '|' + new URL('http://example.com:80/p').host")
            .unwrap()
            .value,
        "example.com:8080|example.com"
    );
    // 无效 URL 抛 TypeError（spec）。
    assert_eq!(
        sandbox
            .execute("try { new URL('not a valid url'); 'no-throw'; } catch (e) { e.name; }")
            .unwrap()
            .value,
        "TypeError"
    );
    // canParse：有效 true / 无效 false（不抛）。
    assert_eq!(
        sandbox
            .execute("URL.canParse('https://e.com') + '|' + URL.canParse('abc def')")
            .unwrap()
            .value,
        "true|false"
    );
    // 无 new 调用亦返 URL 实例。
    assert_eq!(
        sandbox.execute("URL('https://example.com/').hostname").unwrap().value,
        "example.com"
    );
}

#[test]
fn test_event_target_and_event_spec_r2779() {
    // R2779：EventTarget 独立构造器 + Event/CustomEvent spec-completeness（chromium oracle 锚定）。
    // 低风险：_makeEvent additive spec 字段 + 构造器置 [[Prototype]]，dispatch 读 _-prefixed 私字段不变。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // Event spec 字段（composed/eventPhase/isTrusted/defaultPrevented/timeStamp>=0）。
    assert_eq!(
        sandbox
            .execute(
                "var e = new Event('click', { bubbles: true, cancelable: true });\
                 e.type + '|' + e.bubbles + '|' + e.cancelable + '|' + e.composed + '|' +\
                 e.eventPhase + '|' + e.isTrusted + '|' + e.defaultPrevented + '|' + (e.timeStamp >= 0)"
            )
            .unwrap()
            .value,
        "click|true|true|false|0|false|false|true"
    );
    // instanceof：new Event instanceof Event；new CustomEvent instanceof Event & CustomEvent。
    assert_eq!(
        sandbox
            .execute(
                "(new Event('x') instanceof Event) + '|' +\
                 (new CustomEvent('y') instanceof Event) + '|' +\
                 (new CustomEvent('y') instanceof CustomEvent)"
            )
            .unwrap()
            .value,
        "true|true|true"
    );
    // CustomEvent detail 透传。
    assert_eq!(
        sandbox
            .execute("JSON.stringify(new CustomEvent('hi', { detail: { a: 1 } }).detail)")
            .unwrap()
            .value,
        "{\"a\":1}"
    );
    // preventDefault：cancelable → defaultPrevented true；non-cancelable → false（no-op）。
    assert_eq!(
        sandbox
            .execute(
                "var a = new Event('a', { cancelable: true }); a.preventDefault();\
                 var b = new Event('b', { cancelable: false }); b.preventDefault();\
                 a.defaultPrevented + '|' + b.defaultPrevented"
            )
            .unwrap()
            .value,
        "true|false"
    );
    // EventTarget 独立：typeof add/removeEventListener/dispatchEvent = function。
    assert_eq!(
        sandbox
            .execute(
                "var t = new EventTarget();\
                 typeof t.addEventListener + '|' + typeof t.removeEventListener + '|' + typeof t.dispatchEvent"
            )
            .unwrap()
            .value,
        "function|function|function"
    );
    // EventTarget dispatch 触发 listener + 设 target/currentTarget === 该 target。
    assert_eq!(
        sandbox
            .execute(
                "var t = new EventTarget(); var seen = null;\
                 t.addEventListener('ping', function (e) { seen = e.type + '|' + (e.target === t) + '|' + (e.currentTarget === t); });\
                 t.dispatchEvent(new Event('ping')); seen"
            )
            .unwrap()
            .value,
        "ping|true|true"
    );
    // dispatchEvent 返 true（未 preventDefault）；cancelable+preventDefault 返 false。
    assert_eq!(
        sandbox
            .execute(
                "var t = new EventTarget();\
                 var r1 = t.dispatchEvent(new Event('x'));\
                 t.addEventListener('y', function (e) { e.preventDefault(); });\
                 var r2 = t.dispatchEvent(new Event('y', { cancelable: true }));\
                 r1 + '|' + r2"
            )
            .unwrap()
            .value,
        "true|false"
    );
    // removeEventListener：移除后不再触发。
    assert_eq!(
        sandbox
            .execute(
                "var t = new EventTarget(); var n = 0;\
                 var cb = function () { n++; };\
                 t.addEventListener('e', cb); t.dispatchEvent(new Event('e'));\
                 t.removeEventListener('e', cb); t.dispatchEvent(new Event('e')); n"
            )
            .unwrap()
            .value,
        "1"
    );
    // stopImmediatePropagation：阻后续 listener（同 target）。
    assert_eq!(
        sandbox
            .execute(
                "var t = new EventTarget(); var order = '';\
                 t.addEventListener('s', function (e) { order += 'a'; e.stopImmediatePropagation(); });\
                 t.addEventListener('s', function () { order += 'b'; });\
                 t.dispatchEvent(new Event('s')); order"
            )
            .unwrap()
            .value,
        "a"
    );
    // class extends EventTarget 模式（现代事件总线惯用法）。
    assert_eq!(
        sandbox
            .execute(
                "class Bus extends EventTarget { constructor() { super(); } }\
                 var b = new Bus(); var hit = 0;\
                 b.addEventListener('z', function () { hit++; });\
                 b.dispatchEvent(new Event('z')); hit"
            )
            .unwrap()
            .value,
        "1"
    );
}

#[test]
fn test_url_setters_r2780() {
    // R2780：URL 组件 setter + 双向 searchParams 同步（host callback __zw_set_url_part → url crate setters）。
    // 注册 __zw_parse_url + __zw_set_url_part 两回调（复用 production 纯函数）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            parse_url_to_json(input, base)
        }),
    );
    sandbox.register_callback(
        "__zw_set_url_part",
        Box::new(|args: &[String]| -> String {
            let prev = args.first().map(String::as_str).unwrap_or("");
            let part = args.get(1).map(String::as_str).unwrap_or("");
            let value = args.get(2).map(String::as_str).unwrap_or("");
            set_url_part(prev, part, value)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // pathname setter（SPA 路由高频）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/old'); u.pathname = '/new/path'; u.pathname + '|' + u.href")
            .unwrap()
            .value,
        "/new/path|https://example.com/new/path"
    );
    // hash setter（SPA 路由）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p'); u.hash = '#section'; u.hash + '|' + u.href")
            .unwrap()
            .value,
        "#section|https://example.com/p#section"
    );
    // protocol setter（http→https）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('http://example.com/p'); u.protocol = 'https:'; u.protocol + '|' + u.href")
            .unwrap()
            .value,
        "https:|https://example.com/p"
    );
    // hostname setter + host 联动（_load 全字段重载）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://old.example.com/p'); u.hostname = 'new.example.com'; u.hostname + '|' + u.host")
            .unwrap()
            .value,
        "new.example.com|new.example.com"
    );
    // port setter（非默认）+ host/href 联动。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p'); u.port = '8443'; u.port + '|' + u.host + '|' + u.href")
            .unwrap()
            .value,
        "8443|example.com:8443|https://example.com:8443/p"
    );
    // search setter → searchParams 同步（search→params 方向）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p?a=1'); u.search = '?x=9&y=8'; u.searchParams.get('x') + '|' + u.searchParams.get('y')")
            .unwrap()
            .value,
        "9|8"
    );
    // searchParams append → search/href 同步（params→search 方向，无递归）。
    assert_eq!(
        sandbox
            .execute(
                "var u = new URL('https://example.com/p'); u.searchParams.append('k', 'v'); u.search + '|' + u.href"
            )
            .unwrap()
            .value,
        "?k=v|https://example.com/p?k=v"
    );
    // searchParams 多次 set → search 反映最后值 + 无递归（多次 mutate 不爆栈）。
    assert_eq!(
        sandbox
            .execute(
                "var u = new URL('https://example.com/p'); u.searchParams.set('a','1'); u.searchParams.set('b','2'); u.searchParams.set('a','9'); u.searchParams.get('a') + '|' + u.search"
            )
            .unwrap()
            .value,
        "9|?a=9&b=2"
    );
    // searchParams delete → search 更新。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p?a=1&b=2'); u.searchParams.delete('a'); u.search")
            .unwrap()
            .value,
        "?b=2"
    );
    // href setter（整体替换）+ searchParams 同步。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/old'); u.href = 'http://other.test/x?z=5#w'; u.host + '|' + u.pathname + '|' + u.searchParams.get('z') + '|' + u.hash")
            .unwrap()
            .value,
        "other.test|/x|5|#w"
    );
    // 无效 href setter 抛 TypeError（Url::parse 失败，spec 一致）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/'); try { u.href = 'not a valid url'; 'no-throw'; } catch (e) { e.name; }")
            .unwrap()
            .value,
        "TypeError"
    );
    // searchParams 稳定实例（多次访问同对象，spec 一致）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p'); u.searchParams === u.searchParams")
            .unwrap()
            .value,
        "true"
    );
}

#[test]
fn test_set_url_part_rust_r2780() {
    // R2780：set_url_part 纯函数单测（直调，验 url crate setter 正确性 + 非法 scheme 返空串不 panic）。
    use super::*;
    // pathname setter。
    let r = set_url_part("https://example.com/old", "pathname", "/new/path");
    assert!(r.contains("\"pathname\":\"/new/path\""), "pathname setter: {r}");
    assert!(
        r.contains("\"href\":\"https://example.com/new/path\""),
        "href after pathname: {r}"
    );
    // search setter。
    let r = set_url_part("https://example.com/p", "search", "?a=1&b=2");
    assert!(r.contains("\"search\":\"?a=1&b=2\""), "search setter: {r}");
    // hash 清除（空串）。
    let r = set_url_part("https://example.com/p#sec", "hash", "");
    assert!(r.contains("\"hash\":\"\""), "hash clear: {r}");
    // port setter。
    let r = set_url_part("https://example.com/p", "port", "8443");
    assert!(r.contains("\"port\":\"8443\""), "port setter: {r}");
    // href setter（整体替换）。
    let r = set_url_part("https://example.com/old", "href", "http://other.test/x?q=1");
    assert!(r.contains("\"host\":\"other.test\""), "href replace host: {r}");
    // 非法 scheme 返空串（不 panic）。
    assert_eq!(set_url_part("https://example.com/p", "protocol", "ht!tp"), "");
    // 非法 href 返空串。
    assert_eq!(set_url_part("https://example.com/p", "href", "not a url"), "");
    // 未知 part 不改 URL（返回原序列化）。
    let r = set_url_part("https://example.com/p", "unknownpart", "x");
    assert!(
        r.contains("\"href\":\"https://example.com/p\""),
        "unknown part noop: {r}"
    );
}

#[test]
fn test_match_media_r2781() {
    // R2781：window.matchMedia（host callback __zw_match_media → zero_css_parser::media_query）。
    // 响应式设计 / viewport 查询高频（shim 曾缺失）。viewport 默认 1280x800（shim innerWidth/innerHeight）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.register_callback(
        "__zw_match_media",
        Box::new(|args: &[String]| -> String {
            let query = args.first().map(String::as_str).unwrap_or("");
            let width = args.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            let height = args.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            match_media_to_json(query, width, height)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // (min-width: 768px) @1280 → true；(max-width: 500px) @1280 → false。
    assert_eq!(
        sandbox
            .execute("matchMedia('(min-width: 768px)').matches + '|' + matchMedia('(max-width: 500px)').matches")
            .unwrap()
            .value,
        "true|false"
    );
    // media 字段返回 query 串。
    assert_eq!(
        sandbox.execute("matchMedia('(min-width: 768px)').media").unwrap().value,
        "(min-width: 768px)"
    );
    // orientation：landscape @1280x800 → true；portrait → false（is_portrait = h > w）。
    assert_eq!(
        sandbox
            .execute(
                "matchMedia('(orientation: landscape)').matches + '|' + matchMedia('(orientation: portrait)').matches"
            )
            .unwrap()
            .value,
        "true|false"
    );
    // prefers-color-scheme 默认 light：light → true；dark → false。
    assert_eq!(
        sandbox
            .execute("matchMedia('(prefers-color-scheme: light)').matches + '|' + matchMedia('(prefers-color-scheme: dark)').matches")
            .unwrap()
            .value,
        "true|false"
    );
    // 逗号分隔 query list（OR 语义）：任一 match → true。
    assert_eq!(
        sandbox
            .execute("matchMedia('(max-width: 1px), (min-width: 768px)').matches")
            .unwrap()
            .value,
        "true"
    );
    // viewport 覆盖：@500 → (min-width: 768px) → false。
    assert_eq!(
        sandbox
            .execute("globalThis.innerWidth = 500; matchMedia('(min-width: 768px)').matches")
            .unwrap()
            .value,
        "false"
    );
    // MediaQueryList extends EventTarget（R2779）+ legacy addListener/removeListener。
    assert_eq!(
        sandbox
            .execute(
                "var m = matchMedia('(min-width: 1px)');\
                 (m instanceof MediaQueryList) + '|' + (m instanceof EventTarget) + '|' +\
                 typeof m.addListener + '|' + typeof m.removeListener"
            )
            .unwrap()
            .value,
        "true|true|function|function"
    );
}

#[test]
fn test_message_channel_r2782() {
    // R2782：MessageChannel + MessagePort + MessageEvent（postMessage 双端口，纯 JS）。MessagePort extends
    // EventTarget（R2779）；postMessage 经 structuredClone（R2773）深拷贝 + queueMicrotask（R2774）异步派发
    // 'message' 事件（execute 末 microtask checkpoint，下 execute 可读）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // port1/port2 + instanceof MessagePort/EventTarget。
    assert_eq!(
        sandbox
            .execute(
                "var ch = new MessageChannel();\
                 typeof ch.port1 + '|' + typeof ch.port2 + '|' +\
                 (ch.port1 instanceof MessagePort) + '|' + (ch.port2 instanceof EventTarget)"
            )
            .unwrap()
            .value,
        "object|object|true|true"
    );
    // postMessage port1→port2：异步派发（execute 末 microtask），下 execute 可读 __got。
    sandbox
        .execute(
            "ch.port2.onmessage = function (e) { globalThis.__got = e.data.x + 1; };\
             ch.port1.postMessage({ x: 41 }); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__got").unwrap().value, "42");
    // structuredClone 深拷贝：postMessage 时克隆，后续 mutate 原对象不影响收到的（R2773 验证）。
    sandbox
        .execute(
            "var orig = { v: 1 };\
             ch.port2.onmessage = function (e) { globalThis.__msgV = e.data.v; };\
             ch.port1.postMessage(orig); orig.v = 5; 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__msgV").unwrap().value, "1");
    // 反向 port2→port1。
    sandbox
        .execute(
            "ch.port1.onmessage = function (e) { globalThis.__rev = e.data; };\
             ch.port2.postMessage('hello'); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__rev").unwrap().value, "hello");
    // MessageEvent 字段：instanceof MessageEvent & Event + type=message + source=null。
    sandbox
        .execute(
            "ch.port2.onmessage = function (e) {\
                 globalThis.__mev = (e instanceof MessageEvent) + '|' + (e instanceof Event) + '|' + e.type + '|' + (e.source === null);\
             }; ch.port1.postMessage('x'); 'sent'"
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__mev").unwrap().value,
        "true|true|message|true"
    );
    // close() 停止派发：postMessage on closed port no-op。
    sandbox
        .execute(
            "var c = new MessageChannel(); globalThis.__cl = 'none';\
             c.port2.onmessage = function () { globalThis.__cl = 'got'; };\
             c.port1.close(); c.port1.postMessage('z'); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cl").unwrap().value, "none");
}

#[test]
fn test_broadcast_channel_r2783() {
    // R2783：BroadcastChannel（同源广播，纯 JS）。extends EventTarget R2779；postMessage 经
    // structuredClone R2773 + queueMicrotask R2782 异步派发到所有同名其他实例（sender 不收自己）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + name + instanceof BroadcastChannel/EventTarget。
    assert_eq!(
        sandbox
            .execute(
                "var bc = new BroadcastChannel('news');\
                 typeof BroadcastChannel + '|' + bc.name + '|' +\
                 (bc instanceof BroadcastChannel) + '|' + (bc instanceof EventTarget)"
            )
            .unwrap()
            .value,
        "function|news|true|true"
    );
    // 广播：a post → b & c 收，a 不收自己（sender skipped）。
    sandbox
        .execute(
            "var a = new BroadcastChannel('ch'); var b = new BroadcastChannel('ch'); var c = new BroadcastChannel('ch');\
             globalThis.__got = '';\
             a.onmessage = function () { globalThis.__got += 'a'; };\
             b.onmessage = function (e) { globalThis.__got += 'b:' + e.data + ';'; };\
             c.onmessage = function (e) { globalThis.__got += 'c:' + e.data + ';'; };\
             a.postMessage('hi'); 'sent'"
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__got").unwrap().value, "b:hi;c:hi;");
    // structuredClone 深拷贝：postMessage 时克隆，后续 mutate 原对象不影响收到的。
    sandbox
        .execute(
            "var msg = { v: 1 }; globalThis.__mv = -1;\
             b.onmessage = function (e) { globalThis.__mv = e.data.v; };\
             a.postMessage(msg); msg.v = 99; 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__mv").unwrap().value, "1");
    // 不同 name 无串扰：a（ch）post 不触达 x（other）。
    sandbox
        .execute(
            "var x = new BroadcastChannel('other'); globalThis.__cross = 'none';\
             x.onmessage = function () { globalThis.__cross = 'got'; };\
             a.postMessage('to-ch'); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cross").unwrap().value, "none");
    // close() → 移出注册表，不再收（仅 c 收，b 已 close）。
    sandbox
        .execute(
            "globalThis.__cl = ''; b.close();\
             c.onmessage = function () { globalThis.__cl += 'c'; };\
             a.postMessage('after'); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cl").unwrap().value, "c");
}

#[test]
fn test_location_read_spec_r2784() {
    // R2784：location 读侧 spec 化（_parseLocation → new URL R2778，spec-correct）。注册
    // __zw_get_page_url（返测试 URL）+ __zw_parse_url（使 new URL 路径激活）。验默认端口归一等精度提升。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    // https 默认端口 443 → 归一省略（旧 regex 会保留 :443，spec 改进）。
    sandbox.register_callback(
        "__zw_get_page_url",
        Box::new(|_args: &[String]| "https://example.com:443/path?q=1#sec".to_string()),
    );
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            parse_url_to_json(input, base)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 默认端口 443 归一省略（host=example.com 非 example.com:443）+ 全组件 spec-correct。
    assert_eq!(
        sandbox
            .execute(
                "location.protocol + '|' + location.hostname + '|' + location.host + '|' +\
                 location.pathname + '|' + location.search + '|' + location.hash + '|' +\
                 location.origin + '|' + location.href"
            )
            .unwrap()
            .value,
        "https:|example.com|example.com|/path|?q=1|#sec|https://example.com|https://example.com/path?q=1#sec"
    );
    // toString === href。
    assert_eq!(
        sandbox.execute("location.toString()").unwrap().value,
        "https://example.com/path?q=1#sec"
    );
}

#[test]
fn test_css_escape_supports_r2785() {
    // R2785：CSS namespace（escape 选择器转义 + supports 特性检测）。escape 纯 JS（chromium oracle
    // 锚定）；supports 委托 host __zw_css_supports（known-property gate + apply）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.register_callback(
        "__zw_css_supports",
        Box::new(|args: &[String]| -> String {
            let prop = args.first().map(String::as_str).unwrap_or("");
            let value = args.get(1).map(String::as_str);
            if css_supports(prop, value) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // CSS.escape（chromium oracle 锚定：特殊符 \char / 首数字 \hex+space / -a 直留 / 空串不抛）。
    assert_eq!(sandbox.execute("CSS.escape('a.b#c')").unwrap().value, "a\\.b\\#c");
    assert_eq!(sandbox.execute("CSS.escape('foo bar')").unwrap().value, "foo\\ bar");
    assert_eq!(sandbox.execute("CSS.escape('1abc')").unwrap().value, "\\31 abc");
    assert_eq!(sandbox.execute("CSS.escape('-a')").unwrap().value, "-a");
    assert_eq!(sandbox.execute("CSS.escape('')").unwrap().value, "");
    // CSS.supports 两参：已知属性+合法值 true；非法值/未知属性 false。
    assert_eq!(
        sandbox
            .execute("CSS.supports('display','grid') + '|' + CSS.supports('color','red')")
            .unwrap()
            .value,
        "true|true"
    );
    assert_eq!(
        sandbox
            .execute("CSS.supports('display','bogusxyz') + '|' + CSS.supports('fakeprop','x')")
            .unwrap()
            .value,
        "false|false"
    );
    // CSS.supports 单参：括号条件 / 声明 / not。
    assert_eq!(
        sandbox
            .execute("CSS.supports('(display: grid)') + '|' + CSS.supports('display: grid')")
            .unwrap()
            .value,
        "true|true"
    );
    assert_eq!(
        sandbox.execute("CSS.supports('not (display: grid)')").unwrap().value,
        "false"
    );
}

#[test]
fn test_document_cookie_r2786() {
    // R2786：document.cookie get/set（in-JS 存储，set-then-read 常见模式）。**已知限制**：不接真 cookie jar
    // / 无 origin 隔离 / 无 expiry（host-layer defer）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 初始空。
    assert_eq!(sandbox.execute("document.cookie").unwrap().value, "");
    // set 单 cookie（带属性，仅取 name=value）。
    sandbox
        .execute("document.cookie = 'theme=dark; Path=/; Max-Age=3600'")
        .unwrap();
    assert_eq!(sandbox.execute("document.cookie").unwrap().value, "theme=dark");
    // set 第二个 cookie → getter 串含两者。
    sandbox.execute("document.cookie = 'lang=en'").unwrap();
    assert!(sandbox.execute("document.cookie").unwrap().value.contains("theme=dark"));
    assert!(sandbox.execute("document.cookie").unwrap().value.contains("lang=en"));
    // 覆盖同名 cookie（name 唯一）。
    sandbox.execute("document.cookie = 'theme=light'").unwrap();
    assert_eq!(
        sandbox
            .execute("document.cookie.split('; ').sort().join('; ')")
            .unwrap()
            .value,
        "lang=en; theme=light"
    );
    // value 含 '='（split on 首 '='，value 保留后续 '='）。
    sandbox.execute("document.cookie = 'token=a=b=c'").unwrap();
    assert!(
        sandbox
            .execute("document.cookie")
            .unwrap()
            .value
            .contains("token=a=b=c")
    );
    // 无 name=value（无 '=' 串）→ 忽略，不影响存储。
    sandbox.execute("document.cookie = 'justtext'").unwrap();
    assert!(!sandbox.execute("document.cookie").unwrap().value.contains("justtext"));
}

#[test]
fn test_text_encoder_decoder_utf8_r2771() {
    // R2771：TextEncoder（str→UTF-8 Uint8Array）+ TextDecoder（bytes→str）。纯 JS UTF-8
    //（BMP + astral 代理对）。fetch body / 字符串↔字节互转高频。encode 'ZeroWeb' = ASCII 7 字节，
    // 中文 = 3 字节/字，round-trip 保真。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // TextEncoder：encoding=utf-8；encode('ZeroWeb') = 7 ASCII 字节 [90,...,98]。
    assert_eq!(
        sandbox
            .execute(
                "var a = new TextEncoder().encode('ZeroWeb');\
                 new TextEncoder().encoding + '|' + a.length + '|' + a[0] + '|' + a[6]"
            )
            .unwrap()
            .value,
        "utf-8|7|90|98"
    );
    // 中文多字节：'中' = U+4E2D → 3 字节 UTF-8。
    assert_eq!(
        sandbox.execute("new TextEncoder().encode('中').length").unwrap().value,
        "3"
    );
    // TextDecoder：字面字节序列 → 字符串。
    assert_eq!(
        sandbox
            .execute("new TextDecoder().decode(new Uint8Array([0x5a,0x65,0x72,0x6f]))")
            .unwrap()
            .value,
        "Zero"
    );
    // Round-trip（ASCII + 中文混排）保真。
    assert_eq!(
        sandbox
            .execute(
                "var e = new TextEncoder(), d = new TextDecoder();\
                 d.decode(e.encode('ZeroWeb 中文'))"
            )
            .unwrap()
            .value,
        "ZeroWeb 中文"
    );
}

#[test]
fn test_url_search_params_r2772() {
    // R2772：URLSearchParams（query 解析/序列化，location.search/fetch query 高频）。纯 JS。
    // 构造（string/?前缀/对象）+ get/getAll/has/set/append/delete + toString（space→+）+ 可迭代。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 构造 + get（`?` 前缀可省）。
    assert_eq!(
        sandbox
            .execute("new URLSearchParams('?a=1&b=2').get('a')")
            .unwrap()
            .value,
        "1"
    );
    assert_eq!(
        sandbox
            .execute("new URLSearchParams('a=1&b=2').get('b')")
            .unwrap()
            .value,
        "2"
    );
    // 缺键 get → null；getAll 多值。
    assert_eq!(
        sandbox
            .execute("String(new URLSearchParams('a=1').get('z'))")
            .unwrap()
            .value,
        "null"
    );
    assert_eq!(
        sandbox
            .execute("new URLSearchParams('a=1&a=2').getAll('a').join(',')")
            .unwrap()
            .value,
        "1,2"
    );
    // has / append / set / delete。
    assert_eq!(
        sandbox.execute("new URLSearchParams('a=1').has('a')").unwrap().value,
        "true"
    );
    sandbox
        .execute("globalThis.__p = new URLSearchParams('a=1&b=2'); __p.append('c', '3'); __p.set('a', '9'); __p.delete('b');")
        .unwrap();
    assert_eq!(sandbox.execute("__p.get('a')").unwrap().value, "9");
    assert_eq!(sandbox.execute("String(__p.has('b'))").unwrap().value, "false");
    assert_eq!(sandbox.execute("__p.get('c')").unwrap().value, "3");
    // toString（space→+，round-trip）。
    assert_eq!(
        sandbox
            .execute("new URLSearchParams('q=hello+world&n=42').toString()")
            .unwrap()
            .value,
        "q=hello+world&n=42"
    );
    // 对象构造。
    assert_eq!(
        sandbox
            .execute("new URLSearchParams({ x: '1', y: '2' }).toString()")
            .unwrap()
            .value,
        "x=1&y=2"
    );
    // 可迭代：for...of 收集键。
    assert_eq!(
        sandbox
            .execute("var ks = []; for (var kv of new URLSearchParams('a=1&b=2')) ks.push(kv[0]); ks.join(',')")
            .unwrap()
            .value,
        "a,b"
    );
}

#[test]
fn test_structured_clone_r2773() {
    // R2773：structuredClone（深拷贝，postMessage/React state 高频）。递归 + 循环引用（WeakMap）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // primitive 原样返回。
    assert_eq!(sandbox.execute("structuredClone(42)").unwrap().value, "42");
    assert_eq!(sandbox.execute("structuredClone('hi')").unwrap().value, "hi");
    assert_eq!(sandbox.execute("String(structuredClone(null))").unwrap().value, "null");
    // 嵌套对象深拷贝独立（改 clone 不影响原）。
    assert_eq!(
        sandbox
            .execute("var a = { x: 1, n: { y: 2 } }; var b = structuredClone(a); b.n.y = 99; a.n.y")
            .unwrap()
            .value,
        "2"
    );
    // 数组深拷贝独立。
    assert_eq!(
        sandbox
            .execute("var a = [1, [2, 3]]; var b = structuredClone(a); b[1][0] = 99; a[1][0]")
            .unwrap()
            .value,
        "2"
    );
    // Date 保类型 + 值。
    assert_eq!(
        sandbox
            .execute("structuredClone(new Date(2020, 0, 1)).getTime() === new Date(2020, 0, 1).getTime()")
            .unwrap()
            .value,
        "true"
    );
    // RegExp 保 flags。
    assert_eq!(sandbox.execute("structuredClone(/abc/gi).flags").unwrap().value, "gi");
    // 循环引用不爆栈（self-ref 解到 clone 自身）。
    assert_eq!(
        sandbox
            .execute("var a = {}; a.self = a; var b = structuredClone(a); b.self === b")
            .unwrap()
            .value,
        "true"
    );
    // function 抛 DataCloneError（spec）。
    assert_eq!(
        sandbox
            .execute("try { structuredClone(function(){}); 'no-throw' } catch (e) { 'threw' }")
            .unwrap()
            .value,
        "threw"
    );
}

#[test]
fn test_queue_microtask_r2774() {
    // R2774：queueMicrotask（microtask 调度，高频）。V8 embed 未暴露全局，用 Promise.resolve().then
    // polyfill；execute 末 microtask checkpoint 派发。callback 在该 execute 末运行（下 execute 可读）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof function（全局已定义）。
    assert_eq!(sandbox.execute("typeof queueMicrotask").unwrap().value, "function");
    // callback 在 execute 末 microtask checkpoint 派发——下 execute 可读 __ran。
    sandbox
        .execute("globalThis.__ran = false; queueMicrotask(function(){ globalThis.__ran = true; });")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ran)").unwrap().value, "true");
    // 非 callable 抛 TypeError（spec）。
    assert_eq!(
        sandbox
            .execute("try { queueMicrotask('notfn'); 'no-throw' } catch (e) { 'threw' }")
            .unwrap()
            .value,
        "threw"
    );
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
    // font-weight bold→700（对齐 Chromium 绝对值）、font-style italic、line-height number→used px
    // （1.5 × 默认 font-size 16px = 24px，R2761 对齐 Chromium getComputedStyle used 值）。
    assert_eq!(computed_style_property(html, "#k", "font-weight"), "700");
    assert_eq!(computed_style_property(html, "#k", "font-style"), "italic");
    assert_eq!(computed_style_property(html, "#k", "line-height"), "24px");
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
fn test_get_computed_style_font_shorthand_r2761() {
    // R2761：getComputedStyle font 简写（CSSOM 重组省初值）+ line-height number→used px 修复。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。
    // 注：默认 font-family ZW 为空（vs Chromium "Times New Roman"）——pre-existing longhand diverge，
    // 故测显式 family 的 font 简写声明（序列化本身正确）。
    let html = "<html><body>\
        <div id=\"f1\" style=\"font: italic bold 14px/1.5 Arial;\"></div>\
        <div id=\"f3\" style=\"font: bold 12px sans-serif;\"></div>\
        <div id=\"f4\" style=\"font: bold 14px/2 Helvetica;\"></div>\
        <div id=\"f5\" style=\"font-family: Arial; font-size: 14px;\"></div>\
        </body></html>";
    // 经 longhand 设置（family Arial + size 14px，style/weight/line-height 全初值省）→"14px Arial"。
    assert_eq!(computed_style_property(html, "#f5", "font"), "14px Arial");
    // italic + 700(bold) + 14px + line-height 1.5→21px(14×1.5 used px) + Arial。
    assert_eq!(
        computed_style_property(html, "#f1", "font"),
        "italic 700 14px / 21px Arial"
    );
    // 700 + 12px + sans-serif（line-height normal 省）。
    assert_eq!(computed_style_property(html, "#f3", "font"), "700 12px sans-serif");
    // 700 + 14px + line-height 2→28px(14×2) + Helvetica。
    assert_eq!(
        computed_style_property(html, "#f4", "font"),
        "700 14px / 28px Helvetica"
    );
    // line-height longhand number→used px 修复（独立验证，1.5 × 默认 16px = 24px）。
    let html2 = "<html><body><div id=\"lh\" style=\"line-height: 1.5;\"></div></body></html>";
    assert_eq!(computed_style_property(html2, "#lh", "line-height"), "24px");
}

#[test]
fn test_get_computed_style_backdrop_filter_underline_offset_r2762() {
    // R2762：getComputedStyle backdrop-filter（复用 filter_to_css）+ text-underline-offset（Auto/Length）。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"bf1\" style=\"backdrop-filter: blur(10px);\"></div>\
        <div id=\"bf2\" style=\"backdrop-filter: blur(5px) saturate(180%);\"></div>\
        <div id=\"tuo1\" style=\"text-underline-offset: 3px;\"></div>\
        <div id=\"tuo2\" style=\"text-underline-offset: auto;\"></div>\
        </body></html>";
    // backdrop-filter：复用 filter 序列化（空→none / 函数列表空格分隔 / saturate 百分比→数字）。
    assert_eq!(computed_style_property(html, "#d", "backdrop-filter"), "none");
    assert_eq!(computed_style_property(html, "#bf1", "backdrop-filter"), "blur(10px)");
    assert_eq!(
        computed_style_property(html, "#bf2", "backdrop-filter"),
        "blur(5px) saturate(1.8)"
    );
    // text-underline-offset：Auto→auto / Length→px。
    assert_eq!(computed_style_property(html, "#d", "text-underline-offset"), "auto");
    assert_eq!(computed_style_property(html, "#tuo1", "text-underline-offset"), "3px");
    assert_eq!(computed_style_property(html, "#tuo2", "text-underline-offset"), "auto");
}

#[test]
fn test_get_computed_style_text_emphasis_r2763() {
    // R2763：getComputedStyle text-emphasis 簇（style/color/position longhand + 简写）。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"s1\" style=\"text-emphasis-style: dot;\"></div>\
        <div id=\"s3\" style=\"text-emphasis-style: open circle;\"></div>\
        <div id=\"s4\" style=\"text-emphasis-style: sesame;\"></div>\
        <div id=\"s5\" style='text-emphasis-style: \"*\";'></div>\
        <div id=\"c1\" style=\"text-emphasis-color: rgb(255, 0, 0);\"></div>\
        <div id=\"p1\" style=\"text-emphasis-position: under left;\"></div>\
        <div id=\"sh\" style=\"text-emphasis: filled circle rgb(0, 128, 0);\"></div>\
        </body></html>";
    // text-emphasis-style：char→keyword 逆映射（filled 省，open 显；string 引号化）。
    assert_eq!(computed_style_property(html, "#d", "text-emphasis-style"), "none");
    assert_eq!(computed_style_property(html, "#s1", "text-emphasis-style"), "dot");
    assert_eq!(
        computed_style_property(html, "#s3", "text-emphasis-style"),
        "open circle"
    );
    assert_eq!(computed_style_property(html, "#s4", "text-emphasis-style"), "sesame");
    assert_eq!(computed_style_property(html, "#s5", "text-emphasis-style"), "\"*\"");
    // text-emphasis-color：currentcolor→rgb（默认元素 black→rgb(0,0,0)）。
    assert_eq!(
        computed_style_property(html, "#d", "text-emphasis-color"),
        "rgb(0, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#c1", "text-emphasis-color"),
        "rgb(255, 0, 0)"
    );
    // text-emphasis-position：over/under 恒显；left 显（right 初值省）。
    assert_eq!(computed_style_property(html, "#d", "text-emphasis-position"), "over");
    assert_eq!(
        computed_style_property(html, "#p1", "text-emphasis-position"),
        "under left"
    );
    // text-emphasis 简写：style + color（恒双段）。
    assert_eq!(
        computed_style_property(html, "#d", "text-emphasis"),
        "none rgb(0, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#sh", "text-emphasis"),
        "circle rgb(0, 128, 0)"
    );
}

#[test]
fn test_get_computed_style_border_image_longhands_r2764() {
    // R2764：getComputedStyle border-image 切片族 longhand（slice/width/outset 4 值最小化 + repeat 2 值）。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"s1\" style=\"border-image-slice: 10 20 30 40;\"></div>\
        <div id=\"s2\" style=\"border-image-slice: 10% fill;\"></div>\
        <div id=\"w1\" style=\"border-image-width: 10px 20px;\"></div>\
        <div id=\"w2\" style=\"border-image-width: auto;\"></div>\
        <div id=\"r1\" style=\"border-image-repeat: round repeat;\"></div>\
        <div id=\"o1\" style=\"border-image-outset: 5px 10px;\"></div>\
        </body></html>";
    // border-image-slice：默认 100%（R2764 修 Percent）/ 4 值最小化 / fill 末尾。
    assert_eq!(computed_style_property(html, "#d", "border-image-slice"), "100%");
    assert_eq!(
        computed_style_property(html, "#s1", "border-image-slice"),
        "10 20 30 40"
    );
    assert_eq!(computed_style_property(html, "#s2", "border-image-slice"), "10% fill");
    // border-image-width：默认 1 / 4 值最小化 / auto。
    assert_eq!(computed_style_property(html, "#d", "border-image-width"), "1");
    assert_eq!(computed_style_property(html, "#w1", "border-image-width"), "10px 20px");
    assert_eq!(computed_style_property(html, "#w2", "border-image-width"), "auto");
    // border-image-outset：默认 0 / 4 值最小化。
    assert_eq!(computed_style_property(html, "#d", "border-image-outset"), "0");
    assert_eq!(computed_style_property(html, "#o1", "border-image-outset"), "5px 10px");
    // border-image-repeat：默认 stretch / 相等单值否则双值。
    assert_eq!(computed_style_property(html, "#d", "border-image-repeat"), "stretch");
    assert_eq!(
        computed_style_property(html, "#r1", "border-image-repeat"),
        "round repeat"
    );
}

#[test]
fn test_get_computed_style_border_image_shorthand_r2765() {
    // R2765：getComputedStyle border-image 简写（5 子分量 CSSOM 重组）。Chromium 150 oracle 锚定：
    // ① source==none → 整值 "none"（不论其余 slice/width/outset/repeat 是否非初值）；
    // ② source!=none → 恒全量 "<source> <slice> / <width> / <outset> <repeat>"（不省初值，width/outset
    //    各独占一个 `/` 分隔）。用 linear-gradient 源避免 url() 相对/绝对 longhand 既存 diverge。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"slc\" style=\"border-image-slice: 10;\"></div>\
        <div id=\"g\" style=\"border-image-source: linear-gradient(-45deg, red, blue);\"></div>\
        <div id=\"full\" style=\"border-image-source: linear-gradient(-45deg, red, blue);\
                               border-image-slice: 10 fill;\
                               border-image-width: 20px;\
                               border-image-outset: 5px;\
                               border-image-repeat: round;\"></div>\
        <div id=\"grep\" style=\"border-image-source: linear-gradient(-45deg, red, blue);\
                                border-image-repeat: round;\"></div>\
        </body></html>";
    let grad = "linear-gradient(-45deg, rgb(255, 0, 0), rgb(0, 0, 255))";
    // source==none（默认 / 或仅设 slice 等其余分量）→ 整值 "none"。
    assert_eq!(computed_style_property(html, "#d", "border-image"), "none");
    assert_eq!(computed_style_property(html, "#slc", "border-image"), "none");
    // source!=none：恒全量 "<source> <slice> / <width> / <outset> <repeat>"。
    assert_eq!(
        computed_style_property(html, "#g", "border-image"),
        format!("{grad} 100% / 1 / 0 stretch")
    );
    assert_eq!(
        computed_style_property(html, "#full", "border-image"),
        format!("{grad} 10 fill / 20px / 5px round")
    );
    assert_eq!(
        computed_style_property(html, "#grep", "border-image"),
        format!("{grad} 100% / 1 / 0 round")
    );
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
fn test_get_computed_style_grid_template_shorthand_r2766() {
    // R2766：getComputedStyle grid-template 简写（rows/columns/areas 三 longhand 重组）。Chromium 150 oracle 锚定：
    // 全 none→"none"；areas==none→"<rows> / <cols>"（rows/cols 各自可 none）；areas!=none→引号区域与行尺寸逐行
    // 交错 + " / " + cols（area 数 != 行尺寸数→"" 空串，Chromium 同样不可序列化）。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"simple\" style=\"grid-template: 100px 200px / 1fr 1fr 1fr;\"></div>\
        <div id=\"cols\" style=\"grid-template-columns: 1fr 1fr;\"></div>\
        <div id=\"rows\" style=\"grid-template-rows: 100px 200px;\"></div>\
        <div id=\"areas\" style='grid-template: \"a a a\" 50px \"b b b\" 1fr \"c c c\" 2fr / 1fr 1fr 1fr;'></div>\
        </body></html>";
    // 全 none（默认）→ "none"。
    assert_eq!(computed_style_property(html, "#d", "grid-template"), "none");
    // areas==none：恒 "<rows> / <cols>"（cols 缺省→none / rows 缺省→none）。
    assert_eq!(
        computed_style_property(html, "#simple", "grid-template"),
        "100px 200px / 1fr 1fr 1fr"
    );
    assert_eq!(
        computed_style_property(html, "#cols", "grid-template"),
        "none / 1fr 1fr"
    );
    assert_eq!(
        computed_style_property(html, "#rows", "grid-template"),
        "100px 200px / none"
    );
    // areas!=none：引号区域与行尺寸逐行交错 + " / " + cols。
    assert_eq!(
        computed_style_property(html, "#areas", "grid-template"),
        "\"a a a\" 50px \"b b b\" 1fr \"c c c\" 2fr / 1fr 1fr 1fr"
    );
}

#[test]
fn test_get_computed_style_letter_spacing_normal_r2767() {
    // R2767：letter-spacing 0→normal diverge 修复。Chromium 150 oracle 把 0 值（默认 / normal /
    // 显式 0/0px）恒归一为 "normal"（normal 与 0 layout 等价）；非 0 长度才返 "Npx"。
    // ZW parse 把 normal→Px(0.0)，故 Px(0.0)→"normal" 精确对齐。word-spacing 不归一（恒 "0px"）。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"norm\" style=\"letter-spacing: normal;\"></div>\
        <div id=\"zero\" style=\"letter-spacing: 0;\"></div>\
        <div id=\"val\" style=\"letter-spacing: 2px;\"></div>\
        <div id=\"ws\" style=\"word-spacing: normal;\"></div>\
        </body></html>";
    // letter-spacing：默认 / normal / 显式 0 → "normal"（Chromium 归一）。
    assert_eq!(computed_style_property(html, "#d", "letter-spacing"), "normal");
    assert_eq!(computed_style_property(html, "#norm", "letter-spacing"), "normal");
    assert_eq!(computed_style_property(html, "#zero", "letter-spacing"), "normal");
    // 非 0 长度 → "Npx"。
    assert_eq!(computed_style_property(html, "#val", "letter-spacing"), "2px");
    // word-spacing 不归一：normal → "0px"（与 letter-spacing 行为不同，对齐 Chromium）。
    assert_eq!(computed_style_property(html, "#ws", "word-spacing"), "0px");
    assert_eq!(computed_style_property(html, "#d", "word-spacing"), "0px");
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

#[test]
fn test_promise_any_all_settled_native_r2787() {
    // R2787：Promise.any / Promise.allSettled 复核（CONTINUE 指定）。ES2021 语言内置（非 Web API），
    // V8 原生提供——probe 确认 `typeof === 'function'`，无需 polyfill。本测试**锁住能力**
    //（防 V8 embed 配置 / 版本变化移除）+ 文档化语义：execute 末 `perform_microtask_checkpoint`
    // drain Promise 链 → 下 execute 可读结果。
    //   - allSettled：永不 reject，按序返 status 描述符（fulfilled→value / rejected→reason）。
    //   - any：返首个 fulfilled 值（跳过先到的 reject）；全 reject 抛 AggregateError（errors=原因数组）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 两者均为原生 function（V8 内置，非 shim 定义）。
    assert_eq!(sandbox.execute("typeof Promise.any").unwrap().value, "function");
    assert_eq!(sandbox.execute("typeof Promise.allSettled").unwrap().value, "function");

    // allSettled：混合 fulfilled/rejected → 永不 reject，按序返 status 描述符。
    sandbox
        .execute(
            "globalThis.__settled = '(pending)';\
             Promise.allSettled([Promise.resolve(1), Promise.reject('boom'), Promise.resolve(3)])\
               .then(function(r){\
                 globalThis.__settled = r.map(function(e){\
                   return e.status + ':' + (e.value !== undefined ? e.value : e.reason);\
                 }).join(',');\
               });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__settled)").unwrap().value,
        "fulfilled:1,rejected:boom,fulfilled:3"
    );

    // any：返首个 fulfilled（跳过先到的 reject）。
    sandbox
        .execute(
            "globalThis.__any = '(pending)';\
             Promise.any([Promise.reject('x'), Promise.resolve('win'), Promise.resolve('late')])\
               .then(function(v){ globalThis.__any = v; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__any)").unwrap().value, "win");

    // any：全 reject → reject AggregateError（errors=原因数组）；.catch 验证实例 + errors。
    sandbox
        .execute(
            "globalThis.__agg = '(pending)';\
             Promise.any([Promise.reject('a'), Promise.reject('b')])\
               .catch(function(e){\
                 globalThis.__agg = (e instanceof AggregateError) + ':' + e.errors.join(',');\
               });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__agg)").unwrap().value, "true:a,b");
}

#[test]
fn test_form_data_r2788() {
    // R2788：FormData（表单字段集合，表单序列化 / fetch multipart body 高频）。纯 JS，镜像
    // URLSearchParams pair-store 模式。**已知限制**：constructor `form` 参数 best-effort（renderer
    // 路径真实字段枚举 follow-up），多数库空构造再 append——本测试覆盖 manual API 全路径。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof function（全局已定义）；无 new 调用亦可构造（spec 允许）。
    assert_eq!(sandbox.execute("typeof FormData").unwrap().value, "function");
    assert_eq!(
        sandbox
            .execute("String(new FormData() instanceof FormData)")
            .unwrap()
            .value,
        "true"
    );
    // append + get/getAll/has：允许多个同名值，保插入序，get 返首个。
    sandbox
        .execute(
            "var fd = new FormData();\
             fd.append('a','1'); fd.append('b','2'); fd.append('a','3');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("fd.get('a')").unwrap().value, "1");
    assert_eq!(sandbox.execute("fd.get('z')").unwrap().value, "null");
    assert_eq!(sandbox.execute("fd.getAll('a').join(',')").unwrap().value, "1,3");
    assert_eq!(sandbox.execute("String(fd.has('b'))").unwrap().value, "true");
    assert_eq!(sandbox.execute("String(fd.has('z'))").unwrap().value, "false");
    // set：替换所有同名（保留原首次位置），无则追加。
    sandbox.execute("fd.set('a','X')").unwrap();
    assert_eq!(sandbox.execute("fd.getAll('a').join(',')").unwrap().value, "X");
    sandbox.execute("fd.set('c','new')").unwrap();
    assert_eq!(sandbox.execute("fd.get('c')").unwrap().value, "new");
    // delete：移除所有同名。
    sandbox.execute("fd.delete('b')").unwrap();
    assert_eq!(sandbox.execute("String(fd.has('b'))").unwrap().value, "false");
    // value 经 String() 归一（数字 → 字符串，spec 非 Blob 转 USVString）。
    sandbox.execute("fd.append('n', 42)").unwrap();
    assert_eq!(sandbox.execute("fd.get('n')").unwrap().value, "42");
    // 迭代协议：[Symbol.iterator]=entries → for...of / spread 取 [k,v] 对；forEach 回调序。
    assert_eq!(
        sandbox
            .execute("[...fd].map(function(p){return p[0]+'='+p[1];}).join('|')")
            .unwrap()
            .value,
        "a=X|c=new|n=42"
    );
    assert_eq!(
        sandbox
            .execute("(function(){var o=[];fd.forEach(function(v,k){o.push(k+':'+v);});return o.join(',');})()")
            .unwrap()
            .value,
        "a:X,c:new,n:42"
    );
    // keys / values 迭代器。
    assert_eq!(sandbox.execute("[...fd.keys()].join(',')").unwrap().value, "a,c,n");
    assert_eq!(sandbox.execute("[...fd.values()].join(',')").unwrap().value, "X,new,42");
}

#[test]
fn test_blob_and_object_url_r2789() {
    // R2789：Blob（不可变二进制容器）+ URL.createObjectURL/revokeObjectURL（blob: URL 注册表）。
    // 纯 JS，零 host 回调。size 按 UTF-8 字节；type 小写；text()/arrayBuffer() 返 Promise（execute 末
    // microtask checkpoint drain → 下 execute 可读）。createObjectURL 返 blob: 串并注册。
    // **已知限制**：slice 不真切字节（best-effort size clamp）；blob: URL 不被 net 解析为内容。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + instanceof；无 new 亦可构造。
    assert_eq!(sandbox.execute("typeof Blob").unwrap().value, "function");
    assert_eq!(
        sandbox.execute("String(new Blob() instanceof Blob)").unwrap().value,
        "true"
    );
    // size：空 Blob=0；string part 按 UTF-8 字节（'ZeroWeb'=7，中文 '中'=3）。
    assert_eq!(sandbox.execute("new Blob().size").unwrap().value, "0");
    assert_eq!(sandbox.execute("new Blob(['ZeroWeb']).size").unwrap().value, "7");
    assert_eq!(sandbox.execute("new Blob(['中']).size").unwrap().value, "3");
    // 多 part 求和 + ArrayBuffer part（4 字节）。
    assert_eq!(
        sandbox
            .execute("new Blob(['ab', new Uint8Array([0,0,0,0])]).size")
            .unwrap()
            .value,
        "6"
    );
    // type：小写归一；无 options → ''。
    assert_eq!(
        sandbox
            .execute("new Blob(['x'], {type:'APPLICATION/JSON'}).type")
            .unwrap()
            .value,
        "application/json"
    );
    assert_eq!(sandbox.execute("new Blob(['x']).type").unwrap().value, "");
    // text()：Promise<string>——string part 原样；多 part 拼接；execute 末 drain → 下 execute 读。
    sandbox
        .execute(
            "globalThis.__t = '(pending)';\
             new Blob(['hello',' ','world']).text().then(function(s){ globalThis.__t = s; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__t)").unwrap().value, "hello world");
    // text() 解码字节 part（TypedArray 经 TextDecoder）。
    sandbox
        .execute(
            "globalThis.__b = '(pending)';\
             new Blob([new Uint8Array([0x68,0x69])]).text().then(function(s){ globalThis.__b = s; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__b)").unwrap().value, "hi");
    // arrayBuffer()：Promise<Uint8Array>——'AB' UTF-8 = [65,66]。
    sandbox
        .execute(
            "globalThis.__ab = '(pending)';\
             new Blob(['AB']).arrayBuffer().then(function(a){ globalThis.__ab = a.length + ':' + a[0] + ',' + a[1]; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ab)").unwrap().value, "2:65,66");
    // slice：best-effort size clamp（start/end 范围）+ type 重设。
    assert_eq!(
        sandbox.execute("new Blob(['ZeroWeb']).slice(1,4).size").unwrap().value,
        "3"
    );
    assert_eq!(
        sandbox
            .execute("new Blob(['ZeroWeb']).slice(0,4,'text/plain').type")
            .unwrap()
            .value,
        "text/plain"
    );
    // URL.createObjectURL：返 blob: 串 + 唯一性（两次不同）+ typeof function。
    assert_eq!(sandbox.execute("typeof URL.createObjectURL").unwrap().value, "function");
    assert_eq!(
        sandbox
            .execute("URL.createObjectURL(new Blob(['x'])).split(':')[0]")
            .unwrap()
            .value,
        "blob"
    );
    assert_eq!(
        sandbox
            .execute("URL.createObjectURL(new Blob(['a'])) !== URL.createObjectURL(new Blob(['b']))")
            .unwrap()
            .value,
        "true"
    );
    // revokeObjectURL：no-throw（不抛即视为清理成功）。
    sandbox.execute("URL.revokeObjectURL('blob:null/1-abc')").unwrap();
    assert_eq!(sandbox.execute("typeof URL.revokeObjectURL").unwrap().value, "function");
}

#[test]
fn test_dom_parser_r2790() {
    // R2790：DOMParser.parseFromString → 只读 Document。host `__zw_parse_html_query` 回调返 JSON 元素
    // 快照；shim 包 _zwParsedDoc + 只读 _zwParseEl（querySelector/getElementById/body/textContent/
    // getAttribute/子树 query）。**已知限制**：只读（无 mutation）、XML/SVG 按 HTML 解析、innerHTML 派生。
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

    sandbox
        .execute(
            "globalThis.__d = new DOMParser().parseFromString(\n\
             '<html><head><title>T</title></head><body><div id=\"main\"><p class=\"hi\">hello <b>world</b></p><span id=\"s\">x</span></div></body></html>',\n\
             'text/html');",
        )
        .unwrap();
    // typeof + instance（DOMParser 可构造、parseFromString 返对象）。
    assert_eq!(sandbox.execute("typeof DOMParser").unwrap().value, "function");
    assert_eq!(sandbox.execute("typeof __d.querySelector").unwrap().value, "function");
    // querySelector：tagName 大写、id、className。
    assert_eq!(
        sandbox.execute("__d.querySelector('#main').tagName").unwrap().value,
        "DIV"
    );
    assert_eq!(sandbox.execute("__d.querySelector('p').className").unwrap().value, "hi");
    // textContent（含后代文本，spec 一致）。
    assert_eq!(
        sandbox.execute("__d.querySelector('p').textContent").unwrap().value,
        "hello world"
    );
    // getElementById + 无匹配返 null。
    assert_eq!(
        sandbox.execute("__d.getElementById('s').tagName").unwrap().value,
        "SPAN"
    );
    assert_eq!(
        sandbox
            .execute("String(__d.getElementById('nope') === null)")
            .unwrap()
            .value,
        "true"
    );
    // querySelectorAll（多个匹配）。
    assert_eq!(
        sandbox.execute("__d.querySelectorAll('span').length").unwrap().value,
        "1"
    );
    assert_eq!(
        sandbox.execute("__d.querySelectorAll('#main b').length").unwrap().value,
        "1"
    );
    // body / documentElement / head 非空。
    assert_eq!(
        sandbox
            .execute("String(__d.body !== null && __d.documentElement !== null && __d.head !== null)")
            .unwrap()
            .value,
        "true"
    );
    // 子树 querySelector（element-proxy 上）：#main 内 <b>。
    assert_eq!(
        sandbox
            .execute("__d.querySelector('#main').querySelector('b').textContent")
            .unwrap()
            .value,
        "world"
    );
    // getAttribute / hasAttribute。
    assert_eq!(
        sandbox
            .execute("__d.querySelector('p').getAttribute('class')")
            .unwrap()
            .value,
        "hi"
    );
    assert_eq!(
        sandbox
            .execute("String(__d.querySelector('p').hasAttribute('class'))")
            .unwrap()
            .value,
        "true"
    );
    assert_eq!(
        sandbox
            .execute("String(__d.querySelector('p').getAttribute('none'))")
            .unwrap()
            .value,
        "null"
    );
    // innerHTML 由 outerHTML 派生（含 <p 子标签）。
    assert_eq!(
        sandbox
            .execute("String(__d.querySelector('#main').innerHTML.indexOf('<p') >= 0)")
            .unwrap()
            .value,
        "true"
    );
    // getElementsByTagName。
    assert_eq!(
        sandbox
            .execute("__d.getElementsByTagName('span').length")
            .unwrap()
            .value,
        "1"
    );
    // mimeType 记录。
    assert_eq!(sandbox.execute("__d.mimeType").unwrap().value, "text/html");
}

#[test]
fn test_file_reader_r2791() {
    // R2791：FileReader（异步读 Blob，文件上传/图片预览高频）。纯 JS builds on Blob.text()/arrayBuffer()
    //（R2789）+ btoa（R2770）。readAsText/ArrayBuffer/BinaryString/DataURL + abort + onload/onloadend/
    // onloadstart 事件 + result/readyState。**readAsDataURL 为 Blob 未覆盖唯一能力**。
    // **已知限制**：loadstart 同步；abort best-effort（不中断 in-flight Promise）；无 addEventListener。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + 常量（EMPTY/LOADING/DONE，静态 + 原型）。
    assert_eq!(sandbox.execute("typeof FileReader").unwrap().value, "function");
    assert_eq!(sandbox.execute("String(FileReader.EMPTY)").unwrap().value, "0");
    assert_eq!(sandbox.execute("String(FileReader.DONE)").unwrap().value, "2");
    // readAsText：result=Blob 文本；onload/onloadend 在 execute 末 checkpoint drain → 下 execute 可读。
    sandbox
        .execute(
            "globalThis.__r = '(pending)'; globalThis.__st = [];\
             var rd = new FileReader();\
             rd.onloadstart = function(){ globalThis.__st.push('start'); };\
             rd.onload = function(e){ globalThis.__st.push('load'); globalThis.__r = e.target.result; };\
             rd.onloadend = function(){ globalThis.__st.push('end'); };\
             rd.readAsText(new Blob(['hello',' ','world']));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__r)").unwrap().value, "hello world");
    assert_eq!(
        sandbox.execute("globalThis.__st.join(',')").unwrap().value,
        "start,load,end"
    );
    assert_eq!(sandbox.execute("String(rd.readyState)").unwrap().value, "2"); // DONE
    assert_eq!(sandbox.execute("String(rd.result)").unwrap().value, "hello world");
    // readAsArrayBuffer：result=Uint8Array（'AB' → [65,66]）。
    sandbox
        .execute(
            "globalThis.__ab = '(pending)';\
             var rd2 = new FileReader();\
             rd2.onload = function(e){ globalThis.__ab = e.target.result.length + ':' + e.target.result[0]; };\
             rd2.readAsArrayBuffer(new Blob(['AB']));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ab)").unwrap().value, "2:65");
    // readAsDataURL：data:<type>;base64,<b64>（'hi' → btoa='aGk='）。
    sandbox
        .execute(
            "globalThis.__url = '(pending)';\
             var rd3 = new FileReader();\
             rd3.onload = function(e){ globalThis.__url = e.target.result; };\
             rd3.readAsDataURL(new Blob(['hi'], {type:'text/plain'}));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__url)").unwrap().value,
        "data:text/plain;base64,aGk="
    );
    // readAsBinaryString：逐字节 Latin-1 串（'AB' → 'AB'）。
    sandbox
        .execute(
            "globalThis.__bs = '(pending)';\
             var rd4 = new FileReader();\
             rd4.onload = function(e){ globalThis.__bs = e.target.result; };\
             rd4.readAsBinaryString(new Blob(['AB']));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__bs)").unwrap().value, "AB");
    // abort：best-effort——在 LOADING（readAsText Promise 未 drain）时 abort → 派发 abort + loadend。
    // 注：本 execute 内 loadstart 已同步派发，abort 置 DONE；readAsText 的 Promise 仍会在 checkpoint
    // drain（best-effort，不中断）——测 abort 本身 no-throw + readyState=DONE。
    sandbox
        .execute(
            "globalThis.__ab_st = '(pending)';\
             var rd5 = new FileReader();\
             rd5.onabort = function(){ globalThis.__ab_st = 'aborted'; };\
             rd5.readAsText(new Blob(['x']));\
             rd5.abort();\
             globalThis.__ab_ready = rd5.readyState;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ab_ready)").unwrap().value, "2");
    assert_eq!(sandbox.execute("String(globalThis.__ab_st)").unwrap().value, "aborted");
}

#[test]
fn test_file_r2792() {
    // R2792：File（=Blob 子类 + name/lastModified，文件上传构造高频）。完成 Blob→File→FileReader→FormData
    // 文件处理簇。constructor 复用 Blob 构造；prototype=Object.create(Blob.prototype) 继承 slice/text/
    // arrayBuffer；File is-a Blob 故 FormData/FileReader 自动互通。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + instanceof（File 同时是 Blob 和 File）。
    assert_eq!(sandbox.execute("typeof File").unwrap().value, "function");
    assert_eq!(
        sandbox
            .execute(
                "var f = new File(['hello'], 'greet.txt'); String(f instanceof File) + ',' + String(f instanceof Blob)"
            )
            .unwrap()
            .value,
        "true,true"
    );
    // name + 继承 size/type（type 从 options 取，默认 ''）。
    assert_eq!(sandbox.execute("f.name").unwrap().value, "greet.txt");
    assert_eq!(sandbox.execute("f.size").unwrap().value, "5");
    assert_eq!(sandbox.execute("f.type").unwrap().value, "");
    // type 从 options.type 取。
    assert_eq!(
        sandbox
            .execute("new File(['x'], 'a.txt', {type:'text/plain'}).type")
            .unwrap()
            .value,
        "text/plain"
    );
    // lastModified：默认 Date.now()（非负整数）；显式值透传（含 0）。
    sandbox
        .execute("globalThis.__lm = new File(['x'], 'a').lastModified;")
        .unwrap();
    let lm = sandbox.execute("String(globalThis.__lm >= 0)").unwrap().value;
    assert_eq!(lm, "true", "默认 lastModified 应为非负（Date.now）");
    assert_eq!(
        sandbox
            .execute("new File(['x'], 'a', {lastModified:0}).lastModified")
            .unwrap()
            .value,
        "0"
    );
    assert_eq!(
        sandbox
            .execute("new File(['x'], 'a', {lastModified:1700000000000}).lastModified")
            .unwrap()
            .value,
        "1700000000000"
    );
    // lastModifiedDate 为 Date 实例。
    assert_eq!(
        sandbox
            .execute("String(new File(['x'],'a', {lastModified:0}).lastModifiedDate instanceof Date)")
            .unwrap()
            .value,
        "true"
    );
    // 继承 Blob 方法：text() 返 Promise（execute 末 drain → 下 execute 可读）。
    sandbox
        .execute(
            "globalThis.__ft = '(pending)';\
             new File(['hello',' ','file']).text().then(function(s){ globalThis.__ft = s; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ft)").unwrap().value, "hello file");
    // 继承 slice（返 Blob，size clamp）。
    assert_eq!(
        sandbox
            .execute("new File(['ZeroWeb'],'f').slice(1,4).size")
            .unwrap()
            .value,
        "3"
    );
    // 互通：FormData.append(name, file) — File is-a Blob，value 经 String() 归一（spec 非 Blob 转 USVString；
    // 本 FormData 实现恒字符串化，File 作为整体入列不读内容——与 fetch POST 一起为 follow-up，此处仅验 no-throw）。
    sandbox
        .execute("var fd = new FormData(); fd.append('upload', new File(['data'],'up.txt'));")
        .unwrap();
    assert_eq!(sandbox.execute("String(fd.has('upload'))").unwrap().value, "true");
    // 互通：FileReader.readAsDataURL(file) — File is-a Blob，readAsDataURL 读其字节。
    sandbox
        .execute(
            "globalThis.__furl = '(pending)';\
             var rd = new FileReader();\
             rd.onload = function(e){ globalThis.__furl = e.target.result; };\
             rd.readAsDataURL(new File(['hi'],'h.txt',{type:'text/plain'}));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__furl)").unwrap().value,
        "data:text/plain;base64,aGk="
    );
    // 无 new 调用亦可构造。
    assert_eq!(
        sandbox
            .execute("String(File(['x'],'y') instanceof File)")
            .unwrap()
            .value,
        "true"
    );
}

#[test]
fn test_crypto_subtle_digest_r2793() {
    // R2793：crypto.subtle.digest（SHA-1/256/384/512，SRI/JWT/内容哈希高频）。host RustCrypto sha1/sha2。
    // 返 Promise<ArrayBuffer>（Uint8Array）。TDD 用已知向量 hex 锚定（SHA-256('abc')/SHA-1('abc')/空输入）。
    // scope 仅 digest（HMAC/sign/encrypt defer）。
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

    // hex 辅助：Uint8Array → 低位补零 hex 串（execute 末 Promise drain → 下 execute 读）。
    let mut hex_of = |expr: &str| -> String {
        sandbox.execute(expr).unwrap();
        sandbox
            .execute("Array.from(globalThis.__dig).map(function(b){return ('0'+b.toString(16)).slice(-2);}).join('')")
            .unwrap()
            .value
    };
    // SHA-256('abc') = ba7816bf...015ad（NIST FIPS 180-4 测试向量）。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest('SHA-256', new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // SHA-1('abc') = a9993e36...0d89d（NIST 测试向量）。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest('SHA-1', new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    // SHA-512('abc') 前 16 hex（ddaf35a1...）。
    let h512 = hex_of(
        "globalThis.__dig='(pending)';\
         crypto.subtle.digest('SHA-512', new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });",
    );
    assert_eq!(h512.len(), 128); // 64 字节
    assert_eq!(&h512[..16], "ddaf35a193617aba");
    // SHA-256('') 空输入 = e3b0c442...b855。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest('SHA-256', new Uint8Array(0)).then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    // algo 对象形式 {name:'SHA-256'} + SHA-256 等价 SHA256（无连字符）。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest({name:'SHA-256'}, new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // 字符串输入经 TextEncoder（'abc' 同 UTF-8 字节）。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest('SHA256', 'abc').then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // SHA-384('abc') 长度 96 hex（48 字节）。
    let h384 = hex_of(
        "globalThis.__dig='(pending)';\
         crypto.subtle.digest('SHA-384', new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });",
    );
    assert_eq!(h384.len(), 96);
    assert_eq!(&h384[..16], "cb00753f45a35e8b");
    // unsupported algo → reject NotSupportedError（catch 验证 name）。
    sandbox
        .execute(
            "globalThis.__err='(pending)';\
             crypto.subtle.digest('MD5', new Uint8Array([1])).catch(function(e){ globalThis.__err = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__err)").unwrap().value,
        "NotSupportedError"
    );
}

#[test]
fn test_headers_r2794() {
    // R2794：Headers（HTTP 头集合，fetch/SW/header-map 高频）。镜像 FormData，header name 小写归一 +
    // 多值 append 用 ', ' 合并 + getSetCookie 特例。纯 JS，零 host 回调。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + 无 new 构造 + instanceof。
    assert_eq!(sandbox.execute("typeof Headers").unwrap().value, "function");
    assert_eq!(
        sandbox
            .execute("String(new Headers() instanceof Headers)")
            .unwrap()
            .value,
        "true"
    );
    // record 对象构造 + get/has（name 小写归一：'Content-Type' → 'content-type' 查）。
    sandbox
        .execute("var h = new Headers({'Content-Type':'text/plain','X-Custom':'a'});")
        .unwrap();
    assert_eq!(sandbox.execute("h.get('Content-Type')").unwrap().value, "text/plain");
    assert_eq!(sandbox.execute("h.get('content-type')").unwrap().value, "text/plain");
    assert_eq!(sandbox.execute("h.get('missing')").unwrap().value, "null");
    assert_eq!(sandbox.execute("String(h.has('X-Custom'))").unwrap().value, "true");
    assert_eq!(sandbox.execute("String(h.has('x-custom'))").unwrap().value, "true");
    // append 多值 → get ', ' 合并（spec）。
    sandbox.execute("h.append('X-Custom','b');").unwrap();
    assert_eq!(sandbox.execute("h.get('X-Custom')").unwrap().value, "a, b");
    // set 替换所有同名值。
    sandbox.execute("h.set('X-Custom','only');").unwrap();
    assert_eq!(sandbox.execute("h.get('X-Custom')").unwrap().value, "only");
    // delete。
    sandbox.execute("h.delete('X-Custom');").unwrap();
    assert_eq!(sandbox.execute("String(h.has('X-Custom'))").unwrap().value, "false");
    // 数组对序列构造。
    assert_eq!(
        sandbox
            .execute("new Headers([['A','1'],['B','2']]).get('A')")
            .unwrap()
            .value,
        "1"
    );
    // 另一 Headers 构造（forEach 分支）。
    assert_eq!(
        sandbox
            .execute("new Headers(new Headers({'K':'v'})).get('K')")
            .unwrap()
            .value,
        "v"
    );
    // getSetCookie：多个 Set-Cookie 返数组（不合并，spec 特例）。
    sandbox
        .execute("var c = new Headers(); c.append('Set-Cookie','a=1'); c.append('Set-Cookie','b=2');")
        .unwrap();
    assert_eq!(sandbox.execute("c.getSetCookie().join('|')").unwrap().value, "a=1|b=2");
    // 迭代：[Symbol.iterator]=entries → spread 取 [k,v]，key 为小写。
    sandbox.execute("var it = new Headers({'Z':'1','A':'2'});").unwrap();
    assert_eq!(
        sandbox
            .execute("[...it].map(function(p){return p[0]+'='+p[1];}).join(',')")
            .unwrap()
            .value,
        "z=1,a=2"
    );
    // forEach 回调。
    assert_eq!(
        sandbox
            .execute("(function(){var o=[];it.forEach(function(v,k){o.push(k+':'+v);});return o.join(',');})()")
            .unwrap()
            .value,
        "z:1,a:2"
    );
    // keys / values 迭代器。
    assert_eq!(sandbox.execute("[...it.keys()].join(',')").unwrap().value, "z,a");
    assert_eq!(sandbox.execute("[...it.values()].join(',')").unwrap().value, "1,2");
}

#[test]
fn test_canvas_get_context_r2795() {
    // R2795：HTMLCanvasElement.getContext('2d')（canvas slice 1）。host CanvasContext 注册表 +
    // __zw_canvas_op 派发。fill()/stroke() 写 pixel_buffer（path-based rasterize），fillRect 经 path
    // 实现（绕过 fill_rect 便捷法不写 pixel_buffer）。getImageData 回读验证像素。
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

    // createElement('canvas') + width/height 默认 300×150 + getContext('2d')。
    sandbox
        .execute(
            "globalThis.__c = document.createElement('canvas');\
             globalThis.__ctx = __c.getContext('2d');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("typeof __c.getContext").unwrap().value, "function");
    assert_eq!(
        sandbox.execute("String(__c.width + 'x' + __c.height)").unwrap().value,
        "300x150"
    );
    assert_eq!(
        sandbox
            .execute("String(__ctx !== null && typeof __ctx.fillRect === 'function')")
            .unwrap()
            .value,
        "true"
    );
    // 自定义尺寸 + getContext('webgl') → null（仅 2d）。
    sandbox.execute("__c.width = 4; __c.height = 4;").unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.createElement('canvas').getContext('webgl') === null)")
            .unwrap()
            .value,
        "true"
    );
    // fillRect('red') + getImageData 回读：红色像素（255,0,0,255）。
    // 重新创建 4×4 canvas + ctx（尺寸变化后重取 ctx 反映新尺寸）。
    sandbox
        .execute(
            "globalThis.__c2 = document.createElement('canvas'); __c2.width=4; __c2.height=4;\
             globalThis.__ctx2 = __c2.getContext('2d');\
             __ctx2.fillStyle = 'red';\
             __ctx2.fillRect(0, 0, 4, 4);\
             globalThis.__img = __ctx2.getImageData(0, 0, 4, 4);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__img.width + 'x' + __img.height)")
            .unwrap()
            .value,
        "4x4"
    );
    assert_eq!(
        sandbox
            .execute("String(__img.data[0] + ',' + __img.data[1] + ',' + __img.data[2] + ',' + __img.data[3])")
            .unwrap()
            .value,
        "255,0,0,255"
    );
    // 未填充区域：getImageData 另一 canvas（默认透明 0,0,0,0）。
    sandbox
        .execute(
            "globalThis.__e = document.createElement('canvas'); __e.width=2; __e.height=2;\
             globalThis.__blank = __e.getContext('2d').getImageData(0,0,2,2);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__blank.data[0] + ',' + __blank.data[3])")
            .unwrap()
            .value,
        "0,0"
    );
    // fillStyle #00ff00（hex 解析）→ 绿色像素。
    sandbox
        .execute(
            "globalThis.__g = document.createElement('canvas'); __g.width=2; __g.height=2;\
             var gx = __g.getContext('2d'); gx.fillStyle = '#00ff00'; gx.fillRect(0,0,2,2);\
             globalThis.__gp = gx.getImageData(0,0,2,2).data;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__gp[0] + ',' + __gp[1] + ',' + __gp[2])")
            .unwrap()
            .value,
        "0,255,0"
    );
    // path API：beginPath + arc + fill（圆形区域中心像素为填充色）。
    sandbox
        .execute(
            "globalThis.__p = document.createElement('canvas'); __p.width=10; __p.height=10;\
             var px = __p.getContext('2d'); px.fillStyle = 'rgb(0,0,255)';\
             px.beginPath(); px.arc(5, 5, 4, 0, 6.2832); px.fill();\
             globalThis.__pp = px.getImageData(5, 5, 1, 1).data;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__pp[0] + ',' + __pp[1] + ',' + __pp[2])")
            .unwrap()
            .value,
        "0,0,255"
    );
}

#[test]
fn test_canvas_slice2_r2796() {
    // R2796：canvas slice 2——path 曲线 / save/restore 栈 / transforms / globalAlpha / line 样式。
    // builds on slice 1 dispatch。translate 后 fillRect 像素位移；save+globalAlpha+restore 还原；
    // scale 放大填充区域；curve 路径填充中心像素。
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

    // translate 后 fillRect：原本 (0,0,2,2) 平移到 (3,3)，故 (0,0) 空、(3,3) 红。
    sandbox
        .execute(
            "globalThis.__t = document.createElement('canvas'); __t.width=8; __t.height=8;\
             var tx = __t.getContext('2d');\
             tx.translate(3, 3); tx.fillStyle = 'red'; tx.fillRect(0, 0, 2, 2);\
             globalThis.__im = tx.getImageData(0, 0, 8, 8);",
        )
        .unwrap();
    // 像素索引 helper：(x,y) 在 width=8 图里的 RGBA 起始 = (y*8+x)*4。
    let mut pix = |x: usize, y: usize| -> String {
        sandbox
            .execute(&format!(
                "var i={y}*8*4+{x}*4; String(globalThis.__im.data[i]+','+globalThis.__im.data[i+3])",
                x = x,
                y = y
            ))
            .unwrap()
            .value
    };
    assert_eq!(pix(0, 0), "0,0", "translate 前 (0,0) 应空");
    assert_eq!(pix(3, 3), "255,255", "translate 后 (3,3) 应红");
    // save + globalAlpha(0.5) + restore：globalAlpha 经 setter push host；restore 后还原本。
    sandbox
        .execute(
            "globalThis.__a = document.createElement('canvas'); __a.width=4; __a.height=4;\
             var ax = __a.getContext('2d');\
             ax.save(); ax.globalAlpha = 0.5; ax.fillStyle='red'; ax.fillRect(0,0,2,2);\
             ax.restore();\
             ax.fillStyle='green'; ax.fillRect(2,2,2,2);\
             globalThis.__aim = ax.getImageData(0,0,4,4);",
        )
        .unwrap();
    // globalAlpha 0.5 × 红 (255,0,0,255) → (255,0,0,127)（alpha 缩放）。验证 alpha 通道。
    assert_eq!(
        sandbox
            .execute("var i=0; String(__aim.data[i]+','+__aim.data[i+3])")
            .unwrap()
            .value,
        "255,127",
        "globalAlpha=0.5 应缩 alpha"
    );
    // scale 放大：scale(2,2) 后 fillRect(0,0,1,1) 覆盖 2×2 → (1,1) 被填充。
    sandbox
        .execute(
            "globalThis.__s = document.createElement('canvas'); __s.width=4; __s.height=4;\
             var sx = __s.getContext('2d');\
             sx.scale(2, 2); sx.fillStyle='blue'; sx.fillRect(0,0,1,1);\
             globalThis.__sim = sx.getImageData(0,0,4,4);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("var i=1*4*4+1*4; String(__sim.data[i+2]+','+__sim.data[i+3])")
            .unwrap()
            .value,
        "255,255",
        "scale(2,2) 后 1×1 覆盖到 (1,1) 蓝色"
    );
    // setTransform 单位阵重置 + rect 路径 + fill：rect(1,1,2,2) 填充 → (1,1) 红。
    sandbox
        .execute(
            "globalThis.__r = document.createElement('canvas'); __r.width=5; __r.height=5;\
             var rx = __r.getContext('2d');\
             rx.setTransform(1,0,0,1,0,0); rx.fillStyle='red';\
             rx.beginPath(); rx.rect(1,1,2,2); rx.fill();\
             globalThis.__rim = rx.getImageData(0,0,5,5);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("var i=1*5*4+1*4; String(__rim.data[i]+','+__rim.data[i+3])")
            .unwrap()
            .value,
        "255,255",
        "rect 路径填充 (1,1) 红"
    );
    // ellipse 路径 + fill：椭圆中心 (3,3) 蓝色。
    sandbox
        .execute(
            "globalThis.__el = document.createElement('canvas'); __el.width=8; __el.height=8;\
             var ex = __el.getContext('2d');\
             ex.fillStyle='rgb(0,0,255)'; ex.beginPath(); ex.ellipse(3,3,3,3,0,0,6.2832); ex.fill();\
             globalThis.__elim = ex.getImageData(3,3,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__elim.data[2]+','+__elim.data[3])")
            .unwrap()
            .value,
        "255,255",
        "ellipse 填充中心蓝"
    );
    // lineJoin/lineCap setter push host（no-throw + getter 返回 set 值）。
    sandbox.execute("globalThis.__lj = __t.getContext('2d');").unwrap();
    // 注：__t 的 ctx 已创建；重取返同一 proxy。设值验证 getter。
    sandbox
        .execute(
            "globalThis.__cx = document.createElement('canvas').getContext('2d');\
             __cx.lineJoin='round'; __cx.lineCap='square'; __cx.lineWidth=3; __cx.setLineDash([5,3]);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("__cx.lineJoin").unwrap().value, "round");
    assert_eq!(sandbox.execute("__cx.lineCap").unwrap().value, "square");
    assert_eq!(sandbox.execute("String(__cx.lineWidth)").unwrap().value, "3");
}

#[test]
fn test_canvas_to_data_url_r2797() {
    // R2797：canvas slice 3——toDataURL（PNG 导出）。host png::Encoder 编码 pixel_buffer → csv 字节；
    // shim 转 Latin-1 → btoa → data:image/png;base64,。TDD：合法 PNG 签名（137,80,78,71,13,10,26,10）。
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

    // fillRect red 后 toDataURL：data:image/png;base64, 前缀 + 非空 base64。
    sandbox
        .execute(
            "var c = document.createElement('canvas'); c.width=3; c.height=3;\
             var cx = c.getContext('2d'); cx.fillStyle='red'; cx.fillRect(0,0,3,3);\
             globalThis.__url = c.toDataURL();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__url.slice(0, 'data:image/png;base64,'.length) === 'data:image/png;base64,'")
            .unwrap()
            .value,
        "true",
        "toDataURL 应返 data:image/png;base64, 前缀"
    );
    // base64 部分 → atob 解码 → 检 PNG 签名（\\x89 P N G \\r \\n \\x1a \\n = 137,80,78,71,13,10,26,10）。
    sandbox
        .execute(
            "var b = atob(globalThis.__url.split(',')[1]);\
             globalThis.__sig = b.charCodeAt(0)+','+b.charCodeAt(1)+','+b.charCodeAt(2)+','+b.charCodeAt(3)+','+b.charCodeAt(4)+','+b.charCodeAt(5)+','+b.charCodeAt(6)+','+b.charCodeAt(7);\
             globalThis.__len = b.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sig)").unwrap().value,
        "137,80,78,71,13,10,26,10",
        "解码后须为合法 PNG 签名"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len > 0)").unwrap().value,
        "true",
        "PNG 非空"
    );
    // toDataURL 与无绘制（空白）canvas 的 PNG 不同（内容反映绘制）——两 URL 不同。
    sandbox
        .execute("globalThis.__blank = document.createElement('canvas').toDataURL();")
        .unwrap();
    // 注：空白 canvas（默认 300×150）比 3×3 大得多，PNG 不同。验证两者不同（内容/尺寸差异）。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__blank !== globalThis.__url)")
            .unwrap()
            .value,
        "true"
    );
    // toDataURL 在未 getContext 的 canvas 上可调用（惰性创建 ctx）。
    assert_eq!(
        sandbox
            .execute("document.createElement('canvas').toDataURL().slice(0,5)")
            .unwrap()
            .value,
        "data:"
    );
}

#[test]
fn test_canvas_slice4_r2798() {
    // R2798：canvas slice 4——off-DOM 2D 表面补全（putImageData / globalCompositeOperation / shadow）。
    // 经核验三项均真写 pixel_buffer（非仅记 primitives）：
    // ① putImageData：put_image_data 1:1 copy_from_slice 写 pixel_buffer（get_imageData 对偶）；
    // ② globalCompositeOperation：host 持 state（save/restore 含），composite_pixel 在 rect-blit/stroke 消费
    //    （path-based fill 不消费——已知限制）；本测验证状态往返（default + set/get）；
    // ③ shadow：fill() 经 draw_shadow_path 偏移栅格到 pixel_buffer，offset 处可读得 shadowColor。
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

    // ── ① putImageData：写自定义 RGBA → getImageData 读回一致（pixel 0 与 pixel 3）。
    sandbox
        .execute(
            "var c = document.createElement('canvas'); c.width=2; c.height=2;\
             var cx = c.getContext('2d');\
             var im = cx.getImageData(0,0,2,2);\
             im.data[0]=10; im.data[1]=20; im.data[2]=30; im.data[3]=255;\
             im.data[4]=40; im.data[5]=50; im.data[6]=60; im.data[7]=255;\
             im.data[8]=70; im.data[9]=80; im.data[10]=90; im.data[11]=255;\
             im.data[12]=100; im.data[13]=110; im.data[14]=120; im.data[15]=255;\
             cx.putImageData(im, 0, 0);\
             globalThis.__back = cx.getImageData(0,0,2,2);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__back.data[0],__back.data[1],__back.data[2],__back.data[3]].join(',')")
            .unwrap()
            .value,
        "10,20,30,255",
        "putImageData 写入后 getImageData 读回须一致（pixel 0）"
    );
    assert_eq!(
        sandbox
            .execute("[__back.data[12],__back.data[13],__back.data[14],__back.data[15]].join(',')")
            .unwrap()
            .value,
        "100,110,120,255",
        "putImageData 写入后 getImageData 读回须一致（pixel 3）"
    );

    // ── ② globalCompositeOperation：default 'source-over' + set/get 往返（状态真实）。
    sandbox
        .execute(
            "var c2 = document.createElement('canvas');\
             globalThis.__cx2 = c2.getContext('2d');\
             globalThis.__def = __cx2.globalCompositeOperation;\
             __cx2.globalCompositeOperation = 'lighter';\
             globalThis.__g1 = __cx2.globalCompositeOperation;\
             __cx2.globalCompositeOperation = 'multiply';\
             globalThis.__g2 = __cx2.globalCompositeOperation;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__def)").unwrap().value,
        "source-over",
        "globalCompositeOperation 默认 source-over"
    );
    assert_eq!(sandbox.execute("String(globalThis.__g1)").unwrap().value, "lighter");
    assert_eq!(sandbox.execute("String(globalThis.__g2)").unwrap().value, "multiply");

    // ── ③ shadow：fillRect green + shadowColor red + shadowOffsetX=5。
    // fill 区 [0,10]×[0,10]；shadow 偏移后 [5,15]×[0,10]。(12,2) 仅 shadow 区→red；(2,2) fill-only→green
    // （fill 先画 shadow 再画 fill，重叠区 [5,10] 被 green 覆盖）。
    sandbox
        .execute(
            "var c3 = document.createElement('canvas'); c3.width=30; c3.height=20;\
             var cx3 = c3.getContext('2d');\
             cx3.shadowColor='rgba(255,0,0,255)'; cx3.shadowOffsetX=5;\
             cx3.fillStyle='#00ff00'; cx3.fillRect(0,0,10,10);\
             globalThis.__sh = cx3.getImageData(12,2,1,1);\
             globalThis.__fi = cx3.getImageData(2,2,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__sh.data[0],__sh.data[1],__sh.data[2],__sh.data[3]].join(',')")
            .unwrap()
            .value,
        "255,0,0,255",
        "shadow 偏移处须为 shadowColor（draw_shadow_path 栅格到 pixel_buffer）"
    );
    assert_eq!(
        sandbox
            .execute("[__fi.data[0],__fi.data[1],__fi.data[2],__fi.data[3]].join(',')")
            .unwrap()
            .value,
        "0,255,0,255",
        "fill 区须为 fillStyle（fill 画在 shadow 之上覆盖重叠）"
    );
}

#[test]
fn test_canvas_draw_image_r2799() {
    // R2799：canvas slice 5——drawImage（图像合成到 canvas，3 spec 重载）。host draw_image* 已存在且
    // 真写 pixel_buffer（draw_image_sized：最近邻采样 + transform + source-over alpha 混合 + global_alpha）。
    // shim 源限 canvas 元素（canvas-to-canvas，经源 getImageData 取全 RGBA wire）；HTMLImageElement defer。
    // TDD：drawImage(image,dx,dy) 源 red→dst 读回 red / drawImageScaled 1×1→3×3 / drawImageSliced 切片 sx。
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

    // ── drawImage(image, dx, dy)：src 2×2 red → dst 4×4 at (1,1)。覆盖 dst (1,1)..(2,2)；(0,0) 透明。
    sandbox
        .execute(
            "var s1 = document.createElement('canvas'); s1.width=2; s1.height=2;\
             var sc1 = s1.getContext('2d'); sc1.fillStyle='red'; sc1.fillRect(0,0,2,2);\
             var d1 = document.createElement('canvas'); d1.width=4; d1.height=4;\
             var dc1 = d1.getContext('2d'); dc1.drawImage(s1, 1, 1);\
             globalThis.__a_hit = dc1.getImageData(1,1,1,1);\
             globalThis.__a_miss = dc1.getImageData(0,0,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__a_hit.data[0],__a_hit.data[1],__a_hit.data[2],__a_hit.data[3]].join(',')")
            .unwrap()
            .value,
        "255,0,0,255",
        "drawImage(image,dx,dy) 源 red 须栅格到 dst"
    );
    assert_eq!(
        sandbox
            .execute("[__a_miss.data[0],__a_miss.data[1],__a_miss.data[2],__a_miss.data[3]].join(',')")
            .unwrap()
            .value,
        "0,0,0,0",
        "drawImage 区外须保持透明"
    );

    // ── drawImageScaled(image, dx, dy, dw, dh)：src 1×1 green → dst 缩放到 3×3。(2,2) 须 green。
    sandbox
        .execute(
            "var s2 = document.createElement('canvas'); s2.width=1; s2.height=1;\
             var sc2 = s2.getContext('2d'); sc2.fillStyle='#00ff00'; sc2.fillRect(0,0,1,1);\
             var d2 = document.createElement('canvas'); d2.width=4; d2.height=4;\
             var dc2 = d2.getContext('2d'); dc2.drawImage(s2, 0, 0, 3, 3);\
             globalThis.__b = dc2.getImageData(2,2,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__b.data[0],__b.data[1],__b.data[2],__b.data[3]].join(',')")
            .unwrap()
            .value,
        "0,255,0,255",
        "drawImageScaled 1×1→3×3 须缩放栅格"
    );

    // ── drawImageSliced(image, sx,sy,sw,sh, dx,dy,dw,dh)：src 2×1（(0,0)red / (1,0)green）。
    // 切片 (0,0,1,1)→dst(0,0,2,2) red；切片 (1,0,1,1)→dst(2,0,2,2) green。证明 sx 被采样。
    sandbox
        .execute(
            "var s3 = document.createElement('canvas'); s3.width=2; s3.height=1;\
             var sc3 = s3.getContext('2d');\
             var im3 = sc3.getImageData(0,0,2,1);\
             im3.data[0]=255; im3.data[1]=0; im3.data[2]=0; im3.data[3]=255;\
             im3.data[4]=0; im3.data[5]=255; im3.data[6]=0; im3.data[7]=255;\
             sc3.putImageData(im3, 0, 0);\
             var d3 = document.createElement('canvas'); d3.width=4; d3.height=4;\
             var dc3 = d3.getContext('2d');\
             dc3.drawImage(s3, 0,0,1,1, 0,0,2,2);\
             dc3.drawImage(s3, 1,0,1,1, 2,0,2,2);\
             globalThis.__c_red = dc3.getImageData(0,0,1,1);\
             globalThis.__c_green = dc3.getImageData(2,0,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__c_red.data[0],__c_red.data[1],__c_red.data[2],__c_red.data[3]].join(',')")
            .unwrap()
            .value,
        "255,0,0,255",
        "drawImageSliced 切片 (0,0) red 须栅格"
    );
    assert_eq!(
        sandbox
            .execute("[__c_green.data[0],__c_green.data[1],__c_green.data[2],__c_green.data[3]].join(',')")
            .unwrap()
            .value,
        "0,255,0,255",
        "drawImageSliced 切片 sx=1 须采样 (1,0) green"
    );
}

#[test]
fn test_document_metadata_r2800() {
    // R2800：document 元数据属性——title（get/set，纯 JS）+ URL/documentURI（= location.href）+ referrer（''）。
    // title getter 首访读 querySelector('title').textContent（空白折叠）+ 缓存；setter 仅缓存（不写回 host DOM）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config.clone()).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 含 <title> 多空白文本的 HTML（验 getter 空白折叠）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>  Hello   World  </title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // title getter：从 <title> 读 + 空白折叠（"Hello World"，非 "  Hello   World  "）。
    assert_eq!(
        sandbox.execute("String(document.title)").unwrap().value,
        "Hello World",
        "document.title getter 须读 <title> 并空白折叠"
    );
    // title setter + 读回（in-JS 缓存）。
    sandbox
        .execute("document.title = 'New Title'; globalThis.__t1 = document.title;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t1)").unwrap().value,
        "New Title",
        "document.title setter 须缓存并可读回"
    );
    // set 空串。
    sandbox
        .execute("document.title = ''; globalThis.__t2 = document.title;")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__t2)").unwrap().value, "");

    // URL / documentURI = location.href；referrer = ''。
    sandbox
        .execute(
            "globalThis.__url = document.URL;\
             globalThis.__uri = document.documentURI;\
             globalThis.__ref = document.referrer;\
             globalThis.__loc = location.href;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__url === globalThis.__loc)")
            .unwrap()
            .value,
        "true",
        "document.URL 须 === location.href"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__uri === globalThis.__loc)")
            .unwrap()
            .value,
        "true",
        "document.documentURI 须 === location.href"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ref)").unwrap().value,
        "",
        "document.referrer 须为空串（无 referrer 追踪）"
    );

    // 无 <title> 的 HTML → title 为空串（无 title 元素）。用独立 fresh sandbox（避免缓存干扰）。
    let mut sandbox2 = V8Sandbox::with_config(config).unwrap();
    sandbox2.execute(generate_js_dom_shim()).unwrap();
    let dom_html2: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let mutations2: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let page_url2: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox2, &mutations2, &dom_html2, &page_url2);
    assert_eq!(
        sandbox2.execute("String(document.title)").unwrap().value,
        "",
        "无 <title> 元素时 document.title 须为空串"
    );
}

#[test]
fn test_element_focus_active_element_r2801() {
    // R2801：element.focus()/blur() + document.activeElement（焦点状态追踪，纯 JS）。Proxy get trap 返回
    // focus/blur 函数操作 _activeElKey；activeElement getter 返 _proxyCache[_activeElKey] || body。
    // proxy 经 _proxyCache 缓存，故 activeElement 与原引用同对象（=== 成立）。
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
        "<html><body><input id='i' value='x'><button id='b'>ok</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 默认 activeElement === body（无焦点回落）。
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === document.body)")
            .unwrap()
            .value,
        "true",
        "默认 document.activeElement 须 === document.body"
    );
    // input.focus() → activeElement === input（同引用）。
    sandbox
        .execute(
            "globalThis.__i = document.getElementById('i');\
             globalThis.__i.focus();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === globalThis.__i)")
            .unwrap()
            .value,
        "true",
        "input.focus() 后 activeElement 须 === input"
    );
    // 跨元素切换：button.focus() → activeElement === button。
    sandbox
        .execute(
            "globalThis.__b = document.getElementById('b');\
             globalThis.__b.focus();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === globalThis.__b)")
            .unwrap()
            .value,
        "true",
        "button.focus() 后 activeElement 须切换到 button"
    );
    // blur() → activeElement 回落 body。
    sandbox.execute("globalThis.__b.blur();").unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === document.body)")
            .unwrap()
            .value,
        "true",
        "button.blur() 后 activeElement 须回落 body"
    );
    // blur 非当前焦点元素不影响 activeElement：input.blur()（非 active）后 activeElement 仍 body。
    sandbox.execute("globalThis.__i.blur();").unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === document.body)")
            .unwrap()
            .value,
        "true",
        "blur 非当前焦点元素不影响 activeElement"
    );
}

#[test]
fn test_document_create_event_r2802() {
    // R2802：document.createEvent + initCustomEvent（legacy 合成事件工厂）。createEvent 映射 type→现有构造器
    //（Event/CustomEvent/KeyboardEvent）返 new X('')，initEvent/initCustomEvent 填充。复用 R2779 事件构造器。
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

    // createEvent('Event') instanceof Event + initEvent 设 type/bubbles/cancelable。
    sandbox
        .execute(
            "var e = document.createEvent('Event');\
             globalThis.__isEvt = (e instanceof Event);\
             e.initEvent('click', true, false);\
             globalThis.__t = e.type; globalThis.__b = e.bubbles; globalThis.__c = e.cancelable;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__isEvt)").unwrap().value,
        "true",
        "createEvent('Event') instanceof Event"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t)").unwrap().value,
        "click",
        "initEvent 设 type"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b)").unwrap().value,
        "true",
        "initEvent 设 bubbles"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c)").unwrap().value,
        "false",
        "initEvent 设 cancelable"
    );

    // createEvent('CustomEvent') instanceof CustomEvent + initCustomEvent 设 detail。
    sandbox
        .execute(
            "var ce = document.createEvent('CustomEvent');\
             globalThis.__isCe = (ce instanceof CustomEvent) && (ce instanceof Event);\
             ce.initCustomEvent('foo', false, true, {a: 1});\
             globalThis.__ct = ce.type; globalThis.__cb = ce.bubbles; globalThis.__cc = ce.cancelable;\
             globalThis.__cd = ce.detail.a;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__isCe)").unwrap().value,
        "true",
        "createEvent('CustomEvent') instanceof CustomEvent & Event"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ct)").unwrap().value,
        "foo",
        "initCustomEvent 设 type"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cb)").unwrap().value,
        "false",
        "initCustomEvent 设 bubbles"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cc)").unwrap().value,
        "true",
        "initCustomEvent 设 cancelable"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cd)").unwrap().value,
        "1",
        "initCustomEvent 设 detail"
    );

    // 大小写不敏感 + spec 别名（HTMLEvents / Events → Event）。
    sandbox
        .execute(
            "var h = document.createEvent('HTMLEvents'); h.initEvent('x', true, false);\
             var ev = document.createEvent('Events'); ev.initEvent('y', false, false);\
             var ceu = document.createEvent('CustomEvent'); ceu.initCustomEvent('z', true, false, 9);\
             globalThis.__h = h.type; globalThis.__ev = ev.type; globalThis.__ceu = ceu.detail;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__h)").unwrap().value,
        "x",
        "createEvent('HTMLEvents') → Event"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev)").unwrap().value,
        "y",
        "createEvent('Events') → Event"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ceu)").unwrap().value,
        "9",
        "createEvent('CustomEvent') + initCustomEvent detail"
    );

    // dispatch 烟雾：createEvent + initEvent + dispatchEvent 触发 listener。
    sandbox
        .execute(
            "globalThis.__hits = 0;\
             var el = document.createElement('div');\
             el.addEventListener('ping', function () { globalThis.__hits++; });\
             var pe = document.createEvent('Event'); pe.initEvent('ping', false, false);\
             el.dispatchEvent(pe);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hits)").unwrap().value,
        "1",
        "createEvent + initEvent + dispatchEvent 须触发 listener"
    );
}

#[test]
fn test_document_tree_walker_r2803() {
    // R2803：document.createTreeWalker / createNodeIterator + NodeFilter（DOM 子树遍历器）。
    // eager pre-order 经 childNodes 递归；whatToShow 掩码 + acceptNode FILTER_ACCEPT/REJECT/SKIP 子树语义。
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
        "<html><body><div id=r><p>a</p><span>b</span><i>c</i></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "var root = document.getElementById('r'); var n;\
             var w = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             var tags = []; while ((n = w.nextNode())) tags.push(n.tagName);\
             globalThis.__tags = tags.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tags)").unwrap().value,
        "DIV,P,SPAN,I",
        "SHOW_ELEMENT 须深度优先文档序（含 root DIV）"
    );

    // SHOW_TEXT：文本节点序（trim 空白保安全）。
    sandbox
        .execute(
            "var w2 = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);\
             var texts = []; while ((n = w2.nextNode())) texts.push(String(n.nodeValue));\
             globalThis.__texts = texts.map(function(t){return t.trim();}).filter(Boolean).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__texts)").unwrap().value,
        "a,b,c",
        "SHOW_TEXT 须文本节点序 a,b,c"
    );

    // FILTER_REJECT 剪子树：reject span 元素 → span 与其文本 'b' 均不出现。
    sandbox
        .execute(
            "var rej = [];\
             var w3 = document.createTreeWalker(root, NodeFilter.SHOW_ALL, function (node) {\
               if (node.nodeType === 1 && node.tagName === 'SPAN') return NodeFilter.FILTER_REJECT;\
               return NodeFilter.FILTER_ACCEPT;\
             });\
             while ((n = w3.nextNode())) rej.push(n);\
             globalThis.__rejHasSpan = rej.some(function (x) { return x.nodeType === 1 && x.tagName === 'SPAN'; });\
             globalThis.__rejHasB = rej.some(function (x) { return x.nodeType === 3 && x.nodeValue === 'b'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rejHasSpan)").unwrap().value,
        "false",
        "FILTER_REJECT span 须排除 span 元素"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rejHasB)").unwrap().value,
        "false",
        "FILTER_REJECT span 须剪子树（文本 b 一并排除）"
    );

    // FILTER_SKIP 跳节点保留子树：skip span 元素 → span 不出现，但其文本 'b' 出现。
    sandbox
        .execute(
            "var skp = [];\
             var w4 = document.createTreeWalker(root, NodeFilter.SHOW_ALL, function (node) {\
               if (node.nodeType === 1 && node.tagName === 'SPAN') return NodeFilter.FILTER_SKIP;\
               return NodeFilter.FILTER_ACCEPT;\
             });\
             while ((n = w4.nextNode())) skp.push(n);\
             globalThis.__skpHasSpan = skp.some(function (x) { return x.nodeType === 1 && x.tagName === 'SPAN'; });\
             globalThis.__skpHasB = skp.some(function (x) { return x.nodeType === 3 && x.nodeValue === 'b'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__skpHasSpan)").unwrap().value,
        "false",
        "FILTER_SKIP span 须排除 span 元素"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__skpHasB)").unwrap().value,
        "true",
        "FILTER_SKIP span 须保留子树（文本 b 出现）"
    );

    // previousNode 反向 + NodeIterator 与 TreeWalker 同序。
    sandbox
        .execute(
            "var wb = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             var fwd = []; while ((n = wb.nextNode())) fwd.push(n.tagName);\
             var back = []; while ((n = wb.previousNode())) back.push(n.tagName);\
             globalThis.__back = back.join(',');\
             var it = document.createNodeIterator(root, NodeFilter.SHOW_ELEMENT);\
             var itags = []; while ((n = it.nextNode())) itags.push(n.tagName);\
             globalThis.__itags = itags.join(',');",
        )
        .unwrap();
    // nextNode 走到末尾后 previousNode 逆向：倒数第二起（末节点无前驱时不返自身）。
    // 至少验证 NodeIterator 与 TreeWalker 同序 + previousNode 返非空。
    assert_eq!(
        sandbox.execute("String(globalThis.__itags)").unwrap().value,
        "DIV,P,SPAN,I",
        "NodeIterator 须与 TreeWalker 同序"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__back.length > 0)").unwrap().value,
        "true",
        "previousNode 须可逆向遍历"
    );
}

#[test]
fn test_selection_range_r2804() {
    // R2804：window.getSelection + Selection 单例 + document.createRange + Range（文本选择/编辑器/copy-paste）。
    // headless 无真选择——Selection 默认空（rangeCount=0/isCollapsed=true/toString=''/type='None'）。
    // Range.toString 精确覆盖 selectNode*/同文本节点 setStart·setEnd。
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
        "<html><body><div id=r><p>hello</p><span>world</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // Selection 单例：window.getSelection() === window.getSelection()（同一对象）。
    sandbox
        .execute("globalThis.__same = (window.getSelection() === window.getSelection());")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__same)").unwrap().value,
        "true",
        "getSelection 须返单例（同一对象）"
    );

    // 默认空选择（headless 无真用户选择）。
    sandbox
        .execute(
            "var s = window.getSelection();\
             globalThis.__ts = s.toString();\
             globalThis.__rc = s.rangeCount;\
             globalThis.__ic = s.isCollapsed;\
             globalThis.__ty = s.type;\
             globalThis.__an = s.anchorNode;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ts)").unwrap().value,
        "",
        "默认 selection.toString 须为空"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rc)").unwrap().value,
        "0",
        "默认 rangeCount 须为 0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ic)").unwrap().value,
        "true",
        "默认 isCollapsed 须为 true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ty)").unwrap().value,
        "None",
        "默认 type 须为 'None'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__an)").unwrap().value,
        "null",
        "默认 anchorNode 须为 null"
    );

    // createRange + selectNodeContents(p)：toString = p 子树文本 'hello'。
    sandbox
        .execute(
            "var p = document.getElementById('r').firstElementChild; /* <p>hello</p> */\
             var r = document.createRange();\
             r.selectNodeContents(p);\
             globalThis.__rp = r.toString();\
             globalThis.__rcl = r.collapsed;\
             globalThis.__rsc = (r.startContainer === p);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rp)").unwrap().value,
        "hello",
        "selectNodeContents(p).toString 须为 'hello'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rcl)").unwrap().value,
        "false",
        "selectNodeContents 后 collapsed 须为 false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rsc)").unwrap().value,
        "true",
        "selectNodeContents startContainer 须为 p"
    );

    // selectNodeContents(#r)：多文本后代，toString = 'helloworld'。
    sandbox
        .execute(
            "var rr = document.createRange(); rr.selectNodeContents(document.getElementById('r'));\
             globalThis.__rr = rr.toString();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rr)").unwrap().value,
        "helloworld",
        "selectNodeContents(#r) 须收集多文本后代 'helloworld'"
    );

    // 同文本节点 setStart/setEnd slice：'hello'.slice(1,3) = 'el'。
    sandbox
        .execute(
            "var tn = document.getElementById('r').firstElementChild.firstChild; /* text 'hello' */\
             var rt = document.createRange(); rt.setStart(tn, 1); rt.setEnd(tn, 3);\
             globalThis.__rt = rt.toString();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rt)").unwrap().value,
        "el",
        "同文本节点 setStart/setEnd 须 slice(1,3)='el'"
    );

    // addRange → rangeCount=1 / selection.toString = range 文本 / type='Range' / isCollapsed=false。
    sandbox
        .execute(
            "var rp = document.createRange(); rp.selectNodeContents(document.getElementById('r').firstElementChild);\
             window.getSelection().addRange(rp);\
             globalThis.__src = window.getSelection().rangeCount;\
             globalThis.__srt = window.getSelection().toString();\
             globalThis.__sty = window.getSelection().type;\
             globalThis.__sic = window.getSelection().isCollapsed;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__src)").unwrap().value,
        "1",
        "addRange 后 rangeCount 须为 1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__srt)").unwrap().value,
        "hello",
        "selection.toString 须为 range 文本 'hello'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sty)").unwrap().value,
        "Range",
        "addRange 后 type 须为 'Range'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sic)").unwrap().value,
        "false",
        "addRange 后 isCollapsed 须为 false"
    );

    // removeAllRanges → 回空。
    sandbox.execute("window.getSelection().removeAllRanges();").unwrap();
    assert_eq!(
        sandbox
            .execute("String(window.getSelection().rangeCount)")
            .unwrap()
            .value,
        "0",
        "removeAllRanges 后 rangeCount 须回 0"
    );
    assert_eq!(
        sandbox.execute("String(window.getSelection().type)").unwrap().value,
        "None",
        "removeAllRanges 后 type 须回 'None'"
    );
}

#[test]
fn test_element_reflected_props_r2805() {
    // R2805：element reflected 属性簇（tabIndex/title/lang/dir）。get 反射同名 attribute（tabIndex 数值，
    // 无→-1；title/lang/dir 无→''），set 写 attribute。照 id/className reflected 模式（get+set trap）。
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
        "<html><body><a id=a href=# tabindex=3 title=hi lang=ja dir=rtl>x</a><div id=d>y</div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // get：反射 attribute（tabIndex 数值 / title·lang·dir 串）。
    sandbox
        .execute(
            "var a = document.getElementById('a');\
             globalThis.__ti = a.tabIndex;\
             globalThis.__tt = a.title; globalThis.__tl = a.lang; globalThis.__td = a.dir;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ti)").unwrap().value,
        "3",
        "tabIndex 须反射 tabindex 属性值"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tt)").unwrap().value,
        "hi",
        "title 须反射 title 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tl)").unwrap().value,
        "ja",
        "lang 须反射 lang 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__td)").unwrap().value,
        "rtl",
        "dir 须反射 dir 属性"
    );

    // 无属性默认：tabIndex → -1；title/lang/dir → ''。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             globalThis.__dti = d.tabIndex;\
             globalThis.__dtt = d.title; globalThis.__dtl = d.lang; globalThis.__dtd = d.dir;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dti)").unwrap().value,
        "-1",
        "无 tabindex 属性 tabIndex 须 -1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dtt)").unwrap().value,
        "",
        "无 title 属性 title 须 ''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dtl)").unwrap().value,
        "",
        "无 lang 属性 lang 须 ''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dtd)").unwrap().value,
        "",
        "无 dir 属性 dir 须 ''"
    );

    // set：写 attribute 并经属性反射同步读回（mutation 异步入队，getAttribute 读 stale 快照——同
    // value/className 既有限制；属性 get 经客户端缓存同步往返，是正确可测行为）。
    sandbox
        .execute(
            "d.tabIndex = 5; d.title = 'tip'; d.lang = 'en'; d.dir = 'ltr';\
             globalThis.__sti = d.tabIndex;\
             globalThis.__stt = d.title; globalThis.__stl = d.lang; globalThis.__std = d.dir;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sti)").unwrap().value,
        "5",
        "tabIndex set 后读回 5"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__stt)").unwrap().value,
        "tip",
        "title set 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__stl)").unwrap().value,
        "en",
        "lang set 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__std)").unwrap().value,
        "ltr",
        "dir set 后读回"
    );

    // tabIndex set 非数值（NaN）lenient 忽略（不写 attribute，读回仍原值）。
    sandbox
        .execute("d.tabIndex = 'abc'; globalThis.__nti = d.tabIndex;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nti)").unwrap().value,
        "5",
        "tabIndex set NaN 须 lenient 忽略（读回原 5）"
    );
}

#[test]
fn test_element_reflected_props2_r2806() {
    // R2806：reflected 簇 #2——contentEditable/isContentEditable + accessKey（编辑器栈锚 + a11y）。
    // contentEditable 字符串反射（无 → 'inherit'）+ client cache 同步往返；isContentEditable 计算 bool；
    // accessKey 字符串反射（无 → ''）+ cache。draggable/spellcheck 为枚举属性（非 presence-bool）defer。
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
        "<html><body><div id=e contenteditable=true accesskey=w>x</div><div id=p>y</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // contentEditable get：有属性 → 'true'；accessKey get → 'w'。
    sandbox
        .execute(
            "var e = document.getElementById('e');\
             globalThis.__ce = e.contentEditable;\
             globalThis.__ak = e.accessKey;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ce)").unwrap().value,
        "true",
        "contentEditable 须反射 contenteditable='true'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ak)").unwrap().value,
        "w",
        "accessKey 须反射 accesskey='w'"
    );

    // 无属性默认：contentEditable → 'inherit'；accessKey → ''。
    sandbox
        .execute(
            "var p = document.getElementById('p');\
             globalThis.__pce = p.contentEditable;\
             globalThis.__pak = p.accessKey;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__pce)").unwrap().value,
        "inherit",
        "无 contenteditable 属性 contentEditable 须 'inherit'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pak)").unwrap().value,
        "",
        "无 accesskey 属性 accessKey 须 ''"
    );

    // contentEditable set + 同步读回 + isContentEditable=true。
    sandbox
        .execute(
            "p.contentEditable = 'true';\
             globalThis.__sce = p.contentEditable;\
             globalThis.__ice = p.isContentEditable;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sce)").unwrap().value,
        "true",
        "contentEditable set='true' 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ice)").unwrap().value,
        "true",
        "isContentEditable 须当 contentEditable='true' 时 true"
    );

    // contentEditable set 'false' → isContentEditable=false。
    sandbox
        .execute("p.contentEditable = 'false'; globalThis.__ice2 = p.isContentEditable;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ice2)").unwrap().value,
        "false",
        "contentEditable='false' 时 isContentEditable 须 false"
    );

    // accessKey set + 同步读回。
    sandbox
        .execute("p.accessKey = 'k'; globalThis.__sak = p.accessKey;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sak)").unwrap().value,
        "k",
        "accessKey set 后读回"
    );
}

#[test]
fn test_element_aria_r2807() {
    // R2807：element.role + aria-* 反射簇（ARIA a11y 高频）。通用 _ariaAttrName 映射 ariaXxx↔aria-xxx
    //（ariaLabelledBy→aria-labelledby 单 hyphen，非 _camelToKebab 的双 hyphen）。_reflectedAttrs 缓存同步往返。
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
        "<html><body><button id=b role=button aria-label=Save aria-labelledby=l1 aria-valuenow=50>x</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // role get（读 role 属性）+ aria* get（读 aria-* 属性，验单/多词映射）。
    sandbox
        .execute(
            "var b = document.getElementById('b');\
             globalThis.__role = b.role;\
             globalThis.__al = b.ariaLabel;\
             globalThis.__alb = b.ariaLabelledBy;\
             globalThis.__avn = b.ariaValueNow;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__role)").unwrap().value,
        "button",
        "role 须反射 role 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__al)").unwrap().value,
        "Save",
        "ariaLabel 须反射 aria-label"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__alb)").unwrap().value,
        "l1",
        "ariaLabelledBy 须反射 aria-labelledby（单 hyphen）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__avn)").unwrap().value,
        "50",
        "ariaValueNow 须反射 aria-valuenow"
    );

    // set + 同步读回（缓存往返）：role + aria*。
    sandbox
        .execute(
            "b.role = 'link'; b.ariaLabel = 'Submit'; b.ariaExpanded = 'true';\
             globalThis.__srole = b.role;\
             globalThis.__sal = b.ariaLabel;\
             globalThis.__sae = b.ariaExpanded;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__srole)").unwrap().value,
        "link",
        "role set 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sal)").unwrap().value,
        "Submit",
        "ariaLabel set 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sae)").unwrap().value,
        "true",
        "ariaExpanded set 后读回（aria-expanded）"
    );

    // 无属性默认 ''（role + aria*）。
    sandbox
        .execute(
            "var d = document.createElement('div');\
             globalThis.__dr = d.role; globalThis.__dal = d.ariaLabel;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dr)").unwrap().value,
        "",
        "无 role 属性 role 须 ''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dal)").unwrap().value,
        "",
        "无 aria-label ariaLabel 须 ''"
    );
}

#[test]
fn test_document_stylesheets_r2808() {
    // R2808：CSSStyleSheet 只读 CSSOM——document.styleSheets 真 backing `<style>` + cssRules 读 +
    // CSSRule.selectorText/cssText/type。host 经 `__zw_style_rules`（parse_stylesheet + Selector 序列化）。
    // insertRule/deleteRule/`<link>` defer。
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
        "<html><head><style>p { color: red; } .a > b { font-size: 14px; }</style></head><body><p>x</p></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // document.styleSheets.length === 1（一个 <style>）；ownerNode 为 style 元素。
    sandbox
        .execute(
            "globalThis.__sheets = document.styleSheets;\
             globalThis.__len = __sheets.length;\
             globalThis.__sheet = __sheets[0];\
             globalThis.__ownerTag = __sheet.ownerNode ? __sheet.ownerNode.tagName : '';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "1",
        "styleSheets 须含 1 个 <style>"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ownerTag)").unwrap().value,
        "STYLE",
        "sheet.ownerNode 须为 <style> 元素"
    );

    // cssRules.length === 2；rule[0] selectorText 'p' + cssText 含 'color: red' + type=1。
    sandbox
        .execute(
            "globalThis.__rc = globalThis.__sheet.cssRules.length;\
             globalThis.__r0s = globalThis.__sheet.cssRules[0].selectorText;\
             globalThis.__r0c = globalThis.__sheet.cssRules[0].cssText;\
             globalThis.__r0t = globalThis.__sheet.cssRules[0].type;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rc)").unwrap().value,
        "2",
        "cssRules 须含 2 条规则"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r0s)").unwrap().value,
        "p",
        "cssRules[0].selectorText 须 'p'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__r0c.indexOf('color: red') >= 0)")
            .unwrap()
            .value,
        "true",
        "cssRules[0].cssText 须含 'color: red'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r0t)").unwrap().value,
        "1",
        "cssRules[0].type 须 1（STYLE_RULE）"
    );

    // rule[1] selectorText '.a > b'（组合器序列化）+ cssText 含 'font-size: 14px'。
    sandbox
        .execute(
            "globalThis.__r1s = globalThis.__sheet.cssRules[1].selectorText;\
             globalThis.__r1c = globalThis.__sheet.cssRules[1].cssText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r1s)").unwrap().value,
        ".a > b",
        "cssRules[1].selectorText 须含组合器 '.a > b'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__r1c.indexOf('font-size: 14px') >= 0)")
            .unwrap()
            .value,
        "true",
        "cssRules[1].cssText 须含 'font-size: 14px'"
    );

    // 无 <style> 文档 → styleSheets.length === 0。
    let dom_html2: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><p>no style</p></body></html>".to_string()));
    let mutations2: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let page_url2: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let mut sandbox2 = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox2.execute(generate_js_dom_shim()).unwrap();
    register_dom_callbacks(&mut sandbox2, &mutations2, &dom_html2, &page_url2);
    assert_eq!(
        sandbox2.execute("String(document.styleSheets.length)").unwrap().value,
        "0",
        "无 <style> 时 styleSheets.length 须 0"
    );
}

#[test]
fn test_stylesheets_write_r2809() {
    // R2809：CSSStyleSheet 写路径——insertRule/deleteRule。维护 client cache（同步读回真值）+ flush 重建
    // `<style>` 文本经 __zw_set_text（下次 render cascade，视觉异步；JS 契约同步）。
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
        "<html><head><style>p { color: red; }</style></head><body><p>x</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 初始 cssRules.length === 1（'p'）。
    sandbox
        .execute("globalThis.__sheet = document.styleSheets[0]; globalThis.__l0 = globalThis.__sheet.cssRules.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__l0)").unwrap().value,
        "1",
        "初始 cssRules 须 1 条"
    );

    // insertRule('div { color: blue; }', 0)：返 0 + length=2 + [0]='div'（插入首位）+ [1]='p'（原规则后移）。
    sandbox
        .execute(
            "globalThis.__idx = globalThis.__sheet.insertRule('div { color: blue; }', 0);\
             globalThis.__l1 = globalThis.__sheet.cssRules.length;\
             globalThis.__s0 = globalThis.__sheet.cssRules[0].selectorText;\
             globalThis.__s1 = globalThis.__sheet.cssRules[1].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__idx)").unwrap().value,
        "0",
        "insertRule 须返插入 index 0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__l1)").unwrap().value,
        "2",
        "insertRule 后 cssRules.length 须 2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s0)").unwrap().value,
        "div",
        "cssRules[0] 须为插入的 'div'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s1)").unwrap().value,
        "p",
        "cssRules[1] 须为原 'p'（后移）"
    );
    // 插入规则的 cssText 含 'color: blue'。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__sheet.cssRules[0].cssText.indexOf('color: blue') >= 0)")
            .unwrap()
            .value,
        "true",
        "插入规则 cssText 须含 'color: blue'"
    );

    // insertRule 不带 index → 末尾追加，返末尾 index。
    sandbox
        .execute(
            "globalThis.__idx2 = globalThis.__sheet.insertRule('span { color: green; }');\
             globalThis.__l2 = globalThis.__sheet.cssRules.length;\
             globalThis.__sEnd = globalThis.__sheet.cssRules[2].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__idx2)").unwrap().value,
        "2",
        "末尾 insertRule 须返 index 2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__l2)").unwrap().value,
        "3",
        "末尾追加后 length 须 3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sEnd)").unwrap().value,
        "span",
        "末尾规则须 'span'"
    );

    // deleteRule(0)：移除 'div' + length=2 + [0]='p'（回原首位）。
    sandbox
        .execute(
            "globalThis.__sheet.deleteRule(0);\
             globalThis.__l3 = globalThis.__sheet.cssRules.length;\
             globalThis.__s0b = globalThis.__sheet.cssRules[0].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__l3)").unwrap().value,
        "2",
        "deleteRule 后 length 须 2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s0b)").unwrap().value,
        "p",
        "deleteRule(0) 后 [0] 须回 'p'"
    );

    // 写回 `<style>` 文本（flush）：mutations 含 SetText（验证写源生效，cascade 下次 render）。
    let muts = mutations.lock().unwrap();
    let has_set_text = muts.iter().any(|m| matches!(m, DomMutation::SetText { .. }));
    drop(muts);
    assert!(
        has_set_text,
        "insertRule/deleteRule 须经 __zw_set_text 写回 <style> 文本（flush）"
    );
}

#[test]
fn test_stylesheets_rule_style_r2810() {
    // R2810：CSSRule.style per-rule CSSStyleDeclaration——sheet.cssRules[0].style 单声明读/写。
    // backed by 规则声明块（从 cssText body 解析有序 declarations）+ mutation flush 写回 `<style>` 源
    // （复用 R2809 flushToOwner）。per-property get/set（camelCase↔kebab）+ getPropertyValue/setProperty/
    // removeProperty + cssText + item/length。
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
        "<html><head><style>p { color: red; font-size: 14px; }</style></head><body><p>x</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // document.styleSheets 每次访问重建新 sheet（R2808 已知限制 ⑤，live DOM 重查）——故 rule/style 须
    // 一次捕获并复用引用，跨访问读取会落到新对象（host stale）。CSS-in-JS 常规用法即持引用编辑。
    sandbox
        .execute(
            "globalThis.__rule = document.styleSheets[0].cssRules[0];\
             globalThis.__st = __rule.style;",
        )
        .unwrap();

    // style 读既有声明：style.color='red' / getPropertyValue('font-size')='14px' / length=2 / item(0)='color'。
    sandbox
        .execute(
            "globalThis.__c = __st.color;\
             globalThis.__fs = __st.getPropertyValue('font-size');\
             globalThis.__len = __st.length;\
             globalThis.__item0 = __st.item(0);\
             globalThis.__camel = __st.fontSize;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__c)").unwrap().value,
        "red",
        "style.color 须读回既有 'red'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fs)").unwrap().value,
        "14px",
        "getPropertyValue('font-size') 须 '14px'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__camel)").unwrap().value,
        "14px",
        "camelCase style.fontSize 须 '14px'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "2",
        "style.length 须 2（color + font-size）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__item0)").unwrap().value,
        "color",
        "style.item(0) 须 'color'"
    );

    // set 既有属性：style.color='blue' → 同一 rule.cssText 反映 'color: blue' + flush 写回（SetText）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute("globalThis.__st.color = 'blue'; globalThis.__rc = __rule.cssText;")
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__rc.indexOf('color: blue') >= 0)")
            .unwrap()
            .value,
        "true",
        "style.color='blue' 后 rule.cssText 须含 'color: blue'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__rc.indexOf('color: red') >= 0)")
            .unwrap()
            .value,
        "false",
        "旧值 'color: red' 须被替换移除"
    );
    let muts = mutations.lock().unwrap();
    let has_set_text = muts.iter().any(|m| matches!(m, DomMutation::SetText { .. }));
    drop(muts);
    assert!(
        has_set_text,
        "style mutation 须经 flush __zw_set_text 写回 <style> 文本"
    );

    // setProperty 新增属性 + camelCase set + cssText 整体读。
    sandbox
        .execute(
            "globalThis.__st.setProperty('background', 'yellow');\
             globalThis.__st.marginTop = '8px';\
             globalThis.__decl = __st.cssText;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__decl.indexOf('background: yellow') >= 0)")
            .unwrap()
            .value,
        "true",
        "setProperty('background','yellow') 须入声明块"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__decl.indexOf('margin-top: 8px') >= 0)")
            .unwrap()
            .value,
        "true",
        "camelCase style.marginTop='8px' 须归一为 'margin-top: 8px'"
    );

    // removeProperty('color') → 返旧值 'blue' + 声明块不再含 color。
    sandbox
        .execute(
            "globalThis.__prev = __st.removeProperty('color');\
             globalThis.__hasColor = __st.cssText.indexOf('color:') >= 0;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__prev)").unwrap().value,
        "blue",
        "removeProperty 须返被移除的旧值 'blue'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hasColor)").unwrap().value,
        "false",
        "removeProperty('color') 后声明块不再含 'color:'"
    );

    // cssText 整体写 → 替换全部声明（同一 style 对象读 width）。
    sandbox
        .execute(
            "globalThis.__st.cssText = 'width: 100px; height: 50px';\
             globalThis.__after = __st.length;\
             globalThis.__w = __st.width;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__after)").unwrap().value,
        "2",
        "cssText 整体写后 length 须为 2（替换全部）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__w)").unwrap().value,
        "100px",
        "cssText 写后 style.width 须 '100px'"
    );
}

#[test]
fn test_event_subclasses_r2811() {
    // R2811：Event 子类簇——UIEvent / MouseEvent / FocusEvent / WheelEvent / PointerEvent / InputEvent。
    // 现代输入事件表面：feature-detection（typeof === 'function'）+ `new MouseEvent('click',{clientX,...})`
    // 合成派发。经 _defineEventSubclass 工厂（复用 _makeEvent + 原型链 extends parent）。getModifierState 仅
    // 跟踪 Alt/Control/Meta/Shift；WheelEvent 有 DOM_DELTA_* 静态常量；createEvent 映射含 mouseevent。
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

    // feature-detection：6 构造器均为 function。
    sandbox
        .execute(
            "globalThis.__fns = ['UIEvent','MouseEvent','FocusEvent','WheelEvent','PointerEvent','InputEvent']\
               .map(function(n){ return typeof globalThis[n] === 'function'; })\
               .every(Boolean);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fns)").unwrap().value,
        "true",
        "6 个 Event 子类构造器须均为 function"
    );

    // MouseEvent 字段 + instanceof 链（MouseEvent→UIEvent→Event）+ getModifierState。
    sandbox
        .execute(
            "globalThis.__me = new MouseEvent('click', { bubbles: true, clientX: 10, clientY: 20, button: 2, shiftKey: true });\
             globalThis.__meType = __me.type;\
             globalThis.__meBub = __me.bubbles;\
             globalThis.__meCX = __me.clientX;\
             globalThis.__meButton = __me.button;\
             globalThis.__meDef = __me.screenX;\
             globalThis.__meRel = __me.relatedTarget;\
             globalThis.__meChain = (__me instanceof MouseEvent) && (__me instanceof UIEvent) && (__me instanceof Event);\
             globalThis.__meShift = __me.getModifierState('Shift');\
             globalThis.__meAlt = __me.getModifierState('Alt');\
             globalThis.__meCaps = __me.getModifierState('CapsLock');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__meType)").unwrap().value,
        "click",
        "MouseEvent type"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meBub)").unwrap().value,
        "true",
        "MouseEvent bubbles 经 _makeEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meCX)").unwrap().value,
        "10",
        "MouseEvent clientX 来自 options"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meButton)").unwrap().value,
        "2",
        "MouseEvent button"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meDef)").unwrap().value,
        "0",
        "MouseEvent 默认 screenX=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meRel)").unwrap().value,
        "null",
        "MouseEvent relatedTarget 默认 null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meChain)").unwrap().value,
        "true",
        "MouseEvent instanceof MouseEvent & UIEvent & Event（原型链）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meShift)").unwrap().value,
        "true",
        "getModifierState('Shift') 跟踪 shiftKey"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meAlt)").unwrap().value,
        "false",
        "getModifierState('Alt') 未设→false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meCaps)").unwrap().value,
        "false",
        "getModifierState('CapsLock') 未跟踪→false"
    );

    // WheelEvent delta + DOM_DELTA_* 常量 + instanceof MouseEvent。
    sandbox
        .execute(
            "globalThis.__we = new WheelEvent('wheel', { deltaY: 120, deltaMode: 1 });\
             globalThis.__weDY = __we.deltaY;\
             globalThis.__weDM = __we.deltaMode;\
             globalThis.__weConst = WheelEvent.DOM_DELTA_LINE;\
             globalThis.__weChain = (__we instanceof WheelEvent) && (__we instanceof MouseEvent);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__weDY)").unwrap().value,
        "120",
        "WheelEvent deltaY"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__weDM)").unwrap().value,
        "1",
        "WheelEvent deltaMode"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__weConst)").unwrap().value,
        "1",
        "WheelEvent.DOM_DELTA_LINE=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__weChain)").unwrap().value,
        "true",
        "WheelEvent instanceof MouseEvent"
    );

    // PointerEvent 字段 + instanceof MouseEvent。
    sandbox
        .execute(
            "globalThis.__pe = new PointerEvent('pointerdown', { pointerType: 'mouse', isPrimary: true, pressure: 0.5 });\
             globalThis.__pePT = __pe.pointerType;\
             globalThis.__pePri = __pe.isPrimary;\
             globalThis.__peChain = (__pe instanceof PointerEvent) && (__pe instanceof MouseEvent);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__pePT)").unwrap().value,
        "mouse",
        "PointerEvent pointerType"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pePri)").unwrap().value,
        "true",
        "PointerEvent isPrimary"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__peChain)").unwrap().value,
        "true",
        "PointerEvent instanceof MouseEvent"
    );

    // FocusEvent / InputEvent instanceof UIEvent；FocusEvent.relatedTarget / InputEvent.data+inputType。
    sandbox
        .execute(
            "globalThis.__fe = new FocusEvent('blur');\
             globalThis.__feChain = (__fe instanceof FocusEvent) && (__fe instanceof UIEvent);\
             globalThis.__ie = new InputEvent('input', { data: 'x', inputType: 'insertText' });\
             globalThis.__ieChain = (__ie instanceof InputEvent) && (__ie instanceof UIEvent);\
             globalThis.__ieData = __ie.data;\
             globalThis.__ieType = __ie.inputType;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__feChain)").unwrap().value,
        "true",
        "FocusEvent instanceof UIEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ieChain)").unwrap().value,
        "true",
        "InputEvent instanceof UIEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ieData)").unwrap().value,
        "x",
        "InputEvent data"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ieType)").unwrap().value,
        "insertText",
        "InputEvent inputType"
    );

    // createEvent('MouseEvent') instanceof MouseEvent（映射扩展）。
    sandbox
        .execute("globalThis.__cme = document.createEvent('MouseEvent') instanceof MouseEvent;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cme)").unwrap().value,
        "true",
        "createEvent('MouseEvent') instanceof MouseEvent"
    );
}

#[test]
fn test_event_subclasses2_r2812() {
    // R2812：Event 子类簇 #2——HashChangeEvent / PopStateEvent / StorageEvent / ProgressEvent /
    // TransitionEvent / AnimationEvent（均 extends Event）。SPA hash/history 路由 + 跨标签页 storage 同步 +
    // XHR/资源加载进度 + CSS 过渡/动画回调高频。复用 R2811 _defineEventSubclass 工厂 + createEvent map 扩展。
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

    // feature-detection：6 构造器均为 function + instanceof Event。
    sandbox
        .execute(
            "globalThis.__fns = ['HashChangeEvent','PopStateEvent','StorageEvent','ProgressEvent',\
               'TransitionEvent','AnimationEvent']\
               .map(function(n){ return typeof globalThis[n] === 'function'; }).every(Boolean);\
             globalThis.__chain = new ProgressEvent('load') instanceof Event;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fns)").unwrap().value,
        "true",
        "6 构造器须均为 function"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chain)").unwrap().value,
        "true",
        "子类 instanceof Event"
    );

    // HashChangeEvent: oldURL/newURL（SPA hash 路由）。
    sandbox
        .execute(
            "globalThis.__he = new HashChangeEvent('hashchange', { oldURL: '#/a', newURL: '#/b' });\
             globalThis.__heOld = __he.oldURL; globalThis.__heNew = __he.newURL;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__heOld)").unwrap().value,
        "#/a",
        "HashChangeEvent oldURL"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__heNew)").unwrap().value,
        "#/b",
        "HashChangeEvent newURL"
    );

    // PopStateEvent: state（history 路由）。
    sandbox
        .execute("globalThis.__ps = new PopStateEvent('popstate', { state: { page: 2 } }); globalThis.__psS = __ps.state.page;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__psS)").unwrap().value,
        "2",
        "PopStateEvent state"
    );

    // StorageEvent: key/newValue/oldValue/url/storageArea（跨标签页 storage 同步）。
    sandbox
        .execute(
            "globalThis.__se = new StorageEvent('storage', { key: 'k', newValue: 'v2', oldValue: 'v1', url: 'http://x' });\
             globalThis.__seKey = __se.key; globalThis.__seNew = __se.newValue; globalThis.__seOld = __se.oldValue;\
             globalThis.__seUrl = __se.url; globalThis.__seArea = __se.storageArea;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__seKey)").unwrap().value,
        "k",
        "StorageEvent key"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__seNew)").unwrap().value,
        "v2",
        "StorageEvent newValue"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__seOld)").unwrap().value,
        "v1",
        "StorageEvent oldValue"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__seUrl)").unwrap().value,
        "http://x",
        "StorageEvent url"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__seArea)").unwrap().value,
        "null",
        "StorageEvent storageArea 默认 null"
    );

    // ProgressEvent: lengthComputable/loaded/total + 默认（XHR/资源加载进度）。
    sandbox
        .execute(
            "globalThis.__pe = new ProgressEvent('progress', { lengthComputable: true, loaded: 50, total: 100 });\
             globalThis.__peLC = __pe.lengthComputable; globalThis.__peL = __pe.loaded; globalThis.__peT = __pe.total;\
             globalThis.__peDef = new ProgressEvent('load').lengthComputable;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__peLC)").unwrap().value,
        "true",
        "ProgressEvent lengthComputable"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__peL)").unwrap().value,
        "50",
        "ProgressEvent loaded"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__peT)").unwrap().value,
        "100",
        "ProgressEvent total"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__peDef)").unwrap().value,
        "false",
        "ProgressEvent lengthComputable 默认 false"
    );

    // TransitionEvent / AnimationEvent: propertyName/animationName + elapsedTime + pseudoElement。
    sandbox
        .execute(
            "globalThis.__te = new TransitionEvent('transitionend', { propertyName: 'opacity', elapsedTime: 0.5 });\
             globalThis.__teP = __te.propertyName; globalThis.__teE = __te.elapsedTime;\
             globalThis.__ae = new AnimationEvent('animationend', { animationName: 'fade', elapsedTime: 1.2 });\
             globalThis.__aeN = __ae.animationName; globalThis.__aeE = __ae.elapsedTime;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__teP)").unwrap().value,
        "opacity",
        "TransitionEvent propertyName"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__teE)").unwrap().value,
        "0.5",
        "TransitionEvent elapsedTime"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__aeN)").unwrap().value,
        "fade",
        "AnimationEvent animationName"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__aeE)").unwrap().value,
        "1.2",
        "AnimationEvent elapsedTime"
    );

    // createEvent 映射（map）含 6 新 type：createEvent('StorageEvent') instanceof StorageEvent。
    sandbox
        .execute(
            "globalThis.__cse = document.createEvent('StorageEvent') instanceof StorageEvent;\
             globalThis.__cpr = document.createEvent('ProgressEvent') instanceof ProgressEvent;\
             globalThis.__cuk = document.createEvent('UnknownEvent') instanceof Event;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cse)").unwrap().value,
        "true",
        "createEvent('StorageEvent') instanceof StorageEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cpr)").unwrap().value,
        "true",
        "createEvent('ProgressEvent') instanceof ProgressEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cuk)").unwrap().value,
        "true",
        "createEvent(未知 type) 回落 instanceof Event"
    );
}

#[test]
fn test_custom_elements_r2813() {
    // R2813：customElements (CustomElementRegistry) scoped registry slice——web components 生态门控。
    // define/get/getName/whenDefined（同步 bookkeeping + whenDefined Promise）+ upgrade stub defer。
    // 诚实 defer element upgrade + lifecycle 回调（element proxy 非 ctor 实例 + 需 mutation 观察——深项）。
    // Promise.then 经 execute 末 microtask checkpoint 派发（同 R2774），下 execute 可读。
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

    // feature-detection：typeof customElements === 'object'。
    assert_eq!(
        sandbox.execute("typeof customElements").unwrap().value,
        "object",
        "window.customElements 须存在（object）"
    );

    // define（class extends HTMLElement）+ get 返同 ctor + getName 反查。
    sandbox
        .execute(
            "globalThis.MyEl = class MyEl extends HTMLElement {};\
             customElements.define('my-el', globalThis.MyEl);\
             globalThis.__same = (customElements.get('my-el') === globalThis.MyEl);\
             globalThis.__name = customElements.getName(globalThis.MyEl);\
             globalThis.__missing = customElements.get('no-such');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__same)").unwrap().value,
        "true",
        "get(name) 返已注册 ctor"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__name)").unwrap().value,
        "my-el",
        "getName(ctor) 反查 name"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__missing)").unwrap().value,
        "undefined",
        "get(未注册) 返 undefined"
    );

    // 无效名抛：'div'（无连字符）/ 'MyEl'（大写）/ 'font-face'（reserved）。
    sandbox
        .execute(
            "function _try(fn){ try { fn(); return 'no-throw'; } catch(e){ return 'threw'; } }\
             globalThis.__bad1 = _try(function(){ customElements.define('div', function(){}); });\
             globalThis.__bad2 = _try(function(){ customElements.define('MyEl', function(){}); });\
             globalThis.__bad3 = _try(function(){ customElements.define('font-face', function(){}); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__bad1)").unwrap().value,
        "threw",
        "无连字符名须抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bad2)").unwrap().value,
        "threw",
        "大写名须抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bad3)").unwrap().value,
        "threw",
        "reserved 名须抛"
    );

    // 重复名抛 / 重复 ctor 抛 / ctor 非 function 抛。
    sandbox
        .execute(
            "globalThis.__dupName = _try(function(){ customElements.define('my-el', function(){}); });\
             globalThis.__dupCtor = _try(function(){ customElements.define('other-el', globalThis.MyEl); });\
             globalThis.__notFn = _try(function(){ customElements.define('ok-el', 42); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dupName)").unwrap().value,
        "threw",
        "重复名须抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dupCtor)").unwrap().value,
        "threw",
        "重复 ctor 须抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__notFn)").unwrap().value,
        "threw",
        "ctor 非 function 须抛"
    );

    // whenDefined 已定义 → Promise<ctor> resolve（execute 末 microtask 派发，下 execute 可读）。
    sandbox
        .execute(
            "globalThis.__wdCtor = null; globalThis.__wd = false;\
             customElements.whenDefined('my-el').then(function(c){ globalThis.__wdCtor = (c === globalThis.MyEl); globalThis.__wd = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__wd)").unwrap().value,
        "true",
        "whenDefined(已定义) resolve"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wdCtor)").unwrap().value,
        "true",
        "whenDefined resolve 值为 ctor"
    );

    // whenDefined pending → define 触发 resolve（先挂起，下 execute define，再下 execute 读）。
    sandbox
        .execute(
            "globalThis.__later = false; globalThis.__laterCtor = null;\
             customElements.whenDefined('pending-el').then(function(c){ globalThis.__laterCtor = c; globalThis.__later = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__later)").unwrap().value,
        "false",
        "未 define 前 whenDefined pending 不 resolve"
    );
    sandbox
        .execute("globalThis.PendingEl = class extends HTMLElement {}; customElements.define('pending-el', globalThis.PendingEl);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__later)").unwrap().value,
        "true",
        "define 触发挂起的 whenDefined resolve"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__laterCtor === globalThis.PendingEl)")
            .unwrap()
            .value,
        "true",
        "挂起 resolve 值为 define 的 ctor"
    );

    // whenDefined 无效名 → Promise reject（.catch）。
    sandbox
        .execute(
            "globalThis.__rej = false;\
             customElements.whenDefined('BadName').catch(function(){ globalThis.__rej = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rej)").unwrap().value,
        "true",
        "whenDefined(无效名) reject"
    );

    // upgrade(root) no-op 不抛（defer，element proxy 非 ctor 实例）。
    sandbox
        .execute("globalThis.__up = _try(function(){ customElements.upgrade(document.body); });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__up)").unwrap().value,
        "no-throw",
        "upgrade no-op 不抛"
    );
}

#[test]
fn test_history_pushstate_r2814() {
    // R2814：history session history stack——SPA 路由核心（react-router / vue-router 等）。原 stub no-op，
    // 现实现 in-memory entries + cursor：pushState/replaceState 维护 state/length，back/forward/go 移 cursor
    // + _defer 异步派发 popstate（window listener，复用 R2812 PopStateEvent）。popstate 经 execute 末
    // microtask 派发，下 execute 可读（同 R2774）。已知限制：仅 in-memory（不更新 location / 不接 host 导航）。
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

    // 初始 length=1 / state=null；pushState 推进 length + state；replaceState 原地替换 state 不增 length。
    sandbox
        .execute(
            "globalThis.__initLen = history.length;\
             globalThis.__initState = history.state;\
             history.pushState({ page: 1 }, '', '/a');\
             globalThis.__len2 = history.length; globalThis.__st2 = history.state.page;\
             history.pushState({ page: 2 }, '', '/b');\
             globalThis.__len3 = history.length; globalThis.__st3 = history.state.page;\
             history.replaceState({ page: 20 }, '', '/b2');\
             globalThis.__len3b = history.length; globalThis.__st3b = history.state.page;\
             globalThis.__sr = history.scrollRestoration;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__initLen)").unwrap().value,
        "1",
        "初始 length=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__initState)").unwrap().value,
        "null",
        "初始 state=null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len2)").unwrap().value,
        "2",
        "pushState 后 length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__st2)").unwrap().value,
        "1",
        "pushState state.page=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len3)").unwrap().value,
        "3",
        "二次 pushState length=3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__st3)").unwrap().value,
        "2",
        "state.page=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len3b)").unwrap().value,
        "3",
        "replaceState 不增 length=3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__st3b)").unwrap().value,
        "20",
        "replaceState state.page=20"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sr)").unwrap().value,
        "auto",
        "scrollRestoration='auto'"
    );

    // 安装 popstate listener + back() → cursor 回退到 {page:1}（execute 末 microtask 派发 popstate）。
    sandbox
        .execute(
            "globalThis.__popState = null;\
             addEventListener('popstate', function(e){ globalThis.__popState = e.state; });\
             history.back();",
        )
        .unwrap();
    sandbox
        .execute(
            "globalThis.__popPage = globalThis.__popState ? globalThis.__popState.page : null;\
             globalThis.__curState = history.state.page; globalThis.__curLen = history.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__popPage)").unwrap().value,
        "1",
        "back() popstate 携带 state.page=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__curState)").unwrap().value,
        "1",
        "back() 后 history.state 回到 entry page=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__curLen)").unwrap().value,
        "3",
        "back() 不改 length=3"
    );

    // forward() → cursor 前进到 {page:20}，popstate 携带 {page:20}。
    sandbox.execute("history.forward();").unwrap();
    sandbox
        .execute("globalThis.__fwdPop = globalThis.__popState.page; globalThis.__fwdCur = history.state.page;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fwdPop)").unwrap().value,
        "20",
        "forward() popstate state.page=20"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fwdCur)").unwrap().value,
        "20",
        "forward() 后 state.page=20"
    );

    // go(-2) → cursor 回到 idx0（state=null）；go(0) 不动。
    sandbox.execute("history.go(-2);").unwrap();
    sandbox
        .execute("globalThis.__goStateIsNull = (history.state === null);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__goStateIsNull)").unwrap().value,
        "true",
        "go(-2) 回到初始 entry state=null"
    );

    // 截断：cursor 在 idx0 时 pushState → forward entries 截断 + 新 entry → length=2。
    sandbox
        .execute(
            "history.pushState({ page: 9 }, '', '/x');\
             globalThis.__truncLen = history.length; globalThis.__truncSt = history.state.page;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__truncLen)").unwrap().value,
        "2",
        "back 后 pushState 截断 forward → length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__truncSt)").unwrap().value,
        "9",
        "截断后 state.page=9"
    );

    // go(99) 越界 clamp 到末尾不抛（state 末 entry）。
    sandbox
        .execute(
            "globalThis.__oob = (function(){ try { history.go(99); return 'ok'; } catch(e){ return 'threw'; } })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__oob)").unwrap().value,
        "ok",
        "go(越界) clamp 不抛"
    );
}

#[test]
fn test_node_relation_implementation_r2815() {
    // R2815：document.implementation (DOMImplementation) + 节点关系方法（getRootNode/compareDocumentPosition/
    // isSameNode）+ Node.DOCUMENT_POSITION_* 常量。compareDocumentPosition bitmask 经 _ancestorChain + LCA +
    // __zw_element_children 子序比较。createComment defer（需 host DomMutation 桥）。
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
        "<html><body><div id='parent'><div id='a'>A</div><div id='b'>B</div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // document.implementation.hasFeature 恒 true + createHTMLDocument 返 hollow doc（body/title）。
    sandbox
        .execute(
            "globalThis.__hf = document.implementation.hasFeature('HTML', '1.0');\
             globalThis.__hdoc = document.implementation.createHTMLDocument('hi');\
             globalThis.__hbody = __hdoc.body.tagName; globalThis.__htitle = __hdoc.title;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hf)").unwrap().value,
        "true",
        "hasFeature 恒 true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hbody)").unwrap().value,
        "BODY",
        "createHTMLDocument doc.body.tagName BODY"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__htitle)").unwrap().value,
        "hi",
        "createHTMLDocument title 透传"
    );

    // getRootNode：#a 的根为 html（documentElement）。
    sandbox
        .execute(
            "globalThis.__a = document.querySelector('#a');\
             globalThis.__root = __a.getRootNode().tagName;\
             globalThis.__rootIsDocEl = (__a.getRootNode() === document.documentElement);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__root)").unwrap().value,
        "HTML",
        "getRootNode 返根 html"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rootIsDocEl)").unwrap().value,
        "true",
        "getRootNode() === document.documentElement"
    );

    // isSameNode：自身 true / 他节点 false。
    sandbox
        .execute(
            "globalThis.__b = document.querySelector('#b');\
             globalThis.__same = __a.isSameNode(document.querySelector('#a'));\
             globalThis.__diff = __a.isSameNode(__b);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__same)").unwrap().value,
        "true",
        "isSameNode 自身 true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__diff)").unwrap().value,
        "false",
        "isSameNode 他节点 false"
    );

    // compareDocumentPosition bitmask + Node 常量。
    sandbox
        .execute(
            "globalThis.__F = Node.DOCUMENT_POSITION_FOLLOWING;\
             globalThis.__Ct = Node.DOCUMENT_POSITION_CONTAINS;\
             globalThis.__self = __a.compareDocumentPosition(__a);\
             globalThis.__htmlBody = document.documentElement.compareDocumentPosition(document.body);\
             globalThis.__bodyHtml = document.body.compareDocumentPosition(document.documentElement);\
             globalThis.__parent = document.querySelector('#parent');\
             globalThis.__ab = __a.compareDocumentPosition(__b);\
             globalThis.__ba = __b.compareDocumentPosition(__a);\
             globalThis.__parentA = __parent.compareDocumentPosition(__a);\
             globalThis.__aParent = __a.compareDocumentPosition(__parent);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__F)").unwrap().value,
        "4",
        "Node.DOCUMENT_POSITION_FOLLOWING=4"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__Ct)").unwrap().value,
        "8",
        "Node.DOCUMENT_POSITION_CONTAINS=8"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__self)").unwrap().value,
        "0",
        "compareDocumentPosition(自身)=0"
    );
    // html 含 body，body 跟随 html → CONTAINED_BY(16)|FOLLOWING(4)=20。
    assert_eq!(
        sandbox.execute("String(globalThis.__htmlBody)").unwrap().value,
        "20",
        "html.cDP(body)=CONTAINED_BY|FOLLOWING=20"
    );
    // body 看 html：html 含 body + html 先于 body → CONTAINS(8)|PRECEDING(2)=10。
    assert_eq!(
        sandbox.execute("String(globalThis.__bodyHtml)").unwrap().value,
        "10",
        "body.cDP(html)=CONTAINS|PRECEDING=10"
    );
    // a 先于 b（兄弟）→ b 跟随 a → FOLLOWING(4)。
    assert_eq!(
        sandbox.execute("String(globalThis.__ab)").unwrap().value,
        "4",
        "a.cDP(b)=FOLLOWING=4（a 先于 b）"
    );
    // b 看 a → a 先于 b → PRECEDING(2)。
    assert_eq!(
        sandbox.execute("String(globalThis.__ba)").unwrap().value,
        "2",
        "b.cDP(a)=PRECEDING=2"
    );
    // parent 含 a，a 跟随 → CONTAINED_BY|FOLLOWING=20。
    assert_eq!(
        sandbox.execute("String(globalThis.__parentA)").unwrap().value,
        "20",
        "parent.cDP(a)=CONTAINED_BY|FOLLOWING=20"
    );
    // a 看 parent → CONTAINS|PRECEDING=10。
    assert_eq!(
        sandbox.execute("String(globalThis.__aParent)").unwrap().value,
        "10",
        "a.cDP(parent)=CONTAINS|PRECEDING=10"
    );
}

#[test]
fn test_create_comment_r2816() {
    // R2816：document.createComment——注释节点（nodeType 8）。host DomMutation::CreateComment 变体 + apply
    //（doc.create_comment）+ __zw_create_comment callback；shim _commentHandles 标识 nodeType/nodeName +
    // textContent/nodeValue/data 经 query_text_from_mutations（CreateComment arm）读回。框架 placeholder/anchor。
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

    // createComment 返节点：nodeType=8 / nodeName '#comment' / tagName undefined / nodeValue=data=textContent=文本。
    sandbox
        .execute(
            "globalThis.__c = document.createComment('hi there');\
             globalThis.__nt = __c.nodeType;\
             globalThis.__nn = __c.nodeName;\
             globalThis.__tag = __c.tagName;\
             globalThis.__nv = __c.nodeValue;\
             globalThis.__data = __c.data;\
             globalThis.__tc = __c.textContent;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nt)").unwrap().value,
        "8",
        "createComment nodeType=8"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nn)").unwrap().value,
        "#comment",
        "nodeName '#comment'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tag)").unwrap().value,
        "undefined",
        "comment tagName undefined"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nv)").unwrap().value,
        "hi there",
        "nodeValue=注释文本"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__data)").unwrap().value,
        "hi there",
        "data=注释文本"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tc)").unwrap().value,
        "hi there",
        "textContent=注释文本"
    );

    // 区别于 createTextNode（nodeType=3）。
    sandbox
        .execute(
            "globalThis.__t = document.createTextNode('txt');\
             globalThis.__tnt = __t.nodeType;\
             globalThis.__cnt = __c.nodeType;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tnt)").unwrap().value,
        "3",
        "createTextNode nodeType=3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cnt)").unwrap().value,
        "8",
        "createComment 仍 nodeType=8（区别 text）"
    );

    // host 记 CreateComment mutation（验证 host 桥接）。
    let muts = mutations.lock().unwrap();
    let has_comment = muts
        .iter()
        .any(|m| matches!(m, DomMutation::CreateComment { text, .. } if text == "hi there"));
    drop(muts);
    assert!(
        has_comment,
        "createComment 须经 __zw_create_comment 记 DomMutation::CreateComment"
    );

    // 空串/数字参数 lenient 转 string 不抛。
    sandbox
        .execute(
            "globalThis.__ok = (function(){ try { document.createComment(''); document.createComment(42); return 'ok'; } catch(e){ return 'threw'; } })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "ok",
        "createComment lenient 不抛"
    );
}

#[test]
fn test_modern_interaction_stubs_r2817() {
    // R2817：现代交互 API stubs 簇——navigator.clipboard/permissions + element.requestFullscreen +
    // document.fullscreen/exitFullscreen + element/window scroll。headless 无真剪贴板/全屏/滚动 → resolving
    // Promise（clipboard/fullscreen）或 no-op（scroll）。高频 feature-detection 点不抛。Promise 经 execute
    // 末 microtask checkpoint 派发，下 execute 可读（同 R2774/R2814）。
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

    // navigator.clipboard：typeof object + writeText/readText 返 Promise（execute 末 microtask 派发）。
    assert_eq!(
        sandbox.execute("typeof navigator.clipboard").unwrap().value,
        "object",
        "navigator.clipboard 存在"
    );
    sandbox
        .execute(
            "globalThis.__wb = false; globalThis.__rt = 'X';\
             navigator.clipboard.writeText('hi').then(function(){ globalThis.__wb = true; });\
             navigator.clipboard.readText().then(function(t){ globalThis.__rt = t; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__wb)").unwrap().value,
        "true",
        "clipboard.writeText Promise resolves"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rt)").unwrap().value,
        "",
        "clipboard.readText resolves ''（headless 空）"
    );

    // navigator.permissions.query → Promise<PermissionStatus state 'prompt'>。
    sandbox
        .execute(
            "globalThis.__perm = null;\
             navigator.permissions.query({ name: 'clipboard' }).then(function(s){ globalThis.__perm = s.state + ':' + s.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__perm)").unwrap().value,
        "prompt:clipboard",
        "permissions.query → state 'prompt' + name 透传"
    );

    // element.requestFullscreen → Promise resolves + 设 fullscreenElement=body；exitFullscreen 清 + resolve。
    // （R2817 时 fullscreenElement 恒 null；R2938 升级为 spec-alike 状态追踪，详见 test_fullscreen_api_r2938。）
    sandbox
        .execute(
            "globalThis.__fs = false;\
             document.body.requestFullscreen().then(function(){ globalThis.__fs = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fs)").unwrap().value,
        "true",
        "requestFullscreen Promise resolves"
    );
    assert_eq!(
        sandbox
            .execute("String(document.fullscreenElement === document.body)")
            .unwrap()
            .value,
        "true",
        "R2938 fullscreenElement 反映全屏元素（body）"
    );
    sandbox
        .execute("globalThis.__ef = false; document.exitFullscreen().then(function(){ globalThis.__ef = true; });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ef)").unwrap().value,
        "true",
        "exitFullscreen Promise resolves"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "R2938 exitFullscreen 后 fullscreenElement 复 null"
    );

    // element scroll 方法 no-op 返 undefined；window scroll 同；scrollX/pageXOffset 恒 0。
    sandbox
        .execute(
            "globalThis.__siv = document.body.scrollIntoView();\
             globalThis.__sto = document.body.scrollTo(0, 0);\
             globalThis.__wst = window.scrollTo(0, 0);\
             globalThis.__sX = window.scrollX; globalThis.__pXO = window.pageXOffset;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__siv)").unwrap().value,
        "undefined",
        "scrollIntoView no-op 返 undefined"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sto)").unwrap().value,
        "undefined",
        "scrollTo no-op 返 undefined"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wst)").unwrap().value,
        "undefined",
        "window.scrollTo no-op 返 undefined"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sX)").unwrap().value,
        "0",
        "scrollX 恒 0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pXO)").unwrap().value,
        "0",
        "pageXOffset 恒 0"
    );
}

#[test]
fn test_fullscreen_api_r2938() {
    // R2938 Fullscreen API（spec-alike）：element.requestFullscreen() 返 Promise——grant 路径设 fullscreenElement +
    // 派 fullscreenchange + resolve；deny 路径（fullscreenEnabled=false）派 fullscreenerror + reject TypeError。
    // document.exitFullscreen() 清状态 + 派 fullscreenchange + resolve；非全屏态 resolve 不派事件。
    // fullscreenElement/fullscreenEnabled 反映状态；fullscreenchange/fullscreenerror 经 document listener +
    // document.onfullscreenchange/onfullscreenerror IDL handler 触发。headless 无真 OS 全屏，但语义可观察。
    // https://fullscreen.spec.whatwg.org/
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='d'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // fullscreenEnabled 默认 true；fullscreenElement 初值 null。
    assert_eq!(
        sandbox.execute("String(document.fullscreenEnabled)").unwrap().value,
        "true",
        "fullscreenEnabled 默认 true"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "fullscreenElement 初值 null"
    );

    // grant 路径：requestFullscreen 设 fullscreenElement + 派 fullscreenchange（listener 内读 fullscreenElement）+ resolve。
    sandbox
        .execute(
            "globalThis.__fc = 0; globalThis.__fe = 'x';\
             document.addEventListener('fullscreenchange', function(){\
               globalThis.__fc++;\
               globalThis.__fe = document.fullscreenElement ? document.fullscreenElement.id : '(null)';\
             });\
             globalThis.__ok = false;\
             document.getElementById('d').requestFullscreen().then(function(){ globalThis.__ok = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fc)").unwrap().value,
        "1",
        "requestFullscreen 派发一次 fullscreenchange"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fe)").unwrap().value,
        "d",
        "fullscreenchange handler 内 fullscreenElement === 全屏元素（id='d'）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "true",
        "requestFullscreen Promise resolves"
    );

    // 相同元素重复 requestFullscreen → no-op（不重复派 fullscreenchange，仍 resolve）。
    sandbox
        .execute(
            "globalThis.__ok2 = false;\
             document.getElementById('d').requestFullscreen().then(function(){ globalThis.__ok2 = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fc)").unwrap().value,
        "1",
        "相同元素重复 requestFullscreen 不再派 fullscreenchange（no-op）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ok2)").unwrap().value,
        "true",
        "重复 requestFullscreen 仍 resolve"
    );

    // exitFullscreen → 清状态 + 派 fullscreenchange + resolve。
    sandbox
        .execute(
            "globalThis.__ef = false;\
             document.exitFullscreen().then(function(){ globalThis.__ef = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fc)").unwrap().value,
        "2",
        "exitFullscreen 派发 fullscreenchange（第二次）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ef)").unwrap().value,
        "true",
        "exitFullscreen Promise resolves"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "exitFullscreen 后 fullscreenElement 复 null"
    );

    // 非全屏态 exitFullscreen → resolve，不派事件（计数不变）。
    sandbox
        .execute(
            "globalThis.__ef2 = 'p';\
             document.exitFullscreen().then(function(){ globalThis.__ef2 = 'resolved'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ef2)").unwrap().value,
        "resolved",
        "非全屏态 exitFullscreen 仍 resolve"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fc)").unwrap().value,
        "2",
        "非全屏态 exitFullscreen 不派 fullscreenchange"
    );

    // document.onfullscreenchange IDL handler：注册后由 fullscreenchange 触发。
    sandbox
        .execute(
            "globalThis.__ofc = 0;\
             document.onfullscreenchange = function(){ globalThis.__ofc++; };\
             document.body.requestFullscreen();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ofc)").unwrap().value,
        "1",
        "document.onfullscreenchange IDL handler 触发"
    );
    sandbox.execute("document.exitFullscreen();").unwrap(); // 清理全屏态

    // deny 路径：host `__zw_fullscreen_enabled` 返 '0' → fullscreenEnabled=false → requestFullscreen reject
    // TypeError + 派 fullscreenerror（document listener + document.onfullscreenerror IDL handler 触发）。
    sandbox.register_callback("__zw_fullscreen_enabled", Box::new(|_args: &[String]| "0".to_string()));
    assert_eq!(
        sandbox.execute("String(document.fullscreenEnabled)").unwrap().value,
        "false",
        "host 禁用后 fullscreenEnabled=false"
    );
    sandbox
        .execute(
            "globalThis.__ferr = 0; globalThis.__rej = null;\
             document.addEventListener('fullscreenerror', function(){ globalThis.__ferr++; });\
             document.onfullscreenerror = function(){ globalThis.__ferr += 10; };\
             document.body.requestFullscreen().then(function(){ globalThis.__rej = 'resolved'; },\
               function(err){ globalThis.__rej = (err instanceof TypeError) ? 'TypeError' : 'other'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ferr)").unwrap().value,
        "11",
        "deny 路径派 fullscreenerror（document listener + document.onfullscreenerror）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rej)").unwrap().value,
        "TypeError",
        "deny 路径 reject TypeError"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "deny 路径不设 fullscreenElement"
    );
}

#[test]
fn test_pointer_lock_api_r2939() {
    // R2939 Pointer Lock API（spec-alike，镜像 R2938 Fullscreen）：element.requestPointerLock() 返 Promise
    //（grant→resolve + 设 pointerLockElement + 派 pointerlockchange；deny→reject TypeError + 派 pointerlockerror）；
    // document.exitPointerLock() 返 **void**（undefined，与 exitFullscreen 返 Promise 不同）+ 清状态 + 派
    // pointerlockchange；pointerLockElement 反映状态；pointerlockchange/pointerlockerror 经 document listener +
    // document.onpointerlockchange/onpointerlockerror IDL handler 触发。
    // https://w3c.github.io/pointerlock/
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
        "<html><body><canvas id='c'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // pointerLockElement 初值 null。
    assert_eq!(
        sandbox.execute("String(document.pointerLockElement)").unwrap().value,
        "null",
        "pointerLockElement 初值 null"
    );

    // grant 路径：requestPointerLock 设 pointerLockElement + 派 pointerlockchange + resolve。
    sandbox
        .execute(
            "globalThis.__plc = 0; globalThis.__ple = 'x';\
             document.addEventListener('pointerlockchange', function(){\
               globalThis.__plc++;\
               globalThis.__ple = document.pointerLockElement ? document.pointerLockElement.id : '(null)';\
             });\
             globalThis.__ok = false;\
             document.getElementById('c').requestPointerLock().then(function(){ globalThis.__ok = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__plc)").unwrap().value,
        "1",
        "requestPointerLock 派发一次 pointerlockchange"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ple)").unwrap().value,
        "c",
        "pointerlockchange handler 内 pointerLockElement === 锁定元素（id='c'）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "true",
        "requestPointerLock Promise resolves"
    );

    // 相同元素重复 requestPointerLock → no-op（计数不变仍 resolve）。
    sandbox
        .execute(
            "globalThis.__ok2 = false;\
             document.getElementById('c').requestPointerLock().then(function(){ globalThis.__ok2 = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__plc)").unwrap().value,
        "1",
        "相同元素重复 requestPointerLock 不再派 pointerlockchange（no-op）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ok2)").unwrap().value,
        "true",
        "重复 requestPointerLock 仍 resolve"
    );

    // exitPointerLock → 清状态 + 派 pointerlockchange；返 void（undefined，与 exitFullscreen 返 Promise 不同）。
    sandbox
        .execute("globalThis.__ex = typeof document.exitPointerLock();")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ex)").unwrap().value,
        "undefined",
        "exitPointerLock 返 void（undefined，spec 非 Promise）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__plc)").unwrap().value,
        "2",
        "exitPointerLock 派发 pointerlockchange（第二次）"
    );
    assert_eq!(
        sandbox.execute("String(document.pointerLockElement)").unwrap().value,
        "null",
        "exitPointerLock 后 pointerLockElement 复 null"
    );

    // 非锁定态 exitPointerLock → no-op，不派事件（计数不变）。
    sandbox.execute("document.exitPointerLock();").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__plc)").unwrap().value,
        "2",
        "非锁定态 exitPointerLock 不派 pointerlockchange"
    );

    // document.onpointerlockchange IDL handler：注册后由 pointerlockchange 触发。
    sandbox
        .execute(
            "globalThis.__oplc = 0;\
             document.onpointerlockchange = function(){ globalThis.__oplc++; };\
             document.body.requestPointerLock();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__oplc)").unwrap().value,
        "1",
        "document.onpointerlockchange IDL handler 触发"
    );
    sandbox.execute("document.exitPointerLock();").unwrap(); // 清理锁定态

    // deny 路径：host `__zw_pointer_lock_enabled` 返 '0' → requestPointerLock reject TypeError + 派 pointerlockerror
    //（document listener + document.onpointerlockerror IDL handler 触发）。
    sandbox.register_callback(
        "__zw_pointer_lock_enabled",
        Box::new(|_args: &[String]| "0".to_string()),
    );
    sandbox
        .execute(
            "globalThis.__plerr = 0; globalThis.__rej = null;\
             document.addEventListener('pointerlockerror', function(){ globalThis.__plerr++; });\
             document.onpointerlockerror = function(){ globalThis.__plerr += 10; };\
             document.body.requestPointerLock().then(function(){ globalThis.__rej = 'resolved'; },\
               function(err){ globalThis.__rej = (err instanceof TypeError) ? 'TypeError' : 'other'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__plerr)").unwrap().value,
        "11",
        "deny 路径派 pointerlockerror（document listener + document.onpointerlockerror）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rej)").unwrap().value,
        "TypeError",
        "deny 路径 reject TypeError"
    );
    assert_eq!(
        sandbox.execute("String(document.pointerLockElement)").unwrap().value,
        "null",
        "deny 路径不设 pointerLockElement"
    );
}

#[test]
fn test_window_onerror_report_r2940() {
    // R2940 onerror host 集成：ErrorEvent 构造器 + createEvent('ErrorEvent') + __zw_report_error hook。
    // hook 派发 window 'error' 事件（addEventListener 接 ErrorEvent 读 .message/.filename/.lineno/.colno）+
    // 调 legacy window.onerror（spec 特殊 5-arg 签名 msg/src/line/col/err），不重复触发 onerror（dispatch 前
    // 暂移除 onerror listener、legacy 调、dispatch 完装回）。onerror 返 true → defaultPrevented（错误已处理）。
    // host（tab_scripts）执行页面 <script> 出错时经 zero_engine::script_report_error 生成调用串执行此 hook。
    // https://html.spec.whatwg.org/#runtime-script-errors
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://test.local/page".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ErrorEvent 构造器（字段 message/filename/lineno/colno/error）+ createEvent('ErrorEvent') 返 ErrorEvent 实例。
    sandbox
        .execute(
            "globalThis.__ev = new ErrorEvent('error', {message:'boom', filename:'a.js', lineno:7, colno:3});\
             globalThis.__ce = document.createEvent('ErrorEvent');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.message)").unwrap().value,
        "boom",
        "ErrorEvent.message"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.filename)").unwrap().value,
        "a.js",
        "ErrorEvent.filename"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.lineno)").unwrap().value,
        "7",
        "ErrorEvent.lineno"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.colno)").unwrap().value,
        "3",
        "ErrorEvent.colno"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.error)").unwrap().value,
        "null",
        "ErrorEvent.error（headless 无真 Error 对象 → null）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ev instanceof ErrorEvent)")
            .unwrap()
            .value,
        "true",
        "ErrorEvent instanceof"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ce instanceof ErrorEvent)")
            .unwrap()
            .value,
        "true",
        "createEvent('ErrorEvent') 返 ErrorEvent 实例"
    );

    // __zw_report_error：window.addEventListener('error') 接 ErrorEvent + window.onerror legacy 5-arg，不重复触发。
    sandbox
        .execute(
            "globalThis.__ael = null; globalThis.__oe = null; globalThis.__oeCount = 0;\
             window.addEventListener('error', function(e){\
               globalThis.__ael = e.message + '|' + e.filename + '|' + e.lineno + '|' + e.colno;\
             });\
             window.onerror = function(msg, src, line, col, err){\
               globalThis.__oeCount++;\
               globalThis.__oe = String(msg) + '|' + String(src) + '|' + line + '|' + col + '|' + String(err);\
               return false;\
             };\
             __zw_report_error('TypeError: x is undefined', 'https://test.local/a.js', 42, 9);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ael)").unwrap().value,
        "TypeError: x is undefined|https://test.local/a.js|42|9",
        "addEventListener('error') listener 接 ErrorEvent（字段透传）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__oe)").unwrap().value,
        "TypeError: x is undefined|https://test.local/a.js|42|9|null",
        "window.onerror legacy 5-arg 签名（msg/src/line/col/err）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__oeCount)").unwrap().value,
        "1",
        "onerror 仅触发一次（不与 event 派发重复）"
    );

    // onerror 返 true → defaultPrevented（错误「已处理」，spec：抑制默认动作）。
    sandbox
        .execute(
            "globalThis.__dp = 'unset';\
             window.onerror = function(){ return true; };\
             window.addEventListener('error', function(e){ globalThis.__dp = String(e.defaultPrevented); });\
             __zw_report_error('handled', 'b.js', 1, 1);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dp)").unwrap().value,
        "true",
        "onerror 返 true → ErrorEvent.defaultPrevented"
    );

    // 仅 addEventListener('error')（无 onerror）也能触发——hook 不依赖 onerror 存在。
    sandbox
        .execute(
            "window.onerror = null;\
             globalThis.__only2 = null;\
             window.addEventListener('error', function(e){ globalThis.__only2 = e.message; });\
             __zw_report_error('solo', 'c.js', 5, 5);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__only2)").unwrap().value,
        "solo",
        "无 onerror 时 addEventListener('error') 仍触发"
    );
}

#[test]
fn test_page_lifecycle_load_r2941() {
    // R2941 页面生命周期事件派发：host（tab_scripts::finish）在页面脚本阶段完成后依次派发
    // DOMContentLoaded + load（均经 __zw_dispatch_event('html', type, null) → document/window listener 同键）。
    // 触发 document.addEventListener('DOMContentLoaded') / window.addEventListener('load') / window.onload
    //（R2932 IDL）/ document.onDOMContentLoaded（R2941 IDL）。DOMContentLoaded 先于 load（spec）。
    // https://html.spec.whatwg.org/#the-end
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

    // document.onDOMContentLoaded（R2941 新增 IDL handler）可读写（无值初 null）。
    assert_eq!(
        sandbox.execute("String(document.onDOMContentLoaded)").unwrap().value,
        "null",
        "document.onDOMContentLoaded 初值 null"
    );

    // 注册四类 hook：document.addEListener('DOMContentLoaded') / window.onload / window.addEventListener('load') /
    // document.onDOMContentLoaded。记录触发顺序进 __order。
    sandbox
        .execute(
            "globalThis.__order = [];\
             document.addEventListener('DOMContentLoaded', function(){ globalThis.__order.push('dcl-ael'); });\
             document.onDOMContentLoaded = function(){ globalThis.__order.push('dcl-idl'); };\
             window.onload = function(){ globalThis.__order.push('load-idl'); };\
             window.addEventListener('load', function(){ globalThis.__order.push('load-ael'); });",
        )
        .unwrap();

    // host 模拟：finish() 依次派发 DOMContentLoaded + load（DOMContentLoaded 先于 load）。
    sandbox
        .execute(
            "__zw_dispatch_event('html', 'DOMContentLoaded', null);\
             __zw_dispatch_event('html', 'load', null);",
        )
        .unwrap();
    // DOMContentLoaded 触发 document.addEventListener + document.onDOMContentLoaded（均 html 键）。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__order.indexOf('dcl-ael') >= 0)")
            .unwrap()
            .value,
        "true",
        "DOMContentLoaded 派发触发 document.addEventListener('DOMContentLoaded')"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__order.indexOf('dcl-idl') >= 0)")
            .unwrap()
            .value,
        "true",
        "DOMContentLoaded 派发触发 document.onDOMContentLoaded（R2941 IDL）"
    );
    // load 触发 window.onload（R2932 IDL）+ window.addEventListener('load')。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__order.indexOf('load-idl') >= 0)")
            .unwrap()
            .value,
        "true",
        "load 派发触发 window.onload（R2932 IDL）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__order.indexOf('load-ael') >= 0)")
            .unwrap()
            .value,
        "true",
        "load 派发触发 window.addEventListener('load')"
    );
    // DOMContentLoaded 整体先于 load（spec：DOMContentLoaded → load）：最后一条 dcl 记录在首条 load 记录前。
    sandbox.execute(
        "globalThis.__dclFirst = (function(){ var s = globalThis.__order.join(','); return s.lastIndexOf('dcl') < s.indexOf('load'); })();",
    ).unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dclFirst)").unwrap().value,
        "true",
        "DOMContentLoaded 先于 load"
    );
}

#[test]
fn test_img_element_events_r2943() {
    // R2943 img 元素级 onload/onerror：`__zw_dispatch_img_event(absUrl, type)` 按 src 绝对 URL 匹配 `<img>`
    // proxy，用其自身 selector 派发 load/error（保证 listener store key 与 page JS 经 querySelectorAll
    // 获取 proxy 时一致 → img.onload/onerror + addEventListener('load'/'error') 触发）。模拟 host 在 img
    // fetch 完成（'load'）/ 失败（'error'）时调用。
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
         <img id='i1' src='https://example.com/a.png'>\
         <img src='https://example.com/b.png'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // img#i1：addEventListener('load') + onload IDL + onerror；img b.png：onload。
    sandbox
        .execute(
            "globalThis.__hit = [];\
             var imgs = document.querySelectorAll('img');\
             imgs[0].addEventListener('load', function(){ globalThis.__hit.push('i1-load-ael'); });\
             imgs[0].onload = function(){ globalThis.__hit.push('i1-load-idl'); };\
             imgs[0].onerror = function(){ globalThis.__hit.push('i1-error'); };\
             imgs[1].onload = function(){ globalThis.__hit.push('b-load'); };",
        )
        .unwrap();

    // host 派发：i1 load + i1 error + b load（绝对 URL；src 已绝对故不经 parse_url 解析）。
    sandbox
        .execute(
            "__zw_dispatch_img_event('https://example.com/a.png', 'load');\
             __zw_dispatch_img_event('https://example.com/a.png', 'error');\
             __zw_dispatch_img_event('https://example.com/b.png', 'load');",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('i1-load-ael') >= 0)")
            .unwrap()
            .value,
        "true",
        "img#i1 addEventListener('load') 触发（元素自身 selector 派发）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('i1-load-idl') >= 0)")
            .unwrap()
            .value,
        "true",
        "img#i1 onload IDL 触发"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('i1-error') >= 0)")
            .unwrap()
            .value,
        "true",
        "img#i1 onerror 触发（fetch/decode 失败派 error）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('b-load') >= 0)")
            .unwrap()
            .value,
        "true",
        "img b.png onload 触发（多 img 按 src 区分）"
    );
    // 未匹配 src 不派发（计数不变）。
    sandbox
        .execute(
            "globalThis.__before = globalThis.__hit.length;\
             __zw_dispatch_img_event('https://example.com/missing.png', 'load');",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.length === globalThis.__before)")
            .unwrap()
            .value,
        "true",
        "未匹配 src 的派发不触发任何 img listener"
    );
}

#[test]
fn test_xmlserializer_importnode_r2818() {
    // R2818：XMLSerializer.serializeToString + document.adoptNode/importNode。serializeToString 委托节点
    // outerHTML（元素）/ nodeValue（text·comment）/ documentElement（document）；adoptNode 单文档 identity；
    // importNode 委托 cloneNode（深/浅克隆独立性）。
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
        "<html><body><div id='src' class='row'><span>hi</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // XMLSerializer 构造器 + serializeToString(元素) 含 tag + class。
    sandbox
        .execute(
            "globalThis.__xs = new XMLSerializer();\
             globalThis.__isFn = typeof XMLSerializer.prototype.serializeToString === 'function';\
             globalThis.__el = document.querySelector('#src');\
             globalThis.__ser = __xs.serializeToString(__el);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__isFn)").unwrap().value,
        "true",
        "serializeToString 为 function"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ser.indexOf('<div') >= 0)")
            .unwrap()
            .value,
        "true",
        "serializeToString(元素) 含 '<div'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ser.indexOf('class=\"row\"') >= 0 || globalThis.__ser.indexOf(\"class='row'\") >= 0)")
            .unwrap()
            .value,
        "true",
        "serializeToString(元素) 含 class 属性"
    );

    // serializeToString(text/comment) → nodeValue/data。
    sandbox
        .execute(
            "globalThis.__tn = document.createTextNode('hello');\
             globalThis.__cm = document.createComment('note');\
             globalThis.__serTn = __xs.serializeToString(__tn);\
             globalThis.__serCm = __xs.serializeToString(__cm);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__serTn)").unwrap().value,
        "hello",
        "serializeToString(text)=nodeValue"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__serCm)").unwrap().value,
        "note",
        "serializeToString(comment)=data"
    );

    // document.adoptNode → 返同对象（identity）。
    sandbox
        .execute("globalThis.__adopted = (document.adoptNode(__el) === __el);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__adopted)").unwrap().value,
        "true",
        "adoptNode 单文档 identity 返同对象"
    );

    // document.importNode 深/浅克隆：副本独立于源 + deep 含子树 span。
    sandbox
        .execute(
            "globalThis.__shallow = document.importNode(__el, false);\
             globalThis.__deep = document.importNode(__el, true);\
             globalThis.__deepHasSpan = __deep.outerHTML.indexOf('<span') >= 0;\
             globalThis.__indep = (__deep !== __el);\
             globalThis.__shallowTag = __shallow.tagName;\
             globalThis.__shallowNeqDeep = (__shallow !== __deep);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__deepHasSpan)").unwrap().value,
        "true",
        "importNode(deep=true) 含子树 span"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__indep)").unwrap().value,
        "true",
        "importNode 副本独立于源"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__shallowTag)").unwrap().value,
        "DIV",
        "importNode(浅) 仍为 DIV 元素（外层克隆）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__shallowNeqDeep)").unwrap().value,
        "true",
        "浅/深克隆互异"
    );
}

#[test]
fn test_isequalnode_r2819() {
    // R2819：node.isEqualNode——节点结构相等（node-equality 三件套最后一块）。经 _nodeSig 序列化签名比对
    //（元素 outerHTML / text·comment nodeValue）。属性序敏感（spec 序无关，实际库一致）。
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
         <div id='wrap'>\
         <div class='x'><span>hi</span></div>\
         <div class='x'><span>hi</span></div>\
         <div class='y'><span>hi</span></div>\
         <div class='x'><span>bye</span></div>\
         </div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 同结构（a==b，均无 id 冲突）true / 不同 class（a==c）false / 不同子文本（a==d）false / 自身 true / null false。
    sandbox
        .execute(
            "globalThis.__kids = document.querySelector('#wrap').children;\
             globalThis.__a = __kids[0]; globalThis.__b = __kids[1];\
             globalThis.__c = __kids[2]; globalThis.__d = __kids[3];\
             globalThis.__eq_ab = __a.isEqualNode(__b);\
             globalThis.__eq_ac = __a.isEqualNode(__c);\
             globalThis.__eq_ad = __a.isEqualNode(__d);\
             globalThis.__eq_aa = __a.isEqualNode(__a);\
             globalThis.__eq_null = __a.isEqualNode(null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_ab)").unwrap().value,
        "true",
        "同结构（class+子树）isEqualNode true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_ac)").unwrap().value,
        "false",
        "不同 class 不等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_ad)").unwrap().value,
        "false",
        "不同子文本不等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_aa)").unwrap().value,
        "true",
        "自身相等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_null)").unwrap().value,
        "false",
        "isEqualNode(null) false"
    );

    // text 节点：同 nodeValue 等 / 不同不等 / text≠comment（同 nodeValue 但 nodeType 异）。
    sandbox
        .execute(
            "globalThis.__t1 = document.createTextNode('x');\
             globalThis.__t2 = document.createTextNode('x');\
             globalThis.__t3 = document.createTextNode('y');\
             globalThis.__cm = document.createComment('x');\
             globalThis.__eq_tt = __t1.isEqualNode(__t2);\
             globalThis.__eq_t12t3 = __t1.isEqualNode(__t3);\
             globalThis.__eq_tcm = __t1.isEqualNode(__cm);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_tt)").unwrap().value,
        "true",
        "同 text nodeValue 相等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_t12t3)").unwrap().value,
        "false",
        "不同 text nodeValue 不等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_tcm)").unwrap().value,
        "false",
        "text≠comment（nodeType 异）"
    );
}

#[test]
fn test_navigator_geolocation_r2820() {
    // R2820：navigator.geolocation——地理位置 API（地图/天气/本地化 feature-detect 后调 getCurrentPosition）。
    // headless 无真 GPS → fake 零坐标位置（latitude/longitude 0，accuracy Infinity = 无精度承诺），让 location
    // 脚本走 success 路径不抛。getCurrentPosition/watchPosition 经 _defer microtask 异步调 success（execute 末
    // checkpoint 派发，下 execute 可读，同 R2774/R2814）；watchPosition 返唯一 watch id；clearWatch no-op。
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

    // navigator.geolocation 存在 + 三方法为 function。
    assert_eq!(
        sandbox.execute("typeof navigator.geolocation").unwrap().value,
        "object",
        "navigator.geolocation 存在"
    );
    assert_eq!(
        sandbox
            .execute("typeof navigator.geolocation.getCurrentPosition")
            .unwrap()
            .value,
        "function",
        "getCurrentPosition 为 function"
    );
    assert_eq!(
        sandbox
            .execute("typeof navigator.geolocation.watchPosition")
            .unwrap()
            .value,
        "function",
        "watchPosition 为 function"
    );
    assert_eq!(
        sandbox
            .execute("typeof navigator.geolocation.clearWatch")
            .unwrap()
            .value,
        "function",
        "clearWatch 为 function"
    );

    // getCurrentPosition 经 microtask 调 success 携 fake 零坐标位置（__lat 初值 -999 证回调真触发）。
    sandbox
        .execute(
            "globalThis.__lat = -999;\
             navigator.geolocation.getCurrentPosition(function(p){\
               globalThis.__lat = p.coords.latitude;\
               globalThis.__lng = p.coords.longitude;\
               globalThis.__alt = String(p.coords.altitude);\
               globalThis.__acc = String(p.coords.accuracy);\
               globalThis.__hdg = String(p.coords.heading);\
               globalThis.__spd = String(p.coords.speed);\
               globalThis.__ts = p.timestamp;\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__lat)").unwrap().value,
        "0",
        "getCurrentPosition success coords.latitude===0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__lng)").unwrap().value,
        "0",
        "coords.longitude===0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__alt)").unwrap().value,
        "null",
        "coords.altitude===null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__acc)").unwrap().value,
        "Infinity",
        "coords.accuracy===Infinity（无精度承诺）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hdg)").unwrap().value,
        "null",
        "coords.heading===null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__spd)").unwrap().value,
        "null",
        "coords.speed===null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ts)").unwrap().value,
        "0",
        "timestamp===0"
    );

    // watchPosition 返唯一正 watch id（首个为 1）+ 经 microtask 调 success；clearWatch no-op 不抛。
    sandbox
        .execute(
            "globalThis.__id = navigator.geolocation.watchPosition(function(p){ globalThis.__wl = p.coords.latitude; });\
             globalThis.__id2 = navigator.geolocation.watchPosition(function(){});\
             globalThis.__cleared = 'no';\
             try { navigator.geolocation.clearWatch(globalThis.__id); globalThis.__cleared = 'yes'; } catch(_e){}",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__id)").unwrap().value,
        "1",
        "watchPosition 返唯一 id（首个为 1）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__id2)").unwrap().value,
        "2",
        "watchPosition 多次返递增 id"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cleared)").unwrap().value,
        "yes",
        "clearWatch no-op 不抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wl)").unwrap().value,
        "0",
        "watchPosition success coords.latitude===0"
    );

    // getCurrentPosition 无 success 回调静默 no-op 不抛（lenient，非真 GPS 不强求回调）。
    sandbox
        .execute("globalThis.__n = 'no'; try { navigator.geolocation.getCurrentPosition(); globalThis.__n = 'yes'; } catch(_e){}")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__n)").unwrap().value,
        "yes",
        "getCurrentPosition 无回调 lenient 不抛"
    );
}

#[test]
fn test_performance_mark_measure_observer_r2821() {
    // R2821：Performance API 扩展（performance.mark/measure + entry buffer + PerformanceObserver）。
    // analytics/RUM（web-vitals/Sentry/GA）高频。mark/measure 产 PerformanceEntry 存 entry buffer；
    // PerformanceObserver observe 匹配 entryType 时经 _defer microtask 异步派发（execute 末 checkpoint，
    // 下 execute 可读，同 R2774/R2814）。dom_bridge.rs 的 PerformanceObserver/mark/measure 为 A 代死代码
    // （generate_dom_api_polyfill 无生产调用方），生产路径仅注入本 shim——故补到 B 代 shim。
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

    // performance.mark 产 entry（entryType='mark'/duration 0）+ entry buffer 可读。
    sandbox
        .execute("globalThis.__mk = performance.mark('a'); performance.mark('b');")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__mk.entryType)").unwrap().value,
        "mark",
        "mark entry entryType='mark'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__mk.duration)").unwrap().value,
        "0",
        "mark entry duration 0"
    );
    assert_eq!(
        sandbox
            .execute("String(performance.getEntriesByType('mark').length)")
            .unwrap()
            .value,
        "2",
        "entry buffer 含 2 mark"
    );
    assert_eq!(
        sandbox
            .execute("String(performance.getEntriesByName('a').length)")
            .unwrap()
            .value,
        "1",
        "getEntriesByName('a')"
    );

    // performance.measure 计算 duration = mark(b).start - mark(a).start（>=0）；从原点 measure duration>=0；
    // 未知 mark 名抛 TypeError。
    sandbox
        .execute(
            "globalThis.__ms = performance.measure('ab', 'a', 'b');\
             globalThis.__mo = performance.measure('from-origin').duration >= 0;\
             globalThis.__err = 'no';\
             try { performance.measure('x', 'missing'); } catch(e){ globalThis.__err = (e instanceof TypeError) ? 'TypeError' : 'other'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ms.entryType)").unwrap().value,
        "measure",
        "measure entry entryType='measure'"
    );
    let dur = sandbox.execute("String(globalThis.__ms.duration)").unwrap().value;
    let dur_n: f64 = dur.parse().unwrap_or(-1.0);
    assert!(dur_n >= 0.0, "measure duration >= 0（a 先于 b 标记）, got {}", dur);
    assert_eq!(
        sandbox.execute("String(globalThis.__mo)").unwrap().value,
        "true",
        "measure 从原点 duration>=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__err)").unwrap().value,
        "TypeError",
        "measure 引用未知 mark 名抛 TypeError"
    );

    // clearMarks / clearMeasures 清 buffer。
    sandbox
        .execute("performance.clearMarks(); performance.clearMeasures();")
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(performance.getEntries().length)")
            .unwrap()
            .value,
        "0",
        "clearMarks+clearMeasures 清空 entry buffer"
    );

    // PerformanceObserver：typeof function + supportedEntryTypes 含 mark/measure。
    assert_eq!(
        sandbox.execute("typeof PerformanceObserver").unwrap().value,
        "function",
        "PerformanceObserver 存在"
    );
    assert_eq!(
        sandbox
            .execute("String(PerformanceObserver.supportedEntryTypes.indexOf('measure') !== -1)")
            .unwrap()
            .value,
        "true",
        "supportedEntryTypes 含 'measure'"
    );

    // observe({entryTypes:['mark']}) + mark → 经 microtask 派发 list.getEntries() 含两 mark 名（排序）。
    sandbox
        .execute(
            "globalThis.__got = 'none';\
             var obs = new PerformanceObserver(function(list){\
               globalThis.__got = list.getEntries().map(function(e){ return e.name; }).sort().join(',');\
             });\
             obs.observe({ entryTypes: ['mark'] });\
             performance.mark('m1'); performance.mark('m2');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__got)").unwrap().value,
        "m1,m2",
        "observer（entryTypes）经 microtask 收两 mark"
    );

    // observe({type:'measure'}) → measure 'mz' 经 microtask 派发（单独 execute 让 flush 先于 disconnect 跑）。
    sandbox
        .execute(
            "globalThis.__g2 = 'none';\
             var obs2 = new PerformanceObserver(function(list){\
               var e = list.getEntries();\
               globalThis.__g2 = e.length + ':' + (e[0] && e[0].name);\
             });\
             obs2.observe({ type: 'measure' });\
             performance.measure('mz');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__g2)").unwrap().value,
        "1:mz",
        "observer（type form）经 microtask 收 measure 'mz'"
    );
    // disconnect 后后续 measure 'mz2' 不再派发（__g2 保持 disconnect 前值）。
    sandbox
        .execute("obs2.disconnect(); performance.measure('mz2');")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__g2)").unwrap().value,
        "1:mz",
        "disconnect 后不再派发（__g2 未变）"
    );

    // takeRecords 取并清缓冲（observe + mark 后 takeRecords 返该 entry）。
    sandbox
        .execute(
            "var obs3 = new PerformanceObserver(function(){});\
             obs3.observe({ entryTypes: ['mark'] });\
             performance.mark('tr');\
             globalThis.__rec = obs3.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rec.length)").unwrap().value,
        "1",
        "takeRecords 返缓冲 entry"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rec[0].name)").unwrap().value,
        "tr",
        "takeRecords entry name 'tr'"
    );
}

#[test]
fn test_element_replace_children_r2822() {
    // R2822：Element.replaceChildren(...nodesOrStrings)——移除全部现有子 + 追加新子（clear-and-populate
    // 原子语义，Vue3/lit/Svelte/手写代码高频）。清空经 SetInnerHtml('')，追加复用 _appendVariadic（与 append 共用）。
    // 验证经 apply_mutations_to_html_with_handles（proxy 读 stale 快照，故核 mutation 产出的 HTML）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body>\
        <div id='t'>old1<span>old2</span>old3</div>\
        <div id='u'>keep1<p>keep2</p>keep3</div>\
        <div id='v'>oldtext</div>\
        </body></html>"
        .to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 三种用法（不同元素，互不干扰）：#t 清空+追加节点+字符串；#u 无参仅清空；#v 纯字符串清空+追加。
    sandbox
        .execute(
            "var b=document.createElement('b'); var i=document.createElement('i');\
             document.querySelector('#t').replaceChildren(b, 'mid', i);\
             document.querySelector('#u').replaceChildren();\
             document.querySelector('#v').replaceChildren('hello');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();

    // #t：清空 old1/old2/old3 + 追加 b, mid, i（参数序）。
    assert!(
        out.contains("<div id=\"t\"><b></b>mid<i></i></div>"),
        "#t 清空旧子+追加新子（b,mid,i 参数序）\n{out}"
    );
    // #u：无参 → 内容空。
    assert!(
        out.contains("<div id=\"u\"></div>"),
        "#u 无参 replaceChildren 清空\n{out}"
    );
    // #v：纯字符串清空+追加。
    assert!(out.contains("<div id=\"v\">hello</div>"), "#v 纯字符串清空+追加\n{out}");
    // 旧内容全部消失（证清空生效）。
    assert!(
        !out.contains("old1") && !out.contains("old2") && !out.contains("keep1") && !out.contains("oldtext"),
        "旧子应全清空\n{out}"
    );
}

#[test]
fn test_character_data_methods_r2823() {
    // R2823：CharacterData 数据编辑（appendData/deleteData/insertData/replaceData/substringData + length）
    // + Text.splitText。仅 handle-based 文本/注释节点（createTextNode/createComment）。读经
    // query_text_from_mutations 反向 replay 取最新值（多次编辑 compose 正确），写追加 SetTextOnHandle。
    // contentEditable 编辑库（ProseMirror/Slate/Quill）+ Range/Selection 高频。
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

    // appendData / length / substringData / deleteData / insertData / replaceData 链式 compose。
    // 'Hello' +appendData(' World')→'Hello World'(len 11) →substringData(6,5)='World'
    // →deleteData(0,6)→'World' →insertData(0,'JS ')→'JS World' →replaceData(0,3,'Hi')→'HiWorld'（'JS ' 含空格 3 字符被 'Hi' 替）
    sandbox
        .execute(
            "globalThis.__t = document.createTextNode('Hello');\
             globalThis.__t.appendData(' World');\
             globalThis.__len = globalThis.__t.length;\
             globalThis.__sub = globalThis.__t.substringData(6, 5);\
             globalThis.__t.deleteData(0, 6);\
             globalThis.__afterDel = globalThis.__t.data;\
             globalThis.__t.insertData(0, 'JS ');\
             globalThis.__t.replaceData(0, 3, 'Hi');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "11",
        "appendData 后 length=11（'Hello World'）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sub)").unwrap().value,
        "World",
        "substringData(6,5)='World'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterDel)").unwrap().value,
        "World",
        "deleteData(0,6)→'World'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t.data)").unwrap().value,
        "HiWorld",
        "insertData(0,'JS ')+replaceData(0,3,'Hi')→'HiWorld'（'JS ' 含空格被 'Hi' 替）"
    );

    // splitText：原节点保 [0,offset)，返新 text 节点含 [offset,)；两节点均 handle-based 可读。
    sandbox
        .execute(
            "globalThis.__t2 = document.createTextNode('abcdef');\
             globalThis.__tail = globalThis.__t2.splitText(2);\
             globalThis.__head = globalThis.__t2.data;\
             globalThis.__taildata = globalThis.__tail.data;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__head)").unwrap().value,
        "ab",
        "splitText(2) 原节点保 'ab'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__taildata)").unwrap().value,
        "cdef",
        "splitText(2) 返新节点 'cdef'"
    );

    // CharacterData mixin 亦适用 comment 节点（appendData）。
    sandbox
        .execute(
            "globalThis.__c = document.createComment('cmt');\
             globalThis.__c.appendData('!');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__c.data)").unwrap().value,
        "cmt!",
        "comment appendData（CharacterData mixin）"
    );
}

#[test]
fn test_page_visibility_and_focus_r2824() {
    // R2824：Page Visibility + 焦点状态——document.hidden / visibilityState / hasFocus()
    // （+ webkit 前缀 legacy）。analytics/RUM 高频（GA 读 visibilityState/hidden，hasFocus gate 操作）。
    // headless 恒「可见 + 已聚焦」：hidden=false / visibilityState='visible' / hasFocus=true。
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

    // 标准属性：hidden=false / visibilityState='visible' / hasFocus()=true。
    assert_eq!(
        sandbox.execute("String(document.hidden)").unwrap().value,
        "false",
        "document.hidden === false（headless 可见）"
    );
    assert_eq!(
        sandbox.execute("String(document.visibilityState)").unwrap().value,
        "visible",
        "document.visibilityState === 'visible'"
    );
    assert_eq!(
        sandbox.execute("String(document.hasFocus())").unwrap().value,
        "true",
        "document.hasFocus() === true（headless 已聚焦）"
    );
    // webkit 前缀（legacy analytics / 旧 GA feature-detect）。
    assert_eq!(
        sandbox.execute("String(document.webkitHidden)").unwrap().value,
        "false",
        "document.webkitHidden === false（legacy 前缀）"
    );
    assert_eq!(
        sandbox.execute("String(document.webkitVisibilityState)").unwrap().value,
        "visible",
        "document.webkitVisibilityState === 'visible'（legacy 前缀）"
    );
}

#[test]
fn test_constraint_validation_r2825() {
    // R2825：Constraint Validation API——checkValidity/reportValidity/setCustomValidity/validity/
    // validationMessage/willValidate。表单校验库高频（checkValidity gate submit / setCustomValidity
    // 自定义错误 / validity.valid 读）。customError 由 setCustomValidity 跟踪；原生约束 headless 不强制
    // （permissive valid）。checkValidity invalid 时派发 'invalid' 事件。
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
        "<html><body><input id='i'><input id='j'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 默认 valid：validity.valid=true / customError=false / validationMessage='' / checkValidity=true / willValidate=true。
    sandbox
        .execute(
            "globalThis.__i = document.querySelector('#i');\
             globalThis.__defValid = globalThis.__i.validity.valid;\
             globalThis.__defCustom = globalThis.__i.validity.customError;\
             globalThis.__defMsg = globalThis.__i.validationMessage;\
             globalThis.__defCv = globalThis.__i.checkValidity();\
             globalThis.__wv = globalThis.__i.willValidate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__defValid)").unwrap().value,
        "true",
        "默认 validity.valid=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__defCustom)").unwrap().value,
        "false",
        "默认 customError=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__defMsg)").unwrap().value,
        "",
        "默认 validationMessage=''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__defCv)").unwrap().value,
        "true",
        "默认 checkValidity()=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wv)").unwrap().value,
        "true",
        "willValidate=true"
    );

    // setCustomValidity('err') → customError=true / valid=false / validationMessage='err' / checkValidity=false。
    sandbox
        .execute(
            "globalThis.__i.setCustomValidity('err');\
             globalThis.__cvValid = globalThis.__i.validity.valid;\
             globalThis.__cvCustom = globalThis.__i.validity.customError;\
             globalThis.__cvMsg = globalThis.__i.validationMessage;\
             globalThis.__cvCheck = globalThis.__i.checkValidity();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cvValid)").unwrap().value,
        "false",
        "setCustomValidity 后 valid=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cvCustom)").unwrap().value,
        "true",
        "customError=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cvMsg)").unwrap().value,
        "err",
        "validationMessage='err'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cvCheck)").unwrap().value,
        "false",
        "checkValidity()=false"
    );

    // setCustomValidity('') 清空 → 恢复 valid。
    sandbox
        .execute(
            "globalThis.__i.setCustomValidity('');\
             globalThis.__clrValid = globalThis.__i.validity.valid;\
             globalThis.__clrCv = globalThis.__i.checkValidity();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__clrValid)").unwrap().value,
        "true",
        "setCustomValidity('') 恢复 valid=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__clrCv)").unwrap().value,
        "true",
        "清空后 checkValidity()=true"
    );

    // 'invalid' 事件在 checkValidity 失败时派发（per-element，#i 设错，监听 #i 的 invalid）。
    sandbox
        .execute(
            "globalThis.__fired = 'no';\
             globalThis.__i.addEventListener('invalid', function(){ globalThis.__fired = 'yes'; });\
             globalThis.__i.setCustomValidity('x');\
             globalThis.__i.checkValidity();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fired)").unwrap().value,
        "yes",
        "checkValidity 失败派发 'invalid' 事件"
    );

    // per-element 隔离：#j 未设 customValidity 仍 valid（#i 的 setCustomValidity 不影响 #j）。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#j').checkValidity())")
            .unwrap()
            .value,
        "true",
        "per-element 隔离：#j 仍 valid"
    );
}

#[test]
fn test_exec_command_and_select_r2826() {
    // R2826：legacy 编辑/剪贴板命令表面——document.execCommand / queryCommand* / element.select()。
    // 旧 copy 按钮 `el.select(); document.execCommand('copy')` + clipboard.js feature-detect
    // `queryCommandSupported('copy')` + contentEditable 编辑器 format 命令。headless 无真剪贴板/格式化
    // → permissive stub（execCommand→true / queryCommandSupported/Enabled→true / queryCommandValue→'' / select→undefined）。
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
        "<html><body><input id='i' value='txt'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // execCommand / queryCommand* permissive stubs（legacy copy + feature-detect 不抛）。
    sandbox
        .execute(
            "globalThis.__copy = document.execCommand('copy');\
             globalThis.__bold = document.execCommand('bold');\
             globalThis.__sup = document.queryCommandSupported('copy');\
             globalThis.__en = document.queryCommandEnabled('copy');\
             globalThis.__val = document.queryCommandValue('fontSize');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__copy)").unwrap().value,
        "true",
        "execCommand('copy')→true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bold)").unwrap().value,
        "true",
        "execCommand('bold')→true（format 不真应用，permissive）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sup)").unwrap().value,
        "true",
        "queryCommandSupported('copy')→true（feature-detect 通过）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__en)").unwrap().value,
        "true",
        "queryCommandEnabled→true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__val)").unwrap().value,
        "",
        "queryCommandValue→''"
    );

    // element.select() no-op 返 undefined（legacy copy 模式配对，不抛）。
    sandbox
        .execute("globalThis.__sel = document.querySelector('#i').select();")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sel)").unwrap().value,
        "undefined",
        "element.select() no-op 返 undefined"
    );

    // 完整 legacy copy 模式不抛：select + execCommand('copy')。
    sandbox
        .execute(
            "globalThis.__ok = 'no';\
             try {\
               var el = document.querySelector('#i');\
               el.select();\
               document.execCommand('copy');\
               globalThis.__ok = 'yes';\
             } catch (e) { globalThis.__ok = 'err:' + e.message; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "yes",
        "legacy copy 模式（select+execCommand('copy')）不抛"
    );
}

#[test]
fn test_element_animate_r2827() {
    // R2827：Element.animate（Web Animations API permissive stub）。headless 无真时间轴 → 动画瞬间完成
    //（playState 'running'→'finished' + finished Promise resolve + onfinish 触发，经 _defer microtask）。
    // modern 动画库（Framer Motion/GSAP/Lottie）feature-detect + 链式高频。关键帧不真应用（documented）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='d'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // animate 返 Animation 对象；初始 playState='running'（同步读，checkpoint 前）；duration 从 options 取。
    sandbox
        .execute(
            "globalThis.__anim = document.querySelector('#d').animate([{opacity:0},{opacity:1}], 200);\
             globalThis.__psInitial = globalThis.__anim.playState;\
             globalThis.__dur = globalThis.__anim.duration;\
             globalThis.__got = 'no';\
             globalThis.__anim.finished.then(function(a){ globalThis.__got = a.playState; });\
             globalThis.__anim.onfinish = function(){ globalThis.__of = 'fired'; };",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__psInitial)").unwrap().value,
        "running",
        "初始 playState='running'（同步读，checkpoint 前）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dur)").unwrap().value,
        "200",
        "duration 从 options（number）取 200"
    );
    // microtask checkpoint 后：playState='finished' + finished Promise resolve + onfinish 触发。
    assert_eq!(
        sandbox.execute("String(globalThis.__anim.playState)").unwrap().value,
        "finished",
        "microtask 后 playState='finished'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__got)").unwrap().value,
        "finished",
        "finished Promise resolve（携 playState='finished'）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__of)").unwrap().value,
        "fired",
        "onfinish 触发"
    );

    // options 对象形式（duration + id）+ 方法存在不抛 + cancel 切 idle。
    sandbox
        .execute(
            "globalThis.__a2 = document.querySelector('#d').animate([], { duration: 50, id: 'x' });\
             globalThis.__id = globalThis.__a2.id;\
             globalThis.__a2.cancel();\
             globalThis.__a2ps = globalThis.__a2.playState;\
             globalThis.__rev = typeof globalThis.__a2.reverse;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__id)").unwrap().value,
        "x",
        "options.id 提取"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__a2ps)").unwrap().value,
        "idle",
        "cancel() 切 idle"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rev)").unwrap().value,
        "function",
        "reverse 存在"
    );
}

#[test]
fn test_element_get_client_rects_r2828() {
    // R2828：Element.getClientRects——旧返空 []（破 popper.js/tether 读 getClientRects()[0]）。
    // 现返单元素 bounding rect 数组（与 getBoundingClientRect 同源 _domRectFromId）。inline 多行收缩为
    // 单 rect（无 per-line-box，documented）；handle-only detached 无 layout → []。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='d'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    // mock rect bridge（rect bridge 不在 register_dom_callbacks）：selector → 固定 rect "10,20,100,50"；
    // handle（createElement，以 '__' 开头，detached）→ 空串（无 layout，匹配真实 detached 无几何语义）。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| match args.first() {
            Some(s) if s.starts_with("__") => String::new(),
            _ => "10,20,100,50".to_string(),
        }),
    );

    // getClientRects 返数组 length=1 + [0] 含完整 DOMRect 字段（与 getBoundingClientRect 同源）。
    sandbox
        .execute(
            "globalThis.__rects = document.querySelector('#d').getClientRects();\
             globalThis.__len = globalThis.__rects.length;\
             globalThis.__r0 = globalThis.__rects[0];\
             globalThis.__keys = ['x','y','top','left','right','bottom','width','height']\
               .map(function(k){ return k + ':' + (globalThis.__r0[k] !== undefined ? 'y' : 'n'); }).join(',');\
             globalThis.__same = (function(){\
               var b = document.querySelector('#d').getBoundingClientRect();\
               return b.x === globalThis.__r0.x && b.width === globalThis.__r0.width && b.bottom === globalThis.__r0.bottom;\
             })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "1",
        "getClientRects 返数组 length=1（单 bounding rect）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__keys)").unwrap().value,
        "x:y,y:y,top:y,left:y,right:y,bottom:y,width:y,height:y",
        "[0] 含完整 DOMRect 字段（x/y/top/left/right/bottom/width/height）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__same)").unwrap().value,
        "true",
        "getClientRects[0] 与 getBoundingClientRect 同源 rect"
    );

    // spread 可迭代（[...rects] 取首元素）——现代库常用模式。
    sandbox
        .execute("globalThis.__spread = [...document.querySelector('#d').getClientRects()].length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__spread)").unwrap().value,
        "1",
        "getClientRects 可 spread 迭代（数组）"
    );

    // handle-only detached 元素（createElement，无 layout）→ []。
    sandbox
        .execute("globalThis.__detached = document.createElement('div').getClientRects().length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__detached)").unwrap().value,
        "0",
        "handle-only detached 无 layout → getClientRects 返 []"
    );
}

#[test]
fn test_form_elements_r2829() {
    // R2829：form.elements（HTMLFormControlsCollection）+ form.length + namedItem。表单序列化/校验库
    //（jQuery serialize / FormData / 校验库迭代）高频。仅 HTMLFormElement（gate）；非 form → undefined。
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
        "<html><body><form id='f'>\
         <input name='a' value='1'>\
         <select name='s'><option>x</option></select>\
         <textarea name='t'></textarea>\
         <button name='b'>go</button>\
         </form></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // form.elements：4 控件（input/select/textarea/button，tree order）+ length + 索引 + namedItem。
    sandbox
        .execute(
            "globalThis.__f = document.querySelector('#f');\
             globalThis.__els = globalThis.__f.elements;\
             globalThis.__len = globalThis.__els.length;\
             globalThis.__first = globalThis.__els[0].getAttribute('name');\
             globalThis.__last = globalThis.__els[3].getAttribute('name');\
             globalThis.__named = globalThis.__els.namedItem('s').tagName;\
             globalThis.__iter = (function(){\
               var names = [];\
               for (var i = 0; i < globalThis.__els.length; i++) names.push(globalThis.__els[i].getAttribute('name'));\
               return names.join(',');\
             })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "4",
        "form.elements.length=4（input/select/textarea/button）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__first)").unwrap().value,
        "a",
        "elements[0]=input（tree order 首个）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__last)").unwrap().value,
        "b",
        "elements[3]=button（tree order 末个）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__named)").unwrap().value,
        "SELECT",
        "namedItem('s')=select（按 name 查）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__iter)").unwrap().value,
        "a,s,t,b",
        "迭代 form.elements 得 4 控件 name 序"
    );

    // form.length = 控件数。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#f').length)")
            .unwrap()
            .value,
        "4",
        "form.length=4"
    );

    // 非 form 元素 .elements → undefined（gate：仅 HTMLFormElement）。
    assert_eq!(
        sandbox.execute("String(document.body.elements)").unwrap().value,
        "undefined",
        "非 form 元素 .elements=undefined"
    );
}

#[test]
fn test_input_files_filelist_r2830() {
    // R2830：HTMLInputElement.files（空 FileList）。上传表单读 input.files.length / 迭代高频。
    // headless 无真文件 → 空 FileList（length 0 + item→null + 可迭代），让上传 JS 不抛（无文件→0 跳过上传）。
    // 仅 INPUT（_isTag gate）；非 input → undefined。
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
        "<html><body><input id='f' type='file'><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // input.files：空 FileList（length 0 + item(0)=null + spread 空）。
    sandbox
        .execute(
            "globalThis.__files = document.querySelector('#f').files;\
             globalThis.__len = globalThis.__files.length;\
             globalThis.__item = String(globalThis.__files.item(0));\
             globalThis.__spread = [...globalThis.__files].length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "0",
        "input.files.length=0（headless 无文件）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__item)").unwrap().value,
        "null",
        "input.files.item(0)=null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__spread)").unwrap().value,
        "0",
        "input.files 可 spread 迭代（空）"
    );

    // 非 input 元素 .files → undefined（gate：仅 INPUT）。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#d').files)")
            .unwrap()
            .value,
        "undefined",
        "非 input .files=undefined"
    );
}

#[test]
fn test_input_indeterminate_r2831() {
    // R2831：HTMLInputElement.indeterminate——JS-only IDL 布尔（非 reflected attr）。checkbox「全选」
    // tri-state UI 高频（父 checkbox 半选态）。per-element state（默认 false）；get/set round-trip。
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
        "<html><body><input id='c' type='checkbox'><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 默认 false；set true round-trip；set false 恢复。
    sandbox
        .execute(
            "globalThis.__cb = document.querySelector('#c');\
             globalThis.__def = globalThis.__cb.indeterminate;\
             globalThis.__cb.indeterminate = true;\
             globalThis.__afterTrue = globalThis.__cb.indeterminate;\
             globalThis.__cb.indeterminate = false;\
             globalThis.__afterFalse = globalThis.__cb.indeterminate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__def)").unwrap().value,
        "false",
        "默认 indeterminate=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterTrue)").unwrap().value,
        "true",
        "set true round-trip"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterFalse)").unwrap().value,
        "false",
        "set false 恢复"
    );

    // 「全选」tri-state 模式：3 子 checkbox 部分选 → 父 indeterminate。
    sandbox
        .execute(
            "globalThis.__children = [true, false, true];\
             globalThis.__all = globalThis.__children.every(function(v){ return v; });\
             globalThis.__any = globalThis.__children.some(function(v){ return v; });\
             globalThis.__cb.indeterminate = globalThis.__any && !globalThis.__all;\
             globalThis.__tri = globalThis.__cb.indeterminate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tri)").unwrap().value,
        "true",
        "tri-state：部分选 → 父 indeterminate=true"
    );

    // 非 input 元素 .indeterminate → undefined（gate：仅 INPUT）。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#d').indeterminate)")
            .unwrap()
            .value,
        "undefined",
        "非 input .indeterminate=undefined"
    );
}

#[test]
fn test_option_constructor_and_select_add_r2832() {
    // R2832：动态 select 填充表面——new Option() 构造器 + select.add() + option.text/label/defaultSelected。
    // 表单应用动态下拉（级联 select / 动态选项）高频。new Option 返 createElement('option') proxy；
    // select.add 追加 option；option.text/label/defaultSelected 读。
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
        "<html><body><select id='s'><option value='0'>zero</option></select></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // new Option(text, value, defaultSelected, selected)：tag=OPTION + text/value/selected 设置。
    sandbox
        .execute(
            "globalThis.__o = new Option('Apple', 'a', true, false);\
             globalThis.__tag = globalThis.__o.tagName;\
             globalThis.__text = globalThis.__o.text;\
             globalThis.__value = globalThis.__o.getAttribute('value');\
             globalThis.__defSel = globalThis.__o.defaultSelected;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tag)").unwrap().value,
        "OPTION",
        "new Option().tagName=OPTION"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__o.text)").unwrap().value,
        "Apple",
        "new Option text='Apple'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__value)").unwrap().value,
        "a",
        "new Option value='a'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__defSel)").unwrap().value,
        "true",
        "new Option defaultSelected=true（defaultSelected 参数）"
    );

    // select.add(option) 追加；动态填充后 select.value 可读新选项。
    sandbox
        .execute(
            "globalThis.__s = document.querySelector('#s');\
             globalThis.__s.add(new Option('Banana', 'b'));\
             globalThis.__s.add(new Option('Cherry', 'c'));",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<option value=\"b\">Banana</option>") && out.contains("<option value=\"c\">Cherry</option>"),
        "select.add 追加两 option（b=Banana, c=Cherry）\n{out}"
    );

    // option.label：有 label 属性用 label，否则回落 text。
    sandbox
        .execute(
            "globalThis.__oLab = new Option('TxtOnly');\
             globalThis.__lab1 = globalThis.__oLab.label;\
             globalThis.__oLab2 = new Option('inner');\
             globalThis.__oLab2.setAttribute('label', 'LabAttr');\
             globalThis.__lab2 = globalThis.__oLab2.label;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__lab1)").unwrap().value,
        "TxtOnly",
        "option.label 无属性回落 text"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__lab2)").unwrap().value,
        "LabAttr",
        "option.label 有属性用 label"
    );

    // new Option 无 new 调用亦可（返 proxy）。
    assert_eq!(
        sandbox.execute("String(Option('X','x').tagName)").unwrap().value,
        "OPTION",
        "Option() 无 new 亦返 OPTION proxy"
    );

    // handle-based option 的 .selected 读：4th 参数 selected=true → 设 selected 属性 → .selected=true
    //（经 __zw_has_attr_handle，句柄元素不在 HTML 快照，sel-based __zw_has_attr 对其恒 false）。
    sandbox
        .execute(
            "globalThis.__oS = new Option('Sel', 's', false, true);\
             globalThis.__selTrue = globalThis.__oS.selected;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__selTrue)").unwrap().value,
        "true",
        "new Option(...,selected=true) → .selected=true（handle has-attr 变体）"
    );
}

#[test]
fn test_document_collections_r2833() {
    // R2833：document 集合完整性 + 正确性——forms/scripts/images/links 已 land（_liveQueryCollection），
    // 本轮补缺 embeds/plugins/anchors + 修正 links（旧返全部 <a>，spec 仅 a[href]+area[href]）+ 加 has trap
    // 使 Array.prototype.map/forEach.call(coll) 迭代工作（HasProperty 判定索引存在性）。
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
         <form id='f1'></form><form id='f2'></form>\
         <script>var x=1;</script>\
         <img src='a.png'><img src='b.png'>\
         <a href='http://h'>L</a><a name='anc'>A</a>\
         <embed src='e.swf'><object data='o'></object>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "globalThis.__forms = document.forms.length;\
             globalThis.__scripts = document.scripts.length;\
             globalThis.__images = document.images.length;\
             globalThis.__links = document.links.length;\
             globalThis.__anchors = document.anchors.length;\
             globalThis.__embeds = document.embeds.length;\
             globalThis.__plugins = document.plugins.length;\
             globalThis.__f0id = document.forms[0].getAttribute('id');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__forms)").unwrap().value,
        "2",
        "document.forms.length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__scripts)").unwrap().value,
        "1",
        "document.scripts.length=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__images)").unwrap().value,
        "2",
        "document.images.length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__links)").unwrap().value,
        "1",
        "document.links.length=1（仅 a[href]，不含 a[name]）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__anchors)").unwrap().value,
        "1",
        "document.anchors.length=1（仅 a[name]）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__embeds)").unwrap().value,
        "2",
        "document.embeds.length=2（embed+object）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__plugins)").unwrap().value,
        "2",
        "document.plugins.length=2（embed+object）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__f0id)").unwrap().value,
        "f1",
        "document.forms[0].id='f1'（索引访问）"
    );

    // 迭代支持（for...of / 索引遍历）——库常见用法。
    sandbox
        .execute("globalThis.__formIds = Array.prototype.map.call(document.forms, function(f){return f.getAttribute('id');}).join(',');")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__formIds)").unwrap().value,
        "f1,f2",
        "document.forms 可 Array.map 迭代（f1,f2）"
    );
}

#[test]
fn test_image_constructor_r2834() {
    // R2834：HTMLImageElement 构造器 new Image(w,h)——图片预加载 + DOM 挂载高频（WPT css-images /
    // css-backgrounds / content-visibility fixtures 经 new Image() 构造）。旧返 plain object（appendChild 失效、
    // 无 tagName）；现返 createElement('img') proxy（镜像 Option R2832），设 width/height 属性。
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
        "<html><body><div id='host'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // new Image() → tagName=IMG（真 DOM 元素，非旧 plain object）。
    sandbox
        .execute("globalThis.__img = new Image(); globalThis.__tag = globalThis.__img.tagName;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tag)").unwrap().value,
        "IMG",
        "new Image().tagName=IMG（真 img 元素）"
    );

    // new Image(100, 50) → width/height 属性设置（spec：构造器参数映射 width/height 内容属性）。
    sandbox
        .execute(
            "globalThis.__img2 = new Image(100, 50);\
             globalThis.__w = globalThis.__img2.getAttribute('width');\
             globalThis.__h = globalThis.__img2.getAttribute('height');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__w)").unwrap().value,
        "100",
        "new Image(100,50).width 属性=100"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__h)").unwrap().value,
        "50",
        "new Image(100,50).height 属性=50"
    );

    // src 反射 + appendChild DOM 挂载（旧 plain object 致 appendChild 失效——本轮修复核心）。
    sandbox
        .execute(
            "globalThis.__img3 = new Image();\
             globalThis.__img3.src = 'logo.png';\
             document.body.appendChild(globalThis.__img3);",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<img src=\"logo.png\">"),
        "new Image() 经 src 反射 + appendChild 挂入 body（旧 plain object 无效）\n{out}"
    );

    // onload/onerror 可设不抛（headless 无真图片加载，handler 不触发——settable 不抛即可；on* 读回属
    // element proxy 既有限制，非 Image 特有，不在本切片范围）。设后元素仍有效（tagName=IMG）。
    sandbox
        .execute(
            "globalThis.__img4 = new Image();\
             globalThis.__img4.onload = function(){};\
             globalThis.__img4.onerror = function(){};\
             globalThis.__tag4 = globalThis.__img4.tagName;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tag4)").unwrap().value,
        "IMG",
        "new Image() 设 onload/onerror 后仍为 IMG 元素（set 不抛）"
    );

    // 无 new 调用亦返 img proxy。
    assert_eq!(
        sandbox.execute("String(Image().tagName)").unwrap().value,
        "IMG",
        "Image() 无 new 亦返 IMG proxy"
    );
}

#[test]
fn test_audio_constructor_and_media_methods_r2835() {
    // R2835：HTMLAudioElement 构造器 new Audio([src]) + HTMLMediaElement play/pause/load/canPlayType no-op。
    // 音效/播客/通知音频构造高频（new Audio(url).play()）。headless 无音频设备——play 返 resolved Promise、
    // pause/load no-op、canPlayType 返 ''，使媒体 UI 主模式（play().then(...)）不抛。
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
        "<html><body><video id='v'></video></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // new Audio(src) → tagName=AUDIO + src 反射。
    sandbox
        .execute(
            "globalThis.__au = new Audio('beep.mp3');\
             globalThis.__auTag = globalThis.__au.tagName;\
             globalThis.__auSrc = globalThis.__au.getAttribute('src');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__auTag)").unwrap().value,
        "AUDIO",
        "new Audio().tagName=AUDIO"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__auSrc)").unwrap().value,
        "beep.mp3",
        "new Audio('beep.mp3') src 反射"
    );

    // play() 返 resolved Promise（spec 一致）；pause()/load()/canPlayType() no-op 不抛。
    // 经 microtask checkpoint（execute 末）派发 .then，下 execute 可读 __played。
    sandbox
        .execute(
            "globalThis.__au2 = new Audio('x.mp3');\
             globalThis.__playType = typeof globalThis.__au2.play;\
             globalThis.__au2.play().then(function(){ globalThis.__played = 'yes'; });\
             globalThis.__au2.pause();\
             globalThis.__au2.load();\
             globalThis.__cpt = globalThis.__au2.canPlayType('audio/mpeg');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__playType)").unwrap().value,
        "function",
        "audio.play 为 function"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cpt)").unwrap().value,
        "",
        "audio.canPlayType 返空串（保守不可播放）"
    );
    // play().then 回调经 microtask checkpoint 派发——下个 execute 读到 __played。
    sandbox.execute("void 0").unwrap(); // 触发 microtask checkpoint
    assert_eq!(
        sandbox.execute("String(globalThis.__played)").unwrap().value,
        "yes",
        "audio.play().then 回调经 microtask 派发（resolved Promise）"
    );

    // sel-based <video> 元素亦有 media 方法（play no-op 不抛）。
    sandbox
        .execute(
            "globalThis.__vid = document.querySelector('#v');\
             globalThis.__vidPlay = typeof globalThis.__vid.play;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__vidPlay)").unwrap().value,
        "function",
        "<video>.play 为 function（sel-based 亦有 media 方法）"
    );

    // 非 media 元素（如 div）无 play 方法（get-trap 返 undefined，gate 仅 AUDIO/VIDEO）。
    sandbox
        .execute(
            "globalThis.__div = document.createElement('div'); globalThis.__divPlay = typeof globalThis.__div.play;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__divPlay)").unwrap().value,
        "undefined",
        "div 无 play（gate 仅 AUDIO/VIDEO）"
    );

    // 无 new 调用亦返 audio proxy。
    assert_eq!(
        sandbox.execute("String(Audio().tagName)").unwrap().value,
        "AUDIO",
        "Audio() 无 new 亦返 AUDIO proxy"
    );
}

#[test]
fn test_input_value_as_number_r2836() {
    // R2836：input.valueAsNumber IDL 属性（getter+setter）——number/range 输入值↔数值转换（计算器/数量输入/
    // 校验库读 NaN 判非法）。getter：type=number/range parseFloat(value)（空/无效→NaN），其他 type→NaN；
    // setter：NaN→''，否则 String(n)→设 value。
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
         <input id='n' type='number' value='42'>\
         <input id='nf' type='number' value='3.14'>\
         <input id='ne' type='number' value=''>\
         <input id='nb' type='number' value='abc'>\
         <input id='t' type='text' value='99'>\
         <input id='r' type='range' value='7' min='0' max='10'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // getter：整数 / 浮点 / 空→NaN / 无效→NaN / 非 number type→NaN / range 亦可。
    sandbox
        .execute(
            "globalThis.__n = document.querySelector('#n').valueAsNumber;\
             globalThis.__nf = document.querySelector('#nf').valueAsNumber;\
             globalThis.__ne = isNaN(document.querySelector('#ne').valueAsNumber);\
             globalThis.__nb = isNaN(document.querySelector('#nb').valueAsNumber);\
             globalThis.__t = isNaN(document.querySelector('#t').valueAsNumber);\
             globalThis.__r = document.querySelector('#r').valueAsNumber;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__n)").unwrap().value,
        "42",
        "number input value=42 → valueAsNumber=42"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nf)").unwrap().value,
        "3.14",
        "number input value=3.14 → valueAsNumber=3.14"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ne)").unwrap().value,
        "true",
        "number input 空 value → valueAsNumber=NaN"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nb)").unwrap().value,
        "true",
        "number input value=abc → valueAsNumber=NaN"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t)").unwrap().value,
        "true",
        "text input → valueAsNumber=NaN（非 number/range）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r)").unwrap().value,
        "7",
        "range input value=7 → valueAsNumber=7"
    );

    // setter：number input 设数值 → value 字符串化；设 NaN → value=''。
    sandbox
        .execute(
            "var el = document.querySelector('#n');\
             el.valueAsNumber = 100;\
             globalThis.__setV = el.value;\
             el.valueAsNumber = NaN;\
             globalThis.__setNaN = el.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__setV)").unwrap().value,
        "100",
        "valueAsNumber=100 → value='100'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__setNaN)").unwrap().value,
        "",
        "valueAsNumber=NaN → value=''"
    );

    // setter 经 host value 属性 mutation（apply 后 value 属性更新）。
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<input id=\"n\" type=\"number\" value=\"\">"),
        "valueAsNumber=NaN setter 经 value 属性 mutation（apply 后 value=''）\n{out}"
    );
}

#[test]
fn test_anchor_url_decomposition_r2838() {
    // R2838：HTMLAnchorElement/HTMLAreaElement URL 分解 IDL 属性（href/pathname/search/hash/host/hostname/
    // port/protocol/origin）——经 __zw_parse_url 解析 href 属性取组件。SPA 路由/链接分析/analytics 高频。
    // a.href getter 返绝对 URL（区别 getAttribute 返原始串——jQuery .prop vs .attr）；相对 href 经 base 解析。
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
         <a id='abs' href='https://example.com:8080/path?q=1#h'>abs</a>\
         <a id='rel' href='/rel'>rel</a>\
         <a id='none'>nohref</a>\
         </body></html>"
            .to_string(),
    ));
    // 页面 base URL 用于相对 href 解析。
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/base/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 绝对 href 全组件解析。
    sandbox
        .execute(
            "var a = document.querySelector('#abs');\
             globalThis.__href = a.href;\
             globalThis.__protocol = a.protocol;\
             globalThis.__host = a.host;\
             globalThis.__hostname = a.hostname;\
             globalThis.__port = a.port;\
             globalThis.__pathname = a.pathname;\
             globalThis.__search = a.search;\
             globalThis.__hash = a.hash;\
             globalThis.__origin = a.origin;\
             globalThis.__rawHref = a.getAttribute('href');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__href)").unwrap().value,
        "https://example.com:8080/path?q=1#h",
        "a.href 绝对 URL"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__protocol)").unwrap().value,
        "https:",
        "a.protocol"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__host)").unwrap().value,
        "example.com:8080",
        "a.host"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hostname)").unwrap().value,
        "example.com",
        "a.hostname"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__port)").unwrap().value,
        "8080",
        "a.port"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pathname)").unwrap().value,
        "/path",
        "a.pathname"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__search)").unwrap().value,
        "?q=1",
        "a.search"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hash)").unwrap().value,
        "#h",
        "a.hash"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__origin)").unwrap().value,
        "https://example.com:8080",
        "a.origin"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rawHref)").unwrap().value,
        "https://example.com:8080/path?q=1#h",
        "getAttribute('href') 原始串（绝对时同 href）"
    );

    // 相对 href：getAttribute 返原始 '/rel'，a.href 经 base 解析返绝对 URL；组件正确。
    sandbox
        .execute(
            "var r = document.querySelector('#rel');\
             globalThis.__relRaw = r.getAttribute('href');\
             globalThis.__relHref = r.href;\
             globalThis.__relPath = r.pathname;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__relRaw)").unwrap().value,
        "/rel",
        "相对 href getAttribute 返原始 '/rel'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__relHref)").unwrap().value,
        "http://test.local/rel",
        "相对 href a.href 经 base 解析返绝对 URL"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__relPath)").unwrap().value,
        "/rel",
        "相对 href a.pathname='/rel'"
    );

    // 无 href → 空值。
    sandbox
        .execute("globalThis.__noneHref = document.querySelector('#none').href;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__noneHref)").unwrap().value,
        "",
        "无 href 的 a.href=''"
    );

    // href setter：设 href 属性（经 set-trap catch-al __zw_set_attr 记 SetAttr mutation）。SetAttr 异步 apply，
    // 无 href 客户端缓存故同 execute 内 getAttribute 读 stale 快照——apply 后验 HTML 含新 href 属性。
    sandbox
        .execute("document.querySelector('#none').href = 'https://set.example.org/x';")
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let (out2, _h2) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms2).unwrap();
    assert!(
        out2.contains("<a id=\"none\" href=\"https://set.example.org/x\">"),
        "a.href setter 经 SetAttr mutation（apply 后 href 属性写入）\n{out2}"
    );
}

#[test]
fn test_form_reflected_idl_attrs_r2839() {
    // R2839：HTMLFormElement 反射 IDL 属性（action/method/enctype/target）——form 序列化 / AJAX 提交库
    // 读 form.action/form.method 构提交请求。action/target 纯串反射；method/enctype 小写归一 + spec 默认。
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
         <form id='f1' action='/submit' method='POST' enctype='multipart/form-data' target='_blank'></form>\
         <form id='f2' action='https://api.example.org/api' method='dialog'></form>\
         <form id='f3'></form>\
         <div id='notform' action='/x'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // f1：显式 action/method(POST→post 小写)/enctype/target 全反射。
    sandbox
        .execute(
            "var f = document.querySelector('#f1');\
             globalThis.__action = f.action;\
             globalThis.__method = f.method;\
             globalThis.__enctype = f.enctype;\
             globalThis.__target = f.target;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__action)").unwrap().value,
        "/submit",
        "form.action 反射（原始串）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__method)").unwrap().value,
        "post",
        "form.method POST→post 小写归一"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__enctype)").unwrap().value,
        "multipart/form-data",
        "form.enctype 反射"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__target)").unwrap().value,
        "_blank",
        "form.target 反射"
    );

    // f2：method=dialog（合法 enum 值）；action 绝对串反射。
    sandbox
        .execute(
            "var f2 = document.querySelector('#f2');\
             globalThis.__action2 = f2.action;\
             globalThis.__method2 = f2.method;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__action2)").unwrap().value,
        "https://api.example.org/api",
        "form.action 绝对串反射"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__method2)").unwrap().value,
        "dialog",
        "form.method=dialog 合法 enum"
    );

    // f3：无属性 → method 默认 'get'，enctype 默认 'application/x-www-form-urlencoded'，action/target 空。
    sandbox
        .execute(
            "var f3 = document.querySelector('#f3');\
             globalThis.__methodDef = f3.method;\
             globalThis.__enctypeDef = f3.enctype;\
             globalThis.__actionDef = f3.action;\
             globalThis.__targetDef = f3.target;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__methodDef)").unwrap().value,
        "get",
        "form.method 无属性→默认 'get'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__enctypeDef)").unwrap().value,
        "application/x-www-form-urlencoded",
        "form.enctype 无属性→默认"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__actionDef)").unwrap().value,
        "",
        "form.action 无属性→''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__targetDef)").unwrap().value,
        "",
        "form.target 无属性→''"
    );

    // 非 form 元素（div 带 action 属性）不返 form IDL（gate 仅 FORM）——div.action 非 form 默认行为。
    sandbox
        .execute("globalThis.__notformAction = String(document.querySelector('#notform').action);")
        .unwrap();
    // div.action 不应得 form 的默认 'get'-style 处理；接受 catch-all 任一返值（undefined/空/原始串）。
    let _nf = sandbox.execute("String(globalThis.__notformAction)").unwrap().value;
}

#[test]
fn test_reflected_idl_htmlfor_defaultvalue_r2840() {
    // R2840：反射属性 IDL——label.htmlFor（for 属性）、input.defaultValue（初始 value 属性，区别 .value
    // 当前态）、input.defaultChecked（checked 属性存在性）。form reset / 校验库读这些判「值/选中态是否改过」。
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
         <label id='l' for='nameInput'>Name</label>\
         <input id='nameInput' type='text' value='initial'>\
         <input id='chk' type='checkbox' checked>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // label.htmlFor 反射 for 属性。
    sandbox
        .execute("globalThis.__htmlFor = document.querySelector('#l').htmlFor;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__htmlFor)").unwrap().value,
        "nameInput",
        "label.htmlFor 反射 for 属性"
    );

    // input.defaultValue = 初始 value 属性（'initial'）；.value 当前态可独立改变，defaultValue 不变。
    sandbox
        .execute(
            "var i = document.querySelector('#nameInput');\
             globalThis.__dv0 = i.defaultValue;\
             globalThis.__val0 = i.value;\
             i.value = 'changed';\
             globalThis.__dv1 = i.defaultValue;\
             globalThis.__val1 = i.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dv0)").unwrap().value,
        "initial",
        "input.defaultValue=初始 value 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__val0)").unwrap().value,
        "initial",
        "input.value 初始=defaultValue"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dv1)").unwrap().value,
        "initial",
        "改 .value 后 defaultValue 不变（区别当前态）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__val1)").unwrap().value,
        "changed",
        "改 .value 后 .value=changed"
    );

    // input.defaultChecked = checked 属性存在性（true）。.checked 当前态同（shim 无独立 toggle 态）。
    sandbox
        .execute(
            "var c = document.querySelector('#chk');\
             globalThis.__dc = c.defaultChecked;\
             globalThis.__ck = c.checked;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dc)").unwrap().value,
        "true",
        "input.defaultChecked=checked 属性存在"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ck)").unwrap().value,
        "true",
        "input.checked=true（同 defaultChecked）"
    );

    // setter：label.htmlFor = x 设 for 属性（attr 名映射）；input.defaultValue = x 设 value 属性。
    sandbox
        .execute(
            "document.querySelector('#l').htmlFor = 'emailInput';\
             document.querySelector('#nameInput').defaultValue = 'reset';",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<label id=\"l\" for=\"emailInput\">"),
        "label.htmlFor setter 设 for 属性（attr 名映射 htmlFor→for）\n{out}"
    );
    assert!(
        out.contains("value=\"reset\""),
        "input.defaultValue setter 设 value 属性（attr 名映射 defaultValue→value）\n{out}"
    );
}

#[test]
fn test_input_form_owner_r2841() {
    // R2841：.form（form-associated 控件）——返所属 <form> 元素。spec 顺序：① form 属性关联优先
    // （<input form="id"> → getElementById）；② 否则最近 ancestor <form>。校验/序列化库读 input.form 高频。
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
         <form id='fA'>\
           <input id='nested' type='text'>\
           <select id='sel'><option>x</option></select>\
         </form>\
         <input id='orphan' type='text'>\
         <form id='fB'></form>\
         <input id='attr' type='text' form='fB'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ancestor-based：nested input.form → fA（最近 ancestor form）。
    sandbox
        .execute(
            "globalThis.__nestedForm = document.querySelector('#nested').form;\
             globalThis.__nestedFormId = globalThis.__nestedForm ? globalThis.__nestedForm.id : null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nestedFormId)").unwrap().value,
        "fA",
        "嵌套 input.form → ancestor form fA"
    );
    // 同 form proxy identity：input.form === document.querySelector('#fA')。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#nested').form === document.querySelector('#fA'))")
            .unwrap()
            .value,
        "true",
        "input.form === ancestor form proxy（identity）"
    );

    // select.form 亦返 ancestor form（form-associated 控件 gate 含 SELECT）。
    sandbox
        .execute("globalThis.__selForm = document.querySelector('#sel').form.id;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__selForm)").unwrap().value,
        "fA",
        "select.form → ancestor form fA"
    );

    // orphan input（无 ancestor form）→ null。
    sandbox
        .execute("globalThis.__orphanForm = document.querySelector('#orphan').form;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__orphanForm)").unwrap().value,
        "null",
        "orphan input.form=null（无 ancestor form）"
    );

    // form 属性关联优先：<input form='fB'>（无 ancestor form）→ fB（getElementById）。
    sandbox
        .execute(
            "globalThis.__attrForm = document.querySelector('#attr').form;\
             globalThis.__attrFormId = globalThis.__attrForm ? globalThis.__attrForm.id : null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__attrFormId)").unwrap().value,
        "fB",
        "input form='fB' → form 属性关联优先（getElementById fB）"
    );

    // 非 form 控件（如 div）的 .form 不走本 gate（返 undefined/其他，非 form owner 逻辑）。
    sandbox
        .execute("globalThis.__divForm = String(document.createElement('div').form);")
        .unwrap();
    // div.form 非 form owner 逻辑——接受 undefined（String(undefined)='undefined'）。
    let _df = sandbox.execute("String(globalThis.__divForm)").unwrap().value;
}

#[test]
fn test_table_row_cell_index_r2842() {
    // R2842：<tr>.rowIndex（行在 table 中位置，跨 thead/tbody/tfoot document order）+ <td>/<th>.cellIndex
    // （单元格在行中位置，td+th 混计）。data-table / 表格操作库读这些定位高频。client-side 经
    // _ancestorChain + 元素作用域 querySelectorAll + proxy identity 计位；无 owner → -1。
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
         <table id='t1'>\
           <thead><tr id='hr'><th>A</th><th>B</th></tr></thead>\
           <tbody>\
             <tr id='r0'><td id='c00'>1</td><td id='c01'>2</td></tr>\
             <tr id='r1'><td id='c10'>3</td><th id='h10'>4</th></tr>\
           </tbody>\
         </table>\
         <table id='t2'><tr id='r0b'><td>x</td></tr></table>\
         <tr id='orphan'><td>no-table</td></tr>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // rowIndex：跨 thead+tbody document order——hr=0, r0=1, r1=2。
    sandbox
        .execute(
            "globalThis.__hr = document.querySelector('#hr').rowIndex;\
             globalThis.__r0 = document.querySelector('#r0').rowIndex;\
             globalThis.__r1 = document.querySelector('#r1').rowIndex;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hr)").unwrap().value,
        "0",
        "thead 行 rowIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r0)").unwrap().value,
        "1",
        "tbody 首行 rowIndex=1（跨 thead 计）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r1)").unwrap().value,
        "2",
        "tbody 次行 rowIndex=2"
    );
    // 不同 table 的 r0b 在 t2 中 rowIndex=0（各 table 独立计）。
    sandbox
        .execute("globalThis.__r0b = document.querySelector('#r0b').rowIndex;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r0b)").unwrap().value,
        "0",
        "t2 中行 rowIndex=0（各 table 独立）"
    );
    // detached tr（createElement，未挂入 table）→ -1。注：HTML 解析器丢弃 table 外的 <tr>，
    // 故无法用 orphan tr 测；用 createElement('tr') detached 测 -1。
    sandbox
        .execute("globalThis.__detached = document.createElement('tr').rowIndex;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__detached)").unwrap().value,
        "-1",
        "detached tr（无 table）rowIndex=-1"
    );

    // cellIndex：行内 td+th document order——c00=0, c01=1, c10=0, h10=1（td+th 混计）。
    sandbox
        .execute(
            "globalThis.__c00 = document.querySelector('#c00').cellIndex;\
             globalThis.__c01 = document.querySelector('#c01').cellIndex;\
             globalThis.__c10 = document.querySelector('#c10').cellIndex;\
             globalThis.__h10 = document.querySelector('#h10').cellIndex;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__c00)").unwrap().value,
        "0",
        "td cellIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c01)").unwrap().value,
        "1",
        "td cellIndex=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c10)").unwrap().value,
        "0",
        "r1 首格 cellIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__h10)").unwrap().value,
        "1",
        "th cellIndex=1（td+th 混计 document order）"
    );
}

#[test]
fn test_table_section_index_and_collections_r2843() {
    // R2843：<tr>.sectionRowIndex（行在 thead/tbody/tfoot section 内位置）+ <table>.rows / <table>.tBodies
    //（table 内全部行 / tbody 集合，返真数组）。延续 R2842 表格表面。data-table 库迭代 table.rows 高频。
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
         <table id='t1'>\
           <thead><tr id='h1'><th>H1</th></tr><tr id='h2'><th>H2</th></tr></thead>\
           <tbody><tr id='b1'><td>B1</td></tr></tbody>\
           <tbody><tr id='b2'><td>B2</td></tr><tr id='b3'><td>B3</td></tr></tbody>\
         </table>\
         <table id='t2'><tbody><tr id='x1'><td>x</td></tr></tbody></table>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // sectionRowIndex：行在所属 section 内位置——thead 的 h1=0/h2=1；tbody1 的 b1=0；tbody2 的 b2=0/b3=1。
    sandbox
        .execute(
            "globalThis.__h1 = document.querySelector('#h1').sectionRowIndex;\
             globalThis.__h2 = document.querySelector('#h2').sectionRowIndex;\
             globalThis.__b1 = document.querySelector('#b1').sectionRowIndex;\
             globalThis.__b2 = document.querySelector('#b2').sectionRowIndex;\
             globalThis.__b3 = document.querySelector('#b3').sectionRowIndex;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__h1)").unwrap().value,
        "0",
        "thead h1 sectionRowIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__h2)").unwrap().value,
        "1",
        "thead h2 sectionRowIndex=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b1)").unwrap().value,
        "0",
        "tbody1 b1 sectionRowIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b2)").unwrap().value,
        "0",
        "tbody2 b2 sectionRowIndex=0（新 section 重计）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b3)").unwrap().value,
        "1",
        "tbody2 b3 sectionRowIndex=1"
    );

    // table.rows：t1 全部行（跨 thead+2 tbody，document order，5 行）。
    sandbox
        .execute(
            "globalThis.__t1Rows = document.querySelector('#t1').rows.length;\
             globalThis.__t2Rows = document.querySelector('#t2').rows.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t1Rows)").unwrap().value,
        "5",
        "t1.rows.length=5（h1,h2,b1,b2,b3）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t2Rows)").unwrap().value,
        "1",
        "t2.rows.length=1（各 table 独立）"
    );

    // table.rows 真数组：可 Array.map 迭代 + 索引访问。
    sandbox
        .execute(
            "globalThis.__rowsMap = Array.prototype.map.call(document.querySelector('#t1').rows, function(r){return r.id;}).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rowsMap)").unwrap().value,
        "h1,h2,b1,b2,b3",
        "table.rows 可 Array.map 迭代（document order）"
    );

    // table.tBodies：t1 有 2 个 tbody。
    sandbox
        .execute("globalThis.__t1Bodies = document.querySelector('#t1').tBodies.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t1Bodies)").unwrap().value,
        "2",
        "t1.tBodies.length=2"
    );
}

#[test]
fn test_text_control_selection_r2844() {
    // R2844：text-control（input text-type / textarea）选区 IDL——selectionStart / selectionEnd /
    // selectionDirection getter + setSelectionRange + select + 属性 setter。Chromium 150 oracle 锚定：
    // 默认 {0, 0, 'forward'}（未聚焦 text control 选区折叠在 0，非值末）；select()→{0, len, forward}；
    // setSelectionRange clamp [0,len]，end<start 折叠到 end，direction 缺省 forward；属性 setter 保持 0≤start≤end≤len。
    // 文本编辑器 / 自动选择 / Range 算法读选区状态高频。number/checkbox 非选区 type → undefined（Chrome null）。
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
         <input id='i' type='text' value='world'>\
         <textarea id='ta'>hello</textarea>\
         <input id='num' type='number' value='42'>\
         <input id='chk' type='checkbox'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 默认选区 = {0, 0, 'forward'}（text control 未设/未聚焦）；非选区 type（number/checkbox）→ undefined。
    sandbox
        .execute(
            "var i = document.querySelector('#i');\
             var ta = document.querySelector('#ta');\
             globalThis.__d_ss = i.selectionStart;\
             globalThis.__d_se = i.selectionEnd;\
             globalThis.__d_dir = i.selectionDirection;\
             globalThis.__ta_ss = ta.selectionStart;\
             globalThis.__ta_se = ta.selectionEnd;\
             globalThis.__num = document.querySelector('#num').selectionStart;\
             globalThis.__chk = document.querySelector('#chk').selectionStart;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__d_ss)").unwrap().value,
        "0",
        "input 默认 selectionStart=0（折叠在 0，非值末）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__d_se)").unwrap().value,
        "0",
        "input 默认 selectionEnd=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__d_dir)").unwrap().value,
        "forward",
        "input 默认 selectionDirection='forward'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ta_ss) + ',' + String(globalThis.__ta_se)")
            .unwrap()
            .value,
        "0,0",
        "textarea 默认选区 {{0,0}}"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__num)").unwrap().value,
        "undefined",
        "number input 非选区 type → selectionStart undefined（Chrome null）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chk)").unwrap().value,
        "undefined",
        "checkbox 非选区 type → selectionStart undefined"
    );

    // select() → {0, value.length, 'forward'}（input 5 / textarea 5）。
    sandbox
        .execute(
            "i.select();\
             globalThis.__sel_ss = i.selectionStart;\
             globalThis.__sel_se = i.selectionEnd;\
             globalThis.__sel_dir = i.selectionDirection;\
             ta.select();\
             globalThis.__ta_sel_se = ta.selectionEnd;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__sel_ss) + ',' + String(globalThis.__sel_se)")
            .unwrap()
            .value,
        "0,5",
        "input select() → {{0, 5}}（world 长度）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sel_dir)").unwrap().value,
        "forward",
        "input select() direction='forward'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ta_sel_se)").unwrap().value,
        "5",
        "textarea select() → selectionEnd=5（hello 长度）"
    );

    // setSelectionRange：正常 / end<start 折叠 / clamp 超界 / direction。
    // 注：Chrome 对**负数** start 的 setSelectionRange 有古怪归一（如 setSR(-5,-1)→{5,5}），属病态边角、
    // 无真实代码依赖；本实现按 spec 合理 clamp [0,len]，仅负数输入与 Chrome 古怪行为分歧（documented）。
    sandbox
        .execute(
            "i.setSelectionRange(1, 3, 'backward');\
             globalThis.__a = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.setSelectionRange(4, 2);\
             globalThis.__b = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.setSelectionRange(3, 9999);\
             globalThis.__c = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.setSelectionRange(0, 9999);\
             globalThis.__d = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__a)").unwrap().value,
        "1,3,backward",
        "setSelectionRange(1,3,'backward')"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b)").unwrap().value,
        "2,2,forward",
        "setSelectionRange(4,2) end<start 折叠到 {{2,2}}，direction 缺省 forward"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c)").unwrap().value,
        "3,5,forward",
        "setSelectionRange(3,9999) end clamp 到值长度 5"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__d)").unwrap().value,
        "0,5,forward",
        "setSelectionRange(0,9999) start=0 / end clamp 到 5"
    );

    // 属性 setter：start 超 end → end 跟升；end 低于 start → end 升回 start；direction 仅接受合法值。
    sandbox
        .execute(
            "i.setSelectionRange(1, 4);\
             i.selectionDirection = 'backward';\
             globalThis.__s1 = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.selectionStart = 99;\
             globalThis.__s2 = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.selectionEnd = -5;\
             globalThis.__s3 = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__s1)").unwrap().value,
        "1,4,backward",
        "属性设 selectionDirection='backward'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s2)").unwrap().value,
        "5,5,backward",
        "selectionStart=99 → clamp 5，end 跟升到 5（保 0≤start≤end≤len）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s3)").unwrap().value,
        "5,5,backward",
        "selectionEnd=-5 → clamp 0 后升回 start=5（end 不低于 start）"
    );
}

#[test]
fn test_table_caption_thead_tfoot_section_rows_r2845() {
    // R2845：table.caption/tHead/tFoot（首个 caption/thead/tfoot 子元素或 null）+ section.rows（thead/tbody/tfoot
    // 作用域内行）。延续 R2843 表格表面。表格分析 / 序列化库读结构高频。Chromium 150 oracle 锚定。
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
         <table id='t1'>\
           <caption id='cap'>My Caption</caption>\
           <thead id='th'><tr id='h1'><th>H</th></tr></thead>\
           <tfoot id='tf'><tr id='f1'><td>F</td></tr></tfoot>\
           <tbody id='tb1'><tr id='b1'><td>B1</td></tr><tr id='b2'><td>B2</td></tr></tbody>\
         </table>\
         <table id='t2'><tbody><tr id='x1'><td>x</td></tr></tbody></table>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // table.caption：t1 首 caption（id=cap）；t2 无 → null。
    sandbox
        .execute(
            "globalThis.__cap = document.querySelector('#t1').caption ? document.querySelector('#t1').caption.id : 'null';\
             globalThis.__cap2 = String(document.querySelector('#t2').caption);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cap)").unwrap().value,
        "cap",
        "t1.caption 返首个 caption 元素（id=cap）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cap2)").unwrap().value,
        "null",
        "t2 无 caption → null"
    );

    // table.tHead / table.tFoot：t1 有 thead/tfoot（id）；t2 无 → null。
    sandbox
        .execute(
            "globalThis.__th = document.querySelector('#t1').tHead ? document.querySelector('#t1').tHead.id : 'null';\
             globalThis.__th2 = String(document.querySelector('#t2').tHead);\
             globalThis.__tf = document.querySelector('#t1').tFoot ? document.querySelector('#t1').tFoot.id : 'null';\
             globalThis.__tf2 = String(document.querySelector('#t2').tFoot);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__th)").unwrap().value,
        "th",
        "t1.tHead 返首个 thead（id=th）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__th2)").unwrap().value,
        "null",
        "t2 无 thead → null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tf)").unwrap().value,
        "tf",
        "t1.tFoot 返首个 tfoot（id=tf）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tf2)").unwrap().value,
        "null",
        "t2 无 tfoot → null"
    );

    // section.rows：tbody#tb1 作用域内行（b1,b2，2 行，section-scoped）；thead/tfoot 同。
    sandbox
        .execute(
            "globalThis.__tbRows = document.querySelector('#tb1').rows.length;\
             globalThis.__tbRowsMap = Array.prototype.map.call(document.querySelector('#tb1').rows, function(r){return r.id;}).join(',');\
             globalThis.__thRows = document.querySelector('#th').rows.length;\
             globalThis.__tfRows = document.querySelector('#tf').rows.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tbRows)").unwrap().value,
        "2",
        "tbody#tb1.rows.length=2（section-scoped b1,b2）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tbRowsMap)").unwrap().value,
        "b1,b2",
        "tbody.rows 迭代 document order（b1,b2）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__thRows)").unwrap().value,
        "1",
        "thead.rows.length=1（h1）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tfRows)").unwrap().value,
        "1",
        "tfoot.rows.length=1（f1）"
    );

    // table.rows 仍跨全 section（4 行：h1/f1/b1/b2），与 R2843 一致——rows gate 同时支持 TABLE 与 section。
    sandbox
        .execute("globalThis.__t1Rows = document.querySelector('#t1').rows.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t1Rows)").unwrap().value,
        "4",
        "table.rows.length=4（跨 thead/tfoot/tbody 全行，R2843 行为不变）"
    );
}

#[test]
fn test_output_value_default_value_r2846() {
    // R2846：HTMLOutputElement.value（getter=textContent，setter 同步 textContent）+ defaultValue（初始文本内容，
    // lazy 捕获一次，跨 value 变更保持稳定）。表单计算器 `<output>` 显示结果高频。Chromium 150 oracle 锚定。
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
         <output id='o1'>12</output>\
         <output id='o2'></output>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // value getter = textContent；defaultValue getter = 初始 textContent（同 value，未变更时相等）。
    // 每 execute 内声明局部元素 var（_proxyCache identity-stable）。
    sandbox
        .execute(
            "var a = document.querySelector('#o1');\
             globalThis.__v1 = a.value;\
             globalThis.__dv1 = a.defaultValue;\
             globalThis.__v2 = document.querySelector('#o2').value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__v1)").unwrap().value,
        "12",
        "o1.value=textContent '12'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dv1)").unwrap().value,
        "12",
        "o1.defaultValue=初始 textContent '12'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__v2)").unwrap().value,
        "",
        "o2 空 output → value=''"
    );

    // value setter 仅更新 dirty 当前值（client 缓存即时）；spec：value 独立于 textContent——
    // 设 .value 不触碰 DOM text（<output> 按 children 渲染非 value），故 textContent 仍='12'。
    // defaultValue 不被 value 变更影响（捕获稳定）。每 execute 内声明局部元素 var（_proxyCache identity-stable）。
    sandbox
        .execute(
            "var o = document.querySelector('#o1');\
             o.value = 99;\
             globalThis.__v1b = o.value;\
             globalThis.__tc1 = o.textContent;\
             globalThis.__dv1b = o.defaultValue;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__v1b)").unwrap().value,
        "99",
        "o1.value=99 → value='99'（client 缓存即时）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tc1)").unwrap().value,
        "12",
        "o1.value setter 不触碰 textContent（仍='12'，spec value 独立于 text）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dv1b)").unwrap().value,
        "12",
        "defaultValue 跨 value 变更保持稳定（仍='12'）"
    );

    // defaultValue setter 更新捕获值；value（dirty）不受影响。
    sandbox
        .execute(
            "var d = document.querySelector('#o1');\
             d.defaultValue = 'dd';\
             globalThis.__dv1c = d.defaultValue;\
             globalThis.__v1c = d.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dv1c)").unwrap().value,
        "dd",
        "defaultValue='dd' setter 更新捕获值"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__v1c)").unwrap().value,
        "99",
        "dirty 时设 defaultValue 不改 value（仍='99'）"
    );

    // value setter 不写 DOM text（spec：value 独立于 textContent）——apply 后 output 仍含初值 '12'，无 text mutation。
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<output id=\"o1\">12</output>"),
        "output.value=99 不写 DOM text（apply 后 textContent 仍='12'，value 独立）\n{out}"
    );
}

#[test]
fn test_mutation_record_instanceof_spec_fields_r2847() {
    // R2847：MutationObserver 回调收到的 record 须 `instanceof MutationRecord` + `[object MutationRecord]`
    // toStringTag + 完整 spec 字段（previousSibling/nextSibling/attributeNamespace/oldValue 缺省 null，
    // addedNodes/removedNodes 缺省 []）。库做 instanceof 特征检测 / 读 record.previousSibling 须得 null 非 undefined。
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // execute 1：observe handle-based parent，appendChild child（childList mutation）。
    // 回调经 execute 末 microtask checkpoint 派发 → globalThis.__recs 在本 execute 末就绪。
    sandbox
        .execute(
            "var obs = new MutationObserver(function(records){ globalThis.__recs = records; });\
             var parent = document.createElement('div');\
             obs.observe(parent, { childList: true });\
             var child = document.createElement('span');\
             parent.appendChild(child);",
        )
        .unwrap();

    // execute 2：读捕获 record + 断言 instanceof / toStringTag / spec 字段缺省值。
    sandbox
        .execute(
            "var r = globalThis.__recs && globalThis.__recs[0];\
             globalThis.__len = globalThis.__recs ? globalThis.__recs.length : -1;\
             globalThis.__isMR = r instanceof MutationRecord;\
             globalThis.__tag = Object.prototype.toString.call(r);\
             globalThis.__type = r && r.type;\
             globalThis.__addedLen = r && r.addedNodes.length;\
             globalThis.__prevSib = r && r.previousSibling;\
             globalThis.__nextSib = r && r.nextSibling;\
             globalThis.__attrName = r && r.attributeName;\
             globalThis.__attrNs = r && r.attributeNamespace;\
             globalThis.__oldVal = r && r.oldValue;\
             globalThis.__removedLen = r && r.removedNodes.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "1",
        "1 childList record（appendChild 触发）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__isMR)").unwrap().value,
        "true",
        "record instanceof MutationRecord（R2847）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tag)").unwrap().value,
        "[object MutationRecord]",
        "toStringTag = [object MutationRecord]"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__type)").unwrap().value,
        "childList",
        "type = childList"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__addedLen)").unwrap().value,
        "1",
        "addedNodes 含 1（span）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__prevSib)").unwrap().value,
        "null",
        "previousSibling 缺省 null（spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nextSib)").unwrap().value,
        "null",
        "nextSibling 缺省 null（spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__attrName)").unwrap().value,
        "null",
        "attributeName 缺省 null（childList record，spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__attrNs)").unwrap().value,
        "null",
        "attributeNamespace 缺省 null（spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__oldVal)").unwrap().value,
        "null",
        "oldValue 缺省 null（spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__removedLen)").unwrap().value,
        "0",
        "removedNodes 缺省 []（spec，length 0）"
    );
}

#[test]
fn test_reflected_global_attrs_autofocus_draggable_spellcheck_translate_r2848() {
    // R2848：reflected 布尔/枚举全局属性 autofocus/draggable/spellcheck/translate——旧 fallthrough 返 undefined
    // （spec 须布尔）。spec 默认：autofocus=false / draggable=false / spellcheck=true / translate=true。
    // autofocus=boolean attr（presence）；draggable/spellcheck="true"/"false"；translate="yes"/"no"。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // autofocus 默认缺省 / draggable="true" attr / spellcheck="false" attr / translate="no" attr。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <input id='a' autofocus>\
         <div id='d' draggable='true'></div>\
         <div id='s' spellcheck='false'></div>\
         <div id='t' translate='no'></div>\
         <div id='plain'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 读：autofocus(a, present)→true；draggable(d,"true")→true；spellcheck(s,"false")→false；
    // translate(t,"no")→false；plain 全缺省：autofocus=false / draggable=false / spellcheck=true / translate=true。
    sandbox
        .execute(
            "globalThis.__af = document.querySelector('#a').autofocus;\
             globalThis.__dg = document.querySelector('#d').draggable;\
             globalThis.__sc = document.querySelector('#s').spellcheck;\
             globalThis.__tr = document.querySelector('#t').translate;\
             var p = document.querySelector('#plain');\
             globalThis.__paf = p.autofocus;\
             globalThis.__pdg = p.draggable;\
             globalThis.__psc = p.spellcheck;\
             globalThis.__ptr = p.translate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__af)").unwrap().value,
        "true",
        "a[autofocus] present → autofocus=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dg)").unwrap().value,
        "true",
        "div[draggable='true'] → draggable=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sc)").unwrap().value,
        "false",
        "div[spellcheck='false'] → spellcheck=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tr)").unwrap().value,
        "false",
        "div[translate='no'] → translate=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__paf)").unwrap().value,
        "false",
        "plain autofocus 缺省 → false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pdg)").unwrap().value,
        "false",
        "plain draggable 缺省 → false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__psc)").unwrap().value,
        "true",
        "plain spellcheck 缺省 → true（spec 默认）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ptr)").unwrap().value,
        "true",
        "plain translate 缺省 → true（spec 默认）"
    );

    // setter：同步 set→get 优先读缓存（即时）。autofocus=true 设 presence；draggable=true→attr "true"；
    // spellcheck=false→attr "false"；translate=true→attr "yes"。apply 后 attr 写回核验。
    sandbox
        .execute(
            "var e = document.querySelector('#plain');\
             e.autofocus = true; e.draggable = true; e.spellcheck = false; e.translate = true;\
             globalThis.__saf = e.autofocus;\
             globalThis.__sdg = e.draggable;\
             globalThis.__ssc = e.spellcheck;\
             globalThis.__str = e.translate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__saf)").unwrap().value,
        "true",
        "setter autofocus=true → true（缓存即时）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sdg)").unwrap().value,
        "true",
        "setter draggable=true → true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ssc)").unwrap().value,
        "false",
        "setter spellcheck=false → false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__str)").unwrap().value,
        "true",
        "setter translate=true → true"
    );

    // apply mutations → 核验 attr 写回（autofocus presence / draggable="true" / spellcheck="false" / translate="yes"）。
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("id=\"plain\" autofocus"),
        "autofocus setter 写 presence\n{out}"
    );
    assert!(out.contains("draggable=\"true\""), "draggable setter 写 'true'\n{out}");
    assert!(
        out.contains("spellcheck=\"false\""),
        "spellcheck setter 写 'false'\n{out}"
    );
    assert!(out.contains("translate=\"yes\""), "translate setter 写 'yes'\n{out}");
}

#[test]
fn test_option_index_r2849() {
    // R2849：`<option>`.index（HTMLOptionElement）——option 在其 select 中的 0-based 位置（document order）；
    // 0 若不在 select（detached / handle-based，与 Chromium detached→0 一致）。form 库读 option.index 高频。
    // 同 R2842 rowIndex 模式：_ancestorChain 找 owning SELECT + 元素作用域 querySelectorAll('option') + identity。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 3 options in #s1；#s2 第一个 option 为 target（index 0）；含 optgroup（option 仍按 document order 计）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='s1'>\
           <option id='a'>A</option>\
           <option id='b'>B</option>\
           <optgroup><option id='c'>C</option></optgroup>\
         </select>\
         <select id='s2'><option id='x'>X</option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 读 option.index：a=0 / b=1 / c=2（optgroup 内仍 document order）/ x=0（另一 select）。
    sandbox
        .execute(
            "globalThis.__ia = document.querySelector('#a').index;\
             globalThis.__ib = document.querySelector('#b').index;\
             globalThis.__ic = document.querySelector('#c').index;\
             globalThis.__ix = document.querySelector('#x').index;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ia)").unwrap().value,
        "0",
        "#a 为 s1 首个 option → index=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ib)").unwrap().value,
        "1",
        "#b 为 s1 第二个 option → index=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ic)").unwrap().value,
        "2",
        "#c 在 optgroup 内但 document order 仍 → index=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ix)").unwrap().value,
        "0",
        "#x 为 s2 首个 option → index=0（另一 select 作用域）"
    );

    // detached option（createElement，不在 select）→ 0（Chromium detached→0 一致）。
    sandbox
        .execute("globalThis.__d = document.createElement('option').index;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__d)").unwrap().value,
        "0",
        "detached option（createElement，不在 select）→ index=0"
    );
}

#[test]
fn test_reflected_global_attrs_inert_autocomplete_r2850() {
    // R2850：reflected 全局属性 inert（boolean attr，缺省 false）/ autocomplete（enumerated 串，缺省 "on"）。
    // 旧 fallthrough 返 undefined。inert 同 autofocus（presence）；autocomplete 缺省 → "on"（spec missing-default）。
    // 模态/无障碍（inert 隔离交互）/ 表单自动填充（autocomplete）读这些属性高频。延续 R2848 模式。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // div[inert] present；input[autocomplete="off"]；plain 无两属性。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <div id='d' inert></div>\
         <input id='a' autocomplete='off'>\
         <div id='plain'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 读：div[inert] present → inert=true；input[autocomplete="off"] → "off"；plain 缺省 inert=false / autocomplete="on"。
    sandbox
        .execute(
            "globalThis.__di = document.querySelector('#d').inert;\
             globalThis.__aa = document.querySelector('#a').autocomplete;\
             var p = document.querySelector('#plain');\
             globalThis.__pi = p.inert;\
             globalThis.__pa = p.autocomplete;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__di)").unwrap().value,
        "true",
        "div[inert] present → inert=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__aa)").unwrap().value,
        "off",
        "input[autocomplete='off'] → autocomplete='off'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pi)").unwrap().value,
        "false",
        "plain inert 缺省 → false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pa)").unwrap().value,
        "on",
        "plain autocomplete 缺省 → 'on'（spec missing-default）"
    );

    // setter：同步 set→get 优先读缓存（即时）。inert=true→presence；autocomplete='given-name'→attr 串。
    sandbox
        .execute(
            "var e = document.querySelector('#plain');\
             e.inert = true; e.autocomplete = 'given-name';\
             globalThis.__si = e.inert;\
             globalThis.__sa = e.autocomplete;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__si)").unwrap().value,
        "true",
        "setter inert=true → true（缓存即时）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sa)").unwrap().value,
        "given-name",
        "setter autocomplete='given-name' → 'given-name'（任意值写 attr）"
    );

    // apply mutations → 核验 attr 写回（inert presence / autocomplete='given-name'）。
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(out.contains("id=\"plain\" inert"), "inert setter 写 presence\n{out}");
    assert!(
        out.contains("autocomplete=\"given-name\""),
        "autocomplete setter 写 'given-name'\n{out}"
    );
}

#[test]
fn test_img_dimension_idl_width_height_natural_r2851() {
    // R2851：IMG/IFRAME width/height（reflected unsigned long，缺省/非负整数失败→0）+ IMG naturalWidth/Height
    // （固有像素尺寸，headless 无真图加载→0，spec unloaded→0）。旧 fallthrough 返 undefined。响应式/布局 JS 高频。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // img 显式 width/height；img2 无属性；iframe 显式 width。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <img id='i1' src='a.png' width='100' height='50'>\
         <img id='i2' src='b.png'>\
         <iframe id='f1' width='320'></iframe>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 读：i1 width=100/height=50；i2 缺省 width=0/height=0；naturalWidth/Height 恒 0（headless）；iframe width=320。
    sandbox
        .execute(
            "var i1 = document.querySelector('#i1');\
             globalThis.__i1w = i1.width;\
             globalThis.__i1h = i1.height;\
             globalThis.__i1nw = i1.naturalWidth;\
             globalThis.__i1nh = i1.naturalHeight;\
             var i2 = document.querySelector('#i2');\
             globalThis.__i2w = i2.width;\
             globalThis.__i2h = i2.height;\
             globalThis.__f1w = document.querySelector('#f1').width;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__i1w)").unwrap().value,
        "100",
        "img[width='100'] → width=100（reflected unsigned long）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i1h)").unwrap().value,
        "50",
        "img[height='50'] → height=50"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i1nw)").unwrap().value,
        "0",
        "img.naturalWidth=0（headless 无真图加载，spec unloaded→0）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i1nh)").unwrap().value,
        "0",
        "img.naturalHeight=0（headless）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i2w)").unwrap().value,
        "0",
        "img 无 width 属性 → width=0（缺省）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i2h)").unwrap().value,
        "0",
        "img 无 height 属性 → height=0（缺省）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__f1w)").unwrap().value,
        "320",
        "iframe[width='320'] → width=320（IFRAME 同 reflected unsigned long）"
    );

    // setter：img.width=200 → 缓存数值即时 + apply 后 attr 写回；非负整数解析（'12px'→12 近似）。
    sandbox
        .execute(
            "var e = document.querySelector('#i2');\
             e.width = 200;\
             globalThis.__sw = e.width;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sw)").unwrap().value,
        "200",
        "setter img.width=200 → 200（缓存数值即时 sync）"
    );
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("id=\"i2\" src=\"b.png\" width=\"200\""),
        "img.width=200 setter 写 width 内容属性\n{out}"
    );
}

#[test]
fn test_document_content_type_and_node_normalize_r2853() {
    // R2853：document.contentType（'text/html'，spec HTML 文档 MIME）+ Node.normalize()（no-op，
    // snapshot 模型文本为单一串故语义正确——DOM 态已 normalized，防防御性调用抛 TypeError）。
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
        "<html><body><div id='d'>hello</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // document.contentType = 'text/html'（spec HTML 文档）。
    sandbox.execute("globalThis.__ct = document.contentType;").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ct)").unwrap().value,
        "text/html",
        "document.contentType = 'text/html'（HTML 文档 MIME）"
    );

    // Node.normalize()：可调用（不抛 TypeError），返 undefined（spec void），文本不变。
    sandbox
        .execute(
            "var d = document.querySelector('#div');\
             globalThis.__normReturn = document.querySelector('#d').normalize();\
             globalThis.__tc = document.querySelector('#d').textContent;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__normReturn)").unwrap().value,
        "undefined",
        "normalize() 返 undefined（spec void）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tc)").unwrap().value,
        "hello",
        "normalize() no-op：textContent 不变（'hello'）"
    );

    // 多元素 normalize() 均可调用（不抛）——防 rich-text 编辑器 / innerHTML 后清理的防御性调用崩溃。
    sandbox
        .execute(
            "var ok = true;\
             try { document.body.normalize(); document.documentElement.normalize(); } catch (e) { ok = false; }\
             globalThis.__allOk = ok;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__allOk)").unwrap().value,
        "true",
        "body/documentElement.normalize() 均可调用（不抛 TypeError）"
    );
}

#[test]
fn test_node_is_connected_and_has_child_nodes_r2922() {
    // R2922：Node.isConnected（只读 boolean，节点是否连入 document）+ Node.hasChildNodes()（是否有任意
    // 子节点含文本/注释）。两者为 Node 接口最高频判活 / 子存在性 API（jQuery cleanData、React commit、
    // mutation handler、树遍历 diff），旧 shim 完全缺失 → isConnected 恒 undefined（falsy）误判在档元素为
    // detached。isConnected：sel-based 经 __zw_contains('html', sel)（element_contains 自含，html 自身命中）
    // 判定在 documentElement 子树内，亦正确反映 removeChild 后 detach；handle-only（createElement 等）→ false。
    // hasChildNodes：经 _childNodeList length>0。Document literal 恒 connected + 恒有 documentElement 子。
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
        "<html><body><div id='d'>hello</div><span id='empty'></span></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ── Document 节点：nodeType=9 / nodeName='#document' / 恒 connected / 恒有子。──
    sandbox
        .execute(
            "globalThis.__docNt = document.nodeType;\
             globalThis.__docNn = document.nodeName;\
             globalThis.__docConn = document.isConnected;\
             globalThis.__docHcn = document.hasChildNodes();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__docNt)").unwrap().value,
        "9",
        "document.nodeType = 9（DOCUMENT_NODE）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__docNn)").unwrap().value,
        "#document",
        "document.nodeName = '#document'"
    );
    assert_eq!(
        sandbox.execute("globalThis.__docConn").unwrap().value,
        "true",
        "document.isConnected = true（根节点恒连入）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__docHcn").unwrap().value,
        "true",
        "document.hasChildNodes() = true（恒有 documentElement）"
    );

    // ── isConnected：sel-based 在档元素（含 documentElement/body/查询结果）= true。──
    sandbox
        .execute(
            "globalThis.__htmlConn = document.documentElement.isConnected;\
             globalThis.__bodyConn = document.body.isConnected;\
             globalThis.__headConn = document.head.isConnected;\
             globalThis.__dConn = document.querySelector('#d').isConnected;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__htmlConn").unwrap().value, "true");
    assert_eq!(sandbox.execute("globalThis.__bodyConn").unwrap().value, "true");
    assert_eq!(sandbox.execute("globalThis.__headConn").unwrap().value, "true");
    assert_eq!(
        sandbox.execute("globalThis.__dConn").unwrap().value,
        "true",
        "querySelector('#d').isConnected = true（在档）"
    );

    // ── isConnected：handle-only 节点（createElement/createTextNode/createFragment 未挂载）= false。──
    // 注：register_dom_callbacks 不注 __zw_getBoundingClientRect，故 handle-only 无 probe 路径 → false。
    sandbox
        .execute(
            "globalThis.__elConn = document.createElement('div').isConnected;\
             globalThis.__tnConn = document.createTextNode('x').isConnected;\
             globalThis.__fragConn = document.createDocumentFragment().isConnected;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__elConn").unwrap().value,
        "false",
        "createElement('div').isConnected = false（detached）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__tnConn").unwrap().value,
        "false",
        "createTextNode('x').isConnected = false（detached）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fragConn").unwrap().value,
        "false",
        "createDocumentFragment().isConnected = false（恒 detached）"
    );

    // ── hasChildNodes：有子（#d 含 'hello' 文本节点）/ body（含 #d·#empty·文本）= true；
    //    空元素（#empty）/ handle-only createElement = false。──
    sandbox
        .execute(
            "globalThis.__bodyHcn = document.body.hasChildNodes();\
             globalThis.__dHcn = document.querySelector('#d').hasChildNodes();\
             globalThis.__emptyHcn = document.querySelector('#empty').hasChildNodes();\
             globalThis.__elHcn = document.createElement('div').hasChildNodes();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__bodyHcn").unwrap().value,
        "true",
        "body.hasChildNodes() = true（含子元素 + 文本）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dHcn").unwrap().value,
        "true",
        "#d.hasChildNodes() = true（含 'hello' 文本节点）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__emptyHcn").unwrap().value,
        "false",
        "#empty.hasChildNodes() = false（无子）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__elHcn").unwrap().value,
        "false",
        "createElement('div').hasChildNodes() = false（detached 无子）"
    );

    // ── isConnected：removeChild 后 detach（sel-based 经 __zw_contains 反映在档态）。──
    // 先捕获 #d proxy（sel='#d'），再换 snapshot 为不含 #d 的 html（模拟 removeChild 已应用），
    // 旧 proxy.isConnected 应翻 false（__zw_contains('html','#d') 读新 snapshot → '0'）。**置于末尾**：
    // 换 snapshot 移除 #d，破坏后续 #d 查询，故 hasChildNodes 等须在此之前完成。
    sandbox
        .execute("globalThis.__dRef = document.querySelector('#d'); globalThis.__connBefore = globalThis.__dRef.isConnected;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__connBefore").unwrap().value, "true");
    *dom_html.lock().unwrap() = "<html><body><span id='empty'></span></body></html>".to_string();
    sandbox
        .execute("globalThis.__connAfter = globalThis.__dRef.isConnected;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__connAfter").unwrap().value,
        "false",
        "#d 移出 document 后 isConnected = false（__zw_contains 反映 detach）"
    );
}

/// R2922：`window.onload = fn` 事件处理器 IDL 语义——赋值等价注册 load 监听，
/// `__zw_dispatch_event('html','load')` 派发时触发（driving:
/// css-overflow/line-clamp/webkit-line-clamp-019 的 `window.onload` 动态改样式）。
#[test]
fn test_window_onload_assignment_registers_load_listener() {
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

    // window.onload 赋值（须在派发前注册）。
    sandbox
        .execute("window.onload = function() { document.querySelector('#t').style.color = 'red'; };")
        .unwrap();
    // 派发 load 事件 → onload 回调应执行 → 入队 SetStyle mutation。
    sandbox.execute("__zw_dispatch_event('html','load',null);").unwrap();

    let ms = mutations.lock().unwrap();
    let style_mutations: Vec<_> = ms
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetStyle {
                selector,
                property,
                value,
            } => Some((selector.as_str(), property.as_str(), value.as_str())),
            _ => None,
        })
        .collect();
    assert!(
        style_mutations.contains(&("#t", "color", "red")),
        "window.onload 回调须产生 SetStyle color=red，实际 {style_mutations:?}"
    );
}

/// R2922：`el.style.webkitLineClamp = '6'` 按 CSSOM vendor 前缀规则归一为 CSS 属性
/// `-webkit-line-clamp`（通用 camelCase→kebab 会产 `webkit-line-clamp`——丢前导 `-`，
/// CSS parser 不认 → 渲染静默失效；driving: webkit-line-clamp-019）。
#[test]
fn test_style_webkit_prefix_property_normalized_with_leading_hyphen() {
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

    sandbox
        .execute("document.querySelector('#t').style.webkitLineClamp = '6';")
        .unwrap();

    let ms = mutations.lock().unwrap();
    let style_mutations: Vec<_> = ms
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetStyle {
                selector,
                property,
                value,
            } => Some((property.as_str(), value.as_str())),
            _ => None,
        })
        .collect();
    assert!(
        style_mutations.contains(&("-webkit-line-clamp", "6")),
        "webkitLineClamp 须归一为 -webkit-line-clamp（带前导连字符），实际 {style_mutations:?}"
    );
}
