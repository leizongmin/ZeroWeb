  // FR-009：媒体资源状态接口常量。元素 proxy 的实例读由 get trap 提供；构造器仅暴露
  // Web IDL 静态常量，MediaError 实例用于失败状态。
  function HTMLMediaElement() { throw new TypeError('Illegal constructor'); }
  HTMLMediaElement.NETWORK_EMPTY = 0; HTMLMediaElement.NETWORK_IDLE = 1;
  HTMLMediaElement.NETWORK_LOADING = 2; HTMLMediaElement.NETWORK_NO_SOURCE = 3;
  HTMLMediaElement.HAVE_NOTHING = 0; HTMLMediaElement.HAVE_METADATA = 1;
  HTMLMediaElement.HAVE_CURRENT_DATA = 2; HTMLMediaElement.HAVE_FUTURE_DATA = 3;
  HTMLMediaElement.HAVE_ENOUGH_DATA = 4;
  globalThis.HTMLMediaElement = globalThis.HTMLMediaElement || HTMLMediaElement;
  function HTMLTrackElement() { throw new TypeError('Illegal constructor'); }
  HTMLTrackElement.NONE = 0; HTMLTrackElement.LOADING = 1;
  HTMLTrackElement.LOADED = 2; HTMLTrackElement.ERROR = 3;
  globalThis.HTMLTrackElement = globalThis.HTMLTrackElement || HTMLTrackElement;
  function MediaError() { throw new TypeError('Illegal constructor'); }
  MediaError.MEDIA_ERR_ABORTED = 1; MediaError.MEDIA_ERR_NETWORK = 2;
  MediaError.MEDIA_ERR_DECODE = 3; MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED = 4;
  MediaError.prototype.MEDIA_ERR_ABORTED = 1; MediaError.prototype.MEDIA_ERR_NETWORK = 2;
  MediaError.prototype.MEDIA_ERR_DECODE = 3; MediaError.prototype.MEDIA_ERR_SRC_NOT_SUPPORTED = 4;
  globalThis.MediaError = globalThis.MediaError || MediaError;
  function _zwMediaError(code, message) {
    var error = Object.create(globalThis.MediaError.prototype);
    error.code = Number(code) || 0; error.message = String(message || '');
    return error;
  }

  // scroll 事件由实际滚动派发，不受页面 JS 影响）；② 两参数恒为数值（IPC delta），免 `_zwApplyScroll` 的
  // 对象/Number 归一分支。host 经 `script_user_scroll` 注入，typeof 守卫防 shim 未安装时 ReferenceError。
  globalThis.__zw_user_scroll = function (dx, dy) {
    _zwApplyScroll(_winScroll, Number(dx) || 0, Number(dy) || 0, true);
    _zwFireScroll(null, null, null);
  };
  // R3254：宿主「视口尺寸变化」（renderer 收到 browser IPC SetViewportParams）注入钩子——更新
  // `innerWidth/innerHeight`（+ outer，headless outer≈inner）使响应式 JS 读到新尺寸 + 派 'resize' 事件
  // 到 window（window.addEventListener('resize') / innerWidth watcher / matchMedia 触发依赖）。spec：resize
  // 不冒泡（bubbles=false），派到 window（globalThis.dispatchEvent）。host 经 `script_user_resize` 注入。
  globalThis.__zw_user_resize = function (w, h) {
    w = Number(w) || 0; h = Number(h) || 0;
    if (w < 0) w = 0; if (h < 0) h = 0;
    globalThis.innerWidth = w; globalThis.innerHeight = h;
    globalThis.outerWidth = w; globalThis.outerHeight = h;
    try { if (typeof globalThis.dispatchEvent === 'function') globalThis.dispatchEvent(_makeEvent('resize')); } catch (_e) {}
    // R3255：resize 后重评估 matchMedia MQL——matches 翻转的派 'change'（响应式断点 JS 依赖）。typeof 守卫
    //（_zwFireMqlChanges 在 part05 定义，shim 完整加载后可见；运行时调用必已加载）。
    if (typeof _zwFireMqlChanges === 'function') _zwFireMqlChanges();
  };

  // window 弹窗 / 对话框 API（R2979）——alert/confirm/prompt/open 此前全缺，`if (confirm('Delete?'))` /
  // `alert(err)` / `prompt('Name')` / `window.open(url)` 抛 ReferenceError 中断后续脚本。headless 无 UI 用户
  // 交互 → spec 合规的 dismiss 语义：alert 返 undefined（不阻塞，real 浏览器阻塞 headless 无）；confirm 返 false
  //（无用户点 OK = dismiss）；prompt 返 null（无用户输入 = dismiss，spec）；open 返 null（headless 弹窗被阻 =
  // popup-blocked 语义，`if (win)` 守卫自然跳过）。modern 站点的离开页守卫 / 表单确认 / OAuth 弹窗高频。
  globalThis.alert = globalThis.alert || function alert(_message) {};
  globalThis.confirm = globalThis.confirm || function confirm(_message) { return false; };
  globalThis.prompt = globalThis.prompt || function prompt(_message, _defaultValue) { return null; };
  globalThis.open = globalThis.open || function open(_url, _target, _features) { return null; };
  // window.print() / window.stop()（R3246）—— HTML §4.5.6 / Window 接口
  //（https://html.spec.whatwg.org/multipage/window-object.html#dom-print / #dom-stop）。
  // print：提示用户打印页面（headless 无打印机 → no-op；不抛）。stop：中止文档加载（headless JS 执行时
  // 文档已加载完毕 → 无进行中加载可中止 → no-op；不抛）。两者此前全缺，`window.print()`（打印按钮 /
  // 发票 / 收据页高频）/ `window.stop()`（慢加载中止 / 广告拦截 / abort 逻辑）抛 TypeError 中断后续脚本。
  // 同 alert/confirm/prompt/open（R2979）的 headless dismiss/no-op 语义。guard `||` 幂等（不覆盖既有定义）。
  globalThis.print = globalThis.print || function print() {};
  globalThis.stop = globalThis.stop || function stop() {};

  // Performance API（R2768 now + R2821 mark/measure/entry buffer + PerformanceObserver）——
  // DOMHighResTimeStamp（ms，自 time origin 起单调）。host `__zw_performance_now` 返 elapsed ms（子毫秒）；
  // 未注册（polyfill/reftest 路径）走 Date.now() 兜底。mark/measure 产 PerformanceEntry 存 entry buffer，
  // 经 getEntries/getEntriesByType/getEntriesByName 读；PerformanceObserver observe 匹配 entryType 时
  // 经 _defer microtask 异步派发（execute 末 checkpoint，同 R2774/R2814）。analytics/RUM（web-vitals /
  // Sentry / GA）高频。
  function _perfNow() {
    return typeof __zw_performance_now === 'function'
      ? Number(__zw_performance_now())
      : (typeof Date.now === 'function' ? Date.now() : 0);
  }
  // entry buffer + mark startTime 表 + 活跃 observer 表（shim IIFE 内部，不污染 globalThis）。
  var _perfEntries = [];
  var _perfMarks = {};
  var _perfObservers = [];
  // 解析 measure 的 start/end 标记：undefined→（end 用 now / start 用 0）/ number→原值 / string→marks 表查
  // （查无抛 TypeError，spec 一致：measure 引用未注册 mark 名应报错；正确用法先 mark 后 measure）。
  function _resolveMarkTime(mark, isEnd) {
    if (mark === undefined) return isEnd ? _perfNow() : 0;
    if (typeof mark === 'number') return mark;
    if (Object.prototype.hasOwnProperty.call(_perfMarks, mark)) return _perfMarks[mark];
    throw new TypeError("Failed to execute 'measure' on 'Performance': The mark '" + mark + "' does not exist.");
  }
  // observer 派发用 entry list（getEntries/getEntriesByType/getEntriesByName over 传入快照）。
  function _makeObserverList(entries) {
    return {
      getEntries: function () { return entries.slice(); },
      getEntriesByType: function (t) {
        return entries.filter(function (e) { return e.entryType === t; });
      },
      getEntriesByName: function (n, t) {
        return entries.filter(function (e) { return e.name === n && (t === undefined || e.entryType === t); });
      },
    };
  }
  // 新 entry 入 buffer 时，向所有 observe 该 entryType 的活跃 observer 排队，每 observer 至多一个 microtask flush
  // （去抖：pending 期间累积，单次 flush 一次性派发全部 buffered）。
  function _notifyEntry(entry) {
    for (var i = 0; i < _perfObservers.length; i++) {
      var obs = _perfObservers[i];
      if (obs._types.indexOf(entry.entryType) !== -1) {
        obs._buffered.push(entry);
        if (!obs._pending) {
          obs._pending = true;
          (function (o) {
            _defer(function () {
              o._pending = false;
              var recs = o._buffered;
              o._buffered = [];
              o._cb(_makeObserverList(recs));
            });
          })(obs);
        }
      }
    }
  }

  globalThis.performance = globalThis.performance || {
    now: _perfNow,
    // timeOrigin = 0（相对原点：now() 返自原点起 elapsed ms；绝对 epoch 语义未提供，文档记录）。
    timeOrigin: 0,
    mark: function (name) {
      var entry = { name: String(name), entryType: 'mark', startTime: _perfNow(), duration: 0 };
      _perfEntries.push(entry);
      _perfMarks[entry.name] = entry.startTime;
      _notifyEntry(entry);
      return entry;
    },
    measure: function (name, startMark, endMark) {
      var start = _resolveMarkTime(startMark, false);
      var end = _resolveMarkTime(endMark, true);
      var entry = { name: String(name), entryType: 'measure', startTime: start, duration: end - start };
      _perfEntries.push(entry);
      _notifyEntry(entry);
      return entry;
    },
    getEntries: function () { return _perfEntries.slice(); },
    getEntriesByType: function (type) {
      return _perfEntries.filter(function (e) { return e.entryType === type; });
    },
    getEntriesByName: function (name, type) {
      return _perfEntries.filter(function (e) {
        return e.name === name && (type === undefined || e.entryType === type);
      });
    },
    clearMarks: function (name) {
      _perfEntries = _perfEntries.filter(function (e) {
        return !(e.entryType === 'mark' && (name === undefined || e.name === name));
      });
      if (name === undefined) { _perfMarks = {}; }
      else { delete _perfMarks[name]; }
    },
    clearMeasures: function (name) {
      _perfEntries = _perfEntries.filter(function (e) {
        return !(e.entryType === 'measure' && (name === undefined || e.name === name));
      });
    },
  };

  // PerformanceObserver（R2821）——观察 performance entry（mark/measure/longtask/paint/navigation/resource 等）。
  // observe({entryTypes:[...]} 或 {type:'...'}) 注册 entryType；新 entry 经 _notifyEntry 排队，每 observer 至多
  // 一个 _defer microtask flush（spec 为任务队列派发，sandbox 经 execute 末 microtask 近似）；disconnect 移出
  // 活跃表停止派发；takeRecords 取并清缓冲；supportedEntryTypes 静态（feature-detect 高频）。
  function PerformanceObserver(callback) {
    this._cb = callback;
    this._types = [];
    this._buffered = [];
    this._pending = false;
  }
  PerformanceObserver.prototype.observe = function (options) {
    var t = (options && options.entryTypes)
      ? options.entryTypes
      : (options && options.type ? [options.type] : []);
    for (var i = 0; i < t.length; i++) {
      if (this._types.indexOf(t[i]) === -1) this._types.push(t[i]);
    }
    if (_perfObservers.indexOf(this) === -1) _perfObservers.push(this);
  };
  PerformanceObserver.prototype.disconnect = function () {
    this._types = [];
    this._buffered = [];
    var idx = _perfObservers.indexOf(this);
    if (idx !== -1) _perfObservers.splice(idx, 1);
  };
  PerformanceObserver.prototype.takeRecords = function () {
    var r = this._buffered;
    this._buffered = [];
    return r;
  };
  PerformanceObserver.supportedEntryTypes = ['element', 'event', 'first-input', 'largest-contentful-paint', 'longtask', 'mark', 'measure', 'navigation', 'paint', 'resource'];
  globalThis.PerformanceObserver = PerformanceObserver;

  // DOMException——Web IDL 异常类型（name + message + legacy code）。众多 Web API 抛出它（fetch /
  // storage / atob / crypto / structuredClone 等），各 API 用 name 子类区分语义（InvalidCharacterError
  // / DataCloneError / QuotaExceededError 等）。V8 embed 不提供，polyfill 之（本地 Chromium 150 oracle
  // 锚定 R2776）。**关键行为（oracle 锚定）**：无 name 参数时 name='Error'/code=0；name∈legacy 表时
  // code=对应值（余 0）；instance 非 Error 子类（浏览器 DOMException 亦非 Error 子类）；toString="name: message"。
  var _ZW_DE_CODE = {
    IndexSizeError: 1, HierarchyRequestError: 3, WrongDocumentError: 4,
    InvalidCharacterError: 5, NoModificationAllowedError: 7, NotFoundError: 8,
    NotSupportedError: 9, InUseAttributeError: 10, InvalidStateError: 11,
    SyntaxError: 12, InvalidModificationError: 13, NamespaceError: 14,
    InvalidAccessError: 15, TypeMismatchError: 17, SecurityError: 18,
    NetworkError: 19, AbortError: 20, URLMismatchError: 21, QuotaExceededError: 22,
    TimeoutError: 23, InvalidNodeTypeError: 24, DataCloneError: 25
  };
  function DOMException(message, name) {
    // 允许无 new 调用（同 Error 语义）。
    var self = (this instanceof DOMException) ? this : Object.create(DOMException.prototype);
    self.message = (message === undefined) ? '' : String(message);
    self.name = (name === undefined) ? 'Error' : String(name);
    self.code = _ZW_DE_CODE[self.name] || 0;
    return self;
  }
  DOMException.prototype = Object.create(Object.prototype);
  DOMException.prototype.constructor = DOMException;
  DOMException.prototype.toString = function () {
    return this.message === '' ? this.name : this.name + ': ' + this.message;
  };
  // legacy 常量（Web IDL §1.2 code 值；部分码无现代 name，仅常量）。
  DOMException.INDEX_SIZE_ERR = 1;
  DOMException.DOMSTRING_SIZE_ERR = 2;
  DOMException.HIERARCHY_REQUEST_ERR = 3;
  DOMException.WRONG_DOCUMENT_ERR = 4;
  DOMException.INVALID_CHARACTER_ERR = 5;
  DOMException.NO_DATA_ALLOWED_ERR = 6;
  DOMException.NO_MODIFICATION_ALLOWED_ERR = 7;
  DOMException.NOT_FOUND_ERR = 8;
  DOMException.NOT_SUPPORTED_ERR = 9;
  DOMException.INUSE_ATTRIBUTE_ERR = 10;
  DOMException.INVALID_STATE_ERR = 11;
  DOMException.SYNTAX_ERR = 12;
  DOMException.INVALID_MODIFICATION_ERR = 13;
  DOMException.NAMESPACE_ERR = 14;
  DOMException.INVALID_ACCESS_ERR = 15;
  DOMException.VALIDATION_ERR = 16;
  DOMException.TYPE_MISMATCH_ERR = 17;
  DOMException.SECURITY_ERR = 18;
  DOMException.NETWORK_ERR = 19;
  DOMException.ABORT_ERR = 20;
  DOMException.URL_MISMATCH_ERR = 21;
  DOMException.QUOTA_EXCEEDED_ERR = 22;
  DOMException.TIMEOUT_ERR = 23;
  DOMException.INVALID_NODE_TYPE_ERR = 24;
  DOMException.DATA_CLONE_ERR = 25;
  globalThis.DOMException = globalThis.DOMException || DOMException;

  // spec DOM `validate`（Name production，`dom-document-createelement`）：判定 String(tag) 后的名
  // 是否合法 createElement 标签名。与 native dom_bindings is_valid_qualified_name 逻辑对齐（A/B 等价）：
  // 空串 → 非法；首字符须 name-start（ASCII 字母 / `_` / `:` / 非 ASCII）；后续须 name-char（name-start
  // 或数字 / `-` / `.`）。createElement(undefined)→"undefined" 合法通过（WPT valid 列表）。
  function _zwIsNameStartChar(c) {
    return /[A-Za-z_:]/.test(c) || c.charCodeAt(0) >= 0x80;
  }
  function _zwIsNameChar(c) {
    return _zwIsNameStartChar(c) || /[0-9.\-]/.test(c);
  }
  function _zwIsValidQualifiedName(name) {
    if (name === '') return false;
    var chars = Array.from(name);
    if (!_zwIsNameStartChar(chars[0])) return false;
    for (var i = 1; i < chars.length; i++) {
      if (!_zwIsNameChar(chars[i])) return false;
    }
    return true;
  }
  // js-dom M4 R81：HTML createElement 的校验面（WPT Document-createElement valid 列表）——
  // 比 QName 宽：Name production（HTML any-name——`'}'`、`'<'`、`'\uffff'` 等在**非首字符**
  // 合法；首字符限制同 NameStartChar）。区别：QName 校验（createElementNS）拒绝这些；HTML
  // createElement 只要求整体是 Name（浏览器 HTML parser 的宽容性）。首字符仍须 NameStartChar
  // （"1foo"/"}foo"/"<foo" invalid）。
  function _zwIsValidHtmlElementName(name) {
    if (name === '') return false;
    // R81 修正：空白（"fo o"）与 '>'（"foo>"——invalid 列表）拒绝；'}'/'<'/'\uffff' 在非首
    // 字符合法（valid 列表实测）。首字符 NameStartChar。
    if (/[\s>]/.test(name)) return false;
    var chars = Array.from(name);
    if (!_zwIsNameStartChar(chars[0])) return false;
    return true;
  }

  // atob/btoa——Base64 编解码（Web 平台高频：data: URL / JWT / 二进制载荷）。纯 JS（ZW 无 base64
  // crate 在 engine，复用 fetch _b64decode 同款算法）。btoa 对 >255（非 Latin-1）抛 InvalidCharacterError
  // DOMException（spec，R2776 升级自裸 Error）；atob 容错（忽略空白/padding，best-effort）。多字节 UTF-8
  // base64 为已知限制（返 Latin-1）。
  var _b64ch = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  var _b64lut = (function () { var l = {}; for (var i = 0; i < 64; i++) l[_b64ch[i]] = i; return l; })();
  globalThis.btoa = function (s) {
    s = String(s);
    var out = '';
    for (var i = 0; i < s.length; i += 3) {
      var b1 = s.charCodeAt(i), b2 = s.charCodeAt(i + 1), b3 = s.charCodeAt(i + 2);
      if (b1 > 255 || b2 > 255 || b3 > 255)
        throw new DOMException('The string to be encoded contains characters outside of the Latin1 range.', 'InvalidCharacterError');
      out += _b64ch[b1 >> 2];
      out += _b64ch[((b1 & 3) << 4) | (isNaN(b2) ? 0 : b2 >> 4)];
      out += isNaN(b2) ? '=' : _b64ch[((b2 & 15) << 2) | (isNaN(b3) ? 0 : b3 >> 6)];
      out += isNaN(b3) ? '=' : _b64ch[b3 & 63];
    }
    return out;
  };
  globalThis.atob = function (s) {
    s = String(s).replace(/\s+/g, '').replace(/=+$/, '');
    var out = '';
    for (var i = 0; i < s.length; i += 4) {
      var b0 = _b64lut[s[i]] || 0, b1 = _b64lut[s[i + 1]] || 0;
      var b2 = _b64lut[s[i + 2]], b3 = _b64lut[s[i + 3]];
      out += String.fromCharCode((b0 << 2) | (b1 >> 4));
      if (s[i + 2] !== undefined) out += String.fromCharCode(((b1 & 15) << 4) | ((b2 || 0) >> 2));
      if (s[i + 3] !== undefined) out += String.fromCharCode((((b2 || 0) & 3) << 6) | (b3 || 0));
    }
    return out;
  };

  // crypto——Web Crypto 随机源：randomUUID（UUID v4）+ getRandomValues（TypedArray 填充）。高频
  //（id 生成 / CSRF token / analytics / 密钥/IV 随机）。R2960 升级 CSPRNG：经 host
  // `__zw_crypto_get_random_values(n)`（getrandom crate，OS 随机）；host 未注册（engine polyfill / reftest 路径）
  // → 回退 Math.random（非 CSPRNG，仅非安全场景）。
  // 填 view 字节：host 足量则 OS-random，否则 Math.random 回退。
  function _zw_randomFill(view) {
    var csv = (typeof __zw_crypto_get_random_values === 'function')
      ? __zw_crypto_get_random_values(view.length) : '';
    var parts = csv ? csv.split(',') : null;
    if (parts && parts.length >= view.length) {
      for (var i = 0; i < view.length; i++) view[i] = +parts[i] & 0xff;
    } else {
      for (var k = 0; k < view.length; k++) view[k] = (Math.random() * 256) | 0;
    }
    return view;
  }
  globalThis.crypto = globalThis.crypto || {
    // randomUUID（UUID v4，RFC 4122）：16 随机字节（_zw_randomFill，OS-random R2960），设 version(4)/variant
    // 位，格式化 8-4-4-4-12 hex。spec：time_hi_and_version 高 4 位=4，clock_seq_hi variant=10xxxxxx。
    randomUUID: function () {
      var b = new Uint8Array(16);
      _zw_randomFill(b);
      b[6] = (b[6] & 0x0f) | 0x40; // version 4
      b[8] = (b[8] & 0x3f) | 0x80; // variant 10xxxxxx（y ∈ 8,9,a,b）
      var h = '0123456789abcdef';
      var s = '';
      for (var i = 0; i < 16; i++) {
        s += h[(b[i] >> 4) & 0xf] + h[b[i] & 0xf];
        if (i === 3 || i === 5 || i === 7 || i === 9) s += '-';
      }
      return s;
    },
    // getRandomValues(typedArray)：spec 限定 TypedArray（Int8..Uint32 / BigInt64/BigUint64），≤65536
    // 字节。填**底层字节 buffer**（Uint8Array 视图）→ 任意 typed 视图得随机值（含多字节 / 共享 buffer 偏移）。
    getRandomValues: function (arr) {
      if (typeof ArrayBuffer === 'undefined' || !ArrayBuffer.isView(arr)) {
        throw new TypeError('getRandomValues: argument must be a TypedArray');
      }
      if (arr.byteLength > 65536)
        throw new DOMException("The ArrayBufferView byte length (" + arr.byteLength + ") exceeds 65536.", 'QuotaExceededError');
      _zw_randomFill(new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength));
      return arr;
    }
  };

  // BufferSource → number[]（字节值，0-255）：ArrayBuffer / TypedArray / DataView / array-like / string（经
  // TextEncoder）。供 crypto.subtle.digest 把 data 传 host（逗号分隔十进制串，避免 UTF-8 编码歧义）。
  function _zw_bufToBytes(data) {
    if (typeof data === 'string') data = new TextEncoder().encode(data);
    if (data == null) return [];
    var view;
    if (data instanceof ArrayBuffer) view = new Uint8Array(data);
    else if (data && data.buffer) view = new Uint8Array(data.buffer, data.byteOffset || 0, data.byteLength != null ? data.byteLength : data.length);
    else if (typeof data.length === 'number') view = data; // array-like（含 TypedArray 已覆盖上一支）
    else return [];
    var out = [];
    for (var i = 0; i < view.length; i++) out.push(view[i] & 0xff);
    return out;
  }
  // crypto.subtle（R2793 digest + R2955 HMAC sign/verify/importKey）。digest 委托 host
  // `__zw_crypto_subtle_digest`；HMAC sign/verify 委托 `__zw_crypto_subtle_hmac`（手写 HMAC，复用 sha1/sha2）。
  // **scope**：digest 全 hash；HMAC 全 hash（importKey raw + sign + verify）；其余 WebCrypto（RSA/ECDSA/AES/
  // HKDF/PBKDF2/jwk/exportKey）仍 defer——大表面，HMAC 为对称 MAC 最高频子集（JWT HS256 / 请求签名 / webhook 校验）。
  // https://w3c.github.io/webcrypto/#SubtleCrypto-method-sign  https://datatracker.ietf.org/doc/html/rfc2104

  // CryptoKey——密钥对象（importKey 返回值）。type（"secret"/"public"/"private"）+ extractable +
  // algorithm（归一化对象）+ usages（字符串数组）。HMAC 密钥材料存 `_raw`（字节 number[]）——polyfill：host
  // 每次 sign/verify 用，不在 host 持久化（headless 简化）；`extractable=false` 时 exportKey 仍 defer，
  // 故 _raw 技术可访问（exportKey 未实现，无泄漏面）。
  // https://w3c.github.io/webcrypto/#CryptoKey-interface
  function CryptoKey(type, extractable, algorithm, usages, raw) {
    this.type = type;
    this.extractable = !!extractable;
    this.algorithm = algorithm;
    this.usages = usages;
    this._raw = raw || null;
  }

  // hash 名归一化：接受串或 {name:...} → 大写 "SHA-XXX"，或 null。
  function _zw_hashName(h) {
    var n = (typeof h === 'object' && h) ? h.name : h;
    if (n == null) return null;
    return String(n).toUpperCase();
  }

  // importKey 的 algorithm 归一化：{name:"HMAC", hash:"SHA-XXX"} / {name:"PBKDF2"} / null（unsupported）。
  // HMAC 需 hash；PBKDF2 不需（hash 在 deriveBits 参数里）。
  function _zw_normalizeImportAlgorithm(algo) {
    if (!algo) return null;
    var name = (typeof algo === 'object' && algo) ? algo.name : algo;
    if (!name) return null;
    name = String(name).toUpperCase();
    if (name === 'HMAC') {
      var hash = _zw_hashName(typeof algo === 'object' ? algo.hash : null);
      if (!hash) return null;
      return { name: 'HMAC', hash: hash };
    }
    if (name === 'PBKDF2') {
      return { name: 'PBKDF2' };
    }
    if (name === 'AES-GCM') {
      return { name: 'AES-GCM' };
    }
    if (name === 'HKDF') {
      return { name: 'HKDF' };
    }
    return null;
  }

  // usages 归一化：去重 + 仅保留 allowed 内项；含非法项 → null（reject SyntaxError）。
  function _zw_normalizeUsages(usages, allowed) {
    if (usages == null) usages = [];
    if (typeof usages.length !== 'number') return null;
    var out = [], seen = {};
    for (var i = 0; i < usages.length; i++) {
      var u = String(usages[i]);
      if (allowed.indexOf(u) < 0) return null;
      if (!seen[u]) {
        seen[u] = 1;
        out.push(u);
      }
    }
    return out;
  }

  // HMAC MAC 计算（sign/verify 复用）：返 Uint8Array；host 未注册 / unsupported hash → 调 reject 返 null。
  function _zw_hmacMac(algo, key, dataBytes, reject) {
    if (typeof __zw_crypto_subtle_hmac !== 'function') {
      reject(new DOMException('crypto.subtle HMAC requires host callback', 'NotSupportedError'));
      return null;
    }
    var keyCsv = (key._raw || []).map(String).join(',');
    var macCsv = __zw_crypto_subtle_hmac(algo.hash, keyCsv, dataBytes.join(','));
    if (!macCsv) {
      reject(new DOMException("Unsupported HMAC hash: '" + algo.hash + "'", 'NotSupportedError'));
      return null;
    }
    var parts = macCsv.split(',');
    var arr = new Uint8Array(parts.length);
    for (var i = 0; i < parts.length; i++) arr[i] = +parts[i];
    return arr;
  }

  // AES-GCM 调用（encrypt/decrypt 共用）：校验 iv(12B)/tagLength(128)/host 后调 `__zw_crypto_subtle_aes_gcm`，
  // 返 Uint8Array；失败调 reject 返 null。AAD 经 algorithm.additionalData（可选）。
  function _zw_aesGcmCall(op, algorithm, key, dataBytes, reject) {
    var iv = _zw_bufToBytes(algorithm.iv);
    if (iv.length !== 12) {
      reject(new DOMException('AES-GCM iv must be 12 bytes (96 bits)', 'OperationError')); return null;
    }
    if (algorithm.tagLength != null && (algorithm.tagLength | 0) !== 128) {
      reject(new DOMException('Only AES-GCM tagLength=128 supported', 'NotSupportedError')); return null;
    }
    var aadBytes = algorithm.additionalData != null ? _zw_bufToBytes(algorithm.additionalData) : [];
    if (typeof __zw_crypto_subtle_aes_gcm !== 'function') {
      reject(new DOMException('crypto.subtle AES-GCM requires host callback', 'NotSupportedError')); return null;
    }
    var keyCsv = (key._raw || []).map(String).join(',');
    var out = __zw_crypto_subtle_aes_gcm(op, keyCsv, iv.join(','), dataBytes.join(','), aadBytes.join(','));
    if (!out) {
      reject(new DOMException('AES-GCM ' + op + ' failed (bad key/iv/tag)', 'OperationError')); return null;
    }
    var parts = out.split(',');
    var res = new Uint8Array(parts.length);
    for (var i = 0; i < parts.length; i++) res[i] = +parts[i];
    return res;
  }

  // 派生核心（deriveBits/deriveKey 共用）：PBKDF2/HKDF 分派 + host 调用 + csv→arr，**不做 usage 校验**
  //（usage 校验由调用方负责——deriveBits 检 "deriveBits"，deriveKey 检 "deriveKey"）。
  function _zw_performDerive(algorithm, key, length) {
    return new Promise(function (resolve, reject) {
      var name = (typeof algorithm === 'object' && algorithm) ? algorithm.name : algorithm;
      name = String(name == null ? '' : name).toUpperCase();
      var dkLen = length / 8;
      var keyCsv = (key._raw || []).map(String).join(',');
      var hash = _zw_hashName(typeof algorithm === 'object' ? algorithm.hash : null);
      var saltBytes = _zw_bufToBytes(algorithm.salt);
      var out = '';
      if (name === 'PBKDF2') {
        var iters = Math.floor(Number(algorithm.iterations));
        if (!hash || !(iters > 0)) {
          reject(new DOMException('PBKDF2 requires salt/iterations/hash', 'OperationError')); return;
        }
        if (typeof __zw_crypto_subtle_pbkdf2 !== 'function') {
          reject(new DOMException('crypto.subtle deriveBits requires host callback', 'NotSupportedError')); return;
        }
        out = __zw_crypto_subtle_pbkdf2(hash, keyCsv, saltBytes.join(','), String(iters), String(dkLen));
      } else { // HKDF
        if (!hash || typeof __zw_crypto_subtle_hkdf !== 'function') {
          reject(new DOMException('HKDF requires hash + host callback', 'NotSupportedError')); return;
        }
        var infoBytes = _zw_bufToBytes(algorithm.info);
        out = __zw_crypto_subtle_hkdf(hash, keyCsv, saltBytes.join(','), infoBytes.join(','), String(dkLen));
      }
      if (!out) {
        reject(new DOMException("Unsupported deriveBits hash: '" + hash + "'", 'NotSupportedError')); return;
      }
      var parts = out.split(',');
      var arr = new Uint8Array(parts.length);
      for (var i = 0; i < parts.length; i++) arr[i] = +parts[i];
      resolve(arr);
    });
  }

  // 派生/生成的目标密钥长度（位）。AES → 256（spec 默认）；HMAC → hash 块大小（SHA-1/256=512，SHA-384/512=1024）。
  function _zw_keyLengthBits(algo) {
    var n = String((typeof algo === 'object' && algo) ? algo.name : algo).toUpperCase();
    if (n === 'AES-GCM' || n === 'AES-CBC' || n === 'AES-CTR' || n === 'AES-KW') return 256;
    if (n === 'HMAC') {
      var h = _zw_hashName(typeof algo === 'object' ? algo.hash : null);
      return (h === 'SHA-384' || h === 'SHA-512') ? 1024 : 512;
    }
    return 0; // 未知
  }

  // n 个随机字节（Uint8Array）。复用 crypto.getRandomValues（R2770，**Math.random 非 CSPRNG**——安全敏感场景已知限制）。
  function _zw_randomBytes(n) {
    var a = new Uint8Array(n);
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) crypto.getRandomValues(a);
    else for (var i = 0; i < n; i++) a[i] = (Math.random() * 256) | 0;
    return a;
  }

  globalThis.crypto.subtle = globalThis.crypto.subtle || {
    digest: function (algo, data) {
      var a = (typeof algo === 'object' && algo) ? algo.name : algo;
      a = (a == null ? '' : String(a)).toUpperCase();
      return new Promise(function (resolve, reject) {
        var bytes = _zw_bufToBytes(data);
        if (typeof __zw_crypto_subtle_digest !== 'function') {
          reject(new DOMException('crypto.subtle.digest requires host callback', 'NotSupportedError'));
          return;
        }
        var out = __zw_crypto_subtle_digest(a, bytes.join(','));
        if (!out) {
