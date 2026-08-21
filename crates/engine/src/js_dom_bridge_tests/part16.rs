// js_dom_bridge 测试切片 16（R3314+）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-15 共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。
// R3314 起 part14（~2083 行）/ part15（~1985 行）均近/超 2000 行上限（rule 5 文件大小治理），
// 新切片承载 OPFS / Tier 2 Web API e2e 测试（part16 起）。

// ── R3314：OPFS（Origin Private File System）navigator.storage.getDirectory ──
//
// 生产 always-on B-gen shim 路径（js_dom_shim/part02.js navigator.storage IIFE 虚拟 FS 树）。
// spec https://fs.spec.whatwg.org/——headless 无真 OS 文件系统，进程内内存虚拟 FS 近似（参照 clipboard
// IIFE store 模式）。验证：getDirectory → getFileHandle(create) → createWritable+write+close → getFile 读回 +
// 子目录 + removeEntry + keys/entries 迭代 + estimate（配额）。

#[test]
fn test_opfs_write_read_roundtrip_r3314() {
    // OPFS 核心往返：getDirectory → getFileHandle(create:true) → createWritable → write(string) → close
    // → getFile → text() 读回。断言写入字节与读回一致。
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__hasStorage = String(navigator.storage && typeof navigator.storage.getDirectory === 'function');\
             navigator.storage.getDirectory().then(function (root) {\
               globalThis.__rootKind = String(root.kind);\
               /* getFileHandle create:true 建空文件 */\
               return root.getFileHandle('log.txt', { create: true });\
             }).then(function (fh) {\
               globalThis.__fhKind = String(fh.kind);\
               globalThis.__fhName = String(fh.name);\
               /* createWritable + write(string) + close */\
               return fh.createWritable().then(function (w) {\
                 return w.write('hello OPFS').then(function () { return w.close(); }).then(function () { return fh; });\
               });\
             }).then(function (fh) {\
               /* getFile → Blob → text() 读回 */\
               return fh.getFile();\
             }).then(function (file) {\
               globalThis.__fileIsBlob = String(file instanceof Blob);\
               return file.text();\
             }).then(function (txt) {\
               globalThis.__readBack = String(txt);\
               globalThis.__ok = 'ok';\
             }, function (err) {\
               globalThis.__ok = 'reject:' + String(err && err.message ? err.message : err);\
             });",
        )
        .unwrap();
    // pump microtask（OPFS 全 Promise 链，每环异步——多轮 execute drain）。
    sandbox.execute("globalThis.__n = 1;").unwrap();
    sandbox.execute("globalThis.__n = 2;").unwrap();
    sandbox.execute("globalThis.__n = 3;").unwrap();
    sandbox.execute("globalThis.__n = 4;").unwrap();
    sandbox.execute("globalThis.__n = 5;").unwrap();
    sandbox.execute("globalThis.__n = 6;").unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__hasStorage").unwrap().value,
        "true",
        "navigator.storage.getDirectory 存在"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "ok",
        "OPFS write→read 往返应成功 resolve"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rootKind").unwrap().value,
        "directory",
        "getDirectory() 返 directory 句柄"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fhKind").unwrap().value,
        "file",
        "getFileHandle 返 file 句柄"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fhName").unwrap().value,
        "log.txt",
        "文件句柄 name = log.txt"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fileIsBlob").unwrap().value,
        "true",
        "getFile() 返 Blob"
    );
    assert_eq!(
        sandbox.execute("globalThis.__readBack").unwrap().value,
        "hello OPFS",
        "写入内容读回一致（write→close→getFile→text 往返）"
    );
}

