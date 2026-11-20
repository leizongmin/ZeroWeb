// t-g2b impl-008：属性/ns 探测快照层钉面——`__zw_get_attr_map_lw` wire 形状
// （基底+pending 单遍应用）、shim wrap 生效（overlay 同批一致/removeAttribute
// absent/absent vs present-empty 四象限）、视图戳失效重取、get_ns 快照。
// 背景：页面 JS 全文档审计逐条 `__zw_get_attr_lw`/`__zw_has_attr_lw`/`__zw_get_ns`
// 宿主往返洪峰（g26 link 臂窗 265,903 次 ×~24µs）压缩为 per-sel 一次全图取。

// 注意：本文件经 include! 并入 js_dom_bridge_tests 聚合 scope（generate_js_dom_shim/
// register_dom_callbacks 由 part114 的 use 提供），此处不重复 use；引用统一完整路径。

fn make_snap_sandbox(
    html: &str,
) -> (
    zero_script_sandbox::V8Sandbox,
    std::sync::Arc<std::sync::Mutex<String>>,
    std::sync::Arc<std::sync::Mutex<Vec<crate::js_dom_bridge::DomMutation>>>,
) {
    use std::sync::{Arc, Mutex};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = zero_script_sandbox::V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<crate::js_dom_bridge::DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/x.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    (sandbox, dom_html, mutations)
}

/// 宿主 map 回调 wire：JSON 名值对、absent 名不入列、pending SetAttr 覆盖基底值 /
/// 新名 push / RemoveAttr 删名（与 sel_attr_override 同底单遍）。
#[test]
fn p116_attr_map_wire_shape() {
    let (mut sb, dom_html, mutations) =
        make_snap_sandbox("<html><body><div id='a' class='x' data-present=''>t</div></body></html>");
    // 基底：absent 不入列、present-empty 以空串入列。
    let wire = sb.execute("__zw_get_attr_map_lw('#a')").unwrap().value;
    let v: serde_json::Value = serde_json::from_str(&wire).expect("wire 是 JSON 数组");
    let arr = v.as_array().expect("数组");
    let mut names: Vec<&str> = arr.iter().map(|p| p[0].as_str().unwrap()).collect();
    names.sort_unstable();
    assert_eq!(names, vec!["class", "data-present", "id"], "基底三属性: {wire:?}");
    let dp = arr.iter().find(|p| p[0] == "data-present").unwrap();
    assert_eq!(dp[1], "", "present-empty 以空串在列");

    // pending：SetAttr 覆盖 class 值 + 新名 push；RemoveAttr 删 id。
    mutations
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(crate::js_dom_bridge::DomMutation::SetAttr {
            selector: "#a".to_string(),
            name: "class".to_string(),
            value: "y".to_string(),
        });
    mutations
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(crate::js_dom_bridge::DomMutation::SetAttr {
            selector: "#a".to_string(),
            name: "data-new".to_string(),
            value: "n1".to_string(),
        });
    mutations
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(crate::js_dom_bridge::DomMutation::RemoveAttr {
            selector: "#a".to_string(),
            name: "id".to_string(),
        });
    let wire2 = sb.execute("__zw_get_attr_map_lw('#a')").unwrap().value;
    let v2: serde_json::Value = serde_json::from_str(&wire2).expect("JSON");
    let mut names2: Vec<&str> = v2.as_array().unwrap().iter().map(|p| p[0].as_str().unwrap()).collect();
    names2.sort_unstable();
    assert_eq!(
        names2,
        vec!["class", "data-new", "data-present"],
        "覆盖+新增+删除单遍: {wire2:?}"
    );
    let cls = v2.as_array().unwrap().iter().find(|p| p[0] == "class").unwrap();
    assert_eq!(cls[1], "y", "SetAttr 覆盖基底值");
    let dn = v2.as_array().unwrap().iter().find(|p| p[0] == "data-new").unwrap();
    assert_eq!(dn[1], "n1", "新名 push");
    // 无关 sel 不受 pending 影响。
    let wire3 = sb.execute("__zw_get_attr_map_lw('#zz')").unwrap().value;
    assert_eq!(wire3, "[]", "不存在的 sel 返空数组");
}

/// shim wrap 生效：begin_script 后读回调带 __zwSnapWrapped；overlay 同批一致
///（setAttribute→getAttribute 同 turn，未 drain）；removeAttribute→null（R3190）。
/// 全部断言必须在调用 begin_script 的同一次 execute 内——宿主每 execute 入口经
/// global.set 重装 native 回调，wrap 生效窗口 = 单次 execute（跨 execute 恒 native）。
#[test]
fn p116_attr_snapshot_overlay_same_turn() {
    let (mut sb, _dom_html, _mutations) = make_snap_sandbox("<html><body><div id='a' class='x'>t</div></body></html>");
    let r = sb
        .execute(
            "globalThis.__zw_begin_script();\
             var marked = String(__zw_get_attr_lw.__zwSnapWrapped === true) + '|' + String(__zw_has_attr_lw.__zwSnapWrapped === true) + '|' + String(__zw_set_attr.__zwSnapWrapped === true) + '|' + String(__zw_get_ns.__zwSnapWrapped === true);\
             var el = document.getElementById('a');\
             el.setAttribute('data-t', 'v1');\
             var r1 = el.getAttribute('data-t');\
             el.setAttribute('data-r', 'x');\
             el.removeAttribute('data-r');\
             var r2 = String(el.getAttribute('data-r')) + '|' + String(el.hasAttribute('data-r'));\
             marked + '#' + r1 + '#' + r2",
        )
        .unwrap()
        .value;
    assert_eq!(
        r, "true|true|true|true#v1#null|false",
        "wrap 标记+同批 overlay 读+remove absent: {r:?}"
    );
}

