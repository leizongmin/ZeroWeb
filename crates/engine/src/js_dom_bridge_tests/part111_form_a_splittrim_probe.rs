// 编辑命令兼容 Form A 小刀（2026-11-13）：splitText 头截断欠发射面双向钉测。
//
// 背景：D2 钉测 v1 形（contenteditable 单文本节点整段选区）曾实证重放树
// "fooobaz<i>ob</i>az"——splitText 头截断未达 host（Form A 欠发射），后改形
// 三整节点避开本面挂账独立刀；E1 刀 mutation 流分析已实证 handle 世界
// "splitText 双发射正确"。本钉测以 v1 同形复现双向定谳：
// ① 直接 splitText(3) 子探针——头截断须发 SetTextOnHandle（handle trim 通道）、
//   尾段须 CreateTextNode+插入（章完整时 part04 R196 全路径）；
// ② execCommand('italic') 全流程——选 offsets [3,5)（"ob"）净效果须
//   `foo<i>ob</i>az`（Chrome canonical，WPT bold.js 边界形同族实录）。
// 红 = Form A 欠发射臂复发（按 mutation 流归因）；绿 = Form A 已被 B1/D1/E1
// 系列吸收，钉测转常驻（正期望）。
#[test]
fn test_form_a_splittext_trim_emission_r3254_m3() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
        .execute(
            r##"
// ① 直接 splitText 子探针（handle 章完整文本节点，不涉编辑命令）。
var h1 = document.createElement("div");
h1.id = "fah1";
document.body.appendChild(h1);
var t1 = document.createTextNode("fooobaz");
h1.appendChild(t1);
var tail1 = t1.splitText(3);
globalThis.__faHeadData = t1.data;
globalThis.__faTailData = tail1.data;
globalThis.__faJs1 = h1.innerHTML;

// ② execCommand italic 全流程子探针（D2 钉测 v1 同形：单文本节点 offsets [3,5)）。
var h2 = document.createElement("div");
h2.id = "fah2";
h2.setAttribute("contenteditable", "");
document.body.appendChild(h2);
h2.appendChild(document.createTextNode("fooobaz"));
var t2 = h2.firstChild;
var r2 = document.createRange();
r2.setStart(t2, 3);
r2.setEnd(t2, 5);
getSelection().removeAllRanges();
getSelection().addRange(r2);
document.execCommand("italic", false, "");
globalThis.__faJs2 = h2.innerHTML;
"##,
        )
        .unwrap();
    // JS 视图自洽（split 后 head/tail 即时效）。
    assert_eq!(
        sandbox.execute("String(globalThis.__faHeadData)").unwrap().value,
        "foo",
        "splitText(3) 后原节点 data 须为 'foo'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__faTailData)").unwrap().value,
        "obaz",
        "splitText(3) 返回尾节点 data 须为 'obaz'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__faJs1)").unwrap().value,
        "fooobaz",
        "JS 视图 h1 innerHTML 须为 head+tail 拼接"
    );
    // ① 发射面：头截断须发 SetTextOnHandle("foo")，尾段须 CreateTextNode("obaz")。
    {
        let m = mutations.lock().unwrap().clone();
        assert!(
            m.iter().any(|mm| matches!(mm, DomMutation::SetTextOnHandle { text, .. } if text == "foo")),
            "splitText(3) 头截断须发 SetTextOnHandle（text='foo'）——缺发即 Form A 欠发射复发，mutation 流: {m:?}"
        );
        assert!(
            m.iter().any(|mm| matches!(mm, DomMutation::CreateTextNode { text, .. } if text == "obaz")),
            "splitText(3) 尾段须发 CreateTextNode('obaz')，mutation 流: {m:?}"
        );
    }
    // ② 净效果：JS 视图与重放树均须 `foo<i>ob</i>az`（Chrome canonical）。
    assert_eq!(
        sandbox.execute("String(globalThis.__faJs2)").unwrap().value,
        "foo<i>ob</i>az",
        "execCommand('italic') JS 视图净效果须为 foo<i>ob</i>az"
    );
    {
        let m = mutations.lock().unwrap().clone();
        let mut doc = parse_html("<html><body></body></html>");
        apply_dom_mutations(&mut doc, &m).expect("Form A probe batch must apply");
        let host = find_by_selector(&doc, "#fah2").expect("fah2 must resolve");
        assert_eq!(
            doc.inner_html(host),
            "foo<i>ob</i>az",
            "execCommand('italic') 重放净效果须为 foo<i>ob</i>az（D2 v1 形曾实证 fooobaz<i>ob</i>az 欠发射），实际: {}",
            doc.inner_html(host),
        );
    }
}