#[test]
fn test_opfs_directory_and_remove_r3314() {
    // OPFS 目录操作：getDirectoryHandle(create) 建子目录 + 嵌套文件 + removeEntry 删除 + keys 迭代。
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "navigator.storage.getDirectory().then(function (root) {\
               /* 建子目录 + 嵌套文件 */\
               return root.getDirectoryHandle('docs', { create: true }).then(function (dh) {\
                 return dh.getFileHandle('a.txt', { create: true });\
               }).then(function () {\
                 /* root.keys 列举子项（应含 'docs'）*/\
                 return root.keys();\
               }).then(function (ks) {\
                 globalThis.__rootKeys = String(ks.join(','));\
                 /* removeEntry 删子目录（非空须 recursive——R3254-C14 spec 语义）*/\
                 return root.removeEntry('docs', { recursive: true });\
               }).then(function () {\
                 return root.keys();\
               }).then(function (ks2) {\
                 globalThis.__afterRemove = String(ks2.join(','));\
                 /* estimate 配额查询 */\
                 return navigator.storage.estimate();\
               }).then(function (est) {\
                 globalThis.__hasQuota = String(est && typeof est.quota === 'number');\
                 globalThis.__ok = 'ok';\
               });\
             }, function (err) {\
               globalThis.__ok = 'reject:' + String(err && err.message ? err.message : err);\
             });",
        )
        .unwrap();
    sandbox.execute("globalThis.__n = 1;").unwrap();
    sandbox.execute("globalThis.__n = 2;").unwrap();
    sandbox.execute("globalThis.__n = 3;").unwrap();
    sandbox.execute("globalThis.__n = 4;").unwrap();
    sandbox.execute("globalThis.__n = 5;").unwrap();
    sandbox.execute("globalThis.__n = 6;").unwrap();
    sandbox.execute("globalThis.__n = 7;").unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "ok",
        "OPFS 目录操作链应成功"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rootKeys").unwrap().value,
        "docs",
        "建 docs 子目录后 root.keys 含 'docs'"
    );
    assert_eq!(
        sandbox.execute("globalThis.__afterRemove").unwrap().value,
        "",
        "removeEntry('docs') 后 root.keys 为空"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasQuota").unwrap().value,
        "true",
        "estimate() 返配额对象（含 quota 数值）"
    );
}

#[test]
fn test_opfs_writable_seek_truncate_r3315() {
    // R3315：OPFS createWritable seek/truncate/position（spec FileSystemWritableFileStream §8.5）。
    // R3314 createWritable 仅追加式 write/close。本测断言：① write→seek→write 在指定位置写入（覆盖中间，非纯追加）；
    // ② truncate 截断；③ write({position,data}) 带位置写入；④ keepExistingData:true 保留原内容后追加。
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 用 async/await 扁平化 4 子测试（避免手动 Promise 链括号失衡）。V8 原生支持 async/await。
    sandbox
        .execute(
            "(async function () {\
               try {\
                 var root = await navigator.storage.getDirectory();\
                 /* ① write→seek→write：ABCDE → pos1 写 XY → AXYDE */\
                 var fh1 = await root.getFileHandle('f1', { create: true });\
                 var w1 = await fh1.createWritable();\
                 await w1.write('ABCDE');\
                 await w1.seek(1);\
                 await w1.write('XY');\
                 await w1.close();\
                 globalThis.__seekResult = String(await (await fh1.getFile()).text());\
                 /* ② truncate：ABCDEF → truncate(3) → ABC */\
                 var fh2 = await root.getFileHandle('f2', { create: true });\
                 var w2 = await fh2.createWritable();\
                 await w2.write('ABCDEF');\
                 await w2.truncate(3);\
                 await w2.close();\
                 globalThis.__truncResult = String(await (await fh2.getFile()).text());\
                 /* ③ write({position,data}) 带位置：AAAA → pos2 写 BB → AABB */\
                 var fh3 = await root.getFileHandle('f3', { create: true });\
                 var w3 = await fh3.createWritable();\
                 await w3.write('AAAA');\
                 await w3.write({ type: 'write', position: 2, data: 'BB' });\
                 await w3.close();\
                 globalThis.__posResult = String(await (await fh3.getFile()).text());\
                 /* ④ keepExistingData：orig → keepExistingData createWritable → seek(4) 写 + → orig+ */\
                 var fh4 = await root.getFileHandle('f4', { create: true });\
                 var w4a = await fh4.createWritable();\
                 await w4a.write('orig');\
                 await w4a.close();\
                 var w4b = await fh4.createWritable({ keepExistingData: true });\
                 await w4b.seek(4);\
                 await w4b.write('+');\
                 await w4b.close();\
                 globalThis.__keepResult = String(await (await fh4.getFile()).text());\
                 globalThis.__ok = 'ok';\
               } catch (err) {\
                 globalThis.__ok = 'reject:' + String(err && err.message ? err.message : err);\
               }\
             })();",
        )
        .unwrap();
    // pump microtask（async 函数每 await 让出，多轮 execute drain）。
    for i in 1..=15 {
        let _ = sandbox.execute(&format!("globalThis.__p{i} = 1;"));
    }

    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "ok",
        "OPFS seek/truncate/position 链应成功"
    );
    assert_eq!(
        sandbox.execute("globalThis.__seekResult").unwrap().value,
        "AXYDE",
        "write→seek(1)→write('XY')：pos 1-2 被覆盖为 AXYDE"
    );
    assert_eq!(
        sandbox.execute("globalThis.__truncResult").unwrap().value,
        "ABC",
        "truncate(3) 截断 ABCDEF → ABC"
    );
    assert_eq!(
        sandbox.execute("globalThis.__posResult").unwrap().value,
        "AABB",
        "write({{type:'write',position:2,data:'BB'}}) 带位置写入 AAAA → AABB"
    );
    assert_eq!(
        sandbox.execute("globalThis.__keepResult").unwrap().value,
        "orig+",
        "keepExistingData:true 保留原内容后 seek(4)+write('+') → orig+"
    );
}

