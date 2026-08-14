#[test]
fn test_dom_parser_r2790() {
    // R2790：DOMParser.parseFromString → 只读 Document。host `__zw_parse_html_query` 回调返 JSON 元素
    // 快照；shim 包 _zwParsedDoc + 只读 _zwParseEl（querySelector/getElementById/body/textContent/
    // getAttribute/子树 query）。**已知限制**：只读（无 mutation）、XML/SVG 按 HTML 解析、innerHTML 派生。
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
            "globalThis.__d = new DOMParser().parseFromString(\n\
             '<html><head><title>T</title></head><body><div id=\"main\"><p class=\"hi\">hello <b>world</b></p><span id=\"s\">x</span></div></body></html>',\n\
             'text/html');",
        )
        .unwrap();
    // typeof + instance（DOMParser 可构造、parseFromString 返对象）。
    assert_eq!(sandbox.execute("typeof DOMParser").unwrap().value, "function");
    assert_eq!(sandbox.execute("typeof __d.querySelector").unwrap().value, "function");
    // querySelector：tagName 大写、id、className。
    assert_eq!(
        sandbox.execute("__d.querySelector('#main').tagName").unwrap().value,
        "DIV"
    );
    assert_eq!(sandbox.execute("__d.querySelector('p').className").unwrap().value, "hi");
    // textContent（含后代文本，spec 一致）。
    assert_eq!(
        sandbox.execute("__d.querySelector('p').textContent").unwrap().value,
        "hello world"
    );
    // getElementById + 无匹配返 null。
    assert_eq!(
        sandbox.execute("__d.getElementById('s').tagName").unwrap().value,
        "SPAN"
    );
    assert_eq!(
        sandbox
            .execute("String(__d.getElementById('nope') === null)")
            .unwrap()
            .value,
        "true"
    );
    // querySelectorAll（多个匹配）。
    assert_eq!(
        sandbox.execute("__d.querySelectorAll('span').length").unwrap().value,
        "1"
    );
    assert_eq!(
        sandbox.execute("__d.querySelectorAll('#main b').length").unwrap().value,
        "1"
    );
    // body / documentElement / head 非空。
    assert_eq!(
        sandbox
            .execute("String(__d.body !== null && __d.documentElement !== null && __d.head !== null)")
            .unwrap()
            .value,
        "true"
    );
    // 子树 querySelector（element-proxy 上）：#main 内 <b>。
    assert_eq!(
        sandbox
            .execute("__d.querySelector('#main').querySelector('b').textContent")
            .unwrap()
            .value,
        "world"
    );
    // getAttribute / hasAttribute。
    assert_eq!(
        sandbox
            .execute("__d.querySelector('p').getAttribute('class')")
            .unwrap()
            .value,
        "hi"
    );
    assert_eq!(
        sandbox
            .execute("String(__d.querySelector('p').hasAttribute('class'))")
            .unwrap()
            .value,
        "true"
    );
    assert_eq!(
        sandbox
            .execute("String(__d.querySelector('p').getAttribute('none'))")
            .unwrap()
            .value,
        "null"
    );
    // innerHTML 由 outerHTML 派生（含 <p 子标签）。
    assert_eq!(
        sandbox
            .execute("String(__d.querySelector('#main').innerHTML.indexOf('<p') >= 0)")
            .unwrap()
            .value,
        "true"
    );
    // getElementsByTagName。
    assert_eq!(
        sandbox
            .execute("__d.getElementsByTagName('span').length")
            .unwrap()
            .value,
        "1"
    );
    // mimeType 记录。
    assert_eq!(sandbox.execute("__d.mimeType").unwrap().value, "text/html");
}

#[test]
fn test_file_reader_r2791() {
    // R2791：FileReader（异步读 Blob，文件上传/图片预览高频）。纯 JS builds on Blob.text()/arrayBuffer()
    //（R2789）+ btoa（R2770）。readAsText/ArrayBuffer/BinaryString/DataURL + abort + onload/onloadend/
    // onloadstart 事件 + result/readyState。**readAsDataURL 为 Blob 未覆盖唯一能力**。
    // **已知限制**：loadstart 同步；abort best-effort（不中断 in-flight Promise）；无 addEventListener。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + 常量（EMPTY/LOADING/DONE，静态 + 原型）。
    assert_eq!(sandbox.execute("typeof FileReader").unwrap().value, "function");
    assert_eq!(sandbox.execute("String(FileReader.EMPTY)").unwrap().value, "0");
    assert_eq!(sandbox.execute("String(FileReader.DONE)").unwrap().value, "2");
    // readAsText：result=Blob 文本；onload/onloadend 在 execute 末 checkpoint drain → 下 execute 可读。
    sandbox
        .execute(
            "globalThis.__r = '(pending)'; globalThis.__st = [];\
             var rd = new FileReader();\
             rd.onloadstart = function(){ globalThis.__st.push('start'); };\
             rd.onload = function(e){ globalThis.__st.push('load'); globalThis.__r = e.target.result; };\
             rd.onloadend = function(){ globalThis.__st.push('end'); };\
             rd.readAsText(new Blob(['hello',' ','world']));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__r)").unwrap().value, "hello world");
    assert_eq!(
        sandbox.execute("globalThis.__st.join(',')").unwrap().value,
        "start,load,end"
    );
    assert_eq!(sandbox.execute("String(rd.readyState)").unwrap().value, "2"); // DONE
    assert_eq!(sandbox.execute("String(rd.result)").unwrap().value, "hello world");
    // readAsArrayBuffer：result=Uint8Array（'AB' → [65,66]）。
    sandbox
        .execute(
            "globalThis.__ab = '(pending)';\
             var rd2 = new FileReader();\
             rd2.onload = function(e){ globalThis.__ab = e.target.result.length + ':' + e.target.result[0]; };\
             rd2.readAsArrayBuffer(new Blob(['AB']));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ab)").unwrap().value, "2:65");
    // readAsDataURL：data:<type>;base64,<b64>（'hi' → btoa='aGk='）。
    sandbox
        .execute(
            "globalThis.__url = '(pending)';\
             var rd3 = new FileReader();\
             rd3.onload = function(e){ globalThis.__url = e.target.result; };\
             rd3.readAsDataURL(new Blob(['hi'], {type:'text/plain'}));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__url)").unwrap().value,
        "data:text/plain;base64,aGk="
    );
    // readAsBinaryString：逐字节 Latin-1 串（'AB' → 'AB'）。
    sandbox
        .execute(
            "globalThis.__bs = '(pending)';\
             var rd4 = new FileReader();\
             rd4.onload = function(e){ globalThis.__bs = e.target.result; };\
             rd4.readAsBinaryString(new Blob(['AB']));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__bs)").unwrap().value, "AB");
    // abort：best-effort——在 LOADING（readAsText Promise 未 drain）时 abort → 派发 abort + loadend。
    // 注：本 execute 内 loadstart 已同步派发，abort 置 DONE；readAsText 的 Promise 仍会在 checkpoint
    // drain（best-effort，不中断）——测 abort 本身 no-throw + readyState=DONE。
    sandbox
        .execute(
            "globalThis.__ab_st = '(pending)';\
             var rd5 = new FileReader();\
             rd5.onabort = function(){ globalThis.__ab_st = 'aborted'; };\
             rd5.readAsText(new Blob(['x']));\
             rd5.abort();\
             globalThis.__ab_ready = rd5.readyState;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ab_ready)").unwrap().value, "2");
    assert_eq!(sandbox.execute("String(globalThis.__ab_st)").unwrap().value, "aborted");
}

#[test]
fn test_file_r2792() {
    // R2792：File（=Blob 子类 + name/lastModified，文件上传构造高频）。完成 Blob→File→FileReader→FormData
    // 文件处理簇。constructor 复用 Blob 构造；prototype=Object.create(Blob.prototype) 继承 slice/text/
    // arrayBuffer；File is-a Blob 故 FormData/FileReader 自动互通。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + instanceof（File 同时是 Blob 和 File）。
    assert_eq!(sandbox.execute("typeof File").unwrap().value, "function");
    assert_eq!(
        sandbox
            .execute(
                "var f = new File(['hello'], 'greet.txt'); String(f instanceof File) + ',' + String(f instanceof Blob)"
            )
            .unwrap()
            .value,
        "true,true"
    );
    // name + 继承 size/type（type 从 options 取，默认 ''）。
    assert_eq!(sandbox.execute("f.name").unwrap().value, "greet.txt");
    assert_eq!(sandbox.execute("f.size").unwrap().value, "5");
    assert_eq!(sandbox.execute("f.type").unwrap().value, "");
    // type 从 options.type 取。
    assert_eq!(
        sandbox
            .execute("new File(['x'], 'a.txt', {type:'text/plain'}).type")
            .unwrap()
            .value,
        "text/plain"
    );
    // lastModified：默认 Date.now()（非负整数）；显式值透传（含 0）。
    sandbox
        .execute("globalThis.__lm = new File(['x'], 'a').lastModified;")
        .unwrap();
    let lm = sandbox.execute("String(globalThis.__lm >= 0)").unwrap().value;
    assert_eq!(lm, "true", "默认 lastModified 应为非负（Date.now）");
    assert_eq!(
        sandbox
            .execute("new File(['x'], 'a', {lastModified:0}).lastModified")
            .unwrap()
            .value,
        "0"
    );
    assert_eq!(
        sandbox
            .execute("new File(['x'], 'a', {lastModified:1700000000000}).lastModified")
            .unwrap()
            .value,
        "1700000000000"
    );
    // lastModifiedDate 为 Date 实例。
    assert_eq!(
        sandbox
            .execute("String(new File(['x'],'a', {lastModified:0}).lastModifiedDate instanceof Date)")
            .unwrap()
            .value,
        "true"
    );
    // 继承 Blob 方法：text() 返 Promise（execute 末 drain → 下 execute 可读）。
    sandbox
        .execute(
            "globalThis.__ft = '(pending)';\
             new File(['hello',' ','file']).text().then(function(s){ globalThis.__ft = s; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ft)").unwrap().value, "hello file");
    // 继承 slice（返 Blob，size clamp）。
    assert_eq!(
        sandbox
            .execute("new File(['ZeroWeb'],'f').slice(1,4).size")
            .unwrap()
            .value,
        "3"
    );
    // 互通：FormData.append(name, file) — File is-a Blob，value 经 String() 归一（spec 非 Blob 转 USVString；
    // 本 FormData 实现恒字符串化，File 作为整体入列不读内容——与 fetch POST 一起为 follow-up，此处仅验 no-throw）。
    sandbox
        .execute("var fd = new FormData(); fd.append('upload', new File(['data'],'up.txt'));")
        .unwrap();
    assert_eq!(sandbox.execute("String(fd.has('upload'))").unwrap().value, "true");
    // 互通：FileReader.readAsDataURL(file) — File is-a Blob，readAsDataURL 读其字节。
    sandbox
        .execute(
            "globalThis.__furl = '(pending)';\
             var rd = new FileReader();\
             rd.onload = function(e){ globalThis.__furl = e.target.result; };\
             rd.readAsDataURL(new File(['hi'],'h.txt',{type:'text/plain'}));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__furl)").unwrap().value,
        "data:text/plain;base64,aGk="
    );
    // 无 new 调用亦可构造。
    assert_eq!(
        sandbox
            .execute("String(File(['x'],'y') instanceof File)")
            .unwrap()
            .value,
        "true"
    );
}

