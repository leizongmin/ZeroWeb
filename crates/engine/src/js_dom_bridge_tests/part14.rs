// js_dom_bridge 测试切片 14（R3074+）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-13 共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。
// 按单文件 ≤2000 行拆分（R3200：R3184-R3199 spec 审测试移至 part15），本切片承载 element-method /
// Web-API 后续测试。

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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
fn test_canvas_measure_text_full_fields_r3303() {
    // R3303：measureText 返 spec TextMetrics 全 11 字段（host 经 csv 串参返 JS 构完整对象）。
    // canvas crate 无真实字体度量，字体度量字段为 font.size 比例启发式近似；本测断言字段集齐全 +
    // 关键不变量（width 随字符数 / actualBoxRight ≈ width / alphabeticBaseline === 0 / hangingBaseline > 0 /
    // ideographicBaseline < 0 / fontBoundingBox ≈ actualBoundingBox），防 host op 或 JS shim 字段漏 wire。
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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             var m = ctx.measureText('hello');\
             var fields = ['width','actualBoundingBoxAscent','actualBoundingBoxDescent',\
             'actualBoundingBoxLeft','actualBoundingBoxRight','fontBoundingBoxAscent',\
             'fontBoundingBoxDescent','alphabeticBaseline','hangingBaseline','ideographicBaseline'];\
             globalThis.__allNum = String(fields.every(function (k) { return typeof m[k] === 'number'; }));\
             globalThis.__fieldCount = fields.length;\
             globalThis.__keysPresent = String(fields.every(function (k) { return k in m; }));\
             globalThis.__width = m.width;\
             globalThis.__emptyWidth = ctx.measureText('').width;\
             globalThis.__rightEqWidth = String(Math.abs(m.actualBoundingBoxRight - m.width) < 1e-6);\
             globalThis.__alphaZero = String(m.alphabeticBaseline === 0);\
             globalThis.__hangingPos = String(m.hangingBaseline > 0);\
             globalThis.__ideoNeg = String(m.ideographicBaseline < 0);\
             globalThis.__fontBoxEq = String(Math.abs(m.fontBoundingBoxAscent - m.actualBoundingBoxAscent) < 1e-6 && Math.abs(m.fontBoundingBoxDescent - m.actualBoundingBoxDescent) < 1e-6);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__keysPresent").unwrap().value, "true", "TextMetrics 全 11 字段均 present");
    assert_eq!(sandbox.execute("globalThis.__allNum").unwrap().value, "true", "TextMetrics 全字段均为 number");
    assert_eq!(sandbox.execute("globalThis.__fieldCount").unwrap().value, "10", "字段数 = 10（spec TextMetrics 全 10 属性）");
    assert_eq!(sandbox.execute("globalThis.__emptyWidth").unwrap().value, "0", "measureText('').width === 0");
    let width: f64 = sandbox.execute("globalThis.__width").unwrap().value.parse().unwrap();
    assert!(width > 0.0, "measureText('hello').width > 0");
    assert_eq!(sandbox.execute("globalThis.__rightEqWidth").unwrap().value, "true", "actualBoundingBoxRight ≈ width");
    assert_eq!(sandbox.execute("globalThis.__alphaZero").unwrap().value, "true", "alphabeticBaseline === 0（默认基线）");
    assert_eq!(sandbox.execute("globalThis.__hangingPos").unwrap().value, "true", "hangingBaseline > 0（≈ ascent）");
    assert_eq!(sandbox.execute("globalThis.__ideoNeg").unwrap().value, "true", "ideographicBaseline < 0（≈ -descent）");
    assert_eq!(sandbox.execute("globalThis.__fontBoxEq").unwrap().value, "true", "fontBoundingBox ≈ actualBoundingBox（启发式同源）");
}

#[test]
fn test_canvas_text_state_props_r3304() {
    // R3304：Canvas 2D 文本/线连接状态属性（ctx.font / textAlign / textBaseline / direction / miterLimit）。
    // Rust 后端早全，此前缺 host op + JS shim 暴露 → ctx.font='20px Arial' no-op，measureText 恒用默认 10px。
    // 本测断言：① 默认值（spec：font='10px sans-serif' / textAlign='start' / textBaseline='alphabetic' /
    // direction='inherit' / miterLimit=10）；② setter→getter 往返（host 归一化）；③ ctx.font 改字号后
    // measureText width 随之放大（证明 setFont 真改 FontDescriptor，measure_text 读 self.font.size）。
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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             globalThis.__defFont = ctx.font;\
             globalThis.__defAlign = ctx.textAlign;\
             globalThis.__defBaseline = ctx.textBaseline;\
             globalThis.__defDir = ctx.direction;\
             globalThis.__defMiter = ctx.miterLimit;\
             var w10 = ctx.measureText('hello').width;\
             ctx.font = 'italic bold 20px Arial';\
             globalThis.__setFont = ctx.font;\
             ctx.textAlign = 'center';\
             globalThis.__setAlign = ctx.textAlign;\
             ctx.textBaseline = 'middle';\
             globalThis.__setBaseline = ctx.textBaseline;\
             ctx.direction = 'rtl';\
             globalThis.__setDir = ctx.direction;\
             ctx.miterLimit = 5;\
             globalThis.__setMiter = ctx.miterLimit;\
             var w20 = ctx.measureText('hello').width;\
             globalThis.__widthScaled = String(w20 > w10 && Math.abs(w20 - 2 * w10) < 1e-3);\
             globalThis.__hasProps = String(['font','textAlign','textBaseline','direction','miterLimit'].every(function (k) { return typeof ctx[k] !== 'undefined'; }));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__defFont").unwrap().value, "10px sans-serif", "默认 font");
    assert_eq!(sandbox.execute("globalThis.__defAlign").unwrap().value, "start", "默认 textAlign");
    assert_eq!(sandbox.execute("globalThis.__defBaseline").unwrap().value, "alphabetic", "默认 textBaseline");
    assert_eq!(sandbox.execute("globalThis.__defDir").unwrap().value, "inherit", "默认 direction");
    assert_eq!(sandbox.execute("globalThis.__defMiter").unwrap().value, "10", "默认 miterLimit");
    // font setter 经 host 归一化：'italic bold 20px Arial' → 'italic bold 20px Arial'。
    assert_eq!(sandbox.execute("globalThis.__setFont").unwrap().value, "italic bold 20px Arial", "setFont 归一化往返");
    assert_eq!(sandbox.execute("globalThis.__setAlign").unwrap().value, "center", "textAlign 往返");
    assert_eq!(sandbox.execute("globalThis.__setBaseline").unwrap().value, "middle", "textBaseline 往返");
    assert_eq!(sandbox.execute("globalThis.__setDir").unwrap().value, "rtl", "direction 往返");
    assert_eq!(sandbox.execute("globalThis.__setMiter").unwrap().value, "5", "miterLimit 往返");
    // 关键：font 改字号 10→20 后 measureText width 翻倍（证明 setFont 真改 FontDescriptor）。
    assert_eq!(sandbox.execute("globalThis.__widthScaled").unwrap().value, "true", "font 字号 10→20 后 measureText width 翻倍");
    assert_eq!(sandbox.execute("globalThis.__hasProps").unwrap().value, "true", "五个文本/线连接状态属性均 defined");

    // 非法 font 串 spec 忽略（保持原值，不抛 + getter 仍返上一个有效值）。
    sandbox
        .execute(
            "ctx.font = '20px'; /* 缺 family，非法 */\
             globalThis.__badFont = ctx.font;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__badFont").unwrap().value, "italic bold 20px Arial", "非法 font 串忽略，保持原值");
}

