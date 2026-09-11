//! event-loop-spec M2 MO-S1：host 侧 mutation 通知排空（`drain_native_mutations_to_mo`）。
//! kill-switch `set_mo_host_trigger` / env `ZW_MO_HOST_TRIGGER`（默认 OFF）。

use super::super::*;

/// native 写（native 绑定 → `with_dom_mut` → dom 层 `pending_mutations`）→ execute 尾部
/// `sync_render_after_native_dom` 排空 → `__zw_mo_notify_native` → polyfill `_mo_notify`
/// 共享注册表派发 → 页面 MO 回调收到 record（设计片 p1b-mutationobserver-host-trigger
/// §5 MO-S1 验证面：native appendChild/写后 polyfill MO 收 record）。
#[test]
fn test_mo_host_trigger_native_attribute_notifies_polyfill_mo() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 内联 script 必须存在——空脚本时 run_page_scripts 早返、不注册 dom 查询回调。
    wv.load_html("<html><body><div id='t'></div><script>0;</script></body></html>", None);
    // 先经页面脚本通路装 shim + dom 回调（getElementById→__zw_query_match 等注册）。
    wv.run_page_scripts().expect("run page scripts");
    wv.set_mo_host_trigger(true);
    // 页面注册 polyfill MO（沙箱 persistent，跨 execute 存活）。
    wv.execute_script(
        "globalThis.__moRecs = [];\
         globalThis.__mo = new MutationObserver(function (rs) { globalThis.__moRecs = globalThis.__moRecs.concat(rs); });\
         globalThis.__mo.observe(document.getElementById('t'), { attributes: true, attributeOldValue: true });",
    )
    .unwrap();
    // native 写：native 元素绑定 setAttribute → Document::set_attribute → pending_mutations。
    // 两次写：首写 class 无旧值（oldValue null 合法）；二写携带 dom 层捕获的旧值 'c1'
    //（spec：attributeOldValue 时 oldValue = 写前值——MO-S1 排空侧透传验证面）。
    wv.execute_script("(()=>{ const e = __zw_native_element_for_id('t'); e.setAttribute('class', 'c1'); })()")
        .unwrap();
    // MO 回调经 microtask 派发；轮询至到达（沙箱每次 execute 后泵 pending microtasks）。
    let mut arrived = false;
    for _ in 0..50 {
        let n = wv.execute_script("String(globalThis.__moRecs.length)").unwrap();
        if n.trim() == "1" {
            arrived = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(arrived, "native 写应经排空桥投递 polyfill MO（recs=1）");
    let detail = wv
        .execute_script(
            "var r = globalThis.__moRecs[0]; r.type + '/' + r.attributeName + '/' + r.oldValue + '/' + r.target.id",
        )
        .unwrap();
    assert_eq!(
        detail.trim(),
        "attributes/class/null/t",
        "record 形态：type/attributeName/target（首写无旧值 → oldValue null 合法）"
    );
    // 二次 native 写：oldValue 透传（dom 层写前捕获 'c1' → 排空 → observer 收 'c1'）。
    wv.execute_script("(()=>{ const e = __zw_native_element_for_id('t'); e.setAttribute('class', 'c2'); })()")
        .unwrap();
    let mut second = false;
    for _ in 0..50 {
        let detail2 = wv
            .execute_script(
                "var r = globalThis.__moRecs[1]; r ? (r.type + '/' + r.attributeName + '/' + r.oldValue + '/' + r.target.id) : 'pending'",
            )
            .unwrap();
        if detail2.trim() == "attributes/class/c1/t" {
            second = true;
            break;
        }
        assert_ne!(
            detail2.trim(),
            "attributes/class/null/t",
            "二写 oldValue 应为写前值 'c1'，got {}",
            detail2.trim()
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(second, "二次 native 写应投递携 oldValue='c1' 的 record");
    // MO-S2：childList 移除——removedNodes 经 '#id' 回落身份化（脱树节点 unique 路径
    // 必败）+ previousSibling 透传（dom 层写前捕获）。
    let mut wv2 = WebView::new(WebViewConfig::default());
    wv2.load_html(
        "<html><body><ul id='ul'><li id='a'></li><li id='m'></li><li id='l'></li></ul><script>0;</script></body></html>",
        None,
    );
    wv2.run_page_scripts().expect("run page scripts");
    wv2.set_mo_host_trigger(true);
    wv2.execute_script(
        "globalThis.__ulRecs = [];\
         globalThis.__moUl = new MutationObserver(function (rs) { globalThis.__ulRecs = globalThis.__ulRecs.concat(rs); });\
         globalThis.__moUl.observe(document.getElementById('ul'), { childList: true });",
    )
    .unwrap();
    // native 移除中间子 #m：prev=#a（dom 层写前捕获）+ next=#l（排空侧从当前树反推——
    // prev 存在 → prev 的下一兄弟）。
    wv2.execute_script("(()=>{ const ul = __zw_native_element_for_id('ul'); const m = __zw_native_element_for_id('m'); ul.removeChild(m); })()")
        .unwrap();
    let mut removed = false;
    for _ in 0..50 {
        let detail = wv2
            .execute_script(
                "var r = globalThis.__ulRecs.find(function (x) { return x.removedNodes.length > 0; });\
                 r ? (r.target.id + '/' + r.removedNodes.length + '/' + (r.removedNodes[0] ? 'proxy' : 'missing') + '/' + (r.previousSibling ? (r.previousSibling.id || 'nopx') : 'null') + '/' + (r.nextSibling ? (r.nextSibling.id || 'nopx') : 'null')) : 'pending'",
            )
            .unwrap();
        if detail.trim() == "ul/1/proxy/a/l" {
            removed = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        removed,
        "childList 移除应投递 removedNodes(1) + previousSibling=#a + nextSibling=#l"
    );
}

/// MO-S2 fragment flatten（v8-only：quickjs 绑定面缺 `__zw_native_create_document_fragment`——
/// DC-7 对等缺口，master.md 记档）：native appendChild(DocumentFragment) 经绑定层
/// `insert_with_fragment_flatten`（R3132）逐子移动——fragment 自身不入树，每个子各产一条
/// childList record（addedNodes=[该子]；与浏览器「单记录 N addedNodes」的批派发粒度差异
/// 记 master.md MO-S3 候选）。
#[cfg(feature = "v8")]
#[test]
fn test_mo_host_trigger_fragment_flatten_v8() {
    let mut wv3 = WebView::new(WebViewConfig::default());
    wv3.load_html("<html><body><ul id='ul'></ul><script>0;</script></body></html>", None);
    wv3.run_page_scripts().expect("run page scripts");
    wv3.set_mo_host_trigger(true);
    wv3.execute_script(
        "globalThis.__fragRecs = [];\
         globalThis.__moFrag = new MutationObserver(function (rs) { globalThis.__fragRecs = globalThis.__fragRecs.concat(rs); });\
         globalThis.__moFrag.observe(document.getElementById('ul'), { childList: true });",
    )
    .unwrap();
    wv3.execute_script(
        "(()=>{ const ul = __zw_native_element_for_id('ul');\
           const frag = __zw_native_create_document_fragment();\
           const p = __zw_native_create_element('li'); p.id = 'f1';\
           const q = __zw_native_create_element('li'); q.id = 'f2';\
           frag.appendChild(p); frag.appendChild(q);\
           ul.appendChild(frag); })()",
    )
    .unwrap();
    let mut frag_ok = false;
    for _ in 0..50 {
        let detail = wv3
            .execute_script(
                "var total = 0; var known = false;\
                 for (var i = 0; i < globalThis.__fragRecs.length; i++) {\
                   var r = globalThis.__fragRecs[i];\
                   for (var j = 0; j < r.addedNodes.length; j++) {\
                     total++;\
                     if (r.addedNodes[j].id === 'f1' || r.addedNodes[j].id === 'f2') known = true;\
                   }\
                 }\
                 var lis = document.getElementById('ul').querySelectorAll('li');\
                 'added=' + total + '/known=' + known + '/li=' + lis.length",
            )
            .unwrap();
        if detail.trim() == "added=2/known=true/li=2" {
            frag_ok = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        frag_ok,
        "fragment append：两个 li 入树且 MO 记录各自可达（fragment 自身不入树）"
    );
}

/// kill-switch OFF（默认）：native 写不投递——通知端死路保持（行为时序变更门禁）。
#[test]
fn test_mo_host_trigger_default_off_no_notify() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 内联 script 必须存在——空脚本时 run_page_scripts 早返、不注册 dom 查询回调。
    wv.load_html("<html><body><div id='t'></div><script>0;</script></body></html>", None);
    wv.run_page_scripts().expect("run page scripts");
    // 不调 set_mo_host_trigger（默认 OFF；CI 环境无 ZW_MO_HOST_TRIGGER）。
    wv.execute_script(
        "globalThis.__moRecs = [];\
         globalThis.__mo = new MutationObserver(function (rs) { globalThis.__moRecs = globalThis.__moRecs.concat(rs); });\
         globalThis.__mo.observe(document.getElementById('t'), { attributes: true });",
    )
    .unwrap();
    wv.execute_script("(()=>{ const e = __zw_native_element_for_id('t'); e.setAttribute('class', 'c1'); })()")
        .unwrap();
    for _ in 0..20 {
        let n = wv.execute_script("String(globalThis.__moRecs.length)").unwrap();
        assert_eq!(n.trim(), "0", "kill-switch OFF 时 native 写不得投递 MO");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
