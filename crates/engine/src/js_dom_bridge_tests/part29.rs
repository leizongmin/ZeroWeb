// js_dom_bridge 测试切片 29。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// WC-M1 切片 3（web-components goal）：CEReactions 反应链全语义面。
/// 覆盖：① setAttribute/反射 setter 的 attributeChanged（4 参签名带 namespace=null）；
/// ② NS 变体（setAttributeNS/removeAttributeNS）带 ns 派发；③ 跨文档
/// disconnected→adopted→connected 序（spec `concept-node-adopt` + custom-element-reactions）；
/// ④ template content 视图的 mutation 面 + contents owner document。
#[test]
fn wc_m1_ce_reactions_and_adopt() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 主沙箱（v8 native 桥）：主文档 handle proxy 面。
    let js = r#"
var out = [];
var log = [];
function L(s) { log.push(s); }
class Ce extends HTMLElement {
  static get observedAttributes() { return ['id', 'aria-label']; }
  attributeChangedCallback(n, o, v, ns) { L('attr:' + n + ':' + o + '->' + v + ':' + ns); }
  connectedCallback() { L('connected'); }
  disconnectedCallback() { L('disconnected'); }
  adoptedCallback(od, nd) { L('adopted:' + (od === document ? 'main' : 'other') + '>' + (nd === document ? 'main' : 'other')); }
}
customElements.define('wc-ce-el', Ce);
function snap(name) { out.push(name + '[' + log.join(',') + ']'); log.length = 0; }

// A: setAttribute 反应 + 4 参 namespace=null
var a = document.createElement('wc-ce-el');
a.setAttribute('id', 'foo');
a.setAttribute('data-lang', 'en'); // unobserved → 无派发
a.setAttribute('id', 'bar');
snap('A-setattr');

// B: IDL 反射 setter（id → content attribute）
a.id = 'baz';
snap('B-id-reflection');

// C: NS 变体——移除（newValue null）+ 新增（old null），未 observed 的 data-x 不派发
try {
  a.setAttributeNS(null, 'data-x', '1');           // 未 observed → 不派发
  a.removeAttributeNS(null, 'id');                 // observed 移除 → 派发（old baz, new null）
  a.setAttributeNS(null, 'id', 'nsid');            // observed 新增 → 派发（old null）
} catch (eC1) { log.push('c1:' + eC1.name); }
snap('C-ns-set');

// D: 移除派发（newValue null）
a.removeAttribute('id');
snap('D-remove');

// E: 跨文档移动——主文档插入 → createHTMLDocument 移动 → disconnected/adopted/connected
var e = document.createElement('wc-ce-el');
document.body.appendChild(e);
log.length = 0;
var fdoc = document.implementation.createHTMLDocument('f');
fdoc.documentElement.appendChild(e);
snap('E-crossdoc-move');

// F: template content 视图 mutation 面 + contents owner document
var t = document.createElementNS('http://www.w3.org/1999/xhtml', 'template');
var tdoc = t.content.ownerDocument;
out.push('F-owner[' + (tdoc && tdoc !== document) + ']');
if (!tdoc.documentElement) tdoc.appendChild(tdoc.createElement('html'));
var f = document.createElement('wc-ce-factory-el');
class CeF extends HTMLElement { connectedCallback() { L('f-connected'); } adoptedCallback() { L('f-adopted'); } disconnectedCallback() { L('f-disconnected'); } }
customElements.define('wc-ce-factory-el', CeF);
var inst = document.createElement('wc-ce-factory-el');
document.body.appendChild(inst);
log.length = 0;
tdoc.documentElement.appendChild(inst);
snap('F-tpldoc-move');

out.join(' ; ');
"#;
    let out = sandbox.execute(js).unwrap().value;
    assert_eq!(
        out,
        "A-setattr[attr:id:null->foo:null,attr:id:foo->bar:null] ; \
         B-id-reflection[attr:id:bar->baz:null] ; \
         C-ns-set[attr:id:baz->null:null,attr:id:null->nsid:null] ; \
         D-remove[attr:id:nsid->null:null] ; \
         E-crossdoc-move[disconnected,adopted:main>other,connected] ; \
         F-owner[true] ; \
         F-tpldoc-move[f-disconnected,f-adopted,f-connected]",
        "CE 反应链 + 4 参 attributeChanged + 跨文档 adopted 序（spec 对齐）"
    );
}


