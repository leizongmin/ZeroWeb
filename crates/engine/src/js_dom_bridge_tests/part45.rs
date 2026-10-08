// slice45（handle-form 查询 stale 收口三件）：
// ①document 面查询对 handle-form 移除条目 stale（slice44 verdict 残余⑤，缺陷轮 I-1
//   读码新证→本轮自然红实证）。实测机理**两层**（立项实验证伪「仅漏斗缺口」前提）：
//   层1 记账——apply 代际 bump（R379/pa2b）有意保留 pendingAdded 的 handle 条目
//   （re-append 移动语义），apply 后 remove() 走 R51c 消零对冲，移除**不入**
//   _zwPendingRemoved——identity 消费面（gEBI `_zwPRSet().has` / R125 祖先行走 /
//   s43 fresh tag 面）全部失去剔除依据（slice44 verdict「ID 面两态均覆盖无此缺口」
//   的对照前提只对 apply 前窗口与 sel-form 成立）；层2 漏斗——即使入表，
//   `_zwPendingRemovedSels()` 只收含 `__zwSelector` 的 sel-form 条目（createElement
//   产物 `_wrapHandle` → `_makeProxy(null, handle)` 的 `__zwSelector` trap 恒 null），
//   document QS/QSA（slice44 sel 串过滤）对 handle-form stale 命中不剔除。
//   批次边界模拟镜像生产 apply 路径：drain 队列 → `append_mutation_history`（webview
//   apply 前同款，供 handle attr 读回走 latest-wins 重放）→ 持久 handle→selector 表 +
//   `apply_dom_mutations_with_persistent` 落快照（webview render_with_dom_mutations_
//   persistent 同源）→ `__zw_handle_for_selector` / `__zw_selector_for_handle` 双向
//   identity 桥（webview register_identity_bridge_callback / register_forward_identity_
//   bridge_callback 镜像）→ `__zw_apply_generation_bump`。r5010（part25）先例扩展。
//   https://dom.spec.whatwg.org/#concept-node-list-alive
//   https://dom.spec.whatwg.org/#dom-element-remove
//   https://dom.spec.whatwg.org/#dom-parentnode-queryselector

