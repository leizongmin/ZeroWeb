// inline-host 语料 ①簇跨批判别：CreateTextNode + AppendChild 落批 A，RemoveHandle
// 落批 B（persistent_handle_nodes 跨批预植语义）——批 B 后容器须无该文本子。
// webview 生产路径（apply_pending_shared_mutations → render_with_dom_mutations_persistent）
// 传 `Some(&mut persistent_handle_nodes)`；本钉锁 apply 层跨批解析面——若红，泄漏在
// 引擎 apply（handles 预植/remove_child 链）；若绿，泄漏在 runner/shim 层（队列消费
// 或 shim 视图），按层归因。

#[test]
fn ihost_cross_batch_text_remove_handle_resolves_via_persistent_nodes() {
    let html = "<html><body><div id=\"host\"></div></body></html>";
    let mut doc = crate::js_dom_bridge::parse_html(html);

    // 批 A：建文本 + 挂容器。
    let mut persistent: std::collections::HashMap<String, zero_dom::NodeId> =
        std::collections::HashMap::new();
    let batch_a = vec![
        DomMutation::CreateTextNode {
            handle: "__n0".to_string(),
            text: "p1".to_string(),
        },
        DomMutation::AppendChild {
            parent_selector: "#host".to_string(),
            child_handle: "__n0".to_string(),
        },
    ];
    crate::js_dom_bridge::apply_dom_mutations_full(
        &mut doc,
        &batch_a,
        None,
        Some(&mut persistent),
        None,
        None,
    )
    .expect("batch A apply");

    let host = doc
        .query_selector(doc.root(), "#host")
        .expect("host present");
    assert_eq!(
        crate::js_dom_bridge::child_nodes_json_doc(&doc, "#host"),
        "[{\"k\":\"T\",\"v\":\"p1\"}]",
        "批 A 后容器含 p1 文本子"
    );

    // 批 B：跨批 RemoveHandle——生产路径 persistent 表已含 __n0→NodeId。
    assert!(
        persistent.contains_key("__n0"),
        "批 A 回填后 persistent 表须含文本 handle"
    );
    let batch_b = vec![DomMutation::RemoveHandle {
        handle: "__n0".to_string(),
    }];
    crate::js_dom_bridge::apply_dom_mutations_full(
        &mut doc,
        &batch_b,
        None,
        Some(&mut persistent),
        None,
        None,
    )
    .expect("batch B apply");

    assert_eq!(
        crate::js_dom_bridge::child_nodes_json_doc(&doc, "#host"),
        "[]",
        "跨批 RemoveHandle 后容器须空（persistent 预植解析）"
    );
    let _ = host;
}