/// WC-M1 切片 4（web-components goal）：customized built-ins + whenDefined 真等待 +
/// registry 按 document 隔离。
/// 覆盖：① createElement(localName, {is}) 升级（ctor 体 + is 内容属性）；
/// ② connected/attributeChanged 反应链对 customized built-in 生效；③ new klass()
/// 产生真实元素（localName = extends tag，prototype = klass.prototype）；
/// ④ whenDefined define 前挂起、define 后 microtask resolve；⑤ detached doc 的
/// createElement 不触发主 registry 升级（spec look up a custom element registry）。
#[test]
fn wc_m1_ce_customized_builtins() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = r#"
var out = [];
var log = [];
class CbiEl extends HTMLElement {
  static get observedAttributes() { return ['alt']; }
  constructor() { super(); log.push('constructed'); }
  connectedCallback() { log.push('connected'); }
  disconnectedCallback() { log.push('disconnected'); }
  attributeChangedCallback() { log.push('attr'); }
}
customElements.define('wc-cbi-img', CbiEl, { extends: 'img' });

// A: createElement(tag, {is}) 升级 + is 内容属性
var a = document.createElement('img', { is: 'wc-cbi-img' });
out.push('A-ctor:' + (a.constructor === CbiEl));
out.push('A-is:' + a.getAttribute('is'));
out.push('A-log:' + log.join(','));
log.length = 0;

// B: connected + attributeChanged 反应
document.body.appendChild(a);
out.push('B-append:' + log.join(','));
log.length = 0;
a.setAttribute('alt', 'x');
out.push('B-set:' + log.join(','));
log.length = 0;

// C: new klass() 真实元素
var c = new CbiEl();
out.push('C-new:' + (c.nodeType === 1) + ':' + (c.tagName === 'IMG') + ':' + (Object.getPrototypeOf(c) === CbiEl.prototype) + ':' + (c.getAttribute('is') === 'wc-cbi-img'));

// D: whenDefined 真等待
var wdState = 'pending';
customElements.whenDefined('wc-wd-el').then(function () { wdState = 'resolved'; });
class WdEl extends HTMLElement {}
customElements.define('wc-wd-el', WdEl);
out.push('D-sync:' + wdState);
Promise.resolve().then(function () { out.push('D-micro:' + wdState); });

// E: detached doc 的 createElement 不升级（registry 隔离）
var ddoc = document.implementation.createHTMLDocument('d');
var de = ddoc.createElement('wc-cbi-unregistered');
out.push('E-detached:' + (de.constructor !== CbiEl) + ':tag=' + de.tagName);

// F: autonomous define 后已有同名元素升级（spec define upgrade step）
var pre = document.createElement('wc-pre-exist');
document.body.appendChild(pre);
class PreEl extends HTMLElement { constructor() { super(); log.push('pre-constructed'); } }
customElements.define('wc-pre-exist', PreEl);
out.push('F-upgrade:' + (pre.constructor === PreEl) + ':' + log.join(','));
log.length = 0;

out.join(' ; ');
"#;
    let out = sandbox.execute(js).unwrap().value;
    // D-micro：microtask 在 execute turn 结束后跑——第二段 execute 读结果。
    let micro = sandbox.execute("'D-micro:' + wdState").unwrap().value;
    assert_eq!(
        out,
        "A-ctor:true ; A-is:wc-cbi-img ; A-log:constructed ; \
         B-append:connected ; B-set:attr ; \
         C-new:true:true:true:true ; \
         D-sync:pending ; \
         E-detached:true:tag=WC-CBI-UNREGISTERED ; \
         F-upgrade:true:constructed,constructed,attr,connected,constructed,attr,connected,pre-constructed",
        "customized built-ins + whenDefined + registry 隔离（spec 对齐）"
    );
    assert_eq!(micro, "D-micro:resolved",
        "whenDefined 在 define 后的 microtask resolve");
}