/// slice45 钉①：handle-form（createElement 产物）节点 apply 后同批 remove，
/// document 面 QS/QSA 不得再返回该节点（spec 查询面限当下 document tree）；
/// ID 面作对照臂（identity 消费面与 document 面同窗闭合）；换代后换收。
#[test]
fn handle_form_remove_document_query_stale_s45() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><img id='keep45'></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s45".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 N：createElement（handle-form）→ 赋 id → appendChild（mutation 入队，host 快照未落）。
    // 同 turn 正向臂（测效 I1）：e2 create+append+remove 全在同一 turn（未 apply）——
    // 同代际条目消零保持（`_zwPaGen45` === 当前代际，谓词跳过），apply#1 后节点按
    // spec 不在查询可见面（append+remove 对冲落快照）。防跨代际门误吞同 turn 场景。
    // 再登记母体（测效 S2）：e3（div，turn1 无 id）append 后留活——apply#1 落快照
    // （代际戳 gen0），批 N+1 跨 turn 补 id 触发 part04 id 重登记路径（_zwPAIdAdd
    // 再入）后移除——若重登记刷新代际戳（M-E），消零谓词的代际门把已 apply 节点
    // 误判为同 turn，移除不入 removed 表。
    sandbox
        .execute(
            "var d = document.createElement('img');\
             d.id = 'dyn45';\
             document.body.appendChild(d);\
             var e2 = document.createElement('span');\
             e2.id = 'dyn45s';\
             document.body.appendChild(e2);\
             e2.remove();\
             var e3 = document.createElement('div');\
             document.body.appendChild(e3);\
             globalThis.__r_s45_pre = document.querySelectorAll('img').length;",
        )
        .unwrap();

    // 批次边界 apply #1：mutation 历史 + 持久表 + 快照落定 + 双向 identity 桥 + 代际 bump。
    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch1.is_empty(), "批 N 必须产出 mutation 记录（createElement/setAttr/append）");
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch1);
    let mut persistent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch1,
            Some(&persistent),
        )
        .expect("批次边界 apply #1：handle-form 节点落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    // identity 桥双向 stub（生产 webview/renderer 注册的镜像）：selector→handle（R100
    // 反查）与 handle→selector（R145 正置——层1 修复的快照真含判定依赖此向）。
    let sel_to_handle: std::collections::HashMap<String, String> = persistent
        .iter()
        .map(|(h, s)| (s.clone(), h.clone()))
        .collect();
    sandbox.register_callback(
        "__zw_handle_for_selector",
        Box::new(move |args: &[String]| -> String {
            sel_to_handle
                .get(args.first().map(String::as_str).unwrap_or(""))
                .cloned()
                .unwrap_or_default()
        }),
    );
    let fwd_map = persistent.clone();
    sandbox.register_callback(
        "__zw_selector_for_handle",
        Box::new(move |args: &[String]| -> String {
            fwd_map
                .get(args.first().map(String::as_str).unwrap_or(""))
                .cloned()
                .unwrap_or_default()
        }),
    );
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // 批 N+1：apply 已落窗口内 remove → document 面观测（QS / QSA / gEBI 对照臂）。
    // 同 turn 正向臂观测：apply#1 后 e2（append+remove 同 turn 对冲）不在查询可见面。
    // 再登记臂：跨 turn 对已 apply 的 e3 补 id（part04 id 重登记路径，`_zwPaGen45`
    // 首登不覆盖——代际戳保持 gen0）→ remove → 消零谓词代际门放行 → 移除入
    // _zwPendingRemoved → 漏斗对 e3 的 host stale 命中剔除（qsa_div=0）。若重登记
    // 刷新代际戳（M-E），谓词被代际门跳过 → 不入表 → qsa_div=1。
    sandbox
        .execute(
            "globalThis.__r_s45_st_qs = String(document.querySelector('#dyn45s') === null);\
             globalThis.__r_s45_st_id = String(document.getElementById('dyn45s') === null);\
             globalThis.__r_s45_applied_raw = String(__zw_query_all('img'));\
             globalThis.__r_s45_applied_id = String(document.getElementById('dyn45') === d);\
             d.remove();\
             globalThis.__r_s45_after_qs = String(document.querySelector('#dyn45') === null);\
             globalThis.__r_s45_after_qsa = document.querySelectorAll('img').length;\
             globalThis.__r_s45_after_id = String(document.getElementById('dyn45') === null);\
             e3.setAttribute('id', 'dyn45b');\
             e3.remove();\
             globalThis.__r_s45_rb_qsa_div = document.querySelectorAll('div').length;",
        )
        .unwrap();

    // 批次边界 apply #2（换代）：Remove 落快照——stale 窗口闭合（换代即愈）。
    let batch2: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch2.is_empty(), "remove 必须产出 Remove mutation 记录");
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch2);
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch2,
            Some(&persistent),
        )
        .expect("批次边界 apply #2：Remove 落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();
    sandbox
        .execute(
            "globalThis.__r_s45_gen2_qs = String(document.querySelector('#dyn45') === null);\
             globalThis.__r_s45_gen2_qsa = document.querySelectorAll('img').length;\
             globalThis.__r_s45_rb_gen2_qs = String(document.querySelector('#dyn45b') === null);\
             globalThis.__r_s45_rb_gen2_div = document.querySelectorAll('div').length;",
        )
        .unwrap();

    // 单点聚合断言：红 log 一次带出全部观测值（slice44 打红名单不可追溯教训）。
    // applied_raw 断言 host 快照真值（QSA 聚合计数受既有 `_elKeyOf` 未定义缺陷污染——
    // R161 pending-tag 回退 dedup 恒死（ReferenceError 被吞），apply 后 tag 形查询重复
    // 计入 pending 条目，属相邻既有缺陷，本钉不覆盖不申报为修复面）。
    let mut read = |name: &str| -> String {
        sandbox
            .execute(&format!("String(globalThis.{name})"))
            .unwrap()
            .value
    };
    let mut fails: Vec<String> = Vec::new();
    let mut expect = |fails: &mut Vec<String>, name: &str, want: &str| {
        let got = read(name);
        if got != want {
            fails.push(format!("{name}: want {want:?}, got {got:?}"));
        }
    };
    expect(&mut fails, "__r_s45_pre", "2");
    expect(&mut fails, "__r_s45_st_qs", "true");
    expect(&mut fails, "__r_s45_st_id", "true");
    expect(&mut fails, "__r_s45_applied_raw", "#keep45|#dyn45");
    expect(&mut fails, "__r_s45_applied_id", "true");
    expect(&mut fails, "__r_s45_after_qs", "true");
    expect(&mut fails, "__r_s45_after_qsa", "1");
    expect(&mut fails, "__r_s45_after_id", "true");
    expect(&mut fails, "__r_s45_rb_qsa_div", "0");
    expect(&mut fails, "__r_s45_gen2_qs", "true");
    expect(&mut fails, "__r_s45_gen2_qsa", "1");
    expect(&mut fails, "__r_s45_rb_gen2_qs", "true");
    expect(&mut fails, "__r_s45_rb_gen2_div", "0");
    assert!(
        fails.is_empty(),
        "handle-form remove 后 document 面查询 stale 观测（诊断：after_id=false ⇒ 层1 消零对冲\
         未入 removed 表；after_id=true 且 QS/QSA stale ⇒ 层2 漏斗不收 handle-form）：\n{}",
        fails.join("\n")
    );
}