// ── R3317：HTMLInputElement.valueAsDate（date/month/week/time）+ stepUp/stepDown（number/range）──
//
// 生产 always-on B-gen shim 路径（part03.js valueAsDate getter / stepUp·stepDown 方法 + part04.js
// valueAsDate setter）。spec https://html.spec.whatwg.org/multipage/input.html#dom-input-valueasdate 。
// valueAsDate：date/month/week/time 输入 value 串↔Date（UTC），其他 type→null；stepUp/stepDown：number/range
// 按 step 增减并 clamp [min,max]。同步 API（无 Promise），直读 Date 字段断言（toISOString/getTime/getFullYear）。

#[test]
fn test_input_value_as_date_r3317() {
    // valueAsDate getter+setter——date/month/week/time 四类型 ↔ Date（UTC），其他 type/null，空→null。
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
         <input id='d' type='date' value='2020-03-15'>\
         <input id='m' type='month' value='2021-06'>\
         <input id='t' type='time' value='13:45:30'>\
         <input id='w' type='week' value='2021-W01'>\
         <input id='de' type='date' value=''>\
         <input id='n' type='number' value='5'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // getter：date/month/time/week → Date（UTC）；空 date → null；number type → null。
    sandbox
        .execute(
            "globalThis.__dISO = document.querySelector('#d').valueAsDate.toISOString();\
             globalThis.__mISO = document.querySelector('#m').valueAsDate.toISOString();\
             globalThis.__tISO = document.querySelector('#t').valueAsDate.toISOString();\
             globalThis.__wISO = document.querySelector('#w').valueAsDate.toISOString();\
             globalThis.__deNull = String(document.querySelector('#de').valueAsDate === null);\
             globalThis.__nNull = String(document.querySelector('#n').valueAsDate === null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dISO").unwrap().value,
        "2020-03-15T00:00:00.000Z",
        "date 2020-03-15 → Date 当日 00:00:00 UTC"
    );
    assert_eq!(
        sandbox.execute("globalThis.__mISO").unwrap().value,
        "2021-06-01T00:00:00.000Z",
        "month 2021-06 → Date 当月 1 日 00:00:00 UTC"
    );
    assert_eq!(
        sandbox.execute("globalThis.__tISO").unwrap().value,
        "1970-01-01T13:45:30.000Z",
        "time 13:45:30 → Date 1970-01-01 当日 UTC"
    );
    assert_eq!(
        sandbox.execute("globalThis.__wISO").unwrap().value,
        "2021-01-04T00:00:00.000Z",
        "week 2021-W01 → Date 该年 ISO 第 1 周一（2021-01-04）UTC"
    );
    assert_eq!(
        sandbox.execute("globalThis.__deNull").unwrap().value,
        "true",
        "空 date value → valueAsDate=null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__nNull").unwrap().value,
        "true",
        "number type → valueAsDate=null（非 date/time）"
    );

    // setter：date input 设 Date → value 串格式化；month input 设 Date → YYYY-MM。
    sandbox
        .execute(
            "var el = document.querySelector('#d');\
             el.valueAsDate = new Date(Date.UTC(1999, 11, 31));\
             globalThis.__setD = el.value;\
             var em = document.querySelector('#m');\
             em.valueAsDate = new Date(Date.UTC(2022, 2, 1));\
             globalThis.__setM = em.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__setD").unwrap().value,
        "1999-12-31",
        "valueAsDate=Date(1999-12-31) → date value='1999-12-31'"
    );
    assert_eq!(
        sandbox.execute("globalThis.__setM").unwrap().value,
        "2022-03",
        "valueAsDate=Date(2022-03-01) → month value='2022-03'"
    );

    // setter 经 retained 当前值 mutation；内容属性保持默认值不变。
    let ms = mutations.lock().unwrap().clone();
    assert!(
        ms.iter().any(|mutation| matches!(mutation, DomMutation::SetFormValue { selector, value } if selector == "#d" && value == "1999-12-31")),
        "valueAsDate=Date records retained current value '1999-12-31'"
    );
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<input id=\"d\" type=\"date\" value=\"2020-03-15\">"),
        "valueAsDate setter must not change default value content attribute\n{out}"
    );
}

