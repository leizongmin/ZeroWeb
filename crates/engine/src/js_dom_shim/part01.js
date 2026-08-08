(function() {
  var _listenerStore = {};
  // R2933 element 级 IDL on-event handler 存储（per-element-key → { eventType: fn }）。on* setter 把 fn
  // 同时记此 + 注册进 _listenerStore[key]（使 dispatchEvent 触发）；getter 返此存储 fn（或 null）。
  var _onHandlers = {};
  // P1b S2 incr3：元素 proxy 缓存——同一 (sel, handle) 复用同一 Proxy 实例，使
  // `querySelector('#t') === querySelector('#t')` 为真（node === identity，v8::External
  // 精修目标，但纯 JS Proxy 缓存即可达成，无需 rusty_v8 对象绑定）。proxy 无状态（仅委托
  // host 回调），缓存安全；key 复用 `_elKey`（@handle / sel），与 _listenerStore 同生命周期。
  var _proxyCache = {};
  // P1a form input：per-element-key value 缓存（`.value` 属性）。lazy-init 自 value 属性；
  // `.value` set 更新缓存 + 记 value 属性 mutation（供 render）。跨 execute 存活（typing 多键），
  // 导航（URL 变化）经 `__zw_reset_form_state` 清空防跨页 stale value。
  var _inputValues = {};
  // P1a classList：per-element-key class 缓存（`className` / `classList`）。同 _inputValues 动机——
  // classList.add/remove/toggle 旧实现每次读 stale snapshot 算新 class 再 SetAttr 整体替换，
  // 同脚本内连续 add 末次覆盖前次（`add('a');add('b')` 丢 'a'）。缓存累积全量，末次 SetAttr 携带
  // 正确值；className set 同步更新缓存保证一致。导航经 `__zw_reset_form_state` 清空。
  var _classCache = {};
  // Constraint Validation（R2825）：per-element-key 自定义校验消息（setCustomValidity 设置）。
  // 空串/未设=valid；非空=customError + validity.valid=false + validationMessage=msg。原生约束
  // （required/pattern/type 等）headless 不强制（permissive valid）。同 _inputValues/_classCache 经
  // `__zw_reset_form_state` 清空防跨页 stale。
  var _customValidity = {};
  // HTMLInputElement.files 空 FileList（R2830）：headless 无真文件 → 共享空 FileList（length 0 +
  // item→null + 可迭代）。上传表单读 `input.files.length` 不抛（无文件 → 0，跳过上传逻辑）。
  var _emptyFileList = {
    length: 0,
    item: function (_i) { return null; },
    [Symbol.iterator]: function* () {},
  };
  // HTMLInputElement.indeterminate（R2831）：JS-only IDL 布尔（**非 reflected attr**——无 indeterminate
  // 内容属性，纯 JS 状态）。checkbox「全选」tri-state UI 高频（父 checkbox 半选态）。per-element-key，
  // 默认 false。同 _inputValues/_classCache 经 `__zw_reset_form_state` 清空。
  var _indeterminate = {};
  // text-control 选区（selectionStart/End/Direction + setSelectionRange/select，R2844）：per-element-key 选区
  // 状态 { start, end, direction }。仅 text control（textarea + input text/search/url/tel/password）有真实选区；
  // 默认（未设）= {0, 0, 'forward'}（Chromium 150 oracle 锚定——未聚焦/未设的 text control 选区折叠在 0，非值末）。
  // headless 无真 caret/选择渲染，故 selection 为纯 JS 跟踪（供文本编辑器 / 自动选择 / Range 算法读状态）。
  // 同 _inputValues/_classCache 经 `__zw_reset_form_state` 清空。
  var _textSelection = {};
  // HTMLOutputElement（R2846）：value 独立于 textContent（<output> 按 children 渲染非 value；spec：设 .value
  // 不触碰 DOM text）。_outputDefault = 默认值（= 初始 textContent，lazy 捕获一次跨 value 变更稳定）；
  // _outputValue = dirty 后的当前值（key 存在即 dirty）。同 _inputValues 经 `__zw_reset_form_state` 清空。
  var _outputDefault = {};
  var _outputValue = {};
  // reflected 字符串/数值属性（title/lang/dir/tabindex）per-element-key 缓存。同 _inputValues/_classCache
  // 动机——`__zw_set_attr` 仅入队 mutation（异步 apply），同步 set→get 往返须客户端缓存（get 优先读缓存）。
  // 值结构：{ title?: string, lang?: string, dir?: string, tabindex?: number }。
  var _reflectedAttrs = {};
  // P1a DocumentFragment：已创建的 fragment handle 集合（nodeType=11 标识 + appendChild 时
  // flatten 检测）。fragment 为 create 句柄，无 selector，故用此 set 区别于普通元素句柄。
  var _fragmentHandles = {};
  // R2926 Shadow DOM（attachShadow，Tier 2 Web Components 地基）：host 元素 elKey → 其 shadow root
  //（{ handle, mode }）。shadow root 复用 DocumentFragment handle 容器（故 handle 亦入 _fragmentHandles），
  // 另入 _shadowHandles 标 shadow-root 身份（nodeName '#shadow-root' + host/mode）。host 元素调
  // attachShadow 建；shadowRoot getter 读（open 返 root / closed·未建 返 null，spec）。导航清空（页级）。
  var _shadowRoots = {};
  var _shadowHandles = {};
  var _shadowHandleMeta = {};
  // R2927 handle-children registry：容器 handle（shadow root / fragment）→ 其子节点 proxy 列表。
  // 这些容器无 selector（handle-only），既有 childNodes/children 经 `__zw_child_nodes(sel)` 读（须 sel）
  // → 恒返 []。本 registry 在 appendChild（容器父）时同步记录子节点，供 childNodes/firstChild/
  // lastChild/firstElementChild/lastElementChild/childElementCount 读。仅 handle-append 模式覆盖
  //（innerHTML 设内容经 host parse 无 handle，未跟踪——follow-up）。导航清空。
  var _handleChildren = {};
  // P1a Comment（R2816）：已创建的 comment handle 集合（nodeType=8 / nodeName '#comment' 标识）。
  // comment 为 create 句柄无 selector，故用此 set 区别于普通元素句柄（同 _fragmentHandles 模式）。
  var _commentHandles = {};
  // P1a Text（R2816）：已创建的 text handle 集合（nodeType=3 / nodeName '#text' 标识）——修正旧实现 created
  // text 节点误报 nodeType 1（element）的 bug（与 _commentHandles 对称）。createTextNode 经 __zw_create_text。
  var _textHandles = {};
  // ── 浏览器运行时桩（定时器、navigator、location 等）──
  var _timerId = 1;
  // queueMicrotask——调度 microtask（高频：每个异步库 / polyfill / 框架都用）。本 V8 embed 未暴露
  // 全局 queueMicrotask（probe 确认 undefined），用 `Promise.resolve().then` polyfill——V8 在 execute
  // 末 perform_microtask_checkpoint 派发，同 spec「当前 task 末、下 task 前」语义。亦使上方 _defer
  // 走真 queueMicrotask 分支（行为同 Promise.then fallback，零变化）。
  globalThis.queueMicrotask = globalThis.queueMicrotask || function (cb) {
    if (typeof cb !== 'function') throw new TypeError('queueMicrotask: callback not callable');
    Promise.resolve().then(cb);
  };

  // 单次脚本执行内 microtask 派发上限（避免 setTimeout 轮询在 checkpoint 中无限链式调度）。
  var _deferBudget = 256;

  function _defer(fn) {
    if (_deferBudget <= 0) return;
    _deferBudget--;
    if (typeof queueMicrotask === 'function') {
      queueMicrotask(function() { try { fn(); } catch (_e) {} });
    } else if (typeof Promise === 'function') {
      Promise.resolve().then(function() { try { fn(); } catch (_e) {} });
    } else {
      try { fn(); } catch (_e) {}
    }
  }

  // requestAnimationFrame / takeScreenshot 预算：单次脚本执行内同步派发上限，
  // 防止动画循环（rAF(loop)）无限链式触发；reftest 的「double-rAF 后 setup」
  // 模式只需 2-3 帧即可收敛。
  var _rafBudget = 64;

  // P1a 事件循环 slice 1（R2713a）：帧驱动 rAF kill-switch + 注册队列。
  // `__ZW_RAF_FRAME_DRIVEN` 由 host（worker init 读 env `ZW_RAF_FRAME_DRIVEN`）在 execute 前注入：
  // unset/false = 同步 stub（reftest 兼容，rAF 立即 fn(0)）；true = 帧驱动（rAF 注册到
  // `_rafPending`，render 后 host 调 `__zw_raf_tick(ts)` 派发）。OFF 时 `__zw_raf_tick` 早返零开销。
  // 详见 docs/goal/zero-web/p1a-event-loop-raf-slice-design-2026-08-05.md。
  globalThis.__ZW_RAF_FRAME_DRIVEN = globalThis.__ZW_RAF_FRAME_DRIVEN || false;
  var _rafPending = {}; // id -> fn（帧驱动路径注册队列；OFF 路径不填充）

  // P1a Slice 2b：observer 注册表——host render 后经 `__zw_observers_tick()` 对每个活跃
  // observer 调 `_schedule()`，使 IO/RO 在 cross-threshold / size-change 时派发后续通知
  // （observe 仅派发 initial；后续 render 的真实 layout 变化须 host tick 触发复算）。
  // IO/RO 构造时 push；tick 跳过无活跃 target 者（disconnect 后为 no-op）。
  // leak = observer 创建总数（有界，per-page；WeakRef 注册表为后续硬化 follow-up）。
  var _zwObservers = [];

  globalThis.__zw_begin_script = function() {
    _deferBudget = 256;
    _rafBudget = 64;
    // R2946：每页首次脚本执行前反射 <body on*> → window.on*（幂等，按 page URL 去重）。
    if (typeof _zw_reflect_body_window_handlers === 'function') _zw_reflect_body_window_handlers();
  };

  // P1b S1（方案 A）异步回调 resolve 通道（JS 侧契约）：
  // Rust 异步完成（fetch / timer 等后续切片接通）后经
  // `V8Sandbox::resolve_async_callback(id, result)` 执行 `__zwResolveCallback(id, result)`，
  // 从 pending 表取出 resolver 触发 Promise resolve。execute 末尾的 microtask
  // checkpoint 随即 drain `.then` 回调。pending 表 idempotent 初始化——跨脚本执行
  // 存活（resolve 可晚于注册），且 shim 重注入时不覆盖既有 pending 项。
  globalThis.__zw_pending = globalThis.__zw_pending || {};
  globalThis.__zwResolveCallback = function(id, result) {
    var r = globalThis.__zw_pending[id];
    if (typeof r === 'function') {
      delete globalThis.__zw_pending[id];
      r(result);
    }
  };

  // P1b S3 incr-c / R2923 fetch 完整化：fetch 返回 Response 对象（spec-compliance：ok/status/
  // statusText/headers/text()/json()）。host 经 `__zw_fetch` 抓取返 `__zwfr:` wire
  //（status\x1fstatusText\x1fheadersWire\x1fbody）或 `__zw_fetch_error:` 错误标记 → shim 包装为 Response。
  // body 为 wire 末字段（取第 3 个 \x1f 后全部，可含 \x1f）。错误 / 旧 body-only wire → 兜底 _makeResponse。
  function _parseHeadersWire(wire) {
    var out = {};
    if (!wire) return out;
    var parts = wire.split('\x1e');
    for (var i = 0; i + 1 < parts.length; i += 2) out[parts[i]] = parts[i + 1];
    return out;
  }
  // 旧 / 错误路径：body 为裸文本（status 200）或 `__zw_fetch_error:` 前缀（ok:false）。增 headers:{}（向后兼容）。
  function _makeResponse(body) {
    var ok = typeof body === 'string' && body.indexOf('__zw_fetch_error') !== 0;
    return {
      ok: ok,
      status: ok ? 200 : 0,
      statusText: ok ? 'OK' : 'Error',
      headers: {},
      // R2967：body 为 ReadableStream（lazy，单 UTF-8 chunk + close）。网络错误（ok:false）→ null（spec）。
      get body() {
        if (!ok) return null;
        if (!this._bs) this._bs = _bodyToStream(body);
        return this._bs;
      },
      text: function() { return Promise.resolve(ok ? body : ''); },
      json: function() { return Promise.resolve(JSON.parse(ok ? body : 'null')); }
    };
  }
  // 解析 host→JS wire 为 Response。`__zwfr:` 前缀 → status/statusText/headers/body；
  // `__zw_fetch_error:` 或非 wire → 落 _makeResponse（错误 / 旧路径兼容）。
  function _makeResponseFromWire(raw) {
    if (typeof raw !== 'string') return _makeResponse('__zw_fetch_error:bad-wire');
    if (raw.indexOf('__zw_fetch_error') === 0) return _makeResponse(raw);
    if (raw.indexOf('__zwfr:') !== 0) return _makeResponse(raw);
    var rest = raw.slice(7); // strip '__zwfr:'
    var p1 = rest.indexOf('\x1f');
    var p2 = p1 >= 0 ? rest.indexOf('\x1f', p1 + 1) : -1;
    var p3 = p2 >= 0 ? rest.indexOf('\x1f', p2 + 1) : -1;
    if (p1 < 0 || p2 < 0 || p3 < 0) return _makeResponse('__zw_fetch_error:malformed');
    var status = parseInt(rest.slice(0, p1), 10) || 0;
    var statusText = rest.slice(p1 + 1, p2);
    var headersWire = rest.slice(p2 + 1, p3);
    var body = rest.slice(p3 + 1); // 末字段，可含 \x1f
    var headers = _parseHeadersWire(headersWire);
    // R2968：经 new Response 构造（fetch 结果 instanceof Response）。字段 shape 与旧 plain object 一致
    //（headers 经 new Response 封装为 Headers 实例，R2977；body getter 同 R2967）。
    return new Response(body, { status: status, statusText: statusText, headers: headers });
  }
  // 收集 headers 源（Object / [[k,v]] / Headers-like forEach）→ `name\x1evalue\x1e...` wire（空 → ''）。
  function _headersToWire(src) {
    if (!src) return '';
    var pairs = [];
    if (typeof src.forEach === 'function') {
      src.forEach(function(v, k) { pairs.push([String(k), String(v)]); });
    } else if (Array.isArray(src)) {
      for (var i = 0; i < src.length; i++) {
        var e = src[i];
        if (Array.isArray(e)) pairs.push([String(e[0]), String(e[1])]);
      }
    } else {
      for (var k in src) {
        if (Object.prototype.hasOwnProperty.call(src, k)) pairs.push([String(k), String(src[k])]);
      }
    }
    var out = '';
    for (var j = 0; j < pairs.length; j++) out += (j > 0 ? '\x1e' : '') + pairs[j][0] + '\x1e' + pairs[j][1];
    return out;
  }

  // R2923 fetch 完整化：`fetch(input, init)` 透传 method/headers/body → host 返 status/headers/body。
  // input = URL 字符串或 Request-like（.url/.method/.headers/.body）；init = { method, headers, body }。
  // method 默认 GET；GET/HEAD 无 body。`__zw_fetch` 未注册（engine/reftest/polyfill 无 host fetch handler）
  // 时 resolve ok:false Response（stub，避免悬挂，零回归）。
  if (!globalThis.fetch) {
    globalThis.fetch = function(input, init) {
      init = init || {};
      var isObj = input && typeof input === 'object';
      var url = isObj ? String(input.url || '') : String(input);
      var method = String(init.method || (isObj ? input.method : '') || 'GET').toUpperCase();
      var headersWire = _headersToWire(init.headers) || (isObj ? _headersToWire(input.headers) : '');
      var body = '';
      if (init.body != null) body = String(init.body);
      else if (isObj && input.body != null) body = String(input.body);
      if (typeof __zw_fetch !== 'function') {
        return Promise.resolve(_makeResponse('__zw_fetch_error:no-handler'));
      }
      return new Promise(function(resolve) {
        globalThis.__zw_fetch_counter = (globalThis.__zw_fetch_counter | 0) + 1;
        var id = '__zwfid:' + globalThis.__zw_fetch_counter;
        globalThis.__zw_pending[id] = function(raw) { resolve(_makeResponseFromWire(raw)); };
        try {
          __zw_fetch(id, method, url, headersWire, body);
        } catch (_e) {
          resolve(_makeResponse('__zw_fetch_error:throw'));
        }
      });
    };
  }

  // R2968：Response / Request 全局构造器（补全 fetch API 表面——此前仅 fetch()/Headers，缺 new Response/
  // new Request）。`new Response(body, init)` / `new Request(url, init)` 用于 service worker 构造响应、fetch
  // 包装库、测试 mock。`_makeResponseFromWire` 经 new Response 路由 → fetch 结果 instanceof Response（同时保持
  // 字段 shape 与旧 plain object 逐字段一致：ok/status/statusText/headers/body/text()/json()）。
  // R2977：headers 为 Headers 实例（spec Response.headers）。modern 代码经 `response.headers.get('content-type')`
  // 消费（比 bracket `headers['x']` 更常见 + 标准）——Headers 实例提供 get/has/append/set/delete/forEach/entries。
  // `new Headers(init)` 接受 plain dict / Headers-like / [[k,v]] / undefined。clone 经 new Response(headers) 再封装。
  // urlencoded 表单体 → FormData（R2982 抽出，Response.formData / Request.formData 共用）。
  // `+`→space + % 解码，spec application/x-www-form-urlencoded 语义（multipart/form-data 解析 defer）。
  function _zwParseFormUrlencoded(bodyText) {
    var fd = new FormData();
    var body = String(bodyText == null ? '' : bodyText).trim();
    if (body) {
      body.split('&').forEach(function (pair) {
        if (!pair) return;
        var eq = pair.indexOf('=');
        var k = eq >= 0 ? pair.slice(0, eq) : pair;
        var v = eq >= 0 ? pair.slice(eq + 1) : '';
        fd.append(decodeURIComponent(k.replace(/\+/g, ' ')), decodeURIComponent(v.replace(/\+/g, ' ')));
      });
    }
    return fd;
  }
  globalThis.Response = function Response(body, init) {
    if (!(this instanceof Response)) return new Response(body, init);
    init = init || {};
    var status = init.status != null ? (init.status | 0) : 200;
    this.status = status;
    this.ok = status >= 200 && status < 300;
    this.statusText = init.statusText != null ? String(init.statusText) : '';
    this.headers = new Headers(init.headers); // Headers 实例（spec，R2977）
    this.type = 'default';
    this.url = '';
    this.redirected = false;
    this._bodyText = body == null ? '' : String(body);
    var self = this;
    // body 为 ReadableStream（lazy，单 UTF-8 chunk + close，复用 _bodyToStream）。与 _makeResponseFromWire
    // 旧 plain object 的 body getter 行为一致（R2967）。
    Object.defineProperty(this, 'body', {
      get: function () { if (!self._bs) self._bs = _bodyToStream(self._bodyText); return self._bs; },
      configurable: true
    });
    this.text = function () { return Promise.resolve(self._bodyText); };
    this.json = function () { return Promise.resolve(JSON.parse(self._bodyText)); };
    // R2978：补全 Response body-consumption 表面（spec：text/json/blob/arrayBuffer/formData）。
    // blob()：body 包成 Blob；arrayBuffer()：UTF-8 Uint8Array；formData()：application/x-www-form-urlencoded 解析。
    this.blob = function () { return Promise.resolve(new Blob([self._bodyText])); };
    this.arrayBuffer = function () {
      var bytes = _zw_utf8_encode(self._bodyText);
      var arr = new Uint8Array(bytes.length);
      for (var k = 0; k < bytes.length; k++) arr[k] = bytes[k];
      return Promise.resolve(arr);
    };
    this.formData = function () { return Promise.resolve(_zwParseFormUrlencoded(self._bodyText)); };
    this.clone = function () {
      return new Response(self._bodyText, { status: self.status, statusText: self.statusText, headers: self.headers });
    };
  };
  // R2968 Request：`new Request(url|request, init)`。fetch(input) 既接受 string 也接受 Request-like
  //（读 .url/.method/.headers/.body），故 Request 字段对齐 fetch 消费路径（body 为 string|null，非 stream；
  // R2977 headers 为 Headers 实例，同 Response）。clone() 复制自身。R2982 补 body 消费表面
  //（text/json/blob/arrayBuffer/formData，对称 Response R2978）。
  globalThis.Request = function Request(input, init) {
    if (!(this instanceof Request)) return new Request(input, init);
    init = init || {};
    var isObj = input && typeof input === 'object';
    this.url = isObj ? String(input.url || '') : String(input);
    this.method = String(init.method || (isObj ? input.method : '') || 'GET').toUpperCase();
    this.headers = new Headers(init.headers || (isObj ? input.headers : null));
    this.body = init.body != null ? String(init.body) : (isObj && input.body != null ? String(input.body) : null);
    this.cache = init.cache || 'default';
    this.mode = init.mode || 'cors';
    this.redirect = init.redirect || 'follow';
    this.credentials = init.credentials || 'same-origin';
    // R2982：body 消费表面（对称 Response R2978，spec text/json/blob/arrayBuffer/formData）。fetch 包装库 /
    // service worker fetch handler / 请求拦截器 / 测试 mock 读请求体高频。body 为 string|null：null（GET 无体）
    // → text() 返 ''、arrayBuffer() 长度 0；json() 解析空串抛 SyntaxError（spec，非合法 JSON）。
    var self = this;
    this.text = function () { return Promise.resolve(self.body == null ? '' : String(self.body)); };
    this.json = function () { return Promise.resolve(JSON.parse(self.body == null ? '' : String(self.body))); };
    this.blob = function () { return Promise.resolve(new Blob([self.body == null ? '' : String(self.body)])); };
    this.arrayBuffer = function () {
      var bytes = _zw_utf8_encode(self.body == null ? '' : String(self.body));
      var arr = new Uint8Array(bytes.length);
      for (var k = 0; k < bytes.length; k++) arr[k] = bytes[k];
      return Promise.resolve(arr);
    };
    this.formData = function () { return Promise.resolve(_zwParseFormUrlencoded(self.body)); };
  };
  globalThis.Request.prototype.clone = function () {
    return new Request(this.url, { method: this.method, headers: this.headers, body: this.body });
  };

  // P1b S5：setTimeout/setInterval 真实延迟。host（browser/renderer js_worker）注册
  // `__zw_setTimeout(id, delayMs)` 时，回调存 `__zw_pending[id]` + 调本回调；host 子线程
  // sleep 后 resolve → `__zwResolveCallback` 取出调用回调。未注册（engine/reftest/polyfill
  // 等无 host 路径）时 fallback `_defer`（microtask 同步触发）——保持旧行为，零回归。
  function _timerIdKey(handle) { return '__zwtid:' + handle; }
  globalThis.setTimeout = function(fn, delay) {
    var handle = _timerId++;
    if (typeof fn !== 'function') return handle;
    var id = _timerIdKey(handle);
    globalThis.__zw_pending[id] = function() { try { fn(); } catch (_e) {} };
    if (typeof __zw_setTimeout === 'function') {
      try { __zw_setTimeout(id, delay | 0); return handle; }
      catch (_e) { delete globalThis.__zw_pending[id]; }
    }
    // fallback：无 host → microtask 同步触发（旧行为）。
    delete globalThis.__zw_pending[id];
    _defer(fn);
    return handle;
  };
  globalThis.setInterval = function(fn, delay) {
    var handle = _timerId++;
    if (typeof fn !== 'function') return handle;
    var id = _timerIdKey(handle);
    var ms = delay | 0;
    if (typeof __zw_setTimeout === 'function') {
      // host 路径：回调内 re-arm 实现重复触发（host 仅实现单次定时器）。
      var arm = function() {
        globalThis.__zw_pending[id] = function() {
          try { fn(); } catch (_e) {}
          arm();
        };
        try { __zw_setTimeout(id, ms); }
        catch (_e) { delete globalThis.__zw_pending[id]; }
      };
      arm();
    } else {
      // fallback（无 host）：保持旧行为——单次 _defer 触发（非重复）。
      _defer(fn);
    }
    return handle;
  };
  // clearTimeout/clearInterval：删 pending 项——即便 host 子线程后到 resolve，
  // `__zwResolveCallback` 见无 pending 即 no-op（setInterval 的 re-arm 链亦在此断开）。
  globalThis.clearTimeout = function(handle) {
    delete globalThis.__zw_pending[_timerIdKey(handle)];
  };
  globalThis.clearInterval = function(handle) {
    delete globalThis.__zw_pending[_timerIdKey(handle)];
  };
  // requestIdleCallback/cancelIdleCallback：镜像 setTimeout 机制（host __zw_setTimeout + pending 表；
  // 无 host → _defer 微任务，同 setTimeout fallback）。回调传 IdleDeadline（didTimeout/timeRemaining
  // 近似——真实 idle 时序须 event-loop 帧 tick 切片，此为基础可用实现，防 ReferenceError + 延迟执行）。
  function _ricIdKey(handle) { return '__zwric:' + handle; }
  globalThis.requestIdleCallback = function(fn, options) {
    var handle = _timerId++;
    if (typeof fn !== 'function') return handle;
    var deadline = { didTimeout: false, timeRemaining: function() { return 50; } };
    var id = _ricIdKey(handle);
    globalThis.__zw_pending[id] = function() { try { fn(deadline); } catch (_e) {} };
    if (typeof __zw_setTimeout === 'function') {
      try { __zw_setTimeout(id, (options && options.timeout) | 0); return handle; }
      catch (_e) { delete globalThis.__zw_pending[id]; }
    }
    // fallback（无 host）：微任务同步触发（同 setTimeout fallback）。
    delete globalThis.__zw_pending[id];
    _defer(function() { try { fn(deadline); } catch (_e) {} });
    return handle;
  };
  globalThis.cancelIdleCallback = function(handle) {
    delete globalThis.__zw_pending[_ricIdKey(handle)];
  };

  // ── P1b S2 incr1/incr2：MutationObserver（JS 侧拦截 + microtask 派发）──
  // 节点身份用「复合 key」：handle-based（JS 创建子树，`createElement` 返 `"__n{n}"`）+
  // selector-based（现有 DOM，`querySelector` 返 `_makeProxy(sel, null)`）。`_mo_id(handle, sel)`
  // 优先 handle，否则 sel——v8::External 真 object identity（===）非功能必需（RFC 纠正）。
  // `observe(target, options)` 注册 id→options；`_makeProxy` 的 setAttribute/appendChild/etc.
  // 调 `_mo_notify(sel, handle, record)` 排队；`_defer`（microtask）派发回调（spec §4 语义）。
  // incr1 = handle（JS 子树）；incr2 = +selector（现有 DOM）。支持 attributes + childList。
  // 限制：仅观测 JS 驱动的 mutation（host 侧 `__zw_dispatch_event` 等不触发）。
  globalThis.__zw_mo_observers = globalThis.__zw_mo_observers || [];
  var _moFlushScheduled = false;

  // 元素身份 key——handle 优先（JS 创建节点），否则 selector（现有 DOM）。
  function _mo_id(handle, sel) {
    if (handle != null) return 'h:' + handle;
    if (sel) return 's:' + sel;
    return null;
  }

  // 把一条 mutation 记录投递给所有观测该 id 且 options 匹配的 observer。
  // 每个 observer 拿独立 record 副本（target 指向各自 observe() 时的 proxy）。
  function _mo_notify(sel, handle, baseRecord) {
    var id = _mo_id(handle, sel);
    if (id == null) return;
    var observers = globalThis.__zw_mo_observers;
    for (var i = 0; i < observers.length; i++) {
      var obs = observers[i];
      var opts = obs._targets[id];
      if (!opts) continue;
      if (baseRecord.type === 'attributes' && !opts.attributes) continue;
      if (baseRecord.type === 'childList' && !opts.childList) continue;
      var rec = Object.create(globalThis.MutationRecord.prototype);
      rec.type = baseRecord.type;
      rec.target = obs._targetProxies[id];
      // spec 字段：addedNodes/removedNodes 缺省 []（类数组），sibling/attributeNamespace/oldValue 缺省 null。
      rec.addedNodes = baseRecord.addedNodes || [];
      rec.removedNodes = baseRecord.removedNodes || [];
      rec.previousSibling = baseRecord.previousSibling || null;
      rec.nextSibling = baseRecord.nextSibling || null;
      rec.attributeName = baseRecord.attributeName || null;
      rec.attributeNamespace = baseRecord.attributeNamespace || null;
      rec.oldValue = baseRecord.oldValue || null;
      obs._records.push(rec);
      _mo_scheduleFlush();
    }
  }
  function _mo_scheduleFlush() {
    if (_moFlushScheduled) return;
    _moFlushScheduled = true;
    _defer(function() {
      _moFlushScheduled = false;
      var observers = globalThis.__zw_mo_observers;
      for (var i = 0; i < observers.length; i++) {
        var obs = observers[i];
        if (obs._records.length > 0) {
          var records = obs._records;
          obs._records = [];
          try { obs._callback(records, obs); } catch (_e) {}
        }
      }
    });
  }

  globalThis.MutationObserver = function(callback) {
    this._callback = callback;
    this._targets = {};       // id (h:handle / s:sel) -> options
    this._targetProxies = {}; // id -> observe() 时传入的 proxy（record.target 用）
    this._records = [];
    globalThis.__zw_mo_observers.push(this);
  };
  globalThis.MutationObserver.prototype.observe = function(target, options) {
    if (!target) return;
    var id = _mo_id(target.__zwHandle, target.__zwSelector);
    if (id == null) return;
    this._targets[id] = options || {};
    this._targetProxies[id] = target;
  };
  globalThis.MutationObserver.prototype.disconnect = function() {
    this._targets = {};
    this._targetProxies = {};
  };
  globalThis.MutationObserver.prototype.takeRecords = function() {
    var r = this._records;
    this._records = [];
    return r;
  };
  // MutationRecord（R2847）：Web IDL 接口——回调收到的 record 须 `instanceof MutationRecord` +
  // `[object MutationRecord]` toStringTag + 完整 spec 字段（previousSibling/nextSibling/
  // attributeNamespace/oldValue 缺省 null，addedNodes/removedNodes 缺省 []）。库做
  // `record instanceof MutationRecord` 特征检测 / 读 record.previousSibling 须得 null 非 undefined。
  // 无公开构造器入参（字段由 _mo_notify 注入）；仅建 prototype + toStringTag 供 instanceof/序列化。
  globalThis.MutationRecord = function() {};
  globalThis.MutationRecord.prototype[Symbol.toStringTag] = 'MutationRecord';

  // ── P1a Slice 2a：IntersectionObserver（JS 侧，复用 gBCR layout-rect snapshot）──
  // 镜像 MutationObserver：纯 JS，`observe()` 排队 initial notification，经 `_defer`
  // （microtask）派发 `obs._callback(entries, observer)`。intersection 用 host
  // `__zw_getBoundingClientRect(sel)`（gBCR path C，已注册时返真实 rect）+ innerWidth/innerHeight
  // 计算与 root（默认 viewport）的重叠；threshold 越界检测决定是否派发。host 未注册
  // （reftest/polyfill/WebView 路径）→ target rect 为零 → isIntersecting=false，仍派发 initial
  // notification（no-throw，零回归）。旧 shim 完全无 IO → `new IntersectionObserver` 抛
  // ReferenceError **中断整个脚本**，本切片消除之（spec：observe 即排队一次 initial 通知）。
  // 限制（接受，follow-up）：① 仅 observe 时计算，非持续 host tick——scroll/resize/async-layout
  //   变化触发的后续通知为 Slice 2b（须 host render-loop tick 或 __zwResolveCallback 重算钩子）；
  // ② handle-identity（createElement）元素 sel 为空 → 零 rect（同 gBCR 限制，path A follow-up）；
  // ③ rootMargin px/% 已支持（R2966，CSS margin 简写展开/收缩 root rect，% 按 root 维度）；④ root 为元素时取其 selector rect。
  function _io_domRect(x, y, w, h) {
    return { x: x, y: y, top: y, left: x, right: x + w, bottom: y + h, width: w, height: h, toJSON: function() { return this; } };
  }
  // 读 target/root 的 rect（复用 gBCR）；identity = selector 或 handle（path A）。
  // 空 / handler 未注册 / 未命中 → 零 rect。
  function _io_rectFromSel(identity) {
    if (identity && typeof __zw_getBoundingClientRect === 'function') {
      try {
        var s = __zw_getBoundingClientRect(identity);
        if (s && s.indexOf(',') >= 0) {
          var p = s.split(',');
          return { x: +p[0], y: +p[1], w: +p[2], h: +p[3] };
        }
      } catch (_e) {}
    }
    return { x: 0, y: 0, w: 0, h: 0 };
  }
  function _io_intersect(a, b) {
    var x0 = Math.max(a.x, b.x), y0 = Math.max(a.y, b.y);
    var x1 = Math.min(a.x + a.w, b.x + b.w), y1 = Math.min(a.y + a.h, b.y + b.h);
    if (x1 <= x0 || y1 <= y0) return { x: 0, y: 0, w: 0, h: 0 };
    return { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
  }
  // 归一化 threshold：number | number[] → 升序去重、clamp 到 [0,1] 的数组（空→[0]）。
  function _io_normThresholds(threshold) {
    var arr = [];
    if (typeof threshold === 'number') {
      arr = [threshold];
    } else if (Object.prototype.toString.call(threshold) === '[object Array]') {
      for (var i = 0; i < threshold.length; i++) {
        if (typeof threshold[i] === 'number') arr.push(threshold[i]);
      }
    }
    if (arr.length === 0) arr = [0];
    arr.sort(function(a, b) { return a - b; });
    var uniq = [];
    for (var j = 0; j < arr.length; j++) {
      var v = arr[j];
      if (v < 0) v = 0; else if (v > 1) v = 1;
      if (uniq.length === 0 || uniq[uniq.length - 1] !== v) uniq.push(v);
    }
    return uniq;
  }
  function _io_id(handle, sel) {
    if (handle != null) return 'h:' + handle;
    if (sel) return 's:' + sel;
    return null;
  }
  // 解析 rootMargin 串（CSS margin shorthand）→ 4 个 {val, pct} 部分（top/right/bottom/left）。
  // R2966：rootMargin 此前按 0 处理（defer）。px 直取并标记 pct=false；% 标记 pct=true（compute 时按
  // root 维度展开：top/bottom→root 高，left/right→root 宽，spec §2.1）。其它单位/非法值 → 0（spec：
  // rootMargin 仅支持 <length>/<percentage>，fail-to-parse 视为 0）。1-4 值按 CSS margin 简写展开。
  function _io_parseRootMargin(str) {
    var raw = (typeof str === 'string' ? str : '').trim().split(/\s+/).filter(function (s) { return s.length > 0; });
    if (raw.length === 0) raw = ['0px', '0px', '0px', '0px'];
    else if (raw.length === 1) raw = [raw[0], raw[0], raw[0], raw[0]];
    else if (raw.length === 2) raw = [raw[0], raw[1], raw[0], raw[1]];
    else if (raw.length === 3) raw = [raw[0], raw[1], raw[2], raw[1]];
    var norm = function (s) {
      var m = /^(-?\d+(?:\.\d+)?)(px|%)?$/.exec(String(s).trim());
      if (!m) return { val: 0, pct: false };
      return { val: parseFloat(m[1]) || 0, pct: m[2] === '%' };
    };
    return [norm(raw[0]), norm(raw[1]), norm(raw[2]), norm(raw[3])];
  }
  // 按 rootMargin 4 部分展开/收缩 root rect（负 margin 收缩）。% 按 root 自身维度展开（compute 时 rootRect
  // 已知）。返回新 rect（不改原）。零 margin（默认）→ 原样返回（零回归既有 IO 行为）。
  function _io_applyRootMargin(rootRect, margins) {
    var resolve = function (part, dim) { return part.pct ? (part.val / 100) * dim : part.val; };
    var top = resolve(margins[0], rootRect.h);
    var right = resolve(margins[1], rootRect.w);
    var bottom = resolve(margins[2], rootRect.h);
    var left = resolve(margins[3], rootRect.w);
    return { x: rootRect.x - left, y: rootRect.y - top, w: rootRect.w + left + right, h: rootRect.h + top + bottom };
  }
  globalThis.IntersectionObserver = function(callback, options) {
    this._callback = callback;
    var opts = options || {};
    this._thresholds = _io_normThresholds(opts.threshold);
    // root：null（默认 viewport）或元素（取其 __zwSelector 的 rect）。
    this._rootSel = (opts.root && opts.root.__zwSelector) ? opts.root.__zwSelector : null;
    // R2966：rootMargin（CSS margin shorthand，px/%），compute 时展开/收缩 root rect。
    this._rootMargins = _io_parseRootMargin(opts.rootMargin);
    this._targets = {};        // id (h:handle / s:sel) -> { proxy }
    this._lastRatio = {};      // id -> 上次派发的 ratio（undefined = 未派发过 → initial）
    this._scheduled = false;
    _zwObservers.push(this);   // P1a Slice 2b：注册到 tick 表
  };
  // 计算单个 target 的 intersection 数据（rect / ratio / isIntersecting）。
  globalThis.IntersectionObserver.prototype._compute = function(id) {
    var t = this._targets[id];
    if (!t) return null;
    var sel = t.proxy.__zwSelector;
    var rootRect = this._rootSel
      ? _io_rectFromSel(this._rootSel)
      : { x: 0, y: 0, w: globalThis.innerWidth | 0, h: globalThis.innerHeight | 0 };
    // R2966：rootMargin 展开/收缩 root rect（% 按 root 自身维度）。零 margin（默认）原样。
    rootRect = _io_applyRootMargin(rootRect, this._rootMargins);
    // path A：sel 空（createElement 元素）时用 handle，host 查 handle→selector map 解析。
    var targetRect = _io_rectFromSel(sel || t.proxy.__zwHandle);
    var inter = _io_intersect(targetRect, rootRect);
    var targetArea = targetRect.w * targetRect.h;
    var ratio = targetArea > 0 ? (inter.w * inter.h) / targetArea : 0;
    return { target: t.proxy, targetRect: targetRect, rootRect: rootRect, inter: inter, ratio: ratio, isIntersecting: inter.w > 0 && inter.h > 0 };
  };
  // threshold 越界检测：未派发过（initial）或 ratio 与上次跨过任一 threshold 边界。
  globalThis.IntersectionObserver.prototype._crossed = function(id, ratio) {
    var prev = this._lastRatio[id];
    if (prev == null) return true;
    for (var i = 0; i < this._thresholds.length; i++) {
      var th = this._thresholds[i];
      if ((prev >= th) !== (ratio >= th)) return true;
    }
    return false;
  };
  // 排队一次 microtask 派发：遍历所有 target，对越阈值的构造 entry 投递 callback。
  globalThis.IntersectionObserver.prototype._schedule = function() {
    if (this._scheduled) return;
    this._scheduled = true;
    var self = this;
    _defer(function() {
      self._scheduled = false;
      var entries = [];
      for (var id in self._targets) {
        var c = self._compute(id);
        if (!c) continue;
        if (self._crossed(id, c.ratio)) {
          entries.push({
            time: 0,
            target: c.target,
            rootBounds: _io_domRect(c.rootRect.x, c.rootRect.y, c.rootRect.w, c.rootRect.h),
            boundingClientRect: _io_domRect(c.targetRect.x, c.targetRect.y, c.targetRect.w, c.targetRect.h),
            intersectionRect: _io_domRect(c.inter.x, c.inter.y, c.inter.w, c.inter.h),
            intersectionRatio: c.ratio,
            isIntersecting: c.isIntersecting,
            toJSON: function() { return this; }
          });
          self._lastRatio[id] = c.ratio;
        }
      }
      if (entries.length > 0) {
        try { self._callback(entries, self); } catch (_e) {}
      }
    });
  };
  globalThis.IntersectionObserver.prototype.observe = function(target) {
    if (!target) return this;
    var id = _io_id(target.__zwHandle, target.__zwSelector);
    if (id != null) {
      this._targets[id] = { proxy: target };
      this._schedule();
    }
    return this;
  };
  globalThis.IntersectionObserver.prototype.unobserve = function(target) {
    if (!target) return this;
    var id = _io_id(target.__zwHandle, target.__zwSelector);
    if (id != null) {
      delete this._targets[id];
      delete this._lastRatio[id];
    }
    return this;
  };
  globalThis.IntersectionObserver.prototype.disconnect = function() {
    this._targets = {};
    this._lastRatio = {};
    return this;
  };
  globalThis.IntersectionObserver.prototype.takeRecords = function() {
    return [];
  };
  // IntersectionObserverEntry：兼容构造（部分脚本 `new IntersectionObserverEntry(init)`）。
  globalThis.IntersectionObserverEntry = function(init) {
    init = init || {};
    this.time = init.time || 0;
    this.rootBounds = init.rootBounds || null;
    this.boundingClientRect = init.boundingClientRect || null;
    this.intersectionRect = init.intersectionRect || null;
    this.isIntersecting = init.isIntersecting || false;
    this.target = init.target || null;
    this.intersectionRatio = init.intersectionRatio || 0;
  };

  // ── P1a Slice 3：ResizeObserver（JS 侧，复用 gBCR layout-rect snapshot）──
  // 镜像 IntersectionObserver：纯 JS，`observe()` 排队 initial notification，经 `_defer`
  // （microtask）派发 `obs._callback(entries, observer)`。size 取 host `__zw_getBoundingClientRect(sel)`
  // （gBCR path C，直接复用 IO 的 `_io_rectFromSel`/`_io_domRect`/`_io_id` rect 辅助）；
  // size-diff 检测决定是否派发——首次（无 last）=initial 必派发，之后仅宽高变化才派发（spec §4 语义）。
  // host 未注册（reftest/polyfill/WebView 路径）→ contentRect 为零，仍派发 initial notification
  // （no-throw，零回归）。旧 shim 完全无 RO → `new ResizeObserver` 抛 ReferenceError 中断整个脚本
  // （与 IO 同），本切片消除之。
  // 限制（接受，follow-up）：① 仅 observe 时计算，非持续 host tick——resize/async-layout 变化触发的
  //   后续通知为 Slice 2b（与 IO 同，须 host render-loop tick 或 __zwResolveCallback 重算钩子）；
  // ② R2972：contentRect/contentBoxSize/devicePixelContentBoxSize 经 getComputedStyle 真值扣除 padding +
  //   border-width → content-box（borderBoxSize 仍 border-box = gBCR）。host 未注册/属性未覆盖 → 0 扣除
  //   → content = border（fallback，同旧近似行为）。
  // R2972：读计算样式 box-model 像素值（"10px" → 10，未注册/非 px → 0）供 RO content-box 扣除。
  function _ro_px(cs, prop) {
    if (!cs || typeof cs.getPropertyValue !== 'function') return 0;
    var m = /^(-?\d+(?:\.\d+)?)px$/.exec(String(cs.getPropertyValue(prop) || '').trim());
    return m ? parseFloat(m[1]) : 0;
  }
  globalThis.ResizeObserver = function(callback) {
    this._callback = callback;
    this._targets = {};       // id (h:handle / s:sel) -> { proxy }
    this._lastSize = {};      // id -> {w,h}（undefined = 未派发过 → initial）
    this._scheduled = false;
    _zwObservers.push(this);  // P1a Slice 2b：注册到 tick 表
  };
  // 排队一次 microtask 派发：遍历所有 target，对尺寸变化（或 initial）的构造 entry 投递 callback。
  globalThis.ResizeObserver.prototype._schedule = function() {
    if (this._scheduled) return;
    this._scheduled = true;
    var self = this;
    _defer(function() {
      self._scheduled = false;
      var entries = [];
      for (var id in self._targets) {
        var t = self._targets[id];
        // path A：sel 空（createElement 元素）时用 handle。
        var r = _io_rectFromSel(t.proxy.__zwSelector || t.proxy.__zwHandle);
        var prev = self._lastSize[id];
        // initial（prev==null）或宽高变化 → 派发并更新 last。
        if (prev == null || prev.w !== r.w || prev.h !== r.h) {
          self._lastSize[id] = { w: r.w, h: r.h };
          // R2972：box-model 真值扣除。gBCR rect = border-box（含 padding+border）；content-box =
          // border-box - padding - border-width（经 getComputedStyle 真值，host 未覆盖 → 0 = 不扣除）。
          var cs = globalThis.getComputedStyle ? globalThis.getComputedStyle(t.proxy) : null;
          var pT = _ro_px(cs, 'padding-top'), pR = _ro_px(cs, 'padding-right'),
              pB = _ro_px(cs, 'padding-bottom'), pL = _ro_px(cs, 'padding-left');
          var bT = _ro_px(cs, 'border-top-width'), bR = _ro_px(cs, 'border-right-width'),
              bB = _ro_px(cs, 'border-bottom-width'), bL = _ro_px(cs, 'border-left-width');
          var cW = Math.max(0, r.w - pL - pR - bL - bR);
          var cH = Math.max(0, r.h - pT - pB - bT - bB);
          entries.push({
            target: t.proxy,
            // contentRect = content-box rect（spec；origin = border-box origin + border + padding）。
            contentRect: _io_domRect(r.x + bL + pL, r.y + bT + pT, cW, cH),
            // borderBoxSize = border-box（gBCR）；contentBoxSize/devicePixelContentBoxSize = content-box。
            borderBoxSize: [{ inlineSize: r.w, blockSize: r.h }],
            contentBoxSize: [{ inlineSize: cW, blockSize: cH }],
            devicePixelContentBoxSize: [{ inlineSize: cW, blockSize: cH }],
            toJSON: function() { return this; }
          });
        }
      }
      if (entries.length > 0) {
        try { self._callback(entries, self); } catch (_e) {}
      }
    });
  };
  globalThis.ResizeObserver.prototype.observe = function(target) {
    if (!target) return this;
    var id = _io_id(target.__zwHandle, target.__zwSelector);
    if (id != null) {
      // 已观察的 target 重复 observe：spec 视为 no-op（不重置 last），但 _schedule 的 size-diff
      // 检测会在 layout 变化时自然派发（last 保留上次派发尺寸）。
      this._targets[id] = { proxy: target };
      this._schedule();
    }
    return this;
  };
  globalThis.ResizeObserver.prototype.unobserve = function(target) {
    if (!target) return this;
    var id = _io_id(target.__zwHandle, target.__zwSelector);
    if (id != null) {
      delete this._targets[id];
      delete this._lastSize[id];
    }
    return this;
  };
  globalThis.ResizeObserver.prototype.disconnect = function() {
    this._targets = {};
    this._lastSize = {};
    return this;
  };
  globalThis.ResizeObserver.prototype.takeRecords = function() {
    return [];
  };
  // ResizeObserverEntry：兼容构造（部分脚本 `new ResizeObserverEntry(init)`）。
  globalThis.ResizeObserverEntry = function(init) {
    init = init || {};
    this.target = init.target || null;
    this.contentRect = init.contentRect || null;
    this.borderBoxSize = init.borderBoxSize || null;
    this.contentBoxSize = init.contentBoxSize || null;
    this.devicePixelContentBoxSize = init.devicePixelContentBoxSize || null;
  };

  // P1a Slice 2b：host render（snapshot 已填真实 rect）后调本函数，对每个活跃 observer 调
  // `_schedule()` 复算——IO `_crossed`（threshold 越界）/ RO size-diff 仅在变化时派发，故收敛。
  // 跳过无活跃 target 的 observer（disconnect/unobserve-all 后 no-op）。`_defer` microtask 在
  // 本次 execute 末尾 checkpoint drain，回调同步触发；回调内 DOM mutation 由 host apply+rerender。
  globalThis.__zw_observers_tick = function() {
    for (var i = 0; i < _zwObservers.length; i++) {
      var obs = _zwObservers[i];
      if (!obs || !obs._targets) continue;
      var has = false;
      for (var _k in obs._targets) { has = true; break; }
      if (has) {
        try { obs._schedule(); } catch (_e) {}
      }
    }
  };

  // P1a form input：host 在 keydown 可打印字符时调本函数——对焦点 input/textarea 把字符 append
  // 到 value（经 `.value` set 更新缓存 + 记 value 属性 mutation）并派发 'input' 事件。
  // 非 input/textarea 目标 → no-op。限制（follow-up）：仅 append（无 backspace/delete/caret/selection）。
  globalThis.__zw_text_input = function(sel, ch) {
    var el = _resolveInputEl(sel);
    if (!el) return;
    el.value = (el.value || '') + ch;
    el.dispatchEvent(_makeEvent('input', { bubbles: true, cancelable: true }));
  };
  // P1a form input：Backspace → 删末字符 + 派发 'input'（仅当 value 非空）。无 caret/selection
  // （删末字符近似——真实浏览器按 caret 删，follow-up）。
  globalThis.__zw_text_delete = function(sel) {
    var el = _resolveInputEl(sel);
    if (!el) return;
    var cur = el.value || '';
    if (cur.length === 0) return; // 空值 backspace 无变化，不派发（同 real browser）。
    el.value = cur.slice(0, -1);
    el.dispatchEvent(_makeEvent('input', { bubbles: true, cancelable: true }));
  };
  // 解析 selector → canonical stable selector（`__zw_query_match`，与 querySelector 同 identity）+
  // 真实 tag（`__zw_get_tag`，非 `_tagFromSel` 启发式）判 INPUT/TEXTAREA → 返元素 proxy（否则 null）。
  // __zw_text_input / __zw_text_delete 共用。
  function _resolveInputEl(sel) {
    var resolved = typeof __zw_query_match === 'function' ? __zw_query_match(sel) : sel;
    if (!resolved) return null;
    var tag = (typeof __zw_get_tag === 'function' ? __zw_get_tag(resolved) : '').toUpperCase();
    if (tag !== 'INPUT' && tag !== 'TEXTAREA') return null;
    return _wrapSelector(resolved);
  }
  // P1a form input：导航（URL 变化）时清 value 缓存——防跨页同选择器 stale value。
  globalThis.__zw_reset_form_state = function() { _inputValues = {}; _classCache = {}; _customValidity = {}; _indeterminate = {}; _textSelection = {}; _outputDefault = {}; _outputValue = {}; _shadowRoots = {}; _shadowHandles = {}; _shadowHandleMeta = {}; _handleChildren = {}; };

  // 现代动态 reftest 常用模式：`requestAnimationFrame(() => requestAnimationFrame(() => { …setup…; takeScreenshot(); }))`
  // 把 DOM setup 延迟到「布局/绘制后」。harness 在脚本+load 派发后才截图，故 rAF
  // 同步立即执行回调即可让 setup mutation 被记录并应用到二次渲染（镜像 setTimeout 的 microtask 语义，
  // 但同步以保证回调在 sandbox 生命周期内必然执行）。
  globalThis.requestAnimationFrame = function(fn) {
    var id = _timerId++;
    if (globalThis.__ZW_RAF_FRAME_DRIVEN) {
      // 帧驱动（R2713a）：延后到 host render 后的 __zw_raf_tick 派发（spec rAF 语义）。
      if (typeof fn === 'function') _rafPending[id] = fn;
    } else if (typeof fn === 'function' && _rafBudget > 0) {
      // 同步 stub（reftest 兼容，默认路径）：预算内立即 fn(0)，让 double-rAF setup mutation
      // 进入最终 HTML 被 harness 单渲染捕获。
      _rafBudget--;
      try { fn(0); } catch (_e) {}
    }
    return id;
  };
  globalThis.cancelAnimationFrame = function(id) {
    if (globalThis.__ZW_RAF_FRAME_DRIVEN) delete _rafPending[id];
    // OFF 路径 no-op（旧行为）。
  };
  // host 在 render 后调用（renderer tick_observers；OFF 时早返零开销）。ts = DOMHighResTimeStamp（ms）。
  globalThis.__zw_raf_tick = function(ts) {
    if (!globalThis.__ZW_RAF_FRAME_DRIVEN) return;
    var cbs = _rafPending; _rafPending = {}; // 本帧快照、清空（rAF 内重注册入下一帧队列）
    for (var id in cbs) { try { cbs[id](ts); } catch (_e) {} }
  };
  globalThis.webkitRequestAnimationFrame = globalThis.requestAnimationFrame;
  globalThis.mozRequestAnimationFrame = globalThis.requestAnimationFrame;

  // `/common/reftest-wait.js` 提供的完成信号；harness 在 load 后统一截图，故 no-op。
  // 失败保守：返回 resolved Promise（部分测试链式调用 `.then(...)`）。
  globalThis.takeScreenshot = function(_cb) {
    if (typeof _cb === 'function') { try { _cb(); } catch (_e) {} }
    return Promise.resolve();
  };

  // `window.getComputedStyle(elt[, pseudo])`：动态 reftest 极常用作「强制 reflow」
  // 触发器——`getComputedStyle(el).getPropertyValue('grid-template-columns')` 结果
  // 丢弃，仅逼布局发生（css-grid/grid-with-content-dynamic-display-001 line 43 即此
  // 模式，紧接 line 47 的 `display:block` 视觉 mutation 才是测试目的）。
  // 本全局缺失 → 调用抛 ReferenceError **中断整个脚本**，使其后的 DOM mutation 全丢
  // `window.getComputedStyle(elt[, pseudo])`：返 CSSStyleDeclaration。高频作 visibility/hidden
  // 检查（`cs.display === 'none'`）与 reflow 触发器。经 host `__zw_get_computed_style(sel, prop)`
  // 返**计算值**（display/position/visibility/opacity 首批；UA 默认 builtin，`<style>` 级联）。
  // 属性访问（camelCase `.display`/`.backgroundColor`）与 `getPropertyValue(kebab)` 均经
  // `_camelToKebab` 归一后查询。host 未注册（polyfill/WebView）或未覆盖属性 → ''（fallback，
  // 不抛，同旧 stub 行为）；handle-only（无 sel）→ ''。
  globalThis.getComputedStyle = function(elt, _pseudo) {
    var sel = elt && elt.__zwSelector;
    var hasHost = sel && typeof __zw_get_computed_style === 'function';
    var query = function(prop) {
      if (!hasHost) return '';
      try { return __zw_get_computed_style(sel, prop); } catch (_e) { return ''; }
    };
    return new Proxy({}, {
      get: function(_t, prop) {
        var p = String(prop);
        if (p === 'getPropertyValue') {
          return function(name) { return query(_camelToKebab(String(name))); };
        }
        if (p === 'getPropertyPriority' || p === 'item') return function() { return ''; };
        if (p === 'length') return 0;
        if (p === 'parentRule') return null;
        if (p === 'cssText') return '';
        if (typeof prop !== 'string') return undefined; // Symbol 属性返 undefined
        return query(_camelToKebab(p));
      }
    });
  };

  function _emptyCollection() {
    return { length: 0, item: function() { return null; }, namedItem: function() { return null; } };
  }

  function _parseLocation(href) {
    var h = String(href == null ? '' : href);
    // 优先 new URL（R2778，spec-correct：percent-encoding / IDNA / 默认端口归一 / 端口解析），仅在
    // __zw_parse_url 已注册时；否则回退朴素 regex（reftest/裸 sandbox 无回调路径，零回归）。
    if (typeof URL === 'function' && typeof __zw_parse_url === 'function') {
      try {
        var u = new URL(h);
        return {
          href: u.href, protocol: u.protocol, host: u.host, hostname: u.hostname,
          pathname: u.pathname, search: u.search, hash: u.hash, origin: u.origin,
        };
      } catch (_) { /* 解析失败 → 回退 regex */ }
    }
    var m = h.match(/^([^:]+):\/\/([^\/]*)(\/[^?#]*)?(\?[^#]*)?(#.*)?$/);
    if (!m) {
      return { href: h || 'about:blank', protocol: '', host: '', hostname: '', pathname: '/', search: '', hash: '', origin: 'null' };
    }
    var host = m[2] || '';
    return {
      href: h,
      protocol: m[1] + ':',
      host: host,
      hostname: host.split(':')[0] || '',
      pathname: m[3] || '/',
      search: m[4] || '',
      hash: m[5] || '',
      origin: host ? m[1] + '://' + host : 'null',
    };
  }

  function _makeLocation() {
    function href() {
      return typeof __zw_get_page_url === 'function' ? __zw_get_page_url() : 'about:blank';
    }
    return {
      get href() { return _parseLocation(href()).href; },
      get protocol() { return _parseLocation(href()).protocol; },
      get host() { return _parseLocation(href()).host; },
      get hostname() { return _parseLocation(href()).hostname; },
      get pathname() { return _parseLocation(href()).pathname; },
      get search() { return _parseLocation(href()).search; },
      get hash() { return _parseLocation(href()).hash; },
      get origin() { return _parseLocation(href()).origin; },
      assign: function() {},
      replace: function() {},
      reload: function() {},
      toString: function() { return _parseLocation(href()).href; }
    };
  }

  globalThis.location = _makeLocation();
  globalThis.self = globalThis;
  globalThis.top = globalThis;
  globalThis.parent = globalThis;

  globalThis.screen = {
    width: 1280,
    height: 800,
    availWidth: 1280,
    availHeight: 760,
    colorDepth: 24,
    pixelDepth: 24,
    left: 0,
    top: 0,
    orientation: { type: 'landscape-primary', angle: 0 }
  };
  globalThis.innerWidth = 1280;
  globalThis.innerHeight = 800;
  globalThis.outerWidth = 1280;
  globalThis.outerHeight = 800;
  globalThis.devicePixelRatio = 1;
  // R2987 window context / security 全局——库 feature-detect 后再使用 secure-only API（crypto.subtle /
  // SharedArrayBuffer / Service Worker）或错误上报。
  // `isSecureContext`（getter，随 location.protocol）：secure 除非 http:/ws:（about:blank/https/wss/file → secure）。
  // spec secure context 判定含 localhost / 非安全白名单，headless 取协议近似（http/ws 不安全，余皆安全）。
  Object.defineProperty(globalThis, 'isSecureContext', {
    configurable: true,
    get: function () {
      try {
        var p = globalThis.location && globalThis.location.protocol;
        return p !== 'http:' && p !== 'ws:';
      } catch (_e) { return true; }
    }
  });
  // `crossOriginIsolated`：需 COOP+COEP 响应头隔离。headless 无 → false（SharedArrayBuffer / 跨 origin
  // 资源不受隔离，feature-detect 库正确回落）。
  Object.defineProperty(globalThis, 'crossOriginIsolated', { configurable: true, value: false });
  // `reportError(reason)`：向 window 派发 ErrorEvent（error 上报库 / Promise catch 转错误事件 / 兜底未捕获错误
  // 报告高频）。经 globalThis.dispatchEvent（R2932）触 window 'error' listener + onerror IDL handler（R2932 注册）。
  // spec reportError 把 reason 转 ErrorEvent 派发到 window error handler；headless 复用 dispatchEvent 路径。
  globalThis.reportError = function (reason) {
    try {
      var msg = (reason && (reason.message || reason.name)) ? String(reason.message || reason.name) : String(reason);
      var ev = new ErrorEvent('error', {
        message: msg,
        filename: '',
        lineno: 0,
        colno: 0,
        error: (reason instanceof Error) ? reason : null
      });
      if (typeof globalThis.dispatchEvent === 'function') globalThis.dispatchEvent(ev);
    } catch (_e) {}
  };
  // scroll（R2817）——window 滚动方法/属性。headless 无真滚动 → no-op 方法 + 恒 0 偏移（scrollX/scrollY/
  // pageXOffset/pageYOffset）。feature-detect + scroll-to-section 脚本不抛。
  globalThis.scrollX = 0;
  globalThis.scrollY = 0;
  globalThis.pageXOffset = 0;
  globalThis.pageYOffset = 0;
  globalThis.scrollTo = function() {};
  globalThis.scroll = globalThis.scrollTo;
  globalThis.scrollBy = function() {};
  globalThis.scrollIntoView = function() {};

  // window 弹窗 / 对话框 API（R2979）——alert/confirm/prompt/open 此前全缺，`if (confirm('Delete?'))` /
  // `alert(err)` / `prompt('Name')` / `window.open(url)` 抛 ReferenceError 中断后续脚本。headless 无 UI 用户
  // 交互 → spec 合规的 dismiss 语义：alert 返 undefined（不阻塞，real 浏览器阻塞 headless 无）；confirm 返 false
  //（无用户点 OK = dismiss）；prompt 返 null（无用户输入 = dismiss，spec）；open 返 null（headless 弹窗被阻 =
  // popup-blocked 语义，`if (win)` 守卫自然跳过）。modern 站点的离开页守卫 / 表单确认 / OAuth 弹窗高频。
  globalThis.alert = globalThis.alert || function alert(_message) {};
  globalThis.confirm = globalThis.confirm || function confirm(_message) { return false; };
  globalThis.prompt = globalThis.prompt || function prompt(_message, _defaultValue) { return null; };
  globalThis.open = globalThis.open || function open(_url, _target, _features) { return null; };

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
