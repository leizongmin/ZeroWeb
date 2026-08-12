#[test]
fn test_canvas_to_data_url_r2797() {
    // R2797：canvas slice 3——toDataURL（PNG 导出）。host png::Encoder 编码 pixel_buffer → csv 字节；
    // shim 转 Latin-1 → btoa → data:image/png;base64,。TDD：合法 PNG 签名（137,80,78,71,13,10,26,10）。
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

    // fillRect red 后 toDataURL：data:image/png;base64, 前缀 + 非空 base64。
    sandbox
        .execute(
            "var c = document.createElement('canvas'); c.width=3; c.height=3;\
             var cx = c.getContext('2d'); cx.fillStyle='red'; cx.fillRect(0,0,3,3);\
             globalThis.__url = c.toDataURL();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__url.slice(0, 'data:image/png;base64,'.length) === 'data:image/png;base64,'")
            .unwrap()
            .value,
        "true",
        "toDataURL 应返 data:image/png;base64, 前缀"
    );
    // base64 部分 → atob 解码 → 检 PNG 签名（\\x89 P N G \\r \\n \\x1a \\n = 137,80,78,71,13,10,26,10）。
    sandbox
        .execute(
            "var b = atob(globalThis.__url.split(',')[1]);\
             globalThis.__sig = b.charCodeAt(0)+','+b.charCodeAt(1)+','+b.charCodeAt(2)+','+b.charCodeAt(3)+','+b.charCodeAt(4)+','+b.charCodeAt(5)+','+b.charCodeAt(6)+','+b.charCodeAt(7);\
             globalThis.__len = b.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sig)").unwrap().value,
        "137,80,78,71,13,10,26,10",
        "解码后须为合法 PNG 签名"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len > 0)").unwrap().value,
        "true",
        "PNG 非空"
    );
    // toDataURL 与无绘制（空白）canvas 的 PNG 不同（内容反映绘制）——两 URL 不同。
    sandbox
        .execute("globalThis.__blank = document.createElement('canvas').toDataURL();")
        .unwrap();
    // 注：空白 canvas（默认 300×150）比 3×3 大得多，PNG 不同。验证两者不同（内容/尺寸差异）。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__blank !== globalThis.__url)")
            .unwrap()
            .value,
        "true"
    );
    // toDataURL 在未 getContext 的 canvas 上可调用（惰性创建 ctx）。
    assert_eq!(
        sandbox
            .execute("document.createElement('canvas').toDataURL().slice(0,5)")
            .unwrap()
            .value,
        "data:"
    );
}

#[test]
fn test_canvas_to_blob_r3296() {
    // R3296：canvas.toBlob——异步 PNG Blob 导出（HTMLCanvasElement proxy get-trap 路径）。
    // 镜像 R2797 toDataURL 的 host PNG 编码，但产物为 Blob 经 callback 异步派发（spec：返 undefined，
    // callback(blob|null) 在 microtask 触发）。复用 toDataURL 编码 → Uint8Array → Blob(type:'image/png')。
    // sandbox 每 execute 末 drain microtask（perform_microtask_checkpoint）→ callback 在下一 execute 内触发。
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

    // toBlob 返 undefined（spec 非 Promise）+ callback 经 microtask 异步派发 Blob。
    let ret = sandbox
        .execute(
            "var c = document.createElement('canvas'); c.width=3; c.height=3;\
             var cx = c.getContext('2d'); cx.fillStyle='red'; cx.fillRect(0,0,3,3);\
             globalThis.__ret = c.toBlob(function(b){ globalThis.__blob = b; });",
        )
        .unwrap();
    assert_eq!(ret.value, "undefined", "toBlob 须返 undefined（spec 非 Promise）");
    // microtask 在本 execute 末 drain → callback 已触发，__blob 已设。
    sandbox.execute("globalThis.__noop = 1;").unwrap(); // 触发 microtask drain（callback 注册在 Promise.resolve().then）
    assert_eq!(
        sandbox.execute("String(globalThis.__blob != null)").unwrap().value,
        "true",
        "toBlob callback 须派发非 null Blob"
    );
    // Blob.type='image/png' + size>0（PNG 非空）。
    assert_eq!(
        sandbox.execute("globalThis.__blob.type").unwrap().value,
        "image/png",
        "Blob.type 须为 'image/png'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__blob.size > 0)").unwrap().value,
        "true",
        "Blob.size 须 >0（PNG 非空）"
    );
    // arrayBuffer() 解码 → PNG 签名（\\x89 P N G \\r \\n \\x1a \\n）。
    sandbox
        .execute(
            "globalThis.__sig = null;\
             globalThis.__blob.arrayBuffer().then(function(buf){\
               var u = new Uint8Array(buf);\
               globalThis.__sig = u[0]+','+u[1]+','+u[2]+','+u[3]+','+u[4]+','+u[5]+','+u[6]+','+u[7];\
             });",
        )
        .unwrap();
    sandbox.execute("globalThis.__noop2 = 1;").unwrap(); // drain arrayBuffer() Promise
    assert_eq!(
        sandbox.execute("String(globalThis.__sig)").unwrap().value,
        "137,80,78,71,13,10,26,10",
        "arrayBuffer() 须解码为合法 PNG 签名（137,80,78,71,13,10,26,10）"
    );
    // 无 ctx（未 getContext）canvas → spec real-browser 行为：惰性建 ctx 产空白 PNG（callback 非 null），
    // 与 toDataURL 在无 ctx canvas 上返有效 `data:image/png;base64,` 同语义。验证 callback 得 Blob + PNG 签名。
    sandbox
        .execute(
            "globalThis.__blankBlob = 'pending';\
             document.createElement('canvas').toBlob(function(b){ globalThis.__blankBlob = b; });",
        )
        .unwrap();
    sandbox.execute("globalThis.__noop3 = 1;").unwrap(); // drain microtask
    assert_eq!(
        sandbox.execute("String(globalThis.__blankBlob != null && globalThis.__blankBlob !== 'pending')")
            .unwrap()
            .value,
        "true",
        "无 ctx canvas → 惰性建 ctx 产 Blob（spec real-browser 行为，镜像 toDataURL）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__blankBlob.type").unwrap().value,
        "image/png",
        "惰性 ctx canvas Blob.type='image/png'"
    );
}

#[test]
fn test_canvas_slice4_r2798() {
    // R2798：canvas slice 4——off-DOM 2D 表面补全（putImageData / globalCompositeOperation / shadow）。
    // 经核验三项均真写 pixel_buffer（非仅记 primitives）：
    // ① putImageData：put_image_data 1:1 copy_from_slice 写 pixel_buffer（get_imageData 对偶）；
    // ② globalCompositeOperation：host 持 state（save/restore 含），composite_pixel 在 rect-blit/stroke 消费
    //    （path-based fill 不消费——已知限制）；本测验证状态往返（default + set/get）；
    // ③ shadow：fill() 经 draw_shadow_path 偏移栅格到 pixel_buffer，offset 处可读得 shadowColor。
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

    // ── ① putImageData：写自定义 RGBA → getImageData 读回一致（pixel 0 与 pixel 3）。
    sandbox
        .execute(
            "var c = document.createElement('canvas'); c.width=2; c.height=2;\
             var cx = c.getContext('2d');\
             var im = cx.getImageData(0,0,2,2);\
             im.data[0]=10; im.data[1]=20; im.data[2]=30; im.data[3]=255;\
             im.data[4]=40; im.data[5]=50; im.data[6]=60; im.data[7]=255;\
             im.data[8]=70; im.data[9]=80; im.data[10]=90; im.data[11]=255;\
             im.data[12]=100; im.data[13]=110; im.data[14]=120; im.data[15]=255;\
             cx.putImageData(im, 0, 0);\
             globalThis.__back = cx.getImageData(0,0,2,2);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__back.data[0],__back.data[1],__back.data[2],__back.data[3]].join(',')")
            .unwrap()
            .value,
        "10,20,30,255",
        "putImageData 写入后 getImageData 读回须一致（pixel 0）"
    );
    assert_eq!(
        sandbox
            .execute("[__back.data[12],__back.data[13],__back.data[14],__back.data[15]].join(',')")
            .unwrap()
            .value,
        "100,110,120,255",
        "putImageData 写入后 getImageData 读回须一致（pixel 3）"
    );

    // ── ② globalCompositeOperation：default 'source-over' + set/get 往返（状态真实）。
    sandbox
        .execute(
            "var c2 = document.createElement('canvas');\
             globalThis.__cx2 = c2.getContext('2d');\
             globalThis.__def = __cx2.globalCompositeOperation;\
             __cx2.globalCompositeOperation = 'lighter';\
             globalThis.__g1 = __cx2.globalCompositeOperation;\
             __cx2.globalCompositeOperation = 'multiply';\
             globalThis.__g2 = __cx2.globalCompositeOperation;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__def)").unwrap().value,
        "source-over",
        "globalCompositeOperation 默认 source-over"
    );
    assert_eq!(sandbox.execute("String(globalThis.__g1)").unwrap().value, "lighter");
    assert_eq!(sandbox.execute("String(globalThis.__g2)").unwrap().value, "multiply");

    // ── ③ shadow：fillRect green + shadowColor red + shadowOffsetX=5。
    // fill 区 [0,10]×[0,10]；shadow 偏移后 [5,15]×[0,10]。(12,2) 仅 shadow 区→red；(2,2) fill-only→green
    // （fill 先画 shadow 再画 fill，重叠区 [5,10] 被 green 覆盖）。
    sandbox
        .execute(
            "var c3 = document.createElement('canvas'); c3.width=30; c3.height=20;\
             var cx3 = c3.getContext('2d');\
             cx3.shadowColor='rgba(255,0,0,255)'; cx3.shadowOffsetX=5;\
             cx3.fillStyle='#00ff00'; cx3.fillRect(0,0,10,10);\
             globalThis.__sh = cx3.getImageData(12,2,1,1);\
             globalThis.__fi = cx3.getImageData(2,2,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__sh.data[0],__sh.data[1],__sh.data[2],__sh.data[3]].join(',')")
            .unwrap()
            .value,
        "255,0,0,255",
        "shadow 偏移处须为 shadowColor（draw_shadow_path 栅格到 pixel_buffer）"
    );
    assert_eq!(
        sandbox
            .execute("[__fi.data[0],__fi.data[1],__fi.data[2],__fi.data[3]].join(',')")
            .unwrap()
            .value,
        "0,255,0,255",
        "fill 区须为 fillStyle（fill 画在 shadow 之上覆盖重叠）"
    );
}