#[test]
fn test_input_step_up_down_r3317() {
    // stepUp(n)/stepDown(n)——number/range 按 step 增减并 clamp [min,max]；非 number/range→undefined no-op。
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
         <input id='a' type='number' value='10'>\
         <input id='b' type='number' value='10' step='5'>\
         <input id='c' type='range' value='5' min='0' max='10' step='2'>\
         <input id='d' type='text' value='10'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // stepUp 默认 n=1（step 缺省 1）；stepUp(3) step=5；stepDown(2) step=2 clamp max。
    sandbox
        .execute(
            "var a = document.querySelector('#a');\
             a.stepUp();\
             globalThis.__a = a.value;\
             var b = document.querySelector('#b');\
             b.stepUp(3);\
             globalThis.__b = b.value;\
             var c = document.querySelector('#c');\
             c.stepUp(2);\
             globalThis.__c = c.value;\
             c.stepDown(10);\
             globalThis.__c2 = c.value;\
             /* 非 number/range：stepUp 为 undefined（type gate 提前 return），调用抛 TypeError（real-browser 亦不暴露该方法）*/\
             var d = document.querySelector('#d');\
             globalThis.__dRet = String(typeof d.stepUp);\
             try { d.stepUp(); globalThis.__dThrow = 'no'; } catch (e) { globalThis.__dThrow = 'yes'; }\
             globalThis.__dVal = d.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__a").unwrap().value,
        "11",
        "number value=10 stepUp()（step 缺省 1）→ 11"
    );
    assert_eq!(
        sandbox.execute("globalThis.__b").unwrap().value,
        "25",
        "number value=10 step=5 stepUp(3) → 10 + 3*5 = 25"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c").unwrap().value,
        "9",
        "range value=5 min=0 max=10 step=2 stepUp(2) → 5 + 2*2 = 9"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c2").unwrap().value,
        "0",
        "range stepDown(10) 从 9 超 max 退到 min=0（clamp）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dRet").unwrap().value,
        "undefined",
        "text input 无 stepUp（非 number/range）→ typeof undefined"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dThrow").unwrap().value,
        "yes",
        "text input 调 stepUp() 抛 TypeError（undefined 非 callable，real-browser 同行为）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dVal").unwrap().value,
        "10",
        "text input 未调成功，value 保持 10"
    );
}

// ── R3318：navigator.serviceWorker（B-gen 生产 shim 移植）──
//
// 生产 always-on B-gen shim 路径（part02.js navigator.serviceWorker IIFE，R3318 从 A-gen dom_bridge.rs
// 移植——参照 R2821 Performance API 迁移模式：A-gen 为死代码无页面交互生产调用方，故补 B 代 shim）。
// spec https://w3c.github.io/ServiceWorker/。生命周期由 host `ServiceWorkerManager` 单一推进；纯 engine
// sandbox 没有 host bridge，register 必须 fail closed，不能恢复 timer 驱动的私有状态。

