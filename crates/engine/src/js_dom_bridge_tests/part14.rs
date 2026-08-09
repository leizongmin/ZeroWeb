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
