// part56：Constructable CSSStyleSheet + adoptedStyleSheets（CSSOM，siteopt
// qq-20261016-r1 切片 A）——构造器语义、adopted 校验、host 下发通路（shim →
// `__zw_set_adopted_css` → ACTIVE_ADOPTED_CSS → collect_stylesheets → gCS）、
// 级联序（document `<style>` 后接 adopted）与 styleSheets 不含 adopted 表。
// 依据：https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssstylesheet 、
// https://drafts.csswg.org/cssom/#dom-documentorshadowroot-adoptedstylesheets 。

/// 本文件触 adopted 槽的用例串行化（R-t2qq 返修 D1）：槽改进程级共享后，并行 cargo
/// test 下两个用例交错 set/clear 会互踩对方的 gCS 断言。insertBefore 两例不触槽，不取锁。
static P56_ADOPTED_SLOT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 沙箱级通路测试公共装具（part51 同款）。
fn p56_sandbox_with_html(html: &str) -> zero_script_sandbox::V8Sandbox {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
}

fn p56_eval(sandbox: &mut zero_script_sandbox::V8Sandbox, script: &str) -> String {
    use zero_script_sandbox::Sandbox;
    sandbox.execute(script).unwrap().value
}

#[test]
fn test_p56_cssstylesheet_constructor_semantics() {
    // 本例 insertRule/replaceSync/replace() 均触发 shim `_zwAdoptedPush`（未 adopt →
    // 推空串 = 清共享槽；replace() 的 Promise 回调延迟排空同样推）——取文件级锁串行
    //（D1 槽共享后防与并行 adopted 用例互踩，实测首次并发即复现互踩）。
    let _slot_guard = P56_ADOPTED_SLOT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let html = "<html><head></head><body><div id=\"t\">t</div></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    let mut sb = p56_sandbox_with_html(html);
    // 全局构造器存在、new 产物 constructable sheet 面齐备。
    assert_eq!(p56_eval(&mut sb, "typeof CSSStyleSheet"), "function");
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); return [s instanceof CSSStyleSheet, s.ownerNode === null, s.disabled === false, s.cssRules.length === 0].join(','); })()"
        ),
        "true,true,true,true"
    );
    // insertRule / deleteRule 记账（R2809 同语义返回 index）。
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); var i = s.insertRule('div { color: red }'); var c = s.cssRules[0]; return [i, c.selectorText, s.cssRules.length, s instanceof CSSStyleSheet].join('|'); })()"
        ),
        "0|div|1|true"
    );
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); s.insertRule('div { color: red }'); s.insertRule('p { color: blue }'); s.deleteRule(0); return s.cssRules.length + ':' + s.cssRules[0].selectorText; })()"
        ),
        "1:p"
    );
    // replaceSync 整表替换 + replace 返 Promise（resolve this）。
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); s.replaceSync('div { color: red } p { color: blue }'); return s.cssRules.length; })()"
        ),
        "2"
    );
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); var ok = false; s.replace('div { color: red }').then(function (v) { ok = (v === s); }); return [String(ok), typeof s.replace].join(':'); })()"
        ),
        "false:function"
    );
}

#[test]
fn test_p56_adoptedstylesheets_setter_validation_and_getter() {
    // setter 赋值会触发 shim `_zwAdoptedPush`（本例各 sheet 规则恒空 → 推空串 = 清槽，
    // 仍属写共享槽）——取文件级锁串行（D1）。
    let _slot_guard = P56_ADOPTED_SLOT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let html = "<html><head></head><body><div id=\"t\">t</div></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_adopted_css("");
    let mut sb = p56_sandbox_with_html(html);
    // 初始空数组；setter 接受 constructable sheet 数组并回读同列表。
    assert_eq!(p56_eval(&mut sb, "document.adoptedStyleSheets.length"), "0");
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); document.adoptedStyleSheets = [s]; var l = document.adoptedStyleSheets; return [l.length, l[0] === s].join(','); })()"
        ),
        "1,true"
    );

    // 非本构造器产物（<style> 元素的 sheet、plain object）→ TypeError。
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ try { document.adoptedStyleSheets = [{}]; return 'no-throw'; } catch (e) { return e.constructor.name; } })()"
        ),
        "TypeError"
    );
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ try { document.adoptedStyleSheets = 'x'; return 'no-throw'; } catch (e) { return e.constructor.name; } })()"
        ),
        "TypeError"
    );
    // ShadowRoot 版：get/set 往返 + 同款 TypeError。
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var host = document.getElementById('t'); var sh = host.attachShadow({ mode: 'open' }); var s = new CSSStyleSheet(); sh.adoptedStyleSheets = [s]; var l = sh.adoptedStyleSheets; var bad = false; try { sh.adoptedStyleSheets = [1]; } catch (e) { bad = (e.constructor.name === 'TypeError'); } return [l.length, l[0] === s, bad].join(','); })()"
        ),
        "1,true,true"
    );
}