#[test]
fn test_navigator_service_worker_register_r3318() {
    // Engine-only shim has no host manager. Registration must fail closed
    // instead of recreating the removed timer-driven lifecycle simulation.
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__hasSW = String(navigator.serviceWorker !== undefined);\
             globalThis.__ctrlBefore = String(navigator.serviceWorker.controller === null);\
             globalThis.__readyIsPromise = String(navigator.serviceWorker.ready instanceof Promise);\
             navigator.serviceWorker.register('/sw.js').then(function () {\
               globalThis.__ok = 'ok';\
             }, function (err) {\
               globalThis.__ok = 'reject:' + String(err && err.message ? err.message : err);\
             });",
        )
        .unwrap();
    for i in 1..=6 {
        let _ = sandbox.execute(&format!("globalThis.__p{i} = 1;"));
    }

    assert_eq!(
        sandbox.execute("globalThis.__hasSW").unwrap().value,
        "true",
        "navigator.serviceWorker 存在（B-gen shim 移植后）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "reject:Service Worker host bridge unavailable",
        "engine-only shim must reject without a browser/WebView host bridge"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ctrlBefore").unwrap().value,
        "true",
        "register 前 controller 为 null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__readyIsPromise").unwrap().value,
        "true",
        "navigator.serviceWorker.ready 是 Promise"
    );

    sandbox
        .execute(
            "navigator.serviceWorker.getRegistration('/').then(function (reg) {\
               globalThis.__registrationMissing = String(reg === undefined);\
             });\
             navigator.serviceWorker.getRegistrations().then(function (regs) {\
               globalThis.__registrationCount = String(regs.length);\
             });",
        )
        .unwrap();
    for i in 1..=4 {
        let _ = sandbox.execute(&format!("globalThis.__q{i} = 1;"));
    }
    assert_eq!(
        sandbox.execute("globalThis.__registrationMissing").unwrap().value,
        "true",
        "failed registration must not create a private shim registration"
    );
    assert_eq!(
        sandbox.execute("globalThis.__registrationCount").unwrap().value,
        "0",
        "failed registration must leave the query surface empty"
    );
}

#[test]
fn test_navigator_service_worker_query_and_unregister_r3318() {
    // Query shape remains available without a host; invalid input still rejects
    // before host-bridge availability is considered.
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "(async function () {\
               try {\
                 var all = await navigator.serviceWorker.getRegistrations();\
                 globalThis.__numRegs = String(all.length);\
                 var ra = await navigator.serviceWorker.getRegistration('/a/');\
                 globalThis.__aIsUndef = String(ra === undefined);\
                 globalThis.__ok = 'ok';\
               } catch (err) {\
                 globalThis.__ok = 'reject:' + String(err && err.message ? err.message : err);\
               }\
             })();",
        )
        .unwrap();
    for i in 1..=8 {
        let _ = sandbox.execute(&format!("globalThis.__p{i} = 1;"));
    }

    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "ok",
        "serviceWorker query surface remains usable without a host bridge"
    );
    assert_eq!(
        sandbox.execute("globalThis.__numRegs").unwrap().value,
        "0",
        "engine-only shim has no registrations"
    );
    assert_eq!(
        sandbox.execute("globalThis.__aIsUndef").unwrap().value,
        "true",
        "getRegistration returns undefined when the host owns no registration"
    );

    // register 缺 scriptURL → reject TypeError。
    sandbox
        .execute(
            "navigator.serviceWorker.register('').then(function () {\
               globalThis.__emptyReject = 'resolved';\
             }, function (err) {\
               globalThis.__emptyReject = String(err && err.name ? err.name : err);\
             });",
        )
        .unwrap();
    for i in 1..=6 {
        let _ = sandbox.execute(&format!("globalThis.__r{i} = 1;"));
    }
    assert_eq!(
        sandbox.execute("globalThis.__emptyReject").unwrap().value,
        "TypeError",
        "register('') 缺 scriptURL → reject TypeError"
    );
}

// ── R3319：DOMRect + DOMRectReadOnly 全局构造器 + rect 工厂原型化 ──
//
// 生产 always-on B-gen shim 路径（part01.js DOMRectReadOnly/DOMRect 构造器 + _makeDomRect 工厂；
// part04 gBCR fallback / part05 _domRectFromId / part06 Range.getBoundingClientRect 迁移到 _makeDomRect）。
// spec https://drafts.fxtf.org/geometry/#DOMRect。**此前 B-gen 缺 DOMRect/DOMRectReadOnly 全局**——
// getBoundingClientRect / getClientRects / IO·RO entry / Range rect 全返无原型 plain object，
// 库 `rect instanceof DOMRectReadOnly` / `instanceof DOMRect` 恒 false（popper.js / floating-ui identity 检查失败）。
// DOMRect is-a DOMRectReadOnly（继承 prototype）。

