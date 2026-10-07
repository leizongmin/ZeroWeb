// js_dom_bridge 测试模块拆分 part 41（t8e：Element.children 集合对象缓存）。
// 根因：children getter 每次访问全量重建（slice + 逐子 trap 读 nodeType 过滤 + Proxy 构造）
// 且向 _zwLiveCollections 注册新集合（仅快照换代清空）——长驻页轮询 `children[i]` O(n)/读
//（沙箱微基准 15.8ms/次@千子父）+ 注册表无界增长 → 每变异全表扫 O(集合×元素)（bilibili
// e() 稳态块实证机制）。修复：同元素同分支返回同一活集合（写侧 _zwHCLiveInvalidate 维护网
// 保留），scoped 集合树序重排（_zwHCTreeOrderSync：记录锚→槽锚→动态读 + 链走查），换代清缓存。
// 钉测覆盖：身份稳定性 / 活集合语义（append/insertBefore/removeChild/文本子）/ 插入序 /
// 换代重置 / sel 路径（R318 + fallback liveSpec 挂网）/ v2 返修钉：多节点批序（PR #96 审查
// D1：fragment/iAH 批插入逐节点动态锚产生部分排序）/ 文本锚走查 / 同父移动（轮播+快照节点）/ replaceChild。

#[test]
fn t8e_children_collection_identity_and_live_handle_path() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8e-children".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // handle 路径（createElement 容器）：缓存命中恒同一集合；活语义在缓存持有下不退化
    //（append 可见、insertBefore 按树序落位、removeChild 消失、文本子不入元素集合）。
    sandbox
        .execute(
            "var p = document.createElement('div');\n\
             var a = document.createElement('span'); p.appendChild(a);\n\
             var b = document.createElement('em'); p.appendChild(b);\n\
             var c1 = p.children, c2 = p.children;\n\
             globalThis.__ident = (c1 === c2);\n\
             globalThis.__len0 = c1.length;\n\
             var x = document.createElement('i');\n\
             p.appendChild(x);\n\
             globalThis.__lenAfterAppend = c1.length;\n\
             globalThis.__xLast = (c1[2] === x);\n\
             p.insertBefore(x, b);\n\
             globalThis.__orderMid = (c1[0] === a && c1[1] === x && c1[2] === b);\n\
             var y = document.createElement('u');\n\
             p.insertBefore(y, a);\n\
             globalThis.__orderHead = (c1[0] === y && c1[1] === a);\n\
             p.removeChild(y);\n\
             globalThis.__lenAfterRemove = c1.length;\n\
             p.appendChild(document.createTextNode('txt'));\n\
             globalThis.__lenText = c1.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ident").unwrap().value,
        "true",
        "t8e：children 重复访问返回同一集合（缓存命中）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__len0").unwrap().value,
        "2",
        "t8e：初始元素子计数 2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__lenAfterAppend").unwrap().value,
        "3",
        "t8e：缓存集合活语义——append 后 length 增长"
    );
    assert_eq!(
        sandbox.execute("globalThis.__xLast").unwrap().value,
        "true",
        "t8e：尾部 append 落位末尾"
    );
    assert_eq!(
        sandbox.execute("globalThis.__orderMid").unwrap().value,
        "true",
        "t8e：insertBefore 中部落位按树序（nextSibling 锚定）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__orderHead").unwrap().value,
        "true",
        "t8e：insertBefore 头部落位按树序"
    );
    assert_eq!(
        sandbox.execute("globalThis.__lenAfterRemove").unwrap().value,
        "3",
        "t8e：removeChild 后缓存集合收缩"
    );
    assert_eq!(
        sandbox.execute("globalThis.__lenText").unwrap().value,
        "3",
        "t8e：文本子不入元素集合（matches 过滤在缓存持有下仍生效）"
    );
}

#[test]
fn t8e_children_cache_reset_on_pending_state_swap() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8e-reset".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 换代清缓存契约：__zw_reset_pending_state（SetDomSnapshot 挂钩）后 children 返回
    // 新建集合（不再命中旧缓存），且旧集合引用仍可用（冻结 = 旧文档语义，slice32 同款）。
    sandbox
        .execute(
            "var p = document.createElement('div');\n\
             p.appendChild(document.createElement('span'));\n\
             var old = p.children;\n\
             globalThis.__oldLen = old.length;\n\
             globalThis.__zw_reset_pending_state();\n\
             globalThis.__newIsNew = (p.children !== old);\n\
             globalThis.__newLen = p.children.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__oldLen").unwrap().value,
        "1",
        "t8e：换代前缓存集合内容正常"
    );
    assert_eq!(
        sandbox.execute("globalThis.__newIsNew").unwrap().value,
        "true",
        "t8e：换代后 children 不再命中旧缓存（缓存随注册表同点重置）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__newLen").unwrap().value,
        "1",
        "t8e：换代后新建集合视图正确"
    );
}