#[test]
fn test_crypto_subtle_digest_r2793() {
    // R2793：crypto.subtle.digest（SHA-1/256/384/512，SRI/JWT/内容哈希高频）。host RustCrypto sha1/sha2。
    // 返 Promise<ArrayBuffer>（Uint8Array）。TDD 用已知向量 hex 锚定（SHA-256('abc')/SHA-1('abc')/空输入）。
    // scope 仅 digest（HMAC/sign/encrypt defer）。
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

    // hex 辅助：Uint8Array → 低位补零 hex 串（execute 末 Promise drain → 下 execute 读）。
    let mut hex_of = |expr: &str| -> String {
        sandbox.execute(expr).unwrap();
        sandbox
            .execute("Array.from(globalThis.__dig).map(function(b){return ('0'+b.toString(16)).slice(-2);}).join('')")
            .unwrap()
            .value
    };
    // SHA-256('abc') = ba7816bf...015ad（NIST FIPS 180-4 测试向量）。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest('SHA-256', new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // SHA-1('abc') = a9993e36...0d89d（NIST 测试向量）。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest('SHA-1', new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    // SHA-512('abc') 前 16 hex（ddaf35a1...）。
    let h512 = hex_of(
        "globalThis.__dig='(pending)';\
         crypto.subtle.digest('SHA-512', new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });",
    );
    assert_eq!(h512.len(), 128); // 64 字节
    assert_eq!(&h512[..16], "ddaf35a193617aba");
    // SHA-256('') 空输入 = e3b0c442...b855。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest('SHA-256', new Uint8Array(0)).then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    // algo 对象形式 {name:'SHA-256'} + SHA-256 等价 SHA256（无连字符）。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest({name:'SHA-256'}, new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // 字符串输入经 TextEncoder（'abc' 同 UTF-8 字节）。
    assert_eq!(
        hex_of(
            "globalThis.__dig='(pending)';\
             crypto.subtle.digest('SHA256', 'abc').then(function(b){ globalThis.__dig = new Uint8Array(b); });"
        ),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // SHA-384('abc') 长度 96 hex（48 字节）。
    let h384 = hex_of(
        "globalThis.__dig='(pending)';\
         crypto.subtle.digest('SHA-384', new TextEncoder().encode('abc')).then(function(b){ globalThis.__dig = new Uint8Array(b); });",
    );
    assert_eq!(h384.len(), 96);
    assert_eq!(&h384[..16], "cb00753f45a35e8b");
    // unsupported algo → reject NotSupportedError（catch 验证 name）。
    sandbox
        .execute(
            "globalThis.__err='(pending)';\
             crypto.subtle.digest('MD5', new Uint8Array([1])).catch(function(e){ globalThis.__err = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__err)").unwrap().value,
        "NotSupportedError"
    );
}

#[test]
fn test_crypto_subtle_hmac_r2955() {
    // R2955：crypto.subtle HMAC（sign/verify/importKey + CryptoKey 对象）。JWT HS256 / 请求签名 / webhook
    // 校验高频。host 手写 HMAC（RFC 2104，复用 sha1/sha2 原语）。TDD 用 RFC 4231 测试向量锚定（TC1/TC2
    // SHA-256 + SHA-1 已知向量），+ verify 正/篡改/错长 + importKey/sign 错误路径。
    // https://datatracker.ietf.org/doc/html/rfc4231#section-4  https://w3c.github.io/webcrypto/
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

    // importKey + CryptoKey 字段：type="secret"，algorithm.name/hash，extractable=false，usages 去重（'sign' 重复 → 单）。
    sandbox
        .execute(
            "globalThis.__k=null;\
             crypto.subtle.importKey('raw', new Uint8Array(20).fill(0x0b), {name:'HMAC',hash:'SHA-256'}, false, ['sign','verify','sign'])\
               .then(function(k){ globalThis.__k = k; });",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute(
                "String(globalThis.__k && globalThis.__k.type === 'secret'\
                   && globalThis.__k.algorithm.name === 'HMAC'\
                   && globalThis.__k.algorithm.hash === 'SHA-256'\
                   && globalThis.__k.extractable === false\
                   && globalThis.__k.usages.join(',') === 'sign,verify')"
            )
            .unwrap()
            .value,
        "true"
    );

    // sign hex 辅助：执行 importKey→sign 链，下 execute 读 globalThis.__mac hex（微任务链 execute 末排空）。
    let mut hex_mac = |import_and_sign: &str| -> String {
        sandbox.execute(import_and_sign).unwrap();
        sandbox
            .execute("Array.from(globalThis.__mac).map(function(b){return ('0'+b.toString(16)).slice(-2);}).join('')")
            .unwrap()
            .value
    };

    // RFC 4231 TC1：key=0x0b×20，data="Hi There"，SHA-256 → b0344c61...cff7。
    assert_eq!(
        hex_mac(
            "globalThis.__mac='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(20).fill(0x0b), {name:'HMAC',hash:'SHA-256'}, false, ['sign'])\
               .then(function(k){ return crypto.subtle.sign('HMAC', k, new TextEncoder().encode('Hi There')); })\
               .then(function(b){ globalThis.__mac = new Uint8Array(b); });"
        ),
        "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
    );
    // RFC 4231 TC2：key="Jefe"，data="what do ya want for nothing?"，SHA-256 → 5bdcc146...ec3843。
    assert_eq!(
        hex_mac(
            "globalThis.__mac='(pending)';\
             crypto.subtle.importKey('raw', new TextEncoder().encode('Jefe'), {name:'HMAC',hash:'SHA-256'}, false, ['sign'])\
               .then(function(k){ return crypto.subtle.sign('HMAC', k, 'what do ya want for nothing?'); })\
               .then(function(b){ globalThis.__mac = new Uint8Array(b); });"
        ),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
    // HMAC-SHA-1：key="key"，data="The quick brown fox jumps over the lazy dog" → de7c9b85...db4d9（20 字节）。
    assert_eq!(
        hex_mac(
            "globalThis.__mac='(pending)';\
             crypto.subtle.importKey('raw', new TextEncoder().encode('key'), {name:'HMAC',hash:'SHA-1'}, false, ['sign'])\
               .then(function(k){ return crypto.subtle.sign('HMAC', k, 'The quick brown fox jumps over the lazy dog'); })\
               .then(function(b){ globalThis.__mac = new Uint8Array(b); });"
        ),
        "de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9"
    );
    // HMAC-SHA-512 长度 128 hex（64 字节），TC1 key/data。
    let h512 = hex_mac(
        "globalThis.__mac='(pending)';\
         crypto.subtle.importKey('raw', new Uint8Array(20).fill(0x0b), {name:'HMAC',hash:'SHA-512'}, false, ['sign'])\
           .then(function(k){ return crypto.subtle.sign('HMAC', k, 'Hi There'); })\
           .then(function(b){ globalThis.__mac = new Uint8Array(b); });",
    );
    assert_eq!(h512.len(), 128);
    assert_eq!(&h512[..16], "87aa7cdea5ef619d");

    // verify 正确签名 → true（TC1 的 mac）。
    sandbox
        .execute(
            "globalThis.__v='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(20).fill(0x0b), {name:'HMAC',hash:'SHA-256'}, false, ['verify'])\
               .then(function(k){\
                 var sig = new Uint8Array([0xb0,0x34,0x4c,0x61,0xd8,0xdb,0x38,0x53,0x5c,0xa8,0xaf,0xce,0xaf,0x0b,0xf1,0x2b,0x88,0x1d,0xc2,0x00,0xc9,0x83,0x3d,0xa7,0x26,0xe9,0x37,0x6c,0x2e,0x32,0xcf,0xf7]);\
                 return crypto.subtle.verify('HMAC', k, sig, new TextEncoder().encode('Hi There'));\
               }).then(function(ok){ globalThis.__v = String(ok); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__v)").unwrap().value, "true");
    // verify 篡改签名（首字节改 0x00）→ false。
    sandbox
        .execute(
            "globalThis.__v2='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(20).fill(0x0b), {name:'HMAC',hash:'SHA-256'}, false, ['verify'])\
               .then(function(k){\
                 var sig = new Uint8Array([0x00,0x34,0x4c,0x61,0xd8,0xdb,0x38,0x53,0x5c,0xa8,0xaf,0xce,0xaf,0x0b,0xf1,0x2b,0x88,0x1d,0xc2,0x00,0xc9,0x83,0x3d,0xa7,0x26,0xe9,0x37,0x6c,0x2e,0x32,0xcf,0xf7]);\
                 return crypto.subtle.verify('HMAC', k, sig, new TextEncoder().encode('Hi There'));\
               }).then(function(ok){ globalThis.__v2 = String(ok); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__v2)").unwrap().value, "false");
    // verify 错误长度签名（3 字节）→ false。
    sandbox
        .execute(
            "globalThis.__v3='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(20).fill(0x0b), {name:'HMAC',hash:'SHA-256'}, false, ['verify'])\
               .then(function(k){ return crypto.subtle.verify('HMAC', k, new Uint8Array([1,2,3]), 'Hi There'); })\
               .then(function(ok){ globalThis.__v3 = String(ok); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__v3)").unwrap().value, "false");

    // importKey 非法 usage（'encrypt' 不属 {sign,verify}）→ reject SyntaxError。
    sandbox
        .execute(
            "globalThis.__e1='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(4), {name:'HMAC',hash:'SHA-256'}, false, ['encrypt'])\
               .catch(function(e){ globalThis.__e1 = e.name; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e1)").unwrap().value, "SyntaxError");
    // importKey 非 raw 格式（'jwk'）→ reject NotSupportedError。
    sandbox
        .execute(
            "globalThis.__e2='(pending)';\
             crypto.subtle.importKey('jwk', {}, {name:'HMAC',hash:'SHA-256'}, false, ['sign'])\
               .catch(function(e){ globalThis.__e2 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e2)").unwrap().value,
        "NotSupportedError"
    );
    // importKey 不支持的算法（'AES-CBC'，本实现仅 HMAC/PBKDF2/AES-GCM）→ reject NotSupportedError。
    sandbox
        .execute(
            "globalThis.__e4='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-CBC'}, false, ['encrypt'])\
               .catch(function(e){ globalThis.__e4 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e4)").unwrap().value,
        "NotSupportedError"
    );
    // sign 无 "sign" usage（仅 'verify'）→ reject InvalidAccessError。
    sandbox
        .execute(
            "globalThis.__e3='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(4), {name:'HMAC',hash:'SHA-256'}, false, ['verify'])\
               .then(function(k){ return crypto.subtle.sign('HMAC', k, 'x'); })\
               .catch(function(e){ globalThis.__e3 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e3)").unwrap().value,
        "InvalidAccessError"
    );
}