#[test]
fn test_canvas_draw_image_r2799() {
    // R2799：canvas slice 5——drawImage（图像合成到 canvas，3 spec 重载）。host draw_image* 已存在且
    // 真写 pixel_buffer（draw_image_sized：最近邻采样 + transform + source-over alpha 混合 + global_alpha）。
    // shim 源限 canvas 元素（canvas-to-canvas，经源 getImageData 取全 RGBA wire）；HTMLImageElement defer。
    // TDD：drawImage(image,dx,dy) 源 red→dst 读回 red / drawImageScaled 1×1→3×3 / drawImageSliced 切片 sx。
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

    // ── drawImage(image, dx, dy)：src 2×2 red → dst 4×4 at (1,1)。覆盖 dst (1,1)..(2,2)；(0,0) 透明。
    sandbox
        .execute(
            "var s1 = document.createElement('canvas'); s1.width=2; s1.height=2;\
             var sc1 = s1.getContext('2d'); sc1.fillStyle='red'; sc1.fillRect(0,0,2,2);\
             var d1 = document.createElement('canvas'); d1.width=4; d1.height=4;\
             var dc1 = d1.getContext('2d'); dc1.drawImage(s1, 1, 1);\
             globalThis.__a_hit = dc1.getImageData(1,1,1,1);\
             globalThis.__a_miss = dc1.getImageData(0,0,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__a_hit.data[0],__a_hit.data[1],__a_hit.data[2],__a_hit.data[3]].join(',')")
            .unwrap()
            .value,
        "255,0,0,255",
        "drawImage(image,dx,dy) 源 red 须栅格到 dst"
    );
    assert_eq!(
        sandbox
            .execute("[__a_miss.data[0],__a_miss.data[1],__a_miss.data[2],__a_miss.data[3]].join(',')")
            .unwrap()
            .value,
        "0,0,0,0",
        "drawImage 区外须保持透明"
    );

    // ── drawImageScaled(image, dx, dy, dw, dh)：src 1×1 green → dst 缩放到 3×3。(2,2) 须 green。
    sandbox
        .execute(
            "var s2 = document.createElement('canvas'); s2.width=1; s2.height=1;\
             var sc2 = s2.getContext('2d'); sc2.fillStyle='#00ff00'; sc2.fillRect(0,0,1,1);\
             var d2 = document.createElement('canvas'); d2.width=4; d2.height=4;\
             var dc2 = d2.getContext('2d'); dc2.drawImage(s2, 0, 0, 3, 3);\
             globalThis.__b = dc2.getImageData(2,2,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__b.data[0],__b.data[1],__b.data[2],__b.data[3]].join(',')")
            .unwrap()
            .value,
        "0,255,0,255",
        "drawImageScaled 1×1→3×3 须缩放栅格"
    );

    // ── drawImageSliced(image, sx,sy,sw,sh, dx,dy,dw,dh)：src 2×1（(0,0)red / (1,0)green）。
    // 切片 (0,0,1,1)→dst(0,0,2,2) red；切片 (1,0,1,1)→dst(2,0,2,2) green。证明 sx 被采样。
    sandbox
        .execute(
            "var s3 = document.createElement('canvas'); s3.width=2; s3.height=1;\
             var sc3 = s3.getContext('2d');\
             var im3 = sc3.getImageData(0,0,2,1);\
             im3.data[0]=255; im3.data[1]=0; im3.data[2]=0; im3.data[3]=255;\
             im3.data[4]=0; im3.data[5]=255; im3.data[6]=0; im3.data[7]=255;\
             sc3.putImageData(im3, 0, 0);\
             var d3 = document.createElement('canvas'); d3.width=4; d3.height=4;\
             var dc3 = d3.getContext('2d');\
             dc3.drawImage(s3, 0,0,1,1, 0,0,2,2);\
             dc3.drawImage(s3, 1,0,1,1, 2,0,2,2);\
             globalThis.__c_red = dc3.getImageData(0,0,1,1);\
             globalThis.__c_green = dc3.getImageData(2,0,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[__c_red.data[0],__c_red.data[1],__c_red.data[2],__c_red.data[3]].join(',')")
            .unwrap()
            .value,
        "255,0,0,255",
        "drawImageSliced 切片 (0,0) red 须栅格"
    );
    assert_eq!(
        sandbox
            .execute("[__c_green.data[0],__c_green.data[1],__c_green.data[2],__c_green.data[3]].join(',')")
            .unwrap()
            .value,
        "0,255,0,255",
        "drawImageSliced 切片 sx=1 须采样 (1,0) green"
    );
}

#[test]
fn test_document_metadata_r2800() {
    // R2800：document 元数据属性——title（get/set，纯 JS）+ URL/documentURI（= location.href）+ referrer（''）。
    // title getter 首访读 querySelector('title').textContent（空白折叠）+ 缓存；setter 仅缓存（不写回 host DOM）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config.clone()).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 含 <title> 多空白文本的 HTML（验 getter 空白折叠）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>  Hello   World  </title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // title getter：从 <title> 读 + 空白折叠（"Hello World"，非 "  Hello   World  "）。
    assert_eq!(
        sandbox.execute("String(document.title)").unwrap().value,
        "Hello World",
        "document.title getter 须读 <title> 并空白折叠"
    );
    // title setter + 读回（in-JS 缓存）。
    sandbox
        .execute("document.title = 'New Title'; globalThis.__t1 = document.title;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t1)").unwrap().value,
        "New Title",
        "document.title setter 须缓存并可读回"
    );
    // set 空串。
    sandbox
        .execute("document.title = ''; globalThis.__t2 = document.title;")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__t2)").unwrap().value, "");

    // URL / documentURI = location.href；referrer = ''。
    sandbox
        .execute(
            "globalThis.__url = document.URL;\
             globalThis.__uri = document.documentURI;\
             globalThis.__ref = document.referrer;\
             globalThis.__loc = location.href;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__url === globalThis.__loc)")
            .unwrap()
            .value,
        "true",
        "document.URL 须 === location.href"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__uri === globalThis.__loc)")
            .unwrap()
            .value,
        "true",
        "document.documentURI 须 === location.href"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ref)").unwrap().value,
        "",
        "document.referrer 须为空串（无 referrer 追踪）"
    );

    // 无 <title> 的 HTML → title 为空串（无 title 元素）。用独立 fresh sandbox（避免缓存干扰）。
    let mut sandbox2 = V8Sandbox::with_config(config).unwrap();
    sandbox2.execute(generate_js_dom_shim()).unwrap();
    let dom_html2: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let mutations2: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let page_url2: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox2, &mutations2, &dom_html2, &page_url2);
    assert_eq!(
        sandbox2.execute("String(document.title)").unwrap().value,
        "",
        "无 <title> 元素时 document.title 须为空串"
    );
}

