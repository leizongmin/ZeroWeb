// T42 P40：接口原型 Symbol.toStringTag 描述符 configurable 语义钉。sec_sdk/captcha
// 类脚本内嵌 core-js web.dom-iterables 对全局 DOM 接口原型 defineBuiltIn 重定义
// toStringTag（描述符带 configurable:true），守卫 classof(mi)!==symbol 恒真——
// own **不可配置** toStringTag 即抛 "Cannot redefine property:
// Symbol(Symbol.toStringTag)"，整个 bundle 顶层执行中止（juejin captcha 全 14 处
// 栈证）。WebIDL 接口原型对象 @@toStringTag = {writable:false, enumerable:false,
// configurable:true}（Chrome 实测对照全族 c=true；w/e 缺省 false 与 defineProperty
// 默认一致，故本组只钉 configurable 位点）。WPT 上游无单点用例（断言面拆散于
// 各接口 own-keys 测试），以等价本地沙箱钉常驻：Ke 臂 + 完整 dom-iterables 循环
// reduce + 同族描述符面。
// https://webidl.spec.whatwg.org/#interface-prototype-object

/// T42 钉共用沙箱装配（同 s37_sandbox!）。
macro_rules! t42_sandbox {
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
        let page_url = Arc::new(Mutex::new("https://zero.test/t42".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①SVG 列表族：dom-iterables 循环表在册 5 接口（抛错首点 SVGLengthList）。描述符
// configurable:true + core-js Ke 臂（configurable:true 描述符重定义）零抛。
#[test]
fn svg_list_tostringtag_configurable_t42() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var names42 = ['SVGLengthList','SVGNumberList','SVGPointList','SVGStringList','SVGTransformList'];
             globalThis.__r_out = names42.map(function (n) {
               var C = globalThis[n];
               if (!C || !C.prototype) return n + ':absent';
               var d = Object.getOwnPropertyDescriptor(C.prototype, Symbol.toStringTag);
               if (!d) return n + ':none';
               try {
                 Object.defineProperty(C.prototype, Symbol.toStringTag, { value: n, configurable: true });
               } catch (e) { return n + ':THROW'; }
               return n + ':' + d.configurable + ':' + C.prototype[Symbol.toStringTag];
             }).join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r_out").unwrap().value;
    for name in ["SVGLengthList", "SVGNumberList", "SVGPointList", "SVGStringList", "SVGTransformList"] {
        let seg = out.split('|').find(|s| s.starts_with(name)).unwrap_or("");
        assert_eq!(
            seg,
            &format!("{name}:true:{name}"),
            "{name} 原型 toStringTag 须 configurable:true 且可被 core-js Ke 臂重定义（WebIDL 接口原型对象）"
        );
    }
}

// ②完整循环 reduce：captcha/index.js dom-iterables 逐名走表（37 名，恒真守卫同款
// ——classof(mi)!==symbol 恒真、无 own 跳过）。全部在册接口零抛即修复判据；
// 修复前此处于 SVGLengthList 止步（红钉活体 loop_done=7）。
#[test]
fn dom_iterables_full_loop_reduce_t42() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var table42 = {CSSRuleList:0,CSSStyleDeclaration:0,CSSValueList:0,ClientRectList:0,DOMRectList:0,DOMStringList:0,DOMTokenList:1,DataTransferItemList:0,FileList:0,HTMLAllCollection:0,HTMLCollection:0,HTMLFormElement:0,HTMLSelectElement:0,MediaList:0,MimeTypeArray:0,NamedNodeMap:0,NodeList:1,PaintRequestList:0,Plugin:0,PluginArray:0,SVGLengthList:0,SVGNumberList:0,SVGPathSegList:0,SVGPointList:0,SVGStringList:0,SVGTransformList:0,SourceBufferList:0,StyleSheetList:0,TextTrackCueList:0,TextTrackList:0,TouchList:0};
             var hit42 = 0, threw42 = '';
             for (var vi in table42) {
               var gi = globalThis[vi], mi = gi && gi.prototype;
               if (!mi) continue;
               try { Object.defineProperty(mi, Symbol.toStringTag, { value: vi, configurable: true }); hit42++; }
               catch (e) { threw42 = vi + ':' + (e && e.message ? e.message : String(e)); break; }
             }
             globalThis.__r_hit = hit42;
             globalThis.__r_threw = threw42;",
        )
        .unwrap();
    let hit: u32 = sandbox.execute("globalThis.__r_hit").unwrap().value.parse().unwrap();
    let threw = sandbox.execute("globalThis.__r_threw").unwrap().value;
    assert_eq!(
        threw, "",
        "core-js dom-iterables 循环全程零抛（Cannot redefine property 族断根）"
    );
    assert!(
        hit >= 15,
        "在册接口原型覆盖数 {hit} 过少——shim 暴露面或循环走表异常"
    );
}