#[test]
fn test_dom_rect_constructors_r3319() {
    // DOMRect/DOMRectReadOnly 全局存在 + 字段 + 派生属性 + instanceof 继承 + toJSON。
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 构造器存在 + 字段 + 派生属性（top/left/right/bottom 从 x/y/width/height 计算同步）。
    sandbox
        .execute(
            "globalThis.__hasDRO = String(typeof DOMRectReadOnly === 'function');\
             globalThis.__hasDR = String(typeof DOMRect === 'function');\
             var r = new DOMRect(10, 20, 100, 50);\
             globalThis.__rX = String(r.x);\
             globalThis.__rW = String(r.width);\
             globalThis.__rTop = String(r.top);\
             globalThis.__rLeft = String(r.left);\
             globalThis.__rRight = String(r.right);\
             globalThis.__rBottom = String(r.bottom);\
             /* instanceof 继承：DOMRect is-a DOMRectReadOnly */\
             globalThis.__drIsDR = String(r instanceof DOMRect);\
             globalThis.__drIsDRO = String(r instanceof DOMRectReadOnly);\
             /* DOMRectReadOnly 实例非 DOMRect（子类不反向）*/\
             var ro = new DOMRectReadOnly(1, 2, 3, 4);\
             globalThis.__roIsDRO = String(ro instanceof DOMRectReadOnly);\
             globalThis.__roIsDR = String(ro instanceof DOMRect);\
             /* toJSON 含全 8 字段 */\
             globalThis.__jsonKeys = Object.keys(r.toJSON()).sort().join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hasDRO").unwrap().value,
        "true",
        "DOMRectReadOnly 全局构造器存在（B-gen 补缺）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasDR").unwrap().value,
        "true",
        "DOMRect 全局构造器存在"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rX").unwrap().value,
        "10",
        "new DOMRect(10,...).x = 10"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rW").unwrap().value,
        "100",
        "new DOMRect(...,100,...).width = 100"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rTop").unwrap().value,
        "20",
        "派生 top = y = 20"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rLeft").unwrap().value,
        "10",
        "派生 left = x = 10"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rRight").unwrap().value,
        "110",
        "派生 right = x + width = 110"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rBottom").unwrap().value,
        "70",
        "派生 bottom = y + height = 70"
    );
    assert_eq!(
        sandbox.execute("globalThis.__drIsDR").unwrap().value,
        "true",
        "new DOMRect() instanceof DOMRect"
    );
    assert_eq!(
        sandbox.execute("globalThis.__drIsDRO").unwrap().value,
        "true",
        "DOMRect is-a DOMRectReadOnly（继承 prototype）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__roIsDRO").unwrap().value,
        "true",
        "new DOMRectReadOnly() instanceof DOMRectReadOnly"
    );
    assert_eq!(
        sandbox.execute("globalThis.__roIsDR").unwrap().value,
        "false",
        "DOMRectReadOnly 实例非 DOMRect（子类不反向）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__jsonKeys").unwrap().value,
        "bottom,height,left,right,top,width,x,y",
        "toJSON 含全 8 字段（x/y/top/left/right/bottom/width/height）"
    );
}

#[test]
fn test_get_bounding_client_rect_returns_domrect_r3319() {
    // getBoundingClientRect 返回值 instanceof DOMRect/DOMRectReadOnly（真实 rect 经 _domRectFromId +
    // 零 fallback 均原型化）。注册 mock rect bridge。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id='d'>x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // mock rect bridge：selector → 真实 rect "10,20,100,50"；handle（'__' 开头 detached）→ 空串（零 fallback）。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| match args.first() {
            Some(s) if s.starts_with("__") => String::new(),
            _ => "10,20,100,50".to_string(),
        }),
    );

    // 真实 rect 元素（#d，selector 命中）→ instanceof DOMRect/DOMRectReadOnly + 字段正确。
    sandbox
        .execute(
            "var r = document.querySelector('#d').getBoundingClientRect();\
             globalThis.__realIsDR = String(r instanceof DOMRect);\
             globalThis.__realIsDRO = String(r instanceof DOMRectReadOnly);\
             globalThis.__realRight = String(r.right);\
             /* detached createElement（handle，mock 返空 → 零 fallback rect）仍 instanceof DOMRect */\
             var e = document.createElement('div');\
             var z = e.getBoundingClientRect();\
             globalThis.__zeroIsDR = String(z instanceof DOMRect);\
             globalThis.__zeroW = String(z.width);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__realIsDR").unwrap().value,
        "true",
        "getBoundingClientRect（真实 rect）返回 instanceof DOMRect"
    );
    assert_eq!(
        sandbox.execute("globalThis.__realIsDRO").unwrap().value,
        "true",
        "getBoundingClientRect 返回 instanceof DOMRectReadOnly"
    );
    assert_eq!(
        sandbox.execute("globalThis.__realRight").unwrap().value,
        "110",
        "真实 rect 派生 right = x + width = 110"
    );
    assert_eq!(
        sandbox.execute("globalThis.__zeroIsDR").unwrap().value,
        "true",
        "detached createElement getBoundingClientRect（零 fallback）仍 instanceof DOMRect"
    );
    assert_eq!(
        sandbox.execute("globalThis.__zeroW").unwrap().value,
        "0",
        "detached 元素 rect width = 0（零 fallback）"
    );
}


