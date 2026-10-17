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

// webcrypto M4b：crypto.subtle EC 家族（ECDSA/ECDH，namedCurve P-256/P-384/P-521）全链——
// generateKey CryptoKeyPair（publicKey 恒 extractable + usages 类型拆分：ECDSA
// public ⊆ [verify] / private ⊆ [sign]，ECDH public 恒 []）、importKey（spki/pkcs8/jwk/
// raw 含 SEC1 压缩点）、exportKey 四格式（spki / raw = SEC1 未压缩点 / pkcs8 / jwk
// x/y/d 定宽零填充）、ECDSA sign/verify（RFC 6979 确定性 nonce，IEEE P1363 r||s 定宽
// 拼接）、ECDH deriveBits/deriveKey（共享点 x 坐标，length 位截断语义）。向量锚定
// RFC 6979 §A.2.5（P-256 + SHA-256，msg "sample"，签名 r||s 全 64 字节）；WPT
// ec_importKey / generateKey EC 四文件 / sign_verify ecdsa / derive_bits_keys ecdh /
// serialization ecdsa+ecdh 断言语义（corpus 验收标尺）。
// https://w3c.github.io/webcrypto/#ecdsa-operations  https://w3c.github.io/webcrypto/#ecdh-operations
// https://datatracker.ietf.org/doc/html/rfc6979
#[test]
fn test_crypto_subtle_ec_m4b() {
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

    // generateKey ECDSA P-256：密钥对结构（publicKey 恒 extractable、EcKeyAlgorithm
    // 成员无 hash、usages 按类型拆分 sign/verify）+ RFC 6979 §A.2.5 向量（jwk 导入
    // 固定私钥，msg "sample" + SHA-256 → 签名 r||s 全 64 字节逐字节锚定）+ 导出 spki
    // 验签 true / 篡改 false。
    sandbox
        .execute(
            "globalThis.__pair='(pending)'; globalThis.__sigv='(pending)'; globalThis.__vv='(pending)';\
             globalThis.__vvt='(pending)'; globalThis.__det1='(pending)'; globalThis.__det2='(pending)';\
             globalThis.__ecErr='(none)'; globalThis.__vspki='(none)';\
             crypto.subtle.generateKey({name:'ECDSA', namedCurve:'P-256'}, true, ['sign','verify'])\
               .then(function(kp){ globalThis.__pair=JSON.stringify({\
                   pubEx:kp.publicKey.extractable, privEx:kp.privateKey.extractable,\
                   alg:kp.privateKey.algorithm.name, curve:kp.privateKey.algorithm.namedCurve,\
                   pubU:kp.publicKey.usages.join(','), privU:kp.privateKey.usages.join(','),\
                   pubType:kp.publicKey.type, privType:kp.privateKey.type});\
                 var s1=crypto.subtle.sign({name:'ECDSA', hash:'SHA-256'}, kp.privateKey, hex2b('deadbeef'));\
                 var s2=crypto.subtle.sign({name:'ECDSA', hash:'SHA-256'}, kp.privateKey, hex2b('deadbeef'));\
                 return Promise.all([s1,s2]).then(function(ss){\
                   globalThis.__det1=b2hex(new Uint8Array(ss[0]));\
                   globalThis.__det2=b2hex(new Uint8Array(ss[1])); }); });\
             var vkey=null;\
             crypto.subtle.importKey('jwk',\
               {kty:'EC', crv:'P-256',\
                x:'YP7UuiVanTHJYet0xjVtaMBJuJI7Yfps5mliLmDyn7Y',\
                y:'eQP-EAi4vJmkGunpVii8ZPLxsgwtfp9Rd6PClNRGIpk',\
                d:'ya-p2EW6dRZrXCFXZ7HWk05Qw9s26JsSe4piKxIPZyE'},\
               {name:'ECDSA', namedCurve:'P-256'}, true, ['sign'])\
               .then(function(vk){ globalThis.__vkey=vk; vkey=vk;\
                 return crypto.subtle.sign({name:'ECDSA', hash:'SHA-256'}, vk, hex2b('73616d706c65')); })\
               .then(function(sig){ globalThis.__sigv=b2hex(new Uint8Array(sig));\
                 return crypto.subtle.exportKey('spki', vkey); })\
               .then(function(spki){ globalThis.__vspki=b2hex(new Uint8Array(spki)).substring(0,24);\
                 return crypto.subtle.importKey('spki', spki, {name:'ECDSA', namedCurve:'P-256'}, true, ['verify']); })\
               .then(function(pk){\
                 var pv=crypto.subtle.verify({name:'ECDSA', hash:'SHA-256'}, pk, hex2b('efd48b2aacb6a8fd1140dd9cd45e81d69d2c877b56aaf991c34d0ea84eaf3716f7cb1c942d657c41d436c7a1b6e29f65f3e900dbb9aff4064dc4ab2f843acda8').buffer, hex2b('73616d706c65'))\
                   .then(function(ok){ globalThis.__vv=String(ok); });\
                 var bad=hex2b('efd48b2aacb6a8fd1140dd9cd45e81d69d2c877b56aaf991c34d0ea84eaf3716f7cb1c942d657c41d436c7a1b6e29f65f3e900dbb9aff4064dc4ab2f843acda8'); bad[3]^=0xff;\
                 var pt=crypto.subtle.verify({name:'ECDSA', hash:'SHA-256'}, pk, bad.buffer, hex2b('73616d706c65'))\
                   .then(function(ok){ globalThis.__vvt=String(ok); });\
                 return Promise.all([pv,pt]); }, function(e){ globalThis.__ecErr=e.name+': '+e.message; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__pair)").unwrap().value,
        r#"{"pubEx":true,"privEx":true,"alg":"ECDSA","curve":"P-256","pubU":"verify","privU":"sign","pubType":"public","privType":"private"}"#,
        "generateKey ECDSA：密钥对结构 + usages 类型拆分 + publicKey 恒 extractable"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sigv)").unwrap().value,
        "efd48b2aacb6a8fd1140dd9cd45e81d69d2c877b56aaf991c34d0ea84eaf3716f7cb1c942d657c41d436c7a1b6e29f65f3e900dbb9aff4064dc4ab2f843acda8",
        "ECDSA P-256/SHA-256 msg 'sample' == RFC 6979 §A.2.5 签名向量"
    );
    assert_eq!(sandbox.execute("String(globalThis.__ecErr)").unwrap().value, "(none)", "RFC 6979 链无 reject（断言失败时 left 即链上 reject 的错误）");
    assert_eq!(sandbox.execute("String(globalThis.__vv)").unwrap().value, "true", "RFC 6979 向量验签（spki 导入公钥）→ true");
    assert_eq!(sandbox.execute("String(globalThis.__vvt)").unwrap().value, "false", "篡改签名验签 → false");
    assert_eq!(
        sandbox.execute("String(globalThis.__det1)").unwrap().value,
        sandbox.execute("String(globalThis.__det2)").unwrap().value,
        "ECDSA 签名确定性（RFC 6979 nonce）：两次同签"
    );

    // sign/verify 错误序矩阵：public sign → InvalidAccessError、private verify →
    // InvalidAccessError、ECDH 密钥跨算法 sign → InvalidAccessError、hash 缺失 →
    // TypeError、未注册 hash（"SHA256" 无连字符）→ NotSupportedError 先于 key 校验。
    sandbox
        .execute(
            "globalThis.__e1='(pending)'; globalThis.__e2='(pending)'; globalThis.__e3='(pending)';\
             globalThis.__e4='(pending)'; globalThis.__e5='(pending)';\
             crypto.subtle.generateKey({name:'ECDSA', namedCurve:'P-256'}, true, ['sign','verify'])\
               .then(function(kp){\
                 var m=hex2b('aa');\
                 var e1=errname(crypto.subtle.sign({name:'ECDSA', hash:'SHA-256'}, kp.publicKey, m))\
                   .then(function(n){ globalThis.__e1=n; });\
                 var e2=errname(crypto.subtle.verify({name:'ECDSA', hash:'SHA-256'}, kp.privateKey, m, m))\
                   .then(function(n){ globalThis.__e2=n; });\
                 var e4=errname(crypto.subtle.sign({name:'ECDSA'}, kp.privateKey, m))\
                   .then(function(n){ globalThis.__e4=n; });\
                 var e5=errname(crypto.subtle.sign({name:'ECDSA', hash:'SHA256'}, kp.privateKey, m))\
                   .then(function(n){ globalThis.__e5=n; });\
                 return Promise.all([e1,e2,e4,e5]); })\
               .then(function(){ return crypto.subtle.generateKey({name:'ECDH', namedCurve:'P-256'}, true, ['deriveKey','deriveBits']); })\
               .then(function(ep){\
                 return errname(crypto.subtle.sign({name:'ECDSA', hash:'SHA-256'}, ep.privateKey, hex2b('aa')))\
                   .then(function(n){ globalThis.__e3=n; }); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e1)").unwrap().value, "InvalidAccessError", "publicKey sign → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__e2)").unwrap().value, "InvalidAccessError", "privateKey verify → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__e3)").unwrap().value, "InvalidAccessError", "ECDH 密钥跨算法 sign → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__e4)").unwrap().value, "TypeError", "hash 成员缺失 → TypeError");
    assert_eq!(sandbox.execute("String(globalThis.__e5)").unwrap().value, "NotSupportedError", "未注册 hash 名（SHA256）→ NotSupportedError");

    // ECDH deriveBits 代数锚定（双向共享秘密一致）+ length 语义（缺省域全宽 / 截断
    // 左起 N 位 / 0 → 空 / 超域宽 → OperationError / 非 8 倍数尾字节高位掩码）+
    // public base key → InvalidAccessError + 曲线不匹配 → InvalidAccessError +
    // public 缺失 → TypeError；deriveKey → AES-GCM 派生密钥 encrypt/decrypt round-trip。
    sandbox
        .execute(
            "globalThis.__dh='(pending)'; globalThis.__dhr='(pending)'; globalThis.__dh16='(pending)';\
             globalThis.__dh0='(pending)'; globalThis.__dh7='(pending)'; globalThis.__dho='(pending)';\
             globalThis.__dhe1='(pending)'; globalThis.__dhe2='(pending)'; globalThis.__dhe3='(pending)';\
             globalThis.__dk='(pending)';\
             var pA=crypto.subtle.generateKey({name:'ECDH', namedCurve:'P-256'}, false, ['deriveKey','deriveBits']);\
             var pB=crypto.subtle.generateKey({name:'ECDH', namedCurve:'P-256'}, false, ['deriveKey','deriveBits']);\
             var pC=crypto.subtle.generateKey({name:'ECDH', namedCurve:'P-384'}, false, ['deriveKey','deriveBits']);\
             Promise.all([pA,pB,pC]).then(function(ps){\
               var A=ps[0], B=ps[1], C=ps[2];\
               var d1=crypto.subtle.deriveBits({name:'ECDH', public:A.publicKey}, B.privateKey, 256)\
                 .then(function(b){ globalThis.__dh=b2hex(new Uint8Array(b)); });\
               var d2=crypto.subtle.deriveBits({name:'ECDH', public:B.publicKey}, A.privateKey, 256)\
                 .then(function(b){ globalThis.__dhr=b2hex(new Uint8Array(b)); });\
               var d3=crypto.subtle.deriveBits({name:'ECDH', public:A.publicKey}, B.privateKey, 128)\
                 .then(function(b){ globalThis.__dh16=b2hex(new Uint8Array(b)); });\
               var d4=crypto.subtle.deriveBits({name:'ECDH', public:A.publicKey}, B.privateKey, 0)\
                 .then(function(b){ globalThis.__dh0=String(new Uint8Array(b).length); });\
               var d5=crypto.subtle.deriveBits({name:'ECDH', public:A.publicKey}, B.privateKey, 255)\
                 .then(function(b){ globalThis.__dh7=b2hex(new Uint8Array(b)); });\
               var d6=errname(crypto.subtle.deriveBits({name:'ECDH', public:A.publicKey}, B.privateKey, 257))\
                 .then(function(n){ globalThis.__dho=n; });\
               var d7=errname(crypto.subtle.deriveBits({name:'ECDH', public:A.publicKey}, A.publicKey, 256))\
                 .then(function(n){ globalThis.__dhe1=n; });\
               var d8=errname(crypto.subtle.deriveBits({name:'ECDH', public:C.publicKey}, A.privateKey, 256))\
                 .then(function(n){ globalThis.__dhe2=n; });\
               var d9=errname(crypto.subtle.deriveBits({name:'ECDH'}, A.privateKey, 256))\
                 .then(function(n){ globalThis.__dhe3=n; });\
               var dkk=crypto.subtle.deriveKey({name:'ECDH', public:A.publicKey}, B.privateKey,\
                   {name:'AES-GCM', length:128}, true, ['encrypt','decrypt'])\
                 .then(function(k){ return crypto.subtle.encrypt({name:'AES-GCM', iv:hex2b('00112233445566778899aabbccddeeff')}, k, hex2b('cafebabe'))\
                   .then(function(ct){ return crypto.subtle.decrypt({name:'AES-GCM', iv:hex2b('00112233445566778899aabbccddeeff')}, k, ct); })\
                   .then(function(pt){ globalThis.__dk=b2hex(new Uint8Array(pt)); }); });\
               return Promise.all([d1,d2,d3,d4,d5,d6,d7,d8,d9,dkk]); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dh)").unwrap().value,
        sandbox.execute("String(globalThis.__dhr)").unwrap().value,
        "ECDH 双向共享秘密一致"
    );
    let dh_full = sandbox.execute("String(globalThis.__dh)").unwrap().value.clone();
    assert_eq!(dh_full.len(), 64, "ECDH P-256 缺省（256 位）→ 32 字节共享秘密");
    assert_eq!(
        sandbox.execute("String(globalThis.__dh16)").unwrap().value,
        dh_full[..32].to_string(),
        "deriveBits length 128 → 共享秘密左起 16 字节"
    );
    assert_eq!(sandbox.execute("String(globalThis.__dh0)").unwrap().value, "0", "deriveBits length 0 → 空");
    let dh7 = sandbox.execute("String(globalThis.__dh7)").unwrap().value;
    assert_eq!(dh7.len(), 64, "deriveBits length 255 → 32 字节（尾字节部分位）");
    assert_eq!(&dh7[..62], &dh_full[..62], "255 位截断前 31 字节与全宽一致");
    assert_eq!(dh7[62..], format!("{:02x}", u8::from_str_radix(&dh_full[62..64], 16).unwrap() & 0xfe), "255 位尾字节 = 全宽尾字节高位掩码（&0xfe）");
    assert_eq!(sandbox.execute("String(globalThis.__dho)").unwrap().value, "OperationError", "deriveBits length 超域宽 → OperationError");
    assert_eq!(sandbox.execute("String(globalThis.__dhe1)").unwrap().value, "InvalidAccessError", "public base key deriveBits → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__dhe2)").unwrap().value, "InvalidAccessError", "曲线不匹配 → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__dhe3)").unwrap().value, "TypeError", "public 成员缺失 → TypeError");
    assert_eq!(sandbox.execute("String(globalThis.__dk)").unwrap().value, "cafebabe", "ECDH deriveKey → AES-GCM round-trip");

    // P-384/P-521 曲线覆盖：ECDSA P-521 sign/verify round-trip（确定性）、ECDH P-384
    // deriveBits 缺省 48 字节、jwk x/y/d base64url 定宽（P-521 66 字节 → 88 字符，
    // generateKey successes 一致性断言语义）。
    sandbox
        .execute(
            "globalThis.__s521='(pending)'; globalThis.__v521='(pending)'; globalThis.__dh384='(pending)';\
             globalThis.__j521='(pending)';\
             crypto.subtle.generateKey({name:'ECDSA', namedCurve:'P-521'}, true, ['sign','verify'])\
               .then(function(kp){\
                 var m=hex2b('beef');\
                 var s=crypto.subtle.sign({name:'ECDSA', hash:'SHA-384'}, kp.privateKey, m);\
                 var v=s.then(function(sig){ globalThis.__s521=b2hex(new Uint8Array(sig)).length;\
                   return crypto.subtle.verify({name:'ECDSA', hash:'SHA-384'}, kp.publicKey, sig, m); })\
                   .then(function(ok){ globalThis.__v521=String(ok); });\
                 return v; })\
               .then(function(){\
                 var pA=crypto.subtle.generateKey({name:'ECDH', namedCurve:'P-384'}, false, ['deriveBits']);\
                 var pB=crypto.subtle.generateKey({name:'ECDH', namedCurve:'P-384'}, false, ['deriveBits']);\
                 return Promise.all([pA,pB]).then(function(ps){\
                   return Promise.all([\
                     crypto.subtle.deriveBits({name:'ECDH', public:ps[0].publicKey}, ps[1].privateKey),\
                     crypto.subtle.deriveBits({name:'ECDH', public:ps[1].publicKey}, ps[0].privateKey)]); }); })\
               .then(function(bs){ globalThis.__dh384=b2hex(new Uint8Array(bs[0]))===b2hex(new Uint8Array(bs[1])) ? String(bs[0].byteLength) : 'mismatch'; })\
               .then(function(){ return crypto.subtle.generateKey({name:'ECDSA', namedCurve:'P-521'}, true, ['sign']); })\
               .then(function(kp){ return crypto.subtle.exportKey('jwk', kp.privateKey); })\
               .then(function(j){ globalThis.__j521=JSON.stringify({kty:j.kty, crv:j.crv, alg:j.alg,\
                   xLen:b64u2b(j.x).length, yLen:b64u2b(j.y).length, dLen:b64u2b(j.d).length, ext:j.ext}); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__s521)").unwrap().value, "264", "P-521 签名 r||s 定宽 66+66 字节（264 hex）");
    assert_eq!(sandbox.execute("String(globalThis.__v521)").unwrap().value, "true", "P-521 sign/verify round-trip");
    assert_eq!(sandbox.execute("String(globalThis.__dh384)").unwrap().value, "48", "ECDH P-384 缺省 48 字节且双向一致");
    assert_eq!(
        sandbox.execute("String(globalThis.__j521)").unwrap().value,
        r#"{"kty":"EC","crv":"P-521","alg":"ES512","xLen":66,"yLen":66,"dLen":66,"ext":true}"#,
        "P-521 jwk x/y/d 定宽 66 字节 + alg ES512"
    );

    // 导入导出全格式 round-trip：spki/raw/pkcs8 逐字节稳定、SEC1 压缩点导入（raw →
    // 导出 spki 与未压缩导入一致）、jwk 重导入 pkcs8 一致；错误矩阵：spki+sign usage
    // SyntaxError、pkcs8 空 usages SyntaxError、jwk kty/crv DataError、非 extractable
    // 导出 InvalidAccessError、generateKey 缺 namedCurve TypeError、未注册曲线
    // NotSupportedError、非法 usage SyntaxError。
    sandbox
        .execute(
            "globalThis.__rt='(pending)'; globalThis.__crt='(pending)'; globalThis.__jrt='(pending)';\
             globalThis.__spkihex='(pending)'; globalThis.__rawspki='(pending)';\
             globalThis.__f1='(pending)'; globalThis.__f2='(pending)'; globalThis.__f3='(pending)';\
             globalThis.__f4='(pending)'; globalThis.__f5='(pending)'; globalThis.__f6='(pending)';\
             globalThis.__f7='(pending)'; globalThis.__f8='(pending)';\
             crypto.subtle.generateKey({name:'ECDSA', namedCurve:'P-256'}, true, ['sign','verify'])\
               .then(function(kp){\
                 var pk8hex=null;\
                 var r1=crypto.subtle.exportKey('pkcs8', kp.privateKey).then(function(d){ pk8hex=b2hex(new Uint8Array(d));\
                   return crypto.subtle.importKey('pkcs8', d, {name:'ECDSA', namedCurve:'P-256'}, true, ['sign']); })\
                   .then(function(k){ return crypto.subtle.exportKey('pkcs8', k); })\
                   .then(function(d){ globalThis.__rt=b2hex(new Uint8Array(d))===pk8hex ? 'stable' : 'drift'; });\
                 var r2=crypto.subtle.exportKey('spki', kp.publicKey).then(function(d){ globalThis.__spkihex=b2hex(new Uint8Array(d)); });\
                 var r2b=crypto.subtle.exportKey('raw', kp.publicKey)\
                   .then(function(rd){ var u=new Uint8Array(rd);\
                     var c=new Uint8Array(33);\
                     c[0]=(u[64]&1)?3:2; for(var i=0;i<32;i++) c[1+i]=u[1+i];\
                     return crypto.subtle.importKey('raw', rd, {name:'ECDSA', namedCurve:'P-256'}, true, ['verify'])\
                       .then(function(k){ return crypto.subtle.exportKey('spki', k); })\
                       .then(function(d){ globalThis.__rawspki=b2hex(new Uint8Array(d));\
                         return crypto.subtle.importKey('raw', c.buffer, {name:'ECDSA', namedCurve:'P-256'}, true, ['verify']); }); })\
                   .then(function(k){ return crypto.subtle.exportKey('spki', k); })\
                   .then(function(d){ globalThis.__crt=b2hex(new Uint8Array(d))===globalThis.__rawspki ? 'stable' : 'drift'; });\
                 var r3=crypto.subtle.exportKey('jwk', kp.privateKey).then(function(j){\
                   var j2={kty:j.kty, crv:j.crv, x:j.x, y:j.y, d:j.d};\
                   return crypto.subtle.importKey('jwk', j2, {name:'ECDSA', namedCurve:'P-256'}, true, ['sign']); })\
                   .then(function(k){ return crypto.subtle.exportKey('pkcs8', k); })\
                   .then(function(d){ globalThis.__jrt=b2hex(new Uint8Array(d))===pk8hex ? 'stable' : 'drift'; });\
                 var r4=crypto.subtle.exportKey('spki', kp.publicKey).then(function(d){\
                   var f1=errname(crypto.subtle.importKey('spki', d, {name:'ECDSA', namedCurve:'P-256'}, true, ['sign']))\
                     .then(function(n){ globalThis.__f1=n; });\
                   var f2=errname(crypto.subtle.importKey('jwk', {kty:'oct', k:'AA'}, {name:'ECDSA', namedCurve:'P-256'}, true, ['sign']))\
                     .then(function(n){ globalThis.__f3=n; });\
                   var f3=errname(crypto.subtle.importKey('jwk', {kty:'EC', crv:'P-384', x:'AA', y:'BB', d:'CC'}, {name:'ECDSA', namedCurve:'P-256'}, true, ['sign']))\
                     .then(function(n){ globalThis.__f4=n; });\
                   return Promise.all([f1,f2,f3]); });\
                 var r5=crypto.subtle.generateKey({name:'ECDSA', namedCurve:'P-256'}, false, ['sign'])\
                   .then(function(np){ return errname(crypto.subtle.exportKey('pkcs8', np.privateKey)); })\
                   .then(function(n){ globalThis.__f5=n; });\
                 var f5=errname(crypto.subtle.generateKey({name:'ECDSA'}, true, ['sign']))\
                   .then(function(n){ globalThis.__f6=n; });\
                 var f6=errname(crypto.subtle.generateKey({name:'ECDSA', namedCurve:'P-192'}, true, ['sign']))\
                   .then(function(n){ globalThis.__f7=n; });\
                 var f7=errname(crypto.subtle.importKey('pkcs8', hex2b('00').buffer, {name:'ECDSA', namedCurve:'P-256'}, true, ['sign']))\
                   .then(function(n){ globalThis.__f2=n; });\
                 var f8=errname(crypto.subtle.generateKey({name:'ECDSA', namedCurve:'P-256'}, true, ['encrypt']))\
                   .then(function(n){ globalThis.__f8=n; });\
                 return Promise.all([r1,r2b,r3,r4,r5,f5,f6,f7,f8]); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__rt)").unwrap().value, "stable", "pkcs8 导入导出 round-trip 逐字节稳定");
    assert_eq!(
        sandbox.execute("String(globalThis.__rawspki)").unwrap().value,
        sandbox.execute("String(globalThis.__spkihex)").unwrap().value,
        "raw（SEC1）导入 → 导出 spki 与原公钥导出一致"
    );
    assert_eq!(sandbox.execute("String(globalThis.__crt)").unwrap().value, "stable", "SEC1 压缩点导入 → 导出 spki 与未压缩一致");
    assert_eq!(sandbox.execute("String(globalThis.__jrt)").unwrap().value, "stable", "jwk 重导入 → pkcs8 与原导出逐字节一致");
    assert_eq!(sandbox.execute("String(globalThis.__f1)").unwrap().value, "SyntaxError", "spki 公钥 + sign usage → SyntaxError");
    assert_eq!(sandbox.execute("String(globalThis.__f2)").unwrap().value, "DataError", "非法 pkcs8 DER → DataError");
    assert_eq!(sandbox.execute("String(globalThis.__f3)").unwrap().value, "DataError", "jwk kty 'oct' → DataError");
    assert_eq!(sandbox.execute("String(globalThis.__f4)").unwrap().value, "DataError", "jwk crv 与 namedCurve 不匹配 → DataError");
    assert_eq!(sandbox.execute("String(globalThis.__f5)").unwrap().value, "InvalidAccessError", "非 extractable 私钥导出 → InvalidAccessError");
    assert_eq!(sandbox.execute("String(globalThis.__f6)").unwrap().value, "TypeError", "generateKey 缺 namedCurve → TypeError");
    assert_eq!(sandbox.execute("String(globalThis.__f7)").unwrap().value, "NotSupportedError", "generateKey 未注册曲线 → NotSupportedError");
    assert_eq!(sandbox.execute("String(globalThis.__f8)").unwrap().value, "SyntaxError", "generateKey 非法 usage → SyntaxError");
}