// ③同族描述符面：part05 事件族（Event + 子类泛型位点 MessageEvent）+ canvas/位图族
// （ImageData/CanvasFilter/ImageBitmap/OffscreenCanvas）+ MessagePort + SVG 非列表族
// （SVGAnimatedString/SVGLength/SVGRect/SVGMatrix）——同病位点一并钉 configurable:true。
#[test]
fn sibling_families_tostringtag_configurable_t42() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var fam42 = ['Event','MessageEvent','ProgressEvent','ImageData','CanvasFilter','ImageBitmap','OffscreenCanvas','MessagePort','SVGAnimatedString','SVGLength','SVGRect','SVGMatrix'];
             globalThis.__r_fam = fam42.map(function (n) {
               var C = globalThis[n];
               if (!C || !C.prototype) return n + ':absent';
               var d = Object.getOwnPropertyDescriptor(C.prototype, Symbol.toStringTag);
               if (!d) return n + ':none';
               try {
                 Object.defineProperty(C.prototype, Symbol.toStringTag, { value: n, configurable: true });
               } catch (e) { return n + ':THROW'; }
               return n + ':' + d.configurable;
             }).join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r_fam").unwrap().value;
    for seg in out.split('|') {
        assert!(
            seg.ends_with(":true"),
            "{seg}——接口原型 toStringTag 须 configurable:true（core-js Ke 臂可重定义；WebIDL 接口原型对象）"
        );
    }
}
// s229（html-semantics M2，2026-11-02）：`:active` 按压态槽归属契约——thread_local。
// 根因：M2-S196 曾用进程级 static Mutex 承载 per-document 按压态，runner rayon
// 多 job（每线程独立 WebView 顺序多页）下 A 页写入串进并发 B 页查询（WPT
// active-onblur button 页 `:active` 假匹配 body 实证——本页从未标记过 body）。
// 本测试钉两点：他线程写入的按压选择器在本线程 sync 后不得出现；同线程写读
// 回路保持。归属模型判据见 DOC_ACTIVE_SELECTOR 声明处注记。
// https://html.spec.whatwg.org/multipage/interaction.html#selector-active
#[test]
fn s229_doc_active_slot_thread_local() {
    let writer = std::thread::spawn(|| {
        DOC_ACTIVE_SELECTOR.with(|slot| *slot.borrow_mut() = "#s229-bleed-probe".to_string());
    });
    writer.join().expect("写入线程须正常结束");

    let mut doc = zero_dom::parse_html(
        "<html><body><div id=\"s229-bleed-probe\">x</div></body></html>",
    );
    sync_active_state(&mut doc);
    let hit = doc.query_selector(doc.root(), ":active");
    assert!(
        hit.is_none(),
        "他线程写入的按压选择器不得串入本线程查询视图"
    );

    DOC_ACTIVE_SELECTOR.with(|slot| *slot.borrow_mut() = "#s229-bleed-probe".to_string());
    sync_active_state(&mut doc);
    let hit = doc.query_selector(doc.root(), ":active");
    assert!(hit.is_some(), "同线程按压写入须在本线程查询可见");

    // 收尾清槽（测试共用主线程，不给他测试留串态）。
    DOC_ACTIVE_SELECTOR.with(|slot| *slot.borrow_mut() = String::new());
}

