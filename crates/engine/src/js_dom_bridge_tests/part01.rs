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