#[test]
fn test_element_focus_active_element_r2801() {
    // R2801：element.focus()/blur() + document.activeElement（焦点状态追踪，纯 JS）。Proxy get trap 返回
    // focus/blur 函数操作 _activeElKey；activeElement getter 返 _proxyCache[_activeElKey] || body。
    // proxy 经 _proxyCache 缓存，故 activeElement 与原引用同对象（=== 成立）。
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
        "<html><body><input id='i' value='x'><button id='b'>ok</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 默认 activeElement === body（无焦点回落）。
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === document.body)")
            .unwrap()
            .value,
        "true",
        "默认 document.activeElement 须 === document.body"
    );
    // input.focus() → activeElement === input（同引用）。
    sandbox
        .execute(
            "globalThis.__i = document.getElementById('i');\
             globalThis.__i.focus();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === globalThis.__i)")
            .unwrap()
            .value,
        "true",
        "input.focus() 后 activeElement 须 === input"
    );
    // 跨元素切换：button.focus() → activeElement === button。
    sandbox
        .execute(
            "globalThis.__b = document.getElementById('b');\
             globalThis.__b.focus();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === globalThis.__b)")
            .unwrap()
            .value,
        "true",
        "button.focus() 后 activeElement 须切换到 button"
    );
    // blur() → activeElement 回落 body。
    sandbox.execute("globalThis.__b.blur();").unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === document.body)")
            .unwrap()
            .value,
        "true",
        "button.blur() 后 activeElement 须回落 body"
    );
    // blur 非当前焦点元素不影响 activeElement：input.blur()（非 active）后 activeElement 仍 body。
    sandbox.execute("globalThis.__i.blur();").unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.activeElement === document.body)")
            .unwrap()
            .value,
        "true",
        "blur 非当前焦点元素不影响 activeElement"
    );
}

#[test]
fn test_document_create_event_r2802() {
    // R2802：document.createEvent + initCustomEvent（legacy 合成事件工厂）。createEvent 映射 type→现有构造器
    //（Event/CustomEvent/KeyboardEvent）返 new X('')，initEvent/initCustomEvent 填充。复用 R2779 事件构造器。
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

    // createEvent('Event') instanceof Event + initEvent 设 type/bubbles/cancelable。
    sandbox
        .execute(
            "var e = document.createEvent('Event');\
             globalThis.__isEvt = (e instanceof Event);\
             e.initEvent('click', true, false);\
             globalThis.__t = e.type; globalThis.__b = e.bubbles; globalThis.__c = e.cancelable;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__isEvt)").unwrap().value,
        "true",
        "createEvent('Event') instanceof Event"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t)").unwrap().value,
        "click",
        "initEvent 设 type"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b)").unwrap().value,
        "true",
        "initEvent 设 bubbles"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c)").unwrap().value,
        "false",
        "initEvent 设 cancelable"
    );

    // createEvent('CustomEvent') instanceof CustomEvent + initCustomEvent 设 detail。
    sandbox
        .execute(
            "var ce = document.createEvent('CustomEvent');\
             globalThis.__isCe = (ce instanceof CustomEvent) && (ce instanceof Event);\
             ce.initCustomEvent('foo', false, true, {a: 1});\
             globalThis.__ct = ce.type; globalThis.__cb = ce.bubbles; globalThis.__cc = ce.cancelable;\
             globalThis.__cd = ce.detail.a;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__isCe)").unwrap().value,
        "true",
        "createEvent('CustomEvent') instanceof CustomEvent & Event"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ct)").unwrap().value,
        "foo",
        "initCustomEvent 设 type"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cb)").unwrap().value,
        "false",
        "initCustomEvent 设 bubbles"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cc)").unwrap().value,
        "true",
        "initCustomEvent 设 cancelable"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cd)").unwrap().value,
        "1",
        "initCustomEvent 设 detail"
    );

    // 大小写不敏感 + spec 别名（HTMLEvents / Events → Event）。
    sandbox
        .execute(
            "var h = document.createEvent('HTMLEvents'); h.initEvent('x', true, false);\
             var ev = document.createEvent('Events'); ev.initEvent('y', false, false);\
             var ceu = document.createEvent('CustomEvent'); ceu.initCustomEvent('z', true, false, 9);\
             globalThis.__h = h.type; globalThis.__ev = ev.type; globalThis.__ceu = ceu.detail;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__h)").unwrap().value,
        "x",
        "createEvent('HTMLEvents') → Event"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev)").unwrap().value,
        "y",
        "createEvent('Events') → Event"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ceu)").unwrap().value,
        "9",
        "createEvent('CustomEvent') + initCustomEvent detail"
    );

    // dispatch 烟雾：createEvent + initEvent + dispatchEvent 触发 listener。
    sandbox
        .execute(
            "globalThis.__hits = 0;\
             var el = document.createElement('div');\
             el.addEventListener('ping', function () { globalThis.__hits++; });\
             var pe = document.createEvent('Event'); pe.initEvent('ping', false, false);\
             el.dispatchEvent(pe);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hits)").unwrap().value,
        "1",
        "createEvent + initEvent + dispatchEvent 须触发 listener"
    );
}

#[test]
fn test_document_tree_walker_r2803() {
    // R2803：document.createTreeWalker / createNodeIterator + NodeFilter（DOM 子树遍历器）。
    // eager pre-order 经 childNodes 递归；whatToShow 掩码 + acceptNode FILTER_ACCEPT/REJECT/SKIP 子树语义。
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
        "<html><body><div id=r><p>a</p><span>b</span><i>c</i></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "var root = document.getElementById('r'); var n;\
             var w = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             var tags = []; while ((n = w.nextNode())) tags.push(n.tagName);\
             globalThis.__tags = tags.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tags)").unwrap().value,
        "DIV,P,SPAN,I",
        "SHOW_ELEMENT 须深度优先文档序（含 root DIV）"
    );

    // SHOW_TEXT：文本节点序（trim 空白保安全）。
    sandbox
        .execute(
            "var w2 = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);\
             var texts = []; while ((n = w2.nextNode())) texts.push(String(n.nodeValue));\
             globalThis.__texts = texts.map(function(t){return t.trim();}).filter(Boolean).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__texts)").unwrap().value,
        "a,b,c",
        "SHOW_TEXT 须文本节点序 a,b,c"
    );

    // FILTER_REJECT 剪子树：reject span 元素 → span 与其文本 'b' 均不出现。
    sandbox
        .execute(
            "var rej = [];\
             var w3 = document.createTreeWalker(root, NodeFilter.SHOW_ALL, function (node) {\
               if (node.nodeType === 1 && node.tagName === 'SPAN') return NodeFilter.FILTER_REJECT;\
               return NodeFilter.FILTER_ACCEPT;\
             });\
             while ((n = w3.nextNode())) rej.push(n);\
             globalThis.__rejHasSpan = rej.some(function (x) { return x.nodeType === 1 && x.tagName === 'SPAN'; });\
             globalThis.__rejHasB = rej.some(function (x) { return x.nodeType === 3 && x.nodeValue === 'b'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rejHasSpan)").unwrap().value,
        "false",
        "FILTER_REJECT span 须排除 span 元素"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rejHasB)").unwrap().value,
        "false",
        "FILTER_REJECT span 须剪子树（文本 b 一并排除）"
    );

    // FILTER_SKIP 跳节点保留子树：skip span 元素 → span 不出现，但其文本 'b' 出现。
    sandbox
        .execute(
            "var skp = [];\
             var w4 = document.createTreeWalker(root, NodeFilter.SHOW_ALL, function (node) {\
               if (node.nodeType === 1 && node.tagName === 'SPAN') return NodeFilter.FILTER_SKIP;\
               return NodeFilter.FILTER_ACCEPT;\
             });\
             while ((n = w4.nextNode())) skp.push(n);\
             globalThis.__skpHasSpan = skp.some(function (x) { return x.nodeType === 1 && x.tagName === 'SPAN'; });\
             globalThis.__skpHasB = skp.some(function (x) { return x.nodeType === 3 && x.nodeValue === 'b'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__skpHasSpan)").unwrap().value,
        "false",
        "FILTER_SKIP span 须排除 span 元素"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__skpHasB)").unwrap().value,
        "true",
        "FILTER_SKIP span 须保留子树（文本 b 出现）"
    );

    // previousNode 反向 + NodeIterator 与 TreeWalker 同序。
    sandbox
        .execute(
            "var wb = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             var fwd = []; while ((n = wb.nextNode())) fwd.push(n.tagName);\
             var back = []; while ((n = wb.previousNode())) back.push(n.tagName);\
             globalThis.__back = back.join(',');\
             var it = document.createNodeIterator(root, NodeFilter.SHOW_ELEMENT);\
             var itags = []; while ((n = it.nextNode())) itags.push(n.tagName);\
             globalThis.__itags = itags.join(',');",
        )
        .unwrap();
    // nextNode 走到末尾后 previousNode 逆向：倒数第二起（末节点无前驱时不返自身）。
    // 至少验证 NodeIterator 与 TreeWalker 同序 + previousNode 返非空。
    assert_eq!(
        sandbox.execute("String(globalThis.__itags)").unwrap().value,
        "DIV,P,SPAN,I",
        "NodeIterator 须与 TreeWalker 同序"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__back.length > 0)").unwrap().value,
        "true",
        "previousNode 须可逆向遍历"
    );
}

