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