// T43 B6（site-optimizer perf-mem T43）：XMLHttpRequest 接口方法原型形状钉。站点包装
// 库（juejin sdk-glue 类 XHR 代理）按 `XMLHttpRequest.prototype.open` 习惯取方法——
// 修复前实例 own + 原型缺席 → undefined.apply 断链（glue 栈证 `at r.open … .apply`）。
// xhr.spec interface members 挂 interface prototype object；WebIDL §operations 原型层
// 描述符 {writable, enumerable, configurable}=true、arity=必选参数个数、name=操作名
//（WebIDL §create-an-operation-function；Chrome 对照 open=2/send=0（send body 为
// optional）—abort=0/setRequestHeader=2/getResponseHeader=1/getAllResponseHeaders=0/
// overrideMimeType=1，name 一一对应操作名）。https://xhr.spec.whatwg.org/#interface-xmlhttprequest
// https://webidl.spec.whatwg.org/#interface-prototype-object
#[test]
fn xhr_iface_methods_prototype_shape_t43() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var X = globalThis.XMLHttpRequest;
             var names = ['open','overrideMimeType','send','abort','setRequestHeader','getResponseHeader','getAllResponseHeaders'];
             var out = [];
             for (var i = 0; i < names.length; i++) {
               var n = names[i];
               var d = Object.getOwnPropertyDescriptor(X.prototype, n);
               if (!d || typeof d.value !== 'function') { out.push(n + ':absent'); continue; }
               out.push(n + ':' + d.writable + ':' + d.enumerable + ':' + d.configurable + ':' + d.value.length + ':' + d.value.name);
             }
             globalThis.__r = out.join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r").unwrap().value;
    // 期望段：op:w:e:c:arity:fname（WebIDL operations 全 true；arity=必选参数数——
    // send body optional → 0；fname=操作名——匿名委派器须显式补挂，否则 name 为 ''）。
    let expect = [
        "open:true:true:true:2:open",
        "overrideMimeType:true:true:true:1:overrideMimeType",
        "send:true:true:true:0:send",
        "abort:true:true:true:0:abort",
        "setRequestHeader:true:true:true:2:setRequestHeader",
        "getResponseHeader:true:true:true:1:getResponseHeader",
        "getAllResponseHeaders:true:true:true:0:getAllResponseHeaders",
    ];
    let segs: Vec<&str> = out.split('|').collect();
    assert_eq!(segs.len(), expect.len(), "七接口方法逐一在案：{out}");
    for (seg, want) in segs.iter().zip(expect.iter()) {
        assert_eq!(seg, want, "XHR 原型方法描述符/arity 须为 WebIDL operations 形：{seg}");
    }
}

// T43 B6 邻近变体：实例无 own 接口方法（Chrome parity）+ 双实例独立实现槽 +
// open→setRequestHeader 功能回路走原型委派。隔离性以实现槽闭包逐实例不同证明；
// 请求头入实例请求头槽（xhr.spec setRequestHeader 仅入 request headers——未发送时
// getAllResponseHeaders() 恒 ''，响应头契约）。
#[test]
fn xhr_instance_no_own_iface_methods_t43() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var X = globalThis.XMLHttpRequest;
             var a = new X(), b = new X();
             var names = ['open','overrideMimeType','send','abort','setRequestHeader','getResponseHeader','getAllResponseHeaders'];
             var ownBad = [];
             for (var i = 0; i < names.length; i++) {
               if (Object.prototype.hasOwnProperty.call(a, names[i])) ownBad.push(names[i]);
             }
             a.open('GET', 'https://zero.test/t43-a', true);
             a.setRequestHeader('X-T43', 'alpha');
             b.open('POST', 'https://zero.test/t43-b', true);
             b.setRequestHeader('X-T43', 'beta');
             globalThis.__r = [
               ownBad.join(','),
               String(a.readyState), String(b.readyState),
               String(a._zwXhrHeaders['x-t43'] === 'alpha'),
               String(b._zwXhrHeaders['x-t43'] === 'beta'),
               String(a._zwImpl_open !== b._zwImpl_open),
               String(a._zwXhrUrl !== b._zwXhrUrl),
               String(a.getAllResponseHeaders() === ''),
               String(Object.prototype.hasOwnProperty.call(a, '_zwImpl_send')),
               typeof X.prototype.open,
             ].join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r").unwrap().value;
    let parts: Vec<&str> = out.split('|').collect();
    assert_eq!(parts[0], "", "实例不得有 own 接口方法：{}", parts[0]);
    assert_eq!(parts[1], "1", "open 后 readyState=1（OPENED，功能面经原型委派生效）");
    assert_eq!(parts[2], "1");
    assert_eq!(parts[3], "true", "实例 A 请求头入槽（委派生效）");
    assert_eq!(parts[4], "true", "实例 B 请求头入槽（委派生效）");
    assert_eq!(parts[5], "true", "实现槽闭包逐实例独立（互不共享）");
    assert_eq!(parts[6], "true", "请求状态逐实例独立");
    assert_eq!(parts[7], "true", "未发送时 getAllResponseHeaders() 恒 ''（响应头契约）");
    assert_eq!(parts[8], "true", "实现槽为实例 own（非枚举内部槽）");
    assert_eq!(parts[9], "function");
}