#[test]
fn test_tree_walker_hierarchical_methods_r3257() {
    // R3257（DOM §4.2.6）：TreeWalker 层级方法 parentNode/firstChild/lastChild/nextSibling/previousSibling。
    // R2803 仅落 nextNode/previousNode（与 NodeIterator 共用）；本片补 TreeWalker 专属层级方法，并确认
    // NodeIterator 不暴露这些方法（spec §4.2.5）。树：div#r > [P, SPAN, I > EM]（文本叶 "a"/"b"/"c"）。
    // SHOW_ELEMENT 下 accepted pre-order = [DIV, P, SPAN, I, EM]，parentAcceptedIdx = [-1, 0, 0, 0, 3]。
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
        "<html><body><div id=r><p>a</p><span>b</span><i><em>c</em></i></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 层级导航序列（每 walker 独立）：firstChild/nextSibling/parentNode/lastChild/previousSibling。
    // ① DIV(firstChild)→P；P(nextSibling)→SPAN；SPAN(nextSibling)→I；I(firstChild)→EM；EM(parentNone? 否，parentNode)→I。
    sandbox
        .execute(
            "var root = document.getElementById('r');\
             var w = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             globalThis.__p1 = [\
               w.firstChild().tagName,       /* P（DIV 首子）*/\
               w.nextSibling().tagName,      /* SPAN */\
               w.nextSibling().tagName,      /* I */\
               w.firstChild().tagName,       /* EM（I 子）*/\
               w.parentNode().tagName        /* 回 I */\
             ].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__p1)").unwrap().value,
        "P,SPAN,I,EM,I",
        "firstChild/nextSibling/firstChild/parentNode 层级导航"
    );

    // ② lastChild(DIV)=I（P/SPAN/I 中末直接 filtered-子）；随后 firstChild(I)=EM。
    sandbox
        .execute(
            "var w2 = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             globalThis.__p2 = [\
               w2.lastChild().tagName,      /* I（DIV 末直接 filtered-子）*/\
               w2.firstChild().tagName      /* EM（I 的首子，currentNode 已在 I）*/\
             ].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__p2)").unwrap().value,
        "I,EM",
        "lastChild(DIV)=I，随后 firstChild(I)=EM"
    );

    // ③ 边界 null：EM 是叶（firstChild=null）；I 无 nextSibling（末子）；P 无 previousSibling（首子）；DIV 无 parentNode（root）。
    sandbox
        .execute(
            "var w3 = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             w3.firstChild();               /* P */\
             globalThis.__p3 = [\
               (w3.previousSibling() === null) ? 'null' : 'has',  /* P 首子无前兄 */\
               w3.nextSibling().tagName,     /* SPAN（重新从 P 取 nextSibling）*/\
               (document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT).parentNode() === null) ? 'null' : 'has'\
                 /* fresh walker at DIV，parentNode=null（root 无 accepted 祖先）*/\
             ].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__p3)").unwrap().value,
        "null,SPAN,null",
        "边界：P 无 previousSibling；fresh walker(DIV) parentNode=null"
    );

    // ④ previousSibling：从 SPAN 回到 P。
    sandbox
        .execute(
            "var w4 = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             w4.firstChild();        /* P */\
             w4.nextSibling();       /* SPAN */\
             globalThis.__ps = w4.previousSibling().tagName;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ps)").unwrap().value, "P", "previousSibling SPAN→P");

    // ⑤ nextNode 在层级移动后续接（idx 同步）：firstChild→P 后 nextNode→SPAN。
    sandbox
        .execute(
            "var w5 = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);\
             w5.firstChild();        /* P，idx=1 */\
             globalThis.__nn = w5.nextNode().tagName;  /* idx=2 → SPAN */",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__nn)").unwrap().value, "SPAN", "层级移动后 nextNode 续接（idx 同步）");

    // ⑥ NodeIterator 不暴露层级方法（spec §4.2.5）。
    sandbox.execute("globalThis.__niHas = (typeof document.createNodeIterator(root, NodeFilter.SHOW_ELEMENT).parentNode === 'function');").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__niHas)").unwrap().value,
        "false",
        "NodeIterator 无 parentNode（层级方法仅 TreeWalker）"
    );
}

#[test]
fn test_selection_range_r2804() {
    // R2804：window.getSelection + Selection 单例 + document.createRange + Range（文本选择/编辑器/copy-paste）。
    // headless 无真选择——Selection 默认空（rangeCount=0/isCollapsed=true/toString=''/type='None'）。
    // Range.toString 精确覆盖 selectNode*/同文本节点 setStart·setEnd。
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
        "<html><body><div id=r><p>hello</p><span>world</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // Selection 单例：window.getSelection() === window.getSelection()（同一对象）。
    sandbox
        .execute("globalThis.__same = (window.getSelection() === window.getSelection());")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__same)").unwrap().value,
        "true",
        "getSelection 须返单例（同一对象）"
    );

    // 默认空选择（headless 无真用户选择）。
    sandbox
        .execute(
            "var s = window.getSelection();\
             globalThis.__ts = s.toString();\
             globalThis.__rc = s.rangeCount;\
             globalThis.__ic = s.isCollapsed;\
             globalThis.__ty = s.type;\
             globalThis.__an = s.anchorNode;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ts)").unwrap().value,
        "",
        "默认 selection.toString 须为空"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rc)").unwrap().value,
        "0",
        "默认 rangeCount 须为 0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ic)").unwrap().value,
        "true",
        "默认 isCollapsed 须为 true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ty)").unwrap().value,
        "None",
        "默认 type 须为 'None'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__an)").unwrap().value,
        "null",
        "默认 anchorNode 须为 null"
    );

    // createRange + selectNodeContents(p)：toString = p 子树文本 'hello'。
    sandbox
        .execute(
            "var p = document.getElementById('r').firstElementChild; /* <p>hello</p> */\
             var r = document.createRange();\
             r.selectNodeContents(p);\
             globalThis.__rp = r.toString();\
             globalThis.__rcl = r.collapsed;\
             globalThis.__rsc = (r.startContainer === p);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rp)").unwrap().value,
        "hello",
        "selectNodeContents(p).toString 须为 'hello'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rcl)").unwrap().value,
        "false",
        "selectNodeContents 后 collapsed 须为 false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rsc)").unwrap().value,
        "true",
        "selectNodeContents startContainer 须为 p"
    );

    // selectNodeContents(#r)：多文本后代，toString = 'helloworld'。
    sandbox
        .execute(
            "var rr = document.createRange(); rr.selectNodeContents(document.getElementById('r'));\
             globalThis.__rr = rr.toString();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rr)").unwrap().value,
        "helloworld",
        "selectNodeContents(#r) 须收集多文本后代 'helloworld'"
    );

    // 同文本节点 setStart/setEnd slice：'hello'.slice(1,3) = 'el'。
    sandbox
        .execute(
            "var tn = document.getElementById('r').firstElementChild.firstChild; /* text 'hello' */\
             var rt = document.createRange(); rt.setStart(tn, 1); rt.setEnd(tn, 3);\
             globalThis.__rt = rt.toString();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rt)").unwrap().value,
        "el",
        "同文本节点 setStart/setEnd 须 slice(1,3)='el'"
    );

    // addRange → rangeCount=1 / selection.toString = range 文本 / type='Range' / isCollapsed=false。
    sandbox
        .execute(
            "var rp = document.createRange(); rp.selectNodeContents(document.getElementById('r').firstElementChild);\
             window.getSelection().addRange(rp);\
             globalThis.__src = window.getSelection().rangeCount;\
             globalThis.__srt = window.getSelection().toString();\
             globalThis.__sty = window.getSelection().type;\
             globalThis.__sic = window.getSelection().isCollapsed;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__src)").unwrap().value,
        "1",
        "addRange 后 rangeCount 须为 1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__srt)").unwrap().value,
        "hello",
        "selection.toString 须为 range 文本 'hello'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sty)").unwrap().value,
        "Range",
        "addRange 后 type 须为 'Range'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sic)").unwrap().value,
        "false",
        "addRange 后 isCollapsed 须为 false"
    );

    // removeAllRanges → 回空。
    sandbox.execute("window.getSelection().removeAllRanges();").unwrap();
    assert_eq!(
        sandbox
            .execute("String(window.getSelection().rangeCount)")
            .unwrap()
            .value,
        "0",
        "removeAllRanges 后 rangeCount 须回 0"
    );
    assert_eq!(
        sandbox.execute("String(window.getSelection().type)").unwrap().value,
        "None",
        "removeAllRanges 后 type 须回 'None'"
    );
}