/// slice45 钉③：morph 跟随面 >1 命中升格（slice42 残余申报收口）。spec 取值算法
/// 每读按当下 named objects 求值：多命中返 HTMLCollection、单命中返元素。2→1 时
/// 集合 morph 成元素全局（既有基线，slice32/42），1→2 重入时 morph 产物跟随臂
/// `>1 命中不升格`（slice42 口径）使全局停留元素——spec 偏差（Chrome 同 spec：
/// window.x = HTMLCollection）。修后跟随臂镜像恢复臂（g === undefined 臂 ≥2 恢复
/// 集合）升格回 installed 集合。
/// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
#[test]
fn named_access_morph_follow_multi_hit_upgrade_s45() {
    let mut sandbox = s42_sandbox!(
        "<html><body><img name='uu' id='u1'><img name='uu' id='u2'></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.uu;
             globalThis.__r_len0 = col0.length;
             document.getElementById('u1').setAttribute('name', 'off');
             globalThis.__r_morph = window.uu === document.getElementById('u2');
             document.getElementById('u1').setAttribute('name', 'uu');
             globalThis.__r_is_col = window.uu === col0;
             globalThis.__r_len = window.uu.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "双 name=uu 安装集合（slice30 基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_morph").unwrap().value,
        "true",
        "跌破单命中 morph 元素（spec 取值算法，slice32 基线不回退）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_is_col").unwrap().value,
        "true",
        "1→2 重入升格回集合（spec：多命中 named objects 返 HTMLCollection；修前\
         morph 产物停留元素全局）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len").unwrap().value,
        "2",
        "升格后全局反映双命中（live 维护自然延续）"
    );
}


/// t8k WebSocket 钉测共用 mock 宿主装配：shim + DOM 回调 + `__zw_ws_*` mock。
/// mock 契约：connect 记录 url/protocols 并存 `__lastWsId`；next 登记 pid；
/// `__wsEmit(id, wire)` 经 setTimeout(0) 走 `__zwResolveCallback`（test sandbox 零延迟泵送）。
macro_rules! ws45_sandbox {
    () => {{
        use std::sync::{Arc, Mutex};
        use zero_script_sandbox::{Sandbox, V8Sandbox};

        let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox.execute(generate_js_dom_shim()).unwrap();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
        let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
        let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
        let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
            std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
            .execute(
                "globalThis.__wsCalls = [];\
                 globalThis.__lastWsId = '';\
                 globalThis.__wsPids = {};\
                 globalThis.__zw_ws_connect = function(id, url, protocols, origin, cookie) {\
                   globalThis.__lastWsId = id;\
                   globalThis.__wsCalls.push('connect|' + id + '|' + url + '|' + protocols + '|' + origin + '|' + cookie);\
                 };\
                 globalThis.__zw_ws_next = function(id, pid) { globalThis.__wsPids[id] = pid; };\
                 globalThis.__zw_ws_send = function(id, kind, data) {\
                   globalThis.__wsCalls.push('send|' + id + '|' + kind + '|' + data);\
                 };\
                 globalThis.__zw_ws_close = function(id, code, reason) {\
                   globalThis.__wsCalls.push('close|' + id + '|' + code + '|' + reason);\
                 };\
                 globalThis.__wsEmit = function(id, wire) {\
                   var pid = globalThis.__wsPids[id];\
                   if (pid) setTimeout(function() { globalThis.__zwResolveCallback(pid, wire); }, 0);\
                 };",
            )
            .unwrap();
        sandbox
    }};
}

