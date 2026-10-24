// part88 — html-semantics M2 切片 70（s70，2026-10-25）：F2 source-set 族收口。
//
// 根因：srcset/sizes/picture-source 选择面仅有 R4162/R2419「首 URL」近似——
// parse-a-srcset 49F、select-an-image-source 13F、update-the-source-set 84F、
// relevant-mutations 46F、current-pixel-density 21F（密度校正缺失：naturalWidth
// 未按选中密度折算）。s70 四刀：① 共享选择核（part03 `__zwParseSrcsetAttr5277`/
// `__zwParseSizesAttr5277`/`__zwCreateSourceSet5277`/`__zwUpdateSourceSet5277`/
// `__zwSelectImageSource5277`——spec §4.8.4.3.7-12 逐字）；② 全状态写入面切选中
// URL（sweep / R56h `.src` IDL 链 / revalidate——密度入 resourceState）；
// ③ relevant mutations 触发面（attr set/remove + picture 树插入/移除——内容状态
// 枚举比对 crossorigin/referrerpolicy、同 URL 重派 load）；④ naturalWidth 密度
// 校正 + broken 无 width 属性 → 0（broken intrinsic）。附：detached 更新延一步
// + lazy 视口门 + scrollIntoView 触发近似（spec update-the-image-data lazy/
// detached 面——src-then-lazy-load、image-loading-lazy-relevant-mutations、
// image-loading-lazy-crossorigin-change 断言面）。
// https://html.spec.whatwg.org/multipage/images.html#select-an-image-source
// https://html.spec.whatwg.org/multipage/images.html#relevant-mutations

