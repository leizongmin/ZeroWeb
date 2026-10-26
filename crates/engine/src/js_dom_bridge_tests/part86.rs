// part86 — html-semantics M2 切片 69（R5276，2026-10-24）：img.decode() promise 面。
//
// 根因：HTMLImageElement 无 decode IDL（get trap 无 decode 臂，`img.decode` undefined
// → TypeError）——image-decode 族 56F。三刀：① 共享解码核 `__zwImgDecodeCore`
//（part03 SVG 接口环后，IIFE 顶层；select-image-source 近似候选 + 调用时捕获 +
// 恒异步 settle + 活跃文档判别 + 终态/available-images 即决 + 自驱 fetch 链 +
// settle 复检候选变异/adopt 时序——decode 只读不写 settle 状态）；② part04 资源
// get-trap IMG decode 臂（属性读通道 latest-wins + 资源 key）+ part03 detached
// createElement IMG 产物动态臂（调用时活跃性判别——adopt-in 后可达）；③
// SVGImageElement.prototype.decode（href → xlink:href 候选）。
// https://html.spec.whatwg.org/multipage/embedded-content.html#dom-img-decode

/// 钉①：decode 决策序——(a) 无 src/srcset → EncodingError；(b) fetch 链成功 →
/// resolve undefined；(c) 真解码失败（createImageBitmap 拒绝）→ EncodingError；
/// (d) InvalidStateError（host 零尺寸/烂数据同错误名）→ EncodingError（不复刻
/// R56h setter 链的 loaded 臂——corrupt 案断言面）；(e) fetch !ok → EncodingError；
/// (f) srcset 首 token 候选；(g) srcset 变异后旧 promise EncodingError（settle
/// 复检）、新 promise 按新 URL；(h) 同值重赋不算变异；(i) multiple decodes 各自
/// resolve 且 promise 不同一。
/// 断言翻转：decode 臂撤（(b)-(i) 全 TypeError 红）。
#[test]
fn img_decode_promise_r5276() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s69".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // fetch/createImageBitmap stub：URL → 响应变体；位图解码按 blob.__z 口径分流
    //（__zOK 成功 / __zZero InvalidStateError / 缺省真失败）。计数器钉 SVG/位图调用面。
    sandbox
        .execute(
            "globalThis.__bitmapCalls = 0;\
             globalThis.createImageBitmap = function (blob) {\
               globalThis.__bitmapCalls++;\
               if (blob && blob.__zOK) return Promise.resolve({ width: blob.__zW || 4, height: blob.__zH || 2 });\
               if (blob && blob.__zZero) { var e = new Error('zero-size'); e.name = 'InvalidStateError'; return Promise.reject(e); }\
               return Promise.reject(new TypeError('decode fail'));\
             };\
             globalThis.fetch = function (url) {\
               var u = String(url);\
               if (u.indexOf('ok.png') >= 0 || u.indexOf('good.png') >= 0) return Promise.resolve({ ok: true, status: 200, blob: function () { return Promise.resolve({ __zOK: 1, __zW: 10, __zH: 20 }); } });\
               if (u.indexOf('zero.png') >= 0) return Promise.resolve({ ok: true, status: 200, blob: function () { return Promise.resolve({ __zZero: 1 }); } });\
               if (u.indexOf('bad.png') >= 0) return Promise.resolve({ ok: true, status: 200, blob: function () { return Promise.resolve({}); } });\
               return Promise.resolve({ ok: false, status: 404 });\
             };",
        )
        .unwrap();

    sandbox
        .execute(
            "globalThis.__r = {};\
             var __track = function (name, p) { return Promise.resolve(p).then(\
               function (v) { globalThis.__r[name] = 'fulfilled:' + String(v); },\
               function (e) { globalThis.__r[name] = 'rejected:' + (e && e.name); }); };\
             var mk = function () { return document.createElement('img'); };\
             // (a) 无候选
             __track('noSrc', mk().decode());\
             var a2 = mk(); a2.setAttribute('src', ''); __track('emptySrc', a2.decode());\
             // (b) fetch 链成功
             var b = mk(); b.setAttribute('src', 'ok.png'); __track('okChain', b.decode());\
             // (c) 真解码失败
             var c = mk(); c.setAttribute('src', 'bad.png'); __track('badChain', c.decode());\
             // (d) 零尺寸 InvalidStateError 臂
             var d0 = mk(); d0.setAttribute('src', 'zero.png'); __track('zeroChain', d0.decode());\
             // (e) fetch !ok
             var e0 = mk(); e0.setAttribute('src', 'nope.png'); __track('notFound', e0.decode());\
             // (f) srcset 首 token 候选
             var f0 = mk(); f0.setAttribute('srcset', 'ok.png 100w 2x'); __track('srcsetOnly', f0.decode());\
             // (g) srcset 变异：旧 reject / 新按新 URL resolve
             var g = mk(); g.setAttribute('srcset', 'ok.png 100w');\
             var g1 = g.decode(); g.setAttribute('srcset', 'good.png 100w');\
             var g2 = g.decode(); __track('mutOld', g1); __track('mutNew', g2);\
             // (h) 同值重赋不算变异
             var h = mk(); h.setAttribute('src', 'ok.png');\
             var h1 = h.decode(); h.setAttribute('src', 'ok.png'); __track('sameValue', h1);",
        )
        .unwrap();
    // 微任务 checkpoint——fetch 链 + then 回调排水。
    sandbox.execute("0;").unwrap();
    sandbox
        .execute(
            "globalThis.__rKey = function () {\
               var ks = Object.keys(globalThis.__r).sort();\
               var out = [];\
               for (var i = 0; i < ks.length; i++) out.push(ks[i] + '=' + globalThis.__r[ks[i]]);\
               return out.join('|');\
             };",
        )
        .unwrap();
    let summary = sandbox.execute("globalThis.__rKey()").unwrap().value;
    assert_eq!(
        summary,
        "badChain=rejected:EncodingError|emptySrc=rejected:EncodingError|\
         mutNew=fulfilled:undefined|mutOld=rejected:EncodingError|\
         noSrc=rejected:EncodingError|notFound=rejected:EncodingError|\
         okChain=fulfilled:undefined|sameValue=fulfilled:undefined|\
         srcsetOnly=fulfilled:undefined|zeroChain=rejected:EncodingError"
    );

    // (i) multiple decodes：两次调用各返新 promise（无 settle 态——链臂各自 fetch
    // settle），均 resolve 且不同一。
    sandbox
        .execute(
            "var m = mk(); m.setAttribute('src', 'ok.png');\
             var m1 = m.decode(); var m2 = m.decode();\
             globalThis.__r2 = {};\
             Promise.all([m1, m2]).then(function () { globalThis.__r2.both = 'fulfilled'; }, function () { globalThis.__r2.both = 'rejected'; });\
             globalThis.__r2.distinct = String(m1 !== m2);",
        )
        .unwrap();
    sandbox.execute("0;").unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__r2.distinct + '/' + globalThis.__r2.both)")
            .unwrap()
            .value,
        "true/fulfilled"
    );

    // 钉②：detached 文档（createHTMLDocument）IMG 产物 decode 恒 EncodingError
    //（无加载管线 + 永非 fully active）；产物 ownerDocument 指向 detached 文档。
    sandbox
        .execute(
            "globalThis.__r3 = {};\
             var dd = document.implementation.createHTMLDocument('t');\
             var di = dd.createElement('img');\
             globalThis.__r3.ownerDoc = String(di.ownerDocument === dd);\
             di.src = 'ok.png';\
             Promise.resolve(di.decode()).then(function () { globalThis.__r3.dec = 'fulfilled'; }, function (e) { globalThis.__r3.dec = 'rejected:' + (e && e.name); });",
        )
        .unwrap();
    sandbox.execute("0;").unwrap();
    assert_eq!(
        sandbox
            .execute(
                "String(globalThis.__r3.ownerDoc + '/' + globalThis.__r3.dec)"
            )
            .unwrap()
            .value,
        "true/rejected:EncodingError"
    );
}

