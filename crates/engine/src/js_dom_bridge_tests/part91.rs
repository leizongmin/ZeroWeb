// R5274：`__zw_dump_dom_selections` contents 臂**空 text 容器**的记录产出——
// selectNodeContents(元素) 的边界是 (parent,0)-(parent,childCount)，childCount≥1
// 即非折叠（spec dom-range-selectNodeContents），即使 textContent 为空（纯替换
// 元素子节点）也须出记录；paint 侧整 contents 判据 s==0&&e≥len 对 len=0 成立 →
// 图片 wash 生效（active-selection-041 驱动：img-only div + img::selection 底）。
// 旧实现 `e16 > s16` 折叠守卫把 (0,0) 记录整丢 → TEST ≡ notref。

#[test]
fn r5274_dump_dom_selections_contents_empty_text_with_children() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"test\"><img src=\"swatch.png\" width=\"100\" height=\"100\"></div><div id=\"empty\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var imgDiv = document.getElementById('test');
var r = document.createRange();
r.selectNodeContents(imgDiv);
getSelection().removeAllRanges();
getSelection().addRange(r);
globalThis.__recs = __zw_dump_dom_selections();

var emptyDiv = document.getElementById('empty');
var r2 = document.createRange();
r2.selectNodeContents(emptyDiv);
getSelection().removeAllRanges();
getSelection().addRange(r2);
globalThis.__recsEmpty = __zw_dump_dom_selections();
"##,
        )
        .unwrap();

    // img-only 容器：textContent="" 但有子节点 → 出 (0,0) 记录（非折叠）。
    let recs = sandbox.execute("JSON.stringify(globalThis.__recs)").unwrap().value;
    assert_eq!(
        recs,
        r##"[{"selector":"#test","start":0,"end":0}]"##,
        "img-only 容器的 selectNodeContents 应出 (0,0) 记录，实得 {recs}"
    );
    // 真空容器（无子节点）：range 折叠 → 不出记录。
    let empty = sandbox
        .execute("JSON.stringify(globalThis.__recsEmpty)")
        .unwrap()
        .value;
    assert_eq!(empty, "[]", "无子节点的 selectNodeContents 应保持折叠丢弃，实得 {empty}");
}