/// 钉①：srcset/sizes 解析核真值点（parse-a-srcset-attribute / parse-a-sizes-
/// attribute 对位）——(a) 基础密度/宽度描述符；(b) 尾逗号裸 URL 剥除；(c) 空组
/// 跳过；(d) `.5x`/`1e0x`/`-0x` 宽松浮点合法；(e) `+1x`/`1X`/`Infinityx`/`0w`
/// 非法（候选丢弃、后续继续）；(f) `1w 1h` w+h 对合法、`1h` 无 width 丢弃；
/// (g) sizes 逐项 last-value 语义 + media 条件 + auto 忽略 + 全不命中回落视口宽；
/// (h) source-size-value 单位表（裸数字仅 0）；(i) MIME 支持判定（空放行、
/// 白名单外拒）。
/// 断言翻转：解析核撤（全部子断言红）。
#[test]
fn srcset_sizes_parser_r5277() {
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s70p".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.innerWidth = 800; globalThis.innerHeight = 600;\
             var P = function (s) { return JSON.stringify(globalThis.__zwParseSrcsetAttr5277(s)); };\
             globalThis.__r = [];\
             // (a) 基础描述符
             __r.push(P('a.png 1x'));\
             __r.push(P('a.png 100w, b.png 200w'));\
             // (b)(c) 尾逗号剥除 + 空组
             __r.push(P('a.png,'));\
             __r.push(P('a.png,, b.png 2x'));\
             // (d) 宽松浮点
             __r.push(P('a.png .5x'));\
             __r.push(P('a.png 1e0x'));\
             __r.push(P('a.png -0x'));\
             // (e) 非法描述符——候选丢弃
             __r.push(P('a.png +1x'));\
             __r.push(P('a.png 1X'));\
             __r.push(P('a.png Infinityx'));\
             __r.push(P('a.png 0w'));\
             // (f) w+h 对 / h 无 width
             __r.push(P('a.png 1w 1h'));\
             __r.push(P('a.png 1h'));\
             // sizes
             var S = function (s) { return String(globalThis.__zwParseSizesAttr5277(s)); };\
             __r.push(S('50vw'));\
             __r.push(S('(min-width: 99999px) 300px, 60vw'));\
             __r.push(S('(min-width: 0px) 300px, 60vw'));\
             __r.push(S('auto, 50vw'));\
             __r.push(S(''));\
             __r.push(String(globalThis.__zwSourceSizeValue5277('100px', 800, 600)));\
             __r.push(String(globalThis.__zwSourceSizeValue5277('0', 800, 600)));\
             __r.push(String(globalThis.__zwSourceSizeValue5277('50', 800, 600)));\
             __r.push(String(globalThis.__zwSourceSupportedType5277('image/png')) + '/' + String(globalThis.__zwSourceSupportedType5277('')) + '/' + String(globalThis.__zwSourceSupportedType5277('text/plain')) + '/' + String(globalThis.__zwSourceSupportedType5277('image/unknown')));",
        )
        .unwrap();
    let get = |sandbox: &mut V8Sandbox, i: usize| -> String {
        sandbox
            .execute(&format!("globalThis.__r[{}]", i))
            .unwrap()
            .value
    };
    // (a)
    assert_eq!(get(&mut sandbox, 0), r#"[{"url":"a.png","density":1}]"#, "(a1) 密度描述符");
    assert_eq!(
        get(&mut sandbox, 1),
        r#"[{"url":"a.png","width":100},{"url":"b.png","width":200}]"#,
        "(a2) 宽度描述符（density 留待归一）"
    );
    // (b)(c)
    assert_eq!(get(&mut sandbox, 2), r#"[{"url":"a.png"}]"#, "(b) 尾逗号剥除为裸 URL 候选");
    assert_eq!(
        get(&mut sandbox, 3),
        r#"[{"url":"a.png"},{"url":"b.png","density":2}]"#,
        "(c) 空组跳过、后续候选继续"
    );
    // (d)
    assert_eq!(get(&mut sandbox, 4), r#"[{"url":"a.png","density":0.5}]"#, "(d1) .5x 合法");
    assert_eq!(get(&mut sandbox, 5), r#"[{"url":"a.png","density":1}]"#, "(d2) 1e0x 合法");
    assert_eq!(get(&mut sandbox, 6), r#"[{"url":"a.png","density":0}]"#, "(d3) -0x 合法（<0 才 error）");
    // (e)
    for i in 7..11 {
        let msg = format!("(e{}) 非法描述符候选丢弃", i - 6);
        assert_eq!(get(&mut sandbox, i), "[]", "{}", msg.as_str());
    }
    // (f)
    assert_eq!(get(&mut sandbox, 11), r#"[{"url":"a.png","width":1}]"#, "(f1) w+h 对合法（h 兼容描述符）");
    assert_eq!(get(&mut sandbox, 12), "[]", "(f2) h 无 width → 候选丢弃");
    // (g) sizes
    assert_eq!(get(&mut sandbox, 13), "400", "(g1) 50vw = 400（视口 800）");
    assert_eq!(get(&mut sandbox, 14), "480", "(g2) media 不命中走次项 60vw");
    assert_eq!(get(&mut sandbox, 15), "300", "(g3) media 命中取 300px");
    assert_eq!(get(&mut sandbox, 16), "400", "(g4) auto 项忽略");
    assert_eq!(get(&mut sandbox, 17), "800", "(g5) 空值回落视口宽");
    // (h)(i)
    assert_eq!(get(&mut sandbox, 18), "100", "(h1) px 直读");
    assert_eq!(get(&mut sandbox, 19), "0", "(h2) 裸数字仅 0 合法");
    assert_eq!(get(&mut sandbox, 20), "null", "(h3) 裸非零数字非法");
    assert_eq!(get(&mut sandbox, 21), "true/true/false/false", "(i) MIME 支持判定（空放行/白名单外拒）");
}

/// 钉②：select-an-image-source 选择面——(a) picture source media 不匹配跳过
/// → img 自身；(b) source media 命中胜出（img src 不进集）；(c) type 不支持跳过；
/// (d) width/sizes 密度归一（100w/400w @50vw=400px → 0.25/1.0）；(e) 无候选 null；
/// (f) DPR 择面（≤DPR 最大密度）；(g) 同密度去重保首。
/// 断言翻转：选择核撤（全红）。
#[test]
fn img_select_image_source_r5277() {
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
        "<html><body>\
         <picture id='p1'><source srcset='g.png 2x' media='(min-width: 99999px)'><img id='i1' src='a.png' srcset='a.png 1x, b.png 2x'></picture>\
         <picture id='p2'><source srcset='g.png 2x' media='(min-width: 0px)'><img id='i2' src='a.png'></picture>\
         <picture id='p3'><source srcset='t.png' type='image/unknown'><img id='i3' src='a.png'></picture>\
         <img id='i4' srcset='w100.png 100w, w400.png 400w' sizes='50vw'>\
         <img id='i5'>\
         <picture id='p4'><source srcset='d1.png 1x, d2.png 3x'><img id='i6' src='a.png'></picture>\
         <img id='i7' srcset='d1.png 1x, d1.png 1x, d3.png 2x'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s70s".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.innerWidth = 800; globalThis.innerHeight = 600; globalThis.devicePixelRatio = 1;\
             var SEL = function (id) {\
               var el = document.getElementById(id);\
               return JSON.stringify(globalThis.__zwSelectImageSource5277(el.__zwSelector, el.__zwHandle));\
             };\
             globalThis.__r = [SEL('i1'), SEL('i2'), SEL('i3'), SEL('i4'), SEL('i5'), SEL('i6'), SEL('i7')];",
        )
        .unwrap();
    let get = |sandbox: &mut V8Sandbox, i: usize| -> String {
        sandbox
            .execute(&format!("globalThis.__r[{}]", i))
            .unwrap()
            .value
    };
    assert_eq!(
        get(&mut sandbox, 0),
        r#"{"url":"https://zero.test/a.png","density":1}"#,
        "(a) source media 不匹配 → img 自身集（src 为 default 不入——已有 1x）"
    );
    assert_eq!(
        get(&mut sandbox, 1),
        r#"{"url":"https://zero.test/g.png","density":2}"#,
        "(b) source media 命中胜出（无 ≤DPR 候选取最小）"
    );
    assert_eq!(
        get(&mut sandbox, 2),
        r#"{"url":"https://zero.test/a.png","density":1}"#,
        "(c) type 不支持 → source 跳过"
    );
    assert_eq!(
        get(&mut sandbox, 3),
        r#"{"url":"https://zero.test/w400.png","width":400,"density":1}"#,
        "(d) width/sizes 归一（400w/400px = 1x 胜出；归一保留 width 字段）"
    );
    assert_eq!(get(&mut sandbox, 4), "null", "(e) 无候选 → null selection");
    assert_eq!(
        get(&mut sandbox, 5),
        r#"{"url":"https://zero.test/d1.png","density":1}"#,
        "(f1) DPR 1 取 ≤DPR 最大密度"
    );
    assert_eq!(
        get(&mut sandbox, 6),
        r#"{"url":"https://zero.test/d1.png","density":1}"#,
        "(g) 同密度去重保首"
    );
    // (f2) DPR 3 → 3x 胜出
    sandbox
        .execute(
            "globalThis.devicePixelRatio = 3;\
             var el = document.getElementById('i6');\
             globalThis.__rf2 = JSON.stringify(globalThis.__zwSelectImageSource5277(el.__zwSelector, el.__zwHandle));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__rf2").unwrap().value,
        r#"{"url":"https://zero.test/d2.png","density":3}"#,
        "(f2) DPR 3 → 3x 胜出"
    );
}

/// 钉③：relevant mutations 运行触发面——(1) setAttribute('srcset') 重验 settle
/// （available-images 同步 + load 后续任务派）；(2) sizes 变更同 URL **重派** load
///（spec relevant mutations「sizes attribute changed」——R5275 幂等按 spec 收窄）；
/// (3) DPR 翻转后 srcset 重选 → 密度校正 naturalWidth（200/2x = 100）；(4)
/// crossorigin 状态变迁触发、同状态不触发；(5) referrerpolicy 状态变迁/无效值归
/// 缺省态比对；(6) removeAttribute('srcset') → null selection broken、无 error
///（无 src/srcset/picture 关联）；(7) img 入 picture 重验、picture 整体插入不触
/// 发；(8) source 插入 img 前重验选中 source；(9) source 插入 img 后不触发。
/// 断言翻转：触发面撤（计数面全红）；同状态门撤（(4b) 计数多 1）。
#[test]
fn img_relevant_mutations_r5277() {
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s70m".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 注：load 事件 _defer 后续任务派发——每次变更与旗标读取之间以独立 execute("0;")
    // 微任务 checkpoint 分隔（同一 execute 内 `0;` 语句不构成 drain 点）。
    let ckpt = |sandbox: &mut V8Sandbox| {
        sandbox.execute("0;").unwrap();
    };
    sandbox
        .execute(
            "globalThis.innerWidth = 800; globalThis.innerHeight = 600;\
             globalThis.__zwReadyState = 'complete';\
             globalThis.devicePixelRatio = 1;\
             globalThis._zwAvailableImages = { 'https://zero.test/a.png': { width: 100, height: 50 },\
               'https://zero.test/b.png': { width: 200, height: 100 }, 'https://zero.test/s.png': { width: 40, height: 30 } };\
             var img = document.createElement('img');\
             document.body.appendChild(img);\
             globalThis.__img = img;\
             globalThis.__flagO = 0; globalThis.__flagE = 0;\
             img.addEventListener('load', function () { globalThis.__flagO++; });\
             img.addEventListener('error', function () { globalThis.__flagE++; });\
             img.setAttribute('srcset', 'a.png 1x, b.png 2x');\
             globalThis.__r1 = String(img.naturalWidth) + '/' + String(img.complete);",
        )
        .unwrap();
    let v = |sandbox: &mut V8Sandbox, name: &str| -> String {
        sandbox
            .execute(&format!("globalThis.{}", name))
            .unwrap()
            .value
    };
    assert_eq!(v(&mut sandbox, "__r1"), "100/true", "(1) srcset 重验 → avail 同步 settle");
    ckpt(&mut sandbox);
    // (2) sizes 变更同 URL 重派
    sandbox.execute("globalThis.__img.setAttribute('sizes', '50vw');").unwrap();
    ckpt(&mut sandbox);
    assert_eq!(v(&mut sandbox, "__flagO"), "2", "(2) sizes 变更同 URL 重派 load（spec relevant mutations）");
    // (3) DPR 翻转重选 + 密度校正
    sandbox
        .execute("globalThis.devicePixelRatio = 2; globalThis.__img.setAttribute('srcset', 'a.png 1x, b.png 2x');\
                  globalThis.__r3 = String(globalThis.__img.naturalWidth) + '/' + String(globalThis.__img.naturalHeight);")
        .unwrap();
    assert_eq!(
        v(&mut sandbox, "__r3"),
        "100/50",
        "(3) DPR 2 重选 b.png → 密度校正 naturalWidth/Height = 200/2、100/2"
    );
    ckpt(&mut sandbox);
    // (4) crossorigin 状态门
    sandbox.execute("globalThis.__img.crossOrigin = 'anonymous';").unwrap();
    ckpt(&mut sandbox);
    // 旗标链：(1)=1、(2) 同 URL 重派=2、(3) URL 变更 restart=3、(4a) 状态变迁同 URL 重派=4。
    assert_eq!(v(&mut sandbox, "__flagO"), "4", "(4a) crossorigin missing→anonymous 状态变迁触发（同 URL 重派）");
    sandbox.execute("globalThis.__img.crossOrigin = 'anonymous';").unwrap();
    ckpt(&mut sandbox);
    assert_eq!(v(&mut sandbox, "__flagO"), "4", "(4b) 同状态重写不触发");
    // (5) referrerpolicy 状态链（缺省→有效→无效→无效——中段两次变迁）
    sandbox
        .execute("globalThis.__img.referrerPolicy = 'no-referrer';")
        .unwrap();
    ckpt(&mut sandbox);
    sandbox.execute("globalThis.__img.referrerPolicy = 'x-bogus';").unwrap();
    ckpt(&mut sandbox);
    sandbox.execute("globalThis.__img.referrerPolicy = 'also-bogus';").unwrap();
    ckpt(&mut sandbox);
    assert_eq!(
        v(&mut sandbox, "__flagO") + "/" + &v(&mut sandbox, "__flagE"),
        "6/0",
        "(5) referrerpolicy 缺省→有效→无效→无效：中段两次变迁各重派（4→6）、无效→无效不动"
    );
    // (6) srcset 移除 → null selection broken、无 error（无 src/srcset/picture 关联）
    sandbox
        .execute("globalThis.__img.removeAttribute('srcset');\
                  globalThis.__r6 = String(globalThis.__img.naturalWidth) + '/' + String(globalThis.__flagE);")
        .unwrap();
    assert_eq!(v(&mut sandbox, "__r6"), "0/0", "(6) srcset 移除 → broken（naturalWidth 0）、无 error（无关联）");
    ckpt(&mut sandbox);
    assert_eq!(
        v(&mut sandbox, "__flagO") + "/" + &v(&mut sandbox, "__flagE"),
        "6/0",
        "(6b) 计数收口"
    );
    // (7)-(9) picture 树触发面——旗标挂在 pimg 自身（flagO 是 img 的监听器，pimg
    // 的 load 不入 flagO）。detached setAttribute('src') 也触发 load（node
    // document fully active 即可，元素自身 detached 不拦——Image() 预载面）。
    sandbox
        .execute(
            "var pic = document.createElement('picture');\
             var pimg = document.createElement('img');\
             globalThis.__flagP = 0;\
             pimg.addEventListener('load', function () { globalThis.__flagP++; });\
             pimg.setAttribute('src', 'a.png');\
             document.body.appendChild(pic);\
             globalThis.__pic = pic; globalThis.__pimg = pimg;\
             pic.appendChild(pimg);",
        )
        .unwrap();
    ckpt(&mut sandbox);
    assert_eq!(v(&mut sandbox, "__flagP"), "2", "(7) detached src-set load + 入 picture 重验同 URL 重派 load");
    sandbox
        .execute(
            "var src = document.createElement('source');\
             src.setAttribute('srcset', 's.png');\
             globalThis.__pic.insertBefore(src, globalThis.__pimg);\
             globalThis.__r8 = String(globalThis.__pimg.naturalWidth) + '/' + String(globalThis.__pimg.currentSrc);",
        )
        .unwrap();
    assert_eq!(
        v(&mut sandbox, "__r8"),
        "40/https://zero.test/s.png",
        "(8) source 插入 img 前 → 重选 s.png（密度 1：naturalWidth 40、currentSrc 绝对 URL）"
    );
    ckpt(&mut sandbox);
    sandbox
        .execute(
            "var src2 = document.createElement('source');\
             src2.setAttribute('srcset', 'a.png 1x');\
             globalThis.__pic.appendChild(src2);\
             globalThis.__r9 = String(globalThis.__pimg.currentSrc) + '/' + String(globalThis.__pimg.naturalWidth) + '/' + String(globalThis.__flagP);",
        )
        .unwrap();
    assert_eq!(
        v(&mut sandbox, "__r9"),
        "https://zero.test/s.png/40/3",
        "(9) source 插入 img 后不触发（img 只看前序 source；flagP 停 (8) 的 3）"
    );
}

/// 钉③b：lazy 视口门三面（spec update-the-image-data）。(a) detached 更新延
/// 一步——`.src=` 先于 `loading='lazy'` 的任务内时序在延后重读 → 不 fetch、无
/// 事件（src-then-lazy-load 面）；(b) attached lazy img 的 relevant mutation
/// 不触发请求（naturalWidth 0、无事件），scrollIntoView 置视口旗标 → 重验
/// fetch → load（image-loading-lazy-relevant-mutations /
/// image-loading-lazy-crossorigin-change 两对立面）；(c) available-images
/// 检查先于 lazy 决定——lazy img `.src=` 命中可用图照常同步 settle + load
///（image-loading-lazy-available/use-list 面）。
/// 断言翻转：detached 延步撤（(a) 翻红）；lazy 门撤（(b1) 翻红）；视口旗标
/// 撤（(b2) 翻红）；avail 先于 lazy 撤（(c) 翻红）。
#[test]
fn img_lazy_viewport_gate_r5277() {
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s70l".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
        .execute(
            "globalThis.innerWidth = 800; globalThis.innerHeight = 600;\
             globalThis.__zwReadyState = 'complete';\
             globalThis.devicePixelRatio = 1;\
             globalThis._zwAvailableImages = { 'https://zero.test/a.png': { width: 100, height: 50 } };\
             globalThis.fetch = function () {\
               return Promise.resolve({ ok: true, blob: function () { return Promise.resolve({ size: 4, type: 'image/png' }); } });\
             };\
             globalThis.createImageBitmap = function () {\
               return Promise.resolve({ width: 100, height: 50, close: function () {} });\
             };",
        )
        .unwrap();
    let v = |sandbox: &mut V8Sandbox, name: &str| -> String {
        sandbox
            .execute(&format!("globalThis.{}", name))
            .unwrap()
            .value
    };
    // (a) detached `.src=`（任务内 loading 尚未写）→ 延一步 → loading=lazy → 不 fetch。
    sandbox
        .execute(
            "var img = document.createElement('img');\
             globalThis.__flagA = 0;\
             img.addEventListener('load', function () { globalThis.__flagA++; });\
             img.addEventListener('error', function () { globalThis.__flagA++; });\
             img.src = 'missing.png?a';\
             img.loading = 'lazy';\
             document.body.appendChild(img);",
        )
        .unwrap();
    sandbox.execute("0;").unwrap();
    sandbox.execute("0;").unwrap();
    assert_eq!(v(&mut sandbox, "__flagA"), "0", "(a) detached 时序：loading=lazy 在延后重读 → 不 fetch 无事件");
    // (b1) attached lazy img relevant mutation → 不 fetch；(b2) scrollIntoView →
    // fetch。选中 URL 用 v.png（不在 avail 表——a.png 命中 avail 臂会绕过
    // fetch 臂的 lazy 门，那是 (c) 的语义，不是 (b) 的）。
    sandbox
        .execute(
            "var img2 = document.createElement('img');\
             img2.loading = 'lazy';\
             document.body.appendChild(img2);\
             globalThis.__img2 = img2;\
             globalThis.__flagB = 0;\
             img2.addEventListener('load', function () { globalThis.__flagB++; });",
        )
        .unwrap();
    sandbox.execute("globalThis.__img2.srcset = 'v.png 1x';").unwrap();
    sandbox.execute("0;").unwrap();
    assert_eq!(
        v(&mut sandbox, "__flagB") + "/" + &v(&mut sandbox, "__img2.naturalWidth"),
        "0/0",
        "(b1) deferred lazy img mutation 不触发请求"
    );
    sandbox.execute("globalThis.__img2.scrollIntoView();").unwrap();
    sandbox.execute("0;").unwrap();
    assert_eq!(
        v(&mut sandbox, "__flagB") + "/" + &v(&mut sandbox, "__img2.naturalWidth"),
        "1/100",
        "(b2) scrollIntoView 置视口旗标 → 重验 fetch → settle + load"
    );
    // (c) available-images 先于 lazy 决定：lazy img `.src=` 命中可用图照常 settle。
    sandbox
        .execute(
            "var img3 = document.createElement('img');\
             img3.loading = 'lazy';\
             document.body.appendChild(img3);\
             globalThis.__flagC = 0;\
             img3.addEventListener('load', function () { globalThis.__flagC++; });\
             img3.src = 'a.png';\
             globalThis.__rC = String(img3.complete);",
        )
        .unwrap();
    assert_eq!(v(&mut sandbox, "__rC"), "true", "(c1) avail 命中同步 settle（先于 lazy 门）");
    sandbox.execute("0;").unwrap();
    assert_eq!(v(&mut sandbox, "__flagC"), "1", "(c2) load 派发");
}

/// 钉④：静态 sweep 选中源 settle——(a) picture source 密度 2 选中（DPR 1 无
/// ≤DPR 候选取最小）→ naturalWidth/Height 密度校正 80/2=40、40/2=20、
/// available-images 写入选中 URL；(b) 独立 srcset img（无 src）按 srcset settle
///（旧 sweep 无 src 跳过——srcset-only 面全 broken）；(c) broken img 无 width/
/// height 属性 → width/height IDL 0（spec broken density-corrected intrinsic
/// dimension；naturalWidth-naturalHeight-width-height 'non existent image
/// (when not rendered)' width 期望 0 实证）、naturalWidth 0。
/// 断言翻转：sweep 选择核切换撤（(a) 密度不校正）；broken 臂撤（(c) 翻自然
/// 尺寸臂）。
#[test]
fn img_sweep_selection_density_r5277() {
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
        "<html><body>\
         <picture><source srcset='g.png 2x'><img id='p' src='a.png' srcset='a.png 1x, b.png 2x'></picture>\
         <img id='s' srcset='a.png 1x, b.png 2x'>\
         <img id='bk' src='nope.png'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s70w".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.devicePixelRatio = 1;\
             globalThis.__zw_get_image_size = function (src) {\
               if (src === 'https://zero.test/a.png') return '100,50';\
               if (src === 'https://zero.test/b.png') return '200,100';\
               if (src === 'https://zero.test/g.png') return '80,40';\
               return '';\
             };\
             var _p = document.getElementById('p'), _s = document.getElementById('s'), _bk = document.getElementById('bk');\
             globalThis.__flagP = 0; globalThis.__flagBK = 0;\
             _p.addEventListener('load', function () { globalThis.__flagP++; });\
             _bk.addEventListener('error', function () { globalThis.__flagBK++; });\
             globalThis.dispatchEvent(new Event('load'));\
             globalThis.__r1 = String(_p.naturalWidth) + '/' + String(_p.naturalHeight);\
             globalThis.__r2 = String(_p.currentSrc);\
             globalThis.__r3 = JSON.stringify(globalThis._zwAvailableImages['https://zero.test/g.png']);\
             globalThis.__r4 = String(_s.naturalWidth) + '/' + String(_s.naturalHeight) + '/' + String(globalThis.__flagP);\
             globalThis.__r5 = String(_bk.width) + '/' + String(_bk.height) + '/' + String(_bk.naturalWidth) + '/' + String(_bk.complete) + '/' + String(globalThis.__flagBK);",
        )
        .unwrap();
    let v = |sandbox: &mut V8Sandbox, name: &str| -> String {
        sandbox
            .execute(&format!("globalThis.{}", name))
            .unwrap()
            .value
    };
    assert_eq!(
        v(&mut sandbox, "__r1"),
        "40/20",
        "(a) picture source 2x 选中 → 密度校正 naturalWidth/Height = 80/2、40/2"
    );
    assert_eq!(v(&mut sandbox, "__r2"), "https://zero.test/g.png", "(a2) currentSrc = 选中 URL 绝对化");
    assert_eq!(
        v(&mut sandbox, "__r3"),
        r#"{"width":80,"height":40}"#,
        "(a3) available-images 写入选中 URL（80,40 原始尺寸）"
    );
    assert_eq!(
        v(&mut sandbox, "__r4"),
        "100/50/1",
        "(b) srcset-only img 按 srcset settle（密度 1 无校正）+ load 派发"
    );
    assert_eq!(
        v(&mut sandbox, "__r5"),
        "0/0/0/true/1",
        "(c) broken 无 width/height 属性 → IDL 0（broken intrinsic 0 非 UA icon 300/150）、naturalWidth 0、error 派发"
    );
}