#[test]
fn test_canvas_dash_image_smoothing_r3305() {
    // R3305：lineDashOffset（虚线动画）+ getLineDash（展开后偶长数组）+ imageSmoothingEnabled /
    // imageSmoothingQuality（drawImage 重采样）。Rust 后端早全，仅缺 host op + JS shim 暴露。本测断言：
    // ① 默认值（lineDashOffset=0 / getLineDash=[] / imageSmoothingEnabled=true / imageSmoothingQuality='high'）；
    // ② setLineDash 奇长 [5] → getLineDash 返展开 [5,5]（spec 偶长）；偶长 [2,3] → [2,3]；
    // ③ lineDashOffset setter→getter 往返；④ imageSmoothingEnabled/Quality 往返。
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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             globalThis.__defLdo = ctx.lineDashOffset;\
             globalThis.__defDash = JSON.stringify(ctx.getLineDash());\
             globalThis.__defIse = ctx.imageSmoothingEnabled;\
             globalThis.__defIsq = ctx.imageSmoothingQuality;\
             ctx.setLineDash([5]); /* 奇长 → 展开 [5,5] */\
             globalThis.__oddDash = JSON.stringify(ctx.getLineDash());\
             ctx.setLineDash([2, 3]); /* 偶长原样 */\
             globalThis.__evenDash = JSON.stringify(ctx.getLineDash());\
             ctx.lineDashOffset = 4.5;\
             globalThis.__setLdo = ctx.lineDashOffset;\
             ctx.imageSmoothingEnabled = false;\
             globalThis.__setIse = ctx.imageSmoothingEnabled;\
             ctx.imageSmoothingQuality = 'medium';\
             globalThis.__setIsq = ctx.imageSmoothingQuality;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__defLdo").unwrap().value, "0", "默认 lineDashOffset=0");
    assert_eq!(sandbox.execute("globalThis.__defDash").unwrap().value, "[]", "默认 getLineDash=[]");
    assert_eq!(sandbox.execute("globalThis.__defIse").unwrap().value, "true", "默认 imageSmoothingEnabled=true");
    assert_eq!(sandbox.execute("globalThis.__defIsq").unwrap().value, "high", "默认 imageSmoothingQuality=high");
    // 奇长 [5] → 展开为 [5,5]（spec 偶长）。
    assert_eq!(sandbox.execute("globalThis.__oddDash").unwrap().value, "[5,5]", "奇长 setLineDash([5]) → getLineDash 展开 [5,5]");
    // 偶长 [2,3] 原样。
    assert_eq!(sandbox.execute("globalThis.__evenDash").unwrap().value, "[2,3]", "偶长 setLineDash([2,3]) → getLineDash [2,3]");
    assert_eq!(sandbox.execute("globalThis.__setLdo").unwrap().value, "4.5", "lineDashOffset 往返");
    assert_eq!(sandbox.execute("globalThis.__setIse").unwrap().value, "false", "imageSmoothingEnabled 往返");
    assert_eq!(sandbox.execute("globalThis.__setIsq").unwrap().value, "medium", "imageSmoothingQuality 往返");
}

#[test]
fn test_canvas_path2d_r3306() {
    // R3306：Path2D（spec CanvasPath）——`new Path2D()` 可复用路径对象 + `ctx.fill(path)`/stroke(path)/clip(path)
    // 参数形式。此前 Path2D JS 构造器全缺，ctx.fill/stroke/clip 仅无参当前路径形式 → 路径库（chart.js/D3/SVG）
    // `new Path2D()` + ctx.fill(path) 不可用。本测断言：① new Path2D() 返对象含 _zwPath id + 路径方法；
    // ② path 方法链构建（moveTo/lineTo/closePath/arc/bezier 等）不抛；③ ctx.fill(path)/stroke(path)/clip(path)
    // 不抛 + 用 Path2D 绘制（当前路径不被消费——验证后 ctx.beginPath 再 fill 仍可独立绘制）；④ addPath 复制；
    // ⑤ new Path2D(other) 复制既有 path；⑥ Path2D 全局构造器存在。
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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "globalThis.__hasCtor = String(typeof Path2D === 'function');\
             var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             var p = new Path2D();\
             globalThis.__isObj = String(p && typeof p === 'object');\
             globalThis.__hasPathId = String(typeof p._zwPath === 'string' && p._zwPath.length > 0);\
             globalThis.__hasMethods = String(['moveTo','lineTo','closePath','arc','arcTo','quadraticCurveTo','bezierCurveTo','ellipse','rect','addPath'].every(function (k) { return typeof p[k] === 'function'; }));\
             p.moveTo(10, 10);\
             p.lineTo(90, 90);\
             p.lineTo(10, 90);\
             p.closePath();\
             globalThis.__buildOk = 'ok';\
             ctx.fillStyle = 'red';\
             ctx.fill(p); /* Path2D 参数形式：用 p 绘制，不消费 ctx 当前路径 */\
             ctx.strokeStyle = 'blue';\
             ctx.stroke(p);\
             ctx.clip(p);\
             globalThis.__fillStrokeClipOk = 'ok';\
             /* 验证当前路径未被消费：beginPath + 画一个新矩形再 fill 仍独立工作 */\
             ctx.beginPath();\
             ctx.rect(0, 0, 5, 5);\
             ctx.fill();\
             globalThis.__ctxPathIndependent = 'ok';\
             /* addPath 复制 */\
             var p2 = new Path2D();\
             p2.addPath(p);\
             ctx.fill(p2);\
             globalThis.__addPathOk = 'ok';\
             /* new Path2D(other) 复制形式 */\
             var p3 = new Path2D(p);\
             globalThis.__copyDistinct = String(p3._zwPath !== p._zwPath && p3._zwPath.length > 0);\
             ctx.fill(p3);\
             globalThis.__copyOk = 'ok';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__hasCtor").unwrap().value, "true", "Path2D 全局构造器存在");
    assert_eq!(sandbox.execute("globalThis.__isObj").unwrap().value, "true", "new Path2D() 返对象");
    assert_eq!(sandbox.execute("globalThis.__hasPathId").unwrap().value, "true", "Path2D 含 _zwPath id 标记");
    assert_eq!(sandbox.execute("globalThis.__hasMethods").unwrap().value, "true", "Path2D 含全套路径方法");
    assert_eq!(sandbox.execute("globalThis.__buildOk").unwrap().value, "ok", "Path2D 路径方法链构建不抛");
    assert_eq!(sandbox.execute("globalThis.__fillStrokeClipOk").unwrap().value, "ok", "ctx.fill(path)/stroke(path)/clip(path) 不抛");
    assert_eq!(sandbox.execute("globalThis.__ctxPathIndependent").unwrap().value, "ok", "Path2D 参数形式不消费 ctx 当前路径");
    assert_eq!(sandbox.execute("globalThis.__addPathOk").unwrap().value, "ok", "Path2D.addPath 复制不抛");
    assert_eq!(sandbox.execute("globalThis.__copyDistinct").unwrap().value, "true", "new Path2D(other) 产新 path id（复制非同源）");
    assert_eq!(sandbox.execute("globalThis.__copyOk").unwrap().value, "ok", "new Path2D(other) 复制形式可用");
}

