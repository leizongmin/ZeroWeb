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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
                 /* removeEntry 删子目录 */\
                 return root.removeEntry('docs');\
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
// spec https://w3c.github.io/ServiceWorker/。headless 无真 SW 执行环境（无独立 worker 线程/真 fetch 拦截/真
// install·activate 事件派发）→ 进程内注册表近似：register 返 Promise<registration>，setTimeout(0) 模拟
// install→waiting→active 异步生命周期。getRegistration/getRegistrations/ready/unregister 完整查询面。

#[test]
fn test_navigator_service_worker_register_r3318() {
    // navigator.serviceWorker.register → Promise<registration> + scope 派生 + install→active 异步生命周期。
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

    // 存在性 + register 返 Promise<registration> + scope 默认派生 + controller 激活前为 null。
    sandbox
        .execute(
            "globalThis.__hasSW = String(navigator.serviceWorker !== undefined);\
             globalThis.__ctrlBefore = String(navigator.serviceWorker.controller === null);\
             globalThis.__readyIsPromise = String(navigator.serviceWorker.ready instanceof Promise);\
             navigator.serviceWorker.register('/sw.js').then(function (reg) {\
               globalThis.__scope = String(reg.scope);\
               globalThis.__hasUnregister = String(typeof reg.unregister === 'function');\
               globalThis.__ok = 'ok';\
             }, function (err) {\
               globalThis.__ok = 'reject:' + String(err && err.message ? err.message : err);\
             });",
        )
        .unwrap();
    // pump microtask（register Promise + setTimeout(0) 生命周期推进）——多轮 execute drain。
    for i in 1..=10 {
        let _ = sandbox.execute(&format!("globalThis.__p{i} = 1;"));
    }

    assert_eq!(
        sandbox.execute("globalThis.__hasSW").unwrap().value,
        "true",
        "navigator.serviceWorker 存在（B-gen shim 移植后）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "ok",
        "register 返 Promise 成功 resolve"
    );
    assert_eq!(
        sandbox.execute("globalThis.__scope").unwrap().value,
        "/",
        "register('/sw.js') 默认 scope = scriptURL 所在目录 '/'"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasUnregister").unwrap().value,
        "true",
        "registration.unregister 方法存在"
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

    // 生命周期推进后：active 已激活 + controller 非空。
    sandbox
        .execute(
            "var r = null;\
             navigator.serviceWorker.getRegistration('/').then(function (reg) { r = reg; });",
        )
        .unwrap();
    for i in 1..=6 {
        let _ = sandbox.execute(&format!("globalThis.__q{i} = 1;"));
    }
    sandbox
        .execute(
            "globalThis.__activeState = (r && r.active) ? String(r.active.state) : 'null';\
             globalThis.__waitingCleared = String(r ? r.waiting === null : true);\
             globalThis.__installingCleared = String(r ? r.installing === null : true);\
             globalThis.__ctrlAfter = navigator.serviceWorker.controller ? String(navigator.serviceWorker.controller.state) : 'null';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__activeState").unwrap().value,
        "activated",
        "生命周期推进后 registration.active.state = 'activated'"
    );
    assert_eq!(
        sandbox.execute("globalThis.__waitingCleared").unwrap().value,
        "true",
        "激活后 waiting 字段已清空（active 接管）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__installingCleared").unwrap().value,
        "true",
        "激活后 installing 字段已清空"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ctrlAfter").unwrap().value,
        "activated",
        "激活后 navigator.serviceWorker.controller.state = 'activated'"
    );
}

#[test]
fn test_navigator_service_worker_query_and_unregister_r3318() {
    // getRegistration/getRegistrations/unregister + register 缺 scriptURL reject + 显式 scope。
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

    // async/await 扁平化（参照 R3315 OPFS 模式）：2 个显式 scope 注册 + getRegistrations 计数 +
    // getRegistration(scope) 命中 + unregister + 空 scriptURL reject。
    sandbox
        .execute(
            "(async function () {\
               try {\
                 /* register 2：显式 scope */\
                 await navigator.serviceWorker.register('/a/sw.js', { scope: '/a/' });\
                 await navigator.serviceWorker.register('/b/sw.js', { scope: '/b/' });\
                 var all = await navigator.serviceWorker.getRegistrations();\
                 globalThis.__numRegs = String(all.length);\
                 /* getRegistration('/a/') 命中该 scope */\
                 var ra = await navigator.serviceWorker.getRegistration('/a/');\
                 globalThis.__aScope = ra ? String(ra.scope) : 'undef';\
                 /* 不存在的 scope → undefined */\
                 var rz = await navigator.serviceWorker.getRegistration('/zzz/');\
                 globalThis.__zIsUndef = String(rz === undefined);\
                 /* unregister '/a/' */\
                 var ok = await ra.unregister();\
                 globalThis.__unregOk = String(ok);\
                 var all2 = await navigator.serviceWorker.getRegistrations();\
                 globalThis.__numAfter = String(all2.length);\
                 globalThis.__ok = 'ok';\
               } catch (err) {\
                 globalThis.__ok = 'reject:' + String(err && err.message ? err.message : err);\
               }\
             })();",
        )
        .unwrap();
    // pump microtask（async 函数每 await 让出）。
    for i in 1..=20 {
        let _ = sandbox.execute(&format!("globalThis.__p{i} = 1;"));
    }

    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "ok",
        "serviceWorker 注册/查询/unregister 全链成功"
    );
    assert_eq!(
        sandbox.execute("globalThis.__numRegs").unwrap().value,
        "2",
        "注册 2 个 SW 后 getRegistrations().length = 2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__aScope").unwrap().value,
        "/a/",
        "getRegistration('/a/') 返回 scope='/a/' 的 registration"
    );
    assert_eq!(
        sandbox.execute("globalThis.__zIsUndef").unwrap().value,
        "true",
        "getRegistration('/zzz/') 不存在 → undefined"
    );
    assert_eq!(
        sandbox.execute("globalThis.__unregOk").unwrap().value,
        "true",
        "unregister() 返 Promise<true>（spec）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__numAfter").unwrap().value,
        "1",
        "unregister('/a/') 后 getRegistrations().length = 1"
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