#[test]
fn test_crypto_subtle_pbkdf2_r2956() {
    // R2956：crypto.subtle deriveBits("PBKDF2", ...)——PBKDF2-HMAC-SHA-1/256/384/512 密码派生密钥。
    // 复用 R2955 compute_hmac 作 PRF。TDD 用 RFC 6070 SHA-1 向量（c=1/2/4096）+ SHA-256 已知向量锚定，
    // + 多块自一致（dkLen=64 首 32 字节 == dkLen=32 输出）+ deriveBits/importKey 错误路径。
    // https://datatracker.ietf.org/doc/html/rfc2898#section-5.2  https://datatracker.ietf.org/doc/html/rfc6070
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

    // derive hex 辅助：执行 importKey→deriveBits 链，下 execute 读 globalThis.__dk hex。
    let mut hex_dk = |derive: &str| -> String {
        sandbox.execute(derive).unwrap();
        sandbox
            .execute("Array.from(globalThis.__dk).map(function(b){return ('0'+b.toString(16)).slice(-2);}).join('')")
            .unwrap()
            .value
    };

    // RFC 6070 SHA-1：P="password", S="salt", c=1, dkLen=20 → 0c60c80f...e037a6。
    assert_eq!(
        hex_dk(
            "globalThis.__dk='(pending)';\
             crypto.subtle.importKey('raw', new TextEncoder().encode('password'), {name:'PBKDF2'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-1',salt:new TextEncoder().encode('salt'),iterations:1}, k, 160); })\
               .then(function(b){ globalThis.__dk = new Uint8Array(b); });"
        ),
        "0c60c80f961f0e71f3a9b524af6012062fe037a6"
    );
    // RFC 6070 SHA-1：c=2 → ea6c014d...de8957。
    assert_eq!(
        hex_dk(
            "globalThis.__dk='(pending)';\
             crypto.subtle.importKey('raw', 'password', {name:'PBKDF2'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-1',salt:'salt',iterations:2}, k, 160); })\
               .then(function(b){ globalThis.__dk = new Uint8Array(b); });"
        ),
        "ea6c014dc72d6f8ccd1ed92ace1d41f0d8de8957"
    );
    // RFC 6070 SHA-1：c=4096（测迭代循环）→ 4b007901...429c1。
    assert_eq!(
        hex_dk(
            "globalThis.__dk='(pending)';\
             crypto.subtle.importKey('raw', 'password', {name:'PBKDF2'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-1',salt:'salt',iterations:4096}, k, 160); })\
               .then(function(b){ globalThis.__dk = new Uint8Array(b); });"
        ),
        "4b007901b765489abead49d926f721d065a429c1"
    );
    // PBKDF2-HMAC-SHA-256：P="password", S="salt", c=1, dkLen=32 → 120fb6cf...70be17b。
    assert_eq!(
        hex_dk(
            "globalThis.__dk='(pending)';\
             crypto.subtle.importKey('raw', 'password', {name:'PBKDF2'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-256',salt:'salt',iterations:1}, k, 256); })\
               .then(function(b){ globalThis.__dk = new Uint8Array(b); });"
        ),
        "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b"
    );
    // PBKDF2-HMAC-SHA-256：c=2 → ae4d0c95...474c43。
    assert_eq!(
        hex_dk(
            "globalThis.__dk='(pending)';\
             crypto.subtle.importKey('raw', 'password', {name:'PBKDF2'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-256',salt:'salt',iterations:2}, k, 256); })\
               .then(function(b){ globalThis.__dk = new Uint8Array(b); });"
        ),
        "ae4d0c95af6b46d32d0adff928f06dd02a303f8ef3c251dfd6e2d85a95474c43"
    );
    // 多块自一致：dkLen=64（2 个 SHA-256 块）首 32 字节 == dkLen=32 输出（T_1 确定性，证 INT_32_BE 块序 + 截断正确）。
    let dk64 = hex_dk(
        "globalThis.__dk='(pending)';\
         crypto.subtle.importKey('raw', 'password', {name:'PBKDF2'}, false, ['deriveBits'])\
           .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-256',salt:'salt',iterations:1}, k, 512); })\
           .then(function(b){ globalThis.__dk = new Uint8Array(b); });",
    );
    assert_eq!(dk64.len(), 128); // 64 字节
    assert_eq!(
        &dk64[..64],
        "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b"
    );

    // importKey 字段：PBKDF2 CryptoKey type="secret"，algorithm.name="PBKDF2"，usages。
    sandbox
        .execute(
            "globalThis.__k=null;\
             crypto.subtle.importKey('raw', 'p', {name:'PBKDF2'}, false, ['deriveBits','deriveKey'])\
               .then(function(k){ globalThis.__k = k; });",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute(
                "String(globalThis.__k && globalThis.__k.type === 'secret'\
                   && globalThis.__k.algorithm.name === 'PBKDF2'\
                   && globalThis.__k.usages.join(',') === 'deriveBits,deriveKey')"
            )
            .unwrap()
            .value,
        "true"
    );

    // deriveBits 缺 "deriveBits" usage（仅 'deriveKey'）→ reject InvalidAccessError。
    sandbox
        .execute(
            "globalThis.__e1='(pending)';\
             crypto.subtle.importKey('raw', 'p', {name:'PBKDF2'}, false, ['deriveKey'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-256',salt:'s',iterations:1}, k, 256); })\
               .catch(function(e){ globalThis.__e1 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e1)").unwrap().value,
        "InvalidAccessError"
    );
    // importKey PBKDF2 非法 usage（'sign' 不属 {deriveBits,deriveKey}）→ reject SyntaxError。
    sandbox
        .execute(
            "globalThis.__e2='(pending)';\
             crypto.subtle.importKey('raw', 'p', {name:'PBKDF2'}, false, ['sign'])\
               .catch(function(e){ globalThis.__e2 = e.name; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e2)").unwrap().value, "SyntaxError");
    // deriveBits length 非 8 倍数（17）→ reject OperationError。
    sandbox
        .execute(
            "globalThis.__e3='(pending)';\
             crypto.subtle.importKey('raw', 'p', {name:'PBKDF2'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-256',salt:'s',iterations:1}, k, 17); })\
               .catch(function(e){ globalThis.__e3 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e3)").unwrap().value,
        "OperationError"
    );
    // deriveBits algorithm/key 不匹配（HMAC key 用于 PBKDF2）→ reject NotSupportedError。
    sandbox
        .execute(
            "globalThis.__e4='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(4), {name:'HMAC',hash:'SHA-256'}, false, ['sign','verify'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'PBKDF2',hash:'SHA-256',salt:'s',iterations:1}, k, 256); })\
               .catch(function(e){ globalThis.__e4 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e4)").unwrap().value,
        "NotSupportedError"
    );
}