#[test]
fn p56_gcs_reflects_adopted_sheet_via_host_callback() {
    // 槽进程级共享（D1）：先取文件级锁，防并行用例交错 set/clear。
    let _slot_guard = P56_ADOPTED_SLOT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // e2e 通路：页面 JS 设 adoptedStyleSheets → `__zw_set_adopted_css` 装进程级共享槽 →
    // collect_stylesheets 级联 → gCS 反映（视觉生效于 gCS 面；布局管线同函数同源）。
    // 规则选择器用本文件专属 id（#p56h，F1 返修）：槽全局共享后，泛化标签选择器
    //（h1/div/p）会在并行执行时漏进其他用例（如 part51 的 h1/p color 断言）的合成源。
    let html = "<html><head></head><body><h1 id=\"p56h\">t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_adopted_css("");
    let mut sb = p56_sandbox_with_html(html);
    // 未设 adopted：color 无规则 → 初始值。
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('p56h')).color"),
        "rgb(0, 0, 0)"
    );
    // 设 adopted（replaceSync + adopt）：gCS 立即反映（同 execute 内回调同步装填）。
    // F1：color 是 adopted 规则专有断言——原 fontWeight==700 被 h1 的 UA 默认粗体
    // 满足，adopt 生效与否皆绿（断言空洞）。
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); s.replaceSync('#p56h { color: rgb(9, 105, 218) }'); document.adoptedStyleSheets = [s]; return document.adoptedStyleSheets.length; })()"
        ),
        "1"
    );
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('p56h')).color"),
        "rgb(9, 105, 218)",
        "adopt 后 adopted 规则必须进入 gCS 合成源"
    );
    // adopt → insertRule 换色（缺省尾插，同特异性后规则胜）→ 断言 color 更新：
    // 锁定 `__zw_set_adopted_css` 全量重推通路 + generation/cs_cache 两级缓存键
    //（adopted 文本为键分量，变化 ⇒ sheets 重收集 + 换代重算）失效。
    p56_eval(&mut sb, "document.adoptedStyleSheets[0].insertRule('#p56h { color: rgb(255, 0, 0) }')");
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('p56h')).color"),
        "rgb(255, 0, 0)",
        "insertRule 换色后必须重下发并失效两级缓存键"
    );
    // 清空 adopted：回落初始值（跨代残留清理语义同 external css）。
    p56_eval(&mut sb, "document.adoptedStyleSheets = []");
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('p56h')).color"),
        "rgb(0, 0, 0)"
    );
    crate::js_dom_bridge::set_active_adopted_css("");
}

#[test]
fn p56_adopted_slot_cross_thread_visibility() {
    // D1 返修：adopted 槽进程级共享——写入（js worker 线程宿主回调）与读取（渲染线程
    // collect / gCS 键）跨线程可见。thread_local 旧实现下他线程恒读空串，生产级联
    // 永不含 adopted 规则。断言：线程 A 装填 → 线程 B（另一 spawn）读到同值。
    let _slot_guard = P56_ADOPTED_SLOT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    crate::js_dom_bridge::set_active_adopted_css("");
    let setter = std::thread::spawn(|| {
        crate::js_dom_bridge::set_active_adopted_css("#p56xthread { color: rgb(1, 2, 3) }");
    });
    setter.join().unwrap();
    let got =
        std::thread::spawn(crate::js_dom_bridge::active_adopted_css).join().unwrap();
    assert_eq!(
        &*got,
        "#p56xthread { color: rgb(1, 2, 3) }",
        "adopted 槽必须跨线程可见（进程级共享槽语义）"
    );
    crate::js_dom_bridge::set_active_adopted_css("");
}