/// R3254-C2/C3/C4：OPFS writable 数据完整性——负 position 拒绝、字符串 UTF-8 编码、
/// TypedArray 视图范围写入、abort 后 close 拒绝。
#[test]
fn test_opfs_writable_data_integrity_c2c3c4() {
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "navigator.storage.getDirectory().then(function (root) {               return root.getFileHandle('c.txt', { create: true });             }).then(function (fh) {               globalThis.__fh = fh;               return fh.createWritable();             }).then(function (w) {               globalThis.__w1 = w;               return w.seek(-5).then(function () { globalThis.__negSeek = 'resolved'; },                 function (e) { globalThis.__negSeek = 'rejected:' + e.name; });             }).then(function () {               return globalThis.__w1.write({ type: 'write', position: -1, data: new Uint8Array([1]) })                 .then(function () { globalThis.__negWrite = 'resolved'; },                   function (e) { globalThis.__negWrite = 'rejected:' + e.name; });             }).then(function () {               /* C3：'你' 应 UTF-8 编码为 3 字节 */               return globalThis.__w1.write('你').then(function () { return globalThis.__w1.close(); });             }).then(function () { return globalThis.__fh.getFile(); })             .then(function (file) { return file.arrayBuffer(); })             .then(function (ab) {               globalThis.__utf8len = String(new Uint8Array(ab).length);               /* C4：Uint16Array(8).subarray(2,4) 视图应只写 4 字节 */               return navigator.storage.getDirectory();             }).then(function (root) {               return root.getFileHandle('v.txt', { create: true });             }).then(function (fh) {               return fh.createWritable();             }).then(function (w) {               var big = new Uint16Array(8);               return w.write(big.subarray(2, 4)).then(function () { return w.close(); });             }).then(function () {               return navigator.storage.getDirectory();             }).then(function (root) { return root.getFileHandle('v.txt'); })             .then(function (fh) { return fh.getFile(); })             .then(function (file) { return file.arrayBuffer(); })             .then(function (ab) {               globalThis.__viewLen = String(new Uint8Array(ab).length);               /* C10：abort 后 close 拒绝 */               return navigator.storage.getDirectory();             }).then(function (root) {               return root.getFileHandle('a.txt', { create: true });             }).then(function (fh) {               return fh.createWritable();             }).then(function (w) {               return w.write('x').then(function () { return w.abort(); }).then(function () {                 return w.close().then(function () { globalThis.__abortClose = 'resolved'; },                   function (e) { globalThis.__abortClose = 'rejected:' + e.name; });               });             }).then(function () { globalThis.__ok = 'done'; },               function (e) { globalThis.__ok = 'fail:' + String(e && e.message ? e.message : e); });",
        )
        .unwrap();
    // pump microtask（Promise 链多轮 execute drain）。
    for _ in 0..12 {
        sandbox.execute("globalThis.__n = 1;").unwrap();
    }
    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "done",
        "OPFS 链应完成"
    );
    assert_eq!(
        sandbox.execute("globalThis.__negSeek").unwrap().value,
        "rejected:TypeError",
        "负 position seek 应拒绝 TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__negWrite").unwrap().value,
        "rejected:TypeError",
        "负 position write 应拒绝 TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__utf8len").unwrap().value,
        "3",
        "字符串 '你' 应 UTF-8 编码为 3 字节"
    );
    assert_eq!(
        sandbox.execute("globalThis.__viewLen").unwrap().value,
        "4",
        "subarray(2,4) 视图应只写 4 字节（非整块 16 字节）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__abortClose").unwrap().value,
        "rejected:TypeError",
        "abort 后 close 应拒绝"
    );
}

fn assert_text_insert_dispatches_beforeinput_then_input() {
    // https://w3c.github.io/input-events/#input-event-order-during-user-initiated-editing
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><input id='name' value=''></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/input-events".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var e=document.querySelector('#name'),log=[];\
             ['beforeinput','input'].forEach(function(t){e.addEventListener(t,function(ev){\
               log.push(ev.type+':'+ev.inputType+':'+ev.cancelable+':'+(ev instanceof InputEvent)+':'+e.value);\
             });});\
             __zw_text_input('#name','A');\
             globalThis.__inputLog=log.join('|');",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__inputLog").unwrap().value,
        "beforeinput:insertText:true:true:|input:insertText:false:true:A"
    );
    assert!(
        mutations
            .lock()
            .unwrap()
            .iter()
            .any(|mutation| matches!(mutation, DomMutation::SetFormValue { selector, value } if selector == "#name" && value == "A"))
    );
}