// 主钉：ctor → open（protocol 回填）→ text message（origin/url）→ send 转发 → 服务端 close
//（wasClean/code/reason）→ send 已 CLOSED 再抛 InvalidStateError。
#[test]
fn test_t8k_websocket_event_flow_open_message_close() {
    let mut sandbox = ws45_sandbox!();
    sandbox
        .execute(
            "globalThis.__events = [];\
             var ws = new WebSocket('ws://example.com/socket', ['chat', 'v2']);\
             globalThis.__ws = ws;\
             ws.onopen = function() {\
               globalThis.__events.push('open:' + ws.readyState + ':' + ws.protocol + ':' + ws.url);\
             };\
             ws.onmessage = function(e) { globalThis.__events.push('msg:' + e.data + ':' + e.origin); };\
             ws.onclose = function(e) {\
               globalThis.__events.push('close:' + e.wasClean + ':' + e.code + ':' + e.reason + ':' + ws.readyState);\
             };\
             ws.onerror = function() { globalThis.__events.push('error'); };",
        )
        .unwrap();
    // 构造态：CONNECTING(0) + url 序列化 + 常量面（HTML spec §WebSocket readyState）。
    assert_eq!(
        sandbox.execute("globalThis.__ws.readyState + ':' + WebSocket.CONNECTING + WebSocket.OPEN + WebSocket.CLOSING + WebSocket.CLOSED").unwrap().value,
        "0:0123"
    );
    assert_eq!(sandbox.execute("globalThis.__ws.url").unwrap().value, "ws://example.com/socket");
    // 连接期 send → InvalidStateError（spec send：state ≠ OPEN 抛 InvalidStateError）。
    assert_eq!(
        sandbox
            .execute(
                "var invalidBefore = 'no';\
                 try { globalThis.__ws.send('early'); } catch (e) {\
                   invalidBefore = (e instanceof DOMException && e.name === 'InvalidStateError') ? 'yes' : ('wrong:' + e);\
                 } invalidBefore"
            )
            .unwrap()
            .value,
        "yes"
    );
    assert_eq!(sandbox.execute("globalThis.__wsCalls.length").unwrap().value, "1");
    assert_eq!(
        sandbox.execute("globalThis.__wsCalls[0]").unwrap().value,
        "connect|ws1|ws://example.com/socket|chat,v2|ws://example.com|"
    );
    // open wire → open 事件 + protocol 回填 + OPEN 态 send（text）转发。
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'open\\x1fchat')").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__events[0] || '')").unwrap().value, "open:1:chat:ws://example.com/socket");
    sandbox.execute("globalThis.__ws.send('hello-t8k')").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__wsCalls[1]").unwrap().value,
        "send|ws1|t|hello-t8k"
    );
    // msg wire（数据末字段含 \x1f 须原样保留）→ message 事件，origin = 连接 URL 的 origin。
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'msg\\x1fa\\x1fb')").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__events[1] || '')").unwrap().value,
        "msg:a\x1fb:ws://example.com"
    );
    // close wire（clean close 1000）→ CloseEvent 面齐全 + readyState CLOSED。
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'close\\x1f1000\\x1f1\\x1fdone')").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__events[2] || '')").unwrap().value,
        "close:true:1000:done:3"
    );
    // close 后 send → InvalidStateError；close() 幂等（不重复发宿主命令）。
    assert_eq!(
        sandbox
            .execute(
                "var invalidAfter = 'no';\
                 try { globalThis.__ws.send('late'); } catch (e) {\
                   invalidAfter = (e instanceof DOMException && e.name === 'InvalidStateError') ? 'yes' : ('wrong:' + e);\
                 } invalidAfter"
            )
            .unwrap()
            .value,
        "yes"
    );
    let before = sandbox.execute("globalThis.__wsCalls.length").unwrap().value;
    sandbox.execute("globalThis.__ws.close()").unwrap();
    assert_eq!(sandbox.execute("globalThis.__wsCalls.length").unwrap().value, before);
}

