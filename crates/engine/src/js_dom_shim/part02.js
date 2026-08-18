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
      throw (this._reason instanceof DOMException)
        ? this._reason
        : new DOMException('signal is aborted without reason', 'AbortError');
    }
  };
  // 统一 abort 逻辑（controller.abort 与 AbortSignal.abort 共用）。
  function _zw_abort_signal(signal, reason) {
    if (signal._aborted) return; // 重复 abort 静默 no-op（spec）
    signal._aborted = true;
    signal._reason = (typeof reason === 'undefined')
      ? new DOMException('signal is aborted without reason', 'AbortError')
      : reason;
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
      setTimeout(function () { _zw_abort_signal(s, undefined); }, Number(ms) || 0);
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

  // TextEncoder/TextDecoder——UTF-8 编解码（fetch body / 字符串↔字节互转高频）。纯 JS UTF-8
  //（BMP + astral 经代理对；fatal=false 容错，非法序列替 U+FFFD）。仅支持 UTF-8（最通用，
  // spec TextEncoder 恒 utf-8；TextDecoder 标签忽略恒按 utf-8 解，非 utf-8 label 为已知限制）。
  function _zw_utf8_encode(str) {
    str = String(str);
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
      } else {
        bytes.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
      }
    }
    return bytes;
  }
  // R3012：UTF-8 流式解码——返回 { s: 已解码串, tail: 末尾不完整序列字节（待下块前缀拼接）}。
  // carry = 上一块的不完整尾部（前缀）。多字节序列跨 chunk 边界时，末尾不完整字节入 tail，下块 carry 拼接后
  // 补全解码——不再读越界（旧 _zw_utf8_decode 对 truncated 序列读 undefined 字节产垃圾）。
  // valid 完整输入的解码逻辑与旧版逐字节一致（同 U+FFFD 容错 0x80-0xc1 非法前导/连续字节）。
  function _zw_utf8_decode_stream(bytes, carry) {
    var src = [];
    if (carry && carry.length) { for (var c = 0; c < carry.length; c++) src.push(carry[c]); }
    if (bytes) { for (var b = 0; b < bytes.length; b++) src.push(bytes[b]); }
    var s = '';
    var i = 0;
    var n = src.length;
    while (i < n) {
      var b = src[i];
      if (b < 0x80) { s += String.fromCharCode(b); i += 1; }
      else if (b < 0xc2) { s += '�'; i += 1; } // 非法前导字节 / 连续字节 → U+FFFD
      else if (b < 0xe0) { // 2 字节
        if (i + 1 >= n) break; // 不完整 → 缓存尾部
        s += String.fromCharCode(((b & 0x1f) << 6) | (src[i + 1] & 0x3f)); i += 2;
      } else if (b < 0xf0) { // 3 字节
        if (i + 2 >= n) break;
        s += String.fromCharCode(((b & 0x0f) << 12) | ((src[i + 1] & 0x3f) << 6) | (src[i + 2] & 0x3f)); i += 3;
      } else { // 4 字节
        if (i + 3 >= n) break;
        var cp = ((b & 0x07) << 18) | ((src[i + 1] & 0x3f) << 12) | ((src[i + 2] & 0x3f) << 6) | (src[i + 3] & 0x3f);
        cp -= 0x10000;
        // R3012 bug fix：低代理须 10 位（& 0x3ff），旧 & 0x3f（6 位）致 astral char（如 emoji）解码错。
        s += String.fromCharCode(0xd800 + (cp >> 10), 0xdc00 + (cp & 0x3ff)); // astral → 代理对
        i += 4;
      }
    }
    return { s: s, tail: i < n ? src.slice(i) : [] };
  }
  // 单次（flush）解码：valid 完整输入行为同旧；truncated 尾部 → 1 U+FFFD（旧读越界产垃圾，现 spec 容错）。
  function _zw_utf8_decode(bytes) {
    var r = _zw_utf8_decode_stream(bytes, null);
    return r.tail.length ? r.s + '�' : r.s;
  }
  globalThis.TextEncoder = globalThis.TextEncoder || function TextEncoder() {
    if (!(this instanceof TextEncoder)) return new TextEncoder();
  };
  globalThis.TextEncoder.prototype = {
    encoding: 'utf-8',
    encode: function (str) {
      var bytes = _zw_utf8_encode(str);
      var arr = new Uint8Array(bytes.length);
      for (var k = 0; k < bytes.length; k++) arr[k] = bytes[k];
      return arr;
    },
    encodeInto: function (str, dst) {
      var bytes = _zw_utf8_encode(str);
      var m = Math.min(bytes.length, dst.length);
      for (var k = 0; k < m; k++) dst[k] = bytes[k];
      return { read: str.length, written: m };
    }
  };
  globalThis.TextDecoder = globalThis.TextDecoder || function TextDecoder() {
    if (!(this instanceof TextDecoder)) return new TextDecoder();
    this.encoding = 'utf-8';
    this.fatal = false;
    this.ignoreBOM = false;
    this._carry = []; // R3012：stream:true 跨 chunk 不完整尾部（下块前缀拼接补全多字节序列）
  };
  globalThis.TextDecoder.prototype = {
    encoding: 'utf-8',
    fatal: false,
    ignoreBOM: false,
    // R3012：decode(buf, {stream})。stream:true → 不完整尾部入 _carry 待下块（不 flush）；stream:false（缺省）
    // → flush：残余不完整 → 1 U+FFFD，重置 _carry。valid 完整输入行为同旧。
    decode: function (buf, options) {
      var bytes;
      if (buf == null) bytes = new Uint8Array(0);
      else if (buf instanceof ArrayBuffer) bytes = new Uint8Array(buf);
      else if (buf && typeof buf.length === 'number') bytes = buf; // TypedArray / array-like
      else if (buf && buf.buffer) bytes = new Uint8Array(buf.buffer);
      else bytes = new Uint8Array(0);
      var r = _zw_utf8_decode_stream(bytes, this._carry);
      if (options && options.stream === true) { this._carry = r.tail; return r.s; }
      this._carry = []; // flush 重置
      return r.tail.length ? r.s + '�' : r.s;
    }
  };

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
  function _rs_chunk(value) { return { done: false, value: value }; }
  // R3010：strategy → { highWaterMark, size } 解析 + chunk size 计算（spec 背压计量）。无 strategy 时 hwm=1、
  // size 恒 1（CountQueuingStrategy 默认）。size 抛错 / 非有限正数 → 回退 1（spec 应抛 RangeError，headless best-effort）。
  function _zw_streamHwm(strategy) {
    return (strategy && typeof strategy.highWaterMark === 'number' && isFinite(strategy.highWaterMark))
      ? strategy.highWaterMark : 1;
  }
  function _zw_streamSize(sizeFn, chunk) {
    var sz = 1;
    if (typeof sizeFn === 'function') {
      try { sz = sizeFn(chunk); } catch (_e) { sz = 1; }
    }
    if (typeof sz !== 'number' || !isFinite(sz) || sz < 0) sz = 1;
    return sz;
  }
  globalThis.ReadableStream = globalThis.ReadableStream || function ReadableStream(underlyingSource, _strategy) {
    if (!(this instanceof ReadableStream)) return new ReadableStream(underlyingSource, _strategy);
    var source = underlyingSource || {};
    // R3010：背压计量——hwm + size 函数 + queueTotalSize（desiredSize = hwm - queueTotalSize）。
    var hwm = _zw_streamHwm(_strategy);
    var sizeFn = (_strategy && typeof _strategy.size === 'function') ? _strategy.size : null;
    var queue = [];              // 已 enqueue 待消费 { chunk, size }
    var queueTotalSize = 0;
    var state = 'readable';      // readable | closed | errored
    var errorVal = undefined;
    var waiting = [];            // 待 read() 的 {resolve, reject}
    var pulling = false;
    var self = this;
    this._locked = false;

    function enqueueChunk(chunk) {
      if (state !== 'readable') return;
      var sz = _zw_streamSize(sizeFn, chunk);
      // 有等待中的 read → 直接 resolve（零拷贝绕 queue，不计 queueTotalSize）；否则入队 + 累计 size。
      if (waiting.length > 0) waiting.shift().resolve(_rs_chunk(chunk));
      else { queue.push({ chunk: chunk, size: sz }); queueTotalSize += sz; }
    }
    function closeStream() {
      if (state !== 'readable') return;
      state = 'closed';
      while (waiting.length > 0) waiting.shift().resolve(_RS_DONE);
    }
    function errorStream(e) {
      if (state !== 'readable') return;
      errorVal = e;
      state = 'errored';
      while (waiting.length > 0) waiting.shift().reject(e);
    }
    function flushPull() {
      // R3010：readable + 有 pull + desiredSize > 0（queue 有余量）→ 触发一次（pulling 守卫防重入）。
      // desiredSize <= 0（背压）时不 pull，待 read drain 释放余量后再触发。pull 可 enqueue/close/error。
      if (pulling || state !== 'readable' || typeof source.pull !== 'function') return;
      if (hwm - queueTotalSize <= 0) return; // 背压：queue 已达/超 hwm，不 pull
      pulling = true;
      try { source.pull(controller); } catch (_e) { errorStream(_e); }
      pulling = false;
    }
    var controller = {
      get desiredSize() {
        // spec：readable → hwm - queueTotalSize；closed → 0；errored → null。
        if (state === 'errored') return null;
        if (state === 'closed') return 0;
        return hwm - queueTotalSize;
      },
      enqueue: enqueueChunk,
      close: closeStream,
      error: errorStream
    };
    this._doCancel = function (reason) {
      closeStream();
      if (typeof source.cancel === 'function') { try { source.cancel(reason); } catch (_e) {} }
      return Promise.resolve(undefined);
    };
    this.getReader = function () {
      if (self._locked) throw new TypeError('Cannot get a Reader: ReadableStream is locked');
      self._locked = true;
      return {
        read: function () {
          return new Promise(function (resolve, reject) {
            if (state === 'errored') { reject(errorVal); return; }
            // 先 drain 已 enqueue chunk（即便流已 close，剩余 chunk 须先派发，spec §3.5 close 后仍可读余 chunk）。
            if (queue.length > 0) {
              var entry = queue.shift();
              queueTotalSize -= entry.size;
              if (queueTotalSize < 0) queueTotalSize = 0;
              resolve(_rs_chunk(entry.chunk));
              flushPull(); // R3010：drain 释放余量 → 按 desiredSize 重 pull
              return;
            }
            if (state === 'closed') { resolve(_RS_DONE); return; }
            waiting.push({ resolve: resolve, reject: reject });
            flushPull();
          });
        },
        cancel: function (reason) { return self.cancel(reason); },
        releaseLock: function () { self._locked = false; },
        get closed() {
          if (state === 'closed') return Promise.resolve();
          if (state === 'errored') return Promise.reject(errorVal);
          return new Promise(function () {}); // 永挂（headless 无外部 close，read 侧驱动 close）
        }
      };
    };
    this.cancel = function (reason) {
      if (self._locked) return Promise.reject(new TypeError('Cannot cancel: ReadableStream is locked'));
      return self._doCancel(reason);
    };
    Object.defineProperty(this, 'locked', { get: function () { return self._locked; } });
    // async iterator（for await of）：自动 getReader，逐 chunk 迭代，结束/提前 return 时 releaseLock。
    // 流已 locked → getReader 抛 TypeError（caller 应先 releaseLock 或换流）。
    this[Symbol.asyncIterator] = function () {
      var reader = self.getReader();
      return {
        next: function () { return reader.read(); },
        return: function () { try { reader.releaseLock(); } catch (_e) {} return Promise.resolve(_RS_DONE); }
      };
    };
    // R2969 pipeTo(dest)：逐 chunk 从 self 读 → 写入 dest WritableStream，dest 完成（close）；任一侧 error
    // → abort dest + reject。全程持 reader/writer 锁，完成后释放。preventCancel/preventClose/preventAbort
    // options 近似忽略（headless 默认全 false，spec 默认行为）。
    this.pipeTo = function (dest, _options) {
      if (!dest || typeof dest.getWriter !== 'function') {
        return Promise.reject(new TypeError('pipeTo: destination is not a WritableStream'));
      }
      var reader, writer;
      try { reader = self.getReader(); writer = dest.getWriter(); }
      catch (e) { return Promise.reject(e); }
      return new Promise(function (resolve, reject) {
        function finish(err) {
          try { reader.releaseLock(); } catch (_e) {}
          try { writer.releaseLock(); } catch (_e) {}
          if (err) reject(err); else resolve(undefined);
        }
        function pump() {
          reader.read().then(function (r) {
            if (r.done) { writer.close().then(function () { finish(); }, function (e) { finish(e); }); return; }
            writer.write(r.value).then(pump, function (e) {
              try { reader.cancel(e); } catch (_e) {}
              try { dest.abort(e); } catch (_e) {}
              finish(e);
            });
          }, function (e) {
            try { dest.abort(e); } catch (_e) {}
            finish(e);
          });
        }
        pump();
      });
    };
    // R2969 pipeThrough({writable, readable})：fire-and-forget pipeTo(transform.writable)，返 transform.readable。
    // 不 await pipeTo（spec：pipeThrough 立即返 readable，管道后台驱动）。
    this.pipeThrough = function (transform, _options) {
      if (!transform || !transform.writable || !transform.readable) {
        throw new TypeError('pipeThrough: {writable, readable} required');
      }
      self.pipeTo(transform.writable, _options);
      return transform.readable;
    };
    // R2971 tee()：分叉成两独立 ReadableStream（共享同一源）。buffer-based：共享 append-only buffer +
    // 去重源读（readPromise 并发拉源仅一次）+ 每分支 pos（已消费偏移）。分支 pull：buffer 有 chunk → 发；
    // 否则去重拉源 → 入 buffer 后发；源 close/error → 同步到两分支。源须未 locked（tee 持源 reader）。
    // 内存：buffer 随两分支消费速率差增长（慢分支拖累快分支的已读 chunk 保留），headless finite 流可接受。
    this.tee = function () {
      if (self._locked) throw new TypeError('Cannot tee: ReadableStream is locked');
      var reader = self.getReader();
      var buffer = [];            // 共享已读 chunk（append-only，两分支按 pos 各自消费）
      var sourceDone = false;
      var sourceError = null;
      var readPromise = null;     // 去重：并发 pull 共享同一源读 Promise
      function pullOnce() {
        if (sourceDone) return Promise.resolve({ done: true });
        if (sourceError) return Promise.reject(sourceError);
        if (!readPromise) {
          readPromise = reader.read().then(function (r) {
            readPromise = null;
            if (r.done) sourceDone = true; else buffer.push(r.value);
            return r;
          }, function (e) { readPromise = null; sourceError = e; throw e; });
        }
        return readPromise;
      }
      function makeBranch() {
        var pos = 0;
        return new ReadableStream({
          pull: function (controller) {
            if (pos < buffer.length) { controller.enqueue(buffer[pos++]); return; }
            if (sourceDone) { controller.close(); return; }
            if (sourceError) { controller.error(sourceError); return; }
            pullOnce().then(function (r) {
              if (r.done) controller.close();
              else if (pos < buffer.length) controller.enqueue(buffer[pos++]);
            }, function (e) { controller.error(e); });
          }
        });
      }
      return [makeBranch(), makeBranch()];
    };
    // start：同步源初始化（enqueue/close/error 可能在此调用，flush 已等待读者前的 chunk 入 queue）。
    if (typeof source.start === 'function') {
      try { source.start(controller); } catch (_e) { errorStream(_e); }
    }
  };
  // fetch 响应体字符串 → ReadableStream：单 UTF-8 Uint8Array chunk 后 close（headless finite-body 模型，
  // 整体 body 已就绪）。复用 _zw_utf8_encode；空 body → 直接 close（零 chunk）。定义在 part01 _makeResponse
  // 之前调用（runtime），ReadableStream（part02）+ _zw_utf8_encode（part02）在 IIFE 同作用域已就绪。
  function _bodyToStream(src) {
    // R3021：src 可为 Uint8Array（二进制 response body）→ 直接 enqueue 字节；字符串 → UTF-8 编码 Uint8Array chunk。
    var isBytes = src instanceof Uint8Array;
    return new ReadableStream({
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
    });
  }

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
    var sink = underlyingSink || {};
    var state = 'writable';     // writable | closed | errored
    var errorVal = undefined;
    var self = this;
    this._locked = false;
    var resolveClosed, rejectClosed;
    var closedP = new Promise(function (res, rej) { resolveClosed = res; rejectClosed = rej; });
    var pendingWrites = [];       // FIFO {resolve, reject, size}：待 sink.write 完成的 write
    // R3010：背压计量——hwm + size 函数 + queueTotalSize（desiredSize = hwm - queueTotalSize）+ ready Promise。
    var hwm = _zw_streamHwm(_strategy);
    var sizeFn = (_strategy && typeof _strategy.size === 'function') ? _strategy.size : null;
    var queueTotalSize = 0;
    var resolveReady = null;     // ready 阻塞态时的 resolver（desiredSize<=0）；null = ready 已 resolve 态
    var readyPromise = Promise.resolve();
    // ready 在 desiredSize>0 时 resolve（背压释放）；desiredSize<=0 时挂起（背压门控）。
    function updateReady() {
      if (hwm - queueTotalSize > 0) {
        if (resolveReady) { var r = resolveReady; resolveReady = null; readyPromise = Promise.resolve(); r(); }
      } else if (!resolveReady) {
        readyPromise = new Promise(function (res) { resolveReady = res; });
      }
    }
    function errorStream(e) {
      if (state === 'errored' || state === 'closed') return;
      errorVal = e;
      state = 'errored';
      // controller.error → 拒绝所有 pending write + closed（spec：error 使所有未完成 write reject）。
      while (pendingWrites.length > 0) pendingWrites.shift().reject(e);
      rejectClosed(e);
    }
    var controller = { error: errorStream };
    this._controller = controller;

    this.getWriter = function () {
      if (self._locked) throw new TypeError('Cannot get a Writer: WritableStream is locked');
      self._locked = true;
      return {
        get closed() {
          if (state === 'closed') return Promise.resolve();
          if (state === 'errored') return Promise.reject(errorVal);
          return closedP;
        },
        get ready() {
          // R3010：spec 背压门控——errored→reject；closed→resolve；否则 readyPromise（desiredSize<=0 时挂起）。
          if (state === 'errored') return Promise.reject(errorVal);
          if (state === 'closed') return Promise.resolve();
          return readyPromise;
        },
        get desiredSize() {
          // spec：writable→hwm-queueTotalSize；errored→null；closed→0。
          if (state === 'errored') return null;
          if (state === 'closed') return 0;
          return hwm - queueTotalSize;
        },
        write: function (chunk) {
          if (state === 'errored') return Promise.reject(errorVal);
          if (state === 'closed') return Promise.reject(new TypeError('Cannot write to a closed WritableStream'));
          // R3010：入队前累计 size（背压在 pending write 期间生效，desiredSize 降，ready 挂起）。
          var sz = _zw_streamSize(sizeFn, chunk);
          queueTotalSize += sz;
          updateReady();
          return new Promise(function (resolve, reject) {
            var entry = { resolve: resolve, reject: reject, size: sz };
            pendingWrites.push(entry);
            try {
              Promise.resolve(sink.write ? sink.write(chunk, controller) : undefined)
                .then(function () {
                  // sink.write 完成：若期间 controller.error 已拒绝本 entry（state errored），跳过；
                  // 否则 FIFO 取本 entry resolve（多 write 串行完成，顺序匹配）+ 释放对应 size（背压释放）。
                  if (state === 'errored') return;
                  if (pendingWrites.length > 0) {
                    var done = pendingWrites.shift();
                    queueTotalSize -= done.size;
                    if (queueTotalSize < 0) queueTotalSize = 0;
                    updateReady();
                    done.resolve(undefined);
                  }
                },
                function (e) { errorStream(e); });
            } catch (e) { errorStream(e); }
          });
        },
        close: function () {
          if (state === 'errored') return Promise.reject(errorVal);
          if (state === 'closed') return Promise.resolve();
          state = 'closed';
          queueTotalSize = 0; // R3010：close 后 desiredSize=0（closed 态），清背压计量。
          // 残余 pending write 视为完成（headless 串行 sink，正常此时已空，best-effort resolve）。
          while (pendingWrites.length > 0) pendingWrites.shift().resolve(undefined);
          try {
            Promise.resolve(sink.close ? sink.close(controller) : undefined)
              .then(function () { resolveClosed(undefined); },
                    function (e) { errorStream(e); });
          } catch (e) { errorStream(e); }
          return closedP;
        },
        abort: function (reason) { return self.abort(reason); },
        releaseLock: function () { self._locked = false; }
      };
    };
    this.abort = function (reason) {
      if (state === 'closed') return Promise.resolve();
      errorVal = reason;
      state = 'errored';
      try { if (sink.abort) sink.abort(reason); } catch (_e) {}
      rejectClosed(reason);
      return Promise.resolve(undefined);
    };
    Object.defineProperty(this, 'locked', { get: function () { return self._locked; } });
    if (typeof sink.start === 'function') {
      try { sink.start(controller); } catch (_e) { errorStream(_e); }
    }
  };

  // ── P1a TransformStream（Streams API transform，R2969）──
  // {readable, writable} 配对：writable.write(chunk) → transformer.transform(chunk, controller) →
  // controller.enqueue 到 readable；writable.close → transformer.flush(controller) → close readable。
  // 无 transform fn → 恒等（chunk 直 enqueue）。controller.enqueue/close/error 转发到 readable 的 controller。
  // 用于 pipeThrough 管道（如 response.body.pipeThrough(new TextDecoderStream()) 解码——TextDecoderStream
  // 本身属 follow-up，本切片提供 TransformStream 基座）。
  globalThis.TransformStream = globalThis.TransformStream || function TransformStream(transformer, _strategy) {
    if (!(this instanceof TransformStream)) return new TransformStream(transformer, _strategy);
    var tx = transformer || {};
    var enqueueToR, closeR, errorR;
    var transformController = {
      enqueue: function (chunk) { if (enqueueToR) enqueueToR(chunk); },
      close: function () { if (closeR) closeR(); },
      error: function (e) { if (errorR) errorR(e); }
    };
    var self = this;
    this.readable = new ReadableStream({
      start: function (controller) {
        enqueueToR = controller.enqueue;
        closeR = controller.close;
        errorR = controller.error;
        if (typeof tx.start === 'function') tx.start(transformController);
      }
    });
    this.writable = new WritableStream({
      write: function (chunk) {
        if (typeof tx.transform === 'function') tx.transform(chunk, transformController);
        else transformController.enqueue(chunk); // 恒等
      },
      close: function () {
        if (typeof tx.flush === 'function') tx.flush(transformController);
        transformController.close();
      }
    });
  };

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
    TransformStream.call(this, {
      transform: function (chunk, controller) { controller.enqueue(enc.encode(String(chunk))); }
    });
    this.encoding = 'utf-8';
  };
  globalThis.TextEncoderStream.prototype = Object.create(globalThis.TransformStream.prototype);
  globalThis.TextEncoderStream.prototype.constructor = globalThis.TextEncoderStream;
  globalThis.TextDecoderStream = globalThis.TextDecoderStream || function TextDecoderStream(label) {
    if (!(this instanceof TextDecoderStream)) return new TextDecoderStream(label);
    var dec = new TextDecoder(label);
    TransformStream.call(this, {
      transform: function (chunk, controller) {
        var s = dec.decode(chunk, { stream: true }); // R3012：stream:true 跨 chunk 状态
        if (s) controller.enqueue(s); // 空 string（chunk 末切多字节前导字节，缓存在 _carry）跳过，避免空 chunk
      },
      flush: function (controller) {
        var s = dec.decode(); // flush 残余（stream:false，不完整 → U+FFFD；完整输入通常 ''）
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
      out.push([decodeURIComponent(k.replace(/\+/g, ' ')), decodeURIComponent(v.replace(/\+/g, ' '))]);
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
      if (typeof init.forEach === 'function') {
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
      if (arguments.length >= 2) {
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
    entries: function () { return _zw_iter(this._p.map(function (p) { return [p[0], p[1]]; })); },
    keys: function () { return _zw_iter(this._p.map(function (p) { return p[0]; })); },
    values: function () { return _zw_iter(this._p.map(function (p) { return p[1]; })); },
    toString: function () {
      var out = [];
      for (var i = 0; i < this._p.length; i++) {
        out.push(encodeURIComponent(this._p[i][0]).replace(/%20/g, '+') + '=' + encodeURIComponent(this._p[i][1]).replace(/%20/g, '+'));
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
      return [n, value, fn];
    }
    return [n, String(value), undefined];
  }
  globalThis.FormData = globalThis.FormData || function FormData(form) {
    if (!(this instanceof FormData)) return new FormData(form);
    this._p = [];
    if (form != null && typeof form === 'object' && typeof form.querySelectorAll === 'function') {
      // best-effort form 字段枚举；失败静默（不抛、不破坏脚本）。
      try {
        var fields = form.querySelectorAll('input, select, textarea');
        for (var i = 0; i < fields.length; i++) {
          var f = fields[i];
          var name = f.getAttribute ? f.getAttribute('name') : f.name;
          if (!name) continue;
          var type = ((f.getAttribute ? f.getAttribute('type') : f.type) || '').toLowerCase();
          if (type === 'checkbox' || type === 'radio') {
            if (f.checked) this._p.push([String(name), f.value != null ? String(f.value) : 'on', undefined]);
          } else if (type !== 'file' && type !== 'submit' && type !== 'button' && type !== 'reset' && type !== 'image') {
            this._p.push([String(name), f.value != null ? String(f.value) : '', undefined]);
          }
        }
      } catch (_e) { /* best-effort：枚举失败则按空 FormData */ }
    }
  };
  globalThis.FormData.prototype = {
    append: function (name, value, filename) {
      // R3014：Blob/File 值保真（_zwFdEntry）；filename 仅对 Blob 有意义（spec）。
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
    forEach: function (cb, thisArg) {
      for (var i = 0; i < this._p.length; i++) cb.call(thisArg, this._p[i][1], this._p[i][0], this);
    },
    entries: function () { return _zw_iter(this._p.map(function (e) { return [e[0], e[1]]; })); },
    keys: function () { return _zw_iter(this._p.map(function (e) { return e[0]; })); },
    values: function () { return _zw_iter(this._p.map(function (e) { return e[1]; })); },
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
  function _zwIsForbiddenReqHeader(ln) {
    if (_ZW_FORBIDDEN_REQ_HEADERS[ln]) return true;
    // 前缀匹配（byte-case-insensitive，ln 已小写）：Proxy- / Sec-
    return ln.slice(0, 6) === 'proxy-' || ln.slice(0, 4) === 'sec-';
  }
  // R3222：Fetch §3.4.5 forbidden response-header names——response Headers 的 get/has/iterate 不暴露，
  // 但 getSetCookie 仍返 set-cookie 数组（spec 特例）。`_guard`='response' 由 Response ctor 设。
  // https://fetch.spec.whatwg.org/#forbidden-response-header-name
  function _hdrIsForbiddenResponse(ln) {
    return ln === 'set-cookie' || ln === 'set-cookie2';
  }
  // R3223：Headers guard 写侧阻断（Fetch §5.2 append/set/delete step 3/5）——
  // request guard 阻 forbidden request-header；response guard 阻 forbidden response-header；none 不阻。
  function _hdrGuardBlocks(guard, ln) {
    if (guard === 'request') return _zwIsForbiddenReqHeader(ln);
    if (guard === 'response') return _hdrIsForbiddenResponse(ln);
    return false;
  }
  // R3223：Headers fill——逐值 append（尊重目标 guard）。供 Headers ctor（guard none）与 Request ctor
  //（guard request）复用。Headers 实例源直接迭代内部 _h（Fetch §5.1「for each header in init's header list」
  // 指内部列表，含 response guard 隐藏的 Set-Cookie；目标 guard 决定是否过滤）。
  function _fillHeaders(h, init) {
    if (init == null) return;
    if (init._h) {
      for (var k in init._h) {
        if (!Object.prototype.hasOwnProperty.call(init._h, k)) continue;
        var vals = init._h[k];
        for (var vi = 0; vi < vals.length; vi++) h.append(k, vals[vi]);
      }
      return;
    }
    if (Array.isArray(init)) {
      for (var i = 0; i < init.length; i++) {
        var pair = init[i];
        if (pair && pair.length >= 2) h.append(pair[0], pair[1]);
      }
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
    this._h = {}; // lowername -> string[]（保 append 序与多值）
    this._guard = 'none'; // R3223：guard none/request/response（Fetch §5.1）；ctor 构造为 none（不过滤）
    if (init != null) _fillHeaders(this, init); // guard none → 不过滤禁止头
  };
  globalThis.Headers.prototype = {
    append: function (name, value) {
      name = _hdrNorm(name);
      if (!name) return;
      // R3223：guard 写侧阻断（request→forbidden req-header；response→forbidden resp-header）。
      if (_hdrGuardBlocks(this._guard, name)) return;
      (this._h[name] = this._h[name] || []).push(String(value));
    },
    delete: function (name) {
      name = _hdrNorm(name);
      // R3223：禁止头经 guard 不可删（Fetch §5.4 delete step 3/5；request guard 下本就未存，response guard 护 Set-Cookie）。
      if (_hdrGuardBlocks(this._guard, name)) return;
      delete this._h[name];
    },
    get: function (name) {
      name = _hdrNorm(name);
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
      name = _hdrNorm(name);
      if (this._guard === 'response' && _hdrIsForbiddenResponse(name)) return false;
      return Object.prototype.hasOwnProperty.call(this._h, name);
    },
    set: function (name, value) {
      name = _hdrNorm(name);
      if (!name) return;
      // R3223：guard 写侧阻断（同 append）。
      if (_hdrGuardBlocks(this._guard, name)) return;
      this._h[name] = [String(value)];
    },
    forEach: function (cb, thisArg) {
      for (var k in this._h) {
        if (Object.prototype.hasOwnProperty.call(this._h, k) && !(this._guard === 'response' && _hdrIsForbiddenResponse(k))) cb.call(thisArg, this._h[k].join(', '), k, this);
      }
    },
    entries: function () {
      var out = [];
      for (var k in this._h) {
        if (Object.prototype.hasOwnProperty.call(this._h, k) && !(this._guard === 'response' && _hdrIsForbiddenResponse(k))) out.push([k, this._h[k].join(', ')]);
      }
      return _zw_iter(out);
    },
    keys: function () {
      var out = [];
      for (var k in this._h) {
        if (Object.prototype.hasOwnProperty.call(this._h, k) && !(this._guard === 'response' && _hdrIsForbiddenResponse(k))) out.push(k);
      }
      return _zw_iter(out);
    },
    values: function () {
      var out = [];
      for (var k in this._h) {
        if (Object.prototype.hasOwnProperty.call(this._h, k) && !(this._guard === 'response' && _hdrIsForbiddenResponse(k))) out.push(this._h[k].join(', '));
      }
      return _zw_iter(out);
    }
  };
  // 自身可迭代（for (const [k,v] of headers)）：[Symbol.iterator] → entries。
  if (typeof Symbol !== 'undefined') {
    globalThis.Headers.prototype[Symbol.iterator] = globalThis.Headers.prototype.entries;
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
    this.type = (options && options.type != null) ? String(options.type).toLowerCase() : '';
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
  _zwParseEl.prototype.hasAttribute = function (name) {
    return Object.prototype.hasOwnProperty.call(this._attrs, String(name));
  };
  // 子树 query：重解析本元素 outerHTML（host 二次 parse + select），返只读 element-proxy。
  _zwParseEl.prototype.querySelector = function (sel) {
    if (typeof __zw_parse_html_query !== 'function') return null;
    var arr = JSON.parse(__zw_parse_html_query(this.outerHTML, String(sel), '0'));
    return arr.length ? new _zwParseEl(arr[0]) : null;
  };
  _zwParseEl.prototype.querySelectorAll = function (sel) {
    if (typeof __zw_parse_html_query !== 'function') return [];
    var arr = JSON.parse(__zw_parse_html_query(this.outerHTML, String(sel), '1'));
    var out = [];
    for (var i = 0; i < arr.length; i++) out.push(new _zwParseEl(arr[i]));
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
        var arr = JSON.parse(__zw_parse_html_child_nodes(this.outerHTML, tag));
        for (var i = 0; i < arr.length; i++) if (arr[i]) node.childNodes.push(_zwMBuildNode(this.outerHTML, arr[i], node));
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
  _zwParseEl.prototype.removeAttribute = function (n) { this._ensureMutTree().removeAttribute(n); };
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
  globalThis.DOMParser = globalThis.DOMParser || function DOMParser() {};
  globalThis.DOMParser.prototype.parseFromString = function (str, mimeType) {
    // text/html | text/xml | application/xml | application/xhtml+xml | image/svg+xml 统一按 HTML 解析。
    var html = str == null ? '' : String(str);
    var d = new _zwParsedDoc(html);
    d.mimeType = mimeType || 'text/html';
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
      var n = {
        nodeType: 1,
        tagName: tag.toUpperCase(),
        nodeName: tag.toUpperCase(),
        localName: tag.toLowerCase(),
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
    if (t === 'function') throw new DOMException('function could not be cloned.', 'DataCloneError');
    if (t === 'symbol') throw new DOMException('symbol could not be cloned.', 'DataCloneError');
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
  function _maybeFirePageShow() {
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
    // clipboard（R2817 + R2964）——异步剪贴板 API（复制按钮 ubiquitous）。headless 无 OS 剪贴板 →
    // **进程内 store**（IIFE 闭包 `_store`）：writeText/readText 真实往返（同页/同进程 write→read 通，
    // 覆盖复制按钮 + 粘贴检查高频模式）。read/write（ClipboardItem 富 MIME）仍 best-effort stub
    //（headless 无真 MIME 剪贴板，不抛）。spec：writeText 返 Promise<void>，readText 返 Promise<string>。
    clipboard: (function () {
      var _store = '';
      return {
        writeText: function (text) { _store = String(text != null ? text : ''); return Promise.resolve(undefined); },
        readText: function () { return Promise.resolve(_store); },
        read: function () { return Promise.resolve([]); },
        write: function (_data) { return Promise.resolve(undefined); },
      };
    })(),
    // R3314：storage（Storage API + OPFS Origin Private File System）——Done Criteria §3 Tier 2 列项
    //（zero-web.md 行 80「IndexedDB + Cache API + OPFS」，OPFS 此前全缺）。estimate（配额查询，analytics 高频）+
    // getDirectory（OPFS root）。headless 无真 OS 文件系统 → **进程内虚拟 FS 树**（内存近似，参照 clipboard
    // IIFE store 模式）：目录节点 {kind:'dir', children:{name→node}}，文件节点 {kind:'file', data:Uint8Array}。
    // spec https://fs.spec.whatwg.org/ + https://web.dev/file-system-access/。**诚实范围**：① 仅 OPFS
    //（navigator.storage.getDirectory），无 showOpenFilePicker/showSaveFilePicker（用户可见文件选择器，headless 无）；
    // ② 内存后端（非持久，跨页/进程丢）；③ 无 createSyncAccessHandle（worker 同步句柄，headless worker 无真线程）；
    // ④ 无 permission/move/transferable。
    storage: (function () {
      // 根目录节点（OPFS 唯一根）。
      var root = { kind: 'dir', children: {} };
      // 构造 FileSystemDirectoryHandle（绑定某目录节点）。
      // 构造 FileSystemDirectoryHandle（绑定某目录节点）。
      // R3254-C9：节点级缓存——同一目录多次获取返回同一对象（isSameEntry 判 true）。
      function dirHandle(node, name) {
        if (node._dh) return node._dh;
        var h = {
          kind: 'directory',
          name: name || '',
          // getFileHandle(name, {create}) → 文件句柄（create=true 不存在则建空文件）。失败 reject。
          getFileHandle: function (n, opts) {
            return Promise.resolve().then(function () {
              // R3254-C14：名称校验（spec FS：空串、'.'、'..'、含 '/' → TypeError）。
              if (!_zwFsValidName(n)) return Promise.reject(new TypeError('无效的文件名'));
              var child = node.children[n];
              if (child && child.kind !== 'file') return Promise.reject(new TypeError(n + ' 是目录'));
              if (!child && !(opts && opts.create)) return Promise.reject(new TypeError(n + ' 不存在'));
              if (!child) { child = { kind: 'file', data: new Uint8Array(0) }; node.children[n] = child; }
              return fileHandle(child, n);
            });
          },
          // getDirectoryHandle(name, {create}) → 子目录句柄。
          getDirectoryHandle: function (n, opts) {
            return Promise.resolve().then(function () {
              // R3254-C14：名称校验。
              if (!_zwFsValidName(n)) return Promise.reject(new TypeError('无效的目录名'));
              var child = node.children[n];
              if (child && child.kind !== 'dir') return Promise.reject(new TypeError(n + ' 是文件'));
              if (!child && !(opts && opts.create)) return Promise.reject(new TypeError(n + ' 不存在'));
              if (!child) { child = { kind: 'dir', children: {} }; node.children[n] = child; }
              return dirHandle(child, n);
            });
          },
          // removeEntry(name, {recursive}) → 删文件或目录（spec FS：非空目录须 recursive，
          // 否则 InvalidModificationError——此前静默递归删除）。
          removeEntry: function (n, opts) {
            return Promise.resolve().then(function () {
              // R3254-C14：名称校验。
              if (!_zwFsValidName(n)) return Promise.reject(new TypeError('无效的名称'));
              var child = node.children[n];
              if (!child) return Promise.reject(new TypeError(n + ' 不存在'));
              if (child.kind === 'dir' && !(opts && opts.recursive) && Object.keys(child.children).length > 0) {
                return Promise.reject(_zwDomException('目录非空（需 recursive）', 'InvalidModificationError'));
              }
              delete node.children[n];
              return undefined;
            });
          },
          // keys() → 子项名迭代器（spec async iterable）。返数组（近似 [Symbol.asyncIterator]）。
          keys: function () { return Promise.resolve(Object.keys(node.children)); },
          entries: function () {
            return Promise.resolve(Object.keys(node.children).map(function (k) {
              var c = node.children[k];
              return [k, c.kind === 'dir' ? dirHandle(c, k) : fileHandle(c, k)];
            }));
          },
          values: function () {
            return this.entries().then(function (es) { return es.map(function (e) { return e[1]; }); });
          },
          isSameEntry: function (other) { return this === other; },
        };
        node._dh = h;
        return h;
      }
      // R3254-C14：OPFS 句柄名称校验（spec FS §7：空串、'.'、'..'、含 '/' → 无效）。
      function _zwFsValidName(n) {
        return typeof n === 'string' && n.length > 0 && n !== '.' && n !== '..' && n.indexOf('/') < 0;
      }
      // 构造 FileSystemFileHandle（绑定某文件节点）。
      // R3254-C9：节点级句柄缓存——同一文件多次 getFileHandle 返回**同一对象**，
      // isSameEntry 按对象同一性判 true（此前每次新建对象 → 恒 false）。
      function fileHandle(node, name) {
        if (node._fh) return node._fh;
        var h = {
          kind: 'file',
          name: name,
          // getFile() → Blob（读当前内容快照）。
          getFile: function () {
            return Promise.resolve().then(function () {
              return new Blob([node.data.slice()], { type: 'application/octet-stream' });
            });
          },
          // createWritable({keepExistingData}) → FileSystemWritableFileStream（spec FS §8.5）。
          // R3315：内部模型「缓冲 Uint8Array + 文件指针 pos」（替代 R3314 chunks 数组，支持 seek/truncate/position）。
          // keepExistingData:false（默认）→ 空缓冲（整体替换，spec 默认）；true → 复制原文件内容为初始缓冲。
          // write(data) 在 pos 写入（自动扩展缓冲，pos 前进）；write({position,data}) 先 seek(position) 再写；
          // seek(offset) 移 pos（不足自动扩展填零）；truncate(size) 缓冲截断/扩展；close 缓冲落 node.data。
          createWritable: function (opts) {
            // 初始缓冲：keepExistingData 时复制原内容，否则空（整体替换）。
            var buf = (opts && opts.keepExistingData) ? new Uint8Array(node.data) : new Uint8Array(0);
            var pos = 0;
            var closed = false;
            var aborted = false;
            function guard() { if (closed) return Promise.reject(new TypeError('stream 已关闭')); return null; }
            // 把 string/Blob/TypedArray/BufferSource 归一为 Uint8Array。
            // R3254-C3：字符串按 UTF-8 编码（此前 `charCodeAt & 0xff` latin-1 截断，中文乱码）。
            // R3254-C4：TypedArray/DataView 只取**视图范围**（byteOffset+byteLength）——此前
            // `new Uint8Array(data.buffer)` 复制整个底层 ArrayBuffer，subarray 视图越界写入。
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
            // 在 pos 写入 bytes（自动扩展缓冲，不截断既有超出内容）。
            function writeAt(curPos, bytes) {
              var need = curPos + bytes.length;
              if (need > buf.length) {
                var grown = new Uint8Array(need);
                grown.set(buf);
                buf = grown;
              }
              for (var k = 0; k < bytes.length; k++) buf[curPos + k] = bytes[k];
            }
            return Promise.resolve({
              // write(data) 或 write({type:'write', position, data}) 或 write({type:'seek', position}) 或 write({type:'truncate', size})。
              write: function (data) {
                var g = guard(); if (g) return g;
                // 对象形式：write({type:'seek'|'truncate'|'write', position/size/data})。
                if (data && typeof data === 'object' && typeof data.type === 'string') {
                  if (data.type === 'seek') {
                    // R3254-C2：负 position → TypeError（spec FS §8.5）；此前负值静默丢数据
                    //（负索引写 Uint8Array 是 no-op）。
                    if (typeof data.position === 'number' && data.position < 0) {
                      return Promise.reject(new TypeError('seek: position 不能为负'));
                    }
                    pos = (data.position | 0);
                    return Promise.resolve();
                  }
                  if (data.type === 'truncate') {
                    var sz = data.size | 0;
                    if (sz < 0) sz = 0;
                    var tr = new Uint8Array(sz);
                    tr.set(buf.subarray(0, Math.min(sz, buf.length)));
                    buf = tr;
                    if (pos > sz) pos = sz;
                    return Promise.resolve();
                  }
                  // type === 'write'
                  // R3254-C2：显式 position 负 → TypeError（同 seek）。
                  if (typeof data.position === 'number' && data.position < 0) {
                    return Promise.reject(new TypeError('write: position 不能为负'));
                  }
                  var wpos = (typeof data.position === 'number') ? (data.position | 0) : pos;
                  var wb = toBytes(data.data);
                  writeAt(wpos, wb);
                  pos = wpos + wb.length;
                  return Promise.resolve();
                }
                // 简单形式：write(data) 在 pos 写。
                var b = toBytes(data);
                writeAt(pos, b);
                pos += b.length;
                return Promise.resolve();
              },
              seek: function (offset) {
                var g = guard(); if (g) return g;
                // R3254-C2：负 offset → TypeError。
                if (typeof offset === 'number' && offset < 0) {
                  return Promise.reject(new TypeError('seek: offset 不能为负'));
                }
                pos = (offset | 0);
                return Promise.resolve();
              },
              truncate: function (size) {
                var g = guard(); if (g) return g;
                var tz = (size | 0); if (tz < 0) tz = 0;
                var tn = new Uint8Array(tz);
                tn.set(buf.subarray(0, Math.min(tz, buf.length)));
                buf = tn;
                if (pos > tz) pos = tz;
                return Promise.resolve();
              },
              close: function () {
                // R3254-C10：abort 后 close → InvalidStateError（spec：abort 放弃的缓冲不得提交）。
                if (aborted) return Promise.reject(new TypeError('stream 已 abort'));
                closed = true;
                node.data = buf;
                return Promise.resolve();
              },
              abort: function () { closed = true; aborted = true; return Promise.resolve(); },
            });
          },
          isSameEntry: function (other) { return this === other; },
        };
        node._fh = h;
        return h;
      }
      return {
        // estimate() → 配额查询（analytics/存储压力检测高频）。headless 静态近似（usage 按虚拟 FS 字节数估）。
        estimate: function () {
          var bytes = 0;
          (function count(n) { if (n.kind === 'file') bytes += n.data.length; else for (var k in n.children) count(n.children[k]); })(root);
          return Promise.resolve({ usage: bytes, quota: 1024 * 1024 * 100 });
        },
        // getDirectory() → OPFS root 句柄（spec 返 Promise<FileSystemDirectoryHandle>）。
        getDirectory: function () { return Promise.resolve(dirHandle(root, '')); },
      };
    })(),
    // sendBeacon（R2931）——页面卸载/后台分析 beacon（fire-and-forget POST，analytics/RUM 高频：GA 等
    // unload 时上报）。headless 无真网络发送（避免无人值守测试依赖外部网络）→ accept-and-return-true
    //（spec：返 true = 成功入队 best-effort；data 类型不限，忽略）。url 缺省（null/undefined）→ false。
    sendBeacon: function(url, _data) {
      return url != null;
    },
    // permissions（R2817）——权限查询（clipboard/geolocation 等 feature-detect 配对）。headless → state 'prompt'
    //（中性，既非 granted 非 denied）。
    permissions: {
      query: function(desc) {
        var name = (desc && desc.name) || '';
        return Promise.resolve({
          name: name, state: 'prompt', onchange: null,
          addEventListener: function() {}, removeEventListener: function() {},
        });
      },
    },
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
    // serviceWorker（R3318）——Service Worker 注册 API（PWA / 离线缓存 / 推送通知 / 后台同步基础）。
    // Done Criteria §3 Tier 2 + zero-web.md M12 列项。**此前仅存在于 A-gen dom_bridge.rs（死代码，
    // generate_dom_api_polyfill 无页面交互生产调用方——见 R2821 Performance API 迁移说明），B-gen 生产
    // 页面 shim（run_page_scripts 真实 DOM 桥路径，wpt-runner/reftest 同机制）缺失 → `navigator.serviceWorker`
    // 为 undefined，PWA 注册脚本 `navigator.serviceWorker.register(...)` 全抛 TypeError**。本切片移植到生产 shim。
    // spec https://w3c.github.io/ServiceWorker/。headless 无真 SW 执行环境（无独立 worker 线程、无真 fetch
    // 拦截、无真 install/activate 事件派发）→ **进程内注册表近似**（参照 A-gen + storage crate
    // ServiceWorkerRegistry 状态机）：register 返 Promise<registration>，经 setTimeout(0) 模拟 install→
    // waiting→active 异步生命周期（installing/waiting/active 字段逐态推进 + scope 派生 + oncontrollerchange）。
    // getRegistration/getRegistrations/ready/unregister 完整注册查询面。fetch 拦截 / 真事件回调 defer
    //（需独立 worker 执行环境，跨层大改）。
    serviceWorker: (function () {
      var _registrations = [];
      var _controller = null;
      // ready Promise 在首个 registration 激活后 resolve（active registration）。
      var _readyResolve;
      var _ready = new Promise(function (resolve) { _readyResolve = resolve; });
      // 容器自身引用（setTimeout 回调读 oncontrollerchange 经此，避免 this 绑定脆弱）。
      var _container = {
        oncontrollerchange: null,
        onmessage: null
      };
      // 构造 ServiceWorker 实例（state 推进时镜像 A-gen 字段）。
      function makeSW(scriptURL, state) {
        return { scriptURL: scriptURL, state: state, onstatechange: null };
      }
      // 构造 ServiceWorkerRegistration（spec 字段：scope + installing/waiting/active + updateViaCache +
      // onupdatefound；方法：unregister/update）。
      function makeReg(scriptURL, scope) {
        var reg = {
          scope: scope,
          updateViaCache: 'imports',
          installing: null,
          waiting: null,
          active: null,
          onupdatefound: null,
          // unregister → 从注册表移除，返 Promise<true>（spec）。
          unregister: function () {
            for (var i = 0; i < _registrations.length; i++) {
              if (_registrations[i] === reg) { _registrations.splice(i, 1); break; }
            }
            return Promise.resolve(true);
          },
          update: function () { return Promise.resolve(); },
          // spec：getNotifications / showNotification（Notifications API 配对）—— headless 无通知，stub。
          getNotifications: function () { return Promise.resolve([]); },
          showNotification: function () { return Promise.resolve(); }
        };
        return reg;
      }
      _container.register = function (scriptURL, options) {
        if (!scriptURL || typeof scriptURL !== 'string') {
          return Promise.reject(new TypeError('ServiceWorkerContainer.register: scriptURL is required'));
        }
        // scope 缺省 = scriptURL 所在目录（spec §4.5.1）。
        var scope = (options && options.scope) || scriptURL.substring(0, scriptURL.lastIndexOf('/') + 1);
        var reg = makeReg(scriptURL, scope);
        _registrations.push(reg);
        // 模拟 install→installed(waiting)→activated(active) 异步生命周期（每态 setTimeout(0) 推进，
        // 下 execute checkpoint 可读）。installing 立即置（installing→installed 是异步过渡）。
        reg.installing = makeSW(scriptURL, 'installing');
        if (typeof reg.onupdatefound === 'function') {
          try { reg.onupdatefound({ type: 'updatefound', target: reg }); } catch (_e) {}
        }
        setTimeout(function () {
          reg.waiting = makeSW(scriptURL, 'installed');
          reg.installing = null;
        }, 0);
        setTimeout(function () {
          reg.active = makeSW(scriptURL, 'activated');
          reg.waiting = null;
          _controller = reg.active;
          if (_readyResolve) { _readyResolve(reg); _readyResolve = null; }
          if (typeof _container.oncontrollerchange === 'function') {
            try { _container.oncontrollerchange({ type: 'controllerchange', target: _container }); } catch (_e) {}
          }
        }, 0);
        return Promise.resolve(reg);
      };
      _container.getRegistration = function (scope) {
        for (var i = 0; i < _registrations.length; i++) {
          if (!scope || _registrations[i].scope === scope) {
            return Promise.resolve(_registrations[i]);
          }
        }
        return Promise.resolve(undefined);
      };
      _container.getRegistrations = function () {
        return Promise.resolve(_registrations.slice());
      };
      // ready → Promise<ServiceWorkerRegistration>，首个 registration 激活后 resolve（否则挂起，spec 行为）。
      Object.defineProperty(_container, 'ready', { get: function () { return _ready; } });
      // controller → 当前控制页面的 active ServiceWorker（激活前为 null）。
      Object.defineProperty(_container, 'controller', { get: function () { return _controller; } });
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
  function _zwConsoleEmit(level, args) {
    if (typeof __zw_console_log !== 'function') return; // 宿主未注册 → no-op（向后兼容）
    var parts = [];
    for (var i = 0; i < args.length; i++) parts.push(_zwSerializeConsoleArg(args[i]));
    try { __zw_console_log(level, parts.join(' ')); } catch (_) {}
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
  // 返 createElement('audio') proxy（镜像 Image R2834），设 src；允许 new 与无 new。headless 无音频设备——
  // play/pause/load 为 no-op（play 返 resolved Promise，spec），经下方 HTMLMediaElement 方法桩。`instanceof
  // Audio`=false（shim 返 Proxy，同 Image/Option 谱，documented）。
  function Audio(src) {
    var el = globalThis.document.createElement('audio');
    if (src !== undefined) { try { el.setAttribute('src', String(src)); } catch (_e) {} }
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

  function _createStorage() {
    var _data = {};
    return {
      getItem: function(key) { return _data.hasOwnProperty(key) ? _data[key] : null; },
      setItem: function(key, value) { _data[String(key)] = String(value); },
      removeItem: function(key) { delete _data[String(key)]; },
      clear: function() { _data = {}; },
      key: function(index) {
        var keys = Object.keys(_data);
        return index >= 0 && index < keys.length ? keys[index] : null;
      },
      get length() { return Object.keys(_data).length; }
    };
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
    var finish = function () {
      queue.requests.shift();
      queue.running = false;
      queue.retry = null;
      if (queue.requests.length) {
        queueMicrotask(function () { _zwIDBRunConnectionQueue(name); });
      } else {
        delete _zwIDBConnectionQueues[name];
      }
    };
    queue.requests[0](finish, queue);
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

  function _zwIDBInvoke(target, type, event, capture) {
    var listeners = ((target._listeners && target._listeners[type]) || []).slice();
    if (!capture && !event._immediatePropagationStopped) {
      var handler = target['on' + type];
      if (typeof handler === 'function') {
        try { handler.call(target, event); } catch (_) {}
      }
    }
    for (var i = 0; i < listeners.length; i++) {
      if (event._immediatePropagationStopped) break;
      var listener = listeners[i];
      if (listener.capture !== capture) continue;
      var callback = listener.callback;
      try {
        if (typeof callback === 'function') callback.call(target, event);
        else if (callback && typeof callback.handleEvent === 'function') callback.handleEvent(event);
      } catch (_) {}
    }
  }

  function _zwIDBEmit(target, type, event) {
    event.currentTarget = target;
    _zwIDBInvoke(target, type, event, true);
    _zwIDBInvoke(target, type, event, false);
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
    event.target = request;
    function next() {
      if (invoked) _zwIDBDeactivateTransactions(transaction);
      while (position < steps.length) {
        var step = steps[position++];
        if (event._immediatePropagationStopped) break;
        if (event._propagationStopped && step.group !== currentGroup) break;
        currentGroup = step.group;
        event.currentTarget = step.target;
        try {
          if (typeof step.callback === 'function') step.callback.call(step.target, event);
          else if (step.callback && typeof step.callback.handleEvent === 'function') {
            step.callback.handleEvent(event);
          }
        } catch (_) {}
        invoked = true;
        event.currentTarget = null;
        queueMicrotask(next);
        return;
      }
      done();
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
      _zwIDBDispatchRequestEvent(req, transaction, ev, function () {
        dispatch.settled = true;
        if (ev.type === 'error' && transaction && !ev.defaultPrevented && !transaction._aborted) {
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
    this.name = name;
    this.keyPath = Array.isArray(keyPath) ? keyPath.slice() : (keyPath == null ? null : keyPath);
    this.autoIncrement = !!autoIncrement;
    this._records = records; // Map<key, value>
    this._indexes = indexes; // {indexName: {keyPath, unique}}
    this.transaction = transaction || null;
    this._metadata = metadata;
    this.indexNames = _zwIDBStringList(function () { return Object.keys(indexes); });
  }
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
  _zwIDBStore.prototype._getAll = function (query, count, keysOnly, queryProvided) {
    this._assertUsable(false);
    if (queryProvided && query !== undefined) {
      var valid = false;
      try {
        valid = _zwIDBIsKeyRange(query) || !!_zwIDBKey(query, []);
      } catch (_) {}
      if (!valid) {
        throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
      }
    }
    var request = new _zwIDBRequest(this);
    request.transaction = this.transaction;
    var store = this;
    var perform = function () {
      if (store.transaction && store.transaction._hostId !== null) {
        var hostRequest = {
          op: 'transaction_get_all',
          transaction: store.transaction._hostId,
          store: store.name,
          keys_only: !!keysOnly
        };
        if (queryProvided && query !== undefined) hostRequest.query = _zwIDBQueryToWire(query);
        if (count !== undefined) hostRequest.count = Math.max(0, Number(count));
        try {
          var hosted = _zwIDBHostCall(hostRequest);
          var hostedResult = (hosted.records || []).map(function (record) {
            return keysOnly
              ? _zwIDBKeyFromWire(record.key)
              : _zwIDBValueFromWire(record.value);
          });
          _zwIDBDispatch(request, 'success', hostedResult);
        } catch (hostError) {
          _zwIDBRequestHostError(request, hostError);
        }
        return;
      }
      var entries = [];
      store._records.forEach(function (value, key) {
        if (!queryProvided || query === undefined || _zwIDBQueryMatches(query, key)) {
          entries.push({ key: key, value: value });
        }
      });
      entries.sort(function (a, b) { return _zwIDBCompareValues(a.key, b.key); });
      if (count !== undefined) entries = entries.slice(0, Math.max(0, Number(count)));
      var result = entries.map(function (entry) {
        return globalThis.structuredClone(keysOnly ? entry.key : entry.value);
      });
      _zwIDBDispatch(request, 'success', result);
    };
    _zwIDBRunTransactionOperation(this.transaction, perform);
    return request;
  };
  _zwIDBStore.prototype.getAll = function (query, count) {
    return this._getAll(query, count, false, arguments.length >= 1);
  };
  _zwIDBStore.prototype.getAllKeys = function (query, count) {
    return this._getAll(query, count, true, arguments.length >= 1);
  };
  _zwIDBStore.prototype.createIndex = function (name, keyPath, opts) {
    this._assertUsable(true);
    name = String(name);
    keyPath = _zwIDBNormalizeKeyPath(keyPath, true);
    var unique = !!((opts || {}).unique);
    var multiEntry = !!((opts || {}).multiEntry);
    if (Object.prototype.hasOwnProperty.call(this._indexes, name)) {
      throw new globalThis.DOMException('The index already exists.', 'ConstraintError');
    }
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
      this._records.forEach(function (value) {
        var indexKey = this._indexKey(value, keyPath);
        var keys = multiEntry && Array.isArray(indexKey) ? indexKey : [indexKey];
        keys.forEach(function (candidate) {
          if (!_zwIDBKey(candidate, [])) return;
          if (seen.some(function (existing) {
            return _zwIDBCompareValues(existing, candidate) === 0;
          })) this.transaction.abort();
          seen.push(candidate);
        }, this);
      }, this);
    }
    return new _zwIDBIndex(this, name, metadata);
  };
  _zwIDBStore.prototype.deleteIndex = function (name) {
    this._assertUsable(true);
    var metadata = this._indexes[name];
    if (metadata) metadata.deleted = true;
    delete this._indexes[name];
  };
  _zwIDBStore.prototype.index = function (name) {
    this._assertUsable(false);
    name = String(name);
    var idx = this._indexes[name];
    if (!idx) {
      throw new globalThis.DOMException('The index does not exist.', 'NotFoundError');
    }
    return new _zwIDBIndex(this, name, idx);
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
    this.name = name;
    this.keyPath = metadata.keyPath;
    this.unique = !!metadata.unique;
    this.multiEntry = !!metadata.multiEntry;
    this._metadata = metadata;
  }
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
  _zwIDBIndex.prototype.getAll = function (query, count) {
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
        var entries = index._entries(query, queryProvided);
        if (count !== undefined) entries = entries.slice(0, Math.max(0, Number(count)));
        _zwIDBDispatch(req, 'success', entries.map(function (entry) {
          return globalThis.structuredClone(entry.value);
        }));
      } catch (hostError) {
        _zwIDBRequestHostError(req, hostError);
      }
    };
    _zwIDBRunTransactionOperation(this.objectStore.transaction, perform);
    return req;
  };
  _zwIDBIndex.prototype.getAllKeys = function (query, count) {
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
        var entries = index._entries(query, queryProvided);
        if (count !== undefined) entries = entries.slice(0, Math.max(0, Number(count)));
        _zwIDBDispatch(req, 'success', entries.map(function (entry) {
          return globalThis.structuredClone(entry.primaryKey);
        }));
      } catch (hostError) {
        _zwIDBRequestHostError(req, hostError);
      }
    };
    _zwIDBRunTransactionOperation(this.objectStore.transaction, perform);
    return req;
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

  function _zwIDBTransaction(db, names, mode, deferCompletion) {
    var storeNames = Array.isArray(names) ? names.map(String) : [String(names)];
    this._db = db;
    this.db = db;
    this.mode = mode || 'readonly';
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
    return new _zwIDBStore(this._db, name, s.keyPath, s.autoIncrement, s.records, s.indexes, this, s);
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
    if (this._aborted || this._finished || this._committing) {
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
    return new _zwIDBStore(
      this,
      name,
      s.keyPath,
      s.autoIncrement,
      s.records,
      s.indexes,
      this._upgradeTransaction,
      s
    );
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
    delete this._stores[name];
    transaction._scope = transaction._scope.filter(function (entry) { return entry !== name; });
  };
  _zwIDBDatabase.prototype.transaction = function (names, mode) {
    // https://w3c.github.io/IndexedDB/#dom-idbdatabase-transaction
    if (this._closed) {
      throw new globalThis.DOMException('The database connection is closed.', 'InvalidStateError');
    }
    mode = mode === undefined ? 'readonly' : String(mode);
    if (mode !== 'readonly' && mode !== 'readwrite') {
      throw new TypeError('The transaction mode is invalid.');
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
    return new _zwIDBTransaction(this, storeNames, mode);
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
      _zwIDBEmit(transaction, 'abort', new _zwIDBEvent('abort', transaction));
      _zwIDBEmit(db, 'abort', new _zwIDBEvent('abort', db));
      req.result = undefined;
      req.error = transaction._hostError
        || new globalThis.DOMException('The version change transaction was aborted.', 'AbortError');
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
    return new _zwIDBKeyRange(lower, upper, lowerOpen, upperOpen);
  };
  _zwIDBKeyRange.only = function (value) {
    return new _zwIDBKeyRange(value, value, false, false);
  };
  _zwIDBKeyRange.lowerBound = function (lower, open) {
    return new _zwIDBKeyRange(lower, undefined, open, false);
  };
  _zwIDBKeyRange.upperBound = function (upper, open) {
    return new _zwIDBKeyRange(undefined, upper, false, open);
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
      var req = new _zwIDBRequest(null);
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
            db._upgradeTransaction = transaction;
            req.transaction = transaction;
            var upgrade = function () {
              transaction._active = true;
              req.readyState = 'done';
              req.result = db;
              _zwIDBEmit(
                req,
                'upgradeneeded',
                _zwIDBVersionEvent('upgradeneeded', req, oldVersion, requestedVersion)
              );
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
      var req = new _zwIDBRequest(null);
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
      var second = _zwIDBKey(b, []);
      if (!first || !second) {
        throw new globalThis.DOMException('The supplied value is not a valid key.', 'DataError');
      }
      return _zwIDBCompareKeys(first, second);
    },
  };
  // IDB 构造器占位（feature-detection / instanceof 用，rare）。
  globalThis.IDBKeyRange = _zwIDBKeyRange;
  globalThis.IDBRequest = _zwIDBRequest;
  globalThis.IDBOpenDBRequest = _zwIDBRequest;
  globalThis.IDBCursor = _zwIDBCursor;
  globalThis.IDBCursorWithValue = _zwIDBCursorWithValue;
  if (typeof Symbol !== 'undefined' && Symbol.toStringTag) {
    Object.defineProperty(
      _zwIDBRequest.prototype,
      Symbol.toStringTag,
      { configurable: true, value: 'IDBRequest' }
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