#[test]
fn text_delete_dispatches_beforeinput_then_input() {
    // https://w3c.github.io/input-events/#input-event-order-during-user-initiated-editing
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><input id='name' value='A'></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/input-events".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var e=document.querySelector('#name'),log=[];e.setSelectionRange(1,1);\
             ['beforeinput','input'].forEach(function(t){e.addEventListener(t,function(ev){\
               log.push(ev.type+':'+ev.inputType+':'+ev.cancelable+':'+String(ev.data)+':'+e.value);\
             });});\
             __zw_text_delete('#name');\
             globalThis.__deleteLog=log.join('|');",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__deleteLog").unwrap().value,
        "beforeinput:deleteContentBackward:true:null:A|input:deleteContentBackward:false:null:"
    );
    assert!(
        mutations
            .lock()
            .unwrap()
            .iter()
            .any(|mutation| matches!(mutation, DomMutation::SetFormValue { selector, value } if selector == "#name" && value.is_empty()))
    );
}

#[test]
fn label_control_resolution_supports_for_and_nested_controls() {
    let html = r#"<html><body>
        <label id="explicit" for="check">Explicit</label>
        <input id="check" type="checkbox">
        <label id="nested">Nested <input id="radio" type="radio"></label>
        <label id="hidden-label" for="hidden-input">Hidden</label>
        <input id="hidden-input" type="hidden">
        <div id="not-label"><input id="other"></div>
    </body></html>"#;

    assert_eq!(associated_label_control_selector(html, "#explicit").as_deref(), Some("#check"));
    assert_eq!(associated_label_control_selector(html, "#nested").as_deref(), Some("#radio"));
    assert_eq!(associated_label_control_selector(html, "#hidden-label"), None);
    assert_eq!(associated_label_control_selector(html, "#not-label"), None);
}

#[test]
fn text_control_reset_restores_unpolluted_default() {
    // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value
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
        "<html><body><form id='form'>\
         <input id='input' value='input-default'>\
         <textarea id='textarea'>textarea-default</textarea>\
         </form></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var input=document.getElementById('input');\
             var textarea=document.getElementById('textarea');\
             input.value='input-dirty';\
             textarea.value='textarea-dirty';\
             globalThis.__defaults=input.defaultValue+','+textarea.defaultValue;\
             document.getElementById('form').reset();\
             globalThis.__values=input.value+','+textarea.value;",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__defaults").unwrap().value,
        "input-default,textarea-default"
    );
    assert_eq!(
        sandbox.execute("globalThis.__values").unwrap().value,
        "input-default,textarea-default"
    );
}

#[test]
fn non_text_input_rejects_text_selection_operations() {
    // https://html.spec.whatwg.org/multipage/input.html#concept-input-apply
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
         <input id='number' type='number' value='42'>\
         <input id='checkbox' type='checkbox'>\
         <input id='email' type='email' value='a@example.test'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "function errorName(fn){try{fn();return 'none';}catch(error){return error.name;}}\
             var number=document.getElementById('number');\
             var checkbox=document.getElementById('checkbox');\
             var email=document.getElementById('email');\
             globalThis.__getters=[number.selectionStart,number.selectionEnd,\
               number.selectionDirection,checkbox.selectionStart,email.selectionStart].map(String).join(',');\
             globalThis.__errors=[\
               errorName(function(){number.setSelectionRange(0,1);}),\
               errorName(function(){number.selectionStart=0;}),\
               errorName(function(){number.selectionEnd=1;}),\
               errorName(function(){number.selectionDirection='forward';}),\
               errorName(function(){number.setRangeText('x');})\
             ].join(',');\
             globalThis.__value=number.value;",
        )
        .unwrap();

    assert_eq!(sandbox.execute("globalThis.__getters").unwrap().value, "null,null,null,null,null");
    assert_eq!(
        sandbox.execute("globalThis.__errors").unwrap().value,
        "InvalidStateError,InvalidStateError,InvalidStateError,InvalidStateError,InvalidStateError"
    );
    assert_eq!(sandbox.execute("globalThis.__value").unwrap().value, "42");
}