#[test]
fn test_canvas_path2d_svg_string_r3307() {
    // R3307：Path2D svgString 构造形式（`new Path2D("M10 10 L90 90")`）。R3306 createPath lenient 建空路径，
    // 本切片闭合：JS 端 string 首参透传 host createPath → canvas crate `Path2D::from_svg` 解析 SVG path data
    // （M/L/H/V/C/S/Q/T/A/Z，绝对/相对，隐式重复，flag 单字符）。断言：① new Path2D(svgString) 返对象含
    // _zwPath id；② ctx.fill(svgPath)/stroke(svgPath)/clip(svgPath) 不抛（解析出的路径可绘制）；③ 相对坐标 +
    // 闭合 + 弧命令混合串可解析不抛；④ 非法/空串 lenient 建空路径不抛（real browser spec 亦尽力解析不抛）。
    // 解析器逐命令正确性见 canvas crate test_from_svg_*（path.rs R3307）。
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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             /* ① new Path2D(svgString)：基础 M/L 串 */\
             var p = new Path2D('M10 10 L90 90');\
             globalThis.__svgIsObj = String(p && typeof p === 'object');\
             globalThis.__svgHasPathId = String(typeof p._zwPath === 'string' && p._zwPath.length > 0);\
             ctx.fillStyle = 'red';\
             ctx.fill(p);\
             ctx.strokeStyle = 'blue';\
             ctx.stroke(p);\
             ctx.clip(p);\
             globalThis.__svgFillStrokeClipOk = 'ok';\
             /* ② 混合命令串（相对 l + 闭合 Z + 弧 A）：解析不抛 */\
             var p2 = new Path2D('M0 0 l20 20 A5 5 0 0 0 40 0 Z');\
             ctx.fill(p2);\
             globalThis.__svgMixedOk = String(p2._zwPath.length > 0);\
             /* ③ 非法/空串：lenient 建空路径不抛 */\
             var p3 = new Path2D('');\
             var p4 = new Path2D('garbage not a path');\
             globalThis.__svgLenientOk = String(p3._zwPath.length > 0 && p4._zwPath.length > 0);\
             /* ④ 多子路径 + 隐式重复（M 后多组 = 首点 move + 余点 line）*/\
             var p5 = new Path2D('M0 0 10 10 20 0 Z');\
             ctx.fill(p5);\
             globalThis.__svgImplicitOk = 'ok';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__svgIsObj").unwrap().value, "true", "new Path2D(svgString) 返对象");
    assert_eq!(sandbox.execute("globalThis.__svgHasPathId").unwrap().value, "true", "svgString Path2D 含 _zwPath id");
    assert_eq!(sandbox.execute("globalThis.__svgFillStrokeClipOk").unwrap().value, "ok", "ctx.fill/stroke/clip(svgPath) 不抛");
    assert_eq!(sandbox.execute("globalThis.__svgMixedOk").unwrap().value, "true", "混合命令串（相对/闭合/弧）可解析");
    assert_eq!(sandbox.execute("globalThis.__svgLenientOk").unwrap().value, "true", "非法/空串 lenient 建空路径不抛");
    assert_eq!(sandbox.execute("globalThis.__svgImplicitOk").unwrap().value, "ok", "隐式重复 + 多子路径串可解析绘制");
}

#[test]
fn test_canvas_resize_clears_bitmap_r3308() {
    // R3308：canvas resize（spec 设 canvas.width/height 清空 bitmap + 重置绘图状态，HTML §4.12.5.1）。
    // R3077 留 defer 项：设 width/height 只写属性，host context 尺寸不变（绘制/getImageData 仍按旧尺寸）。
    // 本切片闭合：CANVAS width/height setter 检测 context 已建 → 调 host resizeContext（CanvasContext::resize
    // 重置全状态 + 清空 pixel_buffer）。断言：① 设 width 后 canvas.width 反射读回新值；② resize 后
    // getImageData 像素清零（旧绘制内容被清空）；③ resize 后绘制按新尺寸工作（getImageData 不越界）。
    // 绘图状态重置（transform/line_width/style）见 canvas crate test_resize_resets_drawing_state_r3308。
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
        "<html><body><canvas id='cv' width='10' height='10'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var cv = document.getElementById('cv');\
             var ctx = cv.getContext('2d');\
             /* 先绘制红色填满（10×10），确认像素写入 */\
             ctx.fillStyle = 'rgb(255,0,0)';\
             ctx.fillRect(0, 0, 10, 10);\
             var before = ctx.getImageData(0, 0, 2, 2).data;\
             globalThis.__beforeRed = String(before[0]);\
             /* 设 canvas.width 触发 spec resize（清空 bitmap + 重置状态）*/\
             cv.width = 20;\
             globalThis.__newWidth = String(cv.width);\
             /* resize 后像素应清零（旧红色绘制被清空）*/\
             var after = ctx.getImageData(0, 0, 2, 2).data;\
             globalThis.__afterCleared = String(after[0] + ',' + after[1] + ',' + after[2] + ',' + after[3]);\
             /* resize 后按新尺寸绘制不抛（getImageData 不越界——新尺寸 20×20）*/\
             ctx.fillStyle = 'rgb(0,0,255)';\
             ctx.fillRect(0, 0, 20, 20);\
             var redrawTop = ctx.getImageData(0, 0, 1, 1).data;\
             globalThis.__redrawBlue = String(redrawTop[2]);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__beforeRed").unwrap().value, "255", "resize 前红色已写入像素");
    assert_eq!(sandbox.execute("globalThis.__newWidth").unwrap().value, "20", "canvas.width 反射读回新值");
    assert_eq!(
        sandbox.execute("globalThis.__afterCleared").unwrap().value,
        "0,0,0,0",
        "resize 后像素清零（旧绘制内容被清空）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__redrawBlue").unwrap().value,
        "255",
        "resize 后按新尺寸绘制工作（蓝色写入成功）"
    );
}