#[test]
fn test_crypto_subtle_aes_gcm_r2957() {
    // R2957：crypto.subtle encrypt/decrypt("AES-GCM", ...)——AES-128/256-GCM 认证对称加密。
    // PBKDF2 派生密钥的典型消费者（端到端「用密码加密」）。host RustCrypto aes-gcm（新依赖）。
    // TDD 用 NIST GCM TC3（无 AAD）/TC4（带 AAD）向量锚定 + decrypt + round-trip + 错误路径。
    // https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf  https://w3c.github.io/webcrypto/
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

    // hex 辅助（持久 context 跨 execute 复用）。
    sandbox
        .execute(
            "function hex2b(h){var a=[];for(var i=0;i<h.length;i+=2)a.push(parseInt(h.substr(i,2),16));return new Uint8Array(a);}\
             function b2hex(u){var s='';for(var i=0;i<u.length;i++){s+=('0'+u[i].toString(16)).slice(-2);}return s;}",
        )
        .unwrap();

    // NIST GCM TC1：K=0×16，IV=0×12，空 P，空 A → 输出仅 tag = 58e2fcce...455a。
    let ct1 = {
        sandbox
            .execute(
                "globalThis.__out='(pending)';\
                 crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['encrypt','decrypt'])\
                   .then(function(k){ return crypto.subtle.encrypt({name:'AES-GCM', iv:new Uint8Array(12)}, k, new Uint8Array(0)); })\
                   .then(function(b){ globalThis.__out = new Uint8Array(b); });",
            )
            .unwrap();
        sandbox.execute("b2hex(globalThis.__out)").unwrap().value
    };
    assert_eq!(ct1, "58e2fccefa7e3061367f1d57a4e7455a");
    // NIST GCM TC2：K=0×16，IV=0×12，P=0×16（空 A）→ C||T = 0388dace...fe78 || ab6e47d4...bddf。
    let ct2 = {
        sandbox
            .execute(
                "globalThis.__out='(pending)';\
                 crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['encrypt','decrypt'])\
                   .then(function(k){ return crypto.subtle.encrypt({name:'AES-GCM', iv:new Uint8Array(12)}, k, new Uint8Array(16)); })\
                   .then(function(b){ globalThis.__out = new Uint8Array(b); });",
            )
            .unwrap();
        sandbox.execute("b2hex(globalThis.__out)").unwrap().value
    };
    assert_eq!(ct2, "0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf");
    // TC2 decrypt → P（16 字节全零）。stash CT2 供后续 AAD-mismatch 测试复用同密文。
    sandbox
        .execute("globalThis.CT2='0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf';")
        .unwrap();
    let pt2 = {
        sandbox
            .execute(
                "globalThis.__out='(pending)';\
                 crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['encrypt','decrypt'])\
                   .then(function(k){ return crypto.subtle.decrypt({name:'AES-GCM', iv:new Uint8Array(12)}, k, hex2b(globalThis.CT2)); })\
                   .then(function(b){ globalThis.__out = new Uint8Array(b); });",
            )
            .unwrap();
        sandbox.execute("b2hex(globalThis.__out)").unwrap().value
    };
    assert_eq!(pt2, "00000000000000000000000000000000");

    // AAD round-trip（证明 additionalData 接入 encrypt+decrypt 双向）：encrypt 带 AAD → decrypt 同 AAD == 原文；
    // stash 该 ct 供后续 AAD-mismatch 错误测试（用无 AAD 解 → tag 校验失败）。
    sandbox
        .execute(
            "globalThis.__aadrt='(pending)'; globalThis.__aadct=null;\
             crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['encrypt','decrypt'])\
               .then(function(k){\
                 return crypto.subtle.encrypt({name:'AES-GCM', iv:new Uint8Array(12), additionalData:new TextEncoder().encode('aad')}, k, new TextEncoder().encode('secret'));\
               }).then(function(ct){ globalThis.__aadct = new Uint8Array(ct); return crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['encrypt','decrypt']); })\
               .then(function(k){\
                 return crypto.subtle.decrypt({name:'AES-GCM', iv:new Uint8Array(12), additionalData:new TextEncoder().encode('aad')}, k, globalThis.__aadct);\
               }).then(function(pt){ globalThis.__aadrt = new TextDecoder().decode(pt); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__aadrt)").unwrap().value, "secret");

    // Round-trip（AES-256）：encrypt 后 decrypt == 原文。
    sandbox
        .execute(
            "globalThis.__rt='(pending)';\
             crypto.subtle.importKey('raw', hex2b('00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff'), {name:'AES-GCM'}, false, ['encrypt','decrypt'])\
               .then(function(k){\
                 var iv=new Uint8Array([1,2,3,4,5,6,7,8,9,10,11,12]);\
                 return crypto.subtle.encrypt({name:'AES-GCM', iv:iv}, k, new TextEncoder().encode('hello world'));\
               }).then(function(ct){ globalThis.__ct = new Uint8Array(ct); return crypto.subtle.importKey('raw', hex2b('00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff'), {name:'AES-GCM'}, false, ['encrypt','decrypt']); })\
               .then(function(k){\
                 var iv=new Uint8Array([1,2,3,4,5,6,7,8,9,10,11,12]);\
                 return crypto.subtle.decrypt({name:'AES-GCM', iv:iv}, k, globalThis.__ct);\
               }).then(function(pt){ globalThis.__rt = new TextDecoder().decode(pt); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__rt)").unwrap().value, "hello world");

    // importKey AES-GCM 字段：type="secret"，algorithm.name="AES-GCM"，usages。
    sandbox
        .execute(
            "globalThis.__k=null;\
             crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['encrypt','decrypt'])\
               .then(function(k){ globalThis.__k = k; });",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute(
                "String(globalThis.__k && globalThis.__k.type === 'secret'\
                   && globalThis.__k.algorithm.name === 'AES-GCM'\
                   && globalThis.__k.usages.join(',') === 'encrypt,decrypt')"
            )
            .unwrap()
            .value,
        "true"
    );
    // importKey AES-GCM 非 16/32 字节 key（24 字节）→ reject DataError。
    sandbox
        .execute(
            "globalThis.__e1='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(24), {name:'AES-GCM'}, false, ['encrypt'])\
               .catch(function(e){ globalThis.__e1 = e.name; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e1)").unwrap().value, "DataError");
    // encrypt 缺 "encrypt" usage（仅 'decrypt'）→ reject InvalidAccessError。
    sandbox
        .execute(
            "globalThis.__e2='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['decrypt'])\
               .then(function(k){ return crypto.subtle.encrypt({name:'AES-GCM', iv:new Uint8Array(12)}, k, 'x'); })\
               .catch(function(e){ globalThis.__e2 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e2)").unwrap().value,
        "InvalidAccessError"
    );
    // decrypt AAD 不匹配（CT2 用空 AAD 加密，这里带 AAD 解 → tag 校验失败）→ reject OperationError。
    sandbox
        .execute(
            "globalThis.__e3='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['decrypt'])\
               .then(function(k){ return crypto.subtle.decrypt({name:'AES-GCM', iv:new Uint8Array(12), additionalData:new TextEncoder().encode('x')}, k, hex2b(globalThis.CT2)); })\
               .catch(function(e){ globalThis.__e3 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e3)").unwrap().value,
        "OperationError"
    );
    // 非 128 tagLength → reject NotSupportedError。
    sandbox
        .execute(
            "globalThis.__e4='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['encrypt'])\
               .then(function(k){ return crypto.subtle.encrypt({name:'AES-GCM', iv:new Uint8Array(12), tagLength:96}, k, 'x'); })\
               .catch(function(e){ globalThis.__e4 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e4)").unwrap().value,
        "NotSupportedError"
    );
}

#[test]
fn test_crypto_subtle_hkdf_r2958() {
    // R2958：crypto.subtle deriveBits("HKDF", ...)——HKDF-SHA-1/256/384/512（RFC 5869）。密钥协商派生
    // （TLS 1.3 / MLS / WebRTC DTLS-SRTP / E2EE 协议）。复用 R2955 compute_hmac 作 PRF。TDD 用 RFC 5869
    // TC1（带 salt+info）/TC3（空 salt+info，测 HashLen 零填充）SHA-256 向量锚定 + 错误路径。
    // https://datatracker.ietf.org/doc/html/rfc5869#section-A  https://w3c.github.io/webcrypto/
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
            "function hex2b(h){var a=[];for(var i=0;i<h.length;i+=2)a.push(parseInt(h.substr(i,2),16));return new Uint8Array(a);}\
             function b2hex(u){var s='';for(var i=0;i<u.length;i++){s+=('0'+u[i].toString(16)).slice(-2);}return s;}",
        )
        .unwrap();

    // RFC 5869 TC1：IKM=0x0b×22，salt=000102...0c，info=f0f1...f9，L=42 → OKM=3cb25f25...185865。
    let mut hex_dk = |derive: &str| -> String {
        sandbox.execute(derive).unwrap();
        sandbox.execute("b2hex(globalThis.__dk)").unwrap().value
    };
    assert_eq!(
        hex_dk(
            "globalThis.__dk='(pending)';\
             var IKM='0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b';\
             var SALT='000102030405060708090a0b0c';\
             var INFO='f0f1f2f3f4f5f6f7f8f9';\
             crypto.subtle.importKey('raw', hex2b(IKM), {name:'HKDF'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'HKDF',hash:'SHA-256',salt:hex2b(SALT),info:hex2b(INFO)}, k, 336); })\
               .then(function(b){ globalThis.__dk = new Uint8Array(b); });"
        ),
        "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
    );
    // RFC 5869 TC3：IKM=0x0b×22，空 salt + 空 info（host 填 HashLen 零），L=42 → OKM=8da4e775...a96c8。
    assert_eq!(
        hex_dk(
            "globalThis.__dk='(pending)';\
             var IKM='0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b';\
             crypto.subtle.importKey('raw', hex2b(IKM), {name:'HKDF'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'HKDF',hash:'SHA-256'}, k, 336); })\
               .then(function(b){ globalThis.__dk = new Uint8Array(b); });"
        ),
        "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8"
    );
    // HKDF-SHA-1（RFC 5869 TC4 等价输入）：IKM=0x0b×22，salt=000102...0c，info=f0f1...f9，L=42 → OKM。
    let sha1_dk = hex_dk(
        "globalThis.__dk='(pending)';\
         var IKM='0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b';\
         crypto.subtle.importKey('raw', hex2b(IKM), {name:'HKDF'}, false, ['deriveBits'])\
           .then(function(k){ return crypto.subtle.deriveBits({name:'HKDF',hash:'SHA-1',salt:hex2b('000102030405060708090a0b0c'),info:hex2b('f0f1f2f3f4f5f6f7f8f9')}, k, 336); })\
           .then(function(b){ globalThis.__dk = new Uint8Array(b); });",
    );
    assert_eq!(
        sha1_dk,
        "d6000ffb5b50bd3970b260017798fb9c8df9ce2e2c16b6cd709cca07dc3cf9cf26d6c6d750d0aaf5ac94"
    );

    // importKey 字段：HKDF CryptoKey type="secret"，algorithm.name="HKDF"，usages。
    sandbox
        .execute(
            "globalThis.__k=null;\
             crypto.subtle.importKey('raw', new Uint8Array(4), {name:'HKDF'}, false, ['deriveBits','deriveKey'])\
               .then(function(k){ globalThis.__k = k; });",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute(
                "String(globalThis.__k && globalThis.__k.type === 'secret'\
                   && globalThis.__k.algorithm.name === 'HKDF'\
                   && globalThis.__k.usages.join(',') === 'deriveBits,deriveKey')"
            )
            .unwrap()
            .value,
        "true"
    );

    // deriveBits 缺 "deriveBits" usage（仅 'deriveKey'）→ reject InvalidAccessError。
    sandbox
        .execute(
            "globalThis.__e1='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(4), {name:'HKDF'}, false, ['deriveKey'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'HKDF',hash:'SHA-256'}, k, 256); })\
               .catch(function(e){ globalThis.__e1 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e1)").unwrap().value,
        "InvalidAccessError"
    );
    // importKey HKDF 非法 usage（'sign' 不属 {deriveBits,deriveKey}）→ reject SyntaxError。
    sandbox
        .execute(
            "globalThis.__e2='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(4), {name:'HKDF'}, false, ['sign'])\
               .catch(function(e){ globalThis.__e2 = e.name; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e2)").unwrap().value, "SyntaxError");
    // deriveBits algorithm/key 不匹配（PBKDF2 key 用于 HKDF）→ reject NotSupportedError。
    sandbox
        .execute(
            "globalThis.__e3='(pending)';\
             crypto.subtle.importKey('raw', 'p', {name:'PBKDF2'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'HKDF',hash:'SHA-256'}, k, 256); })\
               .catch(function(e){ globalThis.__e3 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e3)").unwrap().value,
        "NotSupportedError"
    );
    // length 非 8 倍数（17）→ reject OperationError。
    sandbox
        .execute(
            "globalThis.__e4='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(4), {name:'HKDF'}, false, ['deriveBits'])\
               .then(function(k){ return crypto.subtle.deriveBits({name:'HKDF',hash:'SHA-256'}, k, 17); })\
               .catch(function(e){ globalThis.__e4 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e4)").unwrap().value,
        "OperationError"
    );
}

