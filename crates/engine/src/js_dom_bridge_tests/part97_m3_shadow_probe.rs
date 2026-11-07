// 45 号 shadow-dom-rendering M3 回归：命令式 attachShadow 面的 move-in 接线
//（R97 noWire 缺口）与移动语义。slot-fallback-content-004 的 recorded mutation
// 序列（attachShadow → appendChild(slot) → A.remove()）——修复前 slot 只落
// JS 侧 `_handleChildren`（engine 摘除后悬垂），shadow 树恒空、快照回写丢子。

#[test]
fn m3_shadow_slot_move_into_shadow_root() {
    let html = r#"<!DOCTYPE html><html><body>
<p>Test passes if there is one line of text "SLOT1" below.</p>
<div id="host1">
  <slot id="slot1" name="slot1">SLOT1</slot>
  <div id="A" slot="slot1">FAIL</div>
</div>
</body></html>"#;

    // shim 侧实录序列（part04 appendChild trap 接线后）：move-out 走 `Remove`
    //（selector 键入 detached_stash），move-in 走 AppendChildByHandleSel（stash 复用
    // NodeId——spec pre-insert 摘除后同节点再插）。
    let mutations = vec![
        DomMutation::CreateDocumentFragment { handle: "__n0".into() },
        DomMutation::AttachShadow {
            host_selector: "#host1".into(),
            root_handle: "__n0".into(),
            mode: "open".into(),
            delegates_focus: false,
            clonable: false,
            serializable: false,
        },
        DomMutation::Remove { selector: "#slot1".into() },
        DomMutation::AppendChildByHandleSel {
            parent_handle: "__n0".into(),
            child_selector: "#slot1".into(),
        },
        DomMutation::Remove { selector: "#A".into() },
    ];
    let (new_html, _handle_selectors) =
        crate::js_dom_bridge::apply_mutations_to_html_with_handles(html, &mutations).expect("apply ok");

    // 快照回写：shadow root 以声明式模板形态序列化，slot1 连同回退文本在根内；
    // 移除的 #A 不再存活。
    assert!(
        new_html.contains(
            r#"<template shadowrootmode="open"><slot id="slot1" name="slot1">SLOT1</slot></template>"#
        ),
        "slot must land inside the shadow root template, got:\n{new_html}"
    );
    assert!(
        !new_html.contains("<div id=\"A\""),
        "removed #A must not survive, got:\n{new_html}"
    );

    // 重解析快照：物化（div 宿主）+ resolve_slots 自动跑——slot1 在 shadow 树内、
    // 分配表为空（无 [slot=slot1] light 子）→ 回退内容可渲染面。
    let doc = zero_dom::parse_html(&new_html);
    let host = doc.query_selector(doc.root(), "#host1").expect("host1 in tree");
    let sr = doc.shadow_root(host).expect("declarative root materialized on re-parse");
    let slot = doc
        .query_selector(sr, "#slot1")
        .expect("slot1 inside materialized shadow tree");
    assert_eq!(doc.text_content(slot).as_deref(), Some("SLOT1"), "fallback text survives round-trip");
    // 分配表新鲜度：slot1 无被分配 light 子（#A 已移除）。
    let assigned = doc.get_assigned_nodes(slot);
    assert!(assigned.is_empty(), "no light children assignable after move, got {assigned:?}");
}

// 45 号 M3：AttachShadowOnHandle 批内 ephemeral 宿主直连（修复前 warn+skip——
// css-scoping-shadow-slotted-nested 内层宿主经 create 句柄构建，内层 shadow 从不
// 物化）。宿主句柄在批内 handles 表即直挂；宿主须先入树（AppendChild 句柄子形态）。
#[test]
fn m3_attach_shadow_on_ephemeral_handle_host() {
    let html = r#"<!DOCTYPE html><html><body><p>outer light</p></body></html>"#;
    let mutations = vec![
        DomMutation::CreateElement { handle: "__ih".into(), tag: "inner-host".into() },
        // 内层宿主挂 body（selector 父 + handle 子，既有 AppendChild 变体）。
        DomMutation::AppendChild {
            parent_selector: "body".into(),
            child_handle: "__ih".into(),
        },
        // handle 形态 attachShadow（closed）。
        DomMutation::CreateDocumentFragment { handle: "__sr0".into() },
        DomMutation::AttachShadowOnHandle {
            host_handle: "__ih".into(),
            root_handle: "__sr0".into(),
            mode: "closed".into(),
            delegates_focus: false,
            clonable: false,
            serializable: false,
        },
        // 内容经容器句柄 append（handle 子形态，既有臂）。
        DomMutation::CreateElement { handle: "__sp".into(), tag: "span".into() },
        DomMutation::AppendChildByHandle { parent_handle: "__sr0".into(), child_handle: "__sp".into() },
    ];
    let (new_html, _hs) =
        crate::js_dom_bridge::apply_mutations_to_html_with_handles(html, &mutations).expect("apply ok");

    // 宿主序列化携带 closed 声明式模板 + 内容（快照回写面）。
    assert!(
        new_html.contains(r#"<template shadowrootmode="closed"><span></span></template>"#),
        "handle-form host shadow must serialize into the snapshot, got:\n{new_html}"
    );
    // 重解析物化 + 内容可查。
    let doc = zero_dom::parse_html(&new_html);
    let host = doc.query_selector(doc.root(), "inner-host").expect("inner-host in tree");
    let sr = doc.shadow_root(host).expect("declarative root materialized on re-parse");
    assert!(doc.query_selector(sr, "span").is_some(), "shadow content queryable after round-trip");
}

// 45 号 M3：声明式物化 valid shadow host name 放开连字符名（dom.spec
// #valid-shadow-host-name——custom element 名宿主词汇法即 valid，无需注册面）。
#[test]
fn m3_declarative_hyphenated_host_materializes() {
    let html = r#"<html><body><inner-host><template shadowrootmode=open><span>SHADOW</span></template><p>light</p></inner-host></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let host = doc.query_selector(doc.root(), "inner-host").expect("host present");
    let sr = doc.shadow_root(host).expect("hyphenated host materializes declarative root");
    assert!(
        doc.query_selector(sr, "span").is_some(),
        "shadow content queryable inside materialized root"
    );
    assert!(doc.query_selector(host, "p").is_some(), "light children stay on host");
}