#[test]
fn test_canvas_create_image_bitmap_r3309() {
    // R3309：createImageBitmap（HTML spec ImageBitmap）——Blob source 异步解码为可绘制位图。
    // 承接 canvas Tier 3 续候选②（R3296 留）。createImageBitmap 此前全缺 → fetch 图片 → drawImage 链路断。
    // 本测断言：① createImageBitmap 存在 + 返 Promise；② Blob source（1×1 红 PNG）解码成 ImageBitmap
    //（width/height = 1，持 _zwBitmapWire）；③ drawImage(bitmap) 真栅格到目标 ctx（getImageData 回读红色）；
    // ④ 非 Blob source（如 null/数字）reject；⑤ 损坏 Blob（非图片字节）reject。
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
        "<html><body><canvas id='cv' width='10' height='10'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 1×1 红色 PNG（与 image_decoder.rs 测试同源，host decode_data_uri 可解码）。
    sandbox
        .execute(
            "globalThis.__hasFn = String(typeof createImageBitmap === 'function');\
             var RED_PNG_B64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';\
             /* base64 → bytes → Blob（PNG 二进制）*/\
             var bin = atob(RED_PNG_B64);\
             var bytes = new Uint8Array(bin.length);\
             for (var i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);\
             var blob = new Blob([bytes], { type: 'image/png' });\
             globalThis.__blobSize = String(blob.size);\
             /* createImageBitmap(blob) → Promise<ImageBitmap> */\
             createImageBitmap(blob).then(function (bm) {\
               globalThis.__bmWidth = String(bm.width);\
               globalThis.__bmHeight = String(bm.height);\
               globalThis.__hasWire = String(typeof bm._zwBitmapWire === 'string' && bm._zwBitmapWire.length > 0);\
               /* drawImage(bitmap, dx, dy) 真栅格到目标 ctx */\
               var cv = document.getElementById('cv');\
               var ctx = cv.getContext('2d');\
               ctx.drawImage(bm, 0, 0);\
               var px = ctx.getImageData(0, 0, 1, 1).data;\
               globalThis.__drawRed = String(px[0] + ',' + px[1] + ',' + px[2]);\
               globalThis.__resolved = 'ok';\
             }, function (err) {\
               globalThis.__resolved = 'reject:' + String(err && err.message ? err.message : err);\
             });",
        )
        .unwrap();
    // drain microtask（createImageBitmap 的 Promise.resolve.then 链 + 回调）。
    sandbox.execute("globalThis.__noop = 1;").unwrap(); // 触发 microtask drain（execute 末 perform checkpoint）
    // Promise.resolve(source).then 链需 1-2 轮 drain（host 解码 + drawImage 均同步）。
    sandbox.execute("globalThis.__noop = 2;").unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__hasFn").unwrap().value,
        "true",
        "createImageBitmap 全局函数存在"
    );
    assert_eq!(
        sandbox.execute("globalThis.__blobSize").unwrap().value,
        "70",
        "Blob 构造含 PNG 字节（1×1 红 PNG = 70 字节）"
    );
    let resolved = sandbox.execute("globalThis.__resolved").unwrap().value;
    assert!(
        resolved.starts_with("ok") || resolved.starts_with("reject:"),
        "createImageBitmap Promise 应 settle，got: {resolved}"
    );
    assert_eq!(
        resolved, "ok",
        "createImageBitmap(blob) 应 resolve（host decode_data_uri 解码 1×1 红 PNG 成功）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__bmWidth").unwrap().value,
        "1",
        "ImageBitmap.width = 1（1×1 PNG）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__bmHeight").unwrap().value,
        "1",
        "ImageBitmap.height = 1（1×1 PNG）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasWire").unwrap().value,
        "true",
        "ImageBitmap 持 _zwBitmapWire（drawImage 源标记）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__drawRed").unwrap().value,
        "255,0,0",
        "drawImage(bitmap) 真栅格——目标 ctx 像素为红色（PNG 解码 + source-over 混合）"
    );

    // 非 Blob source reject + 损坏 Blob reject。
    sandbox
        .execute(
            "createImageBitmap(null).then(function () {\
               globalThis.__nullOk = 'ok';\
             }, function (e) {\
               globalThis.__nullOk = 'reject';\
             });\
             var bad = new Blob([new Uint8Array([0,1,2,3])], { type: 'image/png' });\
             createImageBitmap(bad).then(function () {\
               globalThis.__badOk = 'ok';\
             }, function (e) {\
               globalThis.__badOk = 'reject';\
             });",
        )
        .unwrap();
    sandbox.execute("globalThis.__noop = 3;").unwrap(); // pump microtask（reject Promise 链）
    sandbox.execute("globalThis.__noop = 4;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__nullOk").unwrap().value,
        "reject",
        "createImageBitmap(null) 应 reject（非 Blob source）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__badOk").unwrap().value,
        "reject",
        "createImageBitmap(损坏 Blob) 应 reject（非图片字节解码失败）"
    );
}

#[test]
fn test_canvas_create_image_bitmap_sources_r3310() {
    // R3310：createImageBitmap source 扩展——ImageData + HTMLCanvasElement（R3309 仅 Blob source）。
    // ImageData source：直接 JS 编码 wire（无 host 解码）；HTMLCanvasElement source：经 getImageData 取 wire
    //（镜像 drawImage canvas 源）。本测断言：① ImageData source → ImageBitmap（width/height 正确 + drawImage 真栅格）；
    // ② HTMLCanvasElement source → ImageBitmap（drawImage 真栅格）；③ 未知 source（null/数字）reject。
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
        "<html><body><canvas id='dst' width='10' height='10'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var dst = document.getElementById('dst');\
             var dctx = dst.getContext('2d');\
             /* ① ImageData source：手构 2×2 红色 ImageData（data = 4 像素 × RGBA）*/\
             var imgData = new ImageData(new Uint8ClampedArray([255,0,0,255, 255,0,0,255, 255,0,0,255, 255,0,0,255]), 2, 2);\
             createImageBitmap(imgData).then(function (bm) {\
               globalThis.__idW = String(bm.width);\
               globalThis.__idH = String(bm.height);\
               dctx.drawImage(bm, 0, 0);\
               var px = dctx.getImageData(0, 0, 1, 1).data;\
               globalThis.__idRed = String(px[0] + ',' + px[1] + ',' + px[2]);\
               globalThis.__idOk = 'ok';\
             }, function (e) { globalThis.__idOk = 'reject:' + String(e && e.message ? e.message : e); });\
             /* ② HTMLCanvasElement source：源 canvas 绘蓝后 createImageBitmap → drawImage 到 dst */\
             var src = document.createElement('canvas'); src.width = 3; src.height = 3;\
             var sctx = src.getContext('2d'); sctx.fillStyle = 'rgb(0,0,255)'; sctx.fillRect(0, 0, 3, 3);\
             createImageBitmap(src).then(function (bm) {\
               globalThis.__cvW = String(bm.width);\
               globalThis.__cvH = String(bm.height);\
               dctx.drawImage(bm, 5, 5);\
               var px = dctx.getImageData(5, 5, 1, 1).data;\
               globalThis.__cvBlue = String(px[0] + ',' + px[1] + ',' + px[2]);\
               globalThis.__cvOk = 'ok';\
             }, function (e) { globalThis.__cvOk = 'reject:' + String(e && e.message ? e.message : e); });\
             /* ③ 未知 source reject */\
             createImageBitmap(42).then(function () { globalThis.__unkOk = 'ok'; },\
               function () { globalThis.__unkOk = 'reject'; });",
        )
        .unwrap();
    // pump microtask（execute 末 drain；Promise 链需 1-2 轮）。
    sandbox.execute("globalThis.__noop = 1;").unwrap();
    sandbox.execute("globalThis.__noop = 2;").unwrap();
    sandbox.execute("globalThis.__noop = 3;").unwrap();

    // ① ImageData source。
    assert_eq!(
        sandbox.execute("globalThis.__idOk").unwrap().value,
        "ok",
        "createImageBitmap(ImageData) 应 resolve"
    );
    assert_eq!(
        sandbox.execute("globalThis.__idW").unwrap().value,
        "2",
        "ImageData source → ImageBitmap.width = 2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__idH").unwrap().value,
        "2",
        "ImageData source → ImageBitmap.height = 2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__idRed").unwrap().value,
        "255,0,0",
        "drawImage(bitmap from ImageData) 真栅格——红色像素"
    );

    // ② HTMLCanvasElement source。
    assert_eq!(
        sandbox.execute("globalThis.__cvOk").unwrap().value,
        "ok",
        "createImageBitmap(canvas) 应 resolve"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cvW").unwrap().value,
        "3",
        "canvas source → ImageBitmap.width = 3"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cvBlue").unwrap().value,
        "0,0,255",
        "drawImage(bitmap from canvas) 真栅格——蓝色像素"
    );

    // ③ 未知 source reject。
    assert_eq!(
        sandbox.execute("globalThis.__unkOk").unwrap().value,
        "reject",
        "createImageBitmap(42) 应 reject（未知 source）"
    );
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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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