#[test]
fn test_element_reflected_props_r2805() {
    // R2805：element reflected 属性簇（tabIndex/title/lang/dir）。get 反射同名 attribute（tabIndex 数值，
    // 无→-1；title/lang/dir 无→''），set 写 attribute。照 id/className reflected 模式（get+set trap）。
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
        "<html><body><a id=a href=# tabindex=3 title=hi lang=ja dir=rtl>x</a><div id=d>y</div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // get：反射 attribute（tabIndex 数值 / title·lang·dir 串）。
    sandbox
        .execute(
            "var a = document.getElementById('a');\
             globalThis.__ti = a.tabIndex;\
             globalThis.__tt = a.title; globalThis.__tl = a.lang; globalThis.__td = a.dir;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ti)").unwrap().value,
        "3",
        "tabIndex 须反射 tabindex 属性值"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tt)").unwrap().value,
        "hi",
        "title 须反射 title 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tl)").unwrap().value,
        "ja",
        "lang 须反射 lang 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__td)").unwrap().value,
        "rtl",
        "dir 须反射 dir 属性"
    );

    // 无属性默认：tabIndex → -1；title/lang/dir → ''。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             globalThis.__dti = d.tabIndex;\
             globalThis.__dtt = d.title; globalThis.__dtl = d.lang; globalThis.__dtd = d.dir;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dti)").unwrap().value,
        "-1",
        "无 tabindex 属性 tabIndex 须 -1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dtt)").unwrap().value,
        "",
        "无 title 属性 title 须 ''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dtl)").unwrap().value,
        "",
        "无 lang 属性 lang 须 ''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dtd)").unwrap().value,
        "",
        "无 dir 属性 dir 须 ''"
    );

    // set：写 attribute 并经属性反射同步读回（mutation 异步入队，getAttribute 读 stale 快照——同
    // value/className 既有限制；属性 get 经客户端缓存同步往返，是正确可测行为）。
    sandbox
        .execute(
            "d.tabIndex = 5; d.title = 'tip'; d.lang = 'en'; d.dir = 'ltr';\
             globalThis.__sti = d.tabIndex;\
             globalThis.__stt = d.title; globalThis.__stl = d.lang; globalThis.__std = d.dir;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sti)").unwrap().value,
        "5",
        "tabIndex set 后读回 5"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__stt)").unwrap().value,
        "tip",
        "title set 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__stl)").unwrap().value,
        "en",
        "lang set 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__std)").unwrap().value,
        "ltr",
        "dir set 后读回"
    );

    // tabIndex set 非数值（NaN）lenient 忽略（不写 attribute，读回仍原值）。
    sandbox
        .execute("d.tabIndex = 'abc'; globalThis.__nti = d.tabIndex;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nti)").unwrap().value,
        "5",
        "tabIndex set NaN 须 lenient 忽略（读回原 5）"
    );
}

#[test]
fn test_element_reflected_props2_r2806() {
    // R2806：reflected 簇 #2——contentEditable/isContentEditable + accessKey（编辑器栈锚 + a11y）。
    // contentEditable 字符串反射（无 → 'inherit'）+ client cache 同步往返；isContentEditable 计算 bool；
    // accessKey 字符串反射（无 → ''）+ cache。draggable/spellcheck 为枚举属性（非 presence-bool）defer。
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
        "<html><body><div id=e contenteditable=true accesskey=w>x</div><div id=p>y</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // contentEditable get：有属性 → 'true'；accessKey get → 'w'。
    sandbox
        .execute(
            "var e = document.getElementById('e');\
             globalThis.__ce = e.contentEditable;\
             globalThis.__ak = e.accessKey;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ce)").unwrap().value,
        "true",
        "contentEditable 须反射 contenteditable='true'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ak)").unwrap().value,
        "w",
        "accessKey 须反射 accesskey='w'"
    );

    // 无属性默认：contentEditable → 'inherit'；accessKey → ''。
    sandbox
        .execute(
            "var p = document.getElementById('p');\
             globalThis.__pce = p.contentEditable;\
             globalThis.__pak = p.accessKey;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__pce)").unwrap().value,
        "inherit",
        "无 contenteditable 属性 contentEditable 须 'inherit'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pak)").unwrap().value,
        "",
        "无 accesskey 属性 accessKey 须 ''"
    );

    // contentEditable set + 同步读回 + isContentEditable=true。
    sandbox
        .execute(
            "p.contentEditable = 'true';\
             globalThis.__sce = p.contentEditable;\
             globalThis.__ice = p.isContentEditable;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sce)").unwrap().value,
        "true",
        "contentEditable set='true' 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ice)").unwrap().value,
        "true",
        "isContentEditable 须当 contentEditable='true' 时 true"
    );

    // contentEditable set 'false' → isContentEditable=false。
    sandbox
        .execute("p.contentEditable = 'false'; globalThis.__ice2 = p.isContentEditable;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ice2)").unwrap().value,
        "false",
        "contentEditable='false' 时 isContentEditable 须 false"
    );

    // accessKey set + 同步读回。
    sandbox
        .execute("p.accessKey = 'k'; globalThis.__sak = p.accessKey;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sak)").unwrap().value,
        "k",
        "accessKey set 后读回"
    );
}

#[test]
fn test_element_aria_r2807() {
    // R2807：element.role + aria-* 反射簇（ARIA a11y 高频）。通用 _ariaAttrName 映射 ariaXxx↔aria-xxx
    //（ariaLabelledBy→aria-labelledby 单 hyphen，非 _camelToKebab 的双 hyphen）。_reflectedAttrs 缓存同步往返。
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
        "<html><body><button id=b role=button aria-label=Save aria-labelledby=l1 aria-valuenow=50>x</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // role get（读 role 属性）+ aria* get（读 aria-* 属性，验单/多词映射）。
    sandbox
        .execute(
            "var b = document.getElementById('b');\
             globalThis.__role = b.role;\
             globalThis.__al = b.ariaLabel;\
             globalThis.__alb = b.ariaLabelledBy;\
             globalThis.__avn = b.ariaValueNow;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__role)").unwrap().value,
        "button",
        "role 须反射 role 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__al)").unwrap().value,
        "Save",
        "ariaLabel 须反射 aria-label"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__alb)").unwrap().value,
        "l1",
        "ariaLabelledBy 须反射 aria-labelledby（单 hyphen）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__avn)").unwrap().value,
        "50",
        "ariaValueNow 须反射 aria-valuenow"
    );

    // set + 同步读回（缓存往返）：role + aria*。
    sandbox
        .execute(
            "b.role = 'link'; b.ariaLabel = 'Submit'; b.ariaExpanded = 'true';\
             globalThis.__srole = b.role;\
             globalThis.__sal = b.ariaLabel;\
             globalThis.__sae = b.ariaExpanded;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__srole)").unwrap().value,
        "link",
        "role set 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sal)").unwrap().value,
        "Submit",
        "ariaLabel set 后读回"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sae)").unwrap().value,
        "true",
        "ariaExpanded set 后读回（aria-expanded）"
    );

    // 无属性默认 ''（role + aria*）。
    sandbox
        .execute(
            "var d = document.createElement('div');\
             globalThis.__dr = d.role; globalThis.__dal = d.ariaLabel;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dr)").unwrap().value,
        "",
        "无 role 属性 role 须 ''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dal)").unwrap().value,
        "",
        "无 aria-label ariaLabel 须 ''"
    );
}