#[test]
fn test_crypto_subtle_keyops_r2959() {
    // R2959：crypto.subtle generateKey / deriveKey / exportKey——补全 SubtleCrypto 方法表面（全 10 方法）。
    // generateKey（AES-GCM 256 位 / HMAC 块大小随机）、deriveKey（deriveBits + importKey 包装，AES→256/HMAC→块大小）、
    // exportKey（raw，非 extractable 拒绝）。round-trip + deriveKey↔deriveBits 一致 + 错误路径。
    // https://w3c.github.io/webcrypto/
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

    // generateKey AES-GCM：CryptoKey type="secret" + 32 字节随机 + encrypt/decrypt round-trip。
    sandbox
        .execute(
            "globalThis.__rt='(pending)'; globalThis.__klen='(pending)';\
             crypto.subtle.generateKey({name:'AES-GCM'}, true, ['encrypt','decrypt'])\
               .then(function(k){ globalThis.__klen = String(k._raw.length); globalThis.__genk = k;\
                 var iv=new Uint8Array([1,2,3,4,5,6,7,8,9,10,11,12]);\
                 return crypto.subtle.encrypt({name:'AES-GCM', iv:iv}, k, new TextEncoder().encode('hi')); })\
               .then(function(ct){ globalThis.__ct = new Uint8Array(ct);\
                 return crypto.subtle.decrypt({name:'AES-GCM', iv:new Uint8Array([1,2,3,4,5,6,7,8,9,10,11,12])}, globalThis.__genk, globalThis.__ct); })\
               .then(function(pt){ globalThis.__rt = new TextDecoder().decode(pt); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__klen)").unwrap().value, "32");
    assert_eq!(sandbox.execute("String(globalThis.__rt)").unwrap().value, "hi");
    // generateKey HMAC{SHA-256}：64 字节随机（块大小）+ sign/verify round-trip。
    sandbox
        .execute(
            "globalThis.__hmac='(pending)';\
             crypto.subtle.generateKey({name:'HMAC',hash:'SHA-256'}, true, ['sign','verify'])\
               .then(function(k){ globalThis.__hk = k; return crypto.subtle.sign('HMAC', k, new TextEncoder().encode('msg')); })\
               .then(function(sig){ globalThis.__sig = new Uint8Array(sig);\
                 return crypto.subtle.verify('HMAC', globalThis.__hk, globalThis.__sig, new TextEncoder().encode('msg')); })\
               .then(function(ok){ globalThis.__hmac = String(ok); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__hmac)").unwrap().value, "true");
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hk && globalThis.__hk._raw.length)")
            .unwrap()
            .value,
        "64"
    );

    // deriveKey PBKDF2→AES-GCM：派生密钥 encrypt/decrypt round-trip + exportKey(raw) == deriveBits(256)（证内部派 256 位）。
    sandbox
        .execute(
            "globalThis.__drt='(pending)'; globalThis.__dcons='(pending)';\
             var params={name:'PBKDF2',hash:'SHA-256',salt:new TextEncoder().encode('salt'),iterations:100};\
             crypto.subtle.importKey('raw', new TextEncoder().encode('password'), {name:'PBKDF2'}, false, ['deriveKey','deriveBits'])\
               .then(function(pw){\
                 var p=Promise.resolve(crypto.subtle.deriveKey(params, pw, {name:'AES-GCM'}, true, ['encrypt','decrypt']));\
                 var b=Promise.resolve(crypto.subtle.deriveBits(params, pw, 256));\
                 return Promise.all([p,b]);\
               }).then(function(r){ var dk=r[0], bits=new Uint8Array(r[1]); globalThis.__dk=dk;\
                 // 一致：deriveKey 派 256 位 == deriveBits(256)（逐字节比）。
                 return crypto.subtle.exportKey('raw', dk).then(function(ex){\
                   var a=new Uint8Array(ex); var ok=(a.length===bits.length);\
                   for(var i=0;ok&&i<a.length;i++){if(a[i]!==bits[i])ok=false;} globalThis.__dcons=String(ok); });\
               }).then(function(){\
                 var iv=new Uint8Array([1,2,3,4,5,6,7,8,9,10,11,12]);\
                 return crypto.subtle.encrypt({name:'AES-GCM', iv:iv}, globalThis.__dk, new TextEncoder().encode('secret'));\
               }).then(function(ct){ globalThis.__dct = new Uint8Array(ct);\
                 return crypto.subtle.decrypt({name:'AES-GCM', iv:new Uint8Array([1,2,3,4,5,6,7,8,9,10,11,12])}, globalThis.__dk, globalThis.__dct); })\
               .then(function(pt){ globalThis.__drt = new TextDecoder().decode(pt); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__dcons)").unwrap().value, "true");
    assert_eq!(sandbox.execute("String(globalThis.__drt)").unwrap().value, "secret");
    // deriveKey PBKDF2→HMAC：派生密钥 sign round-trip（verify true）。
    sandbox
        .execute(
            "globalThis.__dsign='(pending)';\
             var params={name:'PBKDF2',hash:'SHA-256',salt:'salt',iterations:50};\
             crypto.subtle.importKey('raw', 'pw', {name:'PBKDF2'}, false, ['deriveKey'])\
               .then(function(pw){ return crypto.subtle.deriveKey(params, pw, {name:'HMAC',hash:'SHA-256'}, false, ['sign','verify']); })\
               .then(function(k){ globalThis.__dhk=k; return crypto.subtle.sign('HMAC', k, 'data'); })\
               .then(function(sig){ return crypto.subtle.verify('HMAC', globalThis.__dhk, new Uint8Array(sig), 'data'); })\
               .then(function(ok){ globalThis.__dsign = String(ok); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__dsign)").unwrap().value, "true");

    // exportKey raw：返 _raw（== importKey 输入）。
    sandbox
        .execute(
            "globalThis.__ex='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]), {name:'AES-GCM'}, true, ['encrypt'])\
               .then(function(k){ return crypto.subtle.exportKey('raw', k); })\
               .then(function(b){ var a=new Uint8Array(b); var s=''; for(var i=0;i<a.length;i++) s+=a[i]+','; globalThis.__ex=s; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ex)").unwrap().value,
        "1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,"
    );
    // exportKey 非 extractable → reject InvalidAccessError。
    sandbox
        .execute(
            "globalThis.__e1='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, false, ['encrypt'])\
               .then(function(k){ return crypto.subtle.exportKey('raw', k); })\
               .catch(function(e){ globalThis.__e1 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e1)").unwrap().value,
        "InvalidAccessError"
    );
    // exportKey 非 raw 格式（'jwk'）→ reject NotSupportedError。
    sandbox
        .execute(
            "globalThis.__e2='(pending)';\
             crypto.subtle.importKey('raw', new Uint8Array(16), {name:'AES-GCM'}, true, ['encrypt'])\
               .then(function(k){ return crypto.subtle.exportKey('jwk', k); })\
               .catch(function(e){ globalThis.__e2 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e2)").unwrap().value,
        "NotSupportedError"
    );
    // deriveKey baseKey 缺 "deriveKey" usage（仅 'deriveBits'）→ reject InvalidAccessError。
    sandbox
        .execute(
            "globalThis.__e3='(pending)';\
             var params={name:'PBKDF2',hash:'SHA-256',salt:'s',iterations:10};\
             crypto.subtle.importKey('raw', 'p', {name:'PBKDF2'}, false, ['deriveBits'])\
               .then(function(pw){ return crypto.subtle.deriveKey(params, pw, {name:'AES-GCM'}, false, ['encrypt']); })\
               .catch(function(e){ globalThis.__e3 = e.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__e3)").unwrap().value,
        "InvalidAccessError"
    );
    // generateKey 非法 usage（AES-GCM 'sign'）→ reject SyntaxError。
    sandbox
        .execute(
            "globalThis.__e4='(pending)';\
             crypto.subtle.generateKey({name:'AES-GCM'}, true, ['sign'])\
               .catch(function(e){ globalThis.__e4 = e.name; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e4)").unwrap().value, "SyntaxError");
}

#[test]
fn test_crypto_csprng_r2960() {
    // R2960：crypto.getRandomValues / randomUUID 升级 OS-random（getrandom crate，host 回调）。
    // 升级前 Math.random（非 CSPRNG，predictable——token/密钥/IV 安全弱点）。本测试验 host 路径属性
    // （长度/字节范围/两次不同/v4 格式/参数校验）+ host 未注册时 Math.random 回退仍工作。
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

    // host 回调已注册（OS-random 路径生效）。
    assert_eq!(
        sandbox
            .execute("String(typeof __zw_crypto_get_random_values === 'function')")
            .unwrap()
            .value,
        "true"
    );
    // getRandomValues：长度保持 + 字节范围 0-255（两次调用不同——OS 随机几乎必异）。
    sandbox
        .execute(
            "var a1 = crypto.getRandomValues(new Uint8Array(16));\
             var a2 = crypto.getRandomValues(new Uint8Array(16));\
             var same = 1, inRange = 1;\
             for (var i = 0; i < 16; i++) { if (a1[i] !== a2[i]) same = 0; if (a1[i] < 0 || a1[i] > 255 || a2[i] > 255) inRange = 0; }\
             globalThis.__rv = String(a1.length === 16 && a2.length === 16 && inRange === 1 && same === 0);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__rv)").unwrap().value, "true");
    // getRandomValues 共享 buffer 偏移视图（Uint32Array 视图看随机 32 位值）。
    sandbox
        .execute(
            "var u32 = new Uint32Array(2); crypto.getRandomValues(u32);\
             globalThis.__u32 = String(u32[0] !== u32[1]);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__u32)").unwrap().value, "true");
    // randomUUID：v4 格式（version=4，variant∈89ab）。
    assert_eq!(
        sandbox
            .execute(
                "/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(crypto.randomUUID())"
            )
            .unwrap()
            .value,
        "true"
    );
    // getRandomValues 参数校验：非 TypedArray → TypeError；>65536 字节 → QuotaExceededError。
    assert_eq!(
        sandbox
            .execute("try { crypto.getRandomValues([1,2,3]); 'no-throw' } catch (e) { e instanceof TypeError ? 'TypeError' : e.name }")
            .unwrap()
            .value,
        "TypeError"
    );
    assert_eq!(
        sandbox
            .execute("try { crypto.getRandomValues(new Uint8Array(65537)); 'no-throw' } catch (e) { e.name }")
            .unwrap()
            .value,
        "QuotaExceededError"
    );

    // 回退路径：host 未注册（纯 shim）→ Math.random 仍工作（getRandomValues 填 + randomUUID v4 格式）。
    let mut fb = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    fb.execute(generate_js_dom_shim()).unwrap();
    assert_eq!(
        fb.execute("String(typeof __zw_crypto_get_random_values === 'function')")
            .unwrap()
            .value,
        "false"
    );
    assert_eq!(
        fb.execute(
            "var a = crypto.getRandomValues(new Uint8Array(4)); String(a.length === 4 && a[0] >= 0 && a[0] <= 255)"
        )
        .unwrap()
        .value,
        "true"
    );
    assert_eq!(
        fb.execute("/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(crypto.randomUUID())")
            .unwrap()
            .value,
        "true"
    );
}

