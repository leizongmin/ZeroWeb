// js_dom_bridge 测试切片 14（R3074+）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-13 共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。
// 按单文件 ≤2000 行拆分，本切片承载 element-method / Web-API 后续测试。

#[test]
fn test_element_check_visibility_r3074() {
    // R3074：Element.checkVisibility(options)——「being rendered」+ 可选 opacity/visibility 检查。
    // ad viewability / lazy-load 库用。经 host __zw_get_computed_style（display/opacity/visibility）+ 祖先链。
    // https://drafts.csswg.org/cssom-view-1/#dom-element-checkvisibility
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <div id='vis'>visible</div>\
         <div id='dn' style='display:none'>dn</div>\
         <div id='anc' style='display:none'><span id='childDn'>child</span></div>\
         <div id='op0' style='opacity:0'>op0</div>\
         <div id='opHalf' style='opacity:0.5'>opHalf</div>\
         <div id='visH' style='visibility:hidden'>visH</div>\
         <div id='visAnc' style='visibility:hidden'><span id='visChild' style='visibility:visible'>child</span></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① 默认 rendered 元素 → true。
    sandbox
        .execute("globalThis.__vis = String(document.getElementById('vis').checkVisibility());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__vis").unwrap().value,
        "true",
        "默认 rendered 元素 checkVisibility() → true"
    );

    // ② display:none 元素 → false（默认，无需 option）。
    sandbox
        .execute("globalThis.__dn = String(document.getElementById('dn').checkVisibility());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dn").unwrap().value,
        "false",
        "display:none 元素 checkVisibility() → false（not rendered）"
    );

    // ③ 祖先 display:none → 子元素 false（祖先链遍历）。
    sandbox
        .execute("globalThis.__childDn = String(document.getElementById('childDn').checkVisibility());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__childDn").unwrap().value,
        "false",
        "祖先 display:none → 子元素 checkVisibility() → false（祖先链）"
    );

    // ④ opacity:0 → 默认 true（opacity 不属 rendered 判定），opacityProperty:true → false。
    sandbox
        .execute(
            "globalThis.__op0Def = String(document.getElementById('op0').checkVisibility());\
             globalThis.__op0Opt = String(document.getElementById('op0').checkVisibility({opacityProperty:true}));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__op0Def").unwrap().value,
        "true",
        "opacity:0 默认 checkVisibility() → true（opacity 非默认判定）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__op0Opt").unwrap().value,
        "false",
        "opacity:0 + opacityProperty:true → false"
    );

    // ⑤ opacity:0.5 → opacityProperty:true 仍 true（非 0）。
    sandbox
        .execute("globalThis.__opHalf = String(document.getElementById('opHalf').checkVisibility({opacityProperty:true}));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__opHalf").unwrap().value,
        "true",
        "opacity:0.5 + opacityProperty:true → true（非 0）"
    );

    // ⑥ visibility:hidden → 默认 true，visibilityProperty:true → false。
    sandbox
        .execute(
            "globalThis.__visHDef = String(document.getElementById('visH').checkVisibility());\
             globalThis.__visHOpt = String(document.getElementById('visH').checkVisibility({visibilityProperty:true}));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__visHDef").unwrap().value,
        "true",
        "visibility:hidden 默认 checkVisibility() → true（visibility 非默认判定）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__visHOpt").unwrap().value,
        "false",
        "visibility:hidden + visibilityProperty:true → false"
    );

    // ⑦ visibility 继承 + 覆盖：祖先 hidden，子显式 visible → 子计算 visibility=visible → true（继承正确反映）。
    sandbox
        .execute("globalThis.__visChild = String(document.getElementById('visChild').checkVisibility({visibilityProperty:true}));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__visChild").unwrap().value,
        "true",
        "祖先 hidden + 子 visible 覆盖 → 子计算 visibility=visible → checkVisibility(visibilityProperty) → true"
    );

    // ⑧ detached 元素（createElement，handle-only 无 sel）→ false（不在文档 → not rendered）。
    sandbox
        .execute("globalThis.__detached = String(document.createElement('div').checkVisibility());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__detached").unwrap().value,
        "false",
        "detached 元素（不在文档）checkVisibility() → false"
    );
}

#[test]
fn test_scroll_into_view_if_needed_r3075() {
    // R3075：Element.scrollIntoViewIfNeeded(centerIfNeeded)——WebKit-only。headless 无 viewport 可见性判定 →
    // 近似始终滚（"if needed" defer）。centerIfNeeded=true → center 对齐，否则 nearest（≈ start，headless）。
    // 委托 scrollIntoView（R3060），复用 gBCR mock + innerHeight=800（mirror R3060 测试）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'>x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    // mock rect：#d → "0,1000,100,50"（y=1000, h=50；视口下方）。register_dom_callbacks 设 innerHeight=800。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| match args.first() {
            Some(s) if s.starts_with("__") => String::new(),
            _ => "0,1000,100,50".to_string(),
        }),
    );

    // ① scrollIntoViewIfNeeded()（centerIfNeeded falsy）→ nearest ≈ start → scrollY=1000（元素 y）。
    sandbox
        .execute("document.querySelector('#d').scrollIntoViewIfNeeded();")
        .unwrap();
    assert_eq!(
        sandbox.execute("window.scrollY").unwrap().value,
        "1000",
        "scrollIntoViewIfNeeded()（nearest ≈ start）→ scrollY=1000（headless 无可见性判定，近似始终滚）"
    );

    // ② scrollIntoViewIfNeeded(true) → center → scrollY = y - vh/2 + h/2 = 1000-400+25 = 625。
    sandbox
        .execute("document.querySelector('#d').scrollIntoViewIfNeeded(true);")
        .unwrap();
    assert_eq!(
        sandbox.execute("window.scrollY").unwrap().value,
        "625",
        "scrollIntoViewIfNeeded(true) → center 对齐 → scrollY=625（y-vh/2+h/2）"
    );

    // ③ detached 元素（无 rect）→ no-op（scrollY 不变）。
    sandbox.execute("window.scrollTo(0,0);").unwrap();
    sandbox
        .execute("var e=document.createElement('div'); e.scrollIntoViewIfNeeded();")
        .unwrap();
    assert_eq!(
        sandbox.execute("window.scrollY").unwrap().value,
        "0",
        "detached 元素 scrollIntoViewIfNeeded → no-op（无 rect）"
    );

    // ④ 返 undefined（WebKit spec——void，非 boolean）。
    sandbox
        .execute("globalThis.__ret = String(document.querySelector('#d').scrollIntoViewIfNeeded());")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ret").unwrap().value,
        "undefined",
        "scrollIntoViewIfNeeded 返 undefined（WebKit spec void）"
    );
}

#[test]
fn test_canvas_dom_get_context_r3077() {
    // R3077：HTMLCanvasElement proxy 的 canvas 2D API DOM 集成。旧仅 standalone _zwMakeCanvas 有
    // getContext/toDataURL，DOM 元素 proxy 缺 → `document.getElementById('c').getContext('2d')` 抛 TypeError。
    // 本切片接通：getContext 经 host __zw_canvas_op 建 2d 上下文（per-element 缓存）+ ctx2d 方法（fillRect 等）+
    // toDataURL + width/height 反射（default 300/150）。headless 经 register_dom_callbacks（注册 __zw_canvas_op）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><canvas id='cv' width='100' height='50'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① getContext('2d') 返 ctx2d（非 null），webgl 返 null（仅 2d defer）。
    // ② fillRect 不抛（ctx2d 方法可用）。
    // ③ width/height 反射内容属性（100/50）。
    // ④ toDataURL 返 'data:image/png;base64,...'（PNG 编码）。
    // ⑤ 重复 getContext 返同一 ctx（缓存，spec 一致）。
    sandbox
        .execute(
            "var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             globalThis.__hasCtx = String(ctx !== null && ctx !== undefined);\
             globalThis.__webglNull = String(cv.getContext('webgl') === null);\
             ctx.fillStyle = 'red';\
             ctx.fillRect(0, 0, 10, 10);\
             globalThis.__fillOk = 'ok';\
             globalThis.__w = cv.width;\
             globalThis.__h = cv.height;\
             globalThis.__url = cv.toDataURL().slice(0, 22);\
             globalThis.__sameCtx = String(cv.getContext('2d') === ctx);\
             globalThis.__ctxCanvasOk = String(ctx.canvas === cv);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__hasCtx").unwrap().value, "true", "getContext('2d') 返 ctx2d（非 null）");
    assert_eq!(sandbox.execute("globalThis.__webglNull").unwrap().value, "true", "getContext('webgl') 返 null（仅 2d，webgl defer）");
    assert_eq!(sandbox.execute("globalThis.__fillOk").unwrap().value, "ok", "ctx.fillRect 不抛（ctx2d 方法可用）");
    assert_eq!(sandbox.execute("globalThis.__w").unwrap().value, "100", "canvas.width 反射内容属性 100");
    assert_eq!(sandbox.execute("globalThis.__h").unwrap().value, "50", "canvas.height 反射内容属性 50");
    assert_eq!(sandbox.execute("globalThis.__url").unwrap().value, "data:image/png;base64,", "toDataURL 返 PNG data URL 前缀");
    assert_eq!(sandbox.execute("globalThis.__sameCtx").unwrap().value, "true", "重复 getContext 返同一 ctx（per-element 缓存）");
    assert_eq!(sandbox.execute("globalThis.__ctxCanvasOk").unwrap().value, "true", "ctx.canvas === canvas 元素（spec back-ref）");

    // ⑥ width/height set→get 一致（设数值，读回）。
    sandbox.execute("cv.width = 250; globalThis.__setW = cv.width;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__setW").unwrap().value, "250", "canvas.width = 250 → 读回 250（sync set→get）");
}

