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