#[test]
fn p56_adopted_overrides_document_style_cascade_order() {
    let _slot_guard = P56_ADOPTED_SLOT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // 级联序：document `<style>` 表在前、adopted 表**尾接**（CSSOM author 源顺序，
    // contract-r2 切片 A 设计）——同特异性同属性 adopted 胜。两侧同用 .p56src 类选择器
    //（同特异性，序定胜负；专属类名防槽共享后漏进并行用例——F1 同理）。
    let html = "<html><head><style>.p56src { color: rgb(0, 128, 0); }</style></head><body><h1 id=\"p56h\" class=\"p56src\">t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_adopted_css("");
    let mut sb = p56_sandbox_with_html(html);
    p56_eval(
        &mut sb,
        "(function(){ var s = new CSSStyleSheet(); s.replaceSync('.p56src { color: rgb(255, 0, 0) }'); document.adoptedStyleSheets = [s]; })()",
    );
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('p56h')).color"),
        "rgb(255, 0, 0)",
        "adopted 表尾接（document sheets 之后），同特异性下 adopted 胜"
    );
    crate::js_dom_bridge::set_active_adopted_css("");
}

#[test]
fn p56_style_sheets_excludes_adopted() {
    let _slot_guard = P56_ADOPTED_SLOT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // document.styleSheets 不含 adopted 表（Chrome 一致：cssom document-stylesheets
    // 只列 document 元素表；R2808 getter 语义不变）。
    let html = "<html><head><style>#p56doc { color: red }</style></head><body><h1 id=\"p56h\">t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_adopted_css("");
    let mut sb = p56_sandbox_with_html(html);
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); s.replaceSync('#p56ad { color: blue }'); document.adoptedStyleSheets = [s]; return document.styleSheets.length; })()"
        ),
        "1",
        "styleSheets 只含 <style> 元素表，不得含 adopted"
    );
    crate::js_dom_bridge::set_active_adopted_css("");
}

#[test]
fn p56_insertbefore_cross_family_handle_container_ref_matches() {
    // R-t2qq 返修 F3：insertBefore 多族判子 arm① 正例（React reconcile 形态——真负例
    // 守卫见 p56_insertbefore_nonchild_ref_still_throws_notfound）。容器与 ref.parentNode
    // 为**同一节点的不同包装族**：容器 = querySelector 命中经 R100 identity 桥反查的
    // handle 族 proxy（`__zwHandle` 章），ref.parentNode = 快照锚定的 sel 族 proxy
    //（`_parentNodeFor`/R5010 锚回路径）。raw `===` 与融合子列表（arm②）均不命中，
    // 仅 `_zwSameNodeIdentity` 的 (sel, handle) 锚定键跨族收敛可判同节点——缺失该臂时
    // 本例误抛 NotFoundError（已实证：禁用臂后测试转红）。
    // 铺垫 = part25 r5010 同款宿主 apply 链：createElement pending →
    // apply_mutations_to_html_with_handles 落快照（产 handle→selector 表）→
    // `__zw_apply_generation_bump` 清 pending → 注册 identity 桥 stub（生产
    // renderer register_identity_bridge_callback 同签名）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let html = "<html><head></head><body></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sb = V8Sandbox::with_config(config).unwrap();
    sb.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sb, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // ① 页面 JS：createElement 容器（handle 章 pending，React mount 形态）。
    sb.execute("var c = document.createElement('div'); c.id = 'p56c'; document.body.appendChild(c);")
        .unwrap();
    // ② 宿主 apply：落快照 + 产 handle→selector 表（生产 mutation merge 同入口）。
    // ref 子节点以**纯快照内容**注入 applied html（server-rendered child 形态——
    // hydration 场景：客户端 mount 的容器下已有服务端输出的子节点；ref 从未过
    // JS merge，不进 handle 注册表，保持快照 sel 锚定）。
    let batch: Vec<DomMutation> = mutations.lock().unwrap_or_else(|e| e.into_inner()).drain(..).collect();
    let (applied_html, handle_selectors) = crate::apply_mutations_to_html_with_handles(
        &dom_html.lock().unwrap_or_else(|e| e.into_inner()),
        &batch,
    )
    .expect("宿主 apply：pending createElement 落快照");
    let empty_container = "<div id=\"p56c\"></div>";
    assert!(applied_html.contains(empty_container), "applied html 应含空容器：{applied_html}");
    let applied_html = applied_html.replacen(
        empty_container,
        "<div id=\"p56c\"><p id=\"p56ref\">r</p></div>",
        1,
    );
    *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = applied_html;
    // ③ apply 代际 purge + identity 桥 stub（sel→handle 反查，生产 R100 同语义）。
    // stub 仅映射容器 '#p56c'：ref 不入桥 → 查询返回点走快照 sel/plain 包装（页面持
    // ref 的另一取径——React reconcile 的双来源形态：容器是 mount 期产物 handle 章、
    // ref 来自解析/快照面）。
    sb.execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();
    let sel_to_handle: std::collections::HashMap<String, String> = handle_selectors
        .into_iter()
        .filter(|(h, s)| s == "#p56c")
        .map(|(h, s)| (s, h))
        .collect();
    assert!(sel_to_handle.contains_key("#p56c"));
    sb.register_callback(
        "__zw_handle_for_selector",
        Box::new(move |args: &[String]| -> String {
            sel_to_handle.get(args.first().map(String::as_str).unwrap_or("")).cloned().unwrap_or_default()
        }),
    );
    // ④ 跨族 insertBefore（arm① 正例）：容器 = getElementById 命中经 R100 identity 桥
    // 反查的 **handle 族 proxy**（锚 (null, __n0)）；ref = 纯快照 sel 族 proxy（锚
    // ('#p56ref', null)）。ref.parentNode = 快照父解析 `_wrapSelector('#p56c')`——与
    // 容器**同节点异对象**（sel 锚 vs handle 锚，`_proxyCache` 异键）。raw `===` 与
    // arm②（ref 不在 handle 注册表、融合子列表迭代产物异对象）均不命中，仅
    // `_zwSameNodeIdentity` 的 (sel, handle) 锚定键跨族收敛可判同节点。
    let verdict = p56_eval(
        &mut sb,
        r#"(function(){
  var c = document.getElementById('p56c');
  var ref = document.getElementById('p56ref');
  if (!c || !ref) return 'missing:' + [!!c, !!ref].join(',');
  var el = document.createElement('i');
  var threw = null;
  try { c.insertBefore(el, ref); } catch (e) { threw = e.name; }
  var parentOk = false;
  try { parentOk = !!el.parentNode; } catch (e) {}
  function anchor(n) { return (n && n.__zwHandle ? 'H' : '') + (n && n.__zwSelector ? 'S' : ''); }
  return [String(threw), String(parentOk), anchor(c), anchor(ref), anchor(ref.parentNode), 'pnIsC:' + (ref.parentNode === c)].join(',');
})()"#,
    );
    assert_eq!(
        verdict, "null,true,H,S,S,pnIsC:false",
        "跨族同节点 ref（handle 容器 × sel ref/parentNode）不得误抛 NotFoundError"
    );
}