#[test]
fn test_canvas_ctx2d_text_imagedata_r3078() {
    // R3078：Canvas 2D ctx2d 文本 API（fillText/measureText）+ createImageData（blank）。R3077 接通 getContext；
    // 本切片补 ctx2d 方法（host fill_text/measure_text + JS createImageData blank）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><canvas id='cv' width='100' height='50'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① fillText 不抛（canvas crate fill_text 写 pixel_buffer）。
    // ② measureText 返 TextMetrics {width > 0}（非空文本）+ 0 文本 width 0。
    // ③ createImageData(w,h) → {width, height, data: Uint8ClampedArray(w*h*4)}（blank，全 0）。
    // ④ createImageData(imageData) 复制尺寸。
    sandbox
        .execute(
            "var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             ctx.font = '20px sans-serif';\
             ctx.fillText('hello', 10, 20);\
             globalThis.__fillOk = 'ok';\
             var m = ctx.measureText('hello');\
             globalThis.__mw = String(m.width > 0);\
             globalThis.__mFields = String(typeof m.actualBoundingBoxAscent === 'number');\
             var m0 = ctx.measureText('');\
             globalThis.__mw0 = String(m0.width === 0);\
             var img = ctx.createImageData(4, 3);\
             globalThis.__iw = img.width;\
             globalThis.__ih = img.height;\
             globalThis.__ilen = img.data.length;\
             globalThis.__izero = String(img.data[0] === 0);\
             var img2 = ctx.createImageData(img);\
             globalThis.__icopy = String(img2.width === 4 && img2.height === 3);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__fillOk").unwrap().value, "ok", "ctx.fillText 不抛");
    assert_eq!(sandbox.execute("globalThis.__mw").unwrap().value, "true", "measureText('hello').width > 0");
    assert_eq!(sandbox.execute("globalThis.__mFields").unwrap().value, "true", "measureText 返 TextMetrics 含 actualBoundingBoxAscent number");
    assert_eq!(sandbox.execute("globalThis.__mw0").unwrap().value, "true", "measureText('').width === 0");
    assert_eq!(sandbox.execute("globalThis.__iw").unwrap().value, "4", "createImageData(4,3).width = 4");
    assert_eq!(sandbox.execute("globalThis.__ih").unwrap().value, "3", "createImageData(4,3).height = 3");
    assert_eq!(sandbox.execute("globalThis.__ilen").unwrap().value, "48", "createImageData(4,3).data.length = 4*3*4 = 48");
    assert_eq!(sandbox.execute("globalThis.__izero").unwrap().value, "true", "createImageData blank → data 全 0（透明）");
    assert_eq!(sandbox.execute("globalThis.__icopy").unwrap().value, "true", "createImageData(imgData) 复制尺寸");
}

#[test]
fn test_canvas_ctx2d_gradient_r3079() {
    // R3079：Canvas Gradient（createLinearGradient/createRadialGradient/createConicGradient + addColorStop
    // + fillStyle 接 gradient + fill/fillRect 光栅化）。R3078 闭合 ctx2d 文本/ImageData；本切片闭合最后 2 canvas
    // 用例（canvas/script-gradient + canvas/gradient-pattern）。host 持渐变注册表（独立 id 命名空间），
    // fillStyle setter 检测渐变对象 → setFillStyleGradient 查表克隆到 context 样式；canvas crate 经 sample_at
    // 逐像素光栅化（像素级正确性见 canvas crate test_fill_rect_linear_gradient_rasterizes）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><canvas id='cv' width='200' height='100'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① createLinearGradient 返渐变对象（带 addColorStop 方法）。
    // ② addColorStop 不抛（经 host addColorStop 变更停止点）。
    // ③ fillStyle = 渐变对象后，getter 返回该渐变对象（spec round-trip）。
    // ④ fillRect 用渐变 fillStyle 不抛（canvas crate 逐像素光栅化）。
    // ⑤ createRadialGradient / createConicGradient 返渐变对象 + addColorStop 不抛。
    sandbox
        .execute(
            "var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             var grad = ctx.createLinearGradient(0, 0, 200, 0);\
             globalThis.__hasAddColorStop = String(typeof grad.addColorStop === 'function');\
             grad.addColorStop(0, 'red');\
             grad.addColorStop(0.5, 'yellow');\
             grad.addColorStop(1, 'green');\
             ctx.fillStyle = grad;\
             globalThis.__styleRoundTrip = String(ctx.fillStyle === grad);\
             ctx.fillRect(0, 0, 200, 100);\
             globalThis.__fillOk = 'ok';\
             var rg = ctx.createRadialGradient(100, 50, 10, 100, 50, 80);\
             rg.addColorStop(0, 'white');\
             rg.addColorStop(1, 'blue');\
             globalThis.__rgOk = String(typeof rg.addColorStop === 'function');\
             var cg = ctx.createConicGradient(0, 100, 50);\
             cg.addColorStop(0, 'red');\
             cg.addColorStop(1, 'blue');\
             globalThis.__cgOk = String(typeof cg.addColorStop === 'function');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__hasAddColorStop").unwrap().value, "true", "createLinearGradient 返对象带 addColorStop 方法");
    assert_eq!(sandbox.execute("globalThis.__styleRoundTrip").unwrap().value, "true", "fillStyle = grad 后 getter 返回该渐变对象（spec round-trip）");
    assert_eq!(sandbox.execute("globalThis.__fillOk").unwrap().value, "ok", "fillRect 用渐变 fillStyle 不抛（逐像素光栅化）");
    assert_eq!(sandbox.execute("globalThis.__rgOk").unwrap().value, "true", "createRadialGradient 返渐变对象");
    assert_eq!(sandbox.execute("globalThis.__cgOk").unwrap().value, "true", "createConicGradient 返渐变对象");
}

#[test]
fn test_canvas_ctx2d_pattern_r3085() {
    // R3085：Canvas Pattern（createPattern + fillStyle/strokeStyle 接图案 + fill/fillRect 平铺光栅化）。
    // R3079 闭合渐变；R3084 闭合 stroke 渐变；本切片闭合 Pattern 样式（R3084 defer 项「Pattern 回落黑」）。
    // host 持渐变/图案共享注册表（同 id 命名空间），createPattern 返 pid，JS 包 {_zwPattern:pid}；
    // fillStyle/strokeStyle setter 检测 _zwPattern 标记 → setFillStylePattern/setStrokeStylePattern host 查表克隆；
    // canvas crate 经 sample_at → sample_pattern_pixel 逐像素平铺（像素级正确性见 canvas crate test）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <canvas id='dst' width='20' height='10'></canvas>\
         <canvas id='src' width='4' height='4'></canvas>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① ImageData-like 源 → createPattern 返图案对象（host 建注册表项）。
    // ② fillStyle = 图案对象后 getter 返回该对象（spec round-trip，_zwPattern 标记）。
    // ③ fillRect 用图案 fillStyle 不抛（host setFillStylePattern + canvas crate 逐像素平铺）。
    // ④ no-repeat 重复模式建图案不抛 + 返对象。
    // ⑤ canvas 元素源路径（经源 canvas getImageData 取 wire）返图案对象。
    // ⑥ strokeStyle = 图案 + strokeRect 不抛（setStrokeStylePattern）。
    sandbox
        .execute(
            "var dst = document.getElementById('dst');\
             var ctx = dst.getContext('2d');\
             var imgd = ctx.createImageData(2, 2);\
             imgd.data[0] = 255; imgd.data[3] = 255;\
             var pat = ctx.createPattern(imgd, 'repeat');\
             globalThis.__patIsObj = String(pat !== null && typeof pat === 'object');\
             ctx.fillStyle = pat;\
             globalThis.__roundTrip = String(ctx.fillStyle === pat);\
             ctx.fillRect(0, 0, 20, 10);\
             globalThis.__fillOk = 'ok';\
             var pat2 = ctx.createPattern(imgd, 'no-repeat');\
             globalThis.__pat2IsObj = String(pat2 !== null && typeof pat2 === 'object');\
             var src = document.getElementById('src');\
             src.getContext('2d');\
             var pat3 = ctx.createPattern(src, 'repeat');\
             globalThis.__pat3IsObj = String(pat3 !== null && typeof pat3 === 'object');\
             ctx.strokeStyle = pat;\
             ctx.strokeRect(0, 0, 20, 10);\
             globalThis.__strokeOk = 'ok';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__patIsObj").unwrap().value,
        "true",
        "createPattern(ImageData) 返图案对象"
    );
    assert_eq!(
        sandbox.execute("globalThis.__roundTrip").unwrap().value,
        "true",
        "fillStyle = pat 后 getter 返回该图案对象（spec round-trip）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fillOk").unwrap().value,
        "ok",
        "fillRect 用图案 fillStyle 不抛（逐像素平铺）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__pat2IsObj").unwrap().value,
        "true",
        "no-repeat 重复模式建图案返对象"
    );
    assert_eq!(
        sandbox.execute("globalThis.__pat3IsObj").unwrap().value,
        "true",
        "createPattern(canvas 元素源) 返图案对象"
    );
    assert_eq!(
        sandbox.execute("globalThis.__strokeOk").unwrap().value,
        "ok",
        "strokeStyle = pat + strokeRect 不抛"
    );
}