// 构造器校验面：URL 解析失败 / 非 ws scheme / 空 protocol / 重复 protocol → SyntaxError
// DOMException（HTML spec 构造 step 2–4）；close(code) 非 1000/3000–4999、reason > 123 字节
// → SyntaxError（spec close 校验）。
#[test]
fn test_t8k_websocket_constructor_validation() {
    let mut sandbox = ws45_sandbox!();
    let cases = r#"
      function ctorErr(expr) {
        try { eval(expr); return 'none'; } catch (e) {
          return (e instanceof DOMException && e.name === 'SyntaxError') ? 'SyntaxError' : ('wrong:' + e);
        }
      }
      [
        ctorErr("new WebSocket('not a url')"),
        ctorErr("new WebSocket('http://example.com/x')"),
        ctorErr("new WebSocket('ftp://example.com/x')"),
        ctorErr("new WebSocket('ws://example.com/s', [''])"),
        ctorErr("new WebSocket('ws://example.com/s', ['chat', 'chat'])"),
        ctorErr("new WebSocket('ws://example.com/s', 'a,b')"),
      ].join('|')"#;
    assert_eq!(
        sandbox
            .execute(cases)
            .unwrap()
            .value,
        "SyntaxError|SyntaxError|SyntaxError|SyntaxError|SyntaxError|SyntaxError"
    );
    // 合法面：string protocols 单条 + 3000–4999 close code 均放行（不再抛）。
    sandbox
        .execute(
            "var ok = 'no';\
             try { var wsp = new WebSocket('ws://example.com/ok', 'chat'); \
                   wsp.close(3000, 'bye'); ok = (wsp.readyState === WebSocket.CLOSING) ? 'yes' : ('state:' + wsp.readyState); }\
             catch (e) { ok = 'threw:' + e; } ok",
        )
        .unwrap();
    assert_eq!(sandbox.execute("ok").unwrap().value, "yes");
    // close code 越界（1001 保留段 / 2999 私用段下界外）→ SyntaxError。
    assert_eq!(
        sandbox
            .execute(
                "function closeErr(expr) {\
                   try { eval(expr); return 'none'; } catch (e) {\
                     return (e instanceof DOMException && e.name === 'SyntaxError') ? 'SyntaxError' : ('wrong:' + e);\
                   }\
                 }\
                 [closeErr(\"wsp.close(1001)\"), closeErr(\"wsp.close(2999)\"),\
                  closeErr(\"wsp.close(1000, new Array(125).join('x'))\")].join('|')"
            )
            .unwrap()
            .value,
        "SyntaxError|SyntaxError|SyntaxError"
    );
}

