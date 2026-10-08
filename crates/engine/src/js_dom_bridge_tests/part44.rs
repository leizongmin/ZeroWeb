// slice44（live 集合残余三件）：
// ①静态选择器查询面 parsed 子树移除 stale（slice43 verdict 残余②探针新证在册）：
//   `querySelectorAll` / `querySelector` 对已移除 parsed 后代仍可解析至换代回收——
//   document 面（part06 `__zw_query_all`/`__zw_query_match`）无 pendingRemoved 消费，
//   元素面 R310 桶过滤仅覆盖「查询容器 === mutation 父」形态，跨容器桶外 stale。
//   本钉以常驻形态实证自然红（slice43 探针为一次性过程证据，未入提交）。
// ②`_zwDocContains36` 根部统一评估（slice43 缺陷轮 I-2 转池）：谓词以 body 为根，
//   head 子树 / documentElement 自身的命名元素漏判（spec named objects 限 document
//   tree 而非 body 子树）。本钉以 head 内挂载命名元素实证自然红 + detached/body
//   两对照臂（统一后零回退面）。
// ③move 生命周期钉覆盖（slice43 verdict 残余④ xS-1 钉候选）：sel 父带 parsed 后代
//   移回文档时后代经 addFlat 重并（R333 门）并从 pendingRemoved 摘除——语义已正确
//   （slice43 勘误），本钉覆盖「移除→回插→集合长度回复 / fresh 查询复见 / NA 重现」，
//   判别力由撤 added 方向消费点的变异 RED 承担（red/mutation-red-s44-*.log）。
// https://dom.spec.whatwg.org/#concept-node-list-alive
// https://dom.spec.whatwg.org/#concept-collection
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object

/// slice44 钉共用沙箱装配（s43_sandbox 同款：shim + 快照 + DOM 回调）。
macro_rules! s44_sandbox {
    ($html:expr) => {{
        use std::sync::{Arc, Mutex};
        use zero_script_sandbox::{Sandbox, V8Sandbox};

        let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox.execute(generate_js_dom_shim()).unwrap();
        let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
        let dom_html = Arc::new(Mutex::new($html.to_string()));
        let page_url = Arc::new(Mutex::new("https://zero.test/s44".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①静态选择器查询面：parsed 容器（sel 父）整树移除后，host 快照未 apply 窗口内
// document 面全查/单查与跨容器元素面查询仍可解析已移除 parsed 后代（spec：查询
// 面限当下 document tree——NodeList/查询结果不含已离树节点）。s43 已修复面
// （id 查询面 / NA 集合面 / fresh tag 面）作对照臂，证同族缺口收窄在查询面。
#[test]
fn static_selector_query_parsed_subtree_remove_s44() {
    let mut sandbox = s44_sandbox!(
        "<html><body><div id='w44'><img name='p44' id='c44'><img name='p44' id='d44'></div><img id='keep44'></body></html>"
    );
    sandbox
        .execute(
            "var na = window.p44;
             globalThis.__r_na0 = na.length;
             globalThis.__r_qsa0 = document.querySelectorAll('img').length;
             document.body.removeChild(document.getElementById('w44'));
             globalThis.__r_gone = String(document.getElementById('c44') === null);
             globalThis.__r_na_after = na.length;
             globalThis.__r_qsa_doc = document.querySelectorAll('img').length;
             globalThis.__r_qs_deep = String(document.querySelector('#w44 img') === null);
             globalThis.__r_qs_first = String((document.querySelector('img') || {}).id);
             globalThis.__r_qsa_html = document.documentElement.querySelectorAll('img').length;
             globalThis.__r_qsa_body = document.body.querySelectorAll('img').length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_na0").unwrap().value,
        "2",
        "双 name=p44 安装 live 集合（安装面基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_qsa0").unwrap().value,
        "3",
        "querySelectorAll('img') 基线 3（快照面成员：w44 双 img + keep44）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_gone").unwrap().value,
        "true",
        "对照：id 查询面即时 null（slice43 remFlat 展开顺带修复面，不回退）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_na_after").unwrap().value,
        "0",
        "对照：held NA 集合随移除清空（slice43 pin1 面，不回退）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_qsa_doc").unwrap().value,
        "1",
        "document.querySelectorAll('img') 不含已移除 parsed 后代（spec 查询面限当下 document tree；修前 stale 3）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_qs_deep").unwrap().value,
        "true",
        "document.querySelector('#w44 img') 不再解析已移除后代（修前可解析）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_qs_first").unwrap().value,
        "keep44",
        "document.querySelector('img') 首命中为在树元素（修前命中已移除 c44）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_qsa_html").unwrap().value,
        "1",
        "跨容器元素面（documentElement）查询不含已移除后代（R310 桶过滤仅覆盖 mutation 父桶；修前 stale 3）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_qsa_body").unwrap().value,
        "1",
        "同桶元素面（body === mutation 父）查询不含已移除后代（R310 桶过滤既有覆盖，对照不回退）"
    );
}