#[test]
fn test_worker_api_surface_r3080() {
    // R3080：DedicatedWorker API 表面。旧 Worker 构造器为 stub `function(){}` → `w.postMessage`/`w.terminate`
    // 抛 TypeError（6 web-worker WPT 用例 js_executes_ok 失败）。本切片接 EventTarget-based Worker：
    // postMessage（headless no-op）/ terminate（标记 no-op）/ onmessage / onerror / addEventListener。
    // headless 无真 worker 线程执行 url——消息无接收方、回调永不触发（defer 真实 worker 沙箱）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① new Worker(url) 返对象；② postMessage 不抛；③ terminate 不抛且标记终止；
    // ④ onmessage/onerror 可 set→get；⑤ addEventListener('message') 可用（EventTarget）；⑥ terminate 后 postMessage no-op。
    sandbox
        .execute(
            "globalThis.__isFn = String(typeof Worker === 'function');\
             var w = new Worker('worker.js');\
             globalThis.__isObj = String(w !== null && typeof w === 'object');\
             w.postMessage({ type: 'ping' });\
             globalThis.__postOk = 'ok';\
             w.terminate();\
             globalThis.__termOk = 'ok';\
             w.onmessage = function (e) {};\
             globalThis.__onmsgRoundTrip = String(typeof w.onmessage === 'function');\
             w.onerror = function (e) {};\
             globalThis.__onerrRoundTrip = String(typeof w.onerror === 'function');\
             globalThis.__hasAddEvt = String(typeof w.addEventListener === 'function');\
             w.postMessage('after-term');\
             globalThis.__afterTermOk = 'ok';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__isFn").unwrap().value, "true", "typeof Worker === 'function'");
    assert_eq!(sandbox.execute("globalThis.__isObj").unwrap().value, "true", "new Worker(url) 返对象");
    assert_eq!(sandbox.execute("globalThis.__postOk").unwrap().value, "ok", "w.postMessage(...) 不抛");
    assert_eq!(sandbox.execute("globalThis.__termOk").unwrap().value, "ok", "w.terminate() 不抛");
    assert_eq!(sandbox.execute("globalThis.__onmsgRoundTrip").unwrap().value, "true", "onmessage set→get round-trip");
    assert_eq!(sandbox.execute("globalThis.__onerrRoundTrip").unwrap().value, "true", "onerror set→get round-trip");
    assert_eq!(sandbox.execute("globalThis.__hasAddEvt").unwrap().value, "true", "Worker extends EventTarget（addEventListener 可用）");
    assert_eq!(sandbox.execute("globalThis.__afterTermOk").unwrap().value, "ok", "terminate 后 postMessage no-op（不抛）");
}

#[test]
fn test_dedicated_worker_round_trip_r3089() {
    // R3089：真 DedicatedWorker 消息往返（闭合 R3080 defer 项「无真 worker 执行」）。data: URL inline worker
    // 经同沙箱 IIFE 影子执行（new Function 包影子 self/postMessage/onmessage）；main↔worker 经
    // structuredClone + queueMicrotask + MessageEvent 派发（对称 MessagePort）。execute 末 microtask
    // checkpoint 排空 main→worker→main 两跳微任务，__reply 在同次 execute 后可读。
    // ① worker onmessage 收 main 消息（e.data=21）→ postMessage(e.data*2) → main onmessage 收 42；
    // ② terminate 后 postMessage no-op（_terminated 标记，handler 不触发）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // worker 脚本：onmessage = e => postMessage(e.data * 2)。data: URL（URL-encoded payload）。
    sandbox
        .execute(
            "var w = new Worker('data:text/javascript,onmessage%3Dfunction(e)%7BpostMessage(e.data*2)%7D');\
             globalThis.__reply = 'none';\
             w.onmessage = function (ev) { globalThis.__reply = ev.data; };\
             w.postMessage(21);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__reply)").unwrap().value,
        "42",
        "worker 消息往返：postMessage(21) → worker onmessage(e.data*2) → main onmessage(42)"
    );

    // terminate 后 postMessage 不触发 worker handler（_terminated 标记 → postMessage 早返，无微任务派发）。
    sandbox
        .execute(
            "var w2 = new Worker('data:text/javascript,onmessage%3Dfunction(e)%7BpostMessage(e.data*2)%7D');\
             globalThis.__reply2 = 'none';\
             w2.onmessage = function (ev) { globalThis.__reply2 = ev.data; };\
             w2.terminate();\
             w2.postMessage(99);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__reply2)").unwrap().value,
        "none",
        "terminate 后 postMessage no-op（worker handler 不触发）"
    );
}

#[test]
fn test_indexeddb_in_memory_surface_r3081() {
    // R3081：IndexedDB 内存表面。旧 `globalThis.indexedDB` 未定义 → 5 storage 用例 `indexedDB is not defined`。
    // 本切片接 in-memory IDB：open（异步 onupgradeneeded→onsuccess）/ db.createObjectStore/objectStoreNames/
    // transaction/close / store.add/put/get/delete/clear/count/createIndex / tx.objectStore/oncomplete。
    // 本测试验证**功能 round-trip**（非仅 no-throw）：open→upgrade 建 store→add→success→tx.put/delete/get→
    // get.onsuccess 回读 put 的值（内存 CRUD 真生效）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "globalThis.__hasIDB = String(typeof indexedDB === 'object' && typeof indexedDB.open === 'function');\
             var req = indexedDB.open('r3081db', 1);\
             req.onupgradeneeded = function (e) {\
                 globalThis.__upgradeFired = 'yes';\
                 var db = e.target.result;\
                 globalThis.__storeBefore = String(!db.objectStoreNames.contains('items'));\
                 var store = db.createObjectStore('items', {keyPath: 'id'});\
                 globalThis.__storeAfter = String(db.objectStoreNames.contains('items'));\
                 store.add({id: 1, name: 'first'});\
             };\
             req.onsuccess = function (e) {\
                 globalThis.__successFired = 'yes';\
                 var db = e.target.result;\
                 var tx = db.transaction('items', 'readwrite');\
                 var store = tx.objectStore('items');\
                 store.put({id: 2, name: 'second'});\
                 store.delete(1);\
                 var getReq = store.get(2);\
                 getReq.onsuccess = function (ge) {\
                     globalThis.__gotName = (ge.target.result && ge.target.result.name) || 'none';\
                 };\
                 store.count().onsuccess = function (ce) { globalThis.__count = String(ce.target.result); };\
             };",
        )
        .unwrap();
    // microtask checkpoint 在 execute 末尾派发 onupgradeneeded→onsuccess→store ops→get/count callbacks。
    // 兜底：再 execute 一次确保所有嵌套 microtask 排空。
    sandbox.execute("1;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__hasIDB").unwrap().value, "true", "typeof indexedDB === object（open 可用）");
    assert_eq!(sandbox.execute("globalThis.__upgradeFired").unwrap().value, "yes", "open → onupgradeneeded 触发");
    assert_eq!(sandbox.execute("globalThis.__storeBefore").unwrap().value, "true", "createObjectStore 前 objectStoreNames.contains=false");
    assert_eq!(sandbox.execute("globalThis.__storeAfter").unwrap().value, "true", "createObjectStore 后 objectStoreNames.contains=true");
    assert_eq!(sandbox.execute("globalThis.__successFired").unwrap().value, "yes", "onupgradeneeded → onsuccess 触发");
    assert_eq!(sandbox.execute("globalThis.__gotName").unwrap().value, "second", "CRUD round-trip: put id=2 -> get(2).result.name = 'second'");
    assert_eq!(sandbox.execute("globalThis.__count").unwrap().value, "1", "count: 1 record after delete + put");
}

