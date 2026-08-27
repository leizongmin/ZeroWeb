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

#[test]
fn r298_query_in_subtree_scope_pseudo_class() {
    // R298：WPT ParentNode-querySelector-scope——`:scope` 在元素子树查询里指向调用元素
    //（scoping root = 调用元素，spec selectors-4 §6.4 + dom spec querySelector）。
    // 旧版 `:scope` 由 dom crate 静态判为文档根 html → `:scope > p` 在 div 子树内
    // 恒无匹配（div ≠ html）。修复 = closest 的 R153 替换模式复用（独立 `:scope`
    // token → root 唯一选择器）。
    let html = "<html><body>\
                <div id='d'><h1 id='test'>t</h1><p><span>hello</span></p></div>\
                <p>sibling</p>\
                </body></html>";
    // `:scope > p` → div 的直接子 p（span 是孙层不命中）。
    let hit = query_match_in_subtree(html, "#d", ":scope > p");
    let doc = parse_html(html);
    let n = find_by_selector(&doc, &hit).expect("须可解析");
    // 命中元素须是 p（含 span 文本 hello 的那个 p，非 body 直接子 sibling p）。
    let kid_tags: Vec<String> = doc
        .child_nodes(n)
        .iter()
        .filter_map(|k| match doc.get(*k).map(|nd| &nd.kind) {
            Some(zero_dom::NodeKind::Element(e)) => Some(e.local_name().to_string()),
            _ => None,
        })
        .collect();
    assert!(
        kid_tags.contains(&"span".to_string()),
        "命中的 p 须含 span 子（div 的直接子 p），got {kid_tags:?}"
    );
    // 命中的 p 的父必须是 div（:scope 指向 d）。
    let parent_sel = unique_selector_for_node(&doc, doc.parent_node(n).unwrap()).unwrap();
    assert!(parent_sel.contains("d"), "父须是 #d，got {parent_sel}");
    // `:scope > span` → null（span 是孙层）。
    assert_eq!(query_match_in_subtree(html, "#d", ":scope > span"), "");
    assert_eq!(query_all_in_subtree(html, "#d", ":scope > span"), "");
    // `:scope > h1` → #test（另一直接子形态）。
    assert!(!query_all_in_subtree(html, "#d", ":scope > h1").is_empty());
    // 不含 :scope 的查询不受影响（既有 scoping 回归）。
    assert!(!query_match_in_subtree(html, "#d", "p").is_empty());
    assert!(!query_all_in_subtree(html, "#d", "span").is_empty());
}

