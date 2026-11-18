// C1 容器尾拆分序反转刀钉测（2026-11-15）：insertParagraph caret 文本尾位
// 入口归一提升 + wrap 面改指 container 后，主拆分 newLineRange 起点留在
// container 域外——extract 把 container 当 lastPartial 整克隆携内容入新块并
// deleteData 清空原块（corpus C1 8 键 got：空块居前 + Arm L 误垫左块 + 下钻
// 展平嵌套克隆）。刀 = newLineRange 起点改用归一前拆分位（ipM1OrigNode/
// ipM1OrigOff），wrap 后拆分位随内容入包裹块恢复域内性。8 键逐键断言
// corpus exp 唯一形（insertparagraph.js :640-658 bq 两形 + :38-56 td 两形，
// dPS div/p 双面）。

#[test]
fn test_insertparagraph_c1_bq_bare_div() {
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
        "<html><body><div id=ce119a contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h119a = document.getElementById("ce119a");
h119a.innerHTML = '<blockquote>foo</blockquote>';
var t119a = h119a.firstChild.firstChild;
var r119a = document.createRange();
r119a.setStart(t119a, 3);
r119a.setEnd(t119a, 3);
getSelection().removeAllRanges();
getSelection().addRange(r119a);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce119a\").innerHTML")
            .unwrap()
            .value,
        "<blockquote><div>foo</div><div><br></div></blockquote>",
        "① C1 `<blockquote>foo[]</blockquote>` dPS div：左块原文保留、右块空垫底（corpus :646 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c1_bq_bare_p() {
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
        "<html><body><div id=ce119b contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h119b = document.getElementById("ce119b");
h119b.innerHTML = '<blockquote>foo</blockquote>';
var t119b = h119b.firstChild.firstChild;
var r119b = document.createRange();
r119b.setStart(t119b, 3);
r119b.setEnd(t119b, 3);
getSelection().removeAllRanges();
getSelection().addRange(r119b);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce119b\").innerHTML")
            .unwrap()
            .value,
        "<blockquote><p>foo</p><p><br></p></blockquote>",
        "② C1 `<blockquote>foo[]</blockquote>` dPS p：同上 p 面（corpus :652 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c1_bq_br_div() {
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
        "<html><body><div id=ce119c contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h119c = document.getElementById("ce119c");
h119c.innerHTML = '<blockquote>foo<br></blockquote>';
var t119c = h119c.firstChild.firstChild;
var r119c = document.createRange();
r119c.setStart(t119c, 3);
r119c.setEnd(t119c, 3);
getSelection().removeAllRanges();
getSelection().addRange(r119c);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce119c\").innerHTML")
            .unwrap()
            .value,
        "<blockquote><div>foo</div><div><br></div></blockquote>",
        "③ C1 `<blockquote>foo[]<br></blockquote>` dPS div：尾随 br 随 extract 入新块（corpus :658 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c1_bq_br_p() {
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
        "<html><body><div id=ce119d contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h119d = document.getElementById("ce119d");
h119d.innerHTML = '<blockquote>foo<br></blockquote>';
var t119d = h119d.firstChild.firstChild;
var r119d = document.createRange();
r119d.setStart(t119d, 3);
r119d.setEnd(t119d, 3);
getSelection().removeAllRanges();
getSelection().addRange(r119d);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce119d\").innerHTML")
            .unwrap()
            .value,
        "<blockquote><p>foo</p><p><br></p></blockquote>",
        "④ C1 `<blockquote>foo[]<br></blockquote>` dPS p：同上 p 面（corpus :664 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c1_td_single_div() {
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
        "<html><body><div id=ce119e contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 删除前言绕行：corpus `fo[o</table>b]ar` 的跨表 delete 边界面有独立偏差
    // （RED 实录 shim 多吞尾字符 `ar`→`r`，另档残留）；钉只锚拆分面，caret
    // 直设删除后等价位（fo 尾）。
    sandbox
        .execute(
            r##"
var h119e = document.getElementById("ce119e");
h119e.innerHTML = '<table><tbody><tr><td>fo</td></tr></tbody></table>ar';
var t119e = h119e.firstChild.firstChild.firstChild.firstChild.firstChild;
var r119e = document.createRange();
r119e.setStart(t119e, 2);
r119e.setEnd(t119e, 2);
getSelection().removeAllRanges();
getSelection().addRange(r119e);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce119e\").innerHTML")
            .unwrap()
            .value,
        "<table><tbody><tr><td><div>fo</div><div><br></div></td></tr></tbody></table>ar",
        "⑤ C1 `<td>fo[o</table>b]ar` dPS div：格内拆分序正（corpus :40 exp 拆分面）"
    );
}

#[test]
fn test_insertparagraph_c1_td_single_p() {
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
        "<html><body><div id=ce119f contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h119f = document.getElementById("ce119f");
h119f.innerHTML = '<table><tbody><tr><td>fo</td></tr></tbody></table>ar';
var t119f = h119f.firstChild.firstChild.firstChild.firstChild.firstChild;
var r119f = document.createRange();
r119f.setStart(t119f, 2);
r119f.setEnd(t119f, 2);
getSelection().removeAllRanges();
getSelection().addRange(r119f);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce119f\").innerHTML")
            .unwrap()
            .value,
        "<table><tbody><tr><td><p>fo</p><p><br></p></td></tr></tbody></table>ar",
        "⑥ C1 `<td>fo[o</table>b]ar` dPS p：同上 p 面（corpus :46 exp 拆分面）"
    );
}

#[test]
fn test_insertparagraph_c1_td_cross_div() {
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
        "<html><body><div id=ce119g contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h119g = document.getElementById("ce119g");
h119g.innerHTML = '<table><tbody><tr><td>fo</td><td>bar</td><td>baz</td></tr></tbody></table>';
var t119g = h119g.firstChild.firstChild.firstChild.firstChild.firstChild;
var r119g = document.createRange();
r119g.setStart(t119g, 2);
r119g.setEnd(t119g, 2);
getSelection().removeAllRanges();
getSelection().addRange(r119g);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce119g\").innerHTML")
            .unwrap()
            .value,
        "<table><tbody><tr><td><div>fo</div><div><br></div></td><td>bar</td><td>baz</td></tr></tbody></table>",
        "⑦ C1 `<td>fo[o<td>b]ar<td>baz</table>` dPS div：首格拆分序正（corpus :52 exp 拆分面，无删除前言）"
    );
}

#[test]
fn test_insertparagraph_c1_td_cross_p() {
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
        "<html><body><div id=ce119h contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h119h = document.getElementById("ce119h");
h119h.innerHTML = '<table><tbody><tr><td>fo</td><td>bar</td><td>baz</td></tr></tbody></table>';
var t119h = h119h.firstChild.firstChild.firstChild.firstChild.firstChild;
var r119h = document.createRange();
r119h.setStart(t119h, 2);
r119h.setEnd(t119h, 2);
getSelection().removeAllRanges();
getSelection().addRange(r119h);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce119h\").innerHTML")
            .unwrap()
            .value,
        "<table><tbody><tr><td><p>fo</p><p><br></p></td><td>bar</td><td>baz</td></tr></tbody></table>",
        "⑧ C1 `<td>fo[o<td>b]ar<td>baz</table>` dPS p：同上 p 面（corpus :58 exp 拆分面，无删除前言）"
    );
}