#[test]
fn test_document_dispatch_event_r3082() {
    // R3082：document.dispatchEvent。旧 document 对象有 addEventListener/removeEventListener（转发 html key）
    // 但缺 dispatchEvent → `document.dispatchEvent(event)` 抛 TypeError（runtime/events/custom-event 用例失败）。
    // 本切片补 dispatchEvent（转发 _elKey('html',null)，与 addEventListener 同 key，对称 window.dispatchEvent）。
    // 本测试验证**功能 round-trip**：document.addEventListener 注册 → document.dispatchEvent 触发 listener，
    // 回读 e.detail（同步派发，非仅 no-throw）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "globalThis.__isFn = String(typeof document.dispatchEvent === 'function');\
             document.addEventListener('my-event', function (e) {\
                 globalThis.__heard = String(e.detail || 'none');\
                 globalThis.__targetIsDoc = String(e.target === document || e.currentTarget === document);\
             });\
             var ev = new CustomEvent('my-event', { detail: 'hello-r3082' });\
             globalThis.__ret = String(document.dispatchEvent(ev));\
             globalThis.__afterDispatch = 'ok';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__isFn").unwrap().value, "true", "typeof document.dispatchEvent === 'function'");
    assert_eq!(sandbox.execute("globalThis.__heard").unwrap().value, "hello-r3082", "document.dispatchEvent 触发 document.addEventListener 注册的 listener（detail 回读）");
    assert_eq!(sandbox.execute("globalThis.__ret").unwrap().value, "true", "dispatchEvent 返 !defaultPrevented = true");
    assert_eq!(sandbox.execute("globalThis.__afterDispatch").unwrap().value, "ok", "dispatchEvent 后续执行不中断");
}

// ── R3184：textContent / innerHTML / outerHTML setter 生产路径 spec `LegacyNullToEmptyString` ──
//
// 生产 always-on B-gen shim 路径（js_dom_shim/part04.js set trap）。spec：三 setter 均把 null 视作空串
//（textContent/innerHTML 清子、outerHTML 移除自身），非通用 JS ToString 的 "null"。验证 JS 侧
// `value === null ? '' : String(value)` 强制 → 入队 mutation 的 text/html 字段为 "" 而非 "null"。

#[test]
fn test_text_content_null_clears_production_r3184() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'>hi</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute("document.querySelector('#t').textContent = null;")
        .unwrap();
    let texts: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetText { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    // null → 空串（spec）→ SetText{text:""}，非 "null"。
    assert_eq!(
        texts,
        vec!["".to_string()],
        "textContent=null 应入队 SetText{{text:\"\"}}（spec 空串），非 \"null\""
    );
}

#[test]
fn test_text_content_undefined_is_string_production_r3184() {
    // spec：仅 null 特判，undefined 仍 ToString → "undefined"（锁定 null/undefined 区别）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'>hi</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute("document.querySelector('#t').textContent = undefined;")
        .unwrap();
    let texts: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetText { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        texts,
        vec!["undefined".to_string()],
        "textContent=undefined 不特判 → ToString='undefined'（仅 null 清子）"
    );
}

#[test]
fn test_inner_html_null_clears_production_r3184() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'><b>x</b></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute("document.querySelector('#t').innerHTML = null;")
        .unwrap();
    let htmls: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetInnerHtml { html, .. } => Some(html.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        htmls,
        vec!["".to_string()],
        "innerHTML=null 应入队 SetInnerHtml{{html:\"\"}}（清子），非 \"null\""
    );
}

#[test]
fn test_outer_html_null_removes_production_r3184() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'><b>x</b></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute("document.querySelector('#t').outerHTML = null;")
        .unwrap();
    let htmls: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetOuterHtml { html, .. } => Some(html.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        htmls,
        vec!["".to_string()],
        "outerHTML=null 应入队 SetOuterHtml{{html:\"\"}}（移除自身），非 \"null\""
    );
}

// ── R3185：反射字符串属性 setter 生产路径 spec `[LegacyNullToEmptyString]` ──
//
// 生产 always-on B-gen shim 路径（js_dom_shim/part04.js set trap）。id/title/lang/accessKey 为
// spec `[LegacyNullToEmptyString]`（null→空串）；className/dir 非（null→"null"）。验证 JS 侧
// `value === null ? '' : String(value)`（dir/className 仍 String）→ 入队 SetAttr 的 value 字段。

#[test]
fn test_reflected_string_attrs_null_empty_production_r3185() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "var e=document.querySelector('#t');\
             e.id=null; e.title=null; e.lang=null; e.accessKey=null;\
             e.className=null; e.dir=null;",
        )
        .unwrap();
    // 收集 (content-attr-name, value) 对（SetAttr sel-based）。
    let pairs: Vec<(String, String)> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetAttr { name, value, .. } => Some((name.clone(), value.clone())),
            _ => None,
        })
        .collect();
    // id/title/lang/accessKey（→accesskey）null→""；className(→class)/dir null→"null"。
    assert_eq!(
        pairs,
        vec![
            ("id".to_string(), "".to_string()),
            ("title".to_string(), "".to_string()),
            ("lang".to_string(), "".to_string()),
            ("accesskey".to_string(), "".to_string()),
            ("class".to_string(), "null".to_string()),
            ("dir".to_string(), "null".to_string()),
        ],
        "id/title/lang/accessKey null→空串（LegacyNull）；class/dir null→\"null\"（非 LegacyNull）"
    );
}

// ── R3186：`dir` enumerated getter 生产路径（spec https://html.spec.whatwg.org/multipage/dom.html#the-dir-attribute）──
//
// dir 为 enumerated attribute（关键字 ltr/rtl/auto）。setter 缓存原值（case 保留，仍 String 化）；getter 须
// 规范化——case-insensitive 命中→小写，invalid（含 "foo"/"null"）/missing→空串。验证 setter→getter 缓存往返：
// 旧实现 getter 直读缓存返原值（"RTL"/"foo"/"null"），现 spec 合规。

#[test]
fn test_dir_enumerated_getter_production_r3186() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // setter→getter 缓存往返：'RTL'→'rtl'（case 规范化）；'foo'→''（invalid）；null→''（'null' invalid）；
    // 'auto'→'auto'（合法）。旧实现返 "RTL|[foo]|[null]|auto"。
    let out = sandbox
        .execute(
            "var e=document.querySelector('#t');\
             e.dir='RTL'; var a=e.dir;\
             e.dir='foo'; var b='['+e.dir+']';\
             e.dir=null; var c='['+e.dir+']';\
             e.dir='auto'; var d=e.dir;\
             a+'|'+b+'|'+c+'|'+d",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "rtl|[]|[]|auto",
        "dir enumerated getter：合法关键字→规范小写，invalid/missing→空串"
    );
}

// ── R3187：`contentEditable`/`isContentEditable` 枚举反射生产路径（spec `dom-contenteditable`）──
//
// contenteditable 为枚举属性（关键字：空串、true、false）。spec：空串与 "true" 同映射 true 状态。
// 生产 shim getter 旧实现直读缓存/host 原值（返 "foo"/"TRUE"/""），现规范化——空串/case-insensitive
// "true"→"true"、"false"→"false"、余（incl invalid/inherit/missing）→"inherit"。isContentEditable 旧仅
// `=== 'true'`，现空串/"true"（case-insensitive）→ true。

