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
  function _zw_utf8_decode(bytes) {
    var s = '';
    var i = 0;
    var n = bytes.length;
    while (i < n) {
      var b = bytes[i];
      if (b < 0x80) { s += String.fromCharCode(b); i += 1; }
      else if (b < 0xc2) { s += '�'; i += 1; } // 非法前导字节 / 连续字节
      else if (b < 0xe0) { s += String.fromCharCode(((b & 0x1f) << 6) | (bytes[i + 1] & 0x3f)); i += 2; }
      else if (b < 0xf0) { s += String.fromCharCode(((b & 0x0f) << 12) | ((bytes[i + 1] & 0x3f) << 6) | (bytes[i + 2] & 0x3f)); i += 3; }
      else {
        var cp = ((b & 0x07) << 18) | ((bytes[i + 1] & 0x3f) << 12) | ((bytes[i + 2] & 0x3f) << 6) | (bytes[i + 3] & 0x3f);
        cp -= 0x10000;
        s += String.fromCharCode(0xd800 + (cp >> 10), 0xdc00 + (cp & 0x3f)); // astral → 代理对
        i += 4;
      }
    }
    return s;
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
  };
  globalThis.TextDecoder.prototype = {
    encoding: 'utf-8',
    fatal: false,
    ignoreBOM: false,
    decode: function (buf) {
      var bytes;
      if (buf == null) bytes = new Uint8Array(0);
      else if (buf instanceof ArrayBuffer) bytes = new Uint8Array(buf);
      else if (buf && typeof buf.length === 'number') bytes = buf; // TypedArray / array-like
      else if (buf && buf.buffer) bytes = new Uint8Array(buf.buffer);
      else bytes = new Uint8Array(0);
      return _zw_utf8_decode(bytes);
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
  // tee（分叉）仍 defer（follow-up）。
  var _RS_DONE = { done: true, value: undefined };
  function _rs_chunk(value) { return { done: false, value: value }; }
  globalThis.ReadableStream = globalThis.ReadableStream || function ReadableStream(underlyingSource, _strategy) {
    if (!(this instanceof ReadableStream)) return new ReadableStream(underlyingSource, _strategy);
    var source = underlyingSource || {};
    var queue = [];              // 已 enqueue 待消费 chunk
    var state = 'readable';      // readable | closed | errored
    var errorVal = undefined;
    var waiting = [];            // 待 read() 的 {resolve, reject}
    var pulling = false;
    var self = this;
    this._locked = false;

    function enqueueChunk(chunk) {
      if (state !== 'readable') return;
      // 有等待中的 read → 直接 resolve（零拷贝绕 queue）；否则入队。
      if (waiting.length > 0) waiting.shift().resolve(_rs_chunk(chunk));
      else queue.push(chunk);
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
      // queue 空 + readable + 有 pull → 触发一次（pulling 守卫防重入）。pull 可 enqueue/close/error。
      if (pulling || state !== 'readable' || typeof source.pull !== 'function') return;
      pulling = true;
      try { source.pull(controller); } catch (_e) { errorStream(_e); }
      pulling = false;
    }
    var controller = {
      desiredSize: 1,
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
            if (queue.length > 0) { resolve(_rs_chunk(queue.shift())); return; }
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
    // start：同步源初始化（enqueue/close/error 可能在此调用，flush 已等待读者前的 chunk 入 queue）。
    if (typeof source.start === 'function') {
      try { source.start(controller); } catch (_e) { errorStream(_e); }
    }
  };
  // fetch 响应体字符串 → ReadableStream：单 UTF-8 Uint8Array chunk 后 close（headless finite-body 模型，
  // 整体 body 已就绪）。复用 _zw_utf8_encode；空 body → 直接 close（零 chunk）。定义在 part01 _makeResponse
  // 之前调用（runtime），ReadableStream（part02）+ _zw_utf8_encode（part02）在 IIFE 同作用域已就绪。
  function _bodyToStream(text) {
    var bodyText = text || '';
    return new ReadableStream({
      start: function (controller) {
        if (bodyText) {
          var bytes = _zw_utf8_encode(bodyText);
          var arr = new Uint8Array(bytes.length);
          for (var k = 0; k < bytes.length; k++) arr[k] = bytes[k];
          controller.enqueue(arr);
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
  globalThis.WritableStream = globalThis.WritableStream || function WritableStream(underlyingSink, _strategy) {
    if (!(this instanceof WritableStream)) return new WritableStream(underlyingSink, _strategy);
    var sink = underlyingSink || {};
    var state = 'writable';     // writable | closed | errored
    var errorVal = undefined;
    var self = this;
    this._locked = false;
    var resolveClosed, rejectClosed;
    var closedP = new Promise(function (res, rej) { resolveClosed = res; rejectClosed = rej; });
    var pendingWrites = [];       // FIFO {resolve, reject}：待 sink.write 完成的 write
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
        get ready() { return Promise.resolve(); },   // headless：无背压门控
        get desiredSize() { return state === 'writable' ? 1 : 0; },
        write: function (chunk) {
          if (state === 'errored') return Promise.reject(errorVal);
          if (state === 'closed') return Promise.reject(new TypeError('Cannot write to a closed WritableStream'));
          return new Promise(function (resolve, reject) {
            var entry = { resolve: resolve, reject: reject };
            pendingWrites.push(entry);
            try {
              Promise.resolve(sink.write ? sink.write(chunk, controller) : undefined)
                .then(function () {
                  // sink.write 完成：若期间 controller.error 已拒绝本 entry（state errored），跳过；
                  // 否则 FIFO 取本 entry resolve（多 write 串行完成，顺序匹配）。
                  if (state === 'errored') return;
                  if (pendingWrites.length > 0) pendingWrites.shift().resolve(undefined);
                },
                function (e) { errorStream(e); });
            } catch (e) { errorStream(e); }
          });
        },
        close: function () {
          if (state === 'errored') return Promise.reject(errorVal);
          if (state === 'closed') return Promise.resolve();
          state = 'closed';
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
  // **已知限制**：底层 TextDecoder.decode 无流式状态（每 chunk 独立解码，非跨 chunk 维护未完成字节序列），
  // 单 chunk（headless finite-body 模型）正确；chunk 边界切多字节 char 的流式场景近似（follow-up 需流式解码状态）。
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
        var s = dec.decode(chunk);
        if (s) controller.enqueue(s); // 空 string（如 chunk 末切多字节前导字节）跳过，避免空 chunk
      },
      flush: function (controller) {
        var s = dec.decode(); // 空参 = flush 剩余（headless 单 chunk 模型下通常 ''）
        if (s) controller.enqueue(s);
      }
    });
    this.encoding = dec.encoding || 'utf-8';
    this.fatal = !!dec.fatal;
    this.ignoreBOM = !!dec.ignoreBOM;
  };
  globalThis.TextDecoderStream.prototype = Object.create(globalThis.TransformStream.prototype);
  globalThis.TextDecoderStream.prototype.constructor = globalThis.TextDecoderStream;

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
  // pair-store 模式（`_p` = [[name,value]] 保序、允许重名）；纯 JS 自包含，零 host 回调。与 USP 不同：
  // ① value 经 `String(value)` 归一（spec 非 Blob 值转 USVString；Blob/File 未实现故恒字符串）；
  // ② append/set 接受可选 `filename`（仅对 Blob 值有意义，字符串值忽略——spec 一致）；
  // ③ 无 toString 序列化（FormData 不直接字符串化，由 fetch 消费为 multipart——fetch POST defer 故
  //    当前 end-to-end 仅构造/迭代，仍消除 `new FormData` ReferenceError 中断脚本）。
  // **已知限制（记录）**：constructor `form` 参数为 best-effort——若传入 `<form>` 元素，尝试枚举其
  // input/select/textarea 命名字段（checkbox/radio 仅 checked 入列），任一步失败静默跳过（不抛）；
  // 不覆盖 select-multiple / file input / disabled / form-attribute 等完整表单语义（renderer 路径
  // 真实字段枚举为 follow-up；多数库 `new FormData()` 空构造再 append，本路径完整支持）。
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
            if (f.checked) this._p.push([String(name), f.value != null ? String(f.value) : 'on']);
          } else if (type !== 'file' && type !== 'submit' && type !== 'button' && type !== 'reset' && type !== 'image') {
            this._p.push([String(name), f.value != null ? String(f.value) : '']);
          }
        }
      } catch (_e) { /* best-effort：枚举失败则按空 FormData */ }
    }
  };
  globalThis.FormData.prototype = {
    append: function (name, value /*, filename */) {
      // filename 仅对 Blob 值有意义（未实现 Blob），字符串值忽略——spec 一致。
      this._p.push([String(name), String(value)]);
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
    set: function (name, value /*, filename */) {
      name = String(name); value = String(value);
      var found = false; var out = [];
      for (var i = 0; i < this._p.length; i++) {
        if (this._p[i][0] === name) { if (!found) { out.push([name, value]); found = true; } }
        else out.push(this._p[i]);
      }
      if (!found) out.push([name, value]);
      this._p = out;
    },
    forEach: function (cb, thisArg) {
      for (var i = 0; i < this._p.length; i++) cb.call(thisArg, this._p[i][1], this._p[i][0], this);
    },
    entries: function () { return _zw_iter(this._p.map(function (e) { return [e[0], e[1]]; })); },
    keys: function () { return _zw_iter(this._p.map(function (e) { return e[0]; })); },
    values: function () { return _zw_iter(this._p.map(function (e) { return e[1]; })); }
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
  globalThis.Headers = globalThis.Headers || function Headers(init) {
    if (!(this instanceof Headers)) return new Headers(init);
    this._h = {}; // lowername -> string[]（保 append 序与多值）
    if (init == null) return;
    if (Array.isArray(init)) {
      for (var i = 0; i < init.length; i++) {
        var pair = init[i];
        if (pair && pair.length >= 2) this.append(pair[0], pair[1]);
      }
    } else if (typeof init.forEach === 'function') {
      // Headers-like（forEach 回调 (value, name, headers)）。
      var self = this;
      init.forEach(function (v, k) { self.append(k, v); });
    } else if (typeof init === 'object') {
      for (var k in init) {
        if (Object.prototype.hasOwnProperty.call(init, k)) this.append(k, init[k]);
      }
    }
  };
  globalThis.Headers.prototype = {
    append: function (name, value) {
      name = _hdrNorm(name);
      if (!name) return;
      (this._h[name] = this._h[name] || []).push(String(value));
    },
    delete: function (name) {
      delete this._h[_hdrNorm(name)];
    },
    get: function (name) {
      name = _hdrNorm(name);
      var v = this._h[name];
      return v && v.length ? v.join(', ') : null;
    },
    // getSetCookie：Set-Cookie 数组（spec 特例——get 合并 Set-Cookie 丢分隔，故单独返数组）。
    getSetCookie: function () {
      var v = this._h['set-cookie'];
      return v ? v.slice() : [];
    },
    has: function (name) {
      return Object.prototype.hasOwnProperty.call(this._h, _hdrNorm(name));
    },
    set: function (name, value) {
      name = _hdrNorm(name);
      if (!name) return;
      this._h[name] = [String(value)];
    },
    forEach: function (cb, thisArg) {
      for (var k in this._h) {
        if (Object.prototype.hasOwnProperty.call(this._h, k)) cb.call(thisArg, this._h[k].join(', '), k, this);
      }
    },
    entries: function () {
      var out = [];
      for (var k in this._h) {
        if (Object.prototype.hasOwnProperty.call(this._h, k)) out.push([k, this._h[k].join(', ')]);
      }
      return _zw_iter(out);
    },
    keys: function () {
      var out = [];
      for (var k in this._h) {
        if (Object.prototype.hasOwnProperty.call(this._h, k)) out.push(k);
      }
      return _zw_iter(out);
    },
    values: function () {
      var out = [];
      for (var k in this._h) {
        if (Object.prototype.hasOwnProperty.call(this._h, k)) out.push(this._h[k].join(', '));
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
  // **已知限制（记录）**：① string part 字节长按 UTF-8（_zw_utf8_encode length），与 spec 一致；
  //   ② slice 仅按 size 范围裁剪（不真正切字节——返浅拷原 parts 的 Blob，size 经 start/end clamp；
  //   足够 size 检查 / type 重设，真正字节级 slice 为 follow-up）；③ 无 stream()（Streams API defer）；
  //   ④ end-encoding 的 type 不解析 charset（原样小写）。File 未实现（File = Blob + name，follow-up）。
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
    // slice：返新 Blob（best-effort——按 size clamp，type 可重设；不真正切字节，浅拷原 parts）。
    slice: function (start, end, contentType) {
      var s = start != null ? (start | 0) : 0;
      if (s < 0) s = Math.max(0, this.size + s);
      var e = end != null ? (end | 0) : this.size;
      if (e < 0) e = Math.max(0, this.size + e);
      e = Math.min(e, this.size);
      var sz = s < e ? (e - s) : 0;
      var b = new Blob([], { type: contentType != null ? String(contentType) : this.type });
      b._parts = this._parts; // 浅拷（不真切字节；size 经 clamp 反映范围）
      b.size = sz;
      return b;
    },
    // text()：Promise<string>——拼接各 part 文本（string/字节经 TextDecoder/Blob 递归）。
    text: function () {
      var parts = this._parts;
      var pro = [];
      for (var i = 0; i < parts.length; i++) pro.push(Blob._partText(parts[i]));
      return Promise.all(pro).then(function (strs) { return strs.join(''); });
    },
    // arrayBuffer()：Promise<Uint8Array>——text() UTF-8 编码（字节视图）。
    arrayBuffer: function () {
      return this.text().then(function (s) { return _zw_utf8_encode(s); });
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
  function _hist_dispatchPopState() {
    // spec：back/forward/go 触发 popstate（pushState/replaceState 不触发），异步派发。
    var st = _hist_current().state;
    _defer(function () {
      var ev = new PopStateEvent('popstate', { state: st });
      ev.target = globalThis;
      _dispatchToListeners(_elKey('html', null), ev, 'all', globalThis);
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
  globalThis.history = {
    get length() { return _hist_entries.length; },
    get state() { return _hist_current().state; },
    get scrollRestoration() { return 'auto'; },
    set scrollRestoration(_v) { /* headless 无真滚动恢复，no-op */ },
    // pushState(state, unused, url?)：截断 forward entries + push 新 entry + 推进 cursor（不触发 popstate）。
    pushState: function (state, _unused, url) {
      _hist_entries = _hist_entries.slice(0, _hist_cursor + 1);
      _hist_entries.push({ state: state, url: url != null ? String(url) : _hist_current().url });
      _hist_cursor = _hist_entries.length - 1;
    },
    // replaceState(state, unused, url?)：原地替换当前 entry 的 state/url（不触发 popstate）。
    replaceState: function (state, _unused, url) {
      var cur = _hist_current();
      cur.state = state;
      if (url != null) cur.url = String(url);
    },
    back: function () { if (_hist_cursor > 0) { _hist_cursor--; _hist_dispatchPopState(); } },
    forward: function () { if (_hist_cursor < _hist_entries.length - 1) { _hist_cursor++; _hist_dispatchPopState(); } },
    go: function (delta) {
      var d = (delta == null) ? -1 : (delta | 0);
      var target = _hist_cursor + d;
      if (target < 0) target = 0;
      if (target > _hist_entries.length - 1) target = _hist_entries.length - 1;
      if (target !== _hist_cursor) { _hist_cursor = target; _hist_dispatchPopState(); }
    },
  };

  globalThis.Worker = function() {
  };

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
    userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) ZeroBrowser/0.1 Chrome/120.0.0.0',
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
    }
  };

  globalThis.console = globalThis.console || {
    log: function() {},
    info: function() {},
    warn: function() {},
    error: function() {},
    debug: function() {},
    trace: function() {},
    dir: function() {},
    clear: function() {},
    count: function() {},
    group: function() {},
    groupEnd: function() {},
    table: function() {}
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

  globalThis.XMLHttpRequest = function() {
    var self = this;
    self.readyState = 0;