#[test]
fn test_eventsource_r2961() {
    // R2961：EventSource（Server-Sent Events，SSE）——服务器单向推送。纯 JS 经 fetch（R2923）拉
    // text/event-stream 全 body 后按 HTML spec §9.2 解析。本测试用 fetch mock（覆写 globalThis.fetch 返
    // 合成 SSE body）验解析 + 派发 + readyState + close()。无需 host fetch handler / 真服务器。
    // https://html.spec.whatwg.org/multipage/server-sent-events.html
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

    // 覆写 fetch 为 mock（返合成 SSE body），建 EventSource 收事件。
    sandbox
        .execute(
            "globalThis.__es = []; globalThis.__open = 'no'; globalThis.__err = 'no';\
             globalThis.fetch = function(url, init) {\
               return Promise.resolve({ ok: true, status: 200, text: function() { return Promise.resolve(globalThis.__BODY); } });\
             };\
             globalThis.__BODY = 'data: hello\\n\\ndata: world\\n\\nevent: custom\\ndata: payload\\n\\n: comment\\ndata: a\\ndata: b\\n\\n';\
             var es = new EventSource('https://example.com/stream');\
             es.onopen = function() { globalThis.__open = 'yes'; };\
             es.onmessage = function(e) { globalThis.__es.push('msg:' + e.data); };\
             es.addEventListener('custom', function(e) { globalThis.__es.push('custom:' + e.data); });\
             es.onerror = function() { globalThis.__err = 'yes'; };",
        )
        .unwrap();
    // fetch 链在 execute 末 microtask 排空 → onopen + 解析派发 + onerror（finite stream 结束）。
    assert_eq!(sandbox.execute("String(globalThis.__open)").unwrap().value, "yes");
    assert_eq!(
        sandbox.execute("globalThis.__es.join('|')").unwrap().value,
        "msg:hello|msg:world|custom:payload|msg:a\nb"
    );
    assert_eq!(sandbox.execute("String(globalThis.__err)").unwrap().value, "yes");
    // readyState：finite stream 结束后 CLOSED（2）。
    assert_eq!(
        sandbox
            .execute("String(typeof EventSource !== 'undefined' ? 1 : 0)")
            .unwrap()
            .value,
        "1"
    );

    // data 后无空格（`data:nospace`）、id 字段 + 多事件流。
    sandbox
        .execute(
            "globalThis.__es2 = [];\
             globalThis.__BODY = 'id:42\\ndata:nospace\\n\\nid:43\\ndata: second\\n\\n';\
             var es2 = new EventSource('https://example.com/s');\
             es2.onmessage = function(e) { globalThis.__es2.push(e.lastEventId + ':' + e.data); };",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__es2.join('|')").unwrap().value,
        "42:nospace|43:second"
    );

    // close() 在派发前调用 → 不派发（_closed 守卫）。
    sandbox
        .execute(
            "globalThis.__es3 = [];\
             globalThis.__BODY = 'data: x\\n\\n';\
             var es3 = new EventSource('https://example.com/c');\
             es3.close();\
             es3.onmessage = function(e) { globalThis.__es3.push(e.data); };",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__es3.length === 0 ? 'closed' : 'fired')")
            .unwrap()
            .value,
        "closed"
    );

    // 无 fetch（host 未注册且无覆写）→ onerror（不悬挂）。
    let mut nf = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    nf.execute(generate_js_dom_shim()).unwrap();
    // shim 自带 fetch（返 no-handler Response），EventSource 拿到 !ok → onerror。
    nf.execute(
        "globalThis.__nferr='no'; var es=new EventSource('https://example.com/n'); es.onerror=function(){globalThis.__nferr='yes';};",
    )
    .unwrap();
    assert_eq!(nf.execute("String(globalThis.__nferr)").unwrap().value, "yes");
}

#[test]
fn test_headers_r2794() {
    // R2794：Headers（HTTP 头集合，fetch/SW/header-map 高频）。镜像 FormData，header name 小写归一 +
    // 多值 append 用 ', ' 合并 + getSetCookie 特例。纯 JS，零 host 回调。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + 无 new 构造 + instanceof。
    assert_eq!(sandbox.execute("typeof Headers").unwrap().value, "function");
    assert_eq!(
        sandbox
            .execute("String(new Headers() instanceof Headers)")
            .unwrap()
            .value,
        "true"
    );
    // record 对象构造 + get/has（name 小写归一：'Content-Type' → 'content-type' 查）。
    sandbox
        .execute("var h = new Headers({'Content-Type':'text/plain','X-Custom':'a'});")
        .unwrap();
    assert_eq!(sandbox.execute("h.get('Content-Type')").unwrap().value, "text/plain");
    assert_eq!(sandbox.execute("h.get('content-type')").unwrap().value, "text/plain");
    assert_eq!(sandbox.execute("h.get('missing')").unwrap().value, "null");
    assert_eq!(sandbox.execute("String(h.has('X-Custom'))").unwrap().value, "true");
    assert_eq!(sandbox.execute("String(h.has('x-custom'))").unwrap().value, "true");
    // append 多值 → get ', ' 合并（spec）。
    sandbox.execute("h.append('X-Custom','b');").unwrap();
    assert_eq!(sandbox.execute("h.get('X-Custom')").unwrap().value, "a, b");
    // set 替换所有同名值。
    sandbox.execute("h.set('X-Custom','only');").unwrap();
    assert_eq!(sandbox.execute("h.get('X-Custom')").unwrap().value, "only");
    // delete。
    sandbox.execute("h.delete('X-Custom');").unwrap();
    assert_eq!(sandbox.execute("String(h.has('X-Custom'))").unwrap().value, "false");
    // 数组对序列构造。
    assert_eq!(
        sandbox
            .execute("new Headers([['A','1'],['B','2']]).get('A')")
            .unwrap()
            .value,
        "1"
    );
    // 另一 Headers 构造（forEach 分支）。
    assert_eq!(
        sandbox
            .execute("new Headers(new Headers({'K':'v'})).get('K')")
            .unwrap()
            .value,
        "v"
    );
    // getSetCookie：多个 Set-Cookie 返数组（不合并，spec 特例）。
    sandbox
        .execute("var c = new Headers(); c.append('Set-Cookie','a=1'); c.append('Set-Cookie','b=2');")
        .unwrap();
    assert_eq!(sandbox.execute("c.getSetCookie().join('|')").unwrap().value, "a=1|b=2");
    // 迭代：[Symbol.iterator]=entries → spread 取 [k,v]，key 为小写。
    sandbox.execute("var it = new Headers({'Z':'1','A':'2'});").unwrap();
    assert_eq!(
        sandbox
            .execute("[...it].map(function(p){return p[0]+'='+p[1];}).join(',')")
            .unwrap()
            .value,
        "z=1,a=2"
    );
    // forEach 回调。
    assert_eq!(
        sandbox
            .execute("(function(){var o=[];it.forEach(function(v,k){o.push(k+':'+v);});return o.join(',');})()")
            .unwrap()
            .value,
        "z:1,a:2"
    );
    // keys / values 迭代器。
    assert_eq!(sandbox.execute("[...it.keys()].join(',')").unwrap().value, "z,a");
    assert_eq!(sandbox.execute("[...it.values()].join(',')").unwrap().value, "1,2");
}

#[test]
fn test_canvas_get_context_r2795() {
    // R2795：HTMLCanvasElement.getContext('2d')（canvas slice 1）。host CanvasContext 注册表 +
    // __zw_canvas_op 派发。fill()/stroke() 写 pixel_buffer（path-based rasterize），fillRect 经 path
    // 实现（绕过 fill_rect 便捷法不写 pixel_buffer）。getImageData 回读验证像素。
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

    // createElement('canvas') + width/height 默认 300×150 + getContext('2d')。
    sandbox
        .execute(
            "globalThis.__c = document.createElement('canvas');\
             globalThis.__ctx = __c.getContext('2d');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("typeof __c.getContext").unwrap().value, "function");
    assert_eq!(
        sandbox.execute("String(__c.width + 'x' + __c.height)").unwrap().value,
        "300x150"
    );
    assert_eq!(
        sandbox
            .execute("String(__ctx !== null && typeof __ctx.fillRect === 'function')")
            .unwrap()
            .value,
        "true"
    );
    // 自定义尺寸 + getContext('webgl') → null（仅 2d）。
    sandbox.execute("__c.width = 4; __c.height = 4;").unwrap();
    assert_eq!(
        sandbox
            .execute("String(document.createElement('canvas').getContext('webgl') === null)")
            .unwrap()
            .value,
        "true"
    );
    // fillRect('red') + getImageData 回读：红色像素（255,0,0,255）。
    // 重新创建 4×4 canvas + ctx（尺寸变化后重取 ctx 反映新尺寸）。
    sandbox
        .execute(
            "globalThis.__c2 = document.createElement('canvas'); __c2.width=4; __c2.height=4;\
             globalThis.__ctx2 = __c2.getContext('2d');\
             __ctx2.fillStyle = 'red';\
             __ctx2.fillRect(0, 0, 4, 4);\
             globalThis.__img = __ctx2.getImageData(0, 0, 4, 4);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__img.width + 'x' + __img.height)")
            .unwrap()
            .value,
        "4x4"
    );
    assert_eq!(
        sandbox
            .execute("String(__img.data[0] + ',' + __img.data[1] + ',' + __img.data[2] + ',' + __img.data[3])")
            .unwrap()
            .value,
        "255,0,0,255"
    );
    // 未填充区域：getImageData 另一 canvas（默认透明 0,0,0,0）。
    sandbox
        .execute(
            "globalThis.__e = document.createElement('canvas'); __e.width=2; __e.height=2;\
             globalThis.__blank = __e.getContext('2d').getImageData(0,0,2,2);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__blank.data[0] + ',' + __blank.data[3])")
            .unwrap()
            .value,
        "0,0"
    );
    // fillStyle #00ff00（hex 解析）→ 绿色像素。
    sandbox
        .execute(
            "globalThis.__g = document.createElement('canvas'); __g.width=2; __g.height=2;\
             var gx = __g.getContext('2d'); gx.fillStyle = '#00ff00'; gx.fillRect(0,0,2,2);\
             globalThis.__gp = gx.getImageData(0,0,2,2).data;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__gp[0] + ',' + __gp[1] + ',' + __gp[2])")
            .unwrap()
            .value,
        "0,255,0"
    );
    // path API：beginPath + arc + fill（圆形区域中心像素为填充色）。
    sandbox
        .execute(
            "globalThis.__p = document.createElement('canvas'); __p.width=10; __p.height=10;\
             var px = __p.getContext('2d'); px.fillStyle = 'rgb(0,0,255)';\
             px.beginPath(); px.arc(5, 5, 4, 0, 6.2832); px.fill();\
             globalThis.__pp = px.getImageData(5, 5, 1, 1).data;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__pp[0] + ',' + __pp[1] + ',' + __pp[2])")
            .unwrap()
            .value,
        "0,0,255"
    );
}