#[test]
fn t8e_children_sel_path_cache_and_fallback_livespec() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    // 初始快照带一个有两个元素子的容器 + 一个空容器（fallback liveSpec 面）。
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id=cc><span></span><em></em></div><div id=ec></div></body></html>"
            .to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8e-sel".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // sel 路径（R318）：缓存命中恒同一集合，insertBefore 树序落位；fallback 面（空容器，
    // 修复前无 liveSpec 不入维护网）先读后 append 必须在缓存持有下可见。
    sandbox
        .execute(
            "var cc = document.getElementById('cc');\n\
             var s1 = cc.children, s2 = cc.children;\n\
             globalThis.__sIdent = (s1 === s2);\n\
             globalThis.__sLen = s1.length;\n\
             var nb = document.createElement('b');\n\
             cc.insertBefore(nb, s1[1]);\n\
             globalThis.__sOrder = (s1[0].tagName === 'SPAN' && s1[1] === nb);\n\
             globalThis.__sLen2 = s1.length;\n\
             var ec = document.getElementById('ec');\n\
             var ef = ec.children;\n\
             globalThis.__fLen0 = ef.length;\n\
             var fz = document.createElement('i');\n\
             ec.appendChild(fz);\n\
             globalThis.__fLen1 = ef.length;\n\
             globalThis.__fRef = (ef[0] === fz);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__sIdent").unwrap().value,
        "true",
        "t8e：sel 路径 children 重复访问返回同一集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__sLen").unwrap().value,
        "2",
        "t8e：sel 路径初始元素子计数（host 快照）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__sOrder").unwrap().value,
        "true",
        "t8e：sel 路径 insertBefore 树序落位"
    );
    assert_eq!(
        sandbox.execute("globalThis.__sLen2").unwrap().value,
        "3",
        "t8e：sel 路径缓存集合活语义"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fLen0").unwrap().value,
        "0",
        "t8e：空容器 children 初始为空（fallback 面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fLen1").unwrap().value,
        "1",
        "t8e：fallback 面缓存集合活语义（liveSpec 挂网后 append 可见）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fRef").unwrap().value,
        "true",
        "t8e：fallback 面集合成员身份正确"
    );
}

// v2 返修钉（PR #96 审查 D1）：多节点单 record 批插入的树序——invalidate 逐节点尾部
// push 后，重排必须按批的**外部锚**（record nextSibling）+ addFlat 序落位，批内序不
// 得被批内互锚破坏（v1 缺陷形：fragment [x,y] insertBefore b → [a,y,b,x]）。
#[test]
fn t8e_children_multi_node_batch_tree_order() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id=bc><span></span><em></em></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8e-batch".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // handle 路径 fragment 批：单 record（addedNodes=[n1,n2]，锚=b）→ [a,n1,n2,b]。
    // sel 路径 iAH afterbegin 批：单 record（addedNodes=[解析B,解析I]，锚=旧首子 SPAN）
    // → [B,I,SPAN,EM]——bilibili 水合顶部插入的真实形态。
    sandbox
        .execute(
            "var p = document.createElement('div');\n\
             var a = document.createElement('span'); p.appendChild(a);\n\
             var b = document.createElement('em'); p.appendChild(b);\n\
             var c1 = p.children;\n\
             var f = document.createDocumentFragment();\n\
             var n1 = document.createElement('u'); f.appendChild(n1);\n\
             var n2 = document.createElement('s'); f.appendChild(n2);\n\
             p.insertBefore(f, b);\n\
             globalThis.__bOrder = (c1[0] === a && c1[1] === n1 && c1[2] === n2 && c1[3] === b);\n\
             globalThis.__bLen = c1.length;\n\
             var cc = document.getElementById('bc');\n\
             var h = cc.children;\n\
             globalThis.__iLen0 = h.length;\n\
             cc.insertAdjacentHTML('afterbegin', '<b></b><i></i>');\n\
             globalThis.__iOrder = (h[0].tagName === 'B' && h[1].tagName === 'I' && h[2].tagName === 'SPAN' && h[3].tagName === 'EM');\n\
             globalThis.__iLen = h.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__bOrder").unwrap().value,
        "true",
        "t8e：fragment 多节点批 insertBefore 保批内树序（外部锚 + addFlat 序）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__bLen").unwrap().value,
        "4",
        "t8e：fragment 批后集合计数正确"
    );
    assert_eq!(
        sandbox.execute("globalThis.__iLen0").unwrap().value,
        "2",
        "t8e：iAH 前集合基线"
    );
    assert_eq!(
        sandbox.execute("globalThis.__iOrder").unwrap().value,
        "true",
        "t8e：iAH afterbegin 批保批内树序（槽锚=旧首子，批节点落批序位）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__iLen").unwrap().value,
        "4",
        "t8e：iAH 批后集合计数正确"
    );
}

