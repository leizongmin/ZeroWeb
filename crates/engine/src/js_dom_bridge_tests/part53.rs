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

// webcrypto M4a：crypto.subtle RSA 家族（RSA-OAEP / RSASSA-PKCS1-v1_5 / RSA-PSS）。
// generateKey 密钥对（publicKey 恒 extractable + usages 类型拆分）、OAEP round-trip（密文
// 长度 = 模长、随机化两密文不同、decrypt 空/坏长度语义）、RSASSA/PSS 签名确定性
//（pkcs1v15 与 saltLength=0 的 PSS 均确定性——两次签名逐字节同，PSS 错 saltLength 验签
// false）、spki/pkcs8/jwk 导入导出 round-trip、错误矩阵（e<3 → OperationError、类型
// usages 子集 SyntaxError、jwk kty/ext DataError、非 extractable 导出 InvalidAccessError）、
// RSA-OAEP 封装路径（wrapKey/unwrapKey raw round-trip）。WPT rsa_importKey/generateKey/
// rsa_oaep/rsa_pss/rsa_pkcs 断言语义（corpus 验收标尺），密钥材料不落日志。
// https://w3c.github.io/webcrypto/#rsa-oaep-operations  https://datatracker.ietf.org/doc/html/rfc8017
#[test]
fn test_crypto_subtle_rsa_m4a() {
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
            "function hex2b(h){var a=[];for(var i=0;i<h.length;i+=2)a.push(parseInt(h.substr(i,2),16));return new Uint8Array(a);}\
             function b2hex(u){var s='';for(var i=0;i<u.length;i++){s+=('0'+u[i].toString(16)).slice(-2);}return s;}\
             function errname(p){return p.then(function(){return 'no-throw';}, function(e){return e.name;});}\
             function b64u2b(s){s=s.replace(/-/g,'+').replace(/_/g,'/');while(s.length%4)s+='=';var bin=atob(s);var a=[];for(var i=0;i<bin.length;i++)a.push(bin.charCodeAt(i));return a;}",
        )
        .unwrap();

    // generateKey RSA-OAEP 1024：密钥对结构（publicKey 恒 extractable、RsaHashedKeyAlgorithm
    // 成员、usages 按类型拆分）+ OAEP round-trip（密文 = 模长字节、随机化）+ usage 校验。
    sandbox
        .execute(
            "globalThis.__pair='(pending)'; globalThis.__rt='(pending)'; globalThis.__rnd='(pending)';\
             globalThis.__e1='(pending)';\
             crypto.subtle.generateKey({name:'RSA-OAEP', hash:'SHA-256', modulusLength:1024, publicExponent:new Uint8Array([1,0,1])}, true, ['encrypt','decrypt','wrapKey','unwrapKey'])\
               .then(function(kp){ globalThis.__pair=JSON.stringify({\
                   pubEx:kp.publicKey.extractable, privEx:kp.privateKey.extractable,\
                   alg:kp.privateKey.algorithm.name, bits:kp.privateKey.algorithm.modulusLength,\
                   pe:Array.prototype.slice.call(new Uint8Array(kp.privateKey.algorithm.publicExponent)),\
                   hash:kp.privateKey.algorithm.hash.name,\
                   pubU:kp.publicKey.usages.join(','), privU:kp.privateKey.usages.join(','),\
                   pubType:kp.publicKey.type, privType:kp.privateKey.type});\
                 var msg=hex2b('00a1b2c3d4e5f60718');\
                 var c1=crypto.subtle.encrypt({name:'RSA-OAEP'}, kp.publicKey, msg);\
                 var c2=crypto.subtle.encrypt({name:'RSA-OAEP'}, kp.publicKey, msg);\
                 var dec=c1.then(function(ct){ globalThis.__ctLen=new Uint8Array(ct).length; return crypto.subtle.decrypt({name:'RSA-OAEP'}, kp.privateKey, ct); })\
                   .then(function(pt){ globalThis.__rt=b2hex(new Uint8Array(pt)); });\
                 var rnd=Promise.all([c1,c2]).then(function(cs){ globalThis.__rnd=(b2hex(new Uint8Array(cs[0]))!==b2hex(new Uint8Array(cs[1])))?'differ':'same'; });\
                 var e1=errname(crypto.subtle.decrypt({name:'RSA-OAEP'}, kp.publicKey, new Uint8Array(128).buffer))\
                   .then(function(n){ globalThis.__e1=n; });\
                 return Promise.all([dec,rnd,e1]); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__pair)").unwrap().value,
        r#"{"pubEx":true,"privEx":true,"alg":"RSA-OAEP","bits":1024,"pe":[1,0,1],"hash":"SHA-256","pubU":"encrypt,wrapKey","privU":"decrypt,unwrapKey","pubType":"public","privType":"private"}"#,
        "generateKey RSA-OAEP：密钥对结构 + RsaHashedKeyAlgorithm + usages 类型拆分"
    );
    assert_eq!(sandbox.execute("String(globalThis.__rt)").unwrap().value, "00a1b2c3d4e5f60718", "RSA-OAEP encrypt/decrypt round-trip");
    assert_eq!(sandbox.execute("String(globalThis.__ctLen)").unwrap().value, "128", "密文长度 = 模长字节（1024 位 → 128）");
    assert_eq!(sandbox.execute("String(globalThis.__rnd)").unwrap().value, "differ", "OAEP 随机化：两次加密密文不同");
    assert_eq!(sandbox.execute("String(globalThis.__e1)").unwrap().value, "InvalidAccessError", "publicKey（无 decrypt usage）decrypt → InvalidAccessError");

    // RSASSA-PKCS1-v1_5：签名确定性（两次同签）+ verify true/篡改 false + 跨算法密钥
    // InvalidAccessError；RSA-PSS saltLength=0 确定性 + 错 saltLength 验签 false。
    sandbox
        .execute(
            "globalThis.__sig1='(pending)'; globalThis.__sig2='(pending)'; globalThis.__v1='(pending)';\
             globalThis.__v2='(pending)'; globalThis.__e2='(pending)'; globalThis.__ps1='(pending)';\
             globalThis.__ps2='(pending)'; globalThis.__pv1='(pending)'; globalThis.__pv2='(pending)';\
             crypto.subtle.generateKey({name:'RSASSA-PKCS1-v1_5', hash:'SHA-256', modulusLength:1024, publicExponent:new Uint8Array([1,0,1])}, false, ['sign','verify'])\
               .then(function(kp){\
                 var msg=hex2b('deadbeef');\
                 var s1=crypto.subtle.sign({name:'RSASSA-PKCS1-v1_5'}, kp.privateKey, msg);\
                 var s2=crypto.subtle.sign({name:'RSASSA-PKCS1-v1_5'}, kp.privateKey, msg);\
                 var v1=s1.then(function(sig){ globalThis.__sig1=b2hex(new Uint8Array(sig));\
                   return crypto.subtle.verify({name:'RSASSA-PKCS1-v1_5'}, kp.publicKey, sig, msg); })\
                   .then(function(ok){ globalThis.__v1=String(ok); });\
                 var v2=s2.then(function(sig){ globalThis.__sig2=b2hex(new Uint8Array(sig));\
                   var t=new Uint8Array(sig); t[5]^=0xff;\
                   return crypto.subtle.verify({name:'RSASSA-PKCS1-v1_5'}, kp.publicKey, t.buffer, msg); })\
                   .then(function(ok){ globalThis.__v2=String(ok); });\
                 var e2=errname(crypto.subtle.sign({name:'RSA-PSS', saltLength:32}, kp.privateKey, msg))\
                   .then(function(n){ globalThis.__e2=n; });\
                 return Promise.all([v1,v2,e2]); })\
               .then(function(){\
                 return crypto.subtle.generateKey({name:'RSA-PSS', hash:'SHA-256', modulusLength:1024, publicExponent:new Uint8Array([1,0,1])}, false, ['sign','verify']); })\
               .then(function(kp){\
                 var msg=hex2b('cafebabe');\
                 var p1=crypto.subtle.sign({name:'RSA-PSS', saltLength:0}, kp.privateKey, msg);\
                 var p2=crypto.subtle.sign({name:'RSA-PSS', saltLength:0}, kp.privateKey, msg);\
                 var pv1=p1.then(function(sig){ globalThis.__ps1=b2hex(new Uint8Array(sig));\
                   return crypto.subtle.verify({name:'RSA-PSS', saltLength:0}, kp.publicKey, sig, msg); })\
                   .then(function(ok){ globalThis.__pv1=String(ok); });\
                 var pv2=p2.then(function(sig){ globalThis.__ps2=b2hex(new Uint8Array(sig));\
                   return crypto.subtle.verify({name:'RSA-PSS', saltLength:32}, kp.publicKey, sig, msg); })\
                   .then(function(ok){ globalThis.__pv2=String(ok); });\
                 return Promise.all([pv1,pv2]); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__sig1)").unwrap().value, sandbox.execute("String(globalThis.__sig2)").unwrap().value, "RSASSA-PKCS1-v1_5 签名确定性：两次同签");
    assert_eq!(sandbox.execute("String(globalThis.__v1)").unwrap().value, "true", "RSASSA verify 好签名 → true");
    assert_eq!(sandbox.execute("String(globalThis.__v2)").unwrap().value, "false", "RSASSA verify 篡改签名 → false");
    assert_eq!(sandbox.execute("String(globalThis.__e2)").unwrap().value, "InvalidAccessError", "RSASSA 密钥跨算法（PSS sign）→ InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__ps1)").unwrap().value, sandbox.execute("String(globalThis.__ps2)").unwrap().value, "RSA-PSS saltLength=0 签名确定性");
    assert_eq!(sandbox.execute("String(globalThis.__pv1)").unwrap().value, "true", "RSA-PSS saltLength=0 verify → true");
    assert_eq!(sandbox.execute("String(globalThis.__pv2)").unwrap().value, "false", "RSA-PSS 错 saltLength verify → false");

    // 导入导出 round-trip：pkcs8/spki 逐字节稳定、jwk 私钥分量（n/e/d）与重导入一致、
    // 非 extractable 导出 → InvalidAccessError。
    sandbox
        .execute(
            "globalThis.__spki='(pending)'; globalThis.__pk8='(pending)'; globalThis.__jwk='(pending)';\
             globalThis.__rt8='(pending)'; globalThis.__e3='(pending)';\
             crypto.subtle.generateKey({name:'RSA-OAEP', hash:'SHA-256', modulusLength:1024, publicExponent:new Uint8Array([1,0,1])}, true, ['encrypt','decrypt','wrapKey','unwrapKey'])\
               .then(function(kp){\
                 var e1=crypto.subtle.exportKey('spki', kp.publicKey).then(function(d){ globalThis.__spki=new Uint8Array(d).length; });\
                 var e2=crypto.subtle.exportKey('pkcs8', kp.privateKey).then(function(d){ return crypto.subtle.importKey('pkcs8', d, {name:'RSA-OAEP', hash:'SHA-256'}, true, ['decrypt','unwrapKey']); })\
                   .then(function(k){ return crypto.subtle.exportKey('pkcs8', k); })\
                   .then(function(d){ globalThis.__pk8=b2hex(new Uint8Array(d)).substring(0,16); });\
                 var e3=crypto.subtle.exportKey('jwk', kp.privateKey).then(function(j){\
                   globalThis.__jwk=JSON.stringify({kty:j.kty, hasD:!!j.d, hasP:!!j.p, hasQi:!!j.qi, alg:j.alg, ext:j.ext, ops:j.key_ops.join(','), nLen:b64u2b(j.n).length});\
                   var j2={kty:j.kty,n:j.n,e:j.e,d:j.d,p:j.p,q:j.q,dp:j.dp,dq:j.dq,qi:j.qi};\
                   return crypto.subtle.importKey('jwk', j2, {name:'RSA-OAEP', hash:'SHA-256'}, true, ['decrypt','unwrapKey']); })\
                   .then(function(k){ return crypto.subtle.exportKey('pkcs8', k); })\
                   .then(function(d){ globalThis.__rt8=b2hex(new Uint8Array(d)).substring(0,16); });\
                 var e4=crypto.subtle.generateKey({name:'RSA-OAEP', hash:'SHA-256', modulusLength:1024, publicExponent:new Uint8Array([1,0,1])}, false, ['decrypt','unwrapKey'])\
                   .then(function(np){ return errname(crypto.subtle.exportKey('pkcs8', np.privateKey)); })\
                   .then(function(n){ globalThis.__e3=n; });\
                 return Promise.all([e1,e2,e3,e4]); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__spki)").unwrap().value, "162", "exportKey spki（1024 位 SPKI 162 字节）");
    let pk8_prefix = sandbox.execute("String(globalThis.__pk8)").unwrap().value.clone();
    assert_eq!(sandbox.execute("String(globalThis.__rt8)").unwrap().value, pk8_prefix, "jwk 私钥重导入 → pkcs8 与原导出逐字节一致");
    assert!(pk8_prefix.len() == 16 && pk8_prefix != "(pending)", "pkcs8 导出非空");
    let jwk = sandbox.execute("String(globalThis.__jwk)").unwrap().value;
    assert_eq!(
        jwk,
        r#"{"kty":"RSA","hasD":true,"hasP":true,"hasQi":true,"alg":"RSA-OAEP-256","ext":true,"ops":"decrypt,unwrapKey","nLen":128}"#,
        "exportKey jwk 私钥：kty/d/p/qi/alg/ext/key_ops + n 128 字节"
    );
    assert_eq!(sandbox.execute("String(globalThis.__e3)").unwrap().value, "InvalidAccessError", "非 extractable 私钥导出 → InvalidAccessError");

    // 错误矩阵：e=1 生成 OperationError、空算法 TypeError、spki+sign usage SyntaxError、
    // pkcs8 空 usages SyntaxError、jwk kty 'oct' DataError、jwk alg 不匹配 DataError。
    sandbox
        .execute(
            "globalThis.__f1='(pending)'; globalThis.__f2='(pending)'; globalThis.__f3='(pending)';\
             globalThis.__f4='(pending)'; globalThis.__f5='(pending)'; globalThis.__f6='(pending)';\
             var f1=errname(crypto.subtle.generateKey({name:'RSA-OAEP', hash:'SHA-256', modulusLength:1024, publicExponent:new Uint8Array([1])}, true, ['encrypt','decrypt']))\
               .then(function(n){ globalThis.__f1=n; });\
             var f2=errname(crypto.subtle.generateKey({}, true, ['sign']))\
               .then(function(n){ globalThis.__f2=n; });\
             crypto.subtle.generateKey({name:'RSA-OAEP', hash:'SHA-256', modulusLength:1024, publicExponent:new Uint8Array([1,0,1])}, true, ['encrypt','decrypt','wrapKey','unwrapKey'])\
               .then(function(kp){\
                 var spki=kp.publicKey._raw.slice();\
                 var f3b=errname(crypto.subtle.importKey('spki', new Uint8Array(spki).buffer, {name:'RSA-OAEP', hash:'SHA-256'}, true, ['sign']))\
                   .then(function(n){ globalThis.__f3=n; });\
                 var pk8=kp.privateKey._raw.slice();\
                 var f4=errname(crypto.subtle.importKey('pkcs8', new Uint8Array(pk8).buffer, {name:'RSA-OAEP', hash:'SHA-256'}, true, []))\
                   .then(function(n){ globalThis.__f4=n; });\
                 var f5=errname(crypto.subtle.importKey('jwk', {kty:'oct', k:'AA'}, {name:'RSA-OAEP', hash:'SHA-256'}, true, ['encrypt']))\
                   .then(function(n){ globalThis.__f5=n; });\
                 var jwkBad={kty:'RSA', n:'AQAB', e:'AQAB', alg:'RS256'};\
                 var f6=errname(crypto.subtle.importKey('jwk', jwkBad, {name:'RSA-OAEP', hash:'SHA-256'}, true, ['encrypt']))\
                   .then(function(n){ globalThis.__f6=n; });\
                 return Promise.all([f3b,f4,f5,f6]); });\
             var _ = Promise.all([f1,f2]);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__f1)").unwrap().value, "OperationError", "e=1（非法指数）generateKey → OperationError");
    assert_eq!(sandbox.execute("String(globalThis.__f2)").unwrap().value, "TypeError", "空算法对象 generateKey → TypeError");
    assert_eq!(sandbox.execute("String(globalThis.__f3)").unwrap().value, "SyntaxError", "spki 公钥 + sign usage → SyntaxError");
    assert_eq!(sandbox.execute("String(globalThis.__f4)").unwrap().value, "SyntaxError", "pkcs8 私钥空 usages → SyntaxError");
    assert_eq!(sandbox.execute("String(globalThis.__f5)").unwrap().value, "DataError", "jwk kty 'oct' → DataError");
    assert_eq!(sandbox.execute("String(globalThis.__f6)").unwrap().value, "DataError", "jwk alg 不匹配（RS256 vs RSA-OAEP）→ DataError");

    // RSA-OAEP 封装路径：wrapKey('raw', AES 密钥, RSA wrapper) → unwrapKey → 解出密钥可用。
    sandbox
        .execute(
            "globalThis.__wrt='(pending)';\
             crypto.subtle.generateKey({name:'RSA-OAEP', hash:'SHA-256', modulusLength:1024, publicExponent:new Uint8Array([1,0,1])}, true, ['wrapKey','unwrapKey'])\
               .then(function(wp){\
                 return crypto.subtle.importKey('raw', hex2b('00112233445566778899aabbccddeeff'), {name:'AES-CBC'}, true, ['encrypt','decrypt'])\
                   .then(function(aes){\
                     return crypto.subtle.wrapKey('raw', aes, wp.publicKey, {name:'RSA-OAEP'})\
                       .then(function(w){ return crypto.subtle.unwrapKey('raw', w, wp.privateKey, {name:'RSA-OAEP'}, {name:'AES-CBC'}, true, ['encrypt','decrypt']); })\
                       .then(function(k){ return crypto.subtle.exportKey('raw', k); }); }); })\
               .then(function(raw){ globalThis.__wrt=b2hex(new Uint8Array(raw)); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__wrt)").unwrap().value,
        "00112233445566778899aabbccddeeff",
        "RSA-OAEP wrapKey/unwrapKey raw round-trip"
    );
}