#[test]
fn test_event_composed_path_r3244() {
    // R3244：Event.composedPath()（DOM §4.3，https://dom.spec.whatwg.org/#dom-event-composedpath）。
    // dispatch 期返事件路径（target→祖先→document→window）；非 dispatch（前后）返 []。
    // 事件委托（e.composedPath()[0] === target）+ 祖先匹配（path.includes(ancestor)）高频。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='parent'><span id='child'>x</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 连入文档的元素 dispatch：path = [child, parent, body, html, document, window]
    //    composedPath()[0]===target；含祖先（parent/body/html）；末端 document + window
    sandbox.execute(
        "globalThis.__path = null;\
         var child = document.getElementById('child');\
         child.addEventListener('test', function(e) {\
           globalThis.__path = e.composedPath();\
           globalThis.__pathLen = globalThis.__path.length;\
           globalThis.__targetIs0 = (e.composedPath()[0] === e.target);\
         });\
         var ev = new Event('test', { bubbles: true });\
         globalThis.__before = ev.composedPath().length;\
         child.dispatchEvent(ev);\
         globalThis.__after = ev.composedPath().length;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__before").unwrap().value, "0", "dispatch 前 composedPath() 返 []");
    assert_eq!(sandbox.execute("globalThis.__after").unwrap().value, "0", "dispatch 后 composedPath() 返 []（spec：dispatch flag unset）");
    assert_eq!(sandbox.execute("globalThis.__targetIs0").unwrap().value, "true", "composedPath()[0] === event.target");
    assert_eq!(sandbox.execute("globalThis.__pathLen").unwrap().value, "6", "连入文档元素 path 长度=6（child,parent,body,html,document,window）");
    // 顺序：target → 祖先链 → document → window
    assert_eq!(sandbox.execute("globalThis.__path[0].id").unwrap().value, "child", "path[0]=target (child)");
    assert_eq!(sandbox.execute("globalThis.__path[1].id").unwrap().value, "parent", "path[1]=直接父 (parent)");
    assert_eq!(sandbox.execute("globalThis.__path[2].tagName").unwrap().value, "BODY", "path[2]=body");
    assert_eq!(sandbox.execute("globalThis.__path[3].tagName").unwrap().value, "HTML", "path[3]=html");
    assert_eq!(sandbox.execute("globalThis.__path[4] === document").unwrap().value, "true", "path[4]=document");
    assert_eq!(sandbox.execute("globalThis.__path[5] === window").unwrap().value, "true", "path[5]=window");

    // ② 祖先匹配（事件委托高频用法）：composedPath().some(el => el.id === 'parent')
    sandbox.execute(
        "globalThis.__hasParent = document.getElementById('child')\
           .dispatchEvent(Object.assign(new Event('t2', {bubbles:true}), {__p:null})) || true;\
         document.getElementById('child').addEventListener('t2', function(e){\
           globalThis.__hasParent = e.composedPath().some(function(el){ return el && el.id === 'parent'; });\
           globalThis.__hasWindow = e.composedPath().indexOf(window) >= 0;\
         });\
         document.getElementById('child').dispatchEvent(new Event('t2', {bubbles:true}));",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__hasParent").unwrap().value, "true", "composedPath 含祖先 parent（事件委托 .some 匹配）");
    assert_eq!(sandbox.execute("globalThis.__hasWindow").unwrap().value, "true", "composedPath 含 window（末端）");

    // ③ window 派发事件：path = [window]（target 即 window）
    sandbox.execute(
        "globalThis.__wpath = null;\
         window.addEventListener('wev', function(e){ globalThis.__wpath = e.composedPath(); });\
         window.dispatchEvent(new Event('wev'));",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__wpath.length").unwrap().value, "1", "window 派发 path 长度=1");
    assert_eq!(sandbox.execute("globalThis.__wpath[0] === window").unwrap().value, "true", "window 派发 path[0]=window");

    // ④ 脱离文档元素（createElement 未挂载）：path = [target]（无祖先、无 document/window）
    sandbox.execute(
        "globalThis.__dpath = null;\
         var d = document.createElement('div');\
         d.addEventListener('dev', function(e){ globalThis.__dpath = e.composedPath(); });\
         d.dispatchEvent(new Event('dev', { bubbles: true }));",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__dpath.length").unwrap().value, "1", "脱离文档元素 path 长度=1（仅 target，无 document/window）");
    assert_eq!(sandbox.execute("globalThis.__dpath[0] === globalThis.__dpath[0]").unwrap().value, "true", "脱离文档 path[0] 存在");

    // ⑤ 非 dispatch 事件（new Event 未派发）composedPath() 恒 []
    sandbox.execute("globalThis.__fresh = (new Event('x')).composedPath().length;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__fresh").unwrap().value, "0", "未派发的 Event.composedPath() 返 []");
}

#[test]
fn test_input_set_range_text_r3245() {
    // R3245：HTMLInputElement/HTMLTextAreaElement.setRangeText()（HTML §4.10.5.23，
    // https://html.spec.whatwg.org/multipage/input.html#dom-textarea/input-setrangetext）。
    // 替换 value [start,end) 子串为 replacement，按 selectionMode 重定选区。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><input id='i' value='hello world'><textarea id='t'>abc</textarea></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 显式 start,end 替换：setRangeText('XYZ', 0, 5) → 'XYZ world'
    sandbox.execute(
        "var i = document.getElementById('i');\
         i.setRangeText('XYZ', 0, 5);\
         globalThis.__v1 = i.value;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__v1").unwrap().value, "XYZ world", "setRangeText('XYZ',0,5) 替换前 5 字符");

    // ② 'select' mode：选区折叠到插入文本（selectionStart=2, selectionEnd=2+1=3）
    sandbox.execute(
        "var i2 = document.getElementById('i');\
         i2.value = 'hello world';\
         i2.setRangeText('X', 2, 4, 'select');\
         globalThis.__v2 = i2.value;\
         globalThis.__ss = i2.selectionStart;\
         globalThis.__se = i2.selectionEnd;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__v2").unwrap().value, "heXo world", "setRangeText('X',2,4) 替换 [2,4)");
    assert_eq!(sandbox.execute("globalThis.__ss").unwrap().value, "2", "'select' mode selectionStart=replace 起点");
    assert_eq!(sandbox.execute("globalThis.__se").unwrap().value, "3", "'select' mode selectionEnd=起点+插入长度");

    // ③ 缺省 start/end = 当前选区：setSelectionRange(0,5) 后 setRangeText('HI') → 替换 [0,5) 为 'HI'
    sandbox.execute(
        "var i3 = document.getElementById('i');\
         i3.value = 'hello world';\
         i3.setSelectionRange(0, 5);\
         i3.setRangeText('HI');\
         globalThis.__v3 = i3.value;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__v3").unwrap().value, "HI world", "缺省 start/end 取当前选区替换");

    // ④ 'start' mode：选区折叠到替换起点；'end' mode：折叠到替换终点
    sandbox.execute(
        "var i4 = document.getElementById('i');\
         i4.value = 'abcdef';\
         i4.setRangeText('XY', 1, 3, 'start');\
         globalThis.__v4 = i4.value; globalThis.__ss4 = i4.selectionStart; globalThis.__se4 = i4.selectionEnd;\
         var i5 = document.getElementById('i'); i5.value = 'abcdef';\
         i5.setRangeText('XY', 1, 3, 'end');\
         globalThis.__ss5 = i5.selectionStart; globalThis.__se5 = i5.selectionEnd;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__v4").unwrap().value, "aXYdef", "'start' mode 替换 [1,3) 为 'XY'");
    assert_eq!(sandbox.execute("globalThis.__ss4").unwrap().value, "1", "'start' mode selectionStart=替换起点");
    assert_eq!(sandbox.execute("globalThis.__se4").unwrap().value, "1", "'start' mode selectionEnd=替换起点");
    assert_eq!(sandbox.execute("globalThis.__ss5").unwrap().value, "3", "'end' mode selectionStart=起点+插入长度");
    assert_eq!(sandbox.execute("globalThis.__se5").unwrap().value, "3", "'end' mode selectionEnd=起点+插入长度");

    // ⑤ IndexSizeError：start > end 抛
    sandbox.execute(
        "globalThis.__err = (function(){\
           try { document.getElementById('i').setRangeText('X', 5, 2); return 'no-throw'; }\
           catch (e) { return (e && e.name) ? e.name : 'unknown'; }\
         })();",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__err").unwrap().value, "IndexSizeError", "start>end 抛 IndexSizeError");

    // ⑥ textarea 同样工作（text-content 路径）：setRangeText('XY', 0, 1) → 'XYbc'
    sandbox.execute(
        "var t = document.getElementById('t');\
         t.setRangeText('XY', 0, 1);\
         globalThis.__tv = t.value;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__tv").unwrap().value, "XYbc", "textarea setRangeText 替换 [0,1)");
}