// v2 返修钉：锚不在集合（文本节点常见）沿 nextSibling 链走查落位——
// insertBefore(x, textNode) 的锚解析不得使 x 滞留尾部。
#[test]
fn t8e_children_text_anchor_walk() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    // 快照含文本子（SPAN 与 EM 之间夹文本）——锚=文本节点（不在元素集合中）。
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id=tc><span></span>tx<em></em></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8e-walk".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var tc = document.getElementById('tc');\n\
             var c = tc.children;\n\
             var t = tc.childNodes[1];\n\
             globalThis.__tIsText = (t && t.nodeType === 3);\n\
             var x = document.createElement('i');\n\
             tc.insertBefore(x, t);\n\
             globalThis.__wOrder = (c[0].tagName === 'SPAN' && c[1] === x && c[2].tagName === 'EM');\n\
             globalThis.__wLen = c.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__tIsText").unwrap().value,
        "true",
        "t8e：childNodes[1] 是夹层文本节点（锚来源）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__wOrder").unwrap().value,
        "true",
        "t8e：文本锚沿链走查到 EM，x 落 SPAN/EM 之间（不滞留尾部）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__wLen").unwrap().value,
        "3",
        "t8e：走查落位后集合计数正确"
    );
}

// v2 返修钉：同父移动（记录锚 null=移到尾 / 槽锚=移动目标位）——held 集合视图跟随。
#[test]
fn t8e_children_same_parent_move() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id=mv><span></span><em></em><b></b></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8e-move".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // handle 路径轮播（appendChild(firstChild) → [b,c,a]，记录锚 null=尾）；sel 路径
    // 快照节点移动（R334b 槽锚=目标位 ref）——sel-only 节点不进反链记账，走槽/动态读。
    sandbox
        .execute(
            "var p = document.createElement('div');\n\
             var a = document.createElement('span'); p.appendChild(a);\n\
             var b = document.createElement('em'); p.appendChild(b);\n\
             var c = document.createElement('u'); p.appendChild(c);\n\
             var c1 = p.children;\n\
             p.appendChild(a);\n\
             globalThis.__mOrder = (c1[0] === b && c1[1] === c && c1[2] === a);\n\
             globalThis.__mLen = c1.length;\n\
             var mv = document.getElementById('mv');\n\
             var mref = mv.children;\n\
             mv.insertBefore(mref[2], mref[0]);\n\
             globalThis.__smOrder = (mref[0].tagName === 'B' && mref[1].tagName === 'SPAN' && mref[2].tagName === 'EM');\n\
             globalThis.__smLen = mref.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__mOrder").unwrap().value,
        "true",
        "t8e：handle 路径同父尾移动（轮播）held 视图正确"
    );
    assert_eq!(
        sandbox.execute("globalThis.__mLen").unwrap().value,
        "3",
        "t8e：移动不改变集合计数"
    );
    assert_eq!(
        sandbox.execute("globalThis.__smOrder").unwrap().value,
        "true",
        "t8e：sel 快照节点同父移动落目标位（槽锚/动态读路径）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__smLen").unwrap().value,
        "3",
        "t8e：sel 移动不改变集合计数"
    );
}

// v2 返修钉（PR #96 审查 G5）：replaceChild 成员守恒与位次——新节点落被替换节点位
//（R100/R47 形态 record 无 nextSibling 锚，靠动态读落位，不得滞留尾部）。
#[test]
fn t8e_children_replace_child_member() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8e-replace".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // R100 handle-handle 形态（WPT replaceChild setup 同款）：[a,b,c] replace b → [a,n,c]。
    sandbox
        .execute(
            "var p = document.createElement('div');\n\
             var a = document.createElement('span'); p.appendChild(a);\n\
             var b = document.createElement('em'); p.appendChild(b);\n\
             var c = document.createElement('u'); p.appendChild(c);\n\
             var c1 = p.children;\n\
             var n = document.createElement('b');\n\
             p.replaceChild(n, b);\n\
             globalThis.__rOrder = (c1[0] === a && c1[1] === n && c1[2] === c);\n\
             globalThis.__rLen = c1.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__rOrder").unwrap().value,
        "true",
        "t8e：replaceChild 新节点落被替换位（无锚 record → 动态读落位）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rLen").unwrap().value,
        "3",
        "t8e：replaceChild 后集合计数守恒"
    );
}