#[test]
fn test_document_stylesheets_r2808() {
    // R2808：CSSStyleSheet 只读 CSSOM——document.styleSheets 真 backing `<style>` + cssRules 读 +
    // CSSRule.selectorText/cssText/type。host 经 `__zw_style_rules`（parse_stylesheet + Selector 序列化）。
    // insertRule/deleteRule/`<link>` defer。
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
        "<html><head><style>p { color: red; } .a > b { font-size: 14px; }</style></head><body><p>x</p></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // document.styleSheets.length === 1（一个 <style>）；ownerNode 为 style 元素。
    sandbox
        .execute(
            "globalThis.__sheets = document.styleSheets;\
             globalThis.__len = __sheets.length;\
             globalThis.__sheet = __sheets[0];\
             globalThis.__ownerTag = __sheet.ownerNode ? __sheet.ownerNode.tagName : '';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "1",
        "styleSheets 须含 1 个 <style>"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ownerTag)").unwrap().value,
        "STYLE",
        "sheet.ownerNode 须为 <style> 元素"
    );

    // cssRules.length === 2；rule[0] selectorText 'p' + cssText 含 'color: red' + type=1。
    sandbox
        .execute(
            "globalThis.__rc = globalThis.__sheet.cssRules.length;\
             globalThis.__r0s = globalThis.__sheet.cssRules[0].selectorText;\
             globalThis.__r0c = globalThis.__sheet.cssRules[0].cssText;\
             globalThis.__r0t = globalThis.__sheet.cssRules[0].type;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rc)").unwrap().value,
        "2",
        "cssRules 须含 2 条规则"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r0s)").unwrap().value,
        "p",
        "cssRules[0].selectorText 须 'p'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__r0c.indexOf('color: red') >= 0)")
            .unwrap()
            .value,
        "true",
        "cssRules[0].cssText 须含 'color: red'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r0t)").unwrap().value,
        "1",
        "cssRules[0].type 须 1（STYLE_RULE）"
    );

    // rule[1] selectorText '.a > b'（组合器序列化）+ cssText 含 'font-size: 14px'。
    sandbox
        .execute(
            "globalThis.__r1s = globalThis.__sheet.cssRules[1].selectorText;\
             globalThis.__r1c = globalThis.__sheet.cssRules[1].cssText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r1s)").unwrap().value,
        ".a > b",
        "cssRules[1].selectorText 须含组合器 '.a > b'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__r1c.indexOf('font-size: 14px') >= 0)")
            .unwrap()
            .value,
        "true",
        "cssRules[1].cssText 须含 'font-size: 14px'"
    );

    // 无 <style> 文档 → styleSheets.length === 0。
    let dom_html2: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><p>no style</p></body></html>".to_string()));
    let mutations2: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let page_url2: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let mut sandbox2 = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox2.execute(generate_js_dom_shim()).unwrap();
    register_dom_callbacks(&mut sandbox2, &mutations2, &dom_html2, &page_url2);
    assert_eq!(
        sandbox2.execute("String(document.styleSheets.length)").unwrap().value,
        "0",
        "无 <style> 时 styleSheets.length 须 0"
    );
}

#[test]
fn test_stylesheets_write_r2809() {
    // R2809：CSSStyleSheet 写路径——insertRule/deleteRule。维护 client cache（同步读回真值）+ flush 重建
    // `<style>` 文本经 __zw_set_text（下次 render cascade，视觉异步；JS 契约同步）。
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
        "<html><head><style>p { color: red; }</style></head><body><p>x</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 初始 cssRules.length === 1（'p'）。
    sandbox
        .execute("globalThis.__sheet = document.styleSheets[0]; globalThis.__l0 = globalThis.__sheet.cssRules.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__l0)").unwrap().value,
        "1",
        "初始 cssRules 须 1 条"
    );

    // insertRule('div { color: blue; }', 0)：返 0 + length=2 + [0]='div'（插入首位）+ [1]='p'（原规则后移）。
    sandbox
        .execute(
            "globalThis.__idx = globalThis.__sheet.insertRule('div { color: blue; }', 0);\
             globalThis.__l1 = globalThis.__sheet.cssRules.length;\
             globalThis.__s0 = globalThis.__sheet.cssRules[0].selectorText;\
             globalThis.__s1 = globalThis.__sheet.cssRules[1].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__idx)").unwrap().value,
        "0",
        "insertRule 须返插入 index 0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__l1)").unwrap().value,
        "2",
        "insertRule 后 cssRules.length 须 2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s0)").unwrap().value,
        "div",
        "cssRules[0] 须为插入的 'div'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s1)").unwrap().value,
        "p",
        "cssRules[1] 须为原 'p'（后移）"
    );
    // 插入规则的 cssText 含 'color: blue'。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__sheet.cssRules[0].cssText.indexOf('color: blue') >= 0)")
            .unwrap()
            .value,
        "true",
        "插入规则 cssText 须含 'color: blue'"
    );

    // insertRule 不带 index → 末尾追加，返末尾 index。
    sandbox
        .execute(
            "globalThis.__idx2 = globalThis.__sheet.insertRule('span { color: green; }');\
             globalThis.__l2 = globalThis.__sheet.cssRules.length;\
             globalThis.__sEnd = globalThis.__sheet.cssRules[2].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__idx2)").unwrap().value,
        "2",
        "末尾 insertRule 须返 index 2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__l2)").unwrap().value,
        "3",
        "末尾追加后 length 须 3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sEnd)").unwrap().value,
        "span",
        "末尾规则须 'span'"
    );

    // deleteRule(0)：移除 'div' + length=2 + [0]='p'（回原首位）。
    sandbox
        .execute(
            "globalThis.__sheet.deleteRule(0);\
             globalThis.__l3 = globalThis.__sheet.cssRules.length;\
             globalThis.__s0b = globalThis.__sheet.cssRules[0].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__l3)").unwrap().value,
        "2",
        "deleteRule 后 length 须 2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s0b)").unwrap().value,
        "p",
        "deleteRule(0) 后 [0] 须回 'p'"
    );

    // 写回 `<style>` 文本（flush）：mutations 含 SetText（验证写源生效，cascade 下次 render）。
    let muts = mutations.lock().unwrap();
    let has_set_text = muts.iter().any(|m| matches!(m, DomMutation::SetText { .. }));
    drop(muts);
    assert!(
        has_set_text,
        "insertRule/deleteRule 须经 __zw_set_text 写回 <style> 文本（flush）"
    );
}

#[test]
fn test_stylesheet_add_rule_remove_rule_r3276() {
    // R3276：CSSStyleSheet IE legacy 别名 addRule/removeRule 真实化。
    // 旧实现：addRule 恒返 -1 不插规则，removeRule no-op → 旧 CSS-in-JS 库 / legacy 样式注入走此路径
    // 时样式静默丢失。spec（IE 扩展，Chrome/Firefox 保留兼容）：
    //   addRule(selector, styleBlock, index?) → 组合 `selector{styleBlock}` 调 insertRule，恒返 -1。
    //   removeRule(index) → 等价 deleteRule(index)。
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
        "<html><head><style>p { color: red; }</style></head><body><p>x</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute("globalThis.__sheet = document.styleSheets[0];")
        .unwrap();

    // addRule('h1', 'color: blue', 0)：恒返 -1（IE 成功 marker）+ 规则真实插入（length 2 + [0]='h1'）。
    sandbox
        .execute(
            "globalThis.__ret = globalThis.__sheet.addRule('h1', 'color: blue', 0);\
             globalThis.__l1 = globalThis.__sheet.cssRules.length;\
             globalThis.__s0 = globalThis.__sheet.cssRules[0].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ret)").unwrap().value,
        "-1",
        "addRule 恒返 -1（IE legacy 成功 marker）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__l1)").unwrap().value,
        "2",
        "addRule 后 cssRules.length 须 2（规则真实插入）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s0)").unwrap().value,
        "h1",
        "addRule 插入首位须为 'h1'"
    );
    // 插入规则的声明块（styleBlock）真实写入 cssText。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__sheet.cssRules[0].cssText.indexOf('color: blue') >= 0)")
            .unwrap()
            .value,
        "true",
        "addRule 组合的 styleBlock 须写进 cssText"
    );

    // addRule 不带 index → 末尾追加（insertRule clamp），仍返 -1。
    sandbox
        .execute(
            "globalThis.__ret2 = globalThis.__sheet.addRule('span', 'color: green');\
             globalThis.__l2 = globalThis.__sheet.cssRules.length;\
             globalThis.__sEnd = globalThis.__sheet.cssRules[2].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ret2)").unwrap().value,
        "-1",
        "addRule（末尾追加）仍返 -1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__l2)").unwrap().value,
        "3",
        "末尾 addRule 后 length 须 3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sEnd)").unwrap().value,
        "span",
        "末尾规则须 'span'"
    );

    // removeRule(0)：等价 deleteRule(0)——移除 'h1' + length=2 + [0]='p'。
    sandbox
        .execute(
            "globalThis.__sheet.removeRule(0);\
             globalThis.__l3 = globalThis.__sheet.cssRules.length;\
             globalThis.__s0b = globalThis.__sheet.cssRules[0].selectorText;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__l3)").unwrap().value,
        "2",
        "removeRule(0) 后 length 须 2（真实删除）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s0b)").unwrap().value,
        "p",
        "removeRule(0) 后 [0] 须回 'p'"
    );

    // 写回 `<style>` 文本（flush）：addRule/removeRule 经 insertRule/deleteRule → __zw_set_text。
    let muts = mutations.lock().unwrap();
    let has_set_text = muts.iter().any(|m| matches!(m, DomMutation::SetText { .. }));
    drop(muts);
    assert!(
        has_set_text,
        "addRule/removeRule 须经 insertRule/deleteRule flush 写回 <style> 文本"
    );
}

