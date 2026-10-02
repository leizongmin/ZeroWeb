          reject(new DOMException("Unsupported hash algorithm: '" + a + "'", 'NotSupportedError'));
          return;
        }
        var parts = out.split(',');
        var arr = new Uint8Array(parts.length);
        for (var i = 0; i < parts.length; i++) arr[i] = +parts[i];
        resolve(arr);
      });
    },
    // importKey(format, keyData, algorithm, extractable, usages) → Promise<CryptoKey>。HMAC：format 须 "raw"
    //（jwk/pkcs8/spki defer）；algorithm {name:"HMAC",hash:"SHA-XXX"}，usages ⊆ {sign,verify}。PBKDF2：
    // {name:"PBKDF2"}，usages ⊆ {deriveBits,deriveKey}。AES-GCM：{name:"AES-GCM"}，usages ⊆ {encrypt,decrypt}，
    // key 须 128/256 位（16/32 字节）。含非法 usage → SyntaxError，bad key 长度 → DataError。
    // https://w3c.github.io/webcrypto/#SubtleCrypto-method-importKey
    importKey: function (format, keyData, algorithm, extractable, usages) {
      return new Promise(function (resolve, reject) {
        var algo = _zw_normalizeImportAlgorithm(algorithm);
        if (!algo) {
          reject(new DOMException('Unsupported or missing algorithm', 'NotSupportedError')); return;
        }
        var fmt = String(format == null ? '' : format).toUpperCase();
        if (fmt !== 'RAW') {
          reject(new DOMException("Unsupported importKey format: '" + fmt + "' (only 'raw' supported)", 'NotSupportedError')); return;
        }
        var raw = _zw_bufToBytes(keyData);
        // AES-GCM 密钥长度校验（spec：128/256 位；192 位本实现不支持）。
        if (algo.name === 'AES-GCM' && raw.length !== 16 && raw.length !== 32) {
          reject(new DOMException('AES-GCM key must be 128 or 256 bits (16/32 bytes)', 'DataError')); return;
        }
        var allowedUsages = (algo.name === 'PBKDF2' || algo.name === 'HKDF') ? ['deriveBits', 'deriveKey']
          : algo.name === 'AES-GCM' ? ['encrypt', 'decrypt']
          : ['sign', 'verify'];
        var u = _zw_normalizeUsages(usages, allowedUsages);
        if (!u) {
          reject(new DOMException("Invalid key usages for " + algo.name, 'SyntaxError')); return;
        }
        resolve(new CryptoKey('secret', extractable, algo, u, raw));
      });
    },
    // sign(algorithm, key, data) → Promise<ArrayBuffer>。HMAC：algorithm "HMAC"/{name:"HMAC"}，hash 取自 key.algorithm.hash。
    // key.usages 须含 "sign" → 否则 InvalidAccessError。https://w3c.github.io/webcrypto/#SubtleCrypto-method-sign
    sign: function (algorithm, key, data) {
      return new Promise(function (resolve, reject) {
        var name = (typeof algorithm === 'object' && algorithm) ? algorithm.name : algorithm;
        name = String(name == null ? '' : name).toUpperCase();
        if (name !== 'HMAC' || !key || key.algorithm.name !== 'HMAC') {
          reject(new DOMException('Unsupported sign algorithm or key', 'NotSupportedError')); return;
        }
        if (!key.usages || key.usages.indexOf('sign') < 0) {
          reject(new DOMException('Key usages do not include "sign"', 'InvalidAccessError')); return;
        }
        var mac = _zw_hmacMac(key.algorithm, key, _zw_bufToBytes(data), reject);
        if (mac) resolve(mac);
      });
    },
    // verify(algorithm, key, signature, data) → Promise<boolean>。计算 MAC 后定长比较（无早退，常时近似）。
    // https://w3c.github.io/webcrypto/#SubtleCrypto-method-verify
    verify: function (algorithm, key, signature, data) {
      return new Promise(function (resolve, reject) {
        var name = (typeof algorithm === 'object' && algorithm) ? algorithm.name : algorithm;
        name = String(name == null ? '' : name).toUpperCase();
        if (name !== 'HMAC' || !key || key.algorithm.name !== 'HMAC') {
          reject(new DOMException('Unsupported verify algorithm or key', 'NotSupportedError')); return;
        }
        if (!key.usages || key.usages.indexOf('verify') < 0) {
          reject(new DOMException('Key usages do not include "verify"', 'InvalidAccessError')); return;
        }
        var mac = _zw_hmacMac(key.algorithm, key, _zw_bufToBytes(data), reject);
        if (!mac) return;
        var sig = _zw_bufToBytes(signature);
        if (mac.length !== sig.length) { resolve(false); return; }
        var ok = 1;
        for (var i = 0; i < mac.length; i++) {
          if ((mac[i] & 0xff) !== (sig[i] & 0xff)) ok = 0;
        }
        resolve(!!ok);
      });
    },
    // deriveBits(algorithm, key, length) → Promise<ArrayBuffer>。length 为**位数**（须正 8 倍数）；key.usages 须含 "deriveBits"。
    // PBKDF2：algorithm {name:"PBKDF2", salt, iterations, hash}，key = importKey("raw", password, {name:"PBKDF2"})。
    //   host `__zw_crypto_subtle_pbkdf2(hash, keyCsv, saltCsv, iterations, dkLen)`。
    // HKDF（RFC 5869）：algorithm {name:"HKDF", salt?, info?, hash}，key = importKey("raw", ikm, {name:"HKDF"})。
    //   host `__zw_crypto_subtle_hkdf(hash, keyCsv, saltCsv, infoCsv, dkLen)`（空 salt → host 填 HashLen 零）。
    // https://w3c.github.io/webcrypto/#SubtleCrypto-method-deriveBits  https://datatracker.ietf.org/doc/html/rfc5869
    deriveBits: function (algorithm, key, length) {
      return new Promise(function (resolve, reject) {
        var name = (typeof algorithm === 'object' && algorithm) ? algorithm.name : algorithm;
        name = String(name == null ? '' : name).toUpperCase();
        if ((name !== 'PBKDF2' && name !== 'HKDF') || !key || key.algorithm.name !== name) {
          reject(new DOMException('Unsupported deriveBits algorithm or key', 'NotSupportedError')); return;
        }
        if (!key.usages || key.usages.indexOf('deriveBits') < 0) {
          reject(new DOMException('Key usages do not include "deriveBits"', 'InvalidAccessError')); return;
        }
        if (typeof length !== 'number' || length <= 0 || length % 8 !== 0) {
          reject(new DOMException('deriveBits length must be a positive multiple of 8', 'OperationError')); return;
        }
        _zw_performDerive(algorithm, key, length).then(resolve, reject);
      });
    },
    // deriveKey(algorithm, baseKey, derivedKeyAlgo, extractable, usages) → Promise<CryptoKey>。
    // 按 derivedKeyAlgo 决定派生长度（AES→256，HMAC→块大小），deriveBits 后 importKey（baseKey.usages 须含 "deriveKey"）。
    // https://w3c.github.io/webcrypto/#SubtleCrypto-method-deriveKey
    deriveKey: function (algorithm, baseKey, derivedKeyAlgo, extractable, usages) {
      return new Promise(function (resolve, reject) {
        var dka = _zw_normalizeImportAlgorithm(derivedKeyAlgo);
        if (!dka) {
          reject(new DOMException('Unsupported derived key algorithm', 'NotSupportedError')); return;
        }
        var lenBits = _zw_keyLengthBits(dka);
        if (!lenBits) {
          reject(new DOMException('Cannot determine derived key length', 'NotSupportedError')); return;
        }
        if (!baseKey || !baseKey.usages || baseKey.usages.indexOf('deriveKey') < 0) {
          reject(new DOMException('Key usages do not include "deriveKey"', 'InvalidAccessError')); return;
        }
        _zw_performDerive(algorithm, baseKey, lenBits).then(function (bits) {
          return crypto.subtle.importKey('raw', bits, dka, extractable, usages);
        }).then(resolve, reject);
      });
    },
    // encrypt(algorithm, key, data) → Promise<ArrayBuffer>。AES-GCM：algorithm {name:"AES-GCM", iv, additionalData?, tagLength?}，
    // 返 ct||tag（tag 固定 128 位）。key.usages 须含 "encrypt"。https://w3c.github.io/webcrypto/#SubtleCrypto-method-encrypt
    encrypt: function (algorithm, key, data) {
      return new Promise(function (resolve, reject) {
        var name = (typeof algorithm === 'object' && algorithm) ? algorithm.name : algorithm;
        name = String(name == null ? '' : name).toUpperCase();
        if (name !== 'AES-GCM' || !key || key.algorithm.name !== 'AES-GCM') {
          reject(new DOMException('Unsupported encrypt algorithm or key', 'NotSupportedError')); return;
        }
        if (!key.usages || key.usages.indexOf('encrypt') < 0) {
          reject(new DOMException('Key usages do not include "encrypt"', 'InvalidAccessError')); return;
        }
        var arr = _zw_aesGcmCall('encrypt', algorithm, key, _zw_bufToBytes(data), reject);
        if (arr) resolve(arr);
      });
    },
    // decrypt(algorithm, key, data) → Promise<ArrayBuffer>。data 为 ct||tag（tag 校验失败 → reject OperationError）。
    // https://w3c.github.io/webcrypto/#SubtleCrypto-method-decrypt
    decrypt: function (algorithm, key, data) {
      return new Promise(function (resolve, reject) {
        var name = (typeof algorithm === 'object' && algorithm) ? algorithm.name : algorithm;
        name = String(name == null ? '' : name).toUpperCase();
        if (name !== 'AES-GCM' || !key || key.algorithm.name !== 'AES-GCM') {
          reject(new DOMException('Unsupported decrypt algorithm or key', 'NotSupportedError')); return;
        }
        if (!key.usages || key.usages.indexOf('decrypt') < 0) {
          reject(new DOMException('Key usages do not include "decrypt"', 'InvalidAccessError')); return;
        }
        var arr = _zw_aesGcmCall('decrypt', algorithm, key, _zw_bufToBytes(data), reject);
        if (arr) resolve(arr);
      });
    },
    // generateKey(algorithm, extractable, usages) → Promise<CryptoKey>。AES-GCM → 256 位随机密钥；
    // HMAC → hash 块大小随机密钥。**随机源 = crypto.getRandomValues（Math.random 非 CSPRNG，已知限制）**。
    // https://w3c.github.io/webcrypto/#SubtleCrypto-method-generateKey
    generateKey: function (algorithm, extractable, usages) {
      return new Promise(function (resolve, reject) {
        var algo = _zw_normalizeImportAlgorithm(algorithm);
        if (!algo) {
          reject(new DOMException('Unsupported or missing algorithm', 'NotSupportedError')); return;
        }
        if (algo.name === 'AES-GCM') {
          var u = _zw_normalizeUsages(usages, ['encrypt', 'decrypt']);
          if (!u) { reject(new DOMException('Invalid key usages for AES-GCM', 'SyntaxError')); return; }
          resolve(new CryptoKey('secret', extractable, algo, u, Array.from(_zw_randomBytes(32))));
        } else if (algo.name === 'HMAC') {
          var hu = _zw_normalizeUsages(usages, ['sign', 'verify']);
          if (!hu) { reject(new DOMException('Invalid key usages for HMAC', 'SyntaxError')); return; }
          resolve(new CryptoKey('secret', extractable, algo, hu, Array.from(_zw_randomBytes(_zw_keyLengthBits(algo) / 8))));
        } else {
          reject(new DOMException('Unsupported generateKey algorithm', 'NotSupportedError'));
        }
      });
    },
    // exportKey(format, key) → Promise<ArrayBuffer>。仅 "raw"（jwk defer）；非 extractable key → reject。
    // https://w3c.github.io/webcrypto/#SubtleCrypto-method-exportKey
    exportKey: function (format, key) {
      return new Promise(function (resolve, reject) {
        var fmt = String(format == null ? '' : format).toUpperCase();
        if (fmt !== 'RAW') {
          reject(new DOMException("Unsupported exportKey format: '" + fmt + "' (only 'raw' supported)", 'NotSupportedError')); return;
        }
        if (!key || !key.extractable) {
          reject(new DOMException('Key is not extractable', 'InvalidAccessError')); return;
        }
        var raw = key._raw || [];
        resolve(new Uint8Array(raw));
      });
    }
  };
  globalThis.CryptoKey = globalThis.CryptoKey || CryptoKey;

  // AbortController/AbortSignal——fetch 中止 / 异步流程控制（cancel token 模式，现代 JS 库 / fetch
  // 高频）。V8 embed 不提供，polyfill 之（本地 Chromium 150 oracle 锚定 R2777）。signal.aborted/
  // reason（getter）+ abort(reason) + addEventListener('abort') 触发 + AbortSignal.abort()/timeout
  // 静态工厂 + throwIfAborted()。**关键行为（oracle 锚定）**：abort() 无参时 reason 默认 AbortError
  // DOMException；abort(val) reason 即 val（不包）；重复 abort 静默 no-op（不抛）；throwIfAborted 在
  // aborted 时抛 AbortError DOMException。**已知限制**：signal 非真 EventTarget 子类（
  // `instanceof EventTarget`=false，但 add/removeEventListener/dispatchEvent API 齐备）；AbortSignal.timeout
  // 依赖 setTimeout 回调真触发（sandbox 事件循环须驱动）。
  function AbortSignal() {
    this._aborted = false;
    this._reason = undefined;
    this._listeners = [];
  }
  AbortSignal.prototype = Object.create(Object.prototype);
  AbortSignal.prototype.constructor = AbortSignal;
  Object.defineProperty(AbortSignal.prototype, 'aborted', {
    configurable: true, enumerable: true,
    get: function () { return this._aborted; }
  });
  Object.defineProperty(AbortSignal.prototype, 'reason', {
    configurable: true, enumerable: true,
    get: function () { return this._reason; }
  });
  AbortSignal.prototype.addEventListener = function (type, cb) {
    if (type === 'abort' && typeof cb === 'function') this._listeners.push(cb);
  };
  AbortSignal.prototype.removeEventListener = function (type, cb) {
    if (type !== 'abort') return;
    var i = this._listeners.indexOf(cb);
    if (i >= 0) this._listeners.splice(i, 1);
  };
  AbortSignal.prototype.dispatchEvent = function () { return true; };
  AbortSignal.prototype.throwIfAborted = function () {
    if (this._aborted) {
      // R384（js-dom M5）：**globalThis 优先**（R9/R382 wrong-global 先例）——M5 flip 后
      // globalThis.DOMException = 原生构造器，词法 `new DOMException` 落 shim 闭包内
      // 构造器 → `reason.constructor === iframeWin.DOMException`（= 全局原生构造器）
      // 恒 false（dom/abort/reason-constructor）。裸名仅作 globalThis 未装时的回落。
      var _r384DE = globalThis.DOMException || DOMException;
      throw (this._reason instanceof _r384DE)
        ? this._reason
        : new _r384DE('signal is aborted without reason', 'AbortError');
    }
  };
  // 统一 abort 逻辑（controller.abort 与 AbortSignal.abort 共用）。
  function _zw_abort_signal(signal, reason) {
    if (signal._aborted) return; // 重复 abort 静默 no-op（spec）
    signal._aborted = true;
    if (typeof reason === 'undefined') {
      // R384（js-dom M5）：globalThis 优先——flip 后 abort 默认 reason 须为全局
      //（原生）构造器实例，iframe realm 断言 `reason.constructor === win.DOMException`
      // 才成立（同 throwIfAborted 注释）。
      var _r384DE2 = globalThis.DOMException || DOMException;
      signal._reason = new _r384DE2('signal is aborted without reason', 'AbortError');
    } else {
      signal._reason = reason;
    }
    var ls = signal._listeners.slice();
    signal._listeners = [];
    for (var i = 0; i < ls.length; i++) {
      try { ls[i]({ type: 'abort', target: signal }); } catch (_) {}
    }
  }
  AbortSignal.abort = function (reason) {
    var s = new AbortSignal();
    _zw_abort_signal(s, reason);
    return s;
  };
  AbortSignal.timeout = function (ms) {
    var s = new AbortSignal();
    if (typeof setTimeout === 'function') {
      var handle = setTimeout(function () { _zw_abort_signal(s, undefined); }, Number(ms) || 0);
      // R373：主 realm 调用也登记（当前 global 的可取消集合）——iframe 域走 _zwTimeoutFor。
      try {
        if (!globalThis.__zwAbortTimerIds) globalThis.__zwAbortTimerIds = [];
        globalThis.__zwAbortTimerIds.push(handle);
      } catch (_e373t) {}
    }
    return s;
  };
  // R373（js-dom M4/DC-3）：**frame-scoped timeout**——iframe realm 调用的
  // AbortSignal.timeout(ms) 其定时器归属该 realm 的 window：frame detach（iframe 移除）
  // 取消定时器 → signal.aborted 保持 false（spec：AbortSignal.timeout 的定时器与
  // 「当前全局」关联；WPT dom/abort abort-signal-timeout "not aborted after frame
  // detach"）。_zwRemoveIframeWindowClientsForNodes（part01，iframe 移除路径）按
  // `win.__zwAbortTimerIds` 取消。
  // https://dom.spec.whatwg.org/#abortsignal-timeout
  AbortSignal._zwTimeoutFor = function (win, ms) {
    var s = new AbortSignal();
    if (typeof setTimeout === 'function') {
      var handle = setTimeout(function () { _zw_abort_signal(s, undefined); }, Number(ms) || 0);
      try {
        if (win) {
          if (!win.__zwAbortTimerIds) win.__zwAbortTimerIds = [];
          win.__zwAbortTimerIds.push(handle);
        }
      } catch (_e373tf) {}
    }
    return s;
  };
  function AbortController() {
    var signal = new AbortSignal();
    this._signal = signal;
    this.abort = function (reason) { _zw_abort_signal(signal, reason); };
  }
  Object.defineProperty(AbortController.prototype, 'signal', {
    configurable: true, enumerable: true,
    get: function () { return this._signal; }
  });
  globalThis.AbortController = globalThis.AbortController || AbortController;
  globalThis.AbortSignal = globalThis.AbortSignal || AbortSignal;

  // TextEncoder/TextDecoder——UTF-8 编解码（fetch body / 字符串↔字节互转高频）。纯 JS UTF-8。
  // encoding-compat M3：编码按 spec #utf-8-encoder——孤立代理（高低皆）→ U+FFFD（旧版
  // CESU-8 式直编代理字节）；解码按 spec #utf-8-decoder 逐字节状态机（非法续字节 U+FFFD +
  // 重处理该字节；不完整序列跨 chunk 驻留 state；fatal → TypeError）+ BOM 前缀嗅探机
  //（EF BB BF 可跨 chunk 拆分，失配退回为内容）。
  function _zw_utf8_encode(str) {
    str = String(str == null ? '' : str);
    var bytes = [];
    for (var i = 0; i < str.length; i++) {
      var c = str.charCodeAt(i);
      if (c < 0x80) {
        bytes.push(c);
      } else if (c < 0x800) {
        bytes.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
      } else if (c >= 0xd800 && c <= 0xdbff && str.charCodeAt(i + 1) >= 0xdc00 && str.charCodeAt(i + 1) <= 0xdfff) {
        // 高代理 + 下一个低代理 → astral 码点（4 字节）
        var lo = str.charCodeAt(++i);
        var cp = 0x10000 + ((c & 0x3ff) << 10) + (lo & 0x3ff);
        bytes.push(0xf0 | (cp >> 18), 0x80 | ((cp >> 12) & 0x3f), 0x80 | ((cp >> 6) & 0x3f), 0x80 | (cp & 0x3f));
      } else if ((c >= 0xd800 && c <= 0xdfff)) {
        // 孤立代理（spec #utf-8-encoder：surrogate → error → U+FFFD）
        bytes.push(0xef, 0xbf, 0xbd);
      } else {
        bytes.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
      }
    }
    return bytes;
  }
  // encoding-compat M3：WHATWG UTF-8 解码状态机（encoding.spec §utf-8 decoder）——逐字节
  // 消费；非法续字节 → U+FFFD + 重处理该字节（spec「prepend b to ioQueue」，i 不前进）；
  // 不完整序列跨 chunk 驻留 state（stream 面）；fatal=true → malformed 处 TypeError
  //（调用方捕获后重置状态——spec fatal 错误后本序列作废）。旧版前置整序列检查会吞
  // 尾部 ASCII（F0 9F 41 → 仅 FFFD 丢 A）且错误子部分合并，eof/mistakes 两案面不达。
  // state = { need, cp, lo, hi } 由调用方持有（跨 decode({stream:true}) 调用驻留）。
  function _zw_utf8_decode_state(bytes, state, fatal) {
    var s = '';
    var n = bytes.length;
    var i = 0;
    while (i < n) {
      var b = bytes[i] & 0xFF;
      if (state.need === 0) {
        if (b <= 0x7F) {
          s += String.fromCharCode(b);
        } else if (b >= 0xC2 && b <= 0xDF) {
          state.need = 1; state.cp = b & 0x1F; state.lo = 0x80; state.hi = 0xBF;
        } else if (b >= 0xE0 && b <= 0xEF) {
          state.need = 2; state.cp = b & 0x0F;
          state.lo = (b === 0xE0) ? 0xA0 : 0x80; // long-form 排除
          state.hi = (b === 0xED) ? 0x9F : 0xBF; // 代理排除
        } else if (b >= 0xF0 && b <= 0xF4) {
          state.need = 3; state.cp = b & 0x07;
          state.lo = (b === 0xF0) ? 0x90 : 0x80;
          state.hi = (b === 0xF4) ? 0x8F : 0xBF; // >U+10FFFF 排除
        } else {
          // 非法前导（0x80-0xC1 / 0xF5+）：字节已消费，产 1 U+FFFD / fatal。
          if (fatal) throw new TypeError('Failed to decode: the encoded data is not valid.');
          s += '�';
        }
        i += 1;
      } else if (b < state.lo || b > state.hi) {
        // 非续字节：重置 + 1 U+FFFD + 重处理该字节（i 不前进——spec prepend b to ioQueue）。
        state.need = 0; state.lo = 0x80; state.hi = 0xBF;
        if (fatal) throw new TypeError('Failed to decode: the encoded data is not valid.');
        s += '�';
      } else {
        state.cp = (state.cp << 6) | (b & 0x3F);
        state.need -= 1;
        state.lo = 0x80; state.hi = 0xBF;
        if (state.need === 0) {
          var cp = state.cp;
          if (cp >= 0x10000) {
            cp -= 0x10000;
            s += String.fromCharCode(0xD800 + (cp >> 10), 0xDC00 + (cp & 0x3FF));
          } else {
            s += String.fromCharCode(cp);
          }
        }
        i += 1;
      }
    }
    return s;
  }
  function _zw_utf8_state_new() { return { need: 0, cp: 0, lo: 0x80, hi: 0xBF }; }
  // BOM 前缀嗅探机（utf-8 JS 路径，ignoreBOM=false）：EF BB BF 可跨 chunk 拆分——
  // 缓冲 ≤3 字节，匹配 → 剥除（缓冲丢弃）；失配 → 缓冲退回为内容 + 嗅探关闭（spec：
  // BOM 嗅探仅限流首）。ignoreBOM=true 时不启用（BOM 按内容解出 U+FEFF）。
  function _zw_u8_bom_feed(dec, bytes) {
    if (!dec._zwSniffOn) return bytes;
    var out = [];
    var n = bytes.length;
    for (var i = 0; i < n; i++) {
      var b = bytes[i] & 0xFF;
      var bl = dec._zwSniffBuf.length;
      if ((bl === 0 && b === 0xEF) || (bl === 1 && b === 0xBB) || (bl === 2 && b === 0xBF)) {
        if (bl === 2) {
          // BOM 剥除——本调用余下字节全部按内容（嗅探仅流首，循环即止）。
          dec._zwSniffBuf = [];
          dec._zwSniffOn = false;
          for (var r = i + 1; r < n; r++) out.push(bytes[r] & 0xFF);
          return out;
        }
        dec._zwSniffBuf.push(b);
      } else {
        for (var j = 0; j < bl; j++) out.push(dec._zwSniffBuf[j]); // 失配退回为内容
        dec._zwSniffBuf = [];
        dec._zwSniffOn = false;
        for (var k = i; k < n; k++) out.push(bytes[k] & 0xFF); // 余下字节内容直通
        return out;
      }
    }
    return out;
  }
  // 单次（flush）解码：truncated 尾部 → 1 U+FFFD（blob text / worker blob script 消费）。
  function _zw_utf8_decode(bytes) {
    var state = _zw_utf8_state_new();
    var s = _zw_utf8_decode_state(bytes, state, false);
    return state.need > 0 ? s + '�' : s;
  }
  globalThis.TextEncoder = globalThis.TextEncoder || function TextEncoder() {
    if (!(this instanceof TextEncoder)) return new TextEncoder();
  };
  globalThis.TextEncoder.prototype = {
    encoding: 'utf-8',
    // spec #dom-textencoder-encode：str 缺省 ""（api-basics Default inputs 面）。
    encode: function (str) {
      var bytes = _zw_utf8_encode(str == null ? '' : str);
      var arr = new Uint8Array(bytes.length);
      for (var k = 0; k < bytes.length; k++) arr[k] = bytes[k];
      return arr;
    },
    // spec #dom-textencoder-encodeinto：read = 消费的 UTF-16 码元数（destination 容不
    // 下下一字符时停在字符边界），written = 写入字节数（字符永不劈开——容量不足整字符
    // 即停）。孤立代理先折算 U+FFFD（同 encode）。非 TypedArray/array-like dst → TypeError。
    encodeInto: function (str, dst) {
      // destination 仅 Uint8Array（含 SAB 背书——spec BufferSource 限定；Int8Array 等
      // 其他 TypedArray → TypeError，encodeInto Invalid destination 面）。
      if (!(typeof Uint8Array !== 'undefined' && dst instanceof Uint8Array)) {
        throw new TypeError("Failed to execute 'encodeInto' on 'TextEncoder': parameter 2 is not of type 'Uint8Array'.");
      }
      str = String(str == null ? '' : str);
      var read = 0, written = 0;
      var i = 0;
      var len = dst.length;
      while (i < str.length) {
        var c = str.charCodeAt(i);
        var cp, units;
        if (c >= 0xD800 && c <= 0xDBFF) {
          var lo = str.charCodeAt(i + 1);
          if (lo >= 0xDC00 && lo <= 0xDFFF) { cp = 0x10000 + ((c & 0x3FF) << 10) + (lo & 0x3FF); units = 2; }
          else { cp = 0xFFFD; units = 1; } // 孤立高代理
        } else if (c >= 0xDC00 && c <= 0xDFFF) {
          cp = 0xFFFD; units = 1; // 孤立低代理
        } else {
          cp = c; units = 1;
        }
        var need = cp < 0x80 ? 1 : cp < 0x800 ? 2 : cp < 0x10000 ? 3 : 4;
        if (written + need > len) break; // destination 容不下整字符 → 停在字符边界
        if (need === 1) dst[written] = cp;
        else if (need === 2) {
          dst[written] = 0xC0 | (cp >> 6); dst[written + 1] = 0x80 | (cp & 0x3F);
        } else if (need === 3) {
          dst[written] = 0xE0 | (cp >> 12); dst[written + 1] = 0x80 | ((cp >> 6) & 0x3F); dst[written + 2] = 0x80 | (cp & 0x3F);
        } else {
          dst[written] = 0xF0 | (cp >> 18); dst[written + 1] = 0x80 | ((cp >> 12) & 0x3F);
          dst[written + 2] = 0x80 | ((cp >> 6) & 0x3F); dst[written + 3] = 0x80 | (cp & 0x3F);
        }
        i += units;
        read = i;
        written += need;
      }
      return { read: read, written: written };
    }
  };
  // TextDecoder（encoding-compat M2）——labels 标签匹配 + legacy 编码解码。
  // https://encoding.spec.whatwg.org/#dom-textdecoder
  // 构造：label（缺省 utf-8）经 host `__zw_text_encoding_of` 查全表（trim ASCII
  // whitespace + ASCII case-insensitive）；未知标签 → RangeError（spec：get an encoding
  // by label failure）；replacement 编码 → RangeError（spec：replacement 不可经 decoder
  // 构造——XHR final-encoding 解码面在 part03 走 host 单 U+FFFD 路径）。非 utf-8 编码走
  // host 有状态 decoder（`__zw_text_decoder_new/decode`——跨 decode({stream}) 调用驻留
  // 半截多字节 lead 与 iso-2022-jp ESC 模式机；BOM 剥除/ignoreBOM 语义在 host decoder
  // 构造选型）。host 未注册（engine/reftest/polyfill 无 script-runtime）→ 留守 utf-8
  // 纯 JS 路径（下方 _zw_utf8_decode_state），label 校验降级为不抛（零回归）。
  globalThis.TextDecoder = globalThis.TextDecoder || function TextDecoder(label, options) {
    if (!(this instanceof TextDecoder)) return new TextDecoder(label, options);
    var raw = label == null ? '' : String(label);
    var name = 'utf-8';
    if (raw !== '' && typeof __zw_text_encoding_of === 'function') {
      name = __zw_text_encoding_of(raw);
      if (name === '') {
        throw new RangeError("Failed to construct 'TextDecoder': The encoding label provided ('" + raw + "') is invalid.");
      }
      if (name === 'replacement') {
        throw new RangeError("Failed to construct 'TextDecoder': The encoding label provided ('" + raw + "') is invalid.");
      }
    }
    this.encoding = name;
    this.fatal = !!(options && options.fatal);
    this.ignoreBOM = !!(options && options.ignoreBOM);
    // utf-8 JS 路径状态：spec #utf-8-decoder 状态机 + BOM 前缀嗅探机（M3——跨 chunk
    // 序列驻留 / BOM 拆分，替代旧 _carry 字节尾 + 单次首块 BOM 检查）。
    this._zwU8State = _zw_utf8_state_new();
    this._zwSniffOn = !this.ignoreBOM;
    this._zwSniffBuf = [];
    // legacy host decoder handle（非 utf-8 且 host 可用）——null = 降级 utf-8 路径。
    this._zwLegacyHandle = (name !== 'utf-8' && typeof __zw_text_decoder_new === 'function')
      ? __zw_text_decoder_new(name, this.ignoreBOM ? '1' : '0')
      : null;
    this._zwDone = false; // 上轮 flush 收尾标记（host decoder finished，下一 decode 序列重建）
  };
  globalThis.TextDecoder.prototype = {
    encoding: 'utf-8',
    fatal: false,
    ignoreBOM: false,
    // R3012：decode(buf, {stream})。stream:true → 不完整尾部入 _carry 待下块（不 flush）；stream:false（缺省）
    // → flush：残余不完整 → 1 U+FFFD，重置 _carry。valid 完整输入行为同旧。
    // net-api M4-S21：UTF-8 decode 剥首部 BOM（encoding spec UTF-8 decode——ignoreBOM=false
    // 缺省；xhr json.any.js data: URL BOM 面与 fetch text() 一致形态）。
    // encoding-compat M2：非 utf-8 编码走 host 有状态 decoder（`last` = !stream；fatal
    // malformed → TypeError 并重建 handle——spec fatal 错误后 decoder 不再可用）。
    decode: function (buf, options) {
      var bytes;
      if (buf == null) bytes = new Uint8Array(0);
      else if (buf instanceof ArrayBuffer) bytes = new Uint8Array(buf);
      else if (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView(buf)) {
        // TypedArray / DataView：respect 视图 byteOffset/byteLength（fatal DataView 子视图
        // 面——取全 buffer 会把截断序列当完整序列）。
        bytes = new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
      } else if (buf && typeof buf.length === 'number') bytes = buf; // array-like
      else if (buf && buf.buffer) bytes = new Uint8Array(buf.buffer);
      else bytes = new Uint8Array(0);
      if (this._zwLegacyHandle != null) {
        var streamFlag = options && options.stream === true;
        // spec：非 stream decode() 重置解码器状态——host decoder 上轮 flush 后已
        // finished（encoding_rs 契约禁复用），新一轮重建 handle。
        if (this._zwDone) {
          this._zwLegacyHandle = __zw_text_decoder_new(this.encoding, this.ignoreBOM ? '1' : '0');
          this._zwDone = false;
        }
        var legacyFatal = this.fatal ? '1' : '0';
        var last = streamFlag ? '0' : '1';
        var csv = typeof bytes.join === 'function' ? bytes.join(',') : _zwBytesCsv(bytes);
        var out = JSON.parse(__zw_text_decoder_decode(this._zwLegacyHandle, csv, legacyFatal, last));
        if (out.err) {
          // spec fatal：错误后 decoder 不再可用——重建 handle 丢弃损坏状态。
          this._zwLegacyHandle = __zw_text_decoder_new(this.encoding, this.ignoreBOM ? '1' : '0');
          throw new TypeError("Failed to decode: the encoded data is not valid.");
        }
        if (!streamFlag) this._zwDone = true;
        return out.text;
      }
      // utf-8 JS 路径：BOM 前缀嗅探（可跨 chunk）→ 逐字节状态机（fatal → TypeError）。
      var streamFlag8 = options && options.stream === true;
      if (this._zwSniffOn || (!streamFlag8 && this._zwSniffBuf.length)) {
        // 嗅探期（流首）或 flush 收尾：过普通数组（TypedArray 无 push；稳态嗅探关闭零开销）。
        var arr8 = [];
        for (var bi8 = 0; bi8 < bytes.length; bi8++) arr8.push(bytes[bi8] & 0xFF);
        if (this._zwSniffOn) {
          arr8 = _zw_u8_bom_feed(this, arr8);
          if (!streamFlag8 && this._zwSniffOn && this._zwSniffBuf.length) {
            // flush：未完成 BOM 前缀退回为内容（非完整 BOM 不剥）。
            for (var sb8 = 0; sb8 < this._zwSniffBuf.length; sb8++) arr8.push(this._zwSniffBuf[sb8]);
            this._zwSniffBuf = [];
            this._zwSniffOn = false;
          }
        }
        bytes = arr8;
      }
      try {
        var s8 = _zw_utf8_decode_state(bytes, this._zwU8State, this.fatal);
        if (streamFlag8) return s8;
        if (this._zwU8State.need > 0) {
          // flush：残余不完整序列 → fatal throw / 1 U+FFFD。
          if (this.fatal) throw new TypeError('Failed to decode: the encoded data is not valid.');
          s8 += '�';
        }
        return s8;
      } catch (e8) {
        // fatal 错误：本序列作废——重置（含 stream 面；错误后调用方拿到 TypeError）。
        this._zwU8State = _zw_utf8_state_new();
        this._zwSniffOn = !this.ignoreBOM;
        this._zwSniffBuf = [];
        throw e8;
      } finally {
        if (!streamFlag8) {
          // spec：非 stream decode() 重置序列状态（正常 flush 完成）。
          this._zwU8State = _zw_utf8_state_new();
          this._zwSniffOn = !this.ignoreBOM;
          this._zwSniffBuf = [];
        }
      }
    }
  };
  // array-like（无 join 的 host wire 形态）→ csv 十进制串（__zw_text_decoder_decode 字节 wire）。
  function _zwBytesCsv(bytes) {
    var s = '';
    for (var i = 0; i < bytes.length; i++) {
      if (i > 0) s += ',';
      s += (bytes[i] & 0xFF);
    }
    return s;
  }

  // ── P1a ReadableStream（Streams API，R2967）──
  // 通用读取流抽象。核心动机：fetch `response.body`（此前全缺——仅有 text()/json() 整体读），
  // 解锁流式消费（@json/streaming / readable-stream / service worker / 逐块解析库）+ 自定义流
  //（测试 mock / 数据管道）。纯 JS 控制器模型：underlyingSource {start, pull, cancel} +
  // ReadableStreamDefaultController {enqueue, close, error, desiredSize}。默认 reader：
  // read()→Promise<{done,value}> / cancel(reason) / releaseLock() / closed；locked 守卫；
  // Symbol.asyncIterator（for await of）。push（start-enqueue）+ pull（queue 空 read 时触发）双源。
  // pipeTo(WritableStream) / pipeThrough({writable,readable})（R2969）。WritableStream/TransformStream 见下。
  // tee()（分叉两独立分支）R2971。
  var _RS_DONE = { done: true, value: undefined };
  if (typeof Object.setPrototypeOf === 'function') Object.setPrototypeOf(_RS_DONE, null); // net-api M2-S3：防 then 投毒（见 _rs_chunk）
  // net-api M2-S3：read-result 对象以 **null 原型** 发布——Object.prototype.then 投毒
  //（response-stream-with-broken-then）经 thenable adoption 劫持 Promise resolution，
  // 裸字面量原型链上查得注入 then。
  function _rs_chunk(value) {
    var out = { done: false, value: value };
    if (typeof Object.setPrototypeOf === 'function') Object.setPrototypeOf(out, null);
    return out;
  }
  // net-api M4-S4：QueuingStrategy dictionary 转换（spec 构造 dictionary 转换序——成员读取
  //（highWaterMark → size 定义序，getter 抛错同步传播）+ size callback 可调用校验（非函数 →
  // TypeError）**先于构造步骤**；NaN/负数 RangeError 归 ExtractHighWaterMark（构造步骤内，
  // 晚于 type:'bytes' 的 size-exists 检查）。bad-strategies / writable bad-strategies 面锚。
  function _zwStrategyDict(strategy) {
    if (strategy == null) return { hwmValue: undefined, size: undefined };
    var hwmValue = strategy.highWaterMark;
    var size = strategy.size;
    if (size !== undefined && typeof size !== 'function') {
      throw new TypeError("The queuing strategy's 'size' property must be a function.");
    }
    return { hwmValue: hwmValue, size: size };
  }
  // net-api M4-S4：ExtractHighWaterMark（spec §7.4——NaN/负数 → RangeError；+∞ 允许；缺省 → defaultHWM）。
  function _zwExtractHwm(hwmValue, defaultHwm) {
    if (hwmValue === undefined) return defaultHwm;
    var v = Number(hwmValue);
    if (v !== v || v < 0) {
      throw new RangeError("The provided strategy's 'highWaterMark' must be a non-negative number.");
    }
    return v;
  }
  // net-api M4-S4：StreamPipeOptions 序贯读取（preventAbort → preventCancel → preventClose →
  // signal——throwing-options 页 touched 序锚；getter 抛错向调用方传播：pipeTo 转 rejected
  // promise、pipeThrough 同步 throw）。提取后传 impl（getter 恰触一次——touched 恰等断言面）。
  function _zwReadPipeOptions(options) {
    var out = { preventAbort: false, preventCancel: false, preventClose: false, signal: undefined };
    if (options != null && typeof options === 'object') {
      out.preventAbort = !!(options.preventAbort);
      out.preventCancel = !!(options.preventCancel);
      out.preventClose = !!(options.preventClose);
      out.signal = options.signal;
    }
    return out;
  }
  // net-api M4-S7：ReadableStreamBYOBReader / ReadableStreamBYOBRequest 全局类——
  // reader 构造须字节流（brand + type 校验 + locked → TypeError，getReader 同厂）；
  // request 不可构造（构造即 TypeError）。
  function ReadableStreamBYOBRequest() {
    throw new TypeError('Illegal constructor');
  }
  function ReadableStreamBYOBReader(stream) {
    if (!stream || !stream._zwRsBrand) {
      throw new TypeError("Failed to construct 'ReadableStreamBYOBReader': stream must be a ReadableStream.");
    }
    if (!stream._zwIsByteStream) {
      throw new TypeError('BYOB reader requires a ReadableStream with type "bytes"');
    }
    return stream.getReader({ mode: 'byob' }); // locked → getReader TypeError
  }
  globalThis.ReadableStreamBYOBRequest = globalThis.ReadableStreamBYOBRequest || ReadableStreamBYOBRequest;
  globalThis.ReadableStreamBYOBReader = globalThis.ReadableStreamBYOBReader || ReadableStreamBYOBReader;

  // net-api M4-S5：ReadableStreamDefaultController 全局类——spec 不可构造（构造即 TypeError；
  // controller 对象经 proto.constructor 暴露同一函数——writable constructor 页 `c.constructor`
  // 面）。实例链：ReadableStream 构造时 Object.create(controllerProto)。
  function ReadableStreamDefaultController() {
    throw new TypeError('Illegal constructor');
  }
  globalThis.ReadableStreamDefaultController = globalThis.ReadableStreamDefaultController || ReadableStreamDefaultController;
  globalThis.ReadableStream = globalThis.ReadableStream || function ReadableStream(underlyingSource, _strategy) {
    if (!(this instanceof ReadableStream)) return new ReadableStream(underlyingSource, _strategy);
    // net-api M4-S5：WebIDL object 转换——显式 null → TypeError（「constructor should throw when
    // the source is null」面）；undefined（含缺省）→ null = 无源流（M4-S4 判例）；其余非 object
    //（原语）→ TypeError（「can't be constructed with garbage」面）。
    if (underlyingSource === null || (underlyingSource !== undefined &&
        typeof underlyingSource !== 'object' && typeof underlyingSource !== 'function')) {
      throw new TypeError("Failed to construct 'ReadableStream': underlyingSource must be an object.");
    }
    // net-api M4-S4：构造 dictionary 转换序——strategy dictionary 转换（成员 getter 读取 + size
    // 可调用校验）**先于构造步骤**（含 source.type 读取）。
    var strat = _zwStrategyDict(_strategy);
    var source = underlyingSource === undefined ? {} : underlyingSource;
    // net-api M4-S7：autoAllocateChunkSize === 0 → TypeError（spec——0 非法，
    // byte-streams/general「autoAllocateChunkSize cannot be 0」面）。
    if (source.autoAllocateChunkSize === 0) {
      throw new TypeError('autoAllocateChunkSize cannot be 0');
    }
    // net-api M4-S4：UnderlyingSource dictionary 转换——start/pull/cancel 成员**构造时一次读取**
    //（定义序，getter 抛错同步传播出构造器——bad-underlying-sources「throwing getter」面），
    // 算法缓存（「second pull does not result in a second get」面）；回调 this = underlyingSource。
    // net-api M4-S5：callback 类型转换——非 null/undefined 且非函数 → TypeError（general
    // 「will not tolerate initial garbage as start/pull/cancel argument」面；WebIDL callback
    // 接受 null/undefined = absent）。
    var srcStart = source.start;
    var srcPull = source.pull;
    var srcCancel = source.cancel;
    if ((srcStart !== undefined && srcStart !== null && typeof srcStart !== 'function') ||
        (srcPull !== undefined && srcPull !== null && typeof srcPull !== 'function') ||
        (srcCancel !== undefined && srcCancel !== null && typeof srcCancel !== 'function')) {
      throw new TypeError("Failed to construct 'ReadableStream': start/pull/cancel must be functions or undefined.");
    }
    // net-api M4-S5：ReadableStreamType enum 转换（定义序在 cancel 后——ToString 抛错传播；
    // 非 'bytes' → TypeError——general「can't be constructed with an invalid type」面）。
    var rawType = source.type;
    if (rawType !== undefined) {
      var typeStr = String(rawType);
      if (typeStr !== 'bytes') {
        throw new TypeError("Failed to construct 'ReadableStream': invalid type.");
      }
      this._zwIsByteStream = true;
    } else {
      this._zwIsByteStream = false;
    }
    // spec 构造步骤 4.1：bytes 流禁 strategy size（RangeError）；ExtractHighWaterMark——bytes 默认 0 /
    // default 默认 1（NaN/负数 → RangeError，+∞ 允许——bad-strategies「invalid strategy.highWaterMark」面）。
    if (this._zwIsByteStream && strat.size !== undefined) {
      throw new RangeError('ReadableStream with type "bytes" cannot have a strategy with a "size" member');
    }
    var hwm = _zwExtractHwm(strat.hwmValue, this._zwIsByteStream ? 0 : 1);
    var sizeFn = strat.size !== undefined ? strat.size : null;
    var aacs = (this._zwIsByteStream && typeof source.autoAllocateChunkSize === 'number' && source.autoAllocateChunkSize > 0)
      ? source.autoAllocateChunkSize : 0; // net-api M4-S7：autoAllocateChunkSize（benign byobRequest 面）
    var queue = [];              // 已 enqueue 待消费 { chunk, size }
    var queueTotalSize = 0;
    var state = 'readable';      // readable | closed | errored
    var errorVal = undefined;
    var waiting = [];            // 待 read() 的 {resolve, reject}
    var pulling = false;
    var pullAgain = false;
    var closeRequested = false;  // close() 时 queue 非空 → 排空后真关（spec closeRequested）
    var started = false;
    // net-api M4-S4：reader.[[closedPromise]] 全局化——**每次 getReader 访问 closed 都新建
    // 永挂 Promise** 的旧形态（closed getter fresh-promise）使「先取 closed、后 error/cancel」
    // 的腿永不 settle（bad-strategies / bad-underlying-sources / default-reader 页级 Timeout
    // 根因）。waiters 列表：closeStream resolve 全部、errorStream reject 全部、releaseLock
    // 摘除并按 spec GenericRelease 拒 TypeError（readable 态）。
    var closedWaiters = [];
    var self = this;
    this._locked = false;
    // net-api M2-S3：disturbed 标记（spec §3.6——read() 首调 / cancel 即 disturbed；
    // Response/Request consume body 的 unusable 判定消费此标记）。
    this._disturbed = false;

    function enqueueChunk(chunk) {
      // net-api M4-S4：spec §ReadableStreamDefaultControllerEnqueue——CanCloseOrEnqueue
      //（readable + 非 closeRequested）假 → TypeError（bad-underlying-sources「enqueue on a
      // canceled/closed stream should throw」面）；有等待 read → FulfillReadRequest（跳过 size
      // 计算）；size 抛错 → Error(controller, e) + 重抛（「strategy.size errors the stream and
      // then throws」）；size 非法返回（NaN/负数/±∞）→ RangeError + error 流；末尾 CallPullIfNeeded。
      if (state !== 'readable' || closeRequested) {
        throw new TypeError('Cannot enqueue a chunk into a ' + (closeRequested ? 'closing' : state) + ' readable stream');
      }
      if (self._zwIsByteStream) {
        // net-api M4-S7：spec 字节流 enqueue——零长度视图/零长缓冲 → TypeError（bad-buffers
        // 「enqueuing a zero-length buffer throws」面）；pending pull-into → 填头描述符
        //（跨描述符续填 + 余量回队）；无 pull-into → 常规队列（字节 size——hwm 默认 0）。
        if (!chunk || typeof chunk.byteLength !== 'number' || chunk.byteLength === 0) {
          throw new TypeError('Cannot enqueue a zero-length view');
        }
        // spec FulfillReadRequest——默认 reader 等待读直接喂 chunk（跳过则队列无界增长——
        // respond-after-enqueue 页内存爆涨根因）。
        if (waiting.length > 0) { waiting.shift().resolve(_rs_chunk(chunk)); flushPull(); return; }
        var srcB = chunk instanceof Uint8Array ? chunk : null;
        if (pullIntos.length > 0) {
          var srcE = srcB || new Uint8Array(0);
          while (srcE !== null && srcE.length > 0 && pullIntos.length > 0) {
            var dFill = pullIntos[0];
            var spaceF = dFill.byteLength - dFill.bytesFilled;
            var takeF = Math.min(spaceF, srcE.length);
            try {
              new Uint8Array(dFill.buffer, dFill.byteOffset + dFill.bytesFilled, takeF).set(srcE.subarray(0, takeF));
            } catch (_eFill) { throw new TypeError('enqueue: buffer fill failed (' + _eFill.message + ')'); }
            dFill.bytesFilled += takeF;
            srcE = takeF < srcE.length ? srcE.subarray(takeF) : null;
            if (dFill.bytesFilled >= dFill.min) {
              pullIntos.shift();
              commitDescriptor(dFill);
            }
          }
          if (srcE) { queue.push({ chunk: srcE, size: srcE.length }); queueTotalSize += srcE.length; }
          // spec Enqueue 步骤 12——CallPullIfNeeded（部分填充未达 min → 续拉补齐：
          // 「multiple enqueue() up to 3 bytes」pullCount===2 面）。
          flushPull();
          return;
        }
        queue.push({ chunk: srcB || chunk, size: chunk.byteLength });
        queueTotalSize += chunk.byteLength;
        flushPull();
        return;
      }
      if (waiting.length > 0) { waiting.shift().resolve(_rs_chunk(chunk)); flushPull(); return; }
      var sz;
      try { sz = (typeof sizeFn === 'function') ? sizeFn(chunk) : 1; }
      catch (eSize) { errorStream(eSize); throw eSize; }
      if (typeof sz !== 'number' || sz !== sz || sz < 0 || sz === Infinity) {
        var reSize = new RangeError('Invalid chunk size');
        errorStream(reSize);
        throw reSize;
      }
      queue.push({ chunk: chunk, size: sz });
      queueTotalSize += sz;
      flushPull();
    }
    var pullIntos = []; // net-api M4-S7：BYOB pull-into 描述符 FIFO {buffer,byteOffset,byteLength,bytesFilled,min,resolve,viewCtor,isDataView,elementSize}
    function closeStream(byCancel) {
      if (state !== 'readable') return;
      state = 'closed';
      while (waiting.length > 0) waiting.shift().resolve(_RS_DONE);
      while (closedWaiters.length > 0) closedWaiters.shift().resolve();
      // net-api M4-S7：字节流 close——pending pull-into close steps（spec：普通 close 交还
      // 已填视图/空视图 done:true；**cancel 路径 close steps given undefined**——spec
      // ReadableStreamCancel 步骤 6）。
      while (pullIntos.length > 0) {
        var dClose = pullIntos.shift();
        var vClose;
        if (byCancel) vClose = undefined;
        else {
          try {
            vClose = dClose.isDataView ? new DataView(dClose.buffer, dClose.byteOffset, dClose.bytesFilled)
                                       : new dClose.viewCtor(dClose.buffer, dClose.byteOffset, dClose.bytesFilled / dClose.elementSize);
          } catch (_eCdB) { vClose = undefined; }
        }
        dClose.resolve({ value: vClose, done: true });
      }
    }
    function errorStream(e) {
      if (state !== 'readable') return;
      errorVal = e;
      state = 'errored';
      while (waiting.length > 0) waiting.shift().reject(e);
      // net-api M4-S7：pending pull-into 描述符 error steps（「read({min}), then error()」面）。
      while (pullIntos.length > 0) pullIntos.shift().reject(e);
      while (closedWaiters.length > 0) {
        var cwE = closedWaiters.shift();
        if (typeof cwE.markHandled === 'function') cwE.markHandled(); // spec ReadableStreamError 步骤 7
        cwE.reject(e);
      }
    }
    function flushPull() {
      // net-api M4-S4：spec §CallPullIfNeeded——started + readable + 非 closeRequested +
      //（有等待 read 请求**或** desiredSize>0——hwm 0 策略 read 请求优先面）；pulling → 置
      // pullAgain；pull 同步完成时 pullAgain 经**微任务**重拉（spec upon-fulfillment 语义，
      // 避免零 size 源同步递归栈爆）。
      if (!started || state !== 'readable' || closeRequested) return;
      if (typeof srcPull !== 'function') return;
      // net-api M4-S7：pull-into 描述符亦触发（spec ShouldCallPull——BYOB read-into-requests > 0）。
      if (waiting.length === 0 && pullIntos.length === 0 && hwm - queueTotalSize <= 0) return;
      if (pulling) { pullAgain = true; return; } // spec：pull 期间 → pullAgain（完成后续拉）
      pulling = true;
      var result;
      try { result = srcPull.call(source, controller); } catch (ePull) { pulling = false; errorStream(ePull); return; }
      if (result && typeof result.then === 'function') {
        Promise.resolve(result).then(function () {
          pulling = false;
          if (pullAgain) { pullAgain = false; flushPull(); }
        }, function (ePull) { pulling = false; pullAgain = false; errorStream(ePull); });
      } else {
        pulling = false;
        if (pullAgain) { pullAgain = false; Promise.resolve().then(flushPull); }
      }
    }
    // net-api M4-S7：pull-into 描述符 commit（chunk steps——同缓冲新视图 done:false）+
    // 排空后续（drain-close 或续拉）。
    function commitDescriptor(d) {
      var vi = pullIntos.indexOf(d);
      if (vi >= 0) pullIntos.splice(vi, 1);
      var out;
      try {
        out = d.isDataView ? new DataView(d.buffer, d.byteOffset, d.bytesFilled)
                           : new d.viewCtor(d.buffer, d.byteOffset, d.bytesFilled / d.elementSize);
      } catch (_eCmt) { out = new Uint8Array(0); }
      d.resolve({ value: out, done: false });
      if (closeRequested && queue.length === 0) closeStream(false);
      else flushPull();
    }
    // net-api M4-S5：controller 原型面（general「start controller parameter should be
    // extensible」——proto own props 恰 close/constructor/desiredSize/enqueue/error）。
    var controllerProto = { constructor: ReadableStreamDefaultController };
    if (self._zwIsByteStream) {
      // net-api M4-S7：ReadableByteStreamController.byobRequest（spec §4.7.3——pending
      // pull-into 非空时返回 BYOBRequest：view = 构造于 buffer 已填偏移之后的余量视图；
      // respond(n) 最小填充契约：filled >= min 才 commit，closed 态零写入收尾）。
      Object.defineProperty(controllerProto, 'byobRequest', {
        get: function () {
          if (pullIntos.length === 0) {
            // net-api M4-S7：autoAllocateChunkSize benign 形（respond-after-enqueue 3 腿——
            // 默认读走队列直填，byobRequest 非空可安全 respond/no-op，不参与填充）。
            if (aacs > 0) {
              var benignBuffer = new ArrayBuffer(aacs);
              var benignView = new Uint8Array(benignBuffer);
              return {
                get view() { return benignView; },
                respond: function () { flushPull(); },
                respondWithNewView: function () { flushPull(); }
              };
            }
            return null;
          }
          var d = pullIntos[0];
          var rem = d.byteLength - d.bytesFilled;
          var view;
          try {
            view = d.isDataView ? new DataView(d.buffer, d.byteOffset + d.bytesFilled, rem)
                                : new d.viewCtor(d.buffer, d.byteOffset + d.bytesFilled, rem / d.elementSize);
          } catch (_eBv) { return null; }
          return {
            get view() { return view; },
            respond: function (bytesWritten) {
              if (state === 'errored') throw new TypeError('Controller is errored');
              if (d.bytesFilled + bytesWritten > d.byteLength) {
                throw new RangeError('bytesWritten exceeds view capacity');
              }
              d.bytesFilled += bytesWritten;
              if (state === 'closed') {
                if (bytesWritten !== 0) throw new TypeError('closed stream requires zero bytesWritten');
                pullIntos.splice(pullIntos.indexOf(d), 1);
                var vCl;
                try {
                  vCl = d.isDataView ? new DataView(d.buffer, d.byteOffset, d.bytesFilled)
                                     : new d.viewCtor(d.buffer, d.byteOffset, d.bytesFilled / d.elementSize);
                } catch (_eCl2) { vCl = undefined; }
                d.resolve({ value: vCl, done: true });
                return;
              }
              if (d.bytesFilled >= d.min) commitDescriptor(d);
              else flushPull(); // 未达 min → 续拉（源可再 respond——「pull must have been called 3 times」面）
            },
            respondWithNewView: function (v2) {
              if (state === 'errored') throw new TypeError('Controller is errored');
              if (!v2 || typeof v2.byteLength !== 'number' || v2.byteLength === 0) {
                throw new TypeError('respondWithNewView: view must be non-zero-length');
              }
              if (state === 'closed' && v2.byteLength !== 0) throw new TypeError('closed stream requires zero-length view');
              // net-api M4-S10：spec RespondWithNewView 校验（steps 7-9）——① 描述符写入偏移
              // = 视图偏移（RangeError——'byobRequest.view.subarray(1,2)' 错位面）；② 描述符
              // **缓冲**字节长 = 视图缓冲字节长（原实现对描述符视图长比较——分支 tee 把整缓冲
              // 分支视图（byteLength 1 / buffer 3）回提交被误拒）；③ 已填 + 视图长 ≤ 描述符长。
              if (d.byteOffset + d.bytesFilled !== v2.byteOffset) {
                throw new RangeError('respondWithNewView: view byteOffset mismatch');
              }
              if (d.buffer.byteLength !== v2.buffer.byteLength) {
                throw new RangeError('respondWithNewView: view buffer length mismatch');
              }
              if (d.bytesFilled + v2.byteLength > d.byteLength) {
                throw new RangeError('respondWithNewView: view exceeds pull-into capacity');
              }
              d.buffer = v2.buffer;
              d.byteOffset = v2.byteOffset;
              d.bytesFilled = v2.byteLength;
              if (state === 'closed') {
                pullIntos.splice(pullIntos.indexOf(d), 1);
                d.resolve({ value: v2, done: true });
                return;
              }
              if (d.bytesFilled >= d.min) commitDescriptor(d);
              else flushPull();
            }
          };
        },
        enumerable: true, configurable: true
      });
    }
    Object.defineProperty(controllerProto, 'desiredSize', {
      // spec：readable → hwm - queueTotalSize；closed → 0；errored → null。
      get: function () {
        if (state === 'errored') return null;
        if (state === 'closed') return 0;
        return hwm - queueTotalSize;
      },
      enumerable: true, configurable: true
    });
    controllerProto.enqueue = enqueueChunk;
    controllerProto.close = function () {
      // net-api M4-S4：spec §close——CanCloseOrEnqueue 假 → TypeError（bad-underlying-sources
      // 「close twice / close after cancel / close after error」面）；queue 非空 → closeRequested
      //（read drain 面排空后真关），空 → 即关。
      if (state !== 'readable' || closeRequested) {
        throw new TypeError('Cannot close a readable stream that is ' + (closeRequested ? 'closing' : state));
      }
      if (queue.length > 0) { closeRequested = true; return; }
      closeStream();
    };
    controllerProto.error = errorStream;
    var controller = Object.create(controllerProto);
    // net-api M4-S4：spec §ReadableStreamCancel——disturbed；closed → resolved；errored →
    // reject storedError（「return() rejects if the stream has errored」面）；否则 close +
    // CancelSteps（await source.cancel——fulfillment → undefined、rejection 传播）。
    function cancelInternal(reason) {
      self._disturbed = true;
      if (state === 'closed') return Promise.resolve();
      if (state === 'errored') return Promise.reject(errorVal);
      return self._doCancel(reason);
    }
    this._doCancel = function (reason) {
      // net-api M4-S10：cancel 的 pull-into close steps **given undefined**（spec
      // ReadableStreamCancel 步骤 6——缓冲丢弃，read 兑现 {value: undefined, done: true}；
      // 原 closeStream() 无参把已填视图交还——'canceling both branches in sequence with
      // delay' 断言 value undefined 面）。
      closeStream(true);
      // spec ReadableStreamCancel 步骤 3——errored → reject storedError（tee composite cancel
      // 的 cancelPromise 兑现值采用该 rejection——'erroring a teed stream should properly
      // handle canceled branches' 双页断言面：分支 cancel 须以源错误拒绝）。
      if (state === 'errored') return Promise.reject(errorVal);
      var p;
      if (typeof srcCancel === 'function') {
        try { p = Promise.resolve(srcCancel.call(source, reason)); } catch (eCancel) { p = Promise.reject(eCancel); }
      } else { p = Promise.resolve(); }
      return p.then(function () {}); // fulfillment → undefined；rejection 传播（cancel 回调抛错面）
    };
    // net-api M4-S4：spec GenericRelease——解锁 + waiter 摘除；readable 态 closed promise 转
    // TypeError 拒绝（「released reader appears errored」面）；closed/errored 态已 settle 不动。
    function releaseReaderLock(entry, closedP) {
      if (!self._locked) return;
      self._locked = false;
      var idx = closedWaiters.indexOf(entry);
      if (idx >= 0) closedWaiters.splice(idx, 1);
      if (state === 'readable') {
        entry.reject(new TypeError('ReadableStream reader released lock'));
        try { closedP.catch(function () {}); } catch (_eRel) {} // 已发布 promise 的拒绝标记 handled（runner unhandled-rejection 面）
      }
    }
    // net-api M4-S10：BYOB read-into 原语上移构造器作用域（原 byob reader 闭包内 readInto——
    // 只引用构造器级状态 queue/pullIntos/state/closeStream/flushPull，无 reader 本地态）——
    // 同一实现同时服务 ① byob reader 的公开 read(view) 与 ② tee 的源侧 byob 读
    //（spec ReadableByteStreamTee pullWithBYOBReader——分支 pull-into 视图直入源 pull-into，
    // 源 enqueue/respond 经分支缓冲物理写入后 respond 回填分支描述符）。
    // spec 校验链：零长度视图/缓冲 → TypeError；min 0/非有限/负 → TypeError（0 明确 TypeError、
    // 缺省 1）；min > 视图长 → RangeError（按元素计）；errored → reject storedError。
    // 队列可满足 → 跨 chunk 拷贝填充 + commit；不足且 closeRequested 排空 → close steps；
    // 否则 pull-into 描述符入列（FIFO）→ flushPull → 源经 byobRequest.respond/enqueue 填充。
    function byobReadInto(view, options, request) {
      // net-api M4-S10：request（read-into request steps 形，tee 源侧 byob 读用）——给定时
      // resolve/reject 以 request steps 包装（chunk/close/error），兑现与 promise 形同点：
      // steps 在队列填充/commitDescriptor 的**同一同步步内**执行，投递微任务先于后续
      // flushPull → pull throw 的前向 reaction 排队（同 _zwReadRawSteps 注）。
      self._disturbed = true;
      var minRaw = (options != null && typeof options === 'object') ? options.min : undefined;
      var min = 1;
      if (minRaw !== undefined) {
        var mn = Number(minRaw);
        if (mn !== mn || mn === Infinity || mn === -Infinity || mn < 0) {
          return Promise.reject(new TypeError('invalid min'));
        }
        min = Math.floor(mn);
        if (min === 0) return Promise.reject(new TypeError('min must be non-zero'));
      }
      if (!view || typeof view.byteLength !== 'number' || view.byteLength === 0) {
        return Promise.reject(new TypeError('view must be non-zero-length'));
      }
      var isDataView = (typeof DataView === 'function') && view instanceof DataView;
      var elementSize = isDataView ? 1 : (view.BYTES_PER_ELEMENT || 1);
      var viewLen = isDataView ? view.byteLength : view.length;
      if (min > viewLen) return Promise.reject(new RangeError('min exceeds view length'));
      var minBytes = min * elementSize;
      var resD = null, rejD = null;
      var dPromise;
      if (request) {
        // net-api M4-S10：steps 形（tee 源侧）——request.chunkSteps/closeSteps/errorSteps。
        resD = function (r) { if (r.done) request.closeSteps(r.value); else request.chunkSteps(r.value); };
        rejD = function (e) { request.errorSteps(e); };
      } else {
        dPromise = new Promise(function (r, j) { resD = r; rejD = j; });
      }
      if (state === 'errored') { rejD(errorVal); return dPromise; }
      var total = 0;
      while (total < view.byteLength && queue.length > 0) {
        var entry = queue.shift();
        queueTotalSize -= entry.size;
        if (queueTotalSize < 0) queueTotalSize = 0;
        var srcR = entry.chunk instanceof Uint8Array ? entry.chunk : new Uint8Array(0);
        var take = Math.min(view.byteLength - total, srcR.length);
        try {
          new Uint8Array(view.buffer, view.byteOffset + total, take).set(srcR.subarray(0, take));
        } catch (_eFill2) {
          return Promise.reject(new TypeError('read: buffer fill failed (' + _eFill2.message + ')'));
        }
        total += take;
        if (take < srcR.length) {
          var rest = srcR.subarray(take);
          queue.unshift({ chunk: rest, size: rest.length });
          queueTotalSize += rest.length;
        }
      }
      function outView() {
        try {
          return isDataView ? new DataView(view.buffer, view.byteOffset, total)
                            : new view.constructor(view.buffer, view.byteOffset, total / elementSize);
        } catch (_eOut) { return new Uint8Array(0); }
      }
      if (total >= minBytes || total >= view.byteLength) {
        if (closeRequested && queue.length === 0) closeStream(false);
        resD({ value: outView(), done: false });
        return dPromise;
      }
      if (state === 'closed' || (closeRequested && queue.length === 0)) {
        if (closeRequested && queue.length === 0) closeStream(false);
        // net-api M4-S10：close steps 经 resD 包装（request 形 → closeSteps——缓冲回交视图）。
        resD({ value: total > 0 ? outView() : (isDataView ? new DataView(view.buffer, view.byteOffset, 0) : new view.constructor(view.buffer, view.byteOffset, 0)), done: true });
        return dPromise;
      }
      pullIntos.push({
        buffer: view.buffer, byteOffset: view.byteOffset, byteLength: view.byteLength,
        bytesFilled: total, min: minBytes, resolve: resD, reject: rejD,
        viewCtor: view.constructor, isDataView: isDataView, elementSize: elementSize
      });
      flushPull();
      return dPromise;
    }
    this._zwByobReadRaw = byobReadInto; // net-api M4-S10：tee 源侧 byob 读入口（see above）
    // net-api M4-S10：spec ReadableStreamDefaultReaderRead 的 **read request steps 形**——
    // tee 源默认读用。与 promise 形（_zwReadRaw + .then）的本质差异：chunk/close steps 在
    // dequeue **同一同步步内**执行——tee 的投递微任务此刻入队，先于同一同步体内后续
    // flushPull → pull throw → errorStream → closedPromise 前向的 reaction（promise 形的
    // .then 注册晚于整个同步体，投递反应恒排在前向之后——'errors in the source should
    // propagate to both branches' 双块队列源 'b' 投递被覆盖的根因）。
    this._zwReadRawSteps = function (readRequest) {
      self._disturbed = true;
      if (state === 'errored') { readRequest.errorSteps(errorVal); return; }
      if (queue.length > 0) {
        var entry = queue.shift();
        queueTotalSize -= entry.size;
        if (queueTotalSize < 0) queueTotalSize = 0;
        readRequest.chunkSteps(entry.chunk);
        if (closeRequested && queue.length === 0) closeStream();
        else flushPull();
        return;
      }
      if (state === 'closed') { readRequest.closeSteps(); return; }
      waiting.push({
        resolve: function (v) { if (v.done) readRequest.closeSteps(); else readRequest.chunkSteps(v.value); },
        reject: function (e) { readRequest.errorSteps(e); }
      });
      flushPull();
    };
    this.getReader = function (options) {
      // net-api M4-S1：byob reader 仅字节流（readable-byte-streams「getReader({mode:
      // 'byob'}) throws on non-bytes streams」面）。
      // net-api M4-S5：ReadableStreamReaderMode enum 转换（undefined/null → 缺省；非 'byob'
      // → TypeError——general「getReader() should only accept mode:undefined」面）。
      var modeVal = (options != null && typeof options === 'object') ? options.mode : undefined;
      if (modeVal !== undefined && modeVal !== null && String(modeVal) !== 'byob') {
        throw new TypeError("Failed to execute 'getReader' on 'ReadableStream': invalid mode.");
      }
      if (modeVal === 'byob' && !self._zwIsByteStream) {
        throw new TypeError('byob reader requires a ReadableStream with type "bytes"');
      }
      // net-api M4-S3：byob reader 专设（M4-S2 首版 waiting 路径 4.2GB 爆涨根因对策）——
      // ① 余量存 **reader 本地 `pending`**（不回 queue、不 unshift——queue/pull 双通道
      // 复制面消除）；② fill 全程 try/catch（detached buffer 等 view 构造 TypeError →
      // reject，探针不挂）；③ done 判定按**标记**（`_RS_DONE` 恒等或 done:true 非字节
      // 视图），填充路径永远不与 done 哨兵混淆；④ 零填充仅在 close 后（spec——read(view)
      // 最小填充契约：有数据必填 ≥1 字节）。
      // net-api M4-S3：byob 形态判定（mode==='byob' 且已过字节流守卫）。
      var byob = modeVal === 'byob';
      if (self._locked) throw new TypeError('Cannot get a Reader: ReadableStream is locked');
      self._locked = true;
      // net-api M4-S4：本 reader 的 closedPromise（waiter 登记——close/error/release 驱动 settle；
      // 取 reader 时流已 closed/errored → 立即 settle——spec GenericInitialize）。
      var resolveClosedP, rejectClosedP;
      var closedP = new Promise(function (res, rej) { resolveClosedP = res; rejectClosedP = rej; });
      var closedEntry = { resolve: resolveClosedP, reject: rejectClosedP };
      // net-api M4-S10：spec ReadableStreamError 步骤 7——closedPromise 拒绝时标记
      // [[PromiseIsHandled]] = true（页面未访问 .closed 的 reader 不产生 unhandled
      // rejection——tee 页 'errors in the source should propagate to both branches' 的
      // 'Unhandled rejection' 失败根因：分支 closedP 在测试挂 handler 前已拒绝）。
      closedEntry.markHandled = function () {
        try { closedP.catch(function () {}); } catch (_eMh) {}
      };
      if (state === 'closed') resolveClosedP();
      else if (state === 'errored') rejectClosedP(errorVal);
      else closedWaiters.push(closedEntry);
      if (byob) {
        // net-api M4-S7：BYOB read **pull-into 描述符化**（spec §4.5/§4.9.5 子集）——实现体
        // net-api M4-S10 上移为构造器级 byobReadInto（同实现服务 tee 源侧 byob 读），此处仅委托。
        return {
          read: byobReadInto,
          // net-api M4-S4：reader.cancel 走 cancelInternal（spec GenericCancel——close steps
          // given undefined 由 closeStream(byCancel) 面）。
          cancel: function (reason) { return cancelInternal(reason); },
          releaseLock: function () { releaseReaderLock(closedEntry, closedP); },
          get closed() { return closedP; }
        };
      }
      var readRaw = function () {
        self._disturbed = true; // net-api M2-S3：read 即 disturbed（spec §3.6）
        return new Promise(function (resolve, reject) {
          if (state === 'errored') { reject(errorVal); return; }
          if (queue.length > 0) {
            var entry = queue.shift();
            queueTotalSize -= entry.size;
            if (queueTotalSize < 0) queueTotalSize = 0;
            resolve(_rs_chunk(entry.chunk));
            // net-api M4-S4：closeRequested 排空 → 真关，否则 CallPullIfNeeded（spec PullSteps）。
            if (closeRequested && queue.length === 0) closeStream();
            else flushPull();
            return;
          }
          if (state === 'closed') { resolve(_RS_DONE); return; }
          waiting.push({ resolve: resolve, reject: reject });
          flushPull();
        });
      };
      return {
        // net-api M4-S4：author read() 结果 %Object.prototype% 形态（spec read-request 步骤
        // ——async-iterator/default-reader 页 [[Prototype]] 断言面）；内部消费（consume/pipe/
        // tee）走 _zwReadRaw（null 原型保留——M2-S3 then 投毒防线：Object.prototype.then 注入
        // 经 Promise resolution thenable adoption 劫持内部读循环）。
        read: function () {
          return readRaw().then(function (r) { return { value: r.value, done: r.done }; });
        },
        _zwReadRaw: readRaw,
        cancel: function (reason) { return cancelInternal(reason); },
        releaseLock: function () { releaseReaderLock(closedEntry, closedP); },
        get closed() { return closedP; }
      };
    };
    this.cancel = function (reason) {
      self._disturbed = true; // net-api M2-S3：cancel 即 disturbed（body.cancel 直调路径）
      if (self._locked) return Promise.reject(new TypeError('Cannot cancel: ReadableStream is locked'));
      return cancelInternal(reason);
    };
    this._zwRsBrand = true; // net-api M4-S1：pipeTo/pipeThrough brand 校验锚
    // net-api M4-S5：TransformStream SourcePull 背压观察锚（state/closeRequested/desired/readRequests
    // ——HasBackpressure = !ShouldCallPull 精确形）。
    this._zwRsProbe = function () {
      return { state: state, closeRequested: closeRequested, desired: hwm - queueTotalSize, readRequests: waiting.length, error: errorVal };
    };
    // net-api M4-S4：values(options) / @@asyncIterator(options)（spec §4.2.5 + WebIDL async
    // iterator 机制）——getReader 锁定（locked → 同步 TypeError）；[[OngoingPromise]] 串行
    //（前序 next/return 未决则链后——「return(); next() [no awaiting]」resolve 序锚）；next
    // close/error steps 均 release（exhaustive 迭代后可再 getReader）；return：preventCancel
    // 假 → cancelInternal（errored → reject storedError）+ **同步 release**（「return() should
    // unlock the stream synchronously」面）+ cancel promise 稳定后 fulfill {value, done:true}；
    // 迭代结果 %Object.prototype% 字面量（assert_iter_result [[Prototype]] 断言）。
    this.values = function (options) {
      var iteratorReader = self.getReader();
      var preventCancel = !!(options != null && typeof options === 'object' && options.preventCancel);
      var isDone = false;
      var ongoing = null;
      function queueOp(steps) {
        var p = ongoing !== null ? ongoing.then(steps, steps) : steps();
        ongoing = p.then(function () {}, function () {});
        return p;
      }
      function nextSteps() {
        if (isDone) return Promise.resolve({ value: undefined, done: true });
        return iteratorReader._zwReadRaw().then(function (r) {
          if (r.done) {
            isDone = true;
            try { iteratorReader.releaseLock(); } catch (_e) {}
            return { value: undefined, done: true };
          }
          return { value: r.value, done: false };
        }, function (e) {
          isDone = true;
          try { iteratorReader.releaseLock(); } catch (_e) {}
          throw e;
        });
      }
      function returnSteps(value) {
        if (isDone) return Promise.resolve({ value: value, done: true });
        isDone = true;
        var cancelP = preventCancel ? Promise.resolve() : cancelInternal(value);
        try { iteratorReader.releaseLock(); } catch (_e) {}
        return cancelP.then(function () { return { value: value, done: true }; });
      }
      var proto = {
        next: function () { return queueOp(nextSteps); },
        return: function (value) { return queueOp(function () { return returnSteps(value); }); }
      };
      try {
        // %AsyncIteratorPrototype% 继承（引擎 async generator 原型链两跳——values 页
        // prototype 链断言；grandparent 供 @@asyncIterator，proto 自身 own props 仅 next/return）。
        Object.setPrototypeOf(proto, Object.getPrototypeOf(Object.getPrototypeOf((async function* () {}).prototype)));
      } catch (_eAip) {
        Object.setPrototypeOf(proto, { [Symbol.asyncIterator]: function () { return this; } });
      }
      return Object.create(proto);
    };
    this[Symbol.asyncIterator] = this.values;
    // R2969 pipeTo(dest, options)：逐 chunk 从 self 读 → 写入 dest WritableStream，dest 完成（close）；
    // 任一侧 error → abort dest + reject。全程持 reader/writer 锁，完成后释放。
    // net-api M4-S4：StreamPipeOptions 完整化——① 成员序贯读取（preventAbort → preventCancel →
    // preventClose → signal）在**rejected-promise 路径**（throwing-options pipeTo 页——
    // promise_rejects_js 面）；② signal 校验（非 AbortSignal 形态 → TypeError reject，事件零触碰）；
    // ③ aborted signal → abortAlgorithm（preventAbort/preventCancel 门控 dest.abort + source
    // cancel，reject signal.reason——abort 页 pre-aborted 腿 + tee×无限源 pipeTo 的 OOM 根因面）；
    // ④ preventClose（done 后不关 dest）/ preventAbort（源 error 后不 abort dest）/ preventCancel
    // （dest error 后不 cancel 源）门控。
    // R2969 pipeTo(dest, options)：spec ReadableStreamPipeTo 适配实现。
    // net-api M4-S11：重做为 spec 收尾机——shutdown / shutdown-with-action / finalize 三段：
    // ① 动作经 **currentWrite 稳定 + 微任务跳**后才执行（构造即首查时 sink start 尚未
    //    started——跳后 started 置位，abort 动作的 sink.abort 同步落地先于 cancel 动作——
    //    'abort() should be called before cancel()' 事件序面；写由写完成驱动续泵）；
    // ② 动作拒绝优先于原错误（shutdown with action 的 rejection → finalize(newError)——
    //    'rejected cancel/abort promise' 双向传播面）；
    // ③ read-ahead 泵（背压门控：desiredSize ≤ 0/null 不读——spec Backpressure 约束；
    //    写完成续泵——'chunks should continue to be enqueued until the HWM is reached' 面）；
    // ④ finalize(err, errGiven) 显式区分「无错误」与「错误值 undefined」——undefined 拒绝
    //    面修复。
    this.pipeTo = function (dest, options) {
      // net-api M4-S1：brand/locked 校验（piping/general「brand」+「locked 不锁源」面
      // ——dest 已锁 → 先 reject 且**不锁** self）。
      if (!this || !this._zwRsBrand) return Promise.reject(new TypeError('pipeTo: Illegal invocation'));
      if (!dest || !dest._zwWsBrand) {
        return Promise.reject(new TypeError('pipeTo: destination is not a WritableStream'));
      }
      if (dest.locked) {
        return Promise.reject(new TypeError('pipeTo: destination WritableStream is locked'));
      }
      var opts;
      try { opts = _zwReadPipeOptions(options); }
      catch (eOpts) { return Promise.reject(eOpts); }
      if (opts.signal !== undefined &&
          (typeof opts.signal !== 'object' || opts.signal === null || typeof opts.signal.aborted !== 'boolean')) {
        return Promise.reject(new TypeError('pipeTo: signal must be an AbortSignal'));
      }
      return pipeToImpl(dest, opts);
    };
    function pipeToImpl(dest, opts) {
      var reader, writer;
      try { reader = self.getReader(); writer = dest.getWriter(); }
      catch (e) { return Promise.reject(e); }
      self._disturbed = true; // spec 步骤 11——pipeTo **同步**置 disturbed（response-stream-disturbed-by-pipe 面）
      var preventAbort = opts.preventAbort;
      var preventCancel = opts.preventCancel;
      var preventClose = opts.preventClose;
      var signal = opts.signal;

      var shuttingDown = false;
      var currentWrite = null; // 最近发出的写 promise——排队写按序完成，末位稳定即全体稳定
      var finished = false;
      var onAbort = null;
      var resolveP, rejectP;
      var promise = new Promise(function (res, rej) { resolveP = res; rejectP = rej; });

      function finalize(err, errGiven) {
        if (finished) return;
        finished = true;
        if (onAbort) { try { signal.removeEventListener('abort', onAbort); } catch (_eFin) {} }
        try { reader.releaseLock(); } catch (_eDr) {}
        try { writer.releaseLock(); } catch (_eDw) {}
        if (errGiven) rejectP(err); else resolveP(undefined);
      }
      function shutdownWithAction(action, originalError, originalGiven) {
        if (shuttingDown) return;
        shuttingDown = true;
        if (onAbort) { try { signal.removeEventListener('abort', onAbort); } catch (_eSw) {} onAbort = null; }
        function runAction() { return action ? action(originalError) : undefined; }
        // spec Shutdown with an action 步骤 3 门控——dest **writable 且非 closing** 才有
        // 「排在飞写」等待（微任务跳——跳后 started 置位、abort 动作的 sink.abort 同步先于
        // cancel——事件序面）；erroring/closed 时动作**同步**执行（erroring 的 abort 须抢在
        // start 微任务的 FinishErroring 前发起——wasAlreadyErroring 路径以 storedError 拒绝
        // =「Trying to abort a stream that is erroring will give the writable's error」面）。
        var wsp0 = dest._zwWsProbe ? dest._zwWsProbe() : null;
        var p;
        if (wsp0 && wsp0.state === 'writable' && !wsp0.closing) {
          p = Promise.resolve(currentWrite).then(runAction, runAction);
        } else {
          try { p = runAction(); } catch (_eSw2) { p = Promise.reject(_eSw2); }
        }
        Promise.resolve(p).then(function () { finalize(originalError, originalGiven); },
                                function (newError) { finalize(newError, true); });
      }
      function shutdown(err, errGiven) {
        if (shuttingDown) return;
        shuttingDown = true;
        if (onAbort) { try { signal.removeEventListener('abort', onAbort); } catch (_eSd) {} onAbort = null; }
        Promise.resolve(currentWrite).then(function () { finalize(err, errGiven); });
      }
      // spec 步骤 14 动作——dest writable 才 abort / source readable 才 cancel；否则 resolved。
      function abortAction(err) {
        var wsp3 = dest._zwWsProbe ? dest._zwWsProbe() : null;
        // net-api M4-S11：erroring 态同样执行 abort（spec WritableStreamAbort 断言 writable
        // 或 erroring——erroring 时 wasAlreadyErroring 路径不给 sink.abort、以 storedError
        // 拒绝 abort 请求——'Trying to abort a stream that is erroring will give the
        // writable's error' multiple-propagation 面）。
        if (wsp3 && (wsp3.state === 'writable' || wsp3.state === 'erroring')) return writer.abort(err);
        return Promise.resolve();
      }
      function cancelAction(err) {
        var rsp3 = self._zwRsProbe ? self._zwRsProbe() : null;
        if (rsp3 && rsp3.state === 'readable') return cancelInternal(err);
        return Promise.resolve();
      }
      // net-api M4-S11：signal 路径的 abort 动作 **writable-only**（spec 步骤 14 动作文本——
      // 非 writable 即 resolved；erroring dest 的 wasAlreadyErroring 拒绝不得劫持 signal.reason
      // ——'abort signal takes priority over errored writable' AbortError 面）。条件 1 路径
      // （源错误传播）的 abortAction 覆盖 erroring（见上）。
      function signalActions(err) {
        var aAbort = null, aCancel = null;
        if (!preventAbort) {
          try {
            var wspS = dest._zwWsProbe ? dest._zwWsProbe() : null;
            aAbort = (wspS && wspS.state === 'writable') ? writer.abort(err) : Promise.resolve();
          } catch (eGa1) { aAbort = Promise.reject(eGa1); }
        }
        if (!preventCancel) {
          try { aCancel = cancelAction(err); } catch (eGa2) { aCancel = Promise.reject(eGa2); }
        }
        var list = [];
        if (aAbort) list.push(aAbort);
        if (aCancel) list.push(aCancel);
        return Promise.all(list.map(function (pp) {
          return Promise.resolve(pp).then(function () { return null; }, function (eGa3) { return eGa3; });
        })).then(function (errs) {
          if (aAbort && errs[0]) throw errs[0];
          if (aCancel && errs[1]) throw errs[1];
        });
      }
      // spec WriterCloseWithErrorPropagation 动作（closing/closed → resolved；errored → 拒绝）。
      function writerCloseAction() {
        var wsp2 = dest._zwWsProbe ? dest._zwWsProbe() : null;
        if (wsp2 && (wsp2.closing || wsp2.state === 'closed')) return Promise.resolve();
        if (wsp2 && wsp2.state === 'errored') return Promise.reject(wsp2.error);
        return writer.close();
      }
      function abortAlgorithm() {
        // spec 步骤 14——signal.reason 收尾 + 动作集（先排空在读块）。
        shutdownWithAction(signalActions, signal.reason, true);
      }
      if (signal != null) {
        if (signal.aborted) { abortAlgorithm(); return promise; }
        if (typeof signal.addEventListener === 'function') {
          onAbort = abortAlgorithm;
          signal.addEventListener('abort', abortAlgorithm);
        }
      }
      // dest error 中途传播（back——cancel 动作 + dest stored error；cancel 拒绝优先）。
      try {
        writer.closed.then(function () {}, function (eWc) {
          shutdownWithAction(preventCancel ? null : cancelAction, eWc, true);
        });
      } catch (_eWcProbe) {}
      // spec 条件 1-4 首查——**经 shutdown-with-action**（动作拒绝优先于原错误传播；微任务
      // 跳后动作才执行）。
      var rsProbe = self._zwRsProbe ? self._zwRsProbe() : null;
      var wsProbe = dest._zwWsProbe ? dest._zwWsProbe() : null;
      if (rsProbe && rsProbe.state === 'errored') {
        shutdownWithAction(preventAbort ? null : abortAction, rsProbe.error, true);
        return promise;
      }
      if (wsProbe && wsProbe.state === 'errored') {
        shutdownWithAction(preventCancel ? null : cancelAction, wsProbe.error, true);
        return promise;
      }
      if (rsProbe && rsProbe.state === 'closed') {
        if (preventClose) { shutdown(undefined, false); return promise; }
        shutdownWithAction(writerCloseAction, undefined, false);
        return promise;
      }
      if (wsProbe && (wsProbe.state === 'closed' || wsProbe.state === 'erroring' || wsProbe.closing)) {
        var destClosedErr = (wsProbe.state === 'erroring') ? wsProbe.error : new TypeError('Destination writable stream is closed or closing');
        shutdownWithAction(preventCancel ? null : cancelAction, destClosedErr, true);
        return promise;
      }
      // net-api M4-S11：源状态监视——read-ahead 泵存在**无在飞读窗口**（背压门控停读），
      // 源 close/error 不能只靠读兑现暴露：reader.closed 兑现 → 条件 3（关 dest）；
      // 拒绝 → 条件 1（abort dest）。**pendingRead 在飞门控**：读在飞时其兑现/拒绝路径
      //（r.done / read 拒绝）已覆盖同一传播，监视器不得抢先关流（背压窗口读出的 chunk
      // 会被 shuttingDown 门控丢弃——'does not desire chunks, but then does' 丢 b 根因）。
      var pendingRead = null;
      var activeOps = 0; // net-api M4-S11：在飞管线索（读→写链）计数——源 close 监视仅在静默时动作
      try {
        reader.closed.then(function () {
          if (activeOps > 0 || shuttingDown || finished) return;
          if (!preventClose) shutdownWithAction(writerCloseAction, undefined, false);
          else shutdown(undefined, false);
        }, function (eRs) {
          if (activeOps > 0 && pendingRead) return; // 在飞读的拒绝路径自会 sourceErrored
          shutdownWithAction(preventAbort ? null : abortAction, eRs, true);
        });
      } catch (_eRsProbe) {}
      // net-api M4-S11：背压解除唤醒——页自持 writer 的写在飞时 desiredSize ≤ 0，其完成
      // （updateBackpressure → ready 兑现）须驱动泵续读（flow-control 'does not desire
      // chunks, but then does' 挂死根因）。ready 每次转移换新 promise，按身份判重避免
      // 已兑现 promise 上的自旋。
      var lastReady = null;
      function watchReady() {
        if (shuttingDown || finished) return;
        var r;
        try { r = writer.ready; } catch (_eWr) { return; }
        if (r === lastReady) return;
        lastReady = r;
        r.then(function () { pump(); watchReady(); }, function () {});
      }
      watchReady();
      // read-ahead 泵——背压门控（desiredSize ≤ 0/null 不读——spec Backpressure 约束）；
      // 写完成续泵（背压解除驱动）；源 done → WriterCloseWithErrorPropagation 动作。
      function pump() {
        if (shuttingDown || finished) return;
        // net-api M4-S28：读在飞时不再入泵（spec pipeTo 逐 chunk read→write→背压检查
        // 串行——watchReady/write 完成路径重入会并发双读，读请求超额消费背压余量——
        // flow-control「desires more chunks before finishing」面：desired 2 被双读打穿）。
        if (pendingRead) return;
        var desired;
        try { desired = writer.desiredSize; } catch (_eDs) { return; }
        if (desired !== null && desired <= 0) return;
        // net-api M4-S4：内部消费走 _zwReadRaw（null 原型——then 投毒防线，见 getReader 注）。
        activeOps++;
        pendingRead = reader._zwReadRaw();
        pendingRead.then(function (r) {
          pendingRead = null;
          if (shuttingDown || finished) { activeOps--; return; }
          if (r.done) {
            activeOps--;
            if (!preventClose) shutdownWithAction(writerCloseAction, undefined, false);
            else shutdown(undefined, false);
            return;
          }
          var w;
          try { w = writer.write(r.value); } catch (eW3) {
            activeOps--;
            shutdownWithAction(preventCancel ? null : cancelAction, eW3, true);
            return;
          }
          currentWrite = w;
          w.then(function () { activeOps--; pump(); }, function (e) {
            activeOps--;
            shutdownWithAction(preventCancel ? null : cancelAction, e, true);
          });
          pump(); // read-ahead（仍按 desired 门控）
        }, function (e) {
          pendingRead = null;
          activeOps--;
          shutdownWithAction(preventAbort ? null : abortAction, e, true);
        });
      }
      pump();
      return promise;
    }
    // R2969 pipeThrough({writable, readable})：fire-and-forget pipeTo(transform.writable)，返
    // transform.readable。net-api M4-S11：成员读取序按 WebIDL ReadableWritablePair 定义序——
    // **readable 先于 writable**（brand-check readable 失败时 writable getter 不得被触；
    // 'should throw if readable/writable getters throw' 断言 readable 错误先抛面）+ 成员
    // 品牌校验（非 ReadableStream/WritableStream → TypeError）+ options 读取**之后**复核
    // writable 锁定（option getter 抓 writer → TypeError 面）。
    this.pipeThrough = function (transform, options) {
      if (!this || !this._zwRsBrand) throw new TypeError('pipeThrough: Illegal invocation');
      if (self._locked) throw new TypeError('pipeThrough: ReadableStream is locked');
      var readable = transform ? transform.readable : undefined; // getter 恰触一次
      if (!readable || !readable._zwRsBrand) throw new TypeError('pipeThrough: readable is not a ReadableStream');
      var writable = transform.writable; // getter 恰触一次
      if (!writable || !writable._zwWsBrand) throw new TypeError('pipeThrough: writable is not a WritableStream');
      var opts = _zwReadPipeOptions(options);
      if (opts.signal !== undefined &&
          (typeof opts.signal !== 'object' || opts.signal === null || typeof opts.signal.aborted !== 'boolean')) {
        throw new TypeError('pipeThrough: signal must be an AbortSignal');
      }
      if (writable.locked) throw new TypeError('pipeThrough: writable is locked');
      var p = pipeToImpl(writable, opts);
      try { p.catch(function () {}); } catch (_ePt) {} // spec：promise.[[PromiseIsHandled]] = true
      return readable;
    };
    // net-api M4-S8/M4-S11：实例闭包别名（prototype 委托锚——`ReadableStream.prototype.X`
    // 可被页面捕获后 .call(rs) 调用）。**必须在全部方法定义之后赋值**——别名捕获 `this.X`
    // 时原型委托已存在，若先于方法定义赋值会捕获到委托自身（proto 委托 → 别名 → 委托
    // 自递归——'pipeTo must check the brand of its WritableStream argument' 的 Maximum
    // call stack 根因）。
    this._zwGetReaderFn = this.getReader;
    this._zwCancelFn = this.cancel;
    this._zwValuesFn = this.values;
    this._zwTeeFn = this.tee;
    this._zwPipeToFn = this.pipeTo;
    this._zwPipeThroughFn = this.pipeThrough;
    // net-api M4-S10：R2971 tee() 重做为 spec ReadableByteStreamTee 结构（spec §4.9.1——
    // 替换原 buffer-based 简化模型：共享 append-only buffer + 分支 pull 取数无法表达字节流
    // tee 的读计数/BYOB 前向/readAgain 语义）。核心件：
    // ① reading 串行门 + readAgainForBranch1/2（spec pull1/pull2Algorithm 步骤 1——读在飞时
    //    分支 pull 只置旗标；chunk/close steps 微任务**开头**复位旗标、末尾按旗标续拉——
    //    「should only pull enough to fill the emptiest queue」pull 计数面）；
    // ② pullWithDefaultReader——源默认读（read request steps 形 _zwReadRawSteps——steps 在
    //    dequeue 同步步内取到，投递微任务先于后续 pull throw 的前向 reaction 入队），chunk
    //    steps 微任务内双分支 enqueue（字节源 clone——M4-S9 克隆面）、close steps 双分支关流
    //    （分支控制器 close 即缓冲回交——spec respond(0) 等价）+ cancelPromise 兑现；
    // ③ pullWithBYOBReader(view, forBranch2)——**源侧 BYOB 读**：分支 byobRequest 视图直入
    //    self._zwByobReadRaw（源 pull-into 描述符即分支缓冲——源 enqueue/respond 物理写入
    //    分支缓冲，源 pull 的 byobRequest.view 非空）；chunk steps：对 byob 分支
    //    respondWithNewView(chunk)（字节物理已在分支缓冲——新视图回提交）、对另一分支
    //    enqueue 克隆；close steps：双分支关流（缓冲回交）+ cancelPromise 兑现；
    // ④ 错误传播维持 M4-S8 reader.closed rejection 前向（单 reader 不切换——spec 双 reader
    //    切换 + forwardReaderError current-reader 判别在本实现不需要：无切换即无旧 reader
    //    假拒绝）；composite cancel 维持 M4-S8。
    this.tee = function () {
      if (self._locked) throw new TypeError('Cannot tee: ReadableStream is locked');
      var reader = self.getReader();
      var reading = false;                    // spec：源读（默认/BYOB）在飞旗标
      var readAgainForBranch1 = false;
      var readAgainForBranch2 = false;
      var canceled1 = false, canceled2 = false;
      var teeReason1, teeReason2;
      var teeCancelResolve = null;
      var teeCancelPromise = new Promise(function (r) { teeCancelResolve = r; });
      function teeCancelBranch(which, reason) {
        if (which === 1) { canceled1 = true; teeReason1 = reason; }
        else { canceled2 = true; teeReason2 = reason; }
        if (canceled1 && canceled2) {
          var p = self._doCancel([teeReason1, teeReason2]);
          teeCancelResolve(p);
          return p;
        }
        return teeCancelPromise;
      }
      function resolveCancelIfAny() {
        // spec chunk/close steps 末步——单侧未取消即兑现 cancelPromise（'canceling branch1
        // should finish when branch2 reads until end of stream' 挂账腿解锁面）。
        if (!canceled1 || !canceled2) teeCancelResolve(Promise.resolve());
      }
      var _zwRsCtorForTee = self.constructor; // 捕获原构造器（页面改全局后 tee 不受扰面）
      var teeIsByte = !!self._zwIsByteStream; // net-api M4-S9：字节源 tee → 分支字节流身份
      function makeBranch(bi) {
        var teeCtl = null; // start 构造期内执行——经外捕获后再挂（branch 自引用 TDZ 面）
        var desc = {
          start: function (controller) {
            teeCtl = controller;
          },
          pull: function () { return pullForBranch(bi); },
          // net-api M4-S10：分支 cancel 算法 = tee composite cancel（spec cancel1/2Algorithm
          // 即分支 controller 的 [[cancelAlgorithm]]——stream.cancel 与 reader.cancel 同经
          // cancelInternal→_doCancel→srcCancel 单路走 teeCancelBranch，取代 M4-S8 的
          // stream.cancel 实例包装——reader.cancel 也要复合取消语义的腿面）。
          cancel: function (reason) { return teeCancelBranch(bi, reason); }
        };
        if (teeIsByte) desc.type = 'bytes'; // 分支字节流身份——byob reader/pull-into 面
        var branch = new _zwRsCtorForTee(desc);
        branch._teeController = teeCtl;
        return branch;
      }
      function cloneChunk(out) {
        // spec CloneAsUint8Array 子集（字节源 tee 双分支独立缓冲——M4-S9 面）。
        if (!(teeIsByte && out instanceof Uint8Array)) return out;
        var cl = new Uint8Array(out.byteLength);
        cl.set(out);
        return cl;
      }
      function clonePreserveView(out) {
        // net-api M4-S10：spec 分支1 侧等价形——源 chunk 经 TransferArrayBuffer 换**新
        // ArrayBuffer 对象**但保留同一内存/byteOffset/length（'reading an array with a byte
        // offset should clone correctly' 断言 view1.byteOffset === 2 + buffer 身份换新）。
        // 本实现无 buffer transfer——新缓冲全量拷贝后按原偏移/长度重建视图。
        if (!(teeIsByte && out instanceof Uint8Array)) return out;
        try {
          var buf2 = new ArrayBuffer(out.buffer.byteLength);
          new Uint8Array(buf2).set(new Uint8Array(out.buffer));
          return new out.constructor(buf2, out.byteOffset, out.length);
        } catch (_eCpv) { return cloneChunk(out); }
      }
      function tryCloseCtl(ctl) {
        // spec `! ReadableByteStreamControllerClose`——已关/closing 静默（本实现 close 抛
        // TypeError，须吞）；字节分支有 pending pull-into 时 close 即缓冲回交
        //（spec close steps 的 respond(0)/respondWithNewView 等价）。
        try { ctl.close(); } catch (_eTc) {}
      }
      function tryEnqueueCtl(ctl, chunk) {
        // spec `! Enqueue`——closeRequested/非 readable 静默。
        try { ctl.enqueue(chunk); } catch (_eTe) {}
      }
      function respondToByobCtl(ctl, chunk) {
        // spec chunk steps——respondWithNewView(chunk)（chunk 即源读兑现的分支缓冲视图）。
        try {
          var req = ctl.byobRequest;
          if (req) req.respondWithNewView(chunk);
        } catch (_eRb) {
          try { ctl.error(_eRb); } catch (_eRb2) {}
        }
      }
      var b1 = makeBranch(1), b2 = makeBranch(2);
      var b1ctl = b1._teeController, b2ctl = b2._teeController;
      // ---- spec pullWithDefaultReader（源默认读路径）----
      function pullWithDefaultReader() {
        // net-api M4-S10：源默认读走 read request steps 形（_zwReadRawSteps）——chunk/close
        // steps 在 dequeue 同步步内取到、其投递微任务（spec「queue a microtask」——错误检测
        // 时序：成功读的投递须排在异步错误前向之后排队、但**先于**已入队的前向 reaction 执行）
        // 先于同一同步体内后续 pull throw 的前向入队。
        self._zwReadRawSteps({
          chunkSteps: function (chunk1) {
            Promise.resolve().then(function () {
              readAgainForBranch1 = false;
              readAgainForBranch2 = false;
              var chunk2 = (!canceled1 && !canceled2) ? cloneChunk(chunk1) : chunk1;
              // net-api M4-S9：字节源 tee **两分支均收克隆**（spec 靠 TransferArrayBuffer 让
              // branch1 的 chunk 换新缓冲对象满足 buffer 身份断言；本实现无 buffer transfer，
              // branch1 用换新缓冲保偏移视图、branch2 用内容克隆——'chunks should be cloned
              // for each branch' + 'byte offset should clone correctly' 双断言面）。
              if (teeIsByte && !canceled1) chunk1 = clonePreserveView(chunk1);
              if (!canceled1) tryEnqueueCtl(b1ctl, chunk1);
              if (!canceled2) tryEnqueueCtl(b2ctl, chunk2);
              reading = false;
              if (readAgainForBranch1) pullForBranch(1);
              else if (readAgainForBranch2) pullForBranch(2);
            });
          },
          closeSteps: function () {
            reading = false;
            if (!canceled1) tryCloseCtl(b1ctl);
            if (!canceled2) tryCloseCtl(b2ctl);
            resolveCancelIfAny();
          },
          errorSteps: function () {
            reading = false; // spec errorSteps——错误经 reader.closed 拒绝前向（M4-S8）双分支 error
          }
        });
      }
      // ---- spec pullWithBYOBReader（源侧 BYOB 读路径——M4-S10 专设）----
      function pullWithBYOBReader(view, forBranch2) {
        // net-api M4-S10：源侧 BYOB 读走 read-into request steps 形（byobReadInto 第三参）——
        // steps 在队列填充/commit 同步步内取到，投递微任务先于后续 pull throw 的前向入队。
        self._zwByobReadRaw(view, { min: 1 }, {
          chunkSteps: function (chunk) {
            Promise.resolve().then(function () {
              var byobCtl = forBranch2 ? b2ctl : b1ctl;
              var otherCtl = forBranch2 ? b1ctl : b2ctl;
              var byobCanceled = forBranch2 ? canceled2 : canceled1;
              var otherCanceled = forBranch2 ? canceled1 : canceled2;
              readAgainForBranch1 = false;
              readAgainForBranch2 = false;
              if (!otherCanceled) {
                var cloned = cloneChunk(chunk);
                if (!byobCanceled) respondToByobCtl(byobCtl, chunk);
                tryEnqueueCtl(otherCtl, cloned);
              } else if (!byobCanceled) {
                respondToByobCtl(byobCtl, chunk);
              }
              reading = false;
              if (readAgainForBranch1) pullForBranch(1);
              else if (readAgainForBranch2) pullForBranch(2);
            });
          },
          closeSteps: function () {
            // close steps——读入请求以 close steps 兑现（byobReadInto 的 closed/
            // closeRequested 路径或源 closeStream 已交还缓冲）；双分支关流。
            var byobCtl = forBranch2 ? b2ctl : b1ctl;
            var otherCtl = forBranch2 ? b1ctl : b2ctl;
            var byobCanceled = forBranch2 ? canceled2 : canceled1;
            var otherCanceled = forBranch2 ? canceled1 : canceled2;
            reading = false;
            if (!byobCanceled) tryCloseCtl(byobCtl);
            if (!otherCanceled) tryCloseCtl(otherCtl);
            resolveCancelIfAny();
          },
          errorSteps: function () {
            reading = false; // spec errorSteps——错误经 reader.closed 拒绝前向双分支 error
          }
        });
      }
      // ---- spec pull1/pull2Algorithm（分支 pull 入口）----
      function pullForBranch(bi) {
        if (reading) {
          if (bi === 2) readAgainForBranch2 = true;
          else readAgainForBranch1 = true;
          return; // spec：resolved promise（分支 flushPull 同步完成等价）
        }
        reading = true;
        var byobRequest = null;
        if (teeIsByte) {
          var ctl = (bi === 2) ? b2ctl : b1ctl;
          try { byobRequest = ctl ? ctl.byobRequest : null; } catch (_eBr) { byobRequest = null; }
        }
        if (byobRequest == null) pullWithDefaultReader();
        else pullWithBYOBReader(byobRequest.view, bi === 2);
      }
      // net-api M4-S8：spec tee 步骤 19——reader.closedPromise rejection → **立即** error 两分支
      //（不待分支 pull；单 reader 不切换——本实现无旧 reader 假拒绝面）。
      try {
        reader.closed.then(function () {}, function (eTee) {
          try {
            try { b1ctl.error(eTee); } catch (_eTc2) {}
            try { b2ctl.error(eTee); } catch (_eTc3) {}
          } catch (_eTc4) {}
          try { teeCancelResolve(Promise.resolve()); } catch (_eTc5) {}
        });
      } catch (_eTeeFwd) {}
      // net-api M4-S10：分支 cancel 包装已撤——cancel 语义由 desc.cancel（= teeCancelBranch）
      // 单路承载（stream.cancel / reader.cancel 同经 cancelInternal → _doCancel → srcCancel）。
      return [b1, b2];
    };
    // start：spec SetUp——start 同步调用（**同步抛错冒出构造器**——§4.2.3「Any thrown exceptions
    // will be re-thrown by the ReadableStream() constructor」）；startPromise（resolved with
    // startResult）**fulfillment 后** started + CallPullIfNeeded（恒为微任务——「cancelling before
    // start finishes should prevent pull」面 + recordingReadableStream 构造后同步补 events 字段
    // 的面）；thenable start 拒绝 → error 流。异步拉链（pullAgain 微任务重拉）+ read 请求触发
    //（PullSteps）——「next(); return() [no awaiting]」timesPulled===2 语义锚。
    if (typeof srcStart === 'function') {
      var startResult = srcStart.call(source, controller);
      Promise.resolve(startResult).then(function () {
        started = true;
        flushPull();
      }, function (eStart2) { errorStream(eStart2); });
    } else {
      Promise.resolve().then(function () { started = true; flushPull(); });
    }
  };
  // net-api M4-S5：locked accessor 上移 prototype（general「Subclassing ReadableStream」面
  // ——Object.getOwnPropertyDescriptor(ReadableStream.prototype, 'locked').get 品牌访问 +
  // subclass 实例共享 getter）。
  Object.defineProperty(globalThis.ReadableStream.prototype, 'locked', {
    get: function () { return this._locked; },
    enumerable: true, configurable: true
  });
  // net-api M4-S8：prototype 方法委托（页面捕获 `prototype.getReader` 后 .call(rs) 面——
  // 方法本为实例闭包，委托转发到实例别名）。
  (function () {
    var proto = globalThis.ReadableStream.prototype;
    var methods = { getReader: '_zwGetReaderFn', cancel: '_zwCancelFn', values: '_zwValuesFn', tee: '_zwTeeFn', pipeTo: '_zwPipeToFn', pipeThrough: '_zwPipeThroughFn' };
    for (var m in methods) {
      (function (name, slot) {
        Object.defineProperty(proto, name, {
          // net-api M4-S11：非流 this（prototype 方法被 .call/apply/bind 到裸对象）——实例
          // 别名缺失时不得崩溃：pipeTo 按品牌语义返回 rejected promise，其余同步抛
          // TypeError（'pipeTo must check the brand of its ReadableStream this value' 面）。
          value: function () {
            var fn = this[slot];
            if (typeof fn !== 'function') {
              if (slot === '_zwPipeToFn') return Promise.reject(new TypeError('pipeTo: Illegal invocation'));
              throw new TypeError('Illegal invocation');
            }
            return fn.apply(this, arguments);
          },
          writable: true, enumerable: true, configurable: true
        });
      })(m, methods[m]);
    }
  })();
  // fetch 响应体字符串 → ReadableStream：单 UTF-8 Uint8Array chunk 后 close（headless finite-body 模型，
  // 整体 body 已就绪）。复用 _zw_utf8_encode；空 body → 直接 close（零 chunk）。定义在 part01 _makeResponse
  // 之前调用（runtime），ReadableStream（part02）+ _zw_utf8_encode（part02）在 IIFE 同作用域已就绪。
  function _bodyToStream(src) {
    // R3021：src 可为 Uint8Array（二进制 response body）→ 直接 enqueue 字节；字符串 → UTF-8 编码 Uint8Array chunk。
    // net-api M4-S5：source **null 原型**（stream-safe-creation 页——Object.prototype.type/start
    // 投毒不得触及内部流创建；自有 start 遮蔽 + 无自有 type → 构造器成员读取免疫）。
    var isBytes = src instanceof Uint8Array;
    var sourceObj = {
      start: function (controller) {
        if (isBytes) {
          if (src.length > 0) controller.enqueue(src);
        } else {
          var bodyText = src || '';
          if (bodyText) {
            var bytes = _zw_utf8_encode(bodyText);
            var arr = new Uint8Array(bytes.length);
            for (var k = 0; k < bytes.length; k++) arr[k] = bytes[k];
            controller.enqueue(arr);
          }
        }
        controller.close();
      }
    };
    if (typeof Object.setPrototypeOf === 'function') Object.setPrototypeOf(sourceObj, null);
    return new ReadableStream(sourceObj);
  }

  // net-api M4-S5：WritableStreamDefaultController / WritableStreamDefaultWriter 全局类
  //（spec——controller 不可构造（构造即 TypeError）；writer 构造须 WritableStream（brand）
  // 且未锁定 → SetUp（getWriter 同一工厂：locked → TypeError））。
  function WritableStreamDefaultController() {
    throw new TypeError('Illegal constructor');
  }
  function WritableStreamDefaultWriter(stream) {
    if (!stream || !stream._zwWsBrand) {
      throw new TypeError("Failed to construct 'WritableStreamDefaultWriter': stream must be a WritableStream.");
    }
    return stream.getWriter(); // locked → getWriter TypeError；未锁 → 标准 writer
  }
  globalThis.WritableStreamDefaultController = globalThis.WritableStreamDefaultController || WritableStreamDefaultController;
  globalThis.WritableStreamDefaultWriter = globalThis.WritableStreamDefaultWriter || WritableStreamDefaultWriter;

  // ── P1a WritableStream（Streams API write 侧，R2969）──
  // ReadableStream 的写入配对（pipeTo 目标 / TransformStream 写侧 / 自定义 sink）。控制器模型：
  // underlyingSink {start, write(chunk,controller), close, abort} + WritableStreamDefaultController
  // {error}。默认 writer：write(chunk)→Promise（sink.write 完成时 resolve）/ close()→Promise /
  // abort(reason)→Promise / releaseLock / closed→Promise / ready→Promise / desiredSize；locked 守卫。
  // headless 背压近似（desiredSize 恒 1，ready 立即 resolve——无真 highWaterMark 队列压力），write 串行化
  //（每个 write 自带 Promise 链，sink.write 异步则 await）。WritableStream 自身错误 → 拒绝 pending write +
  // reject closed。
  // R3010：背压 spec 化——strategy {highWaterMark, size} 计量 queueTotalSize，desiredSize = hwm - queueTotalSize，
  // writer.ready 在 desiredSize<=0 时挂起（背压门控）、>0 时 resolve（背压释放），生产者可 await ready 节流。
  globalThis.WritableStream = globalThis.WritableStream || function WritableStream(underlyingSink, _strategy) {
    if (!(this instanceof WritableStream)) return new WritableStream(underlyingSink, _strategy);
    // net-api M4-S4：strategy dictionary 转换 + ExtractHighWaterMark（default 1）——size 非函数
    // TypeError（conversion 阶段）先于 hwm RangeError（「invalid size beats invalid highWaterMark」
    // ——WebIDL 成员定义序转换序）。
    var strat = _zwStrategyDict(_strategy);
    // net-api M4-S5：WebIDL object 转换（readable 同口径——null/原语 → TypeError）。
    if (underlyingSink === null || (underlyingSink !== undefined &&
        typeof underlyingSink !== 'object' && typeof underlyingSink !== 'function')) {
      throw new TypeError("Failed to construct 'WritableStream': underlyingSink must be an object.");
    }
    var sink = underlyingSink === undefined ? {} : underlyingSink;
    // net-api M4-S5：UnderlyingSink type 成员保留面——存在即 RangeError（spec 构造步骤 3）。
    if (sink.type !== undefined) {
      throw new RangeError("Failed to construct 'WritableStream': type is reserved.");
    }
    var self = this;
    this._locked = false;
    this._zwWsBrand = true; // net-api M4-S1：pipeTo/pipeThrough brand 校验锚
    // ── net-api M4-S6：spec §5.2/5.4/5.5 状态机（slot 对应 [[state]]/[[queue]]/
    // [[writeRequests]]/[[inFlightWriteRequest]]/[[closeRequest]]/[[inFlightCloseRequest]]/
    // [[pendingAbortRequest]]/[[backpressure]]/[[storedError]]）──
    var state = 'writable';          // writable | erroring | closed | errored
    var storedError = undefined;
    var queue = [];                  // {chunk, size}（close 哨兵 size 0）
    var queueTotalSize = 0;
    var closeRequest = undefined;    // deferred（writer.close 的 promise）
    var inFlightCloseRequest = undefined;
    var inFlightWriteRequest = undefined;
    var writeRequests = [];          // {resolve, reject, size}
    var pendingAbortRequest = undefined;
    var started = false;
    var backpressure = false;
    var hwm = _zwExtractHwm(strat.hwmValue, 1);
    var sizeFn = strat.size !== undefined ? strat.size : null;
    var sinkWriteFn = (typeof sink.write === 'function') ? sink.write : null;
    var sinkCloseFn = (typeof sink.close === 'function') ? sink.close : null;
    var sinkAbortFn = (typeof sink.abort === 'function') ? sink.abort : null;
    var closeAlgorithmLive = true;
    var CLOSE_SENTINEL = { __zwCloseSentinel: true };
    // deferred 工厂（promise + resolve/reject + pending 标记——writer closed/ready 用）。
    function mkDeferred() {
      var d = { pending: true, resolve: null, reject: null, promise: null };
      d.promise = new Promise(function (r, j) { d.resolve = r; d.reject = j; });
      return d;
    }
    function makeRejected(e) {
      var p = new Promise(function (_r, rej) { rej(e); });
      try { p.then(function () {}, function () {}); } catch (_e) {} // 已发布拒绝标记 handled
      return p;
    }
    var writerClosed = null, writerReady = null; // deferred | null（release 后仍留 rejected 形）
    function ensureReadyRejected(e) {
      // spec EnsureReadyPromiseRejected——pending → reject；已 settle → 替换；标记 handled。
      if (!writerReady) { writerReady = makeRejected(e); return; }
      if (writerReady.pending) { writerReady.reject(e); writerReady.pending = false; }
      else writerReady = { pending: false, promise: makeRejected(e) };
    }
    function ensureClosedRejected(e) {
      if (!writerClosed) { writerClosed = makeRejected(e); return; }
      if (writerClosed.pending) { writerClosed.reject(e); writerClosed.pending = false; }
      else writerClosed = { pending: false, promise: makeRejected(e) };
    }

    function errorSteps() { queue = []; queueTotalSize = 0; } // spec [[ErrorSteps]]
    function abortSteps(reason) {
      // spec [[AbortSteps]]——abort 算法。
      if (!sinkAbortFn) return Promise.resolve();
      try { return Promise.resolve(sinkAbortFn.call(sink, reason)); }
      catch (eAb) { return Promise.reject(eAb); }
    }
    function clearAlgorithms() {
      // spec ClearAlgorithms——算法引用清除（close 算法置否 + write/abort 引用清空）。
      closeAlgorithmLive = false;
      sinkWriteFn = null;
      sinkCloseFn = null;
      sinkAbortFn = null;
    }
    function rejectCloseAndClosedIfNeeded() {
      // spec RejectCloseAndClosedPromiseIfNeeded——state 须 errored。
      if (closeRequest !== undefined) {
        closeRequest.reject(storedError);
        closeRequest = undefined;
      }
      if (writerClosed) {
        if (writerClosed.pending) { writerClosed.reject(storedError); writerClosed.pending = false; }
        else writerClosed = { pending: false, promise: makeRejected(storedError) };
      }
    }
    function hasOperationMarkedInFlight() {
      return inFlightWriteRequest !== undefined || inFlightCloseRequest !== undefined;
    }
    function dealWithRejection(error) {
      // spec DealWithRejection——writable → StartErroring；erroring → FinishErroring。
      if (state === 'writable') startErroring(error);
      else finishErroring();
    }
    function startErroring(reason) {
      // spec StartErroring——writable → erroring + ready 拒绝；无 in-flight 且已 start → 收尾。
      storedError = reason;
      state = 'erroring';
      if (writerReady) ensureReadyRejected(reason);
      if (!hasOperationMarkedInFlight() && started) finishErroring();
    }
    function finishErroring() {
      // spec FinishErroring——errored + 拒绝 queued 写 + abort 请求收尾。
      state = 'errored';
      errorSteps();
      for (var i = 0; i < writeRequests.length; i++) writeRequests[i].reject(storedError);
      writeRequests = [];
      if (pendingAbortRequest === undefined) { rejectCloseAndClosedIfNeeded(); return; }
      var abortRequest = pendingAbortRequest;
      pendingAbortRequest = undefined;
      if (abortRequest.wasAlreadyErroring) {
        abortRequest.reject(storedError);
        rejectCloseAndClosedIfNeeded();
        return;
      }
      abortSteps(abortRequest.reason).then(function () {
        abortRequest.resolve(undefined);
        rejectCloseAndClosedIfNeeded();
      }, function (rAb) {
        abortRequest.reject(rAb);
        rejectCloseAndClosedIfNeeded();
      });
    }
    function desiredSize() {
      if (state === 'errored' || state === 'erroring') return null;
      if (state === 'closed') return 0;
      return hwm - queueTotalSize;
    }
    function getBackpressure() { return desiredSize() <= 0; }
    function updateBackpressure(bp) {
      // spec UpdateBackpressure——writer 存在且翻转 → ready 挂起/释放。
      if (writerReady && bp !== backpressure) {
        if (bp) writerReady = mkDeferred();
        else if (writerReady.pending) { var dR = writerReady; writerReady = { pending: false, promise: Promise.resolve() }; dR.resolve(undefined); }
      }
      backpressure = bp;
    }
    function closeQueuedOrInFlight() {
      return closeRequest !== undefined || inFlightCloseRequest !== undefined;
    }
    function finishInFlightWrite() {
      // spec FinishInFlightWrite——resolve in-flight 请求 + **DequeueValue**（队首出队——
      // 缺此步队列不排空，close 哨兵永不推进/同一 chunk 重写死循环）+ 背压更新 + 续推。
      inFlightWriteRequest.resolve(undefined);
      inFlightWriteRequest = undefined;
      var done = queue.shift();
      if (done) {
        queueTotalSize -= done.size;
        if (queueTotalSize < 0) queueTotalSize = 0;
      }
      if (!closeQueuedOrInFlight() && state === 'writable') updateBackpressure(getBackpressure());
      advanceQueueIfNeeded();
    }
    function finishInFlightWriteWithError(error) {
      inFlightWriteRequest.reject(error);
      inFlightWriteRequest = undefined;
      if (state === 'writable') clearAlgorithms(); // spec——仅 writable 清算法（erroring 保留 abort 算法）
      dealWithRejection(error);
    }
    function finishInFlightClose() {
      inFlightCloseRequest.resolve(undefined);
      inFlightCloseRequest = undefined;
      if (state === 'erroring') {
        // spec 步骤 5——erroring → storedError 清除 + **abort 请求 resolve**（close 成功即
        // abort 满足；resolve 序：closeRequest 先于 abort 请求——「close before abort」事件序面）。
        storedError = undefined;
        if (pendingAbortRequest !== undefined) {
          pendingAbortRequest.resolve(undefined);
          pendingAbortRequest = undefined;
        }
      }
      state = 'closed';
      queue = []; queueTotalSize = 0;
      if (writerClosed) { if (writerClosed.pending) { var dC = writerClosed; writerClosed.pending = false; dC.resolve(undefined); } }
    }
    function finishInFlightCloseWithError(error) {
      inFlightCloseRequest.reject(error);
      inFlightCloseRequest = undefined;
      if (pendingAbortRequest !== undefined) {
        pendingAbortRequest.reject(error);
        pendingAbortRequest = undefined;
      }
      dealWithRejection(error);
    }
    function advanceQueueIfNeeded() {
      // spec AdvanceQueueIfNeeded——started 门 + erroring 分流 + 队首处理（close 哨兵/写）。
      if (!started) return;
      if (inFlightWriteRequest !== undefined) return;
      if (state === 'erroring') { finishErroring(); return; }
      if (queue.length === 0) return;
      var head = queue[0];
      if (head === CLOSE_SENTINEL) processClose();
      else processWrite(head.chunk);
    }
    function processClose() {
      // spec ProcessClose——MarkCloseRequestInFlight + close 算法 + ClearAlgorithms。
      inFlightCloseRequest = closeRequest;
      closeRequest = undefined;
      queue.shift();
      var p;
      try { p = (closeAlgorithmLive && sinkCloseFn) ? Promise.resolve(sinkCloseFn.call(sink)) : Promise.resolve(); }
      catch (eCl) { p = Promise.reject(eCl); }
      clearAlgorithms();
      p.then(function () { finishInFlightClose(); },
             function (rCl) { finishInFlightCloseWithError(rCl); });
    }
    function processWrite(chunk) {
      // spec ProcessWrite——MarkFirstWriteRequestInFlight + write 算法（串行推进）。
      inFlightWriteRequest = writeRequests.shift();
      var p;
      try { p = sinkWriteFn ? Promise.resolve(sinkWriteFn.call(sink, chunk, controller)) : Promise.resolve(); }
      catch (eWr) { p = Promise.reject(eWr); }
      p.then(function () { finishInFlightWrite(); },
             function (rWr) { finishInFlightWriteWithError(rWr); });
    }
    function controllerWrite(chunk, chunkSize) {
      // spec ControllerWrite——EnqueueValueWithSize（非法 size → ErrorIfNeeded）+ 背压 + 续推。
      if (typeof chunkSize !== 'number' || chunkSize !== chunkSize || chunkSize < 0 || chunkSize === Infinity) {
        var reSize = new RangeError('Invalid chunk size');
        errorIfNeeded(reSize);
        return;
      }
      queue.push({ chunk: chunk, size: chunkSize });
      queueTotalSize += chunkSize;
      if (!closeQueuedOrInFlight() && state === 'writable') updateBackpressure(getBackpressure());
      advanceQueueIfNeeded();
    }
    function errorIfNeeded(e) {
      if (state === 'writable') startErroring(e);
    }
    // spec [[controller]]——signal（AbortSignal——stream.abort 同步 signal 面）+ error。
    var abortSignal = new AbortSignal();
    var controller = {
      constructor: WritableStreamDefaultController,
      get signal() { return abortSignal; },
      error: function (eCtl) {
        // spec ControllerError——非 writable 返回；ClearAlgorithms + StartErroring。
        if (state !== 'writable') return;
        clearAlgorithms();
        startErroring(eCtl);
      }
    };
    this._controller = controller;
    // net-api M4-S4：pipeTo dest 首查锚；net-api M4-S5：TS error 锚（DealWithRejection 同型）。
    this._zwWsProbe = function () { return { state: state, error: storedError, closing: closeQueuedOrInFlight() }; };
    this._zwWsErrorFn = function (e) { dealWithRejection(e); };

    this.getWriter = function () {
      if (self._locked) throw new TypeError('Cannot get a Writer: WritableStream is locked');
      // spec SetUpWritableStreamDefaultWriter——按状态建 closed/ready promise。
      writerClosed = mkDeferred();
      if (state === 'writable') {
        writerReady = (!closeQueuedOrInFlight() && backpressure) ? mkDeferred() : { pending: false, promise: Promise.resolve() };
      } else if (state === 'erroring') {
        var dEr = mkDeferred(); dEr.reject(storedError); dEr.pending = false;
        try { dEr.promise.then(function () {}, function () {}); } catch (_eE1) {}
        writerReady = { pending: false, promise: dEr.promise };
      } else if (state === 'closed') {
        writerClosed.pending = false; writerClosed.resolve(undefined);
        writerReady = { pending: false, promise: Promise.resolve() };
      } else { // errored
        writerClosed = { pending: false, promise: makeRejected(storedError) };
        writerReady = { pending: false, promise: makeRejected(storedError) };
      }
      self._locked = true;
      var released = false; // net-api M4-S6：release 后 write/close/abort → TypeError（spec [[stream]] undefined 面）
      return {
        constructor: WritableStreamDefaultWriter, // net-api M4-S5：构造器身份
        get closed() { return writerClosed.promise; },
        get ready() { return writerReady.promise; },
        get desiredSize() { return desiredSize(); },
        write: function (chunk) {
          // spec WriterWrite——released → TypeError；errored/erroring → storedError；
          // closing/closed → TypeError；size 异常 → error 流 + reject（WPT observable 同型）。
          if (released) return Promise.reject(new TypeError('WritableStreamDefaultWriter: stream is undefined'));
          var sz;
          try { sz = (typeof sizeFn === 'function') ? sizeFn(chunk) : 1; }
          catch (eSize) { errorIfNeeded(eSize); return Promise.reject(eSize); }
          if (state === 'errored') return Promise.reject(storedError);
          if (closeQueuedOrInFlight() || state === 'closed') {
            return Promise.reject(new TypeError('Cannot write to a closing or closed WritableStream'));
          }
          if (state === 'erroring') return Promise.reject(storedError);
          var entry = mkDeferred();
          writeRequests.push({ resolve: entry.resolve, reject: entry.reject, size: sz });
          controllerWrite(chunk, sz);
          return entry.promise;
        },
        close: function () {
          // spec WriterClose——released → TypeError；其余走共享 WritableStreamClose。
          if (released) return Promise.reject(new TypeError('WritableStreamDefaultWriter: stream is undefined'));
          return streamClose(true);
        },
        abort: function (reason) {
          if (released) return Promise.reject(new TypeError('WritableStreamDefaultWriter: stream is undefined'));
          return doAbort(reason);
        },
        releaseLock: function () {
          if (!self._locked) return;
          self._locked = false;
          released = true;
          // spec WriterRelease——ready/closed 转 TypeError 拒绝（已发布面标记 handled）。
          ensureReadyRejected(new TypeError('WritableStreamDefaultWriter released lock'));
          ensureClosedRejected(new TypeError('WritableStreamDefaultWriter released lock'));
        }
      };
    };
    function streamClose(skipLockedCheck) {
      // net-api M4-S6：spec WritableStreamClose 共享体——stream 入口 locked → TypeError
      //（writer 入口已持锁，跳过）；CloseQueuedOrInFlight/closed/errored → TypeError；
      // erroring 不就地拒（closeRequest 经 FinishErroring 以 storedError reject）。
      if (!skipLockedCheck && self._locked) return Promise.reject(new TypeError('Cannot close a WritableStream that is locked'));
      if (closeQueuedOrInFlight() || state === 'closed' || state === 'errored') {
        return Promise.reject(new TypeError('Cannot close a closing, closed or errored WritableStream'));
      }
      var d = mkDeferred();
      closeRequest = d;
      // spec WritableStreamClose 步骤 8——backpressure → resolve writer.ready。
      if (writerReady && backpressure && state === 'writable' && writerReady.pending) {
        var dR = writerReady; writerReady = { pending: false, promise: Promise.resolve() }; dR.resolve(undefined);
      }
      queue.push(CLOSE_SENTINEL);
      advanceQueueIfNeeded();
      return d.promise;
    }
    this.close = function () { return streamClose(); };
    function doAbort(reason) {
      // spec WritableStreamAbort——closed/errored → resolve；**同步 signal abort**；再核状态；
      // pendingAbortRequest 在 → 返回其 promise；否则建 pendingAbortRequest + StartErroring
      //（与 close 排队/in-flight **共存**：close 成功 → FinishInFlightClose resolve abort 请求
      //（「ignore the abort attempt」面）；close 失败 → FinishInFlightCloseWithError 以 close
      // 错误 reject（「abort rejected with the rejection returned from close」面））。
      if (state === 'closed' || state === 'errored') return Promise.resolve();
      _zw_abort_signal(abortSignal, reason);
      if (state === 'closed' || state === 'errored') return Promise.resolve();
      if (pendingAbortRequest !== undefined) return pendingAbortRequest.promise;
      var wasAlreadyErroring = state === 'erroring';
      var abortReason = wasAlreadyErroring ? undefined : reason;
      var dAb = mkDeferred();
      pendingAbortRequest = { promise: dAb.promise, resolve: dAb.resolve, reject: dAb.reject, reason: abortReason, wasAlreadyErroring: wasAlreadyErroring };
      if (!wasAlreadyErroring) startErroring(abortReason);
      return dAb.promise;
    }
    this.abort = function (reason) {
      if (self._locked) return Promise.reject(new TypeError('Cannot abort a WritableStream that is locked'));
      return doAbort(reason);
    };
    // start：spec SetUp——[[started]] 在 startPromise 稳定后置位；拒绝 → started 置位 +
    // DealWithRejection（「sink abort() should not be called until sink start() is done」面）。
    var startResult;
    try { startResult = sink.start ? sink.start.call(sink, controller) : undefined; }
    catch (_eWsStart) { startErroring(_eWsStart); startResult = undefined; }
    Promise.resolve(startResult).then(function () {
      started = true;
      advanceQueueIfNeeded();
    }, function (eStart2) {
      started = true;
      dealWithRejection(eStart2);
    });
  };

  // net-api M4-S5：locked accessor 上移 prototype（readable 同口径——subclass/品牌访问面）。
  Object.defineProperty(globalThis.WritableStream.prototype, 'locked', {
    get: function () { return this._locked; },
    enumerable: true, configurable: true
  });
  // encoding-compat M3：prototype.getWriter 委托（readable-writable-properties 品牌面——
  // `WritableStream.prototype.getWriter.call(ws)` 需原型方法；ReadableStream 侧 M4-S8
  // 已有同款委托，WritableStream 侧此前缺）。非流 this → TypeError（品牌语义，readable
  // 侧同口径）。
  Object.defineProperty(globalThis.WritableStream.prototype, 'getWriter', {
    value: function () {
      var fn = (this != null) ? this.getWriter : null;
      if (typeof fn !== 'function' || !Object.prototype.hasOwnProperty.call(this, 'getWriter')) {
        throw new TypeError("Failed to execute 'getWriter' on 'WritableStream': Illegal invocation.");
      }
      return Object.getOwnPropertyDescriptor(this, 'getWriter').value.call(this);
    },
    enumerable: true, configurable: true, writable: true
  });

  // ── P1a TransformStream（Streams API transform，R2969）──
  // {readable, writable} 配对：writable.write(chunk) → transformer.transform(chunk, controller) →
  // controller.enqueue 到 readable；writable.close → transformer.flush(controller) → close readable。
  // 无 transform fn → 恒等（chunk 直 enqueue）。controller.enqueue/close/error 转发到 readable 的 controller。
  // 用于 pipeThrough 管道（如 response.body.pipeThrough(new TextDecoderStream()) 解码——TextDecoderStream
  // 本身属 follow-up，本切片提供 TransformStream 基座）。
  // ── net-api M4-S1：queuing strategies + ReadableStream.from ──────────────────
  // CountQueuingStrategy / ByteLengthQueuingStrategy（spec §queuing）。
  // net-api M4-S4：size 为**全局共享** size 函数（instances 同一函数——「size is the same
  // function across all instances」面）；箭头函数形态——无 prototype 属性 + 不可 new
  //（「size should not have a prototype property」/「size should not be a constructor」面），
  // name/length 显式定（'size'/0|1）；ByteLength size = GetV(chunk, 'byteLength')（null/
  // undefined chunk → TypeError；缺失 → undefined 原样返回，非 NaN）。
  var _zwCountSizeFn = () => 1;
  var _zwByteLengthSizeFn = (chunk) => {
    if (chunk == null) throw new TypeError('chunk must be an object');
    return chunk.byteLength;
  };
  try { Object.defineProperty(_zwCountSizeFn, 'name', { value: 'size' }); } catch (_eSizeName) {}
  try { Object.defineProperty(_zwByteLengthSizeFn, 'name', { value: 'size' }); } catch (_eSizeName2) {}
  globalThis.CountQueuingStrategy = globalThis.CountQueuingStrategy || function CountQueuingStrategy(init) {
    if (!(this instanceof CountQueuingStrategy)) return new CountQueuingStrategy(init);
    // net-api M4-S4：QueuingStrategyInit required highWaterMark（非对象 init / 缺失成员 →
    // TypeError——「strange arguments」面）；getter 抛错同步传播（throwing getter 面）。
    if (init == null || typeof init !== 'object') {
      throw new TypeError("Failed to construct 'CountQueuingStrategy': the provided value is not of type 'QueuingStrategyInit'.");
    }
    var hwmValue = init.highWaterMark;
    if (hwmValue === undefined) {
      throw new TypeError("Failed to construct 'CountQueuingStrategy': required member highWaterMark is undefined.");
    }
    this.highWaterMark = Number(hwmValue); // unrestricted double 转换（NaN/±0/负数均合法存储）
    this.size = _zwCountSizeFn;
  };
  globalThis.ByteLengthQueuingStrategy = globalThis.ByteLengthQueuingStrategy || function ByteLengthQueuingStrategy(init) {
    if (!(this instanceof ByteLengthQueuingStrategy)) return new ByteLengthQueuingStrategy(init);
    if (init == null || typeof init !== 'object') {
      throw new TypeError("Failed to construct 'ByteLengthQueuingStrategy': the provided value is not of type 'QueuingStrategyInit'.");
    }
    var hwmValue2 = init.highWaterMark;
    if (hwmValue2 === undefined) {
      throw new TypeError("Failed to construct 'ByteLengthQueuingStrategy': required member highWaterMark is undefined.");
    }
    this.highWaterMark = Number(hwmValue2);
    this.size = _zwByteLengthSizeFn;
  };
  // ReadableStream.from（spec §rs.from——async-iterable → ReadableStream）。getMethod/
  // 迭代器获取错误**同步重抛**（from re-throws 面）；@@asyncIterator 优先、回落
  // @@iterator（同步迭代器经 Promise 适配）；pull 逐 next（done → close）；error 传播
  // controller.error；cancel → 调迭代器 return（best-effort）。
  if (!ReadableStream.from) {
    ReadableStream.from = function (source) {
      var itFn = (source != null && source[Symbol.asyncIterator] !== undefined)
        ? source[Symbol.asyncIterator] : undefined;
      var isAsync = typeof itFn === 'function';
      if (!isAsync) {
        itFn = (source != null && source[Symbol.iterator] !== undefined) ? source[Symbol.iterator] : undefined;
        if (typeof itFn !== 'function') {
          throw new TypeError('ReadableStream.from: source is not async-iterable');
        }
      }
      var iterator = itFn.call(source);
      // net-api M4-S4：spec CreateReadableStream(start, pull, cancel, **0**)——hwm 0（惰性拉，
      // 构造即拉面回归防护）。
      return new ReadableStream({
        pull: function (controller) {
          var p;
          try { p = isAsync ? iterator.next() : Promise.resolve(iterator.next()); }
          catch (_eFromNext) { controller.error(_eFromNext); return; }
          return Promise.resolve(p).then(function (r) {
            if (r.done) { controller.close(); return; }
            controller.enqueue(r.value);
          }, function (e) { controller.error(e); });
        },
        cancel: function () {
          try {
            if (typeof iterator.return === 'function') return Promise.resolve(iterator.return());
          } catch (_eFromReturn) {}
          return Promise.resolve();
        }
      }, { highWaterMark: 0 });
    };
  }
  // net-api M4-S5：TransformStreamDefaultController 全局类（不可构造——spec）。
  function TransformStreamDefaultController() {
    throw new TypeError('Illegal constructor');
  }
  globalThis.TransformStreamDefaultController = globalThis.TransformStreamDefaultController || TransformStreamDefaultController;
  // net-api M4-S5：TransformStream spec 化重做（spec §6.2——InitializeTransformStream +
  // DefaultSink/DefaultSource + backpressure 机制）——
  // ① 构造：transformer dictionary（readableType/writableType → RangeError）+ writable/readable
  //   策略分离（writable 默认 hwm 1、readable 默认 hwm 0 + size 算法透传）；
  // ② backpressure：[[backpressureChangePromise]]——初始 true；SinkWrite 背压门控（等 change
  //   promise 再 transform）+ **写链逐写串行**（spec AdvanceQueueIfNeeded 等价——sink.write 事件
  //   即时、transform 串行）；SourcePull 置 false 并返回新 change promise（翻转回 true 时完成）；
  //   ControllerEnqueue 背压 false→true 翻转观察；
  // ③ 传播：transform/flush 拒绝 → 双侧 error；readable cancel → cancel 算法后 error writable；
  //   terminate → close readable + error writable。
  globalThis.TransformStream = globalThis.TransformStream || function TransformStream(transformer, _writableStrategy, _readableStrategy) {
    if (!(this instanceof TransformStream)) return new TransformStream(transformer, _writableStrategy, _readableStrategy);
    var tx = (transformer === undefined || transformer === null) ? {} : transformer;
    if (tx.readableType !== undefined) throw new RangeError('TransformStream: readableType is reserved.');
    if (tx.writableType !== undefined) throw new RangeError('TransformStream: writableType is reserved.');
    var wStrat = _zwStrategyDict(_writableStrategy);
    var rStrat = _zwStrategyDict(_readableStrategy);
    var writableHwm = _zwExtractHwm(wStrat.hwmValue, 1);
    var readableHwm = _zwExtractHwm(rStrat.hwmValue, 0);
    var self = this;

    // ── backpressure 机制 ──
    var backpressure = true;
    var changePromise = null;
    function setBackpressure(bp) {
      // spec：resolve 旧 promise + 换新（初始/翻转各一）。
      if (changePromise && changePromise._zwResolve) { var r = changePromise._zwResolve; changePromise._zwResolve = null; r(); }
      var res = null;
      var p = new Promise(function (rs) { res = rs; });
      p._zwResolve = res;
      changePromise = p;
      backpressure = bp;
    }
    setBackpressure(true);

    var enqueueToR, closeR, errorR;
    function transformError(e) {
      // spec TransformError：error readable + ErrorWritableAndUnblockWrite(writable, e)。
      if (errorR) errorR(e);
      errorWritableAndUnblock(e);
    }
    function closeReadableSafe() {
      // net-api M4-S5：controller.close 在已关/排空关面抛 TypeError——spec DefaultControllerClose
      // 对非可关态静默返回（terminate after readable.cancel 面）。
      var probe = (self.readable && typeof self.readable._zwRsProbe === 'function') ? self.readable._zwRsProbe() : null;
      if (probe && probe.state === 'readable' && !probe.closeRequested && closeR) closeR();
    }
    function errorWritableAndUnblock(e) {
      // 惰性解析（tx.start 即刻 terminate 面——构造期 self.writable 尚未赋值）。
      var ws = self.writable;
      if (ws && typeof ws._zwWsErrorFn === 'function') ws._zwWsErrorFn(e);
      if (backpressure) setBackpressure(false); // spec UnblockWrite
    }
    function performTransform(chunk) {
      // spec PerformTransform：transformAlgorithm 拒绝 → TransformError + 重抛。
      var p;
      try {
        p = (typeof tx.transform === 'function')
          ? Promise.resolve(tx.transform(chunk, transformController))
          : (transformController.enqueue(chunk), Promise.resolve()); // 恒等
      } catch (eT) { p = Promise.reject(eT); }
      return p.then(function () {}, function (rT) { transformError(rT); throw rT; });
    }
    var sinkWriteQueue = Promise.resolve(); // 逐写串行（spec 写队列推进等价）
    function sinkWrite(chunk) {
      var run = sinkWriteQueue.then(function () {
        // spec SinkWrite：backpressure → 等 changePromise（捕获当前）后 transform；醒来后
        // writable 已 errored → 抛 storedError（terminate/error 期间挂起写的收尾面）。
        if (backpressure) {
          var bp = changePromise;
          return Promise.resolve(bp).then(function () {
            var probe = (self.writable && typeof self.writable._zwWsProbe === 'function') ? self.writable._zwWsProbe() : null;
            if (probe && (probe.state === 'errored' || probe.state === 'erroring')) throw probe.error;
            return performTransform(chunk);
          });
        }
        return performTransform(chunk);
      });
      sinkWriteQueue = run.then(function () {}, function () {});
      return run;
    }
    // net-api M4-S5：transformer start promise 门（spec——transform/flush 不早于 start 完成；
    // 「start, transform, and flush should be strictly ordered」面）。
    var startPromise = null;

    var transformController = {
      constructor: TransformStreamDefaultController,
      enqueue: function (chunk) {
        // spec TS ControllerEnqueue：readable enqueue 异常 → TransformErrorWritableAndUnblock
        // + 抛 storedError；背压 false→true 翻转观察（HasBackpressure = !ShouldCallPull）。
        try { enqueueToR(chunk); } catch (eEnq) { transformError(eEnq); throw eEnq; }
        var probe = (self.readable && typeof self.readable._zwRsProbe === 'function') ? self.readable._zwRsProbe() : null;
        var shouldPull = !!probe && probe.state === 'readable' && !probe.closeRequested &&
          (probe.readRequests > 0 || probe.desired > 0);
        var bp = !shouldPull;
        if (bp !== backpressure && bp === true) setBackpressure(true);
      },
      error: function (e) { transformError(e); },
      close: function () { closeReadableSafe(); }, // 内部兼容别名（flush 后关 readable）
      terminate: function () {
        // spec Terminate：close readable + error writable(TypeError)。
        closeReadableSafe();
        errorWritableAndUnblock(new TypeError('TransformStream terminated'));
      }
    };

    // net-api M4-S5：tx.start 恰调一次（readable start 内）；start promise 同时门控写链
    //（transform/flush 不早于 start 完成）。
    this.readable = new ReadableStream({
      start: function (controller) {
        enqueueToR = controller.enqueue;
        closeR = controller.close;
        errorR = controller.error;
        try {
          startPromise = Promise.resolve(typeof tx.start === 'function' ? tx.start(transformController) : undefined);
        } catch (eStart) {
          startPromise = Promise.reject(eStart);
        }
      },
      pull: function () {
        // spec SourcePull：置 false（释放写侧）+ 返回新 change promise（下一次翻回 true 时完成）。
        setBackpressure(false);
        return changePromise;
      },
      cancel: function (reason) {
        // spec SourceCancel：cancel 算法 → ErrorIfNeeded(writable, reason) + UnblockWrite。
        var p;
        try {
          p = (typeof tx.cancel === 'function') ? Promise.resolve(tx.cancel(reason)) : Promise.resolve();
        } catch (eC) { p = Promise.reject(eC); }
        return p.then(function () { errorWritableAndUnblock(reason); },
                      function (rC) { errorWritableAndUnblock(rC); throw rC; });
      }
    }, { highWaterMark: readableHwm, size: rStrat.size !== undefined ? rStrat.size : undefined });

    this.writable = new WritableStream({
      write: function (chunk) { return sinkWrite(chunk); },
      close: function () {
        // spec SinkClose：写链排空 → flush → 关 readable（flush 拒绝 → 双侧 error + 重抛）。
        var fin = sinkWriteQueue.then(function () {
          var p;
          try {
            p = (typeof tx.flush === 'function') ? Promise.resolve(tx.flush(transformController)) : Promise.resolve();
          } catch (eF) { p = Promise.reject(eF); }
          return p.then(function () { closeReadableSafe(); },
                        function (rF) { transformError(rF); throw rF; });
        });
        sinkWriteQueue = fin.then(function () {}, function () {});
        return fin;
      },
      abort: function (reason) {
        // spec SinkAbort：cancel 算法 → error readable(reason)/r。
        var p;
        try {
          p = (typeof tx.cancel === 'function') ? Promise.resolve(tx.cancel(reason)) : Promise.resolve();
        } catch (eA) { p = Promise.reject(eA); }
        return p.then(function () { if (errorR) errorR(reason); },
                      function (rA) { if (errorR) errorR(rA); throw rA; });
      }
    }, { highWaterMark: writableHwm, size: wStrat.size !== undefined ? wStrat.size : undefined });
    // start promise 门挂到写链头部（start 拒绝 → 写侧后续全部拒绝 + 双侧 error——spec 由
    // startPromise rejection → TransformError 传播）。
    startPromise.then(function () {}, function (eStart2) { transformError(eStart2); });
    sinkWriteQueue = sinkWriteQueue.then(function () { return startPromise; }, function () { return startPromise; });
  };
  // net-api M4-S5：readable/writable prototype accessor（general「Subclassing TransformStream」
  // 面——getOwnPropertyDescriptor(...).get 品牌访问）。**必须带 setter**（写回 _zw 槽）——getter-only
  // 形态下 sloppy 构造器的 `this.readable = ...` 静默失效（实例属性永不落位）。
  Object.defineProperty(globalThis.TransformStream.prototype, 'readable', {
    get: function () { return this._zwReadable; },
    set: function (v) { this._zwReadable = v; },
    enumerable: true, configurable: true
  });
  Object.defineProperty(globalThis.TransformStream.prototype, 'writable', {
    get: function () { return this._zwWritable; },
    set: function (v) { this._zwWritable = v; },
    enumerable: true, configurable: true
  });

  // ── P1a TextEncoderStream / TextDecoderStream（编码转换流，R2970）──
  // spec 通用编码转换流（Generic Transform Stream），常与 `response.body.pipeThrough(new TextDecoderStream())`
  // 配对——fetch body 为 UTF-8 字节流（ReadableStream<Uint8Array>），TextDecoderStream 转 string 流供逐块
  // 文本消费（fetch streaming 文本 / NDJSON / SSE 手解析）。薄封装于既有 TextEncoder/TextDecoder（part02）+
  // TransformStream（R2969）：string→Uint8Array（encode）/ Uint8Array→string（decode）。继承 TransformStream
  //（TransformStream.call(this, transformer) 设 readable/writable），补 encoding/fatal/ignoreBOM IDL 属性。
  // R3012：流式状态闭合——TextDecoder.decode({stream:true}) 跨 chunk 维护未完成字节序列（_carry），
  // 故 chunk 边界切多字节 char 正确重组（不再各 chunk 独立解码损坏）。transform 用 stream:true，flush 残余。
  globalThis.TextEncoderStream = globalThis.TextEncoderStream || function TextEncoderStream() {
    if (!(this instanceof TextEncoderStream)) return new TextEncoderStream();
    var enc = new TextEncoder();
    // encoding-compat M3：跨 chunk UTF-16 边界驻留——chunk 末尾孤立高代理（astral 字符
    // 被 chunk 边界切开）待下块成对后再编码（spec TransformStreamEncode）；尾部空 chunk
    // 忽略（不产空输出 chunk——encode-utf8「trailing empty chunk」面）。
    var pending = '';
    TransformStream.call(this, {
      transform: function (chunk, controller) {
        // spec：chunk USVString 全量 ToString 转换（undefined → "undefined"——encode-bad-chunks 面）。
        var s = pending + String(chunk);
        pending = '';
        if (s.length) {
          var last = s.charCodeAt(s.length - 1);
          if (last >= 0xD800 && last <= 0xDBFF) {
            pending = s.charAt(s.length - 1);
            s = s.slice(0, -1);
          }
        }
        if (s) controller.enqueue(enc.encode(s));
      },
      flush: function (controller) {
        if (pending) { controller.enqueue(enc.encode(pending)); pending = ''; }
      }
    });
    this.encoding = 'utf-8';
  };
  globalThis.TextEncoderStream.prototype = Object.create(globalThis.TransformStream.prototype);
  globalThis.TextEncoderStream.prototype.constructor = globalThis.TextEncoderStream;
  globalThis.TextDecoderStream = globalThis.TextDecoderStream || function TextDecoderStream(label, options) {
    if (!(this instanceof TextDecoderStream)) return new TextDecoderStream(label, options);
    // encoding-compat M3：options（fatal/ignoreBOM）转发 TextDecoder（decode-attributes
    // IDL 反射面 + decode-ignore-bom 语义面）；label 默认 "utf-8"（spec TextDecoderStream
    // 构造器——`new TextDecoderStream()` 合法，而 `new TextDecoderStream('')` 经
    // get-an-encoding 失败抛 RangeError——与 TextDecoder 的 "" 默认不同面）。host 缺席时
    // 降级放行（零回归）。
    var rawLabel = label === undefined ? 'utf-8' : String(label);
    var dec;
    if (typeof __zw_text_encoding_of === 'function') {
      var canonical = __zw_text_encoding_of(rawLabel);
      if (canonical === '' || canonical === 'replacement') {
        throw new RangeError("Failed to construct 'TextDecoderStream': The encoding label provided ('" + rawLabel + "') is invalid.");
      }
      dec = new TextDecoder(canonical, options);
    } else {
      dec = new TextDecoder(rawLabel, options); // host 缺席降级（旧形态，label 不校验）
    }
    TransformStream.call(this, {
      transform: function (chunk, controller) {
        if (!(chunk instanceof ArrayBuffer || (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView(chunk)))) {
          throw new TypeError("Failed to execute 'transform' on 'TextDecoderStream': the provided chunk is not a BufferSource.");
        }
        var s = dec.decode(chunk, { stream: true }); // R3012：stream:true 跨 chunk 状态
        if (s) controller.enqueue(s); // 空 string（chunk 末切多字节前导字节，缓存于状态机）跳过，避免空 chunk
      },
      flush: function (controller) {
        var s = dec.decode(); // flush 残余（stream:false；fatal → TypeError → TransformError）
        if (s) controller.enqueue(s);
      }
    });
    this.encoding = dec.encoding || 'utf-8';
    this.fatal = !!dec.fatal;
    this.ignoreBOM = !!dec.ignoreBOM;
  };
  globalThis.TextDecoderStream.prototype = Object.create(globalThis.TransformStream.prototype);
  globalThis.TextDecoderStream.prototype.constructor = globalThis.TextDecoderStream;

  // ── P1a CompressionStream / DecompressionStream（gzip/deflate/deflate-raw，R2986）──
  // spec 通用压缩/解压转换流（Compression Streams API）。常用于 `response.body.pipeThrough(new
  // DecompressionStream('gzip'))` 解压服务端 gzip 流，或压缩上传载荷。host 经 flate2（既有 workspace crate）
  // 压缩/解压；字节经逗号分隔十进制串 wire（同 crypto byte wire）。**buffer-then-process**：transform 累积
  // 全部 chunk（压缩须见全输入产合法 gzip/deflate 帧——逐 chunk 独立压缩会产多帧错误输出），flush 整体
  // 压缩/解压 + enqueue 单输出 chunk。headless finite 流模型正确。
  // 不支持 format → 构造抛 DOMException NotSupportedError（spec）；host 未注册（engine polyfill/reftest）→ no-op。
  // **已知限制**：① 非增量（buffer 全输入再处理，大流内存峰值 = 输入大小，headless finite 可接受）；
  // ② CSV byte wire 4× 开销（V8 字符串往返，大流慢）；③ 仅 gzip/deflate/deflate-raw（brotli defer，需 brotli crate）。
  var _CS_FORMATS = { gzip: 1, deflate: 1, 'deflate-raw': 1 };
  function _csEnqueueBytes(controller, csv) {
    if (!csv) return;
    var parts = String(csv).split(',');
    var arr = new Uint8Array(parts.length);
    for (var i = 0; i < parts.length; i++) arr[i] = parseInt(parts[i], 10) || 0;
    if (arr.length) controller.enqueue(arr);
  }
  globalThis.CompressionStream = globalThis.CompressionStream || function CompressionStream(format) {
    if (!(this instanceof CompressionStream)) return new CompressionStream(format);
    var fmt = String(format == null ? '' : format).toLowerCase();
    if (!_CS_FORMATS[fmt]) {
      throw new DOMException("Failed to construct 'CompressionStream': Unsupported compression format, '" + format + "'. Supported values are: 'gzip', 'deflate', 'deflate-raw'.", 'NotSupportedError');
    }
    var bufs = [];
    TransformStream.call(this, {
      transform: function (chunk, controller) {
        var b = _zw_bufToBytes(chunk);
        for (var i = 0; i < b.length; i++) bufs.push(b[i]);
      },
      flush: function (controller) {
        if (typeof __zw_compress !== 'function') return;
        try { _csEnqueueBytes(controller, __zw_compress(fmt, bufs.join(','))); } catch (_e) {}
      }
    });
  };
  globalThis.CompressionStream.prototype = Object.create(globalThis.TransformStream.prototype);
  globalThis.CompressionStream.prototype.constructor = globalThis.CompressionStream;
  globalThis.DecompressionStream = globalThis.DecompressionStream || function DecompressionStream(format) {
    if (!(this instanceof DecompressionStream)) return new DecompressionStream(format);
    var fmt = String(format == null ? '' : format).toLowerCase();
    if (!_CS_FORMATS[fmt]) {
      throw new DOMException("Failed to construct 'DecompressionStream': Unsupported compression format, '" + format + "'. Supported values are: 'gzip', 'deflate', 'deflate-raw'.", 'NotSupportedError');
    }
    var bufs = [];
    TransformStream.call(this, {
      transform: function (chunk, controller) {
        var b = _zw_bufToBytes(chunk);
        for (var i = 0; i < b.length; i++) bufs.push(b[i]);
      },
      flush: function (controller) {
        if (typeof __zw_decompress !== 'function') return;
        try {
          var out = __zw_decompress(fmt, bufs.join(','));
          if (!out && bufs.length) { controller.error(new DOMException('Decompression failed: corrupt ' + fmt + ' stream', 'DataError')); return; }
          _csEnqueueBytes(controller, out);
        } catch (_e) { controller.error(_e); }
      }
    });
  };
  globalThis.DecompressionStream.prototype = Object.create(globalThis.TransformStream.prototype);
  globalThis.DecompressionStream.prototype.constructor = globalThis.DecompressionStream;

  // URLSearchParams——query string 解析/序列化（location.search / fetch query 高频）。
  // 纯 JS（V8 原生 encodeURIComponent/decodeURIComponent + Symbol.iterator）。application/x-www-form-urlencoded
  // 语义：space→`+`；构造支持 string（`?` 前缀可省）/ 对象 / [k,v] 可迭代。
  function _zw_iter(arr) {
    var i = 0;
    var it = {
      next: function () {
        if (i < arr.length) { return { value: arr[i++], done: false }; }
        return { value: undefined, done: true };
      }
    };
    if (typeof Symbol !== 'undefined') it[Symbol.iterator] = function () { return it; };
    return it;
  }
  // URLSearchParams 查询串解析（'a=1&b=2' / '?a=1' → [[k,v],...]），constructor 与 _zw_reinit 共用。
  // net-api M3-S3：urlencoded percent-decode（urlencoded-parser——%XX → 字节、非法 %
  // 序列字面保留、字节经 UTF-8 lossy 解码，**不抛**（decodeURIComponent 的 URIError
  // 是 %FE%FF/%C2 面根因）；'+' → space）。复用 part01 _zwPercentDecodeBytes（同 IIFE）。
  function _zw_usp_decode(part) {
    var bytes = _zwPercentDecodeBytes(String(part).replace(/\+/g, ' '));
    return new TextDecoder().decode(bytes);
  }
  function _zw_usp_parse(s) {
    var out = [];
    if (typeof s !== 'string' || !s) return out;
    if (s.charAt(0) === '?') s = s.slice(1);
    var parts = s.split('&');
    for (var i = 0; i < parts.length; i++) {
      var p = parts[i];
      if (p === '') continue;
      var eq = p.indexOf('=');
      var k = eq < 0 ? p : p.slice(0, eq);
      var v = eq < 0 ? '' : p.slice(eq + 1);
      out.push([_zw_usp_decode(k), _zw_usp_decode(v)]);
    }
    return out;
  }
  // https://url.spec.whatwg.org/#concept-urlencoded-serializer——`*`/`-`/`.`/`_`/字
  // 数字直出、0x20→'+'、其余 %XX 大写（encodeURIComponent 会漏 !'() 与 ~——send-usp
  // 0x21/0x27/0x28/0x29 断言面）。
  function _zwUrlencodedEncode(s) {
    var str = String(s);
    var bytes = new TextEncoder().encode(str);
    var out = '';
    for (var i = 0; i < bytes.length; i++) {
      var b = bytes[i];
      if (b === 0x20) {
        out += '+';
      } else if ((b >= 0x30 && b <= 0x39) || (b >= 0x41 && b <= 0x5A) || (b >= 0x61 && b <= 0x7A) ||
                 b === 0x2A || b === 0x2D || b === 0x2E || b === 0x5F) {
        out += String.fromCharCode(b);
      } else {
        var hex = b.toString(16).toUpperCase();
        out += '%' + (hex.length < 2 ? '0' + hex : hex);
      }
    }
    return out;
  }
  globalThis.URLSearchParams = globalThis.URLSearchParams || function URLSearchParams(init) {
    if (!(this instanceof URLSearchParams)) return new URLSearchParams(init);
    this._p = [];
    this._onchange = null; // URL 父对象注册的变更回调（searchParams→search 同步，R2780）
    if (init == null) return;
    if (typeof init === 'string') {
      this._p = _zw_usp_parse(init);
    } else if (typeof init === 'object') {
      if (Array.isArray(init)) {
        // net-api M3-S3：sequence 形态（sequence<sequence<StringValue>>）——数组优先于
        // forEach（Array.forEach 的 (val, index) 形态会把数组当 record 序列化成索引键）；
        // 逐项须恰为二元组（spec，非二元组 → TypeError）。
        for (var ai = 0; ai < init.length; ai++) {
          var pair = init[ai];
          if (!pair || !pair.length || pair.length !== 2) {
            throw new TypeError('URLSearchParams init sequence item must have exactly 2 elements');
          }
          this._p.push([String(pair[0]), String(pair[1])]);
        }
      } else if (typeof init.forEach === 'function') {
        var self = this;
        init.forEach(function (val, key) { self._p.push([String(key), String(val)]); });
      } else {
        for (var key in init) {
          if (Object.prototype.hasOwnProperty.call(init, key)) this._p.push([String(key), String(init[key])]);
        }
      }
    }
  };
  globalThis.URLSearchParams.prototype = {
    append: function (n, v) { this._p.push([String(n), String(v)]); this._changed(); },
    delete: function (n, v) {
      n = String(n);
      // net-api M3-S3：第二参可选——显式 undefined = 未给（WebIDL optional；
      // 「Two-argument delete() respects undefined」面）。
      if (v !== undefined) {
        v = String(v);
        this._p = this._p.filter(function (p) { return !(p[0] === n && p[1] === v); });
      } else {
        this._p = this._p.filter(function (p) { return p[0] !== n; });
      }
      this._changed();
    },
    get: function (n) { n = String(n); for (var i = 0; i < this._p.length; i++) if (this._p[i][0] === n) return this._p[i][1]; return null; },
    getAll: function (n) { n = String(n); var r = []; for (var i = 0; i < this._p.length; i++) if (this._p[i][0] === n) r.push(this._p[i][1]); return r; },
    has: function (n, v) {
      n = String(n);
      var hasV = arguments.length >= 2; if (hasV) v = String(v);
      for (var i = 0; i < this._p.length; i++) {
        if (this._p[i][0] === n && (!hasV || this._p[i][1] === v)) return true;
      }
      return false;
    },
    set: function (n, v) {
      n = String(n); v = String(v);
      var found = false; var out = [];
      for (var i = 0; i < this._p.length; i++) {
        if (this._p[i][0] === n) { if (!found) { out.push([n, v]); found = true; } }
        else out.push(this._p[i]);
      }
      if (!found) out.push([n, v]);
      this._p = out;
      this._changed();
    },
    sort: function () { this._p.sort(function (a, b) { return a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0; }); this._changed(); },
    // 内部：触发 _onchange（若注册）。供 append/delete/set/sort 复用（searchParams→search 同步）。
    _changed: function () { if (typeof this._onchange === 'function') this._onchange(); },
    // 内部：从查询串重载 _p（**不触发** _onchange）。URL.search/href setter 同步 searchParams 时调。
    _zw_reinit: function (s) { this._p = _zw_usp_parse(s); },
    forEach: function (cb, thisArg) { for (var i = 0; i < this._p.length; i++) cb.call(thisArg, this._p[i][1], this._p[i][0], this); },
    // net-api M3-S3：live 光标迭代（每次 next 基于当前 _p——delete-during-iteration 面；
    // 与 Headers live 语义同型）。
    entries: function () {
      var self = this;
      return _zwHeadersLiveIter(function () { return self._p; }, function (p) { return [p[0], p[1]]; });
    },
    keys: function () {
      var self = this;
      return _zwHeadersLiveIter(function () { return self._p; }, function (p) { return p[0]; });
    },
    values: function () {
      var self = this;
      return _zwHeadersLiveIter(function () { return self._p; }, function (p) { return p[1]; });
    },
    get size() { return this._p.length; }, // net-api M3-S3：size getter（urlsearchparams-size）
    toString: function () {
      var out = [];
      for (var i = 0; i < this._p.length; i++) {
        out.push(_zwUrlencodedEncode(this._p[i][0]) + '=' + _zwUrlencodedEncode(this._p[i][1]));
      }
      return out.join('&');
    }
  };
  // 自身可迭代（for (const [k,v] of params)）：[Symbol.iterator] → entries。
  if (typeof Symbol !== 'undefined') {
    globalThis.URLSearchParams.prototype[Symbol.iterator] = globalThis.URLSearchParams.prototype.entries;
  }

  // FormData——表单字段集合（表单序列化 / fetch multipart body 高频）。镜像 URLSearchParams 的
  // pair-store 模式（`_p` = [[name,value,filename?]] 保序、允许重名）；纯 JS 自包含，零 host 回调。
  // R3014：Blob/File 值保真（spec：非 Blob 转 USVString；Blob 保留对象，get 返 Blob）+ `_zwMultipart()`
  // 序列化 multipart/form-data body（fetch POST FormData 接线）。entry 第 3 元 filename 仅对 Blob 值有意义。
  // **已知限制（记录）**：constructor `form` 参数为 best-effort——若传入 `<form>` 元素，尝试枚举其
  // input/select/textarea 命名字段（checkbox/radio 仅 checked 入列），任一步失败静默跳过（不抛）；
  // 不覆盖 select-multiple / file input / disabled / form-attribute 等完整表单语义（renderer 路径
  // 真实字段枚举为 follow-up；多数库 `new FormData()` 空构造再 append，本路径完整支持）。
  var _zwFdCounter = 0;
  // R3014：FormData entry 构造——Blob/File 保留对象 + filename（spec get 返 Blob）；非 Blob 转 USVString。
  function _zwFdEntry(name, value, filename) {
    var n = String(name);
    if (value != null && value instanceof Blob) {
      var fn = filename != null ? String(filename) : (value.name != null ? String(value.name) : 'blob');
      // net-api M4-S20：spec 转换规则——Blob（非 File）→ 新 File；File + filename →
      // 按新名复制（lastModified 保真）；File 无 filename → 原对象保留（foreach 身份面）。
      if (typeof File === 'function') {
        var isFile = value instanceof File;
        if (!isFile || filename != null) {
          value = new File([value], fn, { type: value.type || '', lastModified: value.lastModified });
          return [n, value, undefined];
        }
        return [n, value, fn];
      }
      return [n, value, fn];
    }
    return [n, String(value), undefined];
  }
  // net-api M4-S19：(form, submitter) 双参构造（HTML spec §constructing-the-form-data-set）
  // ——submitter 校验（非 submit 按钮 → TypeError；非本 form 属主 → NotFoundError）+
  // submitter 条目按树序插入（image 按钮 → name.x/name.y）+ 'formdata' 事件派发
  //（FormDataEvent——e.formData 可变、构造结果≠事件对象、重入构造 → InvalidStateError）。
  globalThis.FormData = globalThis.FormData || function FormData(form, submitter) {
    if (!(this instanceof FormData)) return new FormData(form, submitter);
    this._p = [];
    if (form == null) {
      if (submitter != null) {
        throw new TypeError("Failed to construct 'FormData': submitter was specified but form was not");
      }
      return;
    }
    if (typeof form !== 'object' || typeof form.querySelectorAll !== 'function') return;
    // submitter 校验（spec：非 submit 按钮 → TypeError；非本 form 属主 → NotFoundError）。
    if (submitter != null && typeof submitter === 'object') {
      var sTag = String(submitter._realTag || submitter.tagName || '').toLowerCase();
      var sType = String((submitter.getAttribute ? submitter.getAttribute('type') : submitter.type) || '').toLowerCase();
      var isButton = (sTag === 'input' && (sType === 'submit' || sType === 'button' || sType === 'image')) ||
        (sTag === 'button' && (sType === '' || sType === 'submit'));
      if (!isButton) {
        throw new TypeError("Failed to construct 'FormData': the provided submitter is not a submit button");
      }
      var owner = null;
      try {
        var formAttr = submitter.getAttribute ? submitter.getAttribute('form') : null;
        if (formAttr) {
          owner = (typeof document !== 'undefined' && document.getElementById)
            ? document.getElementById(formAttr) : null;
        } else if (typeof submitter.closest === 'function') {
          owner = submitter.closest('form');
        }
      } catch (_eOwn) {}
      if (owner !== form) {
        throw new (globalThis.DOMException || Error)(
          "Failed to construct 'FormData': the provided submitter isn't owned by this form", 'NotFoundError');
      }
    }
    if (form.__zwFormDataConstructing) {
      throw new (globalThis.DOMException || Error)(
        "Failed to construct 'FormData': FormData is already being constructed", 'InvalidStateError');
    }
    form.__zwFormDataConstructing = true;
    try {
      // best-effort form 字段枚举（文档树序 + form owner 过滤——form= 属性关联的
      // 站外元素同入列，outerNamed 面；submit 按钮仅 submitter 本尊入列）。
      try {
        var fields = (typeof document !== 'undefined' && document.querySelectorAll)
          ? document.querySelectorAll('input, select, textarea, button')
          : form.querySelectorAll('input, select, textarea, button');
        var subKey = null;
        if (submitter != null && typeof _elKey === 'function') {
          try { subKey = _elKey(submitter.__zwSelector || null, submitter.__zwHandle || null); } catch (_eSk) {}
        }
        for (var i = 0; i < fields.length; i++) {
          var f = fields[i];
          // form owner 判定：form= 属性 → getElementById；否则 closest('form')。
          var owner = null;
          try {
            var formAttr = f.getAttribute ? f.getAttribute('form') : null;
            if (formAttr) {
              owner = (typeof document !== 'undefined' && document.getElementById)
                ? document.getElementById(formAttr) : null;
            } else if (typeof f.closest === 'function') {
              owner = f.closest('form');
            }
          } catch (_eOwn2) {}
          if (owner !== form) continue;
          var name = f.getAttribute ? f.getAttribute('name') : f.name;
          if (!name) continue;
          var type = ((f.getAttribute ? f.getAttribute('type') : f.type) || '').toLowerCase();
          var tag = String(f._realTag || f.tagName || '').toLowerCase();
          var fKey = null;
          try { fKey = (typeof _elKey === 'function') ? _elKey(f.__zwSelector || null, f.__zwHandle || null) : null; } catch (_eFk) {}
          var isSub = (subKey !== null && fKey !== null && subKey === fKey) || (f === submitter);
          if (tag === 'input' && (type === 'submit' || type === 'button')) {
            if (isSub) this._p.push([String(name), f.value != null ? String(f.value) : '', undefined]);
          } else if (tag === 'input' && type === 'image') {
            if (isSub) {
              this._p.push([String(name) + '.x', '0', undefined]);
              this._p.push([String(name) + '.y', '0', undefined]);
            }
          } else if (tag === 'button') {
            if (isSub) this._p.push([String(name), f.value != null ? String(f.value) : '', undefined]);
          } else if (type === 'checkbox' || type === 'radio') {
            if (f.checked) this._p.push([String(name), f.value != null ? String(f.value) : 'on', undefined]);
          } else if (type !== 'file' && type !== 'submit' && type !== 'button' && type !== 'reset' && type !== 'image') {
            this._p.push([String(name), f.value != null ? String(f.value) : '', undefined]);
          }
        }
      } catch (_e) { /* best-effort：枚举失败则按空 FormData */ }
      // 'formdata' 事件派发（e.formData 可变——handler 内增删条目对构造结果可见；
      // 构造结果为事件对象的副本）。
      try {
        if (typeof FormDataEvent === 'function' && typeof form.dispatchEvent === 'function') {
          var evFd = new FormData();
          for (var ci = 0; ci < this._p.length; ci++) evFd._p.push(this._p[ci]);
          var ev = new FormDataEvent('formdata', { formData: evFd });
          form.dispatchEvent(ev);
          this._p = evFd._p.slice();
        }
      } catch (_eFde) {}
    } finally {
      form.__zwFormDataConstructing = false;
    }
  };
  globalThis.FormData.prototype = {
    append: function (name, value, filename) {
      // net-api M4-S19：filename 仅对 Blob 值合法——非 Blob 值带 filename → TypeError
      //（spec append 步骤；testFormDataAppendToFormString/WrongPlatformObject 面）。
      if (filename != null && !(value instanceof Blob)) {
        throw new TypeError("Failed to execute 'append' on 'FormData': parameter 2 is not of type 'Blob'");
      }
      this._p.push(_zwFdEntry(name, value, filename));
    },
    delete: function (name) {
      name = String(name);
      this._p = this._p.filter(function (e) { return e[0] !== name; });
    },
    get: function (name) {
      name = String(name);
      for (var i = 0; i < this._p.length; i++) if (this._p[i][0] === name) return this._p[i][1];
      return null;
    },
    getAll: function (name) {
      name = String(name);
      var r = [];
      for (var i = 0; i < this._p.length; i++) if (this._p[i][0] === name) r.push(this._p[i][1]);
      return r;
    },
    has: function (name) {
      name = String(name);
      for (var i = 0; i < this._p.length; i++) if (this._p[i][0] === name) return true;
      return false;
    },
    set: function (name, value, filename) {
      // net-api M4-S20：filename 仅对 Blob 值合法（set 同 append——set-formelement 面）。
      if (filename != null && !(value instanceof Blob)) {
        throw new TypeError("Failed to execute 'set' on 'FormData': parameter 2 is not of type 'Blob'");
      }
      // R3014：Blob/File 值保真；替换所有同名 entry（首个替换为新值，余删除），无则追加。
      var entry = _zwFdEntry(name, value, filename);
      var found = false; var out = [];
      for (var i = 0; i < this._p.length; i++) {
        if (this._p[i][0] === entry[0]) { if (!found) { out.push(entry); found = true; } }
        else out.push(this._p[i]);
      }
      if (!found) out.push(entry);
      this._p = out;
    },
    // net-api M4-S20：forEach 走迭代器（WebIDL live 语义——迭代中 delete 的移位跳过面
    // 与 entries 迭代一致）。
    forEach: function (cb, thisArg) {
      var it = this.entries();
      var r = it.next();
      while (!r.done) {
        cb.call(thisArg, r.value[1], r.value[0], this);
        r = it.next();
      }
    },
    // net-api M4-S19：live cursor 迭代（WebIDL value pairs iterator——迭代中 delete 使
    // 后续元素前移被跳过、append 元素可达；formdata-iteration 三面）。
    entries: function () {
      var self = this; var i = 0;
      var it = { next: function () {
        if (i >= self._p.length) return { done: true, value: undefined };
        var e = self._p[i]; i++;
        return { done: false, value: [e[0], e[1]] };
      } };
      if (typeof Symbol !== 'undefined' && Symbol.iterator) it[Symbol.iterator] = function () { return it; };
      return it;
    },
    keys: function () {
      var self = this; var i = 0;
      var it = { next: function () {
        if (i >= self._p.length) return { done: true, value: undefined };
        var e = self._p[i]; i++;
        return { done: false, value: e[0] };
      } };
      if (typeof Symbol !== 'undefined' && Symbol.iterator) it[Symbol.iterator] = function () { return it; };
      return it;
    },
    values: function () {
      var self = this; var i = 0;
      var it = { next: function () {
        if (i >= self._p.length) return { done: true, value: undefined };
        var e = self._p[i]; i++;
        return { done: false, value: e[1] };
      } };
      if (typeof Symbol !== 'undefined' && Symbol.iterator) it[Symbol.iterator] = function () { return it; };
      return it;
    },
    // R3014：multipart/form-data 序列化——返 { body: Uint8Array, contentType }。boundary 唯一；
    // 字符串值→text part；Blob/File→file part（filename + Content-Type + _zw_blobBytes 字节）。
    // 供 fetch FormData body 接线（part01）+ 手动构建 multipart body。文本内容经 UTF-8 wire 保真。
    _zwMultipart: function () {
      var boundary = '----ZeroWebForm' + (_zwFdCounter++) + (typeof Math.random === 'function' ? Math.floor(Math.random() * 1e9).toString(36) : '');
      var parts = [];
      function pushStr(s) { var b = _zw_utf8_encode(s); for (var i = 0; i < b.length; i++) parts.push(b[i]); }
      for (var i = 0; i < this._p.length; i++) {
        var e = this._p[i];
        var name = e[0], value = e[1], filename = e[2];
        pushStr('--' + boundary + '\r\n');
        if (value != null && value instanceof Blob) {
          var fn = filename != null ? filename : (value.name != null ? String(value.name) : 'blob');
          var ct = value.type || 'application/octet-stream';
          pushStr('Content-Disposition: form-data; name="' + name + '"; filename="' + fn + '"\r\n');
          pushStr('Content-Type: ' + ct + '\r\n\r\n');
          var vb = _zw_blobBytes(value);
          for (var k = 0; k < vb.length; k++) parts.push(vb[k]);
          pushStr('\r\n');
        } else {
          pushStr('Content-Disposition: form-data; name="' + name + '"\r\n\r\n');
          pushStr(String(value));
          pushStr('\r\n');
        }
      }
      pushStr('--' + boundary + '--\r\n');
      var body = new Uint8Array(parts.length);
      for (var j = 0; j < parts.length; j++) body[j] = parts[j];
      return { body: body, contentType: 'multipart/form-data; boundary=' + boundary };
    }
  };
  // 自身可迭代（for (const [k,v] of formData)）：[Symbol.iterator] → entries。
  if (typeof Symbol !== 'undefined') {
    globalThis.FormData.prototype[Symbol.iterator] = globalThis.FormData.prototype.entries;
  }

  // Headers——HTTP 头集合（fetch / Service Worker / header-map 高频）。镜像 FormData pair-store
  // 模式，但**header name 小写归一**（spec：name 不区分大小写，规范化为小写）+ **多值 append 用
  // ', ' 合并**（spec：get 返非 Set-Cookie 头的值以 ', ' 连接）。纯 JS，零 host 回调。init 接受
  // record 对象 / [[name,value],...] 序列 / 另一 Headers。`getSetCookie` 返 Set-Cookie 数组（spec
  // 特例——get 合并 Set-Cookie 会丢多个 cookie 的分隔，故单独返数组）。
  // **已知限制（记录）**：① name 仅小写 + trim（不做 byte-value 严格校验，lenient）；② 迭代按插入序
  //   （spec 为字典序，浏览器实测为插入序——与主流一致）；③ entries/iteration 暴露**小写** name（spec
  //   一致）；④ 无 Headers 的 mutation 写回 fetch（fetch POST defer，本实现为构造/读/迭代）。
  function _hdrNorm(name) {
    return String(name).toLowerCase().trim();
  }
  // ── net-api M2-S2：Headers 校验 + guard 完整化（Fetch §5.1）────────────────────
  // header name = field-name token（HTTP token code point）；header value = ByteString
  //（≤0xFF）且无 0x00/HTTP newline、无首尾 HTTP tab/space。name/value 非法 → TypeError
  //（validate 阶段抛出，WPT headers-errors）；guard 阻断静默（append/set 返回、不抛）。
  var _ZW_HDR_TOKEN = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;
  function _zwIsByteString(s) {
    for (var i = 0; i < s.length; i++) {
      if (s.charCodeAt(i) > 0xFF) return false;
    }
    return true;
  }
  function _zwIsValidHeaderName(name) {
    return _zwIsByteString(name) && _ZW_HDR_TOKEN.test(name);
  }
  // Normalize（Fetch §2.2）：去首尾 HTTP whitespace（LF/CR/TAB/SPACE；注意非 ASCII
  // whitespace 集合——\x0c FF 不在内）。
  function _zwNormalizeHeaderValue(value) {
    return String(value).replace(/^[\t\n\r ]+/, '').replace(/[\t\n\r ]+$/, '');
  }
  function _zwIsValidHeaderValue(value) {
    if (!_zwIsByteString(value)) return false;
    if (/[\x00\n\r]/.test(value)) return false;
    if (/^[\t ]|[\t ]$/.test(value)) return false;
    return true;
  }
  // CORS-safelisted request-header（Fetch §2.2.2）——request-no-cors guard 写侧判定。
  function _zwHasCorsUnsafeByte(value) {
    for (var i = 0; i < value.length; i++) {
      var c = value.charCodeAt(i);
      if ((c < 0x20 && c !== 0x09) || c === 0x22 || c === 0x28 || c === 0x29 || c === 0x3A ||
          c === 0x3C || c === 0x3E || c === 0x3F || c === 0x40 || c === 0x5B || c === 0x5C ||
          c === 0x5D || c === 0x7B || c === 0x7D || c === 0x7F) return true;
    }
    return false;
  }
  function _zwIsCorsSafelistedReqHeader(ln, value) {
    if (value.length > 128) return false;
    if (ln === 'accept') return !_zwHasCorsUnsafeByte(value);
    if (ln === 'accept-language' || ln === 'content-language') {
      if (_zwHasCorsUnsafeByte(value)) return false;
      for (var i = 0; i < value.length; i++) {
        var c = value.charCodeAt(i);
        var ok = (c >= 0x30 && c <= 0x39) || (c >= 0x41 && c <= 0x5A) || (c >= 0x61 && c <= 0x7A) ||
                 c === 0x20 || c === 0x2A || c === 0x2C || c === 0x2D || c === 0x2E || c === 0x3B || c === 0x3D;
        if (!ok) return false;
      }
      return true;
    }
    if (ln === 'content-type') {
      if (_zwHasCorsUnsafeByte(value)) return false;
      var m = _zwParseMimeType(value);
      if (!m) return false;
      var essence = m.type + '/' + m.subtype;
      return essence === 'application/x-www-form-urlencoded' || essence === 'multipart/form-data' || essence === 'text/plain';
    }
    return false;
  }
  var _ZW_NO_CORS_SAFELISTED = { 'accept': 1, 'accept-language': 1, 'content-language': 1, 'content-type': 1 };
  var _ZW_PRIVILEGED_NO_CORS = { 'range': 1 };
  // get, decode, and split（Fetch §2.2.2——quoted-string 感知逗号切分；X-HTTP-Method-Override
  // 族 forbidden 判定的值解析面）。返切分值数组。
  function _zwGetDecodeSplit(value) {
    value = String(value);
    var values = [];
    var temp = '';
    var pos = 0;
    for (;;) {
      while (pos < value.length && value.charAt(pos) !== '"' && value.charAt(pos) !== ',') {
        temp += value.charAt(pos); pos++;
      }
      if (pos < value.length && value.charAt(pos) === '"') {
        pos++;
        for (;;) {
          while (pos < value.length && value.charAt(pos) !== '"' && value.charAt(pos) !== '\\') {
            temp += value.charAt(pos); pos++;
          }
          if (pos >= value.length) break;
          var qc = value.charAt(pos); pos++;
          if (qc === '\\') {
            if (pos >= value.length) { temp += '\\'; break; }
            temp += value.charAt(pos); pos++;
          } else break;
        }
        if (pos < value.length) continue;
      }
      temp = temp.replace(/^[\t\n\r ]+/, '').replace(/[\t\n\r ]+$/, '');
      values.push(temp);
      temp = '';
      if (pos >= value.length) return values;
      pos++; // past ','
    }
  }
  // R3221：Fetch §3.4.4 forbidden request-header names——JS 不可设（浏览器托管）。
  // `_headersToWire` 出口过滤，保证 JS 设的禁止头永不到达 host/服务器。ln 为已小写归一的 name。
  // https://fetch.spec.whatwg.org/#forbidden-header-name
  var _ZW_FORBIDDEN_REQ_HEADERS = {
    'accept-charset': 1, 'accept-encoding': 1,
    'access-control-request-headers': 1, 'access-control-request-method': 1,
    'connection': 1, 'content-length': 1, 'cookie': 1, 'cookie2': 1,
    'date': 1, 'dnt': 1, 'expect': 1, 'host': 1, 'keep-alive': 1,
    'origin': 1, 'referer': 1, 'te': 1, 'trailer': 1,
    'transfer-encoding': 1, 'upgrade': 1, 'via': 1
  };
  function _zwIsForbiddenReqHeader(ln, value) {
    if (_ZW_FORBIDDEN_REQ_HEADERS[ln]) return true;
    // 前缀匹配（byte-case-insensitive，ln 已小写）：Proxy- / Sec-
    if (ln.slice(0, 6) === 'proxy-' || ln.slice(0, 4) === 'sec-') return true;
    // M2-S2：X-HTTP-Method(-Override) 族——value 经 get-decode-split 任一值为 forbidden
    // method（CONNECT/TRACE/TRACK，byte-case-insensitive）→ forbidden（headers-forbidden-
    // override 面——`r.headers.append(override, '\rtrace')` 须静默丢弃）。
    if (ln === 'x-http-method-override' || ln === 'x-http-method' || ln === 'x-method-override') {
      if (value == null) return false;
      var parts = _zwGetDecodeSplit(value);
      for (var i = 0; i < parts.length; i++) {
        var p = parts[i].toUpperCase();
        if (p === 'CONNECT' || p === 'TRACE' || p === 'TRACK') return true;
      }
    }
    return false;
  }
  // R3222：Fetch §3.4.5 forbidden response-header names——response Headers 的 get/has/iterate 不暴露，
  // 但 getSetCookie 仍返 set-cookie 数组（spec 特例）。`_guard`='response' 由 Response ctor 设。
  // https://fetch.spec.whatwg.org/#forbidden-response-header-name
  function _hdrIsForbiddenResponse(ln) {
    return ln === 'set-cookie' || ln === 'set-cookie2';
  }
  // R3223 + M2-S2：Headers guard 写侧阻断（Fetch §5.2）——request guard 阻 forbidden
  // request-header（含 X-HTTP-Method-Override 值判定，需 value）；response guard 阻
  // forbidden response-header；none 不阻。request-no-cors：append/set 非 CORS-safelisted
  // 阻断（append 按 combine 后整体判定——调用方传 combine 值）、delete 非 safelisted 且非
  // privileged 阻断。
  function _hdrGuardBlocks(guard, ln, value) {
    if (guard === 'request') return _zwIsForbiddenReqHeader(ln, value);
    if (guard === 'request-no-cors') {
      if (_ZW_PRIVILEGED_NO_CORS[ln]) return true; // privileged no-CORS（写侧恒阻）
      if (value != null) return !_zwIsCorsSafelistedReqHeader(ln, value);
      return !_ZW_NO_CORS_SAFELISTED[ln];
    }
    if (guard === 'response') return _hdrIsForbiddenResponse(ln);
    return false;
  }
  // remove privileged no-CORS request-headers（Fetch §5.1——no-cors guard 变更后清 Range）。
  function _zwRemovePrivilegedNoCors(h) {
    if (h._guard === 'request-no-cors') delete h._h['range'];
  }
  // validate a header（Fetch §5.1）：name/value 非法 → TypeError（guard immutable → false）。
  // 调用方在 normalize value 之后调用。
  function _zwValidateHeader(name, value, guard) {
    if (!_zwIsValidHeaderName(name)) throw new TypeError('Invalid header name: ' + name);
    if (!_zwIsValidHeaderValue(value)) throw new TypeError('Invalid header value');
    if (guard === 'immutable') return false;
    return true;
  }
  // R3223：Headers fill——逐值 append（尊重目标 guard）。供 Headers ctor（guard none）与 Request ctor
  //（guard request）复用。Headers 实例源直接迭代内部 _h（Fetch §5.1「for each header in init's header list」
  // 指内部列表，含 response guard 隐藏的 Set-Cookie；目标 guard 决定是否过滤）。
  function _fillHeaders(h, init) {
    if (init == null) return;
    // M2-S2：**自身** Symbol.iterator 优先（Web IDL sequence 判定先于 record——
    // headers-basic「existing headers with custom iterator」在 Headers 实例上覆写
    // own iterator；普通 Headers 走下方 _h 分支保内部列表语义含 guard 隐藏 set-cookie）。
    if (typeof Symbol !== 'undefined' && init[Symbol.iterator] &&
        Object.prototype.hasOwnProperty.call(init, Symbol.iterator) &&
        typeof init[Symbol.iterator] === 'function') {
      var it0 = init[Symbol.iterator]();
      for (;;) {
        var step0 = it0.next();
        if (step0 && step0.done) break;
        var p0 = step0.value;
        if (!p0 || p0.length !== 2) throw new TypeError('Headers init sequence item must have exactly 2 elements');
        h.append(p0[0], p0[1]);
      }
      return;
    }
    if (init._h) {
      for (var k in init._h) {
        if (!Object.prototype.hasOwnProperty.call(init._h, k)) continue;
        var vals = init._h[k];
        for (var vi = 0; vi < vals.length; vi++) h.append(k, vals[vi]);
      }
      return;
    }
    if (Array.isArray(init)) {
      // M2-S2：sequence 形态逐项须恰为二元组（Fetch fill——size ≠ 2 → TypeError，
      // headers-errors `new Headers([["name"]])` / 三元组用例）。
      for (var i = 0; i < init.length; i++) {
        var pair = init[i];
        if (!pair || !pair.length || pair.length !== 2) {
          throw new TypeError('Headers init sequence item must have exactly 2 elements');
        }
        h.append(pair[0], pair[1]);
      }
    } else if (typeof Symbol !== 'undefined' && init[Symbol.iterator] && typeof init[Symbol.iterator] === 'function') {
      // M2-S2：自定义 iterable（sequence 形态）。Headers 自身也带 Symbol.iterator
      //（entries，非 own）——迭代产出 [k, v] 二元组走 append 同路。
      var it = init[Symbol.iterator]();
      for (;;) {
        var step = it.next();
        if (step && step.done) break;
        var p = step.value;
        if (!p || p.length !== 2) throw new TypeError('Headers init sequence item must have exactly 2 elements');
        h.append(p[0], p[1]);
      }
      return;
    } else if (typeof init.forEach === 'function') {
      // Headers-like（forEach 回调 (value, name, headers)）。
      init.forEach(function (v, k) { h.append(k, v); });
    } else if (typeof init === 'object') {
      for (var k in init) {
        if (!Object.prototype.hasOwnProperty.call(init, k)) continue;
        // R3222：多值头（_parseHeadersWire 累加的 Set-Cookie 数组）逐值 append。
        var vs = init[k];
        if (Array.isArray(vs)) {
          for (var vi = 0; vi < vs.length; vi++) h.append(k, vs[vi]);
        } else {
          h.append(k, vs);
        }
      }
    }
  }
  globalThis.Headers = globalThis.Headers || function Headers(init) {
    if (!(this instanceof Headers)) return new Headers(init);
    // M2-S2：Web IDL 可选参数语义——undefined 即缺省（不填充）；null / 非对象（1、字符串
    // 等非 sequence/record）→ TypeError（headers-basic「with null/1 should throw」）。
    if (init === null || (init !== undefined && typeof init !== 'object')) {
      throw new TypeError('Headers init must be a sequence or record');
    }
    this._h = {}; // lowername -> string[]（保 append 序与多值）
    this._guard = 'none'; // R3223：guard none/request/response（Fetch §5.1）；ctor 构造为 none（不过滤）
    if (init != null) _fillHeaders(this, init); // guard none → 不过滤禁止头
  };
  globalThis.Headers.prototype = {
    // M2-S2：append/set/get/has/delete 全走 name/value 校验（非法 → TypeError）；
    // value 先 Normalize（首尾 HTTP whitespace 剥除）再 validate；guard 阻断静默返回。
    append: function (name, value) {
      name = String(name).toLowerCase();
      value = _zwNormalizeHeaderValue(value);
      if (!_zwValidateHeader(name, value, this._guard)) return;
      // request-no-cors：combine 后整体 safelist 判定（spec append step 3）。
      var checkValue = value;
      if (this._guard === 'request-no-cors') {
        var existing = this._h[name];
        checkValue = existing && existing.length ? existing.join(', ') + ', ' + value : value;
      }
      if (_hdrGuardBlocks(this._guard, name, checkValue)) return;
      (this._h[name] = this._h[name] || []).push(value);
      _zwRemovePrivilegedNoCors(this);
    },
    delete: function (name) {
      name = String(name).toLowerCase();
      if (!_zwIsValidHeaderName(name)) throw new TypeError('Invalid header name: ' + name);
      // R3223：禁止头经 guard 不可删（Fetch §5.4 delete step 3/5；request guard 下本就未存，response guard 护 Set-Cookie）。
      if (_hdrGuardBlocks(this._guard, name)) return;
      delete this._h[name];
      _zwRemovePrivilegedNoCors(this);
    },
    get: function (name) {
      name = String(name).toLowerCase();
      if (!_zwIsValidHeaderName(name)) throw new TypeError('Invalid header name: ' + name);
      // R3222：response guard 不暴露 Set-Cookie/Set-Cookie2（Fetch §3.4.5）。
      if (this._guard === 'response' && _hdrIsForbiddenResponse(name)) return null;
      var v = this._h[name];
      return v && v.length ? v.join(', ') : null;
    },
    // getSetCookie：Set-Cookie 数组（spec 特例——get 合并 Set-Cookie 丢分隔，故单独返数组）。
    // R3222：response guard 下 getSetCookie 仍返 set-cookie 数组（forbidden-response 不影响此 API）。
    getSetCookie: function () {
      var v = this._h['set-cookie'];
      return v ? v.slice() : [];
    },
    has: function (name) {
      name = String(name).toLowerCase();
      if (!_zwIsValidHeaderName(name)) throw new TypeError('Invalid header name: ' + name);
      if (this._guard === 'response' && _hdrIsForbiddenResponse(name)) return false;
      return Object.prototype.hasOwnProperty.call(this._h, name);
    },
    set: function (name, value) {
      name = String(name).toLowerCase();
      value = _zwNormalizeHeaderValue(value);
      if (!_zwValidateHeader(name, value, this._guard)) return;
      // R3223：guard 写侧阻断（同 append；no-cors 按单值 safelist 判定——spec set step 3）。
      if (_hdrGuardBlocks(this._guard, name, value)) return;
      this._h[name] = [value];
      _zwRemovePrivilegedNoCors(this);
    },
    // value pairs to iterate（sort-and-combine，Fetch §5.1）：除 set-cookie 外多值合并为
    // 单对；set-cookie 各值独立成对（header-setcookie「iterator does not combine
    // set-cookie」面）。response guard 下 set-cookie/set-cookie2 不暴露（R3222）。
    _zwPairs: function () {
      var out = [];
      for (var k in this._h) {
        if (!Object.prototype.hasOwnProperty.call(this._h, k)) continue;
        if (this._guard === 'response' && _hdrIsForbiddenResponse(k)) continue;
        var vals = this._h[k];
        if (k === 'set-cookie') {
          for (var vi = 0; vi < vals.length; vi++) out.push([k, vals[vi]]);
        } else {
          out.push([k, vals.join(', ')]);
        }
      }
      // sort-and-combine（Fetch §5.1）：value pairs 按 name 升序（byte less than；
      // set-cookie 多值同序名稳定——ES2019 sort 稳定性保持 append 序）。
      out.sort(function (a, b) { return a[0] < b[0] ? -1 : (a[0] > b[0] ? 1 : 0); });
      return out;
    },
    forEach: function (cb, thisArg) {
      var pairs = this._zwPairs();
      for (var i = 0; i < pairs.length; i++) cb.call(thisArg, pairs[i][1], pairs[i][0], this);
    },
    entries: function () {
      var self = this;
      return _zwHeadersLiveIter(function () { return self._zwPairs(); }, function (p) { return [p[0], p[1]]; });
    },
    keys: function () {
      var self = this;
      return _zwHeadersLiveIter(function () { return self._zwPairs(); }, function (p) { return p[0]; });
    },
    values: function () {
      var self = this;
      return _zwHeadersLiveIter(function () { return self._zwPairs(); }, function (p) { return p[1]; });
    }
  };
  // 自身可迭代（for (const [k,v] of headers)）：[Symbol.iterator] → entries。
  // M2-S2：iterator 原型链对齐 %ArrayIteratorPrototype% 链（headers-basic
  // checkIteratorProperties——next 可写/可枚举/可配置 + 外层 proto === 迭代器迭代器原型）
  // + **live 语义**（每次 next 基于当前 header list 重算 pairs——header-setcookie
  // 「iterator is correctly updated with set-cookie changes」光标推进面）。
  var _ZW_HEADERS_ITER_PROTO = (typeof Symbol !== 'undefined')
    ? Object.create(Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]())))
    : Object.prototype;
  if (typeof Symbol !== 'undefined') {
    Object.defineProperty(_ZW_HEADERS_ITER_PROTO, 'next', {
      writable: true, enumerable: true, configurable: true,
      value: function () { return { done: true, value: undefined }; }
    });
  }
  function _zwHeadersLiveIter(getPairs, pick) {
    var pos = 0;
    var it = Object.create(_ZW_HEADERS_ITER_PROTO);
    it.next = function () {
      var pairs = getPairs();
      if (pos >= pairs.length) return { done: true, value: undefined };
      var value = pick(pairs[pos]);
      pos++;
      return { done: false, value: value };
    };
    if (typeof Symbol !== 'undefined') it[Symbol.iterator] = function () { return it; };
    return it;
  }
  if (typeof Symbol !== 'undefined') {
    globalThis.Headers.prototype[Symbol.iterator] = globalThis.Headers.prototype.entries;
    if (Symbol.toStringTag) {
      Object.defineProperty(globalThis.Headers.prototype, Symbol.toStringTag, {
        value: 'Headers',
        configurable: true
      });
    }
  }

  // Blob——不可变二进制数据容器（文件上传 / 下载 / object URL 高频）。纯 JS：parts 为
  // [string|ArrayBuffer|TypedArray|DataView|Blob]；size = 各 part 字节长之和；type = options.type（小写）。
  // text()/arrayBuffer() 返 Promise（V8 原生 Promise + execute 末 microtask checkpoint drain）。
  // R3011：真字节级物化——_zw_partBytes 同步把 part 转 Uint8Array（string→UTF-8 / 字节视图→拷贝 / Blob→递归），
  // _zw_blobBytes 拼接全 part 字节。slice 返真字节范围（旧浅拷全内容）、arrayBuffer/stream 返真字节（二进制
  // TypedArray part 不再经 text() UTF-8 往返损坏）。string 内容行为同旧（UTF-8 字节 == text→encode）。
  // **已知限制（记录）**：① arrayBuffer() 返 Uint8Array（spec ArrayBuffer，既有接口保留，库多按索引访问）；
  //   ② end-encoding 的 type 不解析 charset（原样小写）；③ slice 物化全字节 O(n)（典型用量可接受）。
  function _zw_partBytes(p) {
    if (p == null) return new Uint8Array(0);
    if (typeof p === 'string') {
      var enc = _zw_utf8_encode(p);
      var a = new Uint8Array(enc.length);
      for (var i = 0; i < enc.length; i++) a[i] = enc[i];
      return a;
    }
    if (p instanceof ArrayBuffer) return new Uint8Array(p);
    if (p.buffer instanceof ArrayBuffer) {
      // TypedArray / DataView：取其字节范围（byteOffset/byteLength）拷贝（避免视图别名）。
      var off = p.byteOffset || 0;
      return new Uint8Array(p.buffer.slice(off, off + (p.byteLength || 0)));
    }
    if (p instanceof Blob) return _zw_blobBytes(p); // 递归物化 Blob part
    return new Uint8Array(0);
  }
  function _zw_blobBytes(blob) {
    var parts = blob._parts || [];
    var chunks = [];
    var total = 0;
    for (var i = 0; i < parts.length; i++) {
      var b = _zw_partBytes(parts[i]);
      chunks.push(b);
      total += b.length;
    }
    var out = new Uint8Array(total);
    var off = 0;
    for (var j = 0; j < chunks.length; j++) { out.set(chunks[j], off); off += chunks[j].length; }
    return out;
  }
  var _zwBlobStore = {}; // url → Blob（createObjectURL 注册表，revokeObjectURL 清理）
  globalThis.Blob = globalThis.Blob || function Blob(parts, options) {
    if (!(this instanceof Blob)) return new Blob(parts, options);
    parts = parts || [];
    this._parts = parts;
    var size = 0;
    for (var i = 0; i < parts.length; i++) size += Blob._partSize(parts[i]);
    this.size = size;
    // net-api M2-S1：type 经 MIME parse 失败 → 空串，成功 → 序列化形态（FileAPI Blob
    // constructor + mimesniff mime-types.json——'x/x; bonus=x' → 'x/x;bonus=x'、
    // charset=" gbk" 引号保留）。_zwParseMimeType/_zwSerializeMimeType 为 part01 同
    // IIFE 作用域共享函数。
    var typeRaw = (options && options.type != null) ? String(options.type) : '';
    var typeParsed = _zwParseMimeType(typeRaw);
    this.type = typeParsed ? _zwSerializeMimeType(typeParsed) : '';
  };
  // part 字节长：string→UTF-8；ArrayBuffer/TypedArray/DataView→byteLength；Blob→size；余 0。
  globalThis.Blob._partSize = function (p) {
    if (p == null) return 0;
    if (typeof p === 'string') return _zw_utf8_encode(p).length;
    if (p.byteLength != null) return p.byteLength | 0; // ArrayBuffer / TypedArray / DataView
    if (p.size != null) return p.size | 0;             // Blob
    return 0;
  };
  // part → 文本（用于 text() 拼接）：string 原样；TypedArray/ArrayBuffer 经 TextDecoder；Blob 递归（Promise）。
  globalThis.Blob._partText = function (p) {
    if (typeof p === 'string') return p;
    if (p == null) return '';
    if (p instanceof ArrayBuffer || p.buffer != null || typeof p.length === 'number') {
      return new TextDecoder().decode(p);
    }
    if (p instanceof Blob) return p.text(); // Promise<string>（递归）
    return '';
  };
  globalThis.Blob.prototype = {
    // R3011：slice 返真字节范围（物化全字节 → 取 [start,end) → 包成单 Uint8Array part 的 Blob）。
    // 旧浅拷 _parts（slice().text() 返全内容）；现跨 part 边界正确。start/end 负值相对末尾，clamp + type 重设。
    slice: function (start, end, contentType) {
      var s = start != null ? (start | 0) : 0;
      if (s < 0) s = Math.max(0, this.size + s);
      var e = end != null ? (end | 0) : this.size;
      if (e < 0) e = Math.max(0, this.size + e);
      e = Math.min(e, this.size);
      if (s > e) s = e; // spec：start > end → 空 Blob
      var sliced = _zw_blobBytes(this).slice(s, e); // 真字节范围（Uint8Array.slice 拷贝）
      var b = new Blob([], { type: contentType != null ? String(contentType) : this.type });
      b._parts = [sliced];
      b.size = sliced.length;
      return b;
    },
    // text()：Promise<string>——拼接各 part 文本（string/字节经 TextDecoder/Blob 递归）。
    text: function () {
      var parts = this._parts;
      var pro = [];
      for (var i = 0; i < parts.length; i++) pro.push(Blob._partText(parts[i]));
      return Promise.all(pro).then(function (strs) { return strs.join(''); });
    },
    // R3011：arrayBuffer() 返真拼接字节（二进制 TypedArray part 不再经 text() UTF-8 往返损坏）。
    // 返 Uint8Array（spec ArrayBuffer，既有接口保留——库多按 .length/索引访问）。
    arrayBuffer: function () {
      return Promise.resolve(_zw_blobBytes(this));
    },
    // R3011：stream() 单真字节 chunk 后 close（二进制保真；常配 pipeThrough(TextDecoderStream)）。
    stream: function () {
      var self = this;
      var done = false;
      return new ReadableStream({
        pull: function (controller) {
          if (done) { controller.close(); return; }
          done = true;
          try {
            var bytes = _zw_blobBytes(self);
            if (bytes.length > 0) controller.enqueue(bytes);
            controller.close();
          } catch (e) { controller.error(e); }
        }
      });
    }
  };

  // File——Blob 子类 + 文件名/时间戳（`<input type=file>` / 文件上传构造高频）。完成 Blob→File→
  // FileReader→FormData 文件处理簇。constructor 复用 `Blob.call(this, parts, options)`（File 实例
  // `instanceof Blob` 为真，故 Blob 构造体在 this 上设 `_parts`/`size`/`type`），再加 `name`/
  // `lastModified`（默认 `Date.now()`，V8 原生单调时钟）/`lastModifiedDate`（deprecated 但常见）。
  // prototype = Object.create(Blob.prototype) → 继承 slice/text/arrayBuffer；File is-a Blob 故
  // FormData.append(name, file) / FileReader.readAsDataURL(file) 自动互通。
  // **已知限制（记录）**：① `lastModifiedDate` 取 lastModified 构造（spec 已 deprecated 但库仍读）；
  //   ② 无 webkitRelativePath（目录上传，rare，defer）；③ 不校验 name 非空（spec 允许空名）。
  function File(parts, name, options) {
    if (!(this instanceof File)) return new File(parts, name, options);
    Blob.call(this, parts, options); // 复用 Blob 构造（this instanceof Blob 为真 → 设 _parts/size/type）
    this.name = name == null ? '' : String(name);
    this.lastModified = (options && options.lastModified != null) ? +options.lastModified : Date.now();
    this.lastModifiedDate = new Date(this.lastModified);
  }
  File.prototype = Object.create(Blob.prototype);
  File.prototype.constructor = File;
  globalThis.File = globalThis.File || File;

  // FileReader——异步读 Blob（文件上传 / 图片预览 / data URL 高频）。纯 JS，builds on Blob.text()/
  // arrayBuffer()（R2789）+ btoa（R2770）。**readAsDataURL 为 Blob 未覆盖的唯一能力**（图片预览
  // `img.src = reader.result` 高频）。事件经 microtask：readyState=LOADING（同步）→ loadstart（同步）
  // → Blob Promise resolve（execute 末 checkpoint drain）→ result 赋值 + readyState=DONE → load + loadend。
  // **已知限制（记录）**：① loadstart 同步派发（spec 为 task 异步，多数代码只关心 load/loadend，零影响）；
  //   ② 无真 abort（abort 仅置 readyState=DONE + 派发 abort/loadend，不中断已 in-flight 的 Blob Promise——
  //   纯 JS 无取消原语，best-effort）；③ encoding 参数忽略（恒 UTF-8，同 TextDecoder 限制）；
  //   ④ 不扩展 EventTarget（仅 onXxx handler 属性，非 addEventListener——覆盖 `reader.onload = ...` 主流用法）；
  //   ⑤ readAsDataURL 对非 Latin-1 字节按逐字节 Latin-1→btoa（与 spec 一致：base64 编码原始字节）。
  function FileReader() {
    this.readyState = 0; // EMPTY
    this.result = null;
    this.error = null;
    this.onloadstart = null;
    this.onprogress = null;
    this.onload = null;
    this.onabort = null;
    this.onerror = null;
    this.onloadend = null;
  }
  FileReader.EMPTY = 0;
  FileReader.LOADING = 1;
  FileReader.DONE = 2;
  FileReader.prototype.EMPTY = 0;
  FileReader.prototype.LOADING = 1;
  FileReader.prototype.DONE = 2;
  // 派发命名事件：构造 ProgressEvent-like {type,target,lengthComputable,loaded,total}，调 onXxx handler。
  FileReader.prototype._fire = function (type, loaded, total) {
    var ev = {
      type: type,
      target: this,
      lengthComputable: total != null && total >= 0,
      loaded: loaded || 0,
      total: total != null ? total : 0
    };
    var h = this['on' + type];
    if (typeof h === 'function') {
      try { h.call(this, ev); } catch (_e) { /* handler 异常不中断读取流程 */ }
    }
  };
  // 读取启动：readyState=LOADING + 派发 loadstart（同步）。
  FileReader.prototype._start = function (blob) {
    this.readyState = 1;
    this.result = null;
    this.error = null;
    this._total = (blob && blob.size != null) ? blob.size : 0;
    this._fire('loadstart', 0, this._total);
  };
  // 读取成功收尾：result 赋值 + readyState=DONE + 派发 load + loadend。
  FileReader.prototype._done = function (result) {
    this.readyState = 2;
    this.result = result;
    this._fire('progress', this._total, this._total);
    this._fire('load', this._total, this._total);
    this._fire('loadend', this._total, this._total);
  };
  // 读取失败收尾：error 赋值 + readyState=DONE + 派发 error + loadend。
  FileReader.prototype._fail = function (err) {
    this.readyState = 2;
    this.error = err;
    this._fire('error', 0, this._total);
    this._fire('loadend', 0, this._total);
  };
  FileReader.prototype.readAsText = function (blob /*, encoding */) {
    var self = this;
    this._start(blob);
    blob.text().then(function (s) { self._done(s); }, function (e) { self._fail(e); });
    return; // void（spec）
  };
  FileReader.prototype.readAsArrayBuffer = function (blob) {
    var self = this;
    this._start(blob);
    blob.arrayBuffer().then(function (a) { self._done(a); }, function (e) { self._fail(e); });
  };
  // readAsBinaryString：逐字节 Latin-1 串（spec 保留方法，已弃用但仍可用）。
  FileReader.prototype.readAsBinaryString = function (blob) {
    var self = this;
    this._start(blob);
    blob.arrayBuffer().then(function (buf) {
      var s = '';
      for (var i = 0; i < buf.length; i++) s += String.fromCharCode(buf[i]);
      self._done(s);
    }, function (e) { self._fail(e); });
  };
  // readAsDataURL：data:<type>;base64,<b64>——逐字节 Latin-1 → btoa（base64 编码原始字节，spec 一致）。
  FileReader.prototype.readAsDataURL = function (blob) {
    var self = this;
    this._start(blob);
    blob.arrayBuffer().then(function (buf) {
      var s = '';
      for (var i = 0; i < buf.length; i++) s += String.fromCharCode(buf[i]);
      var type = (blob && blob.type) || '';
      self._done('data:' + type + ';base64,' + btoa(s));
    }, function (e) { self._fail(e); });
  };
  // abort：best-effort——仅 EMPTY/DONE 时 no-op；否则置 DONE + 派发 abort + loadend（不中断 in-flight Promise）。
  FileReader.prototype.abort = function () {
    if (this.readyState === 0 || this.readyState === 2) return;
    this.readyState = 2;
    this.result = null;
    this._fire('abort', 0, this._total);
    this._fire('loadend', 0, this._total);
  };
  globalThis.FileReader = globalThis.FileReader || FileReader;

  // DOMParser——解析 HTML/XML 串为只读 Document（模板引擎 / sanitizer / RSS / 服务端 HTML 高频）。
  // 委托 host `__zw_parse_html_query(html, selector, all)`（dom::parse_html + selector 引擎），返 JSON
  // 元素快照数组；shim 包成 `_zwParsedDoc`（Document-like）+ `_zwParseEl`（只读 element-proxy）。
  // **关键设计**：解析的文档不在 dom_html 快照中（与页面 DOM 隔离），故 querySelector/getElementById/
  // body 经 host 回调每次**重解析** + 取快照，而非走唯一选择器（无处落地）。子树 query 重解析元素 outerHTML。
  // **已知限制（记录）**：① **只读**——element-proxy 不支持 setAttribute/appendChild/innerHTML setter
  //   等 mutation（spec DOMParser 文档可改，但本实现面向读场景；mutation 需 host 写路径，follow-up）；
  //   ② XML/SVG mimeType 统一按 HTML 解析（容错，非 well-formed 不报错）；③ innerHTML 由 outerHTML 派生
  //   （strip 首/尾 tag，void 元素正确返 ''）；④ getElementById 用 `#id` 选择器（id 含特殊字符未转义，
  //   best-effort）；⑤ textContent/getAttribute 只读快照值（无 live 更新）。
  // host 未注册（reftest/纯 sandbox）→ DOMParser 仍可构造，querySelector 返 null（no-throw，零回归）。
  // R158（js-dom M4）：查询入口的非法选择器守卫——spec `dom-parentnode-
  // queryselector` 对 parse 失败的选择器抛 SyntaxError DOMException（WPT
  // runInvalidSelectorTest 断言 root.querySelector/All 双入口；`__zw_selector_valid`
  // 探针 R156 已建）。全查询入口（_zwParseEl / detached doc / Element.prototype /
  // fragment / 主 document）统一消费。
  globalThis._zwQueryGuard = function (sel, nArgs) {
    // R158：无参 TypeError（spec WebIDL 2 必参——WPT "no parameter" 断言；
    // `querySelector(undefined)` 是 1 参 undefined → String 形态 "undefined" 合法
    // type selector 查询，与浏览器一致不抛——仅真无参才 TypeError）。
    if (nArgs === 0) {
      throw new globalThis.TypeError(
        "Failed to execute 'querySelector' on 'ParentNode': 1 argument required, but only 0 present.");
    }
    if (typeof __zw_selector_valid === 'function' && __zw_selector_valid(String(sel)) !== '1') {
      throw new (globalThis.DOMException || Error)(
        "'" + String(sel) + "' is not a valid selector.", 'SyntaxError');
    }
  };

  function _zwParseEl(info) {
    info = info || {};
    var tag = info.tag || '';
    this.nodeType = 1;
    this.tagName = tag.toUpperCase();
    this.nodeName = this.tagName;
    this.localName = tag;
    this.id = info.id || '';
    this.className = info.cls || '';
    this.textContent = info.text || '';
    this.outerHTML = info.outer || '';
    this.innerHTML = _zwInnerFromOuter(this.outerHTML, tag);
    this._attrs = info.attrs || {};
    // R112：祖先链（host path 字段——身份键数组，根→父，\x1f 分隔）。事件派发沿此链
    // 反查注册视图（WPT Event-dispatch-bubbles）。
    this._zwPath = info.path ? String(info.path).split('\x1f') : [];
  }
  // R156（js-dom M4）：Node 接口常量挂解析元素原型（WPT interfaceCheckMatches 的
  // `obj.nodeType === obj.ELEMENT_NODE` 分支判定——解析元素缺常量走错 "should not
  // support" 分支）。与 part03 Node.prototype 常量表同源。
  (function () {
    var _nc = { ELEMENT_NODE: 1, ATTRIBUTE_NODE: 2, TEXT_NODE: 3, CDATA_SECTION_NODE: 4,
      ENTITY_REFERENCE_NODE: 5, ENTITY_NODE: 6, PROCESSING_INSTRUCTION_NODE: 7,
      COMMENT_NODE: 8, DOCUMENT_NODE: 9, DOCUMENT_TYPE_NODE: 10, DOCUMENT_FRAGMENT_NODE: 11,
      NOTATION_NODE: 12 };
    for (var _k in _nc) {
      if (Object.prototype.hasOwnProperty.call(_nc, _k)) {
        Object.defineProperty(_zwParseEl.prototype, _k, { value: _nc[_k], enumerable: false });
      }
    }
  })();
  // innerHTML 从 outerHTML 派生：strip 首 `<tag ...>` + 尾 `</tag>`；void/自闭合无尾标签 → strip 首标签后剩 ''。
  function _zwInnerFromOuter(outer, tag) {
    if (!outer || !tag) return '';
    var s = outer.replace(new RegExp('^<' + tag + '\\b[^>]*>', 'i'), '');
    return s.replace(new RegExp('</' + tag + '\\s*>$', 'i'), '');
  }
  _zwParseEl.prototype.getAttribute = function (name) {
    name = String(name);
    return Object.prototype.hasOwnProperty.call(this._attrs, name) ? this._attrs[name] : null;
  };
  // R156（js-dom M4）：ownerDocument 缺省回落（WPT Element-matches 的
  // runSpecialMatchesTests 断言 `element.ownerDocument.defaultView.TypeError`——
  // 解析元素缺 ownerDocument 直接 'reading defaultView' TypeError 整 subtest 崩）。
  // 查询工厂（detached doc 的 queryOne/queryAll）在产物上设 `_zwOwnerDoc` 精确指向
  // 源文档；未设时回落主 document。
  try {
    Object.defineProperty(_zwParseEl.prototype, 'ownerDocument', {
      configurable: true,
      get: function () {
        if (this._zwOwnerDoc) return this._zwOwnerDoc;
        return globalThis.document;
      },
    });
  } catch (_e156od) {}
  _zwParseEl.prototype.hasAttribute = function (name) {
    return Object.prototype.hasOwnProperty.call(this._attrs, String(name));
  };
  // 子树 query：重解析本元素 outerHTML（host 二次 parse + select），返只读 element-proxy。
  // R156（js-dom M4）：matches（WPT Element-matches 的 `element[method](q, refNode)`
  // 对 root.querySelector 产物——旧 'element.matches is not a function'）。**文档上下文**
  // 优先：元素带 `_zwRootHtml`（来自查询根——root.querySelector 把根源串挂到产物）时，
  // 对整棵根树跑 selector，自身 id 命中匹配集 → true（sibling/descendant 组合器正确——
  // `#a+#b` 等在元素自身 outerHTML 内永远不命中）。无根上下文回落自身包裹（root 化近似）。
  // 无参 → TypeError（spec WebIDL 必参；WPT runSpecialMatchesTests #3）。
  // 第二 refNode 参数忽略（:scope 相对匹配近似——selectors.js 的 ctx 用例族按文档序近似）。
  _zwParseEl.prototype.matches = function (sel) {
    if (arguments.length === 0) {
      throw new globalThis.TypeError(
        "Failed to execute 'matches' on 'Element': 1 argument required, but only 0 present.");
    }
    if (typeof __zw_parse_html_query !== 'function') return false;
    // R156：非法选择器抛 SyntaxError（spec `dom-element-matches`——WPT
    // invalidSelectors 簇；`matches(null)`/`matches(undefined)` 的 String 形态
    // "null"/"undefined" 是合法 type selector → 走正常查询返 false）。
    if (typeof __zw_selector_valid === 'function' && __zw_selector_valid(String(sel)) !== '1') {
      throw new (globalThis.DOMException || Error)(
        "'" + String(sel) + "' is not a valid selector.", 'SyntaxError');
    }
    try {
      var rootHtml = this._zwRootHtml;
      if (rootHtml) {
        // R162：`:target` 需要 fragment URL（doc 级 `:target` 判定同款——
        // matches 的根上下文查询透传 doc._zwFragmentUrl）。
        var furl162 = '';
        try { furl162 = this._zwOwnerDoc && this._zwOwnerDoc._zwFragmentUrl ? String(this._zwOwnerDoc._zwFragmentUrl) : ''; } catch (_e162u) {}
        var rarr = JSON.parse(__zw_parse_html_query(rootHtml, String(sel), '1', furl162)) || [];
        var myId = this.id == null ? '' : String(this.id);
        var myOuter = String(this.outerHTML || '');
        for (var ri = 0; ri < rarr.length; ri++) {
          if (!rarr[ri]) continue;
          if (myId !== '' && rarr[ri].id === myId) return true;
          // R156：无 id 元素回落 outerHTML 相等（`matches('*')` 等全元素场景——
          // 同树同序列化的元素唯一，足够近似 identity）。
          if (myId === '' && myOuter !== '' && rarr[ri].outer === myOuter) return true;
        }
        return false;
      }
      var outer = this.outerHTML;
      if (!outer) outer = '<' + String(this.tagName || this.localName || 'div').toLowerCase() + '></' + String(this.tagName || this.localName || 'div').toLowerCase() + '>';
      var arr = JSON.parse(__zw_parse_html_query(outer, String(sel), '0'));
      if (!arr || !arr.length) return false;
      var first = arr[0];
      return (this.id != null && String(this.id) !== '' && first.id === String(this.id))
        || String(this.tagName || '').toLowerCase() === String(first.tag || '').toLowerCase();
    } catch (_e156m) { return false; }
  };
  _zwParseEl.prototype.matchesSelector = _zwParseEl.prototype.matches;
  _zwParseEl.prototype.webkitMatchesSelector = _zwParseEl.prototype.matches;
  // R158（js-dom M4）：per-element wrapper 缓存——`el.querySelector(x) ===
  // el.querySelectorAll(x)[0]` 的 identity 断言（WPT runFinderTest 的
  // assert_equals(found, foundall[0])；旧版每次 new 全新对象全 fail）。缓存挂
  // 元素自身（_zwQWrapMap），键含 outer（子树变更自然 miss），上限 512 防爆。
  _zwParseEl.prototype._zwWrapQ = function (info) {
    if (!this._zwQWrapMap || !(this._zwQWrapMap instanceof Map)) {
      try { this._zwQWrapMap = new Map(); } catch (_e158w) {
        return new _zwParseEl(info);
      }
    }
    var map = this._zwQWrapMap;
    var key = String(info && info.tag || '') + '\x1f' + String(info && info.id || '') + '\x1f' + String(info && info.outer || '');
    var cached = map.get(key);
    if (cached) return cached;
    if (map.size > 512) map.clear();
    var e = new _zwParseEl(info);
    e._zwRootHtml = this._zwRootHtml || this.outerHTML;
    map.set(key, e);
    return e;
  };
  _zwParseEl.prototype.querySelector = function (sel) {
    if (globalThis._zwQueryGuard) globalThis._zwQueryGuard(sel, arguments.length);
    if (typeof __zw_parse_html_query !== 'function') return null;
    var arr = JSON.parse(__zw_parse_html_query(this.outerHTML, String(sel), '0', '', '1')); // R161: filter_synthetic
    return arr.length ? this._zwWrapQ(arr[0]) : null;
  };
  _zwParseEl.prototype.querySelectorAll = function (sel) {
    if (globalThis._zwQueryGuard) globalThis._zwQueryGuard(sel, arguments.length);
    if (typeof __zw_parse_html_query !== 'function') return [];
    var arr = JSON.parse(__zw_parse_html_query(this.outerHTML, String(sel), '1', '', '1')); // R161: filter_synthetic（R188 扩展：head 同滤）
    // R188（js-dom M4）：子树查询不含根自身——outerHTML 重解析把 root 元素自身
    // 放进查询面（`el.querySelectorAll("*")` 首元素会是 root 镜像；spec
    // `dom-parentnode-queryselectorall` 只查**后代**，WPT ParentNode-querySelector-All
    // "tree order" 断言 result[0] 是 root 首子）。镜像判定：首元素与 root 同 tag+id
    // 且其 outer 的**尾部**与 root outerHTML 尾部一致（root 自身序列化的闭环特征）。
    // 跳过首个镜像；后续同 tag+id 的真实后代不受影响（只查 arr[0]）。
    if (arr.length && this.localName) {
      var _r188First = arr[0];
      var _r188TagOk = String(_r188First && _r188First.tag || '').toLowerCase() === String(this.localName).toLowerCase();
      var _r188IdOk = String(_r188First && _r188First.id || '') === String(this.id || '');
      if (_r188TagOk && _r188IdOk) arr = arr.slice(1);
    }
    var out = [];
    for (var i = 0; i < arr.length; i++) out.push(this._zwWrapQ(arr[i]));
    out.__zwQSA = true;
    return out;
  };
  // R3019：lazy 可变子树桥——DOMPurify / sanitizer / 树遍历库经 DOMParser.parseFromString 拿到 body 后，
  // 用 createNodeIterator 递归 childNodes + removeChild/setAttribute/removeAttribute 清洗 + 读 body.innerHTML。
  // 旧 _zwParseEl 为只读快照（无 childNodes/mutation），walk 恒只见 root。本桥首次 childNodes/mutation 访问时
  // 从 outerHTML 建可变 _zwMEl 子树（复用 part03 的 _zwMEl/_zwMBuildNode，IIFE 内函数声明提升可跨 part 引用）
  // 并把 innerHTML/outerHTML/getAttribute/hasAttribute/textContent rewire 为 live 树视图——读语义对纯读调用方
  // 零变化（未触树建），mutation 后序列化反映变更。后代为真实 _zwMEl/_zwMText 节点（全 mutation 语义）。
  _zwParseEl.prototype._ensureMutTree = function () {
    if (this._mtree) return this._mtree;
    var tag = this.localName || 'div';
    var snap = { tag: tag, id: this.id, cls: this.className, attrs: this._attrs };
    var node = _zwMEl(snap, null);
    if (typeof __zw_parse_html_child_nodes === 'function') {
      try {
        // R5001 M3 片 a：ownerDocument 为 inert 文档（`__zwScriptingEnabled === false`
        // ——DOMParser/createHTMLDocument 产物）时本地树按 markup 解析（noscript 实体
        // 解码）+ 逐节点盖 `__zwTreeScripting=false` 印章（序列化器转义分支）。
        // _zwParseEl 产物全部来自 `__zw_parse_html_query`（parse_html_element_json_full
        // ——R5001 起恒 scripting disabled 的 inert 文档面），树恒 inert 印章。
        var _r5000scr = '0';
        var _r5000inert = true;
        var arr = JSON.parse(__zw_parse_html_child_nodes(this.outerHTML, tag, '', _r5000scr));
        for (var i = 0; i < arr.length; i++) if (arr[i]) {
          var _r5000built = _zwMBuildNode(this.outerHTML, arr[i], node, _r5000inert ? false : undefined);
          if (_r5000inert) {
            (function stampInert(n5i) {
              if (!n5i) return;
              try { n5i.__zwTreeScripting = false; } catch (_e5i) {}
              var k5i = n5i.childNodes || [];
              for (var j5i = 0; j5i < k5i.length; j5i++) stampInert(k5i[j5i]);
            })(_r5000built);
          }
          node.childNodes.push(_r5000built);
        }
      } catch (_e) {}
    }
    this._mtree = node;
    var self = this;
    // rewire 读字段为 live 树视图（mutation 后 body.innerHTML 等反映变更）。
    Object.defineProperty(this, 'innerHTML', { get: function () { return node.innerHTML; }, configurable: true });
    Object.defineProperty(this, 'outerHTML', { get: function () { return node.outerHTML; }, configurable: true });
    Object.defineProperty(this, 'textContent', { get: function () { return node.textContent; }, configurable: true });
    Object.defineProperty(this, 'getAttribute', { value: function (n) { return node.getAttribute(n); }, configurable: true });
    Object.defineProperty(this, 'hasAttribute', { value: function (n) { return node.hasAttribute(n); }, configurable: true });
    Object.defineProperty(this, 'attributes', { get: function () { return node.attributes; }, configurable: true });
    return node;
  };
  Object.defineProperty(_zwParseEl.prototype, 'childNodes', { get: function () { return this._ensureMutTree().childNodes; }, configurable: true });
  Object.defineProperty(_zwParseEl.prototype, 'children', { get: function () { return this._ensureMutTree().children; }, configurable: true });
  Object.defineProperty(_zwParseEl.prototype, 'firstChild', { get: function () { return this._ensureMutTree().firstChild; }, configurable: true });
  Object.defineProperty(_zwParseEl.prototype, 'lastChild', { get: function () { return this._ensureMutTree().lastChild; }, configurable: true });
  _zwParseEl.prototype.insertBefore = function (n, ref) { return this._ensureMutTree().insertBefore(n, ref); };
  _zwParseEl.prototype.appendChild = function (n) { return this._ensureMutTree().appendChild(n); };
  _zwParseEl.prototype.removeChild = function (n) { return this._ensureMutTree().removeChild(n); };
  _zwParseEl.prototype.setAttribute = function (n, v) { this._ensureMutTree().setAttribute(n, v); };
  // R189（js-dom M4）：normalize 委托 mutTree（WPT Node-normalize "Non-text nodes
  // with empty textContent values"——DOMParser 文档的 documentElement 上跑合并语义）。
  // plain 容器版合并（与 part04 R184 的 _r184NormArr plain 分支同款）：exclusive-Text
  // 兜接 + 空 Text 移除 + 递归子树；CDATA（4）与 PI（7）非 Text 不动（R184 的
  // textContent exclusive-Text 同口径）。
  _zwParseEl.prototype.normalize = function () {
    var tree = this._ensureMutTree();
    var _r189Norm = function (kids, recurse) {
      if (!kids || !kids.length) return;
      if (recurse) {
        for (var i = 0; i < kids.length; i++) {
          var k = kids[i];
          if (k && k.nodeType === 1 && k.childNodes && !k.__zwSelector && !k.__zwHandle) {
            _r189Norm(k.childNodes, true);
          }
        }
      }
      var out = [];
      var lastText = null;
      for (var j = 0; j < kids.length; j++) {
        var n = kids[j];
        var isText = n && (n.nodeType === 3 || (n.__zwIsText && n.nodeType !== 7)) && n.nodeType !== 4;
        if (isText) {
          var dv = String(n.data != null ? n.data : (n.nodeValue != null ? n.nodeValue : ''));
          if (lastText != null) {
            try {
              lastText.data = String(lastText.data != null ? lastText.data : '') + dv;
              lastText.nodeValue = lastText.data;
            } catch (_e189m) {}
            continue;
          }
          if (dv === '') continue;
          lastText = n;
          out.push(n);
          continue;
        }
        lastText = null;
        out.push(n);
      }
      try {
        kids.length = 0;
        for (var w = 0; w < out.length; w++) kids.push(out[w]);
      } catch (_e189w) {}
    };
    _r189Norm(tree.childNodes, true);
    return undefined;
  };
  // R156（js-dom M4）：cloneNode（WPT Element-matches init 的 `element.cloneNode(true)`
  // 对 getElementById 产物——旧 'element.cloneNode is not a function' 整页中断）。经
  // _ensureMutTree 转 _zwMEl 后用 Element.prototype 的 deepClone（属性 + 子树全复制，
  // 返独立可变树）。
  _zwParseEl.prototype.cloneNode = function (deep) {
    var tree = this._ensureMutTree();
    if (globalThis.Element && globalThis.Element.prototype
        && typeof globalThis.Element.prototype.cloneNode === 'function') {
      var cloned = globalThis.Element.prototype.cloneNode.call(tree, deep);
      // R156：ownerDocument 继承源（_zwMEl 无 ownerDocument 字段——WPT
      // runInvalidSelectorTestMatches 的 `root.ownerDocument.defaultView.DOMException`
      // 对 clone 产物读，缺字段 'reading defaultView' TypeError 整簇崩）。
      if (cloned && this._zwOwnerDoc) {
        try { cloned.ownerDocument = this._zwOwnerDoc; } catch (_e156od2) {}
      }
      return cloned;
    }
    return tree;
  };
  _zwParseEl.prototype.removeAttribute = function (n) { this._ensureMutTree().removeAttribute(n); };
  // R156（js-dom M4）：NS 属性族（WPT Element-matches 的 setupSpecialElements 对
  // getElementById 产物调 setAttributeNS——解析元素经 _ensureMutTree 变 _zwMEl 后无
  // NS 方法直接 TypeError 整页中断）。限定名存本地 NS 表（qname→{ns,prefix,local}），
  // getAttributeNS 按 (ns,local) 反查——与 detached body 的 _r132BodyAttrNS 同款模式。
  _zwParseEl.prototype._nsAttrMeta = null;
  _zwParseEl.prototype.setAttributeNS = function (ns, qname, v) {
    var q = String(qname);
    this._ensureMutTree().setAttribute(q, v);
    if (!this._nsAttrMeta) this._nsAttrMeta = {};
    var ci = q.indexOf(':');
    this._nsAttrMeta[q] = { ns: ns == null || ns === '' ? null : String(ns),
      prefix: ci > 0 ? q.slice(0, ci) : null, local: ci > 0 ? q.slice(ci + 1) : q };
  };
  _zwParseEl.prototype.getAttributeNS = function (ns, local) {
    if (this._nsAttrMeta) {
      var want = (ns == null || ns === '') ? null : String(ns);
      for (var q in this._nsAttrMeta) {
        if (!Object.prototype.hasOwnProperty.call(this._nsAttrMeta, q)) continue;
        var m = this._nsAttrMeta[q];
        if (m.local === String(local) && m.ns === want) return this.getAttribute(q);
      }
    }
    return null;
  };
  _zwParseEl.prototype.hasAttributeNS = function (ns, local) {
    return this.getAttributeNS(ns, local) !== null;
  };
  _zwParseEl.prototype.hasChildNodes = function () { return this._ensureMutTree().hasChildNodes(); };
  // js-dom M4 R112：detached 解析元素的事件面（WPT Event-dispatch-bubbles "In new Document()"
  // 等——targets 链 [doc, docEl, body, #table, #table-body, #parent] 逐一 addEventListener，
  // _zwParseEl 缺方法直接 TypeError）。listener 存元素自身（per-element 表 _zwEvLs）。
  // **视图注册表**：detached 查询每次返新 _zwParseEl 实例、各自 lazy 建独立 mut 树——
  // 祖先链不能经树 parentNode 直达（不同视图不同树）。改按 **身份键**（id 优先，回落
  // tag+class+outer 前缀哈希）把「带 listener 的视图」注册进 doc 级表；派发沿自身 mut 树
  // parentNode 上行，每层按身份键反查注册视图触发其 listener（capture 逆链 → target →
  // bubble 正链，eventPhase/currentTarget 按 spec）。doc/docEl 的注册由
  // _zwDispatchLocalDoc 链承载（chain 顶端 doc 视图直连）。
  // spec https://dom.spec.whatwg.org/#concept-event-dispatch
  var _zwEvViewRegistry = {}; // 身份键 -> 视图（_zwParseEl 或 wired docEl/body 对象）
  // R112：tag 兜底注册表——detached doc 的 docEl/body（普通对象，无 id/outerHTML 快照）
  // 经 `tag:<TAG>` 键注册；路径键（sig:TAG|... 形态）直查 miss 时按 tag 前缀反查本表。
  // 挂 globalThis（part03 的 _makeDetachedDocument 跨 part 注册）。
  var _zwEvTagRegistry = {};
  globalThis._zwEvTagRegistry = _zwEvTagRegistry;
  _zwParseEl.prototype._zwEvKey = function () {
    if (this.id) return 'id:' + this.id;
    return 'sig:' + this.tagName + '|' + String(this.className) + '|' + String(this.outerHTML).slice(0, 64);
  };
  _zwParseEl.prototype.addEventListener = function (type, fn, opts) {
    if (!this._zwEvLs) this._zwEvLs = {};
    var t = String(type);
    if (!this._zwEvLs[t]) this._zwEvLs[t] = [];
    var cap = opts != null && typeof opts === 'object' ? !!opts.capture : !!opts;
    var once = opts != null && typeof opts === 'object' ? !!opts.once : false;
    this._zwEvLs[t].push({ fn: fn, capture: cap, once: once });
    _zwEvViewRegistry[this._zwEvKey()] = this;
  };
  _zwParseEl.prototype.removeEventListener = function (type, fn, opts) {
    if (!this._zwEvLs) return;
    var t = String(type);
    var cap = opts != null && typeof opts === 'object' ? !!opts.capture : !!opts;
    var ls = this._zwEvLs[t];
    if (!ls) return;
    this._zwEvLs[t] = ls.filter(function (l) { return !(l.fn === fn && l.capture === cap); });
  };
  _zwParseEl.prototype.dispatchEvent = function (event) {
    if (globalThis._zwDispatchGuard) globalThis._zwDispatchGuard(event);
    var self = this;
    var fireView = function (view, phase, captureOnly) {
      if (!view || !view._zwEvLs) return;
      var t = String(event.type);
      var ls = view._zwEvLs[t];
      if (!ls) return;
      var s = ls.slice();
      for (var i = 0; i < s.length; i++) {
        var entry = s[i];
        // captureOnly：capture listener 派（capture 期与 target 期 capture-pass 共用）；
        // 非 capture listener 在 target 期由第二次（captureOnly=false）调用派。AT_TARGET
        // 两 pass 分 capture 先后（spec invoke：capture listener 先于 non-capture）。
        if (captureOnly !== null && captureOnly !== entry.capture) continue;
        var cur = view._zwEvLs[t];
        if (!cur || cur.indexOf(entry) < 0) continue; // 派发中被移除（R111 语义）
        if (entry.once) {
          view._zwEvLs[t] = cur.filter(function (e) { return e !== entry; });
        }
        event.currentTarget = view;
        event.eventPhase = phase;
        var callable = typeof entry.fn === 'function' ? entry.fn : (entry.fn && entry.fn.handleEvent);
        if (typeof callable === 'function') {
          try { callable.call(typeof entry.fn === 'function' ? view : entry.fn, event); } catch (_e) {}
        }
      }
    };
    event.target = self;
    // 祖先链：`_zwPath`（host path 字段——身份键数组，根→父）。每层反查注册视图；
    // 直查 miss 时按 tag 兜底（path 键 sig:TAG|... → tag:<TAG> 表——detached doc 的
    // docEl/body 普通对象经 tag 键注册，其 outerHTML 与解析视图不同源无法 sig 匹配）。
    var viewForKey = function (key) {
      var v = _zwEvViewRegistry[key];
      if (v) return v;
      if (key && key.indexOf('sig:') === 0) {
        var bar = key.indexOf('|');
        if (bar > 4) return _zwEvTagRegistry['tag:' + key.slice(4, bar)] || null;
      }
      return null;
    };
    var path = self._zwPath || [];
    // R112：doc 站（detached doc 的 _zwLocalListeners）——path 顶端（html/body tag 命中
    // _zwEvDocChain）时 doc 是链最外层：capture 最先（path 之前）、bubble 最后（path 之后）。
    var docChain = globalThis._zwEvDocChain;
    var docHasHtml = docChain && (path.indexOf('sig:HTML|') >= 0 || (docChain.docEl && _zwEvTagRegistry['tag:HTML'] === docChain.docEl));
    var fireDoc = function (phase) {
      if (!docChain || !docHasHtml) return;
      var dl = (docChain.doc._zwLocalListeners || {})[String(event.type)] || [];
      var ds = dl.slice();
      for (var i = 0; i < ds.length; i++) {
        var entry = ds[i];
        if (entry.capture !== (phase === 1)) continue;
        var cur = docChain.doc._zwLocalListeners[String(event.type)];
        if (!cur || cur.indexOf(entry) < 0) continue;
        if (entry.once) {
          docChain.doc._zwLocalListeners[String(event.type)] = cur.filter(function (e) { return e !== entry; });
        }
        event.currentTarget = docChain.doc;
        event.eventPhase = phase;
        var callable = typeof entry.fn === 'function' ? entry.fn : (entry.fn && entry.fn.handleEvent);
        if (typeof callable === 'function') {
          try { callable.call(typeof entry.fn === 'function' ? docChain.doc : entry.fn, event); } catch (_eD) {}
        }
      }
    };
    fireDoc(1); // capture：doc 最先（链最外层）
    // capture：path[0]=最外层祖先（html）→ path 末端=最近父级——正序迭代（WPT
    // Event-dispatch-bubbles 预期 capture 序 doc→html→body→…→最近父级，首版逆序
    // 迭代致 currentTarget 序反转，assert_array_equals 实证）。
    for (var ci = 0; ci < path.length; ci++) {
      fireView(viewForKey(path[ci]), 1, true);
    }
    // target：AT_TARGET——capture listener 先。
    fireView(self, 2, true);
    fireView(self, 2, false);
    // bubble：近→远（path 逆序——最近父级先），仅 event.bubbles。doc 站最后。
    if (event.bubbles) {
      for (var bi = path.length - 1; bi >= 0; bi--) {
        fireView(viewForKey(path[bi]), 3, false);
      }
      fireDoc(3);
    }
    event.eventPhase = 0;
    event.currentTarget = null;
    return !event._defaultPrevented;
  };
  // DOMParser 解析出的 Document（只读）。querySelector/getElementById/body 经 host 回调重解析。
  function _zwParsedDoc(html) {
    this._html = html;
    this.nodeType = 9;
    this.nodeName = '#document';
    this.documentElement = this.querySelector('html');
    this.head = this.querySelector('head');
    this.body = this.querySelector('body');
  }
  // R5001 M3 片 a：DOMParser 产物为 inert 文档（scripting disabled）——本地视图
  // 解析（`__zw_parse_html_child_nodes` arg[3]='0'）与序列化（`__zwTreeScripting`
  // 印章 → noscript 转义分支）按旗标分流。
  _zwParsedDoc.prototype.__zwScriptingEnabled = false;
  _zwParsedDoc.prototype.querySelector = function (sel) {
    if (typeof __zw_parse_html_query !== 'function') return null;
    var arr = JSON.parse(__zw_parse_html_query(this._html, String(sel), '0'));
    return arr.length ? new _zwParseEl(arr[0]) : null;
  };
  _zwParsedDoc.prototype.querySelectorAll = function (sel) {
    if (typeof __zw_parse_html_query !== 'function') return [];
    var arr = JSON.parse(__zw_parse_html_query(this._html, String(sel), '1'));
    var out = [];
    for (var i = 0; i < arr.length; i++) out.push(new _zwParseEl(arr[i]));
    return out;
  };
  _zwParsedDoc.prototype.getElementById = function (id) {
    return this.querySelector('#' + String(id));
  };
  _zwParsedDoc.prototype.getElementsByTagName = function (tag) {
    return this.querySelectorAll(String(tag));
  };
  // R128（js-dom M4）：cloneNode（WPT Node-cloneNode-document-with-doctype "Created with
  // DOMParser"——`doc.cloneNode(true)` 断言 childNodes.length 2 + doctype 三字段；旧
  // 'doc.cloneNode is not a function'）。DOMParser 文档只读快照——clone 经
  // createHTMLDocument('') 承载：其自带 [doctype(html,,,), html] 子与用例期望一致。
  _zwParsedDoc.prototype.cloneNode = function (_deep) {
    var impl = globalThis.document && globalThis.document.implementation;
    if (!impl || typeof impl.createHTMLDocument !== 'function') return this;
    return impl.createHTMLDocument('');
  };
  globalThis.DOMParser = globalThis.DOMParser || function DOMParser() {};
  globalThis.DOMParser.prototype.parseFromString = function (str, mimeType) {
    // text/html | text/xml | application/xml | application/xhtml+xml | image/svg+xml 统一按 HTML 解析。
    var html = str == null ? '' : String(str);
    var d = new _zwParsedDoc(html);
    d.mimeType = mimeType || 'text/html';
    // R186（js-dom M4）：XML 文档 documentElement = 源串真根元素（spec DOMParser——XML
    // 根非合成 html 包装；WPT Element-tagName "tagName should be updated when changing
    // ownerDocument" 断言 parseFromString('<div xmlns=…>', 'text/xml').documentElement
    // .tagName === 'div'——旧恒取 querySelector('html') 命中合成包装根得 'HTML'）。
    // 探测规则：strip 注释/PI/doctype 前导后首个标签名（支持 prefix 限定名）；保留源
    // 大小写（XML 大小写敏感）。源串无元素根（空/纯文本）→ null（spec XML 空 doc）。
    if (d.mimeType !== 'text/html') {
      var _r186m = /^[\s]*(?:<!--[\s\S]*?-->|<\?[\s\S]*?\?>|<!DOCTYPE[^>]*>)*[\s]*<([A-Za-z_:][-A-Za-z0-9_:.]*)/.exec(html);
      if (_r186m) {
        var _r186root = d.querySelector(_r186m[1]);
        if (_r186root) {
          // tagName/nodeName 保源大小写：_zwParseEl 构造恒 toUpperCase（HTML 语义），
          // XML 文档元素大小写敏感（spec dom-element-tagname 只对 HTML 文档大写）。
          var _r186Tag = String(_r186root.tagName || '');
          var _r186Src = _r186root.outerHTML || '';
          var _r186Open = new RegExp('^<' + _r186m[1].replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + '\\b[^>]*>', 'i').exec(_r186Src);
          if (_r186Open) {
            var _r186Name = _r186Open[0].slice(1, -1).split(/[\s/>]/)[0] || _r186Tag;
            _r186root.tagName = _r186Name;
            _r186root.nodeName = _r186Name;
          }
          // namespaceURI：默认 xmlns 声明读源串根标签（XML ns 语义最小面——
          // importNode adopt 大写化按 HTML ns 判定，见 part06 R186）。
          var _r186Xmlns = /\sxmlns\s*=\s*"([^"]*)"/.exec(_r186Open ? _r186Open[0] : '');
          _r186root.namespaceURI = _r186Xmlns ? _r186Xmlns[1] : null;
          d.documentElement = _r186root;
        }
      } else {
        d.documentElement = null;
      }
    }
    // js-dom M4 R81：spec DOMParser —— Document.contentType = 解析 MIME；createElement 的
    // namespaceURI 由 contentType 派生（text/html 与 application/xhtml+xml → HTML ns；XML/SVG
    // → null——spec 元素 ns 由文档类型决定，WPT Document-createElement-namespace）。
    d.contentType = d.mimeType;
    var _HTML_NS = 'http://www.w3.org/1999/xhtml';
    d._htmlDoc = (d.mimeType === 'text/html' || d.mimeType === 'application/xhtml+xml');
    d._defaultNS = d._htmlDoc ? _HTML_NS : null;
    // createElement：轻量节点（tagName 大写 + localName 小写 + namespaceURI + parentNode null）。
    d.createElement = function (t) {
      var tag = String(t);
      // R123：XML 文档 createElement 大小写敏感（spec——HTML 文档才 ASCII 大写；
      // WPT PI-attributes check-attribute-value 经 `pi.ownerDocument.createElement('el')`
      // 建 element，outerHTML 提取 regex 按原大小写 'el' 匹配）。
      var _upTag = d._htmlDoc ? tag.toUpperCase() : tag;
      var n = {
        nodeType: 1,
        tagName: _upTag,
        nodeName: _upTag,
        localName: d._htmlDoc ? tag.toLowerCase() : tag,
        namespaceURI: d._defaultNS,
        prefix: null,
        nodeValue: null,
        childNodes: [],
        children: [],
        parentNode: null,
        ownerDocument: d,
        hasChildNodes: function () { return false; },
        contains: function (other) { return globalThis._zwNodeContains ? globalThis._zwNodeContains(n, other) : other === n; },
        compareDocumentPosition: function (other) { return globalThis._zwCompareDocumentPosition ? globalThis._zwCompareDocumentPosition(n, other) : 1 | 32; },
      };
      // R123：轻量属性面 + outerHTML 序列化（WPT PI-attributes check-attribute-value 簇经
      // `pi.ownerDocument.createElement('el')` 建 element 对照——setAttribute 后 outerHTML
      // 提取值与 PI data 逐串相等）。转义与 _zwPiEscape / part04 outerHTML 同款全集。
      var _attrs = {};
      n.setAttribute = function (name, value) {
        _attrs[String(name)] = String(value);
      };
      n.getAttribute = function (name) {
        return Object.prototype.hasOwnProperty.call(_attrs, String(name)) ? _attrs[String(name)] : null;
      };
      n.hasAttribute = function (name) {
        return Object.prototype.hasOwnProperty.call(_attrs, String(name));
      };
      n.removeAttribute = function (name) { delete _attrs[String(name)]; };
      n.getAttributeNames = function () { return Object.keys(_attrs); };
      Object.defineProperty(n, 'outerHTML', {
        get: function () {
          // R5004 M3 片 b（html-syntax-compat）：**XML 文档元素走 XML 序列化**——
          // DOM-Parsing §3.2.1（xmlns 声明 + HTML ns void ` />` 自闭合 + 非 HTML ns
          // 空元素 `/>`），serializing-xml-fragments/outerHTML 语料（createDocument
          // XML 文档 createElementNS 产物）。HTML 文档保持原 HTML 序列化零变化。
          if (!d._htmlDoc && typeof _zwXMLSerialize === 'function') {
            return _zwXMLSerialize(n, null);
          }
          var out = '<' + n.tagName;
          var names = Object.keys(_attrs);
          for (var i = 0; i < names.length; i++) {
            var v = _attrs[names[i]].replace(/&/g, '&amp;').replace(/"/g, '&quot;')
              .replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/ /g, '&nbsp;');
            out += ' ' + names[i] + '="' + v + '"';
          }
          return out + '></' + n.tagName + '>';
        },
        configurable: true, enumerable: true,
      });
      // R5004：轻量元素 attributes 视图（`_zwXMLSerialize`/`_zwMSerialize` 的
      // attrs 读链——closure `_attrs` 的快照数组形态）。
      Object.defineProperty(n, 'attributes', {
        get: function () {
          var out = [];
          var ks = Object.keys(_attrs);
          for (var i = 0; i < ks.length; i++) out.push({ name: ks[i], value: _attrs[ks[i]], nodeName: ks[i], nodeValue: _attrs[ks[i]] });
          return out;
        },
        configurable: true, enumerable: true,
      });
      // R189（js-dom M4）：轻量元素的可变容器面——appendChild/removeChild/normalize
      //（WPT MutationObserver-textContent "CDATASection" 变体：`xml.createElement
      // ("somelement")` 产物 appendChild CDATA + observe + textContent= 的 childList
      // 记录；旧产物无 appendChild 直接 not-a-function 整用例中断）。子存 childNodes
      // 数组（spec reparent 语义：先从旧父摘除）；textContent setter 走 replace-all
      //（removed=旧子快照，added=[新文本节点]——与主文档 R189 语义同款）。
      n.childNodes = [];
      n.children = [];
      n.appendChild = function (c) {
        if (!c) return c;
        if (c.parentNode && c.parentNode.removeChild) { try { c.parentNode.removeChild(c); } catch (_e189rp) {} }
        n.childNodes.push(c);
        try { c.parentNode = n; } catch (_e189pp) {}
        if (c.nodeType === 1) n.children.push(c);
        return c;
      };
      n.removeChild = function (c) {
        for (var _r189i = 0; _r189i < n.childNodes.length; _r189i++) {
          if (n.childNodes[_r189i] === c) {
            n.childNodes.splice(_r189i, 1);
            try { c.parentNode = null; } catch (_e189pn) {}
            var _r189ci = n.children.indexOf(c);
            if (_r189ci >= 0) n.children.splice(_r189ci, 1);
            return c;
          }
        }
        return c;
      };
      n.hasChildNodes = function () { return n.childNodes.length > 0; };
      Object.defineProperty(n, 'firstChild', {
        configurable: true,
        get: function () { return n.childNodes.length ? n.childNodes[0] : null; },
      });
      Object.defineProperty(n, 'lastChild', {
        configurable: true,
        get: function () { return n.childNodes.length ? n.childNodes[n.childNodes.length - 1] : null; },
      });
      Object.defineProperty(n, 'textContent', {
        configurable: true,
        get: function () {
          // R189 修正：textContent 含 CDATA（nodeType 4）的 data（spec
          // `dom-node-textcontent`—— descendant text content 收 Text + CDATA +
          // PI data；normalize 的 exclusive-Text 才排除非 Text）。WPT
          // MutationObserver-textContent CDATA 变体 observe 前断言 "foo"。
          var _r189s = '';
          for (var _r189j = 0; _r189j < n.childNodes.length; _r189j++) {
            var _r189c = n.childNodes[_r189j];
            if (_r189c && (_r189c.nodeType === 3 || _r189c.nodeType === 4)) {
              _r189s += String(_r189c.data != null ? _r189c.data : '');
            }
          }
          return _r189s;
        },
        set: function (v) {
          var _r189val = (v === null || v === undefined) ? '' : String(v);
          var _r189old = n.childNodes.slice();
          n.childNodes = [];
          n.children = [];
          if (_r189val !== '') {
            var _r189t = d.createTextNode(_r189val);
            n.childNodes.push(_r189t);
            try { _r189t.parentNode = n; } catch (_e189tp) {}
          }
          // childList 记录（observe 经 _zwMEl 系注册的 MutationObserver 不覆盖此形态
          //——轻量元素独立派发：有 observer 时入列）。
          if (globalThis.__zw_mo_observers && globalThis.__zw_mo_observers.length) {
            try {
              var _r189key = '__r189:' + String(n.tagName) + ':' + String(n._zwSeq || '');
              for (var _r189oi = 0; _r189oi < globalThis.__zw_mo_observers.length; _r189oi++) {
                var _r189obs = globalThis.__zw_mo_observers[_r189oi];
                if (_r189obs._targets && (_r189obs._targets['h:' + _r189key] || _r189obs._targets[_r189key])) {
                  var _r189rec = Object.create(globalThis.MutationRecord.prototype);
                  _r189rec.type = 'childList';
                  _r189rec.target = n;
                  _r189rec.addedNodes = n.childNodes.slice();
                  _r189rec.removedNodes = _r189old;
                  _r189rec.previousSibling = null;
                  _r189rec.nextSibling = null;
                  _r189rec.attributeName = null;
                  _r189rec.attributeNamespace = null;
                  _r189rec.oldValue = null;
                  _r189obs._records.push(_r189rec);
                }
              }
            } catch (_e189mo) {}
          }
          if (typeof globalThis.__zw_mo_flush_lite === 'function') {
            try { globalThis.__zw_mo_flush_lite(); } catch (_e189fl) {}
          }
        },
      });
      return n;
    };
    d.createElementNS = function (ns, q) {
      var n = d.createElement(q);
      var _nsStr = (ns == null) ? '' : String(ns);
      n.namespaceURI = _nsStr || null;
      var c = String(q).indexOf(':');
      if (c > 0) { n.prefix = String(q).slice(0, c); n.localName = String(q).slice(c + 1); n.tagName = String(q); n.nodeName = String(q); }
      return n;
    };
    // R189（js-dom M4）：DOMParser 文档可变面三工厂——createTextNode/createComment/
    // createCDATASection（WPT Node-normalize "Non-text nodes with empty textContent
    // values"：XML doc 上建 9 类子节点后 normalize 断言非 Text 节点不动——旧缺工厂
    // `doc.createTextNode is not a function` 整 subtest 崩）。产物 = 轻量节点
    //（nodeType 3/8/4 + data/nodeValue + ownerDocument 指 d——与 _zwMText 形态兼容，
    // 可入 _zwParseEl 的 _ensureMutTree 子树（normalize 经 Element.prototype 归一）。
    // CDATA 的 spec 校验：XML 文档才允许（HTML 文档抛 NotSupportedError）。
    d.createTextNode = function (text) {
      return {
        nodeType: 3, nodeName: '#text', data: String(text == null ? '' : text),
        get nodeValue() { return this.data; }, set nodeValue(v) { this.data = String(v == null ? '' : v); },
        parentNode: null, ownerDocument: d, childNodes: [],
      };
    };
    d.createComment = function (text) {
      return {
        nodeType: 8, nodeName: '#comment', data: String(text == null ? '' : text),
        get nodeValue() { return this.data; }, set nodeValue(v) { this.data = String(v == null ? '' : v); },
        parentNode: null, ownerDocument: d, childNodes: [],
      };
    };
    d.createCDATASection = function (text) {
      if (d._htmlDoc) {
        // https://dom.spec.whatwg.org/#dom-document-createcdatasection——HTML 文档
        // 不支持 CDATASection（NotSupportedError）。
        throw new (globalThis.DOMException || Error)("Cannot create CDATASection nodes in HTML documents.", 'NotSupportedError');
      }
      var v = String(text == null ? '' : text);
      if (v.indexOf(']]>') >= 0) {
        throw new (globalThis.DOMException || Error)("Sequence ']]>' not allowed in CDATA sections.", 'InvalidCharacterError');
      }
      return {
        nodeType: 4, nodeName: '#cdata-section', data: v,
        get nodeValue() { return this.data; }, set nodeValue(v2) { this.data = String(v2 == null ? '' : v2); },
        parentNode: null, ownerDocument: d, childNodes: [],
      };
    };
    // R123：PI 属性层（WPT processing-instruction-attributes xml-dom/xml-parser source）。
    // ① createProcessingInstruction：真 PI 视图节点（part03 _zwMPiFromBogus 同款属性五件套，
    //    target/data + data 即属性序列化源）+ spec 校验（target 合法 Name、data 无 '?>'）。
    // ② XML 文档的 firstChild：输入以 '<?…?>' 开头时返 PI 视图（HTML tokenizer 把它落为
    //    html>head 的 bogus comment，这里从原串直接合成 spec 序——doc 级 PI 在 documentElement
    //    之前）。'<?xml' 声明除外（XML 声明不是 PI，spec Document.firstChild 跳过——按
    //    target 是否 'xml' 判定）。
    d.createProcessingInstruction = function (target, data) {
      var t = String(target == null ? '' : target);
      var dd2 = String(data == null ? '' : data);
      if (!/^[A-Za-z_:][-A-Za-z0-9_:.]*$/.test(t)) {
        throw new (globalThis.DOMException || Error)('The target provided is not a valid name.', 'InvalidCharacterError');
      }
      if (dd2.indexOf('?>') !== -1) {
        throw new (globalThis.DOMException || Error)("The data provided contains '?>'.", 'InvalidCharacterError');
      }
      var pi = _zwMPiFromBogus('?' + t + ' ' + dd2 + '?', null);
      pi.ownerDocument = d;
      // R123：自观测键（observe(pi) 直接命中，无祖先可回落——xml-dom source 的
      // MutationObserver observe/takeRecords 簇）。
      pi.__zwMoSelfKey = 'zwpi:' + (d._zwPiSeq = (d._zwPiSeq || 0) + 1);
      return pi;
    };
    d._zwIsXml = !d._htmlDoc;
    Object.defineProperty(d, 'firstChild', {
      get: function () {
        if (!d._zwIsXml) return d.documentElement;
        // R123 修正：负向前瞻字符类构造在含 '?>' 值上失配（正则三 case 全 no-match 实证）——
        // 改 split 提取（trim 后 '<?' 开头 + 首个 '?>' 截断）。
        var _fc = String(d._html || '').trim();
        if (_fc.charAt(0) === '<' && _fc.charAt(1) === '?') {
          var _fcEnd = _fc.indexOf('?>');
          if (_fcEnd >= 0) {
            var _fcInner = _fc.slice(2, _fcEnd);
            var _fcSp = _fcInner.indexOf(' ');
            if (_fcSp > 0) {
              var _fcT = _fcInner.slice(0, _fcSp);
              if (_fcT !== 'xml') {
                return d.createProcessingInstruction(_fcT, _fcInner.slice(_fcSp + 1));
              }
            }
          }
        }
        return d.documentElement;
      },
      configurable: true, enumerable: true,
    });
    return d;
  };

  // XMLSerializer（R2818）——节点 → HTML/XML 字符串（serializeToString，SVG 导出 / XML utils / 序列化对比高频）。
  // 委托节点既有 outerHTML（sel-based 经 __zw_get_outer_html / handle 经 innerHTML 回落）+ text/comment nodeValue。
  // **已知限制**：与 DOMParser 对称——仅 HTML 序列化（无真 XML namespace 声明），document 节点取 documentElement。
  globalThis.XMLSerializer = globalThis.XMLSerializer || function XMLSerializer() {};
  globalThis.XMLSerializer.prototype.serializeToString = function (node) {
    if (node == null) return '';
    var n = node.nodeType === 9 ? node.documentElement : node; // Document → documentElement
    if (n == null) return '';
    // 元素（nodeType 1）→ outerHTML；text/comment（3/8）→ nodeValue/data；其余 best-effort outerHTML。
    if (n.nodeType === 3 || n.nodeType === 8) return String(n.nodeValue != null ? n.nodeValue : n.data || '');
    var oh = n.outerHTML;
    return oh != null ? String(oh) : '';
  };

  // URL——WHATWG URL 解析 + 组件 setter（R2778 解析 + R2780 setter/双向 searchParams 同步）。委托 host
  // `__zw_parse_url`（解析）+ `__zw_set_url_part`（setter），均 spec-correct via `url` crate。组件存内部
  // `_`-prefixed 字段，accessor 暴露读 + 写（setter 经 `_setPart` 回调重解析）。searchParams 为稳定实例 +
  // `_onchange` 双向同步：mutate searchParams → 重设 search/href（`_applySearchParams`，内部直写字段不调
  // `_setPart` 故无递归）；set search/href → `_zw_reinit` 同步 searchParams（不触发 `_onchange` 故无递归）。
  // **已知限制**：href setter 按绝对 URL 重解析（无 base 上下文，相对值失败抛 TypeError，spec 边角）。
  function URL(url, base) {
    if (typeof __zw_parse_url !== 'function') {
      throw new TypeError('URL constructor requires a URL parser (__zw_parse_url not registered)');
    }
    if (!(this instanceof URL)) return new URL(url, base); // 允许无 new
    var raw = __zw_parse_url(String(url), base !== undefined ? String(base) : '');
    var p = raw ? JSON.parse(raw) : null;
    if (!p) throw new TypeError('Invalid URL: ' + url);
    this._load(p);
    // searchParams 稳定实例 + 注册 _onchange（mutate → 同步 search/href）。
    var self = this;
    this._sp = new URLSearchParams(p.search);
    this._sp._onchange = function () { self._applySearchParams(); };
  }
  // 内部：从解析 JSON 加载全部组件字段（不含 searchParams）。
  URL.prototype._load = function (p) {
    this._protocol = p.protocol;
    this._username = p.username;
    this._password = p.password;
    this._hostname = p.hostname;
    this._host = p.host;
    this._port = p.port;
    this._origin = p.origin;
    this._pathname = p.pathname;
    this._search = p.search;
    this._hash = p.hash;
    this._href = p.href;
  };
  // 内部：组件 setter 入口——回调重解析 + 重载字段；search/href 变更时同步 searchParams（不触发其 _onchange）。
  URL.prototype._setPart = function (part, value) {
    if (typeof __zw_set_url_part !== 'function') {
      throw new TypeError('URL setter requires __zw_set_url_part');
    }
    var raw = __zw_set_url_part(this._href, part, String(value));
    if (!raw) throw new TypeError('Invalid URL ' + part + ': ' + value);
    var p = JSON.parse(raw);
    this._load(p);
    // search/href 改变可能变 query → 同步 searchParams（_zw_reinit 不触发 _onchange，无递归）。
    if (part === 'search' || part === 'href') {
      this._sp._zw_reinit(p.search);
    }
  };
  // 内部：searchParams 变更回调——把 params.toString() 设回 search/href（直写字段，不调 _setPart，无递归）。
  URL.prototype._applySearchParams = function () {
    if (typeof __zw_set_url_part !== 'function') return;
    var q = this._sp.toString();
    var raw = __zw_set_url_part(this._href, 'search', q ? '?' + q : '');
    if (!raw) return;
    var p = JSON.parse(raw);
    this._search = p.search;
    this._href = p.href; // mutate query 仅影响 search + href
  };
  // accessor 定义：读返内部字段，写经 _setPart。
  function _urlAcc(field) {
    return {
      get: function () { return this['_' + field]; },
      set: function (v) { this._setPart(field, v); },
      configurable: true,
      enumerable: true,
    };
  }
  var _urlFields = ['protocol', 'username', 'password', 'hostname', 'host', 'port', 'pathname', 'search', 'hash', 'href'];
  for (var _i = 0; _i < _urlFields.length; _i++) {
    Object.defineProperty(URL.prototype, _urlFields[_i], _urlAcc(_urlFields[_i]));
  }
  Object.defineProperty(URL.prototype, 'origin', {
    get: function () { return this._origin; },
    configurable: true,
    enumerable: true,
  });
  Object.defineProperty(URL.prototype, 'searchParams', {
    get: function () { return this._sp; },
    configurable: true,
    enumerable: true,
  });
  URL.prototype.toString = function () { return this._href; };
  URL.prototype.toJSON = function () { return this._href; };
  // canParse 静态——解析成功 true / 失败 false（不抛）。
  URL.canParse = function (url, base) {
    if (typeof __zw_parse_url !== 'function') return false;
    return !!__zw_parse_url(String(url), base !== undefined ? String(base) : '');
  };
  // net-api M3-S3：URL.parse 静态（url-statics-parse——解析成功返 URL 实例、失败返
  // null，**不抛**；undefined 入参按 DOMString 转 'undefined' 走失败路径）。
  if (!URL.parse) {
    URL.parse = function (url, base) {
      if (typeof __zw_parse_url !== 'function') return null;
      var raw = __zw_parse_url(String(url), base !== undefined ? String(base) : '');
      if (!raw) return null;
      var parsed = null;
      try { parsed = JSON.parse(raw); } catch (_eUrlParse) { return null; }
      if (!parsed || !parsed.href) return null;
      return new URL(parsed.href);
    };
  }
  globalThis.URL = globalThis.URL || URL;

  // URL.createObjectURL / revokeObjectURL——blob: URL 注册表（`<img src>` / `<a download>` /
  // 文件预览高频）。纯 JS：createObjectURL 生成 `blob:<origin>/<n>` 并在 `_zwBlobStore` 注册 Blob，
  // 返 URL 串；revokeObjectURL 从 store 移除。**已知限制（记录）**：blob: URL 不被 net/fetch 实际
  // 解析为内容（无 blob store→字节回流路径，follow-up）——但消除 `URL.createObjectURL is not a
  // function` ReferenceError，库可正常调用 + 传给 img.src/a.href。origin 取 location.origin 或 'null'。
  if (!globalThis.URL.createObjectURL) {
    globalThis.URL.createObjectURL = function (obj) {
      var origin = (globalThis.location && globalThis.location.origin) || 'null';
      // 单调 id（counter）+ Math.random 去重（不依赖未定义 crypto.randomUUID 顺序）。
      globalThis.__zwBlobSeq = (globalThis.__zwBlobSeq | 0) + 1;
      var url = 'blob:' + origin + '/' + globalThis.__zwBlobSeq + '-' +
        Math.floor(Math.random() * 1e9).toString(36);
      _zwBlobStore[url] = obj;
      return url;
    };
  }
  if (!globalThis.URL.revokeObjectURL) {
    globalThis.URL.revokeObjectURL = function (url) {
      if (url && Object.prototype.hasOwnProperty.call(_zwBlobStore, url)) delete _zwBlobStore[url];
    };
  }

  // structuredClone——深拷贝（postMessage / React state / immer-like 高频）。递归：primitive/array/
  // plain object/Date/RegExp/Map/Set/ArrayBuffer/TypedArray；循环引用经 WeakMap 记忆不爆栈；
  // function/symbol 抛 DataCloneError（spec）。**已知限制**：symbol-keyed 属性不拷（Object.keys 仅
  // string-keyed）；class 实例 prototype 保留但构造器不重跑（同 spec 平台对象外行为）。
  function _zw_structured_clone(val, seen) {
    if (val === null) return val;
    var t = typeof val;
    // R9/R382 wrong-global 先例：globalThis 构造器优先（页面 instanceof 比对的是已发布全局）。
    var _cloneDE = globalThis.DOMException || DOMException;
    if (t === 'function') throw new _cloneDE('function could not be cloned.', 'DataCloneError');
    if (t === 'symbol') throw new _cloneDE('symbol could not be cloned.', 'DataCloneError');
    if (t !== 'object') return val; // primitive（number/string/boolean/undefined/bigint）原样
    if (seen.has(val)) return seen.get(val); // 循环引用 → 已记忆的克隆
    if (val instanceof Date) { var d = new Date(val.getTime()); seen.set(val, d); return d; }
    if (val instanceof RegExp) { var r = new RegExp(val.source, val.flags); seen.set(val, r); return r; }
    if (val instanceof Map) {
      var m = new Map(); seen.set(val, m);
      val.forEach(function (v, k) { m.set(_zw_structured_clone(k, seen), _zw_structured_clone(v, seen)); });
      return m;
    }
    if (val instanceof Set) {
      var st = new Set(); seen.set(val, st);
      val.forEach(function (v) { st.add(_zw_structured_clone(v, seen)); });
      return st;
    }
    if (val instanceof ArrayBuffer) {
      var ab = new ArrayBuffer(val.byteLength); new Uint8Array(ab).set(new Uint8Array(val));
      seen.set(val, ab); return ab;
    }
    if (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView(val)) {
      var ta = new val.constructor(val); // TypedArray/DataView 拷贝构造
      seen.set(val, ta); return ta;
    }
    var out = Array.isArray(val) ? [] : Object.create(Object.getPrototypeOf(val));
    seen.set(val, out); // 先记忆，再递归子属性（解循环）
    var keys = Object.keys(val);
    for (var i = 0; i < keys.length; i++) out[keys[i]] = _zw_structured_clone(val[keys[i]], seen);
    return out;
  }
  globalThis.structuredClone = globalThis.structuredClone || function structuredClone(value) {
    return _zw_structured_clone(value, typeof WeakMap !== 'undefined' ? new WeakMap() : new Map());
  };

  // history（session history，R2814）——SPA 路由核心（react-router / vue-router / @reach 等）。原为全 stub
  // no-op，现实现真实 in-memory session history stack：pushState/replaceState 维护 entries + cursor，state/length
  // 反映当前；back/forward/go 移 cursor + 异步派发 popstate（window listener，复用 R2812 PopStateEvent）。
  // **已知限制（记录）**：① 仅 in-memory（不接真导航/host page_url——pushState url 仅记 entries，不更新
  // `location`，同源导航 defer host 桥）；② popstate 仅 dispatch 给 window listener（headless 无真用户
  // back 按钮，浏览器 chrome 导航 defer）；③ popstate 经 `_defer` microtask 派发（spec 为 task，本沙箱异步
  // 模型近似）；④ go(delta) 同步移 cursor + microtask 派发（spec 批量合并简化）。
  var _hist_entries = [{ state: null, url: '' }]; // cursor 0 = 初始 entry
  var _hist_cursor = 0;
  function _hist_current() { return _hist_entries[_hist_cursor]; }
  function _hist_dispatchPopState(oldHrefBefore) {
    // spec：back/forward/go 触发 popstate（pushState/replaceState 不触发），异步派发。
    // R3007：跨 hash 变更的导航同时派 hashchange（spec：hash 变更导航派 popstate + hashchange）。
    // oldHrefBefore = cursor 移动**前**的 entry url（back/forward/go 捕获传入）；hash 变化时派 hashchange。
    var cur = _hist_current();
    var st = cur.state;
    var newHref = cur.url;
    var hashChanged = oldHrefBefore !== undefined
      && String(oldHrefBefore).split('#')[1] !== String(newHref).split('#')[1];
    // R3065：back/forward/go 到 hash entry → 滚到锚元素（闭合 R3061 限制②）。real browser 跨 hash 导航滚锚
    //（back 到 #sec entry 滚到 id/name="sec"）。同步滚（mirror _setLocationHash），popstate/hashchange 仍 defer。
    if (hashChanged) {
      _scrollToAnchorForHash(String(newHref).split('#')[1] || '');
    }
    _defer(function () {
      var ev = new PopStateEvent('popstate', { state: st });
      ev.target = globalThis;
      _dispatchToListeners(_elKey('html', null), ev, 'all', globalThis);
      if (hashChanged) {
        var hev = new HashChangeEvent('hashchange', { oldURL: oldHrefBefore, newURL: newHref });
        hev.target = globalThis;
        _dispatchToListeners(_elKey('html', null), hev, 'all', globalThis);
      }
    });
  }

  // R2931 pageshow 派发（pagehide 不自动派发——headless 无 unload）。headless 无 host load 事件钩子，
  // 且 shim install 与 page script 为独立 execute（install 期 _defer 早于 page listener 注册）→ 采
  // 「首次注册 pageshow listener 时 _defer 派发一次」（globalThis/document addEventListener 触发）。
  // 保证 listener 捕获，近似 load 后 pageshow 语义（persisted:false）。仅触发一次（_pageshowFired 守）。
  // PageTransitionEvent 在 ~5448 行 _defineEventSubclass 注册，_defer 回调运行时（全 shim 安装后）已就绪。
  var _pageshowFired = false;
  // R5000 片 d（html-syntax-compat）：runner 生命周期尾（the-end）显式派发 pageshow
  // 前调本钩子关闭首监听自派发路径（防双发）；非 runner 上下文（webview 直载页）钩子
  // 语义不变。
  globalThis.__zwMarkPageShowFired = function () { _pageshowFired = true; };
  function _maybeFirePageShow() {
    // R5000 片 d：runner 生命周期尾接管 pageshow（__zwPageShowRunnerOwned 在
    // runner 预热 execute 置位）——本钩子不再自派发（the-end 的 pageshow 序断言：
    // 钩子 _defer 派发早于 load 即失序）。非 runner 上下文语义不变。
    if (globalThis.__zwPageShowRunnerOwned) { _pageshowFired = true; return; }
    if (_pageshowFired) return;
    _pageshowFired = true;
    _defer(function () {
      var ev = new PageTransitionEvent('pageshow', { persisted: false });
      ev.target = globalThis;
      _dispatchToListeners(_elKey('html', null), ev, 'all', globalThis);
    });
  }
  // R3005：解析 pushState/replaceState 的 url 为绝对 URL（相对当前 location.href）——使 location 反映 SPA
  // 路由变更（router 读 location.pathname）。优先 new URL(rel, base)（spec-correct percent-encoding/路径解析），
  // 未注册/解析失败回退原值。spec 跨源 url 应抛 SecurityError，headless permissive 允许（best-effort，无真安全边界）。
  function _resolveHistUrl(u) {
    if (typeof URL === 'function' && typeof __zw_parse_url === 'function') {
      try { return new URL(u, globalThis.location.href).href; } catch (_e) {}
    }
    return u;
  }
  globalThis.history = {
    get length() { return _hist_entries.length; },
    get state() { return _hist_current().state; },
    get scrollRestoration() { return 'auto'; },
    set scrollRestoration(_v) { /* headless 无真滚动恢复，no-op */ },
    // pushState(state, unused, url?)：截断 forward entries + push 新 entry + 推进 cursor（不触发 popstate）。
    // R3005：url 经 _resolveHistUrl 解析为绝对存入 entry（供 location getter 反映）。
    pushState: function (state, _unused, url) {
      _hist_entries = _hist_entries.slice(0, _hist_cursor + 1);
      _hist_entries.push({ state: state, url: url != null ? _resolveHistUrl(String(url)) : _hist_current().url });
      _hist_cursor = _hist_entries.length - 1;
    },
    // replaceState(state, unused, url?)：原地替换当前 entry 的 state/url（不触发 popstate）。
    // R3005：url 经 _resolveHistUrl 解析为绝对。
    replaceState: function (state, _unused, url) {
      var cur = _hist_current();
      cur.state = state;
      if (url != null) cur.url = _resolveHistUrl(String(url));
    },
    back: function () { if (_hist_cursor > 0) { var oldHref = _hist_current().url; _hist_cursor--; _hist_dispatchPopState(oldHref); } },
    forward: function () { if (_hist_cursor < _hist_entries.length - 1) { var oldHref = _hist_current().url; _hist_cursor++; _hist_dispatchPopState(oldHref); } },
    go: function (delta) {
      // R3004：spec/MDN——out-of-range delta 为 **no-op**（不动 cursor、不派发 popstate）。旧实现 clamp target
      // 到 [0,len-1] 后移动+派发 popstate（SPA router 计算的 delta 过冲时误导航到边界）。delta==null → -1
      //（spec go() 无参为 reload，headless 近似 back，保留旧默认）；delta==0 → 不移动（spec reload，headless no-op）。
      var d = (delta == null) ? -1 : (delta | 0);
      var target = _hist_cursor + d;
      if (target < 0 || target > _hist_entries.length - 1) return; // 越界 no-op
      if (target !== _hist_cursor) { var oldHref = _hist_current().url; _hist_cursor = target; _hist_dispatchPopState(oldHref); }
    },
  };

  // R3059：导航后重置 history——真跨文档导航（anchor/form/JS redirect）加载新文档时，host 经
  // `set_dom_snapshot(new_url)`（url 变化）调本函数清旧页 _hist_entries（pushState/hash-setter 残留）。
  // 重置为初始单 entry（url:''）→ location.href 读 page_url fallback（= 新文档 url，host 已设），
  // history.length=1，history.state=null（新文档初始状态）。闭合 SPA-then-redirect stale latent bug
  //（旧页 pushState 后导航，新页 location.href/history 误读旧 SPA entry）。pushState/replaceState/hash 变更
  //（同文档）**不**触发 host set_dom_snapshot(url 变化)，故不误重置 SPA 路由态。
  globalThis.__zw_reset_history = function () {
    _hist_entries = [{ state: null, url: '' }];
    _hist_cursor = 0;
  };

  // R3006/R3008：location setter 共享导航应用——push 新 history entry（navigation 语义，R3005 location 读之反映）
  // + hash 段变化时异步派 hashchange。供 _setLocationHash / _setLocationPart 复用（DRY）。
  function _pushHistNav(newHref, oldHref) {
    _hist_entries = _hist_entries.slice(0, _hist_cursor + 1);
    _hist_entries.push({ state: null, url: newHref });
    _hist_cursor = _hist_entries.length - 1;
    if (String(oldHref).split('#')[1] !== String(newHref).split('#')[1]) {
      var oldU = oldHref, newU = newHref;
      _defer(function () {
        var ev = new HashChangeEvent('hashchange', { oldURL: oldU, newURL: newU });
        ev.target = globalThis;
        _dispatchToListeners(_elKey('html', null), ev, 'all', globalThis);
      });
    }
  }

  // R3065：滚到锚元素（id 或 name = frag）共享 helper。R3061 _setLocationHash 内联滚锚提取——
  // 供 _setLocationHash（location.hash= setter）+ _hist_dispatchPopState（back/forward/go 到 hash entry，
  // 闭合 R3061 限制②）复用。real browser 同文档片段导航滚锚（<a href="#sec"> / location.hash= / history.back()
  // 到 #sec entry 均滚到 id="sec" 或 name="sec" 元素）。headless 无真 viewport → scrollIntoView 更新 scrollTop
  //（R3060）+ 派 scroll 事件（documented 近似）。无匹配元素 → 不滚。函数声明提升：_hist_dispatchPopState（前定义）可调。
  function _scrollToAnchorForHash(frag) {
    if (!frag || !globalThis.document) return;
    var anchor = null;
    try { anchor = globalThis.document.getElementById(frag); } catch (_e) {}
    if (!anchor) {
      try { anchor = globalThis.document.querySelector('[name="' + frag + '"]'); } catch (_e) {}
    }
    if (anchor && typeof anchor.scrollIntoView === 'function') {
      try { anchor.scrollIntoView(); } catch (_e) {}
    }
  }

  // R3006：`location.hash = v` setter——更新 hash + 新 history entry + 异步派发 hashchange（SPA hash 路由核心，
  // 如 older react-router hash mode）。spec：v 无 '#' 前缀自动补；hash 未变 no-op（不派 hashchange）。
  // newHref = 当前 href 去 hash 段 + 新 hash（hash 总在 URL 末尾）。
  function _setLocationHash(newHash) {
    var raw = String(newHash);
    var h = raw.charAt(0) === '#' ? raw : '#' + raw;
    if (h === '#') h = ''; // 空值 → 无 hash
    var oldHref = globalThis.location.href;
    var newHref = oldHref.split('#')[0] + h;
    if (newHref === oldHref) return; // hash 未变 → no-op（spec：不派 hashchange）
    _pushHistNav(newHref, oldHref);
    // R3061：滚到锚元素（frag = hash 去 '#'）——闭合 R3053 限制①。real browser 同文档片段导航滚锚。
    _scrollToAnchorForHash(h.charAt(0) === '#' ? h.slice(1) : '');
  }

  // R3008：`location.href/pathname/search = v` setter——经 URL part setter 计算新 href（spec-correct 组件替换，
  // 保留其它组件），push history entry（navigation）+ hash 变化派 hashchange。headless 无真文档重载（与 pushState
  // 同 in-memory 近似）；解析失败 / 未变 → no-op。protocol/host 等其它 setter defer（少用 + 涉 origin 变更导航）。
  function _setLocationPart(part, value) {
    var oldHref = globalThis.location.href;
    var newHref = null;
    if (typeof URL === 'function' && typeof __zw_set_url_part === 'function') {
      try { var u = new URL(oldHref); u[part] = String(value); newHref = u.href; } catch (_e) {}
    }
    if (!newHref || newHref === oldHref) return; // 解析失败 / 未变 → no-op
    _pushHistNav(newHref, oldHref);
    // R3058：href/pathname/search setter 改的是非 hash 段 → 跨文档导航 → host 真重载。
    //（hash 段经 _setLocationPath 不走此函数；故此处变更恒跨文档。）
    if (typeof __zw_request_navigate === 'function') __zw_request_navigate(newHref);
  }

  // R3009：replace 当前 history entry 共享导航应用——原地替换当前 entry url（mirror replaceState，不入新 entry，
  // 故 back 不回旧 url）+ hash 段变化时异步派 hashchange（同-document 片段导航语义）。不派 popstate（replace 语义，
  // 与 replaceState 对称——pushState/replaceState 不触发 popstate）。供 location.replace 复用（DRY，与 _pushHistNav 对称）。
  function _replaceHistNav(newHref, oldHref) {
    _hist_current().url = newHref;
    if (String(oldHref).split('#')[1] !== String(newHref).split('#')[1]) {
      var oldU = oldHref, newU = newHref;
      _defer(function () {
        var ev = new HashChangeEvent('hashchange', { oldURL: oldU, newURL: newU });
        ev.target = globalThis;
        _dispatchToListeners(_elKey('html', null), ev, 'all', globalThis);
      });
    }
  }

  // R3058：跨文档导航判定——old/new URL 去掉 hash 段后不同 → 跨文档（需 fetch 新文档）；
  // 仅 hash 变化 → 同文档片段导航（_setLocationHash 已处理，不触发真导航）。供 location
  // assign/replace/href-setter 区分：跨文档 → __zw_request_navigate 投递 host 真导航。
  function _isCrossDocumentNav(oldHref, newHref) {
    return String(oldHref).split('#')[0] !== String(newHref).split('#')[0];
  }

  // R3009：`location.assign(url)` / `location.replace(url)`——spec 导航方法（旧为 stub no-op，redirect 模式失效）。
  // assign(url) 功能等价 `location.href = url`（MDN）：resolve url + push history entry + location 反映 + hash 变化派
  // hashchange。replace(url)：replace 当前 entry（back 不回旧 url）+ hash 变化派 hashchange。两者均经 _resolveHistUrl
  // 解析为绝对（相对当前 location），headless 无真文档重载（与 pushState / location setter 同 in-memory 近似）。
  // 解析失败 / 未变 → no-op（spec assign/replace 同 url 为 no-op 导航）。
  function _locationAssign(url) {
    var oldHref = globalThis.location.href;
    var newHref = _resolveHistUrl(String(url));
    if (!newHref || newHref === oldHref) return; // 解析失败 / 未变 → no-op
    _pushHistNav(newHref, oldHref);
    // R3058：跨文档 assign（非 hash-only）→ host 真导航（fetch 新文档）。hash-only assign = 同文档，不导航。
    if (_isCrossDocumentNav(oldHref, newHref) && typeof __zw_request_navigate === 'function') {
      __zw_request_navigate(newHref);
    }
  }
  function _locationReplace(url) {
    var oldHref = globalThis.location.href;
    var newHref = _resolveHistUrl(String(url));
    if (!newHref || newHref === oldHref) return;
    _replaceHistNav(newHref, oldHref);
    // R3058：跨文档 replace（非 hash-only）→ host 真导航（replace 语义：back 不回旧 url，但跨文档仍重载）。
    if (_isCrossDocumentNav(oldHref, newHref) && typeof __zw_request_navigate === 'function') {
      __zw_request_navigate(newHref);
    }
  }

  // geolocation（R2820）——navigator.geolocation watch id 计数 + fake 零坐标位置工厂。
  var _geoWatchId = 0;
  function _makeGeoPosition() {
    return {
      coords: {
        latitude: 0,
        longitude: 0,
        altitude: null,
        accuracy: Infinity,
        altitudeAccuracy: null,
        heading: null,
        speed: null,
      },
      timestamp: 0,
    };
  }

  // PermissionStatus（WAB2-M3-s1，2026-09-24）——permissions.query 返回类型（WPT permission 案
  // `status instanceof PermissionStatus` 断言）。非法构造（spec [Exposed=(Window)] interface 无
  // 构造器语义）；实例经 query 内 Object.create(prototype) 构造，name/state 只读 getter +
  // onchange IDL 属性 + addEventListener/removeEventListener/dispatchEvent（change 事件 headless
  // 无源，no-op 注册）。
  // https://w3c.github.io/permissions/#dom-permissionstatus
  if (typeof globalThis.PermissionStatus !== 'function') {
    globalThis.PermissionStatus = function PermissionStatus() {
      throw new TypeError("Illegal constructor");
    };
    Object.defineProperty(globalThis.PermissionStatus.prototype, 'name', {
      configurable: true,
      get: function () { return this._zwPermName; },
    });
    Object.defineProperty(globalThis.PermissionStatus.prototype, 'state', {
      configurable: true,
      // security-hardening M4：state 为**活值**——底层状态变化（__zwSetPermission /
      // permissions.request）后既有 status 实例如实反映（change 事件通知语义的前提，
      // spec PermissionStatus.state 为当前状态查询而非快照）。
      get: function () {
        if (typeof globalThis.__zwPermissionState === 'function') {
          return globalThis.__zwPermissionState(this._zwPermName);
        }
        return this._zwPermState || 'prompt';
      },
    });
    Object.defineProperty(globalThis.PermissionStatus.prototype, 'onchange', {
      configurable: true,
      get: function () { return this._zwPermOnChange || null; },
      set: function (v) { this._zwPermOnChange = v; },
    });
    globalThis.PermissionStatus.prototype.addEventListener = function (type, fn) {
      if (typeof fn !== 'function') return;
      var ls = this._zwPermListeners || (this._zwPermListeners = {});
      (ls[type] || (ls[type] = [])).push(fn);
    };
    globalThis.PermissionStatus.prototype.removeEventListener = function (type, fn) {
      var ls = this._zwPermListeners && this._zwPermListeners[type];
      if (!ls) return;
      var i = ls.indexOf(fn);
      if (i >= 0) ls.splice(i, 1);
    };
    globalThis.PermissionStatus.prototype.dispatchEvent = function () { return false; };
  }

  globalThis.navigator = {
    // https://html.spec.whatwg.org/multipage/system-state.html#dom-navigator-useragent
    userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) ZeroBrowser/__ZERO_BUILD_VERSION__ Chrome/120.0.0.0',
    appName: 'Netscape',
    appVersion: '5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
    appCodeName: 'Mozilla',
    product: 'Gecko',
    productSub: '20030107',
    vendor: 'Google Inc.',
    vendorSub: '',
    platform: 'Win32',
    language: 'en-US',
    languages: ['en-US', 'en'],
    onLine: true,
    cookieEnabled: true,
    doNotTrack: null,
    hardwareConcurrency: 4,
    maxTouchPoints: 0,
    // R2988 navigator 环境信息——RUM/analytics（GA）/ 自适应加载库 feature-detect 读取。
    // deviceMemory（GB，spec 离散值 0.25/0.5/1/2/4/8，取 8 常见值）。
    deviceMemory: 8,
    // Network Information API——自适应加载（按 effectiveType 选图片/脚本质量）+ RUM 上报高频。
    // headless 无真网络探测 → 静态 '4g' 近似（real 浏览器桌面默认亦 '4g'）。change 事件注册有效但不触发
    //（headless 无网络变化）。addEventListener/removeEventListener/onchange 经 EventTarget-like no-op。
    connection: {
      effectiveType: '4g',
      type: 'wifi',
      downlink: 10,
      rtt: 50,
      saveData: false,
      addEventListener: function () {},
      removeEventListener: function () {},
      dispatchEvent: function () { return true; }
    },
    // UA Client Hints（navigator.userAgentData，R2988）——modern 替代 navigator.userAgent 字符串解析，
    // analytics / fingerprinting-defense 库 feature-detect 读 brands/mobile/platform。getHighEntropyValues
    // 返 Promise（spec 异步），headless 静态值（无真 UA 解析）。
    userAgentData: {
      brands: [
        { brand: 'Chromium', version: '120' },
        { brand: 'Not(A:Brand', version: '8' },
        { brand: 'ZeroBrowser', version: '__ZERO_BUILD_VERSION__' }
      ],
      mobile: false,
      platform: 'Windows',
      getHighEntropyValues: function (hints) {
        var h = String(hints == null ? '' : hints);
        var out = { brands: this.brands.slice(), mobile: this.mobile, platform: this.platform };
        // 按 hints 请求补高熵字段（headless 静态值）。
        if (h.indexOf('platformVersion') >= 0) out.platformVersion = '15.0.0';
        if (h.indexOf('architecture') >= 0) out.architecture = 'x86';
        if (h.indexOf('model') >= 0) out.model = '';
        if (h.indexOf('uaFullVersion') >= 0) out.uaFullVersion = '120.0.0.0';
        if (h.indexOf('bitness') >= 0) out.bitness = '64';
        if (h.indexOf('fullVersionList') >= 0) out.fullVersionList = this.brands.slice();
        return Promise.resolve(out);
      },
      toJSON: function () { return { brands: this.brands, mobile: this.mobile, platform: this.platform }; }
    },
    webdriver: false,
    plugins: _emptyCollection(),
    mimeTypes: _emptyCollection(),
    javaEnabled: function() { return false; },
    taintEnabled: function() { return false; },
    // clipboard（R2817 + R2964 + WAB2-M2）——异步剪贴板 API（复制按钮 ubiquitous）。headless 无 OS 剪贴板 →
    // **进程内 store**（IIFE 闭包）。R2817/R2964：writeText/readText 真实往返（复制按钮 + 粘贴检查高频
    // 模式）。WAB2-M2（web-api-batch2 goal M2 切片 1，2026-09-23）：富 MIME 面——globalThis.ClipboardItem
    //（record<DOMString, (Blob|DOMString|Promise)> 值 + types/presentationStyle/getType + epoch 失效）+
    // globalThis.Clipboard 接口（instanceof 面）+ read/write 存取。spec
    // https://w3c.github.io/clipboard-apis/#async-clipboard-api。
    // **诚实范围**：① headless 内存后端（同页同进程语义）；OS 剪贴板后端挂 M4（host-runtime 能力评估）；
    // ② 权限 denied 拒绝语义已落（WAB2-M2-s3）——denied → NotAllowedError 拒绝，'prompt' 按 WebKit
    // 风格不拦截（headless 无权限提示 UI；user-activation.js 同款现状注释）；完整权限语义层仍归
    // security-hardening DC-4 对齐；③ write 仅支持单项（>1 项 NotAllowedError，上游用例注释 "not
    // implemented" 同款现状）；④ read() 空store 返 []。
    clipboard: (function () {
      var _current = null; // { blobs: {type: Blob}, types: [type…] }——最近一次 write/writeText 内容
      var _epoch = 0;      // store 代际——read() 返回项绑定当代，后续 write 令旧项 getType 失效（InvalidStateError）
      var _RE = globalThis.DOMException || Error;
      // WAB2-M2-s3：权限状态注册表（name → 'prompt'|'granted'|'denied'，默认 'prompt'）。
      // 注入经 __zwSetPermission（runner testdriver set_permission stub 调用），查询经
      // __zwPermissionState（navigator.permissions.query 消费）——__zw 前缀内部钩子约定同
      // __zwClipboardStoreWrite。spec read/write 步骤 "check clipboard read/write permission"：
      // https://w3c.github.io/clipboard-apis/#dom-clipboard-read 与 #dom-clipboard-write。
      var _permStates = {};
      if (typeof globalThis.__zwSetPermission !== 'function') {
        globalThis.__zwSetPermission = function (name, state) {
          if (state !== 'granted' && state !== 'denied' && state !== 'prompt') {
            return Promise.reject(new TypeError("Failed to set permission: unknown state '" + state + "'."));
          }
          var prev = _permStates[String(name)];
          _permStates[String(name)] = state;
          // security-hardening M4：状态实际变化 → change 通知（既有单例 status 的
          // listener/onchange；helper 由 permissions IIFE 挂载，lazy 调用容忍求值序）。
          if (prev !== state && typeof globalThis.__zwNotifyPermissionChange === 'function') {
            try { globalThis.__zwNotifyPermissionChange(String(name)); } catch (_eN) {}
          }
          return Promise.resolve();
        };
      }
      if (typeof globalThis.__zwPermissionState !== 'function') {
        globalThis.__zwPermissionState = function (name) {
          return _permStates[String(name)] || 'prompt';
        };
      }
      // denied → NotAllowedError（spec：权限检查未通过即 reject，promise-returning 不同步抛）。
      function _permDenied(name) {
        return _permStates[name] === 'denied';
      }
      // WAB2-M2-s3：read(options) 字典校验（WebIDL 转换先于算法体，promise-returning → 异常走
      // rejected promise）。unsanitized 为 sequence<DOMString>：null/非序列 → TypeError；仅支持
      // 单项 'text/html'（上游 unsanitized read-fail 案：多格式或其他格式 → NotAllowedError）；
      // 空序列/缺省 → 常规读。当前 read 本就不改写内容，'text/html' 通道与常规读同载荷。
      function _validateUnsanitized(options) {
        if (options == null || typeof options !== 'object') return;
        var u = options.unsanitized;
        if (u === undefined) return;
        if (u === null || typeof u !== 'object' || typeof u.length !== 'number') {
          throw new TypeError("Failed to execute 'read' on 'Clipboard': Failed to read the 'unsanitized' property from 'ClipboardReadOptions': The provided value cannot be converted to a sequence.");
        }
        if (u.length === 0) return;
        if (u.length !== 1 || String(u[0]) !== 'text/html') {
          throw new _RE("Failed to execute 'read' on 'Clipboard': Unsupported unsanitized format.", 'NotAllowedError');
        }
      }
      // WAB2-M2-s3：image/* 载荷校验（spec write 步骤 "parse the image"——无法解析 → DataError；
      // 上游 malformed 案：文本冒充 image/png → DataError）。headless 无图片解码器 → 按类型魔数
      // 甄别，未识别子类型不拦截（真 PNG 载荷 write/read-image 案不受影响）。
      function _validateImageMagic(type, blob) {
        var bytes = _zw_blobBytes(blob);
        var head = '';
        for (var i = 0; i < bytes.length && i < 12; i++) head += String.fromCharCode(bytes[i]);
        var ok;
        if (type === 'image/png') ok = head.indexOf('PNG') === 0;
        else if (type === 'image/jpeg' || type === 'image/jpg') ok = head.indexOf('ÿØÿ') === 0;
        else if (type === 'image/gif') ok = head.indexOf('GIF8') === 0;
        else if (type === 'image/webp') ok = head.indexOf('RIFF') === 0 && head.indexOf('WEBP', 8) === 8;
        else if (type === 'image/bmp' || type === 'image/x-ms-bmp') ok = head.indexOf('BM') === 0;
        else if (type === 'image/svg+xml') ok = head.indexOf('<') === 0;
        else return; // 未识别 image 子类型不校验
        if (!ok) {
          throw new _RE("Failed to execute 'write' on 'Clipboard': Malformed image data for type '" + type + "'.", 'DataError');
        }
      }

      // 值归一：Blob | DOMString | Promise<Blob|DOMString> → Promise<Blob>（DOMString→Blob(type)）。
      function _resolveValue(value, type) {
        return Promise.resolve(value).then(function (resolved) {
          if (typeof resolved === 'string') return new Blob([resolved], { type: type });
          return resolved;
        });
      }

      // globalThis.ClipboardItem（幂等守卫同 Blob 块）。值**存原始引用**（含未 settle 的 Promise），
      // getType/write 时归一；_epoch -1 = 未绑定 store（write 输入原项），read() 返回项绑定当代。
      var ClipboardItem = globalThis.ClipboardItem;
      if (typeof ClipboardItem !== 'function') {
        ClipboardItem = function ClipboardItem(options) {
          if (options == null || typeof options !== 'object') {
            throw new TypeError("Failed to construct 'ClipboardItem': The provided value is not of type 'record'.");
          }
          this._values = {};
          var keys = Object.keys(options);
          for (var i = 0; i < keys.length; i++) this._values[keys[i]] = options[keys[i]];
          this._epoch = -1;
        };
        globalThis.ClipboardItem = ClipboardItem;
      }
      Object.defineProperty(ClipboardItem.prototype, 'types', {
        get: function () { return Object.keys(this._values); },
        configurable: true,
      });
      Object.defineProperty(ClipboardItem.prototype, 'presentationStyle', {
        get: function () { return 'unspecified'; },
        configurable: true,
      });
      // getType(type) → Promise<Blob>：missing → NotFoundError；read 项遇新 write（代际失配）→
      // InvalidStateError（cached-getType-reject 上游案：缓存不绕过失效检查）。
      ClipboardItem.prototype.getType = function (type) {
        var self = this;
        var t = String(type);
        if (self._epoch >= 0 && self._epoch !== _epoch) {
          return Promise.reject(new _RE("Failed to execute 'getType' on 'ClipboardItem': The item is stale.", 'InvalidStateError'));
        }
        if (self._values[t] === undefined) {
          return Promise.reject(new _RE("Failed to execute 'getType' on 'ClipboardItem': The type was not found.", 'NotFoundError'));
        }
        return _resolveValue(self._values[t], t).then(function (blob) {
          if (!(blob instanceof Blob)) {
            throw new TypeError("Failed to execute 'getType' on 'ClipboardItem': The value is not a Blob.");
          }
          return blob;
        });
      };

      // globalThis.Clipboard 接口（幂等守卫）。promise-returning 操作：转换错误一律走
      // rejected Promise（WebIDL promise-returning 不同步抛）。
      var Clipboard = globalThis.Clipboard;
      if (typeof Clipboard !== 'function') {
        Clipboard = function Clipboard() {};
        globalThis.Clipboard = Clipboard;
      }
      Clipboard.prototype.read = function () {
        try {
          // WAB2-M2-s3：read(options) 字典校验先于权限门（WebIDL 转换先于算法体）。
          _validateUnsanitized(arguments[0]);
          if (_permDenied('clipboard-read')) {
            return Promise.reject(new _RE("Failed to execute 'read' on 'Clipboard': Permission denied.", 'NotAllowedError'));
          }
          if (_current === null) return Promise.resolve([]);
          var blobs = {};
          for (var k in _current.blobs) blobs[k] = _current.blobs[k];
          var item = new ClipboardItem(blobs);
          item._epoch = _epoch;
          return Promise.resolve([item]);
        } catch (e) {
          return Promise.reject(e);
        }
      };
      Clipboard.prototype.readText = function () {
        if (_permDenied('clipboard-read')) {
          return Promise.reject(new _RE("Failed to execute 'readText' on 'Clipboard': Permission denied.", 'NotAllowedError'));
        }
        if (_current === null || !_current.blobs['text/plain']) return Promise.resolve('');
        return Promise.resolve(_zw_utf8_decode(_zw_blobBytes(_current.blobs['text/plain'])));
      };
      Clipboard.prototype.write = function (data) {
        try {
          if (data == null || typeof data !== 'object' || typeof data.length !== 'number') {
            throw new TypeError("Failed to execute 'write' on 'Clipboard': The provided value cannot be converted to a sequence.");
          }
          for (var i = 0; i < data.length; i++) {
            if (!(data[i] instanceof ClipboardItem)) {
              throw new TypeError("Failed to execute 'write' on 'Clipboard': sequence element is not a ClipboardItem.");
            }
          }
          if (data.length > 1) {
            return Promise.reject(new _RE('write only supports a single ClipboardItem', 'NotAllowedError'));
          }
          // WAB2-M2-s3：denied → NotAllowedError（输入校验之后，权限门先于载荷解析）。
          if (_permDenied('clipboard-write')) {
            return Promise.reject(new _RE("Failed to execute 'write' on 'Clipboard': Permission denied.", 'NotAllowedError'));
          }
          if (data.length === 0) return Promise.resolve(undefined);
          var input = data[0];
          var types = Object.keys(input._values);
          return Promise.all(
            types.map(function (t) {
              var raw = input._values[t];
              // image/* 必须 Blob（DOMString 值上游案 "image/png DOMString fails" → TypeError）。
              if (typeof raw === 'string' && t.indexOf('image/') === 0) {
                throw new TypeError('ClipboardItem with type ' + t + ' requires a Blob value.');
              }
              return _resolveValue(raw, t).then(function (blob) {
                if (!(blob instanceof Blob)) {
                  throw new TypeError('ClipboardItem value is not a Blob or DOMString.');
                }
                // WAB2-M2-s3：image/* 载荷魔数校验（throw 在 promise 链内 → DataError 拒绝）。
                if (t.indexOf('image/') === 0) _validateImageMagic(t, blob);
                return [t, blob];
              });
            })
          ).then(function (pairs) {
            var blobs = {};
            var order = [];
            for (var i = 0; i < pairs.length; i++) {
              blobs[pairs[i][0]] = pairs[i][1];
              order.push(pairs[i][0]);
            }
            _current = { blobs: blobs, types: order };
            _epoch++;
            return undefined;
          });
        } catch (e) {
          return Promise.reject(e);
        }
      };
      Clipboard.prototype.writeText = function (text) {
        if (arguments.length < 1) {
          return Promise.reject(new TypeError("Failed to execute 'writeText' on 'Clipboard': 1 argument required, but only 0 present."));
        }
        // WAB2-M2-s3：denied → NotAllowedError（上游 writeText-denied 案）。
        if (_permDenied('clipboard-write')) {
          return Promise.reject(new _RE("Failed to execute 'writeText' on 'Clipboard': Permission denied.", 'NotAllowedError'));
        }
        _current = { blobs: { 'text/plain': new Blob([String(text != null ? text : '')], { type: 'text/plain' }) }, types: ['text/plain'] };
        _epoch++;
        return Promise.resolve(undefined);
      };
      // WAB2-M2-s2：execCommand('copy'/'cut') defaultPrevented 桥（part06 调用）——
      // spec copy 响应 "update the clipboard content"（handler 经 clipboardData.setData
      // 提供的内容即剪贴板新内容）。内部钩子（__zw 前缀同 __zw_opfs 约定，非 web 面）。
      // read-sanitize/read-resource-load 上游案：oncopy preventDefault + setData →
      // navigator.clipboard.read() 取同内容。
      if (typeof globalThis.__zwClipboardStoreWrite !== 'function') {
        globalThis.__zwClipboardStoreWrite = function (typeToText) {
          if (typeToText == null || typeof typeToText !== 'object') return;
          var blobs = {};
          var order = [];
          for (var t in typeToText) {
            if (!Object.prototype.hasOwnProperty.call(typeToText, t)) continue;
            blobs[t] = new Blob([String(typeToText[t])], { type: t });
            order.push(t);
          }
          if (order.length === 0) return;
          _current = { blobs: blobs, types: order };
          _epoch++;
        };
      }
      return new Clipboard();
    })(),
    // R3314：storage（Storage API + OPFS Origin Private File System）——Done Criteria §3 Tier 2 列项
    //（zero-web.md 行 80「IndexedDB + Cache API + OPFS」，OPFS 此前全缺）。estimate（配额查询，analytics 高频）+
    // getDirectory（OPFS root）。storage-opfs M2（2026-09-09）：页面面从**进程内虚拟 FS 树**切换到
    // `__zw_opfs` 宿主命令 → zero-storage opfs 模块真实实现（per-origin 树 + 可选落盘；见
    // zero-page-runtime opfs_host / zero-storage opfs.rs）。**kill-switch**：`__zw_opfs` 未注册
    //（无 Rust 宿主 / 老宿主）→ 回落内存虚拟树（M1 前行为，零回归路径）。
    // spec https://fs.spec.whatwg.org/。**诚实范围**：① 仅 OPFS（navigator.storage.getDirectory），
    // 无 showOpenFilePicker/showSaveFilePicker（用户可见文件选择器，headless 无）；
    // ② 无 createSyncAccessHandle（worker 同步句柄，headless worker 无真线程）；
    // ③ 无 permission/move/transferable。
    // R3314：storage（Storage API + OPFS Origin Private File System）——Done Criteria §3 Tier 2 列项
    //（zero-web.md 行 80「IndexedDB + Cache API + OPFS」，OPFS 此前全缺）。estimate（配额查询，analytics 高频）+
    // getDirectory（OPFS root）。storage-opfs M2（2026-09-09）：页面面从**进程内虚拟 FS 树**切换到
    // `__zw_opfs` 宿主命令 → zero-storage opfs 模块真实实现（per-origin 树 + 可选落盘；见
    // zero-page-runtime opfs_host / zero-storage opfs.rs）。**kill-switch**：`__zw_opfs` 未注册
    //（无 Rust 宿主 / 老宿主）→ 回落内存虚拟树（同构面，零回归路径）。
    // spec https://fs.spec.whatwg.org/。**诚实范围**：① 仅 OPFS（navigator.storage.getDirectory），
    // 无 showOpenFilePicker/showSaveFilePicker（用户可见文件选择器，headless 无）；
    // ② 无 createSyncAccessHandle（worker 同步句柄，headless worker 无真线程）；
    // ③ 无 permission/move/transferable（postMessage 克隆句柄面）。
    storage: (function () {
      // 全局 FileSystemHandle 类（spec instanceof 断言面；WPT iteration/isSameEntry 断言
      // `entry[1] instanceof FileSystemFileHandle` 等）。
      if (typeof globalThis.FileSystemFileHandle !== 'function') {
        globalThis.FileSystemFileHandle = function FileSystemFileHandle() {};
        globalThis.FileSystemDirectoryHandle = function FileSystemDirectoryHandle() {};
      }
      // R3254-C14：OPFS 句柄名称校验（spec FS §7：空串、'.'、'..'、含 '/' → 无效）。
      function _zwFsValidName(n) {
        return typeof n === 'string' && n.length > 0 && n !== '.' && n !== '..' && n.indexOf('/') < 0;
      }
      function toBytes(data) {
        if (data == null) return new Uint8Array(0);
        if (typeof data === 'string') return _zw_utf8_encode(data);
        if (data instanceof Uint8Array) return new Uint8Array(data);
        if (data instanceof Blob) return _zw_blobBytes(data).slice(); // 同步取（headless 近似）
        if (data instanceof ArrayBuffer) return new Uint8Array(data);
        if (data.byteLength != null && data.buffer != null) {
          return new Uint8Array(data.buffer, data.byteOffset || 0, data.byteLength);
        }
        return new Uint8Array(0);
      }
      // spec async iterator 骨架：{next()} 快照数组 + [Symbol.asyncIterator] 自返。
      function asyncIterator(pullItems) {
        var items = null;
        var index = 0;
        function pull() {
          if (items === null) items = pullItems();
          if (index >= items.length) return Promise.resolve({ done: true, value: undefined });
          return Promise.resolve({ done: false, value: items[index++] });
        }
        return { next: pull, [Symbol.asyncIterator]: function () { return this; } };
      }
      // 宿主命令可用 → 真实 Rust 后端（per-origin；宿主从 page_url 推导 origin，页面不可伪造）。
      if (typeof globalThis.__zw_opfs === 'function') {
        return (function () {
          // 同步 wire 调用（`__zw_opfs_ok:` / `__zw_opfs_error:Name|code|message`），
          // 照 IndexedDB `_zwIDBHostCall` 约定。错误映射：TypeError|0| → JS TypeError；
          // 其余 → DOMException(name)（WPT assert_throws_dom 同时断言 name 与 legacy code）。
          function hostCall(request) {
            var wire = String(globalThis.__zw_opfs(JSON.stringify(request)));
            var okPrefix = '__zw_opfs_ok:';
            var errorPrefix = '__zw_opfs_error:';
            if (wire.indexOf(okPrefix) === 0) {
              return JSON.parse(wire.slice(okPrefix.length));
            }
            if (wire.indexOf(errorPrefix) === 0) {
              var detail = wire.slice(errorPrefix.length);
              var parts = detail.split('|');
              var name = parts[0] || 'UnknownError';
              var code = parseInt(parts[1], 10);
              var message = parts.length >= 3 ? parts.slice(2).join('|') : detail;
              if (name === 'TypeError') throw new TypeError(message);
              var ex = new globalThis.DOMException(message, name);
              // legacy code（WPT assert_throws_dom 断言 e.code === 期望值）。
              try { if (!isNaN(code)) ex.code = code; } catch (_e) {}
              throw ex;
            }
            throw new globalThis.DOMException('Invalid OPFS host response.', 'UnknownError');
          }
          // getUniqueId 按路径+kind 键控缓存（Rust 侧每次生成新 GUID；「同路径同 ID /
          // 写后不变」由本缓存保证；file/dir 同路径异 ID → kind 入键，WPT 断言）。
          var uniqueIds = {};
          function uniqueId(path, kind) {
            var key = kind + ':' + JSON.stringify(path);
            if (!uniqueIds[key]) {
              uniqueIds[key] = hostCall({ op: 'getUniqueId', path: path }).uniqueId;
            }
            return uniqueIds[key];
          }
          function handleCommon(proto, path, kind) {
            var h = Object.create(proto);
            h.kind = kind;
            h.name = path.length > 0 ? path[path.length - 1] : '';
            h.path = path;
            h.isSameEntry = function (other) {
              var self = this;
              return Promise.resolve().then(function () {
                // 路径同一性 = 同一条目（同路径同 kind 判 true，与获取次数无关）。
                return !!other && typeof other.path !== 'undefined' && other.kind === self.kind &&
                  JSON.stringify(other.path) === JSON.stringify(self.path);
              });
            };
            h.remove = function (opts) {
              var self = this;
              return Promise.resolve().then(function () {
                hostCall({ op: 'remove', path: self.path, recursive: !!(opts && opts.recursive) });
                return undefined;
              });
            };
            h.getUniqueId = function () {
              var self = this;
              return Promise.resolve().then(function () { return uniqueId(self.path, self.kind); });
            };
            return h;
          }
          function dirHandle(path) {
            var h = handleCommon(globalThis.FileSystemDirectoryHandle.prototype, path, 'directory');
            h.getFileHandle = function (n, opts) {
              var self = this;
              return Promise.resolve().then(function () {
                var r = hostCall({ op: 'getFileHandle', parent: self.path, name: n, create: !!(opts && opts.create) });
                return fileHandle(r.path);
              });
            };
            h.getDirectoryHandle = function (n, opts) {
              var self = this;
              return Promise.resolve().then(function () {
                var r = hostCall({ op: 'getDirectoryHandle', parent: self.path, name: n, create: !!(opts && opts.create) });
                return dirHandle(r.path);
              });
            };
            h.removeEntry = function (n, opts) {
              var self = this;
              return Promise.resolve().then(function () {
                hostCall({ op: 'removeEntry', parent: self.path, name: n, recursive: !!(opts && opts.recursive) });
                return undefined;
              });
            };
            h.resolve = function (child) {
              var self = this;
              return Promise.resolve().then(function () {
                if (!child || typeof child.path === 'undefined') return null;
                var r = hostCall({ op: 'resolve', dir: self.path, child: child.path });
                return r.path;
              });
            };
            h.entries = function () {
              var self = this;
              return asyncIterator(function () {
                return hostCall({ op: 'listEntries', dir: self.path }).entries.map(function (entry) {
                  var child = dirOrFile(entry, self.path);
                  return [entry.name, child];
                });
              });
            };
            h.keys = function () {
              var self = this;
              return asyncIterator(function () {
                return hostCall({ op: 'listEntries', dir: self.path }).keys;
              });
            };
            h.values = function () {
              var self = this;
              return asyncIterator(function () {
                return hostCall({ op: 'listEntries', dir: self.path }).entries.map(function (entry) {
                  return dirOrFile(entry, self.path);
                });
              });
            };
            // 句柄本身可 await-iterate（= entries()：直接迭代产出 [name, handle] 对；
            // WPT `for await (let entry of root)` 断言 Array.isArray(entry)）。
            h[Symbol.asyncIterator] = function () { return this.entries(); };
            return h;
          }
          function fileHandle(path) {
            var h = handleCommon(globalThis.FileSystemFileHandle.prototype, path, 'file');
            // getFile() → Blob（name/size/lastModified 元数据 + 内容快照；WPT getFile 断言面）。
            h.getFile = function () {
              var self = this;
              return Promise.resolve().then(function () {
                var f = hostCall({ op: 'getFile', path: self.path });
                var blob = new Blob([new Uint8Array(f.data)], { type: 'application/octet-stream' });
                blob.name = f.name;
                blob.lastModified = f.lastModified;
                return blob;
              });
            };
            h.createWritable = function (opts) {
              var self = this;
              return Promise.resolve().then(function () {
                var keep = !!(opts && opts.keepExistingData);
                var r = hostCall({ op: 'createWritable', path: self.path, keepExistingData: keep });
                return writableFileStream(r.streamId);
              });
            };
            return h;
          }
          function dirOrFile(entry, parentPath) {
            var childPath = parentPath.concat([entry.name]);
            return entry.kind === 'directory' ? dirHandle(childPath) : fileHandle(childPath);
          }
          // FileSystemWritableFileStream：真 WritableStream 子类面（getWriter/pipeTo/locked）
          // + write/seek/truncate/close/abort 直通方法。sink.write 映射 hostCall streamWrite
          //（write/seek/truncate 命令），sink.close 映射 streamClose（单次成功语义在 Rust 侧）。
          function writableFileStream(streamId) {
            var closed = false;
            function syntaxError(message) {
              // WriteParams 缺参 → SyntaxError DOMException（WPT promise_rejects_dom 断言；
              // SyntaxError 为新式异常 → legacy code 0）。
              return new globalThis.DOMException(message, 'SyntaxError');
            }
            // 命令对象判定：Blob 有 .type（MIME 字符串）——用「非 Blob 且有 string type」
            // 界定 WriteParams 命令对象。
            function isCommandObject(chunk) {
              return chunk && typeof chunk === 'object' && typeof chunk.type === 'string' && !(chunk instanceof Blob);
            }
            // 参数错误路径：write 拒绝 → stream error（WritableStream 语义：errored 后
            // 释放锁并解除 removeEntry 守卫）→ abort 宿主流槽（open-writer 计数归还）。
            function rejectInvalid(error) {
              if (!closed) {
                closed = true;
                try { hostCall({ op: 'streamAbort', streamId: streamId }); } catch (_e) {}
              }
              return Promise.reject(error);
            }
            function typeErrorAfterClose() {
              // WPT write/truncate-after-close 断言 promise_rejects_js TypeError。
              return new TypeError('stream 已关闭');
            }
            var sink = {
              write: function (chunk) {
                if (closed) return Promise.reject(typeErrorAfterClose());
                // 对象命令形式 write({type:'seek'|'truncate'|'write', position/size/data})。
                if (isCommandObject(chunk)) {
                  if (chunk.type === 'seek') {
                    if (chunk.position === undefined) return rejectInvalid(syntaxError('seek without position'));
                    if (typeof chunk.position === 'number' && chunk.position < 0) return rejectInvalid(new TypeError('seek: position 不能为负'));
                    return Promise.resolve(hostCall({ op: 'streamWrite', streamId: streamId, command: { type: 'seek', position: chunk.position } }));
                  }
                  if (chunk.type === 'truncate') {
                    if (chunk.size === undefined) return rejectInvalid(syntaxError('truncate without size'));
                    return Promise.resolve(hostCall({ op: 'streamWrite', streamId: streamId, command: { type: 'truncate', size: chunk.size } }));
                  }
                  // type === 'write'
                  if (typeof chunk.position === 'number' && chunk.position < 0) return rejectInvalid(new TypeError('write: position 不能为负'));
                  if (chunk.data === undefined) return rejectInvalid(syntaxError('write without data'));
                  if (chunk.data === null) return rejectInvalid(new TypeError('write with null data'));
                  return Promise.resolve(hostCall({ op: 'streamWrite', streamId: streamId, command: { type: 'write', position: chunk.position, data: Array.prototype.slice.call(toBytes(chunk.data)) } }));
                }
                // 简单形式 write(data)；null/undefined → TypeError（WPT WriteParams null data 面）。
                if (chunk == null) return rejectInvalid(new TypeError('write with null data'));
                return Promise.resolve(hostCall({ op: 'streamWrite', streamId: streamId, command: { type: 'write', position: null, data: Array.prototype.slice.call(toBytes(chunk)) } }));
              },
              close: function () {
                if (closed) return Promise.reject(typeErrorAfterClose());
                closed = true;
                return Promise.resolve(hostCall({ op: 'streamClose', streamId: streamId }));
              },
              abort: function () {
                closed = true;
                return Promise.resolve(hostCall({ op: 'streamAbort', streamId: streamId }));
              },
            };
            var stream = new WritableStream(sink);
            // 直通方法（spec：FileSystemWritableFileStream 兼有 WritableStream 面 + 直 write/seek/
            // truncate/close 面——不经 getWriter 锁，直接驱动同一 sink）。
            stream.write = sink.write;
            stream.seek = function (position) {
              if (closed) return Promise.reject(typeErrorAfterClose());
              if (typeof position === 'number' && position < 0) return Promise.reject(new TypeError('seek: position 不能为负'));
              return sink.write({ type: 'seek', position: position });
            };
            stream.truncate = function (size) {
              if (typeof size !== 'number') return rejectInvalid(syntaxError('truncate without size'));
              if (closed) return Promise.reject(typeErrorAfterClose());
              return sink.write({ type: 'truncate', size: size });
            };
            stream.close = sink.close;
            return stream;
          }
          return {
            estimate: function () {
              return Promise.resolve().then(function () {
                var usage = hostCall({ op: 'estimate' }).usage;
                return { usage: usage, quota: 1024 * 1024 * 100 };
              });
            },
            getDirectory: function () {
              return Promise.resolve().then(function () {
                hostCall({ op: 'getDirectory' });
                return dirHandle([]);
              });
            },
          };
        })();
      }
      // ── kill-switch 回退：内存虚拟树（`__zw_opfs` 未注册时的 no-storage 路径；M1 前行为）──
      // 结构与真实路径同形（路径句柄 + async iterator + DOMException），数据进程内、不持久化。
      return (function () {
        // 目录 = {}（键 → 子节点），文件 = 字节数组。kind 由值类型判（数组=文件 / 对象=目录）。
        var memTree = {};
        function memNode(path) {
          var node = memTree;
          for (var i = 0; i < path.length; i++) {
            if (!node || typeof node !== 'object' || Array.isArray(node)) return null;
            node = node[path[i]];
          }
          return node === undefined ? null : node;
        }
        function memWrite(path, bytes) {
          var node = memTree;
          for (var i = 0; i < path.length - 1; i++) {
            if (!node[path[i]]) node[path[i]] = {};
            node = node[path[i]];
          }
          node[path[path.length - 1]] = bytes;
        }
        function memDelete(path) {
          var node = memTree;
          for (var i = 0; i < path.length - 1; i++) {
            if (!node[path[i]]) return;
            node = node[path[i]];
          }
          delete node[path[path.length - 1]];
        }
        function memNames(path) {
          var node = memNode(path);
          if (node && !Array.isArray(node)) return Object.keys(node).sort();
          return [];
        }
        function memKind(path) {
          if (path.length === 0) return 'directory';
          var node = memNode(path);
          if (node === null) return null;
          return Array.isArray(node) ? 'file' : 'directory';
        }
        function handleCommon(path, kind) {
          var h = Object.create(kind === 'file' ? globalThis.FileSystemFileHandle.prototype : globalThis.FileSystemDirectoryHandle.prototype);
          h.kind = kind;
          h.name = path.length > 0 ? path[path.length - 1] : '';
          h.path = path;
          h.isSameEntry = function (other) {
            var self = this;
            return Promise.resolve().then(function () {
              return !!other && typeof other.path !== 'undefined' && other.kind === self.kind &&
                JSON.stringify(other.path) === JSON.stringify(self.path);
            });
          };
          h.remove = function (opts) {
            var self = this;
            return Promise.resolve().then(function () {
              if (self.path.length === 0) { memTree = {}; return undefined; }
              if (memKind(self.path) === null) return Promise.reject(_zwDomException('不存在', 'NotFoundError'));
              memDelete(self.path);
              return undefined;
            });
          };
          h.getUniqueId = function () {
            var self = this;
            return Promise.resolve().then(function () { return 'mem-' + self.kind + '-' + JSON.stringify(self.path); });
          };
          return h;
        }
        function dirHandle(path) {
          var h = handleCommon(path, 'directory');
          h.getFileHandle = function (n, opts) {
            var self = this;
            return Promise.resolve().then(function () {
              if (!_zwFsValidName(n)) return Promise.reject(new TypeError('无效的文件名'));
              var child = self.path.concat([n]);
              var k = memKind(child);
              if (k === 'directory') return Promise.reject(_zwDomException(n + ' 是目录', 'TypeMismatchError'));
              if (k === null && !(opts && opts.create)) return Promise.reject(_zwDomException(n + ' 不存在', 'NotFoundError'));
              if (k === null) memWrite(child, []);
              return fileHandle(child);
            });
          };
          h.getDirectoryHandle = function (n, opts) {
            var self = this;
            return Promise.resolve().then(function () {
              if (!_zwFsValidName(n)) return Promise.reject(new TypeError('无效的目录名'));
              var child = self.path.concat([n]);
              var k = memKind(child);
              if (k === 'file') return Promise.reject(_zwDomException(n + ' 是文件', 'TypeMismatchError'));
              if (k === null && !(opts && opts.create)) return Promise.reject(_zwDomException(n + ' 不存在', 'NotFoundError'));
              if (k === null) memWrite(child, {});
              return dirHandle(child);
            });
          };
          h.removeEntry = function (n, opts) {
            var self = this;
            return Promise.resolve().then(function () {
              if (!_zwFsValidName(n)) return Promise.reject(new TypeError('无效的名称'));
              var child = self.path.concat([n]);
              if (memKind(child) === null) return Promise.reject(_zwDomException(n + ' 不存在', 'NotFoundError'));
              var kids = memNames(child);
              if (kids.length > 0 && !(opts && opts.recursive)) {
                return Promise.reject(_zwDomException('目录非空（需 recursive）', 'InvalidModificationError'));
              }
              memDelete(child);
              return undefined;
            });
          };
          h.resolve = function (child) {
            var self = this;
            return Promise.resolve().then(function () {
              if (!child || typeof child.path === 'undefined') return null;
              var childPath = child.path;
              if (childPath.length < self.path.length) return null;
              for (var i = 0; i < self.path.length; i++) {
                if (childPath[i] !== self.path[i]) return null;
              }
              return childPath.slice(self.path.length);
            });
          };
          h.entries = function () {
            var self = this;
            return asyncIterator(function () {
              return memNames(self.path).map(function (name) {
                var k = memKind(self.path.concat([name]));
                return [name, k === 'directory' ? dirHandle(self.path.concat([name])) : fileHandle(self.path.concat([name]))];
              });
            });
          };
          h.keys = function () {
            var self = this;
            return asyncIterator(function () { return memNames(self.path); });
          };
          h.values = function () {
            var self = this;
            return asyncIterator(function () {
              return memNames(self.path).map(function (name) {
                return memKind(self.path.concat([name])) === 'directory'
                  ? dirHandle(self.path.concat([name]))
                  : fileHandle(self.path.concat([name]));
              });
            });
          };
          h[Symbol.asyncIterator] = function () { return this.entries(); };
          return h;
        }
        function fileHandle(path) {
          var h = handleCommon(path, 'file');
          h.getFile = function () {
            var self = this;
            return Promise.resolve().then(function () {
              var data = memNode(self.path) || [];
              var blob = new Blob([new Uint8Array(data)], { type: 'application/octet-stream' });
              blob.name = self.name;
              blob.lastModified = 0;
              return blob;
            });
          };
          h.createWritable = function (opts) {
            var self = this;
            return Promise.resolve().then(function () {
              var buf = (opts && opts.keepExistingData) ? (memNode(self.path) || []).slice() : [];
              var pos = 0;
              var closed = false;
              var aborted = false;
              function writeAt(p, bytes, advance) {
                var end = p + bytes.length;
                while (buf.length < end) buf.push(0);
                for (var k = 0; k < bytes.length; k++) buf[p + k] = bytes[k];
                if (advance) pos = p + bytes.length;
              }
              return {
                write: function (chunk) {
                  return Promise.resolve().then(function () {
                    if (closed) throw new TypeError('stream 已关闭');
                    if (aborted) throw new TypeError('stream 已 abort');
                    if (chunk && typeof chunk === 'object' && typeof chunk.type === 'string' && !(chunk instanceof Blob)) {
                      if (chunk.type === 'seek') {
                        if (typeof chunk.position === 'number' && chunk.position < 0) throw new TypeError('seek: position 不能为负');
                        pos = chunk.position | 0;
                        return;
                      }
                      if (chunk.type === 'truncate') {
                        var sz = Math.max(0, chunk.size | 0);
                        buf.length = sz;
                        if (pos > sz) pos = sz;
                        return;
                      }
                      if (chunk.data == null) throw new TypeError('write without data');
                      if (typeof chunk.position === 'number' && chunk.position < 0) throw new TypeError('write: position 不能为负');
                      var wb = toBytes(chunk.data);
                      writeAt(chunk.position == null ? pos : (chunk.position | 0), wb, chunk.position == null);
                      return;
                    }
                    if (chunk == null) throw new TypeError('write with null data');
                    var b = toBytes(chunk);
                    writeAt(pos, b, true);
                  });
                },
                  seek: function (o) {
                    return Promise.resolve().then(function () {
                      if (closed) throw new TypeError('stream 已关闭');
                      if (typeof o === 'number' && o < 0) throw new TypeError('seek: position 不能为负');
                      pos = o | 0;
                    });
                  },
                truncate: function (s) {
                  return Promise.resolve().then(function () {
                    if (closed) throw new TypeError('stream 已关闭');
                    var sz = Math.max(0, s | 0);
                    buf.length = sz;
                    if (pos > sz) pos = sz;
                  });
                },
                close: function () {
                  return Promise.resolve().then(function () {
                    if (closed) throw new TypeError('stream 已关闭');
                    closed = true;
                    memWrite(self.path, buf);
                  });
                },
                abort: function () {
                  return Promise.resolve().then(function () {
                    closed = true;
                    aborted = true;
                  });
                },
              };
            });
          };
          return h;
        }
        return {
          estimate: function () {
            var bytes = 0;
            (function count(node) {
              if (Array.isArray(node)) { bytes += node.length; return; }
              if (node && typeof node === 'object') Object.keys(node).forEach(function (k) { count(node[k]); });
            })(memTree);
            return Promise.resolve({ usage: bytes, quota: 1024 * 1024 * 100 });
          },
          getDirectory: function () { return Promise.resolve(dirHandle([])); },
        };
      })();
    })(),    // sendBeacon（R2931）——页面卸载/后台分析 beacon（fire-and-forget POST，analytics/RUM 高频：GA 等
    // unload 时上报）。headless 无真网络发送（避免无人值守测试依赖外部网络）→ accept-and-return-true
    //（spec：返 true = 成功入队 best-effort；data 类型不限，忽略）。url 缺省（null/undefined）→ false。
    sendBeacon: function(url, _data) {
      return url != null;
    },
    // userActivation（WAB2-M3-s1，web-api-batch2 goal M3 切片 1，2026-09-24）——瞬态激活面
    // （User Activation API；fullscreen requestFullscreen 激活门消费同源状态）。激活经
    // __zwUserActivate 注入（runner testdriver click/send_keys/Actions.send/bless 命令签发时）。
    // spec https://html.spec.whatwg.org/multipage/interaction.html#dom-navigator-useractivation。
    // **诚实范围**：无 5s 窗口时钟（headless 同步测试序列内窗口恒满足）；hasBeenActive 粘性。
    userActivation: {
      get isActive() {
        return _zwTransientActive === true;
      },
      get hasBeenActive() {
        return _zwActiveEver === true;
      },
    },
    // permissions（R2817 + WAB2-M2-s3 + WAB2-M3-s1 + security-hardening M4）——权限
    // 查询/请求（clipboard/geolocation 等 feature-detect 配对）。headless 默认 state
    // 'prompt'（中性，既非 granted 非 denied）；clipboard-read/clipboard-write/fullscreen
    // 状态经 __zwSetPermission（runner testdriver set_permission stub）注入后由
    // __zwPermissionState 如实返回。PermissionStatus 真实例（WPT permission 案
    // instanceof 断言）+ fullscreen 描述符的 allowWithoutGesture 字典成员读取（getter
    // 触发观测；false → TypeError——explainer 案「allowWithoutGesture false is
    // unsupported」）。
    // security-hardening M4（DC-2 Permissions 面）：①单例 status——spec
    // §dom-permissions-query「identical descriptor → 同一 PermissionStatus 实例」，
    // 亦是 change 事件通知锚点（query 前注册 listener 可达后续状态变化）；②change
    // 派发——状态实际变化（__zwSetPermission/request）时对既有单例派 'change' Event
    // + 调 onchange；③request(desc)——headless 语义：无提示 UI，'prompt' 自动授予
    // （经 __zwSetPermission 走同一注册表 + 通知），'denied' 维持，resolve 同一单例；
    // desc 必填成员校验（name 非字符串 → TypeError，spec 必选字典成员）。
    // spec https://w3c.github.io/permissions/#permissions-interface。
    permissions: (function () {
      var _statuses = {};
      function _getStatus(name) {
        var s = _statuses[name];
        if (!s) {
          s = Object.create(globalThis.PermissionStatus && globalThis.PermissionStatus.prototype
            ? globalThis.PermissionStatus.prototype
            : Object.prototype);
          s._zwPermName = name;
          s._zwPermState = (typeof globalThis.__zwPermissionState === 'function')
            ? globalThis.__zwPermissionState(name)
            : 'prompt';
          s._zwPermListeners = {};
          _statuses[name] = s;
        }
        return s;
      }
      function _descriptorName(desc, method) {
        if (desc == null || typeof desc !== 'object') {
          throw new TypeError("Failed to execute '" + method + "' on 'Permissions': The provided value is not of type 'PermissionDescriptor'.");
        }
        if (typeof desc.name !== 'string' || desc.name === '') {
          throw new TypeError("Failed to execute '" + method + "' on 'Permissions': required member name is undefined.");
        }
        return desc.name;
      }
      // M4：change 派发——'change' Event + listener/onchange（派发异常互不短路）。
      globalThis.__zwNotifyPermissionChange = function (name) {
        var s = _statuses[name];
        if (!s) return;
        var ev;
        try { ev = new Event('change'); } catch (_eE) {
          ev = { type: 'change', bubbles: false, cancelable: false, defaultPrevented: false };
        }
        ev.target = s;
        var ls = s._zwPermListeners && s._zwPermListeners.change;
        if (ls) {
          for (var i = 0; i < ls.length; i++) { try { ls[i].call(s, ev); } catch (_eL) {} }
        }
        if (typeof s._zwPermOnChange === 'function') {
          try { s._zwPermOnChange.call(s, ev); } catch (_eO) {}
        }
      };
      return {
        query: function (desc) {
          // fullscreen explainer：allowWithoutGesture 成员读取（getter 触发）；false 不支持 → TypeError。
          if (desc != null && typeof desc === 'object' && desc.name === 'fullscreen') {
            var awg;
            try { awg = desc.allowWithoutGesture; } catch (_eA) { awg = undefined; }
            if (awg === false) {
              return Promise.reject(new TypeError('Querying "fullscreen" permission with "allowWithoutGesture" false is unsupported.'));
            }
          }
          var name;
          try { name = _descriptorName(desc, 'query'); } catch (_eQ) { return Promise.reject(_eQ); }
          return Promise.resolve(_getStatus(name));
        },
        // M4：request——headless 语义（无权限提示 UI）。'prompt' → 自动授予（经
        // __zwSetPermission 落同一注册表并触发 change 通知）；'denied' 维持。
        request: function (desc) {
          var name;
          try { name = _descriptorName(desc, 'request'); } catch (_eR) { return Promise.reject(_eR); }
          var state = (typeof globalThis.__zwPermissionState === 'function')
            ? globalThis.__zwPermissionState(name)
            : 'prompt';
          if (state === 'prompt' && typeof globalThis.__zwSetPermission === 'function') {
            globalThis.__zwSetPermission(name, 'granted');
          }
          return Promise.resolve(_getStatus(name));
        },
      };
    })(),
    // geolocation（R2820）——地理位置 API（地图/天气/本地化 feature-detect 后调 getCurrentPosition）。
    // headless 无真 GPS → fake 零坐标位置（latitude/longitude 0，accuracy Infinity = 无精度承诺），让
    // location 脚本走 success 路径不抛；getCurrentPosition/watchPosition 经 _defer microtask 异步调 success
    //（execute 末 checkpoint 派发，下 execute 可读，同 R2774/R2814）；watchPosition 返唯一 watch id；
    // clearWatch no-op。
    geolocation: {
      getCurrentPosition: function(success, _error, _options) {
        if (typeof success !== 'function') return;
        _defer(function() { success(_makeGeoPosition()); });
      },
      watchPosition: function(success, _error, _options) {
        _geoWatchId = _geoWatchId + 1;
        var id = _geoWatchId;
        if (typeof success === 'function') {
          _defer(function() { success(_makeGeoPosition()); });
        }
        return id;
      },
      clearWatch: function(_id) {},
    },
    // Service Worker 页面对象投影。生命周期状态来自宿主 ServiceWorkerManager；
    // setTimeout 只逐 task 投影 transition log，不自行推进生命周期。
    serviceWorker: (function () {
      var _registrations = [];
      var _controller = null;
      var _controllerChangeGeneration = 0;
      var _controllerEventWorker = null;
      var _readyResolve;
      var _ready = new Promise(function (resolve) { _readyResolve = resolve; });
      var _container;
      var _documentURL = null;
      function ServiceWorker(scriptURL, state) {
        this._et_listeners = {};
        this.scriptURL = scriptURL;
        this._state = state;
        this._controllerEventState = null;
        this.onstatechange = null;
      }
      function ServiceWorkerRegistration() {
        this._et_listeners = {};
      }
      // https://w3c.github.io/ServiceWorker/#serviceworkercontainer-interface
      function ServiceWorkerContainer() {
        throw new TypeError('Illegal constructor');
      }
      if (typeof Symbol === 'function' && Symbol.toStringTag) {
        Object.defineProperty(ServiceWorker.prototype, Symbol.toStringTag, {
          configurable: true,
          value: 'ServiceWorker'
        });
        Object.defineProperty(ServiceWorkerRegistration.prototype, Symbol.toStringTag, {
          configurable: true,
          value: 'ServiceWorkerRegistration'
        });
        Object.defineProperty(ServiceWorkerContainer.prototype, Symbol.toStringTag, {
          configurable: true,
          value: 'ServiceWorkerContainer'
        });
      }
      globalThis.ServiceWorker = globalThis.ServiceWorker || ServiceWorker;
      Object.defineProperty(globalThis.ServiceWorker.prototype, 'state', {
        configurable: true,
        enumerable: true,
        get: function () {
          return this._controllerEventState || this._state;
        },
        set: function (value) {
          this._state = value;
        }
      });
      globalThis.ServiceWorkerRegistration =
        globalThis.ServiceWorkerRegistration || ServiceWorkerRegistration;
      globalThis.ServiceWorkerContainer =
        globalThis.ServiceWorkerContainer || ServiceWorkerContainer;
      var _container = Object.create(globalThis.ServiceWorkerContainer.prototype);
      _container._et_listeners = {};
      _container.oncontrollerchange = null;
      _container.onmessage = null;
      var _nextSWPortId = 2;
      var _swPorts = Object.create(null);
      var SW_PORT_MARKER = '__zwServiceWorkerTransferredPortIndex';
      var SW_MESSAGE_ERROR_TRANSFER = '__zwServiceWorkerMessageErrorTransfer';
      var SW_MESSAGE_ERROR_MARKER = '__zwServiceWorkerMessageError';
      function cloneSWMessageWithPortMarkers(value, ports, seen) {
        if (value === null || typeof value !== 'object') return structuredClone(value);
        if (ports.indexOf(value) >= 0) {
          var marker = {};
          marker[SW_PORT_MARKER] = ports.indexOf(value);
          return marker;
        }
        if (seen.has(value)) return seen.get(value);
        if (value instanceof Date || value instanceof RegExp ||
            value instanceof ArrayBuffer ||
            (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView(value))) {
          return structuredClone(value);
        }
        var out = Array.isArray(value) ? [] : Object.create(Object.getPrototypeOf(value));
        seen.set(value, out);
        var keys = Object.keys(value);
        for (var i = 0; i < keys.length; i++) {
          out[keys[i]] = cloneSWMessageWithPortMarkers(value[keys[i]], ports, seen);
        }
        return out;
      }
      function isSWMessageErrorTransfer(value) {
        return value !== null && typeof value === 'object' && value[SW_MESSAGE_ERROR_TRANSFER] === true;
      }
      function transferSWPorts(worker, message, transfer) {
        var ids = [];
        var dataPortIndex = null;
        var ports = [];
        var messageError = false;
        if (transfer !== undefined && transfer !== null) {
          ports = Array.from(transfer);
          for (var i = 0; i < ports.length; i++) {
            var port = ports[i];
            if (isSWMessageErrorTransfer(port)) {
              messageError = true;
              continue;
            }
            if (!(port instanceof globalThis.MessagePort) ||
                port._closed || port._zwSwDetached || !port._other) {
              throw new DOMException('Invalid MessagePort transfer', 'DataCloneError');
            }
          }
          if (messageError) {
            return { ids: [], dataPortIndex: null, message: null, messageError: true };
          }
          for (var i = 0; i < ports.length; i++) {
            var port = ports[i];
            if (isSWMessageErrorTransfer(port)) continue;
            if (ids.indexOf(port._zwSwPortId) >= 0) {
              throw new DOMException('Duplicate MessagePort transfer', 'DataCloneError');
            }
            var remote = port._other;
            var portId = _nextSWPortId;
            _nextSWPortId += 2;
            remote._other = null;
            remote._zwSwPortId = portId;
            remote._zwSwWorker = worker;
            _swPorts[String(portId)] = remote;
            port._other = null;
            port._zwSwDetached = true;
            if (message === port) dataPortIndex = ids.length;
            ids.push(portId);
          }
        }
        return {
          ids: ids,
          dataPortIndex: dataPortIndex,
          message: ids.length && dataPortIndex === null ?
            cloneSWMessageWithPortMarkers(message, ports, typeof WeakMap !== 'undefined' ? new WeakMap() : new Map()) :
            message,
          messageError: false
        };
      }
      function postSWMessage(worker, message, transfer, targetPortId) {
        var ports = transferSWPorts(worker, message, transfer);
        var dataJSON;
        try {
          if (ports.messageError) {
            var messageErrorWire = {};
            messageErrorWire[SW_MESSAGE_ERROR_MARKER] = true;
            dataJSON = JSON.stringify(messageErrorWire);
          } else {
            dataJSON =
              ports.dataPortIndex === null ?
                JSON.stringify(structuredClone(ports.message)) :
                'null';
          }
        } catch (_e) {
          throw new DOMException('Service Worker message could not be cloned', 'DataCloneError');
        }
        if (dataJSON === undefined) {
          throw new DOMException('Service Worker message could not be cloned', 'DataCloneError');
        }
        if (typeof __zw_sw_post_message !== 'function') {
          throw new DOMException('Service Worker host bridge unavailable', 'InvalidStateError');
        }
        var wire = JSON.parse(__zw_sw_post_message(
          String(worker._id),
          dataJSON,
          JSON.stringify(ports.ids),
          ports.dataPortIndex === null ? '' : String(ports.dataPortIndex),
          targetPortId === null ? '' : String(targetPortId),
          worker._messageClientId || '',
          worker._messageClientUrl || ''));
        if (!wire || !wire.ok) {
          throw new DOMException(
            wire && wire.error || 'Service Worker postMessage failed',
            'InvalidStateError'
          );
        }
        worker._messagePollTarget++;
        scheduleRegistrationMessagePoll(worker);
      }
      globalThis.ServiceWorker.prototype.postMessage = function (message, transfer) {
        postSWMessage(this, message, transfer, null);
      };
      globalThis.ServiceWorker.prototype._postPortMessage = function(port, message, transfer) {
        postSWMessage(this, message, transfer, port._zwSwPortId);
      };
      function initSWMessageBridge(worker, client) {
        if (!worker) return worker;
        if (worker._messageSequence === undefined) worker._messageSequence = 0;
        if (worker._messagePollTarget === undefined) worker._messagePollTarget = 0;
        if (worker._messagePollPending === undefined) worker._messagePollPending = false;
        if (worker._messagePollDeadline === undefined) worker._messagePollDeadline = 0;
        if (client) {
          worker._messageClientId = client.id || '';
          worker._messageClientUrl = client.url || '';
          worker._messageContainer = client.container || null;
        }
        return worker;
      }
      globalThis.__zwInitServiceWorkerMessageBridge = initSWMessageBridge;
      function makeSW(scriptURL, state) {
        var worker = new globalThis.ServiceWorker(scriptURL, state);
        return initSWMessageBridge(worker);
      }
      function pollClientMessages(worker) {
        if (typeof __zw_sw_client_messages !== 'function') {
          worker._messagePollPending = false;
          return;
        }
        var wire;
        try {
          wire = JSON.parse(__zw_sw_client_messages(
            String(worker._id),
            String(worker._messageSequence),
            worker._messageClientId || ''));
        } catch (_e) {
          wire = null;
        }
        if (wire && wire.ok) {
          worker._messageSequence = Number(wire.latestSequence) || worker._messageSequence;
          var messages = wire.messages || [];
          for (var i = 0; i < messages.length; i++) {
            var messageWire = messages[i];
            var transferred = [];
            var portIds = messageWire.transferredPortIds || [];
            for (var p = 0; p < portIds.length; p++) {
              var port = new globalThis.MessagePort();
              port._zwSwPortId = Number(portIds[p]);
              port._zwSwWorker = worker;
              _swPorts[String(port._zwSwPortId)] = port;
              transferred.push(port);
            }
            var data = messageWire.dataPortIndex === null ?
              messageWire.data : transferred[Number(messageWire.dataPortIndex)];
            var event = new globalThis.MessageEvent('message', {
              data: data,
              origin: '',
              source: worker,
              ports: transferred
            });
            if (messageWire.portId === null) {
              (worker._messageContainer || _container).dispatchEvent(event);
            } else {
              var target = _swPorts[String(messageWire.portId)] || null;
              if (target) {
                var listeners = target._et_listeners && target._et_listeners.message;
                if (!target._onmessage && (!listeners || listeners.length === 0)) {
                  target._zwSwQueue.push(event);
                } else {
                  target.dispatchEvent(event);
                }
              }
            }
          }
            if (messages.length > 0) {
              // https://w3c.github.io/ServiceWorker/#clients-claim
              // A worker message may be emitted after clients.claim() in the
              // same task; refresh controlled clients before page promises
              // resume from that message.
              refreshControllerFromHost(worker._messageContainer || _container);
            }
          if (worker._messageSequence >= worker._messagePollTarget) {
            worker._messagePollPending = false;
            worker._messagePollDeadline = 0;
            refreshControllerFromHost(worker._messageContainer || _container);
            scheduleControllerRefreshFromHost(worker._messageContainer || _container);
            return;
          }
        }
        if (worker._messagePollDeadline && Date.now() >= worker._messagePollDeadline) {
          worker._messagePollTarget = worker._messageSequence;
          worker._messagePollPending = false;
          worker._messagePollDeadline = 0;
          refreshControllerFromHost(worker._messageContainer || _container);
          scheduleControllerRefreshFromHost(worker._messageContainer || _container);
          return;
        }
        if (typeof setTimeout === 'function') {
          setTimeout(function () { pollClientMessages(worker); }, 0);
        } else {
          worker._messagePollPending = false;
          worker._messagePollDeadline = 0;
        }
      }
      function scheduleClientMessagePoll(worker) {
        if (worker._messagePollPending) return;
        worker._messagePollPending = true;
        if (typeof setTimeout === 'function') {
          setTimeout(function () { pollClientMessages(worker); }, 0);
        } else {
          pollClientMessages(worker);
        }
      }
      function scheduleRegistrationMessagePoll(worker) {
        var seen = {};
        function schedule(candidate, target, force) {
          if (!candidate || candidate._id == null) return;
          var id = String(candidate._id);
          if (!force) {
            if (seen[id]) return;
            seen[id] = true;
          }
          candidate._messagePollTarget = Math.max(
            candidate._messagePollTarget || 0,
            target
          );
          candidate._messagePollDeadline = Date.now() + 1000;
          scheduleClientMessagePoll(candidate);
        }
        var nextTarget = (worker._messageSequence || 0) + 1;
        var workerScheduled = false;
        for (var i = 0; i < _registrations.length; i++) {
          var reg = _registrations[i];
          if (!reg || !worker) continue;
          var workerId = String(worker._id);
          var matchesRegistration =
            String(reg._id) === workerId ||
            (reg._worker && String(reg._worker._id) === workerId) ||
            (reg.installing && String(reg.installing._id) === workerId) ||
            (reg.waiting && String(reg.waiting._id) === workerId) ||
            (reg.active && String(reg.active._id) === workerId);
          if (!matchesRegistration) continue;
          if (reg.active && reg.waiting && String(reg.active._id) === workerId) {
            schedule(reg.waiting, nextTarget);
            schedule(worker, worker._messagePollTarget || nextTarget, true);
            workerScheduled = true;
          }
          if (!workerScheduled) {
            schedule(worker, worker._messagePollTarget || nextTarget, true);
            workerScheduled = true;
          }
          schedule(reg._worker, nextTarget);
          schedule(reg.installing, nextTarget);
          schedule(reg.waiting, nextTarget);
          schedule(reg.active, nextTarget);
        }
        if (!workerScheduled) schedule(worker, worker._messagePollTarget || nextTarget, true);
      }
      globalThis.__zwServiceWorkerFetchSettled = function () {
        ensureDocument();
        for (var i = 0; i < _registrations.length; i++) {
          (function (reg) {
            var seen = {};
            var workers = [reg._worker, reg.installing, reg.waiting, reg.active];
            for (var j = 0; j < workers.length; j++) {
              var worker = workers[j];
              if (!worker || worker._id == null) continue;
              var id = String(worker._id);
              if (seen[id]) continue;
              seen[id] = true;
              worker._messagePollTarget = Math.max(
                worker._messagePollTarget,
                worker._messageSequence + 1
              );
              worker._messagePollDeadline = Date.now() + 1000;
              scheduleClientMessagePoll(worker);
            }
          })(_registrations[i]);
        }
      };
      globalThis.__zwPollServiceWorkerMessages = function () {
        ensureDocument();
        for (var i = 0; i < _registrations.length; i++) {
          (function (reg) {
            var seen = {};
            var workers = [reg._worker, reg.installing, reg.waiting, reg.active];
            for (var j = 0; j < workers.length; j++) {
              var worker = workers[j];
              if (!worker || worker._id == null) continue;
              var id = String(worker._id);
              if (seen[id]) continue;
              seen[id] = true;
              worker._messagePollTarget = Math.max(
                worker._messagePollTarget,
                worker._messageSequence + 1
              );
              worker._messagePollDeadline = Date.now() + 1000;
              scheduleClientMessagePoll(worker);
            }
          })(_registrations[i]);
        }
      };
      globalThis.__zwPollServiceWorkerRegistrations = function () {
        if (!_registrations.length && !_controller) return;
        ensureDocument();
        for (var i = 0; i < _registrations.length; i++) {
          pollRegistration(_registrations[i]);
        }
        // https://w3c.github.io/ServiceWorker/#navigator-service-worker-controller
        // Keep the page-side controller projection aligned with host-owned
        // activation when execute_script is the only event-loop driver.
        refreshControllerFromHost(_container);
      };
      function dispatchTargetEvent(target, type) {
        if (target && typeof target.dispatchEvent === 'function' &&
            typeof globalThis.Event === 'function') {
          try { target.dispatchEvent(new globalThis.Event(type)); } catch (_e) {}
        }
      }
      function makeReg(id, scriptURL, scope, updateViaCache) {
        var reg = new globalThis.ServiceWorkerRegistration();
        reg._id = id;
        reg._worker = makeSW(scriptURL, 'installing');
        reg._worker._id = id;
        reg._stateSequence = 0;
        reg._pendingStates = [];
        reg._updateFoundPending = false;
        reg._activationRequested = false;
        reg._claimClientsPending = false;
        reg.scope = scope;
        reg.updateViaCache = updateViaCache || 'imports';
        reg.installing = reg._worker;
        reg.waiting = null;
        reg.active = null;
        reg._previousActive = null;
        // https://w3c.github.io/ServiceWorker/#navigator-service-worker-unregister
        // The JS registration object remains readable after unregister(), but
        // it must not keep scheduling host state polls for a removed entry.
        reg._unregistered = false;
        reg.onupdatefound = null;
        reg.unregister = function () {
            var removed = false;
            if (typeof __zw_sw_unregister === 'function') {
              try { removed = __zw_sw_unregister(String(reg._id)) === 'true'; } catch (_e) {}
            }
            for (var i = 0; i < _registrations.length; i++) {
              if (_registrations[i] === reg) { _registrations.splice(i, 1); break; }
            }
            if (removed && typeof setTimeout === 'function') {
              reg._unregistered = true;
              setTimeout(function () { applyState(reg, 'redundant'); }, 0);
            }
            return Promise.resolve(removed);
          };
        reg.update = function () {
          ensureDocument();
          if (typeof __zw_sw_update !== 'function') {
            return Promise.reject(new TypeError('Service Worker update bridge unavailable'));
          }
          var wire;
          try {
            wire = JSON.parse(__zw_sw_update(String(reg._id)));
          } catch (error) {
            return Promise.reject(error);
          }
          if (!wire || !wire.ok) {
            var message = wire && wire.error || 'Service Worker update failed';
            if (wire && wire.errorName === 'SecurityError') {
              // M7：`globalThis.DOMException`（R9 wrong-global 先例）——native 路径下裸
              // DOMException 解析到 shim 闭包构造器，instanceof 检查解析到 globalThis（native）→ 恒 false。
              return Promise.reject(new (globalThis.DOMException || Error)(message, 'SecurityError'));
            }
            return Promise.reject(new TypeError(message));
          }
          if (!wire.changed) {
            if (String(wire.id) !== String(reg._id)) {
              upsertSnapshot(readSnapshot(wire.id), 'manual');
            }
            return Promise.resolve(reg);
          }
          var snapshot = readSnapshot(wire.id);
          var updated = upsertSnapshot({
            id: wire.id,
            scriptURL: snapshot && snapshot.scriptURL || reg._worker.scriptURL,
            scope: snapshot && snapshot.scope || reg.scope,
            state: 'installing'
          }, 'manual');
          return Promise.resolve(updated).then(function (registration) {
            scheduleRegistrationPoll(registration);
            return registration;
          });
        };
        reg.getNotifications = function () { return Promise.resolve([]); };
        reg.showNotification = function () { return Promise.resolve(); };
        return reg;
      }
      function findReg(id, scope) {
        for (var i = 0; i < _registrations.length; i++) {
          if (String(_registrations[i]._id) === String(id)) return _registrations[i];
        }
        if (scope) {
          for (var j = 0; j < _registrations.length; j++) {
            if (_registrations[j].scope === scope) return _registrations[j];
          }
        }
        return null;
      }
      function readSnapshot(id) {
        if (typeof __zw_sw_snapshot !== 'function') return null;
        try {
          var wire = JSON.parse(__zw_sw_snapshot(String(id)));
          return wire && wire.ok ? wire : null;
        } catch (_e) { return null; }
      }
      function readStateChanges(reg) {
        if (typeof __zw_sw_state_changes !== 'function') return null;
        try {
          var wire = JSON.parse(__zw_sw_state_changes(
            String(reg._id), String(reg._stateSequence)));
          return wire && wire.ok ? wire : null;
        } catch (_e) { return null; }
      }
      function readControllerSnapshot(documentURL, clientId) {
        if (typeof __zw_sw_controller !== 'function') return null;
        try {
          var wire = JSON.parse(__zw_sw_controller(
            documentURL == null ? '' : String(documentURL),
            clientId == null ? '' : String(clientId)
          ));
          return wire && wire.ok ? wire.controller : null;
        } catch (_e) { return null; }
      }
      function hasControlledClientForWorker(worker) {
        if (!worker || worker._id == null) return false;
        if (typeof __zw_sw_has_controlled_client === 'function') {
          try {
            var wire = JSON.parse(__zw_sw_has_controlled_client(String(worker._id)));
            if (wire && wire.ok && wire.controlled === true) return true;
          } catch (_eControlled) {}
        }
        var workerId = String(worker._id);
        if (_controller && String(_controller._id) === workerId) return true;
        if (typeof _iframeDocCache !== 'object') return false;
        var frameKeys = Object.keys(_iframeDocCache);
        for (var i = 0; i < frameKeys.length; i++) {
          var entry = _iframeDocCache[frameKeys[i]];
          var doc = entry && entry.doc;
          if (!doc || !doc._zwURL || !doc._zwSwClientId) continue;
          var controller = readControllerSnapshot(doc._zwURL, doc._zwSwClientId);
          if (controller && String(controller.id) === workerId) return true;
        }
        return false;
      }
      function readRegistrationSnapshot(clientURL) {
        if (typeof __zw_sw_get_registration !== 'function') return null;
        try {
          var wire = JSON.parse(__zw_sw_get_registration(clientURL || ''));
          return wire && wire.ok ? wire.registration : null;
        } catch (_e) { return null; }
      }
      _container.__zwRefreshRegistration = function (clientURL) {
        ensureDocument();
        return upsertSnapshot(readRegistrationSnapshot(clientURL), 'manual') || undefined;
      };
      _container.__zwRefreshSnapshot = function (snapshot) {
        ensureDocument();
        return upsertSnapshot(snapshot, 'manual') || undefined;
      };
      _container.__zwCreateInstallingRegistration = function (snapshot) {
        ensureDocument();
        var reg = upsertSnapshot(snapshot, 'manual');
        if (reg) scheduleRegistrationPoll(reg);
        return reg || undefined;
      };
      function refreshRegistrationAfterRedundant(reg) {
        // https://w3c.github.io/ServiceWorker/#navigator-service-worker-getRegistration
        var clientURL = _documentURL ||
          (globalThis.location && globalThis.location.href) ||
          reg.scope;
        var replacement = readRegistrationSnapshot(clientURL);
        if (replacement && String(replacement.id) !== String(reg._id)) {
          upsertSnapshot(replacement, 'manual');
        }
      }
      function refreshControllerFromHost(container, workerHint, previousHint, eventStateHint) {
        if (container && container !== _container &&
            typeof container.__zwRefreshServiceWorkerController === 'function') {
          container.__zwRefreshServiceWorkerController(workerHint, previousHint, eventStateHint);
          return;
        }
        if (workerHint) {
          setController(workerHint, previousHint, eventStateHint);
          return;
        }
        var snapshot = readControllerSnapshot();
        if (snapshot && snapshot.state === 'activated') {
          var reg = upsertSnapshot(snapshot, 'manual');
          if (reg && reg.active &&
              (!_controller || String(_controller._id) !== String(reg.active._id))) {
            setController(reg.active, _controller, _controller ? 'activating' : null);
          }
        }
        if (typeof _iframeDocCache === 'object') {
          var frameKeys = Object.keys(_iframeDocCache);
          for (var i = 0; i < frameKeys.length; i++) {
            var entry = _iframeDocCache[frameKeys[i]];
            var frameServiceWorker = entry && entry.win &&
              entry.win.navigator && entry.win.navigator.serviceWorker;
            if (frameServiceWorker &&
                typeof frameServiceWorker.__zwRefreshServiceWorkerController === 'function') {
              frameServiceWorker.__zwRefreshServiceWorkerController();
            }
          }
        }
      }
      function scheduleControllerRefreshFromHost(container) {
        if (typeof setTimeout === 'function') {
          setTimeout(function () { refreshControllerFromHost(container); }, 0);
        }
      }
      function stateSequence(state) {
        if (state === 'installed') return 1;
        if (state === 'activating') return 2;
        if (state === 'activated') return 3;
        if (state === 'redundant') return 4;
        return 0;
      }
      function stateForSequence(sequence) {
        if (sequence === 1) return 'installed';
        if (sequence === 2) return 'activating';
        if (sequence === 3) return 'activated';
        if (sequence === 4) return 'redundant';
        return 'installing';
      }
      function setController(worker, previousHint, eventStateHint) {
        if (_controller === worker && eventStateHint == null) return;
        var previous = _controller;
        if (previousHint &&
            (!previous ||
             (previous._id != null &&
              previousHint._id != null &&
              String(previous._id) === String(previousHint._id)))) {
          previous = previousHint;
        }
        _controller = worker;
        var eventState = eventStateHint || (worker ? worker.state : null);
        if (worker && previous &&
            String(previous._id) !== String(worker._id) &&
            eventState === 'activated') {
          eventState = 'activating';
        }
        _controllerChangeGeneration++;
        var generation = _controllerChangeGeneration;
        var expectedId = worker ? String(worker._id) : '';
        var restoreState = worker ? worker._state : null;
        function scheduleControllerEventStateClear() {
          setTimeout(function () {
            setTimeout(function () {
              if (worker && worker._controllerEventState === eventState) {
                worker._controllerEventState = null;
              }
              if (worker && worker._state === eventState && restoreState !== eventState) {
                worker._state = restoreState;
              }
              if (_controllerEventWorker === worker) _controllerEventWorker = null;
            }, 0);
          }, 0);
        }
        if (typeof setTimeout === 'function') {
          setTimeout(function () {
            if (generation !== _controllerChangeGeneration) return;
            if ((_controller ? String(_controller._id) : '') !== expectedId) return;
            if (worker) {
              restoreState = worker._state;
              worker._controllerEventState = eventState;
              if (eventState != null) worker._state = eventState;
            }
            _controllerEventWorker = worker;
            dispatchTargetEvent(_container, 'controllerchange');
            notifyIframeControllerChange(worker, previous, eventState);
            scheduleControllerEventStateClear();
          }, 0);
        } else {
          if (worker) {
            worker._controllerEventState = eventState;
            if (eventState != null) worker._state = eventState;
          }
          _controllerEventWorker = worker;
          dispatchTargetEvent(_container, 'controllerchange');
          notifyIframeControllerChange(worker, previous, eventState);
          scheduleControllerEventStateClear();
        }
      }
      function notifyIframeControllerChange(worker, previous, eventState) {
        if (typeof _iframeDocCache !== 'object') return;
        var frameKeys = Object.keys(_iframeDocCache);
        for (var i = 0; i < frameKeys.length; i++) {
          var entry = _iframeDocCache[frameKeys[i]];
          var frameServiceWorker = entry && entry.win &&
            entry.win.navigator && entry.win.navigator.serviceWorker;
          if (frameServiceWorker &&
              typeof frameServiceWorker.__zwRefreshServiceWorkerController === 'function') {
            frameServiceWorker.__zwRefreshServiceWorkerController(worker, previous, eventState);
          }
        }
      }
      function notifyIframeRegistrationChange(reg) {
        if (typeof _iframeDocCache !== 'object') return;
        var frameKeys = Object.keys(_iframeDocCache);
        for (var i = 0; i < frameKeys.length; i++) {
          var entry = _iframeDocCache[frameKeys[i]];
          var frameServiceWorker = entry && entry.win &&
            entry.win.navigator && entry.win.navigator.serviceWorker;
          if (frameServiceWorker &&
              typeof frameServiceWorker.__zwRefreshServiceWorkerRegistration === 'function') {
            frameServiceWorker.__zwRefreshServiceWorkerRegistration(reg);
          }
        }
      }
      function applyState(reg, state) {
        if (!reg || !reg._worker) return;
        var worker = reg._worker;
        var changed = worker.state !== state;
        worker.state = state;
        reg.installing = state === 'installing' ? worker : null;
        reg.waiting = state === 'installed' ? worker : null;
        if (state === 'installed' &&
            reg._previousActive &&
            !hasControlledClientForWorker(reg._previousActive) &&
            !reg._activationRequested &&
            typeof __zw_sw_activate_waiting === 'function') {
          reg._activationRequested = true;
          try { __zw_sw_activate_waiting(String(reg._id)); } catch (_eActivate) {}
          if (_controller === reg._previousActive) {
            worker.state = 'activating';
            reg.installing = null;
            reg.waiting = null;
            reg.active = worker;
            setController(worker);
          }
        }
        if (state === 'activating') {
          reg.active = worker;
          if (reg._previousActive && _controller === reg._previousActive) {
            setController(worker);
          } else if (reg._previousActive) {
            notifyIframeControllerChange(worker, reg._previousActive, worker.state);
          }
        } else if (state === 'activated') {
          var replaceController = false;
          var previousController = null;
          if (reg._previousActive && reg._previousActive !== worker) {
            var previous = reg._previousActive;
            previousController = previous;
            reg._previousActive = null;
            replaceController = _controller === previous;
            previous.state = 'redundant';
            dispatchTargetEvent(previous, 'statechange');
          }
          reg.active = worker;
          var latestChanges = reg._claimClientsPending ? null : readStateChanges(reg);
          var claimRequested = reg._claimClientsPending ||
            !!(latestChanges && latestChanges.claimClients);
          var claimController = claimRequested ?
            readControllerSnapshot() : null;
          if (replaceController ||
              (claimController &&
               String(claimController.id) === String(reg._id))) {
            setController(worker, previousController, replaceController ? 'activating' : null);
          }
          reg._claimClientsPending = false;
        } else if (state === 'redundant' && reg.active === worker) {
          reg.active = reg._previousActive;
          reg._previousActive = null;
          refreshRegistrationAfterRedundant(reg);
        }
        if (state === 'activated' && _readyResolve) {
          _readyResolve(reg);
          _readyResolve = null;
        }
        if (changed) {
          dispatchTargetEvent(worker, 'statechange');
          notifyIframeRegistrationChange(reg);
        }
      }
      function applySnapshot(reg, snapshot) {
        if (!snapshot) return false;
        reg.scope = snapshot.scope || reg.scope;
        if (String(reg._id) === String(snapshot.id)) {
          reg._worker.scriptURL = snapshot.scriptURL || reg._worker.scriptURL;
        }
        var state = snapshot.state;
        var targetSequence = stateSequence(state);
        if (targetSequence > reg._stateSequence) {
          for (var sequence = reg._stateSequence + 1; sequence <= targetSequence; sequence++) {
            applyState(reg, stateForSequence(sequence));
          }
          reg._stateSequence = targetSequence;
        } else {
          applyState(reg, state);
        }
        if (state === 'redundant') {
          refreshRegistrationAfterRedundant(reg);
        }
        return state === 'activated' || state === 'redundant';
      }
      function pollRegistration(reg) {
        if (reg._unregistered) return;
        if (reg._updateFoundPending) {
          reg._updateFoundPending = false;
          dispatchTargetEvent(reg, 'updatefound');
        } else if (reg._pendingStates.length > 0) {
          var state = reg._pendingStates.shift();
          reg._stateSequence++;
          applyState(reg, state);
          if (state === 'activated' || state === 'redundant') return;
        } else {
          var changes = readStateChanges(reg);
          if (changes && changes.states && changes.states.length) {
            reg._pendingStates = changes.states.slice();
            reg._claimClientsPending = changes.claimClients === true;
          } else if (applySnapshot(reg, readSnapshot(reg._id))) {
            return;
          }
        }
        if (typeof setTimeout === 'function') {
          setTimeout(function () { pollRegistration(reg); }, 0);
        }
      }
      function scheduleRegistrationPoll(reg) {
        if (typeof setTimeout !== 'function') {
          pollRegistration(reg);
          return;
        }
        reg._updateFoundPending = true;
        setTimeout(function () {
          setTimeout(function () { pollRegistration(reg); }, 0);
        }, 0);
      }
      function upsertSnapshot(snapshot, deferPoll) {
        if (!snapshot) return null;
        var reg = findReg(snapshot.id, null);
        if (!reg && snapshot.scope) reg = findReg(null, snapshot.scope);
        if (!reg) {
          reg = makeReg(
            snapshot.id,
            snapshot.scriptURL || '',
            snapshot.scope || '',
            snapshot.updateViaCache || 'imports');
          _registrations.push(reg);
        } else {
          reg.updateViaCache = snapshot.updateViaCache || reg.updateViaCache;
          if (String(reg._id) !== String(snapshot.id)) {
            if (reg.active && String(reg.active._id) === String(snapshot.id)) {
              reg.active.scriptURL = snapshot.scriptURL || reg.active.scriptURL;
              reg.active.state = snapshot.state || reg.active.state;
              notifyIframeRegistrationChange(reg);
              return reg;
            }
            reg._previousActive = reg.active;
            reg._worker = makeSW(snapshot.scriptURL || '', 'installing');
            reg._worker._id = snapshot.id;
            reg._stateSequence = 0;
            reg._pendingStates = [];
            reg._updateFoundPending = false;
            reg._activationRequested = false;
            reg._claimClientsPending = false;
            reg.installing = reg._worker;
            reg.waiting = null;
          }
          reg._id = snapshot.id;
        }
        notifyIframeRegistrationChange(reg);
        if (!applySnapshot(reg, snapshot)) {
          if (deferPoll === 'manual') {
            return reg;
          } else if (deferPoll) {
            scheduleRegistrationPoll(reg);
          } else {
            pollRegistration(reg);
          }
        }
        return reg;
      }
      function commitDocument(resetListeners) {
        _documentURL = globalThis.location.href;
        _registrations = [];
        _controller = null;
        _ready = new Promise(function (resolve) { _readyResolve = resolve; });
        if (resetListeners) {
          _container._et_listeners = {};
          _container.oncontrollerchange = null;
          _container.onmessage = null;
        }
        var snapshot = readControllerSnapshot();
        if (!snapshot || snapshot.state !== 'activated') return;
        var reg = upsertSnapshot(snapshot, false);
        if (reg && reg.active) _controller = reg.active;
      }
      function ensureDocument() {
        if (_documentURL !== globalThis.location.href) commitDocument(false);
      }
      globalThis.__zwServiceWorkerDocumentCommit = function () {
        commitDocument(true);
      };
      _container.register = function (scriptURL, options) {
        ensureDocument();
        if (!scriptURL || typeof scriptURL !== 'string') {
          return Promise.reject(new TypeError('ServiceWorkerContainer.register: scriptURL is required'));
        }
        if (typeof __zw_sw_register !== 'function') {
          return Promise.reject(new Error('Service Worker host bridge unavailable'));
        }
        var scopeProvided = options != null && options.scope !== undefined;
        var scope = scopeProvided ? String(options.scope) : '';
        var updateViaCache =
          options != null && options.updateViaCache !== undefined
            ? String(options.updateViaCache)
            : 'imports';
        if (updateViaCache !== 'imports' && updateViaCache !== 'all' && updateViaCache !== 'none') {
          return Promise.reject(new TypeError('Invalid updateViaCache value'));
        }
        var scriptType =
          options != null && options.type !== undefined ? String(options.type) : 'classic';
        if (scriptType !== 'classic' && scriptType !== 'module') {
          return Promise.reject(new TypeError('Invalid Service Worker script type'));
        }
        var wire;
        try {
          wire = JSON.parse(__zw_sw_register(
            scriptURL,
            scope,
            globalThis.location.href,
            scopeProvided ? 'true' : 'false',
            updateViaCache,
            scriptType));
        } catch (error) {
          return Promise.reject(error);
        }
        if (!wire || !wire.ok) {
          var message = wire && wire.error || 'Service Worker registration failed';
          if (wire && wire.errorName === 'SecurityError') {
            // M7：globalThis.DOMException（R9 先例，同上）。
            return Promise.reject(new (globalThis.DOMException || Error)(message, 'SecurityError'));
          }
          return Promise.reject(new TypeError(message));
        }
        var snapshot = readSnapshot(wire.id);
        var reg = upsertSnapshot({
          id: wire.id,
          scriptURL: snapshot && snapshot.scriptURL || scriptURL,
          scope: snapshot && snapshot.scope || scope,
          updateViaCache: snapshot && snapshot.updateViaCache || updateViaCache,
          state: wire.existing === false ? 'installing' : (snapshot && snapshot.state || 'installing')
        }, 'manual');
        scheduleClientMessagePoll(reg._worker);
        var registrationPromise = Promise.resolve(reg);
        registrationPromise.then(function (registration) {
          if (wire.existing === false) {
            scheduleRegistrationPoll(registration);
          }
        });
        return registrationPromise;
      };
      _container.getRegistration = function (scope) {
        ensureDocument();
        var absolute = scope;
        try {
          var parsed = new URL(scope || globalThis.location.href, globalThis.location.href);
          if (parsed.origin !== globalThis.location.origin) {
            // https://w3c.github.io/ServiceWorker/#navigator-service-worker-getRegistration
            return Promise.reject(new (globalThis.DOMException || Error)(
              'Service Worker document URL origin mismatch',
              'SecurityError'));
          }
          absolute = parsed.href;
        } catch (_e) {}
        if (typeof __zw_sw_get_registration === 'function') {
          try {
            var wire = JSON.parse(__zw_sw_get_registration(absolute));
            if (!wire || !wire.ok) {
              return Promise.reject(new TypeError(wire && wire.error || 'Service Worker discovery failed'));
            }
            return Promise.resolve(upsertSnapshot(wire.registration) || undefined);
          } catch (error) {
            return Promise.reject(error);
          }
        }
        for (var i = 0; i < _registrations.length; i++) {
          if (!scope || _registrations[i].scope === absolute) {
            return Promise.resolve(_registrations[i]);
          }
        }
        return Promise.resolve(undefined);
      };
      _container.getRegistrations = function () {
        ensureDocument();
        if (typeof __zw_sw_get_registrations === 'function') {
          try {
            var wire = JSON.parse(__zw_sw_get_registrations());
            if (!wire || !wire.ok) {
              return Promise.reject(new TypeError(wire && wire.error || 'Service Worker discovery failed'));
            }
            var snapshots = wire.registrations || [];
            var discovered = [];
            var ids = {};
            for (var i = 0; i < snapshots.length; i++) {
              var reg = upsertSnapshot(snapshots[i]);
              if (reg) {
                discovered.push(reg);
                ids[String(reg._id)] = true;
              }
            }
            for (var j = _registrations.length - 1; j >= 0; j--) {
              if (!ids[String(_registrations[j]._id)]) _registrations.splice(j, 1);
            }
            return Promise.resolve(discovered);
          } catch (error) {
            return Promise.reject(error);
          }
        }
        return Promise.resolve(_registrations.slice());
      };
      Object.defineProperty(_container, 'ready', {
        get: function () {
          ensureDocument();
          return _ready;
        }
      });
      Object.defineProperty(_container, 'controller', {
        get: function () {
          ensureDocument();
          if (_controllerEventWorker) return _controllerEventWorker;
          var snapshot = readControllerSnapshot();
          if (snapshot && snapshot.state === 'activated') {
            var reg = upsertSnapshot(snapshot, 'manual');
            if (reg && reg.active) _controller = reg.active;
          }
          return _controller;
        }
      });
      return _container;
    })()
  };

  // R3256：console 桥接宿主——page console.log/warn/error 等经 `__zw_console_log(level,msg)` 回调转发到宿主日志
  //（tracing）。旧实现全 no-op（page console 输出完全丢失，排障/WPT console 断言不可见）。序列化：string 直传，
  // 其余 JSON.stringify（对象/数组可读），JSON 失败（function/circular）回退 String()。`typeof` 守卫：回调未注册
  //（shim 未配 host）时 no-op，**向后兼容**（不抛 ReferenceError）。count/group/time 等非输出类保持 no-op。
  // **无条件覆盖**（非 `||`）：V8 默认上下文自带 native console（`function log(){[native code]}`，不桥接宿主，
  // 输出丢失），`||` 会保留它 → 桥接失效。故强制覆盖为桥接版；typeof 守卫保证无回调时与 native 同效（皆 vanishing）。
  function _zwSerializeConsoleArg(a) {
    if (typeof a === 'string') return a;
    if (a === null) return 'null';
    if (a === undefined) return 'undefined';
    try { return JSON.stringify(a); } catch (_) {}
    try { return String(a); } catch (_) { return '[unknown]'; }
  }
  // S11（cdp-protocol value-only console 面）：逐参值序列化——string/number/boolean 原样，
  // null→null，undefined→标记串（JSON 无法承载，headless 侧映射 type:"undefined"），
  // 对象/数组走 JSON round-trip（保结构），失败回退 String()。与 _zwSerializeConsoleArg
  // 的单行拼接并存：第 3 参 = 值数组 JSON（headless → `Runtime.consoleAPICalled` 的
  // value-only remoteObject 列表），第 2 参仍为单行文本（tracing 面不变）。
  function _zwSerializeConsoleValue(a) {
    var t = typeof a;
    if (t === 'string') return a;
    if (t === 'number' || t === 'boolean') return a;
    if (a === null) return null;
    if (a === undefined) return '__zw_undefined__';
    try { return JSON.parse(JSON.stringify(a)); } catch (_) {}
    try { return String(a); } catch (_) { return '[unknown]'; }
  }
  function _zwConsoleEmit(level, args) {
    if (typeof __zw_console_log !== 'function') return; // 宿主未注册 → no-op（向后兼容）
    var parts = [];
    var values = [];
    for (var i = 0; i < args.length; i++) {
      parts.push(_zwSerializeConsoleArg(args[i]));
      values.push(_zwSerializeConsoleValue(args[i]));
    }
    var valuesJson = '[]';
    try { valuesJson = JSON.stringify(values); } catch (_) {}
    try { __zw_console_log(level, parts.join(' '), valuesJson); } catch (_) {}
  }
  globalThis.console = {
    log: function() { _zwConsoleEmit('log', arguments); },
    info: function() { _zwConsoleEmit('info', arguments); },
    warn: function() { _zwConsoleEmit('warn', arguments); },
    error: function() { _zwConsoleEmit('error', arguments); },
    debug: function() { _zwConsoleEmit('debug', arguments); },
    trace: function() { _zwConsoleEmit('trace', arguments); },
    dir: function() { _zwConsoleEmit('dir', arguments); },
    dirxml: function() { _zwConsoleEmit('dirxml', arguments); },
    table: function() { _zwConsoleEmit('table', arguments); },
    clear: function() {},
    count: function() {},
    countReset: function() {},
    group: function() {},
    groupCollapsed: function() {},
    groupEnd: function() {},
    time: function() {},
    timeLog: function() {},
    timeEnd: function() {},
    assert: function() {}
  };

  // `new Image(width, height)`（HTMLImageElement 构造器，R2834）——图片预加载（`new Image().src = url` 预取 /
  // onload 探测）+ DOM 挂载（`document.body.appendChild(img)`）高频；WPT css-images / css-backgrounds /
  // content-visibility fixtures 经 `new Image()` 构造。旧实现返 plain object（非 DOM 元素，appendChild 失效，
  // 无 tagName）；现返 createElement('img') proxy（镜像 Option R2832 模式），设 width/height 属性；允许 new
  // 与无 new（返值覆盖 this）。shim 元素为 Proxy 非 ctor 实例，故 `instanceof Image` 不成立（documented）。
  function Image(width, height) {
    var el = globalThis.document.createElement('img');
    if (width !== undefined) { try { el.setAttribute('width', String(width)); } catch (_e) {} }
    if (height !== undefined) { try { el.setAttribute('height', String(height)); } catch (_e) {} }
    return el;
  }
  globalThis.Image = globalThis.Image || Image;

  // `new Audio([src])`（HTMLAudioElement 构造器，R2835）——音效/播客/通知音频构造高频（`new Audio(url).play()`）。
  // 返 createElement('audio') proxy（镜像 Image R2834），设 src；**无 new 调用抛 TypeError**
  //（spec WebIDL constructor 语义——WPT the-audio-element/audio_constructor 断言面）；设
  // preload='auto'（spec「The Audio() constructor must set the preload attribute to auto」）。
  // headless 无音频设备——play/pause/load 语义桩见 part03 媒体方法段。`instanceof
  // Audio`=false（shim 返 Proxy，同 Image/Option 谱，documented）。
  // https://html.spec.whatwg.org/multipage/media.html#dom-audio
  function Audio(src) {
    if (!new.target && (!this || this.constructor !== Audio)) {
      throw new globalThis.TypeError("Failed to construct 'Audio': Please use the 'new' operator, this DOM object constructor cannot be called as a function.");
    }
    var el = globalThis.document.createElement('audio');
    try { el.setAttribute('preload', 'auto'); } catch (_eAP) {}
    if (src !== undefined && src !== null) { try { el.setAttribute('src', String(src)); } catch (_e) {} }
    else if (src === null) { try { el.setAttribute('src', 'null'); } catch (_eAS) {} }
    return el;
  }
  globalThis.Audio = globalThis.Audio || Audio;

  // `new Option(text, value, defaultSelected, selected)`（HTMLOptionElement 构造器，R2832）——动态选项
  // 创建（`select.add(new Option('Apple','a'))` 动态下拉填充高频）。返 createElement('option') proxy，
  // 设 text/value/selected；允许 new 与无 new（返值覆盖 this）。shim 元素为 Proxy 非 ctor 实例，故
  // `instanceof Option` 不成立（documented；返回的 proxy 经 tagName='OPTION' + option.text 等可识别）。
  function Option(text, value, defaultSelected, selected) {
    var el = globalThis.document.createElement('option');
    if (text !== undefined) { try { el.textContent = String(text); } catch (_e) {} }
    if (value !== undefined) { try { el.setAttribute('value', String(value)); } catch (_e) {} }
    if (defaultSelected || selected) { try { el.setAttribute('selected', ''); } catch (_e) {} }
    return el;
  }
  globalThis.Option = globalThis.Option || Option;

  // `Storage` 接口对象（R4875）——HTML Web Storage 规范要求全局暴露 Storage 构造器；
  // 站点脚本以 `Storage` 标识符作为 DI token / `instanceof Storage` / Storage.prototype
  // 方法补丁引用（缺失时类定义期 ReferenceError 中止整段脚本，baidu aas.js 判例）。
  // localStorage/sessionStorage 为 Storage 实例；named property 即存储项（spec supported
  // property names），经 Proxy 拦截。简化（FIXME）：接口成员（prototype）查找先于 named
  // item，与 WebIDL legacy platform object 的 [[Get]] 顺序相反（仅当存储项与成员同名时可见）。
  // https://html.spec.whatwg.org/multipage/webstorage.html#the-storage-interface
  function Storage() {
    throw new globalThis.TypeError("Illegal constructor");
  }
  Storage.prototype.key = function key(index) {
    var keys = Object.keys(this.__zwItems || {});
    return index >= 0 && index < keys.length ? keys[index] : null;
  };
  Storage.prototype.getItem = function getItem(key) {
    var items = this.__zwItems || {};
    return Object.prototype.hasOwnProperty.call(items, key) ? items[key] : null;
  };
  Storage.prototype.setItem = function setItem(key, value) {
    this.__zwItems[String(key)] = String(value);
  };
  Storage.prototype.removeItem = function removeItem(key) {
    delete this.__zwItems[String(key)];
  };
  Storage.prototype.clear = function clear() {
    var items = this.__zwItems || {};
    for (var k in items) { delete items[k]; }
  };
  Object.defineProperty(Storage.prototype, 'length', {
    get: function() { return Object.keys(this.__zwItems || {}).length; },
    configurable: true,
    enumerable: true
  });
  globalThis.Storage = globalThis.Storage || Storage;

  function _createStorage() {
    var target = Object.create(globalThis.Storage.prototype);
    target.__zwItems = {};
    return new Proxy(target, {
      get: function(t, p) {
        if (p === '__zwItems') return t.__zwItems;
        if (p in globalThis.Storage.prototype) return t[p];
        if (typeof p === 'string' && Object.prototype.hasOwnProperty.call(t.__zwItems, p)) return t.__zwItems[p];
        return undefined;
      },
      set: function(t, p, v) {
        if (p in globalThis.Storage.prototype || typeof p !== 'string') { t[p] = v; return true; }
        t.__zwItems[p] = String(v);
        return true;
      },
      has: function(t, p) {
        return (p in globalThis.Storage.prototype) ||
          (typeof p === 'string' && Object.prototype.hasOwnProperty.call(t.__zwItems, p));
      },
      deleteProperty: function(t, p) {
        if (typeof p === 'string' && !(p in globalThis.Storage.prototype)) delete t.__zwItems[p];
        return true;
      },
      ownKeys: function(t) { return Object.keys(t.__zwItems); },
      getOwnPropertyDescriptor: function(t, p) {
        if (typeof p === 'string' && Object.prototype.hasOwnProperty.call(t.__zwItems, p)) {
          return { configurable: true, enumerable: true, writable: true, value: t.__zwItems[p] };
        }
        return undefined;
      }
    });
  }

  globalThis.localStorage = _createStorage();
  globalThis.sessionStorage = _createStorage();

  // R3081/M2：IndexedDB 页面表面。factory 与 object-store schema 经 `__zw_idb` 接 zero-storage；
  // CRUD/index/cursor records 暂留 JS，供后续按 key 类型逐步迁移。无宿主 callback 的低层 sandbox
  // 测试保留 in-memory fallback。factory.open（异步 onupgradeneeded→onsuccess 派发）、
  // db.createObjectStore/objectStoreNames/transaction/close、store.add/put/get/delete/clear/count/createIndex、
  // tx.objectStore/oncomplete/abort、index.get/openCursor。records 存内存 Map。
  // spec https://w3c.github.io/IndexedDB/ 。**已知限制**：records 尚无持久化。
  // name → {version, stores, connections}
  var _idb_databases = {};
  var _zwIDBTransactions = [];
  var _zwIDBConnectionQueues = {};
  var _zwIDBHostConnections = {};
  var _zwIDBNextConnectionId = 0;
  var _zwIDBHostCapabilities;

  function _zwIDBCapabilities() {
    if (_zwIDBHostCapabilities !== undefined) return _zwIDBHostCapabilities;
    _zwIDBHostCapabilities = _zwIDBHostCall({ op: 'connection_capabilities' }) || {};
    return _zwIDBHostCapabilities;
  }

  function _zwIDBUsesHostConnections() {
    return !!_zwIDBCapabilities().crossRenderer;
  }

  function _zwIDBUsesHostTransactionScheduling() {
    return !!_zwIDBCapabilities().transactionScheduling;
  }

  function _zwIDBRegisterHostConnection(database) {
    if (!_zwIDBUsesHostConnections() || database._hostConnectionRegistered) return;
    _zwIDBHostCall({
      op: 'register_connection',
      connection: database._hostConnectionId,
      database: database.name,
      version: database.version
    });
    database._hostConnectionRegistered = true;
    _zwIDBHostConnections[database._hostConnectionId] = database;
  }

  function _zwIDBWaitForHostConnections(req, database, newVersion, proceed) {
    var change = _zwIDBHostCall({
      op: 'request_connection_change',
      database: database,
      new_version: newVersion
    });
    if (!change || change.ready) {
      proceed(change ? Number(change.oldVersion || 0) : undefined);
      return;
    }
    var blocked = false;
    var poll = function () {
      var status = _zwIDBHostCall({
        op: 'poll_connection_change',
        request: change.request
      });
      if (status.ready) {
        proceed(Number(change.oldVersion || 0));
        return;
      }
      if (status.blocked && !blocked) {
        blocked = true;
        _zwIDBEmit(
          req,
          'blocked',
          _zwIDBVersionEvent(
            'blocked',
            req,
            Number(change.oldVersion || 0),
            newVersion
          )
        );
      }
      setTimeout(poll, 0);
    };
    setTimeout(poll, 0);
  }

  globalThis.__zw_idb_connection_event = function (connectionId, oldVersion, newVersion) {
    var connection = _zwIDBHostConnections[Number(connectionId)];
    if (!connection || connection._closed) return;
    _zwIDBEmit(
      connection,
      'versionchange',
      _zwIDBVersionEvent('versionchange', connection, oldVersion, newVersion)
    );
  };

  // https://w3c.github.io/IndexedDB/#connection-queues
  function _zwIDBRunConnectionQueue(name) {
    var queue = _zwIDBConnectionQueues[name];
    if (!queue || queue.running || !queue.requests.length) return;
    queue.running = true;
    // finish 幂等（本次运行内一次性）：回调先自行 done 再抛时，兜底 finish 不得重复出队。
    var finished = false;
    var finish = function () {
      if (finished) return;
      finished = true;
      queue.requests.shift();
      queue.running = false;
      queue.retry = null;
      if (queue.requests.length) {
        queueMicrotask(function () { _zwIDBRunConnectionQueue(name); });
      } else {
        delete _zwIDBConnectionQueues[name];
      }
    };
    // 回调同步抛（如 host 不可达时 capabilities 探测抛 DOMException）不得楔死连接队列：
    // 推进队列后重抛，保持异常对页面可见（否则 queue.running 恒 true，同名后续请求永不结算）。
    try {
      queue.requests[0](finish, queue);
    } catch (callbackError) {
      finish();
      throw callbackError;
    }
  }

  function _zwIDBEnqueueConnectionRequest(name, operation) {
    var queue = _zwIDBConnectionQueues[name];
    if (!queue) {
      queue = { requests: [], running: false, retry: null };
      _zwIDBConnectionQueues[name] = queue;
    }
    queue.requests.push(operation);
    queueMicrotask(function () { _zwIDBRunConnectionQueue(name); });
  }

  function _zwIDBDeactivateTransactions(except) {
    _zwIDBTransactions.forEach(function (transaction) {
      if (transaction !== except) transaction._active = false;
    });
  }
  function _zwIDBUntrackTransaction(transaction) {
    var globalIndex = _zwIDBTransactions.indexOf(transaction);
    if (globalIndex !== -1) _zwIDBTransactions.splice(globalIndex, 1);
    var databaseIndex = transaction._db._transactions.indexOf(transaction);
    if (databaseIndex !== -1) transaction._db._transactions.splice(databaseIndex, 1);
    var stateIndex = transaction._db._state.transactions.indexOf(transaction);
    if (stateIndex !== -1) transaction._db._state.transactions.splice(stateIndex, 1);
    _zwIDBStartEligibleTransactions(transaction._db._state);
  }
  var _zwPreviousBeforeTimerTask = globalThis.__zwBeforeTimerTask;
  globalThis.__zwBeforeTimerTask = function () {
    if (typeof _zwPreviousBeforeTimerTask === 'function') _zwPreviousBeforeTimerTask();
    _zwIDBDeactivateTransactions(null);
  };

  // https://webidl.spec.whatwg.org/#idl-DOMString
  var _zwIDBWireNamePrefix = '__zw_utf16_name__:';

  function _zwIDBNameToWire(value) {
    value = String(value);
    if (value.indexOf(_zwIDBWireNamePrefix) !== 0
        && !/[\uD800-\uDFFF]/.test(value)) return value;
    var encoded = '';
    for (var i = 0; i < value.length; i++) {
      encoded += ('0000' + value.charCodeAt(i).toString(16)).slice(-4);
    }
    return _zwIDBWireNamePrefix + encoded;
  }

  function _zwIDBNameFromWire(value) {
    if (typeof value !== 'string'
        || value.indexOf(_zwIDBWireNamePrefix) !== 0) return value;
    var encoded = value.slice(_zwIDBWireNamePrefix.length);
    if (encoded.length % 4 !== 0 || !/^[0-9a-f]*$/.test(encoded)) return value;
    var decoded = '';
    for (var i = 0; i < encoded.length; i += 4) {
      decoded += String.fromCharCode(parseInt(encoded.slice(i, i + 4), 16));
    }
    return decoded;
  }

  function _zwIDBRequestNamesToWire(request) {
    var wire = {};
    Object.keys(request).forEach(function (key) { wire[key] = request[key]; });
    ['name', 'database', 'store', 'index'].forEach(function (key) {
      if (typeof wire[key] === 'string') wire[key] = _zwIDBNameToWire(wire[key]);
    });
    if (Array.isArray(wire.stores)) {
      wire.stores = wire.stores.map(function (store) {
        if (typeof store === 'string') return _zwIDBNameToWire(store);
        var storeWire = {};
        Object.keys(store).forEach(function (key) { storeWire[key] = store[key]; });
        storeWire.name = _zwIDBNameToWire(store.name);
        if (Array.isArray(store.indexes)) {
          storeWire.indexes = store.indexes.map(function (index) {
            var indexWire = {};
            Object.keys(index).forEach(function (key) { indexWire[key] = index[key]; });
            indexWire.name = _zwIDBNameToWire(index.name);
            return indexWire;
          });
        }
        return storeWire;
      });
    }
    return wire;
  }

  function _zwIDBResponseNamesFromWire(response) {
    if (!response || typeof response !== 'object') return response;
    if (response.database && typeof response.database === 'object') {
      response.database.name = _zwIDBNameFromWire(response.database.name);
      (response.database.stores || []).forEach(function (store) {
        store.name = _zwIDBNameFromWire(store.name);
        (store.indexes || []).forEach(function (index) {
          index.name = _zwIDBNameFromWire(index.name);
        });
      });
    }
    if (Array.isArray(response.databases)) {
      response.databases.forEach(function (database) {
        database.name = _zwIDBNameFromWire(database.name);
      });
    }
    if (Array.isArray(response.stores)) {
      response.stores = response.stores.map(_zwIDBNameFromWire);
    }
    return response;
  }

  function _zwIDBHostCall(request) {
    if (typeof globalThis.__zw_idb !== 'function') return undefined;
    var wire = String(globalThis.__zw_idb(JSON.stringify(_zwIDBRequestNamesToWire(request))));
    var okPrefix = '__zw_idb_ok:';
    var errorPrefix = '__zw_idb_error:';
    if (wire.indexOf(okPrefix) === 0) {
      return _zwIDBResponseNamesFromWire(JSON.parse(wire.slice(okPrefix.length)));
    }
    if (wire.indexOf(errorPrefix) === 0) {
      var detail = wire.slice(errorPrefix.length);
      var separator = detail.indexOf(':');
      var name = separator === -1 ? 'UnknownError' : detail.slice(0, separator);
      var message = separator === -1 ? detail : detail.slice(separator + 1).trim();
      if (name === 'TypeError') throw new TypeError(message);
      throw new globalThis.DOMException(message, name);
    }
    throw new globalThis.DOMException('Invalid IndexedDB host response.', 'UnknownError');
  }

  function _zwIDBBinaryKeyBytes(value) {
    if (typeof ArrayBuffer === 'undefined') return undefined;
    try {
      var buffer;
      var byteOffset = 0;
      var byteLength;
      if (value instanceof ArrayBuffer) {
        buffer = value;
        byteLength = value.byteLength;
      } else if (ArrayBuffer.isView(value)) {
        buffer = value.buffer;
        byteOffset = value.byteOffset;
        byteLength = value.byteLength;
      } else {
        return undefined;
      }
      if (value._detached || buffer._detached) return null;
      return new Uint8Array(buffer, byteOffset, byteLength);
    } catch (_) {
      return null;
    }
  }

  function _zwIDBKeyToWire(value, seen) {
    seen = seen || [];
    if (typeof value === 'number') {
      if (value !== value) throw new globalThis.DOMException('Invalid IndexedDB key.', 'DataError');
      return { type: 'number', value: String(value) };
    }
    if (value instanceof Date) {
      var time = value.getTime();
      if (!isFinite(time)) throw new globalThis.DOMException('Invalid IndexedDB Date key.', 'DataError');
      return { type: 'date', value: String(time) };
    }
    if (typeof value === 'string') return { type: 'string', value: value };
    var binary = _zwIDBBinaryKeyBytes(value);
    if (binary !== undefined) {
      if (binary === null) {
        throw new globalThis.DOMException('Detached IndexedDB key.', 'DataError');
      }
      return { type: 'binary', value: Array.prototype.slice.call(binary) };
    }
    if (Array.isArray(value)) {
      if (_zwIDBTrackedProxies && _zwIDBTrackedProxies.has(value)) {
        throw new globalThis.DOMException('Proxy keys are invalid.', 'DataError');
      }
      if (seen.indexOf(value) !== -1) {
        throw new globalThis.DOMException('Cyclic IndexedDB key.', 'DataError');
      }
      seen.push(value);
      var entries = [];
      for (var i = 0; i < value.length; i++) {
        if (!Object.prototype.hasOwnProperty.call(value, i)) {
          seen.pop();
          throw new globalThis.DOMException('Sparse IndexedDB keys are invalid.', 'DataError');
        }
        entries.push(_zwIDBKeyToWire(value[i], seen));
      }
      seen.pop();
      return { type: 'array', value: entries };
    }
    throw new globalThis.DOMException('Invalid IndexedDB key.', 'DataError');
  }

  function _zwIDBKeyFromWire(wire) {
    if (!wire) return undefined;
    if (wire.type === 'number') return Number(wire.value);
    if (wire.type === 'date') return new Date(Number(wire.value));
    if (wire.type === 'string') return wire.value;
    if (wire.type === 'binary') return new Uint8Array(wire.value || []).buffer;
    if (wire.type === 'array') {
      return (wire.value || []).map(function (entry) { return _zwIDBKeyFromWire(entry); });
    }
    throw new globalThis.DOMException('Invalid IndexedDB key response.', 'UnknownError');
  }

  function _zwIDBConvertKey(value) {
    return _zwIDBKeyFromWire(_zwIDBKeyToWire(value));
  }

  function _zwIDBNeedsGraph(value, seen) {
    if (value === null || typeof value !== 'object') return false;
    if (seen.has(value)) return true;
    seen.add(value);
    if (value instanceof Date
        || (typeof Blob !== 'undefined' && value instanceof Blob)
        || (typeof ArrayBuffer !== 'undefined'
            && (value instanceof ArrayBuffer || ArrayBuffer.isView(value)))) return false;
    var keys = Object.keys(value);
    for (var i = 0; i < keys.length; i++) {
      if (_zwIDBNeedsGraph(value[keys[i]], seen)) return true;
    }
    return false;
  }

  function _zwIDBMapOwnArray(value, mapper) {
    var result = new Array(value.length);
    for (var i = 0; i < value.length; i++) {
      Object.defineProperty(result, i, {
        configurable: true,
        enumerable: true,
        value: Object.prototype.hasOwnProperty.call(value, i)
          ? mapper(value[i], i)
          : { __zwIdbType: 'undefined' },
        writable: true
      });
    }
    return result;
  }

  function _zwIDBGraphProjection(value, stack) {
    if (value === null || typeof value !== 'object') return _zwIDBValueToWire(value, []);
    if (value instanceof Date
        || (typeof Blob !== 'undefined' && value instanceof Blob)
        || (typeof ArrayBuffer !== 'undefined'
            && (value instanceof ArrayBuffer || ArrayBuffer.isView(value)))) {
      return _zwIDBValueToWire(value, []);
    }
    if (stack.indexOf(value) !== -1) return { __zwIdbType: 'unindexable' };
    stack.push(value);
    var projection;
    if (Array.isArray(value)) {
      projection = _zwIDBMapOwnArray(value, function (entry) {
        return _zwIDBGraphProjection(entry, stack);
      });
    } else {
      projection = {};
      Object.keys(value).forEach(function (key) {
        projection[key] = _zwIDBGraphProjection(value[key], stack);
      });
    }
    stack.pop();
    return projection;
  }

  function _zwIDBValueToGraphWire(value) {
    var seen = new Map();
    var nodes = [];
    function encode(entry) {
      if (entry === null || typeof entry !== 'object') return _zwIDBValueToWire(entry, []);
      if (seen.has(entry)) return { __zwIdbType: 'ref', value: seen.get(entry) };
      var id = nodes.length;
      seen.set(entry, id);
      var node = {};
      nodes.push(node);
      if (Array.isArray(entry)) {
        node.kind = 'array';
        node.value = _zwIDBMapOwnArray(entry, encode);
      } else if (entry instanceof Date
          || (typeof Blob !== 'undefined' && entry instanceof Blob)
          || (typeof ArrayBuffer !== 'undefined'
              && (entry instanceof ArrayBuffer || ArrayBuffer.isView(entry)))) {
        node.kind = 'value';
        node.value = _zwIDBValueToWire(entry, []);
      } else {
        node.kind = 'object';
        node.value = Object.keys(entry).map(function (key) {
          return [key, encode(entry[key])];
        });
      }
      return { __zwIdbType: 'ref', value: id };
    }
    return {
      __zwIdbType: 'graph',
      root: encode(value),
      nodes: nodes,
      indexProjection: _zwIDBGraphProjection(value, [])
    };
  }

  function _zwIDBValueToWire(value, seen) {
    if (!seen) {
      if (_zwIDBNeedsGraph(value, new Set())) return _zwIDBValueToGraphWire(value);
      seen = [];
    }
    if (value === undefined) return { __zwIdbType: 'undefined' };
    if (value === null || typeof value === 'string' || typeof value === 'boolean') return value;
    if (typeof value === 'number') {
      if (isFinite(value) && !(value === 0 && 1 / value < 0)) return value;
      return { __zwIdbType: 'number', value: String(value) };
    }
    if (value instanceof Date) {
      return { __zwIdbType: 'date', value: String(value.getTime()) };
    }
    if (typeof File !== 'undefined' && value instanceof File) {
      return {
        __zwIdbType: 'file',
        name: value.name || '',
        lastModified: Number(value.lastModified),
        type: value.type || '',
        value: Array.prototype.slice.call(_zw_blobBytes(value))
      };
    }
    if (typeof Blob !== 'undefined' && value instanceof Blob) {
      return {
        __zwIdbType: 'blob',
        type: value.type || '',
        value: Array.prototype.slice.call(_zw_blobBytes(value))
      };
    }
    if (typeof ArrayBuffer !== 'undefined') {
      if (value instanceof ArrayBuffer) {
        if (value._detached) throw new globalThis.DOMException('Detached value.', 'DataCloneError');
        return {
          __zwIdbType: 'arraybuffer',
          value: Array.prototype.slice.call(new Uint8Array(value))
        };
      }
      if (ArrayBuffer.isView(value)) {
        if (value._detached || value.buffer._detached) {
          throw new globalThis.DOMException('Detached value.', 'DataCloneError');
        }
        return {
          __zwIdbType: 'view',
          name: value.constructor && value.constructor.name || 'Uint8Array',
          value: Array.prototype.slice.call(
            new Uint8Array(value.buffer, value.byteOffset || 0, value.byteLength)
          )
        };
      }
    }
    if (typeof value !== 'object') {
      throw new globalThis.DOMException('Value cannot be cloned.', 'DataCloneError');
    }
    if (seen.indexOf(value) !== -1) {
      throw new globalThis.DOMException('Cyclic values are not supported.', 'DataCloneError');
    }
    seen.push(value);
    var wire;
    if (Array.isArray(value)) {
      wire = _zwIDBMapOwnArray(value, function (entry) {
        return _zwIDBValueToWire(entry, seen);
      });
    } else if (!Object.prototype.hasOwnProperty.call(value, '__zwIdbType')) {
      wire = {};
      Object.keys(value).forEach(function (key) {
        wire[key] = _zwIDBValueToWire(value[key], seen);
      });
    } else {
      wire = {
        __zwIdbType: 'object',
        value: Object.keys(value).map(function (key) {
          return [key, _zwIDBValueToWire(value[key], seen)];
        })
      };
    }
    seen.pop();
    return wire;
  }

  function _zwIDBValueFromGraphWire(wire) {
    var graphNodes = wire.nodes || [];
    var values = graphNodes.map(function (node) {
      if (node.kind === 'array') return [];
      if (node.kind === 'object') return {};
      if (node.kind === 'value') return _zwIDBValueFromWire(node.value);
      throw new globalThis.DOMException('Invalid IndexedDB graph node.', 'UnknownError');
    });
    function decode(entry) {
      if (entry && entry.__zwIdbType === 'ref') {
        var id = Number(entry.value);
        if (id < 0 || id >= values.length || Math.floor(id) !== id) {
          throw new globalThis.DOMException('Invalid IndexedDB graph reference.', 'UnknownError');
        }
        return values[id];
      }
      return _zwIDBValueFromWire(entry);
    }
    graphNodes.forEach(function (node, id) {
      if (node.kind === 'array') {
        (node.value || []).forEach(function (entry) { values[id].push(decode(entry)); });
      } else if (node.kind === 'object') {
        (node.value || []).forEach(function (entry) {
          values[id][entry[0]] = decode(entry[1]);
        });
      }
    });
    return decode(wire.root);
  }

  function _zwIDBValueFromWire(wire) {
    if (wire === null || typeof wire !== 'object') return wire;
    if (Array.isArray(wire)) {
      return wire.map(function (entry) { return _zwIDBValueFromWire(entry); });
    }
    if (!wire.__zwIdbType) {
      var plain = {};
      Object.keys(wire).forEach(function (key) {
        plain[key] = _zwIDBValueFromWire(wire[key]);
      });
      return plain;
    }
    if (wire.__zwIdbType === 'undefined') return undefined;
    if (wire.__zwIdbType === 'number') return Number(wire.value);
    if (wire.__zwIdbType === 'date') return new Date(Number(wire.value));
    if (wire.__zwIdbType === 'file') {
      return new File(
        [new Uint8Array(wire.value || [])],
        wire.name || '',
        { type: wire.type || '', lastModified: Number(wire.lastModified) }
      );
    }
    if (wire.__zwIdbType === 'blob') {
      return new Blob([new Uint8Array(wire.value || [])], { type: wire.type || '' });
    }
    if (wire.__zwIdbType === 'arraybuffer') return new Uint8Array(wire.value || []).buffer;
    if (wire.__zwIdbType === 'view') {
      var View = globalThis[wire.name] || Uint8Array;
      try { return new View(new Uint8Array(wire.value || []).buffer); }
      catch (_) { return new Uint8Array(wire.value || []); }
    }
    if (wire.__zwIdbType === 'graph') return _zwIDBValueFromGraphWire(wire);
    if (wire.__zwIdbType === 'array') {
      return (wire.value || []).map(function (entry) { return _zwIDBValueFromWire(entry); });
    }
    if (wire.__zwIdbType === 'object') {
      var object = {};
      (wire.value || []).forEach(function (entry) {
        object[entry[0]] = _zwIDBValueFromWire(entry[1]);
      });
      return object;
    }
    throw new globalThis.DOMException('Invalid IndexedDB value response.', 'UnknownError');
  }

  function _zwIDBQueryToWire(query) {
    if (_zwIDBIsKeyRange(query)) {
      var range = {
        lowerOpen: query.lowerOpen,
        upperOpen: query.upperOpen
      };
      if (query.lower !== undefined) range.lower = _zwIDBKeyToWire(query.lower);
      if (query.upper !== undefined) range.upper = _zwIDBKeyToWire(query.upper);
      return { type: 'range', value: range };
    }
    return { type: 'key', value: _zwIDBKeyToWire(query) };
  }

  function _zwIDBRequestHostError(request, error) {
    request.error = error;
    var event = new _zwIDBEvent('error', request);
    event.bubbles = true;
    event.cancelable = true;
    event._requestError = error;
    _zwIDBDispatch(request, 'error', undefined, event);
    return request;
  }

  function _zwIDBStateFromHost(database) {
    var stores = {};
    (database.stores || []).forEach(function (store) {
      var indexes = {};
      (store.indexes || []).forEach(function (index) {
        indexes[index.name] = {
          keyPath: index.keyPath,
          unique: !!index.unique,
          multiEntry: !!index.multiEntry,
          deleted: false,
          createdInUpgrade: false
        };
      });
      stores[store.name] = {
        keyPath: store.keyPath === undefined ? null : store.keyPath,
        autoIncrement: !!store.autoIncrement,
        records: new Map(),
        indexes: indexes,
        nextKey: 1,
        deleted: false,
        createdInUpgrade: false
      };
    });
    return {
      version: Number(database.version),
      stores: stores,
      connections: [],
      transactions: []
    };
  }

  function _zwIDBSchemaForHost(name, state) {
    return {
      op: 'sync_schema',
      name: name,
      version: state.version,
      stores: Object.keys(state.stores).sort().map(function (storeName) {
        var store = state.stores[storeName];
        return {
          name: storeName,
          keyPath: store.keyPath,
          autoIncrement: !!store.autoIncrement,
          indexes: Object.keys(store.indexes).sort().map(function (indexName) {
            var index = store.indexes[indexName];
            return {
              name: indexName,
              keyPath: index.keyPath,
              unique: !!index.unique,
              multiEntry: !!index.multiEntry
            };
          })
        };
      })
    };
  }

  function _zwIDBEvent(type, target) {
    if (typeof globalThis.Event === 'function'
        && Object.getPrototypeOf(_zwIDBEvent.prototype) !== globalThis.Event.prototype) {
      Object.setPrototypeOf(_zwIDBEvent.prototype, globalThis.Event.prototype);
    }
    this.type = type;
    this.target = target;
    this.currentTarget = target;
    this.bubbles = false;
    this.cancelable = false;
    this.defaultPrevented = false;
    this._propagationStopped = false;
    this._immediatePropagationStopped = false;
    this.timestamp = 0;
  }
  _zwIDBEvent.prototype = Object.create((globalThis.Event || Object).prototype);
  _zwIDBEvent.prototype.constructor = _zwIDBEvent;
  _zwIDBEvent.prototype.preventDefault = function () {
    if (this.cancelable) this.defaultPrevented = true;
  };
  _zwIDBEvent.prototype.stopPropagation = function () { this._propagationStopped = true; };
  _zwIDBEvent.prototype.stopImmediatePropagation = function () {
    this._propagationStopped = true;
    this._immediatePropagationStopped = true;
  };

  function IDBVersionChangeEvent(type, init) {
    init = init || {};
    _zwIDBEvent.call(this, String(type), null);
    this.oldVersion = Number(init.oldVersion || 0);
    this.newVersion = init.newVersion === null ? null
      : (init.newVersion === undefined ? null : Number(init.newVersion));
  }
  IDBVersionChangeEvent.prototype = Object.create(_zwIDBEvent.prototype);
  IDBVersionChangeEvent.prototype.constructor = IDBVersionChangeEvent;
  globalThis.IDBVersionChangeEvent = IDBVersionChangeEvent;

  function _zwIDBRequest(source) {
    this.readyState = 'pending';
    this._result = undefined;
    this._error = null;
    this.source = source || null;
    this.transaction = null;
    this.onsuccess = null;
    this.onerror = null;
    this.onupgradeneeded = null;
    this.onblocked = null;
    this._listeners = {};
  }
  function _zwIDBOpenRequest() {
    _zwIDBRequest.call(this, null);
  }
  _zwIDBOpenRequest.prototype = Object.create(_zwIDBRequest.prototype);
  _zwIDBOpenRequest.prototype.constructor = _zwIDBOpenRequest;
  Object.defineProperties(_zwIDBRequest.prototype, {
    result: {
      configurable: true,
      get: function () {
        if (this.readyState === 'pending') {
          throw new globalThis.DOMException('The request is still pending.', 'InvalidStateError');
        }
        return this._result;
      },
      set: function (value) { this._result = value; }
    },
    error: {
      configurable: true,
      get: function () {
        if (this.readyState === 'pending') {
          throw new globalThis.DOMException('The request is still pending.', 'InvalidStateError');
        }
        return this._error;
      },
      set: function (value) { this._error = value; }
    }
  });
  _zwIDBRequest.prototype.addEventListener = function (type, callback, options) {
    if (callback == null) return;
    type = String(type);
    var listeners = this._listeners[type] || (this._listeners[type] = []);
    var capture = options === true || !!(options && options.capture);
    if (!listeners.some(function (listener) {
      return listener.callback === callback && listener.capture === capture;
    })) listeners.push({ callback: callback, capture: capture });
  };
  _zwIDBRequest.prototype.removeEventListener = function (type, callback, options) {
    var listeners = this._listeners[String(type)];
    if (!listeners) return;
    var capture = options === true || !!(options && options.capture);
    var index = listeners.findIndex(function (listener) {
      return listener.callback === callback && listener.capture === capture;
    });
    if (index !== -1) listeners.splice(index, 1);
  };
  _zwIDBRequest.prototype.dispatchEvent = function (event) {
    if (!event || typeof event.type === 'undefined') {
      throw new TypeError('IDBRequest.dispatchEvent requires an event');
    }
    event.target = this;
    event.currentTarget = this;
    _zwIDBEmit(this, String(event.type), event);
    return !event.defaultPrevented;
  };

  // https://w3c.github.io/IndexedDB/#fire-an-event
  function _zwIDBReportListenerException(error) {
    if (typeof _zwReportListenerError === 'function') {
      _zwReportListenerError(error);
    }
  }
  function _zwIDBCallListener(target, callback, event) {
    try {
      var callable = callback;
      var thisValue = target;
      if (typeof callback !== 'function') {
        callable = callback && callback.handleEvent;
        thisValue = callback;
        if (typeof callable !== 'function') {
          throw new TypeError('The event listener is not callable.');
        }
      }
      callable.call(thisValue, event);
      return false;
    } catch (error) {
      _zwIDBReportListenerException(error);
      return true;
    }
  }
  function _zwIDBInvoke(target, type, event, capture) {
    var listeners = ((target._listeners && target._listeners[type]) || []).slice();
    var exceptionThrown = false;
    if (!capture && !event._immediatePropagationStopped) {
      var handler = target['on' + type];
      if (typeof handler === 'function') {
        exceptionThrown = _zwIDBCallListener(target, handler, event) || exceptionThrown;
      }
    }
    for (var i = 0; i < listeners.length; i++) {
      if (event._immediatePropagationStopped) break;
      var listener = listeners[i];
      if (listener.capture !== capture) continue;
      exceptionThrown = _zwIDBCallListener(target, listener.callback, event)
        || exceptionThrown;
    }
    return exceptionThrown;
  }

  function _zwIDBEmit(target, type, event) {
    event.currentTarget = target;
    var exceptionThrown = _zwIDBInvoke(target, type, event, true);
    exceptionThrown = _zwIDBInvoke(target, type, event, false) || exceptionThrown;
    return exceptionThrown;
  }

  function _zwIDBRequestEventSteps(request, transaction, event) {
    var database = transaction && transaction.db;
    var steps = [];
    function addListeners(target, capture, group) {
      var listeners = ((target && target._listeners && target._listeners[event.type]) || []).slice();
      listeners.forEach(function (listener) {
        if (listener.capture === capture) {
          steps.push({ target: target, callback: listener.callback, group: group });
        }
      });
    }
    function addBubble(target, group) {
      if (!target) return;
      var handler = target['on' + event.type];
      if (typeof handler === 'function') {
        steps.push({ target: target, callback: handler, group: group });
      }
      addListeners(target, false, group);
    }
    if (database) addListeners(database, true, 0);
    if (transaction) addListeners(transaction, true, 1);
    addListeners(request, true, 2);
    addBubble(request, 2);
    if (event.bubbles) {
      addBubble(transaction, 3);
      addBubble(database, 4);
    }
    return steps;
  }

  function _zwIDBDispatchRequestEvent(request, transaction, event, done) {
    var steps = _zwIDBRequestEventSteps(request, transaction, event);
    var position = 0;
    var currentGroup = -1;
    var invoked = false;
    var exceptionThrown = false;
    event.target = request;
    function next() {
      if (invoked) _zwIDBDeactivateTransactions(transaction);
      while (position < steps.length) {
        var step = steps[position++];
        if (event._immediatePropagationStopped) break;
        if (event._propagationStopped && step.group !== currentGroup) break;
        currentGroup = step.group;
        event.currentTarget = step.target;
        exceptionThrown = _zwIDBCallListener(step.target, step.callback, event)
          || exceptionThrown;
        invoked = true;
        event.currentTarget = null;
        queueMicrotask(next);
        return;
      }
      done(exceptionThrown);
    }
    next();
  }

  function _zwIDBEmitTransactionEvent(transaction, type, bubbles) {
    var event = new _zwIDBEvent(type, transaction);
    event.bubbles = !!bubbles;
    event.target = transaction;
    if (transaction.db) {
      event.currentTarget = transaction.db;
      _zwIDBInvoke(transaction.db, type, event, true);
    }
    if (!event._propagationStopped) {
      event.currentTarget = transaction;
      _zwIDBInvoke(transaction, type, event, true);
      _zwIDBInvoke(transaction, type, event, false);
    }
    if (bubbles && transaction.db && !event._propagationStopped) {
      event.currentTarget = transaction.db;
      _zwIDBInvoke(transaction.db, type, event, false);
    }
  }

  // Request 经 timer task 派发；每个 listener callback 之间保留 microtask checkpoint。
  function _zwIDBDispatch(req, type, result, event) {
    var transaction = req.transaction;
    if (transaction) transaction._pending++;
    if (transaction) transaction._autoCommitPending = false;
    var dispatch = {
      request: req,
      type: type,
      result: result,
      event: event,
      firing: false,
      settled: false
    };
    if (transaction) transaction._requestQueue.push(dispatch);
    var fire = function () {
      dispatch.firing = true;
      req.readyState = 'done';
      if (dispatch.result && dispatch.result._isIDBCursor) {
        dispatch.result._applyPendingPosition();
        dispatch.result._gotValue = true;
      }
      if (dispatch.result !== undefined) req.result = dispatch.result;
      var ev = dispatch.event || new _zwIDBEvent(
        dispatch.type === 'error' ? 'error' : (dispatch.type === 'upgradeneeded' ? 'upgradeneeded' : 'success'),
        req
      );
      if (ev._requestError) req.error = ev._requestError;
      if (transaction) transaction._active = true;
      _zwIDBDispatchRequestEvent(req, transaction, ev, function (listenerException) {
        dispatch.settled = true;
        if (listenerException && transaction && !transaction._aborted
            && !transaction._committing && !transaction._finished) {
          transaction._requestError = new globalThis.DOMException(
            'An event listener threw while the request was being dispatched.',
            'AbortError'
          );
          transaction.abort();
        } else if (ev.type === 'error' && transaction
            && !ev.defaultPrevented && !transaction._aborted) {
          transaction._requestError = req.error;
          transaction.abort();
        }
        if (transaction) {
          transaction._active = false;
          transaction._pending--;
          var position = transaction._requestQueue.indexOf(dispatch);
          if (position !== -1) transaction._requestQueue.splice(position, 1);
          if (transaction._pending === 0) {
            transaction._autoCommitPending = true;
            _zwIDBScheduleTransactionCompletion(transaction);
          }
        }
      });
    };
    if (typeof setTimeout === 'function') setTimeout(fire, 0);
    else fire();
  }

  // https://w3c.github.io/IndexedDB/#dom-idbdatabase-objectstorenames
  function _zwIDBStringList(getNames) {
    function values() {
      return getNames().map(String).sort();
    }
    var list = {
      contains: function (name) { return values().indexOf(String(name)) !== -1; },
      item: function (index) {
        var entries = values();
        index = Number(index);
        return index >= 0 && index < entries.length ? entries[index] : null;
      }
    };
    if (typeof Symbol === 'function' && Symbol.iterator) {
      list[Symbol.iterator] = function () { return values()[Symbol.iterator](); };
    }
    return new Proxy(list, {
      get: function (target, property) {
        var entries = values();
        if (property === 'length') return entries.length;
        if (typeof property === 'string' && /^(0|[1-9][0-9]*)$/.test(property)) {
          return entries[Number(property)];
        }
        return target[property];
      },
      has: function (target, property) {
        if (property === 'length') return true;
        if (typeof property === 'string' && /^(0|[1-9][0-9]*)$/.test(property)) {
          return Number(property) < values().length;
        }
        return property in target;
      },
      getOwnPropertyDescriptor: function (target, property) {
        if (typeof property === 'string' && /^(0|[1-9][0-9]*)$/.test(property)
            && Number(property) < values().length) {
          return {
            configurable: true,
            enumerable: true,
            value: values()[Number(property)],
            writable: false
          };
        }
        return Object.getOwnPropertyDescriptor(target, property);
      }
    });
  }

  function _zwIDBValidKeyPathString(keyPath) {
    if (keyPath === '') return true;
    return keyPath.split('.').every(function (part) {
      return /^[$A-Z_a-z\u0080-\uFFFF][$0-9A-Z_a-z\u0080-\uFFFF]*$/.test(part);
    });
  }

  function _zwIDBNormalizeKeyPath(value, supplied) {
    if (!supplied || value == null) return null;
    if (Array.isArray(value)) {
      if (value.length === 0) {
        throw new globalThis.DOMException('The key path sequence is empty.', 'SyntaxError');
      }
      var sequence = value.map(String);
      if (!sequence.every(_zwIDBValidKeyPathString)) {
        throw new globalThis.DOMException('The key path is invalid.', 'SyntaxError');
      }
      return sequence;
    }
    var keyPath = String(value);
    if (!_zwIDBValidKeyPathString(keyPath)) {
      throw new globalThis.DOMException('The key path is invalid.', 'SyntaxError');
    }
    return keyPath;
  }

  // https://w3c.github.io/IndexedDB/#extract-key-from-value
  function _zwIDBKeyPathProperty(value, property) {
    if (value == null) return undefined;
    if (property === 'length' && (typeof value === 'string' || Array.isArray(value))) {
      return value.length;
    }
    if (typeof Blob !== 'undefined' && value instanceof Blob) {
      if (property === 'size') return value.size;
      if (property === 'type') return value.type;
    }
    if (typeof File !== 'undefined' && value instanceof File) {
      if (property === 'name') return value.name;
      if (property === 'lastModified') return value.lastModified;
    }
    if (!Object.prototype.hasOwnProperty.call(Object(value), property)) return undefined;
    return value[property];
  }

  function _zwIDBExtractKeyPath(value, keyPath) {
    if (keyPath === '') return value;
    var key = value;
    String(keyPath).split('.').forEach(function (property) {
      key = _zwIDBKeyPathProperty(key, property);
    });
    return key;
  }

  function _zwIDBCanInjectKey(value, keyPath) {
    if (typeof keyPath !== 'string' || keyPath === '') return false;
    var target = value;
    var parts = keyPath.split('.');
    for (var i = 0; i < parts.length - 1; i++) {
      if (target == null || (typeof target !== 'object' && typeof target !== 'function')) return false;
      if (!Object.prototype.hasOwnProperty.call(target, parts[i])) return true;
      target = target[parts[i]];
    }
    return target != null && (typeof target === 'object' || typeof target === 'function');
  }

  function _zwIDBStore(db, name, keyPath, autoIncrement, records, indexes, transaction, metadata) {
    this._db = db;
    this._name = String(name);
    this.keyPath = Array.isArray(keyPath) ? keyPath.slice() : (keyPath == null ? null : keyPath);
    this.autoIncrement = !!autoIncrement;
    this._records = records; // Map<key, value>
    this._indexes = indexes; // {indexName: {keyPath, unique}}
    this._indexInstances = {};
    this._indexInstanceList = [];
    this.transaction = transaction || null;
    this._metadata = metadata;
    var store = this;
    this.indexNames = _zwIDBStringList(function () { return Object.keys(store._indexes); });
  }
  // https://w3c.github.io/IndexedDB/#dom-idbobjectstore-name
  Object.defineProperty(_zwIDBStore.prototype, 'name', {
    configurable: true,
    enumerable: true,
    get: function () { return this._name; },
    set: function (value) {
      var name = String(value);
      this._assertSchemaChange();
      if (name === this._name) return;
      if (Object.prototype.hasOwnProperty.call(this._db._stores, name)) {
        throw new globalThis.DOMException('The object store already exists.', 'ConstraintError');
      }
      var oldName = this._name;
      delete this._db._stores[oldName];
      this._db._stores[name] = this._metadata;
      this.transaction._scope = this.transaction._scope.map(function (entry) {
        return entry === oldName ? name : entry;
      }).sort();
      delete this.transaction._storeInstances[oldName];
      this.transaction._storeInstances[name] = this;
      this._name = name;
      if (!this._metadata.createdInUpgrade) {
        this.transaction._schemaRenames.push({
          kind: 'store',
          instance: this,
          oldName: oldName
        });
      }
    }
  });
  _zwIDBStore.prototype._assertUsable = function (write) {
    if (this._metadata && this._metadata.deleted) {
      throw new globalThis.DOMException('The object store has been deleted.', 'InvalidStateError');
    }
    if (this.transaction
        && (!this.transaction._active
            || this.transaction._aborted
            || this.transaction._finished
            || this.transaction._committing)) {
      throw new globalThis.DOMException('The transaction is inactive.', 'TransactionInactiveError');
    }
    if (write && this.transaction && this.transaction.mode === 'readonly') {
      throw new globalThis.DOMException('The transaction is read-only.', 'ReadOnlyError');
    }
  };
  _zwIDBStore.prototype._keyOf = function (value) {
    if (this.keyPath === null) return null;
    if (Array.isArray(this.keyPath)) {
      return this.keyPath.map(function (path) { return _zwIDBExtractKeyPath(value, path); });
    }
    return _zwIDBExtractKeyPath(value, this.keyPath);
  };
  _zwIDBStore.prototype._setKeyPath = function (value, key) {
    var parts = String(this.keyPath).split('.');
    var target = value;
    for (var i = 0; i < parts.length - 1; i++) {
      if (target == null || (typeof target !== 'object' && typeof target !== 'function')) {
        throw new globalThis.DOMException('The generated key cannot be inserted.', 'DataError');
      }
      if (!Object.prototype.hasOwnProperty.call(target, parts[i])) {
        Object.defineProperty(target, parts[i], {
          configurable: true,
          enumerable: true,
          value: {},
          writable: true
        });
      }
      target = target[parts[i]];
    }
    if (target == null || (typeof target !== 'object' && typeof target !== 'function')) {
      throw new globalThis.DOMException('The generated key cannot be inserted.', 'DataError');
    }
    var property = parts[parts.length - 1];
    if (Object.prototype.hasOwnProperty.call(target, property)) {
      target[property] = key;
    } else {
      Object.defineProperty(target, property, {
        configurable: true,
        enumerable: true,
        value: key,
        writable: true
      });
    }
  };
  _zwIDBStore.prototype._resolveKey = function (value, key, keyProvided) {
    // https://w3c.github.io/IndexedDB/#store-a-record-into-an-object-store
    var inline = this.keyPath !== null;
    if (inline && keyProvided) {
      throw new globalThis.DOMException('Inline key stores do not accept an explicit key.', 'DataError');
    }
    var resolved = inline ? this._keyOf(value) : key;
    if (resolved === undefined) {
      if (!this.autoIncrement) {
        throw new globalThis.DOMException('A key is required for this object store.', 'DataError');
      }
      resolved = this._metadata.nextKey || 1;
      this._metadata.nextKey = resolved + 1;
      if (inline) this._setKeyPath(value, resolved);
    }
    if (!_zwIDBKey(resolved, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    if (this.autoIncrement && typeof resolved === 'number') {
      var next = Math.floor(resolved) + 1;
      if (!this._metadata.nextKey || next > this._metadata.nextKey) this._metadata.nextKey = next;
    }
    return resolved;
  };
  _zwIDBStore.prototype._recordKey = function (key) {
    var matched;
    this._records.forEach(function (_value, recordKey) {
      if (matched === undefined && _zwIDBCompareValues(recordKey, key) === 0) matched = recordKey;
    });
    return matched;
  };
  _zwIDBStore.prototype._indexKey = function (value, keyPath) {
    if (Array.isArray(keyPath)) {
      var compound = keyPath.map(function (path) {
        return _zwIDBExtractKeyPath(value, path);
      });
      return compound.some(function (key) { return key === undefined; }) ? undefined : compound;
    }
    return _zwIDBExtractKeyPath(value, keyPath);
  };
  _zwIDBStore.prototype._hasUniqueConflict = function (value, primaryKey) {
    var store = this;
    return Object.keys(this._indexes).some(function (name) {
      var index = store._indexes[name];
      if (!index.unique) return false;
      var candidate = store._indexKey(value, index.keyPath);
      if (!_zwIDBKey(candidate, [])) return false;
      var conflict = false;
      store._records.forEach(function (record, recordKey) {
        if (conflict
            || (primaryKey !== undefined && _zwIDBCompareValues(recordKey, primaryKey) === 0)) return;
        var existing = store._indexKey(record, index.keyPath);
        if (_zwIDBKey(existing, []) && _zwIDBCompareValues(existing, candidate) === 0) {
          conflict = true;
        }
      });
      return conflict;
    });
  };
  _zwIDBStore.prototype._constraintError = function (request) {
    var error = new globalThis.DOMException(
      'A record with the same key already exists.',
      'ConstraintError'
    );
    var event = new _zwIDBEvent('error', request);
    event.bubbles = true;
    event.cancelable = true;
    event._requestError = error;
    _zwIDBDispatch(request, 'error', undefined, event);
    return request;
  };
  _zwIDBStore.prototype._mutate = function (op, value, key, keyProvided) {
    this._assertUsable(true);
    var storedValue = globalThis.structuredClone(value);
    var inline = this.keyPath !== null;
    if (inline && keyProvided) {
      throw new globalThis.DOMException('Inline key stores do not accept an explicit key.', 'DataError');
    }
    var candidate = inline ? this._keyOf(storedValue) : key;
    if (candidate === undefined && !this.autoIncrement) {
      throw new globalThis.DOMException('A key is required for this object store.', 'DataError');
    }
    if (candidate === undefined && inline && !_zwIDBCanInjectKey(storedValue, this.keyPath)) {
      throw new globalThis.DOMException('The generated key cannot be inserted.', 'DataError');
    }
    if (candidate !== undefined && !_zwIDBKey(candidate, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var req = new _zwIDBRequest(this);
    req.transaction = this.transaction;
    var store = this;
    var perform = function () {
      if (!(store.transaction && store.transaction._hostId !== null)) {
        var fallbackKey = store._resolveKey(storedValue, key, keyProvided);
        var fallbackExistingKey = store._recordKey(fallbackKey);
        if ((op === 'add' && fallbackExistingKey !== undefined)
            || store._hasUniqueConflict(storedValue, fallbackKey)) {
          store._constraintError(req);
          return;
        }
        store._records.set(
          fallbackExistingKey === undefined ? fallbackKey : fallbackExistingKey,
          storedValue
        );
        _zwIDBDispatch(req, 'success', fallbackKey);
        return;
      }
      var localKey = candidate === undefined ? undefined : store._recordKey(candidate);
      if ((op === 'add' && localKey !== undefined)
          || store._hasUniqueConflict(storedValue, candidate)) {
        store._constraintError(req);
        return;
      }
      var hostRequest = {
        op: op === 'add' ? 'transaction_add' : 'transaction_put',
        transaction: store.transaction._hostId,
        store: store.name,
        value: _zwIDBValueToWire(storedValue)
      };
      if (candidate !== undefined) hostRequest.key = _zwIDBKeyToWire(candidate);
      var response;
      try {
        response = _zwIDBHostCall(hostRequest);
      } catch (hostError) {
        _zwIDBRequestHostError(req, hostError);
        return;
      }
      var k = _zwIDBKeyFromWire(response.key);
      if (candidate === undefined && store.keyPath !== null) store._setKeyPath(storedValue, k);
      if (store.autoIncrement && typeof k === 'number') {
        var next = Math.floor(k) + 1;
        if (!store._metadata.nextKey || next > store._metadata.nextKey) {
          store._metadata.nextKey = next;
        }
      }
      var existingKey = store._recordKey(k);
      store._records.set(existingKey === undefined ? k : existingKey, storedValue);
      _zwIDBDispatch(req, 'success', k);
    };
    _zwIDBRunTransactionOperation(this.transaction, perform);
    return req;
  };
  _zwIDBStore.prototype.add = function (value, key) {
    return this._mutate('add', value, key, arguments.length >= 2);
  };
  _zwIDBStore.prototype.put = function (value, key) {
    return this._mutate('put', value, key, arguments.length >= 2);
  };
  _zwIDBStore.prototype.get = function (key) {
    this._assertUsable(false);
    if (!_zwIDBIsKeyRange(key) && !_zwIDBKey(key, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var req = new _zwIDBRequest(this);
    req.transaction = this.transaction;
    var store = this;
    var perform = function () {
      if (store.transaction && store.transaction._hostId !== null) {
        try {
          var response;
          if (_zwIDBIsKeyRange(key)) {
            response = _zwIDBHostCall({
              op: 'transaction_get_all',
              transaction: store.transaction._hostId,
              store: store.name,
              query: _zwIDBQueryToWire(key),
              count: 1
            });
            var first = response.records && response.records[0];
            _zwIDBDispatch(req, 'success', first ? _zwIDBValueFromWire(first.value) : undefined);
          } else {
            response = _zwIDBHostCall({
              op: 'transaction_get',
              transaction: store.transaction._hostId,
              store: store.name,
              key: _zwIDBKeyToWire(key)
            });
            _zwIDBDispatch(
              req,
              'success',
              response.record ? _zwIDBValueFromWire(response.record.value) : undefined
            );
          }
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
        }
        return;
      }
      var result;
      if (_zwIDBIsKeyRange(key)) {
        var matches = [];
        store._records.forEach(function (value, recordKey) {
          if (key.includes(recordKey)) matches.push({ key: recordKey, value: value });
        });
        matches.sort(function (a, b) { return _zwIDBCompareValues(a.key, b.key); });
        result = matches.length ? matches[0].value : undefined;
      } else {
        store._records.forEach(function (value, recordKey) {
          if (result === undefined && _zwIDBCompareValues(recordKey, key) === 0) result = value;
        });
      }
      _zwIDBDispatch(req, 'success', result);
    };
    _zwIDBRunTransactionOperation(this.transaction, perform);
    return req;
  };
  _zwIDBStore.prototype.getKey = function (query) {
    this._assertUsable(false);
    if (arguments.length === 0) {
      throw new TypeError('IDBObjectStore.getKey requires a query.');
    }
    if (!_zwIDBIsKeyRange(query) && !_zwIDBKey(query, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var req = new _zwIDBRequest(this);
    req.transaction = this.transaction;
    var store = this;
    var perform = function () {
      if (store.transaction && store.transaction._hostId !== null) {
        try {
          var response = _zwIDBHostCall({
            op: 'transaction_get_all',
            transaction: store.transaction._hostId,
            store: store.name,
            query: _zwIDBQueryToWire(query),
            count: 1,
            keys_only: true
          });
          var first = response.records && response.records[0];
          _zwIDBDispatch(req, 'success', first ? _zwIDBKeyFromWire(first.key) : undefined);
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
        }
        return;
      }
      var keys = [];
      store._records.forEach(function (_value, key) {
        if (_zwIDBQueryMatches(query, key)) keys.push(key);
      });
      keys.sort(_zwIDBCompareValues);
      _zwIDBDispatch(req, 'success', keys.length ? keys[0] : undefined);
    };
    _zwIDBRunTransactionOperation(this.transaction, perform);
    return req;
  };
  _zwIDBStore.prototype.delete = function (key) {
    this._assertUsable(true);
    if (!_zwIDBIsKeyRange(key) && !_zwIDBKey(key, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var req = new _zwIDBRequest(this);
    req.transaction = this.transaction;
    var store = this;
    var perform = function () {
      if (store.transaction && store.transaction._hostId !== null) {
        try {
          _zwIDBHostCall(_zwIDBIsKeyRange(key) ? {
            op: 'transaction_delete_range',
            transaction: store.transaction._hostId,
            store: store.name,
            range: _zwIDBQueryToWire(key).value
          } : {
            op: 'transaction_delete',
            transaction: store.transaction._hostId,
            store: store.name,
            key: _zwIDBKeyToWire(key)
          });
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
          return;
        }
      }
      if (_zwIDBIsKeyRange(key)) {
        var records = store._records;
        var keys = [];
        records.forEach(function (_value, recordKey) {
          if (key.includes(recordKey)) keys.push(recordKey);
        });
        keys.forEach(function (recordKey) { records.delete(recordKey); });
      } else {
        var matchingKey;
        store._records.forEach(function (_value, recordKey) {
          if (matchingKey === undefined
              && _zwIDBCompareValues(recordKey, key) === 0) matchingKey = recordKey;
        });
        if (matchingKey !== undefined) store._records.delete(matchingKey);
      }
      _zwIDBDispatch(req, 'success', undefined);
    };
    _zwIDBRunTransactionOperation(this.transaction, perform);
    return req;
  };
  _zwIDBStore.prototype.clear = function () {
    this._assertUsable(true);
    var req = new _zwIDBRequest(this);
    req.transaction = this.transaction;
    var store = this;
    var perform = function () {
      if (store.transaction && store.transaction._hostId !== null) {
        try {
          _zwIDBHostCall({
            op: 'transaction_clear',
            transaction: store.transaction._hostId,
            store: store.name
          });
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
          return;
        }
      }
      store._records.clear();
      _zwIDBDispatch(req, 'success', undefined);
    };
    _zwIDBRunTransactionOperation(this.transaction, perform);
    return req;
  };
  _zwIDBStore.prototype.count = function (query) {
    this._assertUsable(false);
    var req = new _zwIDBRequest(this);
    req.transaction = this.transaction;
    var store = this;
    var queryProvided = arguments.length >= 1;
    if (queryProvided && query !== undefined
        && !_zwIDBIsKeyRange(query) && !_zwIDBKey(query, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var perform = function () {
      if (store.transaction && store.transaction._hostId !== null) {
        var hostRequest = {
          op: 'transaction_count',
          transaction: store.transaction._hostId,
          store: store.name
        };
        if (queryProvided && query !== undefined) hostRequest.query = _zwIDBQueryToWire(query);
        try {
          var hosted = _zwIDBHostCall(hostRequest);
          _zwIDBDispatch(req, 'success', hosted.count);
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
        }
        return;
      }
      var count = 0;
      if (!queryProvided || query === undefined) {
        count = store._records.size;
      } else {
        store._records.forEach(function (_value, recordKey) {
          if (_zwIDBQueryMatches(query, recordKey)) count++;
        });
      }
      _zwIDBDispatch(req, 'success', count);
    };
    _zwIDBRunTransactionOperation(this.transaction, perform);
    return req;
  };
  // https://w3c.github.io/IndexedDB/#dom-idbobjectstore-getall
  function _zwIDBEnforceUnsignedLong(value) {
    value = Number(value);
    if (!isFinite(value) || value < 0 || value > 4294967295) {
      throw new TypeError('The count is outside the unsigned long range.');
    }
    return value < 0 ? Math.ceil(value) : Math.floor(value);
  }
  function _zwIDBIsGetAllOptions(value) {
    return value !== null
      && typeof value === 'object'
      && !_zwIDBIsKeyRange(value)
      && !Array.isArray(value)
      && !(value instanceof Date)
      && !(value instanceof ArrayBuffer)
      && !ArrayBuffer.isView(value);
  }
  function _zwIDBNormalizeGetAllOptions(queryOrOptions, count, argumentCount, recordsOnly) {
    if (recordsOnly || (argumentCount >= 1 && _zwIDBIsGetAllOptions(queryOrOptions))) {
      var dictionary = queryOrOptions === null || queryOrOptions === undefined
        ? {}
        : Object(queryOrOptions);
      // Web IDL dictionary members are read in lexicographic order.
      var countValue = dictionary.count;
      var normalizedCount = countValue === undefined
        ? undefined
        : _zwIDBEnforceUnsignedLong(countValue);
      if (normalizedCount === 0) normalizedCount = undefined;
      var directionValue = dictionary.direction;
      var direction = directionValue === undefined ? 'next' : String(directionValue);
      if (direction !== 'next' && direction !== 'prev'
          && direction !== 'nextunique' && direction !== 'prevunique') {
        throw new TypeError('The get-all direction is invalid.');
      }
      var query = dictionary.query;
      return {
        query: query,
        queryProvided: query !== undefined && query !== null,
        count: normalizedCount,
        direction: direction
      };
    }
    var legacyCount = argumentCount >= 2 && count !== undefined
      ? _zwIDBEnforceUnsignedLong(count)
      : undefined;
    return {
      query: queryOrOptions,
      queryProvided: argumentCount >= 1 && queryOrOptions !== undefined,
      count: legacyCount,
      direction: 'next'
    };
  }
  function _zwIDBRecord(key, primaryKey, value) {
    this._key = key;
    this._primaryKey = primaryKey;
    this._value = value;
  }
  Object.defineProperties(_zwIDBRecord.prototype, {
    key: {
      configurable: true,
      enumerable: true,
      get: function () { return this._key; }
    },
    primaryKey: {
      configurable: true,
      enumerable: true,
      get: function () { return this._primaryKey; }
    },
    value: {
      configurable: true,
      enumerable: true,
      get: function () { return this._value; }
    }
  });
  function _zwIDBRecordFromEntry(entry) {
    return new _zwIDBRecord(
      globalThis.structuredClone(entry.key),
      globalThis.structuredClone(entry.primaryKey),
      globalThis.structuredClone(entry.value)
    );
  }
  function _zwIDBApplyGetAllOptions(entries, options, uniqueByKey) {
    if (uniqueByKey) {
      entries = entries.filter(function (entry, index, all) {
        return index === 0 || _zwIDBCompareValues(all[index - 1].key, entry.key) !== 0;
      });
    }
    if (options.direction === 'prev' || options.direction === 'prevunique') {
      entries.reverse();
    }
    if (options.count !== undefined) entries = entries.slice(0, options.count);
    return entries;
  }
  _zwIDBStore.prototype._getAll = function (options, resultKind) {
    this._assertUsable(false);
    var query = options.query;
    if (options.queryProvided && !_zwIDBIsKeyRange(query) && !_zwIDBKey(query, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var request = new _zwIDBRequest(this);
    request.transaction = this.transaction;
    var store = this;
    var perform = function () {
      var entries = [];
      if (store.transaction && store.transaction._hostId !== null) {
        var hostRequest = {
          op: 'transaction_get_all',
          transaction: store.transaction._hostId,
          store: store.name,
          keys_only: resultKind === 'key'
        };
        if (options.queryProvided) hostRequest.query = _zwIDBQueryToWire(query);
        try {
          var hosted = _zwIDBHostCall(hostRequest);
          entries = (hosted.records || []).map(function (record) {
            var key = _zwIDBKeyFromWire(record.key);
            return {
              key: key,
              primaryKey: key,
              value: resultKind === 'key' ? undefined : _zwIDBValueFromWire(record.value)
            };
          });
        } catch (hostError) {
          _zwIDBRequestHostError(request, hostError);
          return;
        }
      } else {
        store._records.forEach(function (value, key) {
          if (!options.queryProvided || _zwIDBQueryMatches(query, key)) {
            entries.push({ key: key, primaryKey: key, value: value });
          }
        });
        entries.sort(function (a, b) { return _zwIDBCompareValues(a.key, b.key); });
      }
      entries = _zwIDBApplyGetAllOptions(entries, options, false);
      var result = entries.map(function (entry) {
        if (resultKind === 'record') return _zwIDBRecordFromEntry(entry);
        return globalThis.structuredClone(resultKind === 'key' ? entry.key : entry.value);
      });
      _zwIDBDispatch(request, 'success', result);
    };
    _zwIDBRunTransactionOperation(this.transaction, perform);
    return request;
  };
  _zwIDBStore.prototype.getAll = function (queryOrOptions, count) {
    var options = _zwIDBNormalizeGetAllOptions(queryOrOptions, count, arguments.length, false);
    return this._getAll(options, 'value');
  };
  _zwIDBStore.prototype.getAllKeys = function (queryOrOptions, count) {
    var options = _zwIDBNormalizeGetAllOptions(queryOrOptions, count, arguments.length, false);
    return this._getAll(options, 'key');
  };
  _zwIDBStore.prototype.getAllRecords = function (options) {
    options = _zwIDBNormalizeGetAllOptions(options, undefined, arguments.length, true);
    return this._getAll(options, 'record');
  };
  _zwIDBStore.prototype._assertSchemaChange = function () {
    if (this._metadata && this._metadata.deleted) {
      throw new globalThis.DOMException('The object store has been deleted.', 'InvalidStateError');
    }
    var transaction = this.transaction;
    if (!transaction || transaction.mode !== 'versionchange') {
      throw new globalThis.DOMException(
        'Indexes can only be changed during an upgrade transaction.',
        'InvalidStateError'
      );
    }
    if (!transaction._active
        || transaction._aborted
        || transaction._finished
        || transaction._committing) {
      throw new globalThis.DOMException('The transaction is inactive.', 'TransactionInactiveError');
    }
  };
  _zwIDBStore.prototype.createIndex = function (name, keyPath, opts) {
    // https://w3c.github.io/IndexedDB/#dom-idbobjectstore-createindex
    this._assertSchemaChange();
    name = String(name);
    if (Object.prototype.hasOwnProperty.call(this._indexes, name)) {
      throw new globalThis.DOMException('The index already exists.', 'ConstraintError');
    }
    keyPath = _zwIDBNormalizeKeyPath(keyPath, true);
    var unique = !!((opts || {}).unique);
    var multiEntry = !!((opts || {}).multiEntry);
    if (multiEntry && Array.isArray(keyPath)) {
      throw new globalThis.DOMException(
        'A multiEntry index cannot use a compound key path.',
        'InvalidAccessError'
      );
    }
    var metadata = {
      keyPath: keyPath,
      unique: unique,
      multiEntry: multiEntry,
      deleted: false,
      createdInUpgrade: !!(this.transaction && this.transaction.mode === 'versionchange')
    };
    this._indexes[name] = metadata;
    if (unique && this.transaction) {
      var seen = [];
      var conflict = false;
      this._records.forEach(function (value) {
        var indexKey = this._indexKey(value, keyPath);
        var keys = multiEntry && Array.isArray(indexKey) ? indexKey : [indexKey];
        keys.forEach(function (candidate) {
          if (!_zwIDBKey(candidate, [])) return;
          if (seen.some(function (existing) {
            return _zwIDBCompareValues(existing, candidate) === 0;
          })) conflict = true;
          seen.push(candidate);
        });
      }, this);
      if (conflict) {
        var transaction = this.transaction;
        setTimeout(function () {
          if (transaction._aborted || transaction._finished) return;
          transaction._requestError = new globalThis.DOMException(
            'The unique index contains duplicate values.',
            'ConstraintError'
          );
          transaction.abort();
        }, 0);
      }
    }
    var index = new _zwIDBIndex(this, name, metadata);
    this._indexInstances[name] = index;
    this._indexInstanceList.push(index);
    return index;
  };
  _zwIDBStore.prototype.deleteIndex = function (name) {
    // https://w3c.github.io/IndexedDB/#dom-idbobjectstore-deleteindex
    this._assertSchemaChange();
    name = String(name);
    var metadata = this._indexes[name];
    if (!metadata) {
      throw new globalThis.DOMException('The index does not exist.', 'NotFoundError');
    }
    metadata.deleted = true;
    delete this._indexes[name];
    delete this._indexInstances[name];
  };
  _zwIDBStore.prototype.index = function (name) {
    name = String(name);
    if (this._metadata && this._metadata.deleted) {
      throw new globalThis.DOMException('The object store has been deleted.', 'InvalidStateError');
    }
    if (this.transaction
        && (this.transaction._aborted
            || this.transaction._finished
            || this.transaction._committing)) {
      throw new globalThis.DOMException('The transaction is finished.', 'InvalidStateError');
    }
    if (this.transaction && !this.transaction._active) {
      throw new globalThis.DOMException('The transaction is inactive.', 'TransactionInactiveError');
    }
    var idx = this._indexes[name];
    if (!idx) {
      throw new globalThis.DOMException('The index does not exist.', 'NotFoundError');
    }
    if (!this._indexInstances[name]) {
      this._indexInstances[name] = new _zwIDBIndex(this, name, idx);
      this._indexInstanceList.push(this._indexInstances[name]);
    }
    return this._indexInstances[name];
  };

  function _zwIDBCursor(source, store, request, entries, direction, hostId, keyOnly) {
    this._isIDBCursor = true;
    this._source = source;
    this._direction = direction || 'next';
    this._store = store;
    this._request = request;
    this._entries = entries;
    this._position = 0;
    this._hostId = hostId === undefined ? null : hostId;
    this._keyOnly = !!keyOnly;
    this._gotValue = false;
    this._sync();
  }
  // https://w3c.github.io/IndexedDB/#idbcursor
  Object.defineProperties(_zwIDBCursor.prototype, {
    source: {
      configurable: true,
      enumerable: true,
      get: function () { return this._source; }
    },
    direction: {
      configurable: true,
      enumerable: true,
      get: function () { return this._direction; }
    },
    key: {
      configurable: true,
      enumerable: true,
      get: function () { return this._key; }
    },
    primaryKey: {
      configurable: true,
      enumerable: true,
      get: function () { return this._primaryKey; }
    },
    request: {
      configurable: true,
      enumerable: true,
      get: function () { return this._request; }
    }
  });
  function _zwIDBCursorWithValue(source, store, request, entries, direction, hostId) {
    _zwIDBCursor.call(this, source, store, request, entries, direction, hostId, false);
  }
  _zwIDBCursorWithValue.prototype = Object.create(_zwIDBCursor.prototype);
  _zwIDBCursorWithValue.prototype.constructor = _zwIDBCursorWithValue;
  Object.defineProperty(_zwIDBCursorWithValue.prototype, 'value', {
    configurable: true,
    enumerable: true,
    get: function () { return this._value; }
  });
  _zwIDBCursor.prototype._sync = function () {
    var entry = this._entries[this._position];
    this._key = entry.key;
    this._primaryKey = entry.primaryKey;
    this._value = entry.value;
  };
  _zwIDBCursor.prototype._applyPendingPosition = function () {
    if (this._pendingEntry) {
      this._entries = [this._pendingEntry];
      this._position = 0;
      this._pendingEntry = null;
    }
    this._sync();
  };
  _zwIDBCursor.prototype._assertCanIterate = function () {
    var transaction = this._store.transaction;
    if (!transaction
        || !transaction._active
        || transaction._aborted
        || transaction._finished
        || transaction._committing) {
      throw new globalThis.DOMException('The transaction is inactive.', 'TransactionInactiveError');
    }
    if ((this._store._metadata && this._store._metadata.deleted)
        || (this.source._metadata && this.source._metadata.deleted)) {
      throw new globalThis.DOMException('The cursor source has been deleted.', 'InvalidStateError');
    }
    if (!this._gotValue) {
      throw new globalThis.DOMException('The cursor is not positioned on a value.', 'InvalidStateError');
    }
  };
  _zwIDBCursor.prototype._assertCanMutate = function () {
    var transaction = this._store.transaction;
    if (!transaction
        || !transaction._active
        || transaction._aborted
        || transaction._finished
        || transaction._committing) {
      throw new globalThis.DOMException('The transaction is inactive.', 'TransactionInactiveError');
    }
    if ((this._store._metadata && this._store._metadata.deleted)
        || (this.source._metadata && this.source._metadata.deleted)) {
      throw new globalThis.DOMException('The cursor source has been deleted.', 'InvalidStateError');
    }
    if (transaction.mode === 'readonly') {
      throw new globalThis.DOMException('The transaction is read-only.', 'ReadOnlyError');
    }
    if (!this._gotValue || this._keyOnly) {
      throw new globalThis.DOMException('The cursor is not positioned on a value.', 'InvalidStateError');
    }
  };
  _zwIDBCursor.prototype.delete = function () {
    // https://w3c.github.io/IndexedDB/#dom-idbcursor-delete
    this._assertCanMutate();
    var req = new _zwIDBRequest(this);
    req.transaction = this._store.transaction;
    var cursor = this;
    var perform = function () {
      if (cursor._hostId !== null) {
        try {
          _zwIDBHostCall({
            op: 'transaction_delete',
            transaction: cursor._store.transaction._hostId,
            store: cursor._store.name,
            key: _zwIDBKeyToWire(cursor.primaryKey)
          });
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
          return;
        }
      }
      var localKey = cursor._store._recordKey(cursor.primaryKey);
      if (localKey !== undefined) cursor._store._records.delete(localKey);
      _zwIDBDispatch(req, 'success', undefined);
    };
    _zwIDBRunTransactionOperation(this._store.transaction, perform);
    return req;
  };
  _zwIDBCursor.prototype.update = function (value) {
    // https://w3c.github.io/IndexedDB/#dom-idbcursor-update
    if (arguments.length === 0) {
      throw new TypeError('IDBCursor.update requires a value.');
    }
    this._assertCanMutate();
    var storedValue = globalThis.structuredClone(value);
    var store = this._store;
    if (store.keyPath !== null) {
      var inlineKey = store._keyOf(storedValue);
      if (!_zwIDBKey(inlineKey, [])
          || _zwIDBCompareValues(inlineKey, this.primaryKey) !== 0) {
        throw new globalThis.DOMException(
          'The updated value changes the record key.',
          'DataError'
        );
      }
    }
    var req = new _zwIDBRequest(this);
    req.transaction = store.transaction;
    var cursor = this;
    var perform = function () {
      if (store._hasUniqueConflict(storedValue, cursor.primaryKey)) {
        store._constraintError(req);
        return;
      }
      if (cursor._hostId !== null) {
        var hostRequest = {
          op: 'transaction_put',
          transaction: store.transaction._hostId,
          store: store.name,
          value: _zwIDBValueToWire(storedValue),
          key: _zwIDBKeyToWire(cursor.primaryKey)
        };
        try {
          _zwIDBHostCall(hostRequest);
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
          return;
        }
      }
      var localKey = store._recordKey(cursor.primaryKey);
      store._records.set(localKey === undefined ? cursor.primaryKey : localKey, storedValue);
      _zwIDBDispatch(req, 'success', cursor.primaryKey);
    };
    _zwIDBRunTransactionOperation(store.transaction, perform);
    return req;
  };
  _zwIDBCursor.prototype.continue = function (key) {
    // https://w3c.github.io/IndexedDB/#dom-idbcursor-continue
    this._assertCanIterate();
    var keyProvided = arguments.length >= 1 && key !== undefined;
    if (keyProvided) {
      if (!_zwIDBKey(key, [])) {
        throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
      }
      var comparedToCurrent = _zwIDBCompareValues(key, this.key);
      var reverseDirection = this.direction === 'prev' || this.direction === 'prevunique';
      if (reverseDirection ? comparedToCurrent >= 0 : comparedToCurrent <= 0) {
        throw new globalThis.DOMException('The key does not move the cursor forward.', 'DataError');
      }
    }
    if (this._hostId !== null) {
      var hostRequest = {
        op: 'transaction_cursor_continue',
        transaction: this._store.transaction._hostId,
        cursor: this._hostId
      };
      if (keyProvided) hostRequest.key = _zwIDBKeyToWire(key);
      var hosted = _zwIDBHostCall(hostRequest);
      var hostedResult = null;
      if (hosted.entry) {
        this._pendingEntry = _zwIDBCursorEntryFromHost(hosted.entry, this._keyOnly);
        hostedResult = this;
      }
      this._gotValue = false;
      this._request.readyState = 'pending';
      this._request.result = undefined;
      _zwIDBDispatch(this._request, 'success', hostedResult);
      return;
    }
    var next = this._position + 1;
    if (keyProvided) {
      var reverse = this.direction === 'prev' || this.direction === 'prevunique';
      while (next < this._entries.length) {
        var compared = _zwIDBCompareValues(this._entries[next].key, key);
        if (reverse ? compared <= 0 : compared >= 0) break;
        next++;
      }
    }
    this._position = next;
    var result = null;
    if (this._position < this._entries.length) {
      result = this;
    }
    this._gotValue = false;
    this._request.readyState = 'pending';
    this._request.result = undefined;
    _zwIDBDispatch(this._request, 'success', result);
  };
  _zwIDBCursor.prototype.continuePrimaryKey = function (key, primaryKey) {
    // https://w3c.github.io/IndexedDB/#dom-idbcursor-continueprimarykey
    var transaction = this._store.transaction;
    if (transaction
        && (!transaction._active
            || transaction._aborted
            || transaction._finished
            || transaction._committing)) {
      throw new globalThis.DOMException('The transaction is inactive.', 'TransactionInactiveError');
    }
    this.source._assertUsable(false);
    if (!(this.source instanceof _zwIDBIndex)) {
      throw new globalThis.DOMException('The cursor source is not an index.', 'InvalidAccessError');
    }
    if (this.direction !== 'next' && this.direction !== 'prev') {
      throw new globalThis.DOMException(
        'The cursor direction must not be unique.',
        'InvalidAccessError'
      );
    }
    if (!this._gotValue) {
      throw new globalThis.DOMException('The cursor is not positioned on a value.', 'InvalidStateError');
    }
    if (!_zwIDBKey(key, []) || !_zwIDBKey(primaryKey, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var keyComparison = _zwIDBCompareValues(key, this.key);
    var primaryComparison = _zwIDBCompareValues(primaryKey, this.primaryKey);
    var reverse = this.direction === 'prev';
    var valid = reverse
      ? keyComparison < 0 || (keyComparison === 0 && primaryComparison < 0)
      : keyComparison > 0 || (keyComparison === 0 && primaryComparison > 0);
    if (!valid) {
      throw new globalThis.DOMException('The keys do not move the cursor forward.', 'DataError');
    }
    if (this._hostId !== null) {
      var hosted = _zwIDBHostCall({
        op: 'transaction_cursor_continue_primary_key',
        transaction: transaction._hostId,
        cursor: this._hostId,
        key: _zwIDBKeyToWire(key),
        primary_key: _zwIDBKeyToWire(primaryKey)
      });
      var hostedResult = null;
      if (hosted.entry) {
        this._pendingEntry = _zwIDBCursorEntryFromHost(hosted.entry, this._keyOnly);
        hostedResult = this;
      }
      this._gotValue = false;
      this._request.readyState = 'pending';
      this._request.result = undefined;
      _zwIDBDispatch(this._request, 'success', hostedResult);
      return;
    }
    var next = this._position + 1;
    while (next < this._entries.length) {
      var entry = this._entries[next];
      var comparedKey = _zwIDBCompareValues(entry.key, key);
      var comparedPrimary = _zwIDBCompareValues(entry.primaryKey, primaryKey);
      if (reverse
          ? comparedKey < 0 || (comparedKey === 0 && comparedPrimary <= 0)
          : comparedKey > 0 || (comparedKey === 0 && comparedPrimary >= 0)) break;
      next++;
    }
    this._position = next;
    var result = this._position < this._entries.length ? this : null;
    this._gotValue = false;
    this._request.readyState = 'pending';
    this._request.result = undefined;
    _zwIDBDispatch(this._request, 'success', result);
  };
  _zwIDBCursor.prototype.advance = function (count) {
    // https://w3c.github.io/IndexedDB/#dom-idbcursor-advance
    count = Number(count);
    if (!isFinite(count)) {
      throw new TypeError('The cursor advance count must be an unsigned long greater than zero.');
    }
    count = count < 0 ? Math.ceil(count) : Math.floor(count);
    if (count <= 0 || count > 4294967295) {
      throw new TypeError('The cursor advance count must be an unsigned long greater than zero.');
    }
    this._assertCanIterate();
    if (this._hostId !== null) {
      var hosted = _zwIDBHostCall({
        op: 'transaction_cursor_advance',
        transaction: this._store.transaction._hostId,
        cursor: this._hostId,
        count: count
      });
      var hostedResult = null;
      if (hosted.entry) {
        this._pendingEntry = _zwIDBCursorEntryFromHost(hosted.entry, this._keyOnly);
        hostedResult = this;
      }
      this._gotValue = false;
      this._request.readyState = 'pending';
      this._request.result = undefined;
      _zwIDBDispatch(this._request, 'success', hostedResult);
      return;
    }
    this._position = Math.min(this._entries.length, this._position + count);
    var result = null;
    if (this._position < this._entries.length) {
      result = this;
    }
    this._gotValue = false;
    this._request.readyState = 'pending';
    this._request.result = undefined;
    _zwIDBDispatch(this._request, 'success', result);
  };

  function _zwIDBCursorEntryFromHost(entry, keyOnly) {
    return {
      key: _zwIDBKeyFromWire(entry.key),
      primaryKey: _zwIDBKeyFromWire(entry.primaryKey),
      value: keyOnly ? undefined : _zwIDBValueFromWire(entry.value)
    };
  }

  function _zwIDBOpenStoreCursor(store, query, direction, keyOnly) {
    store._assertUsable(false);
    if (query != null && !_zwIDBIsKeyRange(query) && !_zwIDBKey(query, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    direction = direction || 'next';
    var req = new _zwIDBRequest(store);
    req.transaction = store.transaction;
    var perform = function () {
      var entries = [];
      var hosted;
      if (store.transaction && store.transaction._hostId !== null) {
        var hostRequest = {
          op: 'transaction_open_cursor',
          transaction: store.transaction._hostId,
          store: store.name,
          direction: direction,
          key_only: !!keyOnly
        };
        if (query != null) hostRequest.query = _zwIDBQueryToWire(query);
        try {
          hosted = _zwIDBHostCall(hostRequest);
          if (hosted.entry) entries.push(_zwIDBCursorEntryFromHost(hosted.entry, keyOnly));
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
          return;
        }
      } else {
        store._records.forEach(function (value, key) {
          if (query == null || _zwIDBQueryMatches(query, key)) {
            entries.push({
              key: key,
              primaryKey: key,
              value: keyOnly ? undefined : value
            });
          }
        });
        entries.sort(function (a, b) { return _zwIDBCompareValues(a.key, b.key); });
        if (direction === 'prev' || direction === 'prevunique') entries.reverse();
      }
      var hostId = hosted && hosted.cursor !== null ? hosted.cursor : undefined;
      var cursor = entries.length
        ? keyOnly
          ? new _zwIDBCursor(store, store, req, entries, direction, hostId, true)
          : new _zwIDBCursorWithValue(store, store, req, entries, direction, hostId)
        : null;
      _zwIDBDispatch(req, 'success', cursor);
    };
    _zwIDBRunTransactionOperation(store.transaction, perform);
    return req;
  }
  _zwIDBStore.prototype.openCursor = function (query, direction) {
    return _zwIDBOpenStoreCursor(this, query, direction, false);
  };
  _zwIDBStore.prototype.openKeyCursor = function (query, direction) {
    return _zwIDBOpenStoreCursor(this, query, direction, true);
  };

  function _zwIDBIndex(store, name, metadata) {
    this.objectStore = store;
    this._name = String(name);
    this.keyPath = Array.isArray(metadata.keyPath) ? metadata.keyPath.slice() : metadata.keyPath;
    this.unique = !!metadata.unique;
    this.multiEntry = !!metadata.multiEntry;
    this._metadata = metadata;
  }
  // https://w3c.github.io/IndexedDB/#dom-idbindex-name
  Object.defineProperty(_zwIDBIndex.prototype, 'name', {
    configurable: true,
    enumerable: true,
    get: function () { return this._name; },
    set: function (value) {
      var name = String(value);
      if (this._metadata.deleted) {
        throw new globalThis.DOMException('The index has been deleted.', 'InvalidStateError');
      }
      this.objectStore._assertSchemaChange();
      if (name === this._name) return;
      if (Object.prototype.hasOwnProperty.call(this.objectStore._indexes, name)) {
        throw new globalThis.DOMException('The index already exists.', 'ConstraintError');
      }
      var oldName = this._name;
      delete this.objectStore._indexes[oldName];
      this.objectStore._indexes[name] = this._metadata;
      delete this.objectStore._indexInstances[oldName];
      this.objectStore._indexInstances[name] = this;
      this._name = name;
      if (!this._metadata.createdInUpgrade) {
        this.objectStore.transaction._schemaRenames.push({
          kind: 'index',
          instance: this,
          oldName: oldName
        });
      }
    }
  });
  _zwIDBIndex.prototype._assertUsable = function () {
    if (this._metadata.deleted) {
      throw new globalThis.DOMException('The index has been deleted.', 'InvalidStateError');
    }
    this.objectStore._assertUsable(false);
  };
  _zwIDBIndex.prototype._entries = function (query, queryProvided) {
    if (this.objectStore.transaction && this.objectStore.transaction._hostId !== null) {
      var hostRequest = {
        op: 'transaction_index_get_all',
        transaction: this.objectStore.transaction._hostId,
        store: this.objectStore.name,
        index: this.name
      };
      if (queryProvided) hostRequest.query = _zwIDBQueryToWire(query);
      var hosted = _zwIDBHostCall(hostRequest);
      return (hosted.entries || []).map(function (entry) {
        return {
          key: _zwIDBKeyFromWire(entry.key),
          primaryKey: _zwIDBKeyFromWire(entry.primaryKey),
          value: _zwIDBValueFromWire(entry.value)
        };
      });
    }
    var entries = [];
    var index = this;
    this.objectStore._records.forEach(function (value, primaryKey) {
      var indexKey = index.objectStore._indexKey(value, index.keyPath);
      var indexKeys = index.multiEntry && Array.isArray(indexKey) ? indexKey : [indexKey];
      indexKeys.forEach(function (candidate) {
        if (_zwIDBKey(candidate, [])
            && (!queryProvided || _zwIDBQueryMatches(query, candidate))) {
          entries.push({ key: candidate, primaryKey: primaryKey, value: value });
        }
      });
    });
    entries.sort(function (a, b) {
      var compared = _zwIDBCompareValues(a.key, b.key);
      return compared !== 0 ? compared : _zwIDBCompareValues(a.primaryKey, b.primaryKey);
    });
    return entries;
  };
  _zwIDBIndex.prototype._query = function (key, primaryKeyOnly) {
    this._assertUsable();
    if (!_zwIDBIsKeyRange(key) && !_zwIDBKey(key, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var req = new _zwIDBRequest(this);
    req.transaction = this.objectStore.transaction;
    var index = this;
    var perform = function () {
      try {
        var entries = index._entries(key, true);
        var result;
        if (entries.length) {
          result = globalThis.structuredClone(
            primaryKeyOnly ? entries[0].primaryKey : entries[0].value
          );
        }
        _zwIDBDispatch(req, 'success', result);
      } catch (hostError) {
        _zwIDBRequestHostError(req, hostError);
      }
    };
    _zwIDBRunTransactionOperation(this.objectStore.transaction, perform);
    return req;
  };
  _zwIDBIndex.prototype.get = function (key) {
    return this._query(key, false);
  };
  _zwIDBIndex.prototype.getKey = function (key) {
    return this._query(key, true);
  };
  _zwIDBIndex.prototype.count = function (query) {
    this._assertUsable();
    var queryProvided = arguments.length >= 1 && query !== undefined;
    if (queryProvided && !_zwIDBIsKeyRange(query) && !_zwIDBKey(query, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var req = new _zwIDBRequest(this);
    req.transaction = this.objectStore.transaction;
    var index = this;
    var perform = function () {
      try {
        _zwIDBDispatch(req, 'success', index._entries(query, queryProvided).length);
      } catch (hostError) {
        _zwIDBRequestHostError(req, hostError);
      }
    };
    _zwIDBRunTransactionOperation(this.objectStore.transaction, perform);
    return req;
  };
  _zwIDBIndex.prototype._getAll = function (options, resultKind) {
    this._assertUsable();
    var query = options.query;
    if (options.queryProvided && !_zwIDBIsKeyRange(query) && !_zwIDBKey(query, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    var req = new _zwIDBRequest(this);
    req.transaction = this.objectStore.transaction;
    var index = this;
    var perform = function () {
      try {
        var entries = index._entries(query, options.queryProvided);
        var unique = options.direction === 'nextunique'
          || options.direction === 'prevunique';
        entries = _zwIDBApplyGetAllOptions(entries, options, unique);
        _zwIDBDispatch(req, 'success', entries.map(function (entry) {
          if (resultKind === 'record') return _zwIDBRecordFromEntry(entry);
          return globalThis.structuredClone(
            resultKind === 'key' ? entry.primaryKey : entry.value
          );
        }));
      } catch (hostError) {
        _zwIDBRequestHostError(req, hostError);
      }
    };
    _zwIDBRunTransactionOperation(this.objectStore.transaction, perform);
    return req;
  };
  _zwIDBIndex.prototype.getAll = function (queryOrOptions, count) {
    var options = _zwIDBNormalizeGetAllOptions(queryOrOptions, count, arguments.length, false);
    return this._getAll(options, 'value');
  };
  _zwIDBIndex.prototype.getAllKeys = function (queryOrOptions, count) {
    var options = _zwIDBNormalizeGetAllOptions(queryOrOptions, count, arguments.length, false);
    return this._getAll(options, 'key');
  };
  _zwIDBIndex.prototype.getAllRecords = function (options) {
    options = _zwIDBNormalizeGetAllOptions(options, undefined, arguments.length, true);
    return this._getAll(options, 'record');
  };
  function _zwIDBOpenIndexCursor(index, query, direction, keyOnly) {
    index._assertUsable();
    if (query != null && !_zwIDBIsKeyRange(query) && !_zwIDBKey(query, [])) {
      throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
    }
    direction = direction || 'next';
    var req = new _zwIDBRequest(index);
    req.transaction = index.objectStore.transaction;
    var perform = function () {
      var entries = [];
      var hosted;
      if (index.objectStore.transaction && index.objectStore.transaction._hostId !== null) {
        var hostRequest = {
          op: 'transaction_open_cursor',
          transaction: index.objectStore.transaction._hostId,
          store: index.objectStore.name,
          index: index.name,
          direction: direction,
          key_only: !!keyOnly
        };
        if (query != null) hostRequest.query = _zwIDBQueryToWire(query);
        try {
          hosted = _zwIDBHostCall(hostRequest);
          if (hosted.entry) entries.push(_zwIDBCursorEntryFromHost(hosted.entry, keyOnly));
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
          return;
        }
      } else {
        try {
          entries = index._entries(query, query != null);
        } catch (hostError) {
          _zwIDBRequestHostError(req, hostError);
          return;
        }
        if (direction === 'nextunique' || direction === 'prevunique') {
          entries = entries.filter(function (entry, position) {
            return position === 0 || _zwIDBCompareValues(entries[position - 1].key, entry.key) !== 0;
          });
        }
        if (direction === 'prev' || direction === 'prevunique') entries.reverse();
        if (keyOnly) {
          entries = entries.map(function (entry) {
            return { key: entry.key, primaryKey: entry.primaryKey, value: undefined };
          });
        }
      }
      var hostId = hosted && hosted.cursor !== null ? hosted.cursor : undefined;
      var cursor = entries.length
        ? keyOnly
          ? new _zwIDBCursor(index, index.objectStore, req, entries, direction, hostId, true)
          : new _zwIDBCursorWithValue(index, index.objectStore, req, entries, direction, hostId)
        : null;
      _zwIDBDispatch(req, 'success', cursor);
    };
    _zwIDBRunTransactionOperation(index.objectStore.transaction, perform);
    return req;
  }
  _zwIDBIndex.prototype.openCursor = function (query, direction) {
    return _zwIDBOpenIndexCursor(this, query, direction, false);
  };
  _zwIDBIndex.prototype.openKeyCursor = function (query, direction) {
    return _zwIDBOpenIndexCursor(this, query, direction, true);
  };

  function _zwIDBRestoreTransactionSnapshot(transaction) {
    if (!transaction._snapshot) return;
    transaction._db._state.stores = transaction._snapshot;
    transaction._db._stores = transaction._snapshot;
  }

  function _zwIDBRestoreVersionchangeInstances(transaction) {
    if (transaction.mode !== 'versionchange' || !transaction._snapshot) return;
    var stores = [];
    Object.keys(transaction._storeInstances).forEach(function (name) {
      var store = transaction._storeInstances[name];
      if (stores.indexOf(store) === -1) stores.push(store);
    });
    stores.forEach(function (store) {
      if (store._metadata.createdInUpgrade) {
        store._metadata.deleted = true;
        store._indexInstanceList.forEach(function (index) {
          index._metadata.deleted = true;
        });
        store._indexes = {};
      }
    });
    for (var i = transaction._schemaRenames.length - 1; i >= 0; i--) {
      var rename = transaction._schemaRenames[i];
      rename.instance._name = rename.oldName;
    }
    var restored = {};
    stores.forEach(function (store) {
      if (store._metadata.createdInUpgrade) return;
      var metadata = transaction._snapshot[store._name];
      if (!metadata) return;
      store._metadata = metadata;
      store._records = metadata.records;
      store._indexes = metadata.indexes;
      var indexes = {};
      store._indexInstanceList.forEach(function (index) {
        if (index._metadata.createdInUpgrade) {
          index._metadata.deleted = true;
          return;
        }
        var restoredIndex = metadata.indexes[index._name];
        if (!restoredIndex) return;
        index._metadata = restoredIndex;
        indexes[index._name] = index;
      });
      store._indexInstances = indexes;
      restored[store._name] = store;
    });
    transaction._storeInstances = restored;
    transaction._scope = Object.keys(transaction._snapshot).sort();
  }

  function _zwIDBFailHostTransaction(transaction, error) {
    transaction._hostError = error;
    transaction._aborted = true;
    _zwIDBRestoreTransactionSnapshot(transaction);
    var errorEvent = new _zwIDBEvent('error', transaction);
    errorEvent.bubbles = true;
    errorEvent.cancelable = true;
    _zwIDBEmit(transaction, 'error', errorEvent);
    _zwIDBEmit(transaction, 'abort', new _zwIDBEvent('abort', transaction));
  }

  function _zwIDBTransactionsConflict(first, second) {
    if (first.mode === 'readonly' && second.mode === 'readonly') return false;
    return first._scope.some(function (name) { return second._scope.indexOf(name) !== -1; });
  }

  function _zwIDBBeginHostTransaction(transaction, lease) {
    var request = {
      op: 'begin_transaction',
      database: transaction._db.name,
      stores: transaction._scope,
      mode: transaction.mode
    };
    if (lease !== undefined) request.lease = lease;
    var begun = _zwIDBHostCall(request);
    if (begun !== undefined) {
      transaction._hostId = begun.transaction;
      transaction._snapshot = _zwIDBCloneStores(transaction._db._stores);
    }
    transaction._started = true;
    var operations = transaction._operations.splice(0);
    operations.forEach(function (operation) { operation(); });
    _zwIDBScheduleTransactionCompletion(transaction);
  }

  function _zwIDBFailTransactionStart(transaction, error) {
    transaction._hostStartRequest = null;
    _zwIDBFailHostTransaction(transaction, error);
    transaction._finished = true;
    _zwIDBUntrackTransaction(transaction);
  }

  function _zwIDBPollTransactionStart(transaction) {
    if (transaction._aborted || transaction._finished) {
      if (transaction._hostStartRequest !== null) {
        try {
          _zwIDBHostCall({
            op: 'cancel_transaction_start',
            request: transaction._hostStartRequest
          });
        } catch (_) {}
        transaction._hostStartRequest = null;
      }
      return;
    }
    var status;
    try {
      status = _zwIDBHostCall({
        op: 'poll_transaction_start',
        request: transaction._hostStartRequest
      });
      if (status && status.ready) {
        transaction._hostStartRequest = null;
        _zwIDBBeginHostTransaction(transaction, status.lease);
        return;
      }
    } catch (error) {
      _zwIDBFailTransactionStart(transaction, error);
      return;
    }
    setTimeout(function () { _zwIDBPollTransactionStart(transaction); }, 0);
  }

  // https://w3c.github.io/IndexedDB/#transaction-scheduling
  function _zwIDBStartTransaction(transaction) {
    if (transaction._started
        || transaction._aborted
        || transaction._finished
        || transaction._hostStartRequest !== null) return;
    if (!_zwIDBUsesHostTransactionScheduling()) {
      _zwIDBBeginHostTransaction(transaction);
      return;
    }
    var status = _zwIDBHostCall({
      op: 'request_transaction_start',
      database: transaction._db.name,
      stores: transaction._scope,
      mode: transaction.mode
    });
    if (!status || status.ready) {
      _zwIDBBeginHostTransaction(transaction, status && status.lease);
      return;
    }
    transaction._hostStartRequest = status.request;
    setTimeout(function () { _zwIDBPollTransactionStart(transaction); }, 0);
  }

  function _zwIDBStartEligibleTransactions(state) {
    state.transactions.forEach(function (transaction, position) {
      if (transaction._started || transaction._aborted || transaction._finished) return;
      var blocked = state.transactions.slice(0, position).some(function (earlier) {
        return !earlier._aborted
          && !earlier._finished
          && _zwIDBTransactionsConflict(earlier, transaction);
      });
      if (!blocked) _zwIDBStartTransaction(transaction);
    });
  }

  function _zwIDBRunTransactionOperation(transaction, operation) {
    if (!transaction || transaction._started) {
      operation();
      return;
    }
    transaction._operations.push(operation);
  }

  function _zwIDBRunTransactionCompletion(transaction) {
    transaction._completionCheckScheduled = false;
    transaction._active = false;
    if (transaction._aborted || transaction._finished || transaction._deferCompletion) return;
    var position = transaction._db._state.transactions.indexOf(transaction);
    var earlierActive = transaction._db._state.transactions.slice(0, position).some(function (earlier) {
      return !earlier._aborted
        && !earlier._finished
        && _zwIDBTransactionsConflict(earlier, transaction);
    });
    if (!transaction._started || transaction._pending > 0 || earlierActive) return;
    transaction._committing = true;
    if (transaction._hostId !== null) {
      try {
        _zwIDBHostCall({ op: 'commit_transaction', transaction: transaction._hostId });
      } catch (hostError) {
        _zwIDBFailHostTransaction(transaction, hostError);
        transaction._finished = true;
        _zwIDBUntrackTransaction(transaction);
        return;
      }
      transaction._hostId = null;
    }
    transaction._finished = true;
    _zwIDBUntrackTransaction(transaction);
    _zwIDBEmit(transaction, 'complete', new _zwIDBEvent('complete', transaction));
  }

  function _zwIDBScheduleTransactionCompletion(transaction) {
    if (transaction._aborted
        || transaction._finished
        || transaction._deferCompletion
        || transaction._completionCheckScheduled) return;
    transaction._completionCheckScheduled = true;
    setTimeout(function () { _zwIDBRunTransactionCompletion(transaction); }, 0);
  }

  function _zwIDBTransaction(db, names, mode, deferCompletion, durability) {
    var storeNames = Array.isArray(names) ? names.map(String) : [String(names)];
    this._db = db;
    this.db = db;
    this.mode = mode || 'readonly';
    this.durability = durability || 'default';
    this._scope = storeNames.filter(function (name, index, all) {
      return all.indexOf(name) === index;
    }).sort();
    var transaction = this;
    this.objectStoreNames = _zwIDBStringList(function () {
      return transaction._scope.slice();
    });
    this.oncomplete = null;
    this.onerror = null;
    this.onabort = null;
    this._listeners = {};
    this._aborted = false;
    this._finished = false;
    this._committing = false;
    this._autoCommitPending = false;
    this._active = true;
    this._pending = 0;
    this._requestQueue = [];
    this._requestError = null;
    this.error = null;
    this._hostId = null;
    this._hostStartRequest = null;
    this._snapshot = null;
    this._storeInstances = {};
    this._schemaRenames = [];
    this._deferCompletion = !!deferCompletion;
    this._completionCheckScheduled = false;
    this._started = !!deferCompletion;
    this._operations = [];
    _zwIDBTransactions.push(this);
    db._transactions.push(this);
    db._state.transactions.push(this);
    if (!deferCompletion && this.mode !== 'versionchange') {
      _zwIDBStartEligibleTransactions(db._state);
    }
    _zwIDBScheduleTransactionCompletion(this);
  }
  _zwIDBTransaction.prototype.addEventListener = _zwIDBRequest.prototype.addEventListener;
  _zwIDBTransaction.prototype.removeEventListener = _zwIDBRequest.prototype.removeEventListener;
  _zwIDBTransaction.prototype.dispatchEvent = _zwIDBRequest.prototype.dispatchEvent;
  _zwIDBTransaction.prototype.objectStore = function (name) {
    if (this._aborted || this._finished || this._committing) {
      throw new globalThis.DOMException('The transaction is finished.', 'InvalidStateError');
    }
    name = String(name);
    if (this._scope.indexOf(name) === -1) {
      throw new globalThis.DOMException('The object store is not in this transaction.', 'NotFoundError');
    }
    var s = this._db._stores[name];
    if (!s) {
      throw new globalThis.DOMException('The object store is not in this transaction.', 'NotFoundError');
    }
    if (!this._storeInstances[name]) {
      this._storeInstances[name] =
        new _zwIDBStore(this._db, name, s.keyPath, s.autoIncrement, s.records, s.indexes, this, s);
    }
    return this._storeInstances[name];
  };
  _zwIDBTransaction.prototype.abort = function () {
    if (this._aborted || this._finished || this._committing || this._autoCommitPending) {
      throw new globalThis.DOMException('The transaction is finished.', 'InvalidStateError');
    }
    if (this._hostStartRequest !== null) {
      try {
        _zwIDBHostCall({
          op: 'cancel_transaction_start',
          request: this._hostStartRequest
        });
      } catch (_) {}
      this._hostStartRequest = null;
    }
    if (this._hostId !== null) {
      try {
        _zwIDBHostCall({ op: 'abort_transaction', transaction: this._hostId });
      } catch (_) {}
      this._hostId = null;
    }
    this._aborted = true;
    this.error = this._requestError
      || new globalThis.DOMException('The transaction was aborted.', 'AbortError');
    this._requestError = null;
    this._requestQueue.forEach(function (dispatch) {
      if (dispatch.settled || dispatch.firing) return;
      var requestError = new globalThis.DOMException('The transaction was aborted.', 'AbortError');
      var requestEvent = new _zwIDBEvent('error', dispatch.request);
      requestEvent.bubbles = true;
      requestEvent.cancelable = true;
      requestEvent._requestError = requestError;
      dispatch.type = 'error';
      dispatch.result = undefined;
      dispatch.event = requestEvent;
    });
    _zwIDBRestoreVersionchangeInstances(this);
    _zwIDBRestoreTransactionSnapshot(this);
    if (this.mode === 'versionchange') {
      Object.keys(this._db._stores).forEach(function (storeName) {
        var store = this._db._stores[storeName];
        if (store.createdInUpgrade) store.deleted = true;
        Object.keys(store.indexes).forEach(function (indexName) {
          var index = store.indexes[indexName];
          if (index.createdInUpgrade) index.deleted = true;
        });
      }, this);
    } else {
      this._finished = true;
      _zwIDBUntrackTransaction(this);
      var transaction = this;
      var fireAbort = function () {
        _zwIDBEmitTransactionEvent(transaction, 'abort', true);
      };
      if (typeof queueMicrotask === 'function') queueMicrotask(fireAbort);
      else fireAbort();
    }
  };
  _zwIDBTransaction.prototype.commit = function () {
    if (!this._active || this._aborted || this._finished || this._committing) {
      throw new globalThis.DOMException('The transaction is inactive.', 'InvalidStateError');
    }
    this._committing = true;
    _zwIDBScheduleTransactionCompletion(this);
  };

  function _zwIDBDatabase(name, state) {
    _zwIDBNextConnectionId++;
    this._hostConnectionId = _zwIDBNextConnectionId;
    this._hostConnectionRegistered = false;
    this.name = name;
    this.version = state.version;
    this._state = state;
    this._stores = state.stores; // name → {keyPath, autoIncrement, records: Map, indexes: {}}
    this._transactions = [];
    this._closed = false;
    this._closedStoreNames = null;
    this.onversionchange = null;
    this.onabort = null;
    this.onerror = null;
    this._listeners = {};
    var self = this;
    this.objectStoreNames = _zwIDBStringList(function () {
      return self._closedStoreNames || Object.keys(self._stores);
    });
  }
  _zwIDBDatabase.prototype.addEventListener = _zwIDBRequest.prototype.addEventListener;
  _zwIDBDatabase.prototype.removeEventListener = _zwIDBRequest.prototype.removeEventListener;
  _zwIDBDatabase.prototype.dispatchEvent = _zwIDBRequest.prototype.dispatchEvent;
  _zwIDBDatabase.prototype.createObjectStore = function (name, opts) {
    // https://w3c.github.io/IndexedDB/#dom-idbdatabase-createobjectstore
    name = String(name);
    var transaction = this._upgradeTransaction;
    if (!transaction) {
      throw new globalThis.DOMException(
        'Object stores can only be created during an upgrade transaction.',
        'InvalidStateError'
      );
    }
    if (!transaction._active
        || transaction._aborted
        || transaction._finished
        || transaction._committing) {
      throw new globalThis.DOMException('The transaction is inactive.', 'TransactionInactiveError');
    }
    opts = opts == null ? {} : Object(opts);
    var keyPath = _zwIDBNormalizeKeyPath(
      opts.keyPath,
      Object.prototype.hasOwnProperty.call(opts, 'keyPath')
    );
    if (Object.prototype.hasOwnProperty.call(this._stores, name)) {
      throw new globalThis.DOMException('The object store already exists.', 'ConstraintError');
    }
    var autoIncrement = !!opts.autoIncrement;
    if (autoIncrement && (keyPath === '' || Array.isArray(keyPath))) {
      throw new globalThis.DOMException(
        'autoIncrement cannot be combined with this key path.',
        'InvalidAccessError'
      );
    }
    var s = {
      keyPath: keyPath,
      autoIncrement: autoIncrement,
      records: new Map(),
      indexes: {},
      nextKey: 1,
      deleted: false,
      createdInUpgrade: true
    };
    this._stores[name] = s;
    transaction._scope.push(name);
    transaction._scope = transaction._scope.filter(function (entry, index, all) {
      return all.indexOf(entry) === index;
    }).sort();
    var store = new _zwIDBStore(
      this,
      name,
      s.keyPath,
      s.autoIncrement,
      s.records,
      s.indexes,
      this._upgradeTransaction,
      s
    );
    transaction._storeInstances[name] = store;
    return store;
  };
  _zwIDBDatabase.prototype.deleteObjectStore = function (name) {
    // https://w3c.github.io/IndexedDB/#dom-idbdatabase-deleteobjectstore
    name = String(name);
    var transaction = this._upgradeTransaction;
    if (!transaction) {
      throw new globalThis.DOMException(
        'Object stores can only be deleted during an upgrade transaction.',
        'InvalidStateError'
      );
    }
    if (!transaction._active
        || transaction._aborted
        || transaction._finished
        || transaction._committing) {
      throw new globalThis.DOMException('The transaction is inactive.', 'TransactionInactiveError');
    }
    var store = this._stores[name];
    if (!store) {
      throw new globalThis.DOMException('The object store does not exist.', 'NotFoundError');
    }
    store.deleted = true;
    Object.keys(store.indexes).forEach(function (indexName) {
      store.indexes[indexName].deleted = true;
    });
    var storeInstance = transaction._storeInstances[name];
    if (storeInstance) {
      storeInstance._indexes = {};
      storeInstance._indexInstances = {};
    }
    delete this._stores[name];
    transaction._scope = transaction._scope.filter(function (entry) { return entry !== name; });
  };
  _zwIDBDatabase.prototype.transaction = function (names, mode, options) {
    // https://w3c.github.io/IndexedDB/#dom-idbdatabase-transaction
    if (this._closed) {
      throw new globalThis.DOMException('The database connection is closed.', 'InvalidStateError');
    }
    if (this._upgradeTransaction) {
      throw new globalThis.DOMException('A version change transaction is running.', 'InvalidStateError');
    }
    var storeNames = Array.isArray(names) ? names.map(String) : [String(names)];
    if (storeNames.length === 0) {
      throw new globalThis.DOMException('The transaction scope is empty.', 'InvalidAccessError');
    }
    storeNames = storeNames.filter(function (name, index, all) {
      return all.indexOf(name) === index;
    }).sort();
    for (var i = 0; i < storeNames.length; i++) {
      if (!Object.prototype.hasOwnProperty.call(this._stores, storeNames[i])) {
        throw new globalThis.DOMException('The object store does not exist.', 'NotFoundError');
      }
    }
    mode = mode === undefined ? 'readonly' : String(mode);
    if (mode !== 'readonly' && mode !== 'readwrite') {
      throw new TypeError('The transaction mode is invalid.');
    }
    options = options === undefined ? {} : Object(options);
    var durability = options.durability === undefined ? 'default' : String(options.durability);
    if (durability !== 'default' && durability !== 'strict' && durability !== 'relaxed') {
      throw new TypeError('The transaction durability is invalid.');
    }
    return new _zwIDBTransaction(this, storeNames, mode, false, durability);
  };
  _zwIDBDatabase.prototype.close = function () {
    if (this._closed) return;
    this._closedStoreNames = Object.keys(this._stores).sort();
    this._closed = true;
    var index = this._state.connections.indexOf(this);
    if (index !== -1) this._state.connections.splice(index, 1);
    if (this._hostConnectionRegistered) {
      delete _zwIDBHostConnections[this._hostConnectionId];
      _zwIDBHostCall({
        op: 'close_connection',
        connection: this._hostConnectionId
      });
      this._hostConnectionRegistered = false;
    }
    var queue = _zwIDBConnectionQueues[this.name];
    if (queue && typeof queue.retry === 'function') queue.retry();
  };

  function _zwIDBVersionEvent(type, target, oldVersion, newVersion) {
    var event = new IDBVersionChangeEvent(type, { oldVersion: oldVersion, newVersion: newVersion });
    event.target = target;
    event.currentTarget = target;
    return event;
  }

  function _zwIDBNotifyConnections(state, oldVersion, newVersion) {
    state.connections.slice().forEach(function (connection) {
      if (connection._closed) return;
      var fire = function () {
        _zwIDBEmit(connection, 'versionchange',
          _zwIDBVersionEvent('versionchange', connection, oldVersion, newVersion));
      };
      if (typeof queueMicrotask === 'function') queueMicrotask(fire);
      else fire();
    });
  }

  function _zwIDBWaitForConnections(req, state, oldVersion, newVersion, queue, proceed) {
    var remaining = function () {
      return state.connections.some(function (connection) { return !connection._closed; });
    };
    if (!remaining()) {
      proceed();
      return;
    }
    _zwIDBNotifyConnections(state, oldVersion, newVersion);
    setTimeout(function () {
      if (!remaining()) {
        proceed();
        return;
      }
      _zwIDBEmit(
        req,
        'blocked',
        _zwIDBVersionEvent('blocked', req, oldVersion, newVersion)
      );
      if (!remaining()) {
        proceed();
        return;
      }
      queue.retry = function () {
        if (!remaining()) {
          queue.retry = null;
          proceed();
        }
      };
    }, 0);
  }

  function _zwIDBCloneStores(stores) {
    var cloned = {};
    Object.keys(stores).forEach(function (name) {
      var source = stores[name];
      var records = new Map();
      source.records.forEach(function (value, key) { records.set(key, value); });
      var indexes = {};
      Object.keys(source.indexes).forEach(function (indexName) {
        var index = source.indexes[indexName];
        indexes[indexName] = {
          keyPath: index.keyPath,
          unique: !!index.unique,
          multiEntry: !!index.multiEntry,
          deleted: false,
          createdInUpgrade: false
        };
      });
      cloned[name] = {
        keyPath: source.keyPath,
        autoIncrement: source.autoIncrement,
        records: records,
        indexes: indexes,
        nextKey: source.nextKey || 1,
        deleted: false,
        createdInUpgrade: false
      };
    });
    return cloned;
  }

  function _zwIDBSeedHostRecords(db, state) {
    if (typeof globalThis.__zw_idb !== 'function') return;
    var storeNames = Object.keys(state.stores);
    if (!storeNames.length) return;
    var beginRequest = {
      op: 'begin_transaction',
      database: db.name,
      stores: storeNames,
      mode: 'readwrite'
    };
    if (_zwIDBUsesHostTransactionScheduling()) {
      var status = _zwIDBHostCall({
        op: 'request_transaction_start',
        database: db.name,
        stores: storeNames,
        mode: 'readwrite'
      });
      if (!status || !status.ready) {
        if (status && status.request !== undefined) {
          try {
            _zwIDBHostCall({
              op: 'cancel_transaction_start',
              request: status.request
            });
          } catch (_) {}
        }
        throw new globalThis.DOMException(
          'The IndexedDB upgrade seed transaction could not start.',
          'InvalidStateError'
        );
      }
      beginRequest.lease = status.lease;
    }
    var begun = _zwIDBHostCall(beginRequest);
    if (begun === undefined) return;
    try {
      storeNames.forEach(function (storeName) {
        state.stores[storeName].records.forEach(function (value, key) {
          _zwIDBHostCall({
            op: 'transaction_put',
            transaction: begun.transaction,
            store: storeName,
            value: _zwIDBValueToWire(value),
            key: _zwIDBKeyToWire(key)
          });
        });
      });
      _zwIDBHostCall({ op: 'commit_transaction', transaction: begun.transaction });
    } catch (error) {
      try {
        _zwIDBHostCall({ op: 'abort_transaction', transaction: begun.transaction });
      } catch (_) {}
      throw error;
    }
  }

  function _zwIDBFinishUpgrade(req, db, transaction, state, snapshot, created, done) {
    if (transaction._pending > 0) {
      var retry = function () {
        _zwIDBFinishUpgrade(req, db, transaction, state, snapshot, created, done);
      };
      if (typeof setTimeout === 'function') setTimeout(retry, 0);
      else retry();
      return;
    }
    if (!transaction._aborted) {
      try {
        _zwIDBHostCall(_zwIDBSchemaForHost(db.name, state));
        _zwIDBSeedHostRecords(db, state);
      } catch (hostError) {
        transaction._aborted = true;
        transaction._hostError = hostError;
      }
    }
    db._upgradeTransaction = null;
    transaction._finished = true;
    _zwIDBUntrackTransaction(transaction);
    if (transaction._aborted) {
      state.version = snapshot.version;
      state.stores = snapshot.stores;
      db.version = snapshot.version;
      db._stores = state.stores;
      var connectionIndex = state.connections.indexOf(db);
      if (connectionIndex !== -1) state.connections.splice(connectionIndex, 1);
      if (created) delete _idb_databases[db.name];
      _zwIDBEmitTransactionEvent(transaction, 'abort', true);
      req.result = undefined;
      // https://w3c.github.io/IndexedDB/#abort-a-transaction
      req.error = new globalThis.DOMException(
        'The version change transaction was aborted.',
        'AbortError'
      );
      req.transaction = null;
      var errorEvent = new _zwIDBEvent('error', req);
      errorEvent.bubbles = true;
      errorEvent.cancelable = true;
      _zwIDBEmit(req, 'error', errorEvent);
      done();
      return;
    }
    Object.keys(state.stores).forEach(function (storeName) {
      var store = state.stores[storeName];
      store.createdInUpgrade = false;
      Object.keys(store.indexes).forEach(function (indexName) {
        store.indexes[indexName].createdInUpgrade = false;
      });
    });
    _zwIDBEmit(transaction, 'complete', new _zwIDBEvent('complete', transaction));
    req.transaction = null;
    if (db._closed) {
      req.readyState = 'done';
      req.result = undefined;
      req.error = new globalThis.DOMException(
        'The connection was closed during the upgrade.',
        'AbortError'
      );
      var closeErrorEvent = new _zwIDBEvent('error', req);
      closeErrorEvent.bubbles = true;
      closeErrorEvent.cancelable = true;
      _zwIDBEmit(req, 'error', closeErrorEvent);
      done();
      return;
    }
    _zwIDBRegisterHostConnection(db);
    var successEvent = new _zwIDBEvent('success', req);
    req.readyState = 'done';
    req.result = db;
    _zwIDBEmit(req, 'success', successEvent);
    done();
  }

  var _zwIDBTrackedProxies = typeof WeakSet !== 'undefined' ? new WeakSet() : null;
  var _zwIDBNativeProxy = globalThis.Proxy;
  if (_zwIDBTrackedProxies && typeof _zwIDBNativeProxy === 'function') {
    var _zwIDBTrackingProxy = function Proxy(target, handler) {
      if (!(this instanceof _zwIDBTrackingProxy)) {
        throw new TypeError('Constructor Proxy requires new');
      }
      var proxy = new _zwIDBNativeProxy(target, handler);
      _zwIDBTrackedProxies.add(proxy);
      return proxy;
    };
    _zwIDBTrackingProxy.revocable = function (target, handler) {
      var record = _zwIDBNativeProxy.revocable(target, handler);
      _zwIDBTrackedProxies.add(record.proxy);
      return record;
    };
    globalThis.Proxy = _zwIDBTrackingProxy;
  }

  // https://w3c.github.io/IndexedDB/#compare-two-keys
  // Key type order: Number < Date < String < Binary < Array.
  function _zwIDBKey(value, seen) {
    if (typeof value === 'number') {
      return value === value ? { rank: 1, value: value } : null;
    }
    if (value instanceof Date) {
      var time = value.getTime();
      return time === time ? { rank: 2, value: time } : null;
    }
    if (typeof value === 'string') return { rank: 3, value: value };
    var binary = _zwIDBBinaryKeyBytes(value);
    if (binary !== undefined) return binary === null ? null : { rank: 4, value: binary };
    if (Array.isArray(value)) {
      if (_zwIDBTrackedProxies && _zwIDBTrackedProxies.has(value)) return null;
      if (seen.indexOf(value) !== -1) return null;
      seen.push(value);
      var entries = [];
      for (var i = 0; i < value.length; i++) {
        if (!Object.prototype.hasOwnProperty.call(value, i)) {
          seen.pop();
          return null;
        }
        var entry = _zwIDBKey(value[i], seen);
        if (!entry) {
          seen.pop();
          return null;
        }
        entries.push(entry);
      }
      seen.pop();
      return { rank: 5, value: entries };
    }
    return null;
  }

  function _zwIDBCompareKeys(a, b) {
    if (a.rank !== b.rank) return a.rank < b.rank ? -1 : 1;
    if (a.rank <= 3) return a.value < b.value ? -1 : (a.value > b.value ? 1 : 0);
    var limit = Math.min(a.value.length, b.value.length);
    for (var i = 0; i < limit; i++) {
      var av = a.value[i];
      var bv = b.value[i];
      var compared = a.rank === 5 ? _zwIDBCompareKeys(av, bv) : (av < bv ? -1 : (av > bv ? 1 : 0));
      if (compared !== 0) return compared;
    }
    return a.value.length < b.value.length ? -1 : (a.value.length > b.value.length ? 1 : 0);
  }

  function _zwIDBCompareValues(a, b) {
    var first = _zwIDBKey(a, []);
    var second = _zwIDBKey(b, []);
    if (!first || !second) return 0;
    return _zwIDBCompareKeys(first, second);
  }

  function _zwIDBIsKeyRange(value) {
    return !!(value && value._zwIDBKeyRange === true);
  }

  function _zwIDBQueryMatches(query, key) {
    return _zwIDBIsKeyRange(query) ? query.includes(key) : _zwIDBCompareValues(query, key) === 0;
  }

  function _zwIDBKeyRange(lower, upper, lowerOpen, upperOpen) {
    this.lower = lower;
    this.upper = upper;
    this.lowerOpen = !!lowerOpen;
    this.upperOpen = !!upperOpen;
    this._zwIDBKeyRange = true;
  }
  _zwIDBKeyRange.prototype.includes = function (key) {
    if (arguments.length < 1) throw new TypeError('IDBKeyRange.includes requires a key');
    key = _zwIDBConvertKey(key);
    if (this.lower !== undefined) {
      var lower = _zwIDBCompareValues(key, this.lower);
      if (this.lowerOpen ? lower <= 0 : lower < 0) return false;
    }
    if (this.upper !== undefined) {
      var upper = _zwIDBCompareValues(key, this.upper);
      if (this.upperOpen ? upper >= 0 : upper > 0) return false;
    }
    return true;
  };
  _zwIDBKeyRange.bound = function (lower, upper, lowerOpen, upperOpen) {
    if (arguments.length < 2) throw new TypeError('IDBKeyRange.bound requires two keys');
    lower = _zwIDBConvertKey(lower);
    upper = _zwIDBConvertKey(upper);
    var compared = _zwIDBCompareValues(lower, upper);
    if (compared > 0 || (compared === 0 && (lowerOpen || upperOpen))) {
      throw new globalThis.DOMException('The key range is empty.', 'DataError');
    }
    return new _zwIDBKeyRange(lower, upper, lowerOpen, upperOpen);
  };
  _zwIDBKeyRange.only = function (value) {
    if (arguments.length < 1) throw new TypeError('IDBKeyRange.only requires a key');
    value = _zwIDBConvertKey(value);
    return new _zwIDBKeyRange(value, value, false, false);
  };
  _zwIDBKeyRange.lowerBound = function (lower, open) {
    if (arguments.length < 1) throw new TypeError('IDBKeyRange.lowerBound requires a key');
    return new _zwIDBKeyRange(_zwIDBConvertKey(lower), undefined, open, true);
  };
  _zwIDBKeyRange.upperBound = function (upper, open) {
    if (arguments.length < 1) throw new TypeError('IDBKeyRange.upperBound requires a key');
    return new _zwIDBKeyRange(undefined, _zwIDBConvertKey(upper), true, open);
  };

  // https://w3c.github.io/IndexedDB/#dom-idbfactory-open
  // WebIDL [EnforceRange] unsigned long long, additionally restricted to JS safe integers.
  function _zwIDBOpenVersion(value, supplied) {
    if (!supplied || value === undefined) return undefined;
    var number = Number(value);
    if (!isFinite(number)) throw new TypeError('IDBFactory.open version is outside the accepted range');
    number = number < 0 ? Math.ceil(number) : Math.floor(number);
    if (number <= 0 || number > Number.MAX_SAFE_INTEGER) {
      throw new TypeError('IDBFactory.open version is outside the accepted range');
    }
    return number;
  }

  globalThis.indexedDB = {
    // open(name, version)：建/取 db，异步派发 onupgradeneeded（version change，建 store 窗口）→ onsuccess。
    open: function (name, version) {
      name = String(name);
      version = _zwIDBOpenVersion(version, arguments.length >= 2);
      var req = new _zwIDBOpenRequest();
      _zwIDBEnqueueConnectionRequest(name, function (done, queue) {
        var state = _idb_databases[name];
        if (state
            && !state.connections.some(function (connection) { return !connection._closed; })
            && _zwIDBUsesHostConnections()
            && typeof globalThis.__zw_idb === 'function') {
          try {
            var refreshed = _zwIDBHostCall({ op: 'inspect', name: name });
            if (refreshed && refreshed.database) {
              state = _zwIDBStateFromHost(refreshed.database);
              _idb_databases[name] = state;
            } else {
              state = undefined;
              delete _idb_databases[name];
            }
          } catch (refreshError) {
            req.error = refreshError;
            var refreshErrorEvent = new _zwIDBEvent('error', req);
            refreshErrorEvent.bubbles = true;
            refreshErrorEvent.cancelable = true;
            _zwIDBDispatch(req, 'error', undefined, refreshErrorEvent);
            setTimeout(done, 0);
            return;
          }
        }
        if (!state) {
          try {
            var inspected = _zwIDBHostCall({ op: 'inspect', name: name });
            if (inspected !== undefined && inspected.database) {
              state = _zwIDBStateFromHost(inspected.database);
              _idb_databases[name] = state;
            }
          } catch (hostError) {
            req.error = hostError;
            var hostErrorEvent = new _zwIDBEvent('error', req);
            hostErrorEvent.bubbles = true;
            hostErrorEvent.cancelable = true;
            _zwIDBDispatch(req, 'error', undefined, hostErrorEvent);
            setTimeout(done, 0);
            return;
          }
        }
        var created = !state;
        var oldVersion = state ? state.version : 0;
        var requestedVersion = version === undefined ? (oldVersion || 1) : version;
        if (oldVersion > 0 && requestedVersion < oldVersion) {
          req.error = new globalThis.DOMException(
            'The requested version is lower than the current version.',
            'VersionError'
          );
          var errorEvent = new _zwIDBEvent('error', req);
          errorEvent.bubbles = true;
          errorEvent.cancelable = true;
          _zwIDBDispatch(req, 'error', undefined, errorEvent);
          setTimeout(done, 0);
          return;
        }
        if (!state) {
          state = { version: 0, stores: {}, connections: [], transactions: [] };
          _idb_databases[name] = state;
        }
        var needsUpgrade = requestedVersion > oldVersion;
        var openConnection = function () {
          var snapshot = needsUpgrade ? {
            version: oldVersion,
            stores: _zwIDBCloneStores(state.stores)
          } : null;
          if (needsUpgrade) state.version = requestedVersion;
          var db = new _zwIDBDatabase(name, state);
          state.connections.push(db);
          if (needsUpgrade) {
            var transaction = new _zwIDBTransaction(
              db,
              Object.keys(state.stores),
              'versionchange',
              true
            );
            transaction._snapshot = snapshot.stores;
            db._upgradeTransaction = transaction;
            req.transaction = transaction;
            var upgrade = function () {
              transaction._active = true;
              req.readyState = 'done';
              req.result = db;
              var upgradeException = _zwIDBEmit(
                req,
                'upgradeneeded',
                _zwIDBVersionEvent('upgradeneeded', req, oldVersion, requestedVersion)
              );
              if (upgradeException && !transaction._aborted) {
                transaction._requestError = new globalThis.DOMException(
                  'An upgradeneeded event listener threw.',
                  'AbortError'
                );
                transaction.abort();
              }
              var finish = function () {
                _zwIDBFinishUpgrade(
                  req,
                  db,
                  transaction,
                  state,
                  snapshot,
                  created,
                  done
                );
              };
              if (typeof setTimeout === 'function') setTimeout(finish, 0);
              else finish();
            };
            if (typeof queueMicrotask === 'function') queueMicrotask(upgrade);
            else upgrade();
            return;
          }
          var success = function () {
            var ev = new _zwIDBEvent('success', req);
            req.readyState = 'done';
            req.result = db;
            _zwIDBRegisterHostConnection(db);
            _zwIDBEmit(req, 'success', ev);
            done();
          };
          if (typeof queueMicrotask === 'function') queueMicrotask(success);
          else success();
        };
        if (needsUpgrade && oldVersion > 0) {
          if (_zwIDBUsesHostConnections()) {
            _zwIDBWaitForHostConnections(req, name, requestedVersion, openConnection);
          } else {
            _zwIDBWaitForConnections(
              req,
              state,
              oldVersion,
              requestedVersion,
              queue,
              openConnection
            );
          }
        } else {
          openConnection();
        }
      });
      return req;
    },
    deleteDatabase: function (name) {
      name = String(name);
      var req = new _zwIDBOpenRequest();
      _zwIDBEnqueueConnectionRequest(name, function (done, queue) {
        var state = _idb_databases[name];
        var oldVersion = state ? state.version : 0;
        var performDeletion = function () {
          try {
            var hostDeletion = _zwIDBHostCall({ op: 'delete_database', name: name });
            if (!state && hostDeletion) {
              oldVersion = Number(hostDeletion.oldVersion || 0);
            }
          } catch (hostError) {
            req.error = hostError;
            var hostErrorEvent = new _zwIDBEvent('error', req);
            hostErrorEvent.bubbles = true;
            hostErrorEvent.cancelable = true;
            _zwIDBDispatch(req, 'error', undefined, hostErrorEvent);
            setTimeout(done, 0);
            return;
          }
          delete _idb_databases[name];
          setTimeout(function () {
            req.readyState = 'done';
            req.result = undefined;
            _zwIDBEmit(
              req,
              'success',
              _zwIDBVersionEvent('success', req, oldVersion, null)
            );
            done();
          }, 0);
        };
        // https://w3c.github.io/IndexedDB/#deleting-a-database——删除前置步骤失败（host 不可达时
        // capabilities 探测同步抛 DOMException）→ error 事件送达 request，而非异常逃逸致请求无事件。
        try {
          if (_zwIDBUsesHostConnections()) {
            _zwIDBWaitForHostConnections(req, name, null, function (hostOldVersion) {
              oldVersion = hostOldVersion;
              performDeletion();
            });
          } else if (state) {
            _zwIDBWaitForConnections(
              req,
              state,
              oldVersion,
              null,
              queue,
              performDeletion
            );
          } else {
            performDeletion();
          }
        } catch (setupError) {
          req.error = setupError;
          var setupErrorEvent = new _zwIDBEvent('error', req);
          setupErrorEvent.bubbles = true;
          setupErrorEvent.cancelable = true;
          _zwIDBDispatch(req, 'error', undefined, setupErrorEvent);
          setTimeout(done, 0);
        }
      });
      return req;
    },
    databases: function () {
      try {
        var hosted = _zwIDBHostCall({ op: 'databases' });
        if (hosted !== undefined) return Promise.resolve(hosted.databases || []);
        return Promise.resolve(Object.keys(_idb_databases).map(function (n) {
          return { name: n, version: _idb_databases[n].version };
        }));
      } catch (hostError) {
        return Promise.reject(hostError);
      }
    },
    cmp: function (a, b) {
      if (arguments.length < 2) throw new TypeError('IDBFactory.cmp requires two keys');
      var first = _zwIDBKey(a, []);
      if (!first) {
        throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
      }
      var second = _zwIDBKey(b, []);
      if (!second) throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
      return _zwIDBCompareKeys(first, second);
    },
  };
  // IDB 构造器占位（feature-detection / instanceof 用，rare）。
  globalThis.IDBKeyRange = _zwIDBKeyRange;
  globalThis.IDBRequest = _zwIDBRequest;
  globalThis.IDBOpenDBRequest = _zwIDBOpenRequest;
  globalThis.IDBCursor = _zwIDBCursor;
  globalThis.IDBCursorWithValue = _zwIDBCursorWithValue;
  globalThis.IDBRecord = _zwIDBRecord;
  if (typeof Symbol !== 'undefined' && Symbol.toStringTag) {
    Object.defineProperty(
      _zwIDBRequest.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBRequest' }
    );
    Object.defineProperty(
      _zwIDBOpenRequest.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBOpenDBRequest' }
    );
    Object.defineProperty(
      _zwIDBCursor.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBCursor' }
    );
    Object.defineProperty(
      _zwIDBCursorWithValue.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBCursorWithValue' }
    );
    Object.defineProperty(
      _zwIDBStore.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBObjectStore' }
    );
    Object.defineProperty(
      _zwIDBIndex.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBIndex' }
    );
    Object.defineProperty(
      _zwIDBRecord.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBRecord' }
    );
    Object.defineProperty(
      _zwIDBDatabase.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBDatabase' }
    );
    Object.defineProperty(
      _zwIDBTransaction.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBTransaction' }
    );
  }
  globalThis.IDBDatabase = _zwIDBDatabase;
  globalThis.IDBObjectStore = _zwIDBStore;
  globalThis.IDBTransaction = _zwIDBTransaction;
  globalThis.IDBIndex = _zwIDBIndex;
  globalThis.IDBFactory = globalThis.IDBFactory || function IDBFactory() {};
  if (Object.getPrototypeOf(globalThis.indexedDB) !== globalThis.IDBFactory.prototype) {
    Object.setPrototypeOf(globalThis.indexedDB, globalThis.IDBFactory.prototype);
  }

  globalThis.XMLHttpRequest = function() {
    var self = this;
    self.readyState = 0;
