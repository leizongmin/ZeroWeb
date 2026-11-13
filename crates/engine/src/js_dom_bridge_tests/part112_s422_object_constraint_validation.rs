// js_dom_bridge 测试切片 112。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s422：constraint validation candidate 门——object/fieldset/output 非
/// candidate（spec §4.10.5.2.2 #candidate-for-constraint-validation 仅
/// input/select/textarea/form-associated custom elements），checkValidity/
/// reportValidity 恒 true 且不派 invalid（#dom-cva-checkvalidity/
/// #reportvalidity 步 1 candidacy 门）；setCustomValidity 仍记 customError
/// （validity IDL 是被动状态照常翻转；willValidate false 同条款）。
///
/// 复现面：object-constraint-validation.html 1F——`setCustomValidity('custom
/// error')` 后 `assert_true(element.checkValidity())` 失败（无 candidacy 门，
/// customError 落穿 v.valid=false）。spec 链接
/// https://html.spec.whatwg.org/multipage/#dom-cva-checkvalidity 。
///
/// 断言面：①object 驱动案镜像（前/后 setCustomValidity 三断言 false/true/
/// true 双轮 + validity.customError=true/.valid=false 被动翻转）；②output/
/// fieldset 同门钉（共享臂三 tag）；③input candidate 不受门影响（set 后
/// checkValidity false、清后复 true——candidacy 门不过界）；④FORM 级遍历臂
/// 不受影响（invalid input → form.checkValidity false）。
#[test]
fn test_s422_object_constraint_validation_candidacy_gate() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    crate::js_dom_bridge::publish_mut_applied_cursor(0);
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(Vec::new()));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><object id='element'></object>\
         <fieldset id='fs'></fieldset><output id='out'></output>\
         <input id='inp'><form id='f'><input id='finp'></form></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    let snapshot = crate::rect_bridge::new_layout_rect_snapshot();
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, Some(&snapshot));

    sandbox
        .execute(
            "globalThis.__el = document.getElementById('element');\
             globalThis.__fs = document.getElementById('fs');\
             globalThis.__out = document.getElementById('out');\
             globalThis.__inp = document.getElementById('inp');\
             globalThis.__f = document.getElementById('f');\
             globalThis.__r = [];",
        )
        .unwrap();
    // ① object 驱动案镜像：前/后 setCustomValidity 双轮三断言（false/true/true）
    // + validity 被动面（customError true / valid false）。
    sandbox
        .execute(
            "var el = __el;\
             __r.push('wv' + el.willValidate, 'cv' + el.checkValidity(), 'rv' + el.reportValidity());\
             el.setCustomValidity('custom error');\
             __r.push('wv' + el.willValidate, 'cv' + el.checkValidity(), 'rv' + el.reportValidity());\
             __r.push('ce' + el.validity.customError, 'vd' + el.validity.valid);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("__r.join(',')").unwrap().value,
        "wvfalse,cvtrue,rvtrue,wvfalse,cvtrue,rvtrue,cetrue,vdfalse",
        "object 非 candidate：setCustomValidity 前后三断言恒 false/true/true，validity 被动翻转"
    );

    // ② output/fieldset 同门钉（共享臂三 tag）。
    sandbox
        .execute(
            "__out.setCustomValidity('x');\
             __fs.setCustomValidity('y');\
             globalThis.__r2 = __out.checkValidity() + ',' + __fs.checkValidity() + ','\
               + __out.reportValidity() + ',' + __fs.reportValidity();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("__r2").unwrap().value,
        "true,true,true,true",
        "output/fieldset 非 candidate：checkValidity/reportValidity 恒 true"
    );

    // ③ input candidate 不受门影响：set 后 false（invalid 派发面保持）、清后复 true。
    sandbox
        .execute(
            "__inp.setCustomValidity('bad');\
             globalThis.__r3 = 'a' + __inp.checkValidity();\
             __inp.setCustomValidity('');\
             __r3 += ',b' + __inp.checkValidity();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("__r3").unwrap().value,
        "afalse,btrue",
        "input candidate：candidacy 门不越界，customError 行为原样"
    );

    // ④ FORM 级遍历臂：invalid 控件 → form.checkValidity false（R57 形态保持）。
    sandbox
        .execute(
            "document.getElementById('finp').setCustomValidity('form-level');\
             globalThis.__r4 = String(__f.checkValidity());",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("__r4").unwrap().value,
        "false",
        "FORM 级 checkValidity 遍历臂不受 candidate 门影响"
    );
}