/// 钉③：恒异步 settle + 调用时捕获 + adopt 时序 + picture 选择 + SVG 原型——
/// (a) available-images 命中同 turn adopt-out → 旧 promise EncodingError（同步
/// 即决版漏判的 corpus adopted-active-into-inactive 形状）；(b) 对照：同预置不
/// adopt → resolve；(c) detached 产物 adopt 进主文档后 decode resolve（pinned
/// 动态臂活跃性判别）；(d) picture：source 优先于 img 自身 src（source URL
/// resolve）；(e) SVGImageElement.prototype.decode——href resolve / xlink:href
/// 回落 / corrupt EncodingError / 无 href EncodingError / 原型成员在档。
/// 断言翻转：恒异步 settle 退同步即决 → (a) 红；活跃性判别撤 → (a)(c) 错臂红；
/// picture source 选择撤 → (d) 红（自身空候选 EncodingError）；SVG 原型撤 →
/// (e) TypeError 红。
#[test]
fn img_decode_adopt_svg_picture_r5276() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s69b".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.createImageBitmap = function () { return Promise.resolve({ width: 4, height: 2 }); };\
             globalThis.fetch = function (url) {\
               var u = String(url);\
               if (u.indexOf('pic.png') >= 0 || u.indexOf('ok.png') >= 0 || u.indexOf('good.png') >= 0) return Promise.resolve({ ok: true, status: 200, blob: function () { return Promise.resolve({ z: 1 }); } });\
               if (u.indexOf('data:') === 0) return Promise.resolve({ ok: true, status: 200, blob: function () { return Promise.resolve({ z: 1 }); } });\
               return Promise.resolve({ ok: false, status: 404 });\
             };\
             globalThis.__r = {};\
             var __track = function (name, p) { return Promise.resolve(p).then(\
               function (v) { globalThis.__r[name] = 'fulfilled:' + String(v); },\
               function (e) { globalThis.__r[name] = 'rejected:' + (e && e.name); }); };",
        )
        .unwrap();

    sandbox
        .execute(
            "(function () {\
               // (a) available-images 命中 + 同 turn adopt-out → reject（异步决断见 adopt）
               globalThis._zwAvailableImages = globalThis._zwAvailableImages || {};\
               globalThis._zwAvailableImages['ok.png'] = 1;\
               var a = document.createElement('img'); a.setAttribute('src', 'ok.png');\
               var p1 = a.decode();\
               var dd = document.implementation.createHTMLDocument('t');\
               dd.body.appendChild(a);\
               __track('adoptOut', p1);\
               // (b) 对照：同预置不 adopt → resolve
               var b = document.createElement('img'); b.setAttribute('src', 'ok.png');\
               __track('stayActive', b.decode());\
               // (c) detached 产物 adopt 进主文档 → resolve（动态臂走核）
               var c = dd.createElement('img'); c.setAttribute('src', 'ok.png');\
               document.body.appendChild(c);\
               __track('adoptIn', c.decode());\
               // (c2) detached 产物 expando src（img.src= 无 setter 落 expando）adopt-in → resolve
               var c2 = dd.createElement('img'); c2.src = 'ok.png';\
               document.body.appendChild(c2);\
               __track('adoptInExpando', c2.decode());\
               // (d) picture：source 优先于 img 自身 src
               var pic = document.createElement('picture');\
               var src = document.createElement('source'); src.setAttribute('srcset', 'pic.png');\
               var d = document.createElement('img'); d.setAttribute('src', 'nope.png');\
               pic.appendChild(src); pic.appendChild(d);\
               document.body.appendChild(pic);\
               __track('pictureSource', d.decode());\
               // (d2) picture：data: URL source 不被逗号截断
               var pic2 = document.createElement('picture');\
               var src2 = document.createElement('source'); src2.setAttribute('srcset', 'data:image/png;base64,AAAA');\
               var d2 = document.createElement('img');\
               pic2.appendChild(src2); pic2.appendChild(d2);\
               document.body.appendChild(pic2);\
               __track('pictureDataUrl', d2.decode());\
               // (a2) available-images 命中 + 同 turn attach 进含 source picture → reject（快捷面变异复检）
               globalThis._zwAvailableImages['pic.png'] = 1;\
               var pic3 = document.createElement('picture');\
               var src3 = document.createElement('source'); src3.setAttribute('srcset', 'pic.png');\
               var e2 = document.createElement('img'); e2.setAttribute('src', 'ok.png');\
               var pE2 = e2.decode();\
               pic3.appendChild(src3); pic3.appendChild(e2);\
               document.body.appendChild(pic3);\
               __track('availMutated', pE2);\
               // (e) SVG 原型：href / xlink:href / corrupt / 无 href
               var mkSvg = function () { return document.createElementNS('http://www.w3.org/2000/svg', 'image'); };\
               globalThis.__svgProtoFn = String(typeof globalThis.SVGImageElement.prototype.decode);\
               var s1 = mkSvg(); s1.setAttribute('href', 'ok.png'); __track('svgHref', s1.decode());\
               var s2 = mkSvg(); s2.setAttribute('xlink:href', 'good.png'); __track('svgXlink', s2.decode());\
               var s3 = mkSvg(); s3.setAttribute('href', 'nope.png'); __track('svgBad', s3.decode());\
               var s4 = mkSvg(); __track('svgNone', s4.decode());\
             })();",
        )
        .unwrap();
    sandbox.execute("0;").unwrap();
    let summary = sandbox
        .execute(
            "globalThis.__rKey = function () {\
               var ks = Object.keys(globalThis.__r).sort();\
               var out = [];\
               for (var i = 0; i < ks.length; i++) out.push(ks[i] + '=' + globalThis.__r[ks[i]]);\
               return out.join('|');\
             };\
             globalThis.__rKey() + '#' + globalThis.__svgProtoFn",
        )
        .unwrap()
        .value;
    assert_eq!(
        summary,
        "adoptIn=fulfilled:undefined|adoptInExpando=fulfilled:undefined|\
         adoptOut=rejected:EncodingError|availMutated=rejected:EncodingError|\
         pictureDataUrl=fulfilled:undefined|pictureSource=fulfilled:undefined|\
         stayActive=fulfilled:undefined|svgBad=rejected:EncodingError|\
         svgHref=fulfilled:undefined|svgNone=rejected:EncodingError|\
         svgXlink=fulfilled:undefined#function"
    );
}