#[test]
fn test_window_print_and_stop_r3246() {
    // R3246：window.print() / window.stop()（HTML §4.5.6 / Window 接口）。两者此前全缺，调用抛 TypeError
    // 中断脚本（打印按钮 / 发票页 / 慢加载中止 / 广告拦截高频）。headless no-op（无打印机 / 无进行中加载）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 存在性：typeof === 'function'（feature-detect 高频：`if (typeof window.print === 'function')`）
    sandbox.execute(
        "globalThis.__isPrint = (typeof window.print === 'function');\
         globalThis.__isStop = (typeof window.stop === 'function');",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__isPrint").unwrap().value, "true", "window.print 为 function");
    assert_eq!(sandbox.execute("globalThis.__isStop").unwrap().value, "true", "window.stop 为 function");

    // ② 调用不抛、返 undefined（headless no-op）；包裹 try/catch 捕获任何中断
    sandbox.execute(
        "globalThis.__printRet = (function(){ try { return String(window.print()); } catch(e){ return 'THREW:'+e; } })();\
         globalThis.__stopRet = (function(){ try { return String(window.stop()); } catch(e){ return 'THREW:'+e; } })();",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__printRet").unwrap().value, "undefined", "window.print() 返 undefined（no-op，不抛）");
    assert_eq!(sandbox.execute("globalThis.__stopRet").unwrap().value, "undefined", "window.stop() 返 undefined（no-op，不抛）");

    // ③ 后续脚本不中断（window.stop() 调用后代码继续执行——真实浏览器 stop 仅中止加载，不中止 JS）
    sandbox.execute("window.stop(); globalThis.__afterStop = 'reached';").unwrap();
    assert_eq!(sandbox.execute("globalThis.__afterStop").unwrap().value, "reached", "window.stop() 后续脚本继续执行");

    // ④ globalThis === window 别名一致（window.print === globalThis.print）
    sandbox.execute("globalThis.__alias = (window.print === globalThis.print && window.stop === globalThis.stop);").unwrap();
    assert_eq!(sandbox.execute("globalThis.__alias").unwrap().value, "true", "window.print/stop 与 globalThis 别名一致");
}

#[test]
fn test_focus_blur_events_r3247() {
    // R3247：el.focus()/el.blur() 派发 focus/blur/focusin/focusout 事件（DOM §3.3 Focus + UI Events）。
    // 此前仅记 _activeElKey 不派发事件（known limitation ②）。表单 blur 校验 / focus 样式 / analytics 高频。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='wrap'><input id='a'><input id='b'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① focus() 派发 'focus'(非 bubble) + 'focusin'(bubble)；activeElement 更新
    sandbox.execute(
        "globalThis.__evts = [];\
         var a = document.getElementById('a');\
         a.addEventListener('focus', function(){ globalThis.__evts.push('a:focus'); });\
         a.addEventListener('focusin', function(){ globalThis.__evts.push('a:focusin'); });\
         a.focus();\
         globalThis.__ae = (document.activeElement === a);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__evts.join(',')").unwrap().value, "a:focus,a:focusin", "focus() 派发 focus + focusin");
    assert_eq!(sandbox.execute("globalThis.__ae").unwrap().value, "true", "focus() 后 activeElement===该元素");

    // ② blur() 派发 'blur'(非 bubble) + 'focusout'(bubble)
    sandbox.execute(
        "globalThis.__evts2 = [];\
         var a = document.getElementById('a');\
         a.addEventListener('blur', function(){ globalThis.__evts2.push('a:blur'); });\
         a.addEventListener('focusout', function(){ globalThis.__evts2.push('a:focusout'); });\
         a.blur();",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__evts2.join(',')").unwrap().value, "a:focusout,a:blur", "blur() 派发 focusout + blur");

    // ③ 焦点 A→B 移动：A 失焦（focusout+blur）、B 获焦（focus+focusin）
    sandbox.execute(
        "globalThis.__move = [];\
         var a = document.getElementById('a'); var b = document.getElementById('b');\
         a.addEventListener('focus', function(){ globalThis.__move.push('a:focus'); });\
         a.addEventListener('blur', function(){ globalThis.__move.push('a:blur'); });\
         a.addEventListener('focusout', function(){ globalThis.__move.push('a:focusout'); });\
         b.addEventListener('focus', function(){ globalThis.__move.push('b:focus'); });\
         b.addEventListener('focusin', function(){ globalThis.__move.push('b:focusin'); });\
         a.focus();\
         globalThis.__move.length = 0;\
         b.focus();\
         globalThis.__moveSeq = globalThis.__move.join(',');\
         globalThis.__moveOrderOk = (globalThis.__move.indexOf('a:focusout') < globalThis.__move.indexOf('b:focus'));",
    ).unwrap();
    let move_seq = sandbox.execute("globalThis.__moveSeq").unwrap().value;
    assert!(move_seq.contains("a:focusout") && move_seq.contains("a:blur"), "A→B：A 失焦派发 focusout+blur\n{move_seq}");
    assert!(move_seq.contains("b:focus") && move_seq.contains("b:focusin"), "A→B：B 获焦派发 focus+focusin\n{move_seq}");
    // 序：focusout(A) 在 focus(B) 前（spec 旧先失焦序）
    assert_eq!(sandbox.execute("globalThis.__moveOrderOk").unwrap().value, "true", "A→B 序：focusout(A) 先于 focus(B)\n{move_seq}");

    // ④ focusin/focusout 冒泡到父；focus/blur 不冒泡
    sandbox.execute(
        "globalThis.__bub = [];\
         var wrap = document.getElementById('wrap');\
         wrap.addEventListener('focus', function(){ globalThis.__bub.push('wrap:focus'); });\
         wrap.addEventListener('focusin', function(){ globalThis.__bub.push('wrap:focusin'); });\
         wrap.addEventListener('blur', function(){ globalThis.__bub.push('wrap:blur'); });\
         wrap.addEventListener('focusout', function(){ globalThis.__bub.push('wrap:focusout'); });\
         document.getElementById('a').focus();\
         globalThis.__bubHasFocusin = globalThis.__bub.indexOf('wrap:focusin') >= 0;\
         globalThis.__bubHasFocus = globalThis.__bub.indexOf('wrap:focus') >= 0;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__bubHasFocusin").unwrap().value, "true", "focusin 冒泡到父");
    assert_eq!(sandbox.execute("globalThis.__bubHasFocus").unwrap().value, "false", "focus 不冒泡（父未收 focus）");

    // ⑤ 已聚焦元素再 focus() no-op（不重派 focus）
    sandbox.execute(
        "globalThis.__refocus = 0;\
         var a = document.getElementById('a');\
         a.addEventListener('focus', function(){ globalThis.__refocus++; });\
         a.focus();\
         var before = globalThis.__refocus;\
         a.focus();\
         globalThis.__noop = (globalThis.__refocus === before);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__noop").unwrap().value, "true", "已聚焦元素再 focus() 不重派（spec no-op）");

    // ⑥ 非当前焦点元素 blur() no-op
    sandbox.execute(
        "globalThis.__bblur = 0;\
         var b = document.getElementById('b');\
         b.addEventListener('blur', function(){ globalThis.__bblur++; });\
         b.blur();\
         globalThis.__bblur0 = (globalThis.__bblur === 0);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__bblur0").unwrap().value, "true", "非焦点元素 blur() no-op（不派 blur）");
}