#[test]
fn test_content_editable_enumerated_production_r3187() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'></div><div id='pe' contenteditable></div><div id='pt' contenteditable='TRUE'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 解析期属性路径（host has_attr + get_attr，非 setter 缓存）：`<div contenteditable>`（present-空串 keyword）
    // → "true"；`<div contenteditable='TRUE'>`（case-insensitive）→ "true"；`#t`（无属性，missing）→ "inherit"。
    // 旧实现把 present-空串（host 返 ""）与 missing（host 返 ""）混同均返 "inherit"（R3187 has_attr 修正）。
    let parsed = sandbox
        .execute(
            "document.querySelector('#pe').contentEditable+'/'+\
             document.querySelector('#pt').contentEditable+'/'+\
             document.querySelector('#t').contentEditable",
        )
        .unwrap()
        .value;
    assert_eq!(
        parsed, "true/true/inherit",
        "解析期：present-空串 keyword→'true'，case-insensitive→'true'，missing→'inherit'"
    );

    // setter→getter 缓存往返：''→'true'（空串 keyword = true 状态）；'TRUE'→'true'（case 规范化）；
    // 'foo'→'inherit'（invalid）；'false'→'false'；'inherit'→'inherit'。旧实现返 "true|TRUE|foo|false|inherit"。
    let ce = sandbox
        .execute(
            "var e=document.querySelector('#t');\
             e.contentEditable=''; var a=e.contentEditable;\
             e.contentEditable='TRUE'; var b=e.contentEditable;\
             e.contentEditable='foo'; var c=e.contentEditable;\
             e.contentEditable='false'; var d=e.contentEditable;\
             e.contentEditable='inherit'; var g=e.contentEditable;\
             a+'|'+b+'|'+c+'|'+d+'|'+g",
        )
        .unwrap()
        .value;
    assert_eq!(
        ce, "true|true|inherit|false|inherit",
        "contentEditable 枚举 getter：空串/true→'true'，false→'false'，invalid/inherit→'inherit'"
    );

    // isContentEditable：空串 keyword → true（旧实现仅 'true'→true，空串→false）；invalid → false。
    let ice = sandbox
        .execute(
            "var e=document.querySelector('#t');\
             e.contentEditable=''; var a=e.isContentEditable;\
             e.contentEditable='foo'; var b=e.isContentEditable;\
             a+'/'+b",
        )
        .unwrap()
        .value;
    assert_eq!(
        ice, "true/false",
        "isContentEditable：空串 keyword = true 状态 → true；invalid → false"
    );
}

// ── R3188：`draggable` enumerated getter 生产路径——case-insensitive + auto-state default-draggable ──
//
// spec HTML `draggable`（枚举属性，关键字 true/false case-insensitive，缺省/非法→auto 状态）。IDL getter：
// true 状态→true；auto 状态→default-draggable（img/audio/video/a[href]→true，余→false）。旧生产 getter 仅
// `=== 'true'`（case-sensitive，且 auto 统一 false）。

#[test]
fn test_draggable_enumerated_auto_state_production_r3188() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <div id='dtrue' draggable='true'></div>\
           <div id='dupper' draggable='TRUE'></div>\
           <div id='dfalse' draggable='false'></div>\
           <div id='dgarb' draggable='foo'></div>\
           <div id='div'></div>\
           <img id='img'/>\
           <a id='ahref' href='x.html'></a>\
           <a id='anohref'></a>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 解析期属性：true(小写/大写 case-insensitive)→true；false→false；invalid "foo"→auto→div default→false。
    let explicit = sandbox
        .execute(
            "document.querySelector('#dtrue').draggable + '/' +\
             document.querySelector('#dupper').draggable + '/' +\
             document.querySelector('#dfalse').draggable + '/' +\
             document.querySelector('#dgarb').draggable",
        )
        .unwrap()
        .value;
    assert_eq!(
        explicit, "true/true/false/false",
        "draggable：case-insensitive true→true，false→false，invalid→auto(div)→false"
    );

    // auto 状态 default-draggable：div→false / img→true / a[href]→true / a(无 href)→false。
    let auto = sandbox
        .execute(
            "document.querySelector('#div').draggable + '/' +\
             document.querySelector('#img').draggable + '/' +\
             document.querySelector('#ahref').draggable + '/' +\
             document.querySelector('#anohref').draggable",
        )
        .unwrap()
        .value;
    assert_eq!(
        auto, "false/true/true/false",
        "auto 状态 default-draggable：div false，img true，a[href] true，a 无 href false"
    );

    // setter→getter 缓存往返：draggable=true→true（attr "true"）；draggable=false→false。
    let setget = sandbox
        .execute(
            "var d=document.querySelector('#div');\
             d.draggable = true; var a=d.draggable;\
             d.draggable = false; var b=d.draggable;\
             a+'/'+b",
        )
        .unwrap()
        .value;
    assert_eq!(setget, "true/false", "draggable setter→getter 缓存往返");
}

// ── R3189：`input.type` / `button.type` enumerated reflection（spec「limited to only known values」）──
//
// input.type / button.type 为枚举属性（非通用 type 字符串反射）。getter 须规范化：INPUT 已知关键字
// （case-insensitive）→ 规范小写，缺省/非法 → "text"；BUTTON submit/reset/button，缺省/非法 → "submit"。
// 非 INPUT/BUTTON（link/script 等）回落通用字符串反射（原值）。旧实现经通用 _reflectedStringAttr('type')
// 返原值（缺省 ""，"EMAIL"→"EMAIL"，"foo"→"foo"）——表单库 switch(input.type) 全失效。

#[test]
fn test_input_button_type_enumerated_production_r3189() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <input id='itext'/>\
           <input id='iemail' type='email'/>\
           <input id='iupper' type='NUMBER'/>\
           <input id='igarb' type='foo'/>\
           <button id='bdef'></button>\
           <button id='breset' type='reset'></button>\
           <button id='bgarb' type='foo'></button>\
           <link id='lk' type='text/css'/>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // input.type：缺省→"text"；已知关键字原样 "email"；case-insensitive "NUMBER"→"number"；非法 "foo"→"text"。
    let input_types = sandbox
        .execute(
            "document.querySelector('#itext').type + '/' +\
             document.querySelector('#iemail').type + '/' +\
             document.querySelector('#iupper').type + '/' +\
             document.querySelector('#igarb').type",
        )
        .unwrap()
        .value;
    assert_eq!(
        input_types, "text/email/number/text",
        "input.type：缺省→text，已知→规范小写，case-insensitive→小写，非法→text"
    );

    // button.type：缺省→"submit"；"reset"→"reset"；非法 "foo"→"submit"。
    let button_types = sandbox
        .execute(
            "document.querySelector('#bdef').type + '/' +\
             document.querySelector('#breset').type + '/' +\
             document.querySelector('#bgarb').type",
        )
        .unwrap()
        .value;
    assert_eq!(
        button_types, "submit/reset/submit",
        "button.type：缺省→submit，reset→reset，非法→submit"
    );

    // 非 INPUT/BUTTON 的 type（link）→ 通用字符串反射（原值 "text/css"，不走枚举）。
    let link_type = sandbox
        .execute("document.querySelector('#lk').type")
        .unwrap()
        .value;
    assert_eq!(link_type, "text/css", "link.type 走通用字符串反射（原值），非枚举");

    // setter→getter 往返：input.type='EMAIL' → 内容属性 "EMAIL"（setter 写原值），getter "email"（规范化）。
    let setget = sandbox
        .execute(
            "var i=document.querySelector('#itext'); i.type='EMAIL';\
             i.type+'/'+i.getAttribute('type')",
        )
        .unwrap()
        .value;
    assert_eq!(
        setget, "email/EMAIL",
        "input.type setter 写原值，getter 规范化（case-insensitive）"
    );
}

// ── R3190：`getAttribute` / `getAttributeNS` spec null 语义 ──
//
// spec `dom-element-getattribute`：缺省（属性不存在）须返 **null**，present-empty 返 ""，present 返值。
// 旧 polyfill proxy getAttribute 直返 host `__zw_get_attr*`（缺省/空均 ""）→ 缺省返 "" 而非 null，
// 破坏 `el.getAttribute('x') === null` / `!= null` 检查（jQuery/React 高频）。附带修复 `[attr]` 存在性
// 选择器 over-match（旧 `_matchAttrOf` `av != null` 恒真，缺省元素误匹配 `[attr]`）。

