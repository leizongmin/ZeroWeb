// t-g2b impl-006：`__zw_get_tag_attrs_resolved` 批量属性读 + `__zw_dispatch_element_event`
// 批量路径——相对 URL 解析命中、多元素同 URL 全派发、空 attr 跳过、wire 形状。
// 背景：逐元素 `__zw_get_attr`/`__zw_parse_url` 在真实大页放大成 O(N) host 往返火车
// （github link ×115 每臂 4.2s ×10 臂），automation eval 插队被饿。

use crate::js_dom_bridge::{generate_js_dom_shim, register_dom_callbacks, DomMutation};

fn make_sandbox(
    html: &str,
    page_url: &str,
) -> zero_script_sandbox::V8Sandbox {
    use std::sync::{Arc, Mutex};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = zero_script_sandbox::V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url_arc: Arc<Mutex<String>> = Arc::new(Mutex::new(page_url.to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url_arc,
        &canvas_registry,
        None,
    );
    sandbox
}

/// wire 形状：`sel\x1fabsUrl|…`——相对 href 以 page_url 为 base 绝对化；空 attr 跳过。
#[test]
fn p114_batch_attrs_wire_shape_and_relative_resolution() {
    let mut sb = make_sandbox(
        "<html><head>\
         <link rel='stylesheet' href='assets/rel.css'>\
         <link rel='stylesheet' href='https://cdn.example.com/abs.css'>\
         <link rel='preload' as='font'>\
         </head><body></body></html>",
        "https://example.com/page/index.html",
    );
    let wire = sb
        .execute("__zw_get_tag_attrs_resolved('link', 'href')")
        .unwrap()
        .value;
    // 三个 link：相对解析、绝对原样、无 href 跳过（wire 恰两对）。
    let pairs: Vec<&str> = wire.split('|').collect();
    assert_eq!(pairs.len(), 2, "空 attr 条目跳过，wire 恰两对: {wire:?}");
    let (sel0, url0) = pairs[0].split_once('\u{1f}').expect("第一对含 \\x1f");
    let (sel1, url1) = pairs[1].split_once('\u{1f}').expect("第二对含 \\x1f");
    assert!(
        url0 == "https://example.com/page/assets/rel.css",
        "相对 href 按 page_url 绝对化: {url0:?}"
    );
    assert_eq!(url1, "https://cdn.example.com/abs.css", "绝对 href 原样");
    assert!(!sel0.is_empty() && !sel1.is_empty(), "selector 非空");
    assert_ne!(sel0, sel1, "两元素 selector 互异");
    // 未知 tag / 空 attr 参数返空 wire。
    assert_eq!(
        sb.execute("__zw_get_tag_attrs_resolved('', 'href')").unwrap().value,
        "",
        "空 tag 返空"
    );
}

/// 批量路径派发语义：相对 URL 元素经 host 绝对化后命中 dispatch；多元素同 URL 全派发；
/// 无匹配零派发（mirror R2944 语义在批量实现下的保持）。
#[test]
fn p114_batch_dispatch_relative_and_multi_match() {
    let mut sb = make_sandbox(
        "<html><head>\
         <link rel='stylesheet' href='assets/rel.css'>\
         <link rel='stylesheet' href='rel.css'>\
         </head><body></body></html>",
        "https://example.com/page/index.html",
    );
    sb.execute(
        "globalThis.__hit = [];\
         var links = document.querySelectorAll('link');\
         for (var i = 0; i < links.length; i++) {\
           links[i].onload = function(){ globalThis.__hit.push('load'); };\
         }",
    )
    .unwrap();
    // 两个 link 的相对 href 解析后不同（assets/rel.css vs rel.css）——绝对 URL 各命中一个。
    sb.execute("__zw_dispatch_link_event('https://example.com/page/assets/rel.css', 'load');")
        .unwrap();
    assert_eq!(
        sb.execute("String(globalThis.__hit.length)").unwrap().value,
        "1",
        "恰命中 assets/rel.css 一个元素"
    );
    sb.execute("__zw_dispatch_link_event('https://example.com/page/rel.css', 'load');")
        .unwrap();
    assert_eq!(
        sb.execute("String(globalThis.__hit.length)").unwrap().value,
        "2",
        "第二个 URL 命中另一元素"
    );
    // 无匹配零派发。
    sb.execute(
        "globalThis.__before = globalThis.__hit.length;\
         __zw_dispatch_link_event('https://example.com/missing.css', 'load');",
    )
    .unwrap();
    assert_eq!(
        sb.execute("String(globalThis.__hit.length === globalThis.__before)")
            .unwrap()
            .value,
        "true",
        "无匹配不派发"
    );
}

/// 多元素同 URL：批量 wire 中两对同 absUrl，派发两次（spec：每元素至多一次，两元素两事件）。
#[test]
fn p114_batch_dispatch_same_url_all_matched() {
    let mut sb = make_sandbox(
        "<html><head>\
         <link rel='stylesheet' href='https://cdn.example.com/twin.css'>\
         <link rel='stylesheet' href='/twin.css'>\
         </head><body></body></html>",
        "https://cdn.example.com/page",
    );
    sb.execute(
        "globalThis.__hits = 0;\
         var links = document.querySelectorAll('link');\
         for (var i = 0; i < links.length; i++) {\
           links[i].onload = function(){ globalThis.__hits++; };\
         }",
    )
    .unwrap();
    sb.execute("__zw_dispatch_link_event('https://cdn.example.com/twin.css', 'load');")
        .unwrap();
    assert_eq!(
        sb.execute("String(globalThis.__hits)").unwrap().value,
        "2",
        "同 URL 两元素（绝对+相对解析同值）全派发"
    );
}