// 本地 close()：OPEN 态 → CLOSING 过渡 + 宿主命令（code/reason 穿参）；宿主随后回 close wire
// → close 事件（wasClean=true）收尾。CONNECTING 态 close → fail the connection（宿主回
// clean=false close wire）。
#[test]
fn test_t8k_websocket_local_close_handshake() {
    let mut sandbox = ws45_sandbox!();
    sandbox
        .execute(
            "globalThis.__ev = [];\
             var ws = new WebSocket('ws://example.com/close-me');\
             globalThis.__wsc = ws;\
             ws.onclose = function(e) { globalThis.__ev.push('close:' + e.wasClean + ':' + e.code + ':' + ws.readyState); };",
        )
        .unwrap();
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'open\\x1f')").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__wsc.readyState)").unwrap().value, "1");
    sandbox.execute("globalThis.__wsc.close(1000, 'bye')").unwrap();
    // CLOSING 过渡 + 宿主命令穿参。
    assert_eq!(sandbox.execute("String(globalThis.__wsc.readyState)").unwrap().value, "2");
    assert_eq!(
        sandbox.execute("globalThis.__wsCalls[globalThis.__wsCalls.length - 1]").unwrap().value,
        "close|ws1|1000|bye"
    );
    // 宿主确认 close wire → close 事件 + CLOSED。
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'close\\x1f1000\\x1f1\\x1fbye')").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ev[0] || '')").unwrap().value, "close:true:1000:3");
    // CONNECTING 态 close → fail：宿主回 clean=false close wire → wasClean=false close 事件。
    sandbox
        .execute(
            "globalThis.__ev2 = [];\
             var ws2 = new WebSocket('ws://example.com/abort');\
             globalThis.__wsc2 = ws2;\
             ws2.onclose = function(e) { globalThis.__ev2.push('close:' + e.wasClean + ':' + e.code); };",
        )
        .unwrap();
    sandbox.execute("globalThis.__wsc2.close()").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__wsCalls[globalThis.__wsCalls.length - 1]").unwrap().value,
        "close|ws2|0|"
    );
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'close\\x1f1006\\x1f0\\x1f')").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ev2[0] || '')").unwrap().value, "close:false:1006");
}

// binary wire：binaryType='blob'（默认）→ message data 为 Blob；binaryType='arraybuffer' →
// ArrayBuffer；send(Uint8Array) → binary wire csv-decimal。
#[test]
fn test_t8k_websocket_binary_frames() {
    let mut sandbox = ws45_sandbox!();
    sandbox
        .execute(
            "globalThis.__data = [];\
             var ws = new WebSocket('ws://example.com/bin');\
             globalThis.__wsb = ws;\
             ws.onmessage = function(e) {\
               globalThis.__data.push(e.data instanceof Blob ? 'blob:' + e.data.size\
                 : (e.data instanceof ArrayBuffer ? 'ab:' + e.data.byteLength : 'other'));\
             };",
        )
        .unwrap();
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'open\\x1f')").unwrap();
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'bin\\x1f104,105,0,255')").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__data[0] || '')").unwrap().value, "blob:4");
    sandbox.execute("globalThis.__wsb.binaryType = 'arraybuffer'").unwrap();
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'bin\\x1f1,2')").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__data[1] || '')").unwrap().value, "ab:2");
    // send(Uint8Array) → 'b' wire csv。
    sandbox.execute("globalThis.__wsb.send(new Uint8Array([72, 255]))").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__wsCalls[globalThis.__wsCalls.length - 1]").unwrap().value,
        "send|ws1|b|72,255"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__wsb.bufferedAmount >= 2 ? 'grown' : 'stuck'")
            .unwrap()
            .value,
        "grown"
    );
}

// err wire 非终结：onerror 派发、readyState 不变、泵继续（后续 msg 照常投递）。
#[test]
fn test_t8k_websocket_error_nonterminal() {
    let mut sandbox = ws45_sandbox!();
    sandbox
        .execute(
            "globalThis.__ev3 = [];\
             var ws = new WebSocket('ws://example.com/err');\
             globalThis.__wse = ws;\
             ws.onerror = function() { globalThis.__ev3.push('error:' + ws.readyState); };\
             ws.onmessage = function(e) { globalThis.__ev3.push('msg:' + e.data); };",
        )
        .unwrap();
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'open\\x1f')").unwrap();
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'err\\x1freset')").unwrap();
    sandbox.execute("globalThis.__wsEmit(globalThis.__lastWsId, 'msg\\x1fafter-error')").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ev3.join('|')").unwrap().value,
        "error:1|msg:after-error"
    );
}