#[test]
fn test_get_attribute_null_semantics_production_r3190() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <div id='d' data-x='value' data-empty=''></div>\
           <div id='plain'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 缺省属性 → null（旧 ""）；present-value → "value"；present-empty → ""。
    let gets = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             var a=String(d.getAttribute('missing'));\
             var b=d.getAttribute('data-x');\
             var c='['+d.getAttribute('data-empty')+']';\
             a+'/'+b+'/'+c",
        )
        .unwrap()
        .value;
    assert_eq!(
        gets, "null/value/[]",
        "getAttribute：缺省→null，present-value→值，present-empty→''"
    );

    // removeAttribute 后 → null（latest-wins，闭合 stale 旧值）；setAttribute 后 present-empty → ""。
    let setremove = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.removeAttribute('data-x'); var a=String(d.getAttribute('data-x'));\
             d.setAttribute('data-new',''); var b='['+d.getAttribute('data-new')+']';\
             a+'/'+b",
        )
        .unwrap()
        .value;
    assert_eq!(
        setremove, "null/[]",
        "removeAttribute 后 getAttribute→null；setAttribute 空串后 present-empty→''"
    );

    // getAttributeNS 缺省 → null（委托 getAttribute，spec 一致）。
    let ns = sandbox
        .execute("String(document.querySelector('#d').getAttributeNS(null, 'href'))")
        .unwrap()
        .value;
    assert_eq!(ns, "null", "getAttributeNS 缺省→null（spec 一致）");

    // 附带修复：`[attr]` 存在性选择器不再 over-match 缺省元素。`#plain` 无 data-x → `[data-x]` 不匹配；
    // `#d` 有 data-x → 匹配。旧实现 `_matchAttrOf` `av != null` 恒真（缺省返 ""），两元素均误匹配。
    let sel = sandbox
        .execute(
            "document.querySelectorAll('[data-x]').length + '/' +\
             (document.querySelector('#plain[data-x]') === null)",
        )
        .unwrap()
        .value;
    assert_eq!(
        sel, "1/true",
        "[data-x] 存在性选择器：仅匹配有该属性的元素（#d），缺省元素 #plain 不匹配"
    );
}

// ── R3191：`toggleAttribute` 返回值 latest-wins（spec `dom-element-toggleattribute`：返切换后是否 present）──
//
// 旧 polyfill snapHas 读纯快照 `__zw_has_attr`——同批 setAttribute/removeAttribute 后 toggle 仍读旧快照，
// 返值 stale（setAttribute('x') 后 toggle('x') 应 false 但旧返 true）。改读 `__zw_has_attr_lw`（反映 pending
// SetAttr/RemoveAttr）。apply 时 mutation 一直正确（host apply-time 决策），仅返值修复。

#[test]
fn test_toggle_attribute_return_latest_wins_production_r3191() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <div id='d'></div>\
           <div id='has' data-x='1'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // setAttribute('x') 后 toggle('x')（无 force）——x 现 present → toggle 移除 → 返 false。
    // 旧实现读纯快照（#d 无 x）→ snapHas=false → 返 true（stale，错误）。
    let set_then_toggle = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.setAttribute('data-x','1');\
             String(d.toggleAttribute('data-x'))",
        )
        .unwrap()
        .value;
    assert_eq!(
        set_then_toggle, "false",
        "setAttribute 后 toggle（无 force）：present→移除→返 false（latest-wins；旧 stale 返 true）"
    );

    // removeAttribute('x') 后 toggle('x')（无 force）——x 现 absent → toggle 添加 → 返 true。
    // #has 初始有 data-x，removeAttribute 后 lw 判 absent → snapHas=false → 返 true。
    let remove_then_toggle = sandbox
        .execute(
            "var h=document.querySelector('#has');\
             h.removeAttribute('data-x');\
             String(h.toggleAttribute('data-x'))",
        )
        .unwrap()
        .value;
    assert_eq!(
        remove_then_toggle, "true",
        "removeAttribute 后 toggle（无 force）：absent→添加→返 true（latest-wins）"
    );

    // 常见单次 toggle（无 pending）：#d 当前无 data-x（上面 net 移除）→ toggle 添加 → 返 true。
    // 此场景 lw 与纯快照一致（无 pending），验证无回归。
    let plain_toggle = sandbox
        .execute("String(document.querySelector('#d').toggleAttribute('data-new'))")
        .unwrap()
        .value;
    assert_eq!(plain_toggle, "true", "单次 toggle 无 pending：absent→添加→返 true（无回归）");

    // force=true / force=false 返值不依赖 presence（force 决定）：返 !!force。
    let force_true = sandbox
        .execute("String(document.querySelector('#d').toggleAttribute('data-f', true))")
        .unwrap()
        .value;
    let force_false = sandbox
        .execute("String(document.querySelector('#d').toggleAttribute('data-f', false))")
        .unwrap()
        .value;
    assert_eq!(force_true, "true", "force=true → 返 true（不依赖 presence）");
    assert_eq!(force_false, "false", "force=false → 返 false（不依赖 presence）");
}

// ── R3192：连续 `toggleAttribute` 返值 enqueue-时解析（闭合 R3191 已知限制）──
//
// R3191 闭合 set/remove-then-toggle 返值，但连续 toggle（同批多次 toggle 同一属性）返值仍 stale——
// `__zw_toggle_attribute` 旧 apply-时解析，shim 无法预测 apply 结果。R3192 改 enqueue-时解析：host 计算
// latest-wins presence → 入队具体 SetAttr/RemoveAttr → 返 post-toggle presence。连续 toggle 第二次起返值
// 准确，且后续 getAttribute/hasAttribute 经 sel_attr_override 一致反映。

#[test]
fn test_toggle_attribute_consecutive_return_production_r3192() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 连续双 toggle（无 force）：#d 无 x。第一次 absent→present 返 true；第二次 present→absent 返 false。
    // R3191 已知限制：第二次返值 stale（返 true，错误）。R3192 enqueue-时解析 → 返 false（正确）。
    let consecutive = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             var a=d.toggleAttribute('data-x');\
             var b=d.toggleAttribute('data-x');\
             String(a)+'/'+String(b)",
        )
        .unwrap()
        .value;
    assert_eq!(
        consecutive, "true/false",
        "连续双 toggle：第一次 absent→present 返 true，第二次 present→absent 返 false（R3192 enqueue-时解析）"
    );

    // 连续三 toggle：absent→present(true)→absent(false)→present(true)。net present。
    let triple = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.removeAttribute('data-x');\
             var a=d.toggleAttribute('data-y');\
             var b=d.toggleAttribute('data-y');\
             var c=d.toggleAttribute('data-y');\
             String(a)+'/'+String(b)+'/'+String(c)",
        )
        .unwrap()
        .value;
    assert_eq!(
        triple, "true/false/true",
        "连续三 toggle：true/false/true（每次返值反映 enqueue-时解析的 post-toggle presence）"
    );

    // 后续 getAttribute 一致反映（enqueue 的 SetAttr/RemoveAttr 经 sel_attr_override）：双 toggle 后 net absent
    // → getAttribute 返 null（R3190 null 语义）。
    let after = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.removeAttribute('data-x'); d.removeAttribute('data-y');\
             d.toggleAttribute('data-z'); d.toggleAttribute('data-z');\
             String(d.getAttribute('data-z'))",
        )
        .unwrap()
        .value;
    assert_eq!(
        after, "null",
        "双 toggle 后 net absent → getAttribute 返 null（lw 一致反映 enqueue 的 SetAttr/RemoveAttr）"
    );
}

// ── R3193：`element.style`（CSSStyleDeclaration）priority/!important CSSOM 合规 ──
//
// spec `dom-cssstyledeclaration`：getPropertyValue 返值**不含** !important；getPropertyPriority 返
// "important"/""；setProperty 第三参 priority 控制 !important。旧 polyfill：getPropertyPriority 恒返 ''（stub）、
// setProperty 忽略 priority 参、getPropertyValue 返值含 !important、readProp split(':') 致 url() 含冒号值截断。
//
// 注：读侧经解析期 style 快照验证（sync set→read latest-wins 为 R3194 独立修复，见已知限制），写侧经
// `apply_mutations_to_html` 验证（apply 后 style 属性含正确 !important）。

