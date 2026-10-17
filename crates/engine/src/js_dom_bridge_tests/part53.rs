// webcrypto M3b：crypto.subtle wrapKey/unwrapKey（AES-KW RFC 3394 + AES-GCM/CBC/CTR 封装路径）
// + jwk 导入/导出格式（AES 家族/HMAC）+ spki/pkcs8 对称密钥语义 + usages/length 错误矩阵。
// 向量锚定 RFC 3394 §4.1（KEK 000102..0e0f，KeyData 00112233..ddeeff，wrapped
// 1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5）；WPT wrapKey_unwrapKey /
// symmetric_importKey 断言语义（corpus 验收标尺）。AES-KW jwk 封装要求 JSON 8 字节对齐
//（spec 非对齐 → OperationError，WPT wrappingIsPossible 同门控），jwk 格式 round-trip 走
// AES-CBC wrapper 覆盖。
// https://w3c.github.io/webcrypto/#SubtleCrypto-method-wrapKey  https://datatracker.ietf.org/doc/html/rfc3394
#[test]
fn test_crypto_subtle_kw_jwk_m3b() {
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

    // hex 辅助（part05 同款）。
    sandbox
        .execute(
            "function hex2b(h){var a=[];for(var i=0;i<h.length;i+=2)a.push(parseInt(h.substr(i,2),16));return new Uint8Array(a);}\
             function b2hex(u){var s='';for(var i=0;i<u.length;i++){s+=('0'+u[i].toString(16)).slice(-2);}return s;}\
             function errname(p){return p.then(function(){return 'no-throw';}, function(e){return e.name;});}",
        )
        .unwrap();

    // RFC 3394 §4.1 wrap 向量：AES-128 KEK 封装 16 字节 AES-CBC 密钥 → 24 字节封装输出。
    sandbox
        .execute(
            "globalThis.__kwv='(pending)';\
             crypto.subtle.importKey('raw', hex2b('000102030405060708090a0b0c0d0e0f'), {name:'AES-KW'}, false, ['wrapKey','unwrapKey'])\
               .then(function(kek){ globalThis.__kek=kek;\
                 return crypto.subtle.importKey('raw', hex2b('00112233445566778899aabbccddeeff'), {name:'AES-CBC'}, true, ['encrypt','decrypt']); })\
               .then(function(tk){ globalThis.__tk=tk;\
                 return crypto.subtle.wrapKey('raw', tk, globalThis.__kek, {name:'AES-KW'}); })\
               .then(function(w){ globalThis.__kwv=String(b2hex(new Uint8Array(w))); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__kwv)").unwrap().value,
        "1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5",
        "AES-KW wrap == RFC 3394 §4.1 test vector"
    );

    // unwrap 回 RFC 3394 向量 + 坏输入矩阵：未对齐 payload → OperationError、错 KEK unwrap →
    // OperationError、篡改密文 unwrap → OperationError（IV 校验）。
    sandbox
        .execute(
            "globalThis.__kwrt='(pending)'; globalThis.__kwa='(pending)'; globalThis.__kwb='(pending)'; globalThis.__kwc='(pending)';\
             var wrapped=hex2b('1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5');\
             var p0=crypto.subtle.unwrapKey('raw', wrapped.buffer, globalThis.__kek, {name:'AES-KW'}, {name:'AES-CBC'}, true, ['encrypt'])\
               .then(function(k){ globalThis.__kwrt=k; });\
             crypto.subtle.importKey('raw', hex2b('aabbccdd'), {name:'HMAC',hash:'SHA-256'}, true, ['sign'])\
               .then(function(shortKey){\
                 var p1=errname(crypto.subtle.wrapKey('raw', shortKey, globalThis.__kek, {name:'AES-KW'}))\
                   .then(function(n){ globalThis.__kwa=n; });\
                 var p2=errname(crypto.subtle.wrapKey('raw', globalThis.__kek, globalThis.__kek, {name:'AES-KW'}))\
                   .then(function(n){ globalThis.__kwb=n; });\
                 var tampered=new Uint8Array(wrapped); tampered[10]^=0xff;\
                 var p3=errname(crypto.subtle.unwrapKey('raw', tampered.buffer, globalThis.__kek, {name:'AES-KW'}, {name:'AES-CBC'}, true, ['encrypt']))\
                   .then(function(n){ globalThis.__kwc=n; });\
                 return Promise.all([p1,p2,p3]); });",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(b2hex(new Uint8Array(globalThis.__kwrt._raw)))")
            .unwrap()
            .value,
        "00112233445566778899aabbccddeeff",
        "AES-KW unwrap 回 RFC 3394 §4.1 明文"
    );
    assert_eq!(sandbox.execute("String(globalThis.__kwa)").unwrap().value, "OperationError", "未对齐 payload → OperationError");
    assert_eq!(sandbox.execute("String(globalThis.__kwb)").unwrap().value, "InvalidAccessError", "非 extractable 密钥不可封装 → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__kwc)").unwrap().value, "OperationError", "篡改密文（IV 不匹配）→ OperationError");

    // wrapKey/unwrapKey 全链 round-trip：AES-KW raw、AES-CBC wrapper jwk、256 位 KEK。
    sandbox
        .execute(
            "globalThis.__rt1='(pending)'; globalThis.__rt2='(pending)'; globalThis.__rt3='(pending)';\
             var kek256=hex2b('603deb1015ca71be2b73aef0857d77811f352c073b6108d72d9810a30914dff4');\
             var cbcWrapK=null;\
             crypto.subtle.importKey('raw', hex2b('000102030405060708090a0b0c0d0e0f'), {name:'AES-CBC'}, false, ['wrapKey','unwrapKey'])\
               .then(function(cwk){ cbcWrapK=cwk;\
                 var a=crypto.subtle.wrapKey('raw', globalThis.__tk, globalThis.__kek, {name:'AES-KW'})\
                   .then(function(w){ return crypto.subtle.unwrapKey('raw', w, globalThis.__kek, {name:'AES-KW'}, {name:'AES-CBC'}, true, ['encrypt','decrypt']); })\
                   .then(function(k){ return crypto.subtle.exportKey('raw', k); })\
                   .then(function(raw){ globalThis.__rt1=String(b2hex(new Uint8Array(raw))); });\
                 var b=crypto.subtle.wrapKey('jwk', globalThis.__tk, cbcWrapK, {name:'AES-CBC', iv:new Uint8Array(16)})\
                   .then(function(w){ return crypto.subtle.unwrapKey('jwk', w, cbcWrapK, {name:'AES-CBC', iv:new Uint8Array(16)}, {name:'AES-CBC'}, true, ['encrypt','decrypt']); })\
                   .then(function(k){ return crypto.subtle.exportKey('raw', k); })\
                   .then(function(raw){ globalThis.__rt2=String(b2hex(new Uint8Array(raw))); });\
                 var c=crypto.subtle.importKey('raw', kek256, {name:'AES-KW'}, false, ['wrapKey','unwrapKey'])\
                   .then(function(k256){ return crypto.subtle.wrapKey('raw', globalThis.__tk, k256, {name:'AES-KW'})\
                     .then(function(w){ return crypto.subtle.unwrapKey('raw', w, k256, {name:'AES-KW'}, {name:'AES-CBC'}, true, ['encrypt','decrypt']); }); })\
                   .then(function(k){ return crypto.subtle.exportKey('raw', k); })\
                   .then(function(raw){ globalThis.__rt3=String(b2hex(new Uint8Array(raw))); });\
                 return Promise.all([a,b,c]); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__rt1)").unwrap().value, "00112233445566778899aabbccddeeff", "AES-KW raw 封装 round-trip");
    assert_eq!(sandbox.execute("String(globalThis.__rt2)").unwrap().value, "00112233445566778899aabbccddeeff", "AES-CBC wrapper jwk 封装 round-trip");
    assert_eq!(sandbox.execute("String(globalThis.__rt3)").unwrap().value, "00112233445566778899aabbccddeeff", "AES-KW 256 位 KEK round-trip");

    // wrapKey usage/算法匹配错误矩阵：wrapper 无 "wrapKey" → InvalidAccessError、wrap 算法与
    // wrapper 不匹配 → InvalidAccessError、spki/pkcs8 对称导入 → NotSupportedError、AES-KW
    // encrypt usage → SyntaxError、空 usages → SyntaxError（importKey 与 generateKey 两侧）。
    sandbox
        .execute(
            "globalThis.__e1='(pending)'; globalThis.__e2='(pending)';\
             globalThis.__e4='(pending)'; globalThis.__e5='(pending)'; globalThis.__e6='(pending)';\
             globalThis.__e7='(pending)'; globalThis.__e8='(pending)';\
             crypto.subtle.importKey('raw', hex2b('000102030405060708090a0b0c0d0e0f'), {name:'AES-CBC'}, true, ['encrypt','decrypt'])\
               .then(function(noWrapKey){\
                 var p1=errname(crypto.subtle.wrapKey('raw', globalThis.__tk, noWrapKey, {name:'AES-KW'}))\
                   .then(function(n){ globalThis.__e1=n; });\
                 var p2=errname(crypto.subtle.wrapKey('raw', globalThis.__tk, globalThis.__kek, {name:'AES-GCM', iv:new Uint8Array(16)}))\
                   .then(function(n){ globalThis.__e2=n; });\
                 return Promise.all([p1,p2]); })\
               .then(function(){\
                 var p4=errname(crypto.subtle.importKey('spki', new Uint8Array(16).buffer, {name:'AES-CBC'}, true, ['encrypt']))\
                   .then(function(n){ globalThis.__e4=n; });\
                 var p5=errname(crypto.subtle.importKey('raw', hex2b('00112233'), {name:'AES-KW'}, true, ['encrypt']))\
                   .then(function(n){ globalThis.__e5=n; });\
                 var p6=errname(crypto.subtle.importKey('raw', new Uint8Array(16).buffer, {name:'AES-CBC'}, true, []))\
                   .then(function(n){ globalThis.__e6=n; });\
                 var p7=errname(crypto.subtle.generateKey({name:'AES-KW'}, true, []))\
                   .then(function(n){ globalThis.__e7=n; });\
                 var p8=errname(crypto.subtle.generateKey({name:'AES-CBC', length:128}, true, ['encrypt','decrypt','sign']))\
                   .then(function(n){ globalThis.__e8=n; });\
                 return Promise.all([p4,p5,p6,p7,p8]); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e1)").unwrap().value, "InvalidAccessError", "wrapper 无 wrapKey usage → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__e2)").unwrap().value, "InvalidAccessError", "wrap 算法名与 wrapper 不匹配 → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__e4)").unwrap().value, "NotSupportedError", "对称密钥 spki 导入 → NotSupportedError");
    assert_eq!(sandbox.execute("String(globalThis.__e5)").unwrap().value, "SyntaxError", "AES-KW encrypt usage 非法 → SyntaxError");
    assert_eq!(sandbox.execute("String(globalThis.__e6)").unwrap().value, "SyntaxError", "importKey 空 usages → SyntaxError");
    assert_eq!(sandbox.execute("String(globalThis.__e7)").unwrap().value, "SyntaxError", "generateKey AES-KW 空 usages → SyntaxError");
    assert_eq!(sandbox.execute("String(globalThis.__e8)").unwrap().value, "SyntaxError", "generateKey 含非法 usage → SyntaxError（先于 length 校验）");

    // jwk 导入/导出 round-trip：字段断言（kty/k/alg/key_ops/ext）+ 误用 kty → DataError +
    // ext:false + extractable:true → DataError。
    sandbox
        .execute(
            "globalThis.__j1='(pending)'; globalThis.__j2='(pending)'; globalThis.__j3='(pending)'; globalThis.__j4='(pending)';\
             crypto.subtle.exportKey('jwk', globalThis.__tk)\
               .then(function(j){ globalThis.__jwk=j;\
                 globalThis.__j1=JSON.stringify({kty:j.kty,alg:j.alg,ext:j.ext,klen:atob(j.k.replace(/-/g,'+').replace(/_/g,'/')).length,ops:j.key_ops.join(',')});\
                 var copy={kty:j.kty,k:j.k,alg:j.alg,ext:j.ext};\
                 return crypto.subtle.importKey('jwk', copy, {name:'AES-CBC'}, true, ['encrypt','decrypt']); })\
               .then(function(k){ return crypto.subtle.exportKey('raw', k); })\
               .then(function(raw){ globalThis.__j2=String(b2hex(new Uint8Array(raw))); });\
             var p3=errname(crypto.subtle.importKey('jwk', {kty:'RSA', k:'AA'}, {name:'AES-CBC'}, true, ['encrypt']))\
               .then(function(n){ globalThis.__j3=n; });\
             var p4=errname(crypto.subtle.importKey('jwk', {kty:'oct', k:'MDEyMzQ1Njc4OWFiY2RlZg', alg:'A128CBC', ext:false}, {name:'AES-CBC'}, true, ['encrypt']))\
               .then(function(n){ globalThis.__j4=n; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__j1)").unwrap().value,
        r#"{"kty":"oct","alg":"A128CBC","ext":true,"klen":16,"ops":"encrypt,decrypt"}"#,
        "jwk 导出字段（kty/k/alg/key_ops/ext）"
    );
    assert_eq!(sandbox.execute("String(globalThis.__j2)").unwrap().value, "00112233445566778899aabbccddeeff", "jwk 导入 round-trip");
    assert_eq!(sandbox.execute("String(globalThis.__j3)").unwrap().value, "DataError", "kty 非 oct → DataError");
    assert_eq!(sandbox.execute("String(globalThis.__j4)").unwrap().value, "DataError", "ext:false + extractable:true → DataError");

    // AES-GCM 16 字节 IV 封装路径（wrapKey 测试 GCM wrapper 固定 iv 16B）。
    sandbox
        .execute(
            "globalThis.__g1='(pending)';\
             crypto.subtle.importKey('raw', hex2b('000102030405060708090a0b0c0d0e0f'), {name:'AES-GCM'}, false, ['wrapKey','unwrapKey'])\
               .then(function(gk){\
                 return crypto.subtle.wrapKey('raw', globalThis.__tk, gk, {name:'AES-GCM', iv:new Uint8Array(16), additionalData:new Uint8Array(16), tagLength:128})\
                   .then(function(w){ return crypto.subtle.unwrapKey('raw', w, gk, {name:'AES-GCM', iv:new Uint8Array(16), additionalData:new Uint8Array(16), tagLength:128}, {name:'AES-CBC'}, true, ['encrypt','decrypt']); }); })\
               .then(function(k){ return crypto.subtle.exportKey('raw', k); })\
               .then(function(raw){ globalThis.__g1=String(b2hex(new Uint8Array(raw))); }, function(e){ globalThis.__g1='ERR:'+e.name+':'+e.message; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__g1)").unwrap().value, "00112233445566778899aabbccddeeff", "AES-GCM 16 字节 IV 封装 round-trip");

    // generateKey 密钥算法成员：AES-KW → algorithm.length=128 + usages 集合；HMAC 显式 length
    // → key.algorithm.length 与 32 字节材料；importKey HMAC 显式 length 语义；length 0 →
    // DataError。
    sandbox
        .execute(
            "globalThis.__k1='(pending)'; globalThis.__k2='(pending)'; globalThis.__k3='(pending)'; globalThis.__k4='(pending)';\
             crypto.subtle.generateKey({name:'AES-KW', length:128}, true, ['wrapKey','unwrapKey'])\
               .then(function(k){ globalThis.__k1=JSON.stringify({name:k.algorithm.name,len:k.algorithm.length,u:k.usages.join(','),tag:Object.prototype.toString.call(k).slice(8,-1)}); });\
             crypto.subtle.generateKey({name:'HMAC', hash:'SHA-256', length:256}, true, ['sign'])\
               .then(function(k){ globalThis.__k2=JSON.stringify({name:k.algorithm.name,hash:k.algorithm.hash,len:k.algorithm.length,raw:k._raw.length}); });\
             var p3=crypto.subtle.importKey('raw', new Uint8Array(20).fill(0x0b), {name:'HMAC',hash:'SHA-1',length:160}, false, ['sign'])\
               .then(function(k){ globalThis.__k3=JSON.stringify({len:k.algorithm.length,raw:k._raw.length}); });\
             var p4=errname(crypto.subtle.importKey('raw', new Uint8Array(20), {name:'HMAC',hash:'SHA-1',length:0}, false, ['sign']))\
               .then(function(n){ globalThis.__k4=n; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__k1)").unwrap().value,
        r#"{"name":"AES-KW","len":128,"u":"wrapKey,unwrapKey","tag":"CryptoKey"}"#,
        "generateKey AES-KW：AesKeyAlgorithm.length + usages + toStringTag"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__k2)").unwrap().value,
        r#"{"name":"HMAC","hash":{"name":"SHA-256"},"len":256,"raw":32}"#,
        "generateKey HMAC 显式 length：HmacKeyAlgorithm（hash 为 KeyAlgorithm 字典）+ 密钥材料"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__k3)").unwrap().value,
        r#"{"len":160,"raw":20}"#,
        "importKey HMAC 显式 length == dataBits"
    );
    assert_eq!(sandbox.execute("String(globalThis.__k4)").unwrap().value, "DataError", "HMAC length 0 → DataError");
}
