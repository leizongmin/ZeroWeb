// part75 — html-semantics M2 切片 43（R5246，2026-10-23）：replaceWith self-removal
// 父桶记账——sel 臂此前只走 `__zw_remove`（apply 面 Remove mutation）+ `_zwMarkRemoved`
// （sel 字符串表：parentNode/isConnected 读），合并视图子读（childNodes overlay
// `_zwOverlayPendingChildNodes` / R322 归并查询）按**父桶 removed identity** 剔除——桶缺
// removed 项 → 被替换者（快照 sel 节点，如 corpus dialog-focus-shadow 的 template）在
// 合并视图残留至 apply 换代（s26 挂账「R371 rekey 时 dst 桶不存在（dstA:-1），不记父桶
// removed」收口）。spec：replaceWith = insert-then-remove，remove 步同步反映全部树视图。
// https://dom.spec.whatwg.org/#dom-childnode-replacewith

/// 断言反转面：父桶记账缺失 → A/B/C/G/H/I 全 false（残留可见）；记账误杀活节点 →
/// D/E/J（插入侧/反链/父链既有面）翻红。
#[test]
fn test_dialog_replacewith_parent_bucket_accounting_r5246() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body><dialog class='d'><template class='t'><button>x</button></template><button class='after'>a</button></dialog><template class='t2'><b>y</b></template></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // R1：corpus 同形（dialog > template + 后随兄弟）——replaceWith(newDiv) 同一 execute 内
    // 被替换 template 须即刻退出全部树视图（A 作用域查询 / B childNodes identity / C div 位序
    // 补位），插入侧 D/E 与被替换者父链 F 为既有面回归守卫。
    sandbox
        .execute(
            "var d = document.querySelector('dialog'); var t = document.querySelector('.t'); var nv = document.createElement('div'); t.replaceWith(nv); globalThis.__r5246a = [String(d.querySelector('template') == null), String([].indexOf.call(d.childNodes, t) === -1), String([].indexOf.call(d.childNodes, nv) === 0), String(d.querySelector('div') === nv), String(nv.parentNode === d), String(t.parentNode == null)].join('|');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5246a").unwrap().value,
        "true|true|true|true|true|true",
        "R1：dialog 内 replaceWith 后被替换 template 同步退出合并视图（A/B/C），div 补位、反链与父链保持（D/E/F）"
    );

    // R2：body 直子 template（document 漏斗面）——replaceWith 后 document 查询与
    // body 作用域查询（本移除的父桶 R310 消费面）、body childNodes 均不再见被替换者，
    // 新 span 反链到位。类限定隔离 R1 的 dialog 桶记账（R310 祖先作用域不传播——
    // 共享路径独立挂账，不在本切片断言面）。
    sandbox
        .execute(
            "var t2 = document.querySelector('.t2'); var sp = document.createElement('span'); t2.replaceWith(sp); globalThis.__r5246b = [String(document.querySelector('.t2') == null), String(document.body.querySelector('.t2') == null), String([].indexOf.call(document.body.childNodes, t2) === -1), String(sp.parentNode === document.body)].join('|');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5246b").unwrap().value,
        "true|true|true|true",
        "R2：body 直子 replaceWith 后 document/body 查询与 childNodes 同步（G/H/I），span 反链到位（J）"
    );
}