#[test]
fn test_stylesheets_rule_style_r2810() {
    // R2810：CSSRule.style per-rule CSSStyleDeclaration——sheet.cssRules[0].style 单声明读/写。
    // backed by 规则声明块（从 cssText body 解析有序 declarations）+ mutation flush 写回 `<style>` 源
    // （复用 R2809 flushToOwner）。per-property get/set（camelCase↔kebab）+ getPropertyValue/setProperty/
    // removeProperty + cssText + item/length。
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
        "<html><head><style>p { color: red; font-size: 14px; }</style></head><body><p>x</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // document.styleSheets 每次访问重建新 sheet（R2808 已知限制 ⑤，live DOM 重查）——故 rule/style 须
    // 一次捕获并复用引用，跨访问读取会落到新对象（host stale）。CSS-in-JS 常规用法即持引用编辑。
    sandbox
        .execute(
            "globalThis.__rule = document.styleSheets[0].cssRules[0];\
             globalThis.__st = __rule.style;",
        )
        .unwrap();

    // style 读既有声明：style.color='red' / getPropertyValue('font-size')='14px' / length=2 / item(0)='color'。
    sandbox
        .execute(
            "globalThis.__c = __st.color;\
             globalThis.__fs = __st.getPropertyValue('font-size');\
             globalThis.__len = __st.length;\
             globalThis.__item0 = __st.item(0);\
             globalThis.__camel = __st.fontSize;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__c)").unwrap().value,
        "red",
        "style.color 须读回既有 'red'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fs)").unwrap().value,
        "14px",
        "getPropertyValue('font-size') 须 '14px'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__camel)").unwrap().value,
        "14px",
        "camelCase style.fontSize 须 '14px'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "2",
        "style.length 须 2（color + font-size）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__item0)").unwrap().value,
        "color",
        "style.item(0) 须 'color'"
    );

    // set 既有属性：style.color='blue' → 同一 rule.cssText 反映 'color: blue' + flush 写回（SetText）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute("globalThis.__st.color = 'blue'; globalThis.__rc = __rule.cssText;")
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__rc.indexOf('color: blue') >= 0)")
            .unwrap()
            .value,
        "true",
        "style.color='blue' 后 rule.cssText 须含 'color: blue'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__rc.indexOf('color: red') >= 0)")
            .unwrap()
            .value,
        "false",
        "旧值 'color: red' 须被替换移除"
    );
    let muts = mutations.lock().unwrap();
    let has_set_text = muts.iter().any(|m| matches!(m, DomMutation::SetText { .. }));
    drop(muts);
    assert!(
        has_set_text,
        "style mutation 须经 flush __zw_set_text 写回 <style> 文本"
    );

    // setProperty 新增属性 + camelCase set + cssText 整体读。
    sandbox
        .execute(
            "globalThis.__st.setProperty('background', 'yellow');\
             globalThis.__st.marginTop = '8px';\
             globalThis.__decl = __st.cssText;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__decl.indexOf('background: yellow') >= 0)")
            .unwrap()
            .value,
        "true",
        "setProperty('background','yellow') 须入声明块"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__decl.indexOf('margin-top: 8px') >= 0)")
            .unwrap()
            .value,
        "true",
        "camelCase style.marginTop='8px' 须归一为 'margin-top: 8px'"
    );

    // removeProperty('color') → 返旧值 'blue' + 声明块不再含 color。
    sandbox
        .execute(
            "globalThis.__prev = __st.removeProperty('color');\
             globalThis.__hasColor = __st.cssText.indexOf('color:') >= 0;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__prev)").unwrap().value,
        "blue",
        "removeProperty 须返被移除的旧值 'blue'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hasColor)").unwrap().value,
        "false",
        "removeProperty('color') 后声明块不再含 'color:'"
    );

    // cssText 整体写 → 替换全部声明（同一 style 对象读 width）。
    sandbox
        .execute(
            "globalThis.__st.cssText = 'width: 100px; height: 50px';\
             globalThis.__after = __st.length;\
             globalThis.__w = __st.width;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__after)").unwrap().value,
        "2",
        "cssText 整体写后 length 须为 2（替换全部）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__w)").unwrap().value,
        "100px",
        "cssText 写后 style.width 须 '100px'"
    );
}

#[test]
fn test_event_subclasses_r2811() {
    // R2811：Event 子类簇——UIEvent / MouseEvent / FocusEvent / WheelEvent / PointerEvent / InputEvent。
    // 现代输入事件表面：feature-detection（typeof === 'function'）+ `new MouseEvent('click',{clientX,...})`
    // 合成派发。经 _defineEventSubclass 工厂（复用 _makeEvent + 原型链 extends parent）。getModifierState 仅
    // 跟踪 Alt/Control/Meta/Shift；WheelEvent 有 DOM_DELTA_* 静态常量；createEvent 映射含 mouseevent。
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

    // feature-detection：6 构造器均为 function。
    sandbox
        .execute(
            "globalThis.__fns = ['UIEvent','MouseEvent','FocusEvent','WheelEvent','PointerEvent','InputEvent']\
               .map(function(n){ return typeof globalThis[n] === 'function'; })\
               .every(Boolean);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fns)").unwrap().value,
        "true",
        "6 个 Event 子类构造器须均为 function"
    );

    // MouseEvent 字段 + instanceof 链（MouseEvent→UIEvent→Event）+ getModifierState。
    sandbox
        .execute(
            "globalThis.__me = new MouseEvent('click', { bubbles: true, clientX: 10, clientY: 20, button: 2, shiftKey: true });\
             globalThis.__meType = __me.type;\
             globalThis.__meBub = __me.bubbles;\
             globalThis.__meCX = __me.clientX;\
             globalThis.__meButton = __me.button;\
             globalThis.__meDef = __me.screenX;\
             globalThis.__meRel = __me.relatedTarget;\
             globalThis.__meChain = (__me instanceof MouseEvent) && (__me instanceof UIEvent) && (__me instanceof Event);\
             globalThis.__meShift = __me.getModifierState('Shift');\
             globalThis.__meAlt = __me.getModifierState('Alt');\
             globalThis.__meCaps = __me.getModifierState('CapsLock');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__meType)").unwrap().value,
        "click",
        "MouseEvent type"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meBub)").unwrap().value,
        "true",
        "MouseEvent bubbles 经 _makeEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meCX)").unwrap().value,
        "10",
        "MouseEvent clientX 来自 options"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meButton)").unwrap().value,
        "2",
        "MouseEvent button"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meDef)").unwrap().value,
        "0",
        "MouseEvent 默认 screenX=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meRel)").unwrap().value,
        "null",
        "MouseEvent relatedTarget 默认 null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meChain)").unwrap().value,
        "true",
        "MouseEvent instanceof MouseEvent & UIEvent & Event（原型链）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meShift)").unwrap().value,
        "true",
        "getModifierState('Shift') 跟踪 shiftKey"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meAlt)").unwrap().value,
        "false",
        "getModifierState('Alt') 未设→false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__meCaps)").unwrap().value,
        "false",
        "getModifierState('CapsLock') 未跟踪→false"
    );

    // WheelEvent delta + DOM_DELTA_* 常量 + instanceof MouseEvent。
    sandbox
        .execute(
            "globalThis.__we = new WheelEvent('wheel', { deltaY: 120, deltaMode: 1 });\
             globalThis.__weDY = __we.deltaY;\
             globalThis.__weDM = __we.deltaMode;\
             globalThis.__weConst = WheelEvent.DOM_DELTA_LINE;\
             globalThis.__weChain = (__we instanceof WheelEvent) && (__we instanceof MouseEvent);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__weDY)").unwrap().value,
        "120",
        "WheelEvent deltaY"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__weDM)").unwrap().value,
        "1",
        "WheelEvent deltaMode"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__weConst)").unwrap().value,
        "1",
        "WheelEvent.DOM_DELTA_LINE=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__weChain)").unwrap().value,
        "true",
        "WheelEvent instanceof MouseEvent"
    );

    // PointerEvent 字段 + instanceof MouseEvent。
    sandbox
        .execute(
            "globalThis.__pe = new PointerEvent('pointerdown', { pointerType: 'mouse', isPrimary: true, pressure: 0.5 });\
             globalThis.__pePT = __pe.pointerType;\
             globalThis.__pePri = __pe.isPrimary;\
             globalThis.__peChain = (__pe instanceof PointerEvent) && (__pe instanceof MouseEvent);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__pePT)").unwrap().value,
        "mouse",
        "PointerEvent pointerType"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pePri)").unwrap().value,
        "true",
        "PointerEvent isPrimary"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__peChain)").unwrap().value,
        "true",
        "PointerEvent instanceof MouseEvent"
    );

    // FocusEvent / InputEvent instanceof UIEvent；FocusEvent.relatedTarget / InputEvent.data+inputType。
    sandbox
        .execute(
            "globalThis.__fe = new FocusEvent('blur');\
             globalThis.__feChain = (__fe instanceof FocusEvent) && (__fe instanceof UIEvent);\
             globalThis.__ie = new InputEvent('input', { data: 'x', inputType: 'insertText' });\
             globalThis.__ieChain = (__ie instanceof InputEvent) && (__ie instanceof UIEvent);\
             globalThis.__ieData = __ie.data;\
             globalThis.__ieType = __ie.inputType;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__feChain)").unwrap().value,
        "true",
        "FocusEvent instanceof UIEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ieChain)").unwrap().value,
        "true",
        "InputEvent instanceof UIEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ieData)").unwrap().value,
        "x",
        "InputEvent data"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ieType)").unwrap().value,
        "insertText",
        "InputEvent inputType"
    );

    // createEvent('MouseEvent') instanceof MouseEvent（映射扩展）。
    sandbox
        .execute("globalThis.__cme = document.createEvent('MouseEvent') instanceof MouseEvent;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cme)").unwrap().value,
        "true",
        "createEvent('MouseEvent') instanceof MouseEvent"
    );
}