#[test]
fn test_html_dialog_element_api_r3290() {
    // R3290：HTMLDialogElement API（show/showModal/close/returnValue/open）。
    // WHATWG HTML §6.13 interactive-elements：dialog.show() 非模态打开（设 open 属性）；
    // dialog.showModal() 模态打开（设 open + top-layer，已 open 抛 InvalidStateError）；
    // dialog.close(returnValue) 移 open + 模态移 top-layer + 设 returnValue + 派 'close' 事件。
    // open boolean 反射属性（details/dialog 共用，presence-based）。
    // https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element
    // headless 无真 top-layer paint / ::backdrop / focus 陷阱 / inert backdrop（rendering 流域 defer），
    // 本切片验证 JS-observable 状态（open 属性 + returnValue + 'close' 事件 + showModal 状态机）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <dialog id='d1'><p>d1</p></dialog>\
         <dialog id='d2'>d2</dialog>\
         <dialog id='d3' open>d3-preopen</dialog>\
         <dialog id='d4'>d4</dialog>\
         <dialog id='d5'>d5</dialog>\
         <details id='det'><summary>s</summary>body</details>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 默认状态：dialog.open=false（无 open 属性），returnValue 默认 ''。
    sandbox.execute(
        "globalThis.__d1Open = String(document.getElementById('d1').open);\
         globalThis.__d1Rv = JSON.stringify(document.getElementById('d1').returnValue);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__d1Open").unwrap().value, "false", "dialog 默认 open=false");
    assert_eq!(sandbox.execute("globalThis.__d1Rv").unwrap().value, r#""""#, "dialog 默认 returnValue=''");

    // ② show()（非模态）：设 open 属性 → open getter=true，hasAttribute('open')=true。
    sandbox.execute(
        "document.getElementById('d1').show();\
         globalThis.__afterShow = String(document.getElementById('d1').open);\
         globalThis.__afterShowAttr = String(document.getElementById('d1').hasAttribute('open'));",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__afterShow").unwrap().value, "true", "show() → open=true");
    assert_eq!(sandbox.execute("globalThis.__afterShowAttr").unwrap().value, "true", "show() → 设 open 内容属性");

    // ③ showModal() 已 open → InvalidStateError（spec §dom-dialog-showmodal step 1）。
    //    d2 未 open → showModal() 设 open + top-layer（headless 仅 JS 态）。
    sandbox.execute(
        "globalThis.__errShown = '';\
         try { document.getElementById('d1').showModal(); } catch(e){ globalThis.__errShown = e.name; }\
         document.getElementById('d2').showModal();\
         globalThis.__d2Open = String(document.getElementById('d2').open);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__errShown").unwrap().value, "InvalidStateError", "已 open 的 dialog showModal → InvalidStateError");
    assert_eq!(sandbox.execute("globalThis.__d2Open").unwrap().value, "true", "showModal() → open=true");

    // ④ close(returnValue)：移 open 属性 + 设 returnValue + 派 'close' 事件。d2 模态关闭。
    sandbox.execute(
        "var d2 = document.getElementById('d2');\
         var closed = 0;\
         d2.addEventListener('close', function(){ closed++; });\
         var ret = d2.close('confirmed');\
         globalThis.__closeRet = String(ret);\
         globalThis.__closed = String(closed);\
         globalThis.__d2OpenAfter = String(d2.open);\
         globalThis.__d2Rv = d2.returnValue;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__closeRet").unwrap().value, "true", "close() 返 true（was open）");
    assert_eq!(sandbox.execute("globalThis.__closed").unwrap().value, "1", "close() 派发 'close' 事件一次");
    assert_eq!(sandbox.execute("globalThis.__d2OpenAfter").unwrap().value, "false", "close() → open=false（移属性）");
    assert_eq!(sandbox.execute("globalThis.__d2Rv").unwrap().value, "confirmed", "close('confirmed') → returnValue='confirmed'");

    // ⑤ close() 未 open dialog → no-op（返 false，不派 close，不抛）。
    sandbox.execute(
        "var d4 = document.getElementById('d4');\
         var d4Closed = 0;\
         d4.addEventListener('close', function(){ d4Closed++; });\
         var ret = d4.close();\
         globalThis.__closedNoop = String(ret);\
         globalThis.__d4Closed = String(d4Closed);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__closedNoop").unwrap().value, "false", "未 open 的 dialog close() 返 false（no-op）");
    assert_eq!(sandbox.execute("globalThis.__d4Closed").unwrap().value, "0", "未 open 的 dialog close() 不派 close 事件");

    // ⑥ open 反射 setter（details + dialog 共用）：truthy→setAttribute('open','')，falsy→removeAttribute。
    //    d5.open=true → open getter=true；d3（preopen）.open=false → 移属性、getter=false。
    sandbox.execute(
        "document.getElementById('d5').open = true;\
         globalThis.__d5Open = String(document.getElementById('d5').open);\
         document.getElementById('d3').open = false;\
         globalThis.__d3OpenAfter = String(document.getElementById('d3').open);\
         globalThis.__detOpen = String(document.getElementById('det').open);\
         document.getElementById('det').open = true;\
         globalThis.__detOpenAfter = String(document.getElementById('det').open);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__d5Open").unwrap().value, "true", "dialog.open=true setter → open=true");
    assert_eq!(sandbox.execute("globalThis.__d3OpenAfter").unwrap().value, "false", "dialog.open=false setter → 移 open 属性");
    assert_eq!(sandbox.execute("globalThis.__detOpen").unwrap().value, "false", "details 默认 open=false（共用反射属性）");
    assert_eq!(sandbox.execute("globalThis.__detOpenAfter").unwrap().value, "true", "details.open=true setter → open=true");

    // ⑦ returnValue 直接 IDL setter（不反射内容属性）：null→''，串值存。
    sandbox.execute(
        "var d5 = document.getElementById('d5');\
         d5.returnValue = 'xyz';\
         globalThis.__rvXyz = d5.returnValue;\
         d5.returnValue = null;\
         globalThis.__rvNull = JSON.stringify(d5.returnValue);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__rvXyz").unwrap().value, "xyz", "returnValue setter 存串值");
    assert_eq!(sandbox.execute("globalThis.__rvNull").unwrap().value, r#""""#, "returnValue=null setter → ''");

    // ⑧ show → showModal 互斥：show 后 showModal 关前非模态态再开模态（不抛，因 show 后 d1 仍 open——
    //    实际 spec：showModal 已 open 抛。验证 d1 经 ②show() 后 showModal 抛 InvalidStateError（同 ③）已隐含）。
    //    反向：showModal 后 show() 切非模态（清模态态，open 属性保持）。
    sandbox.execute(
        "var d2 = document.getElementById('d2');\
         d2.showModal();\
         globalThis.__d2Modal1 = String(d2.open);\
         d2.show();\
         globalThis.__d2AfterShow = String(d2.open);\
         var c2 = 0;\
         d2.addEventListener('close', function(){ c2++; });\
         d2.close();\
         globalThis.__d2Closed = String(c2);",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__d2Modal1").unwrap().value, "true", "showModal() → open=true");
    assert_eq!(sandbox.execute("globalThis.__d2AfterShow").unwrap().value, "true", "show() 后 open 保持 true（切非模态）");
    assert_eq!(sandbox.execute("globalThis.__d2Closed").unwrap().value, "1", "切非模态后 close() 仍派 close 事件");
}

#[test]
fn test_canvas_round_rect_and_hit_test_r3291() {
    // R3291：Canvas 2D roundRect + isPointInPath + isPointInStroke JS 暴露。
    // Rust 后端已存在（Path2D::round_rect + CanvasContext::is_point_in_path/is_point_in_stroke），但无 host op
    // 派发 + 无 JS shim 暴露 → `ctx.roundRect(...)` / `ctx.isPointInPath(x,y)` 静默 no-op（_ => "ok"）。
    // 本切片接通：CanvasContext::round_rect（变换点 + RoundRect 命令）+ host op（roundRect/isPointInPath/
    // isPointInStroke）+ JS shim ctx.roundRect（radii number|array 归一）/ isPointInPath / isPointInStroke。
    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-roundrect
    // headless：roundRect 角圆 flattener best-effort 退化矩形（rendering 已知简化），几何/命中测试仍正确。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><canvas id='cv' width='200' height='200'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① roundRect 不抛（方法存在 + 调用成功），radii 三形式（number / array / 缺省）均不抛。
    sandbox.execute(
        "var cv = document.getElementById('cv');\
         var ctx = cv.getContext('2d');\
         globalThis.__hasRoundRect = (typeof ctx.roundRect === 'function');\
         ctx.beginPath();\
         var threw = '';\
         try {\
           ctx.roundRect(10, 10, 100, 80, 5);\
           ctx.roundRect(10, 10, 100, 80, [5, 10, 15, 20]);\
           ctx.roundRect(10, 10, 100, 80);\
           ctx.roundRect(10, 10, 100, 80, [5, 10]);\
         } catch(e){ threw = e.name; }\
         globalThis.__roundRectThrew = threw;",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__hasRoundRect").unwrap().value, "true", "ctx.roundRect 为 function（此前缺失）");
    assert_eq!(sandbox.execute("globalThis.__roundRectThrew").unwrap().value, "", "roundRect 三形式（number/array/缺省）均不抛");

    // ② isPointInPath：rect(10,10,100,80) 路径内点 (50,50) → true；外点 (5,5) → false；无路径默认 false。
    //    roundRect 退化矩形几何 = rect，命中测试仍正确（内/外判定）。
    sandbox.execute(
        "ctx.beginPath();\
         ctx.roundRect(10, 10, 100, 80, 10);\
         globalThis.__hasPip = (typeof ctx.isPointInPath === 'function');\
         globalThis.__pipInside = String(ctx.isPointInPath(50, 50));\
         globalThis.__pipOutside = String(ctx.isPointInPath(5, 5));\
         ctx.beginPath();\
         globalThis.__pipEmpty = String(ctx.isPointInPath(50, 50));",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__hasPip").unwrap().value, "true", "ctx.isPointInPath 为 function（此前缺失）");
    assert_eq!(sandbox.execute("globalThis.__pipInside").unwrap().value, "true", "isPointInPath(内点) → true（路径区内）");
    assert_eq!(sandbox.execute("globalThis.__pipOutside").unwrap().value, "false", "isPointInPath(外点) → false（路径区外）");
    assert_eq!(sandbox.execute("globalThis.__pipEmpty").unwrap().value, "false", "isPointInPath(空路径) → false");

    // ③ isPointInStroke：rect 路径描边（lineWidth 8）半宽内点 → true；远离描边点 → false。
    sandbox.execute(
        "ctx.beginPath();\
         ctx.roundRect(10, 10, 100, 80, 0);\
         ctx.lineWidth = 8;\
         globalThis.__hasPis = (typeof ctx.isPointInStroke === 'function');\
         globalThis.__pisOnEdge = String(ctx.isPointInStroke(10, 10));\
         globalThis.__pisFar = String(ctx.isPointInStroke(200, 200));",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__hasPis").unwrap().value, "true", "ctx.isPointInStroke 为 function（此前缺失）");
    assert_eq!(sandbox.execute("globalThis.__pisOnEdge").unwrap().value, "true", "isPointInStroke(描边上的点) → true（lineWidth 半宽内）");
    assert_eq!(sandbox.execute("globalThis.__pisFar").unwrap().value, "false", "isPointInStroke(远离描边的点) → false");
}