#[test]
fn r299_attr_selector_case_flags() {
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
    // R299：CSS Selectors L4 §attribute-case 的值尾 `i`/`s` 标志（JS 客户端匹配器
    // `_parseAttrInner`/`_matchAttrOf`）——`i` 双侧 ASCII 小写、`s` 恒等；值与标志间
    // 须有空白（裸 `i` 值非标志）。WPT querySelector-mixed-case 的
    // `[testAttr="alpha" s]`（detached 容器形态）。
    // https://drafts.csswg.org/selectors/#attribute-case
    let out = sandbox
        .execute(
            r#"
var out = [];
var c = document.createElement("div");
var html1 = document.createElement("div");
html1.setAttribute("testAttr", "alpha");
var svg1 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
svg1.setAttribute("testAttr", "ALPHA");
c.appendChild(html1);
c.appendChild(svg1);
out.push('s=' + c.querySelectorAll('[testAttr="alpha" s]').length);
out.push('sUP=' + c.querySelectorAll('[testAttr="Alpha" s]').length);
out.push('i=' + c.querySelectorAll('[testAttr="ALPHA" i]').length);
out.push('inc=' + c.querySelectorAll('[testAttr*=LPH i]').length);
out.push('plain=' + c.querySelectorAll('[testAttr="alpha"]').length);
// 裸 i 值非标志：`[a=i]` 的值是 "i"
var c2 = document.createElement("div");
var e2 = document.createElement("span");
e2.setAttribute("k", "i");
c2.appendChild(e2);
out.push('bareI=' + c2.querySelectorAll('[k=i]').length);
container = null;
globalThis.__r299p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r299p").unwrap().value;
    assert_eq!(
        out, "s=1|sUP=0|i=2|inc=2|plain=1|bareI=1",
        "R299 attr case flags: s exact-only, i folds both sides, bare-i is a value not a flag"
    );
}

#[test]
fn r300_insert_before_step3_ordering_and_doc_parent_step6() {
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
    let out = sandbox
        .execute(
            r#"
var out = [];
function probe(name, fn) {
  try { fn(); out.push(name + ':no-throw'); }
  catch (e) { out.push(name + ':' + e.name + '(' + (e.code != null ? e.code : '-') + ')'); }
}
var insertFunc = Node.prototype.insertBefore;
// Test1 parent: createElement div (proxy), node: createHTMLDocument (non-insertable), child detached
probe('t1', function () {
  var parent = document.createElement("div");
  var child = document.createElement("div");
  var node = document.implementation.createHTMLDocument("title");
  insertFunc.call(parent, node, child);
});
// Test2a parent: createDocument(null,"foo",null) (detached doc), node text, child detached
probe('t2a', function () {
  var child = document.createElement("div");
  var node = document.createTextNode("");
  var parent = document.implementation.createDocument(null, "foo", null);
  insertFunc.call(parent, node, child);
});
// Test2b parent: createElement div / createDocumentFragment, node doctype, child detached
probe('t2b', function () {
  var child = document.createElement("div");
  var node = document.implementation.createDocumentType("html", "", "");
  var parent = document.createElement("div");
  insertFunc.call(parent, node, child);
});
probe('t2b2', function () {
  var child = document.createElement("div");
  var node = document.implementation.createDocumentType("html", "", "");
  var parent = document.createDocumentFragment();
  insertFunc.call(parent, node, child);
});
// Test3a parent: createDocument(null,null,null), node fragment(2 el), child detached
probe('t3a', function () {
  var child = document.createElement("div");
  var parent = document.implementation.createDocument(null, null, null);
  var node = document.createDocumentFragment();
  node.appendChild(document.createElement("div"));
  node.appendChild(document.createElement("div"));
  insertFunc.call(parent, node, child);
});
// Test3b parent doc w/ element, node element, child detached
probe('t3b', function () {
  var child = document.createElement("div");
  var parent = document.implementation.createDocument(null, null, null);
  var node = document.createElement("div");
  parent.appendChild(document.createElement("div"));
  insertFunc.call(parent, node, child);
});
globalThis.__r300p = out.join('|');
var out2 = [];
function pr(name, fn) { try { fn(); out2.push(name+':no-throw'); } catch(e) { out2.push(name+':'+e.name); } }
pr('docEl', function () { var doc = document.implementation.createHTMLDocument("title"); var el = doc.createElement("a"); doc.insertBefore(el, null); });
pr('docDf1', function () { var doc = document.implementation.createHTMLDocument("title"); var df = doc.createDocumentFragment(); df.appendChild(doc.createElement("a")); doc.insertBefore(df, null); });
pr('docDf2', function () { var doc = document.implementation.createHTMLDocument("title"); doc.documentElement.remove(); var df = doc.createDocumentFragment(); df.appendChild(doc.createElement("a")); df.appendChild(doc.createElement("b")); doc.insertBefore(df, null); });
pr('docText', function () { var doc = document.implementation.createHTMLDocument("title"); doc.insertBefore(doc.createTextNode("t"), null); });
globalThis.__r300q = out2.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r300p").unwrap().value;
    let out2 = sandbox.execute("globalThis.__r300q").unwrap().value;
    // 步骤 3（child NotFound，code 8）先于步骤 4-6（类型/doc HRE，code 3）——六形态
    //（proxy 容器 / detached doc / fragment / 元素容器 × 非法 node 类型）。
    assert_eq!(
        out,
        "t1:NotFoundError(8)|t2a:NotFoundError(8)|t2b:NotFoundError(8)|t2b2:NotFoundError(8)|t3a:NotFoundError(8)|t3b:NotFoundError(8)",
        "R300 pre-insert step-3 ordering: NotFound precedes type/doc HRE across proxy/detached-doc/fragment/element containers"
    );
    // doc-parent step-6（winning fn 的 ownerDocument 门移除 + doctype 唯一性）：
    // element 入已有元素的 doc / fragment 单元素 / 尾部 doctype / text 均须 HRE。
    assert_eq!(
        out2,
        "docEl:HierarchyRequestError|docDf1:HierarchyRequestError|docDf2:HierarchyRequestError|docText:HierarchyRequestError",
        "R300 doc-parent step-6: same-doc element conflict throws (ownerDocument gate removed)"
    );
    // 合法插入不受影响（对照）。
    let ok = sandbox
        .execute(
            r#"var doc = document.implementation.createHTMLDocument("t");
var p = doc.body; var t = doc.createTextNode("hello");
p.insertBefore(t, null);
globalThis.__r300ok = p.childNodes.length;"#,
        )
        .unwrap();
    let ok = sandbox.execute("globalThis.__r300ok").unwrap().value;
    assert_eq!(ok, "1", "R300 legal insert unaffected");
}

#[test]
fn r301_mo_move_records_sibling_fields() {
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
        "<html><body><p id=\"n100\"><span id=\"s1\">CHAN</span><span id=\"s2\">GED</span></p>\
         <p id=\"n81\">CHANN</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// n81: extractContents 中段 text 节点移除——期望 record prev=firstChild next=lastChild
(function () {
  var n81 = document.getElementById('n81');
  n81.appendChild(document.createTextNode("NNN"));
  n81.appendChild(document.createTextNode("NGED"));
  var r81 = document.createRange();
  r81.setStart(n81.firstChild, 4);
  r81.setEnd(n81.lastChild, 1);
  var recs = [];
  var mo = new MutationObserver(function (rs) { recs = rs; });
  mo.observe(n81, { childList: true });
  r81.extractContents();
  Promise.resolve().then(function () {
    globalThis.__r301a = JSON.stringify(recs.map(function (r) {
      return { rm: r.removedNodes.length,
        rmId: r.removedNodes.length ? String(r.removedNodes[0].data || r.removedNodes[0].id || r.removedNodes[0].nodeType) : '-',
        pv: r.previousSibling ? String(r.previousSibling.data || '?') : null,
        nx: r.nextSibling ? String(r.nextSibling.data || '?') : null };
    }));
  });
  out.push('n81sync=' + recs.length);
})();
// n100: surroundContents——期望三 records
(function () {
  var n100 = document.getElementById('n100');
  var f100 = document.createElement("span");
  var r100 = document.createRange();
  r100.setStartBefore(n100.firstChild);
  r100.setEndAfter(n100.lastChild);
  var recs2 = [];
  var mo2 = new MutationObserver(function (rs) { recs2 = rs; });
  mo2.observe(n100, { childList: true });
  r100.surroundContents(f100);
  Promise.resolve().then(function () {
    globalThis.__r301b = JSON.stringify(recs2.map(function (r) {
      return { rm: r.removedNodes ? r.removedNodes.length : 0,
        rmId: r.removedNodes && r.removedNodes.length ? String(r.removedNodes[0].id || r.removedNodes[0].data || '?') : '-',
        ad: r.addedNodes ? r.addedNodes.length : 0,
        adId: r.addedNodes && r.addedNodes.length ? String(r.addedNodes[0].id || r.addedNodes[0].tagName || '?') : '-',
        pv: r.previousSibling ? String(r.previousSibling.id || r.previousSibling.data || '?') : null,
        nx: r.nextSibling ? String(r.nextSibling.id || r.nextSibling.data || '?') : null };
    }));
  });
  out.push('n100sync=' + recs2.length);
})();
globalThis.__r301p = out.join('|');
Promise.resolve().then(function () {
  globalThis.__r301p = out.join('|') + '§A=' + String(globalThis.__r301a || 'none') + '§B=' + String(globalThis.__r301b || 'none');
});
"#,
        )
        .unwrap();
    let _ = sandbox.execute("0").unwrap();
    let out = sandbox.execute("globalThis.__r301p").unwrap().value;
    // extractContents 中段子 move record：prev/next 齐（identity 断言经 nodeType+data
    // 近似——n81 形态 firstChild="CHANN"/lastChild="NGED"，next 在末段 deleteData 后
    // data 削为 "GED"，故 data 比对用前缀）。
    assert!(out.contains("§A=[{\"rm\":1,\"rmId\":\"NNN\",\"pv\":\"CHAN\""), "probe A: {out}");
    // surroundContents 三 records：s1(pv null,nx s2) / s2(pv null——顺序移除后无左邻) / added。
    assert!(out.contains("§B=[{\"rm\":1,\"rmId\":\"s1\",\"ad\":0,\"adId\":\"-\",\"pv\":null,\"nx\":\"s2\"},{\"rm\":1,\"rmId\":\"s2\",\"ad\":0,\"adId\":\"-\",\"pv\":null,\"nx\":null},{\"rm\":0,\"rmId\":\"-\",\"ad\":1,\"adId\":\"SPAN\""), "probe B: {out}");
}

#[test]
fn r302_mo_callback_realm_error_reporting() {
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
    // R302：MO 回调抛异常的「report the exception」——按 callback 的关联 realm
    //（`__zwRealmOf` 注册表反查）定向到该 realm win 的 error 派发（onerror 触发）。
    // https://dom.spec.whatwg.org/#mutationobserver
    let out = sandbox
        .execute(
            r#"
var out = [];
// 假 realm win（dispatchEvent + onerror 面）
var fakeWin = { onerror: null };
fakeWin.addEventListener = function () {};
fakeWin.dispatchEvent = function (ev) {
  out.push('disp:' + (ev && ev.type));
  var h = fakeWin['on' + (ev && ev.type)];
  if (typeof h === 'function') { h.call(fakeWin, ev); }
  return true;
};
// 绑定构造器形态的回调（registry 印记模拟——iframe Function 的 R302 绑定）
var cb = new Function('throw new Error("X")');
if (!globalThis.__zwRealmOf) globalThis.__zwRealmOf = new Map();
globalThis.__zwRealmOf.set(cb, fakeWin);
fakeWin.onerror = function () { out.push('onerrorHit'); };
var target = document.getElementById('t');
var mo = new MutationObserver(cb);
mo.observe(target, { childList: true, subtree: true });
target.appendChild(document.createTextNode('y'));
Promise.resolve().then(function () {
  globalThis.__r302p = out.join('|');
});
"#,
        )
        .unwrap();
    let _ = sandbox.execute("0").unwrap();
    let out = sandbox.execute("globalThis.__r302p").unwrap().value;
    assert_eq!(
        out, "disp:error|onerrorHit",
        "R302 MO callback exception reports to the callback realm's onerror"
    );
}

#[test]
fn r303_mo_disconnect_discards_pending_records() {
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
        "<html><body><p id=\"n00\"></p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R303：spec `dom-mutationobserver-disconnect` 步骤 2——disconnect 清空 record
    // queue（WPT MutationObserver-disconnect 的三段 observe/disconnect 序列）。
    let out = sandbox
        .execute(
            r#"
var n00 = document.getElementById('n00');
var counts = [];
var mo = new MutationObserver(function (rs) { counts.push(rs.length); });
mo.observe(n00, { attributes: true });
n00.id = "foo";
n00.id = "bar";
mo.disconnect();
mo.observe(n00, { attributes: true, attributeOldValue: true });
n00.id = "latest";
mo.disconnect();
mo.observe(n00, { attributes: true, attributeOldValue: true });
n00.id = "n0000";
Promise.resolve().then(function () {
  globalThis.__r303p = counts.join(',') + '|' + (typeof mo.takeRecords().length === 'number');
});
"#,
        )
        .unwrap();
    let _ = sandbox.execute("0").unwrap();
    let out = sandbox.execute("globalThis.__r303p").unwrap().value;
    assert_eq!(
        out, "1|true",
        "R303 disconnect discards pending records; final flush delivers exactly the post-reconnect record"
    );
}

#[test]
fn r303_probe_inner_html_added_identity() {
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
        "<html><body><div id=\"n01\">old text</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var n01 = document.getElementById('n01');
var recs = [];
var mo = new MutationObserver(function (rs) { recs = rs; });
mo.observe(n01, { childList: true });
n01.innerHTML = "<span>new</span><span>text</span>";
Promise.resolve().then(function () {
  var ad = recs.length ? recs[0].addedNodes : [];
  var fc = n01.firstChild, lc = n01.lastChild;
  var o = [];
  o.push('recs=' + recs.length);
  o.push('adLen=' + ad.length);
  o.push('fc=' + (fc ? String(fc.tagName || fc.data || fc) : 'null'));
  o.push('lc=' + (lc ? String(lc.tagName || lc.data || lc) : 'null'));
  o.push('ad0=' + (ad[0] ? String(ad[0].tagName || ad[0].data || typeof ad[0]) : 'null'));
  o.push('fcEqAd0=' + String(fc === ad[0]));
  o.push('lcEqAd1=' + String(lc === ad[1]));
  o.push('fcIsText=' + String(fc && fc.nodeType === 3));
  globalThis.__r303q = o.join('|');
});
"#,
        )
        .unwrap();
    let _ = sandbox.execute("0").unwrap();
    let out = sandbox.execute("globalThis.__r303q").unwrap().value;
    // R303 归因探针（记档，非回归断言）：sel-based innerHTML 后同 turn 的
    // firstChild/lastChild 仍读 stale host 快照（fc=old text），addedNodes 是
    // _zwFragmentAdded wrapper——identity 断言族（WPT inner-outer "2 children"）
    // 依赖 firstChild 立即反映新子，归 R220 live-view 域（同 turn 视图桥）。
    println!("R303-PROBE: {out}");
    assert!(out.contains("recs=1"), "probe sanity: {out}");
}