/// absent vs present-empty 四象限（has 层 '0'/'1'，get 层 null/''——R3190 语义在
/// 快照层逐位保持；跨 setAttribute 前后：快照 miss 取全图后仍正确）。
#[test]
fn p116_attr_snapshot_absent_vs_empty() {
    let (mut sb, _dom_html, _mutations) =
        make_snap_sandbox("<html><body><div id='a' data-empty=''>t</div></body></html>");
    sb.execute("globalThis.__zw_begin_script(); 'ok'").unwrap();
    let r = sb
        .execute(
            "var el = document.getElementById('a');\
             var g1 = String(el.getAttribute('data-empty'));\
             var h1 = String(el.hasAttribute('data-empty'));\
             var g2 = String(el.getAttribute('data-absent'));\
             var h2 = String(el.hasAttribute('data-absent'));\
             g1 + '|' + h1 + '|' + g2 + '|' + h2",
        )
        .unwrap()
        .value;
    assert_eq!(r, "|true|null|false", "present-empty='' / absent=null 四象限: {r:?}");

    // setAttribute 空串（present-empty 写入）后同一象限翻转。
    let r2 = sb
        .execute(
            "var el = document.getElementById('a'); el.setAttribute('data-absent', ''); String(el.getAttribute('data-absent')) + '|' + String(el.hasAttribute('data-absent'))",
        )
        .unwrap()
        .value;
    assert_eq!(r2, "|true", "写空串后 present-empty: {r2:?}");
}

/// 视图戳失效：pending 写（mutation append → 戳 count 变）后重取快照新值；
/// 队列清空 + doc 落地（drain 模拟）后重取 = doc 值（overlay 值相等论证）。
#[test]
fn p116_attr_snapshot_stamp_invalidate() {
    let (mut sb, dom_html, mutations) = make_snap_sandbox("<html><body><div id='a' class='old'>t</div></body></html>");
    sb.execute("globalThis.__zw_begin_script(); 'ok'").unwrap();
    // 首读建立快照（class='old' 基底）。
    let r0 = sb
        .execute("document.getElementById('a').getAttribute('class')")
        .unwrap()
        .value;
    assert_eq!(r0, "old", "首读基底值");
    // pending SetAttr（绕过 shim wrapper 直接入队——模拟宿主侧/其他注册站写入）。
    mutations
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(crate::js_dom_bridge::DomMutation::SetAttr {
            selector: "#a".to_string(),
            name: "class".to_string(),
            value: "mutated".to_string(),
        });
    // 戳 count 变 → 重取（map 回调单遍含 pending）→ 读到新值。
    let r1 = sb
        .execute("document.getElementById('a').getAttribute('class')")
        .unwrap()
        .value;
    assert_eq!(r1, "mutated", "戳失效后重取 pending 值: {r1:?}");
    // drain 模拟：值落地 doc + 队列清空（count 归零 → 戳再变）→ 读 doc 值。
    {
        let mut list = mutations.lock().unwrap_or_else(|e| e.into_inner());
        list.clear();
    }
    *dom_html.lock().unwrap_or_else(|e| e.into_inner()) =
        "<html><body><div id='a' class='mutated'>t</div></body></html>".to_string();
    let r2 = sb
        .execute("document.getElementById('a').getAttribute('class')")
        .unwrap()
        .value;
    assert_eq!(r2, "mutated", "drain 后 doc 吸收值（overlay 值相等）: {r2:?}");
}

/// get_ns 快照：svg 元素 ns 首取 + 二次读同值（纯快照语义不变）；HTML 元素 ns
/// 含 xhtml URI（DOM 标准：HTML 文档中 HTML 元素 namespaceURI 恒 xhtml 命名空间）。
/// ns 调用与 begin_script 同 execute（wrap 生效窗口 = 单次 execute）。
#[test]
fn p116_attr_snapshot_ns() {
    let (mut sb, _dom_html, _mutations) =
        make_snap_sandbox("<html><body><svg id='s' viewBox='0 0 1 1'></svg><div id='d'>t</div></body></html>");
    let r = sb
        .execute(
            "globalThis.__zw_begin_script();\
             var s1 = __zw_get_ns(document.getElementById('s')._zwSelector || '#s');\
             var s2 = __zw_get_ns('#s');\
             var d1 = __zw_get_ns('#d');\
             String(s1 === s2) + '|' + String((s1 || '').indexOf('svg') >= 0) + '|' + String((d1 || '').indexOf('xhtml') >= 0)",
        )
        .unwrap()
        .value;
    assert_eq!(r, "true|true|true", "svg ns 一致且含 svg；HTML ns 含 xhtml: {r:?}");
}