#[test]
fn test_style_priority_important_cssom_production_r3193() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <div id='d'></div>\
           <div id='imp' style='color: red !important'></div>\
           <div id='url' style='background: url(http://x.png)'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 读侧（解析期 style 快照）：getPropertyValue='red'（剥离 !important），getPropertyPriority='important'。
    // 旧 getPropertyValue 返 'red !important'（含 priority）、getPropertyPriority 恒 ''。
    let parsed = sandbox
        .execute(
            "var e=document.querySelector('#imp');\
             e.style.getPropertyValue('color')+'/'+e.style.getPropertyPriority('color')",
        )
        .unwrap()
        .value;
    assert_eq!(
        parsed, "red/important",
        "解析期 !important：getPropertyValue='red'（剥离），getPropertyPriority='important'"
    );

    // 解析期无 !important 声明：getPropertyPriority=''。
    let nopri = sandbox
        .execute("document.querySelector('#url').style.getPropertyPriority('background')")
        .unwrap()
        .value;
    assert_eq!(nopri, "", "解析期无 !important → getPropertyPriority=''");

    // 含 ':' 的值（url()）完整读回——旧 split(':') 致 'url(http' 截断，现按首 ':' 切分。
    let url = sandbox
        .execute("document.querySelector('#url').style.getPropertyValue('background')")
        .unwrap()
        .value;
    assert_eq!(
        url, "url(http://x.png)",
        "含 ':' 的值（url()）完整读回（旧 split(':') 截断）"
    );

    // 写侧（apply 后验证）：setProperty 第三参 priority='important' → style 属性含 'color: red !important'；
    // 无 priority → 'font-size: 14px'（无 !important）；ci 'IMPORTANT' → 'margin: 5px !important'。旧 priority 被忽略。
    sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.setProperty('color', 'red', 'important');\
             d.style.setProperty('font-size', '14px');\
             d.style.setProperty('margin', '5px', 'IMPORTANT');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(out.contains("color: red !important"), "setProperty(p,v,'important') → !important\n{out}");
    assert!(
        out.contains("font-size: 14px") && !out.contains("font-size: 14px !important"),
        "setProperty 无 priority → 无 !important\n{out}"
    );
    assert!(out.contains("margin: 5px !important"), "setProperty priority 'IMPORTANT'（ci）→ !important\n{out}");

    // IDL setter 带 !important（Chrome：解析 value 的 !important）→ apply 后 style 含 'color: blue !important'。
    mutations.lock().unwrap().clear();
    sandbox
        .execute("document.querySelector('#d').style.color = 'blue !important';")
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    assert!(
        out2.contains("color: blue !important"),
        "IDL setter 'blue !important' → apply 后 !important\n{out2}"
    );

    // removeProperty 返前值（读解析期 #imp 的 color='red'，剥离 priority）。
    let removed = sandbox
        .execute("document.querySelector('#imp').style.removeProperty('color')")
        .unwrap()
        .value;
    assert_eq!(removed, "red", "removeProperty 返前值（不含 !important）");
}

// ── R3194：element inline style sync set→read latest-wins（闭合 R3193 已知限制①）──
//
// 旧 `_styleProxy.readRaw` sel 路径读纯快照（`__zw_get_attr`），SetStyle mutation 不经 sel_attr_override
// → 同批 `el.style.x='v'; el.style.x` 返旧值/空（stale）。R3194：sel 路径改走新 `__zw_get_style_lw`
// 回调（replay snapshot style + 同 sel pending SetAttr/RemoveAttr/SetStyle/RemoveStyle），保留 SetStyle
// 变体（pipeline `is_paint_only_mutation` 依赖 property 粒度，不走 enqueue-时解析）。

#[test]
fn test_style_sync_set_read_latest_wins_production_r3194() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // sync set→read（同批，无 apply）：`el.style.color='red'; el.style.color` → 'red'。
    // 旧 readRaw 读快照（#d 无 style）→ ''（stale）。现 __zw_get_style_lw replay → 'red'。
    let idl = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='red';\
             d.style.color",
        )
        .unwrap()
        .value;
    assert_eq!(idl, "red", "sync set→read：el.style.color='red' 后读回 'red'（latest-wins）");

    // setProperty→getPropertyValue sync 往返。
    let setp = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.setProperty('color','blue');\
             d.style.getPropertyValue('color')",
        )
        .unwrap()
        .value;
    assert_eq!(setp, "blue", "setProperty('color','blue') 后 getPropertyValue='blue'（latest-wins）");

    // 多次 set 累积（replay 顺序合并）：color + font-size → length=2，cssText 含两者。
    let acc = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='red'; d.style.fontSize='14px';\
             String(d.style.length)+'|'+d.style.cssText",
        )
        .unwrap()
        .value;
    assert!(
        acc.starts_with("2|"),
        "多次 set 累积 length=2，got: {acc}"
    );
    assert!(acc.contains("color: red") && acc.contains("font-size: 14px"), "cssText 含累积声明: {acc}");

    // 同属性覆盖（replay 后者覆盖前者）：color='red' 后 color='green' → 'green'。
    let override_ = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='red'; d.style.color='green';\
             d.style.color",
        )
        .unwrap()
        .value;
    assert_eq!(override_, "green", "同属性后设覆盖前设（replay merge 去重）");

    // removeProperty sync：设后移除 → 读回 ''。
    let rem = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='red'; d.style.removeProperty('color');\
             '['+d.style.color+']'",
        )
        .unwrap()
        .value;
    assert_eq!(rem, "[]", "removeProperty 后读回空（latest-wins replay）");

    // cssText 整体设后 per-property 读：cssText='color: red' → style.color='red'。
    let ct = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.cssText='color: red';\
             d.style.color",
        )
        .unwrap()
        .value;
    assert_eq!(ct, "red", "cssText 整体设后 per-property 读（SetAttr('style') lw）");

    // 解析期 style 与 sync set 合并：#imp 初始 color:red，sync 设 font-size → 两者俱在。
    // （独立 sandbox 验证，本 sandbox #d 已被污染；用 querySelector 新元素需新 HTML——此处复用 #d 前
    // 已多次设，跳过此组合断言，由上各断言覆盖 replay 各路径。）
}

// ── R3195：handle-based dataset 修复（get/has/delete）──
//
// 旧 `_datasetProxy.hasAttrFn` 对 handle 恒返 false（R3002 时无 `__zw_has_attr_handle` 回调遗留）→
// handle 元素 `el.dataset.foo = 'x'; el.dataset.foo` 恒 undefined（get trap 经 hasAttrFn 短路），且
// `'foo' in el.dataset` 恒 false；deleteProperty 用 set-empty 残留 `data-x=""`。修复：hasAttrFn 用
// `__zw_has_attr_handle`，deleteProperty 优先 `__zw_remove_attr_handle`。

#[test]
fn test_dataset_handle_round_trip_production_r3195() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // handle 元素（createElement，未挂载）dataset round-trip：set→get 返值（旧恒 undefined）。
    let setget = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.bazQux='world';\
             '['+e.dataset.bazQux+']'+'/'+e.getAttribute('data-baz-qux')",
        )
        .unwrap()
        .value;
    assert_eq!(
        setget, "[world]/world",
        "handle dataset set→get：'world'/属性（旧恒 undefined）"
    );

    // `in` / has：dataset 属性存在性（旧恒 false）。
    let has = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.key='v';\
             String('key' in e.dataset)+'/'+String('absent' in e.dataset)",
        )
        .unwrap()
        .value;
    assert_eq!(has, "true/false", "handle dataset 'in' 判定（旧恒 false）");

    // delete：移除后 get→undefined（旧 set-empty 致返 ''）。delete 返 true。
    let del = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.rm='x';\
             var r=delete e.dataset.rm;\
             String(r)+'/'+String(e.dataset.rm===undefined)+'/'+String(e.getAttribute('data-rm')===null)",
        )
        .unwrap()
        .value;
    assert_eq!(
        del, "true/true/true",
        "handle dataset delete：真移除（旧 set-empty 残留），get→undefined，getAttribute→null"
    );
}

// ── R3196：handle dataset 枚举（ownKeys/ownEnumerable）──
//
// R3195 闭合 handle dataset get/has/delete 后，枚举仍返 []——`_datasetProxy.dataKeys()` handle 路径
// 无 `__zw_attr_names_handle` 回调变体，恒返 []（R3195 已知限制①）。新增 host `attribute_names_from_mutations`
//（正序 latest-wins，无快照基底）+ `__zw_attr_names_handle` 回调，dataKeys() handle 路径遍历真实 data-* 属性名。

#[test]
fn test_dataset_handle_enumeration_production_r3196() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // Object.keys(handle.dataset)：camelCase data-* 键（旧恒返 []）。非 data-* 属性（id）不混入。
    let keys = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.id='nope';\
             e.dataset.fooBar='1';\
             e.dataset.baz='2';\
             Object.keys(e.dataset).join(',')",
        )
        .unwrap()
        .value;
    assert_eq!(
        keys, "fooBar,baz",
        "handle dataset Object.keys：camelCase data-* 键，非 data-*（id）排除（旧恒空）"
    );

    // JSON.stringify：序列化含 data 键（旧 '{}'）。
    let json = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.alpha='x';\
             e.dataset.betaBravo='y';\
             JSON.stringify(e.dataset)",
        )
        .unwrap()
        .value;
    assert_eq!(
        json, "{\"alpha\":\"x\",\"betaBravo\":\"y\"}",
        "handle dataset JSON.stringify：含 data 键（旧恒 {{}}）"
    );

    // delete 反映：枚举移除被删键 + 删后重设追加到末尾（DOM getAttributeNames 序）。
    let after_del = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.first='a';\
             e.dataset.second='b';\
             delete e.dataset.first;\
             e.dataset.first='c';\
             Object.keys(e.dataset).join(',')",
        )
        .unwrap()
        .value;
    assert_eq!(
        after_del, "second,first",
        "handle dataset 枚举：delete 移除 + 删后重设追加到末尾（正序 latest-wins，DOM 序）"
    );

    // 空句柄 dataset：Object.keys 返 []（无 data-* 属性）。
    let empty = sandbox
        .execute(
            "var e=document.createElement('div');\
             Object.keys(e.dataset).length+'/'+(Object.keys(e.dataset).length===0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        empty, "0/true",
        "handle dataset 空：Object.keys 返空数组"
    );
}