// ②`_zwDocContains36` 根部：head 内挂载 id 命名元素应入 named objects（spec 限
// document tree；body 根谓词漏判 head 子树）。对照臂：detached 赋名不入册（s43
// pin2 面）、body 挂载单命中元素全局（既有面零回退）。
#[test]
fn named_access_doc_tree_root_head_mount_s44() {
    let mut sandbox = s44_sandbox!("<html><head><title>t44</title></head><body><img name='r44' id='r1'></body></html>");
    sandbox
        .execute(
            "globalThis.__r_base = String(typeof window.r44);
             var st = document.createElement('style');
             document.head.appendChild(st);
             st.setAttribute('id', 'hd44');
             globalThis.__r_head_named = String(window.hd44 === st);
             var dt = document.createElement('img');
             dt.setAttribute('id', 'dt44');
             globalThis.__r_detached_named = String(window.dt44 === dt);
             var bi = document.createElement('img');
             bi.setAttribute('name', 'b44');
             document.body.appendChild(bi);
             globalThis.__r_body_named = String(window.b44 === bi);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_base").unwrap().value,
        "object",
        "body 内 name=img 安装期元素全局（安装面基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_head_named").unwrap().value,
        "true",
        "head 内挂载 id 命名元素入 named objects（spec document tree 判定；修前 body 根谓词漏判为 undefined）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_detached_named").unwrap().value,
        "false",
        "对照：detached 赋名不入册（连接性门排除面保持，s43 pin2 口径）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_body_named").unwrap().value,
        "true",
        "对照：body 挂载单命中元素全局（body 面行为零回退）"
    );
}

// ③move 生命周期：sel 父带 parsed 后代「移除→回插」——后代经 addFlat 重并
// （R333 门）并从 pendingRemoved 摘除，集合长度回复 / fresh 查询复见 / NA 重现
// （spec：集合视图限当下 document tree，重并即复见）。语义已正确（slice43 勘误
// 残余④），本钉为覆盖钉；判别力见变异 RED（撤 pendingRemoved 摘除点）。
#[test]
fn parsed_subtree_move_lifecycle_s44() {
    let mut sandbox = s44_sandbox!(
        "<html><body><div id='m44'><img name='mv44' id='m1'><img name='mv44' id='m2'></div></body></html>"
    );
    sandbox
        .execute(
            "var na = window.mv44;
             var box = document.getElementById('m44');
             globalThis.__r_na0 = na.length;
             document.body.removeChild(box);
             globalThis.__r_removed_na = na.length;
             globalThis.__r_removed_fresh = document.getElementsByTagName('img').length;
             globalThis.__r_removed_id = String(document.getElementById('m1') === null);
             document.body.appendChild(box);
             globalThis.__r_moved_na = na.length;
             globalThis.__r_moved_fresh = document.getElementsByTagName('img').length;
             globalThis.__r_moved_id = String(document.getElementById('m1') !== null);
             globalThis.__r_moved_win = (window.mv44 || {}).length || 0;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_na0").unwrap().value,
        "2",
        "双 name=mv44 安装 live 集合（安装面基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_removed_na").unwrap().value,
        "0",
        "移除向：held NA 集合清空（slice43 pin1 同族面，本钉基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_removed_fresh").unwrap().value,
        "0",
        "移除向：fresh tag 查询不复见（pendingRemoved 剔除面，本钉基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_removed_id").unwrap().value,
        "true",
        "移除向：id 查询面即时 null（本钉基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_moved_na").unwrap().value,
        "2",
        "回插向：held 集合长度回复 2（addFlat 重并 R333 门；变异 RED 撤摘除点即红）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_moved_fresh").unwrap().value,
        "2",
        "回插向：fresh 查询复见（pendingRemoved 摘除后剔除面失效）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_moved_id").unwrap().value,
        "true",
        "回插向：id 查询面复见"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_moved_win").unwrap().value,
        "2",
        "回插向：NA 全局读重现（window.mv44.length === 2）"
    );
}

