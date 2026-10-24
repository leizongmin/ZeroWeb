// part85 — html-semantics M2 切片 68（R5275，2026-10-24）：img 静态加载管线收口。
//
// 根因：img 资源 settle 只接 `.src` IDL setter 通道（R56h fetch→createImageBitmap
// →_resourceStates），parser 静态 `<img src>` 无 fetch/settle——naturalWidth 恒 0、
// load/error 恒不派；broken 终态缺失（error 后 complete 恒 false）；运行期事件在
// runner 页面脚本阶段即时派发（spec：DCL 不等图片、图片延迟 window load）。
// 三刀：① `__zwFireStaticImgSettle` 扫描（part04，window load 派发钩子调用——
// lazy/空 src 跳过、host 快照命中 → loaded+load / 未命中 → broken+error）；
// ② R56h 生命周期闸门（readyState 'loading' 时事件入队、settle 时 drain + broken
// 补写）+ available-images 同步 settle + restart 语义；③ IMG width/height 属性
// 缺失回落自然尺寸 + currentSrc 绝对 URL。
// https://html.spec.whatwg.org/multipage/images.html#updating-the-image-data
// https://html.spec.whatwg.org/multipage/images.html#the-list-of-available-images
// https://html.spec.whatwg.org/multipage/images.html#dom-img-currentsrc

/// 钉①：静态 img settle 扫描——(a) host 快照命中 → loaded：naturalWidth/Height
/// 32/16、width/height 属性缺失回落自然尺寸、complete true、load 事件派发；
/// (b) loading=lazy 跳过（快照有值仍不 settle——naturalWidth 0、无事件）；
/// (c) 无/空 src 无事件且 complete true（无当前请求）；(d) 快照未命中 → broken：
/// error 事件、complete true、naturalWidth 0。
/// 断言翻转：part06 load 钩子的 settle 调用被撤 / 扫描体被撤（全部子断言红）；
/// lazy 门被撤（(b) naturalWidth 变 5、load 旗标变 true）。
#[test]
fn img_static_sweep_settle_r5275() {
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
        "<html><body><img id='a' src='pix.png'><img id='l' src='x.png' loading='lazy'>\
         <img id='n'><img id='e' src=''><img id='b' src='missing.png'></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s68a".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 契约沙箱无 image_source_fetcher（webview __zw_get_image_size 未注册）——
    // 显式注入 host 快照 stub：pix.png=32×16、x.png=5×5（lazy 门断言需快照有值）、
    // missing.png 未命中（broken 面）。
    sandbox
        .execute(
            "globalThis.__zw_get_image_size = function (src) {\
               if (src === 'pix.png') return '32,16';\
               if (src === 'x.png') return '5,5';\
               return '';\
             };\
             var _a = document.getElementById('a'), _l = document.getElementById('l'),\
                 _b = document.getElementById('b');\
             globalThis.__flagA = 0; globalThis.__flagL = 0; globalThis.__flagB = 0;\
             _a.addEventListener('load', function () { globalThis.__flagA++; });\
             _l.addEventListener('load', function () { globalThis.__flagL++; });\
             _b.addEventListener('error', function () { globalThis.__flagB++; });",
        )
        .unwrap();
    // window load 派发 → part06 钩子 → __zwFireStaticImgSettle。
    sandbox
        .execute("globalThis.dispatchEvent(new Event('load'));")
        .unwrap();
    sandbox
        .execute(
            "globalThis.__r5275a1 = String(_a.naturalWidth) + '/' + String(_a.naturalHeight);\
             globalThis.__r5275a2 = String(_a.width) + '/' + String(_a.height);\
             globalThis.__r5275a3 = String(_a.complete) + ':' + String(globalThis.__flagA);\
             globalThis.__r5275b = String(_l.naturalWidth) + '/' + String(_l.complete) + '/' + String(globalThis.__flagL);\
             globalThis.__r5275c = String(document.getElementById('n').complete) + '/' + String(document.getElementById('e').complete);\
             globalThis.__r5275d = String(_b.complete) + '/' + String(_b.naturalWidth) + '/' + String(globalThis.__flagB);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5275a1").unwrap().value,
        "32/16",
        "(a1) 快照命中 → naturalWidth/Height 32/16"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5275a2").unwrap().value,
        "32/16",
        "(a2) width/height 属性缺失回落自然尺寸（rendered ≈ intrinsic headless 近似）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5275a3").unwrap().value,
        "true:1",
        "(a3) complete true + load 事件恰派一次"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5275b").unwrap().value,
        "0/false/0",
        "(b) lazy 跳过：快照有值仍不 settle（naturalWidth 0、无事件、complete false）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5275c").unwrap().value,
        "true/true",
        "(c) 无/空 src：无当前请求 → complete true（getter 面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5275d").unwrap().value,
        "true/0/1",
        "(d) 快照未命中 → broken：complete true、naturalWidth 0、error 事件派发"
    );
}

