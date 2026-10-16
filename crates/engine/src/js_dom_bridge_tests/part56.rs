// part56：Constructable CSSStyleSheet + adoptedStyleSheets（CSSOM，siteopt
// qq-20261016-r1 切片 A）——构造器语义、adopted 校验、host 下发通路（shim →
// `__zw_set_adopted_css` → ACTIVE_ADOPTED_CSS → collect_stylesheets → gCS）、
// 级联序（document `<style>` 后接 adopted）与 styleSheets 不含 adopted 表。
// 依据：https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssstylesheet 、
// https://drafts.csswg.org/cssom/#dom-documentorshadowroot-adoptedstylesheets 。

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
    let html = "<html><head></head><body><div id=\"t\">t</div></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
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
    // e2e 通路：页面 JS 设 adoptedStyleSheets → `__zw_set_adopted_css` 装线程局部 →
    // collect_stylesheets 级联 → gCS 反映（视觉生效于 gCS 面；布局管线同函数同源）。
    let html = "<html><head></head><body><h1 id=\"h\">t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_adopted_css("");
    let mut sb = p56_sandbox_with_html(html);
    // 未设 adopted：color 无规则 → 初始值。
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('h')).color"),
        "rgb(0, 0, 0)"
    );
    // 设 adopted（replaceSync + adopt）：gCS 立即反映（同 execute 内回调同步装填）。
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); s.replaceSync('h1 { color: rgb(9, 105, 218) }'); document.adoptedStyleSheets = [s]; return document.adoptedStyleSheets.length; })()"
        ),
        "1"
    );
    // insertRule 增量变更后重推送：gCS 跟随（_zwAdoptedPush 挂 flush）。
    p56_eval(&mut sb, "document.adoptedStyleSheets[0].insertRule('h1 { font-weight: 700 }')");
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('h')).fontWeight"),
        "700",
        "adopted 表内 insertRule 必须重下发"
    );
    // 清空 adopted：回落初始值（跨代残留清理语义同 external css）。
    p56_eval(&mut sb, "document.adoptedStyleSheets = []");
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('h')).color"),
        "rgb(0, 0, 0)"
    );
}

#[test]
fn p56_adopted_overrides_document_style_cascade_order() {
    // 级联序：document `<style>` 表在前、adopted 表**尾接**（CSSOM author 源顺序，
    // contract-r2 切片 A 设计）——同特异性同属性 adopted 胜。
    let html = "<html><head><style>h1 { color: rgb(0, 128, 0); }</style></head><body><h1 id=\"h\">t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_adopted_css("");
    let mut sb = p56_sandbox_with_html(html);
    p56_eval(
        &mut sb,
        "(function(){ var s = new CSSStyleSheet(); s.replaceSync('h1 { color: rgb(255, 0, 0) }'); document.adoptedStyleSheets = [s]; })()",
    );
    assert_eq!(
        p56_eval(&mut sb, "getComputedStyle(document.getElementById('h')).color"),
        "rgb(255, 0, 0)",
        "adopted 表尾接（document sheets 之后），同特异性下 adopted 胜"
    );
}

#[test]
fn p56_style_sheets_excludes_adopted() {
    // document.styleSheets 不含 adopted 表（Chrome 一致：cssom document-stylesheets
    // 只列 document 元素表；R2808 getter 语义不变）。
    let html = "<html><head><style>h1 { color: red }</style></head><body><h1 id=\"h\">t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_adopted_css("");
    let mut sb = p56_sandbox_with_html(html);
    assert_eq!(
        p56_eval(
            &mut sb,
            "(function(){ var s = new CSSStyleSheet(); s.replaceSync('h1 { color: blue }'); document.adoptedStyleSheets = [s]; return document.styleSheets.length; })()"
        ),
        "1",
        "styleSheets 只含 <style> 元素表，不得含 adopted"
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