/// R4875：`Storage` 接口对象 + localStorage/sessionStorage 实例语义（HTML Web Storage）。
/// 站点脚本以 `Storage` 标识符作为 DI token / paramtypes 元数据引用（baidu aas.js 判例：
/// 缺失时类定义期 ReferenceError 同步中止整段 AMD 初始化链），并依赖
/// `instanceof Storage`、Storage.prototype 方法、named property 即存储项。
/// https://html.spec.whatwg.org/multipage/webstorage.html#the-storage-interface
#[test]
fn r4875_storage_interface_and_named_properties() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // ① Storage 全局为函数；直接调用/构造抛 TypeError（spec：无构造器，实例仅由
    // localStorage/sessionStorage 返回）。
    // ② localStorage/sessionStorage instanceof Storage；API 与 named property 双通道同域：
    // setItem 可经属性读回、属性写可经 getItem 读回；delete/removeItem 对称。
    // ③ length/key 全语义；两存储区相互隔离。
    // ④ Storage.prototype 方法补丁对实例生效（站点打补丁惯用法）。
    sandbox.execute(
        "try {\
         var r = [];\
         r.push(typeof Storage === 'function');\
         r.push(localStorage instanceof Storage);\
         r.push(sessionStorage instanceof Storage);\
         var ctorErr = '';\
         try { Storage(); } catch (e) { ctorErr = 'TypeError'; }\
         r.push(ctorErr);\
         localStorage.setItem('k1', 'v1');\
         r.push(localStorage.k1 === 'v1');\
         localStorage.k2 = 'v2';\
         r.push(localStorage.getItem('k2') === 'v2');\
         r.push(localStorage.length === 2);\
         r.push(localStorage.key(0) === 'k1' && localStorage.key(1) === 'k2');\
         r.push(localStorage.key(9) === null);\
         r.push(sessionStorage.getItem('k1') === null);\
         delete localStorage.k2;\
         r.push(localStorage.getItem('k2') === null && localStorage.length === 1);\
         var patched = false;\
         Storage.prototype.getItem = function (k) { patched = true; return 'patched'; };\
         r.push(localStorage.getItem('k1') === 'patched' && patched);\
         globalThis.__r4875 = r.join('|');\
         } catch (err) { globalThis.__r4875 = 'ERR:' + err.message; }",
    ).unwrap();
    let out = sandbox.execute("globalThis.__r4875").unwrap().value;
    assert_eq!(
        out,
        "true|true|true|TypeError|true|true|true|true|true|true|true|true",
        "R4875：Storage 接口 + 实例语义全断言面"
    );
}

/// R-baidu3 文档直读视图 + 增量链等价（baidu loader 风暴修复）：insertAdjacentHTML
/// 与 querySelectorAll 交错推进 count——每步查询结果必须等于全量重放参考
///（字符串路径每步 parse(732KB)+serialize 全量重放 × 数百步 = 宿主侧 15s 重解析
/// 风暴主力；文档路径每步只 parse 插入片段）。50 步长度阶梯 2..51 即逐步等价断言。
/// https://drafts.csswg.org/selectors-4/ + R57 FV M3 同批查询语义。
#[test]
fn test_query_view_doc_incremental_equivalence_r_baidu3() {
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
        "<html><body><ul id='list'><li>x</li></ul></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 每步插入推进 count 后立即查询（触发增量链步进），记录长度阶梯。
    let lens = sandbox
        .execute(
            "var lens = [];\
             for (var i = 0; i < 50; i++) {\
               document.querySelector('#list').insertAdjacentHTML('beforeend', '<li>li-' + i + '</li>');\
               lens.push(document.querySelectorAll('#list li').length);\
             }\
             lens.join(',')",
        )
        .unwrap()
        .value;
    let expected: Vec<String> = (2..=51).map(|n| n.to_string()).collect();
    assert_eq!(
        lens,
        expected.join(","),
        "插入/查询交错 50 步的长度阶梯必须逐步等于全量重放参考（增量链等价）"
    );
    // 末元素可见（链尾插入未丢）。
    let last = sandbox
        .execute("document.querySelectorAll('#list li')[50].textContent")
        .unwrap()
        .value;
    assert_eq!(last, "li-49", "链尾插入元素须在视图文档中可见");
}

/// R358/R3243 快照就地换代代际（DOM_VIEW_GEN）：快照 Arc 被回调捕获（Box<dyn Fn>
/// 不可达 ⇒ 无法换装新 Arc），`*snap = 新内容` 对 `(Arc, count)` 键视而不见——
/// 换代后同 count 查询命中换代前解析的视图。换代写入点 bump 代际 → 缓存整体失效。
#[test]
fn test_query_view_doc_inplace_snapshot_swap_gen_r358() {
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
        "<html><body><p id='a'>old</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 首查询：视图文档烘焙旧快照（count=0 条目）。
    let v1 = sandbox
        .execute("document.querySelector('#a') ? 'hit' : 'miss'")
        .unwrap()
        .value;
    assert_eq!(v1, "hit");
    // 就地换代（模拟 SetDomSnapshot 的 `*snap = html`——Arc 不变）+ 生产同款 bump。
    {
        let mut snap = dom_html.lock().unwrap();
        *snap = "<html><body><p id='b'>new</p></body></html>".to_string();
    }
    crate::js_dom_bridge::bump_dom_view_gen();
    // 同 count（0）查询必须见新快照——无代际项时命中 stale 视图返 miss（R358 形态）。
    let v2 = sandbox
        .execute("document.querySelector('#b') ? 'hit' : 'miss'")
        .unwrap()
        .value;
    assert_eq!(v2, "hit", "就地换代 + bump 后同 count 查询必须见新快照");
    let v3 = sandbox
        .execute("document.querySelector('#a') ? 'hit' : 'miss'")
        .unwrap()
        .value;
    assert_eq!(v3, "miss", "换代后旧快照元素不得再命中（无跨代污染）");
}