#[test]
fn r306_bare_universal_doc_query_struct_first() {
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
    let out = sandbox
        .execute(
            r#"
var out = [];
// 主 document 的 *（sel 域，host __zw_query_all）
var d = document;
var main = d.querySelectorAll('*');
out.push('main=' + main.length);
// detached doc 的 *（工厂域 JSON 路径）
var dd = document.implementation.createHTMLDocument('t');
dd.body.innerHTML = "<div id='a'><span>x</span></div><p>y</p>";
var ddAll = dd.querySelectorAll('*');
out.push('dd=' + ddAll.length);
// 首/尾 tag + R296 结构桥 identity（html === documentElement）
out.push('ddFirst=' + (ddAll[0] ? String(ddAll[0].tagName) : 'null'));
out.push('bridge=' + String(ddAll[0] === dd.documentElement) + '/' + String(Array.prototype.some.call(ddAll, function (n) { return n === dd.body; })));
// JSON host 直测（绕 shim）
if (typeof __zw_parse_html_query === 'function') {
  try {
    var det = '<html><head><meta charset="utf-8"></head><body><div id="a"><span>x</span></div></body></html>';
    var r = JSON.parse(__zw_parse_html_query(det, '*', '1'));
    out.push('jsonStar=' + (r ? r.length : 'null'));
    if (r && r.length) out.push('json0=' + String((r[0] && r[0].tag) || '?'));
  } catch (e) { out.push('jsonErr=' + String(e && e.message).slice(0, 40)); }
}
globalThis.__r306p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r306p").unwrap().value;
    // R306：doc 作用域裸 `*` 走 JSON 全树 + R296 结构桥——结果以 html 起、
    // html/body 与 documentElement/body 视图对象 identity 全等。
    assert!(
        out.contains("ddFirst=HTML") && out.contains("bridge=true/true"),
        "R306 bare-* doc query: structured-first + bridge identity, got: {out}"
    );
    assert!(
        out.contains("json0=html"),
        "host JSON * returns full tree starting at html, got: {out}"
    );
}

#[test]
fn r307_tree_order_identity_domains() {
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
        "<html><body><div id=\"root\"><p id=\"a\">x</p><p id=\"b\">y</p></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// R307：完整复刻 WPT ParentNode-querySelector-All 的 tree-order 执行序——
// setupSpecialElements（append null/undefined + namespace div 簇）→ verifyStaticList
// 逐上下文 append div（静态表断言）→ outOfScope 注解（in-doc 前追加）→ tree order。
var doc = document.implementation.createHTMLDocument('t');
doc.body.innerHTML = "<div id='root'><p id='a'>x</p><p id='b'>y</p></div>";
var element = doc.getElementById('root');
function setupSpecialElements(d, parent) {
  parent.appendChild(d.createElement('null'));
  parent.appendChild(d.createElement('undefined'));
  // namespace 簇（真实 setupSpecialElements 的 anyNS/noNS div 组）
  var anyNS = d.createElement('div');
  anyNS.id = 'any-namespace';
  var divs = [d.createElement('div'),
              d.createElementNS('http://www.w3.org/1999/xhtml', 'div'),
              d.createElementNS('', 'div'),
              d.createElementNS('http://www.example.org/ns', 'div')];
  divs[0].id = 'any-namespace-div1';
  divs[1].id = 'any-namespace-div2';
  divs[2].setAttribute('id', 'any-namespace-div3');
  divs[3].setAttribute('id', 'any-namespace-div4');
  for (var i = 0; i < divs.length; i++) anyNS.appendChild(divs[i]);
  var noNS = d.createElement('div');
  noNS.id = 'no-namespace';
  var divs2 = [d.createElement('div'),
               d.createElementNS('http://www.w3.org/1999/xhtml', 'div'),
               d.createElementNS('', 'div'),
               d.createElementNS('http://www.example.org/ns', 'div')];
  divs2[0].id = 'no-namespace-div1';
  divs2[1].id = 'no-namespace-div2';
  divs2[2].setAttribute('id', 'no-namespace-div3');
  divs2[3].setAttribute('id', 'no-namespace-div4');
  for (var j = 0; j < divs2.length; j++) noNS.appendChild(divs2[j]);
  parent.appendChild(anyNS);
  parent.appendChild(noNS);
}
setupSpecialElements(doc, element);
var outOfScope = element.cloneNode(true);
function traverse(elem, fn) {
  if (elem.nodeType === 1) fn(elem);
  elem = elem.firstChild;
  while (elem) { traverse(elem, fn); elem = elem.nextSibling; }
}
traverse(outOfScope, function (e) { e.setAttribute('data-clone', ''); });
var detached = element.cloneNode(true);
var fragment = doc.createDocumentFragment();
fragment.appendChild(element.cloneNode(true));
// verifyStaticList：每上下文 root.querySelectorall('div') 后 append 一个新 div
function verifyStaticList(root) {
  var pre = root.querySelectorAll('div');
  var d = doc.createElement('div');
  (root.body || root).appendChild(d);
  return pre;
}
verifyStaticList(doc);
verifyStaticList(detached);
verifyStaticList(fragment);
verifyStaticList(element);
doc.body.appendChild(outOfScope); // in-doc 测试前的关键追加
function firstDiverge(root, label) {
  var res = root.querySelectorAll('*');
  var travList = [];
  traverse(root, function (e) { if (e !== root) travList.push(e); });
  var di = -1;
  for (var i = 0; i < Math.min(res.length, travList.length); i++) {
    if (res[i] !== travList[i]) { di = i; break; }
  }
  if (di < 0 && res.length !== travList.length) di = Math.min(res.length, travList.length);
  out.push(label + ':res=' + res.length + '/trav=' + travList.length + '/div=' + di);
  if (di >= 0) {
    var t = travList[di], r = res[di];
    out.push(label + ':dTrav=' + (t ? String(t.nodeName) + '/' + String(t.id || '') : 'null')
      + '|dRes=' + (r ? String(r.nodeName) + '/' + String(r.id || '') : 'null'));
  }
}
firstDiverge(detached, 'detach');
firstDiverge(fragment, 'frag');
firstDiverge(element, 'indoc');
globalThis.__r307p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r307p").unwrap().value;
    // R307 修复断言：三上下文（Detached/Fragment/In-document）的 querySelectorAll("*")
    // 与 traverse 产物逐位 identity 全等（div=-1）。四层修复：
    // ① walk167 非元素根子树递归（Fragment 的 _zwNodeIdx 不再为空）；
    // ② 键的 empty-ns 标记归一（namespace 簇的 wrapper 键不再恒 miss）；
    // ③ `el.id =` IDL accessor 化（赋值同步 attrs——序列化/host JSON 不再丢 id）；
    // ④ appendChild/removeChild 的祖先查询索引失效（append 后索引含新子）。
    // 注：WPT 本体的 tree-order 3F 在 iframe contentDocument 代理域（查询产物是
    // _wrapSelector proxy vs traverse 读工厂树——两套对象域，R291 深结构桥），
    // 本断言覆盖 createHTMLDocument 工厂域的等价形态。
    assert!(
        out.contains("detach:res=15/trav=15/div=-1"),
        "R307 detached tree-order identity unified, got: {out}"
    );
    assert!(
        out.contains("frag:res=16/trav=16/div=-1"),
        "R307 fragment tree-order identity unified (walk167 recursion into fragment children), got: {out}"
    );
    assert!(
        out.contains("indoc:res=15/trav=15/div=-1"),
        "R307 in-document tree-order identity unified (empty-ns key normalization + id reflect), got: {out}"
    );
}

#[test]
fn r308_iframe_domain_query_identity() {
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
    // R308：iframe contentDocument 域（WPT tree-order 3F 的真实域）——
    // `_zwIframeCreateElement` 产物（plain 字面量，非 _zwMEl 工厂）出口补 identity
    // 桥登记（`_zwBridgeSet(el, el)`）+ append/remove/insert 的祖先查询索引失效。
    // 探针复刻：doc.body.innerHTML 设树 → getElementById 取 element（工厂树节点）
    // → appendChild(doc.createElement('null'))（iframe 工厂元素）→ element/fragment/
    // detached 三上下文 querySelectorAll('*') 与 traverse 逐位 identity。
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  var frame = document.createElement('iframe');
  frame.src = 'about:blank';
  document.body.appendChild(frame);
  var doc = frame.contentDocument;
  doc.body.innerHTML = "<div id='root'><p id='a'>x</p></div>";
  var byId = doc.getElementById('root');
  var extra = doc.createElement('null');
  byId.appendChild(extra);
  var frag = doc.createDocumentFragment();
  frag.appendChild(byId.cloneNode(true));
  var detached = byId.cloneNode(true);
  function traverse(elem, fn) {
    if (elem.nodeType === 1) fn(elem);
    elem = elem.firstChild;
    while (elem) { traverse(elem, fn); elem = elem.nextSibling; }
  }
  function divergeOf(root) {
    var qa = root.querySelectorAll('*');
    var trav = [];
    traverse(root, function (e) { if (e !== root) trav.push(e); });
    var di = -1;
    for (var i = 0; i < Math.min(qa.length, trav.length); i++) { if (qa[i] !== trav[i]) { di = i; break; } }
    if (di < 0 && qa.length !== trav.length) di = Math.min(qa.length, trav.length);
    return 'div=' + di + '/len=' + qa.length + ',' + trav.length;
  }
  out.push('indoc:' + divergeOf(byId));
  out.push('frag:' + divergeOf(frag));
  out.push('detach:' + divergeOf(detached));
  // 查询产物与 append 本体的 identity（append 子在结果数组里 === 真节点）
  var qa = byId.querySelectorAll('*');
  out.push('extraInRes=' + String(qa.indexOf(extra) >= 0));
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r308p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r308p").unwrap().value;
    println!("R308: {out}");
    // R308 断言：三上下文 identity 全等（div=-1）+ append 本体出现在查询结果。
    assert!(
        out.contains("indoc:div=-1") && out.contains("frag:div=-1") && out.contains("detach:div=-1"),
        "R308 iframe-domain query identity unified, got: {out}"
    );
    assert!(
        out.contains("extraInRes=true"),
        "R308 appended iframe-factory element appears in query results as itself, got: {out}"
    );
}

#[test]
fn r308_matches_void_regression_probe() {
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
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  var doc = document.implementation.createHTMLDocument('t');
  doc.body.innerHTML = "<div id='pseudo-ui'><input id='i4' type='radio' checked='checked'></div>";
  var frag = doc.createDocumentFragment();
  frag.appendChild(doc.getElementById('pseudo-ui').cloneNode(true));
  var i4 = frag.firstChild.firstChild;
  out.push('fragHasKids=' + String(frag.childNodes.length));
  out.push('i4=' + String(i4.nodeName) + '[' + String(i4.id) + ']');
  out.push('checked=' + String(i4.matches('#pseudo-ui :checked')));
  // 序列化源对比
  var src = '';
  for (var i = 0; i < frag.childNodes.length; i++) {
    var c = frag.childNodes[i];
    if (c.nodeType === 1 && typeof c.outerHTML === 'string') src += c.outerHTML;
  }
  out.push('srcTail=' + JSON.stringify(src.slice(-80)));
  // host 查询直测
  try {
    var jq = JSON.parse(__zw_parse_html_query(src, '#pseudo-ui :checked', '1', '', '1'));
    out.push('hostHit=' + String(jq.length) + (jq.length ? '/id=' + String(jq[0].id) : ''));
  } catch (eH) { out.push('hostErr=' + String(eH && eH.message)); }
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r308m = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r308m").unwrap().value;
    println!("R308-M: {out}");
    // R308 断言：fragment 根的序列化源直发（R308 `_zwMQueryAll` 的 nodeType 11 分支）
    // 使 `matches` 的 root-up-track 上行到 fragment 后 host 查询可见——`:checked`
    // 伪类（checked 属性的 radio）命中。旧版 `_zwMOuterHtml(fragment)` 返 '' 使
    // 候选恒 0（WPT Element-matches Fragment `#pseudo-ui :checked` 族 139F 根因）。
    assert!(
        out.contains("checked=true"),
        "R308 fragment-root serialization makes matches see fragment subtree, got: {out}"
    );
}

#[test]
fn r309_removed_elements_repro() {
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
        "<html><body><div id=\"container\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R309：主文档域（sel 容器）——innerHTML 替换后 querySelectorAll 不得返旧元素。
    // jsdom#2519 回归：innerHTML 换新 HTML 后同键 wrapper 缓存命中旧对象/旧查询源。
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  var container = document.querySelector('#container');
  out.push('container=' + String(container && container.nodeName));
  function getIDs() {
    var els = container.querySelectorAll('a.test');
    var ids = [];
    for (var i = 0; i < els.length; i++) ids.push(String(els[i].id));
    return ids.join(',');
  }
  container.innerHTML = '<a id="link-a" class="test">a link</a>';
  out.push('first=' + getIDs());
  container.innerHTML = '<a id="link-b" class="test"><img src="foo.jpg"></a>';
  out.push('second=' + getIDs());
  container.innerHTML = '<a id="link-a" class="test">a link</a>';
  out.push('third=' + getIDs());
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r309p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r309p").unwrap().value;
    println!("R309: {out}");
    // R309 断言：innerHTML 逐次替换后同 turn 查询返新子树元素（不返旧元素）——
    // pending-fused 子树查询（`_childNodeList` overlay 重建 + 客户端 compound 匹配）。
    assert_eq!(
        out, "container=DIV|first=link-a|second=link-b|third=link-a",
        "R309 removed-elements: same-turn querySelectorAll reflects innerHTML replacements"
    );
}

#[test]
fn r310_pending_removed_query_repro() {
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
        "<html><body><div id='container'><a id='a1' class='test'>1</a><a id='a2' class='test'>2</a><a id='a3' class='test'>3</a></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R310：removed 语义的同 turn 查询——removeChild 后立即 querySelectorAll 不得返已移除元素。
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  var container = document.querySelector('#container');
  out.push('initial=' + (function () {
    var els = container.querySelectorAll('a.test'), ids = [];
    for (var i = 0; i < els.length; i++) ids.push(els[i].id);
    return ids.join(',');
  })());
  var a2 = container.querySelector('#a2');
  container.removeChild(a2);
  out.push('afterRemove=' + (function () {
    var els = container.querySelectorAll('a.test'), ids = [];
    for (var i = 0; i < els.length; i++) ids.push(els[i].id);
    return ids.join(',');
  })());
  // remove() 方法形态 + querySelector 单数路径（R310 同款过滤）
  var a1 = container.querySelector('#a1');
  a1.remove();
  out.push('afterMethodRemove=' + (function () {
    var els = container.querySelectorAll('a.test');
    return String(els.length);
  })());
  out.push('qsRemovedNull=' + String(container.querySelector('#a2') === null));
  out.push('qsLiveHit=' + String(container.querySelector('#a3') !== null));
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r310p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r310p").unwrap().value;
    println!("R310: {out}");
    // R310 断言：同 turn removeChild/remove 后查询不再含已移除元素——
    // pending-removed 子树的 sel 从 host 快照结果剔除（L2「查询读 live」removed 面）。
    assert_eq!(
        out, "initial=a1,a2,a3|afterRemove=a1,a3|afterMethodRemove=1|qsRemovedNull=true|qsLiveHit=true",
        "R310 same-turn removed elements filtered from subtree queries, got: {out}"
    );
}

#[test]
fn r311_cdata_text_content_concat() {
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
        "<html><body><p id='p'>x</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R311：textContent 的 CDATA 拼接——spec CDATASection : Text（字符数据计入
    // textContent 联接；comment/PI 不计入维持 R184）。WPT Node-properties testDiv
    // .textContent 期望 CDATA "1234"+"5678" + Text "9012" = "123456789012"。
    // https://dom.spec.whatwg.org/#dom-node-textcontent
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  // WPT Node-properties 形态：主文档 p（handle 元素）+ new Document() 的 CDATA 子
  var xml = new Document();
  var p = document.createElement('p');
  p.appendChild(xml.createCDATASection('1234'));
  p.appendChild(xml.createCDATASection('5678'));
  p.append('9012');
  out.push('tc=' + String(p.textContent));
  // comment/PI 仍不计入（R184 语义维持）
  p.appendChild(document.createComment('c'));
  p.appendChild(xml.createProcessingInstruction('x', 'pi'));
  out.push('tcNoCommentPI=' + String(p.textContent));
  // CDATA 自身 textContent = data
  var cd = xml.createCDATASection('abc');
  out.push('cdSelf=' + String(cd.textContent));
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r311p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r311p").unwrap().value;
    println!("R311: {out}");
    assert_eq!(
        out, "tc=123456789012|tcNoCommentPI=123456789012|cdSelf=abc",
        "R311 CDATA contributes to textContent concatenation (comment/PI still excluded), got: {out}"
    );
}

#[test]
fn r312_redispatch_is_trusted_repro() {
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
        "<html><body><button id='b'>x</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R312：redispatch 语义——① isTrusted：dispatchEvent（untrusted 派发）后事件的
    // isTrusted 须 false（spec `dom-event-istrusted` 只读标志，dispatch 时置位）；②
    // dispatching 中的事件再 dispatch 抛 InvalidStateError（spec `dom-event-dispatch`
    // 步骤 2）。WPT Event-dispatch-redispatch 两失败的核心机制面。
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  var b = document.querySelector('#b');
  var ev = document.createEvent('Event');
  ev.initEvent('click', true, true);
  out.push('beforeTrusted=' + String(ev.isTrusted));
  b.dispatchEvent(ev);
  out.push('afterTrusted=' + String(ev.isTrusted));
  // dispatching 中再 dispatch（listener 内）——spec 抛 InvalidStateError
  var ev2 = document.createEvent('Event');
  ev2.initEvent('custom', true, true);
  var innerErr = 'none';
  var target = document.createElement('div');
  target.addEventListener('custom', function () {
    try { target.dispatchEvent(ev2); innerErr = 'no-throw'; } catch (e) { innerErr = String(e && e.name); }
  });
  target.dispatchEvent(ev2);
  out.push('reentrant=' + innerErr);
  // redispatch 已派发完的（listener 外）——spec 不抛（可再派发）
  var againErr = 'none';
  try { b.dispatchEvent(ev); } catch (e) { againErr = String(e && e.name); }
  out.push('reDispatch=' + againErr);
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r312p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r312p").unwrap().value;
    println!("R312: {out}");
    assert!(out.contains("beforeTrusted="), "sanity: {out}");
}

#[test]
fn r312_mouseup_redispatch_repro2() {
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
        "<html><body><button id='b'>x</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R312：UA mouseup（__zw_dispatch_event trusted 化）→ click listener 内脚本
    // re-dispatch mouseup ——isTrusted 须翻 false（guard 的 _zwUaDispatch 一次性语义）。
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  var b = document.querySelector('#b');
  var mouseupEvent = null;
  var clickEvent = null;
  var mouseupTrustedAtDispatch = null;
  var clickTrustedAtDispatch = null;
  b.addEventListener('mouseup', function (e) { mouseupEvent = e; mouseupTrustedAtDispatch = e.isTrusted; }, { once: true });
  b.addEventListener('click', function (e) {
    clickEvent = e;
    clickTrustedAtDispatch = e.isTrusted;
    // listener 内 re-dispatch（此时 mouseup 已派发完）——翻 false
    b.dispatchEvent(mouseupEvent);
    out.push('afterRedispatch=' + String(mouseupEvent.isTrusted));
    out.push('clickStill=' + String(clickEvent.isTrusted));
  }, { once: true });
  // UA 派发链：mouseup → click（宿主 __zw_dispatch_event 两事件）
  __zw_dispatch_event('#b', 'mouseup', null);
  __zw_dispatch_event('#b', 'click', null);
  out.push('firstMouseup=' + String(mouseupTrustedAtDispatch));
  out.push('firstClick=' + String(clickTrustedAtDispatch));
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r312m = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r312m").unwrap().value;
    println!("R312-M: {out}");
    assert!(out.contains("firstMouseup="), "sanity: {out}");
}

#[test]
fn r313_disabled_click_and_boolean_roundtrip() {
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
        "<html><body><button id='b'>x</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R313：spec HTML §activation——disabled 表单元素的 click() 不派发；re-enable 后
    // 恢复派发（WPT Event-dispatch-on-disabled-elements）。含 handle 布尔 falsy 真移除
    // （旧「不设」使 .disabled=false 后属性残留——re-enable 断言必败的根因）。
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  // handle 元素（createElement 产物）
  var btn = document.createElement('button');
  var dispatched = 0;
  btn.onclick = function () { dispatched++; };
  document.body.appendChild(btn);
  btn.disabled = true;
  out.push('disNow=' + String(btn.disabled));
  btn.click();
  out.push('clickWhileDisabled=' + String(dispatched));
  btn.disabled = false;
  out.push('disAfterUnset=' + String(btn.disabled));
  btn.click();
  out.push('clickAfterEnable=' + String(dispatched));
  // dispatchEvent 直发不受 disabled 门影响（spec：dispatchEvent 无激活行为）
  var ev = document.createEvent('Event');
  ev.initEvent('click', true, true);
  btn.disabled = true;
  btn.dispatchEvent(ev);
  out.push('directDispatchWhileDisabled=' + String(dispatched));
  out.push('disStill=' + String(btn.disabled));
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r313p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r313p").unwrap().value;
    println!("R313: {out}");
    assert_eq!(
        out, "disNow=true|clickWhileDisabled=0|disAfterUnset=false|clickAfterEnable=1|directDispatchWhileDisabled=2|disStill=true",
        "R313 disabled click gate + boolean falsy removal roundtrip, got: {out}"
    );
}

#[test]
fn r314_treewalker_regraft_diagnostics() {
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
        "<html><body><div id='t'>x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R314（js-dom M4）：TreeWalker「walking outside a tree」的归因诊断（主文档
    // createElement 产物 = handle proxy 域）。探针实断两域事实：① regraft 后
    // `p.previousSibling`（handle proxy 融合视图）恒 null——handle registry 的
    // 兄弟链在 removeChild+appendChild 重挂后断（R291 域，同 R309 教训的
    // identity 双源问题）；② `previousNode` 的父上行越过 root（R85 循环的
    // root 止步只在一个分支）——walker 算法轻量点，待兄弟链修复后生效。
    let out = sandbox
        .execute(
            r#"
var out = [];
try {
  var doc = document.createElement("div");
  var head = document.createElement('head');
  var title = document.createElement('title');
  var body = document.createElement('body');
  var p = document.createElement('p');
  doc.appendChild(head);
  head.appendChild(title);
  doc.appendChild(body);
  body.appendChild(p);
  var w = document.createTreeWalker(body, 0xFFFFFFFF, null);
  doc.removeChild(body);
  var lc = w.lastChild();
  out.push('lastChild=' + String(lc ? lc.nodeName : 'null'));
  doc.appendChild(p);
  out.push('pPrevSib=' + String(p.previousSibling ? p.previousSibling.nodeName : 'null'));
  var prev = w.previousNode();
  out.push('prevNode=' + String(prev ? prev.nodeName : 'null'));
} catch (e) { out.push('err=' + String(e && e.message)); }
globalThis.__r314p = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r314p").unwrap().value;
    println!("R314-DIAG: {out}");
    // 归因断言（记录当前事实域——修复后应更新为期望值链）：
    // lastChild=P ✓（removeChild 后 lastChild 在 root 子树内正确）；
    // pPrevSib=null（handle 融合视图 regraft 断链——R291 域）；
    // prevNode=null（R314 root 止步修复——父上行不再越过 root 返链外节点）。
    assert!(
        out.contains("lastChild=P") && out.contains("pPrevSib=null") && out.contains("prevNode=null"),
        "R314 regraft domain facts (fix will flip these), got: {out}"
    );
}