#[test]
fn test_scroll_into_view_r3060() {
    // R3060：scrollIntoView 真实化（闭合 R3047 no-op）。把文档 scrollTop 设为元素 gBCR.y 按 block 对齐，
    // 复用 globalThis.scrollTo（更新 _winScroll + 派发 scroll 事件，R3047/R3051）。元素置于视口下方
    //（y=1000，innerHeight=800）使 block 各对齐产生可区分值：start→1000 / end→250 / center→625。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='d'>x</div></body></html>".to_string()));
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
    assert_eq!(sandbox.execute("String(globalThis.innerHeight)").unwrap().value, "800", "innerHeight=800（视口高）");

    // 初始 scrollY=0；注册 scroll listener 计数。
    assert_eq!(sandbox.execute("window.scrollY").unwrap().value, "0", "初始 scrollY=0");
    sandbox.execute("globalThis.__sc=0; addEventListener('scroll', function(){ globalThis.__sc++; });").unwrap();

    // ① scrollIntoView()（block start）→ newTop=y=1000 + scroll 事件。
    sandbox.execute("document.querySelector('#d').scrollIntoView();").unwrap();
    assert_eq!(sandbox.execute("window.scrollY").unwrap().value, "1000", "scrollIntoView()（start）→ scrollY=1000（元素 y）");
    assert_eq!(sandbox.execute("String(globalThis.__sc)").unwrap().value, "1", "scrollIntoView → 派发 scroll 事件");

    // ② scrollIntoView({block:'end'}) → newTop = y + h - vh = 1000+50-800 = 250。
    sandbox.execute("globalThis.__sc=0; document.querySelector('#d').scrollIntoView({block:'end'});").unwrap();
    assert_eq!(sandbox.execute("window.scrollY").unwrap().value, "250", "scrollIntoView(block:end) -> scrollY=250 (y+h-vh)");

    // ③ scrollIntoView({block:'center'}) → newTop = y - vh/2 + h/2 = 1000-400+25 = 625。
    sandbox.execute("globalThis.__sc=0; document.querySelector('#d').scrollIntoView({block:'center'});").unwrap();
    assert_eq!(sandbox.execute("window.scrollY").unwrap().value, "625", "scrollIntoView(block:center) -> scrollY=625 (y-vh/2+h/2)");

    // ④ 无 rect（detached createElement 元素）→ no-op（scrollY 不变，不派 scroll）。
    sandbox.execute("globalThis.__sc=0; var e=document.createElement('div'); e.scrollIntoView();").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__sc)").unwrap().value, "0", "detached 元素无 rect → scrollIntoView no-op（不派 scroll）");
}

#[test]
fn test_hash_scroll_to_anchor_r3061() {
    // R3061：location.hash= 设值（含 <a href="#sec"> click 经 R3053 路径）滚到锚元素（id/name=hash），
    // 闭合 R3053 限制①。复用 R3060 scrollIntoView（更新 scrollTop + 派 scroll 事件）。无匹配元素 → 不滚。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id='sec'>x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    // mock rect：非 handle 选择器 → "0,500,100,50"（y=500）；handle（detached）→ 空。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| match args.first() {
            Some(s) if s.starts_with("__") => String::new(),
            _ => "0,500,100,50".to_string(),
        }),
    );

    assert_eq!(sandbox.execute("window.scrollY").unwrap().value, "0", "初始 scrollY=0");
    sandbox.execute("globalThis.__hc=0; addEventListener('hashchange', function(){ globalThis.__hc++; });").unwrap();

    // ① location.hash='#sec' → 滚到 #sec 元素（y=500）+ hashchange（R3006 不受影响）。
    sandbox.execute("location.hash = '#sec';").unwrap();
    assert_eq!(
        sandbox.execute("window.scrollY").unwrap().value,
        "500",
        "location.hash='#sec' -> 滚到 #sec 元素（scrollY=500）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hc)").unwrap().value,
        "1",
        "hashchange 仍派发（R3006 不受 R3061 滚锚影响）"
    );

    // ② 无匹配元素的 hash（#nope）→ 不滚（scrollY 不变）但 hashchange 派发。
    sandbox.execute("globalThis.__hc=0; location.hash = '#nope';").unwrap();
    assert_eq!(
        sandbox.execute("window.scrollY").unwrap().value,
        "500",
        "无匹配元素 -> 不滚（scrollY 保持 500）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hc)").unwrap().value,
        "1",
        "无匹配元素 -> hashchange 仍派发"
    );
}

#[test]
fn test_history_back_forward_cross_hash_scroll_r3065() {
    // R3065：history back/forward/go 到 hash entry 滚到锚元素（闭合 R3061 限制②）。R3061 仅 _setLocationHash
    //（location.hash= setter）滚锚，back/forward（_hist_dispatchPopState）到 hash entry 不滚——提取
    // _scrollToAnchorForHash 共享 helper，back/forward hashChanged 时调用。real browser 跨 hash 导航滚锚
    //（back 到 #sec entry 滚到 id/name="sec"）。同步滚（mirror _setLocationHash），popstate/hashchange 仍 defer。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='sec'>S</div><div id='other'>O</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    // mock rect：#sec → y=500，#other → y=900（scrollIntoView block:start → scrollY=y）；handle/detached → 空。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| match args.first() {
            Some(s) if s.starts_with("__") => String::new(),
            Some(s) if s.contains("#other") => "0,900,100,50".to_string(),
            Some(s) if s.contains("#sec") => "0,500,100,50".to_string(),
            _ => "0,0,0,0".to_string(),
        }),
    );

    assert_eq!(sandbox.execute("window.scrollY").unwrap().value, "0", "初始 scrollY=0");

    // 建 history：page → page#sec → page#other（每步 location.hash= 滚到锚，R3061）。
    sandbox.execute("location.hash = '#sec';").unwrap();
    assert_eq!(sandbox.execute("window.scrollY").unwrap().value, "500", "location.hash='#sec' -> scrollY=500");
    sandbox.execute("location.hash = '#other';").unwrap();
    assert_eq!(sandbox.execute("window.scrollY").unwrap().value, "900", "location.hash='#other' -> scrollY=900");

    // history.back() → 回 page#sec entry，hashChanged(other->sec) → 滚到 #sec（scrollY 900->500）。
    sandbox.execute("history.back();").unwrap();
    assert_eq!(
        sandbox.execute("window.scrollY").unwrap().value,
        "500",
        "history.back() 到 #sec entry -> 滚到 #sec（scrollY=500）"
    );

    // history.forward() → 进 page#other entry，hashChanged(sec->other) → 滚到 #other（scrollY 500->900）。
    sandbox.execute("history.forward();").unwrap();
    assert_eq!(
        sandbox.execute("window.scrollY").unwrap().value,
        "900",
        "history.forward() 到 #other entry -> 滚到 #other（scrollY=900）"
    );

    // history.go(-1) → 回 page#sec entry → 滚到 #sec（go 共享 _hist_dispatchPopState 路径）。
    sandbox.execute("history.go(-1);").unwrap();
    assert_eq!(
        sandbox.execute("window.scrollY").unwrap().value,
        "500",
        "history.go(-1) 到 #sec entry -> 滚到 #sec（scrollY=500）"
    );
}
