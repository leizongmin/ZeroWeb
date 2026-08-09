// js_dom_bridge 测试切片 14（R3074+）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-13 共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。
// 按单文件 ≤2000 行拆分，本切片承载 element-method / Web-API 后续测试。

#[test]
fn test_element_check_visibility_r3074() {
    // R3074：Element.checkVisibility(options)——「being rendered」+ 可选 opacity/visibility 检查。
    // ad viewability / lazy-load 库用。经 host __zw_get_computed_style（display/opacity/visibility）+ 祖先链。
    // https://drafts.csswg.org/cssom-view-1/#dom-element-checkvisibility
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
         <div id='vis'>visible</div>\
         <div id='dn' style='display:none'>dn</div>\
         <div id='anc' style='display:none'><span id='childDn'>child</span></div>\
         <div id='op0' style='opacity:0'>op0</div>\
         <div id='opHalf' style='opacity:0.5'>opHalf</div>\
         <div id='visH' style='visibility:hidden'>visH</div>\
         <div id='visAnc' style='visibility:hidden'><span id='visChild' style='visibility:visible'>child</span></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① 默认 rendered 元素 → true。
    sandbox
        .execute("globalThis.__vis = String(document.getElementById('vis').checkVisibility());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__vis").unwrap().value,
        "true",
        "默认 rendered 元素 checkVisibility() → true"
    );

    // ② display:none 元素 → false（默认，无需 option）。
    sandbox
        .execute("globalThis.__dn = String(document.getElementById('dn').checkVisibility());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dn").unwrap().value,
        "false",
        "display:none 元素 checkVisibility() → false（not rendered）"
    );

    // ③ 祖先 display:none → 子元素 false（祖先链遍历）。
    sandbox
        .execute("globalThis.__childDn = String(document.getElementById('childDn').checkVisibility());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__childDn").unwrap().value,
        "false",
        "祖先 display:none → 子元素 checkVisibility() → false（祖先链）"
    );

    // ④ opacity:0 → 默认 true（opacity 不属 rendered 判定），opacityProperty:true → false。
    sandbox
        .execute(
            "globalThis.__op0Def = String(document.getElementById('op0').checkVisibility());\
             globalThis.__op0Opt = String(document.getElementById('op0').checkVisibility({opacityProperty:true}));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__op0Def").unwrap().value,
        "true",
        "opacity:0 默认 checkVisibility() → true（opacity 非默认判定）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__op0Opt").unwrap().value,
        "false",
        "opacity:0 + opacityProperty:true → false"
    );

    // ⑤ opacity:0.5 → opacityProperty:true 仍 true（非 0）。
    sandbox
        .execute("globalThis.__opHalf = String(document.getElementById('opHalf').checkVisibility({opacityProperty:true}));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__opHalf").unwrap().value,
        "true",
        "opacity:0.5 + opacityProperty:true → true（非 0）"
    );

    // ⑥ visibility:hidden → 默认 true，visibilityProperty:true → false。
    sandbox
        .execute(
            "globalThis.__visHDef = String(document.getElementById('visH').checkVisibility());\
             globalThis.__visHOpt = String(document.getElementById('visH').checkVisibility({visibilityProperty:true}));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__visHDef").unwrap().value,
        "true",
        "visibility:hidden 默认 checkVisibility() → true（visibility 非默认判定）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__visHOpt").unwrap().value,
        "false",
        "visibility:hidden + visibilityProperty:true → false"
    );

    // ⑦ visibility 继承 + 覆盖：祖先 hidden，子显式 visible → 子计算 visibility=visible → true（继承正确反映）。
    sandbox
        .execute("globalThis.__visChild = String(document.getElementById('visChild').checkVisibility({visibilityProperty:true}));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__visChild").unwrap().value,
        "true",
        "祖先 hidden + 子 visible 覆盖 → 子计算 visibility=visible → checkVisibility(visibilityProperty) → true"
    );

    // ⑧ detached 元素（createElement，handle-only 无 sel）→ false（不在文档 → not rendered）。
    sandbox
        .execute("globalThis.__detached = String(document.createElement('div').checkVisibility());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__detached").unwrap().value,
        "false",
        "detached 元素（不在文档）checkVisibility() → false"
    );
}