#[test]
fn p56_insertbefore_nonchild_ref_still_throws_notfound() {
    // 切片 B 不退化：真非子 ref（他父子节点）insertBefore 照抛 NotFoundError
    //（跨族 match-only 命中臂不放宽真负例；Chrome 对照同抛）。
    let html = "<html><head></head><body><div id=\"p1\"><p id=\"c1\">one</p></div><ul id=\"p2\"><li id=\"c2\">x</li></ul></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    let mut sb = p56_sandbox_with_html(html);
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var p1 = document.getElementById('p1'); var el = document.createElement('i'); try { p1.insertBefore(el, document.getElementById('c2')); return 'no-throw'; } catch (e) { return e.name; } })()"
        ),
        "NotFoundError"
    );
    // 正例回归：同容器 ref（querySelector / firstChild / children 集合三取径）均通过。
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var p1 = document.getElementById('p1'); var el = document.createElement('i'); p1.insertBefore(el, document.getElementById('c1')); var el2 = document.createElement('b'); p1.insertBefore(el2, p1.firstChild); var el3 = document.createElement('u'); p1.insertBefore(el3, p1.children[0]); return [el.parentNode === p1, el2.parentNode === p1, el3.parentNode === p1].join(','); })()"
        ),
        "true,true,true"
    );
    // 跨族命中臂：innerHTML 解析的 named-prop 族节点作 ref（解析 wrapper ≠ sel proxy）。
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var host = document.getElementById('p1'); host.innerHTML = '<p id=\"np\">parsed</p>'; var ref = globalThis['np']; if (!ref) return 'no-named-prop'; var el = document.createElement('i'); var threw = null; try { host.insertBefore(el, ref); } catch (e) { threw = e.name; } return [String(threw), el.parentNode === host].join(','); })()"
        ),
        "null,true",
        "named-prop 族 ref 作参照不得误抛 NotFoundError（guidelines #32 多族包装）"
    );
}
