// R297+（js-dom M4）：part24——js_dom_bridge 测试第 24 段（part23 超 2000 行后的
// 新增切片段；CLAUDE.md §5 文件大小控制）。

#[test]
fn r297_query_selector_escapes_lone_surrogate_never_match() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R297：WPT ParentNode-querySelector-escapes 的孤立代理 never-match 族——
    // CSS 转义 `\d83d` 解码为 U+FFFD（spec css-syntax 对 surrogate 码点的 special
    // replacement），id 本体是孤立代理（≠ U+FFFD）→ 不得命中。JS 侧 id 原值缓存
    //（`_zwRawIds`）保持 lone surrogate 读回；旧版 host lossy 换损后 U+FFFD ===
    // U+FFFD 误命中。
    // https://drafts.csswg.org/css-syntax/#consume-escaped-code-point
    let out = sandbox
        .execute(
            r##"
var out = [];
// handle 容器形态（WPT 用例本体：createElement div + span 子）——首/尾孤立代理双向
function probe(id, selSuffix, tag) {
  var container = document.createElement("div");
  var child = document.createElement("span");
  child.id = id;
  container.appendChild(child);
  out.push(tag + ':' + (container.querySelector(selSuffix) === null ? 'null' : 'WRONG'));
}
probe("\ud83dsurrogateFirst", "#\\d83d" + " surrogateFirst", 'leadSurrogate');
probe("surrogateSecond\udd11", "#surrogateSecond\\dd11", 'trailSurrogate');
// 对照组：U+FFFD 本体的 id 与 `\d83d` 解码产物相等 → 应命中（WPT testMatched 族）
(function () {
  var container = document.createElement("div");
  var child = document.createElement("span");
  child.id = "�" + "surrogateFirst";
  container.appendChild(child);
  out.push('fffdMatch:' + (container.querySelector("#\\d83d" + " surrogateFirst") === child ? 'hit' : 'MISS'));
})();
// id 读回保真（lone surrogate 不被 lossy 换损）
(function () {
  var container = document.createElement("div");
  var child = document.createElement("span");
  child.id = "\ud83dsurrogateFirst";
  container.appendChild(child);
  out.push('idRoundtrip:' + JSON.stringify(child.id));
})();
// setAttribute / removeAttribute 路径的原值同步与清理
(function () {
  var container = document.createElement("div");
  var child = document.createElement("span");
  container.appendChild(child);
  child.setAttribute('id', 'a\ud83db');
  out.push('saId:' + JSON.stringify(child.id));
  out.push('saNever:' + (container.querySelector("#a\\d83db") === null ? 'null' : 'WRONG'));
  child.removeAttribute('id');
  out.push('rmId:' + JSON.stringify(child.id));
  child.id = 'plain';
  out.push('plain:' + ((container.querySelector('#plain') === child) ? 'hit' : 'MISS'));
})();
globalThis.__r297p = out.join('|');
"##,
        )
        .unwrap();
    // 期望串含孤立代理（Rust String 不能直接字面持有）——经 V8 侧 String.fromCharCode
    // 拼出完整期望行，两侧同源比较。
    let expect = sandbox
        .execute(
            r#"var e = [];
e.push('leadSurrogate:null');
e.push('trailSurrogate:null');
e.push('fffdMatch:hit');
e.push('idRoundtrip:' + JSON.stringify(String.fromCharCode(0xd83d) + 'surrogateFirst'));
e.push('saId:' + JSON.stringify('a' + String.fromCharCode(0xd83d) + 'b'));
e.push('saNever:null');
e.push('rmId:' + JSON.stringify(''));
e.push('plain:hit');
globalThis.__r297e = e.join('|');"#,
        )
        .unwrap();
    let expect = sandbox.execute("globalThis.__r297e").unwrap().value;
    let out = sandbox.execute("globalThis.__r297p").unwrap().value;
    assert_eq!(
        out, expect,
        "R297 escapes lone-surrogate never-match: raw id preserved, selector FFFD decode never equals lone surrogate, attribute/remove paths in sync"
    );
}