// 宿主缺失（webview 未配置 / 裸 engine-reftest 环境）：构造不抛 ReferenceError，异步
// fail the connection——error + close(1006, wasClean=false)（HTML spec fail the
// WebSocket connection 语义）。
#[test]
fn test_t8k_websocket_no_host_graceful_degradation() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 不注入 __zw_ws_* mock——裸 shim 环境。
    sandbox
        .execute(
            "globalThis.__ev4 = [];\
             var ws = null;\
             try { ws = new WebSocket('ws://example.com/x'); } catch (e) { globalThis.__ev4.push('threw:' + e); }\
             globalThis.__wsn = ws;\
             if (ws) {\
               ws.onerror = function() { globalThis.__ev4.push('error:' + ws.readyState); };\
               ws.onclose = function(e) { globalThis.__ev4.push('close:' + e.wasClean + ':' + e.code + ':' + ws.readyState); };\
             }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ev4.join('|')").unwrap().value,
        "error:3|close:false:1006:3"
    );
}

// ── t8k 第二断层：createEvent + initEvent + dispatchEvent legacy 三连（native overlay）──
//
// 同页空白菜的伴生根因：shim `document.createEvent` 在 **native** Event 实例上置
// `_zwUninitialized`（js-dom M4 R106 spec initialized flag），而页面实际解析到的
// initEvent 是 native 模板版（dom_bindings R3141——先于 flag 契约，不清 flag）；
// shim 版 initEvent 因「`Event.prototype.initEvent` 已存在则不覆盖」守卫（part05 R106）
// 永不安装 → dispatch 守卫 `_zwDispatchGuard` 见 flag 仍置 → InvalidStateError。
// bili-header emitter（`createEvent("HTMLEvents")` + initEvent + dispatch）与 core-js
// unhandledrejection polyfill（`createEvent("Event")` + expando + initEvent + dispatch）
// 均按此三连构造，真实站实证报 `Uncaught InvalidStateError: The event is not initialized`，
// 头部初始化链死亡、播放器不挂载。
// spec：initEvent 属 initialize 步骤，须设 initialized flag——
// https://dom.spec.whatwg.org/#concept-event-initialize
// https://dom.spec.whatwg.org/#dom-event-initevent
#[test]
fn test_t8k_create_event_init_event_dispatch_native_overlay() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 生产 overlay：native V8 绑定 + shim 双装（真实页面环境——globalThis.Event 为 native 模板，
    // `Event.prototype.initEvent` 由 native 提供、shim 版永不覆盖）。
    sandbox.install_native_bindings(Box::new(|scope, ctx| {
        let dom = std::rc::Rc::new(std::cell::RefCell::new(zero_dom::parse_html(
            "<html><body></body></html>",
        )));
        crate::dom_bindings::install_dom_bindings(scope, ctx, dom);
    }));
    // 三序列：① bili-header emit 原样（HTMLEvents + initEvent + dispatch，监听器须触发）；
    // ② core-js polyfill 变体（Event + expando 先置 + initEvent 后初始化）；
    // ③ R106 防过修对照——createEvent 后不经 initEvent 直接 dispatch 仍须抛 InvalidStateError
    //（initialized flag 契约不可因本修复失效）。
    sandbox
        .execute(
            "globalThis.__out = '';\
             try {\
               var fired = 0;\
               document.addEventListener('onlogin', function() { fired++; }, false);\
               var n = document.createEvent('HTMLEvents');\
               n.initEvent('onlogin', true, true);\
               n.data = 1;\
               var ret = document.dispatchEvent(n);\
               globalThis.__out += 'bili:' + ret + ',' + fired;\
             } catch (e) { globalThis.__out += 'bili:ERR:' + e.name; }\
             try {\
               var m = document.createEvent('Event');\
               m.promise = null; m.reason = null;\
               m.initEvent('unhandledrejection', false, true);\
               document.dispatchEvent(m);\
               globalThis.__out += '|corejs:ok';\
             } catch (e) { globalThis.__out += '|corejs:ERR:' + e.name; }\
             try {\
               var u = document.createEvent('Event');\
               document.dispatchEvent(u);\
               globalThis.__out += '|uninit:no-throw';\
             } catch (e) { globalThis.__out += '|uninit:' + e.name; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__out").unwrap().value,
        "bili:true,1|corejs:ok|uninit:InvalidStateError",
        "createEvent+initEvent+dispatchEvent legacy 三连在 native overlay 须完整走通；未初始化事件仍须抛 InvalidStateError（R106 契约保持）"
    );
}