#[test]
fn test_canvas_slice2_r2796() {
    // R2796：canvas slice 2——path 曲线 / save/restore 栈 / transforms / globalAlpha / line 样式。
    // builds on slice 1 dispatch。translate 后 fillRect 像素位移；save+globalAlpha+restore 还原；
    // scale 放大填充区域；curve 路径填充中心像素。
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

    // translate 后 fillRect：原本 (0,0,2,2) 平移到 (3,3)，故 (0,0) 空、(3,3) 红。
    sandbox
        .execute(
            "globalThis.__t = document.createElement('canvas'); __t.width=8; __t.height=8;\
             var tx = __t.getContext('2d');\
             tx.translate(3, 3); tx.fillStyle = 'red'; tx.fillRect(0, 0, 2, 2);\
             globalThis.__im = tx.getImageData(0, 0, 8, 8);",
        )
        .unwrap();
    // 像素索引 helper：(x,y) 在 width=8 图里的 RGBA 起始 = (y*8+x)*4。
    let mut pix = |x: usize, y: usize| -> String {
        sandbox
            .execute(&format!(
                "var i={y}*8*4+{x}*4; String(globalThis.__im.data[i]+','+globalThis.__im.data[i+3])",
                x = x,
                y = y
            ))
            .unwrap()
            .value
    };
    assert_eq!(pix(0, 0), "0,0", "translate 前 (0,0) 应空");
    assert_eq!(pix(3, 3), "255,255", "translate 后 (3,3) 应红");
    // save + globalAlpha(0.5) + restore：globalAlpha 经 setter push host；restore 后还原本。
    sandbox
        .execute(
            "globalThis.__a = document.createElement('canvas'); __a.width=4; __a.height=4;\
             var ax = __a.getContext('2d');\
             ax.save(); ax.globalAlpha = 0.5; ax.fillStyle='red'; ax.fillRect(0,0,2,2);\
             ax.restore();\
             ax.fillStyle='green'; ax.fillRect(2,2,2,2);\
             globalThis.__aim = ax.getImageData(0,0,4,4);",
        )
        .unwrap();
    // globalAlpha 0.5 × 红 (255,0,0,255) → (255,0,0,127)（alpha 缩放）。验证 alpha 通道。
    assert_eq!(
        sandbox
            .execute("var i=0; String(__aim.data[i]+','+__aim.data[i+3])")
            .unwrap()
            .value,
        "255,127",
        "globalAlpha=0.5 应缩 alpha"
    );
    // scale 放大：scale(2,2) 后 fillRect(0,0,1,1) 覆盖 2×2 → (1,1) 被填充。
    sandbox
        .execute(
            "globalThis.__s = document.createElement('canvas'); __s.width=4; __s.height=4;\
             var sx = __s.getContext('2d');\
             sx.scale(2, 2); sx.fillStyle='blue'; sx.fillRect(0,0,1,1);\
             globalThis.__sim = sx.getImageData(0,0,4,4);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("var i=1*4*4+1*4; String(__sim.data[i+2]+','+__sim.data[i+3])")
            .unwrap()
            .value,
        "255,255",
        "scale(2,2) 后 1×1 覆盖到 (1,1) 蓝色"
    );
    // setTransform 单位阵重置 + rect 路径 + fill：rect(1,1,2,2) 填充 → (1,1) 红。
    sandbox
        .execute(
            "globalThis.__r = document.createElement('canvas'); __r.width=5; __r.height=5;\
             var rx = __r.getContext('2d');\
             rx.setTransform(1,0,0,1,0,0); rx.fillStyle='red';\
             rx.beginPath(); rx.rect(1,1,2,2); rx.fill();\
             globalThis.__rim = rx.getImageData(0,0,5,5);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("var i=1*5*4+1*4; String(__rim.data[i]+','+__rim.data[i+3])")
            .unwrap()
            .value,
        "255,255",
        "rect 路径填充 (1,1) 红"
    );
    // ellipse 路径 + fill：椭圆中心 (3,3) 蓝色。
    sandbox
        .execute(
            "globalThis.__el = document.createElement('canvas'); __el.width=8; __el.height=8;\
             var ex = __el.getContext('2d');\
             ex.fillStyle='rgb(0,0,255)'; ex.beginPath(); ex.ellipse(3,3,3,3,0,0,6.2832); ex.fill();\
             globalThis.__elim = ex.getImageData(3,3,1,1);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(__elim.data[2]+','+__elim.data[3])")
            .unwrap()
            .value,
        "255,255",
        "ellipse 填充中心蓝"
    );
    // lineJoin/lineCap setter push host（no-throw + getter 返回 set 值）。
    sandbox.execute("globalThis.__lj = __t.getContext('2d');").unwrap();
    // 注：__t 的 ctx 已创建；重取返同一 proxy。设值验证 getter。
    sandbox
        .execute(
            "globalThis.__cx = document.createElement('canvas').getContext('2d');\
             __cx.lineJoin='round'; __cx.lineCap='square'; __cx.lineWidth=3; __cx.setLineDash([5,3]);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("__cx.lineJoin").unwrap().value, "round");
    assert_eq!(sandbox.execute("__cx.lineCap").unwrap().value, "square");
    assert_eq!(sandbox.execute("String(__cx.lineWidth)").unwrap().value, "3");
}

#[test]
fn test_form_select_multiple_and_fieldset_disabled_r3056() {
    // R3056：collect_form_data 精化——<select multiple> 全选 selected option + <fieldset disabled> 联动禁用后代控件。
    let base = "https://example.com/page";

    // ① select multiple：全部 selected option 各入一项（name=val&name=val2，文档序）。
    let html = "<html><body><form id='f' action='/s'>\
        <select name='top' multiple>\
          <option value='a'>A</option>\
          <option value='b' selected>B</option>\
          <option value='c' selected>C</option>\
          <option value='d'>D</option>\
        </select>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html, "#f", None, base),
        Some("https://example.com/s?top=b&top=c".to_string()),
        "select multiple：全部 selected option 各入（b & c）"
    );

    // ② select multiple 无 selected → 不提交（区别于单选「默认首项」quirk）。
    let html2 = "<html><body><form id='f' action='/s'><select name='top' multiple><option>a</option><option>b</option></select></form></body></html>";
    assert_eq!(
        form_get_submission_url(html2, "#f", None, base),
        Some("https://example.com/s".to_string()),
        "select multiple 无 selected → 不提交（无 name=top）"
    );

    // ③ select multiple 中 selected 但 disabled 的 option → 跳过。
    let html3 = "<html><body><form id='f' action='/s'>\
        <select name='top' multiple>\
          <option value='a' selected disabled>A</option>\
          <option value='b' selected>B</option>\
        </select>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html3, "#f", None, base),
        Some("https://example.com/s?top=b".to_string()),
        "select multiple：disabled selected option 跳过（仅 b）"
    );

    // ④ 单选 select：selected option disabled → 回落首个未 disabled option（spec 默认选中 quirk）。
    let html4 = "<html><body><form id='f' action='/s'>\
        <select name='top'>\
          <option value='a' selected disabled>A</option>\
          <option value='b'>B</option>\
        </select>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html4, "#f", None, base),
        Some("https://example.com/s?top=b".to_string()),
        "单选：selected option disabled → 回落首个未 disabled option（b）"
    );

    // ⑤ fieldset disabled：后代控件全部跳过（即使控件自身无 disabled）。
    let html5 = "<html><body><form id='f' action='/s'>\
        <input name='a' value='1'>\
        <fieldset disabled>\
          <input name='b' value='2'>\
          <input name='c' value='3'>\
        </fieldset>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html5, "#f", None, base),
        Some("https://example.com/s?a=1".to_string()),
        "fieldset disabled：内部控件 b/c 跳过（仅 a=1）"
    );

    // ⑥ fieldset 未 disabled：后代控件正常提交。
    let html6 = "<html><body><form id='f' action='/s'>\
        <fieldset><input name='a' value='1'></fieldset>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html6, "#f", None, base),
        Some("https://example.com/s?a=1".to_string()),
        "fieldset 未 disabled：内部控件正常提交"
    );

    // ⑦ POST 表单同样遵循 fieldset disabled + select multiple（form_post_submission 复用 collect_form_data）。
    let html7 = "<html><body><form id='f' method='post' action='/s'>\
        <select name='t' multiple><option value='x' selected>X</option><option value='y' selected>Y</option></select>\
        <fieldset disabled><input name='skip' value='z'></fieldset>\
        </form></body></html>";
    assert_eq!(
        form_post_submission(html7, "#f", None, base),
        Some(("https://example.com/s".to_string(), "t=x&t=y".to_string())),
        "POST 表单：select multiple + fieldset disabled 一致（body=t=x&t=y，skip 跳过）"
    );

    // ⑧ 嵌套 fieldset：外层 disabled 联动内层控件（祖先链上行命中）。
    let html8 = "<html><body><form id='f' action='/s'>\
        <fieldset disabled><fieldset><input name='deep' value='1'></fieldset></fieldset>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html8, "#f", None, base),
        Some("https://example.com/s".to_string()),
        "嵌套 fieldset disabled：内层控件 deep 跳过（祖先链）"
    );
}