/// MUT_DRAIN_GEN drain 代际（增量链前提）：drain 后队列重新增长越过旧 count 时，
/// (Arc, count) 键会被误判为「只增长」——没有代际项会把新队列前段当已应用基座
///（错视图）。drain 站点 bump → 增量链作废 → 全量重建新基座。
#[test]
fn test_query_view_doc_drain_regrowth_gen_r_baidu3() {
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
        "<html><body><ul id='l1'><li>base</li></ul><ul id='l2'></ul></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 批 1：插入 2 条 → 查询烘焙视图（count=2，结构性已烘入）。
    sandbox
        .execute(
            "document.getElementById('l1').insertAdjacentHTML('beforeend', '<li>b1</li><li>b2</li>');\
             globalThis.__n1 = document.querySelectorAll('#l1 li').length;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__n1").unwrap().value, "3");
    // drain：host 批末清队列 + bump 代际 + apply 后快照回写（生产 drain 站点三件套：
    // `mutations.clear()` + `bump_mut_drain_gen` + `apply_pending_shared_mutations` 落
    // 活 DOM 后的 cached_html 快照换代）。
    mutations.lock().unwrap().clear();
    crate::js_dom_bridge::bump_mut_drain_gen();
    {
        let mut snap = dom_html.lock().unwrap();
        *snap = "<html><body><ul id='l1'><li>base</li><li>b1</li><li>b2</li></ul><ul id='l2'></ul></body></html>".to_string();
    }
    crate::js_dom_bridge::bump_dom_view_gen();
    // 批 2：队列重新增长越过旧 count（≥2）——必须全量重建，不得把批 2 队列前段
    // 当批 1 的已应用基座（错视图会把 #l2 插入错位/丢 #l1 烘焙项）。
    sandbox
        .execute(
            "document.getElementById('l2').insertAdjacentHTML('beforeend', '<li>c1</li>');\
             globalThis.__n2 = document.querySelectorAll('#l1 li').length + ':' + document.querySelectorAll('#l2 li').length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__n2").unwrap().value,
        "3:1",
        "drain 重建后批 1 烘焙项与批 2 插入必须并存（代际项防 stale 链）"
    );
}

/// MUT_DRAIN_GEN 精确命中碰撞（drain 代际回归补测试）：drain（clear + bump
/// drain 代际，**不**换代）后队列以**不同内容**重长回**恰好旧 count**——
/// 精确命中缺 drain_gen 项会把 pre-drain 视图原样端出（旧条目复活、新条目
/// 丢失）。生产形态：drain 站点三件套里 clear/bump 与 apply/快照换代之间的
/// 窗口内，查询落在重长回同 count 的时刻。drain_gen 必须进精确命中键。
///
/// 关键构造：drain 后的**首个**视图读必须落在重长后的 count 上——插入走裸
/// host 回调 `__zw_insert_adjacent_html`（纯 mutation push，零查询副作用）。
/// proxy 级 `insertAdjacentHTML` 的解析机械（`_zwFragmentAdded`/`_makeProxy`）
/// 会在 push **前**触发一次 count=0 视图读，先把条目重建到 (0, 新 drain 代际)、
/// 掩盖碰撞窗口——碰撞只在窗口内首个视图读就落在同 count 查询上时显形。
#[test]
fn test_query_view_doc_drain_exact_count_collision_r_baidu3() {
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
        "<html><body><ul id='a'></ul><ul id='b'></ul></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 批 1：裸回调插入 x1（纯 push）→ 查询烘焙视图（count=1，pre-drain 基座含 x1）。
    sandbox
        .execute(
            "globalThis.__zw_insert_adjacent_html('#a', 'beforeend', '<li>x1</li>');\
             globalThis.__n1 = document.querySelectorAll('#a li').length;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__n1").unwrap().value, "1");
    // drain：清队列 + bump drain 代际，不换代（生产 drain 站点 clear/bump 与
    // apply/快照换代之间的窗口形态；快照内容保持原样——x1 仅存在于旧视图）。
    mutations.lock().unwrap().clear();
    crate::js_dom_bridge::bump_mut_drain_gen();
    // 批 2：裸回调插入 y1（重长回恰好旧 count=1，纯 push 零查询）→ drain 后首个
    // 视图读就是紧随的 count=1 查询。端出 pre-drain 视图的失败形态：
    // #b li=0（y1 丢）且 #a li=1（x1 复活）。
    sandbox
        .execute(
            "globalThis.__zw_insert_adjacent_html('#b', 'beforeend', '<li>y1</li>');\
             globalThis.__n2 = document.querySelectorAll('#b li').length + ':' + document.querySelectorAll('#a li').length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__n2").unwrap().value,
        "1:0",
        "drain 后同 count 精确命中不得端出 pre-drain 视图（y1 须可见、x1 不得复活）"
    );
}