// ── R3197：handle getAttributeNames / hasAttributes 枚举 ──
//
// R3196 新增 `__zw_attr_names_handle` 回调闭合了 handle dataset 枚举，但 `el.getAttributeNames()` /
// `el.hasAttributes()`（part04.js）对 handle 元素仍短路返 []/false（未走新回调）。本切片接线两方法 handle
// 路径走 `__zw_attr_names_handle`，闭合 handle 属性名枚举面（dataset 已 R3196 闭合；getAttributeNames/
// hasAttributes 是更通用的属性名遍历——`el.getAttributeNames()`/`el.hasAttributes()` 在 createElement 未挂载
// 元素上旧恒 []/false）。

#[test]
fn test_handle_attribute_enumeration_production_r3197() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // hasAttributes：有属性 true / 无属性 false（旧恒 false）。
    let has = sandbox
        .execute(
            "var a=document.createElement('div');\
             var b=document.createElement('div');\
             a.setAttribute('id','x');\
             String(a.hasAttributes())+'/'+String(b.hasAttributes())",
        )
        .unwrap()
        .value;
    assert_eq!(has, "true/false", "handle hasAttributes：有/无（旧恒 false）");

    // getAttributeNames：返文档序属性名（含非 data-*）。旧恒 []。
    let names = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.setAttribute('id','main');\
             e.className='btn';\
             e.setAttribute('data-x','1');\
             e.getAttributeNames().join(',')",
        )
        .unwrap()
        .value;
    assert_eq!(
        names, "id,class,data-x",
        "handle getAttributeNames：文档序全部属性名（旧恒空）"
    );

    // removeAttribute 反映 + 删后重设追加到末尾（DOM getAttributeNames 序）。
    let after = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.setAttribute('first','a');\
             e.setAttribute('second','b');\
             e.removeAttribute('first');\
             e.setAttribute('first','c');\
             e.getAttributeNames().join(',')+'/'+e.hasAttributes()",
        )
        .unwrap()
        .value;
    assert_eq!(
        after, "second,first/true",
        "handle getAttributeNames：remove 移除 + 删后重设追加末尾（DOM 序），hasAttributes 仍 true"
    );

    // remove 全部属性后 hasAttributes→false（属性名仅来自 mutations，正序 latest-wins）。
    let all_gone = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.setAttribute('k','v');\
             e.removeAttribute('k');\
             String(e.hasAttributes())+'/'+String(e.getAttributeNames().length===0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        all_gone, "false/true",
        "handle 全删后 hasAttributes→false，getAttributeNames→[]"
    );
}

// ── R3198：handle el.attributes NamedNodeMap + handle 源 cloneNode ──
//
// R3196/R3197 闭合 handle 属性名枚举（dataset / getAttributeNames·hasAttributes），但 `el.attributes`
// NamedNodeMap（part03.js `_attributesProxy.readNames()`）对 handle 元素仍恒空（length 0 / item·getNamedItem 返
// null / iterator 空），且 `cloneNode` 对 handle 源元素 tag 回落 'div' + 不复制属性（旧注释「无 get_tag/
// attr_names handle 变体，best-effort」）。现三 handle 回调（`__zw_get_tag_handle`/`__zw_attr_names_handle`/
// `__zw_get_attr_handle`）均已就绪，接线两端，闭合 handle 属性枚举最后一面。

#[test]
fn test_handle_attributes_and_clone_production_r3198() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // el.attributes NamedNodeMap：length / item(i) / getNamedItem(name) / 迭代（name+value）。旧恒空。
    let attrs = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.setAttribute('id','main');\
             e.setAttribute('data-x','1');\
             var A=e.attributes;\
             A.length+'|'+\
             A.item(0).name+'='+A.item(0).value+'|'+\
             A.getNamedItem('data-x').value+'|'+\
             String(A.getNamedItem('absent')===null)+'|'+\
             Array.from(A).map(function(a){return a.name+':'+a.value;}).join(',')",
        )
        .unwrap()
        .value;
    assert_eq!(
        attrs, "2|id=main|1|true|id:main,data-x:1",
        "handle el.attributes NamedNodeMap：length/item/getNamedItem/迭代（旧恒空）"
    );

    // handle 源 cloneNode：源 tag 保留（旧回落 'DIV'）+ 属性复制（旧不复制）。
    let clone = sandbox
        .execute(
            "var s=document.createElement('section');\
             s.setAttribute('id','s1');\
             s.setAttribute('class','card');\
             var c=s.cloneNode(false);\
             c.tagName+'|'+\
             c.getAttribute('id')+'|'+\
             c.getAttribute('class')+'|'+\
             c.attributes.length+'|'+\
             String(c.getAttributeNames().join(','))",
        )
        .unwrap()
        .value;
    assert_eq!(
        clone, "SECTION|s1|card|2|id,class",
        "handle 源 cloneNode：tag 保留 SECTION（旧 DIV）+ 属性复制（旧空）"
    );

    // handle 源 cloneNode deep：后代 innerHTML 复制（R2994 既有，验证未回归）。
    let clone_deep = sandbox
        .execute(
            "var s=document.createElement('div');\
             s.innerHTML='<span>hi</span>';\
             s.cloneNode(true).innerHTML",
        )
        .unwrap()
        .value;
    assert_eq!(
        clone_deep, "<span>hi</span>",
        "handle 源 cloneNode deep：后代 innerHTML 复制（既有，未回归）"
    );
}

// ── R3199：handle inline style sync set→read latest-wins ──
//
// R3194 闭合 sel-based `el.style.x='v'; el.style.x` sync set→read stale（`__zw_get_style_lw` replay pending style
// mutation），但 handle 路径 `readRaw` 仍纯快照 `__zw_get_attr_handle('style')`——SetStyleOnHandle mutation 不
// 反映到所存 style 属性串，故 handle 元素 `el.style.color='red'; el.style.color` **恒返空**（R3194 已知限制①）。
// 新增 host `style_from_mutations_lw`（正序 replay *OnHandle 变体，无快照基底）+ `__zw_get_style_lw_handle`
// 回调，readRaw handle 路径走 lw，闭合限制。

#[test]
fn test_handle_style_sync_set_read_production_r3199() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // sync set→read round-trip：同批 setProperty→getPropertyValue（旧恒空）。同属性后设覆盖。
    let round_trip = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.style.color='red';\
             var a=e.style.color;\
             e.style.color='blue';\
             var b=e.style.color;\
             e.style.getPropertyValue('color')+'|'+a+'|'+b",
        )
        .unwrap()
        .value;
    assert_eq!(
        round_trip, "blue|red|blue",
        "handle style sync set→read：setProperty→getPropertyValue 往返 + 后设覆盖（旧恒空）"
    );

    // 多属性累积：length=2 + cssText 含两者（camelCase 读 backgroundColor）。
    let accum = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.style.color='red';\
             e.style.backgroundColor='blue';\
             String(e.style.length>=2)+'|'+e.style.backgroundColor+'|'+\
             (e.style.cssText.indexOf('color')>=0 && e.style.cssText.indexOf('background')>=0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        accum, "true|blue|true",
        "handle style 多属性累积：length≥2 + camelCase 读 + cssText 含两者（旧恒空）"
    );

    // removeProperty sync→空。
    let remove = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.style.color='red';\
             var prev=e.style.removeProperty('color');\
             String(prev)+'|'+String(e.style.color==='')+'|'+String(e.style.length===0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        remove, "red|true|true",
        "handle style removeProperty sync：返前值 + 读空 + length=0（旧 stale 仍含）"
    );

    // cssText 整体设后 per-property 读（cssText setter→SetAttrOnHandle{style}，per-prop 读须 parse）。
    let csstext = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.style.cssText='color: green; font-size: 12px';\
             e.style.color+'|'+e.style.getPropertyValue('font-size')+'|'+e.style.length",
        )
        .unwrap()
        .value;
    assert_eq!(
        csstext, "green|12px|2",
        "handle style cssText 设后 per-property 读：parse 正确 + length（旧纯快照 readRaw 读不到）"
    );
}