#[test]
fn test_form_disabled_legend_exemption_r3066() {
    // R3066：disabled fieldset 首个 <legend> 子内控件豁免（闭合 R3056 限制①）。spec HTML §4.10.18「not a
    // descendant of that fieldset's first legend element child」——legend 内控件即使 fieldset disabled 也启用提交。
    let base = "https://example.com/page";

    // ① disabled fieldset 首个 legend 内控件启用（提交），legend 外控件禁用（跳过）。
    let html = "<html><body><form id='f' action='/s'>\
        <fieldset disabled>\
          <legend><input name='leg' value='1'></legend>\
          <input name='body' value='2'>\
        </fieldset>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html, "#f", None, base),
        Some("https://example.com/s?leg=1".to_string()),
        "disabled fieldset：首个 legend 内控件启用（leg=1），legend 外控件禁用（body 跳过）"
    );

    // ② legend 内嵌套控件仍豁免（descendant）。
    let html2 = "<html><body><form id='f' action='/s'>\
        <fieldset disabled>\
          <legend><label>x<input name='leg' value='1'></label></legend>\
        </fieldset>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html2, "#f", None, base),
        Some("https://example.com/s?leg=1".to_string()),
        "legend 内嵌套控件豁免（descendant，leg=1）"
    );

    // ③ 仅首个 legend 豁免：第二个 legend 内控件**不**豁免（禁用，跳过）。
    let html3 = "<html><body><form id='f' action='/s'>\
        <fieldset disabled>\
          <legend>first</legend>\
          <legend><input name='leg2' value='1'></legend>\
        </fieldset>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html3, "#f", None, base),
        Some("https://example.com/s".to_string()),
        "仅首个 legend 豁免：第二个 legend 内控件禁用（leg2 跳过）"
    );

    // ④ 无 legend 的 disabled fieldset：内部控件全部禁用（既有行为，回归守卫）。
    let html4 = "<html><body><form id='f' action='/s'>\
        <fieldset disabled><input name='x' value='1'></fieldset>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html4, "#f", None, base),
        Some("https://example.com/s".to_string()),
        "无 legend 的 disabled fieldset：内部控件禁用（x 跳过，回归守卫）"
    );

    // ⑤ POST 表单同样遵循 legend 豁免（form_post_submission 复用 collect_form_data）。
    let html5 = "<html><body><form id='f' method='post' action='/s'>\
        <fieldset disabled><legend><input name='leg' value='1'></legend><input name='skip' value='2'></fieldset>\
        </form></body></html>";
    assert_eq!(
        form_post_submission(html5, "#f", None, base),
        Some(("https://example.com/s".to_string(), "leg=1".to_string())),
        "POST：legend 内控件启用（leg=1），legend 外禁用（skip 跳过）"
    );
}

#[test]
fn test_image_data_constructor_r3297() {
    // R3297：全局 `new ImageData(...)` 构造器（HTML ImageData spec）。此前缺 → 抛 TypeError。
    // 两形式：`new ImageData(w,h)` 透明黑全零；`new ImageData(dataArray, w[, h])` 包裹既有数据。
    // 产物 {width,height,data:Uint8ClampedArray,colorSpace:'srgb'}，与 ctx.createImageData 同构。
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

    // new ImageData(10, 10)：width/height + 透明黑全零 + data.length = 10*10*4。
    sandbox
        .execute(
            "var img = new ImageData(10, 10);\
             globalThis.__w = img.width; globalThis.__h = img.height;\
             globalThis.__len = img.data.length; globalThis.__first = img.data[0];\
             globalThis.__cs = img.colorSpace; globalThis.__ctor = (img instanceof ImageData);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__w)").unwrap().value, "10", "width=10");
    assert_eq!(sandbox.execute("String(globalThis.__h)").unwrap().value, "10", "height=10");
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "400",
        "data.length = 10*10*4 = 400"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__first)").unwrap().value,
        "0",
        "默认透明黑全零（data[0]=0）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cs)").unwrap().value,
        "srgb",
        "colorSpace='srgb'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ctor)").unwrap().value,
        "true",
        "instanceof ImageData"
    );
    // data 须为 Uint8ClampedArray（spec）。
    assert_eq!(
        sandbox
            .execute("String(new ImageData(2,2).data instanceof Uint8ClampedArray)")
            .unwrap()
            .value,
        "true",
        "data 须为 Uint8ClampedArray"
    );

    // new ImageData(dataArray, width)：包裹既有数据，高度由 length/(width*4) 推导。
    sandbox
        .execute(
            "var data = new Uint8ClampedArray(2*3*4);\
             for (var i = 0; i < data.length; i++) data[i] = (i % 256);\
             var img2 = new ImageData(data, 2);\
             globalThis.__w2 = img2.width; globalThis.__h2 = img2.height;\
             globalThis.__sameData = (img2.data === data); globalThis.__v = img2.data[5];",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__w2)").unwrap().value, "2", "width=2");
    assert_eq!(
        sandbox.execute("String(globalThis.__h2)").unwrap().value,
        "3",
        "height=3（24 字节 / (2*4) = 3）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sameData)").unwrap().value,
        "true",
        "data 引用同一数组（包裹非拷贝）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__v)").unwrap().value,
        "5",
        "data[5]=5（数据保真）"
    );

    // new ImageData(dataArray, width, height)：显式高度。
    sandbox
        .execute("var img3 = new ImageData(new Uint8ClampedArray(16), 2, 2); globalThis.__h3 = img3.height;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__h3)").unwrap().value,
        "2",
        "显式 height 参数生效"
    );

    // 互操作：new ImageData(2,2) 经 putImageData 写入 canvas，再 getImageData 回读（同构消费）。
    sandbox
        .execute(
            "var c = document.createElement('canvas'); c.width=2; c.height=2;\
             var cx = c.getContext('2d');\
             var src = new ImageData(2,2); src.data[0]=255;\
             cx.putImageData(src, 0, 0);\
             globalThis.__back = cx.getImageData(0,0,2,2).data[0];",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__back)").unwrap().value,
        "255",
        "new ImageData 经 putImageData 写入 + getImageData 回读保真"
    );
}

#[test]
fn test_canvas_float16_overlay_roundtrip_r34xx() {
    // R34xx：float16 上下文 + float16 ImageData → ImageBitmap → drawImage → getImageData
    // 越界值往返（[1.0, 2.0, -1.0, 1.0]——u8 wire/缓冲无法表达 2/-1，JS 侧覆盖层回读原始
    // 浮点像素）。驱动 WPT: 2d.imageData.createImageBitmap.srgb.rgba.float16。
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
            "globalThis.__c = document.createElement('canvas'); __c.width = 4; __c.height = 4;\
             globalThis.__ctx = __c.getContext('2d', {colorType: 'float16'});\
             globalThis.__id = new ImageData(2, 2, {pixelFormat: 'rgba-float16'});\
             for (var i = 0; i < 16; i++) __id.data[i] = [1.0, 2.0, -1.0, 1.0][i % 4];\
             globalThis.__bm = null;\
             createImageBitmap(__id).then(function (bm) { globalThis.__bm = bm; });",
        )
        .unwrap();
    // createImageBitmap 同步编码 + microtask 排空 → then 回调已执行。
    assert_eq!(
        sandbox.execute("String(__bm !== null)").unwrap().value,
        "true",
        "createImageBitmap promise 已解析"
    );
    assert_eq!(
        sandbox
            .execute("String(__bm.width + 'x' + __bm.height)")
            .unwrap()
            .value,
        "2x2",
        "bitmap 尺寸 2x2"
    );
    sandbox
        .execute(
            "__ctx.drawImage(__bm, 0, 0);\
             globalThis.__px = __ctx.getImageData(0, 0, 1, 1, {pixelFormat: 'rgba-float16'});",
        )
        .unwrap();
    // 越界值往返（spec 允许 float16 存 1.0/2.0/-1.0）：
    for (i, v) in [1.0, 2.0, -1.0, 1.0].iter().enumerate() {
        assert_eq!(
            sandbox
                .execute(&format!("String(Math.abs(__px.data[{i}] - {v}) <= 0.01)"))
                .unwrap()
                .value,
            "true",
            "channel {i} 往返 ≈ {v}"
        );
    }
    // 覆盖层失效：clearRect 后回读 → u8 域（0..1 归一化），非陈旧原始浮点。
    sandbox
        .execute(
            "__ctx.clearRect(0, 0, 4, 4);\
             globalThis.__px2 = __ctx.getImageData(0, 0, 1, 1, {pixelFormat: 'rgba-float16'});",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(__px2.data[0] <= 1.0)").unwrap().value,
        "true",
        "clearRect 后覆盖层失效（回读 u8 归一化域）"
    );

    // DOM 解析 canvas（WPT 用例形态——document.getElementById('c') 取 host-backed proxy，
    // getContext 走 part04 DOM canvas 路径，须与 standalone 同语义记录覆盖层）。
    *dom_html.lock().unwrap() =
        "<html><body><canvas id=\"c\" width=\"100\" height=\"50\"></canvas></body></html>".to_string();
    sandbox
        .execute(
            "globalThis.__dc = document.getElementById('c');\
             globalThis.__dctx = __dc.getContext('2d', {colorSpace: 'srgb', colorType: 'float16'});\
             globalThis.__id2 = new ImageData(10, 10, {colorSpace: 'srgb', pixelFormat: 'rgba-float16'});\
             for (var i = 0; i < 400; i++) __id2.data[i] = [1.0, 2.0, -1.0, 1.0][i % 4];\
             globalThis.__bm2 = null;\
             createImageBitmap(__id2).then(function (bm) { globalThis.__bm2 = bm; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(__bm2 !== null)").unwrap().value,
        "true",
        "DOM canvas createImageBitmap promise 已解析"
    );
    sandbox
        .execute(
            "__dctx.drawImage(__bm2, 0, 0);\
             globalThis.__dpx = __dctx.getImageData(0, 0, 1, 1, {colorSpace: 'srgb', pixelFormat: 'rgba-float16'});",
        )
        .unwrap();
    for (i, v) in [1.0, 2.0, -1.0, 1.0].iter().enumerate() {
        assert_eq!(
            sandbox
                .execute(&format!("String(Math.abs(__dpx.data[{i}] - {v}) <= 0.01)"))
                .unwrap()
                .value,
            "true",
            "DOM canvas channel {i} 往返 ≈ {v}"
        );
    }
}
