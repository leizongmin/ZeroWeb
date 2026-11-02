  // ===== WebCrypto 门禁面（三期 M2-1 切片，goal webcrypto-compat-v3）：SubtleCrypto
  // 接口对象 + 静态 supports(operation, algorithm, lengthOrAdditionalAlgorithm)。
  // getPublicKey 实例方法在 part02.js _zwSubtle 字面量内。规范：
  // https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-supports
  // 挂载位置（part01c/01d 先例）：part01b/part02 边界在 _zwSubtle 对象字面量内部、
  // part02 尾在语句中间（R2963 字节精确切分残留），均无干净语句边界可插，故挂
  // part06 尾。**本文件自包含**：shim 三段 IIFE 结构下 crypto helper（_zw_upperAscii/
  // _zw_bufToBytes/CryptoKey 等）所在内层闭包（part05:3818 收口）不覆盖本挂载点
  //（part01c 同况——_getSelection 声明域不覆盖其挂载点），故引用面仅限 globalThis
  // 契约：isSecureContext、globalThis.crypto.subtle（part02 [SecureContext] 门禁
  // 已挂）、globalThis.CryptoKey（同门禁挂载）；upperAscii/bufBytes 本地镜像
  //（ASCII 大写映射与 BufferSource 字节提取语义同 part01b 源）。isSecureContext
  // !== true 不挂载（一期 M6 [SecureContext] 门禁语义不回退）。
  (function () {
    if (globalThis.isSecureContext !== true) return;
    if (globalThis.crypto == null || globalThis.crypto.subtle == null) return;

    // 合法 operation 白名单（大小写敏感，spec supports 步 1——'GENERATEKEY'/''/
    // 未知名一律 false）。
    var SUPPORT_OPS = {
      encrypt: 1, decrypt: 1, sign: 1, verify: 1, digest: 1,
      generateKey: 1, deriveKey: 1, deriveBits: 1, importKey: 1, exportKey: 1,
      wrapKey: 1, unwrapKey: 1, getPublicKey: 1,
      encapsulateKey: 1, encapsulateBits: 1, decapsulateKey: 1, decapsulateBits: 1
    };

    // SHA 摘要输出位长 / EC 曲线派生域宽（ECDH deriveBits max = 曲线字节数×8——
    // P-521 521 bit 域取整字节 528，与 Chromium 一致）。
    var SHA_BITS = { 'SHA-1': 160, 'SHA-256': 256, 'SHA-384': 384, 'SHA-512': 512 };
    // SHA 块大小——HMAC generateKey 缺省密钥长度 = hash 块大小（spec get key
    // length of the generated key：无 length 成员取 block size，非摘要输出位长；
    // SHA-1/256 → 512 bit、SHA-384/512 → 1024 bit，shim generateKey 注释同源）。
    var SHA_BLOCK_BITS = { 'SHA-1': 512, 'SHA-256': 512, 'SHA-384': 1024, 'SHA-512': 1024 };
    var EC_CURVE_MAX_BITS = { 'P-256': 256, 'P-384': 384, 'P-521': 528 };

    // AES-GCM tagLength 合法集（spec：32/64/72/80/96/104/112/120/128）。
    var GCM_TAG_LENGTHS = { 32: 1, 64: 1, 72: 1, 80: 1, 96: 1, 104: 1, 112: 1, 120: 1, 128: 1 };

    // 注册面 = 本实现既有算法（shim part01b/02）× op 真实支持矩阵。supports 语义
    // =「归一成功 + op 注册 + 参数面校验」——**只登记真实现面，不虚报未实现域**：
    // ML-KEM/ML-DSA/X448/Ed448/KMAC/Hybrid 未注册 → 全 op
    // false（supports-modern matrix 期望 true，待三期按域切片逐行扩表）。
    // pub=1：算法支持私→公派生（getPublicKey 域，非对称族）。
    // wrapKey/unwrapKey 直注册仅 AES-KW（RFC 3394 原生封装）；AES-CBC/CTR/GCM、
    // RSA-OAEP 按 spec 步 5 经 encrypt/decrypt fallback（归一后参数校验照走）。
    var REG = {
      'SHA-1':   { ops: { digest: 1 } },
      'SHA-256': { ops: { digest: 1 } },
      'SHA-384': { ops: { digest: 1 } },
      'SHA-512': { ops: { digest: 1 } },
      'HMAC':    { ops: { sign: 1, verify: 1, generateKey: 1, importKey: 1, exportKey: 1 } },
      'AES-CBC': { ops: { encrypt: 1, decrypt: 1, generateKey: 1, importKey: 1, exportKey: 1 } },
      'AES-CTR': { ops: { encrypt: 1, decrypt: 1, generateKey: 1, importKey: 1, exportKey: 1 } },
      'AES-GCM': { ops: { encrypt: 1, decrypt: 1, generateKey: 1, importKey: 1, exportKey: 1 } },
      'AES-KW':  { ops: { generateKey: 1, importKey: 1, exportKey: 1, wrapKey: 1, unwrapKey: 1 } },
      // REG 键一律用归一后形态（upperAscii 只映射 a-z——v1_5 → V1_5、Ed25519 →
      // ED25519，algNameOf 查表键不匹配原始拼法）。
      'RSASSA-PKCS1-V1_5': { ops: { sign: 1, verify: 1, generateKey: 1, importKey: 1, exportKey: 1 }, pub: 1 },
      'RSA-PSS':  { ops: { sign: 1, verify: 1, generateKey: 1, importKey: 1, exportKey: 1 }, pub: 1 },
      'RSA-OAEP': { ops: { encrypt: 1, decrypt: 1, generateKey: 1, importKey: 1, exportKey: 1 }, pub: 1 },
      'ECDSA':  { ops: { sign: 1, verify: 1, generateKey: 1, importKey: 1, exportKey: 1 }, pub: 1 },
      'ECDH':   { ops: { deriveBits: 1, deriveKey: 1, generateKey: 1, importKey: 1, exportKey: 1 }, pub: 1 },
      'ED25519': { ops: { sign: 1, verify: 1, generateKey: 1, importKey: 1, exportKey: 1 }, pub: 1 },
      'X25519':  { ops: { deriveBits: 1, deriveKey: 1, generateKey: 1, importKey: 1, exportKey: 1 }, pub: 1 },
      'HKDF':    { ops: { importKey: 1, deriveBits: 1, deriveKey: 1 } },
      'PBKDF2':  { ops: { importKey: 1, deriveBits: 1 } },
      // webcrypto-compat-v3 域切片（D1 拍板 2026-11-02）：AES-OCB / ChaCha20-Poly1305
      // 全加密面注册（wrapKey/unwrapKey 与 AES-CBC/CTR/GCM 同构走 fallback，spec 步 5）；
      // Argon2d/i/id 派生面（raw-secret import + deriveBits）。
      'AES-OCB': { ops: { encrypt: 1, decrypt: 1, generateKey: 1, importKey: 1, exportKey: 1 } },
      'CHACHA20-POLY1305': { ops: { encrypt: 1, decrypt: 1, generateKey: 1, importKey: 1, exportKey: 1 } },
      'ARGON2D':  { ops: { importKey: 1, deriveBits: 1 } },
      'ARGON2I':  { ops: { importKey: 1, deriveBits: 1 } },
      'ARGON2ID': { ops: { importKey: 1, deriveBits: 1 } },
      // webcrypto-compat-v3 域切片（D1 拍板 2026-11-02）XOF 摘要面：SHA-3 定长家族
      // （FIPS 202）+ SHAKE/cSHAKE（SP 800-185）+ KT/TurboSHAKE（RFC 9861）——digest
      // 唯一 op（host `__zw_crypto_subtle_xof_digest`，hex wire）。
      'SHA3-224': { ops: { digest: 1 } },
      'SHA3-256': { ops: { digest: 1 } },
      'SHA3-384': { ops: { digest: 1 } },
      'SHA3-512': { ops: { digest: 1 } },
      'SHAKE128': { ops: { digest: 1 } },
      'SHAKE256': { ops: { digest: 1 } },
      'CSHAKE128': { ops: { digest: 1 } },
      'CSHAKE256': { ops: { digest: 1 } },
      'KT128': { ops: { digest: 1 } },
      'KT256': { ops: { digest: 1 } },
      'TURBOSHAKE128': { ops: { digest: 1 } },
      'TURBOSHAKE256': { ops: { digest: 1 } }
    };

    // ASCII 大写映射（a-z → A-Z，非 ASCII 不动）——part01b _zw_upperAscii 同语义
    //（WebCrypto normalize 名归一）。
    function upperAscii(s) {
      var out = '';
      for (var i = 0; i < s.length; i++) {
        var c = s.charCodeAt(i);
        out += (c >= 97 && c <= 122) ? String.fromCharCode(c - 32) : s.charAt(i);
      }
      return out;
    }

    // BufferSource → 字节数组（ArrayBuffer / TypedArray / DataView / array-like）；
    // detached/异常 → null（invalid BufferSource 由 isBufferSource 前置拦截）。
    function bufBytes(v) {
      var view;
      try {
        if (v instanceof ArrayBuffer) view = new Uint8Array(v);
        else if (v && v.buffer) view = new Uint8Array(v.buffer, v.byteOffset || 0,
          v.byteLength != null ? v.byteLength : v.length);
        else if (typeof v.length === 'number') view = v;
        else return null;
      } catch (_detached) {
        return null;
      }
      var out = [];
      for (var i = 0; i < view.length; i++) out.push(view[i] & 0xff);
      return out;
    }

    function isBufferSource(v) {
      return !!(v && typeof v === 'object' && typeof v.byteLength === 'number');
    }

    // AlgorithmIdentifier → 大写规范名；非法 → null。
    function algNameOf(alg) {
      var n = (typeof alg === 'string') ? alg
        : (alg && typeof alg === 'object' && typeof alg.name === 'string' ? alg.name : null);
      return n ? upperAscii(n) : null;
    }

    // hash 成员（string 或 {name} 字典）→ 规范名（缺失/非法 SHA 名 → null）。
    function hashNameOf(v) {
      var n = (typeof v === 'string') ? v
        : (v && typeof v === 'object' && typeof v.name === 'string' ? v.name : null);
      if (!n) return null;
      var u = upperAscii(n);
      return SHA_BITS[u] ? u : null;
    }

    function paramsOf(alg) {
      return (alg && typeof alg === 'object') ? alg : null;
    }

    // spec check-support-for-an-algorithm：encap/decap op 名映射 → 注册面 → 参数校验。
    function checkSupport(operation, alg, length) {
      // 步 1：encapsulateKey/Bits → encapsulate、decapsulateKey/Bits → decapsulate
      //（KEM 域未注册 → false；encapsulateBits/decapsulateBits 无 length 语义）。
      var op = operation;
      if (op === 'encapsulateKey' || op === 'encapsulateBits') op = 'encapsulate';
      else if (op === 'decapsulateKey' || op === 'decapsulateBits') op = 'decapsulate';

      var name = algNameOf(alg);
      var entry = name ? REG[name] : null;
      if (!entry) return false;

      // getPublicKey：normalize(exportKey) 成功 + 算法支持私→公派生（非对称族；
      // 对称/HMAC/KDF/摘要域 pub 缺位 → false，exportKey 未注册 → false）。
      if (op === 'getPublicKey') return !!entry.pub && !!entry.ops.exportKey;

      var op2 = op;
      if (!entry.ops[op2]) {
        // 步 5 fallback：wrapKey → encrypt、unwrapKey → decrypt 重归一（AES-GCM/
        // CBC/CTR、RSA-OAEP 走此径；HKDF/PBKDF2/SHA 无 encrypt/decrypt → false）。
        if (op2 === 'wrapKey' && entry.ops.encrypt) op2 = 'encrypt';
        else if (op2 === 'unwrapKey' && entry.ops.decrypt) op2 = 'decrypt';
        else return false;
      }
      return validateParams(op2, name, entry, paramsOf(alg), length);
    }

    function validateParams(op, name, entry, params, length) {
      switch (op) {
        case 'digest':
        case 'exportKey':
          // digest 无参数面；exportKey 注册即答案（HKDF/PBKDF2/SHA 无此 op）。
          return true;
        case 'sign':
        case 'verify':
          return validateSignVerify(name, params);
        case 'encrypt':
        case 'decrypt':
          return validateSeal(name, params);
        case 'generateKey':
          return validateGenerateKey(name, params);
        case 'importKey':
          return validateImportKey(name, params);
        case 'deriveBits':
        case 'deriveKey':
          return validateDerive(name, params, length);
        case 'wrapKey':
        case 'unwrapKey':
          // 仅 AES-KW 直注册到此（无参数面）；AES-GCM/CBC/CTR、RSA-OAEP 已在
          // fallback 后按 encrypt/decrypt 校验，不落此分支。
          return true;
      }
      return false;
    }

    function validateSignVerify(name, params) {
      if (!params) return true;
      // RSA-PSS：saltLength 若存在须为有限数（sign 步成员类型校验）。
      if (name === 'RSA-PSS' && params.saltLength != null &&
          (typeof params.saltLength !== 'number' || !isFinite(params.saltLength))) return false;
      // hash 若存在须合法 SHA（sign 归一不强制 hash 存在——supports('sign',
      // {name:'ECDSA'}) 无 hash 期望 true，WPT 断言面）。
      if (params.hash != null && !hashNameOf(params.hash)) return false;
      return true;
    }

    function validateSeal(name, params) {
      // RSA-OAEP：label 可选 BufferSource；裸名（无参）归一即成功。
      if (name === 'RSA-OAEP') {
        return !params || params.label == null || isBufferSource(params.label);
      }
      // AES-CBC：iv 必需且恒 16 字节。
      if (name === 'AES-CBC') {
        return isBufferSource(params && params.iv) && params.iv.byteLength === 16;
      }
      // AES-CTR：counter 16 字节 + length ∈ [1,128] 整数（均必需成员）。
      if (name === 'AES-CTR') {
        var len = params ? params.length : null;
        return isBufferSource(params && params.counter) && params.counter.byteLength === 16 &&
          typeof len === 'number' && isFinite(len) && len >= 1 && len <= 128 &&
          len === Math.floor(len);
      }
      // AES-GCM：iv 恒 12 字节；tagLength 若存在 ∈ 合法集（缺省 128）。
      if (name === 'AES-GCM') {
        if (!isBufferSource(params && params.iv) || params.iv.byteLength !== 12) return false;
        if (params.tagLength != null &&
            (typeof params.tagLength !== 'number' || !GCM_TAG_LENGTHS[params.tagLength])) return false;
        return true;
      }
      // AES-OCB（webcrypto-compat-v3 域切片）：iv 6..=15 字节（spec 拦 >15，短 nonce
      // 面按实现密封集收敛）；tagLength 若存在 ∈ {64,96,128}（缺省 128）。
      if (name === 'AES-OCB') {
        if (!isBufferSource(params && params.iv) ||
            params.iv.byteLength < 6 || params.iv.byteLength > 15) return false;
        if (params.tagLength != null &&
            (typeof params.tagLength !== 'number' ||
             (params.tagLength !== 64 && params.tagLength !== 96 && params.tagLength !== 128))) return false;
        return true;
      }
      // ChaCha20-Poly1305（webcrypto-compat-v3 域切片）：iv 恒 12 字节（spec ≠12 →
      // OperationError）；tagLength 若存在须 =128（缺省合法）。
      if (name === 'CHACHA20-POLY1305') {
        if (!isBufferSource(params && params.iv) || params.iv.byteLength !== 12) return false;
        if (params.tagLength != null &&
            (typeof params.tagLength !== 'number' || params.tagLength !== 128)) return false;
        return true;
      }
      return false;
    }

    function validateGenerateKey(name, params) {
      // AES 家族：length 可选（缺省 256），存在须 ∈ {128,192,256}。
      if (name.indexOf('AES-') === 0) {
        if (params && params.length != null) {
          var l = params.length;
          return l === 128 || l === 192 || l === 256;
        }
        return true;
      }
      // HMAC：hash 必需合法；length 可选，存在须 >0 且 8 的倍数（0/负/非字节对齐
      // → OperationError 域，supports 即 false）。
      if (name === 'HMAC') {
        if (!hashNameOf(params && params.hash)) return false;
        if (params.length != null) {
          var hl = params.length;
          if (typeof hl !== 'number' || !(hl > 0) || hl % 8 !== 0) return false;
        }
        return true;
      }
      // RSA 家族：hash 必需合法；modulusLength 必需 8 的倍数；publicExponent 必需
      // 奇数、≥3、位长 < modulusLength（Chromium rsa_key_generation 同语义——
      // WPT 四非法组：modulus 3、e=1、e=4（偶）、e=2^modulusBits−1）。
      if (name === 'RSASSA-PKCS1-V1_5' || name === 'RSA-PSS' || name === 'RSA-OAEP') {
        if (!hashNameOf(params && params.hash)) return false;
        var ml = params ? params.modulusLength : null;
        if (typeof ml !== 'number' || !(ml > 0) || ml % 8 !== 0 || ml !== Math.floor(ml)) return false;
        return rsaExponentOk(params && params.publicExponent, ml);
      }
      // EC 家族：namedCurve 必需且已注册（'P-51' → false）。
      if (name === 'ECDSA' || name === 'ECDH') {
        var c = params ? params.namedCurve : null;
        return typeof c === 'string' && !!EC_CURVE_MAX_BITS[upperAscii(c)];
      }
      // OKP（Ed25519/X25519）：无参数面。
      return true;
    }

    // publicExponent：BufferSource 大端整数。去前导零后：奇数、≥3（位长 ≥2）、
    // 位长 < modulusBits（2^modulusBits−1 位长恰等 → 拒绝）。
    function rsaExponentOk(v, modulusBits) {
      if (!isBufferSource(v)) return false;
      var bytes = bufBytes(v);
      if (!bytes) return false;
      var i = 0;
      while (i < bytes.length && bytes[i] === 0) i++;
      if (i >= bytes.length) return false; // 全零 → 0 < 3
      if (bytes[bytes.length - 1] % 2 === 0) return false; // 偶数
      var bitLen = (bytes.length - i - 1) * 8;
      var first = bytes[i];
      for (var b = 7; b >= 0; b--) {
        if (first & (1 << b)) { bitLen += b + 1; break; }
      }
      return bitLen >= 2 && bitLen < modulusBits;
    }

    function validateImportKey(name, params) {
      // HMAC：hash 必需合法；length 可选，存在须 >0 且 8 的倍数（0 → DataError）。
      if (name === 'HMAC') {
        if (!hashNameOf(params && params.hash)) return false;
        if (params.length != null) {
          var l = params.length;
          if (typeof l !== 'number' || !(l > 0) || l % 8 !== 0) return false;
        }
        return true;
      }
      // EC 家族：namedCurve 必需且已注册（'not-a-curve' → false）。
      if (name === 'ECDSA' || name === 'ECDH') {
        var c = params ? params.namedCurve : null;
        return typeof c === 'string' && !!EC_CURVE_MAX_BITS[upperAscii(c)];
      }
      // RSA 家族：hash 必需合法。
      if (name === 'RSASSA-PKCS1-V1_5' || name === 'RSA-PSS' || name === 'RSA-OAEP') {
        return !!hashNameOf(params && params.hash);
      }
      // AES/HKDF/PBKDF2/OKP：raw/jwk 导入无参数约束面（HKDF/PBKDF2 裸名 → true，
      // WPT matrix 期望）。
      return true;
    }

    function validateDerive(name, params, length) {
      // ECDH/X25519：public 成员存在时校验 key 身份（本实现 CryptoKey、type
      // public、同算法名、ECDH 曲线已注册——'ECDH rejects a private public
      // property' / 'public property for another algorithm' 断言面）；域宽取
      // public key 曲线（P-256→256/P-384→384/P-521→528；X25519 恒 256）。裸名/
      // 无 public → 归一即过（public 成员完备性属运算期——WPT matrix 传裸名
      // 期望 true）。length 若存在：>0、8 的倍数、≤ 域宽。
      if (name === 'ECDH' || name === 'X25519') {
        var maxBits = (name === 'X25519') ? 256 : null;
        var pub = params ? params.public : null;
        if (pub != null) {
          var CK = globalThis.CryptoKey;
          if (!(pub instanceof CK) || pub.type !== 'public') return false;
          if (!pub.algorithm || algNameOf(pub.algorithm) !== name) return false;
          if (name === 'ECDH') {
            var cv = pub.algorithm.namedCurve;
            if (typeof cv !== 'string' || !EC_CURVE_MAX_BITS[upperAscii(cv)]) return false;
            maxBits = EC_CURVE_MAX_BITS[upperAscii(cv)];
          }
        }
        if (length != null) {
          if (typeof length !== 'number' || !(length > 0) || length % 8 !== 0) return false;
          if (maxBits != null && length > maxBits) return false;
        }
        return true;
      }
      // HKDF/PBKDF2：hash 必需合法；length 必需（null/未传 → false）；>0、8 的
      // 倍数；HKDF ≤ 255×摘要输出位长（spec：输出 ≤ 255×HashLen 字节——SHA-256
      // 上限 65280 bit，WPT max/+8 断言面；PBKDF2 无上限）。
      if (name === 'HKDF' || name === 'PBKDF2') {
        var h = hashNameOf(params && params.hash);
        if (!h) return false;
        if (typeof length !== 'number' || !(length > 0) || length % 8 !== 0) return false;
        if (name === 'HKDF' && length > 255 * SHA_BITS[h]) return false;
        return true;
      }
      // Argon2d/i/id（webcrypto-compat-v3 域切片）：length >0、8 倍数且 ≥32 位
      //（spec：length null/<32/非 8 倍数 → OperationError 域）；Argon2Params required
      // 成员 nonce/parallelism/memory/passes 存在（version/secretValue/associatedData
      // 可选——required 完备性属 supports 静态面，运算期数值约束归 deriveBits）。
      if (name === 'ARGON2D' || name === 'ARGON2I' || name === 'ARGON2ID') {
        if (typeof length !== 'number' || !(length >= 32) || length % 8 !== 0) return false;
        return isBufferSource(params && params.nonce) &&
          typeof (params && params.parallelism) === 'number' &&
          typeof (params && params.memory) === 'number' &&
          typeof (params && params.passes) === 'number';
      }
      return false;
    }

    // spec get key length of the generated key：HMAC = length 成员（>0 且字节对齐）
    // 或 hash 块大小（512/1024，非摘要输出位长——ECDH P-256+HMAC-SHA-256 deriveKey
    // 即因 512 > 256 域宽 false，WPT 断言面）；AES = length 成员（∈{128,192,256}）
    // 或缺省 256；其余域无生成面 → null（deriveKey 组合即 false）。
    function keyLengthOfAdditional(alg) {
      var name = algNameOf(alg);
      var entry = name ? REG[name] : null;
      if (!entry || !entry.ops.generateKey) return null;
      var params = paramsOf(alg);
      if (name === 'HMAC') {
        var h = hashNameOf(params && params.hash);
        if (!h) return null;
        if (params && params.length != null) {
          var l = params.length;
          if (typeof l !== 'number' || !(l > 0) || l % 8 !== 0) return null;
          return l;
        }
        return SHA_BLOCK_BITS[h];
      }
      if (name.indexOf('AES-') === 0) {
        if (params && params.length != null) {
          var al = params.length;
          return (al === 128 || al === 192 || al === 256) ? al : null;
        }
        return 256;
      }
      return null;
    }

    function supports(operation, algorithm, lengthOrAdditionalAlgorithm) {
      // 步 1：operation 白名单（大小写敏感）。
      if (typeof operation !== 'string' ||
          !Object.prototype.hasOwnProperty.call(SUPPORT_OPS, operation)) return false;
      // 步 3-4：第三参分流——number → length；string/对象 → additionalAlgorithm；
      // undefined → 均缺省 null；其余（null/布尔）→ false。
      var length = null;
      var additional = null;
      if (lengthOrAdditionalAlgorithm !== undefined) {
        if (typeof lengthOrAdditionalAlgorithm === 'number') {
          length = lengthOrAdditionalAlgorithm;
        } else if (typeof lengthOrAdditionalAlgorithm === 'string' ||
                   (lengthOrAdditionalAlgorithm !== null &&
                    typeof lengthOrAdditionalAlgorithm === 'object')) {
          additional = lengthOrAdditionalAlgorithm;
        } else {
          return false;
        }
      }
      // 步 5：additionalAlgorithm 组合语义——deriveKey/unwrapKey/encapsulateKey/
      // decapsulateKey 查附加算法 importKey 面、wrapKey 查 exportKey 面；其余 op
      // 带第三参 → false。deriveKey 以附加算法生成密钥长度转 deriveBits（步 5.4）。
      if (additional !== null) {
        if (operation === 'deriveKey' || operation === 'unwrapKey' ||
            operation === 'encapsulateKey' || operation === 'decapsulateKey') {
          if (!checkSupport('importKey', additional, null)) return false;
        } else if (operation === 'wrapKey') {
          if (!checkSupport('exportKey', additional, null)) return false;
        } else {
          return false;
        }
        if (operation === 'deriveKey') {
          var derived = keyLengthOfAdditional(additional);
          if (derived == null) return false;
          length = derived;
          operation = 'deriveBits';
        }
      }
      return checkSupport(operation, algorithm, length);
    }

    // 接口对象（WebIDL interface 无 [Constructor] → Illegal constructor）；supports
    // 仅静态面（实例无 supports——Chromium 同）。命名函数表达式保 name 属性。
    function SubtleCrypto() {
      throw new TypeError('Illegal constructor');
    }
    SubtleCrypto.supports = supports;
    globalThis.SubtleCrypto = globalThis.SubtleCrypto || SubtleCrypto;
  })();