// T43 B6 邻近变体：外来 this（分离调用/伪造宿主对象）→ TypeError（Illegal invocation
// 同型）；跨实例 .call 正确委派。
#[test]
fn xhr_prototype_method_detached_this_t43() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var X = globalThis.XMLHttpRequest;
             var detachedErr = '';
             try { X.prototype.open('GET', 'https://zero.test/x', true); }
             catch (e) { detachedErr = e.constructor.name + ':' + e.message; }
             var xhr = new X();
             var callOk = '';
             try { X.prototype.open.call(xhr, 'GET', 'https://zero.test/call', true); callOk = String(xhr.readyState); }
             catch (e) { callOk = 'THROW'; }
             globalThis.__r = detachedErr + '|' + callOk;",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r").unwrap().value;
    let parts: Vec<&str> = out.split('|').collect();
    // 仅断言构造器名不足以区分 guard 语义（修复前实例 own+原型缺席时分离调用抛
    // '… is not a function' 型 TypeError）——message 须为 Illegal invocation（同 Chrome
    // 拒绝外来 this 的文本），防委派器丢失 guard。
    assert_eq!(
        parts[0], "TypeError:Illegal invocation",
        "分离调用（this 无实现槽）须 TypeError('Illegal invocation')"
    );
    assert_eq!(parts[1], "1", "prototype.open.call(xhr,…) 委派生效");
}

// T43 B6 邻近变体（TV-F3）：iframe 包装路。part05.js IframeXMLHttpRequest 构造真实
// XMLHttpRequest 实例（stamp _zwXhrBaseUrl/_zwXhrFetch own 属性）并共享主 realm 原型——
// 原型委派不受实例自有附加属性遮蔽（委派器只查 this 的 _zwImpl_<name> 槽），wrapper
// 侧 open/setRequestHeader 与主 realm 同链同形状（无 own 接口方法）。
#[test]
fn xhr_iframe_wrapper_shared_prototype_t43() {
    let mut sandbox = t42_sandbox!("<html><body><iframe name='cw43'></iframe></body></html>");
    sandbox
        .execute(
            "var cw = document.getElementsByTagName('iframe')[0].contentWindow;
             var IX = cw.XMLHttpRequest;
             var ix = new IX();
             var names = ['open','overrideMimeType','send','abort','setRequestHeader','getResponseHeader','getAllResponseHeaders'];
             var ownBad = [];
             for (var i = 0; i < names.length; i++) {
               if (Object.prototype.hasOwnProperty.call(ix, names[i])) ownBad.push(names[i]);
             }
             ix.open('GET', 'sub/page.html', true);
             ix.setRequestHeader('X-T43', 'gamma');
             globalThis.__r = [
               String(Object.getPrototypeOf(ix) === globalThis.XMLHttpRequest.prototype),
               ownBad.join(','),
               String(ix.readyState),
               String(ix._zwXhrHeaders['x-t43'] === 'gamma'),
               typeof ix._zwXhrBaseUrl,
               typeof IX.prototype.open,
             ].join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r").unwrap().value;
    let parts: Vec<&str> = out.split('|').collect();
    assert_eq!(parts[0], "true", "wrapper 实例共享主 realm XHR 原型");
    assert_eq!(parts[1], "", "wrapper 实例无 own 接口方法（同主 realm 形状）");
    assert_eq!(parts[2], "1", "wrapper 侧 open 经共享原型委派生效");
    assert_eq!(parts[3], "true", "wrapper 侧 setRequestHeader 入本实例槽");
    assert_eq!(parts[4], "string", "wrapper stamp 属性在位（baseUrl）");
    assert_eq!(parts[5], "function", "共享原型委派器在案");
}