/// 钉②：R56h 生命周期闸门 + available-images 同步 settle + currentSrc 绝对 URL。
/// readyState 'loading' 时 `img.src` 触发的 error 事件入队不即派（complete 保持
/// false）；settle drain 后 error 派发且 broken 补写（complete true）。
/// 'complete' 后 available-images 命中 → 状态同步 settle（naturalWidth/complete
/// 同步可读）、load 事件 _defer 后续任务派发（setter 同步任务内注册的 onload 不
/// 丢——available-images-onload i2 链面）、currentSrc 绝对 URL；同 URL 重复 set
/// 幂等（load 不重派）。
/// 断言翻转：闸门被撤（error 即派 → (a) 翻红）；broken 补写被撤（(b) complete
/// false）；available-images 同步面被撤（(c)(d) 走 fetch 链异步，同步读翻红）；
/// _defer 事件面被撤（(c2) 计数 0——同步即派则 1，但探针面 onload 后注册丢事件
/// → available-images-onload 文件级 Timeout）；同 URL 幂等被撤（(e) 计数 2）。
#[test]
fn img_runtime_deferred_settle_r5275() {
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s68b".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // (a)(b) 闸门 + broken 补写：readyState 'loading'（runner 页面脚本阶段语义）。
    sandbox
        .execute(
            "globalThis.__zwReadyState = 'loading';\
             var img = document.createElement('img');\
             document.body.appendChild(img);\
             globalThis.__flagE = 0;\
             img.addEventListener('error', function () { globalThis.__flagE++; });\
             img.src = 'no-such-host-file.png';\
             globalThis.__r5275a = String(img.complete) + '/' + String(globalThis.__flagE);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5275a").unwrap().value,
        "false/0",
        "(a) loading 阶段 error 入队不即派、complete 保持 false（闸门）"
    );
    sandbox
        .execute("globalThis.dispatchEvent(new Event('load'));")
        .unwrap();
    sandbox
        .execute(
            "globalThis.__r5275b = String(img.complete) + '/' + String(globalThis.__flagE) + '/' + String(img.naturalWidth);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5275b").unwrap().value,
        "true/1/0",
        "(b) settle drain：error 派发 + broken 补写（complete true、naturalWidth 0）"
    );
    // (c)(d)(e) available-images 同步 settle（**状态同步可读**、load 事件 _defer
    // 后续任务派发——spec image loading task 异步面）+ currentSrc 绝对 URL + 幂等。
    sandbox
        .execute(
            "globalThis.__zwReadyState = 'complete';\
             globalThis._zwAvailableImages = { 'ok.png': { width: 7, height: 9 } };\
             globalThis.__flagO = 0;\
             img.addEventListener('load', function () { globalThis.__flagO++; });\
             img.src = 'ok.png';\
             globalThis.__r5275c = String(img.naturalWidth) + '/' + String(img.complete);\
             globalThis.__r5275d = String(img.currentSrc);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5275c").unwrap().value,
        "7/true",
        "(c) available-images 命中 → 状态同步 settle：naturalWidth 立即可读 + complete true（事件未即派）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5275d").unwrap().value,
        "https://zero.test/ok.png",
        "(d) currentSrc 绝对 URL（页锚解析）"
    );
    sandbox.execute("0;").unwrap(); // 微任务 checkpoint——_defer 派 load
    sandbox
        .execute("globalThis.__r5275c2 = String(globalThis.__flagO);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5275c2").unwrap().value,
        "1",
        "(c2) load 事件后续任务派发（setter 同步任务内不即派——onload 后注册不丢）"
    );
    sandbox.execute("img.src = 'ok.png';").unwrap();
    sandbox
        .execute("globalThis.__r5275e = String(globalThis.__flagO);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5275e").unwrap().value,
        "1",
        "(e) 同 URL 重复 set 幂等——load 不重派（stable state 合并）"
    );
}