// webcrypto M5：crypto.subtle OKP 家族（Ed25519 RFC 8032 纯 EdDSA + X25519 RFC 7748）。
// 向量锚定：WPT eddsa_vectors fixture（pkcs8 seed / spki / 64 字节消息 / 64 字节签名
// 逐字节）、RFC 7748 §6.1（Alice/Bob X25519 KAT，shared K 4a5d9d5b..1742）、小阶点
// 全零拒绝（spec deriveBits all-zero → OperationError）。语义面：generateKey 结构
//（publicKey 恒 extractable / privateKey 按请求值 + usages 类型拆分）、deriveBits
// length 矩阵（256/230/0/null/384）、四格式 round-trip（raw/spki/pkcs8/jwk 含
// alg=Ed25519 与 X25519 无 alg）、jwk d/x 配对校验、错误矩阵。
// https://w3c.github.io/webcrypto/#ed25519  https://w3c.github.io/webcrypto/#x25519
#[test]
fn test_crypto_subtle_okp_m5() {
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
             function b2b64u(u){var s='';for(var i=0;i<u.length;i++)s+=String.fromCharCode(u[i]);return btoa(s).replace(/\\+/g,'-').replace(/\\//g,'_').replace(/=/g,'');}",
        )
        .unwrap();

    // Ed25519 KAT（WPT eddsa_vectors fixture）：pkcs8 导入 → sign 64 字节消息 →
    // 签名逐字节锚定（RFC 8032 纯 EdDSA 确定性）→ spki 导入验签 true / 篡改 false /
    // 小阶 A 验签 false（verify_strict 拒小阶，eprint 2020/1244 面）。
    sandbox
        .execute(
            "globalThis.__okpErr='(none)'; globalThis.__k1='(pending)'; globalThis.__k2='(pending)';\
             globalThis.__k3='(pending)'; globalThis.__k4='(pending)'; globalThis.__gk='(pending)';\
             var MSG=hex2b('2b7ed0bc7795694ab4acd35903fe8cd7d80f6a1c8688a6c3414409457514a1457855bbb219e30a1beea8fe869082d99fc8282f9050d024e59eaf0730ba9db70a');\
             var SIG='3d90de5e5743dfc28225bfadb341b116cbf8a3f1ceedbf4adc350ef5d3471843a418614dcb6e61486261 4cf7af1496f9340b3c844ea4dceab1d3d155eb7ecc00'.replace(/ /g,'');\
             var PKCS8='302e020100300506032b657004220420f3c8f4c48df878146e8cd3bf6df4e50e389ba7074e15c2352dcd5d308d4ca81f';\
             var SPKI='302a300506032b6570032100d8e18963d809d487d9549accaec6742e7eeba24d8a0d3b14b7e3caea06893dcc';\
             var SMALL_A=hex2b('302a300506032b6570032100c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a');\
             crypto.subtle.importKey('pkcs8', hex2b(PKCS8), {name:'Ed25519'}, true, ['sign'])\
               .then(function(sk){ globalThis.__sk=sk;\
                 return crypto.subtle.sign({name:'Ed25519'}, sk, MSG); })\
               .then(function(sig){ globalThis.__k1=b2hex(new Uint8Array(sig))===SIG;\
                 return crypto.subtle.exportKey('jwk', globalThis.__sk); })\
               .then(function(j){ globalThis.__k1j=JSON.stringify({crv:j.crv,kty:j.kty,alg:j.alg,hasD:!!j.d,nKeys:Object.keys(j).length});\
                 return crypto.subtle.importKey('spki', hex2b(SPKI), {name:'Ed25519'}, true, ['verify']); })\
               .then(function(pk){ globalThis.__pk=pk;\
                 var v1=crypto.subtle.verify({name:'Ed25519'}, pk, hex2b(SIG).buffer, MSG).then(function(ok){ return 'v'+ok; });\
                 var v2=crypto.subtle.verify({name:'Ed25519'}, pk, hex2b(SIG).buffer, hex2b('deadbeef')).then(function(ok){ return 't'+ok; });\
                 return Promise.all([v1, v2]); })\
               .then(function(rs){ globalThis.__k2=rs.join(',');\
                 return crypto.subtle.importKey('spki', SMALL_A, {name:'Ed25519'}, true, ['verify']); })\
               .then(function(spk){\
                 return crypto.subtle.verify({name:'Ed25519'}, spk, hex2b(SIG).buffer, MSG).then(function(ok){ globalThis.__k3='small'+ok; }); })\
               .catch(function(e){ globalThis.__okpErr=e.name+': '+e.message; });\
             crypto.subtle.generateKey({name:'Ed25519'}, true, ['sign','verify'])\
               .then(function(kp){ globalThis.__gk=JSON.stringify({\
                   pubEx:kp.publicKey.extractable, privEx:kp.privateKey.extractable,\
                   alg:kp.privateKey.algorithm.name,\
                   pubU:kp.publicKey.usages.join(','), privU:kp.privateKey.usages.join(',')});\
                 var s1=crypto.subtle.sign({name:'Ed25519'}, kp.privateKey, hex2b('deadbeef'));\
                 var s2=crypto.subtle.sign({name:'Ed25519'}, kp.privateKey, hex2b('deadbeef'));\
                 return Promise.all([s1,s2]).then(function(ss){\
                   globalThis.__gdet=(b2hex(new Uint8Array(ss[0]))===b2hex(new Uint8Array(ss[1])))+'|'+(b2hex(new Uint8Array(ss[0])).length===128);\
                   return crypto.subtle.verify({name:'Ed25519'}, kp.publicKey, ss[0], hex2b('deadbeef')); }); })\
               .then(function(ok){ globalThis.__gk += '|v'+ok; })\
               .catch(function(e){ globalThis.__okpErr += ';g='+e.name; });",
        )
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert_eq!(sandbox.execute("String(globalThis.__okpErr)").unwrap().value, "(none)", "Ed25519 KAT 链无异常");
    assert_eq!(sandbox.execute("String(globalThis.__k1)").unwrap().value, "true", "RFC 8032 签名逐字节锚定（WPT eddsa_vectors）");
    assert_eq!(sandbox.execute("String(globalThis.__k1j)").unwrap().value, r#"{"crv":"Ed25519","kty":"OKP","alg":"Ed25519","hasD":true,"nKeys":7}"#, "私钥 jwk 导出（kty/crv/alg/d + key_ops/ext 共 7 成员）");
    assert_eq!(sandbox.execute("String(globalThis.__k2)").unwrap().value, "vtrue,tfalse", "KAT 验签 true / 篡改 false");
    assert_eq!(sandbox.execute("String(globalThis.__k3)").unwrap().value, "smallfalse", "小阶 A 验签 false（verify_strict，WPT eddsa_small_order_points）");
    assert_eq!(sandbox.execute("String(globalThis.__gk)").unwrap().value, r#"{"pubEx":true,"privEx":true,"alg":"Ed25519","pubU":"verify","privU":"sign"}|vtrue"#, "generateKey Ed25519 结构 + 生成对 sign/verify round-trip");
    assert_eq!(sandbox.execute("String(globalThis.__gdet)").unwrap().value, "true|true", "生成对签名确定性 + 64 字节定宽");

    // X25519 RFC 7748 §6.1 KAT：Alice/Bob 双向 deriveBits（256 位全量锚定）+
    // length 矩阵（230 位截断尾字节高位保留、0 空、null 全量、384 OperationError）+
    // 小阶公钥全零 → OperationError + deriveKey→AES-GCM round-trip。
    sandbox
        .execute(
            "globalThis.__xErr='(none)'; globalThis.__x1='(pending)'; globalThis.__x2='(pending)';\
             globalThis.__x3='(pending)'; globalThis.__x4='(pending)'; globalThis.__x5='(pending)'; globalThis.__xk='(pending)';\
             var A_PR='302e020100300506032b656e0422042077076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a';\
             var A_PU='302a300506032b656e0321008520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a';\
             var B_PR='302e020100300506032b656e042204205dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb';\
             var B_PU='302a300506032b656e032100de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f';\
             var K='4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742';\
             var SMALL0=hex2b('302a300506032b656e032100'+'00'.repeat(32));\
             var ap, bp;\
             Promise.all([\
               crypto.subtle.importKey('pkcs8', hex2b(A_PR), {name:'X25519'}, true, ['deriveKey','deriveBits']),\
               crypto.subtle.importKey('pkcs8', hex2b(B_PR), {name:'X25519'}, true, ['deriveKey','deriveBits']),\
               crypto.subtle.importKey('spki', hex2b(A_PU), {name:'X25519'}, true, []),\
               crypto.subtle.importKey('spki', hex2b(B_PU), {name:'X25519'}, true, []),\
               crypto.subtle.importKey('spki', SMALL0, {name:'X25519'}, true, [])\
             ]).then(function(keys){\
               ap=keys[0]; bp=keys[1];\
               var ab=crypto.subtle.deriveBits({name:'X25519', public:keys[3]}, ap, 256).then(function(b){ return b2hex(new Uint8Array(b)); });\
               var ba=crypto.subtle.deriveBits({name:'X25519', public:keys[2]}, bp, null).then(function(b){ return b2hex(new Uint8Array(b)); });\
               var l230=crypto.subtle.deriveBits({name:'X25519', public:keys[3]}, ap, 230).then(function(b){ return '230:'+b2hex(new Uint8Array(b)); });\
               var l0=crypto.subtle.deriveBits({name:'X25519', public:keys[3]}, ap, 0).then(function(b){ return '0:'+new Uint8Array(b).length; });\
               var l384=crypto.subtle.deriveBits({name:'X25519', public:keys[3]}, ap, 384).then(function(){ return '384:no-throw'; }, function(e){ return '384:'+e.name; });\
               var small=crypto.subtle.deriveBits({name:'X25519', public:keys[4]}, ap, 256).then(function(){ return 'small:no-throw'; }, function(e){ return 'small:'+e.name; });\
               var dk=crypto.subtle.deriveKey({name:'X25519', public:keys[3]}, ap, {name:'AES-GCM', length:256}, false, ['encrypt','decrypt'])\
                 .then(function(aes){\
                   return crypto.subtle.encrypt({name:'AES-GCM', iv:hex2b('000102030405060708090a0b')}, aes, hex2b('cafebabe'))\
                     .then(function(ct){ return crypto.subtle.decrypt({name:'AES-GCM', iv:hex2b('000102030405060708090a0b')}, aes, ct); })\
                     .then(function(pt){ return 'dk:'+b2hex(new Uint8Array(pt)); });\
                 }, function(e){ return 'dk:'+e.name; });\
               return Promise.all([ab, ba, l230, l0, l384, small, dk]); })\
             .then(function(rs){ globalThis.__x1=(rs[0]===K)+'|'+(rs[1]===K);\
               globalThis.__x2=rs[2]; globalThis.__x3=rs[3]; globalThis.__x4=rs[4]+'|'+rs[5]; globalThis.__xk=rs[6]; })\
             .catch(function(e){ globalThis.__xErr=e.name+': '+e.message; });",
        )
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert_eq!(sandbox.execute("String(globalThis.__xErr)").unwrap().value, "(none)", "X25519 KAT 链无异常");
    assert_eq!(sandbox.execute("String(globalThis.__x1)").unwrap().value, "true|true", "RFC 7748 §6.1 双向 KAT（shared K 逐字节）");
    assert_eq!(sandbox.execute("String(globalThis.__x2)").unwrap().value, "230:4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1c", "230 位截断（28 整字节 + 尾字节 0x1e&0xFC=0x1C，同 WPT cfrg 截断语义）");
    assert_eq!(sandbox.execute("String(globalThis.__x3)").unwrap().value, "0:0", "length 0 → 空 ArrayBuffer");
    assert_eq!(sandbox.execute("String(globalThis.__x4)").unwrap().value, "384:OperationError|small:OperationError", "384 超输出宽 → OperationError；小阶公钥全零 → OperationError");
    assert_eq!(sandbox.execute("String(globalThis.__xk)").unwrap().value, "dk:cafebabe", "deriveKey→AES-GCM round-trip（WebKit bug 282096 同径）");

    // 四格式 round-trip + jwk 语义（Ed25519 alg ∈ {"Ed25519","EdDSA"}、X25519 忽略
    // alg 且导出无成员）+ 错误矩阵（bad length/kty/crv/alg 错拼/d/x 失配/usages 家族）。
    sandbox
        .execute(
            "globalThis.__tErr='(none)'; globalThis.__t1='(pending)'; globalThis.__t2='(pending)';\
             globalThis.__t3='(pending)'; globalThis.__f1='(pending)'; globalThis.__f2='(pending)';\
             globalThis.__f3='(pending)'; globalThis.__f4='(pending)'; globalThis.__f5='(pending)';\
             var XJ='302a300506032b656e0321001cf2b1e6022ec537371ed7f53e54fa1154d83e98eb64ea51fae5b3307cfe9706';\
             var xraw=null; var xjpk=null;\
             var xpubB=b2b64u(hex2b('1cf2b1e6022ec537371ed7f53e54fa1154d83e98eb64ea51fae5b3307cfe9706'));\
             var xdB=b2b64u(hex2b('c8838e76d057dfb7d8c95a69e138160add6373fd71a4d276bb56e3a81b64ff61'));\
             crypto.subtle.importKey('spki', hex2b(XJ), {name:'X25519'}, true, [])\
               .then(function(k){ xraw=k;\
                 return crypto.subtle.exportKey('raw', k).then(function(r){ return 'raw:'+(b2hex(new Uint8Array(r))==='1cf2b1e6022ec537371ed7f53e54fa1154d83e98eb64ea51fae5b3307cfe9706'); }); })\
               .then(function(r1){ globalThis.__t1=r1;\
                 return crypto.subtle.exportKey('spki', xraw).then(function(d){ return 'spki:'+(b2hex(new Uint8Array(d))===XJ); }); })\
               .then(function(r2){ globalThis.__t1 += '|'+r2;\
                 return crypto.subtle.importKey('jwk', {kty:'OKP', crv:'X25519', x:xpubB, alg:'this is ignored'}, {name:'X25519'}, true, []); })\
               .then(function(jk){ return crypto.subtle.exportKey('jwk', jk).then(function(j){ return 'alg:'+('alg' in j)+'|x:'+(j.x===xpubB)+'|n:'+Object.keys(j).length; }); })\
               .then(function(r3){ globalThis.__t2=r3;\
                 return crypto.subtle.importKey('jwk', {kty:'OKP', crv:'X25519', x:xpubB, d:xdB}, {name:'X25519'}, true, ['deriveBits']); })\
               .then(function(jpk){ xjpk=jpk;\
                 return crypto.subtle.exportKey('pkcs8', jpk).then(function(d){ return 'pk8:'+(b2hex(new Uint8Array(d)).substring(0,32)==='302e020100300506032b656e04220420'); }); })\
               .then(function(r4){ globalThis.__t3=r4;\
                 var ed=hex2b('302e020100300506032b657004220420f3c8f4c48df878146e8cd3bf6df4e50e389ba7074e15c2352dcd5d308d4ca81f');\
                 var bads=[\
                   crypto.subtle.importKey('raw', hex2b('00'.repeat(31)), {name:'Ed25519'}, true, ['verify']),\
                   crypto.subtle.importKey('jwk', {kty:'EC', crv:'X25519', x:xpubB}, {name:'X25519'}, true, []),\
                   crypto.subtle.importKey('jwk', {kty:'OKP', crv:'Ed448', x:xpubB}, {name:'X25519'}, true, []),\
                   crypto.subtle.importKey('jwk', {kty:'OKP', crv:'Ed25519', x:'2OGJY9gJ1IfZVJrMrsZ0Ln7rok2KDTsUt-PK6gaJPcw', alg:'ed25519'}, {name:'Ed25519'}, true, ['verify']),\
                   crypto.subtle.importKey('jwk', {kty:'OKP', crv:'Ed25519', x:'11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo', d:'88j0xI34eBRujNO_bfTlDjibpwdOFcI1Lc1dMI1MqB8'}, {name:'Ed25519'}, true, ['sign']),\
                   crypto.subtle.importKey('raw', hex2b('00'.repeat(32)), {name:'Ed25519'}, true, ['deriveBits'])\
                 ];\
                 return Promise.all(bads.map(function(p){ return errname(p); })); })\
               .then(function(names){ globalThis.__f1=names.join(',');\
                 return crypto.subtle.exportKey('jwk', xjpk); })\
               .then(function(j){ globalThis.__f2=JSON.stringify({alg:('alg' in j), d:(j.d===xdB), n:Object.keys(j).length}); })\
               .catch(function(e){ globalThis.__tErr=e.name+': '+e.message; });",
        )
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert_eq!(sandbox.execute("String(globalThis.__tErr)").unwrap().value, "(none)", "round-trip/错误矩阵链无异常");
    assert_eq!(sandbox.execute("String(globalThis.__t1)").unwrap().value, "raw:true|spki:true", "X25519 raw/spki round-trip 逐字节稳定");
    assert_eq!(sandbox.execute("String(globalThis.__t2)").unwrap().value, "alg:false|x:true|n:5", "X25519 jwk 导入忽略 alg、导出无 alg 成员（kty/crv/x/key_ops/ext 共 5）");
    assert_eq!(sandbox.execute("String(globalThis.__t3)").unwrap().value, "pk8:true", "X25519 私钥 jwk→pkcs8 导出（RFC 8410 前缀）");
    assert_eq!(sandbox.execute("String(globalThis.__f1)").unwrap().value, "DataError,DataError,DataError,DataError,DataError,SyntaxError", "错误矩阵：31 字节 raw / kty='EC' / crv 失配 / alg 错拼 / d-x 失配（DataError）+ 非法 usages（SyntaxError）");
    assert_eq!(sandbox.execute("String(globalThis.__f2)").unwrap().value, r#"{"alg":false,"d":true,"n":6}"#, "X25519 私钥 jwk 导出无 alg + d 定宽（共 6 成员）");
}
